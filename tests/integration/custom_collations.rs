//! Connection-owned custom collations named in the schema. Like SQLite: creating the schema needs the
//! collation registered; loading it back does not; any statement that would have to compare with it
//! fails with "no such collation sequence" instead of silently comparing as BINARY.

use crate::common::{ExecRows, TempDatabase};
use std::cmp::Ordering;
use std::sync::Arc;
use turso_core::Connection;

/// Reverse byte order, so its effect cannot be confused with BINARY.
unsafe extern "C" fn reverse_collation(
    _context: usize,
    left_ptr: *const u8,
    left_len: usize,
    right_ptr: *const u8,
    right_len: usize,
) -> i32 {
    let left = unsafe { std::slice::from_raw_parts(left_ptr, left_len) };
    let right = unsafe { std::slice::from_raw_parts(right_ptr, right_len) };
    match right.cmp(left) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}

/// Case-insensitive after dropping trailing spaces, so it makes values equal that BINARY keeps apart.
unsafe extern "C" fn nocase_trim_collation(
    _context: usize,
    left_ptr: *const u8,
    left_len: usize,
    right_ptr: *const u8,
    right_len: usize,
) -> i32 {
    let left = unsafe { std::slice::from_raw_parts(left_ptr, left_len) };
    let right = unsafe { std::slice::from_raw_parts(right_ptr, right_len) };
    let normalize = |bytes: &[u8]| {
        String::from_utf8_lossy(bytes)
            .trim_end()
            .to_ascii_lowercase()
    };
    match normalize(left).cmp(&normalize(right)) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}

fn register_reverse(conn: &Arc<Connection>, name: &str) {
    conn.register_external_collation(name.to_string(), 0, reverse_collation, None);
}

fn register_nocase_trim(conn: &Arc<Connection>, name: &str) {
    conn.register_external_collation(name.to_string(), 0, nocase_trim_collation, None);
}

fn error_of(conn: &Arc<Connection>, sql: &str) -> String {
    conn.execute(sql)
        .expect_err(&format!("{sql} should have failed"))
        .to_string()
}

fn texts(conn: &Arc<Connection>, sql: &str) -> Vec<String> {
    let rows: Vec<(String,)> = conn.exec_rows(sql);
    rows.into_iter().map(|(value,)| value).collect()
}

// ------------------------------------------------------- CREATE time

#[test]
fn creating_a_schema_that_names_an_unregistered_collation_fails() -> anyhow::Result<()> {
    let db = TempDatabase::new("collation_create_unknown.db");
    let conn = db.connect_limbo();

    let err = error_of(&conn, "CREATE TABLE t(x TEXT COLLATE mycoll)");
    assert!(
        err.contains("no such collation sequence: mycoll"),
        "unexpected error: {err}"
    );

    conn.execute("CREATE TABLE t(x TEXT)")?;
    let err = error_of(&conn, "CREATE INDEX i ON t(x COLLATE mycoll)");
    assert!(
        err.contains("no such collation sequence: mycoll"),
        "unexpected error: {err}"
    );

    let err = error_of(
        &conn,
        "CREATE TABLE u(x TEXT, PRIMARY KEY(x COLLATE mycoll))",
    );
    assert!(
        err.contains("no such collation sequence: mycoll"),
        "unexpected error: {err}"
    );
    Ok(())
}

#[test]
fn a_registered_collation_can_be_named_in_a_column_and_an_index() -> anyhow::Result<()> {
    let db = TempDatabase::new("collation_create_registered.db");
    let conn = db.connect_limbo();
    register_reverse(&conn, "revcoll");

    conn.execute("CREATE TABLE t(x TEXT COLLATE revcoll)")?;
    conn.execute("CREATE INDEX i ON t(x)")?;
    conn.execute("INSERT INTO t VALUES ('a'), ('b'), ('c')")?;

    // Under `revcoll` the order is 'c' < 'b' < 'a'.
    assert_eq!(texts(&conn, "SELECT x FROM t ORDER BY x"), ["c", "b", "a"]);
    assert_eq!(texts(&conn, "SELECT x FROM t WHERE x > 'b'"), ["a"]);
    let ok: Vec<(String,)> = conn.exec_rows("PRAGMA integrity_check");
    assert_eq!(ok, vec![("ok".to_string(),)]);
    Ok(())
}

