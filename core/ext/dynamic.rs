use crate::{
    ext::{
        register_aggregate_function, register_scalar_function_with_options, register_vtab_module,
        register_window_function, unregister_function,
    },
    Connection, LimboError,
};
#[cfg(not(target_family = "wasm"))]
use libloading::{Library, Symbol};
use std::{
    ffi::{c_char, CString},
    sync::{Arc, Mutex, OnceLock},
};
use turso_ext::{
    ExtensionApi, ExtensionApiRef, ExtensionEntryPoint, ResultCode, VfsImpl, TURSO_EXT_API_VERSION,
};

/// Optional symbol an extension exports to report its ABI version.
#[cfg(not(target_family = "wasm"))]
type ExtensionApiVersionFn = unsafe extern "C" fn() -> u32;

/// ABI version an extension speaks. `reported` is `None` when it exports no
/// version symbol, which means version 1. An older extension is fine because
/// the ABI only grows; a newer one may expect fields this host lacks.
fn extension_abi_version(reported: Option<u32>, host: u32) -> crate::Result<u32> {
    let version = reported.unwrap_or(1);
    if version > host {
        return Err(LimboError::ExtensionError(format!(
            "extension built for turso_ext ABI v{version}, this build supports v{host}"
        )));
    }
    Ok(version)
}

#[cfg(not(target_family = "wasm"))]
type ExtensionStore = Vec<(Arc<Library>, ExtensionApiRef)>;
#[cfg(not(target_family = "wasm"))]
static EXTENSIONS: OnceLock<Arc<Mutex<ExtensionStore>>> = OnceLock::new();
#[cfg(not(target_family = "wasm"))]
pub fn get_extension_libraries() -> Arc<Mutex<ExtensionStore>> {
    EXTENSIONS
        .get_or_init(|| Arc::new(Mutex::new(Vec::new())))
        .clone()
}

type Vfs = (String, Arc<VfsMod>);
static VFS_MODULES: OnceLock<Mutex<Vec<Vfs>>> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct VfsMod {
    pub ctx: *const VfsImpl,
}

unsafe impl Send for VfsMod {}
unsafe impl Sync for VfsMod {}
crate::assert::assert_send_sync!(VfsMod);

impl Connection {
    #[cfg(not(target_family = "wasm"))]
    pub fn load_extension<P: AsRef<std::ffi::OsStr>>(
        self: &Arc<Connection>,
        path: P,
    ) -> crate::Result<()> {
        use turso_ext::ExtensionApiRef;

        let lib =
            unsafe { Library::new(path).map_err(|e| LimboError::ExtensionError(e.to_string()))? };
        let reported_abi_version = unsafe {
            lib.get::<ExtensionApiVersionFn>(b"turso_ext_api_version")
                .ok()
                .map(|version| version())
        };
        let abi_version = extension_abi_version(reported_abi_version, TURSO_EXT_API_VERSION)?;
        let api = Box::new(unsafe { self.build_turso_ext_for_abi(abi_version) });
        let entry: Symbol<ExtensionEntryPoint> = unsafe {
            lib.get(b"register_extension")
                .map_err(|e| LimboError::ExtensionError(e.to_string()))?
        };
        let api_ptr: *const ExtensionApi = Box::into_raw(api);
        let api_ref = ExtensionApiRef { api: api_ptr };
        let result_code = unsafe { entry(api_ptr) };
        if result_code.is_ok() {
            let extensions = get_extension_libraries();
            extensions
                .lock()
                .map_err(|_| {
                    LimboError::ExtensionError("Error locking extension libraries".to_string())
                })?
                .push((Arc::new(lib), api_ref));
            if self.is_db_initialized() {
                self.reparse_schema_after_extension_load()?;
            }
            Ok(())
        } else {
            if !api_ptr.is_null() {
                let _ = unsafe { Box::from_raw(api_ptr.cast_mut()) };
            }
            Err(LimboError::ExtensionError(
                "Extension registration failed".to_string(),
            ))
        }
    }
}

