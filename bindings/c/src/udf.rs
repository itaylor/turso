//! The user-defined-function half of the sqlite3 C API. Registrations live on
//! the connection, not in process-global state.

use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::cell::Cell;
use std::ffi::{self, CStr};
use std::sync::Arc;

use turso_core::udf::MAX_FUNCTION_NAME_LEN;
use turso_core::{
    AggregateFunction, AggregateState, Dialect, FunctionContext, FunctionFlags, LimboError,
    Numeric, ScalarFunction, SqliteDialect, Value, ValueRef, MAX_FUNCTION_ARG,
};

use crate::{
    sqlite3, SQLITE_BLOB, SQLITE_BUSY, SQLITE_ERROR, SQLITE_FLOAT, SQLITE_INTEGER, SQLITE_MISUSE,
    SQLITE_NOMEM, SQLITE_NULL, SQLITE_OK, SQLITE_TEXT, SQLITE_TOOBIG,
};

/* Low bits of the `enc` argument. Turso stores text as UTF-8 only, so the
function is registered once whichever encoding is asked for. */
pub const SQLITE_UTF8: ffi::c_int = 1;
pub const SQLITE_UTF16LE: ffi::c_int = 2;
pub const SQLITE_UTF16BE: ffi::c_int = 3;
pub const SQLITE_UTF16: ffi::c_int = 4;
pub const SQLITE_ANY: ffi::c_int = 5;
pub const SQLITE_UTF16_ALIGNED: ffi::c_int = 8;

/* Property flags, or-ed into `enc`. Same bit values as `FunctionFlags`. */
pub const SQLITE_DETERMINISTIC: ffi::c_int = 0x0000_0800;
pub const SQLITE_DIRECTONLY: ffi::c_int = 0x0008_0000;
pub const SQLITE_SUBTYPE: ffi::c_int = 0x0010_0000;
pub const SQLITE_INNOCUOUS: ffi::c_int = 0x0020_0000;
pub const SQLITE_RESULT_SUBTYPE: ffi::c_int = 0x0100_0000;

/// SQLite's default `SQLITE_MAX_LENGTH`; past it a call fails SQLITE_TOOBIG.
const MAX_LENGTH: u64 = 1_000_000_000;

/// The `xFunc`, `xStep` and `xInverse` callback shape.
pub type XFunc = unsafe extern "C" fn(*mut sqlite3_context, ffi::c_int, *mut *mut sqlite3_value);
/// The `xFinal` and `xValue` callback shape.
pub type XFinal = unsafe extern "C" fn(*mut sqlite3_context);
/// The `xDestroy` callback shape, and the destructor of a text/blob result.
pub type XDestroy = unsafe extern "C" fn(*mut ffi::c_void);

/// One SQL value as a C callback sees it. An argument is an owned copy that
/// lives only for the call, which is all SQLite promises.
pub struct sqlite3_value {
    value: Value,
    /// Null unless this is an argument. Only [`sqlite3_value_pointer`] needs
    /// it: a pointer object rides beside the register, not inside the `Value`.
    core: *mut FunctionContext<'static>,
    index: usize,
    subtype: u8,
    /// Always false: the engine does not mark registers a bind filled.
    from_bind: bool,
    /// UTF-8 rendering with a trailing NUL that `sqlite3_value_bytes` skips.
    text: Option<Vec<u8>>,
    /// The same in UTF-16, each with a trailing NUL code unit.
    text16le: Option<Vec<u8>>,
    text16be: Option<Vec<u8>>,
}

impl sqlite3_value {
    pub(crate) fn new(value: Value) -> Self {
        let subtype = value.subtype();
        Self {
            value,
            core: std::ptr::null_mut(),
            index: 0,
            subtype,
            from_bind: false,
            text: None,
            text16le: None,
            text16be: None,
        }
    }

    fn from_arg(
        value: ValueRef<'_>,
        core: *mut FunctionContext<'static>,
        index: usize,
        subtype: u8,
    ) -> Self {
        let mut copy = Self::new(value.to_owned().unwrap_or(Value::Null));
        copy.subtype = subtype;
        copy.core = core;
        copy.index = index;
        copy
    }

    /// Keeps the subtype, which belongs to the argument and not to whatever
    /// `sqlite3_value_numeric_type` made of it.
    fn set(&mut self, value: Value) {
        self.value = value;
        self.text = None;
        self.text16le = None;
        self.text16be = None;
    }

    fn into_value(self) -> Value {
        self.value
    }

    fn type_code(&self) -> ffi::c_int {
        match self.value {
            Value::Null => SQLITE_NULL,
            Value::Numeric(Numeric::Integer(_)) => SQLITE_INTEGER,
            Value::Numeric(Numeric::Float(_)) => SQLITE_FLOAT,
            Value::Text(_) => SQLITE_TEXT,
            Value::Blob(_) => SQLITE_BLOB,
        }
    }

    /// Blobs render as raw bytes and numbers as decimal, per
    /// `sqlite3_value_text`.
    fn text_bytes(&mut self) -> Option<&[u8]> {
        if self.text.is_none() {
            let mut buf = Vec::new();
            match &self.value {
                Value::Null => return None,
                Value::Text(t) => buf.extend_from_slice(t.as_str().as_bytes()),
                Value::Blob(b) => buf.extend_from_slice(b),
                numeric => buf.extend_from_slice(numeric.to_string().as_bytes()),
            }
            buf.push(0);
            self.text = Some(buf);
        }
        self.text.as_deref()
    }

    fn text16_bytes(&mut self, big_endian: bool) -> Option<&[u8]> {
        let cached = if big_endian {
            self.text16be.is_some()
        } else {
            self.text16le.is_some()
        };
        if !cached {
            let utf8 = self.text_bytes()?;
            let text = String::from_utf8_lossy(&utf8[..utf8.len() - 1]).into_owned();
            let mut buf = Vec::with_capacity(text.len() * 2 + 2);
            for unit in text.encode_utf16() {
                if big_endian {
                    buf.extend_from_slice(&unit.to_be_bytes());
                } else {
                    buf.extend_from_slice(&unit.to_le_bytes());
                }
            }
            buf.extend_from_slice(&[0, 0]);
            if big_endian {
                self.text16be = Some(buf);
            } else {
                self.text16le = Some(buf);
            }
        }
        if big_endian {
            self.text16be.as_deref()
        } else {
            self.text16le.as_deref()
        }
    }

