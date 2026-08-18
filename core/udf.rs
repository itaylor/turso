//! User-defined functions: the traits the engine calls and the connection-level registration API.

use std::any::Any;
use std::borrow::Cow;
use std::fmt::{self, Debug, Display};
use std::marker::PhantomData;
// Pointer values hold caller-built objects, so they use the real std Arc, not the shuttle-swappable crate::sync::Arc.
use std::sync::Arc as StdArc;

use bitflags::bitflags;

use crate::function::Deterministic;
use crate::sync::Arc;
use crate::types::{Value, ValueRef};
use crate::{Connection, LimboError, Result};

bitflags! {
    /// Bit values match SQLITE_DETERMINISTIC and friends so the C layer passes them straight through.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct FunctionFlags: u32 {
        const DETERMINISTIC = 0x0000_0800;
        const DIRECTONLY = 0x0008_0000;
        const SUBTYPE = 0x0010_0000;
        const INNOCUOUS = 0x0020_0000;
        const RESULT_SUBTYPE = 0x0100_0000;
    }
}

/// SQLite's default `SQLITE_MAX_FUNCTION_ARG`.
pub const MAX_FUNCTION_ARG: i32 = 127;

pub const MAX_FUNCTION_NAME_LEN: usize = 255;

struct AuxDataEntry {
    pc: u32,
    arg: u32,
    data: Box<dyn Any + Send + Sync>,
}

#[derive(Default)]
pub(crate) struct AuxDataStore {
    entries: Vec<AuxDataEntry>,
}

impl AuxDataStore {
    fn get(&self, pc: u32, arg: u32) -> Option<&(dyn Any + Send + Sync)> {
        self.entries
            .iter()
            .find(|entry| entry.pc == pc && entry.arg == arg)
            .map(|entry| &*entry.data)
    }

    fn set(&mut self, pc: u32, arg: u32, data: Box<dyn Any + Send + Sync>) {
        match self
            .entries
            .iter_mut()
            .find(|entry| entry.pc == pc && entry.arg == arg)
        {
            Some(entry) => entry.data = data,
            None => self.entries.push(AuxDataEntry { pc, arg, data }),
        }
    }

    /// SQLite keeps aux data across rows only for constant arguments (sqlite3VdbeDeleteAuxData).
    /// The mask covers the first 32 arguments; anything past that is always dropped.
    pub(crate) fn drop_non_constant(&mut self, pc: u32, constant_mask: i32) {
        self.entries.retain(|entry| {
            entry.pc != pc || (entry.arg < 32 && constant_mask & (1 << entry.arg) != 0)
        });
    }

    pub(crate) fn clear(&mut self) {
        self.entries.clear();
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

struct AuxDataSlot<'a> {
    store: &'a mut AuxDataStore,
    pc: u32,
}

/// The subtype `sqlite3_result_pointer` attaches: ASCII `p`.
pub const POINTER_SUBTYPE: u8 = b'p';

/// `sqlite3_result_pointer` / `sqlite3_value_pointer`: a NULL that only a function asking for the
/// same type name can unwrap.
pub struct PointerValue {
    ptype: Cow<'static, str>,
    object: StdArc<dyn Any + Send + Sync>,
}

impl PointerValue {
    pub fn new(object: StdArc<dyn Any + Send + Sync>, ptype: impl Into<Cow<'static, str>>) -> Self {
        Self {
            ptype: ptype.into(),
            object,
        }
    }

    pub fn ptype(&self) -> &str {
        &self.ptype
    }

    /// Names are compared with strcmp semantics: case-sensitive, no subtyping.
    pub fn get(&self, ptype: &str) -> Option<StdArc<dyn Any + Send + Sync>> {
        (self.ptype == ptype).then(|| self.object.clone())
    }
}

impl Debug for PointerValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pointer({})", self.ptype)
    }
}

