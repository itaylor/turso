//! Native user-defined functions registered through `Connection::create_scalar_function` /
//! `create_aggregate_function`.

use crate::common::{ExecRows, TempDatabase};
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
use std::sync::Arc;
use turso_core::{
    AggregateFunction, AggregateState, FunctionContext, FunctionFlags, LimboError, Numeric, Result,
    StepResult, Value, ValueRef,
};

fn arg_int(arg: &ValueRef<'_>) -> i64 {
    match arg {
        ValueRef::Numeric(Numeric::Integer(i)) => *i,
        ValueRef::Numeric(Numeric::Float(f)) => f64::from(*f) as i64,
        _ => 0,
    }
}

fn sum_args(args: &[ValueRef<'_>]) -> i64 {
    args.iter().map(arg_int).sum()
}

// ---------------------------------------------------------------- scalars

#[turso_macros::test]
fn native_scalar_closure_sees_its_arguments(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    conn.create_scalar_function(
        "triple",
        1,
        FunctionFlags::DETERMINISTIC,
        |_ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]| -> Result<Value> {
            Ok(Value::from_i64(arg_int(&args[0]) * 3))
        },
    )?;

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT triple(7)");
    assert_eq!(rows, vec![(21,)]);

    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (1), (2), (3)")?;
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT triple(x) FROM t ORDER BY x");
    assert_eq!(rows, vec![(3,), (6,), (9,)]);
    Ok(())
}

#[turso_macros::test]
fn native_scalar_errors_abort_the_statement(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    conn.create_scalar_function(
        "boom",
        0,
        FunctionFlags::empty(),
        |_ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            Err(LimboError::ExtensionError("boom happened".to_string()))
        },
    )?;

    let err = conn.execute("SELECT boom()").unwrap_err();
    assert!(err.to_string().contains("boom happened"), "{err}");

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT 1");
    assert_eq!(rows, vec![(1,)]);
    Ok(())
}

#[turso_macros::test]
fn function_list_reports_native_functions(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    conn.create_scalar_function(
        "listed_scalar",
        2,
        FunctionFlags::DETERMINISTIC | FunctionFlags::INNOCUOUS,
        |_ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]| -> Result<Value> {
            Ok(Value::from_i64(sum_args(args)))
        },
    )?;
    conn.create_aggregate_function(
        "listed_agg",
        1,
        FunctionFlags::empty(),
        SumAggregate { window: false },
    )?;

    let listed: Vec<(String, i64, String, String, i64, i64)> =
        conn.exec_rows("PRAGMA function_list");
    let scalar = listed
        .iter()
        .find(|(name, ..)| name == "listed_scalar")
        .expect("listed_scalar should be listed");
    assert_eq!(scalar.1, 0, "not a builtin");
    assert_eq!(scalar.2, "s");
    assert_eq!(scalar.4, 2);
    assert_eq!(scalar.5, 0x800 | 0x200000);

    let agg = listed
        .iter()
        .find(|(name, ..)| name == "listed_agg")
        .expect("listed_agg should be listed");
    assert_eq!(agg.2, "a");
    assert_eq!(agg.4, 1);
    assert_eq!(agg.5, 0);
    Ok(())
}

// ------------------------------------------------------------- aggregates

/// Sums its first argument; `window` controls whether it advertises `value` / `inverse`.
struct SumAggregate {
    window: bool,
}

struct SumAggregateState {
    total: i64,
    finals: Option<Arc<AtomicUsize>>,
}

impl AggregateFunction for SumAggregate {
    fn init(&self, _ctx: &mut FunctionContext<'_>) -> Result<Box<dyn AggregateState>> {
        Ok(Box::new(SumAggregateState {
            total: 0,
            finals: None,
        }))
    }

    fn supports_window(&self) -> bool {
        self.window
    }
}

impl AggregateState for SumAggregateState {
    fn step(&mut self, _ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]) -> Result<()> {
        self.total += sum_args(args);
        Ok(())
    }

    fn finalize(self: Box<Self>, _ctx: &mut FunctionContext<'_>) -> Result<Value> {
        if let Some(finals) = &self.finals {
            finals.fetch_add(1, AtomicOrdering::SeqCst);
        }
        Ok(Value::from_i64(self.total))
    }

    fn value(&self, _ctx: &mut FunctionContext<'_>) -> Result<Value> {
        Ok(Value::from_i64(self.total))
    }

    fn inverse(&mut self, _ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]) -> Result<()> {
        self.total -= sum_args(args);
        Ok(())
    }
}

struct CountingSum {
    finals: Arc<AtomicUsize>,
}

impl AggregateFunction for CountingSum {
    fn init(&self, _ctx: &mut FunctionContext<'_>) -> Result<Box<dyn AggregateState>> {
        Ok(Box::new(SumAggregateState {
            total: 0,
            finals: Some(self.finals.clone()),
        }))
    }
}

#[turso_macros::test]
fn native_aggregate_accumulates_per_group(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    let finals = Arc::new(AtomicUsize::new(0));
    conn.create_aggregate_function(
        "nsum",
        1,
        FunctionFlags::empty(),
        CountingSum {
            finals: finals.clone(),
        },
    )?;

    conn.execute("CREATE TABLE t(g TEXT, x INTEGER)")?;
    conn.execute("INSERT INTO t VALUES ('a', 1), ('a', 2), ('b', 10)")?;

    let rows: Vec<(String, i64)> = conn.exec_rows("SELECT g, nsum(x) FROM t GROUP BY g ORDER BY g");
    assert_eq!(rows, vec![("a".to_string(), 3), ("b".to_string(), 10)]);
    assert_eq!(finals.load(AtomicOrdering::SeqCst), 2);

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT nsum(x) FROM t WHERE 0");
    assert_eq!(rows, vec![(0,)]);
    assert_eq!(finals.load(AtomicOrdering::SeqCst), 3);
    Ok(())
}

#[turso_macros::test]
fn aggregate_built_from_closures_works(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    conn.create_aggregate_function(
        "closure_sum",
        1,
        FunctionFlags::empty(),
        turso_core::aggregate_from_fns(
            || 0i64,
            |total: &mut i64, args: &[ValueRef<'_>]| -> Result<()> {
                *total += sum_args(args);
                Ok(())
            },
            |total: i64| -> Result<Value> { Ok(Value::from_i64(total)) },
        ),
    )?;

    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (4), (5), (6)")?;
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT closure_sum(x) FROM t");
    assert_eq!(rows, vec![(15,)]);
    Ok(())
}

#[turso_macros::test]
fn aggregate_in_scalar_context_is_a_compile_error(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    conn.create_aggregate_function(
        "nsum",
        1,
        FunctionFlags::empty(),
        SumAggregate { window: false },
    )?;
    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (1)")?;

    let err = conn
        .prepare("SELECT * FROM t WHERE nsum(x) > 0")
        .unwrap_err();
    assert!(
        err.to_string().contains("misuse of aggregate function"),
        "{err}"
    );
    Ok(())
}

// ------------------------------------------------------------- overloading

#[turso_macros::test]
fn overloading_dispatches_on_argument_count(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    conn.create_scalar_function(
        "f",
        1,
        FunctionFlags::empty(),
        |_ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            Ok(Value::from_i64(1))
        },
    )?;
    conn.create_scalar_function(
        "f",
        2,
        FunctionFlags::empty(),
        |_ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            Ok(Value::from_i64(2))
        },
    )?;
    conn.create_scalar_function(
        "f",
        -1,
        FunctionFlags::empty(),
        |_ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]| -> Result<Value> {
            Ok(Value::from_i64(100 + args.len() as i64))
        },
    )?;

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT f(1)");
    assert_eq!(rows, vec![(1,)]);
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT f(1, 2)");
    assert_eq!(rows, vec![(2,)]);
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT f(1, 2, 3)");
    assert_eq!(rows, vec![(103,)]);
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT f()");
    assert_eq!(rows, vec![(100,)]);
    Ok(())
}

#[turso_macros::test]
fn unknown_name_and_wrong_arity_report_different_errors(
    tmp_db: TempDatabase,
) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    conn.create_scalar_function(
        "f",
        1,
        FunctionFlags::empty(),
        |_ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            Ok(Value::from_i64(1))
        },
    )?;

    let err = conn.prepare("SELECT f(1, 2)").unwrap_err();
    assert_eq!(
        err.to_string(),
        "Parse error: wrong number of arguments to function f()"
    );

    let err = conn.prepare("SELECT no_such_udf(1)").unwrap_err();
    assert_eq!(
        err.to_string(),
        "Parse error: no such function: no_such_udf"
    );
    Ok(())
}

#[turso_macros::test]
fn re_registering_the_same_arity_replaces_it(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    conn.create_scalar_function(
        "f",
        1,
        FunctionFlags::empty(),
        |_ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            Ok(Value::from_i64(1))
        },
    )?;
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT f(0)");
    assert_eq!(rows, vec![(1,)]);

    conn.create_scalar_function(
        "f",
        1,
        FunctionFlags::empty(),
        |_ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            Ok(Value::from_i64(2))
        },
    )?;
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT f(0)");
    assert_eq!(rows, vec![(2,)]);

    let listed: Vec<(String, i64, String, String, i64, i64)> =
        conn.exec_rows("PRAGMA function_list");
    assert_eq!(listed.iter().filter(|(name, ..)| name == "f").count(), 1);
    Ok(())
}

