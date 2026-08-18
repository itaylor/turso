use super::{BackendError, DatabaseFileHandle, DatabaseInstance, QueryResult, SqlBackend};
use crate::{
    backends::DefaultDatabaseResolver,
    parser::ast::{Backend, Capability, DatabaseConfig, DatabaseLocation},
};
use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use tempfile::NamedTempFile;
use turso::udf::{AggregateFunction, AggregateState, FunctionFlags};
use turso::{Builder, Connection, Database, Value, ValueRef};

const TURSO_RUST_EXPERIMENTAL_FEATURES: &[&str] = &[
    "attach",
    "index_method",
    "views",
    "custom_types",
    "generated_columns",
    "vacuum",
    "without_rowid",
];

fn apply_turso_experimental_features(mut builder: Builder) -> Builder {
    for feature in TURSO_RUST_EXPERIMENTAL_FEATURES {
        builder = match *feature {
            "attach" => builder.experimental_attach(true),
            "index_method" => builder.experimental_index_method(true),
            "views" => builder.experimental_materialized_views(true),
            "custom_types" => builder.experimental_custom_types(true),
            "generated_columns" => builder.experimental_generated_columns(true),
            "vacuum" => builder.experimental_vacuum(true),
            "without_rowid" => builder.experimental_without_rowid(true),
            _ => unreachable!("unexpected sqltests Rust backend experimental feature"),
        };
    }
    builder
}

// The standard `udf_*` functions a `.sqltest` file gets by asking for
// `@requires udf`. Registered on every connection this backend creates, so a
// test file never registers anything itself. Only this backend advertises
// `Capability::Udf`.

/// Anything that is not a number, NULL included, reads as 0.
fn arg_i64(v: &ValueRef<'_>) -> i64 {
    match v {
        ValueRef::Integer(i) => *i,
        ValueRef::Real(f) => *f as i64,
        _ => 0,
    }
}

fn value_ref_to_value(v: &ValueRef<'_>) -> Value {
    match v {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(i) => Value::Integer(*i),
        ValueRef::Real(f) => Value::Real(*f),
        ValueRef::Text(s) => Value::Text(String::from_utf8_lossy(s).to_string()),
        ValueRef::Blob(b) => Value::Blob(b.to_vec()),
    }
}

/// `udf_sum(x)`: a plain (non-window) aggregate.
struct UdfSum;
struct UdfSumState(i64);

impl AggregateFunction for UdfSum {
    fn init(&self) -> turso::Result<Box<dyn AggregateState>> {
        Ok(Box::new(UdfSumState(0)))
    }
}

impl AggregateState for UdfSumState {
    fn step(&mut self, args: &[ValueRef<'_>]) -> turso::Result<()> {
        self.0 += arg_i64(&args[0]);
        Ok(())
    }

    fn finalize(self: Box<Self>) -> turso::Result<Value> {
        Ok(Value::Integer(self.0))
    }
}

/// `udf_wsum(x)`: a window-capable aggregate.
struct UdfWSum;
struct UdfWSumState(i64);

impl AggregateFunction for UdfWSum {
    fn init(&self) -> turso::Result<Box<dyn AggregateState>> {
        Ok(Box::new(UdfWSumState(0)))
    }