impl PartialEq for PointerValue {
    fn eq(&self, other: &Self) -> bool {
        self.ptype == other.ptype && StdArc::ptr_eq(&self.object, &other.object)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
/// Subtype byte and optional pointer a function attached to its result. It lives beside the value
/// in the register (`Register::Tagged`); any write that is not a function result drops it, which is
/// how SQLite keeps subtypes and pointers transient.
pub struct ValueTag {
    subtype: u8,
    pointer: Option<StdArc<PointerValue>>,
}

impl ValueTag {
    pub fn from_subtype(subtype: u8) -> Self {
        Self {
            subtype,
            pointer: None,
        }
    }

    pub fn from_pointer(pointer: StdArc<PointerValue>) -> Self {
        Self {
            subtype: POINTER_SUBTYPE,
            pointer: Some(pointer),
        }
    }

    pub fn subtype(&self) -> u8 {
        self.subtype
    }

    pub fn pointer(&self) -> Option<&StdArc<PointerValue>> {
        self.pointer.as_ref()
    }

    pub fn is_pointer(&self) -> bool {
        self.pointer.is_some()
    }
}

/// What a running user-defined function can reach.
pub struct FunctionContext<'a> {
    connection: &'a Arc<Connection>,
    args: &'a [ValueRef<'a>],
    arg_tags: &'a [Option<&'a ValueTag>],
    /// `None` everywhere except a scalar call from `Insn::Function`.
    aux: Option<AuxDataSlot<'a>>,
    result_tag: Option<ValueTag>,
}

impl<'a> FunctionContext<'a> {
    pub(crate) fn new(connection: &'a Arc<Connection>) -> Self {
        Self {
            connection,
            args: &[],
            arg_tags: &[],
            aux: None,
            result_tag: None,
        }
    }

    pub(crate) fn with_args(mut self, args: &'a [ValueRef<'a>]) -> Self {
        self.args = args;
        self
    }

    pub(crate) fn with_arg_tags(mut self, tags: &'a [Option<&'a ValueTag>]) -> Self {
        self.arg_tags = tags;
        self
    }

    pub(crate) fn with_auxdata(mut self, store: &'a mut AuxDataStore, pc: u32) -> Self {
        self.aux = Some(AuxDataSlot { store, pc });
        self
    }

    pub(crate) fn take_result_tag(&mut self) -> Option<ValueTag> {
        self.result_tag.take()
    }

    pub fn connection(&self) -> &Arc<Connection> {
        self.connection
    }

    /// `sqlite3_value_subtype`: 0 unless a function tagged this argument (or it is TEXT carrying an
    /// inline subtype). Subtypes do not survive a record write or a co-routine boundary.
    pub fn arg_subtype(&self, i: usize) -> u8 {
        match self.arg_tags.get(i).copied().flatten() {
            Some(tag) => tag.subtype(),
            None => self.args.get(i).map_or(0, |arg| arg.subtype()),
        }
    }

    pub fn arg_tag(&self, i: usize) -> Option<ValueTag> {
        self.arg_tags.get(i).copied().flatten().cloned()
    }

    /// `sqlite3_value_pointer`.
    pub fn arg_pointer(&self, i: usize, ptype: &str) -> Option<StdArc<dyn Any + Send + Sync>> {
        self.arg_tags
            .get(i)
            .copied()
            .flatten()?
            .pointer()?
            .get(ptype)
    }

    pub fn set_result_subtype(&mut self, subtype: u8) {
        self.result_tag = Some(ValueTag::from_subtype(subtype));
    }

    /// `sqlite3_result_pointer`: the returned value is discarded and the result is NULL.
    pub fn set_result_pointer(
        &mut self,
        object: StdArc<dyn Any + Send + Sync>,
        ptype: impl Into<Cow<'static, str>>,
    ) {
        self.result_tag = Some(ValueTag::from_pointer(StdArc::new(PointerValue::new(
            object, ptype,
        ))));
    }

    /// `sqlite3_get_auxdata`; see `AuxDataStore::drop_non_constant` for what survives between rows.
    pub fn get_auxdata(&self, i: usize) -> Option<&(dyn Any + Send + Sync)> {
        let aux = self.aux.as_ref()?;
        aux.store.get(aux.pc, i as u32)
    }

    pub fn set_auxdata(&mut self, i: usize, data: Box<dyn Any + Send + Sync>) {
        if let Some(aux) = self.aux.as_mut() {
            aux.store.set(aux.pc, i as u32, data);
        }
    }

    pub fn error(&self, message: impl Into<String>) -> LimboError {
        self.error_code(SQLITE_ERROR, message)
    }

    /// `sqlite3_result_error_code`: the code reaches `sqlite3_step`, the message `sqlite3_errmsg`.
    pub fn error_code(&self, code: i32, message: impl Into<String>) -> LimboError {
        LimboError::UserFunction {
            code,
            message: message.into(),
        }
    }

    pub fn error_toobig(&self) -> LimboError {
        self.error_code(SQLITE_TOOBIG, "string or blob too big")
    }

    pub fn error_nomem(&self) -> LimboError {
        self.error_code(SQLITE_NOMEM, "out of memory")
    }
}