#[turso_macros::test]
fn remove_function_only_removes_the_matching_arity(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    conn.create_scalar_function(
        "f",
        1,
        FunctionFlags::empty(),
        |_ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            Ok(Value::from_i64(1))
        },
    )?;
    conn.create_scalar_function(
        "f",
        2,
        FunctionFlags::empty(),
        |_ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            Ok(Value::from_i64(2))
        },
    )?;

    conn.remove_function("f", 1)?;
    let err = conn.prepare("SELECT f(1)").unwrap_err();
    assert_eq!(
        err.to_string(),
        "Parse error: wrong number of arguments to function f()"
    );
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT f(1, 2)");
    assert_eq!(rows, vec![(2,)]);

    conn.remove_function("f", 7)?;
    conn.remove_function("never_registered", 1)?;

    conn.remove_function("f", 2)?;
    let err = conn.prepare("SELECT f(1, 2)").unwrap_err();
    assert_eq!(err.to_string(), "Parse error: no such function: f");
    Ok(())
}

#[turso_macros::test]
fn registration_validates_name_and_argument_count(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    let noop = |_ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
        Ok(Value::Null)
    };

    assert!(conn
        .create_scalar_function("", 1, FunctionFlags::empty(), noop)
        .is_err());
    assert!(conn
        .create_scalar_function(&"x".repeat(256), 1, FunctionFlags::empty(), noop)
        .is_err());
    assert!(conn
        .create_scalar_function("f", -2, FunctionFlags::empty(), noop)
        .is_err());
    assert!(conn
        .create_scalar_function(
            "f",
            turso_core::MAX_FUNCTION_ARG + 1,
            FunctionFlags::empty(),
            noop
        )
        .is_err());

    conn.create_scalar_function(&"x".repeat(255), -1, FunctionFlags::empty(), noop)?;
    conn.create_scalar_function(
        "g",
        turso_core::MAX_FUNCTION_ARG,
        FunctionFlags::empty(),
        noop,
    )?;
    Ok(())
}

// ------------------------------------------------------------------ windows

#[turso_macros::test]
fn window_capable_aggregate_runs_over_a_moving_frame(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    conn.create_aggregate_function(
        "wsum",
        1,
        FunctionFlags::empty(),
        SumAggregate { window: true },
    )?;
    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (1), (2), (3)")?;

    let rows: Vec<(i64, i64)> =
        conn.exec_rows("SELECT x, wsum(x) OVER (ORDER BY x) FROM t ORDER BY x");
    assert_eq!(rows, vec![(1, 1), (2, 3), (3, 6)]);

    let rows: Vec<(i64, i64)> = conn.exec_rows(
        "SELECT x, wsum(x) OVER (ORDER BY x ROWS BETWEEN 1 PRECEDING AND CURRENT ROW) \
         FROM t ORDER BY x",
    );
    assert_eq!(rows, vec![(1, 1), (2, 3), (3, 5)]);
    Ok(())
}

#[turso_macros::test]
fn aggregate_without_window_support_is_rejected_in_a_running_frame(
    tmp_db: TempDatabase,
) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    conn.create_aggregate_function(
        "nsum",
        1,
        FunctionFlags::empty(),
        SumAggregate { window: false },
    )?;
    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (1), (2), (3)")?;

    let err = conn
        .prepare("SELECT x, nsum(x) OVER (ORDER BY x) FROM t")
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("nsum() may not be used as a window function"),
        "{err}"
    );

    let err = conn
        .prepare(
            "SELECT x, nsum(x) OVER (ORDER BY x ROWS BETWEEN 1 PRECEDING AND CURRENT ROW) FROM t",
        )
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("nsum() may not be used as a window function"),
        "{err}"
    );

    // `OVER ()` still reads the accumulator once per output row (AggValue), so it needs `value` too.
    let err = conn
        .prepare("SELECT x, nsum(x) OVER () FROM t")
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("nsum() may not be used as a window function"),
        "{err}"
    );

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT nsum(x) FROM t");
    assert_eq!(rows, vec![(6,)]);
    Ok(())
}

/// Regression: `AggValue` used to run the destructive finalize once per row and abort the process.
#[turso_macros::test]
fn builtin_median_over_a_running_frame_errors_instead_of_aborting(
    tmp_db: TempDatabase,
) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (1), (2), (3)")?;

    let err = conn
        .prepare("SELECT x, median(x) OVER (ORDER BY x) FROM t")
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("median() may not be used as a window function"),
        "{err}"
    );
    Ok(())
}

// ------------------------------------------------- invalidation, reentrancy

#[turso_macros::test]
fn a_running_statement_keeps_the_function_it_started_with(
    tmp_db: TempDatabase,
) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    conn.create_scalar_function(
        "f",
        1,
        FunctionFlags::empty(),
        |_ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            Ok(Value::from_i64(1))
        },
    )?;
    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (1), (2)")?;

    let mut stmt = conn.prepare("SELECT f(x) FROM t")?;
    assert!(matches!(stmt.step()?, StepResult::Row));
    assert_eq!(stmt.row().expect("row").get::<i64>(0)?, 1);

    // A running statement holds its own snapshot of the function.
    conn.create_scalar_function(
        "f",
        1,
        FunctionFlags::empty(),
        |_ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            Ok(Value::from_i64(2))
        },
    )?;
    assert!(matches!(stmt.step()?, StepResult::Row));
    assert_eq!(stmt.row().expect("row").get::<i64>(0)?, 1);
    drop(stmt);

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT f(0)");
    assert_eq!(rows, vec![(2,)]);
    Ok(())
}

#[turso_macros::test]
fn a_scalar_function_can_query_its_own_connection(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (10), (20)")?;
    conn.execute("CREATE TABLE lookup(k INTEGER, v INTEGER)")?;
    conn.execute("INSERT INTO lookup VALUES (10, 111), (20, 222)")?;

    conn.create_scalar_function(
        "lookup_v",
        1,
        FunctionFlags::empty(),
        |ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]| -> Result<Value> {
            let key = arg_int(&args[0]);
            let conn = ctx.connection().clone();
            let mut stmt = conn.prepare(format!("SELECT v FROM lookup WHERE k = {key}"))?;
            loop {
                match stmt.step()? {
                    StepResult::Row => {
                        let row = stmt.row().expect("row is available after StepResult::Row");
                        return Ok(Value::from_i64(row.get::<i64>(0)?));
                    }
                    StepResult::Done => return Ok(Value::Null),
                    StepResult::IO => stmt._io().step()?,
                    other => {
                        return Err(LimboError::InternalError(format!(
                            "nested statement returned {other:?}"
                        )))
                    }
                }
            }
        },
    )?;

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT lookup_v(x) FROM t ORDER BY x");
    assert_eq!(rows, vec![(111,), (222,)]);
    Ok(())
}

// -------------------------------------------------- shadowing built-ins

fn register_constant(
    conn: &Arc<turso_core::Connection>,
    name: &str,
    argc: i32,
    answer: i64,
) -> Result<()> {
    conn.create_scalar_function(
        name,
        argc,
        FunctionFlags::DETERMINISTIC,
        move |_ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            Ok(Value::from_i64(answer))
        },
    )
}

#[turso_macros::test]
fn an_app_function_shadows_the_builtin_of_the_same_name_and_arity(
    tmp_db: TempDatabase,
) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_constant(&conn, "abs", 1, 42)?;

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT abs(-1)");
    assert_eq!(rows, vec![(42,)]);

    let rows: Vec<(String,)> = conn.exec_rows("SELECT upper('ab')");
    assert_eq!(rows, vec![("AB".to_string(),)]);

    conn.remove_function("abs", 1)?;
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT abs(-1)");
    assert_eq!(rows, vec![(1,)]);
    Ok(())
}

/// `sqlite3FindFunction` only falls back to the built-in when no application function fits the
/// argument count, so registering `abs(x, y)` leaves `abs(x)` alone.
#[turso_macros::test]
fn an_app_function_with_another_arity_leaves_the_builtin_reachable(
    tmp_db: TempDatabase,
) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_constant(&conn, "abs", 2, 42)?;

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT abs(-1)");
    assert_eq!(rows, vec![(1,)], "the one-argument built-in still answers");
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT abs(-1, -2)");
    assert_eq!(rows, vec![(42,)]);
    Ok(())
}

#[turso_macros::test]
fn a_variadic_app_function_hides_every_arity_of_the_builtin(
    tmp_db: TempDatabase,
) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (1), (5), (3)")?;
    // Built-in `max` is both a variadic scalar and a one-argument aggregate; a variadic registration takes over both.
    conn.create_scalar_function(
        "max",
        -1,
        FunctionFlags::DETERMINISTIC,
        |_ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]| -> Result<Value> {
            Ok(Value::from_i64(1000 + args.len() as i64))
        },
    )?;

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT max(1, 2, 3)");
    assert_eq!(rows, vec![(1003,)]);
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT max(x) FROM t");
    assert_eq!(rows, vec![(1001,), (1001,), (1001,)]);
    Ok(())
}

#[turso_macros::test]
fn a_two_argument_app_max_leaves_the_aggregate_max_alone(
    tmp_db: TempDatabase,
) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (1), (5), (3)")?;
    register_constant(&conn, "max", 2, 42)?;

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT max(1, 2)");
    assert_eq!(rows, vec![(42,)]);
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT max(x) FROM t");
    assert_eq!(rows, vec![(5,)], "the built-in aggregate still answers");
    Ok(())
}

#[turso_macros::test]
fn an_app_aggregate_shadows_the_builtin_sum(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (1), (2), (3)")?;

    let plain: Vec<(i64,)> = conn.exec_rows("SELECT sum(x) FROM t");
    assert_eq!(plain, vec![(6,)]);

    conn.create_aggregate_function(
        "sum",
        1,
        FunctionFlags::empty(),
        turso_core::aggregate_from_fns(
            || 0i64,
            |total: &mut i64, args: &[ValueRef<'_>]| -> Result<()> {
                *total += 10 * sum_args(args);
                Ok(())
            },
            |total: i64| -> Result<Value> { Ok(Value::from_i64(total)) },
        ),
    )?;

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT sum(x) FROM t");
    assert_eq!(rows, vec![(60,)]);
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT sum(x) FROM t GROUP BY x ORDER BY x");
    assert_eq!(rows, vec![(10,), (20,), (30,)]);
    Ok(())
}

