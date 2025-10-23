use std::marker::{PhantomData, PhantomPinned};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DxcShimStatus {
  Ok = 0,
  FailedToOpenLibrary = 1,
  FailedToCreateInstance = 2,
  FailedToCreateCompiler = 3,
  FailedToCreateUtils = 4,
  FailedToCompileUnknown = 5,
  FailedToCompile = 6,
}

#[repr(C)]
pub struct DxcShimLoader {
    _data: (),
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

#[repr(C)]
pub struct DxcShimCompiler {
    _data: (),
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

#[repr(C)]
pub struct DxcShimResult {
    _data: (),
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}


unsafe extern "C" {
    pub unsafe fn dxc_shim_loader_open(loader: *mut *mut DxcShimLoader) -> DxcShimStatus;
    pub unsafe fn dxc_shim_loader_close(loader: *mut DxcShimLoader);
    pub unsafe fn dxc_shim_create_compiler(loader: *mut DxcShimLoader, compiler: *mut *mut DxcShimCompiler) -> DxcShimStatus;
    pub unsafe fn dxc_shim_compile(compiler: *mut DxcShimCompiler, data: *const std::ffi::c_char) -> *mut DxcShimResult;
    pub unsafe fn dxc_shim_compilation_result_is_successful(result: *mut DxcShimResult) -> bool;
    pub unsafe fn dxc_shim_compilation_result_get_error_message(result: *mut DxcShimResult) -> *const std::ffi::c_char;
    pub unsafe fn dxc_shim_compilation_result_get_bytecode(result: *mut DxcShimResult, bytecode: *mut *mut std::ffi::c_void, size: *mut usize);
    pub unsafe fn dxc_shim_compilation_result_free(result: *mut DxcShimResult);
}