    fn int64(&self) -> i64 {
        match &self.value {
            Value::Numeric(Numeric::Integer(i)) => *i,
            other => match other.exec_cast("INTEGER") {
                Ok(Value::Numeric(Numeric::Integer(i))) => i,
                _ => 0,
            },
        }
    }

    fn double(&self) -> f64 {
        match &self.value {
            Value::Numeric(n) => match n {
                Numeric::Float(f) => f64::from(*f),
                Numeric::Integer(i) => *i as f64,
            },
            other => match other.exec_cast("REAL") {
                Ok(Value::Numeric(Numeric::Float(f))) => f64::from(f),
                Ok(Value::Numeric(Numeric::Integer(i))) => i as f64,
                _ => 0.0,
            },
        }
    }
}

fn take_digits(bytes: &[u8], at: &mut usize) -> usize {
    let digits = bytes[*at..]
        .iter()
        .take_while(|b| b.is_ascii_digit())
        .count();
    *at += digits;
    digits
}

/// SQLite's NUMERIC affinity: "12abc" stays text, where CAST would yield 12.
fn numeric_affinity(text: &str) -> Option<Value> {
    let trimmed = text.trim_matches(|c: char| c.is_ascii_whitespace());
    let bytes = trimmed.as_bytes();
    let mut at = 0;
    if matches!(bytes.first(), Some(b'+' | b'-')) {
        at += 1;
    }
    let mut digits = take_digits(bytes, &mut at);
    let mut is_float = false;
    if bytes.get(at) == Some(&b'.') {
        is_float = true;
        at += 1;
        digits += take_digits(bytes, &mut at);
    }
    if digits == 0 {
        return None;
    }
    if matches!(bytes.get(at), Some(b'e' | b'E')) {
        let mut after_exponent = at + 1;
        if matches!(bytes.get(after_exponent), Some(b'+' | b'-')) {
            after_exponent += 1;
        }
        if take_digits(bytes, &mut after_exponent) == 0 {
            return None;
        }
        is_float = true;
        at = after_exponent;
    }
    if at != bytes.len() {
        return None;
    }
    if !is_float {
        if let Ok(integer) = trimmed.parse::<i64>() {
            return Some(Value::from_i64(integer));
        }
    }
    trimmed.parse::<f64>().ok().map(Value::from_f64)
}

/// The zero-filled `sqlite3_aggregate_context` buffer, freed after `xFinal`.
struct AggBuffer {
    ptr: Cell<*mut u8>,
    len: Cell<usize>,
}

/// SQLite aligns the aggregate context to 8 bytes.
const AGG_ALIGN: usize = 8;

impl AggBuffer {
    fn new() -> Self {
        Self {
            ptr: Cell::new(std::ptr::null_mut()),
            len: Cell::new(0),
        }
    }

    /// SQLite's rule: the first call fixes the size, later calls get that
    /// same buffer, and a first call asking for nothing returns NULL.
    fn get(&self, n_bytes: ffi::c_int) -> *mut ffi::c_void {
        if !self.ptr.get().is_null() {
            return self.ptr.get() as *mut ffi::c_void;
        }
        if n_bytes <= 0 {
            return std::ptr::null_mut();
        }
        let len = n_bytes as usize;
        let Ok(layout) = Layout::from_size_align(len, AGG_ALIGN) else {
            return std::ptr::null_mut();
        };
        // SAFETY: non-zero size, and released exactly once in Drop.
        let ptr = unsafe { alloc_zeroed(layout) };
        if ptr.is_null() {
            return std::ptr::null_mut();
        }
        self.ptr.set(ptr);
        self.len.set(len);
        ptr as *mut ffi::c_void
    }
}

impl Drop for AggBuffer {
    fn drop(&mut self) {
        let ptr = self.ptr.get();
        if ptr.is_null() {
            return;
        }
        let layout = Layout::from_size_align(self.len.get(), AGG_ALIGN)
            .expect("the layout was already accepted when the buffer was allocated");
        // SAFETY: `ptr` came from `alloc_zeroed` with this layout.
        unsafe { dealloc(ptr, layout) };
    }
}

/// The call a C callback is running inside.
pub struct sqlite3_context {
    db: *mut sqlite3,
    p_app: *mut ffi::c_void,
    result: sqlite3_value,
    /// Cleared by every result setter, so a value set afterwards wins.
    result_subtype: u8,
    /// Cleared by any later result setter; dropping it runs the destructor.
    result_pointer: Option<(Arc<CPointer>, String)>,
    /// Once set this wins over the result, as in SQLite, which never clears
    /// its error flag either.
    error: Option<(ffi::c_int, String)>,
    agg: *const AggBuffer,
    /// Only dereferenced while the callback runs.
    core: *mut FunctionContext<'static>,
}

impl sqlite3_context {
    fn new(callbacks: &CCallbacks, agg: *const AggBuffer, core: &mut FunctionContext<'_>) -> Self {
        Self {
            db: callbacks.db,
            p_app: callbacks.p_app,
            result: sqlite3_value::new(Value::Null),
            result_subtype: 0,
            result_pointer: None,
            error: None,
            agg,
            core: std::ptr::from_mut(core).cast(),
        }
    }

    /// Drops any subtype or pointer set earlier: those come after their value.
    fn set_result(&mut self, value: Value) {
        self.result.set(value);
        self.result_subtype = 0;
        self.result_pointer = None;
    }

    fn finish(mut self, core: &mut FunctionContext<'_>) -> turso_core::Result<Value> {
        if let Some((code, message)) = self.error.take() {
            return Err(core.error_code(code, message));
        }
        if let Some((pointer, ptype)) = self.result_pointer.take() {
            core.set_result_pointer(pointer, ptype);
            return Ok(Value::Null);
        }
        if self.result_subtype != 0 {
            core.set_result_subtype(self.result_subtype);
        }
        Ok(self.result.into_value())
    }