#[turso_macros::test]
fn an_app_count_of_one_argument_leaves_count_star_alone(
    tmp_db: TempDatabase,
) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (1), (2), (3)")?;
    register_constant(&conn, "count", 1, 7)?;

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT count(x) FROM t");
    assert_eq!(rows, vec![(7,), (7,), (7,)]);
    // `count(*)` passes no arguments, so it finds no app match and stays the built-in aggregate.
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT count(*) FROM t");
    assert_eq!(rows, vec![(3,)]);
    Ok(())
}

/// `coalesce`, `ifnull` and `iif` are rewritten by the translator rather than called like ordinary functions.
#[turso_macros::test]
fn overriding_a_rewritten_builtin_calls_the_app_function(
    tmp_db: TempDatabase,
) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_constant(&conn, "coalesce", 2, 11)?;
    register_constant(&conn, "ifnull", 2, 22)?;
    register_constant(&conn, "iif", 3, 33)?;
    register_constant(&conn, "likely", 1, 44)?;
    register_constant(&conn, "nullif", 2, 55)?;

    let rows: Vec<(i64, i64, i64, i64, i64)> = conn
        .exec_rows("SELECT coalesce(1, 2), ifnull(1, 2), iif(1, 2, 3), likely(1), nullif(1, 2)");
    assert_eq!(rows, vec![(11, 22, 33, 44, 55)]);
    Ok(())
}

/// Unnesting a scalar aggregate subquery writes `coalesce(x, 0)` into the plan; that rewrite must not
/// fire when the connection has its own `coalesce`.
#[turso_macros::test]
fn overriding_coalesce_does_not_change_an_unnested_subquery(
    tmp_db: TempDatabase,
) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    conn.execute("CREATE TABLE a(k)")?;
    conn.execute("CREATE TABLE b(k)")?;
    conn.execute("INSERT INTO a VALUES (1), (2)")?;
    conn.execute("INSERT INTO b VALUES (1), (1)")?;

    let query = "SELECT k, (SELECT count(*) FROM b WHERE b.k = a.k) FROM a ORDER BY k";
    let before: Vec<(i64, i64)> = conn.exec_rows(query);
    assert_eq!(before, vec![(1, 2), (2, 0)]);

    register_constant(&conn, "coalesce", 2, 999)?;
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT coalesce(1, 2)");
    assert_eq!(rows, vec![(999,)], "the registration took effect");

    let after: Vec<(i64, i64)> = conn.exec_rows(query);
    assert_eq!(after, before);
    Ok(())
}

#[turso_macros::test]
fn an_app_scalar_may_not_carry_an_over_clause(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (1)")?;
    register_constant(&conn, "konst", 1, 1)?;

    let err = conn.prepare("SELECT konst(x) OVER () FROM t").unwrap_err();
    assert!(
        err.to_string()
            .contains("konst() may not be used as a window function"),
        "{err}"
    );
    Ok(())
}

// ------------------------------------------- UDFs in schema expressions

/// `dbl(x)` is deterministic; `roll()` is not and counts its calls.
fn register_schema_test_functions(conn: &Arc<turso_core::Connection>) -> Result<()> {
    conn.create_scalar_function(
        "dbl",
        1,
        FunctionFlags::DETERMINISTIC,
        |_ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]| -> Result<Value> {
            Ok(Value::from_i64(arg_int(&args[0]) * 2))
        },
    )?;
    let calls = AtomicUsize::new(0);
    conn.create_scalar_function(
        "roll",
        0,
        FunctionFlags::empty(),
        move |_ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            Ok(Value::from_i64(
                calls.fetch_add(1, AtomicOrdering::SeqCst) as i64
            ))
        },
    )
}

#[turso_macros::test]
fn a_deterministic_udf_can_be_indexed(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_schema_test_functions(&conn)?;
    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (1), (2), (3)")?;
    conn.execute("CREATE INDEX i ON t(dbl(x))")?;

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT x FROM t WHERE dbl(x) = 4");
    assert_eq!(rows, vec![(2,)]);

    conn.execute("INSERT INTO t VALUES (10)")?;
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT x FROM t WHERE dbl(x) = 20");
    assert_eq!(rows, vec![(10,)]);

    let plan: Vec<(i64, i64, i64, String)> =
        conn.exec_rows("EXPLAIN QUERY PLAN SELECT x FROM t WHERE dbl(x) = 4");
    assert!(
        plan.iter().any(|(_, _, _, detail)| detail.contains("i")),
        "the query should use the expression index: {plan:?}"
    );
    Ok(())
}

#[turso_macros::test]
fn a_non_deterministic_udf_may_not_be_indexed(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_schema_test_functions(&conn)?;
    conn.execute("CREATE TABLE t(x)")?;

    let err = conn.execute("CREATE INDEX i ON t(x + roll())").unwrap_err();
    assert_eq!(
        err.to_string(),
        "Parse error: non-deterministic functions prohibited in index expressions"
    );

    let err = conn
        .execute("CREATE INDEX i ON t(x) WHERE roll() > 0")
        .unwrap_err();
    assert_eq!(
        err.to_string(),
        "Parse error: non-deterministic functions prohibited in partial index WHERE clauses"
    );
    Ok(())
}

#[turso_macros::test]
fn a_deterministic_udf_can_filter_a_partial_index(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_schema_test_functions(&conn)?;
    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (1), (2), (3)")?;
    conn.execute("CREATE INDEX i ON t(x) WHERE dbl(x) > 4")?;

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT x FROM t WHERE dbl(x) > 4 ORDER BY x");
    assert_eq!(rows, vec![(3,)]);
    Ok(())
}

#[turso_macros::test]
fn an_aggregate_udf_may_not_be_indexed(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    conn.create_aggregate_function(
        "nsum",
        1,
        FunctionFlags::DETERMINISTIC,
        SumAggregate { window: false },
    )?;
    conn.execute("CREATE TABLE t(x)")?;

    let err = conn.execute("CREATE INDEX i ON t(nsum(x))").unwrap_err();
    assert!(
        err.to_string()
            .contains("invalid expression in CREATE INDEX"),
        "{err}"
    );
    Ok(())
}

#[turso_macros::test]
fn indexing_a_function_nobody_registered_says_no_such_function(
    tmp_db: TempDatabase,
) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    conn.execute("CREATE TABLE t(x)")?;
    let err = conn.execute("CREATE INDEX i ON t(nope(x))").unwrap_err();
    assert_eq!(err.to_string(), "Parse error: no such function: nope");
    Ok(())
}

#[turso_macros::test]
fn a_non_deterministic_udf_is_allowed_in_a_check_constraint(
    tmp_db: TempDatabase,
) -> anyhow::Result<()> {
    // SQLite allows non-deterministic functions in CHECK, unlike in index expressions and generated columns.
    let conn = tmp_db.connect_limbo();
    register_schema_test_functions(&conn)?;
    conn.execute("CREATE TABLE t(x INTEGER, CHECK (dbl(x) < 100))")?;
    conn.execute("INSERT INTO t VALUES (10)")?;
    let err = conn.execute("INSERT INTO t VALUES (60)").unwrap_err();
    assert!(err.to_string().contains("CHECK constraint failed"), "{err}");

    conn.execute("CREATE TABLE u(x INTEGER, CHECK (roll() >= 0))")?;
    conn.execute("INSERT INTO u VALUES (1)")?;
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT x FROM u");
    assert_eq!(rows, vec![(1,)]);
    Ok(())
}

/// On its own file so the reopen tests can close it and open it again.
fn generated_columns_db(path: &std::path::Path) -> TempDatabase {
    TempDatabase::builder()
        .with_db_path(path)
        .with_opts(
            turso_core::DatabaseOpts::new()
                .with_generated_columns(true)
                .with_views(true),
        )
        .build()
}

#[test]
fn a_deterministic_udf_can_compute_a_generated_column() -> anyhow::Result<()> {
    let dir = tempfile::TempDir::new()?;
    let db = generated_columns_db(&dir.path().join("gencol_udf.db"));
    let conn = db.connect_limbo();
    register_schema_test_functions(&conn)?;

    conn.execute("CREATE TABLE t(a INTEGER, b AS (dbl(a)))")?;
    conn.execute("INSERT INTO t(a) VALUES (1), (2)")?;
    let rows: Vec<(i64, i64)> = conn.exec_rows("SELECT a, b FROM t ORDER BY a");
    assert_eq!(rows, vec![(1, 2), (2, 4)]);
    Ok(())
}

#[test]
fn a_non_deterministic_udf_may_not_compute_a_generated_column() -> anyhow::Result<()> {
    let dir = tempfile::TempDir::new()?;
    let db = generated_columns_db(&dir.path().join("gencol_udf_bad.db"));
    let conn = db.connect_limbo();
    register_schema_test_functions(&conn)?;

    let err = conn
        .execute("CREATE TABLE t(a INTEGER, b AS (roll() + a))")
        .unwrap_err();
    assert_eq!(
        err.to_string(),
        "Parse error: non-deterministic functions prohibited in generated columns"
    );

    conn.create_aggregate_function(
        "nsum",
        1,
        FunctionFlags::DETERMINISTIC,
        SumAggregate { window: false },
    )?;
    let err = conn
        .execute("CREATE TABLE u(a INTEGER, b AS (nsum(a)))")
        .unwrap_err();
    assert_eq!(
        err.to_string(),
        "Parse error: aggregate functions prohibited in generated columns"
    );

    let err = conn
        .execute("CREATE TABLE v(a INTEGER, b AS (nope(a)))")
        .unwrap_err();
    assert_eq!(err.to_string(), "Parse error: no such function: nope");
    Ok(())
}

