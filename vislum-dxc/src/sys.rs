use std::{
    marker::{PhantomData, PhantomPinned},
    ptr::NonNull,
};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DxcShimStatus {
    Ok = 0,
    OpenLibraryError = 1,
    GetCreateInstance2SymbolError = 2,
    GetDxcCompilerInstanceError = 3,
    GetDxcUtilsInstanceError = 4,
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
pub struct DxcShimCompilationResult {
    _data: (),
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

pub type DxcShimUserCallback = Option<
    unsafe extern "C" fn(
        filename: *const std::ffi::c_char,
        user_data: *mut std::ffi::c_void,
    ) -> *const std::ffi::c_char,
>;

unsafe extern "C" {
    pub unsafe fn dxc_loader_open(loader: *mut *mut DxcShimLoader) -> DxcShimStatus;
    pub unsafe fn dxc_loader_close(loader: *mut DxcShimLoader);
    pub unsafe fn dxc_create_compiler(
        loader: *mut DxcShimLoader,
        compiler: *mut *mut DxcShimCompiler,
    ) -> DxcShimStatus;
    pub unsafe fn dxc_compiler_release(compiler: *mut DxcShimCompiler);
    pub unsafe fn dxc_compile(
        compiler: *mut DxcShimCompiler,
        data: *const std::ffi::c_char,
        user_callback: DxcShimUserCallback,
        user_data: *mut std::ffi::c_void,
    ) -> *mut DxcShimCompilationResult;
    pub unsafe fn dxc_compilation_result_is_successful(
        result: *mut DxcShimCompilationResult,
    ) -> bool;
    pub unsafe fn dxc_compilation_result_get_error_message(
        result: *mut DxcShimCompilationResult,
    ) -> *const std::ffi::c_char;
    pub unsafe fn dxc_compilation_result_get_bytecode(
        result: *mut DxcShimCompilationResult,
        bytecode: *mut *mut std::ffi::c_void,
        size: *mut usize,
    );
    pub unsafe fn dxc_compilation_result_free(result: *mut DxcShimCompilationResult);
}
