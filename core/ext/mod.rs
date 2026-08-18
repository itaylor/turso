pub mod adapter;
#[cfg(feature = "fs")]
mod dynamic;
mod vtab_xconnect;
use crate::index_method::backing_btree::BackingBtreeIndexMethod;
#[cfg(all(feature = "fts", not(target_family = "wasm")))]
use crate::index_method::fts::{FtsIndexMethod, FTS_INDEX_METHOD_NAME};
use crate::index_method::toy_vector_sparse_ivf::VectorSparseInvertedIndexMethod;
use crate::index_method::{
    BACKING_BTREE_INDEX_METHOD_NAME, TOY_VECTOR_SPARSE_IVF_INDEX_METHOD_NAME,
};
use crate::schema::{Schema, Table};
use crate::sync::atomic::{AtomicU64, Ordering};
use crate::sync::Mutex;
#[cfg(all(target_os = "linux", feature = "io_uring", not(miri)))]
use crate::UringIO;
#[cfg(all(target_os = "windows", feature = "experimental_win_iocp", not(miri)))]
use crate::WindowsIOCP;

use crate::sync::RwLock;
use crate::udf::{validate_registration, ExternalFunc, FunctionFlags};
use crate::{vtab::VirtualTable, SymbolTable};
use crate::{Connection, Database};
#[cfg(feature = "fs")]
use crate::{LimboError, IO};
pub use adapter::{
    ExtAggregateAdapter, ExtScalarAdapter, WindowInverseFunction, WindowValueFunction,
};
#[cfg(feature = "fs")]
pub use dynamic::{add_builtin_vfs_extensions, add_vfs_module, list_vfs_modules, VfsMod};
use std::{
    ffi::{c_char, c_void, CStr, CString},
    sync::Arc,
};
use turso_ext::{
    ContextDestructor, ExtensionApi, InitAggFunction, ResultCode, ScalarFunction, VTabKind,
    VTabModuleImpl, ValueDestructor, TURSO_EXT_API_VERSION,
};
pub use turso_ext::{FinalizeFunction, StepFunction, Value as ExtValue, ValueType as ExtValueType};
pub use vtab_xconnect::{execute, prepare_stmt};

/// The context passed to extensions to register with Core
/// along with the function pointers.
///
/// Holds the symbol table's lock, not the table, so the C-ABI shims take the
/// write lock like every other writer. The pointer stays valid because the
/// context only lives between `_build_turso_ext` and `_free_extension_ctx`.
#[repr(C)]
pub struct ExtensionCtx {
    syms: *const RwLock<SymbolTable>,
    schema: *mut c_void,
    /// We must bump the prepare context generation so prepared statements
    /// know they need to be reprepared after extension registration.
    prepare_context_generation: *const AtomicU64,
    /// ABI the registering extension was built against. One context is built
    /// per `load_extension` call, so this is per-extension.
    abi_version: u32,
}

impl ExtensionCtx {
    unsafe fn bump_generation(&self) {
        if !self.prepare_context_generation.is_null() {
            unsafe { (*self.prepare_context_generation).fetch_add(1, Ordering::Release) };
        }
    }
}

pub(crate) unsafe extern "C" fn register_vtab_module(
    ctx: *mut c_void,
    name: *const c_char,
    module: VTabModuleImpl,
    kind: VTabKind,
) -> ResultCode {
    if name.is_null() || ctx.is_null() {
        return ResultCode::Error;
    }

    let c_str = unsafe { CString::from_raw(name as *mut c_char) };
    let name_str = match c_str.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return ResultCode::Error,
    };

    let ext_ctx = unsafe { &mut *(ctx as *mut ExtensionCtx) };
    let module = Arc::new(module);
    let vmodule = VTabImpl {
        module_kind: kind,
        implementation: module,
    };

    unsafe {
        let vtab = {
            let mut syms = (*ext_ctx.syms).write();
            syms.vtab_modules.insert(name_str.clone(), vmodule.into());
            if kind == VTabKind::TableValuedFunction {
                match VirtualTable::function(&name_str, &syms) {
                    Ok(vtab) => Some(vtab),
                    Err(_) => return ResultCode::Error,
                }
            } else {
                None
            }
        };
        ext_ctx.bump_generation();

        // Take the schema lock only after dropping the symbol-table lock, so
        // the two never nest.
        if let Some(vtab) = vtab {
            let table = Arc::new(Table::Virtual(vtab));
            let mutex = &*(ext_ctx.schema as *mut Mutex<Arc<Schema>>);
            let mut guard = mutex.lock();
            let Ok(schema) = Schema::try_make_mut(&mut guard) else {
                return ResultCode::Error;
            };
            schema.tables.insert(name_str, table);
        }
    }
    ResultCode::OK
}

