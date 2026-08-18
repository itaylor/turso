//! User-defined functions for `sdk-kit`, mostly re-exported from `turso_core`.
//!
//! The `*PtrFunction` types and `*PtrAdapter`s here are the out-parameter
//! variants of the C ABI, for FFIs that cannot receive a struct returned by
//! value. They behave identically to the by-value adapters in `turso_core`.

use turso_ext::{AggCtx, ContextDestructor, InitAggFunction, Value as ExtValue, ValueDestructor};

pub use turso_core::ext::{
    ExtAggregateAdapter, ExtScalarAdapter, WindowInverseFunction, WindowValueFunction,
};
use turso_core::types::ValueRef;
pub use turso_core::udf::{
    aggregate_from_fns, AggregateFunction, AggregateState, FunctionContext, FunctionFlags,
    ScalarFunction,
};
pub use turso_core::LimboError;
pub use turso_core::Result as CoreResult;
use turso_core::Value;

/// [`turso_ext::ScalarFunction`] with an out-parameter result.
pub type ScalarPtrFunction =
    unsafe extern "C" fn(context: usize, argc: i32, argv: *const ExtValue, result: *mut ExtValue);

/// `xStep` with an out-parameter result. Writing an Error value fails the
/// statement; anything else written is discarded.
pub type AggregateStepPtrFunction = unsafe extern "C" fn(
    context: usize,
    aggregate_context: *mut AggCtx,
    argc: i32,
    argv: *const ExtValue,
    result: *mut ExtValue,
);

/// `xFinal` with an out-parameter result.
pub type AggregateFinalPtrFunction =
    unsafe extern "C" fn(context: usize, aggregate_context: *mut AggCtx, result: *mut ExtValue);

/// `xValue` with an out-parameter result.
pub type AggregateValuePtrFunction = AggregateFinalPtrFunction;

/// `xInverse` with an out-parameter result.
pub type AggregateInversePtrFunction = AggregateStepPtrFunction;

/// The caller must free the result with [`free_ext_values`].
fn to_ext_values(args: &[ValueRef<'_>]) -> Vec<ExtValue> {
    args.iter().map(|arg| arg.to_ffi()).collect()
}

fn free_ext_values(values: Vec<ExtValue>) {
    for value in values {
        unsafe { value.__free_internal_type() };
    }
}

/// Converts the FFI value a callback wrote, then frees it with the binding's
/// destructor.
fn take_ext_result(
    mut result: ExtValue,
    value_destructor: Option<ValueDestructor>,
) -> CoreResult<Value> {
    let value = Value::from_ffi_ref(&result);
    if let Some(value_destructor) = value_destructor {
        unsafe { value_destructor(&mut result) };
    } else {
        unsafe { result.__free_internal_type() };
    }
    value
}

/// A scalar function registered over the out-parameter C ABI.
pub struct ExtScalarPtrAdapter {
    context: usize,
    callback: ScalarPtrFunction,
    context_destructor: Option<ContextDestructor>,
    value_destructor: Option<ValueDestructor>,
}

impl ExtScalarPtrAdapter {
    pub fn new(
        context: usize,
        callback: ScalarPtrFunction,
        context_destructor: Option<ContextDestructor>,
        value_destructor: Option<ValueDestructor>,
    ) -> Self {
        Self {
            context,
            callback,
            context_destructor,
            value_destructor,
        }
    }
}

impl ScalarFunction for ExtScalarPtrAdapter {
    fn call(&self, _ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]) -> CoreResult<Value> {
        let ext_values = to_ext_values(args);
        let argv = if ext_values.is_empty() {
            std::ptr::null()
        } else {
            ext_values.as_ptr()
        };
        let mut result = ExtValue::null();
        unsafe { (self.callback)(self.context, args.len() as i32, argv, &mut result) };
        let value = take_ext_result(result, self.value_destructor);
        free_ext_values(ext_values);
        value
    }
}

impl Drop for ExtScalarPtrAdapter {
    fn drop(&mut self) {
        if let Some(context_destructor) = self.context_destructor {
            unsafe { context_destructor(self.context) };
        }
    }
}

/// An aggregate function registered over the out-parameter C ABI.
pub struct ExtAggregatePtrAdapter {
    context: usize,
    init: InitAggFunction,
    step: AggregateStepPtrFunction,
    finalize: AggregateFinalPtrFunction,
    /// Both present or both absent; required for window use.
    window: Option<(AggregateValuePtrFunction, AggregateInversePtrFunction)>,
    context_destructor: Option<ContextDestructor>,
    aggregate_destructor: Option<ContextDestructor>,
    value_destructor: Option<ValueDestructor>,
}