const SQLITE_ERROR: i32 = 1;
const SQLITE_NOMEM: i32 = 7;
const SQLITE_TOOBIG: i32 = 18;

/// A function that produces one value per call.
pub trait ScalarFunction: Send + Sync {
    fn call(&self, ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]) -> Result<Value>;
}

/// A function that accumulates over many rows.
pub trait AggregateFunction: Send + Sync {
    fn init(&self, ctx: &mut FunctionContext<'_>) -> Result<Box<dyn AggregateState>>;

    /// True when the state implements `value` and `inverse`.
    fn supports_window(&self) -> bool {
        false
    }
}

/// The accumulator an `AggregateFunction` builds for one group or window frame.
pub trait AggregateState: Send {
    fn step(&mut self, ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]) -> Result<()>;

    /// Called exactly once per group, even when the group had no rows.
    fn finalize(self: Box<Self>, ctx: &mut FunctionContext<'_>) -> Result<Value>;

    /// Only called when `AggregateFunction::supports_window` is true.
    fn value(&self, _ctx: &mut FunctionContext<'_>) -> Result<Value> {
        Err(LimboError::ParseError(
            "aggregate may not be used as a window function".to_string(),
        ))
    }

    /// Only called when `AggregateFunction::supports_window` is true.
    fn inverse(&mut self, _ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]) -> Result<()> {
        Err(LimboError::ParseError(
            "aggregate may not be used as a window function".to_string(),
        ))
    }
}

impl<F> ScalarFunction for F
where
    F: Fn(&mut FunctionContext<'_>, &[ValueRef<'_>]) -> Result<Value> + Send + Sync + 'static,
{
    fn call(&self, ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]) -> Result<Value> {
        self(ctx, args)
    }
}

struct ClosureAggInner<S, St, Fin> {
    step: St,
    finalize: Fin,
    _state: PhantomData<fn() -> S>,
}

pub struct ClosureAggregate<S, I, St, Fin> {
    init: I,
    inner: Arc<ClosureAggInner<S, St, Fin>>,
}

struct ClosureAggregateState<S, St, Fin> {
    state: S,
    inner: Arc<ClosureAggInner<S, St, Fin>>,
}