    fn set_error(&mut self, code: ffi::c_int, message: String) {
        self.error = Some((code, message));
    }

    fn set_error_toobig(&mut self) {
        self.set_error(SQLITE_TOOBIG, "string or blob too big".to_string());
    }
}

/// Copied into each aggregate accumulator so a running group need not reach
/// back into the registration.
#[derive(Clone, Copy)]
struct CCallbacks {
    x_func: Option<XFunc>,
    x_step: Option<XFunc>,
    x_final: Option<XFinal>,
    x_value: Option<XFinal>,
    x_inverse: Option<XFunc>,
    p_app: *mut ffi::c_void,
    db: *mut sqlite3,
}

// SAFETY: `p_app` and `db` are opaque here, and the C API forbids using one
// database handle from two threads at once.
unsafe impl Send for CCallbacks {}
unsafe impl Sync for CCallbacks {}

/// A function registered through `sqlite3_create_function*`. Dropping it runs
/// `xDestroy`, the lifetime SQLite documents.
pub(crate) struct CFunction {
    callbacks: CCallbacks,
    x_destroy: Option<XDestroy>,
}

impl CFunction {
    /// What a deletion leaves behind: the deleting call's destructor runs
    /// when the entry is registered again or the connection closes, not now.
    fn destructor_only(p_app: *mut ffi::c_void, x_destroy: Option<XDestroy>) -> Self {
        Self {
            callbacks: CCallbacks {
                x_func: None,
                x_step: None,
                x_final: None,
                x_value: None,
                x_inverse: None,
                p_app,
                db: std::ptr::null_mut(),
            },
            x_destroy,
        }
    }
}

impl Drop for CFunction {
    fn drop(&mut self) {
        if let Some(x_destroy) = self.x_destroy {
            // SAFETY: the registering caller paired destructor and pointer.
            unsafe { x_destroy(self.callbacks.p_app) };
        }
    }
}

