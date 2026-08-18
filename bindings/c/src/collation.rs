//! The collating-sequence half of the sqlite3 C API.
//!
//! Every registration made under one name shares one heap [`CollationSet`],
//! whose address is the `context` core holds, with one slot per text
//! encoding. Core always compares UTF-8 bytes, so a UTF-16 registration gets
//! its two strings re-encoded before the callback sees them.

use std::collections::HashSet;
use std::ffi::{self, CStr, CString};
use std::sync::{Arc, RwLock};

use crate::udf::{SQLITE_UTF16, SQLITE_UTF16BE, SQLITE_UTF16LE, SQLITE_UTF16_ALIGNED, SQLITE_UTF8};
use crate::{sqlite3, sqlite3Inner, SQLITE_BUSY, SQLITE_MISUSE, SQLITE_OK};

/// `(pArg, nLeft, pLeft, nRight, pRight)`, with both lengths in bytes.
pub type XCompare = unsafe extern "C" fn(
    *mut ffi::c_void,
    ffi::c_int,
    *const ffi::c_void,
    ffi::c_int,
    *const ffi::c_void,
) -> ffi::c_int;

pub type XDestroy = unsafe extern "C" fn(*mut ffi::c_void);

/// The `xCollNeeded` callback. Its last argument is the collation name, UTF-8
/// or UTF-16 depending on which entry point registered the callback.
pub type XCollationNeeded =
    unsafe extern "C" fn(*mut ffi::c_void, *mut sqlite3, ffi::c_int, *const ffi::c_char);

#[derive(Clone, Copy, PartialEq, Eq)]
enum TextEncoding {
    Utf8,
    Utf16Le,
    Utf16Be,
}

impl TextEncoding {
    /// The "whatever is native" spellings become the native one. SQLite
    /// truncates `enc` to a byte first, so or-ed in flag bits are ignored.
    fn from_arg(enc: ffi::c_int) -> Option<Self> {
        match enc & 0xff {
            SQLITE_UTF16 | SQLITE_UTF16_ALIGNED => Some(if cfg!(target_endian = "big") {
                Self::Utf16Be
            } else {
                Self::Utf16Le
            }),
            SQLITE_UTF8 => Some(Self::Utf8),
            SQLITE_UTF16LE => Some(Self::Utf16Le),
            SQLITE_UTF16BE => Some(Self::Utf16Be),
            _ => None,
        }
    }

    fn slot(self) -> usize {
        match self {
            Self::Utf8 => 0,
            Self::Utf16Le => 1,
            Self::Utf16Be => 2,
        }
    }
}

/// Dropping it runs `xDestroy(pArg)`, the lifetime SQLite documents.
struct CollationEntry {
    p_arg: *mut ffi::c_void,
    /// `None` for the empty entry a deletion leaves behind, which only holds
    /// the deleting call's destructor until the handle closes.
    x_compare: Option<XCompare>,
    x_destroy: Option<XDestroy>,
    enc: TextEncoding,
}

impl Drop for CollationEntry {
    fn drop(&mut self) {
        if let Some(x_destroy) = self.x_destroy {
            // SAFETY: the registering caller paired destructor and pointer.
            unsafe { x_destroy(self.p_arg) };
        }
    }
}

/// What core holds as the collation's `context`. Core and this handle both
/// hold a reference, so core keeps comparing while the handle swaps entries.
pub(crate) struct CollationSet {
    entries: RwLock<[Option<CollationEntry>; 3]>,
}

// SAFETY: the lock covers every access to the entries, and `pArg` is opaque
// here — never read, only handed back to the caller's own callbacks.
unsafe impl Send for CollationSet {}
unsafe impl Sync for CollationSet {}

/// UTF-8 needs no conversion and so wins; otherwise BE before LE, the order
/// SQLite's `synthCollSeq` uses.
const PREFERENCE: [TextEncoding; 3] = [
    TextEncoding::Utf8,
    TextEncoding::Utf16Be,
    TextEncoding::Utf16Le,
];

impl CollationSet {
    pub(crate) fn new() -> Self {
        Self {
            entries: RwLock::new([None, None, None]),
        }
    }

    /// The empty entry a deletion leaves behind does not count.
    fn has_comparison(&self, enc: TextEncoding) -> bool {
        self.entries.read().unwrap()[enc.slot()]
            .as_ref()
            .is_some_and(|entry| entry.x_compare.is_some())
    }

    /// Whether a statement naming this collation can run at all.
    fn has_any_comparison(&self) -> bool {
        self.entries
            .read()
            .unwrap()
            .iter()
            .flatten()
            .any(|entry| entry.x_compare.is_some())
    }

