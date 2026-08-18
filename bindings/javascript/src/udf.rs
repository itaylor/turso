//! User-defined functions for the JavaScript bindings.
//!
//! A JavaScript value may only be touched on the thread whose `napi_env`
//! created it, but `turso_core` requires registered functions to be
//! `Send + Sync`. [`JsRef`] therefore records its owning thread; reads from
//! any other thread fail, and `Drop` leaks instead of deleting the reference.

use std::cell::RefCell;
use std::ffi::c_char;
use std::ptr;
use std::sync::{Arc, Mutex};
use std::thread::ThreadId;

use napi::bindgen_prelude::*;
use napi::{sys, Env};
use turso_core::{
    AggregateFunction, AggregateState, FunctionContext, LimboError, ScalarFunction, Value, ValueRef,
};

thread_local! {
    /// The value a user-defined function threw during the current step, so
    /// `step_sync` can rethrow the original error object instead of a string.
    static PENDING_JS_ERROR: RefCell<Option<napi::Error>> = const { RefCell::new(None) };
}

pub(crate) fn clear_pending_js_error() {
    PENDING_JS_ERROR.with(|slot| slot.borrow_mut().take());
}

pub(crate) fn take_pending_js_error() -> Option<napi::Error> {
    PENDING_JS_ERROR.with(|slot| slot.borrow_mut().take())
}

fn set_pending_js_error(err: napi::Error) {
    PENDING_JS_ERROR.with(|slot| *slot.borrow_mut() = Some(err));
}

/// A napi reference to a JavaScript value, owned by the thread that made it.
struct JsRef {
    env: sys::napi_env,
    inner: sys::napi_ref,
    owner: ThreadId,
}

// SAFETY: every read of the referenced value checks `owner` first, and `Drop`
// leaks rather than deleting the reference from a foreign thread.
unsafe impl Send for JsRef {}
unsafe impl Sync for JsRef {}

impl JsRef {
    fn new(env: sys::napi_env, value: sys::napi_value) -> napi::Result<Self> {
        let mut inner = ptr::null_mut();
        check_status!(
            unsafe { sys::napi_create_reference(env, value, 1, &mut inner) },
            "failed to create a reference to the JavaScript callback"
        )?;
        Ok(Self {
            env,
            inner,
            owner: std::thread::current().id(),
        })
    }

    fn value(&self) -> turso_core::Result<sys::napi_value> {
        if std::thread::current().id() != self.owner {
            return Err(LimboError::SqlError(
                "a JavaScript user-defined function can only run on the thread that registered it"
                    .to_string(),
            ));
        }
        let mut value = ptr::null_mut();
        let status = unsafe { sys::napi_get_reference_value(self.env, self.inner, &mut value) };
        if status != sys::Status::napi_ok || value.is_null() {
            return Err(LimboError::SqlError(
                "the JavaScript user-defined function is no longer available".to_string(),
            ));
        }
        Ok(value)
    }
}

impl Drop for JsRef {
    fn drop(&mut self) {
        if std::thread::current().id() != self.owner {
            // Deleting a napi reference off its owning thread is undefined
            // behaviour, so leak it instead.
            tracing::warn!("leaking a user-defined function reference dropped on another thread");
            return;
        }
        let status = unsafe { sys::napi_delete_reference(self.env, self.inner) };
        debug_assert_eq!(status, sys::Status::napi_ok, "delete reference failed");
    }
}

/// A napi handle scope held for one callback, so per-row arguments and results
/// are freed as stepping proceeds instead of piling up until it returns.
struct HandleScope {
    env: sys::napi_env,
    scope: sys::napi_handle_scope,
}

impl HandleScope {
    fn open(env: sys::napi_env) -> turso_core::Result<Self> {
        let mut scope = ptr::null_mut();
        let status = unsafe { sys::napi_open_handle_scope(env, &mut scope) };
        if status != sys::Status::napi_ok {
            return Err(LimboError::SqlError(
                "failed to open a JavaScript handle scope".to_string(),
            ));
        }
        Ok(Self { env, scope })
    }
}

impl Drop for HandleScope {
    fn drop(&mut self) {
        let status = unsafe { sys::napi_close_handle_scope(self.env, self.scope) };
        debug_assert_eq!(status, sys::Status::napi_ok, "close handle scope failed");
    }
}

