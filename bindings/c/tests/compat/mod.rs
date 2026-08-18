#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_imports)]
use std::ptr;

#[repr(C)]
struct sqlite3 {
    _private: [u8; 0],
}

#[repr(C)]
struct sqlite3_stmt {
    _private: [u8; 0],
}

#[cfg_attr(
    all(not(feature = "sqlite3"), not(target_os = "windows")),
    link(name = "turso_sqlite3")
)]
#[cfg_attr(
    all(
        feature = "sqlite3",
        not(all(target_os = "windows", target_env = "msvc"))
    ),
    link(name = "sqlite3")
)]
#[cfg_attr(
    all(feature = "sqlite3", target_os = "windows", target_env = "msvc"),
    link(name = "sqlite3", kind = "static")
)]
#[cfg_attr(
    all(not(feature = "sqlite3"), target_os = "windows"),
    link(name = "turso_sqlite3", kind = "raw-dylib")
)]
extern "C" {
    fn sqlite3_libversion() -> *const libc::c_char;
    fn sqlite3_libversion_number() -> i32;
    fn sqlite3_close(db: *mut sqlite3) -> i32;
    fn sqlite3_open(filename: *const libc::c_char, db: *mut *mut sqlite3) -> i32;
    fn sqlite3_db_filename(db: *mut sqlite3, db_name: *const libc::c_char) -> *const libc::c_char;
    fn sqlite3_exec(
        db: *mut sqlite3,
        sql: *const libc::c_char,
        callback: Option<
            unsafe extern "C" fn(
                arg1: *mut libc::c_void,
                arg2: libc::c_int,
                arg3: *mut *mut libc::c_char,
                arg4: *mut *mut libc::c_char,
            ) -> libc::c_int,
        >,
        arg: *mut libc::c_void,
        errmsg: *mut *mut libc::c_char,
    ) -> i32;
    fn sqlite3_free(ptr: *mut libc::c_void);
    fn sqlite3_prepare_v2(
        db: *mut sqlite3,
        sql: *const libc::c_char,
        n_bytes: i32,
        stmt: *mut *mut sqlite3_stmt,
        tail: *mut *const libc::c_char,
    ) -> i32;
    fn sqlite3_step(stmt: *mut sqlite3_stmt) -> i32;
    fn sqlite3_reset(stmt: *mut sqlite3_stmt) -> i32;
    fn sqlite3_finalize(stmt: *mut sqlite3_stmt) -> i32;
    fn sqlite3_wal_checkpoint(db: *mut sqlite3, db_name: *const libc::c_char) -> i32;
    fn sqlite3_wal_checkpoint_v2(
        db: *mut sqlite3,
        db_name: *const libc::c_char,
        mode: i32,
        log_size: *mut i32,
        checkpoint_count: *mut i32,
    ) -> i32;
    fn sqlite3_column_int64(stmt: *mut sqlite3_stmt, idx: i32) -> i64;
    fn libsql_wal_frame_count(db: *mut sqlite3, p_frame_count: *mut u32) -> i32;
    fn libsql_wal_get_frame(
        db: *mut sqlite3,
        frame_no: u32,
        p_frame: *mut u8,
        frame_len: u32,
    ) -> i32;
    fn libsql_wal_disable_checkpoint(db: *mut sqlite3) -> i32;
    fn sqlite3_column_int(stmt: *mut sqlite3_stmt, idx: i32) -> i64;
    fn sqlite3_next_stmt(db: *mut sqlite3, stmt: *mut sqlite3_stmt) -> *mut sqlite3_stmt;
    fn sqlite3_bind_int(stmt: *mut sqlite3_stmt, idx: i32, val: i64) -> i32;
    fn sqlite3_bind_parameter_count(stmt: *mut sqlite3_stmt) -> i32;
    fn sqlite3_bind_parameter_name(stmt: *mut sqlite3_stmt, idx: i32) -> *const libc::c_char;
    fn sqlite3_bind_parameter_index(stmt: *mut sqlite3_stmt, name: *const libc::c_char) -> i32;
    fn sqlite3_clear_bindings(stmt: *mut sqlite3_stmt) -> i32;
    fn sqlite3_column_name(stmt: *mut sqlite3_stmt, idx: i32) -> *const libc::c_char;
    fn sqlite3_column_table_name(stmt: *mut sqlite3_stmt, idx: i32) -> *const libc::c_char;
    fn sqlite3_last_insert_rowid(db: *mut sqlite3) -> i32;
    fn sqlite3_column_count(stmt: *mut sqlite3_stmt) -> i32;
    fn sqlite3_bind_text(
        stmt: *mut sqlite3_stmt,
        idx: i32,
        text: *const libc::c_char,
        len: i32,
        destructor: Option<unsafe extern "C" fn(*mut libc::c_void)>,
    ) -> i32;
    fn sqlite3_bind_blob(
        stmt: *mut sqlite3_stmt,
        idx: i32,
        blob: *const libc::c_void,
        len: i32,
        destructor: Option<unsafe extern "C" fn(*mut libc::c_void)>,
    ) -> i32;
    fn sqlite3_column_text(stmt: *mut sqlite3_stmt, idx: i32) -> *const libc::c_char;
    fn sqlite3_column_bytes(stmt: *mut sqlite3_stmt, idx: i32) -> i64;
    fn sqlite3_column_blob(stmt: *mut sqlite3_stmt, idx: i32) -> *const libc::c_void;
    fn sqlite3_column_type(stmt: *mut sqlite3_stmt, idx: i32) -> i32;
    fn sqlite3_column_decltype(stmt: *mut sqlite3_stmt, idx: i32) -> *const libc::c_char;
    fn sqlite3_get_autocommit(db: *mut sqlite3) -> i32;
    fn sqlite3_changes(db: *mut sqlite3) -> i32;
    fn sqlite3_changes64(db: *mut sqlite3) -> i64;
    fn sqlite3_table_column_metadata(
        db: *mut sqlite3,
        z_db_name: *const libc::c_char,
        z_table_name: *const libc::c_char,
        z_column_name: *const libc::c_char,
        pz_data_type: *mut *const libc::c_char,
        pz_coll_seq: *mut *const libc::c_char,
        p_not_null: *mut libc::c_int,
        p_primary_key: *mut libc::c_int,
        p_autoinc: *mut libc::c_int,
    ) -> i32;
    fn sqlite3_busy_handler(
        db: *mut sqlite3,
        callback: Option<unsafe extern "C" fn(*mut libc::c_void, i32) -> i32>,
        arg: *mut libc::c_void,
    ) -> i32;
    fn sqlite3_progress_handler(
        db: *mut sqlite3,
        n: i32,
        callback: Option<unsafe extern "C" fn(*mut libc::c_void) -> i32>,
        arg: *mut libc::c_void,
    );
    fn sqlite3_busy_timeout(db: *mut sqlite3, ms: i32) -> i32;
    fn sqlite3_interrupt(db: *mut sqlite3);
    fn sqlite3_get_table(
        db: *mut sqlite3,
        sql: *const libc::c_char,
        paz_result: *mut *mut *mut libc::c_char,
        pn_row: *mut libc::c_int,
        pn_column: *mut libc::c_int,
        pz_err_msg: *mut *mut libc::c_char,
    ) -> i32;
    fn sqlite3_free_table(az_result: *mut *mut libc::c_char);
    fn sqlite3_bind_null(stmt: *mut sqlite3_stmt, idx: i32) -> i32;
    fn sqlite3_value_type(value: *mut libc::c_void) -> i32;
    fn sqlite3_value_blob(value: *mut libc::c_void) -> *const libc::c_void;
    fn sqlite3_value_bytes(value: *mut libc::c_void) -> i32;
    fn sqlite3_result_int64(context: *mut libc::c_void, val: i64);
    fn sqlite3_create_function_v2(
        db: *mut sqlite3,
        name: *const libc::c_char,
        n_args: i32,
        enc: i32,
        context: *mut libc::c_void,
        func: XFunc,
        step: XFunc,
        final_: XFinal,
        destroy: XDestroy,
    ) -> i32;
    fn sqlite3_prepare_v3(
        db: *mut sqlite3,
        sql: *const libc::c_char,
        n_bytes: i32,
        prep_flags: u32,
        stmt: *mut *mut sqlite3_stmt,
        tail: *mut *const libc::c_char,
    ) -> i32;
    fn sqlite3_db_handle(stmt: *mut sqlite3_stmt) -> *mut sqlite3;
    fn sqlite3_value_int(value: *mut libc::c_void) -> i32;
    fn sqlite3_result_int(context: *mut libc::c_void, val: i32);
    fn sqlite3_initialize() -> i32;
    fn sqlite3_open_v2(
        filename: *const libc::c_char,
        db: *mut *mut sqlite3,
        flags: i32,
        z_vfs: *const libc::c_char,
    ) -> i32;
    fn sqlite3_column_value(stmt: *mut sqlite3_stmt, idx: i32) -> *mut libc::c_void;
    fn sqlite3_value_int64(value: *mut libc::c_void) -> i64;
    fn sqlite3_value_double(value: *mut libc::c_void) -> f64;
    fn sqlite3_value_text(value: *mut libc::c_void) -> *const libc::c_char;
    fn sqlite3_value_dup(value: *mut libc::c_void) -> *mut libc::c_void;
    fn sqlite3_value_free(value: *mut libc::c_void);
    fn sqlite3_context_db_handle(context: *mut libc::c_void) -> *mut libc::c_void;
    fn sqlite3_errmsg(db: *mut sqlite3) -> *const libc::c_char;
    fn sqlite3_create_function(
        db: *mut sqlite3,
        name: *const libc::c_char,
        n_args: i32,
        enc: i32,
        context: *mut libc::c_void,
        func: XFunc,
        step: XFunc,
        final_: XFinal,
    ) -> i32;
    fn sqlite3_create_function16(
        db: *mut sqlite3,
        name: *const libc::c_void,
        n_args: i32,
        enc: i32,
        context: *mut libc::c_void,
        func: XFunc,
        step: XFunc,
        final_: XFinal,
    ) -> i32;
    fn sqlite3_create_window_function(
        db: *mut sqlite3,
        name: *const libc::c_char,
        n_args: i32,
        enc: i32,
        context: *mut libc::c_void,
        step: XFunc,
        final_: XFinal,
        value: XFinal,
        inverse: XFunc,
        destroy: XDestroy,
    ) -> i32;
    fn sqlite3_overload_function(db: *mut sqlite3, name: *const libc::c_char, n_args: i32) -> i32;
    fn sqlite3_aggregate_context(context: *mut libc::c_void, n_bytes: i32) -> *mut libc::c_void;
    fn sqlite3_user_data(context: *mut libc::c_void) -> *mut libc::c_void;
    fn sqlite3_get_auxdata(context: *mut libc::c_void, arg: i32) -> *mut libc::c_void;
    fn sqlite3_set_auxdata(
        context: *mut libc::c_void,
        arg: i32,
        data: *mut libc::c_void,
        destroy: XDestroy,
    );
    fn sqlite3_result_null(context: *mut libc::c_void);
    fn sqlite3_result_double(context: *mut libc::c_void, val: f64);
    fn sqlite3_result_text(
        context: *mut libc::c_void,
        text: *const libc::c_char,
        n: i32,
        destructor: *mut libc::c_void,
    );
    fn sqlite3_result_blob(
        context: *mut libc::c_void,
        blob: *const libc::c_void,
        n: i32,
        destructor: *mut libc::c_void,
    );
    fn sqlite3_result_zeroblob(context: *mut libc::c_void, n: i32);
    fn sqlite3_result_value(context: *mut libc::c_void, value: *mut libc::c_void);
    fn sqlite3_result_subtype(context: *mut libc::c_void, subtype: u32);
    fn sqlite3_result_error(context: *mut libc::c_void, err: *const libc::c_char, n: i32);
    fn sqlite3_result_error_code(context: *mut libc::c_void, code: i32);
    fn sqlite3_result_error_toobig(context: *mut libc::c_void);
    fn sqlite3_result_error_nomem(context: *mut libc::c_void);
    fn sqlite3_value_subtype(value: *mut libc::c_void) -> u32;
    fn sqlite3_value_numeric_type(value: *mut libc::c_void) -> i32;
    fn sqlite3_value_frombind(value: *mut libc::c_void) -> i32;
    fn sqlite3_value_nochange(value: *mut libc::c_void) -> i32;
    fn sqlite3_value_encoding(value: *mut libc::c_void) -> i32;
    fn sqlite3_value_bytes16(value: *mut libc::c_void) -> i32;
    fn sqlite3_value_text16(value: *mut libc::c_void) -> *const libc::c_void;
    fn sqlite3_result_text16(
        context: *mut libc::c_void,
        text: *const libc::c_void,
        n: i32,
        destructor: *mut libc::c_void,
    );
    fn sqlite3_create_collation(
        db: *mut sqlite3,
        name: *const libc::c_char,
        enc: i32,
        p_arg: *mut libc::c_void,
        x_compare: XCompare,
    ) -> i32;
    fn sqlite3_create_collation_v2(
        db: *mut sqlite3,
        name: *const libc::c_char,
        enc: i32,
        p_arg: *mut libc::c_void,
        x_compare: XCompare,
        x_destroy: XDestroy,
    ) -> i32;
    fn sqlite3_create_collation16(
        db: *mut sqlite3,
        name: *const libc::c_void,
        enc: i32,
        p_arg: *mut libc::c_void,
        x_compare: XCompare,
    ) -> i32;
    fn sqlite3_collation_needed(
        db: *mut sqlite3,
        p_arg: *mut libc::c_void,
        x_coll_needed: XCollationNeeded,
    ) -> i32;
    fn sqlite3_result_pointer(
        context: *mut libc::c_void,
        ptr: *mut libc::c_void,
        type_name: *const libc::c_char,
        destructor: XDestroy,
    );
    fn sqlite3_value_pointer(
        value: *mut libc::c_void,
        type_name: *const libc::c_char,
    ) -> *mut libc::c_void;
    fn sqlite3_bind_pointer(
        stmt: *mut sqlite3_stmt,
        idx: i32,
        ptr: *mut libc::c_void,
        type_name: *const libc::c_char,
        destructor: XDestroy,
    ) -> i32;
}

type XFunc = Option<unsafe extern "C" fn(*mut libc::c_void, i32, *mut *mut libc::c_void)>;
type XFinal = Option<unsafe extern "C" fn(*mut libc::c_void)>;
type XDestroy = Option<unsafe extern "C" fn(*mut libc::c_void)>;
/// `(pArg, nLeft, pLeft, nRight, pRight)`, the lengths in bytes.
type XCompare = Option<
    unsafe extern "C" fn(
        *mut libc::c_void,
        i32,
        *const libc::c_void,
        i32,
        *const libc::c_void,
    ) -> i32,
>;
type XCollationNeeded =
    Option<unsafe extern "C" fn(*mut libc::c_void, *mut sqlite3, i32, *const libc::c_char)>;

const SQLITE_OK: i32 = 0;
const SQLITE_ERROR: i32 = 1;
const SQLITE_BUSY: i32 = 5;
const SQLITE_MISUSE: i32 = 21;
const SQLITE_RANGE: i32 = 25;
const SQLITE_CANTOPEN: i32 = 14;
const SQLITE_INTERRUPT: i32 = 9;
const SQLITE_ROW: i32 = 100;
const SQLITE_DONE: i32 = 101;
const SQLITE_PREPARE_PERSISTENT: u32 = 0x01;

const SQLITE_CHECKPOINT_PASSIVE: i32 = 0;
const SQLITE_CHECKPOINT_FULL: i32 = 1;
const SQLITE_CHECKPOINT_RESTART: i32 = 2;
const SQLITE_CHECKPOINT_TRUNCATE: i32 = 3;
const SQLITE_INTEGER: i32 = 1;
const SQLITE_FLOAT: i32 = 2;
const SQLITE_ABORT: i32 = 4;
const SQLITE_TEXT: i32 = 3;
const SQLITE3_TEXT: i32 = 3;
const SQLITE_BLOB: i32 = 4;
const SQLITE_NULL: i32 = 5;
const SQLITE_UTF8: i32 = 1;
const SQLITE_UTF16LE: i32 = 2;
const SQLITE_UTF16BE: i32 = 3;
const SQLITE_UTF16: i32 = 4;
const SQLITE_ANY: i32 = 5;
const SQLITE_UTF16_ALIGNED: i32 = 8;
const SQLITE_NOMEM: i32 = 7;
const SQLITE_TOOBIG: i32 = 18;
const SQLITE_CONSTRAINT: i32 = 19;
const SQLITE_DETERMINISTIC: i32 = 0x0000_0800;
const SQLITE_SUBTYPE: i32 = 0x0010_0000;
const SQLITE_RESULT_SUBTYPE: i32 = 0x0100_0000;
/// The subtype SQLite's JSON functions stamp on a result: the ASCII 'J'.
const JSON_SUBTYPE: u32 = 74;

/// SQLITE_TRANSIENT, the "copy this, I am about to free it" sentinel.
fn transient() -> *mut libc::c_void {
    -1isize as *mut libc::c_void
}
const SQLITE_OPEN_READWRITE: i32 = 0x00000002;
const SQLITE_OPEN_CREATE: i32 = 0x00000004;
const SQLITE_OPEN_URI: i32 = 0x00000040;

mod tests {
    use super::*;

    #[test]
    fn test_libversion() {
        unsafe {
            let version = sqlite3_libversion();
            assert!(!version.is_null());
            #[cfg(not(feature = "sqlite3"))]
            assert_eq!(std::ffi::CStr::from_ptr(version), c"3.50.4");
        }
    }

    #[test]
    fn test_libversion_number() {
        unsafe {
            let version_num = sqlite3_libversion_number();
            #[cfg(feature = "sqlite3")]
            assert!(version_num > 0);
            #[cfg(not(feature = "sqlite3"))]
            assert_eq!(version_num, 3050004);
        }
    }

    #[test]
    fn test_open_not_found() {
        unsafe {
            let mut db = ptr::null_mut();
            assert_eq!(
                sqlite3_open(c"not-found/local.db".as_ptr(), &mut db),
                SQLITE_CANTOPEN
            );
        }
    }

