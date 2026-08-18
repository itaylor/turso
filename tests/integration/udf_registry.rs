//! Database-level function registration, built-in shadowing where the compiler infers a type from a
//! function name, materialized views, and function name resolution before the optimizer runs.

use crate::common::{ExecRows, TempDatabase};
use std::sync::Arc;
use turso_core::{
    AggregateFunction, AggregateState, Connection, FunctionContext, FunctionFlags, Numeric, Result,
    Value, ValueRef,
};

fn arg_int(arg: &ValueRef<'_>) -> i64 {
    match arg {
        ValueRef::Numeric(Numeric::Integer(i)) => *i,
        ValueRef::Numeric(Numeric::Float(f)) => f64::from(*f) as i64,
        _ => 0,
    }
}

/// Always answers `answer`, so a test can tell which registration ran.
fn constant_function(
    answer: i64,
) -> impl Fn(&mut FunctionContext<'_>, &[ValueRef<'_>]) -> Result<Value> {
    move |_ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| Ok(Value::from_i64(answer))
}

fn int_of(conn: &Arc<Connection>, sql: &str) -> i64 {
    let rows: Vec<(i64,)> = conn.exec_rows(sql);
    assert_eq!(rows.len(), 1, "{sql} should return exactly one row");
    rows[0].0
}

/// The first column of the single row `sql` returns, as a raw `Value`.
fn first_value(conn: &Arc<Connection>, sql: &str) -> anyhow::Result<Value> {
    let mut stmt = conn.prepare(sql)?;
    let mut values = Vec::new();
    stmt.run_with_row_callback(|row| {
        values.push(row.get_value(0).clone());
        Ok(())
    })?;
    assert_eq!(values.len(), 1, "{sql} should return exactly one row");
    Ok(values.pop().unwrap())
}

// ------------------------------------------------- database-level registry

#[test]
fn a_database_level_function_is_inherited_by_every_new_connection() -> anyhow::Result<()> {
    let db = TempDatabase::new("db_level_udf.db");
    db.db.create_scalar_function(
        "answer",
        0,
        FunctionFlags::DETERMINISTIC,
        constant_function(42),
    )?;

    let first = db.connect_limbo();
    let second = db.connect_limbo();
    assert_eq!(int_of(&first, "SELECT answer()"), 42);
    assert_eq!(int_of(&second, "SELECT answer()"), 42);
    Ok(())
}

#[test]
fn a_database_level_function_does_not_reach_connections_that_are_already_open() -> anyhow::Result<()>
{
    let db = TempDatabase::new("db_level_udf_late.db");
    let early = db.connect_limbo();

    db.db.create_scalar_function(
        "answer",
        0,
        FunctionFlags::DETERMINISTIC,
        constant_function(42),
    )?;

    let err = early
        .prepare("SELECT answer()")
        .expect_err("the connection copied the function table before the registration");
    assert_eq!(err.to_string(), "Parse error: no such function: answer");

    let late = db.connect_limbo();
    assert_eq!(int_of(&late, "SELECT answer()"), 42);
    Ok(())
}

#[test]
fn a_connection_registration_shadows_the_database_one() -> anyhow::Result<()> {
    let db = TempDatabase::new("db_level_udf_shadow.db");
    db.db.create_scalar_function(
        "answer",
        0,
        FunctionFlags::DETERMINISTIC,
        constant_function(42),
    )?;

    let own = db.connect_limbo();
    own.create_scalar_function(
        "answer",
        0,
        FunctionFlags::DETERMINISTIC,
        constant_function(7),
    )?;
    let inherited = db.connect_limbo();

    assert_eq!(int_of(&own, "SELECT answer()"), 7);
    assert_eq!(
        int_of(&inherited, "SELECT answer()"),
        42,
        "the connection-level registration is private to the connection that made it"
    );
    Ok(())
}