fn core_error(context: &str, err: napi::Error) -> LimboError {
    LimboError::SqlError(format!("{context}: {}", err.reason))
}

fn named_property(
    env: sys::napi_env,
    object: sys::napi_value,
    name: *const c_char,
) -> turso_core::Result<sys::napi_value> {
    let mut value = ptr::null_mut();
    let status = unsafe { sys::napi_get_named_property(env, object, name, &mut value) };
    if status != sys::Status::napi_ok {
        return Err(LimboError::SqlError(
            "failed to read a property of a user-defined function".to_string(),
        ));
    }
    Ok(value)
}

fn set_named_property(
    env: sys::napi_env,
    object: sys::napi_value,
    name: *const c_char,
    value: sys::napi_value,
) -> turso_core::Result<()> {
    let status = unsafe { sys::napi_set_named_property(env, object, name, value) };
    if status != sys::Status::napi_ok {
        return Err(LimboError::SqlError(
            "failed to update the state of a user-defined aggregate".to_string(),
        ));
    }
    Ok(())
}

fn value_type(
    env: sys::napi_env,
    value: sys::napi_value,
) -> turso_core::Result<sys::napi_valuetype> {
    let mut kind = 0;
    let status = unsafe { sys::napi_typeof(env, value, &mut kind) };
    if status != sys::Status::napi_ok {
        return Err(LimboError::SqlError(
            "failed to inspect a JavaScript value".to_string(),
        ));
    }
    Ok(kind)
}

/// Calls with `this` set to `undefined`. A thrown value aborts the statement
/// and is stashed for `step_sync` to rethrow.
fn call_js(
    env: sys::napi_env,
    func: sys::napi_value,
    args: &[sys::napi_value],
) -> turso_core::Result<sys::napi_value> {
    let mut this = ptr::null_mut();
    let status = unsafe { sys::napi_get_undefined(env, &mut this) };
    if status != sys::Status::napi_ok {
        return Err(LimboError::SqlError(
            "failed to prepare a call into JavaScript".to_string(),
        ));
    }
    let mut result = ptr::null_mut();
    let status =
        unsafe { sys::napi_call_function(env, this, func, args.len(), args.as_ptr(), &mut result) };
    match status {
        sys::Status::napi_ok => Ok(result),
        sys::Status::napi_pending_exception => Err(take_thrown_value(env)),
        _ => Err(LimboError::SqlError(
            "calling a JavaScript user-defined function failed".to_string(),
        )),
    }
}

/// Leaving the exception pending would make the next napi call abort the
/// process, so it is cleared here even though it is only rethrown later.
fn take_thrown_value(env: sys::napi_env) -> LimboError {
    let mut exception = ptr::null_mut();
    let status = unsafe { sys::napi_get_and_clear_last_exception(env, &mut exception) };
    if status != sys::Status::napi_ok || exception.is_null() {
        return LimboError::SqlError("a user-defined function threw".to_string());
    }
    let err = napi::Error::from(unsafe { Unknown::from_raw_unchecked(env, exception) });
    let message = err.reason.clone();
    set_pending_js_error(err);
    LimboError::SqlError(message)
}

fn value_ref_to_js<'env>(
    env: &'env Env,
    value: ValueRef<'_>,
    safe_integers: bool,
) -> napi::Result<Unknown<'env>> {
    match value {
        ValueRef::Null => ToNapiValue::into_unknown(Null, env),
        ValueRef::Numeric(turso_core::Numeric::Integer(i)) => {
            if safe_integers {
                ToNapiValue::into_unknown(BigInt::from(i), env)
            } else {
                ToNapiValue::into_unknown(i as f64, env)
            }
        }
        ValueRef::Numeric(turso_core::Numeric::Float(f)) => {
            ToNapiValue::into_unknown(f64::from(f), env)
        }
        ValueRef::Text(text) => ToNapiValue::into_unknown(text.as_str(), env),
        ValueRef::Blob(blob) => {
            #[cfg(not(feature = "browser"))]
            {
                ToNapiValue::into_unknown(Buffer::from(blob), env)
            }
            // emnapi does not support Buffer.
            #[cfg(feature = "browser")]
            {
                ToNapiValue::into_unknown(Uint8Array::from(blob), env)
            }
        }
    }
}