/// Build an `AggregateFunction` from `init` / `step` / `finalize` closures. No window support.
pub fn aggregate_from_fns<S, I, St, Fin>(
    init: I,
    step: St,
    finalize: Fin,
) -> ClosureAggregate<S, I, St, Fin>
where
    S: Send + 'static,
    I: Fn() -> S + Send + Sync + 'static,
    St: Fn(&mut S, &[ValueRef<'_>]) -> Result<()> + Send + Sync + 'static,
    Fin: Fn(S) -> Result<Value> + Send + Sync + 'static,
{
    ClosureAggregate {
        init,
        inner: Arc::new(ClosureAggInner {
            step,
            finalize,
            _state: PhantomData,
        }),
    }
}

impl<S, I, St, Fin> AggregateFunction for ClosureAggregate<S, I, St, Fin>
where
    S: Send + 'static,
    I: Fn() -> S + Send + Sync + 'static,
    St: Fn(&mut S, &[ValueRef<'_>]) -> Result<()> + Send + Sync + 'static,
    Fin: Fn(S) -> Result<Value> + Send + Sync + 'static,
{
    fn init(&self, _ctx: &mut FunctionContext<'_>) -> Result<Box<dyn AggregateState>> {
        Ok(Box::new(ClosureAggregateState {
            state: (self.init)(),
            inner: self.inner.clone(),
        }))
    }
}

impl<S, St, Fin> AggregateState for ClosureAggregateState<S, St, Fin>
where
    S: Send + 'static,
    St: Fn(&mut S, &[ValueRef<'_>]) -> Result<()> + Send + Sync + 'static,
    Fin: Fn(S) -> Result<Value> + Send + Sync + 'static,
{
    fn step(&mut self, _ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]) -> Result<()> {
        (self.inner.step)(&mut self.state, args)
    }

    fn finalize(self: Box<Self>, _ctx: &mut FunctionContext<'_>) -> Result<Value> {
        let this = *self;
        (this.inner.finalize)(this.state)
    }
}

#[derive(Clone)]
pub enum FunctionImpl {
    Scalar(Arc<dyn ScalarFunction>),
    Aggregate(Arc<dyn AggregateFunction>),
}

/// A function registered against a connection.
pub struct ExternalFunc {
    name: String,
    /// -1 means any number of arguments.
    argc: i32,
    flags: FunctionFlags,
    imp: FunctionImpl,
}

impl ExternalFunc {
    pub fn new_scalar(
        name: String,
        argc: i32,
        flags: FunctionFlags,
        f: Arc<dyn ScalarFunction>,
    ) -> Self {
        Self {
            name,
            argc,
            flags,
            imp: FunctionImpl::Scalar(f),
        }
    }

    pub fn new_aggregate(
        name: String,
        argc: i32,
        flags: FunctionFlags,
        f: Arc<dyn AggregateFunction>,
    ) -> Self {
        Self {
            name,
            argc,
            flags,
            imp: FunctionImpl::Aggregate(f),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn argc(&self) -> i32 {
        self.argc
    }

    pub fn flags(&self) -> FunctionFlags {
        self.flags
    }

    pub fn imp(&self) -> &FunctionImpl {
        &self.imp
    }

    pub fn is_aggregate(&self) -> bool {
        matches!(self.imp, FunctionImpl::Aggregate(_))
    }

    pub fn matches_arg_count(&self, arg_count: usize) -> bool {
        self.argc < 0 || self.argc as usize == arg_count
    }

    pub fn supports_window(&self) -> bool {
        match &self.imp {
            FunctionImpl::Aggregate(agg) => agg.supports_window(),
            FunctionImpl::Scalar(_) => false,
        }
    }

    pub fn as_scalar(&self) -> Option<&Arc<dyn ScalarFunction>> {
        match &self.imp {
            FunctionImpl::Scalar(f) => Some(f),
            FunctionImpl::Aggregate(_) => None,
        }
    }

    pub fn as_aggregate(&self) -> Option<&Arc<dyn AggregateFunction>> {
        match &self.imp {
            FunctionImpl::Aggregate(f) => Some(f),
            FunctionImpl::Scalar(_) => None,
        }
    }
}

impl Deterministic for ExternalFunc {
    fn is_deterministic(&self) -> bool {
        self.flags.contains(FunctionFlags::DETERMINISTIC)
    }
}

impl Debug for ExternalFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Display for ExternalFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Clone)]
/// An external aggregate at one call site. A variadic aggregate can be called with different
/// argument counts in one statement, and the VDBE needs the call's own count.
pub struct ExternalAggCall {
    pub func: Arc<ExternalFunc>,
    pub argc: usize,
}

impl Debug for ExternalAggCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.func.name())
    }
}

impl Display for ExternalAggCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.func.name())
    }
}