#[test]
fn removing_a_database_level_function_only_affects_future_connections() -> anyhow::Result<()> {
    let db = TempDatabase::new("db_level_udf_remove.db");
    db.db.create_scalar_function(
        "answer",
        0,
        FunctionFlags::DETERMINISTIC,
        constant_function(42),
    )?;
    let before = db.connect_limbo();
    assert_eq!(int_of(&before, "SELECT answer()"), 42);

    db.db.remove_function("answer", 0)?;

    assert_eq!(
        int_of(&before, "SELECT answer()"),
        42,
        "a connection keeps its own copy of the function table"
    );
    let after = db.connect_limbo();
    let err = after
        .prepare("SELECT answer()")
        .expect_err("connections opened after the removal must not see the function");
    assert_eq!(err.to_string(), "Parse error: no such function: answer");

    db.db.remove_function("answer", 3)?;
    db.db.remove_function("nothing_here", 1)?;
    Ok(())
}

struct SumAggregate;
struct SumAggregateState(i64);

impl AggregateFunction for SumAggregate {
    fn init(&self, _ctx: &mut FunctionContext<'_>) -> Result<Box<dyn AggregateState>> {
        Ok(Box::new(SumAggregateState(0)))
    }
}

impl AggregateState for SumAggregateState {
    fn step(&mut self, _ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]) -> Result<()> {
        self.0 += arg_int(&args[0]);
        Ok(())
    }

    fn finalize(self: Box<Self>, _ctx: &mut FunctionContext<'_>) -> Result<Value> {
        Ok(Value::from_i64(self.0))
    }
}

#[test]
fn a_database_level_aggregate_is_inherited_too() -> anyhow::Result<()> {
    let db = TempDatabase::new("db_level_agg.db");
    db.db
        .create_aggregate_function("mysum", 1, FunctionFlags::empty(), SumAggregate)?;

    let conn = db.connect_limbo();
    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (1), (2), (3)")?;
    assert_eq!(int_of(&conn, "SELECT mysum(x) FROM t"), 6);
    Ok(())
}

#[test]
fn database_level_registration_validates_its_arguments() -> anyhow::Result<()> {
    let db = TempDatabase::new("db_level_validate.db");
    let err = db
        .db
        .create_scalar_function("bad", -2, FunctionFlags::empty(), constant_function(1))
        .expect_err("-2 is not a legal argument count");
    assert!(
        err.to_string().contains("argument count must be between"),
        "{err}"
    );

    let long_name = "x".repeat(256);
    let err = db
        .db
        .create_scalar_function(&long_name, 0, FunctionFlags::empty(), constant_function(1))
        .expect_err("names longer than 255 bytes are rejected");
    assert!(err.to_string().contains("longer than"), "{err}");
    Ok(())
}

#[test]
fn register_function_on_the_database_takes_a_prebuilt_function() -> anyhow::Result<()> {
    let db = TempDatabase::new("db_level_register.db");
    let func = Arc::new(turso_core::ExternalFunc::new_scalar(
        "answer".to_string(),
        0,
        FunctionFlags::DETERMINISTIC,
        Arc::new(constant_function(42)),
    ));
    db.db.register_function(func)?;

    let conn = db.connect_limbo();
    assert_eq!(int_of(&conn, "SELECT answer()"), 42);
    Ok(())
}

// ------------------------------------------------ shadowing and array types

fn custom_types_db(name: &str) -> TempDatabase {
    TempDatabase::builder()
        .with_db_name(name)
        .with_opts(turso_core::DatabaseOpts::new().with_custom_types(true))
        .build()
}