/// Returns `Ok(None)` for a value with no SQL equivalent, leaving the caller to
/// choose between coercing and rejecting.
pub(crate) fn js_to_value(value: Unknown<'_>) -> napi::Result<Option<Value>> {
    let converted = match value.get_type()? {
        ValueType::Null => Value::Null,
        ValueType::Number => {
            let n: f64 = unsafe { value.cast()? };
            if n.fract() == 0.0 && n >= i64::MIN as f64 && n <= i64::MAX as f64 {
                Value::from_i64(n as i64)
            } else {
                Value::from_f64(n)
            }
        }
        ValueType::BigInt => {
            let bigint_str = value.coerce_to_string()?.into_utf8()?.as_str()?.to_owned();
            let bigint_value = bigint_str.parse::<i64>().map_err(|e| {
                napi::Error::new(
                    Status::NumberExpected,
                    format!("failed to parse BigInt: {e}"),
                )
            })?;
            Value::from_i64(bigint_value)
        }
        ValueType::String => {
            let s = value.coerce_to_string()?.into_utf8()?;
            Value::Text(s.as_str()?.to_owned().into())
        }
        ValueType::Boolean => {
            let b: bool = unsafe { value.cast()? };
            Value::from_i64(if b { 1 } else { 0 })
        }
        ValueType::Object => {
            let obj = value.coerce_to_object()?;
            if !obj.is_buffer()? && !obj.is_typedarray()? {
                return Ok(None);
            }
            let length = obj.get_named_property::<u32>("length")?;
            let mut bytes = Vec::with_capacity(length as usize);
            for i in 0..length {
                bytes.push(obj.get_element::<u32>(i)? as u8);
            }
            Value::Blob(bytes)
        }
        _ => return Ok(None),
    };
    Ok(Some(converted))
}

/// `undefined` and `null` both mean SQL NULL; anything the engine has no type
/// for is an error, as in better-sqlite3.
fn js_result_to_value(
    env: sys::napi_env,
    value: sys::napi_value,
    what: &str,
) -> turso_core::Result<Value> {
    if value_type(env, value)? == sys::ValueType::napi_undefined {
        return Ok(Value::Null);
    }
    let unknown = unsafe { Unknown::from_raw_unchecked(env, value) };
    match js_to_value(unknown) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(type_error(
            env,
            &format!("{what} returned an invalid value"),
        )),
        Err(err) => Err(core_error(what, err)),
    }
}

/// better-sqlite3 reports a bad callback result as a `TypeError`.
fn type_error(env: sys::napi_env, message: &str) -> LimboError {
    let mut text = ptr::null_mut();
    let created = unsafe {
        sys::napi_create_string_utf8(
            env,
            message.as_ptr().cast(),
            message.len() as isize,
            &mut text,
        )
    };
    let mut error = ptr::null_mut();
    if created == sys::Status::napi_ok {
        let created =
            unsafe { sys::napi_create_type_error(env, ptr::null_mut(), text, &mut error) };
        if created == sys::Status::napi_ok {
            set_pending_js_error(napi::Error::from(unsafe {
                Unknown::from_raw_unchecked(env, error)
            }));
        }
    }
    LimboError::SqlError(message.to_string())
}

struct JsFunction {
    /// Object holding the callbacks alive: `{ func }` for a scalar function,
    /// `{ start, step, inverse, result }` for an aggregate.
    holder: JsRef,
    /// `None` follows the database's `defaultSafeIntegers`, read at call time.
    safe_integers: Option<bool>,
    default_safe_integers: Arc<Mutex<bool>>,
    /// How the function is named in error messages.
    what: String,
}

impl JsFunction {
    fn new(
        env: &Env,
        holder: sys::napi_value,
        safe_integers: Option<bool>,
        default_safe_integers: Arc<Mutex<bool>>,
        what: String,
    ) -> napi::Result<Self> {
        Ok(Self {
            holder: JsRef::new(env.raw(), holder)?,
            safe_integers,
            default_safe_integers,
            what,
        })
    }

    fn env(&self) -> sys::napi_env {
        self.holder.env
    }