// ------------------------------- reopening a database without the function

/// A schema that leans on `dbl()` in an expression index, a generated column, a view and a trigger.
fn build_schema_that_needs_dbl(path: &std::path::Path) -> anyhow::Result<()> {
    let db = generated_columns_db(path);
    let conn = db.connect_limbo();
    register_schema_test_functions(&conn)?;

    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (1), (2), (3)")?;
    conn.execute("CREATE INDEX i ON t(dbl(x))")?;

    conn.execute("CREATE TABLE g(a INTEGER, b AS (dbl(a)))")?;
    conn.execute("INSERT INTO g(a) VALUES (5)")?;

    conn.execute("CREATE VIEW v AS SELECT dbl(x) AS d FROM t")?;

    conn.execute("CREATE TABLE plain(y)")?;
    conn.execute("INSERT INTO plain VALUES (9)")?;
    conn.execute("CREATE TABLE log(m)")?;
    conn.execute(
        "CREATE TRIGGER tr AFTER INSERT ON plain BEGIN INSERT INTO log VALUES (dbl(new.y)); END",
    )?;

    conn.close()?;
    Ok(())
}

fn assert_needs_dbl(conn: &Arc<turso_core::Connection>, sql: &str) {
    let err = match conn.prepare(sql) {
        Err(err) => err,
        Ok(mut stmt) => stmt
            .run_with_row_callback(|_| Ok(()))
            .expect_err(&format!("{sql} should not have succeeded")),
    };
    assert_eq!(
        err.to_string(),
        "Parse error: no such function: dbl",
        "unexpected error for {sql}"
    );
}

#[test]
fn a_schema_that_uses_an_unregistered_function_still_loads() -> anyhow::Result<()> {
    let dir = tempfile::TempDir::new()?;
    let path = dir.path().join("missing_udf.db");
    build_schema_that_needs_dbl(&path)?;

    let db = generated_columns_db(&path);
    let conn = db.connect_limbo();

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT y FROM plain");
    assert_eq!(rows, vec![(9,)]);
    conn.execute("CREATE TABLE fresh(z)")?;
    conn.execute("INSERT INTO fresh VALUES (1)")?;

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT x FROM t ORDER BY x");
    assert_eq!(rows, vec![(1,), (2,), (3,)]);
    let indexes: Vec<(String,)> =
        conn.exec_rows("SELECT name FROM sqlite_schema WHERE type = 'index' AND tbl_name = 't'");
    assert_eq!(indexes, vec![("i".to_string(),)]);

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT a FROM g");
    assert_eq!(rows, vec![(5,)]);
    Ok(())
}

#[test]
fn writes_fail_loudly_when_the_schema_needs_an_unregistered_function() -> anyhow::Result<()> {
    let dir = tempfile::TempDir::new()?;
    let path = dir.path().join("missing_udf_writes.db");
    build_schema_that_needs_dbl(&path)?;

    let db = generated_columns_db(&path);
    let conn = db.connect_limbo();

    // A write that cannot maintain the index must fail rather than leave the index half-maintained.
    assert_needs_dbl(&conn, "INSERT INTO t VALUES (4)");
    assert_needs_dbl(&conn, "UPDATE t SET x = 7 WHERE x = 1");
    assert_needs_dbl(&conn, "DELETE FROM t WHERE x = 1");

    assert_needs_dbl(&conn, "SELECT b FROM g");
    assert_needs_dbl(&conn, "INSERT INTO g(a) VALUES (6)");
    assert_needs_dbl(&conn, "SELECT d FROM v");
    assert_needs_dbl(&conn, "INSERT INTO plain VALUES (10)");

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT x FROM t ORDER BY x");
    assert_eq!(rows, vec![(1,), (2,), (3,)]);
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT count(*) FROM log");
    assert_eq!(rows, vec![(0,)]);
    Ok(())
}

/// A query naming the missing function may still be answered from the stored index keys, because
/// matching against the index expression is textual. What must never happen is a wrong answer.
#[test]
fn a_query_over_the_expression_index_is_never_silently_wrong() -> anyhow::Result<()> {
    let dir = tempfile::TempDir::new()?;
    let path = dir.path().join("missing_udf_read.db");
    build_schema_that_needs_dbl(&path)?;

    let db = generated_columns_db(&path);
    let conn = db.connect_limbo();

    match conn.prepare("SELECT x FROM t WHERE dbl(x) = 4") {
        Err(err) => assert_eq!(err.to_string(), "Parse error: no such function: dbl"),
        Ok(mut stmt) => {
            let mut rows = Vec::new();
            stmt.run_with_row_callback(|row| {
                rows.push(row.get::<i64>(0)?);
                Ok(())
            })?;
            assert_eq!(rows, vec![2]);
        }
    }
    Ok(())
}

#[test]
fn registering_the_function_again_makes_everything_work() -> anyhow::Result<()> {
    let dir = tempfile::TempDir::new()?;
    let path = dir.path().join("missing_udf_back.db");
    build_schema_that_needs_dbl(&path)?;

    let db = generated_columns_db(&path);
    let conn = db.connect_limbo();
    register_schema_test_functions(&conn)?;

    conn.execute("INSERT INTO t VALUES (10)")?;
    conn.execute("INSERT INTO g(a) VALUES (6)")?;
    conn.execute("INSERT INTO plain VALUES (11)")?;

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT x FROM t WHERE dbl(x) = 20");
    assert_eq!(rows, vec![(10,)], "the index picked up the new row");
    let rows: Vec<(i64, i64)> = conn.exec_rows("SELECT a, b FROM g ORDER BY a");
    assert_eq!(rows, vec![(5, 10), (6, 12)]);
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT d FROM v ORDER BY d");
    assert_eq!(rows, vec![(2,), (4,), (6,), (20,)]);
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT m FROM log");
    assert_eq!(rows, vec![(22,)], "the trigger fired");

    let checks: Vec<(String,)> = conn.exec_rows("PRAGMA integrity_check");
    assert_eq!(checks, vec![("ok".to_string(),)]);
    Ok(())
}

// ----------------------------------------------------------------- aux data

/// Stashes the "compiled" first argument in aux data and counts how often it had to recompile.
fn register_compiling_matcher(
    conn: &Arc<turso_core::Connection>,
    name: &str,
    counter: Arc<AtomicUsize>,
) -> Result<()> {
    conn.create_scalar_function(
        name,
        2,
        FunctionFlags::DETERMINISTIC,
        move |ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]| -> Result<Value> {
            let compiled = match ctx.get_auxdata(0).and_then(|d| d.downcast_ref::<String>()) {
                Some(compiled) => compiled.clone(),
                None => {
                    counter.fetch_add(1, AtomicOrdering::SeqCst);
                    let compiled = match args[0] {
                        ValueRef::Text(text) => format!("<{}>", text.as_str()),
                        ref other => format!("<{}>", arg_int(other)),
                    };
                    ctx.set_auxdata(0, Box::new(compiled.clone()));
                    compiled
                }
            };
            Ok(Value::build_text(format!(
                "{compiled}{}",
                arg_int(&args[1])
            )))
        },
    )
}

#[turso_macros::test]
fn auxdata_for_a_constant_argument_is_computed_once_for_the_whole_scan(
    tmp_db: TempDatabase,
) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    let compiles = Arc::new(AtomicUsize::new(0));
    register_compiling_matcher(&conn, "matcher", compiles.clone())?;

    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (1), (2), (3), (4), (5)")?;

    let rows: Vec<(String,)> = conn.exec_rows("SELECT matcher('abc', x) FROM t ORDER BY x");
    assert_eq!(
        rows,
        vec![
            ("<abc>1".to_string(),),
            ("<abc>2".to_string(),),
            ("<abc>3".to_string(),),
            ("<abc>4".to_string(),),
            ("<abc>5".to_string(),),
        ]
    );
    assert_eq!(
        compiles.load(AtomicOrdering::SeqCst),
        1,
        "a constant first argument is compiled once and reused for every row"
    );
    Ok(())
}

#[turso_macros::test]
fn auxdata_for_a_column_argument_is_thrown_away_after_every_row(
    tmp_db: TempDatabase,
) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    let compiles = Arc::new(AtomicUsize::new(0));
    register_compiling_matcher(&conn, "matcher", compiles.clone())?;

    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (1), (2), (3), (4), (5)")?;

    let rows: Vec<(String,)> = conn.exec_rows("SELECT matcher(x, x) FROM t ORDER BY x");
    assert_eq!(rows.len(), 5);
    assert_eq!(rows[0], ("<1>1".to_string(),));
    assert_eq!(rows[4], ("<5>5".to_string(),));
    assert_eq!(
        compiles.load(AtomicOrdering::SeqCst),
        5,
        "a column argument is a different value each row, so nothing is kept"
    );
    Ok(())
}

#[turso_macros::test]
fn auxdata_does_not_survive_a_statement_reset(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    let compiles = Arc::new(AtomicUsize::new(0));
    register_compiling_matcher(&conn, "matcher", compiles.clone())?;

    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (1), (2)")?;

    let mut stmt = conn.prepare("SELECT matcher('abc', x) FROM t")?;
    stmt.run_with_row_callback(|_row| Ok(()))?;
    assert_eq!(compiles.load(AtomicOrdering::SeqCst), 1);

    stmt.reset()?;
    stmt.run_with_row_callback(|_row| Ok(()))?;
    assert_eq!(
        compiles.load(AtomicOrdering::SeqCst),
        2,
        "the second run starts with an empty aux-data store"
    );
    Ok(())
}