#[allow(clippy::arc_with_non_send_sync)]
pub(crate) unsafe extern "C" fn register_vfs(
    name: *const c_char,
    vfs: *const VfsImpl,
) -> ResultCode {
    if name.is_null() || vfs.is_null() {
        return ResultCode::Error;
    }
    let c_str = unsafe { CString::from_raw(name as *mut _) };
    let name_str = match c_str.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return ResultCode::Error,
    };
    add_vfs_module(name_str, Arc::new(VfsMod { ctx: vfs }));
    ResultCode::OK
}

/// Get pointers to all the vfs extensions that need to be built in at compile time.
/// any other types that are defined in the same extension will not be registered
/// until the database file is opened and `register_builtins` is called.
#[cfg(feature = "fs")]
#[allow(clippy::arc_with_non_send_sync)]
pub fn add_builtin_vfs_extensions(
    api: Option<ExtensionApi>,
) -> crate::Result<Vec<(String, Arc<VfsMod>)>> {
    use turso_ext::VfsInterface;

    let mut vfslist: Vec<*const VfsImpl> = Vec::new();
    let mut api = match api {
        None => ExtensionApi {
            ctx: std::ptr::null_mut(),
            register_scalar_function: register_scalar_function_with_options,
            register_aggregate_function,
            unregister_function,
            register_vtab_module,
            vfs_interface: VfsInterface {
                register_vfs,
                builtin_vfs: vfslist.as_mut_ptr(),
                builtin_vfs_count: 0,
            },
            api_version: TURSO_EXT_API_VERSION,
            register_window_function,
        },
        Some(mut api) => {
            api.vfs_interface.builtin_vfs = vfslist.as_mut_ptr();
            api
        }
    };
    register_static_vfs_modules(&mut api);
    let mut vfslist = Vec::with_capacity(api.vfs_interface.builtin_vfs_count as usize);
    let slice = unsafe {
        std::slice::from_raw_parts_mut(
            api.vfs_interface.builtin_vfs,
            api.vfs_interface.builtin_vfs_count as usize,
        )
    };
    for vfs in slice {
        if vfs.is_null() {
            continue;
        }
        let vfsimpl = unsafe { &**vfs };
        let name = unsafe {
            CString::from_raw(vfsimpl.name as *mut _)
                .to_str()
                .map_err(|_| {
                    LimboError::ExtensionError("unable to register vfs extension".to_string())
                })?
                .to_string()
        };
        vfslist.push((
            name,
            Arc::new(VfsMod {
                ctx: vfsimpl as *const _,
            }),
        ));
    }
    Ok(vfslist)
}

#[allow(dead_code)]
#[cfg(feature = "fs")]
fn register_static_vfs_modules(_api: &mut ExtensionApi) {
    /* Placeholder for any VFS modules to build in at compile time */
}

pub fn add_vfs_module(name: String, vfs: Arc<VfsMod>) {
    let mut modules = VFS_MODULES
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .unwrap();
    if !modules.iter().any(|v| v.0 == name) {
        modules.push((name, vfs));
    }
}

pub fn list_vfs_modules() -> Vec<String> {
    VFS_MODULES
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .unwrap()
        .iter()
        .map(|v| v.0.clone())
        .collect()
}

pub fn get_vfs_modules() -> Vec<Vfs> {
    VFS_MODULES
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .unwrap()
        .clone()
}

#[cfg(test)]
mod tests {
    use super::extension_abi_version;

    #[test]
    fn extension_without_a_version_symbol_is_the_first_abi() {
        assert_eq!(extension_abi_version(None, 2).unwrap(), 1);
    }

    #[test]
    fn extension_older_than_the_host_loads_at_its_own_version() {
        assert_eq!(extension_abi_version(Some(1), 2).unwrap(), 1);
        assert_eq!(extension_abi_version(Some(2), 5).unwrap(), 2);
    }

    #[test]
    fn extension_matching_the_host_loads() {
        assert_eq!(extension_abi_version(Some(2), 2).unwrap(), 2);
    }

    #[test]
    fn extension_newer_than_the_host_is_refused() {
        let err = extension_abi_version(Some(3), 2).unwrap_err();
        assert_eq!(
            err.to_string(),
            "Extension error: extension built for turso_ext ABI v3, this build supports v2"
        );
    }
}