#[test]
fn an_index_can_name_a_collation_the_column_does_not_have() -> anyhow::Result<()> {
    let db = TempDatabase::new("collation_index_collate.db");
    let conn = db.connect_limbo();
    register_nocase_trim(&conn, "trimcoll");

    conn.execute("CREATE TABLE t(x TEXT)")?;
    conn.execute("CREATE UNIQUE INDEX i ON t(x COLLATE trimcoll)")?;
    conn.execute("INSERT INTO t VALUES ('Alpha')")?;

    let err = error_of(&conn, "INSERT INTO t VALUES ('alpha  ')");
    assert!(err.contains("UNIQUE constraint failed"), "{err}");

    conn.execute("INSERT INTO t VALUES ('beta')")?;
    let ok: Vec<(String,)> = conn.exec_rows("PRAGMA integrity_check");
    assert_eq!(ok, vec![("ok".to_string(),)]);
    Ok(())
}

// ------------------------------------------------- loading the schema back

#[test]
fn a_connection_without_the_collation_loads_the_schema_but_cannot_compare() -> anyhow::Result<()> {
    let db = TempDatabase::new("collation_cross_connection.db");
    let creator = db.connect_limbo();
    register_reverse(&creator, "revcoll");
    creator.execute("CREATE TABLE t(id INTEGER PRIMARY KEY, x TEXT COLLATE revcoll, y TEXT)")?;
    creator.execute("INSERT INTO t VALUES (1, 'a', 'one'), (2, 'b', 'two')")?;

    let other = db.connect_limbo();

    assert_eq!(texts(&other, "SELECT y FROM t ORDER BY id"), ["one", "two"]);
    assert_eq!(texts(&other, "SELECT x FROM t ORDER BY id"), ["a", "b"]);
    let schema: Vec<(String,)> = other.exec_rows("SELECT sql FROM sqlite_schema WHERE name = 't'");
    assert!(
        schema[0].0.contains("revcoll"),
        "stored schema lost the collation name: {}",
        schema[0].0
    );

    for sql in [
        "SELECT x FROM t ORDER BY x",
        "SELECT x FROM t WHERE x = 'a'",
        "SELECT count(*) FROM t GROUP BY x",
        "SELECT min(x) FROM t",
        "SELECT DISTINCT x FROM t",
        "SELECT x FROM t UNION SELECT 'z'",
    ] {
        let err = error_of(&other, sql);
        assert!(
            err.contains("no such collation sequence: revcoll"),
            "`{sql}` gave: {err}"
        );
    }

    register_reverse(&other, "revcoll");
    assert_eq!(texts(&other, "SELECT x FROM t ORDER BY x"), ["b", "a"]);
    Ok(())
}