#[derive(Clone)]
pub struct VTabImpl {
    pub module_kind: VTabKind,
    pub implementation: Arc<VTabModuleImpl>,
}

pub(crate) unsafe fn register_scalar_function(
    ctx: *mut c_void,
    name: *const c_char,
    func: ScalarFunction,
) -> ResultCode {
    unsafe { register_scalar_function_with_options(ctx, name, -1, false, 0, func, None, None) }
}

pub(crate) unsafe extern "C" fn register_scalar_function_with_options(
    ctx: *mut c_void,
    name: *const c_char,
    argc: i32,
    deterministic: bool,
    context: usize,
    callback: ScalarFunction,
    context_destructor: Option<ContextDestructor>,
    value_destructor: Option<ValueDestructor>,
) -> ResultCode {
    if ctx.is_null() || name.is_null() {
        return ResultCode::InvalidArgs;
    }
    let c_str = unsafe { CStr::from_ptr(name) };
    let name_str = match c_str.to_str() {
        Ok(s) => crate::util::normalize_ident(s),
        Err(_) => return ResultCode::InvalidArgs,
    };
    if validate_registration(&name_str, argc).is_err() {
        return ResultCode::InvalidArgs;
    }
    let mut flags = FunctionFlags::empty();
    if deterministic {
        flags |= FunctionFlags::DETERMINISTIC;
    }
    let func = Arc::new(ExternalFunc::new_scalar(
        name_str,
        argc,
        flags,
        Arc::new(
            ExtScalarAdapter::new(context, callback, context_destructor, value_destructor)
                .with_abi_version(unsafe { (*(ctx as *mut ExtensionCtx)).abi_version }),
        ),
    ));
    let ext_ctx = unsafe { &mut *(ctx as *mut ExtensionCtx) };
    unsafe {
        (*ext_ctx.syms).write().insert_function(func);
        ext_ctx.bump_generation();
    }
    ResultCode::OK
}

pub(crate) unsafe extern "C" fn unregister_function(
    ctx: *mut c_void,
    name: *const c_char,
) -> ResultCode {
    if ctx.is_null() || name.is_null() {
        return ResultCode::InvalidArgs;
    }
    let c_str = unsafe { CStr::from_ptr(name) };
    let name_str = match c_str.to_str() {
        Ok(s) => crate::util::normalize_ident(s),
        Err(_) => return ResultCode::InvalidArgs,
    };
    let ext_ctx = unsafe { &mut *(ctx as *mut ExtensionCtx) };
    unsafe {
        // The C ABI takes no argument count, so every arity goes.
        if !(*ext_ctx.syms)
            .write()
            .remove_all_functions_named(&name_str)
        {
            return ResultCode::NotFound;
        }
        ext_ctx.bump_generation();
    }
    ResultCode::OK
}

pub(crate) unsafe extern "C" fn register_aggregate_function(
    ctx: *mut c_void,
    name: *const c_char,
    args: i32,
    context: usize,
    init_func: InitAggFunction,
    step_func: StepFunction,
    finalize_func: FinalizeFunction,
    context_destructor: Option<ContextDestructor>,
    aggregate_destructor: Option<ContextDestructor>,
    value_destructor: Option<ValueDestructor>,
) -> ResultCode {
    if ctx.is_null() || name.is_null() {
        return ResultCode::InvalidArgs;
    }
    let c_str = unsafe { CStr::from_ptr(name) };
    let name_str = match c_str.to_str() {
        Ok(s) => crate::util::normalize_ident(s),
        Err(_) => return ResultCode::InvalidArgs,
    };
    if validate_registration(&name_str, args).is_err() {
        return ResultCode::InvalidArgs;
    }
    // The ABI has no way to declare flags for a plain aggregate.
    let func = Arc::new(ExternalFunc::new_aggregate(
        name_str,
        args,
        FunctionFlags::empty(),
        Arc::new(
            ExtAggregateAdapter::new(
                context,
                init_func,
                step_func,
                finalize_func,
                context_destructor,
                aggregate_destructor,
                value_destructor,
            )
            .with_abi_version(unsafe { (*(ctx as *mut ExtensionCtx)).abi_version }),
        ),
    ));
    let ext_ctx = unsafe { &mut *(ctx as *mut ExtensionCtx) };
    unsafe {
        (*ext_ctx.syms).write().insert_function(func);
        ext_ctx.bump_generation();
    }
    ResultCode::OK
}