/// `expr_is_array` treats `coalesce`/`ifnull`/`min`/`max` as array pass-throughs; an application function
/// of the same name and arity replaces the built-in, so that inference must not fire.
#[test]
fn a_udf_named_coalesce_is_not_treated_as_an_array_pass_through() -> anyhow::Result<()> {
    let db = custom_types_db("udf_shadows_coalesce.db");
    let conn = db.connect_limbo();
    conn.execute("CREATE TABLE t(id INTEGER PRIMARY KEY, vals integer[]) STRICT")?;
    conn.execute("INSERT INTO t VALUES (1, '{4,5,6}')")?;

    let plain: Vec<(String,)> = conn.exec_rows("SELECT COALESCE(vals, vals) FROM t");
    assert_eq!(plain, vec![("{4,5,6}".to_string(),)]);

    // A blob that is not an encoded array: if the compiler still believed the call produced an array it
    // would hand this to the array decoder.
    conn.create_scalar_function(
        "coalesce",
        2,
        FunctionFlags::DETERMINISTIC,
        |_ctx: &mut FunctionContext<'_>, _args: &[ValueRef<'_>]| -> Result<Value> {
            Ok(Value::Blob(vec![0xff, 0xfe, 0xfd]))
        },
    )?;

    assert_eq!(
        first_value(&conn, "SELECT COALESCE(vals, vals) FROM t")?,
        Value::Blob(vec![0xff, 0xfe, 0xfd]),
        "the application's coalesce ran and its result was returned undecoded"
    );

    // The application function's arity does not cover this call, so the built-in still applies.
    let three: Vec<(String,)> = conn.exec_rows("SELECT COALESCE(NULL, vals, vals) FROM t");
    assert_eq!(three, vec![("{4,5,6}".to_string(),)]);
    Ok(())
}

// -------------------------------------------------------- materialized views

fn views_db(name: &str) -> TempDatabase {
    TempDatabase::builder()
        .with_db_name(name)
        .with_opts(turso_core::DatabaseOpts::new().with_views(true))
        .build()
}

#[test]
fn a_materialized_view_may_not_call_a_user_defined_function() -> anyhow::Result<()> {
    let db = views_db("mv_udf.db");
    let conn = db.connect_limbo();
    conn.create_scalar_function(
        "dbl",
        1,
        FunctionFlags::DETERMINISTIC,
        |_ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]| -> Result<Value> {
            Ok(Value::from_i64(arg_int(&args[0]) * 2))
        },
    )?;
    conn.execute("CREATE TABLE t(x)")?;

    let err = conn
        .execute("CREATE MATERIALIZED VIEW v AS SELECT dbl(x) AS d FROM t")
        .expect_err("a UDF in a materialized view must be rejected");
    assert_eq!(
        err.to_string(),
        "Parse error: user-defined functions are not supported in materialized views: dbl()"
    );

    let err = conn
        .execute("CREATE MATERIALIZED VIEW v AS SELECT x FROM t WHERE x IN (SELECT dbl(x) FROM t)")
        .expect_err("a UDF anywhere in the view body must be rejected");
    assert_eq!(
        err.to_string(),
        "Parse error: user-defined functions are not supported in materialized views: dbl()"
    );

    // A regular view is compiled per statement, on the connection that runs it.
    conn.execute("CREATE VIEW plain AS SELECT dbl(x) AS d FROM t")?;
    conn.execute("INSERT INTO t VALUES (3)")?;
    assert_eq!(int_of(&conn, "SELECT d FROM plain"), 6);
    Ok(())
}

/// The view body names a built-in that this connection has replaced: the DBSP circuit would compute
/// the built-in while every other query computes the application's.
#[test]
fn a_materialized_view_may_not_name_a_builtin_the_connection_replaced() -> anyhow::Result<()> {
    let db = views_db("mv_shadowed_builtin.db");
    let conn = db.connect_limbo();
    conn.create_scalar_function(
        "abs",
        1,
        FunctionFlags::DETERMINISTIC,
        constant_function(4242),
    )?;
    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (-1), (-2)")?;

    let err = conn
        .execute("CREATE MATERIALIZED VIEW v AS SELECT abs(x) AS d FROM t")
        .expect_err("a shadowed built-in in a materialized view must be rejected");
    assert_eq!(
        err.to_string(),
        "Parse error: user-defined functions are not supported in materialized views: abs()"
    );
    Ok(())
}