#[turso_macros::test]
fn two_call_sites_of_the_same_function_keep_separate_auxdata(
    tmp_db: TempDatabase,
) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    let compiles = Arc::new(AtomicUsize::new(0));
    register_compiling_matcher(&conn, "matcher", compiles.clone())?;

    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (1), (2), (3)")?;

    let rows: Vec<(String, String)> =
        conn.exec_rows("SELECT matcher('a', x), matcher('b', x) FROM t ORDER BY x");
    assert_eq!(rows[0], ("<a>1".to_string(), "<b>1".to_string()));
    assert_eq!(rows[2], ("<a>3".to_string(), "<b>3".to_string()));
    assert_eq!(
        compiles.load(AtomicOrdering::SeqCst),
        2,
        "one compile per call site, not one per row"
    );
    Ok(())
}

// -------------------------------------------------------------- error codes

#[turso_macros::test]
fn a_function_picks_the_result_code_the_statement_fails_with(
    tmp_db: TempDatabase,
) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    conn.create_scalar_function(
        "plain_error",
        0,
        FunctionFlags::empty(),
        |ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            Err(ctx.error("something went wrong"))
        },
    )?;
    conn.create_scalar_function(
        "coded_error",
        0,
        FunctionFlags::empty(),
        |ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            Err(ctx.error_code(19, "constraint-ish"))
        },
    )?;
    conn.create_scalar_function(
        "too_big",
        0,
        FunctionFlags::empty(),
        |ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            Err(ctx.error_toobig())
        },
    )?;
    conn.create_scalar_function(
        "no_mem",
        0,
        FunctionFlags::empty(),
        |ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            Err(ctx.error_nomem())
        },
    )?;

    for (sql, code, message) in [
        ("SELECT plain_error()", 1, "something went wrong"),
        ("SELECT coded_error()", 19, "constraint-ish"),
        ("SELECT too_big()", 18, "string or blob too big"),
        ("SELECT no_mem()", 7, "out of memory"),
    ] {
        let mut stmt = conn.prepare(sql)?;
        let err = loop {
            match stmt.step() {
                Ok(StepResult::Row) => panic!("{sql} should not produce a row"),
                Ok(StepResult::IO) => stmt._io().step()?,
                Ok(other) => panic!("{sql} should have failed, got {other:?}"),
                Err(err) => break err,
            }
        };
        assert_eq!(err.to_string(), message, "{sql}");
        assert_eq!(err.sqlite_result_code(), code, "{sql}");
    }

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT 1");
    assert_eq!(rows, vec![(1,)]);
    Ok(())
}

// ------------------------------------------------------------ JSON subtypes