impl ExtAggregatePtrAdapter {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        context: usize,
        init: InitAggFunction,
        step: AggregateStepPtrFunction,
        finalize: AggregateFinalPtrFunction,
        window: Option<(AggregateValuePtrFunction, AggregateInversePtrFunction)>,
        context_destructor: Option<ContextDestructor>,
        aggregate_destructor: Option<ContextDestructor>,
        value_destructor: Option<ValueDestructor>,
    ) -> Self {
        Self {
            context,
            init,
            step,
            finalize,
            window,
            context_destructor,
            aggregate_destructor,
            value_destructor,
        }
    }
}

impl AggregateFunction for ExtAggregatePtrAdapter {
    fn init(&self, _ctx: &mut FunctionContext<'_>) -> CoreResult<Box<dyn AggregateState>> {
        let state = unsafe { (self.init)(self.context) };
        if state.is_null() {
            return Err(LimboError::ExtensionError(
                "aggregate init returned a null accumulator".to_string(),
            ));
        }
        Ok(Box::new(ExtAggPtrState {
            context: self.context,
            state,
            step: self.step,
            finalize: self.finalize,
            window: self.window,
            aggregate_destructor: self.aggregate_destructor,
            value_destructor: self.value_destructor,
            live: true,
        }))
    }

    fn supports_window(&self) -> bool {
        self.window.is_some()
    }
}

impl Drop for ExtAggregatePtrAdapter {
    fn drop(&mut self) {
        if let Some(context_destructor) = self.context_destructor {
            unsafe { context_destructor(self.context) };
        }
    }
}

struct ExtAggPtrState {
    context: usize,
    state: *mut AggCtx,
    step: AggregateStepPtrFunction,
    finalize: AggregateFinalPtrFunction,
    window: Option<(AggregateValuePtrFunction, AggregateInversePtrFunction)>,
    aggregate_destructor: Option<ContextDestructor>,
    value_destructor: Option<ValueDestructor>,
    /// False once `state` has been handed back to the binding, so `Drop`
    /// doesn't free it twice.
    live: bool,
}

// SAFETY: `state` lives in a VDBE register, so only one thread touches an
// accumulator at a time.
unsafe impl Send for ExtAggPtrState {}

impl ExtAggPtrState {
    /// Hands `state` back to the binding. Runs at most once.
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

impl AggregateState for ExtAggPtrState {
    fn step(&mut self, _ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]) -> CoreResult<()> {
        if !self.live {
            return Err(LimboError::ExtensionError(
                "extension aggregate stepped after it was finalized".to_string(),
            ));
        }
        let ext_values = to_ext_values(args);
        let argv = if ext_values.is_empty() {
            std::ptr::null()
        } else {
            ext_values.as_ptr()
        };
        let mut result = ExtValue::null();
        unsafe {
            (self.step)(
                self.context,
                self.state,
                args.len() as i32,
                argv,
                &mut result,
            )
        };
        let value = take_ext_result(result, self.value_destructor);
        free_ext_values(ext_values);
        if let Err(err) = value {
            self.destroy();
            return Err(err);
        }
        Ok(())
    }

    fn finalize(mut self: Box<Self>, _ctx: &mut FunctionContext<'_>) -> CoreResult<Value> {
        if !self.live {
            return Err(LimboError::ExtensionError(
                "extension aggregate finalized twice".to_string(),
            ));
        }
        let mut result = ExtValue::null();
        unsafe { (self.finalize)(self.context, self.state, &mut result) };
        let value = take_ext_result(result, self.value_destructor);
        self.destroy();
        value
    }

    fn value(&self, _ctx: &mut FunctionContext<'_>) -> CoreResult<Value> {
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
        let mut result = ExtValue::null();
        unsafe { value(self.context, self.state, &mut result) };
        take_ext_result(result, self.value_destructor)
    }

    fn inverse(&mut self, _ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]) -> CoreResult<()> {
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
        let ext_values = to_ext_values(args);
        let argv = if ext_values.is_empty() {
            std::ptr::null()
        } else {
            ext_values.as_ptr()
        };
        let mut result = ExtValue::null();
        unsafe {
            inverse(
                self.context,
                self.state,
                args.len() as i32,
                argv,
                &mut result,
            )
        };
        let value = take_ext_result(result, self.value_destructor);
        free_ext_values(ext_values);
        if let Err(err) = value {
            self.destroy();
            return Err(err);
        }
        Ok(())
    }
}

impl Drop for ExtAggPtrState {
    fn drop(&mut self) {
        self.destroy();
    }
}