#[test]
fn writes_fail_when_an_index_needs_a_collation_the_connection_lacks() -> anyhow::Result<()> {
    let db = TempDatabase::new("collation_index_writes.db");
    let creator = db.connect_limbo();
    register_reverse(&creator, "revcoll");
    creator.execute("CREATE TABLE t(id INTEGER PRIMARY KEY, x TEXT, y TEXT)")?;
    creator.execute("CREATE INDEX i ON t(x COLLATE revcoll)")?;
    creator.execute("INSERT INTO t VALUES (1, 'a', 'one'), (2, 'b', 'two')")?;

    let other = db.connect_limbo();

    assert_eq!(texts(&other, "SELECT y FROM t ORDER BY id"), ["one", "two"]);

    // Writing BINARY-ordered keys into an index built with `revcoll` would leave it permanently mis-ordered.
    for sql in [
        "INSERT INTO t VALUES (3, 'c', 'three')",
        "UPDATE t SET x = 'z' WHERE id = 1",
        "DELETE FROM t WHERE id = 1",
        "PRAGMA integrity_check",
    ] {
        let err = error_of(&other, sql);
        assert!(
            err.contains("no such collation sequence: revcoll"),
            "`{sql}` gave: {err}"
        );
    }

    // `x` itself has no collation, so only an explicit COLLATE orders by `revcoll`.
    assert_eq!(
        texts(&creator, "SELECT x FROM t ORDER BY x COLLATE revcoll"),
        ["b", "a"]
    );

    register_reverse(&other, "revcoll");
    other.execute("INSERT INTO t VALUES (3, 'c', 'three')")?;
    assert_eq!(
        texts(&other, "SELECT x FROM t ORDER BY x COLLATE revcoll"),
        ["c", "b", "a"]
    );
    let ok: Vec<(String,)> = other.exec_rows("PRAGMA integrity_check");
    assert_eq!(ok, vec![("ok".to_string(),)]);
    Ok(())
}

#[test]
fn reopening_the_file_without_the_collation_still_loads_the_schema() -> anyhow::Result<()> {
    let db = TempDatabase::new("collation_reopen.db");
    {
        let conn = db.connect_limbo();
        register_reverse(&conn, "revcoll");
        conn.execute("CREATE TABLE t(id INTEGER PRIMARY KEY, x TEXT COLLATE revcoll)")?;
        conn.execute("CREATE INDEX i ON t(x)")?;
        conn.execute("INSERT INTO t VALUES (1, 'a'), (2, 'b')")?;
        conn.close()?;
    }

    let reopened = TempDatabase::new_with_existent(&db.path);
    let conn = reopened.connect_limbo();
    let schema: Vec<(String,)> = conn.exec_rows("SELECT sql FROM sqlite_schema WHERE name = 't'");
    assert!(schema[0].0.contains("revcoll"), "{}", schema[0].0);
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT id FROM t ORDER BY id");
    assert_eq!(rows, vec![(1,), (2,)]);

    let err = error_of(&conn, "SELECT x FROM t ORDER BY x");
    assert!(err.contains("no such collation sequence: revcoll"), "{err}");

    register_reverse(&conn, "revcoll");
    assert_eq!(texts(&conn, "SELECT x FROM t ORDER BY x"), ["b", "a"]);
    Ok(())
}

#[test]
fn unregistering_a_collation_the_schema_needs_makes_writes_fail_again() -> anyhow::Result<()> {
    let db = TempDatabase::new("collation_unregister.db");
    let conn = db.connect_limbo();
    register_reverse(&conn, "revcoll");
    conn.execute("CREATE TABLE t(id INTEGER PRIMARY KEY, x TEXT)")?;
    conn.execute("CREATE INDEX i ON t(x COLLATE revcoll)")?;
    conn.execute("INSERT INTO t VALUES (1, 'a')")?;

    conn.unregister_external_collation("revcoll");
    let err = error_of(&conn, "INSERT INTO t VALUES (2, 'b')");
    assert!(err.contains("no such collation sequence: revcoll"), "{err}");

    register_reverse(&conn, "revcoll");
    conn.execute("INSERT INTO t VALUES (2, 'b')")?;
    assert_eq!(
        texts(&conn, "SELECT x FROM t ORDER BY x COLLATE revcoll"),
        ["b", "a"]
    );
    Ok(())
}