#[turso_macros::test]
fn a_function_sees_the_json_subtype_of_its_argument(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    conn.create_scalar_function(
        "subtype_of",
        1,
        FunctionFlags::DETERMINISTIC | FunctionFlags::SUBTYPE,
        |ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]| -> Result<Value> {
            assert_eq!(ctx.arg_subtype(0), args[0].subtype());
            Ok(Value::from_i64(ctx.arg_subtype(0) as i64))
        },
    )?;

    let rows: Vec<(i64,)> = conn.exec_rows(r#"SELECT subtype_of(json('{"a":1}'))"#);
    assert_eq!(rows, vec![(74,)], "'J', the JSON subtype");

    let rows: Vec<(i64,)> = conn.exec_rows(r#"SELECT subtype_of('{"a":1}')"#);
    assert_eq!(rows, vec![(0,)], "a plain string has no subtype");

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT subtype_of(1)");
    assert_eq!(rows, vec![(0,)]);
    Ok(())
}

#[turso_macros::test]
fn a_function_can_return_json_subtyped_text(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    conn.create_scalar_function(
        "make_json",
        0,
        FunctionFlags::DETERMINISTIC | FunctionFlags::RESULT_SUBTYPE,
        |_ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            Ok(Value::json_text(r#"{"a":1,"b":[2,3]}"#.to_string()))
        },
    )?;
    conn.create_scalar_function(
        "make_text",
        0,
        FunctionFlags::DETERMINISTIC,
        |_ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            Ok(Value::build_text(r#"{"a":1,"b":[2,3]}"#))
        },
    )?;

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT json_extract(make_json(), '$.a')");
    assert_eq!(rows, vec![(1,)]);

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT make_json() ->> '$.b[1]'");
    assert_eq!(rows, vec![(3,)]);

    let rows: Vec<(String,)> = conn.exec_rows("SELECT typeof(make_json())");
    assert_eq!(rows, vec![("text".to_string(),)], "still a text value");

    let quoted: Vec<(String,)> = conn.exec_rows("SELECT json_quote(make_json())");
    let quoted_plain: Vec<(String,)> = conn.exec_rows("SELECT json_quote(make_text())");
    assert_ne!(quoted, quoted_plain);
    assert_eq!(quoted, vec![(r#"{"a":1,"b":[2,3]}"#.to_string(),)]);
    Ok(())
}

// -------------------------------------------------- general (non-JSON) subtypes

/// `tag(x, n)` returns text `x` with subtype `n`; `subtype_of(x)` reports the subtype it was handed.
fn register_subtype_functions(conn: &Arc<turso_core::Connection>) -> Result<()> {
    conn.create_scalar_function(
        "tag",
        2,
        FunctionFlags::DETERMINISTIC | FunctionFlags::RESULT_SUBTYPE,
        |_ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]| -> Result<Value> {
            let ValueRef::Text(text) = &args[0] else {
                return Ok(Value::Null);
            };
            Ok(Value::text_with_subtype(
                text.as_str().to_string(),
                arg_int(&args[1]) as u8,
            ))
        },
    )?;
    conn.create_scalar_function(
        "subtype_of",
        1,
        FunctionFlags::DETERMINISTIC | FunctionFlags::SUBTYPE,
        |ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]| -> Result<Value> {
            assert_eq!(ctx.arg_subtype(0), args[0].subtype());
            Ok(Value::from_i64(ctx.arg_subtype(0) as i64))
        },
    )?;
    Ok(())
}

#[turso_macros::test]
fn a_result_subtype_reaches_the_next_function(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_subtype_functions(&conn)?;

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT subtype_of(tag('hi', 200))");
    assert_eq!(rows, vec![(200,)]);

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT subtype_of(tag('hi', 255))");
    assert_eq!(rows, vec![(255,)]);

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT subtype_of(tag('hi', 0))");
    assert_eq!(rows, vec![(0,)]);

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT subtype_of('hi')");
    assert_eq!(rows, vec![(0,)]);

    let rows: Vec<(String,)> = conn.exec_rows("SELECT tag('hi', 200)");
    assert_eq!(rows, vec![("hi".to_string(),)]);
    let rows: Vec<(String,)> = conn.exec_rows("SELECT typeof(tag('hi', 200))");
    assert_eq!(rows, vec![("text".to_string(),)]);

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT subtype_of(tag(tag('hi', 7), 200))");
    assert_eq!(rows, vec![(200,)]);
    let rows: Vec<(i64,)> =
        conn.exec_rows("SELECT subtype_of(CASE WHEN 1 THEN tag('hi', 200) END)");
    assert_eq!(rows, vec![(200,)]);
    Ok(())
}

#[turso_macros::test]
fn a_result_subtype_reaches_the_caller_reading_the_row(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_subtype_functions(&conn)?;

    let mut stmt = conn.prepare("SELECT tag('hi', 200), tag('hi', 0), 'hi', 1")?;
    let mut seen = Vec::new();
    stmt.run_with_row_callback(|row| {
        seen.push((
            row.get_value(0).subtype(),
            row.get_value(1).subtype(),
            row.get_value(2).subtype(),
            row.get_value(3).subtype(),
        ));
        Ok(())
    })?;
    assert_eq!(seen, vec![(200, 0, 0, 0)]);
    Ok(())
}

#[turso_macros::test]
fn a_subtype_does_not_survive_a_subquery_boundary(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_subtype_functions(&conn)?;

    // SQLite clears the subtype on co-routine output columns (OP_Copy P5=0x0002).
    let rows: Vec<(i64,)> =
        conn.exec_rows("SELECT subtype_of(x) FROM (SELECT tag('hi', 200) AS x)");
    assert_eq!(rows, vec![(0,)]);

    let rows: Vec<(i64,)> =
        conn.exec_rows("WITH t(x) AS (SELECT tag('hi', 200)) SELECT subtype_of(x) FROM t");
    assert_eq!(rows, vec![(0,)]);
    Ok(())
}

#[turso_macros::test]
fn a_subtype_does_not_survive_being_stored(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_subtype_functions(&conn)?;
    conn.execute("CREATE TABLE t (x)")?;
    conn.execute("INSERT INTO t VALUES (tag('hi', 200))")?;
    conn.execute(r#"INSERT INTO t VALUES (json('{"a":1}'))"#)?;

    // A record has nowhere to keep a subtype.
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT subtype_of(x) FROM t");
    assert_eq!(rows, vec![(0,), (0,)]);

    let rows: Vec<(String,)> = conn.exec_rows("SELECT x FROM t LIMIT 1");
    assert_eq!(rows, vec![("hi".to_string(),)]);
    Ok(())
}

#[turso_macros::test]
fn only_text_values_carry_a_subtype(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_subtype_functions(&conn)?;
    // `with_subtype` is a no-op on everything that is not text.
    conn.create_scalar_function(
        "tagged_int",
        0,
        FunctionFlags::DETERMINISTIC | FunctionFlags::RESULT_SUBTYPE,
        |_ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            Ok(Value::from_i64(1).with_subtype(200))
        },
    )?;
    conn.create_scalar_function(
        "tagged_blob",
        0,
        FunctionFlags::DETERMINISTIC | FunctionFlags::RESULT_SUBTYPE,
        |_ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            Ok(Value::from_slice(b"xy")
                .expect("two bytes fit")
                .with_subtype(200))
        },
    )?;
    conn.create_scalar_function(
        "tagged_null",
        0,
        FunctionFlags::DETERMINISTIC | FunctionFlags::RESULT_SUBTYPE,
        |_ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            Ok(Value::Null.with_subtype(200))
        },
    )?;

    for sql in [
        "SELECT subtype_of(tagged_int())",
        "SELECT subtype_of(tagged_blob())",
        "SELECT subtype_of(tagged_null())",
    ] {
        let rows: Vec<(i64,)> = conn.exec_rows(sql);
        assert_eq!(rows, vec![(0,)], "{sql}");
    }

    let rows: Vec<(String,)> =
        conn.exec_rows("SELECT typeof(tagged_int()) || ' ' || typeof(tagged_blob())");
    assert_eq!(rows, vec![("integer blob".to_string(),)]);
    Ok(())
}

#[turso_macros::test]
fn the_json_subtype_is_just_one_subtype_byte(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_subtype_functions(&conn)?;

    // 74 is 'J'.
    let rows: Vec<(i64,)> = conn.exec_rows(r#"SELECT subtype_of(json('{"a":1}'))"#);
    assert_eq!(rows, vec![(74,)]);

    let rows: Vec<(String,)> = conn.exec_rows(r#"SELECT json_quote(tag('{"a":1}', 74))"#);
    assert_eq!(rows, vec![(r#"{"a":1}"#.to_string(),)], "treated as JSON");

    let rows: Vec<(String,)> = conn.exec_rows(r#"SELECT json_quote(tag('{"a":1}', 200))"#);
    assert_eq!(
        rows,
        vec![(r#""{\"a\":1}""#.to_string(),)],
        "subtype 200 means nothing to the JSON functions"
    );

    let rows: Vec<(i64,)> = conn.exec_rows(r#"SELECT json_extract(tag('{"a":1}', 74), '$.a')"#);
    assert_eq!(rows, vec![(1,)]);
    Ok(())
}

// ------------------------------- DIRECTONLY / INNOCUOUS and trusted_schema

/// `unsafe_fn` (DIRECTONLY), `innocuous_fn` (INNOCUOUS) and `plain_fn` (neither), all deterministic doublers.
fn register_safety_functions(conn: &Arc<turso_core::Connection>) -> Result<()> {
    for (name, extra) in [
        ("unsafe_fn", FunctionFlags::DIRECTONLY),
        ("innocuous_fn", FunctionFlags::INNOCUOUS),
        ("plain_fn", FunctionFlags::empty()),
    ] {
        conn.create_scalar_function(
            name,
            1,
            FunctionFlags::DETERMINISTIC | extra,
            |_ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]| -> Result<Value> {
                Ok(Value::from_i64(arg_int(&args[0]) * 2))
            },
        )?;
    }
    Ok(())
}

#[turso_macros::test]
fn a_directonly_function_works_when_called_directly(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_safety_functions(&conn)?;
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT unsafe_fn(1)");
    assert_eq!(rows, vec![(2,)]);
    Ok(())
}

#[turso_macros::test]
fn a_directonly_function_in_a_view_fails_when_the_view_is_read(
    tmp_db: TempDatabase,
) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_safety_functions(&conn)?;
    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (1), (2)")?;

    // SQLite only checks at first use.
    conn.execute("CREATE VIEW v AS SELECT unsafe_fn(x) AS d FROM t")?;

    let err = conn.execute("SELECT * FROM v").unwrap_err();
    assert!(
        err.to_string().contains("unsafe use of unsafe_fn()"),
        "{err}"
    );
    Ok(())
}

#[turso_macros::test]
fn a_directonly_function_in_a_trigger_body_fails_when_the_trigger_fires(
    tmp_db: TempDatabase,
) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_safety_functions(&conn)?;
    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("CREATE TABLE log(m)")?;
    conn.execute(
        "CREATE TRIGGER tr AFTER INSERT ON t BEGIN INSERT INTO log VALUES (unsafe_fn(NEW.x)); END",
    )?;

    let err = conn.execute("INSERT INTO t VALUES (1)").unwrap_err();
    assert!(
        err.to_string().contains("unsafe use of unsafe_fn()"),
        "{err}"
    );

    conn.execute("DROP TRIGGER tr")?;
    conn.execute(
        "CREATE TRIGGER tr AFTER INSERT ON t BEGIN INSERT INTO log VALUES (innocuous_fn(NEW.x)); END",
    )?;
    conn.execute("INSERT INTO t VALUES (3)")?;
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT m FROM log");
    assert_eq!(rows, vec![(6,)]);
    Ok(())
}

#[turso_macros::test]
fn a_directonly_function_in_a_check_constraint_fails_the_insert(
    tmp_db: TempDatabase,
) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_safety_functions(&conn)?;
    conn.execute("CREATE TABLE t(x, CHECK (unsafe_fn(x) < 100))")?;

    let err = conn.execute("INSERT INTO t VALUES (1)").unwrap_err();
    assert!(
        err.to_string().contains("unsafe use of unsafe_fn()"),
        "{err}"
    );
    Ok(())
}

#[turso_macros::test]
fn a_directonly_function_in_a_default_expression_fails_the_insert(
    tmp_db: TempDatabase,
) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_safety_functions(&conn)?;
    conn.execute("CREATE TABLE t(x, y DEFAULT (unsafe_fn(2)))")?;

    for sql in [
        "INSERT INTO t(x) VALUES (1)",
        "INSERT INTO t VALUES (1, DEFAULT)",
        "INSERT INTO t DEFAULT VALUES",
    ] {
        let err = conn.execute(sql).unwrap_err();
        assert!(
            err.to_string().contains("unsafe use of unsafe_fn()"),
            "{sql}: {err}"
        );
    }
    Ok(())
}

#[turso_macros::test]
fn a_directonly_function_may_not_be_indexed(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_safety_functions(&conn)?;
    conn.execute("CREATE TABLE t(x)")?;

    // SQLite marks index expressions as schema SQL while still resolving the CREATE INDEX
    // (`sqlite3ResolveSelfReference`), so building the index is already the first use.
    let err = conn
        .execute("CREATE INDEX i ON t(unsafe_fn(x))")
        .unwrap_err();
    assert!(
        err.to_string().contains("unsafe use of unsafe_fn()"),
        "{err}"
    );

    conn.execute("CREATE INDEX i ON t(innocuous_fn(x))")?;
    conn.execute("INSERT INTO t VALUES (1), (2)")?;
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT x FROM t WHERE innocuous_fn(x) = 4");
    assert_eq!(rows, vec![(2,)]);
    Ok(())
}

#[test]
fn a_directonly_function_may_not_compute_a_generated_column() -> anyhow::Result<()> {
    let dir = tempfile::TempDir::new()?;
    let db = generated_columns_db(&dir.path().join("gencol_directonly.db"));
    let conn = db.connect_limbo();
    register_safety_functions(&conn)?;

    // Like an index expression, SQLite resolves the column expression as schema SQL while
    // still compiling the CREATE TABLE, so the statement itself fails.
    let err = conn
        .execute("CREATE TABLE t(a INTEGER, b AS (unsafe_fn(a)))")
        .unwrap_err();
    assert!(
        err.to_string().contains("unsafe use of unsafe_fn()"),
        "{err}"
    );
    conn.execute("CREATE TABLE t(a INTEGER)")?;
    let err = conn
        .execute("ALTER TABLE t ADD COLUMN c AS (unsafe_fn(a))")
        .unwrap_err();
    assert!(
        err.to_string().contains("unsafe use of unsafe_fn()"),
        "{err}"
    );

    // A plain function is fine while the schema is trusted, and rejected on read once it is not.
    conn.execute("CREATE TABLE u(a INTEGER, b AS (plain_fn(a)))")?;
    conn.execute("INSERT INTO u(a) VALUES (1)")?;
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT b FROM u");
    assert_eq!(rows, vec![(2,)]);
    conn.execute("PRAGMA trusted_schema=OFF")?;
    let err = conn.execute("SELECT b FROM u").unwrap_err();
    assert!(
        err.to_string().contains("unsafe use of plain_fn()"),
        "{err}"
    );
    let err = conn.execute("INSERT INTO u(a) VALUES (2)").unwrap_err();
    assert!(
        err.to_string().contains("unsafe use of plain_fn()"),
        "{err}"
    );
    Ok(())
}

#[turso_macros::test]
fn trusted_schema_is_on_by_default_and_settable(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    let rows: Vec<(i64,)> = conn.exec_rows("PRAGMA trusted_schema");
    assert_eq!(rows, vec![(1,)]);

    conn.execute("PRAGMA trusted_schema=OFF")?;
    let rows: Vec<(i64,)> = conn.exec_rows("PRAGMA trusted_schema");
    assert_eq!(rows, vec![(0,)]);

    conn.execute("PRAGMA trusted_schema=ON")?;
    let rows: Vec<(i64,)> = conn.exec_rows("PRAGMA trusted_schema");
    assert_eq!(rows, vec![(1,)]);
    Ok(())
}

#[turso_macros::test]
fn with_trusted_schema_off_only_innocuous_functions_run_from_a_view(
    tmp_db: TempDatabase,
) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_safety_functions(&conn)?;
    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (1), (2)")?;
    conn.execute("CREATE VIEW vi AS SELECT innocuous_fn(x) AS d FROM t")?;
    conn.execute("CREATE VIEW vp AS SELECT plain_fn(x) AS d FROM t")?;

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT d FROM vp ORDER BY d");
    assert_eq!(rows, vec![(2,), (4,)]);

    conn.execute("PRAGMA trusted_schema=OFF")?;
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT d FROM vi ORDER BY d");
    assert_eq!(rows, vec![(2,), (4,)], "INNOCUOUS is still allowed");

    let err = conn.execute("SELECT * FROM vp").unwrap_err();
    assert!(
        err.to_string().contains("unsafe use of plain_fn()"),
        "{err}"
    );

    conn.execute("CREATE VIEW vb AS SELECT abs(x) AS d FROM t")?;
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT d FROM vb ORDER BY d");
    assert_eq!(rows, vec![(1,), (2,)]);

    conn.execute("PRAGMA trusted_schema=ON")?;
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT d FROM vp ORDER BY d");
    assert_eq!(rows, vec![(2,), (4,)]);
    Ok(())
}

// -------------------------------------------------- argument-count limit

#[turso_macros::test]
fn a_call_with_more_than_127_arguments_is_rejected(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    conn.create_scalar_function(
        "variadic",
        -1,
        FunctionFlags::DETERMINISTIC,
        |_ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]| -> Result<Value> {
            Ok(Value::from_i64(sum_args(args)))
        },
    )?;

    let at_limit = (0..127).map(|_| "1").collect::<Vec<_>>().join(",");
    let rows: Vec<(i64,)> = conn.exec_rows(&format!("SELECT variadic({at_limit})"));
    assert_eq!(rows, vec![(127,)], "exactly the limit is fine");

    let too_many = (0..128).map(|_| "1").collect::<Vec<_>>().join(",");
    let err = conn
        .execute(format!("SELECT variadic({too_many})"))
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("too many arguments on function variadic"),
        "{err}"
    );

    let err = conn.execute(format!("SELECT max({too_many})")).unwrap_err();
    assert!(
        err.to_string()
            .contains("too many arguments on function max"),
        "{err}"
    );
    Ok(())
}

// ------------------------------------------- determinism in custom types

#[test]
fn a_custom_type_expression_rejects_a_non_deterministic_udf() -> anyhow::Result<()> {
    let dir = tempfile::TempDir::new()?;
    let path = dir.path().join("udf_custom_types.db");
    let opts = turso_core::DatabaseOpts::new().with_custom_types(true);
    let db = TempDatabase::new_with_existent_with_opts(&path, opts);
    let conn = db.connect_limbo();
    register_schema_test_functions(&conn)?;

    let err = conn
        .execute("CREATE TYPE rolled BASE integer ENCODE value + roll() DECODE value")
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("non-deterministic functions prohibited in ENCODE expressions"),
        "{err}"
    );

    conn.execute("CREATE TYPE doubled BASE integer ENCODE dbl(value) DECODE value / 2")?;
    conn.execute("CREATE TABLE t(id INTEGER PRIMARY KEY, amount doubled) STRICT")?;
    conn.execute("INSERT INTO t VALUES (1, 21)")?;
    let rows: Vec<(i64, i64)> = conn.exec_rows("SELECT id, amount FROM t");
    assert_eq!(rows, vec![(1, 21)]);
    Ok(())
}

// -------------------------------------- subtypes on values of any type

/// A subtype byte that means nothing to the engine.
const TEST_SUBTYPE: u8 = 42;

/// `st(x)` reports the subtype of whatever it was handed, of any type.
fn register_general_subtype_reader(conn: &Arc<turso_core::Connection>) -> Result<()> {
    conn.create_scalar_function(
        "st",
        1,
        FunctionFlags::DETERMINISTIC | FunctionFlags::SUBTYPE,
        |ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            Ok(Value::from_i64(ctx.arg_subtype(0) as i64))
        },
    )?;
    Ok(())
}

/// One nullary function per value type, each attaching `TEST_SUBTYPE` to its result.
fn register_subtyped_producers(conn: &Arc<turso_core::Connection>) -> Result<()> {
    conn.create_scalar_function(
        "sub_int",
        0,
        FunctionFlags::DETERMINISTIC | FunctionFlags::RESULT_SUBTYPE,
        |ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            ctx.set_result_subtype(TEST_SUBTYPE);
            Ok(Value::from_i64(7))
        },
    )?;
    conn.create_scalar_function(
        "sub_real",
        0,
        FunctionFlags::DETERMINISTIC | FunctionFlags::RESULT_SUBTYPE,
        |ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            ctx.set_result_subtype(TEST_SUBTYPE);
            Ok(Value::from_f64(1.5))
        },
    )?;
    conn.create_scalar_function(
        "sub_blob",
        0,
        FunctionFlags::DETERMINISTIC | FunctionFlags::RESULT_SUBTYPE,
        |ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            ctx.set_result_subtype(TEST_SUBTYPE);
            Ok(Value::from_slice(b"xy").expect("two bytes fit"))
        },
    )?;
    conn.create_scalar_function(
        "sub_null",
        0,
        FunctionFlags::DETERMINISTIC | FunctionFlags::RESULT_SUBTYPE,
        |ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            ctx.set_result_subtype(TEST_SUBTYPE);
            Ok(Value::Null)
        },
    )?;
    conn.create_scalar_function(
        "sub_text",
        0,
        FunctionFlags::DETERMINISTIC | FunctionFlags::RESULT_SUBTYPE,
        |ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            ctx.set_result_subtype(TEST_SUBTYPE);
            Ok(Value::build_text("hi"))
        },
    )?;
    Ok(())
}