    #[test]
    fn test_open_existing() {
        unsafe {
            let mut db = ptr::null_mut();
            assert_eq!(
                sqlite3_open(c"../../testing/system/testing_clone.db".as_ptr(), &mut db),
                SQLITE_OK
            );
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_close() {
        unsafe {
            assert_eq!(sqlite3_close(ptr::null_mut()), SQLITE_OK);
        }
    }

    #[test]
    fn test_prepare_misuse() {
        unsafe {
            let mut db = ptr::null_mut();
            assert_eq!(sqlite3_open(c":memory:".as_ptr(), &mut db), SQLITE_OK);

            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(db, c"SELECT 1".as_ptr(), -1, &mut stmt, ptr::null_mut()),
                SQLITE_OK
            );

            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    // Per SQLite spec (https://www.sqlite.org/c3ref/prepare.html):
    //   "If nByte is positive, it is the number of bytes read from zSql."
    // Consumers like rusqlite/libsqlite3-sys pass non-NUL-terminated &str slices with
    // explicit positive nByte. The three tests below cover the contract:
    //   - positive nByte respects the bound (no over-read past the slice)
    //   - negative nByte keeps the read-until-NUL behavior
    //   - NUL within the bound truncates (per spec)
    // Each test actively sets up the failure mode (garbage bytes after the bound, or
    // a "should-not-be-seen" suffix after an embedded NUL) so an over-reading impl
    // would produce a parser error rather than silently passing.

    #[test]
    fn test_prepare_v2_n_byte_positive_no_nul() {
        unsafe {
            let mut db = ptr::null_mut();
            assert_eq!(sqlite3_open(c":memory:".as_ptr(), &mut db), SQLITE_OK);

            // 8 bytes of valid SQL followed by 4 bytes of garbage. n_byte = 8.
            // An over-reading impl would walk past the bound, hit \xff, and error.
            let sql_and_garbage: &[u8] = b"SELECT 1\xff\xff\xff\xff";
            let mut stmt = ptr::null_mut();
            let rc = sqlite3_prepare_v2(
                db,
                sql_and_garbage.as_ptr() as *const libc::c_char,
                8,
                &mut stmt,
                ptr::null_mut(),
            );
            assert_eq!(
                rc, SQLITE_OK,
                "prepare_v2 must respect positive n_byte (no over-read)"
            );
            assert!(!stmt.is_null());
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_prepare_v2_n_byte_negative_uses_nul() {
        // Regression: n_byte = -1 path must still read until NUL terminator.
        unsafe {
            let mut db = ptr::null_mut();
            assert_eq!(sqlite3_open(c":memory:".as_ptr(), &mut db), SQLITE_OK);

            let mut stmt = ptr::null_mut();
            let rc = sqlite3_prepare_v2(db, c"SELECT 1".as_ptr(), -1, &mut stmt, ptr::null_mut());
            assert_eq!(rc, SQLITE_OK);
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_prepare_v2_n_byte_truncates_at_embedded_nul() {
        // Spec: "the SQL text is read up to the first zero terminator or the nByte-th
        // byte, whichever comes first." Embedded NUL within the bound must truncate.
        unsafe {
            let mut db = ptr::null_mut();
            assert_eq!(sqlite3_open(c":memory:".as_ptr(), &mut db), SQLITE_OK);

            // NUL at offset 8; bound is 20 (claims more). Reader must stop at NUL.
            // If under-truncating, parser sees "GARBAGE_AFTER_NUL" and errors out.
            let sql: &[u8] = b"SELECT 1\x00GARBAGE_AFTER_NUL";
            let mut stmt = ptr::null_mut();
            let rc = sqlite3_prepare_v2(
                db,
                sql.as_ptr() as *const libc::c_char,
                20,
                &mut stmt,
                ptr::null_mut(),
            );
            assert_eq!(
                rc, SQLITE_OK,
                "embedded NUL within bound must truncate, not over-read"
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_sqlite3_bind_int() {
        unsafe {
            let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
            let path = std::ffi::CString::new(temp_file.path().to_str().unwrap()).unwrap();
            let mut db = ptr::null_mut();
            assert_eq!(sqlite3_open(path.as_ptr(), &mut db), SQLITE_OK);

            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"CREATE TABLE test_bind (id INTEGER PRIMARY KEY, value INTEGER)".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"INSERT INTO test_bind (value) VALUES (?)".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_bind_int(stmt, 1, 42), SQLITE_OK);
            assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT value FROM test_bind LIMIT 1".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            assert_eq!(sqlite3_column_int(stmt, 0), 42);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_sqlite3_bind_int_range_checks() {
        unsafe {
            let mut db = ptr::null_mut();
            assert_eq!(sqlite3_open(c":memory:".as_ptr(), &mut db), SQLITE_OK);

            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(db, c"SELECT ?1".as_ptr(), -1, &mut stmt, ptr::null_mut()),
                SQLITE_OK
            );

            assert_eq!(sqlite3_bind_int(stmt, 0, 1), SQLITE_RANGE);
            assert_eq!(sqlite3_bind_int(stmt, 2, 1), SQLITE_RANGE);
            assert_eq!(sqlite3_bind_int(stmt, 1, 7), SQLITE_OK);

            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_sqlite3_bind_parameter_name_and_count() {
        unsafe {
            let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
            let path = std::ffi::CString::new(temp_file.path().to_str().unwrap()).unwrap();
            let mut db = ptr::null_mut();
            assert_eq!(sqlite3_open(path.as_ptr(), &mut db), SQLITE_OK);

            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"CREATE TABLE test_params (id INTEGER PRIMARY KEY, value TEXT)".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"INSERT INTO test_params (id, value) VALUES (?1, ?2)".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );

            let param_count = sqlite3_bind_parameter_count(stmt);
            assert_eq!(param_count, 2);

            println!("parameter count {param_count}");
            let name1 = sqlite3_bind_parameter_name(stmt, 1);
            assert!(!name1.is_null());
            let name1_str = std::ffi::CStr::from_ptr(name1).to_str().unwrap();
            assert_eq!(name1_str, "?1");

            let name2 = sqlite3_bind_parameter_name(stmt, 2);
            assert!(!name2.is_null());
            let name2_str = std::ffi::CStr::from_ptr(name2).to_str().unwrap();
            assert_eq!(name2_str, "?2");

            let invalid_name = sqlite3_bind_parameter_name(stmt, 99);
            assert!(invalid_name.is_null());

            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_sqlite3_bind_parameter_count_sparse_positional() {
        unsafe {
            let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
            let path = std::ffi::CString::new(temp_file.path().to_str().unwrap()).unwrap();
            let mut db = ptr::null_mut();
            assert_eq!(sqlite3_open(path.as_ptr(), &mut db), SQLITE_OK);

            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(db, c"SELECT ?3".as_ptr(), -1, &mut stmt, ptr::null_mut(),),
                SQLITE_OK
            );

            assert_eq!(sqlite3_bind_parameter_count(stmt), 3);
            assert!(sqlite3_bind_parameter_name(stmt, 1).is_null());
            assert!(sqlite3_bind_parameter_name(stmt, 2).is_null());

            let name3 = sqlite3_bind_parameter_name(stmt, 3);
            assert!(!name3.is_null());
            let name3 = std::ffi::CStr::from_ptr(name3).to_str().unwrap();
            assert_eq!(name3, "?3");

            assert_eq!(sqlite3_bind_int(stmt, 1, 1), SQLITE_OK);
            assert_eq!(sqlite3_bind_int(stmt, 3, 9), SQLITE_OK);

            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            assert_eq!(sqlite3_column_int(stmt, 0), 9);

            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_sqlite3_last_insert_rowid() {
        unsafe {
            let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
            let path = std::ffi::CString::new(temp_file.path().to_str().unwrap()).unwrap();
            let mut db = std::ptr::null_mut();
            assert_eq!(sqlite3_open(path.as_ptr(), &mut db), SQLITE_OK);

            let mut stmt = std::ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"CREATE TABLE test_rowid (value INTEGER)".as_ptr(),
                    -1,
                    &mut stmt,
                    std::ptr::null_mut(),
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            let mut stmt = std::ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"INSERT INTO test_rowid (value) VALUES (6)".as_ptr(),
                    -1,
                    &mut stmt,
                    std::ptr::null_mut(),
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            let last_rowid = sqlite3_last_insert_rowid(db);
            assert!(last_rowid > 0);
            println!("last insert rowid: {last_rowid}");
            let query = format!("SELECT value FROM test_rowid WHERE rowid = {last_rowid}");
            let query_cstring = std::ffi::CString::new(query).unwrap();

            let mut stmt = std::ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    query_cstring.as_ptr(),
                    -1,
                    &mut stmt,
                    std::ptr::null_mut(),
                ),
                SQLITE_OK
            );

            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            let value_int = sqlite3_column_int(stmt, 0);
            assert_eq!(value_int, 6);

            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }
    #[test]
    fn test_sqlite3_column_name() {
        unsafe {
            let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
            let path = std::ffi::CString::new(temp_file.path().to_str().unwrap()).unwrap();
            let mut db = std::ptr::null_mut();
            assert_eq!(sqlite3_open(path.as_ptr(), &mut db), SQLITE_OK);

            let mut stmt = std::ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"CREATE TABLE test_cols (id INTEGER PRIMARY KEY, value TEXT)".as_ptr(),
                    -1,
                    &mut stmt,
                    std::ptr::null_mut(),
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            let mut stmt = std::ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT id, value FROM test_cols".as_ptr(),
                    -1,
                    &mut stmt,
                    std::ptr::null_mut(),
                ),
                SQLITE_OK
            );

            let col_count = sqlite3_column_count(stmt);
            assert_eq!(col_count, 2);

            let name1 = sqlite3_column_name(stmt, 0);
            assert!(!name1.is_null());
            let name1_str = std::ffi::CStr::from_ptr(name1).to_str().unwrap();
            assert_eq!(name1_str, "id");

            let table_name1 = sqlite3_column_table_name(stmt, 0);
            assert!(!table_name1.is_null());
            let table_name1_str = std::ffi::CStr::from_ptr(table_name1).to_str().unwrap();
            assert_eq!(table_name1_str, "test_cols");

            let name2 = sqlite3_column_name(stmt, 1);
            assert!(!name2.is_null());
            let name2_str = std::ffi::CStr::from_ptr(name2).to_str().unwrap();
            assert_eq!(name2_str, "value");

            // will lead to panic
            //let invalid = sqlite3_column_name(stmt, 5);
            //assert!(invalid.is_null());

            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_ctas_column_count_is_zero() {
        unsafe {
            let mut db = ptr::null_mut();
            assert_eq!(sqlite3_open(c":memory:".as_ptr(), &mut db), SQLITE_OK);

            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"CREATE TABLE dst AS SELECT 1 AS a, 'x' AS b".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_column_count(stmt), 0);
            assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn column_text_is_nul_terminated_and_bytes_match() {
        unsafe {
            let mut db = std::ptr::null_mut();
            assert_eq!(
                sqlite3_open(c"../../testing/system/testing.db".as_ptr(), &mut db),
                SQLITE_OK
            );
            let mut stmt = std::ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT first_name FROM users ORDER BY rowid ASC LIMIT 1;".as_ptr(),
                    -1,
                    &mut stmt,
                    std::ptr::null_mut()
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            let p = sqlite3_column_text(stmt, 0);
            assert!(!p.is_null());
            let bytes = sqlite3_column_bytes(stmt, 0) as usize;
            // NUL at [bytes], and no extra counted
            let slice = std::slice::from_raw_parts(p, bytes + 1);
            assert_eq!(slice[bytes], 0);
            assert_eq!(libc::strlen(p), bytes);

            let s = std::ffi::CStr::from_ptr(p).to_str().unwrap();
            assert_eq!(s, "Jamie");
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_sqlite3_bind_text() {
        unsafe {
            let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
            let path = std::ffi::CString::new(temp_file.path().to_str().unwrap()).unwrap();
            let mut db = ptr::null_mut();
            assert_eq!(sqlite3_open(path.as_ptr(), &mut db), SQLITE_OK);

            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"CREATE TABLE test_bind_text_rs (id INTEGER PRIMARY KEY, value TEXT)".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            let destructor = std::mem::transmute::<
                isize,
                Option<unsafe extern "C" fn(*mut std::ffi::c_void)>,
            >(-1isize);
            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"INSERT INTO test_bind_text_rs (value) VALUES (?)".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );
            let val = std::ffi::CString::new("hello world").unwrap();
            assert_eq!(
                sqlite3_bind_text(stmt, 1, val.as_ptr(), -1, destructor),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"INSERT INTO test_bind_text_rs (value) VALUES (?)".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );
            let val2 = std::ffi::CString::new("abcdef").unwrap();
            assert_eq!(
                sqlite3_bind_text(stmt, 1, val2.as_ptr(), 3, destructor),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT value FROM test_bind_text_rs ORDER BY id".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );

            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            let col1_ptr = sqlite3_column_text(stmt, 0);
            assert!(!col1_ptr.is_null());
            let col1_str = std::ffi::CStr::from_ptr(col1_ptr).to_str().unwrap();
            assert_eq!(col1_str, "hello world");

            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);

            let col2_ptr = sqlite3_column_text(stmt, 0);
            let col2_len = sqlite3_column_bytes(stmt, 0);
            assert!(!col2_ptr.is_null());

            let col2_slice = std::slice::from_raw_parts(col2_ptr as *const u8, col2_len as usize);
            let col2_str = std::str::from_utf8(col2_slice).unwrap().to_owned();

            assert_eq!(col2_str, "abc");
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_sqlite3_bind_blob() {
        unsafe {
            let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
            let path = std::ffi::CString::new(temp_file.path().to_str().unwrap()).unwrap();
            let mut db = ptr::null_mut();
            assert_eq!(sqlite3_open(path.as_ptr(), &mut db), SQLITE_OK);

            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"CREATE TABLE test_bind_blob_rs (id INTEGER PRIMARY KEY, data BLOB)".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"INSERT INTO test_bind_blob_rs (data) VALUES (?)".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );
            let data1 = b"\x01\x02\x03\x04\x05";
            assert_eq!(
                sqlite3_bind_blob(
                    stmt,
                    1,
                    data1.as_ptr() as *const _,
                    data1.len() as i32,
                    None
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"INSERT INTO test_bind_blob_rs (data) VALUES (?)".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );
            let data2 = b"\xAA\xBB\xCC\xDD";
            assert_eq!(
                sqlite3_bind_blob(stmt, 1, data2.as_ptr() as *const _, 2, None),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT data FROM test_bind_blob_rs ORDER BY id".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );

            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            let col1_ptr = sqlite3_column_blob(stmt, 0);
            let col1_len = sqlite3_column_bytes(stmt, 0);
            let col1_slice = std::slice::from_raw_parts(col1_ptr as *const u8, col1_len as usize);
            assert_eq!(col1_slice, data1);

            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            let col2_ptr = sqlite3_column_blob(stmt, 0);
            let col2_len = sqlite3_column_bytes(stmt, 0);
            let col2_slice = std::slice::from_raw_parts(col2_ptr as *const u8, col2_len as usize);
            assert_eq!(col2_slice, &data2[..2]);

            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_sqlite3_column_type() {
        unsafe {
            let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
            let path = std::ffi::CString::new(temp_file.path().to_str().unwrap()).unwrap();
            let mut db = std::ptr::null_mut();
            assert_eq!(sqlite3_open(path.as_ptr(), &mut db), SQLITE_OK);

            let mut stmt = std::ptr::null_mut();
            assert_eq!(
            sqlite3_prepare_v2(
                db,
                c"CREATE TABLE test_types (col_int INTEGER, col_float REAL, col_text TEXT, col_blob BLOB, col_null text)".as_ptr(),
                -1,
                &mut stmt,
                std::ptr::null_mut(),
            ),
            SQLITE_OK
        );
            assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            let mut stmt = std::ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"INSERT INTO test_types VALUES (123, 45.67, 'hello', x'010203', null)"
                        .as_ptr(),
                    -1,
                    &mut stmt,
                    std::ptr::null_mut(),
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            let mut stmt = std::ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT col_int, col_float, col_text, col_blob, col_null FROM test_types"
                        .as_ptr(),
                    -1,
                    &mut stmt,
                    std::ptr::null_mut(),
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);

            assert_eq!(sqlite3_column_type(stmt, 0), SQLITE_INTEGER);
            assert_eq!(sqlite3_column_type(stmt, 1), SQLITE_FLOAT);
            assert_eq!(sqlite3_column_type(stmt, 2), SQLITE_TEXT);
            assert_eq!(sqlite3_column_type(stmt, 3), SQLITE_BLOB);
            assert_eq!(sqlite3_column_type(stmt, 4), SQLITE_NULL);

            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_sqlite3_column_decltype() {
        unsafe {
            let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
            let path = std::ffi::CString::new(temp_file.path().to_str().unwrap()).unwrap();
            let mut db = std::ptr::null_mut();
            assert_eq!(sqlite3_open(path.as_ptr(), &mut db), SQLITE_OK);

            let mut stmt = std::ptr::null_mut();
            assert_eq!(
            sqlite3_prepare_v2(
                db,
                c"CREATE TABLE test_decltype (col_int INTEGER, col_float REAL, col_text TEXT, col_blob BLOB, col_null NULL)".as_ptr(),
                -1,
                &mut stmt,
                std::ptr::null_mut(),
            ),
            SQLITE_OK
        );
            assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            let mut stmt = std::ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT col_int, col_float, col_text, col_blob, col_null FROM test_decltype"
                        .as_ptr(),
                    -1,
                    &mut stmt,
                    std::ptr::null_mut(),
                ),
                SQLITE_OK
            );

            let expected = [
                Some("INTEGER"),
                Some("REAL"),
                Some("TEXT"),
                Some("BLOB"),
                None,
            ];

            for i in 0..sqlite3_column_count(stmt) {
                let decl = sqlite3_column_decltype(stmt, i);

                if decl.is_null() {
                    assert!(expected[i as usize].is_none());
                } else {
                    let s = std::ffi::CStr::from_ptr(decl)
                        .to_string_lossy()
                        .into_owned();
                    assert_eq!(Some(s.as_str()), expected[i as usize]);
                }
            }

            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_exec_multi_statement_dml() {
        unsafe {
            let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
            let path = std::ffi::CString::new(temp_file.path().to_str().unwrap()).unwrap();
            let mut db = ptr::null_mut();
            assert_eq!(sqlite3_open(path.as_ptr(), &mut db), SQLITE_OK);

            // Multiple DML statements in one exec call
            let rc = sqlite3_exec(
                db,
                c"CREATE TABLE bind_text(x TEXT);\
              INSERT INTO bind_text(x) VALUES('TEXT1');\
              INSERT INTO bind_text(x) VALUES('TEXT2');"
                    .as_ptr(),
                None,
                ptr::null_mut(),
                ptr::null_mut(),
            );
            assert_eq!(rc, SQLITE_OK);

            // Verify the data was inserted
            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT COUNT(*) FROM bind_text".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            assert_eq!(sqlite3_column_int(stmt, 0), 2);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_exec_multi_statement_with_semicolons_in_strings() {
        unsafe {
            let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
            let path = std::ffi::CString::new(temp_file.path().to_str().unwrap()).unwrap();
            let mut db = ptr::null_mut();
            assert_eq!(sqlite3_open(path.as_ptr(), &mut db), SQLITE_OK);

            // Semicolons inside strings should not split statements
            let rc = sqlite3_exec(
                db,
                c"CREATE TABLE test_semicolon(x TEXT);\
              INSERT INTO test_semicolon(x) VALUES('value;with;semicolons');\
              INSERT INTO test_semicolon(x) VALUES(\"another;value\");"
                    .as_ptr(),
                None,
                ptr::null_mut(),
                ptr::null_mut(),
            );
            assert_eq!(rc, SQLITE_OK);

            // Verify the values contain semicolons
            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT x FROM test_semicolon ORDER BY rowid".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );

            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            let val1 = std::ffi::CStr::from_ptr(sqlite3_column_text(stmt, 0))
                .to_str()
                .unwrap();
            assert_eq!(val1, "value;with;semicolons");

            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            let val2 = std::ffi::CStr::from_ptr(sqlite3_column_text(stmt, 0))
                .to_str()
                .unwrap();
            assert_eq!(val2, "another;value");

            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_exec_multi_statement_with_escaped_quotes() {
        unsafe {
            let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
            let path = std::ffi::CString::new(temp_file.path().to_str().unwrap()).unwrap();
            let mut db = ptr::null_mut();
            assert_eq!(sqlite3_open(path.as_ptr(), &mut db), SQLITE_OK);

            // Test escaped quotes
            let rc = sqlite3_exec(
                db,
                c"CREATE TABLE test_quotes(x TEXT);\
              INSERT INTO test_quotes(x) VALUES('it''s working');\
              INSERT INTO test_quotes(x) VALUES(\"quote\"\"test\"\"\");"
                    .as_ptr(),
                None,
                ptr::null_mut(),
                ptr::null_mut(),
            );
            assert_eq!(rc, SQLITE_OK);

            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT x FROM test_quotes ORDER BY rowid".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );

            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            let val1 = std::ffi::CStr::from_ptr(sqlite3_column_text(stmt, 0))
                .to_str()
                .unwrap();
            assert_eq!(val1, "it's working");

            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            let val2 = std::ffi::CStr::from_ptr(sqlite3_column_text(stmt, 0))
                .to_str()
                .unwrap();
            assert_eq!(val2, "quote\"test\"");

            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_exec_with_select_callback() {
        unsafe {
            // Callback that collects results
            unsafe extern "C" fn exec_callback(
                context: *mut std::ffi::c_void,
                n_cols: std::ffi::c_int,
                values: *mut *mut std::ffi::c_char,
                _cols: *mut *mut std::ffi::c_char,
            ) -> std::ffi::c_int {
                let results = &mut *(context as *mut Vec<Vec<String>>);
                let mut row = Vec::new();

                for i in 0..n_cols as isize {
                    let value_ptr = *values.offset(i);
                    let value = if value_ptr.is_null() {
                        String::from("NULL")
                    } else {
                        std::ffi::CStr::from_ptr(value_ptr)
                            .to_str()
                            .unwrap()
                            .to_owned()
                    };
                    row.push(value);
                }
                results.push(row);
                0 // Continue
            }

            let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
            let path = std::ffi::CString::new(temp_file.path().to_str().unwrap()).unwrap();
            let mut db = ptr::null_mut();
            assert_eq!(sqlite3_open(path.as_ptr(), &mut db), SQLITE_OK);

            // Setup data
            let rc = sqlite3_exec(
                db,
                c"CREATE TABLE test_select(id INTEGER, name TEXT);\
              INSERT INTO test_select VALUES(1, 'Alice');\
              INSERT INTO test_select VALUES(2, 'Bob');"
                    .as_ptr(),
                None,
                ptr::null_mut(),
                ptr::null_mut(),
            );
            assert_eq!(rc, SQLITE_OK);

            // Execute SELECT with callback
            let mut results: Vec<Vec<String>> = Vec::new();
            let rc = sqlite3_exec(
                db,
                c"SELECT id, name FROM test_select ORDER BY id".as_ptr(),
                Some(exec_callback),
                &mut results as *mut _ as *mut std::ffi::c_void,
                ptr::null_mut(),
            );
            assert_eq!(rc, SQLITE_OK);

            assert_eq!(results.len(), 2);
            assert_eq!(results[0], vec!["1", "Alice"]);
            assert_eq!(results[1], vec!["2", "Bob"]);

            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_exec_multi_statement_mixed_dml_select() {
        unsafe {
            // Callback that counts invocations
            unsafe extern "C" fn count_callback(
                context: *mut std::ffi::c_void,
                _n_cols: std::ffi::c_int,
                _values: *mut *mut std::ffi::c_char,
                _cols: *mut *mut std::ffi::c_char,
            ) -> std::ffi::c_int {
                let count = &mut *(context as *mut i32);
                *count += 1;
                0
            }

            let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
            let path = std::ffi::CString::new(temp_file.path().to_str().unwrap()).unwrap();
            let mut db = ptr::null_mut();
            assert_eq!(sqlite3_open(path.as_ptr(), &mut db), SQLITE_OK);

            let mut callback_count = 0;

            // Mix of DDL/DML/DQL
            let rc = sqlite3_exec(
                db,
                c"CREATE TABLE mixed(x INTEGER);\
              INSERT INTO mixed VALUES(1);\
              INSERT INTO mixed VALUES(2);\
              SELECT x FROM mixed;\
              INSERT INTO mixed VALUES(3);\
              SELECT COUNT(*) FROM mixed;"
                    .as_ptr(),
                Some(count_callback),
                &mut callback_count as *mut _ as *mut std::ffi::c_void,
                ptr::null_mut(),
            );
            assert_eq!(rc, SQLITE_OK);

            // Callback should be called 3 times total:
            // 2 times for first SELECT (2 rows)
            // 1 time for second SELECT (1 row with COUNT)
            assert_eq!(callback_count, 3);

            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_exec_callback_abort() {
        unsafe {
            // Callback that aborts after first row
            unsafe extern "C" fn abort_callback(
                context: *mut std::ffi::c_void,
                _n_cols: std::ffi::c_int,
                _values: *mut *mut std::ffi::c_char,
                _cols: *mut *mut std::ffi::c_char,
            ) -> std::ffi::c_int {
                let count = &mut *(context as *mut i32);
                *count += 1;
                if *count >= 1 {
                    return 1; // Abort
                }
                0
            }

            let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
            let path = std::ffi::CString::new(temp_file.path().to_str().unwrap()).unwrap();
            let mut db = ptr::null_mut();
            assert_eq!(sqlite3_open(path.as_ptr(), &mut db), SQLITE_OK);

            sqlite3_exec(
                db,
                c"CREATE TABLE test(x INTEGER);\
              INSERT INTO test VALUES(1),(2),(3);"
                    .as_ptr(),
                None,
                ptr::null_mut(),
                ptr::null_mut(),
            );

            let mut count = 0;
            let rc = sqlite3_exec(
                db,
                c"SELECT x FROM test".as_ptr(),
                Some(abort_callback),
                &mut count as *mut _ as *mut std::ffi::c_void,
                ptr::null_mut(),
            );

            assert_eq!(rc, SQLITE_ABORT);
            assert_eq!(count, 1); // Only processed one row before aborting

            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_exec_error_stops_execution() {
        unsafe {
            let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
            let path = std::ffi::CString::new(temp_file.path().to_str().unwrap()).unwrap();
            let mut db = ptr::null_mut();
            assert_eq!(sqlite3_open(path.as_ptr(), &mut db), SQLITE_OK);

            let mut err_msg = ptr::null_mut();

            // Second statement has error, third should not execute
            let rc = sqlite3_exec(
                db,
                c"CREATE TABLE test(x INTEGER);\
              INSERT INTO nonexistent VALUES(1);\
              CREATE TABLE should_not_exist(y INTEGER);"
                    .as_ptr(),
                None,
                ptr::null_mut(),
                &mut err_msg,
            );

            assert_eq!(rc, SQLITE_ERROR);

            // Verify third statement didn't execute
            let mut stmt = ptr::null_mut();
            let check_rc = sqlite3_prepare_v2(
                db,
                c"SELECT name FROM sqlite_master WHERE type='table' AND name='should_not_exist'"
                    .as_ptr(),
                -1,
                &mut stmt,
                ptr::null_mut(),
            );
            assert_eq!(check_rc, SQLITE_OK);
            assert_eq!(sqlite3_step(stmt), SQLITE_DONE); // No rows = table doesn't exist
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            if !err_msg.is_null() {
                sqlite3_free(err_msg as *mut std::ffi::c_void);
            }

            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_exec_empty_statements() {
        unsafe {
            let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
            let path = std::ffi::CString::new(temp_file.path().to_str().unwrap()).unwrap();
            let mut db = ptr::null_mut();
            assert_eq!(sqlite3_open(path.as_ptr(), &mut db), SQLITE_OK);

            // Multiple semicolons and whitespace should be handled gracefully
            let rc = sqlite3_exec(
                db,
                c"CREATE TABLE test(x INTEGER);;;\n\n;\t;INSERT INTO test VALUES(1);;;".as_ptr(),
                None,
                ptr::null_mut(),
                ptr::null_mut(),
            );
            assert_eq!(rc, SQLITE_OK);

            // Verify both statements executed
            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT x FROM test".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            assert_eq!(sqlite3_column_int(stmt, 0), 1);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }
    #[test]
    fn test_exec_with_comments() {
        unsafe {
            let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
            let path = std::ffi::CString::new(temp_file.path().to_str().unwrap()).unwrap();
            let mut db = ptr::null_mut();
            assert_eq!(sqlite3_open(path.as_ptr(), &mut db), SQLITE_OK);

            // SQL comments shouldn't affect statement splitting
            let rc = sqlite3_exec(
                db,
                c"-- This is a comment\n\
              CREATE TABLE test(x INTEGER); -- inline comment\n\
              INSERT INTO test VALUES(1); -- semicolon in comment ;\n\
              INSERT INTO test VALUES(2) -- end with comment"
                    .as_ptr(),
                None,
                ptr::null_mut(),
                ptr::null_mut(),
            );
            assert_eq!(rc, SQLITE_OK);

            // Verify both inserts worked
            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT COUNT(*) FROM test".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            assert_eq!(sqlite3_column_int(stmt, 0), 2);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_exec_nested_quotes() {
        unsafe {
            let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
            let path = std::ffi::CString::new(temp_file.path().to_str().unwrap()).unwrap();
            let mut db = ptr::null_mut();
            assert_eq!(sqlite3_open(path.as_ptr(), &mut db), SQLITE_OK);

            // Mix of quote types and nesting
            let rc = sqlite3_exec(
                db,
                c"CREATE TABLE test(x TEXT);\
              INSERT INTO test VALUES('single \"double\" inside');\
              INSERT INTO test VALUES(\"double 'single' inside\");\
              INSERT INTO test VALUES('mix;\"quote\";types');"
                    .as_ptr(),
                None,
                ptr::null_mut(),
                ptr::null_mut(),
            );
            assert_eq!(rc, SQLITE_OK);

            // Verify values
            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT x FROM test ORDER BY rowid".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );

            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            let val1 = std::ffi::CStr::from_ptr(sqlite3_column_text(stmt, 0))
                .to_str()
                .unwrap();
            assert_eq!(val1, "single \"double\" inside");

            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            let val2 = std::ffi::CStr::from_ptr(sqlite3_column_text(stmt, 0))
                .to_str()
                .unwrap();
            assert_eq!(val2, "double 'single' inside");

            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            let val3 = std::ffi::CStr::from_ptr(sqlite3_column_text(stmt, 0))
                .to_str()
                .unwrap();
            assert_eq!(val3, "mix;\"quote\";types");

            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_exec_transaction_rollback() {
        unsafe {
            let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
            let path = std::ffi::CString::new(temp_file.path().to_str().unwrap()).unwrap();
            let mut db = ptr::null_mut();
            assert_eq!(sqlite3_open(path.as_ptr(), &mut db), SQLITE_OK);

            // Test transaction rollback in multi-statement
            let rc = sqlite3_exec(
                db,
                c"CREATE TABLE test(x INTEGER);\
              BEGIN TRANSACTION;\
              INSERT INTO test VALUES(1);\
              INSERT INTO test VALUES(2);\
              ROLLBACK;"
                    .as_ptr(),
                None,
                ptr::null_mut(),
                ptr::null_mut(),
            );
            assert_eq!(rc, SQLITE_OK);

            // Table should exist but be empty due to rollback
            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT COUNT(*) FROM test".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            assert_eq!(sqlite3_column_int(stmt, 0), 0); // No rows due to rollback
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_exec_with_pragma() {
        unsafe {
            let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
            let path = std::ffi::CString::new(temp_file.path().to_str().unwrap()).unwrap();
            let mut db = ptr::null_mut();
            assert_eq!(sqlite3_open(path.as_ptr(), &mut db), SQLITE_OK);

            // Callback to capture pragma results
            unsafe extern "C" fn pragma_callback(
                context: *mut std::ffi::c_void,
                _n_cols: std::ffi::c_int,
                _values: *mut *mut std::ffi::c_char,
                _cols: *mut *mut std::ffi::c_char,
            ) -> std::ffi::c_int {
                let count = &mut *(context as *mut i32);
                *count += 1;
                0
            }

            let mut callback_count = 0;

            // PRAGMA should be treated as DQL when it returns results
            let rc = sqlite3_exec(
                db,
                c"CREATE TABLE test(x INTEGER);\
              PRAGMA table_info(test);"
                    .as_ptr(),
                Some(pragma_callback),
                &mut callback_count as *mut _ as *mut std::ffi::c_void,
                ptr::null_mut(),
            );
            assert_eq!(rc, SQLITE_OK);
            assert!(callback_count > 0); // PRAGMA should return at least one row

            // PRAGMA without callback should discard row
            let mut err_msg = ptr::null_mut();
            let rc = sqlite3_exec(
                db,
                c"PRAGMA table_info(test)".as_ptr(),
                None,
                ptr::null_mut(),
                &mut err_msg,
            );
            assert_eq!(rc, SQLITE_OK);
            if !err_msg.is_null() {
                sqlite3_free(err_msg as *mut std::ffi::c_void);
            }

            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_exec_with_cte() {
        unsafe {
            // Callback that collects results
            unsafe extern "C" fn exec_callback(
                context: *mut std::ffi::c_void,
                n_cols: std::ffi::c_int,
                values: *mut *mut std::ffi::c_char,
                _cols: *mut *mut std::ffi::c_char,
            ) -> std::ffi::c_int {
                let results = &mut *(context as *mut Vec<Vec<String>>);
                let mut row = Vec::new();
                for i in 0..n_cols as isize {
                    let value_ptr = *values.offset(i);
                    let value = if value_ptr.is_null() {
                        String::from("NULL")
                    } else {
                        std::ffi::CStr::from_ptr(value_ptr)
                            .to_str()
                            .unwrap()
                            .to_owned()
                    };
                    row.push(value);
                }
                results.push(row);
                0
            }

            let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
            let path = std::ffi::CString::new(temp_file.path().to_str().unwrap()).unwrap();
            let mut db = ptr::null_mut();
            assert_eq!(sqlite3_open(path.as_ptr(), &mut db), SQLITE_OK);

            // CTE should be recognized as DQL
            let mut results: Vec<Vec<String>> = Vec::new();
            let rc = sqlite3_exec(
                db,
                c"CREATE TABLE test(x INTEGER);\
              INSERT INTO test VALUES(1),(2),(3);\
              WITH cte AS (SELECT x FROM test WHERE x > 1) SELECT * FROM cte;"
                    .as_ptr(),
                Some(exec_callback),
                &mut results as *mut _ as *mut std::ffi::c_void,
                ptr::null_mut(),
            );
            assert_eq!(rc, SQLITE_OK);
            assert_eq!(results.len(), 2); // Should get 2 and 3
            assert_eq!(results[0], vec!["2"]);
            assert_eq!(results[1], vec!["3"]);

            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_exec_with_returning_clause() {
        unsafe {
            // Callback for RETURNING results
            unsafe extern "C" fn exec_callback(
                context: *mut std::ffi::c_void,
                n_cols: std::ffi::c_int,
                values: *mut *mut std::ffi::c_char,
                _cols: *mut *mut std::ffi::c_char,
            ) -> std::ffi::c_int {
                let results = &mut *(context as *mut Vec<Vec<String>>);
                let mut row = Vec::new();
                for i in 0..n_cols as isize {
                    let value_ptr = *values.offset(i);
                    let value = if value_ptr.is_null() {
                        String::from("NULL")
                    } else {
                        std::ffi::CStr::from_ptr(value_ptr)
                            .to_str()
                            .unwrap()
                            .to_owned()
                    };
                    row.push(value);
                }
                results.push(row);
                0
            }

            let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
            let path = std::ffi::CString::new(temp_file.path().to_str().unwrap()).unwrap();
            let mut db = ptr::null_mut();
            assert_eq!(sqlite3_open(path.as_ptr(), &mut db), SQLITE_OK);

            let mut results: Vec<Vec<String>> = Vec::new();

            // INSERT...RETURNING with callback should capture the returned values
            let rc = sqlite3_exec(
                db,
                c"CREATE TABLE test(id INTEGER PRIMARY KEY, x INTEGER);\
              INSERT INTO test(x) VALUES(42) RETURNING id, x;"
                    .as_ptr(),
                Some(exec_callback),
                &mut results as *mut _ as *mut std::ffi::c_void,
                ptr::null_mut(),
            );
            assert_eq!(rc, SQLITE_OK);
            assert_eq!(results.len(), 1);
            assert_eq!(results[0][1], "42"); // x value

            // Add another row for testing
            sqlite3_exec(
                db,
                c"INSERT INTO test(x) VALUES(99)".as_ptr(),
                None,
                ptr::null_mut(),
                ptr::null_mut(),
            );

            // should still delete the row but discard the RETURNING results
            let rc = sqlite3_exec(
                db,
                c"UPDATE test SET id = 3, x = 41 WHERE x=42 RETURNING id".as_ptr(),
                None,
                ptr::null_mut(),
                ptr::null_mut(),
            );
            assert_eq!(rc, SQLITE_OK);

            // Verify the row was actually updated
            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT COUNT(*) FROM test WHERE x=42".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            assert_eq!(sqlite3_column_int(stmt, 0), 0); // Should be 0 rows with x=42
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            // Verify
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT COUNT(*) FROM test".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            assert_eq!(sqlite3_column_int(stmt, 0), 2);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[cfg(not(feature = "sqlite3"))]
    mod libsql_ext {

        use super::*;

        #[test]
        fn test_wal_frame_count() {
            unsafe {
                let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
                let path = temp_file.path();
                let c_path = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
                let mut db = ptr::null_mut();
                assert_eq!(sqlite3_open(c_path.as_ptr(), &mut db), SQLITE_OK);
                // Ensure that WAL is initially empty.
                let mut frame_count = 0;
                assert_eq!(libsql_wal_frame_count(db, &mut frame_count), SQLITE_OK);
                assert_eq!(frame_count, 0);
                // Create a table and insert a row.
                let mut stmt = ptr::null_mut();
                assert_eq!(
                    sqlite3_prepare_v2(
                        db,
                        c"CREATE TABLE test (id INTEGER PRIMARY KEY)".as_ptr(),
                        -1,
                        &mut stmt,
                        ptr::null_mut()
                    ),
                    SQLITE_OK
                );
                assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
                assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
                let mut stmt = ptr::null_mut();
                assert_eq!(
                    sqlite3_prepare_v2(
                        db,
                        c"INSERT INTO test (id) VALUES (1)".as_ptr(),
                        -1,
                        &mut stmt,
                        ptr::null_mut()
                    ),
                    SQLITE_OK
                );
                assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
                assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
                // Check that WAL has three frames.
                assert_eq!(libsql_wal_frame_count(db, &mut frame_count), SQLITE_OK);
                assert_eq!(frame_count, 3);
                assert_eq!(sqlite3_close(db), SQLITE_OK);
            }
        }

        #[test]
        fn test_read_frame() {
            unsafe {
                let mut db = ptr::null_mut();
                let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
                let path = temp_file.path();
                let c_path = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
                assert_eq!(sqlite3_open(c_path.as_ptr(), &mut db), SQLITE_OK);
                // Create a table and insert a row.
                let mut stmt = ptr::null_mut();
                assert_eq!(
                    sqlite3_prepare_v2(
                        db,
                        c"CREATE TABLE test (id INTEGER PRIMARY KEY)".as_ptr(),
                        -1,
                        &mut stmt,
                        ptr::null_mut()
                    ),
                    SQLITE_OK
                );
                assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
                assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
                let mut stmt = ptr::null_mut();
                assert_eq!(
                    sqlite3_prepare_v2(
                        db,
                        c"INSERT INTO test (id) VALUES (1)".as_ptr(),
                        -1,
                        &mut stmt,
                        ptr::null_mut()
                    ),
                    SQLITE_OK
                );
                assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
                assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
                // Check that WAL has three frames.
                let mut frame_count = 0;
                assert_eq!(libsql_wal_frame_count(db, &mut frame_count), SQLITE_OK);
                assert_eq!(frame_count, 3);
                for i in 1..frame_count + 1 {
                    let frame_len = 4096 + 24;
                    let mut frame = vec![0; frame_len];
                    assert_eq!(
                        libsql_wal_get_frame(db, i, frame.as_mut_ptr(), frame_len as u32),
                        SQLITE_OK
                    );
                }
                assert_eq!(sqlite3_close(db), SQLITE_OK);
            }
        }

        #[test]
        fn test_disable_wal_checkpoint() {
            let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
            unsafe {
                let mut db = ptr::null_mut();
                let path = temp_file.path();
                let c_path = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
                assert_eq!(sqlite3_open(c_path.as_ptr(), &mut db), SQLITE_OK);
                // Create a table and insert a row.
                let mut stmt = ptr::null_mut();
                assert_eq!(
                    sqlite3_prepare_v2(
                        db,
                        c"CREATE TABLE test (id INTEGER PRIMARY KEY)".as_ptr(),
                        -1,
                        &mut stmt,
                        ptr::null_mut()
                    ),
                    SQLITE_OK
                );
                assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
                assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
                let mut stmt = ptr::null_mut();
                assert_eq!(
                    sqlite3_prepare_v2(
                        db,
                        c"INSERT INTO test (id) VALUES (0)".as_ptr(),
                        -1,
                        &mut stmt,
                        ptr::null_mut()
                    ),
                    SQLITE_OK
                );
                assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
                assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

                let mut log_size = 0;
                let mut checkpoint_count = 0;

                assert_eq!(
                    sqlite3_wal_checkpoint_v2(
                        db,
                        ptr::null(),
                        SQLITE_CHECKPOINT_PASSIVE,
                        &mut log_size,
                        &mut checkpoint_count
                    ),
                    SQLITE_OK
                );
                assert_eq!(sqlite3_close(db), SQLITE_OK);
            }
            let mut wal_path = temp_file.path().to_path_buf();
            assert!(wal_path.set_extension("db-wal"));
            std::fs::remove_file(wal_path.clone()).unwrap();

            {
                let mut db = ptr::null_mut();
                unsafe {
                    let path = temp_file.path();
                    let c_path = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
                    assert_eq!(sqlite3_open(c_path.as_ptr(), &mut db), SQLITE_OK);
                    assert_eq!(libsql_wal_disable_checkpoint(db), SQLITE_OK);
                    // Insert at least 1000 rows to go over checkpoint threshold.
                    let mut stmt = ptr::null_mut();
                    for i in 1..2000 {
                        let sql =
                            std::ffi::CString::new(format!("INSERT INTO test (id) VALUES ({i})"))
                                .unwrap();
                        assert_eq!(
                            sqlite3_prepare_v2(db, sql.as_ptr(), -1, &mut stmt, ptr::null_mut()),
                            SQLITE_OK
                        );
                        assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
                        assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
                    }
                    assert_eq!(sqlite3_close(db), SQLITE_OK);
                }
            }

            // Delete WAL to ensure that we don't load anything from it
            std::fs::remove_file(wal_path).unwrap();
            let mut db = ptr::null_mut();
            unsafe {
                let path = temp_file.path();
                let c_path = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
                assert_eq!(sqlite3_open(c_path.as_ptr(), &mut db), SQLITE_OK);
                // Insert at least 1000 rows to go over checkpoint threshold.
                let mut stmt = ptr::null_mut();
                assert_eq!(
                    sqlite3_prepare_v2(
                        db,
                        c"SELECT count() FROM test".as_ptr(),
                        -1,
                        &mut stmt,
                        ptr::null_mut()
                    ),
                    SQLITE_OK
                );
                assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
                let count = sqlite3_column_int64(stmt, 0);
                assert_eq!(count, 1);
                assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
                assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            }
        }

        #[test]
        fn test_get_autocommit() {
            unsafe {
                let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
                let path = std::ffi::CString::new(temp_file.path().to_str().unwrap()).unwrap();
                let mut db = ptr::null_mut();
                assert_eq!(sqlite3_open(path.as_ptr(), &mut db), SQLITE_OK);

                // Should be in autocommit mode by default
                assert_eq!(sqlite3_get_autocommit(db), 1);

                // Begin a transaction
                let mut stmt = ptr::null_mut();
                assert_eq!(
                    sqlite3_prepare_v2(db, c"BEGIN".as_ptr(), -1, &mut stmt, ptr::null_mut()),
                    SQLITE_OK
                );
                assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
                assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

                // Should NOT be in autocommit mode during transaction
                assert_eq!(sqlite3_get_autocommit(db), 0);

                // Create a table within the transaction
                let mut stmt = ptr::null_mut();
                assert_eq!(
                    sqlite3_prepare_v2(
                        db,
                        c"CREATE TABLE test (id INTEGER PRIMARY KEY)".as_ptr(),
                        -1,
                        &mut stmt,
                        ptr::null_mut()
                    ),
                    SQLITE_OK
                );
                assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
                assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

                // Still not in autocommit mode
                assert_eq!(sqlite3_get_autocommit(db), 0);

                // Commit the transaction
                let mut stmt = ptr::null_mut();
                assert_eq!(
                    sqlite3_prepare_v2(db, c"COMMIT".as_ptr(), -1, &mut stmt, ptr::null_mut()),
                    SQLITE_OK
                );
                assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
                assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

                // Should be back in autocommit mode after commit
                assert_eq!(sqlite3_get_autocommit(db), 1);

                // Test with ROLLBACK
                let mut stmt = ptr::null_mut();
                assert_eq!(
                    sqlite3_prepare_v2(db, c"BEGIN".as_ptr(), -1, &mut stmt, ptr::null_mut()),
                    SQLITE_OK
                );
                assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
                assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

                assert_eq!(sqlite3_get_autocommit(db), 0);

                let mut stmt = ptr::null_mut();
                assert_eq!(
                    sqlite3_prepare_v2(db, c"ROLLBACK".as_ptr(), -1, &mut stmt, ptr::null_mut()),
                    SQLITE_OK
                );
                assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
                assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

                // Should be back in autocommit mode after rollback
                assert_eq!(sqlite3_get_autocommit(db), 1);

                assert_eq!(sqlite3_close(db), SQLITE_OK);
            }
        }

        #[test]
        fn test_wal_checkpoint() {
            let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
            unsafe {
                let mut db = ptr::null_mut();
                let path = temp_file.path();
                let c_path = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
                assert_eq!(sqlite3_open(c_path.as_ptr(), &mut db), SQLITE_OK);
                // Create a table and insert a row.
                let mut stmt = ptr::null_mut();
                assert_eq!(
                    sqlite3_prepare_v2(
                        db,
                        c"CREATE TABLE test (id INTEGER PRIMARY KEY)".as_ptr(),
                        -1,
                        &mut stmt,
                        ptr::null_mut()
                    ),
                    SQLITE_OK
                );
                assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
                assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
                let mut stmt = ptr::null_mut();
                assert_eq!(
                    sqlite3_prepare_v2(
                        db,
                        c"INSERT INTO test (id) VALUES (0)".as_ptr(),
                        -1,
                        &mut stmt,
                        ptr::null_mut()
                    ),
                    SQLITE_OK
                );
                assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
                assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

                let mut log_size = 0;
                let mut checkpoint_count = 0;

                assert_eq!(
                    sqlite3_wal_checkpoint_v2(
                        db,
                        ptr::null(),
                        SQLITE_CHECKPOINT_PASSIVE,
                        &mut log_size,
                        &mut checkpoint_count
                    ),
                    SQLITE_OK
                );
                assert_eq!(sqlite3_close(db), SQLITE_OK);
            }
            let mut wal_path = temp_file.path().to_path_buf();
            assert!(wal_path.set_extension("db-wal"));
            std::fs::remove_file(wal_path.clone()).unwrap();

            {
                let mut db = ptr::null_mut();
                unsafe {
                    let path = temp_file.path();
                    let c_path = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
                    assert_eq!(sqlite3_open(c_path.as_ptr(), &mut db), SQLITE_OK);
                    // Insert at least 1000 rows to go over checkpoint threshold.
                    let mut stmt = ptr::null_mut();
                    for i in 1..2000 {
                        let sql =
                            std::ffi::CString::new(format!("INSERT INTO test (id) VALUES ({i})"))
                                .unwrap();
                        assert_eq!(
                            sqlite3_prepare_v2(db, sql.as_ptr(), -1, &mut stmt, ptr::null_mut()),
                            SQLITE_OK
                        );
                        assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
                        assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
                    }
                    assert_eq!(sqlite3_close(db), SQLITE_OK);
                }
            }

            // Delete WAL to ensure that we don't load anything from it
            std::fs::remove_file(wal_path).unwrap();
            let mut db = ptr::null_mut();
            unsafe {
                let path = temp_file.path();
                let c_path = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
                assert_eq!(sqlite3_open(c_path.as_ptr(), &mut db), SQLITE_OK);
                // Insert at least 1000 rows to go over checkpoint threshold.
                let mut stmt = ptr::null_mut();
                assert_eq!(
                    sqlite3_prepare_v2(
                        db,
                        c"SELECT count() FROM test".as_ptr(),
                        -1,
                        &mut stmt,
                        ptr::null_mut()
                    ),
                    SQLITE_OK
                );
                assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
                let count = sqlite3_column_int64(stmt, 0);
                // with a sane `should_checkpoint` method we have no guarantee that all 2000 rows are present, as the checkpoint was
                // triggered by cacheflush on insertions. the pattern will trigger a checkpoint when the wal has > 1000 frames,
                // so it will be triggered but will no longer be triggered on each consecutive
                // write. here we can assert that we have > 1500 rows.
                assert!(count > 1500, "count: {count}");
                assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
                assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            }
        }
    }

    #[test]
    fn test_sqlite3_clear_bindings() {
        unsafe {
            let mut db: *mut sqlite3 = ptr::null_mut();
            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();

            assert_eq!(sqlite3_open(c":memory:".as_ptr(), &mut db), SQLITE_OK);

            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"CREATE TABLE person (id INTEGER, name TEXT, age INTEGER)".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut()
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"INSERT INTO person (id, name, age) VALUES (1, 'John', 25), (2, 'Jane', 30)"
                        .as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut()
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT * FROM person WHERE id = ? AND age > ?".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut()
                ),
                SQLITE_OK
            );

            // Bind parameters - should find John (id=1, age=25 > 20)
            assert_eq!(sqlite3_bind_int(stmt, 1, 1), SQLITE_OK);
            assert_eq!(sqlite3_bind_int(stmt, 2, 20), SQLITE_OK);
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            assert_eq!(sqlite3_column_int(stmt, 0), 1);
            assert_eq!(sqlite3_column_int(stmt, 2), 25);

            // Reset and clear bindings, query should return no rows
            assert_eq!(sqlite3_reset(stmt), SQLITE_OK);
            assert_eq!(sqlite3_clear_bindings(stmt), SQLITE_OK);
            assert_eq!(sqlite3_step(stmt), SQLITE_DONE);

            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_sqlite3_bind_parameter_index() {
        const SQLITE_OK: i32 = 0;

        unsafe {
            let mut db: *mut sqlite3 = ptr::null_mut();
            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();

            assert_eq!(sqlite3_open(c":memory:".as_ptr(), &mut db), SQLITE_OK);

            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT * FROM sqlite_master WHERE name = :table_name AND type = :object_type"
                        .as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut()
                ),
                SQLITE_OK
            );

            let index1 = sqlite3_bind_parameter_index(stmt, c":table_name".as_ptr());
            assert_eq!(index1, 1);

            let index2 = sqlite3_bind_parameter_index(stmt, c":object_type".as_ptr());
            assert_eq!(index2, 2);

            let index3 = sqlite3_bind_parameter_index(stmt, c":nonexistent".as_ptr());
            assert_eq!(index3, 0);

            let index4 = sqlite3_bind_parameter_index(stmt, ptr::null());
            assert_eq!(index4, 0);

            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
        }
    }

    #[test]
    fn test_sqlite3_db_filename() {
        const SQLITE_OK: i32 = 0;

        unsafe {
            // Test with in-memory database
            let mut db: *mut sqlite3 = ptr::null_mut();
            assert_eq!(sqlite3_open(c":memory:".as_ptr(), &mut db), SQLITE_OK);
            let filename = sqlite3_db_filename(db, c"main".as_ptr());
            assert!(!filename.is_null());
            let filename_str = std::ffi::CStr::from_ptr(filename).to_str().unwrap();
            assert_eq!(filename_str, "");
            assert_eq!(sqlite3_close(db), SQLITE_OK);

            // Open a file-backed database
            let temp_file = tempfile::NamedTempFile::with_suffix(".db").unwrap();
            let path = std::ffi::CString::new(temp_file.path().to_str().unwrap()).unwrap();
            let mut db = ptr::null_mut();
            assert_eq!(sqlite3_open(path.as_ptr(), &mut db), SQLITE_OK);

            // Test with "main" database name
            let filename = sqlite3_db_filename(db, c"main".as_ptr());
            assert!(!filename.is_null());
            let filename_pathbuf =
                std::fs::canonicalize(std::ffi::CStr::from_ptr(filename).to_str().unwrap())
                    .unwrap();
            assert_eq!(filename_pathbuf, temp_file.path().canonicalize().unwrap());

            // Test with NULL database name (defaults to main)
            let filename_default = sqlite3_db_filename(db, ptr::null());
            assert!(!filename_default.is_null());
            assert_eq!(filename, filename_default);

            // Test with non-existent database name
            let filename = sqlite3_db_filename(db, c"temp".as_ptr());
            assert!(filename.is_null());

            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_sqlite3_next_stmt() {
        const SQLITE_OK: i32 = 0;

        unsafe {
            let mut db: *mut sqlite3 = ptr::null_mut();
            assert_eq!(sqlite3_open(c":memory:".as_ptr(), &mut db), SQLITE_OK);

            // Initially, there should be no prepared statements
            let iter = sqlite3_next_stmt(db, ptr::null_mut());
            assert!(iter.is_null());

            // Prepare first statement
            let mut stmt1: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(db, c"SELECT 1;".as_ptr(), -1, &mut stmt1, ptr::null_mut()),
                SQLITE_OK
            );
            assert!(!stmt1.is_null());

            // Now there should be one statement
            let iter = sqlite3_next_stmt(db, ptr::null_mut());
            assert_eq!(iter, stmt1);

            // And no more after that
            let iter = sqlite3_next_stmt(db, stmt1);
            assert!(iter.is_null());

            // Prepare second statement
            let mut stmt2: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(db, c"SELECT 2;".as_ptr(), -1, &mut stmt2, ptr::null_mut()),
                SQLITE_OK
            );
            assert!(!stmt2.is_null());

            // Prepare third statement
            let mut stmt3: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(db, c"SELECT 3;".as_ptr(), -1, &mut stmt3, ptr::null_mut()),
                SQLITE_OK
            );
            assert!(!stmt3.is_null());

            // Count all statements
            let mut count = 0;
            let mut iter = sqlite3_next_stmt(db, ptr::null_mut());
            while !iter.is_null() {
                count += 1;
                iter = sqlite3_next_stmt(db, iter);
            }
            assert_eq!(count, 3);

            // Finalize the middle statement
            assert_eq!(sqlite3_finalize(stmt2), SQLITE_OK);

            // Count should now be 2
            count = 0;
            iter = sqlite3_next_stmt(db, ptr::null_mut());
            while !iter.is_null() {
                count += 1;
                iter = sqlite3_next_stmt(db, iter);
            }
            assert_eq!(count, 2);

            // Finalize remaining statements
            assert_eq!(sqlite3_finalize(stmt1), SQLITE_OK);
            assert_eq!(sqlite3_finalize(stmt3), SQLITE_OK);

            // Should be no statements left
            let iter = sqlite3_next_stmt(db, ptr::null_mut());
            assert!(iter.is_null());

            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_sqlite3_changes() {
        unsafe {
            let mut db: *mut sqlite3 = ptr::null_mut();
            assert_eq!(sqlite3_open(c":memory:".as_ptr(), &mut db), SQLITE_OK);

            // // Initially no changes
            assert_eq!(sqlite3_changes(db), 0);
            assert_eq!(sqlite3_changes64(db), 0);

            // Create a table
            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"CREATE TABLE test_changes (id INTEGER PRIMARY KEY, value TEXT)".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            // Still no changes after CREATE TABLE
            assert_eq!(sqlite3_changes(db), 0);
            assert_eq!(sqlite3_changes64(db), 0);

            // Insert a single row
            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"INSERT INTO test_changes (value) VALUES ('test1')".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            // Should have 1 change
            assert_eq!(sqlite3_changes(db), 1);
            assert_eq!(sqlite3_changes64(db), 1);

            // Insert multiple rows
            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"INSERT INTO test_changes (value) VALUES ('test2'), ('test3'), ('test4')"
                        .as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            // Should have 3 changes
            assert_eq!(sqlite3_changes(db), 3);
            assert_eq!(sqlite3_changes64(db), 3);

            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_sqlite3_table_column_metadata() {
        unsafe {
            let mut db = ptr::null_mut();
            assert_eq!(sqlite3_open(c":memory:".as_ptr(), &mut db), SQLITE_OK);

            // Create a test table
            let mut stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"CREATE TABLE test_metadata (id INTEGER PRIMARY KEY, name TEXT NOT NULL, value REAL)"
                        .as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            // Test column metadata for 'id' column
            let mut data_type: *const libc::c_char = ptr::null();
            let mut coll_seq: *const libc::c_char = ptr::null();
            let mut not_null: libc::c_int = 0;
            let mut primary_key: libc::c_int = 0;
            let mut autoinc: libc::c_int = 0;

            assert_eq!(
                sqlite3_table_column_metadata(
                    db,
                    ptr::null(), // main database
                    c"test_metadata".as_ptr(),
                    c"id".as_ptr(),
                    &mut data_type,
                    &mut coll_seq,
                    &mut not_null,
                    &mut primary_key,
                    &mut autoinc,
                ),
                SQLITE_OK
            );

            // Verify the results
            assert!(!data_type.is_null());
            assert!(!coll_seq.is_null());
            assert_eq!(primary_key, 1); // id is primary key
            assert_eq!(not_null, 0); // INTEGER columns don't have NOT NULL by default
            assert_eq!(autoinc, 0); // not auto-increment

            // Test column metadata for 'name' column
            let mut data_type2: *const libc::c_char = ptr::null();
            let mut coll_seq2: *const libc::c_char = ptr::null();
            let mut not_null2: libc::c_int = 0;
            let mut primary_key2: libc::c_int = 0;
            let mut autoinc2: libc::c_int = 0;

            assert_eq!(
                sqlite3_table_column_metadata(
                    db,
                    ptr::null(), // main database
                    c"test_metadata".as_ptr(),
                    c"name".as_ptr(),
                    &mut data_type2,
                    &mut coll_seq2,
                    &mut not_null2,
                    &mut primary_key2,
                    &mut autoinc2,
                ),
                SQLITE_OK
            );

            // Verify the results
            assert!(!data_type2.is_null());
            assert!(!coll_seq2.is_null());
            assert_eq!(primary_key2, 0); // name is not primary key
            assert_eq!(not_null2, 1); // name has NOT NULL constraint
            assert_eq!(autoinc2, 0); // not auto-increment

            // Test non-existent column
            let mut data_type3: *const libc::c_char = ptr::null();
            let mut coll_seq3: *const libc::c_char = ptr::null();
            let mut not_null3: libc::c_int = 0;
            let mut primary_key3: libc::c_int = 0;
            let mut autoinc3: libc::c_int = 0;

            assert_eq!(
                sqlite3_table_column_metadata(
                    db,
                    ptr::null(), // main database
                    c"test_metadata".as_ptr(),
                    c"nonexistent".as_ptr(),
                    &mut data_type3,
                    &mut coll_seq3,
                    &mut not_null3,
                    &mut primary_key3,
                    &mut autoinc3,
                ),
                SQLITE_ERROR
            );

            // Test non-existent table
            let mut data_type4: *const libc::c_char = ptr::null();
            let mut coll_seq4: *const libc::c_char = ptr::null();
            let mut not_null4: libc::c_int = 0;
            let mut primary_key4: libc::c_int = 0;
            let mut autoinc4: libc::c_int = 0;

            assert_eq!(
                sqlite3_table_column_metadata(
                    db,
                    ptr::null(), // main database
                    c"nonexistent_table".as_ptr(),
                    c"id".as_ptr(),
                    &mut data_type4,
                    &mut coll_seq4,
                    &mut not_null4,
                    &mut primary_key4,
                    &mut autoinc4,
                ),
                SQLITE_ERROR
            );

            // Test rowid column
            let mut data_type5: *const libc::c_char = ptr::null();
            let mut coll_seq5: *const libc::c_char = ptr::null();
            let mut not_null5: libc::c_int = 0;
            let mut primary_key5: libc::c_int = 0;
            let mut autoinc5: libc::c_int = 0;

            assert_eq!(
                sqlite3_table_column_metadata(
                    db,
                    ptr::null(), // main database
                    c"test_metadata".as_ptr(),
                    c"rowid".as_ptr(),
                    &mut data_type5,
                    &mut coll_seq5,
                    &mut not_null5,
                    &mut primary_key5,
                    &mut autoinc5,
                ),
                SQLITE_OK
            );

            // Verify rowid results
            assert!(!data_type5.is_null());
            assert!(!coll_seq5.is_null());
            assert_eq!(primary_key5, 1); // rowid is primary key
            assert_eq!(not_null5, 0);
            assert_eq!(autoinc5, 0);

            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_sqlite3_busy_timeout() {
        unsafe {
            let mut db: *mut sqlite3 = ptr::null_mut();
            assert_eq!(sqlite3_open(c":memory:".as_ptr(), &mut db), SQLITE_OK);

            // Test setting a positive timeout
            assert_eq!(sqlite3_busy_timeout(db, 1000), SQLITE_OK);

            // Test setting a zero timeout (disables busy handler)
            assert_eq!(sqlite3_busy_timeout(db, 0), SQLITE_OK);

            // Test setting a negative timeout (also disables busy handler)
            assert_eq!(sqlite3_busy_timeout(db, -1), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    /// Busy handler callback that retries up to N times
    unsafe extern "C" fn busy_handler_retry_n(data: *mut libc::c_void, count: i32) -> i32 {
        if data.is_null() {
            return 0;
        }
        let max_retries = *(data as *const i32);
        if count < max_retries {
            1
        } else {
            0
        }
    }

    /// Busy handler callback that never retries
    unsafe extern "C" fn busy_handler_never_retry(_data: *mut libc::c_void, _count: i32) -> i32 {
        0
    }

    /// Busy handler callback that always retries (with safety limit)
    unsafe extern "C" fn busy_handler_always_retry(_data: *mut libc::c_void, count: i32) -> i32 {
        if count < 1000 {
            1
        } else {
            0
        }
    }

    #[test]
    fn test_sqlite3_busy_handler() {
        unsafe {
            let mut db: *mut sqlite3 = ptr::null_mut();
            assert_eq!(sqlite3_open(c":memory:".as_ptr(), &mut db), SQLITE_OK);

            // Test setting a custom busy handler with context
            let mut max_retries: i32 = 3;
            assert_eq!(
                sqlite3_busy_handler(
                    db,
                    Some(busy_handler_retry_n),
                    &mut max_retries as *mut i32 as *mut libc::c_void
                ),
                SQLITE_OK
            );

            // Test clearing the busy handler by passing NULL callback
            assert_eq!(sqlite3_busy_handler(db, None, ptr::null_mut()), SQLITE_OK);

            // Test setting busy handler that never retries
            assert_eq!(
                sqlite3_busy_handler(db, Some(busy_handler_never_retry), ptr::null_mut()),
                SQLITE_OK
            );

            // Test setting busy handler that always retries
            assert_eq!(
                sqlite3_busy_handler(db, Some(busy_handler_always_retry), ptr::null_mut()),
                SQLITE_OK
            );

            // Test that busy_timeout clears a previously set busy_handler
            assert_eq!(
                sqlite3_busy_handler(
                    db,
                    Some(busy_handler_retry_n),
                    &mut max_retries as *mut i32 as *mut libc::c_void
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_busy_timeout(db, 500), SQLITE_OK);

            // Test that busy_handler clears a previously set busy_timeout
            assert_eq!(sqlite3_busy_timeout(db, 1000), SQLITE_OK);
            assert_eq!(
                sqlite3_busy_handler(
                    db,
                    Some(busy_handler_retry_n),
                    &mut max_retries as *mut i32 as *mut libc::c_void
                ),
                SQLITE_OK
            );

            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    struct ProgressContext {
        calls: std::sync::atomic::AtomicI32,
        interrupt_after: i32,
    }

    unsafe extern "C" fn progress_handler_interrupt_after(data: *mut libc::c_void) -> i32 {
        let ctx = &*(data as *const ProgressContext);
        let current = ctx.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
        if current >= ctx.interrupt_after {
            1
        } else {
            0
        }
    }

    #[test]
    fn test_sqlite3_progress_handler_interrupts_statement() {
        unsafe {
            let mut db: *mut sqlite3 = ptr::null_mut();
            assert_eq!(sqlite3_open(c":memory:".as_ptr(), &mut db), SQLITE_OK);
            assert_eq!(
                sqlite3_exec(
                    db,
                    c"CREATE TABLE numbers(v INTEGER); INSERT INTO numbers(v) VALUES (0),(1),(2),(3),(4),(5),(6),(7),(8),(9);".as_ptr(),
                    None,
                    ptr::null_mut(),
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );

            let ctx = ProgressContext {
                calls: std::sync::atomic::AtomicI32::new(0),
                interrupt_after: 8,
            };
            sqlite3_progress_handler(
                db,
                1,
                Some(progress_handler_interrupt_after),
                &ctx as *const ProgressContext as *mut libc::c_void,
            );

            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT count(*) FROM numbers AS a, numbers AS b, numbers AS c, numbers AS d, numbers AS e, numbers AS f, numbers AS g".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );

            assert_eq!(sqlite3_step(stmt), SQLITE_INTERRUPT);
            assert!(
                ctx.calls.load(std::sync::atomic::Ordering::SeqCst) >= ctx.interrupt_after,
                "progress handler did not fire enough times"
            );

            sqlite3_progress_handler(db, 0, None, ptr::null_mut());
            let finalize_rc = sqlite3_finalize(stmt);
            assert!(
                matches!(finalize_rc, SQLITE_OK | SQLITE_INTERRUPT),
                "unexpected finalize rc: {finalize_rc}"
            );
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_sqlite3_interrupt_without_active_statement_is_ignored() {
        unsafe {
            let mut db: *mut sqlite3 = ptr::null_mut();
            assert_eq!(sqlite3_open(c":memory:".as_ptr(), &mut db), SQLITE_OK);

            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(db, c"SELECT 1".as_ptr(), -1, &mut stmt, ptr::null_mut()),
                SQLITE_OK
            );

            sqlite3_interrupt(db);
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);

            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_sqlite3_interrupt_active_statement() {
        unsafe {
            let mut db: *mut sqlite3 = ptr::null_mut();
            assert_eq!(sqlite3_open(c":memory:".as_ptr(), &mut db), SQLITE_OK);
            assert_eq!(
                sqlite3_exec(
                    db,
                    c"CREATE TABLE numbers(v INTEGER); INSERT INTO numbers(v) VALUES (0),(1),(2),(3),(4),(5),(6),(7),(8),(9);".as_ptr(),
                    None,
                    ptr::null_mut(),
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );

            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT a.v FROM numbers AS a, numbers AS b, numbers AS c, numbers AS d, numbers AS e, numbers AS f, numbers AS g, numbers AS h".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );

            let db_addr = db as usize;
            let interrupter = std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(10));
                sqlite3_interrupt(db_addr as *mut sqlite3);
            });

            let rc = loop {
                let rc = sqlite3_step(stmt);
                if rc != SQLITE_ROW {
                    break rc;
                }
            };
            interrupter.join().unwrap();
            assert_eq!(rc, SQLITE_INTERRUPT, "expected SQLITE_INTERRUPT, got {rc}");

            let finalize_rc = sqlite3_finalize(stmt);
            assert!(
                matches!(finalize_rc, SQLITE_OK | SQLITE_INTERRUPT),
                "unexpected finalize rc: {finalize_rc}"
            );
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_sqlite3_get_table() {
        unsafe {
            let mut db: *mut sqlite3 = ptr::null_mut();
            assert_eq!(sqlite3_open(c":memory:".as_ptr(), &mut db), SQLITE_OK);

            // Create and populate a table
            assert_eq!(
                sqlite3_exec(
                    db,
                    c"CREATE TABLE t1(id INTEGER, name TEXT)".as_ptr(),
                    None,
                    ptr::null_mut(),
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );
            assert_eq!(
                sqlite3_exec(
                    db,
                    c"INSERT INTO t1 VALUES(1, 'alice')".as_ptr(),
                    None,
                    ptr::null_mut(),
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );
            assert_eq!(
                sqlite3_exec(
                    db,
                    c"INSERT INTO t1 VALUES(2, 'bob')".as_ptr(),
                    None,
                    ptr::null_mut(),
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );

            // Query via sqlite3_get_table
            let mut result: *mut *mut libc::c_char = ptr::null_mut();
            let mut n_row: libc::c_int = 0;
            let mut n_col: libc::c_int = 0;
            let mut err_msg: *mut libc::c_char = ptr::null_mut();
            assert_eq!(
                sqlite3_get_table(
                    db,
                    c"SELECT id, name FROM t1 ORDER BY id".as_ptr(),
                    &mut result,
                    &mut n_row,
                    &mut n_col,
                    &mut err_msg,
                ),
                SQLITE_OK
            );

            assert_eq!(n_row, 2);
            assert_eq!(n_col, 2);

            // result layout: [col0_name, col1_name, row0_val0, row0_val1, row1_val0, row1_val1]
            let col0 = std::ffi::CStr::from_ptr(*result.add(0));
            let col1 = std::ffi::CStr::from_ptr(*result.add(1));
            assert_eq!(col0.to_str().unwrap(), "id");
            assert_eq!(col1.to_str().unwrap(), "name");

            let r0v0 = std::ffi::CStr::from_ptr(*result.add(2));
            let r0v1 = std::ffi::CStr::from_ptr(*result.add(3));
            assert_eq!(r0v0.to_str().unwrap(), "1");
            assert_eq!(r0v1.to_str().unwrap(), "alice");

            let r1v0 = std::ffi::CStr::from_ptr(*result.add(4));
            let r1v1 = std::ffi::CStr::from_ptr(*result.add(5));
            assert_eq!(r1v0.to_str().unwrap(), "2");
            assert_eq!(r1v1.to_str().unwrap(), "bob");

            sqlite3_free_table(result);

            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_sqlite3_get_table_empty_result() {
        unsafe {
            let mut db: *mut sqlite3 = ptr::null_mut();
            assert_eq!(sqlite3_open(c":memory:".as_ptr(), &mut db), SQLITE_OK);

            assert_eq!(
                sqlite3_exec(
                    db,
                    c"CREATE TABLE t1(id INTEGER)".as_ptr(),
                    None,
                    ptr::null_mut(),
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );

            let mut result: *mut *mut libc::c_char = ptr::null_mut();
            let mut n_row: libc::c_int = 0;
            let mut n_col: libc::c_int = 0;
            assert_eq!(
                sqlite3_get_table(
                    db,
                    c"SELECT id FROM t1".as_ptr(),
                    &mut result,
                    &mut n_row,
                    &mut n_col,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );

            assert_eq!(n_row, 0);
            assert_eq!(n_col, 0);

            sqlite3_free_table(result);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_sqlite3_free_table_null() {
        unsafe {
            // Passing null should not crash
            sqlite3_free_table(ptr::null_mut());
        }
    }

    /// Regression test: sqlite3_value_blob returns a dangling pointer (use-after-free).
    ///
    /// The internal `ExtValue::to_blob()` returns `Option<Vec<u8>>`, so
    /// `sqlite3_value_blob` calls `.as_ptr()` on a temporary Vec that is
    /// immediately dropped, leaving the returned pointer dangling.
    ///
    /// This test creates a scalar function that receives a BLOB argument,
    /// reads it via `sqlite3_value_blob` + `sqlite3_value_bytes`, and
    /// verifies the data is correct. On the buggy version, the blob data
    /// is corrupted (reads freed memory).
    #[test]
    fn test_sqlite3_value_blob_use_after_free() {
        /// Scalar function callback: verifies that the first argument is a BLOB
        /// with the expected content [0xDE, 0xAD, 0xBE, 0xEF].
        /// Returns 1 if the blob matches, 0 otherwise.
        unsafe extern "C" fn check_blob_fn(
            ctx: *mut libc::c_void,
            argc: i32,
            argv: *mut *mut libc::c_void,
        ) {
            assert_eq!(argc, 1);
            let value = *argv.add(0);
            assert!(!value.is_null());

            let vtype = sqlite3_value_type(value);
            if vtype != SQLITE_BLOB {
                sqlite3_result_int64(ctx, -1); // wrong type
                return;
            }

            let blob_ptr = sqlite3_value_blob(value);
            let blob_len = sqlite3_value_bytes(value);

            if blob_ptr.is_null() || blob_len != 4 {
                sqlite3_result_int64(ctx, -2); // null ptr or wrong length
                return;
            }

            // Read the blob data
            let data = std::slice::from_raw_parts(blob_ptr as *const u8, blob_len as usize);
            let expected: &[u8] = &[0xDE, 0xAD, 0xBE, 0xEF];
            if data == expected {
                sqlite3_result_int64(ctx, 1); // match
            } else {
                // Print actual bytes for debugging
                eprintln!("blob mismatch: expected {expected:?}, got {data:?}");
                sqlite3_result_int64(ctx, 0); // corrupted data
            }
        }

        unsafe {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("test_value_blob.db");
            let path_cstr = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
            let mut db: *mut sqlite3 = ptr::null_mut();
            assert_eq!(sqlite3_open(path_cstr.as_ptr(), &mut db), SQLITE_OK);

            // Register the scalar function
            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    c"check_blob".as_ptr(),
                    1,
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    Some(check_blob_fn),
                    None,
                    None,
                    None,
                ),
                SQLITE_OK,
                "create function failed"
            );

            // Create table and insert a row with a known BLOB
            let mut errmsg: *mut libc::c_char = ptr::null_mut();
            assert_eq!(
                sqlite3_exec(
                    db,
                    c"CREATE TABLE t (id INTEGER PRIMARY KEY, data BLOB)".as_ptr(),
                    None,
                    ptr::null_mut(),
                    &mut errmsg,
                ),
                SQLITE_OK
            );

            // Insert using prepared statement with bound BLOB
            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"INSERT INTO t (id, data) VALUES (1, ?1)".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );
            let blob_data: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];
            assert_eq!(
                sqlite3_bind_blob(stmt, 1, blob_data.as_ptr() as *const libc::c_void, 4, None,),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_DONE);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            // Call check_blob(data) via SELECT — this triggers sqlite3_value_blob
            // on the ExtValue passed to the scalar function
            let mut stmt2: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT check_blob(data) FROM t WHERE id = 1".as_ptr(),
                    -1,
                    &mut stmt2,
                    ptr::null_mut(),
                ),
                SQLITE_OK,
                "prepare SELECT with check_blob"
            );
            let rc = sqlite3_step(stmt2);
            assert_eq!(rc, SQLITE_ROW, "expected SQLITE_ROW, got {rc}");

            let result = sqlite3_column_int(stmt2, 0);
            assert_eq!(
                result, 1,
                "check_blob returned {result}: blob data was corrupted (use-after-free in sqlite3_value_blob)"
            );

            assert_eq!(sqlite3_finalize(stmt2), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_sqlite3_prepare_v3() {
        unsafe {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("test.db");
            let path_cstr = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
            let mut db: *mut sqlite3 = ptr::null_mut();
            assert_eq!(sqlite3_open(path_cstr.as_ptr(), &mut db), SQLITE_OK);

            let mut errmsg: *mut libc::c_char = ptr::null_mut();
            assert_eq!(
                sqlite3_exec(
                    db,
                    c"CREATE TABLE t1 (id INTEGER PRIMARY KEY, val TEXT);".as_ptr(),
                    None,
                    ptr::null_mut(),
                    &mut errmsg,
                ),
                SQLITE_OK
            );

            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v3(
                    db,
                    c"INSERT INTO t1 VALUES (?1, ?2)".as_ptr(),
                    -1,
                    SQLITE_PREPARE_PERSISTENT,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );
            assert!(!stmt.is_null());

            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_sqlite3_db_handle() {
        unsafe {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("test.db");
            let path_cstr = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
            let mut db: *mut sqlite3 = ptr::null_mut();
            assert_eq!(sqlite3_open(path_cstr.as_ptr(), &mut db), SQLITE_OK);

            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(db, c"SELECT 1".as_ptr(), -1, &mut stmt, ptr::null_mut()),
                SQLITE_OK
            );
            assert!(!stmt.is_null());

            let returned_db = sqlite3_db_handle(stmt);
            assert_eq!(returned_db, db);

            let null_db = sqlite3_db_handle(ptr::null_mut());
            assert!(null_db.is_null());

            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    /// Test: sqlite3_initialize must not panic when a global tracing subscriber
    /// is already installed.
    #[test]
    fn test_sqlite3_initialize_no_panic_with_existing_subscriber() {
        let _ = tracing_subscriber::fmt::try_init();

        unsafe {
            let rc = sqlite3_initialize();
            assert_eq!(rc, SQLITE_OK);

            let rc2 = sqlite3_initialize();
            assert_eq!(rc2, SQLITE_OK);
        }
    }

    /// Test: URI filename with mode=memory should create an in-memory database.
    #[test]
    fn test_uri_mode_memory() {
        unsafe {
            let dir = tempfile::tempdir().unwrap();
            let db_path = dir.path().join("memtest.db");
            let uri = format!("file:{}?mode=memory&cache=shared", db_path.display());
            let uri_cstr = std::ffi::CString::new(uri).unwrap();

            let mut db: *mut sqlite3 = ptr::null_mut();
            let rc = sqlite3_open_v2(
                uri_cstr.as_ptr(),
                &mut db,
                SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_URI,
                ptr::null(),
            );
            assert_eq!(rc, SQLITE_OK, "open_v2 with URI mode=memory failed rc={rc}");
            assert!(!db.is_null());

            let mut errmsg: *mut libc::c_char = ptr::null_mut();
            assert_eq!(
                sqlite3_exec(
                    db,
                    c"CREATE TABLE t1 (id INTEGER PRIMARY KEY, val TEXT)".as_ptr(),
                    None,
                    ptr::null_mut(),
                    &mut errmsg
                ),
                SQLITE_OK,
                "CREATE TABLE in memory DB failed"
            );
            assert_eq!(
                sqlite3_exec(
                    db,
                    c"INSERT INTO t1 VALUES (1, 'hello')".as_ptr(),
                    None,
                    ptr::null_mut(),
                    &mut errmsg
                ),
                SQLITE_OK,
                "INSERT in memory DB failed"
            );

            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT val FROM t1 WHERE id = 1".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut()
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            let text = sqlite3_column_text(stmt, 0);
            assert!(!text.is_null());
            assert_eq!(std::ffi::CStr::from_ptr(text).to_str().unwrap(), "hello");
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);

            assert!(
                !db_path.exists(),
                "URI mode=memory should NOT create a file on disk"
            );
        }
    }

    /// Test: file::memory: URI should behave like :memory:
    #[test]
    fn test_uri_file_colon_memory() {
        unsafe {
            let mut db: *mut sqlite3 = ptr::null_mut();
            let rc = sqlite3_open_v2(
                c"file::memory:".as_ptr(),
                &mut db,
                SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_URI,
                ptr::null(),
            );
            assert_eq!(rc, SQLITE_OK, "open_v2 with file::memory: failed rc={rc}");

            let mut errmsg: *mut libc::c_char = ptr::null_mut();
            assert_eq!(
                sqlite3_exec(
                    db,
                    c"CREATE TABLE t1 (id INTEGER PRIMARY KEY)".as_ptr(),
                    None,
                    ptr::null_mut(),
                    &mut errmsg
                ),
                SQLITE_OK
            );
            assert_eq!(
                sqlite3_exec(
                    db,
                    c"INSERT INTO t1 VALUES (42)".as_ptr(),
                    None,
                    ptr::null_mut(),
                    &mut errmsg
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_close(db), SQLITE_OK);

            let mut db2: *mut sqlite3 = ptr::null_mut();
            let rc = sqlite3_open_v2(
                c"file::memory:".as_ptr(),
                &mut db2,
                SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_URI,
                ptr::null(),
            );
            assert_eq!(rc, SQLITE_OK);

            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            let rc = sqlite3_prepare_v2(
                db2,
                c"SELECT id FROM t1".as_ptr(),
                -1,
                &mut stmt,
                ptr::null_mut(),
            );
            assert_eq!(
                rc, SQLITE_ERROR,
                "file::memory: data persisted — not truly in-memory (rc={rc})"
            );
            assert_eq!(sqlite3_close(db2), SQLITE_OK);
        }
    }

    /// Test: URI with file: prefix and no query params should open a real file.
    #[test]
    fn test_uri_file_path() {
        unsafe {
            let dir = tempfile::tempdir().unwrap();
            let db_path = dir.path().join("uri_test.db");
            let uri = format!("file:{}", db_path.to_str().unwrap());
            let uri_cstr = std::ffi::CString::new(uri).unwrap();

            let mut db: *mut sqlite3 = ptr::null_mut();
            let rc = sqlite3_open_v2(
                uri_cstr.as_ptr(),
                &mut db,
                SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_URI,
                ptr::null(),
            );
            assert_eq!(rc, SQLITE_OK, "open_v2 with file: URI path failed rc={rc}");

            let mut errmsg: *mut libc::c_char = ptr::null_mut();
            assert_eq!(
                sqlite3_exec(
                    db,
                    c"CREATE TABLE t1 (id INTEGER PRIMARY KEY)".as_ptr(),
                    None,
                    ptr::null_mut(),
                    &mut errmsg
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_close(db), SQLITE_OK);

            assert!(
                db_path.exists(),
                "file: URI with path should create a real database file"
            );
        }
    }

    #[test]
    #[cfg(not(feature = "sqlite3"))]
    fn test_shared_memory_uri_connections() {
        unsafe {
            let uri = c"file:shared_mem_test?mode=memory&cache=shared";
            let flags = SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_URI;

            let mut db1: *mut sqlite3 = ptr::null_mut();
            assert_eq!(
                sqlite3_open_v2(uri.as_ptr(), &mut db1, flags, ptr::null()),
                SQLITE_OK,
                "first open failed"
            );

            let mut errmsg: *mut libc::c_char = ptr::null_mut();
            assert_eq!(
                sqlite3_exec(
                    db1,
                    c"CREATE TABLE t1 (id INTEGER PRIMARY KEY, val TEXT)".as_ptr(),
                    None,
                    ptr::null_mut(),
                    &mut errmsg,
                ),
                SQLITE_OK,
                "CREATE TABLE failed"
            );
            assert_eq!(
                sqlite3_exec(
                    db1,
                    c"INSERT INTO t1 VALUES (1, 'from_conn1')".as_ptr(),
                    None,
                    ptr::null_mut(),
                    &mut errmsg,
                ),
                SQLITE_OK,
                "INSERT failed"
            );

            // Second connection to the same URI must see shared state
            let mut db2: *mut sqlite3 = ptr::null_mut();
            assert_eq!(
                sqlite3_open_v2(uri.as_ptr(), &mut db2, flags, ptr::null()),
                SQLITE_OK,
                "second open failed"
            );

            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db2,
                    c"SELECT val FROM t1 WHERE id = 1".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK,
                "SELECT on connection 2 failed — table not shared"
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            let text = sqlite3_column_text(stmt, 0);
            assert!(!text.is_null());
            let val = std::ffi::CStr::from_ptr(text).to_str().unwrap();
            assert_eq!(val, "from_conn1");
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            assert_eq!(sqlite3_close(db2), SQLITE_OK);
            assert_eq!(sqlite3_close(db1), SQLITE_OK);
        }
    }

    #[test]
    #[cfg(not(feature = "sqlite3"))]
    fn test_different_named_memory_dbs_are_independent() {
        unsafe {
            let flags = SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_URI;

            // Open db "alpha"
            let mut db1: *mut sqlite3 = ptr::null_mut();
            assert_eq!(
                sqlite3_open_v2(
                    c"file:alpha_iso?mode=memory&cache=shared".as_ptr(),
                    &mut db1,
                    flags,
                    ptr::null(),
                ),
                SQLITE_OK
            );
            let mut errmsg: *mut libc::c_char = ptr::null_mut();
            assert_eq!(
                sqlite3_exec(
                    db1,
                    c"CREATE TABLE t1 (id INTEGER PRIMARY KEY)".as_ptr(),
                    None,
                    ptr::null_mut(),
                    &mut errmsg,
                ),
                SQLITE_OK
            );

            // Open db "beta" — should NOT see t1
            let mut db2: *mut sqlite3 = ptr::null_mut();
            assert_eq!(
                sqlite3_open_v2(
                    c"file:beta_iso?mode=memory&cache=shared".as_ptr(),
                    &mut db2,
                    flags,
                    ptr::null(),
                ),
                SQLITE_OK
            );
            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            let rc = sqlite3_prepare_v2(
                db2,
                c"SELECT * FROM t1".as_ptr(),
                -1,
                &mut stmt,
                ptr::null_mut(),
            );
            // t1 only exists in "alpha", not "beta"
            assert_eq!(rc, SQLITE_ERROR);

            assert_eq!(sqlite3_close(db2), SQLITE_OK);
            assert_eq!(sqlite3_close(db1), SQLITE_OK);
        }
    }

    #[test]
    #[cfg(not(feature = "sqlite3"))]
    fn test_plain_memory_stays_independent() {
        unsafe {
            let flags = SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_URI;

            let mut db1: *mut sqlite3 = ptr::null_mut();
            assert_eq!(
                sqlite3_open_v2(c"file::memory:".as_ptr(), &mut db1, flags, ptr::null(),),
                SQLITE_OK
            );
            let mut errmsg: *mut libc::c_char = ptr::null_mut();
            assert_eq!(
                sqlite3_exec(
                    db1,
                    c"CREATE TABLE t1 (id INTEGER PRIMARY KEY)".as_ptr(),
                    None,
                    ptr::null_mut(),
                    &mut errmsg,
                ),
                SQLITE_OK
            );

            // Unnamed :memory: must remain independent
            let mut db2: *mut sqlite3 = ptr::null_mut();
            assert_eq!(
                sqlite3_open_v2(c"file::memory:".as_ptr(), &mut db2, flags, ptr::null(),),
                SQLITE_OK
            );
            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            let rc = sqlite3_prepare_v2(
                db2,
                c"SELECT * FROM t1".as_ptr(),
                -1,
                &mut stmt,
                ptr::null_mut(),
            );
            assert_eq!(rc, SQLITE_ERROR, "plain :memory: should not share state");

            assert_eq!(sqlite3_close(db2), SQLITE_OK);
            assert_eq!(sqlite3_close(db1), SQLITE_OK);
        }
    }

    #[test]
    #[cfg(not(feature = "sqlite3"))]
    fn test_unnamed_memory_cache_shared() {
        unsafe {
            let uri = c"file::memory:?cache=shared";
            let flags = SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_URI;

            let mut db1: *mut sqlite3 = ptr::null_mut();
            assert_eq!(
                sqlite3_open_v2(uri.as_ptr(), &mut db1, flags, ptr::null()),
                SQLITE_OK,
            );
            let mut errmsg: *mut libc::c_char = ptr::null_mut();
            assert_eq!(
                sqlite3_exec(
                    db1,
                    c"CREATE TABLE t1 (id INTEGER PRIMARY KEY, val TEXT)".as_ptr(),
                    None,
                    ptr::null_mut(),
                    &mut errmsg,
                ),
                SQLITE_OK,
            );
            assert_eq!(
                sqlite3_exec(
                    db1,
                    c"INSERT INTO t1 VALUES (1, 'shared')".as_ptr(),
                    None,
                    ptr::null_mut(),
                    &mut errmsg,
                ),
                SQLITE_OK,
            );

            // Same URI must share the database
            let mut db2: *mut sqlite3 = ptr::null_mut();
            assert_eq!(
                sqlite3_open_v2(uri.as_ptr(), &mut db2, flags, ptr::null()),
                SQLITE_OK,
            );
            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db2,
                    c"SELECT val FROM t1 WHERE id = 1".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK,
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            let text = sqlite3_column_text(stmt, 0);
            assert!(!text.is_null());
            let val = std::ffi::CStr::from_ptr(text).to_str().unwrap();
            assert_eq!(val, "shared");
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            assert_eq!(sqlite3_close(db2), SQLITE_OK);
            assert_eq!(sqlite3_close(db1), SQLITE_OK);
        }
    }

    #[test]
    #[cfg(not(feature = "sqlite3"))]
    fn test_sqlite3_column_value_and_value_dup_free() {
        unsafe {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("test.db");
            let path_cstr = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
            let mut db: *mut sqlite3 = ptr::null_mut();
            assert_eq!(sqlite3_open(path_cstr.as_ptr(), &mut db), SQLITE_OK);

            let mut errmsg: *mut libc::c_char = ptr::null_mut();
            assert_eq!(
                sqlite3_exec(
                    db,
                    c"CREATE TABLE t2 (i INTEGER, f REAL, t TEXT, b BLOB);".as_ptr(),
                    None,
                    ptr::null_mut(),
                    &mut errmsg,
                ),
                SQLITE_OK
            );
            assert_eq!(
                sqlite3_exec(
                    db,
                    c"INSERT INTO t2 VALUES (42, 1.5, 'hello', X'DEADBEEF');".as_ptr(),
                    None,
                    ptr::null_mut(),
                    &mut errmsg,
                ),
                SQLITE_OK
            );

            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT i, f, t, b FROM t2".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);

            let val0 = sqlite3_column_value(stmt, 0);
            assert!(!val0.is_null());
            assert_eq!(sqlite3_value_type(val0), SQLITE_INTEGER);
            assert_eq!(sqlite3_value_int64(val0), 42);

            let val1 = sqlite3_column_value(stmt, 1);
            assert!(!val1.is_null());
            assert_eq!(sqlite3_value_type(val1), SQLITE_FLOAT);
            assert!((sqlite3_value_double(val1) - 1.5).abs() < 0.001);

            let val2 = sqlite3_column_value(stmt, 2);
            assert!(!val2.is_null());
            assert_eq!(sqlite3_value_type(val2), SQLITE_TEXT);
            let text_ptr = sqlite3_value_text(val2);
            assert!(!text_ptr.is_null());
            let text_len = sqlite3_value_bytes(val2) as usize;
            let text =
                std::str::from_utf8(std::slice::from_raw_parts(text_ptr as *const u8, text_len))
                    .unwrap();
            assert_eq!(text, "hello");

            let val3 = sqlite3_column_value(stmt, 3);
            assert!(!val3.is_null());
            assert_eq!(sqlite3_value_type(val3), SQLITE_BLOB);
            assert_eq!(sqlite3_value_bytes(val3), 4);

            let dup = sqlite3_value_dup(val0);
            assert!(!dup.is_null());
            assert_eq!(sqlite3_value_type(dup), SQLITE_INTEGER);
            assert_eq!(sqlite3_value_int64(dup), 42);

            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(sqlite3_value_int64(dup), 42);

            sqlite3_value_free(dup);

            sqlite3_value_free(ptr::null_mut());
            let null_dup = sqlite3_value_dup(ptr::null_mut());
            assert!(null_dup.is_null());

            let null_val = sqlite3_column_value(ptr::null_mut(), 0);
            assert!(null_val.is_null());

            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    #[cfg(not(feature = "sqlite3"))]
    fn test_sqlite3_context_db_handle() {
        unsafe {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("test.db");
            let path_cstr = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
            let mut db: *mut sqlite3 = ptr::null_mut();
            assert_eq!(sqlite3_open(path_cstr.as_ptr(), &mut db), SQLITE_OK);

            use std::sync::atomic::{AtomicPtr, Ordering};
            static CAPTURED_DB: AtomicPtr<libc::c_void> = AtomicPtr::new(ptr::null_mut());

            unsafe extern "C" fn test_func(
                ctx: *mut libc::c_void,
                _argc: i32,
                _argv: *mut *mut libc::c_void,
            ) {
                CAPTURED_DB.store(sqlite3_context_db_handle(ctx), Ordering::SeqCst);
                sqlite3_result_int(ctx, 1);
            }

            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    c"test_db_handle".as_ptr(),
                    0,
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    Some(test_func),
                    None,
                    None,
                    None,
                ),
                SQLITE_OK
            );

            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT test_db_handle()".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut(),
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);

            assert_eq!(CAPTURED_DB.load(Ordering::SeqCst), db as *mut libc::c_void);

            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            let null_handle = sqlite3_context_db_handle(ptr::null_mut());
            assert!(null_handle.is_null());

            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    // User-defined functions. Every test below runs against turso and, with
    // --features sqlite3, against the real library, so it asserts SQLite.

    use std::ffi::{CStr, CString};
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A fresh on-disk database. The TempDir must outlive the handle.
    fn open_udf_db() -> (tempfile::TempDir, *mut sqlite3) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("udf.db");
        let path = CString::new(path.to_str().unwrap()).unwrap();
        let mut db: *mut sqlite3 = ptr::null_mut();
        unsafe {
            assert_eq!(sqlite3_open(path.as_ptr(), &mut db), SQLITE_OK);
        }
        (dir, db)
    }

    unsafe fn run_sql(db: *mut sqlite3, sql: &CStr) {
        let mut errmsg: *mut libc::c_char = ptr::null_mut();
        let rc = sqlite3_exec(db, sql.as_ptr(), None, ptr::null_mut(), &mut errmsg);
        assert_eq!(rc, SQLITE_OK, "failed to run {sql:?}");
    }

    unsafe fn query_ints(db: *mut sqlite3, sql: &CStr) -> Vec<i64> {
        let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
        assert_eq!(
            sqlite3_prepare_v2(db, sql.as_ptr(), -1, &mut stmt, ptr::null_mut()),
            SQLITE_OK,
            "failed to prepare {sql:?}"
        );
        let mut rows = Vec::new();
        loop {
            let rc = sqlite3_step(stmt);
            if rc == SQLITE_DONE {
                break;
            }
            assert_eq!(rc, SQLITE_ROW, "unexpected step result for {sql:?}");
            rows.push(sqlite3_column_int64(stmt, 0));
        }
        assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
        rows
    }

    unsafe fn query_texts(db: *mut sqlite3, sql: &CStr) -> Vec<String> {
        let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
        assert_eq!(
            sqlite3_prepare_v2(db, sql.as_ptr(), -1, &mut stmt, ptr::null_mut()),
            SQLITE_OK,
            "failed to prepare {sql:?}"
        );
        let mut rows = Vec::new();
        loop {
            let rc = sqlite3_step(stmt);
            if rc == SQLITE_DONE {
                break;
            }
            assert_eq!(rc, SQLITE_ROW, "unexpected step result for {sql:?}");
            let text = sqlite3_column_text(stmt, 0);
            rows.push(if text.is_null() {
                String::new()
            } else {
                CStr::from_ptr(text).to_str().unwrap().to_string()
            });
        }
        assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
        rows
    }

    unsafe fn errmsg(db: *mut sqlite3) -> String {
        let msg = sqlite3_errmsg(db);
        if msg.is_null() {
            String::new()
        } else {
            CStr::from_ptr(msg).to_string_lossy().into_owned()
        }
    }

    /// Never dereferenced, only handed back to the destructor.
    fn marker_p_app() -> *mut libc::c_void {
        std::ptr::dangling_mut::<libc::c_void>()
    }

    unsafe extern "C" fn no_op_step(
        _ctx: *mut libc::c_void,
        _argc: i32,
        _argv: *mut *mut libc::c_void,
    ) {
    }
    unsafe extern "C" fn no_op_final(_ctx: *mut libc::c_void) {}

    /// Mirrors SQLite's `abuse_create_function` test.
    #[test]
    fn test_create_function_rejects_illegal_registrations() {
        unsafe {
            let (_dir, db) = open_udf_db();
            let name = c"tx".as_ptr();
            let step: XFunc = Some(no_op_step);
            let fin: XFinal = Some(no_op_final);

            // xFunc together with xFinal.
            assert_eq!(
                sqlite3_create_function(db, name, 1, SQLITE_UTF8, ptr::null_mut(), step, step, fin),
                SQLITE_MISUSE
            );
            // xStep without xFinal.
            assert_eq!(
                sqlite3_create_function(
                    db,
                    name,
                    1,
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    step,
                    step,
                    None
                ),
                SQLITE_MISUSE
            );
            // xFunc with xFinal but no xStep.
            assert_eq!(
                sqlite3_create_function(db, name, 1, SQLITE_UTF8, ptr::null_mut(), step, None, fin),
                SQLITE_MISUSE
            );
            // xFinal without xStep.
            assert_eq!(
                sqlite3_create_function(db, name, 1, SQLITE_UTF8, ptr::null_mut(), None, None, fin),
                SQLITE_MISUSE
            );
            // xStep without xFinal, again with no xFunc.
            assert_eq!(
                sqlite3_create_function(
                    db,
                    name,
                    1,
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    None,
                    step,
                    None
                ),
                SQLITE_MISUSE
            );
            // Argument counts outside [-1, 127].
            assert_eq!(
                sqlite3_create_function(
                    db,
                    name,
                    -2,
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    step,
                    None,
                    None
                ),
                SQLITE_MISUSE
            );
            assert_eq!(
                sqlite3_create_function(
                    db,
                    name,
                    128,
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    step,
                    None,
                    None
                ),
                SQLITE_MISUSE
            );
            // A name one byte over the 255-byte limit.
            let too_long = CString::new("f".repeat(256)).unwrap();
            assert_eq!(
                sqlite3_create_function(
                    db,
                    too_long.as_ptr(),
                    1,
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    step,
                    None,
                    None
                ),
                SQLITE_MISUSE
            );
            assert_eq!(
                sqlite3_create_function(
                    db,
                    ptr::null(),
                    1,
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    step,
                    None,
                    None
                ),
                SQLITE_MISUSE
            );
            // The longest legal name, with an argument count every build
            // takes: older SQLite caps SQLITE_MAX_FUNCTION_ARG at 127.
            let longest = CString::new("f".repeat(255)).unwrap();
            assert_eq!(
                sqlite3_create_function(
                    db,
                    longest.as_ptr(),
                    127,
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    step,
                    None,
                    None
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    /// Each destructor runs once, never twice, as func3.test checks.
    #[test]
    fn test_destructor_runs_on_replace_delete_and_close() {
        static DESTROYED: AtomicUsize = AtomicUsize::new(0);
        unsafe extern "C" fn count_destroy(_p_app: *mut libc::c_void) {
            DESTROYED.fetch_add(1, Ordering::SeqCst);
        }
        unsafe extern "C" fn answer(
            ctx: *mut libc::c_void,
            _argc: i32,
            _argv: *mut *mut libc::c_void,
        ) {
            sqlite3_result_int(ctx, 1);
        }

        unsafe {
            DESTROYED.store(0, Ordering::SeqCst);
            let (_dir, db) = open_udf_db();
            let name = c"d".as_ptr();
            let register = |destroy: XDestroy| {
                sqlite3_create_function_v2(
                    db,
                    name,
                    0,
                    SQLITE_UTF8,
                    marker_p_app(),
                    Some(answer),
                    None,
                    None,
                    destroy,
                )
            };

            assert_eq!(register(Some(count_destroy)), SQLITE_OK);
            assert_eq!(
                DESTROYED.load(Ordering::SeqCst),
                0,
                "a live registration keeps its data"
            );

            assert_eq!(register(Some(count_destroy)), SQLITE_OK);
            assert_eq!(DESTROYED.load(Ordering::SeqCst), 1);

            // All-NULL callbacks delete the function.
            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    name,
                    0,
                    SQLITE_UTF8,
                    marker_p_app(),
                    None,
                    None,
                    None,
                    Some(count_destroy),
                ),
                SQLITE_OK
            );
            // The deleting call's own data is not destroyed yet: SQLite keeps
            // it on the emptied entry.
            assert_eq!(DESTROYED.load(Ordering::SeqCst), 2);

            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            assert_ne!(
                sqlite3_prepare_v2(db, c"SELECT d()".as_ptr(), -1, &mut stmt, ptr::null_mut()),
                SQLITE_OK
            );

            // Registering the name again claims the emptied entry and
            // destroys what the deletion left on it.
            assert_eq!(register(Some(count_destroy)), SQLITE_OK);
            assert_eq!(DESTROYED.load(Ordering::SeqCst), 3);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
            assert_eq!(
                DESTROYED.load(Ordering::SeqCst),
                4,
                "closing destroys what is left"
            );
        }
    }

    /// A refused registration never stores pApp, so SQLite destroys it first.
    #[test]
    fn test_destructor_runs_when_registration_is_refused() {
        static DESTROYED: AtomicUsize = AtomicUsize::new(0);
        unsafe extern "C" fn count_destroy(_p_app: *mut libc::c_void) {
            DESTROYED.fetch_add(1, Ordering::SeqCst);
        }

        unsafe {
            DESTROYED.store(0, Ordering::SeqCst);
            let (_dir, db) = open_udf_db();
            // xFunc together with xFinal is misuse.
            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    c"bad".as_ptr(),
                    1,
                    SQLITE_UTF8,
                    marker_p_app(),
                    Some(no_op_step),
                    None,
                    Some(no_op_final),
                    Some(count_destroy),
                ),
                SQLITE_MISUSE
            );
            assert_eq!(DESTROYED.load(Ordering::SeqCst), 1);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
            assert_eq!(DESTROYED.load(Ordering::SeqCst), 1);
        }
    }

    /// The accumulator is zero-filled and keeps the size the first call asked for.
    #[test]
    fn test_aggregate_sums_rows_and_finalizes_empty_groups() {
        static FINAL_CALLS: AtomicUsize = AtomicUsize::new(0);

        unsafe extern "C" fn sum_step(
            ctx: *mut libc::c_void,
            argc: i32,
            argv: *mut *mut libc::c_void,
        ) {
            assert_eq!(argc, 1);
            let total = sqlite3_aggregate_context(ctx, 8) as *mut i64;
            assert!(!total.is_null());
            // A later call gets the same buffer whatever size it asks for.
            assert_eq!(
                sqlite3_aggregate_context(ctx, 64),
                total as *mut libc::c_void
            );
            *total += sqlite3_value_int64(*argv.add(0));
        }
        unsafe extern "C" fn sum_final(ctx: *mut libc::c_void) {
            FINAL_CALLS.fetch_add(1, Ordering::SeqCst);
            // Asking for nothing must not allocate, so an empty group has a
            // null accumulator.
            let total = sqlite3_aggregate_context(ctx, 0) as *mut i64;
            if total.is_null() {
                sqlite3_result_int64(ctx, -1);
            } else {
                sqlite3_result_int64(ctx, *total);
            }
        }

        unsafe {
            FINAL_CALLS.store(0, Ordering::SeqCst);
            let (_dir, db) = open_udf_db();
            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    c"mysum".as_ptr(),
                    1,
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    None,
                    Some(sum_step),
                    Some(sum_final),
                    None,
                ),
                SQLITE_OK
            );
            run_sql(db, c"CREATE TABLE t(x)");
            run_sql(db, c"INSERT INTO t VALUES (1), (2), (3)");

            assert_eq!(query_ints(db, c"SELECT mysum(x) FROM t"), vec![6]);
            assert_eq!(FINAL_CALLS.load(Ordering::SeqCst), 1);

            // No rows, and xFinal still runs exactly once.
            assert_eq!(
                query_ints(db, c"SELECT mysum(x) FROM t WHERE x > 100"),
                vec![-1]
            );
            assert_eq!(FINAL_CALLS.load(Ordering::SeqCst), 2);

            run_sql(db, c"CREATE TABLE g(k, v)");
            run_sql(db, c"INSERT INTO g VALUES (1, 10), (1, 20), (2, 5)");
            assert_eq!(
                query_ints(db, c"SELECT mysum(v) FROM g GROUP BY k ORDER BY k"),
                vec![30, 5]
            );
            assert_eq!(FINAL_CALLS.load(Ordering::SeqCst), 4);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_window_function_runs_over_a_moving_frame() {
        unsafe extern "C" fn win_step(
            ctx: *mut libc::c_void,
            _argc: i32,
            argv: *mut *mut libc::c_void,
        ) {
            let total = sqlite3_aggregate_context(ctx, 8) as *mut i64;
            *total += sqlite3_value_int64(*argv.add(0));
        }
        unsafe extern "C" fn win_inverse(
            ctx: *mut libc::c_void,
            _argc: i32,
            argv: *mut *mut libc::c_void,
        ) {
            let total = sqlite3_aggregate_context(ctx, 8) as *mut i64;
            *total -= sqlite3_value_int64(*argv.add(0));
        }
        unsafe extern "C" fn win_value(ctx: *mut libc::c_void) {
            let total = sqlite3_aggregate_context(ctx, 8) as *mut i64;
            sqlite3_result_int64(ctx, if total.is_null() { 0 } else { *total });
        }

        unsafe {
            let (_dir, db) = open_udf_db();
            assert_eq!(
                sqlite3_create_window_function(
                    db,
                    c"wsum".as_ptr(),
                    1,
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    Some(win_step),
                    Some(win_value),
                    Some(win_value),
                    Some(win_inverse),
                    None,
                ),
                SQLITE_OK
            );
            run_sql(db, c"CREATE TABLE t(x)");
            run_sql(db, c"INSERT INTO t VALUES (1), (2), (3), (4)");
            assert_eq!(
                query_ints(
                    db,
                    c"SELECT wsum(x) OVER (ORDER BY x ROWS BETWEEN 1 PRECEDING AND CURRENT ROW) FROM t ORDER BY x"
                ),
                vec![1, 3, 5, 7]
            );
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    /// xValue and xInverse must come as a pair, and so must xStep and xFinal.
    #[test]
    fn test_create_window_function_rejects_illegal_registrations() {
        unsafe {
            let (_dir, db) = open_udf_db();
            let name = c"w".as_ptr();
            let step: XFunc = Some(no_op_step);
            let fin: XFinal = Some(no_op_final);

            // xValue without xInverse.
            assert_eq!(
                sqlite3_create_window_function(
                    db,
                    name,
                    1,
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    step,
                    fin,
                    fin,
                    None,
                    None
                ),
                SQLITE_MISUSE
            );
            // xInverse without xValue.
            assert_eq!(
                sqlite3_create_window_function(
                    db,
                    name,
                    1,
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    step,
                    fin,
                    None,
                    step,
                    None
                ),
                SQLITE_MISUSE
            );
            // xStep without xFinal.
            assert_eq!(
                sqlite3_create_window_function(
                    db,
                    name,
                    1,
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    step,
                    None,
                    fin,
                    step,
                    None
                ),
                SQLITE_MISUSE
            );
            // Every callback NULL on a function that is not there is a no-op.
            assert_eq!(
                sqlite3_create_window_function(
                    db,
                    name,
                    1,
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    None,
                    None,
                    None,
                    None,
                    None
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    /// Refused at prepare time.
    #[test]
    fn test_plain_aggregate_may_not_be_used_as_a_window_function() {
        unsafe extern "C" fn count_step(
            ctx: *mut libc::c_void,
            _argc: i32,
            _argv: *mut *mut libc::c_void,
        ) {
            let n = sqlite3_aggregate_context(ctx, 8) as *mut i64;
            *n += 1;
        }
        unsafe extern "C" fn count_final(ctx: *mut libc::c_void) {
            let n = sqlite3_aggregate_context(ctx, 0) as *mut i64;
            sqlite3_result_int64(ctx, if n.is_null() { 0 } else { *n });
        }

        unsafe {
            let (_dir, db) = open_udf_db();
            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    c"plainagg".as_ptr(),
                    1,
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    None,
                    Some(count_step),
                    Some(count_final),
                    None,
                ),
                SQLITE_OK
            );
            run_sql(db, c"CREATE TABLE t(x)");
            run_sql(db, c"INSERT INTO t VALUES (1), (2)");
            assert_eq!(query_ints(db, c"SELECT plainagg(x) FROM t"), vec![2]);

            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            let rc = sqlite3_prepare_v2(
                db,
                c"SELECT plainagg(x) OVER () FROM t".as_ptr(),
                -1,
                &mut stmt,
                ptr::null_mut(),
            );
            assert_ne!(rc, SQLITE_OK);
            assert!(
                errmsg(db).contains("window function"),
                "unexpected message: {}",
                errmsg(db)
            );
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_application_function_overrides_a_builtin() {
        unsafe extern "C" fn sum_step(
            ctx: *mut libc::c_void,
            _argc: i32,
            _argv: *mut *mut libc::c_void,
        ) {
            let n = sqlite3_aggregate_context(ctx, 8) as *mut i64;
            *n += 100;
        }
        unsafe extern "C" fn sum_final(ctx: *mut libc::c_void) {
            let n = sqlite3_aggregate_context(ctx, 0) as *mut i64;
            sqlite3_result_int64(ctx, if n.is_null() { 0 } else { *n });
        }
        unsafe extern "C" fn always_seven(
            ctx: *mut libc::c_void,
            _argc: i32,
            _argv: *mut *mut libc::c_void,
        ) {
            sqlite3_result_int(ctx, 7);
        }

        unsafe {
            let (_dir, db) = open_udf_db();
            run_sql(db, c"CREATE TABLE t(x)");
            run_sql(db, c"INSERT INTO t VALUES (1), (2), (3)");
            assert_eq!(query_ints(db, c"SELECT sum(x) FROM t"), vec![6]);
            assert_eq!(query_ints(db, c"SELECT abs(-5)"), vec![5]);

            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    c"sum".as_ptr(),
                    1,
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    None,
                    Some(sum_step),
                    Some(sum_final),
                    None,
                ),
                SQLITE_OK
            );
            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    c"abs".as_ptr(),
                    1,
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    Some(always_seven),
                    None,
                    None,
                    None,
                ),
                SQLITE_OK
            );
            assert_eq!(query_ints(db, c"SELECT sum(x) FROM t"), vec![300]);
            assert_eq!(query_ints(db, c"SELECT abs(-5)"), vec![7]);
            // A different arity still finds the built-in.
            assert_eq!(query_ints(db, c"SELECT length('abcd')"), vec![4]);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_auxdata_survives_only_for_a_constant_argument() {
        static AUX_FREED: AtomicUsize = AtomicUsize::new(0);
        unsafe extern "C" fn free_aux(data: *mut libc::c_void) {
            AUX_FREED.fetch_add(1, Ordering::SeqCst);
            drop(Box::from_raw(data as *mut u64));
        }
        unsafe extern "C" fn aux_probe(
            ctx: *mut libc::c_void,
            argc: i32,
            _argv: *mut *mut libc::c_void,
        ) {
            let mut marks = Vec::new();
            for arg in 0..argc {
                if sqlite3_get_auxdata(ctx, arg).is_null() {
                    marks.push(b'0');
                    let data = Box::into_raw(Box::new(1u64)) as *mut libc::c_void;
                    sqlite3_set_auxdata(ctx, arg, data, Some(free_aux));
                } else {
                    marks.push(b'1');
                }
            }
            let marks = CString::new(marks).unwrap();
            sqlite3_result_text(ctx, marks.as_ptr(), -1, transient());
        }

        unsafe {
            AUX_FREED.store(0, Ordering::SeqCst);
            let (_dir, db) = open_udf_db();
            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    c"auxprobe".as_ptr(),
                    2,
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    Some(aux_probe),
                    None,
                    None,
                    None,
                ),
                SQLITE_OK
            );
            run_sql(db, c"CREATE TABLE t(x)");
            run_sql(db, c"INSERT INTO t VALUES (1), (2), (3)");
            assert_eq!(
                query_texts(db, c"SELECT auxprobe('constant', x) FROM t ORDER BY x"),
                vec!["00", "10", "10"]
            );
            // Each row dropped the column argument's data; finishing the
            // statement dropped the constant argument's.
            assert_eq!(AUX_FREED.load(Ordering::SeqCst), 4);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_function_errors_reach_step_and_errmsg() {
        unsafe extern "C" fn plain_error(
            ctx: *mut libc::c_void,
            _argc: i32,
            _argv: *mut *mut libc::c_void,
        ) {
            sqlite3_result_error(ctx, c"boom".as_ptr(), -1);
        }
        unsafe extern "C" fn coded_error(
            ctx: *mut libc::c_void,
            _argc: i32,
            _argv: *mut *mut libc::c_void,
        ) {
            sqlite3_result_error(ctx, c"nope".as_ptr(), -1);
            sqlite3_result_error_code(ctx, SQLITE_CONSTRAINT);
        }
        unsafe extern "C" fn too_big(
            ctx: *mut libc::c_void,
            _argc: i32,
            _argv: *mut *mut libc::c_void,
        ) {
            sqlite3_result_error_toobig(ctx);
        }
        unsafe extern "C" fn no_mem(
            ctx: *mut libc::c_void,
            _argc: i32,
            _argv: *mut *mut libc::c_void,
        ) {
            sqlite3_result_error_nomem(ctx);
        }
        // An explicit length may cover an interior NUL; the message is cut there.
        unsafe extern "C" fn nul_error(
            ctx: *mut libc::c_void,
            _argc: i32,
            _argv: *mut *mut libc::c_void,
        ) {
            sqlite3_result_error(ctx, b"a\0b".as_ptr() as *const libc::c_char, 3);
        }
        // A code with no message and a NULL result gets the code's standard message.
        unsafe extern "C" fn code_only(
            ctx: *mut libc::c_void,
            _argc: i32,
            _argv: *mut *mut libc::c_void,
        ) {
            sqlite3_result_error_code(ctx, SQLITE_CONSTRAINT);
        }

        unsafe fn step_error(db: *mut sqlite3, sql: &CStr) -> (i32, String) {
            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(db, sql.as_ptr(), -1, &mut stmt, ptr::null_mut()),
                SQLITE_OK
            );
            let rc = sqlite3_step(stmt);
            let message = errmsg(db);
            sqlite3_finalize(stmt);
            (rc, message)
        }

        unsafe {
            let (_dir, db) = open_udf_db();
            for (name, func) in [
                (
                    c"plain_error",
                    plain_error
                        as unsafe extern "C" fn(*mut libc::c_void, i32, *mut *mut libc::c_void),
                ),
                (c"coded_error", coded_error),
                (c"too_big", too_big),
                (c"no_mem", no_mem),
                (c"nul_error", nul_error),
                (c"code_only", code_only),
            ] {
                assert_eq!(
                    sqlite3_create_function_v2(
                        db,
                        name.as_ptr(),
                        0,
                        SQLITE_UTF8,
                        ptr::null_mut(),
                        Some(func),
                        None,
                        None,
                        None,
                    ),
                    SQLITE_OK
                );
            }

            assert_eq!(
                step_error(db, c"SELECT plain_error()"),
                (SQLITE_ERROR, "boom".to_string())
            );
            assert_eq!(
                step_error(db, c"SELECT coded_error()"),
                (SQLITE_CONSTRAINT, "nope".to_string())
            );
            assert_eq!(
                step_error(db, c"SELECT too_big()"),
                (SQLITE_TOOBIG, "string or blob too big".to_string())
            );
            assert_eq!(
                step_error(db, c"SELECT no_mem()"),
                (SQLITE_NOMEM, "out of memory".to_string())
            );
            assert_eq!(
                step_error(db, c"SELECT nul_error()"),
                (SQLITE_ERROR, "a".to_string())
            );
            assert_eq!(
                step_error(db, c"SELECT code_only()"),
                (SQLITE_CONSTRAINT, "constraint failed".to_string())
            );
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    /// A function may prepare and run SQL on its own connection, including from a
    /// statement that `sqlite3_exec` is driving.
    #[test]
    fn test_function_can_reenter_its_connection_during_exec() {
        unsafe extern "C" fn count_rows(
            ctx: *mut libc::c_void,
            _argc: i32,
            _argv: *mut *mut libc::c_void,
        ) {
            let db = sqlite3_context_db_handle(ctx) as *mut sqlite3;
            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT count(*) FROM t".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut()
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            let count = sqlite3_column_int64(stmt, 0);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            sqlite3_result_int64(ctx, count);
        }

        unsafe {
            let (_dir, db) = open_udf_db();
            run_sql(db, c"CREATE TABLE t(x)");
            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    c"count_rows".as_ptr(),
                    0,
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    Some(count_rows),
                    None,
                    None,
                    None,
                ),
                SQLITE_OK
            );
            run_sql(db, c"INSERT INTO t VALUES (count_rows())");
            run_sql(db, c"INSERT INTO t VALUES (count_rows())");
            assert_eq!(query_ints(db, c"SELECT x FROM t ORDER BY x"), vec![0, 1]);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    /// SQLite is done with the buffer by finalize, and destroys it exactly once.
    #[test]
    fn test_result_text_runs_its_destructor() {
        static FREED: AtomicUsize = AtomicUsize::new(0);
        unsafe extern "C" fn free_text(data: *mut libc::c_void) {
            FREED.fetch_add(1, Ordering::SeqCst);
            drop(CString::from_raw(data as *mut libc::c_char));
        }
        unsafe extern "C" fn owned_text(
            ctx: *mut libc::c_void,
            _argc: i32,
            _argv: *mut *mut libc::c_void,
        ) {
            let text = CString::new("hello").unwrap().into_raw();
            sqlite3_result_text(ctx, text, -1, free_text as *mut libc::c_void);
        }

        unsafe {
            FREED.store(0, Ordering::SeqCst);
            let (_dir, db) = open_udf_db();
            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    c"owned_text".as_ptr(),
                    0,
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    Some(owned_text),
                    None,
                    None,
                    None,
                ),
                SQLITE_OK
            );
            assert_eq!(query_texts(db, c"SELECT owned_text()"), vec!["hello"]);
            assert_eq!(FREED.load(Ordering::SeqCst), 1);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_result_setters_produce_the_right_values() {
        unsafe extern "C" fn give_null(
            ctx: *mut libc::c_void,
            _argc: i32,
            _argv: *mut *mut libc::c_void,
        ) {
            sqlite3_result_null(ctx);
        }
        unsafe extern "C" fn give_double(
            ctx: *mut libc::c_void,
            _argc: i32,
            _argv: *mut *mut libc::c_void,
        ) {
            sqlite3_result_double(ctx, 1.5);
        }
        unsafe extern "C" fn give_blob(
            ctx: *mut libc::c_void,
            _argc: i32,
            _argv: *mut *mut libc::c_void,
        ) {
            let bytes: [u8; 3] = [0xDE, 0xAD, 0x00];
            sqlite3_result_blob(ctx, bytes.as_ptr() as *const libc::c_void, 3, transient());
        }
        unsafe extern "C" fn give_zeroblob(
            ctx: *mut libc::c_void,
            _argc: i32,
            _argv: *mut *mut libc::c_void,
        ) {
            sqlite3_result_zeroblob(ctx, 5);
        }
        unsafe extern "C" fn echo(ctx: *mut libc::c_void, argc: i32, argv: *mut *mut libc::c_void) {
            assert_eq!(argc, 1);
            sqlite3_result_value(ctx, *argv.add(0));
        }

        unsafe {
            let (_dir, db) = open_udf_db();
            for (name, argc, func) in [
                (
                    c"give_null",
                    0,
                    give_null
                        as unsafe extern "C" fn(*mut libc::c_void, i32, *mut *mut libc::c_void),
                ),
                (c"give_double", 0, give_double),
                (c"give_blob", 0, give_blob),
                (c"give_zeroblob", 0, give_zeroblob),
                (c"echo", 1, echo),
            ] {
                assert_eq!(
                    sqlite3_create_function_v2(
                        db,
                        name.as_ptr(),
                        argc,
                        SQLITE_UTF8,
                        ptr::null_mut(),
                        Some(func),
                        None,
                        None,
                        None,
                    ),
                    SQLITE_OK
                );
            }

            assert_eq!(query_texts(db, c"SELECT typeof(give_null())"), vec!["null"]);
            assert_eq!(query_texts(db, c"SELECT give_double()"), vec!["1.5"]);
            assert_eq!(query_texts(db, c"SELECT typeof(give_blob())"), vec!["blob"]);
            assert_eq!(query_ints(db, c"SELECT length(give_blob())"), vec![3]);
            assert_eq!(query_ints(db, c"SELECT length(give_zeroblob())"), vec![5]);
            assert_eq!(
                query_ints(db, c"SELECT give_zeroblob() = zeroblob(5)"),
                vec![1]
            );
            assert_eq!(query_texts(db, c"SELECT echo('hi')"), vec!["hi"]);
            assert_eq!(query_ints(db, c"SELECT echo(42)"), vec![42]);
            assert_eq!(
                query_texts(db, c"SELECT typeof(echo(x'aabb'))"),
                vec!["blob"]
            );
            assert_eq!(query_texts(db, c"SELECT typeof(echo(NULL))"), vec!["null"]);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    /// Text is NUL-terminated, and text that spells a whole number converts.
    #[test]
    fn test_argument_accessors_match_sqlite() {
        static REPORT: AtomicUsize = AtomicUsize::new(0);

        unsafe extern "C" fn inspect(
            ctx: *mut libc::c_void,
            argc: i32,
            argv: *mut *mut libc::c_void,
        ) {
            assert_eq!(argc, 1);
            let value = *argv.add(0);
            let text = sqlite3_value_text(value);
            let bytes = sqlite3_value_bytes(value);
            if !text.is_null() {
                // NUL-terminated, so strlen matches sqlite3_value_bytes.
                assert_eq!(*text.add(bytes as usize) as u8, 0);
                assert_eq!(CStr::from_ptr(text).to_bytes().len() as i32, bytes);
            }
            assert_eq!(sqlite3_value_encoding(value), SQLITE_UTF8);
            assert_eq!(sqlite3_value_nochange(value), 0);
            REPORT.store(sqlite3_value_numeric_type(value) as usize, Ordering::SeqCst);
            sqlite3_result_int(ctx, bytes);
        }

        unsafe {
            let (_dir, db) = open_udf_db();
            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    c"inspect".as_ptr(),
                    1,
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    Some(inspect),
                    None,
                    None,
                    None,
                ),
                SQLITE_OK
            );

            // Text spelling a whole number converts; "12abc" stays text.
            assert_eq!(query_ints(db, c"SELECT inspect('12')"), vec![2]);
            assert_eq!(REPORT.load(Ordering::SeqCst) as i32, SQLITE_INTEGER);
            assert_eq!(query_ints(db, c"SELECT inspect('1.5')"), vec![3]);
            assert_eq!(REPORT.load(Ordering::SeqCst) as i32, SQLITE_FLOAT);
            assert_eq!(query_ints(db, c"SELECT inspect('12abc')"), vec![5]);
            assert_eq!(REPORT.load(Ordering::SeqCst) as i32, SQLITE_TEXT);
            assert_eq!(query_ints(db, c"SELECT inspect('')"), vec![0]);
            assert_eq!(REPORT.load(Ordering::SeqCst) as i32, SQLITE_TEXT);
            // A number reports the length of its text rendering.
            assert_eq!(query_ints(db, c"SELECT inspect(4321)"), vec![4]);
            assert_eq!(REPORT.load(Ordering::SeqCst) as i32, SQLITE_INTEGER);
            // NULL has no text and no bytes.
            assert_eq!(query_ints(db, c"SELECT inspect(NULL)"), vec![0]);
            assert_eq!(REPORT.load(Ordering::SeqCst) as i32, SQLITE_NULL);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_json_subtype_travels_from_result_to_argument() {
        unsafe extern "C" fn make_json(
            ctx: *mut libc::c_void,
            _argc: i32,
            _argv: *mut *mut libc::c_void,
        ) {
            sqlite3_result_text(ctx, c"[1,2]".as_ptr(), -1, transient());
            sqlite3_result_subtype(ctx, JSON_SUBTYPE);
        }
        unsafe extern "C" fn read_subtype(
            ctx: *mut libc::c_void,
            _argc: i32,
            argv: *mut *mut libc::c_void,
        ) {
            sqlite3_result_int(ctx, sqlite3_value_subtype(*argv.add(0)) as i32);
        }

        unsafe {
            let (_dir, db) = open_udf_db();
            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    c"make_json".as_ptr(),
                    0,
                    SQLITE_UTF8 | SQLITE_RESULT_SUBTYPE,
                    ptr::null_mut(),
                    Some(make_json),
                    None,
                    None,
                    None,
                ),
                SQLITE_OK
            );
            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    c"read_subtype".as_ptr(),
                    1,
                    SQLITE_UTF8 | SQLITE_SUBTYPE,
                    ptr::null_mut(),
                    Some(read_subtype),
                    None,
                    None,
                    None,
                ),
                SQLITE_OK
            );
            assert_eq!(
                query_ints(db, c"SELECT read_subtype(make_json())"),
                vec![JSON_SUBTYPE as i64]
            );
            assert_eq!(query_ints(db, c"SELECT read_subtype('[1,2]')"), vec![0]);
            assert_eq!(query_texts(db, c"SELECT make_json()"), vec!["[1,2]"]);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_create_function16_takes_a_utf16_name() {
        unsafe extern "C" fn seven(
            ctx: *mut libc::c_void,
            _argc: i32,
            _argv: *mut *mut libc::c_void,
        ) {
            sqlite3_result_int(ctx, 7);
        }

        unsafe {
            let (_dir, db) = open_udf_db();
            let mut name: Vec<u16> = "wide_fn".encode_utf16().collect();
            name.push(0);
            assert_eq!(
                sqlite3_create_function16(
                    db,
                    name.as_ptr() as *const libc::c_void,
                    0,
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    Some(seven),
                    None,
                    None,
                ),
                SQLITE_OK
            );
            assert_eq!(query_ints(db, c"SELECT wide_fn()"), vec![7]);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    /// Calling the placeholder is an error, and an existing function is left alone.
    #[test]
    fn test_overload_function_makes_a_name_usable() {
        unsafe {
            let (_dir, db) = open_udf_db();
            assert_eq!(
                sqlite3_overload_function(db, c"nosuch".as_ptr(), 1),
                SQLITE_OK
            );

            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT nosuch(1)".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut()
                ),
                SQLITE_OK,
                "the overloaded name must parse"
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_ERROR);
            assert_eq!(
                errmsg(db),
                "unable to use function nosuch in the requested context"
            );
            // Not asserting finalize's code: SQLite repeats the statement's
            // error there where Turso returns SQLITE_OK.
            sqlite3_finalize(stmt);

            // Overloading a name that already works leaves it working.
            assert_eq!(sqlite3_overload_function(db, c"abs".as_ptr(), 1), SQLITE_OK);
            assert_eq!(query_ints(db, c"SELECT abs(-3)"), vec![3]);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    /// There is no ceiling on how many functions a connection may hold.
    #[test]
    fn test_many_functions_on_two_connections() {
        unsafe extern "C" fn report_user_data(
            ctx: *mut libc::c_void,
            _argc: i32,
            _argv: *mut *mut libc::c_void,
        ) {
            sqlite3_result_int64(ctx, sqlite3_user_data(ctx) as i64);
        }

        unsafe {
            let (_dir_a, db_a) = open_udf_db();
            let (_dir_b, db_b) = open_udf_db();
            const COUNT: i64 = 40;

            for (db, base) in [(db_a, 1000i64), (db_b, 2000i64)] {
                for i in 0..COUNT {
                    let name = CString::new(format!("f{i}")).unwrap();
                    assert_eq!(
                        sqlite3_create_function_v2(
                            db,
                            name.as_ptr(),
                            0,
                            SQLITE_UTF8,
                            (base + i) as *mut libc::c_void,
                            Some(report_user_data),
                            None,
                            None,
                            None,
                        ),
                        SQLITE_OK,
                        "registration {i} failed"
                    );
                }
            }

            for (db, base) in [(db_a, 1000i64), (db_b, 2000i64)] {
                for i in 0..COUNT {
                    let sql = CString::new(format!("SELECT f{i}()")).unwrap();
                    assert_eq!(query_ints(db, &sql), vec![base + i]);
                }
            }

            assert_eq!(sqlite3_close(db_a), SQLITE_OK);
            assert_eq!(sqlite3_close(db_b), SQLITE_OK);
        }
    }

    #[test]
    fn test_function_may_query_its_own_connection() {
        unsafe extern "C" fn count_rows(
            ctx: *mut libc::c_void,
            _argc: i32,
            _argv: *mut *mut libc::c_void,
        ) {
            let db = sqlite3_context_db_handle(ctx) as *mut sqlite3;
            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT count(*) FROM t".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut()
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            let count = sqlite3_column_int64(stmt, 0);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            sqlite3_result_int64(ctx, count);
        }

        unsafe {
            let (_dir, db) = open_udf_db();
            run_sql(db, c"CREATE TABLE t(x)");
            run_sql(db, c"INSERT INTO t VALUES (1), (2), (3)");
            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    c"count_rows".as_ptr(),
                    0,
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    Some(count_rows),
                    None,
                    None,
                    None,
                ),
                SQLITE_OK
            );
            assert_eq!(query_ints(db, c"SELECT count_rows()"), vec![3]);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_deterministic_function_is_called_once_for_a_constant() {
        static CALLS: AtomicUsize = AtomicUsize::new(0);
        unsafe extern "C" fn counted(
            ctx: *mut libc::c_void,
            _argc: i32,
            argv: *mut *mut libc::c_void,
        ) {
            CALLS.fetch_add(1, Ordering::SeqCst);
            sqlite3_result_value(ctx, *argv.add(0));
        }

        unsafe {
            CALLS.store(0, Ordering::SeqCst);
            let (_dir, db) = open_udf_db();
            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    c"counted".as_ptr(),
                    1,
                    SQLITE_UTF8 | SQLITE_DETERMINISTIC,
                    ptr::null_mut(),
                    Some(counted),
                    None,
                    None,
                    None,
                ),
                SQLITE_OK
            );
            run_sql(db, c"CREATE TABLE t(x)");
            run_sql(db, c"INSERT INTO t VALUES (1), (2), (3)");

            assert_eq!(query_ints(db, c"SELECT counted(7) FROM t"), vec![7, 7, 7]);
            assert_eq!(
                CALLS.load(Ordering::SeqCst),
                1,
                "a deterministic call on a constant is hoisted out of the loop"
            );

            CALLS.store(0, Ordering::SeqCst);
            assert_eq!(
                query_ints(db, c"SELECT counted(x) FROM t ORDER BY x"),
                vec![1, 2, 3]
            );
            assert_eq!(CALLS.load(Ordering::SeqCst), 3);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_redefining_a_function_while_a_statement_runs_is_busy() {
        unsafe extern "C" fn one(
            ctx: *mut libc::c_void,
            _argc: i32,
            _argv: *mut *mut libc::c_void,
        ) {
            sqlite3_result_int(ctx, 1);
        }
        unsafe extern "C" fn two(
            ctx: *mut libc::c_void,
            _argc: i32,
            _argv: *mut *mut libc::c_void,
        ) {
            sqlite3_result_int(ctx, 2);
        }
        unsafe fn register(db: *mut sqlite3, name: &CStr, argc: i32, body: XFunc) -> i32 {
            sqlite3_create_function_v2(
                db,
                name.as_ptr(),
                argc,
                SQLITE_UTF8,
                ptr::null_mut(),
                body,
                None,
                None,
                None,
            )
        }

        unsafe {
            let (_dir, db) = open_udf_db();
            run_sql(db, c"CREATE TABLE t(x)");
            run_sql(db, c"INSERT INTO t VALUES (1), (2)");
            assert_eq!(register(db, c"pick", 0, Some(one)), SQLITE_OK);

            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT pick() FROM t".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut()
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            assert_eq!(sqlite3_column_int64(stmt, 0), 1);

            assert_eq!(register(db, c"pick", 0, Some(two)), SQLITE_BUSY);
            assert_eq!(
                errmsg(db),
                "unable to delete/modify user-function due to active statements"
            );
            // Deleting it is a change too.
            assert_eq!(register(db, c"pick", 0, None), SQLITE_BUSY);
            // A different argument count is its own registration.
            assert_eq!(register(db, c"pick", 1, Some(two)), SQLITE_OK);
            assert_eq!(register(db, c"fresh", 0, Some(two)), SQLITE_OK);

            // The running statement keeps the definition it compiled against.
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            assert_eq!(sqlite3_column_int64(stmt, 0), 1);

            // Once it is reset nothing is running, so the change takes.
            assert_eq!(sqlite3_reset(stmt), SQLITE_OK);
            assert_eq!(register(db, c"pick", 0, Some(two)), SQLITE_OK);
            assert_eq!(query_ints(db, c"SELECT pick()"), vec![2]);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_utf16_argument_and_result() {
        static BYTES16: AtomicUsize = AtomicUsize::new(0);

        unsafe extern "C" fn reverse16(
            ctx: *mut libc::c_void,
            argc: i32,
            argv: *mut *mut libc::c_void,
        ) {
            assert_eq!(argc, 1);
            let value = *argv.add(0);
            let bytes16 = sqlite3_value_bytes16(value);
            BYTES16.store(bytes16 as usize, Ordering::SeqCst);
            let text16 = sqlite3_value_text16(value) as *const u16;
            assert!(!text16.is_null());
            let units = std::slice::from_raw_parts(text16, bytes16 as usize / 2);
            assert_eq!(*text16.add(bytes16 as usize / 2), 0);
            let text: String = String::from_utf16(units).unwrap().chars().rev().collect();
            let mut out: Vec<u16> = text.encode_utf16().collect();
            out.push(0);
            sqlite3_result_text16(ctx, out.as_ptr() as *const libc::c_void, -1, transient());
        }

        unsafe {
            let (_dir, db) = open_udf_db();
            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    c"reverse16".as_ptr(),
                    1,
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    Some(reverse16),
                    None,
                    None,
                    None,
                ),
                SQLITE_OK
            );
            assert_eq!(query_texts(db, c"SELECT reverse16('abc')"), vec!["cba"]);
            assert_eq!(BYTES16.load(Ordering::SeqCst), 6);
            // Characters outside the BMP take two code units each, so the
            // UTF-16 length is not the UTF-8 length.
            assert_eq!(
                query_texts(db, c"SELECT reverse16('h\u{e9}')"),
                vec!["\u{e9}h"]
            );
            assert_eq!(BYTES16.load(Ordering::SeqCst), 4);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    // Collating sequences

    unsafe extern "C" fn nocase_utf8(
        _p_arg: *mut libc::c_void,
        n_left: i32,
        left: *const libc::c_void,
        n_right: i32,
        right: *const libc::c_void,
    ) -> i32 {
        let left =
            std::slice::from_raw_parts(left as *const u8, n_left as usize).to_ascii_lowercase();
        let right =
            std::slice::from_raw_parts(right as *const u8, n_right as usize).to_ascii_lowercase();
        match left.cmp(&right) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        }
    }

    /// Reverse order, so a collation swap shows up in the results.
    unsafe extern "C" fn reverse_utf8(
        _p_arg: *mut libc::c_void,
        n_left: i32,
        left: *const libc::c_void,
        n_right: i32,
        right: *const libc::c_void,
    ) -> i32 {
        let left = std::slice::from_raw_parts(left as *const u8, n_left as usize);
        let right = std::slice::from_raw_parts(right as *const u8, n_right as usize);
        match right.cmp(left) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        }
    }

    unsafe fn make_names_table(db: *mut sqlite3) {
        run_sql(db, c"CREATE TABLE names(value TEXT)");
        run_sql(
            db,
            c"INSERT INTO names VALUES ('beta'), ('ALPHA'), ('alpha'), ('Gamma')",
        );
    }

    #[test]
    fn test_custom_collation_orders_and_compares() {
        unsafe {
            let (_dir, db) = open_udf_db();
            make_names_table(db);
            assert_eq!(
                sqlite3_create_collation(
                    db,
                    c"turso_nocase".as_ptr(),
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    Some(nocase_utf8),
                ),
                SQLITE_OK
            );

            assert_eq!(
                query_texts(
                    db,
                    c"SELECT value FROM names ORDER BY value COLLATE turso_nocase, value"
                ),
                vec!["ALPHA", "alpha", "beta", "Gamma"]
            );
            assert_eq!(
                query_texts(
                    db,
                    c"SELECT value FROM names WHERE value = 'ALPHA' COLLATE turso_nocase \
                      ORDER BY value"
                ),
                vec!["ALPHA", "alpha"]
            );
            assert_eq!(
                query_texts(db, c"SELECT max(value COLLATE turso_nocase) FROM names"),
                vec!["Gamma"]
            );
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_registering_a_collation_again_replaces_it() {
        unsafe {
            let (_dir, db) = open_udf_db();
            make_names_table(db);
            let order = c"SELECT value FROM names ORDER BY value COLLATE turso_swap, value";
            assert_eq!(
                sqlite3_create_collation(
                    db,
                    c"turso_swap".as_ptr(),
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    Some(nocase_utf8),
                ),
                SQLITE_OK
            );
            assert_eq!(
                query_texts(db, order),
                vec!["ALPHA", "alpha", "beta", "Gamma"]
            );
            assert_eq!(
                sqlite3_create_collation(
                    db,
                    c"turso_swap".as_ptr(),
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    Some(reverse_utf8),
                ),
                SQLITE_OK
            );
            assert_eq!(
                query_texts(db, order),
                vec!["beta", "alpha", "Gamma", "ALPHA"]
            );
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    /// Once each, never twice.
    #[test]
    fn test_collation_destructor_runs_on_replace_delete_and_close() {
        static DESTROYS: AtomicUsize = AtomicUsize::new(0);
        unsafe extern "C" fn count_destroy(_p_arg: *mut libc::c_void) {
            DESTROYS.fetch_add(1, Ordering::SeqCst);
        }

        unsafe {
            let (_dir, db) = open_udf_db();
            let name = c"turso_counted";
            assert_eq!(
                sqlite3_create_collation_v2(
                    db,
                    name.as_ptr(),
                    SQLITE_UTF8,
                    marker_p_app(),
                    Some(nocase_utf8),
                    Some(count_destroy),
                ),
                SQLITE_OK
            );
            assert_eq!(DESTROYS.load(Ordering::SeqCst), 0);

            assert_eq!(
                sqlite3_create_collation_v2(
                    db,
                    name.as_ptr(),
                    SQLITE_UTF8,
                    marker_p_app(),
                    Some(reverse_utf8),
                    Some(count_destroy),
                ),
                SQLITE_OK
            );
            assert_eq!(DESTROYS.load(Ordering::SeqCst), 1);

            // A NULL comparison deletes the live registration; this call's own
            // destructor waits for the close.
            assert_eq!(
                sqlite3_create_collation_v2(
                    db,
                    name.as_ptr(),
                    SQLITE_UTF8,
                    marker_p_app(),
                    None,
                    Some(count_destroy),
                ),
                SQLITE_OK
            );
            assert_eq!(DESTROYS.load(Ordering::SeqCst), 2);

            assert_eq!(sqlite3_close(db), SQLITE_OK);
            assert_eq!(DESTROYS.load(Ordering::SeqCst), 3);
        }
    }

    #[test]
    fn test_collation_destructor_runs_on_close() {
        static DESTROYS: AtomicUsize = AtomicUsize::new(0);
        unsafe extern "C" fn count_destroy(_p_arg: *mut libc::c_void) {
            DESTROYS.fetch_add(1, Ordering::SeqCst);
        }

        unsafe {
            let (_dir, db) = open_udf_db();
            assert_eq!(
                sqlite3_create_collation_v2(
                    db,
                    c"turso_closed".as_ptr(),
                    SQLITE_UTF8,
                    marker_p_app(),
                    Some(nocase_utf8),
                    Some(count_destroy),
                ),
                SQLITE_OK
            );
            assert_eq!(DESTROYS.load(Ordering::SeqCst), 0);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
            assert_eq!(DESTROYS.load(Ordering::SeqCst), 1);
        }
    }

    #[test]
    fn test_deleted_collation_is_gone() {
        unsafe {
            let (_dir, db) = open_udf_db();
            let sql = c"SELECT 'a' = 'A' COLLATE turso_deleted";
            assert_eq!(
                sqlite3_create_collation(
                    db,
                    c"turso_deleted".as_ptr(),
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    Some(nocase_utf8),
                ),
                SQLITE_OK
            );
            assert_eq!(query_ints(db, sql), vec![1]);

            assert_eq!(
                sqlite3_create_collation(
                    db,
                    c"turso_deleted".as_ptr(),
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    None,
                ),
                SQLITE_OK
            );
            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(db, sql.as_ptr(), -1, &mut stmt, ptr::null_mut()),
                SQLITE_ERROR
            );
            assert_eq!(errmsg(db), "no such collation sequence: turso_deleted");
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_create_collation16_takes_a_utf16_name() {
        /// Also checks that what it was handed really is UTF-16.
        unsafe extern "C" fn reverse_utf16(
            _p_arg: *mut libc::c_void,
            n_left: i32,
            left: *const libc::c_void,
            n_right: i32,
            right: *const libc::c_void,
        ) -> i32 {
            assert_eq!(n_left % 2, 0);
            assert_eq!(n_right % 2, 0);
            let left = std::slice::from_raw_parts(left as *const u16, n_left as usize / 2).to_vec();
            let right =
                std::slice::from_raw_parts(right as *const u16, n_right as usize / 2).to_vec();
            match right.cmp(&left) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            }
        }

        unsafe {
            let (_dir, db) = open_udf_db();
            make_names_table(db);
            let mut name: Vec<u16> = "turso_wide".encode_utf16().collect();
            name.push(0);
            assert_eq!(
                sqlite3_create_collation16(
                    db,
                    name.as_ptr() as *const libc::c_void,
                    SQLITE_UTF16,
                    ptr::null_mut(),
                    Some(reverse_utf16),
                ),
                SQLITE_OK
            );
            assert_eq!(
                query_texts(
                    db,
                    c"SELECT value FROM names ORDER BY value COLLATE turso_wide, value"
                ),
                vec!["beta", "alpha", "Gamma", "ALPHA"]
            );
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_collation_registered_as_utf16be_gets_big_endian_bytes() {
        unsafe extern "C" fn reverse_utf16be(
            _p_arg: *mut libc::c_void,
            n_left: i32,
            left: *const libc::c_void,
            n_right: i32,
            right: *const libc::c_void,
        ) -> i32 {
            unsafe fn units_be(bytes: *const libc::c_void, n: i32) -> Vec<u16> {
                std::slice::from_raw_parts(bytes as *const u8, n as usize)
                    .chunks_exact(2)
                    .map(|pair| u16::from_be_bytes([pair[0], pair[1]]))
                    .collect()
            }
            let left = units_be(left, n_left);
            let right = units_be(right, n_right);
            // The fixture is all ASCII, so both byte orders would sort the
            // same; check the bytes themselves instead.
            assert!(left.iter().all(|unit| *unit < 128));
            match right.cmp(&left) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            }
        }

        unsafe {
            let (_dir, db) = open_udf_db();
            make_names_table(db);
            assert_eq!(
                sqlite3_create_collation(
                    db,
                    c"turso_be".as_ptr(),
                    SQLITE_UTF16BE,
                    ptr::null_mut(),
                    Some(reverse_utf16be),
                ),
                SQLITE_OK
            );
            assert_eq!(
                query_texts(
                    db,
                    c"SELECT value FROM names ORDER BY value COLLATE turso_be, value"
                ),
                vec!["beta", "alpha", "Gamma", "ALPHA"]
            );
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    /// Every UTF-16 flavour registers; anything else is misuse.
    #[test]
    fn test_create_collation_checks_the_encoding() {
        unsafe {
            let (_dir, db) = open_udf_db();
            for enc in [
                SQLITE_UTF8,
                SQLITE_UTF16LE,
                SQLITE_UTF16BE,
                SQLITE_UTF16,
                SQLITE_UTF16_ALIGNED,
            ] {
                let name = CString::new(format!("turso_enc{enc}")).unwrap();
                assert_eq!(
                    sqlite3_create_collation(
                        db,
                        name.as_ptr(),
                        enc,
                        ptr::null_mut(),
                        Some(nocase_utf8),
                    ),
                    SQLITE_OK,
                    "encoding {enc} should register"
                );
            }
            for enc in [0, SQLITE_ANY, 6, 7, 9, 100] {
                let name = CString::new(format!("turso_bad{enc}")).unwrap();
                assert_eq!(
                    sqlite3_create_collation(
                        db,
                        name.as_ptr(),
                        enc,
                        ptr::null_mut(),
                        Some(nocase_utf8),
                    ),
                    SQLITE_MISUSE,
                    "encoding {enc} should be misuse"
                );
            }
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_collation_needed_supplies_a_missing_collation() {
        static CALLS: AtomicUsize = AtomicUsize::new(0);
        unsafe extern "C" fn supply(
            p_arg: *mut libc::c_void,
            db: *mut sqlite3,
            enc: i32,
            name: *const libc::c_char,
        ) {
            CALLS.fetch_add(1, Ordering::SeqCst);
            assert_eq!(p_arg, marker_p_app());
            assert_eq!(enc, SQLITE_UTF8);
            let name = CStr::from_ptr(name);
            if name.to_bytes().eq_ignore_ascii_case(b"turso_late") {
                assert_eq!(
                    sqlite3_create_collation(
                        db,
                        name.as_ptr(),
                        SQLITE_UTF8,
                        ptr::null_mut(),
                        Some(nocase_utf8),
                    ),
                    SQLITE_OK
                );
            }
        }

        unsafe {
            let (_dir, db) = open_udf_db();
            assert_eq!(
                sqlite3_collation_needed(db, marker_p_app(), Some(supply)),
                SQLITE_OK
            );
            assert_eq!(
                query_ints(db, c"SELECT 'a' = 'A' COLLATE turso_late"),
                vec![1]
            );
            assert!(CALLS.load(Ordering::SeqCst) >= 1);

            // Registered now, so a second statement does not ask again.
            let asked = CALLS.load(Ordering::SeqCst);
            assert_eq!(
                query_ints(db, c"SELECT 'b' = 'B' COLLATE turso_late"),
                vec![1]
            );
            assert_eq!(CALLS.load(Ordering::SeqCst), asked);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    /// The callback is asked once for that statement, not forever.
    #[test]
    fn test_collation_needed_that_registers_nothing_still_fails() {
        static CALLS: AtomicUsize = AtomicUsize::new(0);
        unsafe extern "C" fn ignore(
            _p_arg: *mut libc::c_void,
            _db: *mut sqlite3,
            _enc: i32,
            _name: *const libc::c_char,
        ) {
            CALLS.fetch_add(1, Ordering::SeqCst);
        }

        unsafe {
            let (_dir, db) = open_udf_db();
            assert_eq!(
                sqlite3_collation_needed(db, ptr::null_mut(), Some(ignore)),
                SQLITE_OK
            );
            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT 'a' = 'A' COLLATE turso_never".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut()
                ),
                SQLITE_ERROR
            );
            assert_eq!(errmsg(db), "no such collation sequence: turso_never");
            assert!(CALLS.load(Ordering::SeqCst) >= 1);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    /// The UTF-8 one is used when there is one, else a UTF-16 one, and the name
    /// is gone once every registration has been deleted.
    #[test]
    fn test_a_collation_keeps_one_registration_per_encoding() {
        unsafe {
            let (_dir, db) = open_udf_db();
            make_names_table(db);
            let order = c"SELECT value FROM names ORDER BY value COLLATE turso_enc, value";
            let name = c"turso_enc";

            assert_eq!(
                sqlite3_create_collation(
                    db,
                    name.as_ptr(),
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    Some(nocase_utf8),
                ),
                SQLITE_OK
            );
            assert_eq!(
                query_texts(db, order),
                vec!["ALPHA", "alpha", "beta", "Gamma"]
            );

            // Another encoding does not displace the UTF-8 registration.
            assert_eq!(
                sqlite3_create_collation(
                    db,
                    name.as_ptr(),
                    SQLITE_UTF16LE,
                    ptr::null_mut(),
                    Some(reverse_utf8),
                ),
                SQLITE_OK
            );
            assert_eq!(
                query_texts(db, order),
                vec!["ALPHA", "alpha", "beta", "Gamma"]
            );

            // Deleting the UTF-8 one leaves the UTF-16 one to fall back on.
            assert_eq!(
                sqlite3_create_collation(db, name.as_ptr(), SQLITE_UTF8, ptr::null_mut(), None),
                SQLITE_OK
            );
            assert_eq!(
                query_texts(db, order),
                vec!["beta", "alpha", "Gamma", "ALPHA"]
            );

            // Deleting the last one leaves nothing to compare with.
            assert_eq!(
                sqlite3_create_collation(db, name.as_ptr(), SQLITE_UTF16LE, ptr::null_mut(), None),
                SQLITE_OK
            );
            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(db, order.as_ptr(), -1, &mut stmt, ptr::null_mut()),
                SQLITE_ERROR
            );
            assert_eq!(errmsg(db), "no such collation sequence: turso_enc");

            // Deleting an encoding that was never registered is a no-op.
            assert_eq!(
                sqlite3_create_collation(db, name.as_ptr(), SQLITE_UTF16BE, ptr::null_mut(), None),
                SQLITE_OK
            );
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    /// SQLite's `synthCollSeq` tries them in that order.
    #[test]
    fn test_utf16be_registration_is_preferred_over_utf16le() {
        unsafe {
            let (_dir, db) = open_udf_db();
            make_names_table(db);
            let order = c"SELECT value FROM names ORDER BY value COLLATE turso_wide16, value";
            let name = c"turso_wide16";
            assert_eq!(
                sqlite3_create_collation(
                    db,
                    name.as_ptr(),
                    SQLITE_UTF16LE,
                    ptr::null_mut(),
                    Some(nocase_utf8),
                ),
                SQLITE_OK
            );
            assert_eq!(
                sqlite3_create_collation(
                    db,
                    name.as_ptr(),
                    SQLITE_UTF16BE,
                    ptr::null_mut(),
                    Some(reverse_utf8),
                ),
                SQLITE_OK
            );
            assert_eq!(
                query_texts(db, order),
                vec!["beta", "alpha", "Gamma", "ALPHA"]
            );
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_replacing_one_encoding_destroys_only_that_registration() {
        static DESTROYS: AtomicUsize = AtomicUsize::new(0);
        unsafe extern "C" fn count_destroy(_p_arg: *mut libc::c_void) {
            DESTROYS.fetch_add(1, Ordering::SeqCst);
        }

        unsafe {
            DESTROYS.store(0, Ordering::SeqCst);
            let (_dir, db) = open_udf_db();
            let name = c"turso_percoding";
            let register = |enc: i32| {
                sqlite3_create_collation_v2(
                    db,
                    name.as_ptr(),
                    enc,
                    marker_p_app(),
                    Some(nocase_utf8),
                    Some(count_destroy),
                )
            };
            assert_eq!(register(SQLITE_UTF8), SQLITE_OK);
            assert_eq!(register(SQLITE_UTF16LE), SQLITE_OK);
            assert_eq!(
                DESTROYS.load(Ordering::SeqCst),
                0,
                "another encoding does not displace the first registration"
            );
            assert_eq!(register(SQLITE_UTF8), SQLITE_OK);
            assert_eq!(DESTROYS.load(Ordering::SeqCst), 1);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
            assert_eq!(
                DESTROYS.load(Ordering::SeqCst),
                3,
                "closing destroys both registrations that were left"
            );
        }
    }

    #[test]
    fn test_redefining_a_collation_while_a_statement_runs_is_busy() {
        unsafe {
            let (_dir, db) = open_udf_db();
            make_names_table(db);
            assert_eq!(
                sqlite3_create_collation(
                    db,
                    c"turso_busy".as_ptr(),
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    Some(nocase_utf8),
                ),
                SQLITE_OK
            );

            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT value FROM names ORDER BY value COLLATE turso_busy, value".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut()
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);

            assert_eq!(
                sqlite3_create_collation(
                    db,
                    c"turso_busy".as_ptr(),
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    Some(reverse_utf8),
                ),
                SQLITE_BUSY
            );
            assert_eq!(
                errmsg(db),
                "unable to delete/modify collation sequence due to active statements"
            );
            // Deleting it is a change too.
            assert_eq!(
                sqlite3_create_collation(
                    db,
                    c"turso_busy".as_ptr(),
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    None,
                ),
                SQLITE_BUSY
            );
            // Another encoding of the same name is its own registration.
            assert_eq!(
                sqlite3_create_collation(
                    db,
                    c"turso_busy".as_ptr(),
                    SQLITE_UTF16LE,
                    ptr::null_mut(),
                    Some(reverse_utf8),
                ),
                SQLITE_OK
            );
            assert_eq!(
                sqlite3_create_collation(
                    db,
                    c"turso_fresh".as_ptr(),
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    Some(nocase_utf8),
                ),
                SQLITE_OK
            );

            // The running statement keeps the comparison it started with.
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);

            // Once it is reset nothing is running, so the change takes.
            assert_eq!(sqlite3_reset(stmt), SQLITE_OK);
            assert_eq!(
                sqlite3_create_collation(
                    db,
                    c"turso_busy".as_ptr(),
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    Some(reverse_utf8),
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    // Pointer values

    unsafe extern "C" fn make_pointer(
        ctx: *mut libc::c_void,
        _argc: i32,
        _argv: *mut *mut libc::c_void,
    ) {
        sqlite3_result_pointer(ctx, marker_p_app(), c"mytype".as_ptr(), None);
    }

    unsafe extern "C" fn read_pointer(
        ctx: *mut libc::c_void,
        _argc: i32,
        argv: *mut *mut libc::c_void,
    ) {
        let ptr = sqlite3_value_pointer(*argv.add(0), c"mytype".as_ptr());
        sqlite3_result_int(ctx, i32::from(ptr == marker_p_app()));
    }

    unsafe extern "C" fn read_pointer_other_type(
        ctx: *mut libc::c_void,
        _argc: i32,
        argv: *mut *mut libc::c_void,
    ) {
        let ptr = sqlite3_value_pointer(*argv.add(0), c"othertype".as_ptr());
        sqlite3_result_int(ctx, i32::from(!ptr.is_null()));
    }

    unsafe fn register_pointer_functions(db: *mut sqlite3) {
        assert_eq!(
            sqlite3_create_function_v2(
                db,
                c"make_pointer".as_ptr(),
                0,
                SQLITE_UTF8 | SQLITE_RESULT_SUBTYPE,
                ptr::null_mut(),
                Some(make_pointer),
                None,
                None,
                None,
            ),
            SQLITE_OK
        );
        assert_eq!(
            sqlite3_create_function_v2(
                db,
                c"read_pointer".as_ptr(),
                1,
                SQLITE_UTF8 | SQLITE_SUBTYPE,
                ptr::null_mut(),
                Some(read_pointer),
                None,
                None,
                None,
            ),
            SQLITE_OK
        );
        assert_eq!(
            sqlite3_create_function_v2(
                db,
                c"read_other_type".as_ptr(),
                1,
                SQLITE_UTF8 | SQLITE_SUBTYPE,
                ptr::null_mut(),
                Some(read_pointer_other_type),
                None,
                None,
                None,
            ),
            SQLITE_OK
        );
    }

    /// And only under its own type name.
    #[test]
    fn test_pointer_travels_from_result_to_argument() {
        unsafe {
            let (_dir, db) = open_udf_db();
            register_pointer_functions(db);
            assert_eq!(
                query_ints(db, c"SELECT read_pointer(make_pointer())"),
                vec![1]
            );
            assert_eq!(
                query_ints(db, c"SELECT read_other_type(make_pointer())"),
                vec![0],
                "a pointer is only readable under the type name it was made with"
            );
            assert_eq!(
                query_ints(db, c"SELECT read_pointer('1')"),
                vec![0],
                "a plain value is never a pointer"
            );
            assert_eq!(query_ints(db, c"SELECT read_pointer(NULL)"), vec![0]);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_bind_pointer_reaches_a_function() {
        unsafe {
            let (_dir, db) = open_udf_db();
            register_pointer_functions(db);
            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT read_pointer(?), read_other_type(?)".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut()
                ),
                SQLITE_OK
            );
            assert_eq!(
                sqlite3_bind_pointer(stmt, 1, marker_p_app(), c"mytype".as_ptr(), None),
                SQLITE_OK
            );
            assert_eq!(
                sqlite3_bind_pointer(stmt, 2, marker_p_app(), c"mytype".as_ptr(), None),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            assert_eq!(sqlite3_column_int64(stmt, 0), 1);
            assert_eq!(sqlite3_column_int64(stmt, 1), 0);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);

            // A failed bind hands the pointer back through its destructor,
            // the only way the caller could free it.
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_bound_pointer_destructor_runs_on_rebind_and_finalize() {
        static DESTROYS: AtomicUsize = AtomicUsize::new(0);
        unsafe extern "C" fn count_destroy(_ptr: *mut libc::c_void) {
            DESTROYS.fetch_add(1, Ordering::SeqCst);
        }

        unsafe {
            let (_dir, db) = open_udf_db();
            register_pointer_functions(db);
            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT read_pointer(?)".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut()
                ),
                SQLITE_OK
            );
            assert_eq!(
                sqlite3_bind_pointer(
                    stmt,
                    1,
                    marker_p_app(),
                    c"mytype".as_ptr(),
                    Some(count_destroy)
                ),
                SQLITE_OK
            );
            assert_eq!(DESTROYS.load(Ordering::SeqCst), 0);

            // Binding something else over it releases the pointer.
            assert_eq!(sqlite3_bind_int(stmt, 1, 7), SQLITE_OK);
            assert_eq!(DESTROYS.load(Ordering::SeqCst), 1);
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            assert_eq!(sqlite3_column_int64(stmt, 0), 0);
            assert_eq!(sqlite3_reset(stmt), SQLITE_OK);

            // A pointer that is still bound is released by the finalize.
            assert_eq!(
                sqlite3_bind_pointer(
                    stmt,
                    1,
                    marker_p_app(),
                    c"mytype".as_ptr(),
                    Some(count_destroy)
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            assert_eq!(sqlite3_column_int64(stmt, 0), 1);
            assert_eq!(DESTROYS.load(Ordering::SeqCst), 1);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(DESTROYS.load(Ordering::SeqCst), 2);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_clear_bindings_and_a_failed_bind_release_the_pointer() {
        static DESTROYS: AtomicUsize = AtomicUsize::new(0);
        unsafe extern "C" fn count_destroy(_ptr: *mut libc::c_void) {
            DESTROYS.fetch_add(1, Ordering::SeqCst);
        }

        unsafe {
            let (_dir, db) = open_udf_db();
            register_pointer_functions(db);
            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT read_pointer(?)".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut()
                ),
                SQLITE_OK
            );
            assert_eq!(
                sqlite3_bind_pointer(
                    stmt,
                    1,
                    marker_p_app(),
                    c"mytype".as_ptr(),
                    Some(count_destroy)
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_clear_bindings(stmt), SQLITE_OK);
            assert_eq!(DESTROYS.load(Ordering::SeqCst), 1);

            assert_eq!(
                sqlite3_bind_pointer(
                    stmt,
                    9,
                    marker_p_app(),
                    c"mytype".as_ptr(),
                    Some(count_destroy)
                ),
                SQLITE_RANGE
            );
            assert_eq!(
                DESTROYS.load(Ordering::SeqCst),
                2,
                "a bind that fails must not swallow the pointer"
            );
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(DESTROYS.load(Ordering::SeqCst), 2);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    /// And only once.
    #[test]
    fn test_result_pointer_destructor_runs_by_finalize() {
        static DESTROYS: AtomicUsize = AtomicUsize::new(0);
        unsafe extern "C" fn count_destroy(_ptr: *mut libc::c_void) {
            DESTROYS.fetch_add(1, Ordering::SeqCst);
        }
        unsafe extern "C" fn make_owned_pointer(
            ctx: *mut libc::c_void,
            _argc: i32,
            _argv: *mut *mut libc::c_void,
        ) {
            sqlite3_result_pointer(ctx, marker_p_app(), c"mytype".as_ptr(), Some(count_destroy));
        }

        unsafe {
            let (_dir, db) = open_udf_db();
            register_pointer_functions(db);
            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    c"make_owned_pointer".as_ptr(),
                    0,
                    SQLITE_UTF8 | SQLITE_RESULT_SUBTYPE,
                    ptr::null_mut(),
                    Some(make_owned_pointer),
                    None,
                    None,
                    None,
                ),
                SQLITE_OK
            );

            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT read_pointer(make_owned_pointer())".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut()
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            assert_eq!(sqlite3_column_int64(stmt, 0), 1);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(DESTROYS.load(Ordering::SeqCst), 1);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
            assert_eq!(DESTROYS.load(Ordering::SeqCst), 1);
        }
    }

    /// Storing it stores a plain NULL that carries no pointer.
    #[test]
    fn test_a_pointer_value_is_a_null_in_sql() {
        unsafe {
            let (_dir, db) = open_udf_db();
            register_pointer_functions(db);
            assert_eq!(
                query_texts(db, c"SELECT typeof(make_pointer())"),
                vec!["null"]
            );
            run_sql(db, c"CREATE TABLE kept(value)");
            run_sql(db, c"INSERT INTO kept VALUES (make_pointer())");
            assert_eq!(
                query_texts(db, c"SELECT typeof(value) FROM kept"),
                vec!["null"]
            );
            assert_eq!(
                query_ints(db, c"SELECT read_pointer(value) FROM kept"),
                vec![0],
                "a pointer does not survive being stored"
            );
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_a_pointer_value_has_the_p_subtype() {
        unsafe {
            let (_dir, db) = open_udf_db();
            register_pointer_functions(db);
            register_subtype_reader(db);
            assert_eq!(
                query_ints(db, c"SELECT read_subtype(make_pointer())"),
                vec![i64::from(b'p')]
            );
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_result_value_carries_a_pointer_across() {
        unsafe extern "C" fn copy_value(
            ctx: *mut libc::c_void,
            _argc: i32,
            argv: *mut *mut libc::c_void,
        ) {
            sqlite3_result_value(ctx, *argv.add(0));
        }

        unsafe {
            let (_dir, db) = open_udf_db();
            register_pointer_functions(db);
            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    c"copy_value".as_ptr(),
                    1,
                    SQLITE_UTF8 | SQLITE_SUBTYPE | SQLITE_RESULT_SUBTYPE,
                    ptr::null_mut(),
                    Some(copy_value),
                    None,
                    None,
                    None,
                ),
                SQLITE_OK
            );
            assert_eq!(
                query_ints(db, c"SELECT read_pointer(copy_value(make_pointer()))"),
                vec![1]
            );
            assert_eq!(
                query_ints(db, c"SELECT read_other_type(copy_value(make_pointer()))"),
                vec![0],
                "the copy keeps the type name it was made with"
            );
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    /// The value is copied out of the subquery's own registers there. A scalar
    /// subquery is not such a boundary and keeps the pointer.
    #[test]
    fn test_a_pointer_does_not_cross_a_subquery_boundary() {
        unsafe {
            let (_dir, db) = open_udf_db();
            register_pointer_functions(db);
            assert_eq!(
                query_ints(
                    db,
                    c"SELECT read_pointer(p) FROM (SELECT make_pointer() AS p)"
                ),
                vec![0]
            );
            assert_eq!(
                query_ints(
                    db,
                    c"WITH q(p) AS (SELECT make_pointer()) SELECT read_pointer(p) FROM q"
                ),
                vec![0]
            );
            assert_eq!(
                query_ints(db, c"SELECT read_pointer((SELECT make_pointer()))"),
                vec![1]
            );
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    // Result subtypes

    unsafe extern "C" fn read_subtype(
        ctx: *mut libc::c_void,
        _argc: i32,
        argv: *mut *mut libc::c_void,
    ) {
        sqlite3_result_int(ctx, sqlite3_value_subtype(*argv.add(0)) as i32);
    }

    unsafe fn register_subtype_reader(db: *mut sqlite3) {
        assert_eq!(
            sqlite3_create_function_v2(
                db,
                c"read_subtype".as_ptr(),
                1,
                SQLITE_UTF8 | SQLITE_SUBTYPE,
                ptr::null_mut(),
                Some(read_subtype),
                None,
                None,
                None,
            ),
            SQLITE_OK
        );
    }

    const TAG: u32 = 42;

    unsafe extern "C" fn tag_int(
        ctx: *mut libc::c_void,
        _argc: i32,
        _argv: *mut *mut libc::c_void,
    ) {
        sqlite3_result_int(ctx, 7);
        sqlite3_result_subtype(ctx, TAG);
    }
    unsafe extern "C" fn tag_real(
        ctx: *mut libc::c_void,
        _argc: i32,
        _argv: *mut *mut libc::c_void,
    ) {
        sqlite3_result_double(ctx, 1.5);
        sqlite3_result_subtype(ctx, TAG);
    }
    unsafe extern "C" fn tag_blob(
        ctx: *mut libc::c_void,
        _argc: i32,
        _argv: *mut *mut libc::c_void,
    ) {
        sqlite3_result_blob(ctx, c"ab".as_ptr() as *const libc::c_void, 2, transient());
        sqlite3_result_subtype(ctx, TAG);
    }
    unsafe extern "C" fn tag_null(
        ctx: *mut libc::c_void,
        _argc: i32,
        _argv: *mut *mut libc::c_void,
    ) {
        sqlite3_result_null(ctx);
        sqlite3_result_subtype(ctx, TAG);
    }
    unsafe extern "C" fn tag_text(
        ctx: *mut libc::c_void,
        _argc: i32,
        _argv: *mut *mut libc::c_void,
    ) {
        sqlite3_result_text(ctx, c"hi".as_ptr(), -1, transient());
        sqlite3_result_subtype(ctx, TAG);
    }
    /// Sets the subtype before the value, which SQLite drops.
    unsafe extern "C" fn tag_then_value(
        ctx: *mut libc::c_void,
        _argc: i32,
        _argv: *mut *mut libc::c_void,
    ) {
        sqlite3_result_subtype(ctx, TAG);
        sqlite3_result_int(ctx, 7);
    }

    #[test]
    fn test_a_subtype_travels_on_a_result_of_every_type() {
        unsafe {
            let (_dir, db) = open_udf_db();
            register_subtype_reader(db);
            for (name, body) in [
                (c"tag_int", tag_int as unsafe extern "C" fn(_, _, _)),
                (c"tag_real", tag_real),
                (c"tag_blob", tag_blob),
                (c"tag_null", tag_null),
                (c"tag_text", tag_text),
            ] {
                assert_eq!(
                    sqlite3_create_function_v2(
                        db,
                        name.as_ptr(),
                        0,
                        SQLITE_UTF8 | SQLITE_RESULT_SUBTYPE,
                        ptr::null_mut(),
                        Some(body),
                        None,
                        None,
                        None,
                    ),
                    SQLITE_OK
                );
                let sql =
                    CString::new(format!("SELECT read_subtype({}())", name.to_str().unwrap()))
                        .unwrap();
                assert_eq!(
                    query_ints(db, &sql),
                    vec![i64::from(TAG)],
                    "{name:?} should carry its subtype"
                );
            }
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_a_result_set_after_the_subtype_clears_it() {
        unsafe {
            let (_dir, db) = open_udf_db();
            register_subtype_reader(db);
            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    c"tag_then_value".as_ptr(),
                    0,
                    SQLITE_UTF8 | SQLITE_RESULT_SUBTYPE,
                    ptr::null_mut(),
                    Some(tag_then_value),
                    None,
                    None,
                    None,
                ),
                SQLITE_OK
            );
            assert_eq!(
                query_ints(db, c"SELECT read_subtype(tag_then_value())"),
                vec![0]
            );
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    /// A literal, a column read back out of a table and a bound parameter read 0.
    #[test]
    fn test_a_value_no_function_stamped_has_no_subtype() {
        unsafe {
            let (_dir, db) = open_udf_db();
            register_subtype_reader(db);
            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    c"tag_int".as_ptr(),
                    0,
                    SQLITE_UTF8 | SQLITE_RESULT_SUBTYPE,
                    ptr::null_mut(),
                    Some(tag_int),
                    None,
                    None,
                    None,
                ),
                SQLITE_OK
            );
            assert_eq!(query_ints(db, c"SELECT read_subtype(5)"), vec![0]);
            assert_eq!(query_ints(db, c"SELECT read_subtype('x')"), vec![0]);

            // A subtype does not survive being written into a row.
            run_sql(db, c"CREATE TABLE tagged(value)");
            run_sql(db, c"INSERT INTO tagged VALUES (tag_int())");
            assert_eq!(
                query_ints(db, c"SELECT read_subtype(value) FROM tagged"),
                vec![0]
            );

            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            assert_eq!(
                sqlite3_prepare_v2(
                    db,
                    c"SELECT read_subtype(?)".as_ptr(),
                    -1,
                    &mut stmt,
                    ptr::null_mut()
                ),
                SQLITE_OK
            );
            assert_eq!(sqlite3_bind_int(stmt, 1, 5), SQLITE_OK);
            assert_eq!(sqlite3_step(stmt), SQLITE_ROW);
            assert_eq!(sqlite3_column_int64(stmt, 0), 0);
            assert_eq!(sqlite3_finalize(stmt), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_the_builtin_json_result_carries_the_json_subtype() {
        unsafe {
            let (_dir, db) = open_udf_db();
            register_subtype_reader(db);
            assert_eq!(
                query_ints(db, c"SELECT read_subtype(json('[1,2]'))"),
                vec![i64::from(JSON_SUBTYPE)]
            );
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_value_dup_keeps_the_subtype() {
        unsafe extern "C" fn read_dup_subtype(
            ctx: *mut libc::c_void,
            _argc: i32,
            argv: *mut *mut libc::c_void,
        ) {
            let copy = sqlite3_value_dup(*argv.add(0));
            sqlite3_result_int(ctx, sqlite3_value_subtype(copy) as i32);
            sqlite3_value_free(copy);
        }

        unsafe {
            let (_dir, db) = open_udf_db();
            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    c"tag_int".as_ptr(),
                    0,
                    SQLITE_UTF8 | SQLITE_RESULT_SUBTYPE,
                    ptr::null_mut(),
                    Some(tag_int),
                    None,
                    None,
                    None,
                ),
                SQLITE_OK
            );
            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    c"read_dup_subtype".as_ptr(),
                    1,
                    SQLITE_UTF8 | SQLITE_SUBTYPE,
                    ptr::null_mut(),
                    Some(read_dup_subtype),
                    None,
                    None,
                    None,
                ),
                SQLITE_OK
            );
            assert_eq!(
                query_ints(db, c"SELECT read_dup_subtype(tag_int())"),
                vec![i64::from(TAG)]
            );
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_an_aggregate_step_sees_its_arguments_subtypes() {
        unsafe extern "C" fn sum_subtypes_step(
            ctx: *mut libc::c_void,
            _argc: i32,
            argv: *mut *mut libc::c_void,
        ) {
            let total = sqlite3_aggregate_context(ctx, 4) as *mut i32;
            if !total.is_null() {
                *total += sqlite3_value_subtype(*argv.add(0)) as i32;
            }
        }
        unsafe extern "C" fn sum_subtypes_final(ctx: *mut libc::c_void) {
            let total = sqlite3_aggregate_context(ctx, 4) as *mut i32;
            sqlite3_result_int(ctx, if total.is_null() { 0 } else { *total });
        }

        unsafe {
            let (_dir, db) = open_udf_db();
            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    c"tag_int".as_ptr(),
                    0,
                    SQLITE_UTF8 | SQLITE_RESULT_SUBTYPE,
                    ptr::null_mut(),
                    Some(tag_int),
                    None,
                    None,
                    None,
                ),
                SQLITE_OK
            );
            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    c"sum_subtypes".as_ptr(),
                    1,
                    SQLITE_UTF8 | SQLITE_SUBTYPE,
                    ptr::null_mut(),
                    None,
                    Some(sum_subtypes_step),
                    Some(sum_subtypes_final),
                    None,
                ),
                SQLITE_OK
            );
            run_sql(db, c"CREATE TABLE t(x)");
            run_sql(db, c"INSERT INTO t VALUES (1), (2), (3)");
            assert_eq!(
                query_ints(db, c"SELECT sum_subtypes(tag_int()) FROM t"),
                vec![i64::from(TAG) * 3]
            );
            assert_eq!(query_ints(db, c"SELECT sum_subtypes(x) FROM t"), vec![0]);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_a_window_functions_value_can_stamp_a_subtype() {
        unsafe extern "C" fn no_op_step(
            _ctx: *mut libc::c_void,
            _argc: i32,
            _argv: *mut *mut libc::c_void,
        ) {
        }
        unsafe extern "C" fn tagged_five(ctx: *mut libc::c_void) {
            sqlite3_result_int(ctx, 5);
            sqlite3_result_subtype(ctx, TAG);
        }

        unsafe {
            let (_dir, db) = open_udf_db();
            register_subtype_reader(db);
            assert_eq!(
                sqlite3_create_window_function(
                    db,
                    c"tag_window".as_ptr(),
                    1,
                    SQLITE_UTF8 | SQLITE_RESULT_SUBTYPE,
                    ptr::null_mut(),
                    Some(no_op_step),
                    Some(tagged_five),
                    Some(tagged_five),
                    Some(no_op_step),
                    None,
                ),
                SQLITE_OK
            );
            run_sql(db, c"CREATE TABLE t(x)");
            run_sql(db, c"INSERT INTO t VALUES (1), (2), (3)");
            assert_eq!(
                query_ints(
                    db,
                    c"SELECT read_subtype(tag_window(x) OVER (ORDER BY x)) FROM t"
                ),
                vec![i64::from(TAG); 3]
            );
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_an_aggregate_can_stamp_a_subtype_on_its_result() {
        unsafe extern "C" fn count_step(
            _ctx: *mut libc::c_void,
            _argc: i32,
            _argv: *mut *mut libc::c_void,
        ) {
        }
        unsafe extern "C" fn tagged_final(ctx: *mut libc::c_void) {
            sqlite3_result_int(ctx, 99);
            sqlite3_result_subtype(ctx, TAG);
        }

        unsafe {
            let (_dir, db) = open_udf_db();
            register_subtype_reader(db);
            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    c"tag_agg".as_ptr(),
                    1,
                    SQLITE_UTF8 | SQLITE_RESULT_SUBTYPE,
                    ptr::null_mut(),
                    None,
                    Some(count_step),
                    Some(tagged_final),
                    None,
                ),
                SQLITE_OK
            );
            run_sql(db, c"CREATE TABLE t(x)");
            run_sql(db, c"INSERT INTO t VALUES (1), (2)");
            assert_eq!(
                query_ints(db, c"SELECT read_subtype(tag_agg(x)) FROM t"),
                vec![i64::from(TAG)]
            );
            assert_eq!(query_ints(db, c"SELECT tag_agg(x) FROM t"), vec![99]);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    #[test]
    fn test_any_subtype_travels_from_result_to_argument() {
        const TAG: u32 = 200;

        unsafe extern "C" fn make_tagged(
            ctx: *mut libc::c_void,
            _argc: i32,
            _argv: *mut *mut libc::c_void,
        ) {
            sqlite3_result_text(ctx, c"tagged".as_ptr(), -1, transient());
            sqlite3_result_subtype(ctx, TAG);
        }
        unsafe extern "C" fn copy_value(
            ctx: *mut libc::c_void,
            _argc: i32,
            argv: *mut *mut libc::c_void,
        ) {
            sqlite3_result_value(ctx, *argv.add(0));
        }
        unsafe extern "C" fn read_subtype(
            ctx: *mut libc::c_void,
            _argc: i32,
            argv: *mut *mut libc::c_void,
        ) {
            sqlite3_result_int(ctx, sqlite3_value_subtype(*argv.add(0)) as i32);
        }

        unsafe {
            let (_dir, db) = open_udf_db();
            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    c"make_tagged".as_ptr(),
                    0,
                    SQLITE_UTF8 | SQLITE_RESULT_SUBTYPE,
                    ptr::null_mut(),
                    Some(make_tagged),
                    None,
                    None,
                    None,
                ),
                SQLITE_OK
            );
            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    c"copy_value".as_ptr(),
                    1,
                    SQLITE_UTF8 | SQLITE_SUBTYPE | SQLITE_RESULT_SUBTYPE,
                    ptr::null_mut(),
                    Some(copy_value),
                    None,
                    None,
                    None,
                ),
                SQLITE_OK
            );
            assert_eq!(
                sqlite3_create_function_v2(
                    db,
                    c"read_subtype".as_ptr(),
                    1,
                    SQLITE_UTF8 | SQLITE_SUBTYPE,
                    ptr::null_mut(),
                    Some(read_subtype),
                    None,
                    None,
                    None,
                ),
                SQLITE_OK
            );

            assert_eq!(
                query_ints(db, c"SELECT read_subtype(make_tagged())"),
                vec![TAG as i64]
            );
            // sqlite3_result_value carries the subtype with the value.
            assert_eq!(
                query_ints(db, c"SELECT read_subtype(copy_value(make_tagged()))"),
                vec![TAG as i64]
            );
            assert_eq!(query_ints(db, c"SELECT read_subtype('tagged')"), vec![0]);
            assert_eq!(query_texts(db, c"SELECT make_tagged()"), vec!["tagged"]);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }

    // Where Turso deviates

    /// Only the name is stored, so CREATE fails when the connection cannot resolve
    /// it, and a connection lacking it reads the schema but cannot compare with it.
    #[test]
    fn test_custom_collation_in_the_schema() {
        unsafe {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("collation_schema.db");
            let path = CString::new(path.to_str().unwrap()).unwrap();
            let mut db: *mut sqlite3 = ptr::null_mut();
            assert_eq!(sqlite3_open(path.as_ptr(), &mut db), SQLITE_OK);
            make_names_table(db);

            // A name the connection cannot resolve is refused at CREATE time.
            for sql in [
                c"CREATE TABLE bad(value TEXT COLLATE turso_missing)",
                c"CREATE INDEX bad_idx ON names(value COLLATE turso_missing)",
            ] {
                let mut err: *mut libc::c_char = ptr::null_mut();
                assert_ne!(
                    sqlite3_exec(db, sql.as_ptr(), None, ptr::null_mut(), &mut err),
                    SQLITE_OK,
                    "{sql:?} should be refused"
                );
                let message = CStr::from_ptr(err).to_string_lossy().into_owned();
                assert!(
                    message.contains("no such collation sequence: turso_missing"),
                    "unexpected message: {message}"
                );
                sqlite3_free(err as *mut libc::c_void);
            }

            // A registered one is accepted, in a column and in an index.
            assert_eq!(
                sqlite3_create_collation(
                    db,
                    c"turso_schema".as_ptr(),
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    Some(nocase_utf8),
                ),
                SQLITE_OK
            );
            run_sql(
                db,
                c"CREATE TABLE collated(value TEXT COLLATE turso_schema)",
            );
            run_sql(db, c"INSERT INTO collated VALUES ('alpha'), ('BETA')");
            run_sql(
                db,
                c"CREATE INDEX names_idx ON names(value COLLATE turso_schema)",
            );
            // The column's own collation drives the comparison.
            assert_eq!(
                query_ints(db, c"SELECT count(*) FROM collated WHERE value = 'ALPHA'"),
                vec![1]
            );

            // A connection that never registered it opens the database, but
            // writing through the index needs it.
            let mut other: *mut sqlite3 = ptr::null_mut();
            assert_eq!(sqlite3_open(path.as_ptr(), &mut other), SQLITE_OK);
            let mut err: *mut libc::c_char = ptr::null_mut();
            assert_ne!(
                sqlite3_exec(
                    other,
                    c"INSERT INTO names VALUES ('delta')".as_ptr(),
                    None,
                    ptr::null_mut(),
                    &mut err,
                ),
                SQLITE_OK,
                "insert without the collation should be refused"
            );
            let message = CStr::from_ptr(err).to_string_lossy().into_owned();
            assert!(
                message.contains("no such collation sequence: turso_schema"),
                "unexpected message: {message}"
            );
            sqlite3_free(err as *mut libc::c_void);

            // Registering the same name makes the write work again.
            assert_eq!(
                sqlite3_create_collation(
                    other,
                    c"turso_schema".as_ptr(),
                    SQLITE_UTF8,
                    ptr::null_mut(),
                    Some(nocase_utf8),
                ),
                SQLITE_OK
            );
            run_sql(other, c"INSERT INTO names VALUES ('delta')");
            assert_eq!(query_ints(other, c"SELECT count(*) FROM names"), vec![5]);

            assert_eq!(sqlite3_close(other), SQLITE_OK);
            assert_eq!(sqlite3_close(db), SQLITE_OK);
        }
    }
}