    fn supports_window(&self) -> bool {
        true
    }
}

impl AggregateState for UdfWSumState {
    fn step(&mut self, args: &[ValueRef<'_>]) -> turso::Result<()> {
        self.0 += arg_i64(&args[0]);
        Ok(())
    }

    fn finalize(self: Box<Self>) -> turso::Result<Value> {
        Ok(Value::Integer(self.0))
    }

    fn value(&self) -> turso::Result<Value> {
        Ok(Value::Integer(self.0))
    }

    fn inverse(&mut self, args: &[ValueRef<'_>]) -> turso::Result<()> {
        self.0 -= arg_i64(&args[0]);
        Ok(())
    }
}

/// `udf_count0()`: a zero-argument aggregate, counts rows.
struct UdfCount0;
struct UdfCount0State(i64);

impl AggregateFunction for UdfCount0 {
    fn init(&self) -> turso::Result<Box<dyn AggregateState>> {
        Ok(Box::new(UdfCount0State(0)))
    }
}

impl AggregateState for UdfCount0State {
    fn step(&mut self, _args: &[ValueRef<'_>]) -> turso::Result<()> {
        self.0 += 1;
        Ok(())
    }

    fn finalize(self: Box<Self>) -> turso::Result<Value> {
        Ok(Value::Integer(self.0))
    }
}

/// Registers the standard `udf_*` test set on `conn`.
fn register_udf_functions(conn: &Connection) -> Result<(), BackendError> {
    let to_backend_err = |e: turso::Error| BackendError::CreateDatabase(e.to_string());

    // udf_add(a, b): deterministic scalar, NULL-propagating integer add.
    conn.create_scalar_function(
        "udf_add",
        2,
        FunctionFlags::DETERMINISTIC,
        |args: &[ValueRef<'_>]| -> turso::Result<Value> {
            if args[0].is_null() || args[1].is_null() {
                return Ok(Value::Null);
            }
            Ok(Value::Integer(arg_i64(&args[0]) + arg_i64(&args[1])))
        },
    )
    .map_err(to_backend_err)?;

    // udf_concat(...): variadic deterministic text concat; NULL contributes
    // nothing, every other type is stringified.
    conn.create_scalar_function(
        "udf_concat",
        -1,
        FunctionFlags::DETERMINISTIC,
        |args: &[ValueRef<'_>]| -> turso::Result<Value> {
            let mut out = String::new();
            for arg in args {
                match arg {
                    ValueRef::Null => {}
                    ValueRef::Integer(i) => out.push_str(&i.to_string()),
                    ValueRef::Real(f) => out.push_str(&f.to_string()),
                    ValueRef::Text(t) => out.push_str(&String::from_utf8_lossy(t)),
                    ValueRef::Blob(b) => out.push_str(&String::from_utf8_lossy(b)),
                }
            }
            Ok(Value::Text(out))
        },
    )
    .map_err(to_backend_err)?;

    // udf_nondet(x): non-deterministic — returns x plus a call counter, so
    // two calls with the same argument differ.
    let nondet_calls = Arc::new(AtomicI64::new(0));
    conn.create_scalar_function(
        "udf_nondet",
        1,
        FunctionFlags::empty(),
        move |args: &[ValueRef<'_>]| -> turso::Result<Value> {
            let n = nondet_calls.fetch_add(1, Ordering::SeqCst) + 1;
            Ok(Value::Integer(arg_i64(&args[0]) + n))
        },
    )
    .map_err(to_backend_err)?;

    // udf_direct(x): DIRECTONLY identity — top-level SQL only, rejected from
    // schema SQL.
    conn.create_scalar_function(
        "udf_direct",
        1,
        FunctionFlags::DIRECTONLY,
        |args: &[ValueRef<'_>]| -> turso::Result<Value> { Ok(value_ref_to_value(&args[0])) },
    )
    .map_err(to_backend_err)?;

    // udf_innocuous(x): INNOCUOUS identity — allowed in schema SQL even with
    // `PRAGMA trusted_schema=OFF`.
    conn.create_scalar_function(
        "udf_innocuous",
        1,
        FunctionFlags::INNOCUOUS,
        |args: &[ValueRef<'_>]| -> turso::Result<Value> { Ok(value_ref_to_value(&args[0])) },
    )
    .map_err(to_backend_err)?;

    // udf_plain(x): identity with no flags.
    conn.create_scalar_function(
        "udf_plain",
        1,
        FunctionFlags::empty(),
        |args: &[ValueRef<'_>]| -> turso::Result<Value> { Ok(value_ref_to_value(&args[0])) },
    )
    .map_err(to_backend_err)?;

    // udf_fail(x): always errors.
    conn.create_scalar_function(
        "udf_fail",
        1,
        FunctionFlags::empty(),
        |_args: &[ValueRef<'_>]| -> turso::Result<Value> {
            Err(turso::Error::Error("udf_fail called".to_string()))
        },
    )
    .map_err(to_backend_err)?;

    conn.create_aggregate_function("udf_sum", 1, FunctionFlags::empty(), UdfSum)
        .map_err(to_backend_err)?;
    conn.create_aggregate_function("udf_wsum", 1, FunctionFlags::empty(), UdfWSum)
        .map_err(to_backend_err)?;
    conn.create_aggregate_function("udf_count0", 0, FunctionFlags::empty(), UdfCount0)
        .map_err(to_backend_err)?;

    Ok(())
}

/// Native Rust backend using Turso bindings directly
pub struct RustBackend {
    /// Resolver for default database paths
    default_db_resolver: Option<Arc<dyn DefaultDatabaseResolver>>,
    /// Enable MVCC mode
    mvcc: bool,
}

impl RustBackend {
    pub fn new() -> Self {
        Self {
            default_db_resolver: None,
            mvcc: false,
        }
    }

    /// Set the default database resolver
    pub fn with_default_db_resolver(mut self, resolver: Arc<dyn DefaultDatabaseResolver>) -> Self {
        self.default_db_resolver = Some(resolver);
        self
    }

    /// Enable MVCC mode (experimental journal mode)
    pub fn with_mvcc(mut self, mvcc: bool) -> Self {
        self.mvcc = mvcc;
        self
    }
}

impl Default for RustBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SqlBackend for RustBackend {
    fn name(&self) -> &str {
        "rust"
    }

    fn backend_type(&self) -> Backend {
        Backend::Rust
    }

    fn capabilities(&self) -> HashSet<Capability> {
        HashSet::from_iter([
            Capability::Trigger,
            Capability::MaterializedViews,
            Capability::CustomTypes,
            Capability::Udf,
        ])
    }

    fn supports_snapshots(&self) -> bool {
        true
    }

    async fn create_database(
        &self,
        config: &DatabaseConfig,
    ) -> Result<Box<dyn DatabaseInstance>, BackendError> {
        let (db_path, temp_file) = match &config.location {
            DatabaseLocation::Memory => (":memory:".to_string(), None),
            DatabaseLocation::TempFile => {
                let temp = NamedTempFile::new()
                    .map_err(|e| BackendError::CreateDatabase(e.to_string()))?;
                let path = temp.path().to_string_lossy().to_string();
                (path, Some(temp))
            }
            DatabaseLocation::Path(path) => (path.to_string_lossy().to_string(), None),
            DatabaseLocation::Default | DatabaseLocation::DefaultNoRowidAlias => {
                // Resolve the path using the resolver
                let resolved = self
                    .default_db_resolver
                    .as_ref()
                    .and_then(|r| r.resolve(&config.location))
                    .ok_or_else(|| {
                        BackendError::CreateDatabase(
                            "default database not generated - no resolver configured".to_string(),
                        )
                    })?;
                (resolved.to_string_lossy().to_string(), None)
            }
        };

        // Create the database using the Turso builder
        let db = apply_turso_experimental_features(Builder::new_local(&db_path))
            .build()
            .await
            .map_err(|e| BackendError::CreateDatabase(e.to_string()))?;

        // Connect to the database
        let conn = db
            .connect()
            .map_err(|e| BackendError::CreateDatabase(e.to_string()))?;

        register_udf_functions(&conn)?;

        // Prepend MVCC pragma if enabled (skip for readonly databases; the generated readonly DBs are already in MVCC mode).
        if self.mvcc && !config.readonly {
            let mut rows = conn
                .query("PRAGMA journal_mode = 'mvcc'", ())
                .await
                .map_err(|e| {
                    BackendError::CreateDatabase(format!("failed to enable MVCC mode: {e}"))
                })?;
            // Consume the result row
            let _ = rows.next().await;
        }

        Ok(Box::new(RustDatabaseInstance {
            _db: db,
            conn,
            _temp_file: temp_file,
        }))
    }
}

/// A database instance backed by Turso Rust bindings
pub struct RustDatabaseInstance {
    /// The database handle (kept alive for the connection)
    _db: Database,
    /// The connection to execute queries on
    conn: Connection,
    /// Keep temp file alive - deleted when dropped
    _temp_file: Option<NamedTempFile>,
}

impl RustDatabaseInstance {
    /// Execute SQL (which may contain multiple statements) and collect results
    /// Results are returned from all SELECT/PRAGMA statements, concatenated together
    async fn execute_query(&self, sql: &str) -> Result<QueryResult, turso::Error> {
        let mut all_rows = Vec::new();
        let mut remaining = sql;

        // FIXME: Overhead of having to parse the query twice, but at least we are correct
        // Use turso_parser to properly split SQL into statements
        while !remaining.trim().is_empty() {
            let mut parser = turso_parser::parser::Parser::new(remaining.as_bytes());

            match parser.next() {
                Some(Ok(_cmd)) => {
                    // Get the offset where the parser stopped (after this statement)
                    let offset = parser.offset();

                    // Extract the statement SQL (everything up to the offset)
                    let stmt_sql = &remaining[..offset].trim();

                    if !stmt_sql.is_empty() {
                        // Prepare and execute the statement
                        let mut stmt = self.conn.prepare(stmt_sql).await?;

                        // Use query() which works for both SELECT and non-SELECT statements
                        let mut rows_result = stmt.query(()).await?;
                        while let Some(row) = rows_result.next().await? {
                            let mut row_values = Vec::new();
                            let col_count = row.column_count();
                            for i in 0..col_count {
                                let value = row.get_value(i)?;
                                row_values.push(value_to_string(&value));
                            }
                            all_rows.push(row_values);
                        }
                    }

                    // Move to the remaining SQL
                    remaining = &remaining[offset..];
                }
                Some(Err(e)) => {
                    return Err(turso::Error::Error(format!("Parse error: {e}")));
                }
                None => {
                    // No more statements
                    break;
                }
            }
        }

        Ok(QueryResult::success(all_rows))
    }
}

#[async_trait]
impl DatabaseInstance for RustDatabaseInstance {
    async fn execute_setup(&mut self, sql: &str) -> Result<(), BackendError> {
        // Use execute_batch for setup SQL (may contain multiple statements)
        self.conn
            .execute_batch(sql)
            .await
            .map_err(|e| BackendError::Execute(e.to_string()))?;
        Ok(())
    }

    async fn execute(&mut self, sql: &str) -> Result<QueryResult, BackendError> {
        // Try to execute as a query to capture results
        match self.execute_query(sql).await {
            Ok(result) => Ok(result),
            Err(e) => {
                // Return error as QueryResult (not BackendError)
                // This matches how the test framework expects errors
                Ok(QueryResult::error(e.to_string()))
            }
        }
    }

    async fn close(self: Box<Self>) -> Result<DatabaseFileHandle, BackendError> {
        // Connection and database are dropped automatically
        match self._temp_file {
            Some(tf) => Ok(DatabaseFileHandle::temp(tf)),
            None => Ok(DatabaseFileHandle::none()),
        }
    }
}

/// Convert a Turso Value to its string representation
///
/// This matches the output format expected by the test framework:
/// - NULL becomes empty string (matching CLI list mode)
/// - Integer and Real use standard formatting
/// - Text is used as-is
/// - Blob is converted to UTF-8 string (matching CLI list mode behavior)
fn value_to_string(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Integer(i) => i.to_string(),
        Value::Real(f) => format_real(*f),
        Value::Text(s) => s.clone(),
        Value::Blob(bytes) => String::from_utf8_lossy(bytes).to_string(),
    }
}

/// Format a floating-point number to match SQLite's output
/// SQLite uses %!.15g format (15 significant digits, no trailing zeros)
/// Also used by the matrix oracle so both engines' reals are rendered
/// through the identical formatter, making text comparison value-exact.
pub(crate) fn format_real(f: f64) -> String {
    if f.is_nan() {
        return "NaN".to_string();
    }
    if f.is_infinite() {
        return if f.is_sign_positive() {
            "Inf".to_string()
        } else {
            "-Inf".to_string()
        };
    }

    // Use exponential notation for very large or very small numbers (like SQLite's %g)
    let abs_f = f.abs();
    if abs_f != 0.0 && !(1e-4..1e15).contains(&abs_f) {
        // Use exponential notation with up to 15 significant digits
        let formatted = format!("{f:.14e}");
        // Clean up: remove trailing zeros in mantissa and unnecessary + in exponent
        let formatted = clean_exponential(&formatted);
        return formatted;
    }

    // Check if it's a whole number that can be represented exactly
    if f.fract() == 0.0 {
        format!("{}.0", f as i64)
    } else {
        // Format with 15 significant digits to match SQLite's %!.15g format
        // Then remove trailing zeros after the decimal point
        format_with_significant_digits(f, 15)
    }
}

/// Format a float with a specific number of significant digits, removing trailing zeros
fn format_with_significant_digits(f: f64, sig_digits: usize) -> String {
    if f == 0.0 {
        return "0.0".to_string();
    }

    let abs_f = f.abs();
    // Count digits before decimal point
    let digits_before_decimal = if abs_f >= 1.0 {
        (abs_f.log10().floor() as usize) + 1
    } else {
        0
    };

    // Calculate decimal places needed for the desired significant digits
    let decimal_places = sig_digits.saturating_sub(digits_before_decimal);

    // Format with calculated decimal places
    let formatted = format!("{f:.decimal_places$}");

    // Remove trailing zeros after decimal point, but keep at least one digit after decimal
    if formatted.contains('.') {
        let trimmed = formatted.trim_end_matches('0');
        if trimmed.ends_with('.') {
            format!("{trimmed}0")
        } else {
            trimmed.to_string()
        }
    } else {
        formatted
    }
}

/// Clean up exponential notation to match SQLite's format
fn clean_exponential(s: &str) -> String {
    // Input format: "-1.23456789012340e+18" or "1.23456789012340e-05"
    if let Some(e_pos) = s.find('e') {
        let mantissa = &s[..e_pos];
        let exponent = &s[e_pos + 1..];

        // Remove trailing zeros from mantissa (but keep at least one digit after decimal)
        let mantissa = mantissa.trim_end_matches('0');
        let mantissa = if mantissa.ends_with('.') {
            format!("{mantissa}0")
        } else {
            mantissa.to_string()
        };

        // Parse and format exponent (remove leading zeros and +)
        let exp_num: i32 = exponent.parse().unwrap_or(0);
        format!("{mantissa}e{exp_num:+}")
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_to_string_null() {
        assert_eq!(value_to_string(&Value::Null), "");
    }

    #[test]
    fn test_value_to_string_integer() {
        assert_eq!(value_to_string(&Value::Integer(42)), "42");
        assert_eq!(value_to_string(&Value::Integer(-1)), "-1");
        assert_eq!(value_to_string(&Value::Integer(0)), "0");
    }

    #[test]
    fn test_value_to_string_real() {
        assert_eq!(value_to_string(&Value::Real(3.15)), "3.15");
        assert_eq!(value_to_string(&Value::Real(1.0)), "1.0");
        assert_eq!(value_to_string(&Value::Real(-2.5)), "-2.5");
    }

    #[test]
    fn test_value_to_string_text() {
        assert_eq!(value_to_string(&Value::Text("hello".into())), "hello");
        assert_eq!(value_to_string(&Value::Text("".into())), "");
    }

    #[test]
    fn test_value_to_string_blob() {
        // Blobs are converted to UTF-8 strings (matching CLI list mode)
        assert_eq!(value_to_string(&Value::Blob(b"hello".to_vec())), "hello");
        assert_eq!(
            value_to_string(&Value::Blob(b"independent_jaeckle".to_vec())),
            "independent_jaeckle"
        );
        // Valid UTF-8 sequences are decoded properly
        // 0xDE 0xAD is valid UTF-8 for U+07AD
        assert_eq!(value_to_string(&Value::Blob(vec![0xDE, 0xAD])), "\u{07AD}");
        // Invalid UTF-8 uses replacement character (U+FFFD)
        let result = value_to_string(&Value::Blob(vec![0xFF, 0xFE]));
        assert!(result.contains('\u{FFFD}'));
    }

    #[tokio::test]
    async fn test_create_memory_database() {
        let backend = RustBackend::new();
        let config = DatabaseConfig {
            location: DatabaseLocation::Memory,
            readonly: false,
        };
        let instance = backend.create_database(&config).await;
        assert!(instance.is_ok());
    }

    #[tokio::test]
    async fn test_execute_simple_query() {
        let backend = RustBackend::new();
        let config = DatabaseConfig {
            location: DatabaseLocation::Memory,
            readonly: false,
        };
        let mut instance = backend.create_database(&config).await.unwrap();

        instance
            .execute_setup("CREATE TABLE t(x INTEGER)")
            .await
            .unwrap();
        instance
            .execute_setup("INSERT INTO t VALUES (1), (2)")
            .await
            .unwrap();

        let result = instance
            .execute("SELECT x FROM t ORDER BY x")
            .await
            .unwrap();
        assert!(!result.is_error());
        assert_eq!(result.rows, vec![vec!["1"], vec!["2"]]);
    }

    #[tokio::test]
    async fn test_execute_error_returns_query_result() {
        let backend = RustBackend::new();
        let config = DatabaseConfig {
            location: DatabaseLocation::Memory,
            readonly: false,
        };
        let mut instance = backend.create_database(&config).await.unwrap();

        let result = instance.execute("SELECT * FROM nonexistent").await.unwrap();
        assert!(result.is_error());
        assert!(result.error.unwrap().contains("no such table"));
    }

    #[tokio::test]
    async fn test_execute_multi_statement_insert_then_select() {
        let backend = RustBackend::new();
        let config = DatabaseConfig {
            location: DatabaseLocation::Memory,
            readonly: false,
        };
        let mut instance = backend.create_database(&config).await.unwrap();

        let result = instance
            .execute("CREATE TABLE test (id INTEGER PRIMARY KEY AUTOINCREMENT, col_a TEXT, col_b TEXT, col_c TEXT, col_d TEXT); INSERT INTO test (col_b, col_d, col_a, col_c) VALUES ('1', '2', '3', '4'); SELECT * FROM test;")
            .await
            .unwrap();

        assert!(!result.is_error());
        assert_eq!(result.rows, vec![vec!["1", "3", "1", "4", "2"]]);
    }
}