/// The single value `sql` returns, so a test can tell NULL from anything else.
fn single_value(conn: &Arc<turso_core::Connection>, sql: &str) -> anyhow::Result<Value> {
    let mut stmt = conn.prepare(sql)?;
    let mut values = Vec::new();
    stmt.run_with_row_callback(|row| {
        values.push(row.get_value(0).clone());
        Ok(())
    })?;
    assert_eq!(values.len(), 1, "{sql} should return exactly one row");
    Ok(values.pop().unwrap())
}

#[turso_macros::test]
fn a_function_can_put_a_subtype_on_a_result_of_any_type(
    tmp_db: TempDatabase,
) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_general_subtype_reader(&conn)?;
    register_subtyped_producers(&conn)?;

    for producer in ["sub_int", "sub_real", "sub_blob", "sub_null", "sub_text"] {
        let rows: Vec<(i64,)> = conn.exec_rows(&format!("SELECT st({producer}())"));
        assert_eq!(rows, vec![(TEST_SUBTYPE as i64,)], "{producer}");
    }

    let rows: Vec<(String,)> = conn.exec_rows(
        "SELECT typeof(sub_int()) || ' ' || typeof(sub_real()) || ' ' || typeof(sub_blob())
                || ' ' || typeof(sub_null()) || ' ' || typeof(sub_text())",
    );
    assert_eq!(rows, vec![("integer real blob null text".to_string(),)]);

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT sub_int() + 1");
    assert_eq!(rows, vec![(8,)]);
    Ok(())
}

#[turso_macros::test]
fn a_subtype_on_a_non_text_value_is_visible_only_through_the_context(
    tmp_db: TempDatabase,
) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_subtyped_producers(&conn)?;
    // `Value::subtype` only sees the byte TEXT keeps inline; `arg_subtype` sees them all.
    conn.create_scalar_function(
        "inline_subtype_of",
        1,
        FunctionFlags::DETERMINISTIC | FunctionFlags::SUBTYPE,
        |_ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]| -> Result<Value> {
            Ok(Value::from_i64(args[0].subtype() as i64))
        },
    )?;

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT inline_subtype_of(sub_text())");
    assert_eq!(rows, vec![(TEST_SUBTYPE as i64,)]);
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT inline_subtype_of(sub_int())");
    assert_eq!(rows, vec![(0,)]);
    Ok(())
}

#[turso_macros::test]
fn a_subtype_on_any_type_survives_a_register_copy(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_general_subtype_reader(&conn)?;
    register_subtyped_producers(&conn)?;

    // A scalar subquery hands its result over with two Copy opcodes. SQLite keeps the subtype here too.
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT st((SELECT sub_int()))");
    assert_eq!(rows, vec![(TEST_SUBTYPE as i64,)]);
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT st((SELECT sub_blob()))");
    assert_eq!(rows, vec![(TEST_SUBTYPE as i64,)]);

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT st(CASE WHEN 1 THEN sub_int() END)");
    assert_eq!(rows, vec![(TEST_SUBTYPE as i64,)]);
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT st(CASE WHEN 1 THEN sub_blob() END)");
    assert_eq!(rows, vec![(TEST_SUBTYPE as i64,)]);
    Ok(())
}

#[turso_macros::test]
fn a_subtype_on_any_type_does_not_survive_being_stored(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_general_subtype_reader(&conn)?;
    register_subtyped_producers(&conn)?;
    conn.execute("CREATE TABLE t (x)")?;
    conn.execute("INSERT INTO t SELECT sub_int()")?;
    conn.execute("INSERT INTO t SELECT sub_blob()")?;

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT st(x) FROM t");
    assert_eq!(rows, vec![(0,), (0,)]);

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT x FROM t LIMIT 1");
    assert_eq!(rows, vec![(7,)]);
    Ok(())
}

#[turso_macros::test]
fn a_subtype_on_any_type_does_not_survive_a_subquery_boundary(
    tmp_db: TempDatabase,
) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_general_subtype_reader(&conn)?;
    register_subtyped_producers(&conn)?;

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT st(x) FROM (SELECT sub_int() AS x)");
    assert_eq!(rows, vec![(0,)]);
    let rows: Vec<(i64,)> = conn.exec_rows("WITH t(x) AS (SELECT sub_blob()) SELECT st(x) FROM t");
    assert_eq!(rows, vec![(0,)]);
    Ok(())
}

