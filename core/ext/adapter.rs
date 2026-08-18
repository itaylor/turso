//! Adapters that make C-ABI (`turso_ext`) functions look like native
//! [`crate::udf`] functions, so the VDBE only has one execution path.

use turso_ext::{
    AggCtx, ContextDestructor, FinalizeFunction, InitAggFunction, ScalarFunction as ExtScalarFn,
    StepFunction, ValueDestructor,
};

use crate::types::{Value, ValueRef};
use crate::udf::{AggregateFunction, AggregateState, FunctionContext, ScalarFunction};
use crate::{LimboError, Result};

use super::ExtValue;

/// Below this ABI version an extension never writes the subtype byte, so its
/// contents are garbage and must not be read.
const FIRST_ABI_VERSION_WITH_SUBTYPES: u32 = 2;

/// Copies the arguments into owned FFI values. The caller must free them with
/// [`free_ext_values`] once the callback has returned.
fn to_ext_values(ctx: &FunctionContext<'_>, args: &[ValueRef<'_>]) -> Vec<ExtValue> {
    args.iter()
        .enumerate()
        .map(|(i, arg)| {
            // `to_ffi` already carries the subtype TEXT keeps inline.
            let value = arg.to_ffi();
            match ctx.arg_subtype(i) {
                0 => value,
                subtype => value.with_subtype(subtype),
            }
        })
        .collect()
}

fn free_ext_values(values: Vec<ExtValue>) {
    for value in values {
        unsafe { value.__free_internal_type() };
    }
}

/// Converts the FFI value a callback returned into a `Value` and frees it.
/// `keeps_result` says whether the engine keeps the result, and so whether
/// its subtype is worth propagating.
fn take_ext_result(
    ctx: &mut FunctionContext<'_>,
    abi_version: u32,
    keeps_result: bool,
    mut result: ExtValue,
    value_destructor: Option<ValueDestructor>,
) -> Result<Value> {
    let value = Value::from_ffi_ref(&result);
    if keeps_result && abi_version >= FIRST_ABI_VERSION_WITH_SUBTYPES {
        let subtype = result.subtype();
        if subtype != 0 {
            ctx.set_result_subtype(subtype);
        }
    }
    if let Some(value_destructor) = value_destructor {
        unsafe { value_destructor(&mut result) };
    } else {
        unsafe { result.__free_internal_type() };
    }
    value
}

/// A scalar function registered over the C ABI.
pub struct ExtScalarAdapter {
    context: usize,
    callback: ExtScalarFn,
    context_destructor: Option<ContextDestructor>,
    value_destructor: Option<ValueDestructor>,
    /// ABI the registering extension was built against; decides whether the
    /// subtype byte of the values it returns can be trusted.
    abi_version: u32,
}

impl ExtScalarAdapter {
    /// Assumes the oldest ABI, the safe default; callers that know better use
    /// [`ExtScalarAdapter::with_abi_version`].
    pub fn new(
        context: usize,
        callback: ExtScalarFn,
        context_destructor: Option<ContextDestructor>,
        value_destructor: Option<ValueDestructor>,
    ) -> Self {
        Self {
            context,
            callback,
            context_destructor,
            value_destructor,
            abi_version: 1,
        }
    }

    #[must_use]
    pub fn with_abi_version(mut self, abi_version: u32) -> Self {
        self.abi_version = abi_version;
        self
    }
}

impl ScalarFunction for ExtScalarAdapter {
    fn call(&self, ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]) -> Result<Value> {
        let ext_values = to_ext_values(ctx, args);
        let argv = if ext_values.is_empty() {
            std::ptr::null()
        } else {
            ext_values.as_ptr()
        };
        let result = unsafe {
            (self.callback)(
                self.context,
                args.len() as i32,
                argv,
                self.context_destructor,
                self.value_destructor,
            )
        };
        let value = take_ext_result(ctx, self.abi_version, true, result, self.value_destructor);
        free_ext_values(ext_values);
        value
    }
}

impl Drop for ExtScalarAdapter {
    fn drop(&mut self) {
        if let Some(context_destructor) = self.context_destructor {
            unsafe { context_destructor(self.context) };
        }
    }
}

/// Mirrors SQLite's `xValue`: reads the running value without destroying the
/// accumulator.
pub type WindowValueFunction = unsafe extern "C" fn(context: usize, agg: *mut AggCtx) -> ExtValue;

/// Mirrors SQLite's `xInverse`: undoes the step for a row that left the frame.
pub type WindowInverseFunction = StepFunction;

/// An aggregate function registered over the C ABI.
pub struct ExtAggregateAdapter {
    context: usize,
    init: InitAggFunction,
    step: StepFunction,
    finalize: FinalizeFunction,
    /// Both present or both absent; `Some` means it can drive a window frame.
    window: Option<(WindowValueFunction, WindowInverseFunction)>,
    context_destructor: Option<ContextDestructor>,
    aggregate_destructor: Option<ContextDestructor>,
    value_destructor: Option<ValueDestructor>,
    /// ABI the registering extension was built against.
    abi_version: u32,
}