/// [`register_aggregate_function`] plus the `xValue`/`xInverse` callbacks that
/// let the aggregate run over a window frame.
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe extern "C" fn register_window_function(
    ctx: *mut c_void,
    name: *const c_char,
    args: i32,
    flags: u32,
    context: usize,
    init_func: InitAggFunction,
    step_func: StepFunction,
    finalize_func: FinalizeFunction,
    value_func: WindowValueFunction,
    inverse_func: WindowInverseFunction,
    context_destructor: Option<ContextDestructor>,
    aggregate_destructor: Option<ContextDestructor>,
    value_destructor: Option<ValueDestructor>,
) -> ResultCode {
    if ctx.is_null() || name.is_null() {
        return ResultCode::InvalidArgs;
    }
    let c_str = unsafe { CStr::from_ptr(name) };
    let name_str = match c_str.to_str() {
        Ok(s) => crate::util::normalize_ident(s),
        Err(_) => return ResultCode::InvalidArgs,
    };
    if validate_registration(&name_str, args).is_err() {
        return ResultCode::InvalidArgs;
    }
    let func = Arc::new(ExternalFunc::new_aggregate(
        name_str,
        args,
        // Truncate rather than reject, so a newer header still registers.
        FunctionFlags::from_bits_truncate(flags),
        Arc::new(
            ExtAggregateAdapter::new_window(
                context,
                init_func,
                step_func,
                finalize_func,
                value_func,
                inverse_func,
                context_destructor,
                aggregate_destructor,
                value_destructor,
            )
            .with_abi_version(unsafe { (*(ctx as *mut ExtensionCtx)).abi_version }),
        ),
    ));
    let ext_ctx = unsafe { &mut *(ctx as *mut ExtensionCtx) };
    unsafe {
        (*ext_ctx.syms).write().insert_function(func);
        ext_ctx.bump_generation();
    }
    ResultCode::OK
}

impl Database {
    #[cfg(feature = "fs")]
    #[allow(clippy::arc_with_non_send_sync, dead_code)]
    pub fn open_with_vfs(
        &self,
        path: &str,
        vfs: &str,
        dialect: Arc<dyn crate::Dialect>,
    ) -> crate::Result<(Arc<dyn IO>, Arc<Database>)> {
        use crate::{MemoryIO, SyscallIO};
        use dynamic::get_vfs_modules;

        let io: Arc<dyn IO> = match vfs {
            "memory" => Arc::new(MemoryIO::new()),
            #[cfg(feature = "io_memory_yield")]
            "memory_yield" => Arc::new(crate::MemoryYieldIO::new()),
            "syscall" => Arc::new(SyscallIO::new()?),
            #[cfg(all(target_os = "linux", feature = "io_uring", not(miri)))]
            "io_uring" => Arc::new(UringIO::new()?),
            #[cfg(all(target_os = "windows", feature = "experimental_win_iocp", not(miri)))]
            "experimental_win_iocp" => Arc::new(WindowsIOCP::new()?),
            other => match get_vfs_modules().iter().find(|v| v.0 == vfs) {
                Some((_, vfs)) => vfs.clone(),
                None => {
                    return Err(LimboError::InvalidArgument(format!("no such VFS: {other}")));
                }
            },
        };
        let db = Self::open_file(io.clone(), path, dialect)?;
        Ok((io, db))
    }

