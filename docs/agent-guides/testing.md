---
name: testing
description: Test types, when to use each, how to write and run tests
---
# Testing Guide

## Test Types & When to Use

| Type | Location | Use Case |
|------|----------|----------|
| `.sqltest` | `sqlite/conformance/sqlite-sqltests/` | SQL compatibility. **Preferred for new tests** |
| TCL `.test` | `testing/` | Legacy SQL compat (being phased out) |
| Rust integration | `tests/integration/` | Regression tests, complex scenarios |
| Fuzz | `tests/fuzz/` | Complex features, edge case discovery |

**Note:** TCL tests are being phased out in favor of the `.sqltest` suites in `sqlite/conformance/`. The `.sqltest` format allows the same test cases to run against multiple backends (CLI, Rust bindings, etc.).

## Running Tests

```bash
# Main test suite (TCL compat, sqlite3 compat, Python wrappers)
make test

# Single TCL test
make test-single TEST=select.test

# SQL test runner
make -C sqlite/conformance run-cli

# Rust unit/integration tests (full workspace)
cargo test
```

## Writing Tests

### .sqltest (Preferred)
```sql
@database :memory:

@query
SELECT 1 + 1;
@expected
2
```
Location: `sqlite/conformance/sqlite-sqltests/*.sqltest`

### TCL
```tcl
do_execsql_test_on_specific_db {:memory:} test-name {
  SELECT 1 + 1;
} {2}
```
Location: `testing/*.test`

### Rust Integration
```rust
// tests/integration/test_foo.rs
#[test]
fn test_something() {
    let conn = Connection::open_in_memory().unwrap();
    // ...
}
```

### UDFs in .sqltest

A `.sqltest` file can require `@requires-file udf "<reason>"` (or `@requires
udf "<reason>"` on a single test) to use the standard `udf_*` test set:
`udf_add(a, b)` (deterministic, NULL-propagating add), `udf_concat(...)`
(variadic, deterministic text concat), `udf_nondet(x)` (non-deterministic,
adds a call counter), `udf_direct(x)`/`udf_innocuous(x)`/`udf_plain(x)`
(identity functions carrying `DIRECTONLY`/`INNOCUOUS`/no flags), `udf_fail(x)`
(always errors with `udf_fail called`), and the aggregates `udf_sum(x)`
(plain), `udf_wsum(x)` (window-capable) and `udf_count0()` (zero-argument).
Only the rust backend registers them (right after connecting, in
`testing/sqltest/src/backends/rust.rs`), so files (or tests) that require
`udf` are skipped on the CLI, JS and PG backends — there is no way to call
`Connection::create_scalar_function` against a `tursodb` subprocess or those
other bindings' test harnesses. See
`sqlite/conformance/turso-sqltests/udf.sqltest` for the full behavior of each
function and example usage.

## Key Rules

- Every functional change needs a test
- Test must fail without change, pass with it
- Prefer in-memory DBs: `:memory:` (sqltest) or `{:memory:}` (TCL)
- Don't invent new test formats. Follow existing patterns.
- Use minimal tests, i.e. the bare minimum that triggers the behaviour. No types, no PKs, etc. unless necessary.
- Use column names a, b, c... table names t1, t2, t3... view names v1, v2, v3... 
- Write tests first when possible
- If tasked with identifying a reproducer for a bug, strongly prefer using only user-facing APIs. Manipulating DB internals to artificially trigger a condition, or asserting internal state, is a bad reproducer.
- A reproducer must serve as a regression test once the bug is fixed.


## Test Database Schema

`testing/testing.db` has `users` and `products` tables. See [docs/testing.md](../testing.md) for schema.

## Logging During Tests

```bash
RUST_LOG=none,turso_core=trace make test
```
Output: `testing/test.log`. Warning: very verbose.