#[test]
fn a_materialized_view_still_takes_builtins_the_connection_left_alone() -> anyhow::Result<()> {
    let db = views_db("mv_builtin_ok.db");
    let conn = db.connect_limbo();
    // Registered at two arguments, so a one-argument `abs(x)` still finds the built-in.
    conn.create_scalar_function(
        "abs",
        2,
        FunctionFlags::DETERMINISTIC,
        constant_function(4242),
    )?;
    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (-1), (-2)")?;

    conn.execute("CREATE MATERIALIZED VIEW v AS SELECT abs(x) AS d FROM t")?;
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT d FROM v ORDER BY d");
    assert_eq!(rows, vec![(1,), (2,)]);
    Ok(())
}

// --------------------------------------- name resolution before optimization

fn build_schema_that_needs_dbl(path: &std::path::Path) -> anyhow::Result<()> {
    let db = TempDatabase::builder()
        .with_db_path(path)
        .with_opts(turso_core::DatabaseOpts::new().with_generated_columns(true))
        .build();
    let conn = db.connect_limbo();
    conn.create_scalar_function(
        "dbl",
        1,
        FunctionFlags::DETERMINISTIC,
        |_ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]| -> Result<Value> {
            Ok(Value::from_i64(arg_int(&args[0]) * 2))
        },
    )?;
    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (1), (2), (3)")?;
    conn.execute("CREATE INDEX i ON t(dbl(x))")?;
    conn.close()?;
    Ok(())
}

/// SQLite resolves function names before the optimizer can turn `WHERE dbl(x) = 4` into an
/// expression-index seek, so the statement fails even though nothing would have evaluated the call.
#[test]
fn a_statement_naming_an_unregistered_function_fails_even_when_an_index_answers_it(
) -> anyhow::Result<()> {
    let dir = tempfile::TempDir::new()?;
    let path = dir.path().join("resolve_before_optimize.db");
    build_schema_that_needs_dbl(&path)?;

    let db = TempDatabase::builder()
        .with_db_path(&path)
        .with_opts(turso_core::DatabaseOpts::new().with_generated_columns(true))
        .build();
    let conn = db.connect_limbo();

    let err = conn
        .prepare("SELECT x FROM t WHERE dbl(x) = 4")
        .expect_err("the name must be resolved before the optimizer sees the expression");
    assert_eq!(err.to_string(), "Parse error: no such function: dbl");
    Ok(())
}

#[test]
fn an_unregistered_function_fails_wherever_the_statement_names_it() -> anyhow::Result<()> {
    let db = TempDatabase::new("resolve_everywhere.db");
    let conn = db.connect_limbo();
    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (1), (2), (3)")?;

    for sql in [
        "SELECT nope(x) FROM t",
        "SELECT x FROM t WHERE nope(x) = 1",
        "SELECT x FROM t ORDER BY nope(x)",
        "SELECT x FROM t GROUP BY x HAVING sum(nope(x)) > 0",
        "SELECT x FROM t WHERE x IN (SELECT nope(x) FROM t)",
        "SELECT (SELECT nope(1))",
        "INSERT INTO t VALUES (nope(1))",
        "UPDATE t SET x = nope(x)",
        "DELETE FROM t WHERE nope(x)",
        "SELECT nope(*) FROM t",
    ] {
        let err = match conn.prepare(sql) {
            Err(err) => err,
            Ok(mut stmt) => stmt
                .run_with_row_callback(|_| Ok(()))
                .expect_err(&format!("{sql} should not succeed")),
        };
        assert_eq!(
            err.to_string(),
            "Parse error: no such function: nope",
            "for {sql}"
        );
    }
    Ok(())
}

