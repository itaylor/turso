use crate::{ResultCode, Value};
use std::{
    ffi::{c_char, c_void},
    fmt::Display,
};

pub type ContextDestructor = unsafe extern "C" fn(context: usize);
pub type ValueDestructor = unsafe extern "C" fn(result: *mut Value);
pub type ScalarFunction = unsafe extern "C" fn(
    context: usize,
    argc: i32,
    argv: *const Value,
    context_destructor: Option<ContextDestructor>,
    value_destructor: Option<ValueDestructor>,
) -> Value;

pub type RegisterScalarFn = unsafe extern "C" fn(
    ctx: *mut c_void,
    name: *const c_char,
    argc: i32,
    deterministic: bool,
    context: usize,
    func: ScalarFunction,
    context_destructor: Option<ContextDestructor>,
    value_destructor: Option<ValueDestructor>,
) -> ResultCode;

pub type UnregisterFunctionFn =
    unsafe extern "C" fn(ctx: *mut c_void, name: *const c_char) -> ResultCode;

pub type RegisterAggFn = unsafe extern "C" fn(
    ctx: *mut c_void,
    name: *const c_char,
    args: i32,
    context: usize,
    init: InitAggFunction,
    step: StepFunction,
    finalize: FinalizeFunction,
    context_destructor: Option<ContextDestructor>,
    aggregate_destructor: Option<ContextDestructor>,
    value_destructor: Option<ValueDestructor>,
) -> ResultCode;

/// Like [`RegisterAggFn`] plus SQLite's `xValue` and `xInverse`. `flags` uses
/// the same bits as `sqlite3_create_function`; pass `0` for none.
pub type RegisterWindowFn = unsafe extern "C" fn(
    ctx: *mut c_void,
    name: *const c_char,
    args: i32,
    flags: u32,
    context: usize,
    init: InitAggFunction,
    step: StepFunction,
    finalize: FinalizeFunction,
    value: WindowValueFunction,
    inverse: WindowInverseFunction,
    context_destructor: Option<ContextDestructor>,
    aggregate_destructor: Option<ContextDestructor>,
    value_destructor: Option<ValueDestructor>,
) -> ResultCode;

pub type InitAggFunction = unsafe extern "C" fn(context: usize) -> *mut AggCtx;
pub type StepFunction =
    unsafe extern "C" fn(context: usize, ctx: *mut AggCtx, argc: i32, argv: *const Value) -> Value;
pub type FinalizeFunction = unsafe extern "C" fn(context: usize, ctx: *mut AggCtx) -> Value;

/// Mirrors SQLite's `xValue`: reads the running value and must leave `ctx`
/// usable for further steps.
pub type WindowValueFunction = unsafe extern "C" fn(context: usize, ctx: *mut AggCtx) -> Value;

/// Mirrors SQLite's `xInverse`: undoes the step for a row that left the frame.
pub type WindowInverseFunction = StepFunction;

#[repr(C)]
pub struct AggCtx {
    pub state: *mut c_void,
}

pub trait AggFunc {
    type State: Default;
    type Error: Display;
    const NAME: &'static str;
    const ARGS: i32;

    /// Set to `true` only if [`AggFunc::value`] and [`AggFunc::inverse`] are
    /// also implemented. Aggregates that cannot cheaply undo a step (a median,
    /// say) should leave this `false`.
    const WINDOW: bool = false;

    fn step(state: &mut Self::State, args: &[Value]);
    fn finalize(state: Self::State) -> Result<Value, Self::Error>;

    /// Running value; must not consume the state. Only called when
    /// [`AggFunc::WINDOW`].
    fn value(_state: &Self::State) -> Result<Value, Self::Error> {
        Ok(Value::error_with_message(format!(
            "{}() may not be used as a window function",
            Self::NAME
        )))
    }

    /// Undoes the [`AggFunc::step`] for a row that left the frame. Only called
    /// when [`AggFunc::WINDOW`].
    fn inverse(_state: &mut Self::State, _args: &[Value]) {}
}

/// A scalar function that carries opaque, per-registration state.
///
/// `State` is constructed once per registration via [`ScalarFunc::init`], shared
/// by reference across every invocation, and dropped when the function is
/// unregistered or the owning connection is dropped. Because a single registration
/// is shared across connections and may be invoked concurrently, `State` must be
/// `Send + Sync`.
///
/// Stateless functions should use the `#[scalar]` attribute macro instead.
pub trait ScalarFunc {
    type State: Send + Sync;
    const NAME: &'static str;
    const ALIAS: Option<&'static str> = None;
    /// Argument count, or `-1` for variadic.
    const ARGC: i32 = -1;
    const DETERMINISTIC: bool = false;

    fn init() -> Self::State;
    fn call(state: &Self::State, args: &[Value]) -> Value;
}