/// The same checks as SQLite's `sqlite3CreateFunc`.
pub(crate) fn validate_registration(name: &str, argc: i32) -> Result<()> {
    if name.is_empty() {
        return Err(LimboError::InvalidArgument(
            "function name may not be empty".to_string(),
        ));
    }
    if name.len() > MAX_FUNCTION_NAME_LEN {
        return Err(LimboError::InvalidArgument(format!(
            "function name is longer than {MAX_FUNCTION_NAME_LEN} bytes"
        )));
    }
    if !(-1..=MAX_FUNCTION_ARG).contains(&argc) {
        return Err(LimboError::InvalidArgument(format!(
            "function argument count must be between -1 and {MAX_FUNCTION_ARG}, got {argc}"
        )));
    }
    Ok(())
}

impl Connection {
    /// Replaces an existing function with the same name and argument count.
    pub fn create_scalar_function(
        &self,
        name: &str,
        argc: i32,
        flags: FunctionFlags,
        f: impl ScalarFunction + 'static,
    ) -> Result<()> {
        validate_registration(name, argc)?;
        let name = crate::util::normalize_ident(name);
        self.register_function(Arc::new(ExternalFunc::new_scalar(
            name,
            argc,
            flags,
            Arc::new(f),
        )))
    }

    /// Replaces an existing function with the same name and argument count.
    pub fn create_aggregate_function(
        &self,
        name: &str,
        argc: i32,
        flags: FunctionFlags,
        f: impl AggregateFunction + 'static,
    ) -> Result<()> {
        validate_registration(name, argc)?;
        let name = crate::util::normalize_ident(name);
        self.register_function(Arc::new(ExternalFunc::new_aggregate(
            name,
            argc,
            flags,
            Arc::new(f),
        )))
    }

    /// The name is used as given; callers must normalize it first.
    pub fn register_function(&self, func: Arc<ExternalFunc>) -> Result<()> {
        validate_registration(func.name(), func.argc())?;
        self.syms.write().insert_function(func);
        self.bump_prepare_context_generation();
        Ok(())
    }

    /// Removing a function that was never registered is not an error, matching SQLite.
    pub fn remove_function(&self, name: &str, argc: i32) -> Result<()> {
        validate_registration(name, argc)?;
        let name = crate::util::normalize_ident(name);
        if self.syms.write().remove_function(&name, argc) {
            self.bump_prepare_context_generation();
        }
        Ok(())
    }

    /// What the C ABI's `unregister_function` exposes: every arity under `name`.
    pub fn remove_all_functions_named(&self, name: &str) -> bool {
        let name = crate::util::normalize_ident(name);
        let removed = self.syms.write().remove_all_functions_named(&name);
        if removed {
            self.bump_prepare_context_generation();
        }
        removed
    }
}

/// SQLite's `sqlite3ExprFunctionUsable`: DIRECTONLY is never allowed in schema SQL (triggers, views,
/// CHECK, DEFAULT, generated columns, index expressions), and with `trusted_schema` off only
/// INNOCUOUS is. Built-ins are not routed through here.
pub(crate) fn check_usable_from_schema(
    name: &str,
    flags: FunctionFlags,
    trusted_schema: bool,
) -> Result<()> {
    let direct_only = flags.contains(FunctionFlags::DIRECTONLY);
    let untrusted = !trusted_schema && !flags.contains(FunctionFlags::INNOCUOUS);
    if direct_only || untrusted {
        return Err(LimboError::ParseError(format!("unsafe use of {name}()")));
    }
    Ok(())
}