    #[must_use]
    fn replace(&self, entry: CollationEntry) -> Option<CollationEntry> {
        let slot = entry.enc.slot();
        self.entries.write().unwrap()[slot].replace(entry)
    }

    /// Copied out so no lock is held while the caller's callback runs.
    fn chosen(&self) -> Option<(*mut ffi::c_void, XCompare, TextEncoding)> {
        let entries = self.entries.read().unwrap();
        PREFERENCE.iter().find_map(|enc| {
            let entry = entries[enc.slot()].as_ref()?;
            Some((entry.p_arg, entry.x_compare?, entry.enc))
        })
    }
}

pub(crate) struct HandleCollation {
    pub(crate) name: String,
    pub(crate) set: Arc<CollationSet>,
}

/// `u16` rather than loose bytes so the buffer is two-byte aligned, which
/// SQLITE_UTF16_ALIGNED promises. Each unit holds its bytes in the asked-for
/// order, so reading the buffer as bytes gives that encoding.
fn to_utf16(bytes: &[u8], big_endian: bool) -> Vec<u16> {
    let text = String::from_utf8_lossy(bytes);
    text.encode_utf16()
        .map(|unit| {
            if big_endian {
                unit.to_be()
            } else {
                unit.to_le()
            }
        })
        .collect()
}

/// The one comparison callback core sees; `context` is a [`CollationSet`].
unsafe extern "C" fn compare_trampoline(
    context: usize,
    left_ptr: *const u8,
    left_len: usize,
    right_ptr: *const u8,
    right_len: usize,
) -> i32 {
    let set = &*(context as *const CollationSet);
    // Deleting a collation is refused while a statement runs and re-prepares the rest, so an
    // empty set here means that guard failed; comparing as equal would silently mis-order keys.
    let Some((p_arg, x_compare, enc)) = set.chosen() else {
        unreachable!("collation compared after every registration under its name was deleted");
    };
    let left = std::slice::from_raw_parts(left_ptr, left_len);
    let right = std::slice::from_raw_parts(right_ptr, right_len);
    match enc {
        TextEncoding::Utf8 => x_compare(
            p_arg,
            left.len() as ffi::c_int,
            left.as_ptr() as *const ffi::c_void,
            right.len() as ffi::c_int,
            right.as_ptr() as *const ffi::c_void,
        ),
        enc => {
            let big_endian = enc == TextEncoding::Utf16Be;
            let left = to_utf16(left, big_endian);
            let right = to_utf16(right, big_endian);
            x_compare(
                p_arg,
                (left.len() * 2) as ffi::c_int,
                left.as_ptr() as *const ffi::c_void,
                (right.len() * 2) as ffi::c_int,
                right.as_ptr() as *const ffi::c_void,
            )
        }
    }
}

/// Releases core's reference; the registrations' destructors run once the
/// handle lets go of the set too.
unsafe extern "C" fn destroy_trampoline(context: usize) {
    drop(Arc::from_raw(context as *const CollationSet));
}

/// `None` when the name is missing or not UTF-8, which SQLite would accept.
unsafe fn collation_name(name: *const ffi::c_char) -> Option<String> {
    if name.is_null() {
        return None;
    }
    CStr::from_ptr(name).to_str().ok().map(str::to_owned)
}

