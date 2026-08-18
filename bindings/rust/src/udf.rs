//! Closure-based user-defined functions. Arguments are borrowed straight out
//! of the query, with no extra copy.

use crate::value::{Value, ValueRef};
use crate::{Error, Result};

pub use turso_sdk_kit::rsapi::udf::FunctionFlags;

/// A function that produces one value per call.
///
/// # Examples
///
/// ```rust,no_run
/// use turso::udf::FunctionFlags;
/// use turso::{Builder, Value, ValueRef};
///
/// # async fn run() -> turso::Result<()> {
/// let db = Builder::new_local(":memory:").build().await?;
/// let conn = db.connect()?;
/// conn.create_scalar_function(
///     "double",
///     1,
///     FunctionFlags::DETERMINISTIC,
///     |args: &[ValueRef<'_>]| {
///         let n = args[0].as_integer().copied().unwrap_or(0);
///         Ok(Value::Integer(n * 2))
///     },
/// )?;
/// let mut rows = conn.query("SELECT double(21)", ()).await?;
/// let row = rows.next().await?.unwrap();
/// assert_eq!(row.get_value(0)?, Value::Integer(42));
/// # Ok(())
/// # }
/// ```
pub trait ScalarFunction: Send + Sync {
    fn call(&self, args: &[ValueRef<'_>]) -> Result<Value>;
}

impl<F> ScalarFunction for F
where
    F: Fn(&[ValueRef<'_>]) -> Result<Value> + Send + Sync + 'static,
{
    fn call(&self, args: &[ValueRef<'_>]) -> Result<Value> {
        self(args)
    }
}

/// A function that accumulates over many rows, e.g. `SUM` or `COUNT`.
///
/// Implement [`AggregateState::value`] and [`AggregateState::inverse`] and
/// return `true` from [`Self::supports_window`] to also allow `OVER (...)`.
///
/// # Examples
///
/// ```rust,no_run
/// use turso::udf::{AggregateFunction, AggregateState, FunctionFlags};
/// use turso::{Builder, Value, ValueRef};
///
/// struct Sum;
/// struct SumState(i64);
///
/// impl AggregateFunction for Sum {
///     fn init(&self) -> turso::Result<Box<dyn AggregateState>> {
///         Ok(Box::new(SumState(0)))
///     }
/// }
///
/// impl AggregateState for SumState {
///     fn step(&mut self, args: &[ValueRef<'_>]) -> turso::Result<()> {
///         self.0 += args[0].as_integer().copied().unwrap_or(0);
///         Ok(())
///     }
///
///     fn finalize(self: Box<Self>) -> turso::Result<Value> {
///         Ok(Value::Integer(self.0))
///     }
/// }
///
/// # async fn run() -> turso::Result<()> {
/// let db = Builder::new_local(":memory:").build().await?;
/// let conn = db.connect()?;
/// conn.create_aggregate_function("my_sum", 1, FunctionFlags::empty(), Sum)?;
/// conn.execute("CREATE TABLE t (n INTEGER)", ()).await?;
/// conn.execute("INSERT INTO t VALUES (1), (2), (3)", ()).await?;
/// let mut rows = conn.query("SELECT my_sum(n) FROM t", ()).await?;
/// let row = rows.next().await?.unwrap();
/// assert_eq!(row.get_value(0)?, Value::Integer(6));
/// # Ok(())
/// # }
/// ```
pub trait AggregateFunction: Send + Sync {
    /// Make the accumulator for one group (or one window frame).
    fn init(&self) -> Result<Box<dyn AggregateState>>;

    /// True when the state implements [`AggregateState::value`] and
    /// [`AggregateState::inverse`].
    fn supports_window(&self) -> bool {
        false
    }
}

/// The accumulator an [`AggregateFunction`] builds for one group.
pub trait AggregateState: Send {
    fn step(&mut self, args: &[ValueRef<'_>]) -> Result<()>;

    /// Called exactly once per group, even when the group had no rows.
    fn finalize(self: Box<Self>) -> Result<Value>;

    /// The answer so far, without destroying the accumulator. Only called when
    /// [`AggregateFunction::supports_window`] returns true.
    fn value(&self) -> Result<Value> {
        Err(Error::Misuse(
            "aggregate may not be used as a window function".to_string(),
        ))
    }

    /// Undoes an earlier `step` for a row that left the window frame. Only
    /// called when [`AggregateFunction::supports_window`] returns true.
    fn inverse(&mut self, _args: &[ValueRef<'_>]) -> Result<()> {
        Err(Error::Misuse(
            "aggregate may not be used as a window function".to_string(),
        ))
    }
}

pub(crate) struct ScalarAdapter<F>(pub(crate) F);

impl<F: ScalarFunction> turso_core::udf::ScalarFunction for ScalarAdapter<F> {
    fn call(
        &self,
        ctx: &mut turso_core::udf::FunctionContext<'_>,
        args: &[turso_core::types::ValueRef<'_>],
    ) -> turso_core::Result<turso_core::Value> {
        let args: Vec<ValueRef<'_>> = args.iter().copied().map(to_public_value_ref).collect();
        self.0
            .call(&args)
            .map(Into::into)
            .map_err(|err| to_core_error(ctx, err))
    }
}

pub(crate) struct AggregateAdapter<F>(pub(crate) F);

impl<F: AggregateFunction> turso_core::udf::AggregateFunction for AggregateAdapter<F> {
    fn init(
        &self,
        ctx: &mut turso_core::udf::FunctionContext<'_>,
    ) -> turso_core::Result<Box<dyn turso_core::udf::AggregateState>> {
        let state = self.0.init().map_err(|err| to_core_error(ctx, err))?;
        Ok(Box::new(AggregateStateAdapter(state)))
    }

    fn supports_window(&self) -> bool {
        self.0.supports_window()
    }
}

struct AggregateStateAdapter(Box<dyn AggregateState>);

impl turso_core::udf::AggregateState for AggregateStateAdapter {
    fn step(
        &mut self,
        ctx: &mut turso_core::udf::FunctionContext<'_>,
        args: &[turso_core::types::ValueRef<'_>],
    ) -> turso_core::Result<()> {
        let args: Vec<ValueRef<'_>> = args.iter().copied().map(to_public_value_ref).collect();
        self.0.step(&args).map_err(|err| to_core_error(ctx, err))
    }

    fn finalize(
        self: Box<Self>,
        ctx: &mut turso_core::udf::FunctionContext<'_>,
    ) -> turso_core::Result<turso_core::Value> {
        self.0
            .finalize()
            .map(Into::into)
            .map_err(|err| to_core_error(ctx, err))
    }

    fn value(
        &self,
        ctx: &mut turso_core::udf::FunctionContext<'_>,
    ) -> turso_core::Result<turso_core::Value> {
        self.0
            .value()
            .map(Into::into)
            .map_err(|err| to_core_error(ctx, err))
    }

    fn inverse(
        &mut self,
        ctx: &mut turso_core::udf::FunctionContext<'_>,
        args: &[turso_core::types::ValueRef<'_>],
    ) -> turso_core::Result<()> {
        let args: Vec<ValueRef<'_>> = args.iter().copied().map(to_public_value_ref).collect();
        self.0.inverse(&args).map_err(|err| to_core_error(ctx, err))
    }
}

fn to_public_value_ref(v: turso_core::types::ValueRef<'_>) -> ValueRef<'_> {
    use turso_core::types::ValueRef as CoreValueRef;
    use turso_core::Numeric;
    match v {
        CoreValueRef::Null => ValueRef::Null,
        CoreValueRef::Numeric(Numeric::Integer(i)) => ValueRef::Integer(i),
        CoreValueRef::Numeric(Numeric::Float(f)) => ValueRef::Real(f64::from(f)),
        CoreValueRef::Text(text) => ValueRef::Text(text.as_str().as_bytes()),
        CoreValueRef::Blob(blob) => ValueRef::Blob(blob),
    }
}

fn to_core_error(ctx: &turso_core::udf::FunctionContext<'_>, err: Error) -> turso_core::LimboError {
    ctx.error(err.to_string())
}