#[test]
fn two_connections_may_define_the_same_collation_name_differently() -> anyhow::Result<()> {
    // The schema stores only the name; two connections giving it different callbacks each see their own
    // ordering, exactly as in SQLite.
    let db = TempDatabase::new("collation_conflicting_definitions.db");
    let reverse_conn = db.connect_limbo();
    register_reverse(&reverse_conn, "shared");
    reverse_conn.execute("CREATE TABLE t(x TEXT COLLATE shared)")?;
    reverse_conn.execute("INSERT INTO t VALUES ('a'), ('b')")?;

    let trim_conn = db.connect_limbo();
    register_nocase_trim(&trim_conn, "shared");

    assert_eq!(
        texts(&reverse_conn, "SELECT x FROM t ORDER BY x"),
        ["b", "a"]
    );
    assert_eq!(texts(&trim_conn, "SELECT x FROM t ORDER BY x"), ["a", "b"]);
    let trim_match: Vec<(i64,)> = trim_conn.exec_rows("SELECT count(*) FROM t WHERE x = 'A  '");
    assert_eq!(trim_match, vec![(1,)]);
    let reverse_match: Vec<(i64,)> =
        reverse_conn.exec_rows("SELECT count(*) FROM t WHERE x = 'A  '");
    assert_eq!(reverse_match, vec![(0,)]);
    Ok(())
}

// ------------------------------------------- collation through a function call

#[test]
fn a_function_call_does_not_pass_its_arguments_collation_to_a_comparison() -> anyhow::Result<()> {
    // `lower(x) = 'A'` compares with BINARY even when `x` has a collation; an explicit COLLATE inside
    // the arguments still propagates out.
    let db = TempDatabase::new("collation_function_calls.db");
    let conn = db.connect_limbo();
    register_nocase_trim(&conn, "trimcoll");
    conn.execute("CREATE TABLE t(id INTEGER PRIMARY KEY, x TEXT COLLATE trimcoll)")?;
    conn.execute("INSERT INTO t VALUES (1, 'alpha'), (2, 'ALPHA  ')")?;

    let count = |sql: &str| -> i64 {
        let rows: Vec<(i64,)> = conn.exec_rows(sql);
        rows[0].0
    };

    assert_eq!(count("SELECT count(*) FROM t WHERE x = 'Alpha'"), 2);
    assert_eq!(count("SELECT count(*) FROM t WHERE ltrim(x) = 'Alpha'"), 0);
    assert_eq!(
        count("SELECT count(*) FROM t WHERE ltrim(x COLLATE trimcoll) = 'Alpha'"),
        2
    );
    Ok(())
}

#[test]
fn a_schema_collation_drives_every_kind_of_comparison() -> anyhow::Result<()> {
    // `trimcoll` makes 'alpha' and 'ALPHA  ' equal, which BINARY does not.
    let db = TempDatabase::new("collation_all_comparisons.db");
    let conn = db.connect_limbo();
    register_nocase_trim(&conn, "trimcoll");
    conn.execute("CREATE TABLE t(id INTEGER PRIMARY KEY, x TEXT COLLATE trimcoll)")?;
    conn.execute("CREATE INDEX i ON t(x)")?;
    conn.execute("INSERT INTO t VALUES (1, 'alpha'), (2, 'ALPHA  '), (3, 'beta')")?;

    let count = |sql: &str| -> i64 {
        let rows: Vec<(i64,)> = conn.exec_rows(sql);
        rows[0].0
    };

    assert_eq!(count("SELECT count(*) FROM t WHERE x = 'ALPHA'"), 2);
    assert_eq!(count("SELECT count(*) FROM t WHERE x IN ('ALPHA')"), 2);
    assert_eq!(texts(&conn, "SELECT x FROM t WHERE x > 'alpha'"), ["beta"]);
    let groups: Vec<(i64,)> = conn.exec_rows("SELECT count(*) FROM t GROUP BY x ORDER BY 1");
    assert_eq!(groups, vec![(1,), (2,)]);
    assert_eq!(texts(&conn, "SELECT min(x) FROM t"), ["alpha"]);
    assert_eq!(texts(&conn, "SELECT max(x) FROM t"), ["beta"]);
    assert_eq!(
        count("SELECT count(*) FROM t AS a JOIN t AS b ON a.x = b.x"),
        5
    );
    assert_eq!(
        count("SELECT count(*) FROM (SELECT x FROM t UNION SELECT 'z')"),
        3
    );
    conn.execute("REINDEX")?;
    let ok: Vec<(String,)> = conn.exec_rows("PRAGMA integrity_check");
    assert_eq!(ok, vec![("ok".to_string(),)]);
    Ok(())
}