    fn safe_integers(&self) -> bool {
        self.safe_integers
            .unwrap_or_else(|| *self.default_safe_integers.lock().unwrap())
    }

    fn enter(&self) -> turso_core::Result<(HandleScope, sys::napi_value)> {
        let scope = HandleScope::open(self.env())?;
        let holder = self.holder.value()?;
        Ok((scope, holder))
    }

    /// `leading` reserves the first slot for an aggregate's running total.
    fn js_args(
        &self,
        args: &[ValueRef<'_>],
        leading: Option<sys::napi_value>,
    ) -> turso_core::Result<Vec<sys::napi_value>> {
        let env = Env::from_raw(self.env());
        let mut js_args = Vec::with_capacity(args.len() + usize::from(leading.is_some()));
        js_args.extend(leading);
        for arg in args {
            let converted = value_ref_to_js(&env, *arg, self.safe_integers())
                .map_err(|err| core_error(&self.what, err))?;
            js_args.push(converted.raw());
        }
        Ok(js_args)
    }
}

pub(crate) struct JsScalarFunction {
    func: JsFunction,
}

impl ScalarFunction for JsScalarFunction {
    fn call(
        &self,
        _ctx: &mut FunctionContext<'_>,
        args: &[ValueRef<'_>],
    ) -> turso_core::Result<Value> {
        let (_scope, holder) = self.func.enter()?;
        let callback = named_property(self.func.env(), holder, c"func".as_ptr())?;
        let js_args = self.func.js_args(args, None)?;
        let result = call_js(self.func.env(), callback, &js_args)?;
        js_result_to_value(self.func.env(), result, &self.func.what)
    }
}

pub(crate) struct JsAggregateFunction {
    func: Arc<JsFunction>,
    has_inverse: bool,
    has_result: bool,
}

impl AggregateFunction for JsAggregateFunction {
    fn init(&self, _ctx: &mut FunctionContext<'_>) -> turso_core::Result<Box<dyn AggregateState>> {
        let (_scope, holder) = self.func.enter()?;
        let env = self.func.env();
        let start = named_property(env, holder, c"start".as_ptr())?;
        // `start` may be the initial value itself or a function producing a
        // fresh one for every group.
        let total = if value_type(env, start)? == sys::ValueType::napi_function {
            call_js(env, start, &[])?
        } else {
            start
        };

        // A napi reference to a primitive is not portable, so the running
        // total lives on a JavaScript object.
        let mut state = ptr::null_mut();
        let status = unsafe { sys::napi_create_object(env, &mut state) };
        if status != sys::Status::napi_ok {
            return Err(LimboError::SqlError(
                "failed to create the state of a user-defined aggregate".to_string(),
            ));
        }
        set_named_property(env, state, c"total".as_ptr(), total)?;

        Ok(Box::new(JsAggregateState {
            func: self.func.clone(),
            state: JsRef::new(env, state).map_err(|err| core_error(&self.func.what, err))?,
            has_inverse: self.has_inverse,
            has_result: self.has_result,
        }))
    }