#[turso_macros::test]
fn a_value_nobody_tagged_has_no_subtype(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_general_subtype_reader(&conn)?;
    conn.execute("CREATE TABLE t (a, b, c, d)")?;
    conn.execute("INSERT INTO t VALUES (1, 1.5, 'hi', x'0102')")?;

    let rows: Vec<(i64, i64, i64, i64, i64)> =
        conn.exec_rows("SELECT st(1), st(1.5), st('hi'), st(x'0102'), st(NULL)");
    assert_eq!(rows, vec![(0, 0, 0, 0, 0)]);

    let rows: Vec<(i64, i64, i64, i64)> =
        conn.exec_rows("SELECT st(a), st(b), st(c), st(d) FROM t");
    assert_eq!(rows, vec![(0, 0, 0, 0)]);
    Ok(())
}

/// Sums its argument and stamps `TEST_SUBTYPE` on the result, from both `finalize` and `value`.
struct SubtypedSum;

struct SubtypedSumState {
    total: i64,
}

impl AggregateFunction for SubtypedSum {
    fn init(&self, _ctx: &mut FunctionContext<'_>) -> Result<Box<dyn AggregateState>> {
        Ok(Box::new(SubtypedSumState { total: 0 }))
    }

    fn supports_window(&self) -> bool {
        true
    }
}

impl AggregateState for SubtypedSumState {
    fn step(&mut self, _ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]) -> Result<()> {
        self.total += sum_args(args);
        Ok(())
    }

    fn finalize(self: Box<Self>, ctx: &mut FunctionContext<'_>) -> Result<Value> {
        ctx.set_result_subtype(TEST_SUBTYPE);
        Ok(Value::from_i64(self.total))
    }

    fn value(&self, ctx: &mut FunctionContext<'_>) -> Result<Value> {
        ctx.set_result_subtype(TEST_SUBTYPE);
        Ok(Value::from_i64(self.total))
    }

    fn inverse(&mut self, _ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]) -> Result<()> {
        self.total -= sum_args(args);
        Ok(())
    }
}

#[turso_macros::test]
fn an_aggregate_can_put_a_subtype_on_its_result(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_general_subtype_reader(&conn)?;
    conn.create_aggregate_function("subsum", 1, FunctionFlags::empty(), SubtypedSum)?;
    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (1), (2), (3)")?;

    let rows: Vec<(i64, i64)> = conn.exec_rows("SELECT subsum(x), st(subsum(x)) FROM t");
    assert_eq!(rows, vec![(6, TEST_SUBTYPE as i64)]);

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT st(subsum(x)) FROM t WHERE 0");
    assert_eq!(rows, vec![(TEST_SUBTYPE as i64,)]);
    Ok(())
}

#[turso_macros::test]
fn a_window_function_can_put_a_subtype_on_its_result(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_general_subtype_reader(&conn)?;
    conn.create_aggregate_function("subsum", 1, FunctionFlags::empty(), SubtypedSum)?;
    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (1), (2), (3)")?;

    let rows: Vec<(i64, i64)> = conn.exec_rows(
        "SELECT running, st(running) FROM (
             SELECT subsum(x) OVER (ORDER BY x ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW)
                    AS running
             FROM t
         )",
    );
    // The subquery strips the subtype, so read it inside the same query level.
    assert_eq!(rows, vec![(1, 0), (3, 0), (6, 0)]);

    let rows: Vec<(i64,)> = conn.exec_rows(
        "SELECT st(subsum(x) OVER (ORDER BY x ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW))
         FROM t",
    );
    assert_eq!(
        rows,
        vec![
            (TEST_SUBTYPE as i64,),
            (TEST_SUBTYPE as i64,),
            (TEST_SUBTYPE as i64,)
        ]
    );
    Ok(())
}

// --------------------------------------------------------- pointer values

/// `mk()` returns "payload" as a pointer under "mytype"; `rd(x)` asks for "mytype", `rd_wrong(x)` for another name.
fn register_pointer_functions(conn: &Arc<turso_core::Connection>) -> Result<()> {
    conn.create_scalar_function(
        "mk",
        0,
        FunctionFlags::DETERMINISTIC | FunctionFlags::RESULT_SUBTYPE,
        |ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            ctx.set_result_pointer(Arc::new(String::from("payload")), "mytype");
            Ok(Value::from_i64(1))
        },
    )?;
    conn.create_scalar_function(
        "rd",
        1,
        FunctionFlags::DETERMINISTIC | FunctionFlags::SUBTYPE,
        |ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            let Some(object) = ctx.arg_pointer(0, "mytype") else {
                return Ok(Value::Null);
            };
            let text = object
                .downcast_ref::<String>()
                .expect("mytype pointers always hold a String");
            Ok(Value::build_text(text.clone()))
        },
    )?;
    conn.create_scalar_function(
        "rd_wrong",
        1,
        FunctionFlags::DETERMINISTIC | FunctionFlags::SUBTYPE,
        |ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            match ctx.arg_pointer(0, "othertype") {
                Some(_) => Ok(Value::build_text("wrong name matched")),
                None => Ok(Value::Null),
            }
        },
    )?;
    Ok(())
}

#[turso_macros::test]
fn a_pointer_result_reaches_the_next_function(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_pointer_functions(&conn)?;
    register_general_subtype_reader(&conn)?;

    let rows: Vec<(String,)> = conn.exec_rows("SELECT typeof(mk())");
    assert_eq!(rows, vec![("null".to_string(),)]);
    assert_eq!(single_value(&conn, "SELECT mk()")?, Value::Null);

    let rows: Vec<(String,)> = conn.exec_rows("SELECT rd(mk())");
    assert_eq!(rows, vec![("payload".to_string(),)]);

    assert_eq!(single_value(&conn, "SELECT rd_wrong(mk())")?, Value::Null);

    assert_eq!(single_value(&conn, "SELECT rd(NULL)")?, Value::Null);
    assert_eq!(single_value(&conn, "SELECT rd(1)")?, Value::Null);

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT st(mk())");
    assert_eq!(rows, vec![(b'p' as i64,)]);
    Ok(())
}

#[turso_macros::test]
fn a_pointer_does_not_survive_a_subquery_boundary_or_a_table(
    tmp_db: TempDatabase,
) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_pointer_functions(&conn)?;

    assert_eq!(
        single_value(&conn, "SELECT rd(x) FROM (SELECT mk() AS x)")?,
        Value::Null
    );

    conn.execute("CREATE TABLE t (x)")?;
    conn.execute("INSERT INTO t SELECT mk()")?;
    let rows: Vec<(String,)> = conn.exec_rows("SELECT typeof(x) FROM t");
    assert_eq!(rows, vec![("null".to_string(),)]);
    assert_eq!(single_value(&conn, "SELECT rd(x) FROM t")?, Value::Null);
    Ok(())
}

#[turso_macros::test]
fn a_pointer_survives_a_register_copy(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_pointer_functions(&conn)?;

    let rows: Vec<(String,)> = conn.exec_rows("SELECT rd((SELECT mk()))");
    assert_eq!(rows, vec![("payload".to_string(),)]);

    let rows: Vec<(String,)> = conn.exec_rows("SELECT rd(CASE WHEN 1 THEN mk() END)");
    assert_eq!(rows, vec![("payload".to_string(),)]);
    Ok(())
}

#[turso_macros::test]
fn a_bound_pointer_parameter_reaches_a_function(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_pointer_functions(&conn)?;

    let mut stmt = conn.prepare("SELECT rd(?)")?;
    stmt.bind_pointer(
        1.try_into().unwrap(),
        Arc::new(String::from("bound")),
        "mytype",
    )?;
    let mut seen = Vec::new();
    stmt.run_with_row_callback(|row| {
        seen.push(row.get_value(0).clone());
        Ok(())
    })?;
    assert_eq!(seen, vec![Value::build_text("bound")]);

    stmt.reset()?;
    stmt.bind_at(1.try_into().unwrap(), Value::from_i64(5))?;
    let mut seen = Vec::new();
    stmt.run_with_row_callback(|row| {
        seen.push(row.get_value(0).clone());
        Ok(())
    })?;
    assert_eq!(seen, vec![Value::Null]);

    stmt.reset()?;
    stmt.bind_pointer(
        1.try_into().unwrap(),
        Arc::new(String::from("bound again")),
        "mytype",
    )?;
    stmt.clear_bindings();
    let mut seen = Vec::new();
    stmt.run_with_row_callback(|row| {
        seen.push(row.get_value(0).clone());
        Ok(())
    })?;
    assert_eq!(seen, vec![Value::Null]);
    Ok(())
}

/// Reports the subtype its `step` saw on the last row's argument.
struct SubtypeWatchingAggregate;

struct SubtypeWatchingState {
    last_subtype: u8,
}

impl AggregateFunction for SubtypeWatchingAggregate {
    fn init(&self, _ctx: &mut FunctionContext<'_>) -> Result<Box<dyn AggregateState>> {
        Ok(Box::new(SubtypeWatchingState { last_subtype: 0 }))
    }
}

impl AggregateState for SubtypeWatchingState {
    fn step(&mut self, ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]) -> Result<()> {
        self.last_subtype = ctx.arg_subtype(0);
        Ok(())
    }

    fn finalize(self: Box<Self>, _ctx: &mut FunctionContext<'_>) -> Result<Value> {
        Ok(Value::from_i64(self.last_subtype as i64))
    }
}

#[turso_macros::test]
fn an_aggregate_step_sees_the_subtype_of_its_argument(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    register_subtyped_producers(&conn)?;
    conn.create_aggregate_function(
        "seen_subtype",
        1,
        FunctionFlags::SUBTYPE,
        SubtypeWatchingAggregate,
    )?;
    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (1), (2)")?;

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT seen_subtype(sub_int()) FROM t");
    assert_eq!(rows, vec![(TEST_SUBTYPE as i64,)]);

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT seen_subtype(x) FROM t");
    assert_eq!(rows, vec![(0,)]);
    Ok(())
}