#[test]
fn distinct_over_a_custom_collation_uses_the_comparator() -> anyhow::Result<()> {
    // DISTINCT dedups through a hash table that hashes only "this is text" for a custom-collated column.
    let db = TempDatabase::new("collation_distinct.db");
    let conn = db.connect_limbo();
    register_nocase_trim(&conn, "trimcoll");
    conn.execute("CREATE TABLE t(a INTEGER, x TEXT COLLATE trimcoll)")?;
    conn.execute("INSERT INTO t VALUES (1, 'alpha'), (1, 'ALPHA  '), (2, 'beta')")?;

    let count = |sql: &str| -> i64 {
        let rows: Vec<(i64,)> = conn.exec_rows(sql);
        rows[0].0
    };

    // Like SQLite, the row that survives is the first one seen, spelling included.
    assert_eq!(
        texts(&conn, "SELECT DISTINCT x FROM t ORDER BY x"),
        ["alpha", "beta"]
    );
    assert_eq!(count("SELECT count(DISTINCT x) FROM t"), 2);

    conn.execute("CREATE TABLE t2(x TEXT)")?;
    conn.execute("INSERT INTO t2 VALUES ('alpha'), ('ALPHA  '), ('beta')")?;
    assert_eq!(count("SELECT count(*) FROM (SELECT DISTINCT x FROM t2)"), 3);
    assert_eq!(
        count("SELECT count(*) FROM (SELECT DISTINCT x COLLATE trimcoll FROM t2)"),
        2
    );

    conn.execute("INSERT INTO t VALUES (3, 'ALPHA')")?;
    assert_eq!(
        count("SELECT count(*) FROM (SELECT DISTINCT a, x FROM t)"),
        3
    );
    Ok(())
}

#[test]
fn distinct_keeps_rows_a_custom_collation_calls_different() -> anyhow::Result<()> {
    // `revcoll` calls nothing equal that BINARY does not, so dedup must keep every row.
    let db = TempDatabase::new("collation_distinct_reverse.db");
    let conn = db.connect_limbo();
    register_reverse(&conn, "revcoll");
    conn.execute("CREATE TABLE t(x TEXT COLLATE revcoll)")?;
    conn.execute("INSERT INTO t VALUES ('a'), ('A'), ('a  '), ('a')")?;

    assert_eq!(
        texts(&conn, "SELECT DISTINCT x FROM t ORDER BY x"),
        ["a  ", "a", "A"]
    );
    let count: Vec<(i64,)> = conn.exec_rows("SELECT count(DISTINCT x) FROM t");
    assert_eq!(count, vec![(3,)]);
    Ok(())
}

#[test]
fn distinct_over_a_custom_collation_survives_growing_and_spilling() -> anyhow::Result<()> {
    // Every text in a custom-collated column gets the same hash: this walks grow, rehash and spill with
    // a key that never spreads over buckets.
    let db = TempDatabase::new("collation_distinct_big.db");
    let conn = db.connect_limbo();
    register_nocase_trim(&conn, "trimcoll");
    conn.execute("CREATE TABLE big(x TEXT COLLATE trimcoll)")?;
    conn.execute(
        "INSERT INTO big WITH RECURSIVE seq(i) AS (
             SELECT 1 UNION ALL SELECT i + 1 FROM seq WHERE i < 3000
         ) SELECT 'value_' || i FROM seq",
    )?;
    conn.execute("INSERT INTO big SELECT upper(x) || '   ' FROM big")?;
    let total: Vec<(i64,)> = conn.exec_rows("SELECT count(*) FROM big");
    assert_eq!(total, vec![(6000,)]);

    let distinct: Vec<(i64,)> = conn.exec_rows("SELECT count(DISTINCT x) FROM big");
    assert_eq!(distinct, vec![(3000,)]);
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT count(*) FROM (SELECT DISTINCT x FROM big)");
    assert_eq!(rows, vec![(3000,)]);
    Ok(())
}