#[test]
fn a_known_name_called_with_the_wrong_arity_says_so() -> anyhow::Result<()> {
    let db = TempDatabase::new("resolve_wrong_arity.db");
    let conn = db.connect_limbo();
    conn.create_scalar_function("dbl", 1, FunctionFlags::DETERMINISTIC, constant_function(2))?;
    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (1)")?;

    let err = conn
        .prepare("SELECT x FROM t WHERE dbl(x, x) = 4")
        .expect_err("dbl takes one argument");
    assert_eq!(
        err.to_string(),
        "Parse error: wrong number of arguments to function dbl()"
    );
    Ok(())
}

/// Forms whose call-site argument list is not the function's, or whose names live outside the
/// scalar/aggregate table, must survive bind-time resolution.
#[test]
fn bind_time_resolution_leaves_the_special_call_forms_alone() -> anyhow::Result<()> {
    let db = TempDatabase::new("resolve_special_forms.db");
    let conn = db.connect_limbo();
    conn.execute("CREATE TABLE t(x)")?;
    conn.execute("INSERT INTO t VALUES (1), (2), (3)")?;

    let rows: Vec<(i64, i64)> = conn.exec_rows("SELECT x, row_number() OVER (ORDER BY x) FROM t");
    assert_eq!(rows, vec![(1, 1), (2, 2), (3, 3)]);
    let rows: Vec<(i64, i64)> = conn.exec_rows("SELECT x, sum(x) OVER (ORDER BY x) FROM t");
    assert_eq!(rows, vec![(1, 1), (2, 3), (3, 6)]);

    let rows: Vec<(i64,)> =
        conn.exec_rows("SELECT x FROM t GROUP BY x HAVING count(*) = 1 ORDER BY x");
    assert_eq!(rows, vec![(1,), (2,), (3,)]);
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT x FROM t WHERE likelihood(x = 1, 0.5)");
    assert_eq!(rows, vec![(1,)]);
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT x FROM t WHERE likely(x = 1)");
    assert_eq!(rows, vec![(1,)]);

    // WITHIN GROUP: the written arguments are not the aggregate's arguments.
    let rows: Vec<(i64,)> =
        conn.exec_rows("SELECT percentile_disc(0.5) WITHIN GROUP (ORDER BY x) FROM t");
    assert_eq!(rows, vec![(2,)]);

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT value FROM generate_series(1, 3)");
    assert_eq!(rows, vec![(1,), (2,), (3,)]);

    conn.execute("CREATE TABLE guard(y)")?;
    conn.execute(
        "CREATE TRIGGER tr BEFORE INSERT ON guard
         BEGIN SELECT RAISE(ABORT, 'nope') WHERE new.y < 0; END",
    )?;
    conn.execute("INSERT INTO guard VALUES (1)")?;
    let err = conn
        .execute("INSERT INTO guard VALUES (-1)")
        .expect_err("the trigger should abort the insert");
    assert!(err.to_string().contains("nope"), "{err}");
    Ok(())
}

/// A schema naming an unregistered function still loads; only statements that need it fail.
#[test]
fn a_schema_that_names_an_unregistered_function_still_loads() -> anyhow::Result<()> {
    let dir = tempfile::TempDir::new()?;
    let path = dir.path().join("schema_load_tolerance.db");
    build_schema_that_needs_dbl(&path)?;

    let db = TempDatabase::builder()
        .with_db_path(&path)
        .with_opts(turso_core::DatabaseOpts::new().with_generated_columns(true))
        .build();
    let conn = db.connect_limbo();

    let rows: Vec<(i64,)> = conn.exec_rows("SELECT x FROM t ORDER BY x");
    assert_eq!(rows, vec![(1,), (2,), (3,)]);

    let err = conn
        .execute("INSERT INTO t VALUES (4)")
        .expect_err("index maintenance needs the function");
    assert_eq!(err.to_string(), "Parse error: no such function: dbl");
    Ok(())
}