    /// Register any built-in extensions that can be stored on the Database so we do not have
    /// to register these once-per-connection, and the connection can just extend its symbol table
    pub fn register_global_builtin_extensions(&self) -> Result<(), String> {
        {
            let mut syms = self.builtin_syms.write();
            syms.index_methods.insert(
                TOY_VECTOR_SPARSE_IVF_INDEX_METHOD_NAME.to_string(),
                Arc::new(VectorSparseInvertedIndexMethod),
            );
            syms.index_methods.insert(
                BACKING_BTREE_INDEX_METHOD_NAME.to_string(),
                Arc::new(BackingBtreeIndexMethod),
            );
            #[cfg(all(feature = "fts", not(target_family = "wasm")))]
            syms.index_methods
                .insert(FTS_INDEX_METHOD_NAME.to_string(), Arc::new(FtsIndexMethod));
        }
        let syms = &self.builtin_syms as *const RwLock<SymbolTable>;
        // Pass the mutex pointer and the appropriate handler
        let schema_mutex_ptr =
            &*self.schema as *const Mutex<Arc<Schema>> as *mut Mutex<Arc<Schema>>;
        let ctx = Box::into_raw(Box::new(ExtensionCtx {
            syms,
            schema: schema_mutex_ptr as *mut c_void,
            prepare_context_generation: std::ptr::null(),
            // Built-ins are compiled together with the host.
            abi_version: TURSO_EXT_API_VERSION,
        }));
        #[allow(unused)]
        let mut ext_api = ExtensionApi {
            ctx: ctx as *mut c_void,
            register_scalar_function: register_scalar_function_with_options,
            register_aggregate_function,
            unregister_function,
            register_vtab_module,
            #[cfg(feature = "fs")]
            vfs_interface: turso_ext::VfsInterface {
                register_vfs: dynamic::register_vfs,
                builtin_vfs: std::ptr::null_mut(),
                builtin_vfs_count: 0,
            },
            api_version: TURSO_EXT_API_VERSION,
            register_window_function,
        };

        #[cfg(feature = "uuid")]
        crate::uuid::register_extension(&mut ext_api);
        #[cfg(feature = "series")]
        crate::series::register_extension(&mut ext_api);
        #[cfg(feature = "time")]
        crate::time::register_extension(&mut ext_api);
        #[cfg(feature = "percentile")]
        crate::percentile::register_extension(&mut ext_api);
        crate::regexp::register_extension(&mut ext_api);
        #[cfg(feature = "fs")]
        {
            let vfslist = add_builtin_vfs_extensions(Some(ext_api)).map_err(|e| e.to_string())?;
            for (name, vfs) in vfslist {
                add_vfs_module(name, vfs);
            }
        }
        let _ = unsafe { Box::from_raw(ctx) };
        Ok(())
    }
}

impl Connection {
    /// Register statically linked functions or virtual tables against this
    /// connection using the generic extension API.
    pub fn register_static_extension<F>(&self, register: F)
    where
        F: FnOnce(&mut ExtensionApi),
    {
        unsafe {
            let mut ext_api = self._build_turso_ext();
            register(&mut ext_api);
            self._free_extension_ctx(ext_api);
        }
    }

    /// Build the connection's extension api context for manually registering an extension.
    /// you probably want to use `Connection::load_extension(path)`.
    ///
    /// # Safety
    /// Only to be used when registering a staticly linked extension manually.
    /// You should only ever call this method on your applications startup,
    /// The caller is responsible for calling `_free_extension_ctx` after registering the
    /// extension.
    ///
    /// usage:
    /// ```ignore
    /// let ext_api = conn._build_turso_ext();
    /// unsafe {
    ///     my_extension::register_extension(&mut ext_api);
    ///     conn._free_extension_ctx(ext_api);
    /// }
    ///```
    pub unsafe fn _build_turso_ext(&self) -> ExtensionApi {
        // Statically linked extensions are compiled with the host.
        unsafe { self.build_turso_ext_for_abi(TURSO_EXT_API_VERSION) }
    }

    /// Same, for an extension that reported its own ABI version.
    ///
    /// # Safety
    /// Same as [`Connection::_build_turso_ext`].
    pub(crate) unsafe fn build_turso_ext_for_abi(&self, abi_version: u32) -> ExtensionApi {
        let schema_mutex_ptr =
            &*self.db.schema as *const Mutex<Arc<Schema>> as *mut Mutex<Arc<Schema>>;
        let ctx = ExtensionCtx {
            syms: &self.syms as *const RwLock<SymbolTable>,
            schema: schema_mutex_ptr as *mut c_void,
            prepare_context_generation: &self.prepare_context_generation as *const _,
            abi_version,
        };
        let ctx = Box::into_raw(Box::new(ctx)) as *mut c_void;
        ExtensionApi {
            ctx,
            register_scalar_function: register_scalar_function_with_options,
            register_aggregate_function,
            unregister_function,
            register_vtab_module,
            #[cfg(feature = "fs")]
            vfs_interface: turso_ext::VfsInterface {
                register_vfs: dynamic::register_vfs,
                builtin_vfs: std::ptr::null_mut(),
                builtin_vfs_count: 0,
            },
            api_version: TURSO_EXT_API_VERSION,
            register_window_function,
        }
    }

    /// Free the connection's extension libary context after registering an extension manually.
    /// # Safety
    /// Only to be used if you have previously called Connection::build_turso_ext
    pub unsafe fn _free_extension_ctx(&self, api: ExtensionApi) {
        if api.ctx.is_null() {
            return;
        }
        let _ = unsafe { Box::from_raw(api.ctx as *mut ExtensionCtx) };
    }
}