/// Run one C callback that takes arguments (`xFunc`, `xStep`, `xInverse`).
unsafe fn call_with_args(
    callbacks: &CCallbacks,
    callback: XFunc,
    core: &mut FunctionContext<'_>,
    args: &[ValueRef<'_>],
    agg: *const AggBuffer,
) -> turso_core::Result<Value> {
    let core_ptr: *mut FunctionContext<'static> = std::ptr::from_mut(core).cast();
    let mut values: Vec<sqlite3_value> = args
        .iter()
        .enumerate()
        .map(|(i, arg)| sqlite3_value::from_arg(*arg, core_ptr, i, core.arg_subtype(i)))
        .collect();
    let mut argv: Vec<*mut sqlite3_value> = values
        .iter_mut()
        .map(|value| value as *mut sqlite3_value)
        .collect();
    let mut ctx = sqlite3_context::new(callbacks, agg, core);
    callback(&mut ctx, args.len() as ffi::c_int, argv.as_mut_ptr());
    // The wrappers point back at `core`; drop them before `finish` reborrows.
    drop(argv);
    drop(values);
    ctx.finish(core)
}

/// Run one C callback that takes no arguments (`xFinal`, `xValue`).
unsafe fn call_no_args(
    callbacks: &CCallbacks,
    callback: XFinal,
    core: &mut FunctionContext<'_>,
    agg: *const AggBuffer,
) -> turso_core::Result<Value> {
    let mut ctx = sqlite3_context::new(callbacks, agg, core);
    callback(&mut ctx);
    ctx.finish(core)
}

impl ScalarFunction for CFunction {
    fn call(
        &self,
        ctx: &mut FunctionContext<'_>,
        args: &[ValueRef<'_>],
    ) -> turso_core::Result<Value> {
        let Some(x_func) = self.callbacks.x_func else {
            return Err(ctx.error("user-defined function has no implementation"));
        };
        // SAFETY: the caller of sqlite3_create_function promised this shape.
        unsafe { call_with_args(&self.callbacks, x_func, ctx, args, std::ptr::null()) }
    }
}

struct CAggState {
    callbacks: CCallbacks,
    buffer: AggBuffer,
}

// SAFETY: only the statement owning the accumulator touches the buffer.
unsafe impl Send for CAggState {}

impl AggregateFunction for CFunction {
    fn init(&self, _ctx: &mut FunctionContext<'_>) -> turso_core::Result<Box<dyn AggregateState>> {
        Ok(Box::new(CAggState {
            callbacks: self.callbacks,
            buffer: AggBuffer::new(),
        }))
    }

    fn supports_window(&self) -> bool {
        self.callbacks.x_value.is_some() && self.callbacks.x_inverse.is_some()
    }
}

impl AggregateState for CAggState {
    fn step(
        &mut self,
        ctx: &mut FunctionContext<'_>,
        args: &[ValueRef<'_>],
    ) -> turso_core::Result<()> {
        let Some(x_step) = self.callbacks.x_step else {
            return Err(ctx.error("aggregate function has no step implementation"));
        };
        // SQLite ignores a result set by xStep, but not an error.
        // SAFETY: see `CFunction::call`.
        unsafe { call_with_args(&self.callbacks, x_step, ctx, args, &self.buffer) }?;
        Ok(())
    }

    fn finalize(self: Box<Self>, ctx: &mut FunctionContext<'_>) -> turso_core::Result<Value> {
        let Some(x_final) = self.callbacks.x_final else {
            return Err(ctx.error("aggregate function has no final implementation"));
        };
        // SAFETY: see `CFunction::call`.
        unsafe { call_no_args(&self.callbacks, x_final, ctx, &self.buffer) }
    }

    fn value(&self, ctx: &mut FunctionContext<'_>) -> turso_core::Result<Value> {
        let Some(x_value) = self.callbacks.x_value else {
            return Err(ctx.error("aggregate may not be used as a window function"));
        };
        // SAFETY: see `CFunction::call`.
        unsafe { call_no_args(&self.callbacks, x_value, ctx, &self.buffer) }
    }

    fn inverse(
        &mut self,
        ctx: &mut FunctionContext<'_>,
        args: &[ValueRef<'_>],
    ) -> turso_core::Result<()> {
        let Some(x_inverse) = self.callbacks.x_inverse else {
            return Err(ctx.error("aggregate may not be used as a window function"));
        };
        // SAFETY: see `CFunction::call`.
        unsafe { call_with_args(&self.callbacks, x_inverse, ctx, args, &self.buffer) }?;
        Ok(())
    }
}

/// `None` when the name is missing or not UTF-8, which SQLite would accept.
unsafe fn function_name(name: *const ffi::c_char) -> Option<String> {
    if name.is_null() {
        return None;
    }
    CStr::from_ptr(name).to_str().ok().map(str::to_owned)
}

/// Decodes a UTF-16 name in the platform's byte order.
unsafe fn function_name16(name: *const ffi::c_void) -> Option<String> {
    if name.is_null() {
        return None;
    }
    let units = name as *const u16;
    let mut len = 0usize;
    while *units.add(len) != 0 {
        len += 1;
    }
    String::from_utf16(std::slice::from_raw_parts(units, len)).ok()
}

/// Shared body of the `sqlite3_create_function` family.
#[allow(clippy::too_many_arguments)]
unsafe fn create_function(
    db: *mut sqlite3,
    name: Option<String>,
    n_args: ffi::c_int,
    enc: ffi::c_int,
    p_app: *mut ffi::c_void,
    x_func: Option<XFunc>,
    x_step: Option<XFunc>,
    x_final: Option<XFinal>,
    x_value: Option<XFinal>,
    x_inverse: Option<XFunc>,
    x_destroy: Option<XDestroy>,
) -> ffi::c_int {
    // Nothing took ownership of `p_app`, so destroy it here, as SQLite does.
    let unregistered = |code: ffi::c_int| -> ffi::c_int {
        if let Some(x_destroy) = x_destroy {
            x_destroy(p_app);
        }
        code
    };

    let Some(name) = name else {
        return unregistered(SQLITE_MISUSE);
    };
    if db.is_null()
        || (x_func.is_some() && x_final.is_some())
        || (x_final.is_none() != x_step.is_none())
        || (x_value.is_none() != x_inverse.is_none())
        || !(-1..=MAX_FUNCTION_ARG).contains(&n_args)
        || name.len() > MAX_FUNCTION_NAME_LEN
        || name.is_empty()
    {
        return unregistered(SQLITE_MISUSE);
    }

    let db_ref = &*db;
    let mut inner = db_ref.inner.lock().unwrap();

    // A definition may not move under a running statement, but a (name,
    // argc) nobody registered yet is new and may be added even then.
    if inner.knows_function(&name, n_args) && crate::has_running_statement(&inner) {
        crate::set_db_err_msg(
            &mut inner,
            SQLITE_BUSY,
            "unable to delete/modify user-function due to active statements",
        );
        drop(inner);
        return unregistered(SQLITE_BUSY);
    }

    // Any tombstone here dies now; a live registration dies when core drops it.
    let had_entry = inner.knows_function(&name, n_args);
    let had_tombstone = inner.take_deleted_function(&name, n_args).is_some();

    // SQLite deletes the registration when every callback is NULL.
    if x_func.is_none() && x_final.is_none() {
        if !had_entry && !had_tombstone {
            drop(inner);
            return unregistered(SQLITE_OK);
        }
        let _ = inner.conn.remove_function(&name, n_args);
        inner.forget_function(&name, n_args);
        if x_destroy.is_some() {
            inner.remember_deleted_function(
                &name,
                n_args,
                CFunction::destructor_only(p_app, x_destroy),
            );
        }
        return SQLITE_OK;
    }

    let function = CFunction {
        callbacks: CCallbacks {
            x_func,
            x_step,
            x_final,
            x_value,
            x_inverse,
            p_app,
            db,
        },
        x_destroy,
    };
    let flags = FunctionFlags::from_bits_truncate(enc as u32);

    let registered = if x_func.is_some() {
        inner
            .conn
            .create_scalar_function(&name, n_args, flags, function)
    } else {
        inner
            .conn
            .create_aggregate_function(&name, n_args, flags, function)
    };
    match registered {
        // A failing call dropped `function`, so its xDestroy already ran.
        Err(LimboError::InvalidArgument(_)) => SQLITE_MISUSE,
        Err(_) => SQLITE_ERROR,
        Ok(()) => {
            inner.remember_function(&name, n_args);
            SQLITE_OK
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_create_function(
    db: *mut sqlite3,
    name: *const ffi::c_char,
    n_args: ffi::c_int,
    enc: ffi::c_int,
    p_app: *mut ffi::c_void,
    x_func: Option<XFunc>,
    x_step: Option<XFunc>,
    x_final: Option<XFinal>,
) -> ffi::c_int {
    create_function(
        db,
        function_name(name),
        n_args,
        enc,
        p_app,
        x_func,
        x_step,
        x_final,
        None,
        None,
        None,
    )
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_create_function_v2(
    db: *mut sqlite3,
    name: *const ffi::c_char,
    n_args: ffi::c_int,
    enc: ffi::c_int,
    p_app: *mut ffi::c_void,
    x_func: Option<XFunc>,
    x_step: Option<XFunc>,
    x_final: Option<XFinal>,
    x_destroy: Option<XDestroy>,
) -> ffi::c_int {
    create_function(
        db,
        function_name(name),
        n_args,
        enc,
        p_app,
        x_func,
        x_step,
        x_final,
        None,
        None,
        x_destroy,
    )
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_create_function16(
    db: *mut sqlite3,
    name: *const ffi::c_void,
    n_args: ffi::c_int,
    enc: ffi::c_int,
    p_app: *mut ffi::c_void,
    x_func: Option<XFunc>,
    x_step: Option<XFunc>,
    x_final: Option<XFinal>,
) -> ffi::c_int {
    create_function(
        db,
        function_name16(name),
        n_args,
        enc,
        p_app,
        x_func,
        x_step,
        x_final,
        None,
        None,
        None,
    )
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_create_window_function(
    db: *mut sqlite3,
    name: *const ffi::c_char,
    n_args: ffi::c_int,
    enc: ffi::c_int,
    p_app: *mut ffi::c_void,
    x_step: Option<XFunc>,
    x_final: Option<XFinal>,
    x_value: Option<XFinal>,
    x_inverse: Option<XFunc>,
    x_destroy: Option<XDestroy>,
) -> ffi::c_int {
    create_function(
        db,
        function_name(name),
        n_args,
        enc,
        p_app,
        None,
        x_step,
        x_final,
        x_value,
        x_inverse,
        x_destroy,
    )
}

/// Placeholder registered so a name parses; calling it is an error.
unsafe extern "C" fn invalid_function(
    ctx: *mut sqlite3_context,
    _argc: ffi::c_int,
    _argv: *mut *mut sqlite3_value,
) {
    let name = sqlite3_user_data(ctx) as *const ffi::c_char;
    let name = if name.is_null() {
        String::new()
    } else {
        CStr::from_ptr(name).to_string_lossy().into_owned()
    };
    let message = format!("unable to use function {name} in the requested context");
    sqlite3_result_error(
        ctx,
        message.as_ptr() as *const ffi::c_char,
        message.len() as ffi::c_int,
    );
}

/// Free the name `sqlite3_overload_function` leaked into `pApp`.
unsafe extern "C" fn free_overload_name(p_app: *mut ffi::c_void) {
    if !p_app.is_null() {
        drop(std::ffi::CString::from_raw(p_app as *mut ffi::c_char));
    }
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_overload_function(
    db: *mut sqlite3,
    name: *const ffi::c_char,
    n_args: ffi::c_int,
) -> ffi::c_int {
    if db.is_null() || n_args < -2 {
        return SQLITE_MISUSE;
    }
    let Some(name) = function_name(name) else {
        return SQLITE_MISUSE;
    };
    {
        let db_ref = &*db;
        let inner = db_ref.inner.lock().unwrap();
        if inner.knows_function(&name, n_args) {
            return SQLITE_OK;
        }
    }
    // A built-in already lets the SQL parse, and this would hide it.
    if n_args >= 0
        && SqliteDialect
            .resolve_function(&name, n_args as usize)
            .ok()
            .flatten()
            .is_some()
    {
        return SQLITE_OK;
    }
    let Ok(owned_name) = std::ffi::CString::new(name.as_str()) else {
        return SQLITE_MISUSE;
    };
    create_function(
        db,
        Some(name),
        n_args,
        SQLITE_UTF8,
        owned_name.into_raw() as *mut ffi::c_void,
        Some(invalid_function),
        None,
        None,
        None,
        None,
        Some(free_overload_name),
    )
}

/// Drops every function this handle registered, running each `xDestroy`.
pub(crate) unsafe fn drop_all_functions(inner: &mut crate::sqlite3Inner) {
    for (name, argc) in std::mem::take(&mut inner.functions) {
        let _ = inner.conn.remove_function(&name, argc);
    }
    inner.deleted_functions.clear();
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_user_data(ctx: *mut sqlite3_context) -> *mut ffi::c_void {
    if ctx.is_null() {
        return std::ptr::null_mut();
    }
    (*ctx).p_app
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_context_db_handle(ctx: *mut sqlite3_context) -> *mut sqlite3 {
    if ctx.is_null() {
        return std::ptr::null_mut();
    }
    (*ctx).db
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_aggregate_context(
    ctx: *mut sqlite3_context,
    n_bytes: ffi::c_int,
) -> *mut ffi::c_void {
    if ctx.is_null() || (*ctx).agg.is_null() {
        return std::ptr::null_mut();
    }
    (*(*ctx).agg).get(n_bytes)
}

struct CAuxData {
    ptr: *mut ffi::c_void,
    destroy: Option<XDestroy>,
}

// SAFETY: opaque here, and only handed back to the same connection.
unsafe impl Send for CAuxData {}
unsafe impl Sync for CAuxData {}

impl Drop for CAuxData {
    fn drop(&mut self) {
        if let Some(destroy) = self.destroy {
            // SAFETY: the caller of sqlite3_set_auxdata paired them.
            unsafe { destroy(self.ptr) };
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_get_auxdata(
    ctx: *mut sqlite3_context,
    arg: ffi::c_int,
) -> *mut ffi::c_void {
    if ctx.is_null() || arg < 0 || (*ctx).core.is_null() {
        return std::ptr::null_mut();
    }
    let core = &*(*ctx).core;
    match core
        .get_auxdata(arg as usize)
        .and_then(|data| data.downcast_ref::<CAuxData>())
    {
        Some(aux) => aux.ptr,
        None => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_set_auxdata(
    ctx: *mut sqlite3_context,
    arg: ffi::c_int,
    data: *mut ffi::c_void,
    destroy: Option<XDestroy>,
) {
    if ctx.is_null() || arg < 0 || (*ctx).core.is_null() {
        // Nowhere to store it, so the data would leak: destroy it now.
        if let Some(destroy) = destroy {
            destroy(data);
        }
        return;
    }
    let core = &mut *(*ctx).core;
    core.set_auxdata(arg as usize, Box::new(CAuxData { ptr: data, destroy }));
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_value_type(value: *mut sqlite3_value) -> ffi::c_int {
    if value.is_null() {
        return SQLITE_NULL;
    }
    (*value).type_code()
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_value_numeric_type(value: *mut sqlite3_value) -> ffi::c_int {
    if value.is_null() {
        return SQLITE_NULL;
    }
    let value = &mut *value;
    if let Value::Text(text) = &value.value {
        if let Some(numeric) = numeric_affinity(text.as_str()) {
            value.set(numeric);
        }
    }
    value.type_code()
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_value_int64(value: *mut sqlite3_value) -> i64 {
    if value.is_null() {
        return 0;
    }
    (*value).int64()
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_value_int(value: *mut sqlite3_value) -> ffi::c_int {
    sqlite3_value_int64(value) as ffi::c_int
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_value_double(value: *mut sqlite3_value) -> f64 {
    if value.is_null() {
        return 0.0;
    }
    (*value).double()
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_value_text(value: *mut sqlite3_value) -> *const ffi::c_uchar {
    if value.is_null() {
        return std::ptr::null();
    }
    match (*value).text_bytes() {
        Some(text) => text.as_ptr() as *const ffi::c_uchar,
        None => std::ptr::null(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_value_text16(value: *mut sqlite3_value) -> *const ffi::c_void {
    if cfg!(target_endian = "big") {
        sqlite3_value_text16be(value)
    } else {
        sqlite3_value_text16le(value)
    }
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_value_text16le(value: *mut sqlite3_value) -> *const ffi::c_void {
    if value.is_null() {
        return std::ptr::null();
    }
    match (*value).text16_bytes(false) {
        Some(text) => text.as_ptr() as *const ffi::c_void,
        None => std::ptr::null(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_value_text16be(value: *mut sqlite3_value) -> *const ffi::c_void {
    if value.is_null() {
        return std::ptr::null();
    }
    match (*value).text16_bytes(true) {
        Some(text) => text.as_ptr() as *const ffi::c_void,
        None => std::ptr::null(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_value_blob(value: *mut sqlite3_value) -> *const ffi::c_void {
    if value.is_null() {
        return std::ptr::null();
    }
    match (*value).text_bytes() {
        // SQLite returns NULL for a zero-length blob too.
        Some(bytes) if bytes.len() > 1 => bytes.as_ptr() as *const ffi::c_void,
        _ => std::ptr::null(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_value_bytes(value: *mut sqlite3_value) -> ffi::c_int {
    if value.is_null() {
        return 0;
    }
    match (*value).text_bytes() {
        Some(bytes) => (bytes.len() - 1) as ffi::c_int,
        None => 0,
    }
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_value_bytes16(value: *mut sqlite3_value) -> ffi::c_int {
    if value.is_null() {
        return 0;
    }
    match (*value).text16_bytes(cfg!(target_endian = "big")) {
        Some(bytes) => (bytes.len() - 2) as ffi::c_int,
        None => 0,
    }
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_value_subtype(value: *mut sqlite3_value) -> ffi::c_uint {
    if value.is_null() {
        return 0;
    }
    (*value).subtype as ffi::c_uint
}

/// Always SQLITE_UTF8: Turso stores text as UTF-8 and converts on the way out.
#[no_mangle]
pub unsafe extern "C" fn sqlite3_value_encoding(_value: *mut sqlite3_value) -> ffi::c_int {
    SQLITE_UTF8
}

/// Always 0: only a virtual table's `xUpdate` reports otherwise.
#[no_mangle]
pub unsafe extern "C" fn sqlite3_value_nochange(_value: *mut sqlite3_value) -> ffi::c_int {
    0
}

/// Always 0: Turso does not mark the registers a bound parameter filled.
#[no_mangle]
pub unsafe extern "C" fn sqlite3_value_frombind(value: *mut sqlite3_value) -> ffi::c_int {
    if value.is_null() {
        return 0;
    }
    ffi::c_int::from((*value).from_bind)
}

/// The pointer this value carries, if the type name matches exactly. Only a
/// live argument can carry one: the object rides beside the register.
#[no_mangle]
pub unsafe extern "C" fn sqlite3_value_pointer(
    value: *mut sqlite3_value,
    type_name: *const ffi::c_char,
) -> *mut ffi::c_void {
    if value.is_null() || type_name.is_null() {
        return std::ptr::null_mut();
    }
    let value = &*value;
    if value.core.is_null() {
        return std::ptr::null_mut();
    }
    let core = &*value.core;
    let type_name = CStr::from_ptr(type_name).to_string_lossy();
    match core.arg_pointer(value.index, &type_name) {
        Some(object) => object
            .downcast_ref::<CPointer>()
            .map_or(std::ptr::null_mut(), |pointer| pointer.ptr),
        None => std::ptr::null_mut(),
    }
}

/// The copy outlives the call and must be released with `sqlite3_value_free`.
#[no_mangle]
pub unsafe extern "C" fn sqlite3_value_dup(value: *const sqlite3_value) -> *mut sqlite3_value {
    if value.is_null() {
        return std::ptr::null_mut();
    }
    let source = &*value;
    let mut copy = sqlite3_value::new(source.value.clone());
    copy.subtype = source.subtype;
    copy.from_bind = source.from_bind;
    // No back-pointer to the call, so a pointer value does not survive this.
    Box::into_raw(Box::new(copy))
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_value_free(value: *mut sqlite3_value) {
    if value.is_null() {
        return;
    }
    drop(Box::from_raw(value));
}

/// SQLite reserves two `destructor` values: 0 (STATIC) and `(void*)-1`
/// (TRANSIENT). Turso copies either way, so only the destructor differs.
unsafe fn run_destructor(destructor: *mut ffi::c_void, data: *const ffi::c_void) {
    const TRANSIENT: isize = -1;
    if destructor.is_null() || destructor as isize == TRANSIENT {
        return;
    }
    let destructor = std::mem::transmute::<*mut ffi::c_void, XDestroy>(destructor);
    destructor(data as *mut ffi::c_void);
}

/// Read `n` bytes of a text argument, or up to the NUL when `n` is negative.
unsafe fn text_arg<'a>(text: *const ffi::c_char, n: i64) -> &'a [u8] {
    if n < 0 {
        CStr::from_ptr(text).to_bytes()
    } else {
        std::slice::from_raw_parts(text as *const u8, n as usize)
    }
}

/// Reads `n` UTF-16 code units, or up to the NUL unit when `n` is negative.
unsafe fn text16_arg<'a>(text: *const ffi::c_void, n: i64) -> &'a [u8] {
    if n >= 0 {
        return std::slice::from_raw_parts(text as *const u8, n as usize);
    }
    let units = text as *const u16;
    let mut len = 0usize;
    while *units.add(len) != 0 {
        len += 1;
    }
    std::slice::from_raw_parts(text as *const u8, len * 2)
}

fn decode_utf16(bytes: &[u8], big_endian: bool) -> String {
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|pair| {
            if big_endian {
                u16::from_be_bytes([pair[0], pair[1]])
            } else {
                u16::from_le_bytes([pair[0], pair[1]])
            }
        })
        .collect();
    String::from_utf16_lossy(&units)
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_result_null(ctx: *mut sqlite3_context) {
    if ctx.is_null() {
        return;
    }
    (*ctx).set_result(Value::Null);
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_result_int64(ctx: *mut sqlite3_context, val: i64) {
    if ctx.is_null() {
        return;
    }
    (*ctx).set_result(Value::from_i64(val));
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_result_int(ctx: *mut sqlite3_context, val: ffi::c_int) {
    sqlite3_result_int64(ctx, val as i64);
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_result_double(ctx: *mut sqlite3_context, val: f64) {
    if ctx.is_null() {
        return;
    }
    (*ctx).set_result(Value::from_f64(val));
}

unsafe fn set_text_result(ctx: *mut sqlite3_context, bytes: &[u8]) {
    let ctx = &mut *ctx;
    if bytes.len() as u64 > MAX_LENGTH {
        ctx.set_error_toobig();
        return;
    }
    // Divergence: invalid UTF-8 is replaced, where SQLite keeps it verbatim.
    ctx.set_result(Value::build_text(
        String::from_utf8_lossy(bytes).into_owned(),
    ));
}

unsafe fn set_blob_result(ctx: *mut sqlite3_context, bytes: &[u8]) {
    let ctx = &mut *ctx;
    if bytes.len() as u64 > MAX_LENGTH {
        ctx.set_error_toobig();
        return;
    }
    match Value::from_slice(bytes) {
        Ok(value) => ctx.set_result(value),
        Err(_) => ctx.set_error(SQLITE_NOMEM, "out of memory".to_string()),
    }
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_result_text(
    ctx: *mut sqlite3_context,
    text: *const ffi::c_char,
    n: ffi::c_int,
    destructor: *mut ffi::c_void,
) {
    if ctx.is_null() {
        return;
    }
    if text.is_null() {
        (*ctx).set_result(Value::Null);
        return;
    }
    set_text_result(ctx, text_arg(text, n as i64));
    run_destructor(destructor, text as *const ffi::c_void);
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_result_text64(
    ctx: *mut sqlite3_context,
    text: *const ffi::c_char,
    n: u64,
    destructor: *mut ffi::c_void,
    encoding: ffi::c_uchar,
) {
    if ctx.is_null() {
        return;
    }
    if text.is_null() {
        (*ctx).set_result(Value::Null);
        return;
    }
    if n > MAX_LENGTH {
        (*ctx).set_error_toobig();
        run_destructor(destructor, text as *const ffi::c_void);
        return;
    }
    let bytes = std::slice::from_raw_parts(text as *const u8, n as usize);
    match encoding as ffi::c_int {
        SQLITE_UTF16LE => set_text_result_utf16(ctx, bytes, false),
        SQLITE_UTF16BE => set_text_result_utf16(ctx, bytes, true),
        SQLITE_UTF16 => set_text_result_utf16(ctx, bytes, cfg!(target_endian = "big")),
        _ => set_text_result(ctx, bytes),
    }
    run_destructor(destructor, text as *const ffi::c_void);
}

unsafe fn set_text_result_utf16(ctx: *mut sqlite3_context, bytes: &[u8], big_endian: bool) {
    let text = decode_utf16(bytes, big_endian);
    set_text_result(ctx, text.as_bytes());
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_result_text16(
    ctx: *mut sqlite3_context,
    text: *const ffi::c_void,
    n: ffi::c_int,
    destructor: *mut ffi::c_void,
) {
    if cfg!(target_endian = "big") {
        sqlite3_result_text16be(ctx, text, n, destructor)
    } else {
        sqlite3_result_text16le(ctx, text, n, destructor)
    }
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_result_text16le(
    ctx: *mut sqlite3_context,
    text: *const ffi::c_void,
    n: ffi::c_int,
    destructor: *mut ffi::c_void,
) {
    if ctx.is_null() {
        return;
    }
    if text.is_null() {
        (*ctx).set_result(Value::Null);
        return;
    }
    set_text_result_utf16(ctx, text16_arg(text, n as i64), false);
    run_destructor(destructor, text);
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_result_text16be(
    ctx: *mut sqlite3_context,
    text: *const ffi::c_void,
    n: ffi::c_int,
    destructor: *mut ffi::c_void,
) {
    if ctx.is_null() {
        return;
    }
    if text.is_null() {
        (*ctx).set_result(Value::Null);
        return;
    }
    set_text_result_utf16(ctx, text16_arg(text, n as i64), true);
    run_destructor(destructor, text);
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_result_blob(
    ctx: *mut sqlite3_context,
    blob: *const ffi::c_void,
    n: ffi::c_int,
    destructor: *mut ffi::c_void,
) {
    if ctx.is_null() {
        return;
    }
    if blob.is_null() || n < 0 {
        (*ctx).set_result(Value::Null);
        return;
    }
    set_blob_result(
        ctx,
        std::slice::from_raw_parts(blob as *const u8, n as usize),
    );
    run_destructor(destructor, blob);
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_result_blob64(
    ctx: *mut sqlite3_context,
    blob: *const ffi::c_void,
    n: u64,
    destructor: *mut ffi::c_void,
) {
    if ctx.is_null() {
        return;
    }
    if blob.is_null() {
        (*ctx).set_result(Value::Null);
        return;
    }
    if n > MAX_LENGTH {
        (*ctx).set_error_toobig();
    } else {
        set_blob_result(
            ctx,
            std::slice::from_raw_parts(blob as *const u8, n as usize),
        );
    }
    run_destructor(destructor, blob);
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_result_zeroblob(ctx: *mut sqlite3_context, n: ffi::c_int) {
    sqlite3_result_zeroblob64(ctx, if n > 0 { n as u64 } else { 0 });
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_result_zeroblob64(
    ctx: *mut sqlite3_context,
    n: u64,
) -> ffi::c_int {
    if ctx.is_null() {
        return SQLITE_MISUSE;
    }
    if n > MAX_LENGTH {
        (*ctx).set_error_toobig();
        return SQLITE_TOOBIG;
    }
    // Turso has no lazily-expanded zero blob, so it is materialized here.
    set_blob_result(ctx, &vec![0u8; n as usize]);
    SQLITE_OK
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_result_value(
    ctx: *mut sqlite3_context,
    value: *mut sqlite3_value,
) {
    if ctx.is_null() {
        return;
    }
    let ctx = &mut *ctx;
    let Some(value) = value.as_ref() else {
        ctx.set_result(Value::Null);
        return;
    };
    ctx.set_result(value.value.clone());
    // Subtype travels with the value, and the pointer too for a live argument.
    ctx.result_subtype = value.subtype;
    if !value.core.is_null() {
        ctx.result_pointer = pointer_of_arg(&*value.core, value.index);
    }
}

unsafe fn pointer_of_arg(
    core: &FunctionContext<'_>,
    index: usize,
) -> Option<(Arc<CPointer>, String)> {
    let tag = core.arg_tag(index)?;
    let pointer = tag.pointer()?;
    let ptype = pointer.ptype().to_string();
    let object = pointer.get(&ptype)?;
    let pointer = Arc::downcast::<CPointer>(object).ok()?;
    Some((pointer, ptype))
}

/// A later `sqlite3_result_*` clears the subtype, so this must come after the
/// value it belongs to, the order SQLite requires.
#[no_mangle]
pub unsafe extern "C" fn sqlite3_result_subtype(ctx: *mut sqlite3_context, subtype: ffi::c_uint) {
    if ctx.is_null() {
        return;
    }
    (*ctx).result_subtype = (subtype & 0xff) as u8;
}

/// Produces a NULL only a function asking for this exact type name can
/// unwrap. `destructor` runs when the last holder of the value gives it up.
#[no_mangle]
pub unsafe extern "C" fn sqlite3_result_pointer(
    ctx: *mut sqlite3_context,
    ptr: *mut ffi::c_void,
    type_name: *const ffi::c_char,
    destructor: Option<XDestroy>,
) {
    let Some(ctx) = ctx.as_mut() else {
        // Nowhere to keep it and nobody to read it, so it would leak.
        if let Some(destructor) = destructor {
            destructor(ptr);
        }
        return;
    };
    ctx.set_result(Value::Null);
    ctx.result_pointer = Some((
        Arc::new(CPointer {
            ptr,
            destroy: destructor,
        }),
        pointer_type_name(type_name),
    ));
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_result_error(
    ctx: *mut sqlite3_context,
    err: *const ffi::c_char,
    n: ffi::c_int,
) {
    if ctx.is_null() {
        return;
    }
    let message = if err.is_null() {
        String::new()
    } else {
        String::from_utf8_lossy(text_arg(err, n as i64)).into_owned()
    };
    (*ctx).set_error(SQLITE_ERROR, message);
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_result_error16(
    ctx: *mut sqlite3_context,
    err: *const ffi::c_void,
    n: ffi::c_int,
) {
    if ctx.is_null() {
        return;
    }
    let message = if err.is_null() {
        String::new()
    } else {
        decode_utf16(text16_arg(err, n as i64), cfg!(target_endian = "big"))
    };
    (*ctx).set_error(SQLITE_ERROR, message);
}

/// Changes the failure code, keeping any message already set.
#[no_mangle]
pub unsafe extern "C" fn sqlite3_result_error_code(ctx: *mut sqlite3_context, code: ffi::c_int) {
    if ctx.is_null() {
        return;
    }
    let ctx = &mut *ctx;
    // Like SQLite, a NULL result becomes sqlite3ErrStr(code).
    let message = match ctx.error.take() {
        Some((_, message)) => message,
        None => match ctx.result.text_bytes() {
            Some(bytes) => String::from_utf8_lossy(&bytes[..bytes.len() - 1]).into_owned(),
            None => CStr::from_ptr(crate::sqlite3_errstr(code))
                .to_string_lossy()
                .into_owned(),
        },
    };
    ctx.set_error(code, message);
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_result_error_nomem(ctx: *mut sqlite3_context) {
    if ctx.is_null() {
        return;
    }
    (*ctx).set_error(SQLITE_NOMEM, "out of memory".to_string());
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_result_error_toobig(ctx: *mut sqlite3_context) {
    if ctx.is_null() {
        return;
    }
    (*ctx).set_error_toobig();
}

/// One raw pointer from `sqlite3_result_pointer` or `sqlite3_bind_pointer`.
/// The engine holds it beside the value as an `Arc`, so the destructor runs
/// exactly once, when the last holder gives it up.
pub(crate) struct CPointer {
    pub(crate) ptr: *mut ffi::c_void,
    pub(crate) destroy: Option<XDestroy>,
}

// SAFETY: opaque here, only handed back to the caller's own callbacks. As in
// SQLite, the destructor runs on whichever thread lets go of the value last.
unsafe impl Send for CPointer {}
unsafe impl Sync for CPointer {}

impl Drop for CPointer {
    fn drop(&mut self) {
        if let Some(destroy) = self.destroy {
            // SAFETY: the registering caller paired destructor and pointer.
            unsafe { destroy(self.ptr) };
        }
    }
}

/// A NULL type name becomes the empty string, matching nothing.
pub(crate) unsafe fn pointer_type_name(type_name: *const ffi::c_char) -> String {
    if type_name.is_null() {
        return String::new();
    }
    CStr::from_ptr(type_name).to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_affinity_converts_only_complete_numbers() {
        assert!(matches!(
            numeric_affinity("12"),
            Some(Value::Numeric(Numeric::Integer(12)))
        ));
        assert!(matches!(
            numeric_affinity(" -3 "),
            Some(Value::Numeric(Numeric::Integer(-3)))
        ));
        assert!(matches!(
            numeric_affinity("1.5"),
            Some(Value::Numeric(Numeric::Float(_)))
        ));
        assert!(matches!(
            numeric_affinity("1e3"),
            Some(Value::Numeric(Numeric::Float(_)))
        ));
        assert!(matches!(
            numeric_affinity("5."),
            Some(Value::Numeric(Numeric::Float(_)))
        ));
        assert!(numeric_affinity("12abc").is_none());
        assert!(numeric_affinity("abc").is_none());
        assert!(numeric_affinity("").is_none());
        assert!(numeric_affinity("+").is_none());
        assert!(numeric_affinity("1e").is_none());
    }
}