    fn supports_window(&self) -> bool {
        self.has_inverse
    }
}

struct JsAggregateState {
    func: Arc<JsFunction>,
    /// `{ total }`, the accumulator for one group or window frame.
    state: JsRef,
    has_inverse: bool,
    has_result: bool,
}

impl JsAggregateState {
    /// `step` and `inverse` take the running total first and return the new
    /// one, or `undefined` to keep the old one.
    fn accumulate(
        &mut self,
        callback: *const c_char,
        args: &[ValueRef<'_>],
    ) -> turso_core::Result<()> {
        let (_scope, holder) = self.func.enter()?;
        let env = self.func.env();
        let state = self.state.value()?;
        let total = named_property(env, state, c"total".as_ptr())?;
        let callback = named_property(env, holder, callback)?;
        let js_args = self.func.js_args(args, Some(total))?;
        let updated = call_js(env, callback, &js_args)?;
        if value_type(env, updated)? != sys::ValueType::napi_undefined {
            set_named_property(env, state, c"total".as_ptr(), updated)?;
        }
        Ok(())
    }

    fn current(&self) -> turso_core::Result<Value> {
        let (_scope, holder) = self.func.enter()?;
        let env = self.func.env();
        let state = self.state.value()?;
        let total = named_property(env, state, c"total".as_ptr())?;
        let final_value = if self.has_result {
            let result = named_property(env, holder, c"result".as_ptr())?;
            call_js(env, result, &[total])?
        } else {
            total
        };
        js_result_to_value(env, final_value, &self.func.what)
    }
}

impl AggregateState for JsAggregateState {
    fn step(
        &mut self,
        _ctx: &mut FunctionContext<'_>,
        args: &[ValueRef<'_>],
    ) -> turso_core::Result<()> {
        self.accumulate(c"step".as_ptr(), args)
    }

    fn finalize(self: Box<Self>, _ctx: &mut FunctionContext<'_>) -> turso_core::Result<Value> {
        self.current()
    }

    fn value(&self, _ctx: &mut FunctionContext<'_>) -> turso_core::Result<Value> {
        self.current()
    }

    fn inverse(
        &mut self,
        _ctx: &mut FunctionContext<'_>,
        args: &[ValueRef<'_>],
    ) -> turso_core::Result<()> {
        if !self.has_inverse {
            return Err(LimboError::ParseError(format!(
                "{} may not be used as a window function",
                self.func.what
            )));
        }
        self.accumulate(c"inverse".as_ptr(), args)
    }
}

fn make_holder(env: &Env, entries: &[(&str, sys::napi_value)]) -> napi::Result<sys::napi_value> {
    let mut holder = Object::new(env)?;
    for (name, value) in entries {
        let value = unsafe { Unknown::from_raw_unchecked(env.raw(), *value) };
        holder.set(*name, value)?;
    }
    Ok(holder.raw())
}

pub(crate) fn scalar_function(
    env: &Env,
    name: &str,
    func: Unknown<'_>,
    safe_integers: Option<bool>,
    default_safe_integers: Arc<Mutex<bool>>,
) -> napi::Result<JsScalarFunction> {
    expect_function(func, "the function")?;
    let holder = make_holder(env, &[("func", func.raw())])?;
    Ok(JsScalarFunction {
        func: JsFunction::new(
            env,
            holder,
            safe_integers,
            default_safe_integers,
            format!("user-defined function {name}()"),
        )?,
    })
}

pub(crate) struct AggregateCallbacks<'a> {
    pub start: Unknown<'a>,
    pub step: Unknown<'a>,
    pub inverse: Option<Unknown<'a>>,
    pub result: Option<Unknown<'a>>,
}

pub(crate) fn aggregate_function(
    env: &Env,
    name: &str,
    callbacks: AggregateCallbacks<'_>,
    safe_integers: Option<bool>,
    default_safe_integers: Arc<Mutex<bool>>,
) -> napi::Result<JsAggregateFunction> {
    let AggregateCallbacks {
        start,
        step,
        inverse,
        result,
    } = callbacks;
    expect_function(step, "the \"step\" option")?;
    if let Some(inverse) = inverse {
        expect_function(inverse, "the \"inverse\" option")?;
    }
    if let Some(result) = result {
        expect_function(result, "the \"result\" option")?;
    }
    let missing = undefined_value(env.raw())?;
    let holder = make_holder(
        env,
        &[
            ("start", start.raw()),
            ("step", step.raw()),
            ("inverse", inverse.map_or(missing, |f| f.raw())),
            ("result", result.map_or(missing, |f| f.raw())),
        ],
    )?;
    Ok(JsAggregateFunction {
        func: Arc::new(JsFunction::new(
            env,
            holder,
            safe_integers,
            default_safe_integers,
            format!("user-defined aggregate {name}()"),
        )?),
        has_inverse: inverse.is_some(),
        has_result: result.is_some(),
    })
}

fn undefined_value(env: sys::napi_env) -> napi::Result<sys::napi_value> {
    let mut undefined = ptr::null_mut();
    check_status!(
        unsafe { sys::napi_get_undefined(env, &mut undefined) },
        "failed to read undefined"
    )?;
    Ok(undefined)
}

fn expect_function(value: Unknown<'_>, what: &str) -> napi::Result<()> {
    if value.get_type()? != ValueType::Function {
        return Err(napi::Error::new(
            Status::InvalidArg,
            format!("expected {what} to be a function"),
        ));
    }
    Ok(())
}