#[test]
fn a_binary_index_cannot_satisfy_an_order_by_with_a_custom_collate() -> anyhow::Result<()> {
    // The planner has no symbol table to look a custom collation up in; it must not read
    // `COLLATE revcoll` as BINARY and skip the sort because a BINARY index exists.
    let db = TempDatabase::new("collation_order_by_index.db");
    let conn = db.connect_limbo();
    register_reverse(&conn, "revcoll");
    conn.execute("CREATE TABLE t(x TEXT)")?;
    conn.execute("CREATE INDEX i ON t(x)")?;
    conn.execute("INSERT INTO t VALUES ('a'), ('B'), ('c')")?;

    // BINARY order is B < a < c.
    assert_eq!(
        texts(&conn, "SELECT x FROM t ORDER BY x COLLATE revcoll"),
        ["c", "a", "B"]
    );
    assert_eq!(texts(&conn, "SELECT x FROM t ORDER BY x"), ["B", "a", "c"]);
    Ok(())
}

#[test]
fn altering_a_table_keeps_its_columns_collations_in_the_stored_schema() -> anyhow::Result<()> {
    // ALTER TABLE rewrites the stored CREATE TABLE text; losing the COLLATE clause would silently turn
    // the column BINARY on the next schema load.
    let db = TempDatabase::new("collation_alter_table.db");
    {
        let conn = db.connect_limbo();
        register_reverse(&conn, "revcoll");
        conn.execute("CREATE TABLE t(x TEXT COLLATE revcoll, scratch TEXT)")?;
        conn.execute("ALTER TABLE t ADD COLUMN y TEXT COLLATE revcoll")?;
        conn.execute("ALTER TABLE t DROP COLUMN scratch")?;
        conn.execute("INSERT INTO t VALUES ('a', 'a'), ('b', 'b')")?;
        conn.close()?;
    }

    let reopened = TempDatabase::new_with_existent(&db.path);
    let conn = reopened.connect_limbo();
    let schema: Vec<(String,)> = conn.exec_rows("SELECT sql FROM sqlite_schema WHERE name = 't'");
    assert_eq!(
        schema[0].0.matches("revcoll").count(),
        2,
        "both columns should keep their collation: {}",
        schema[0].0
    );
    register_reverse(&conn, "revcoll");
    assert_eq!(texts(&conn, "SELECT x FROM t ORDER BY x"), ["b", "a"]);
    assert_eq!(texts(&conn, "SELECT y FROM t ORDER BY y"), ["b", "a"]);
    Ok(())
}

#[test]
fn scalar_min_max_and_nullif_honor_a_custom_column_collation() -> anyhow::Result<()> {
    // min(), max() and nullif() pick their collation from the first argument with one; a custom collation
    // must drive that like a built-in. Verified against sqlite3 with NOCASE as the stand-in for `trimcoll`:
    //   SELECT max(x, 'ALPHA') FROM t;     -- 'alpha': max() keeps the first of an equal pair
    //   SELECT nullif(x, 'ALPHA') FROM t;  -- NULL
    let db = TempDatabase::new("collation_scalar_min_max_nullif.db");
    let conn = db.connect_limbo();
    register_nocase_trim(&conn, "trimcoll");
    conn.execute("CREATE TABLE t(x TEXT COLLATE trimcoll)")?;
    conn.execute("INSERT INTO t VALUES ('alpha')")?;

    assert_eq!(
        texts(&conn, "SELECT typeof(nullif(x, 'ALPHA  ')) FROM t"),
        ["null"]
    );

    assert_eq!(texts(&conn, "SELECT max(x, 'ALPHA  ') FROM t"), ["alpha"]);

    assert_eq!(texts(&conn, "SELECT max(x, 'beta') FROM t"), ["beta"]);
    assert_eq!(texts(&conn, "SELECT nullif(x, 'beta') FROM t"), ["alpha"]);
    Ok(())
}