impl ExtAggregateAdapter {
    /// Assumes the oldest ABI; see [`ExtScalarAdapter::new`].
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        context: usize,
        init: InitAggFunction,
        step: StepFunction,
        finalize: FinalizeFunction,
        context_destructor: Option<ContextDestructor>,
        aggregate_destructor: Option<ContextDestructor>,
        value_destructor: Option<ValueDestructor>,
    ) -> Self {
        Self {
            context,
            init,
            step,
            finalize,
            window: None,
            context_destructor,
            aggregate_destructor,
            value_destructor,
            abi_version: 1,
        }
    }

    /// Same, plus the callbacks that let the aggregate run as a window
    /// function.
    #[allow(clippy::too_many_arguments)]
    pub fn new_window(
        context: usize,
        init: InitAggFunction,
        step: StepFunction,
        finalize: FinalizeFunction,
        value: WindowValueFunction,
        inverse: WindowInverseFunction,
        context_destructor: Option<ContextDestructor>,
        aggregate_destructor: Option<ContextDestructor>,
        value_destructor: Option<ValueDestructor>,
    ) -> Self {
        Self {
            context,
            init,
            step,
            finalize,
            window: Some((value, inverse)),
            context_destructor,
            aggregate_destructor,
            value_destructor,
            abi_version: 1,
        }
    }

    #[must_use]
    pub fn with_abi_version(mut self, abi_version: u32) -> Self {
        self.abi_version = abi_version;
        self
    }
}

impl AggregateFunction for ExtAggregateAdapter {
    fn init(&self, _ctx: &mut FunctionContext<'_>) -> Result<Box<dyn AggregateState>> {
        let state = unsafe { (self.init)(self.context) };
        if state.is_null() {
            return Err(LimboError::ExtensionError(
                "aggregate init returned a null accumulator".to_string(),
            ));
        }
        Ok(Box::new(ExtAggState {
            context: self.context,
            state,
            step: self.step,
            finalize: self.finalize,
            window: self.window,
            aggregate_destructor: self.aggregate_destructor,
            value_destructor: self.value_destructor,
            abi_version: self.abi_version,
            live: true,
        }))
    }

    fn supports_window(&self) -> bool {
        self.window.is_some()
    }
}

impl Drop for ExtAggregateAdapter {
    fn drop(&mut self) {
        if let Some(context_destructor) = self.context_destructor {
            unsafe { context_destructor(self.context) };
        }
    }
}

/// Per-group accumulator of a C-ABI aggregate: a pointer the extension owns,
/// plus the callbacks that operate on it.
struct ExtAggState {
    context: usize,
    state: *mut AggCtx,
    step: StepFunction,
    finalize: FinalizeFunction,
    window: Option<(WindowValueFunction, WindowInverseFunction)>,
    aggregate_destructor: Option<ContextDestructor>,
    value_destructor: Option<ValueDestructor>,
    abi_version: u32,
    /// False once the state was handed back, so `Drop` doesn't free it twice.
    live: bool,
}

// SAFETY: `state` lives in a VDBE register of the statement that created it,
// so only one thread ever touches an accumulator at a time.
unsafe impl Send for ExtAggState {}

impl ExtAggState {
    /// Hands the accumulator back to the extension. Runs at most once.
    fn destroy(&mut self) {
        if !self.live {
            return;
        }
        self.live = false;
        if let Some(aggregate_destructor) = self.aggregate_destructor {
            unsafe { aggregate_destructor(self.state as usize) };
        }
    }
}

impl AggregateState for ExtAggState {
    fn step(&mut self, ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]) -> Result<()> {
        if !self.live {
            return Err(LimboError::ExtensionError(
                "extension aggregate stepped after it was finalized".to_string(),
            ));
        }
        let ext_values = to_ext_values(ctx, args);
        let argv = if ext_values.is_empty() {
            std::ptr::null()
        } else {
            ext_values.as_ptr()
        };
        let result = unsafe { (self.step)(self.context, self.state, args.len() as i32, argv) };
        // xStep's return value only says whether the step failed.
        let value = take_ext_result(ctx, self.abi_version, false, result, self.value_destructor);
        free_ext_values(ext_values);
        // A failing xStep ends the aggregate; give the state back now.
        if let Err(err) = value {
            self.destroy();
            return Err(err);
        }
        Ok(())
    }

    fn finalize(mut self: Box<Self>, ctx: &mut FunctionContext<'_>) -> Result<Value> {
        if !self.live {
            return Err(LimboError::ExtensionError(
                "extension aggregate finalized twice".to_string(),
            ));
        }
        let result = unsafe { (self.finalize)(self.context, self.state) };
        let value = take_ext_result(ctx, self.abi_version, true, result, self.value_destructor);
        self.destroy();
        value
    }

    fn value(&self, ctx: &mut FunctionContext<'_>) -> Result<Value> {
        let Some((value, _)) = self.window else {
            return Err(LimboError::ExtensionError(
                "extension aggregate has no xValue callback".to_string(),
            ));
        };
        if !self.live {
            return Err(LimboError::ExtensionError(
                "extension aggregate read after it was finalized".to_string(),
            ));
        }
        let result = unsafe { value(self.context, self.state) };
        take_ext_result(ctx, self.abi_version, true, result, self.value_destructor)
    }

    fn inverse(&mut self, ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]) -> Result<()> {
        let Some((_, inverse)) = self.window else {
            return Err(LimboError::ExtensionError(
                "extension aggregate has no xInverse callback".to_string(),
            ));
        };
        if !self.live {
            return Err(LimboError::ExtensionError(
                "extension aggregate inverse-stepped after it was finalized".to_string(),
            ));
        }
        let ext_values = to_ext_values(ctx, args);
        let argv = if ext_values.is_empty() {
            std::ptr::null()
        } else {
            ext_values.as_ptr()
        };
        let result = unsafe { inverse(self.context, self.state, args.len() as i32, argv) };
        let value = take_ext_result(ctx, self.abi_version, false, result, self.value_destructor);
        free_ext_values(ext_values);
        if let Err(err) = value {
            self.destroy();
            return Err(err);
        }
        Ok(())
    }
}

impl Drop for ExtAggState {
    fn drop(&mut self) {
        self.destroy();
    }
}
