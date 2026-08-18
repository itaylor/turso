mod functions;
mod types;
#[cfg(feature = "vfs")]
mod vfs_modules;
mod vtabs;
pub use functions::{
    AggCtx, AggFunc, ContextDestructor, FinalizeFunction, InitAggFunction, ScalarFunc,
    ScalarFunction, StepFunction, ValueDestructor, WindowInverseFunction, WindowValueFunction,
};
use functions::{RegisterAggFn, RegisterScalarFn, RegisterWindowFn, UnregisterFunctionFn};
use std::os::raw::c_void;
#[cfg(feature = "vfs")]
pub use turso_macros::VfsDerive;
pub use turso_macros::{
    register_extension, scalar, AggregateDerive, ScalarDerive, VTabModuleDerive,
};
pub use types::{ResultCode, StepResult, Value, ValueType, JSON_SUBTYPE};
#[cfg(feature = "vfs")]
pub use vfs_modules::{
    BufferRef, Callback, IOCallback, RegisterVfsFn, SendPtr, VfsExtension, VfsFile, VfsFileImpl,
    VfsImpl, VfsInterface,
};
use vtabs::RegisterModuleFn;
pub use vtabs::{
    Conn, Connection, ConstraintInfo, ConstraintOp, ConstraintUsage, ExtIndexInfo, IndexInfo,
    OrderByInfo, Statement, Stmt, VTabCreateResult, VTabCursor, VTabKind, VTabModule,
    VTabModuleImpl, VTable,
};

pub type ExtResult<T> = std::result::Result<T, ResultCode>;

pub type ExtensionEntryPoint = unsafe extern "C" fn(api: *const ExtensionApi) -> ResultCode;

/// Layout version of [`ExtensionApi`]. The host writes it into
/// [`ExtensionApi::api_version`], and `register_extension!` exports it as
/// `turso_ext_api_version` so the host can reject an extension built against a
/// newer layout than it understands.
///
/// Fields may only be appended below `api_version`, and this constant must be
/// bumped in the same change. An extension must check `api.api_version` before
/// reading a field added after the version it was built against.
///
/// The version does not cover `vfs_interface`, which is behind the `vfs`
/// feature: host and extension must enable the same `turso_ext` features or
/// the offsets after it will not line up.
pub const TURSO_EXT_API_VERSION: u32 = 2;

#[repr(C)]
pub struct ExtensionApi {
    pub ctx: *mut c_void,
    pub register_scalar_function: RegisterScalarFn,
    pub register_aggregate_function: RegisterAggFn,
    pub unregister_function: UnregisterFunctionFn,
    pub register_vtab_module: RegisterModuleFn,
    #[cfg(feature = "vfs")]
    pub vfs_interface: VfsInterface,
    // Append-only below this point; bump `TURSO_EXT_API_VERSION` when you do.
    pub api_version: u32,
    pub register_window_function: RegisterWindowFn,
}

unsafe impl Send for ExtensionApi {}
unsafe impl Send for ExtensionApiRef {}

#[repr(C)]
pub struct ExtensionApiRef {
    pub api: *const ExtensionApi,
}