/// Decodes a UTF-16 name in the platform's byte order.
unsafe fn collation_name16(name: *const ffi::c_void) -> Option<String> {
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

unsafe fn create_collation(
    db: *mut sqlite3,
    name: Option<String>,
    enc: ffi::c_int,
    p_arg: *mut ffi::c_void,
    x_compare: Option<XCompare>,
    x_destroy: Option<XDestroy>,
) -> ffi::c_int {
    // Unlike every other destructor in this API, a failed
    // sqlite3_create_collation_v2 does not run xDestroy, so neither do we.
    if db.is_null() {
        return SQLITE_MISUSE;
    }
    let Some(name) = name else {
        return SQLITE_MISUSE;
    };
    let Some(enc) = TextEncoding::from_arg(enc) else {
        return SQLITE_MISUSE;
    };

    let db_ref = &*db;
    let mut inner = db_ref.inner.lock().unwrap();

    // A comparison may not move under a running statement, but an encoding
    // nobody registered yet is new and may be added even then.
    let known = inner.collation_set(&name);
    if known.as_ref().is_some_and(|set| set.has_comparison(enc))
        && crate::has_running_statement(&inner)
    {
        return crate::set_db_err_message(
            &mut inner,
            SQLITE_BUSY,
            "unable to delete/modify collation sequence due to active statements",
        );
    }

    let set = match known {
        Some(set) => set,
        None => inner.remember_collation(&name),
    };

    // A NULL comparison deletes, leaving this call's destructor to run when
    // the handle closes rather than now.
    let was_usable = set.has_any_comparison();
    let replaced = set.replace(CollationEntry {
        p_arg,
        x_compare,
        x_destroy,
        enc,
    });
    let is_usable = set.has_any_comparison();

    if is_usable && !was_usable {
        // Core is told about the set once; the trampoline picks the entry.
        inner.conn.register_external_collation(
            name.clone(),
            Arc::into_raw(set.clone()) as usize,
            compare_trampoline,
            Some(destroy_trampoline),
        );
    } else if was_usable && !is_usable {
        // Nothing left to compare with, so the name must fail again.
        inner.conn.unregister_external_collation(&name);
    }
    drop(inner);
    // Runs the old registration's destructor, with no lock held.
    drop(replaced);
    SQLITE_OK
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_create_collation(
    db: *mut sqlite3,
    name: *const ffi::c_char,
    enc: ffi::c_int,
    p_arg: *mut ffi::c_void,
    x_compare: Option<XCompare>,
) -> ffi::c_int {
    create_collation(db, collation_name(name), enc, p_arg, x_compare, None)
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_create_collation_v2(
    db: *mut sqlite3,
    name: *const ffi::c_char,
    enc: ffi::c_int,
    p_arg: *mut ffi::c_void,
    x_compare: Option<XCompare>,
    x_destroy: Option<XDestroy>,
) -> ffi::c_int {
    create_collation(db, collation_name(name), enc, p_arg, x_compare, x_destroy)
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_create_collation16(
    db: *mut sqlite3,
    name: *const ffi::c_void,
    enc: ffi::c_int,
    p_arg: *mut ffi::c_void,
    x_compare: Option<XCompare>,
) -> ffi::c_int {
    create_collation(db, collation_name16(name), enc, p_arg, x_compare, None)
}

/// Drops every collation this handle registered, running each `xDestroy`.
pub(crate) fn drop_all_collations(inner: &mut sqlite3Inner) {
    for known in std::mem::take(&mut inner.collations) {
        // Releases core's reference; the handle's own goes with `known`.
        inner.conn.unregister_external_collation(&known.name);
    }
}

#[derive(Clone, Copy)]
pub(crate) struct CollationNeeded {
    callback: XCollationNeeded,
    p_arg: *mut ffi::c_void,
    utf16: bool,
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_collation_needed(
    db: *mut sqlite3,
    p_arg: *mut ffi::c_void,
    callback: Option<XCollationNeeded>,
) -> ffi::c_int {
    set_collation_needed(db, p_arg, callback, false)
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_collation_needed16(
    db: *mut sqlite3,
    p_arg: *mut ffi::c_void,
    callback: Option<XCollationNeeded>,
) -> ffi::c_int {
    set_collation_needed(db, p_arg, callback, true)
}

unsafe fn set_collation_needed(
    db: *mut sqlite3,
    p_arg: *mut ffi::c_void,
    callback: Option<XCollationNeeded>,
    utf16: bool,
) -> ffi::c_int {
    if db.is_null() {
        return SQLITE_MISUSE;
    }
    let mut inner = (*db).inner.lock().unwrap();
    inner.collation_needed = callback.map(|callback| CollationNeeded {
        callback,
        p_arg,
        utf16,
    });
    SQLITE_OK
}

/// The collation a prepare failed on, if that is why it failed.
pub(crate) fn missing_collation(err: &turso_core::LimboError) -> Option<String> {
    const PREFIX: &str = "no such collation sequence: ";
    let message = err.to_string();
    let at = message.find(PREFIX)?;
    Some(message[at + PREFIX.len()..].trim().to_string())
}

/// Asks the `sqlite3_collation_needed` callback for `name`. False when there
/// is no callback or this prepare already asked, which ends the retry loop.
///
/// The handle lock must not be held: the callback is expected to call
/// `sqlite3_create_collation` on the same handle.
pub(crate) unsafe fn request_collation(
    db: *mut sqlite3,
    name: &str,
    already_asked: &mut HashSet<String>,
) -> bool {
    let needed = {
        let inner = (*db).inner.lock().unwrap();
        inner.collation_needed
    };
    let Some(needed) = needed else {
        return false;
    };
    if !already_asked.insert(name.to_ascii_lowercase()) {
        return false;
    }
    // The encoding argument is the connection's, always UTF-8 here.
    if needed.utf16 {
        let mut units: Vec<u16> = name.encode_utf16().collect();
        units.push(0);
        (needed.callback)(
            needed.p_arg,
            db,
            SQLITE_UTF8,
            units.as_ptr() as *const ffi::c_char,
        );
    } else {
        let Ok(name) = CString::new(name) else {
            return false;
        };
        (needed.callback)(needed.p_arg, db, SQLITE_UTF8, name.as_ptr());
    }
    true
}