#[test]
fn full_outer_join_on_a_custom_collated_key_uses_the_comparator() -> anyhow::Result<()> {
    // FULL OUTER JOIN has no nested-loop form, so a custom-collated join key must be allowed to hash join.
    let db = TempDatabase::new("collation_full_outer.db");
    let conn = db.connect_limbo();
    register_nocase_trim(&conn, "trimcoll");
    conn.execute("CREATE TABLE l(x TEXT COLLATE trimcoll)")?;
    conn.execute("CREATE TABLE r(y TEXT)")?;
    conn.execute("INSERT INTO l VALUES ('alpha'), ('beta')")?;
    conn.execute("INSERT INTO r VALUES ('ALPHA  '), ('gamma')")?;

    let rows = texts(
        &conn,
        "SELECT coalesce(x, '-') || '|' || coalesce(y, '-') FROM l FULL OUTER JOIN r ON l.x = r.y ORDER BY 1",
    );
    assert_eq!(rows, ["-|gamma", "alpha|ALPHA  ", "beta|-"]);

    conn.execute("CREATE TABLE l2(x TEXT)")?;
    conn.execute("INSERT INTO l2 VALUES ('alpha'), ('beta')")?;
    let rows = texts(
        &conn,
        "SELECT coalesce(x, '-') || '|' || coalesce(y, '-') FROM l2 FULL OUTER JOIN r ON l2.x = r.y COLLATE trimcoll ORDER BY 1",
    );
    assert_eq!(rows, ["-|gamma", "alpha|ALPHA  ", "beta|-"]);
    let n: Vec<(i64,)> =
        conn.exec_rows("SELECT count(*) FROM l2 JOIN r ON l2.x = r.y COLLATE trimcoll");
    assert_eq!(n, vec![(1,)]);
    Ok(())
}

#[test]
fn window_partitions_and_peers_compare_with_a_custom_collation() -> anyhow::Result<()> {
    let db = TempDatabase::new("collation_window.db");
    let conn = db.connect_limbo();
    register_nocase_trim(&conn, "trimcoll");
    conn.execute("CREATE TABLE t(x TEXT COLLATE trimcoll, v INT)")?;
    conn.execute("INSERT INTO t VALUES ('alpha', 1), ('ALPHA  ', 2), ('beta', 4)")?;

    // 'alpha' and 'ALPHA  ' are one partition under trimcoll.
    let rows: Vec<(i64,)> = conn.exec_rows("SELECT sum(v) OVER (PARTITION BY x) FROM t ORDER BY v");
    assert_eq!(rows, vec![(3,), (3,), (4,)]);

    // ...and peers of each other in a RANGE frame.
    let rows: Vec<(i64,)> = conn.exec_rows(
        "SELECT sum(v) OVER (ORDER BY x RANGE BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) FROM t ORDER BY v",
    );
    assert_eq!(rows, vec![(3,), (3,), (7,)]);

    // The same through an explicit COLLATE on a column without one of its own.
    conn.execute("CREATE TABLE u(x TEXT, v INT)")?;
    conn.execute("INSERT INTO u VALUES ('alpha', 1), ('ALPHA  ', 2), ('beta', 4)")?;
    let rows: Vec<(i64,)> = conn.exec_rows(
        "SELECT sum(v) OVER (PARTITION BY x COLLATE trimcoll ORDER BY x COLLATE trimcoll) FROM u ORDER BY v",
    );
    assert_eq!(rows, vec![(3,), (3,), (4,)]);
    Ok(())
}
