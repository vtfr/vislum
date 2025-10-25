use std::{ffi::{CStr, CString}, mem::MaybeUninit, sync::Arc};

pub mod sys;

#[derive(thiserror::Error, Debug)]
pub enum DxcLoaderError {
    #[error("failed to open library")]
    OpenLibraryError,
    #[error("failed to get DxcCreateInstance2 symbol")]
    GetCreateInstance2SymbolError,
}

pub struct DxcLoader {
    inner: *mut sys::DxcShimLoader,
}

impl Drop for DxcLoader {
    fn drop(&mut self) {
        unsafe { sys::dxc_loader_close(self.inner) };
    }
}

impl DxcLoader {
    pub fn new() -> Result<Arc<Self>, DxcLoaderError> {
        let mut inner = MaybeUninit::<*mut sys::DxcShimLoader>::uninit();
        let status = unsafe { sys::dxc_loader_open(inner.as_mut_ptr()) };

        match status {
            sys::DxcShimStatus::Ok => {
                let inner = unsafe { inner.assume_init() };
                Ok(Arc::new(Self { inner }))
            }
            sys::DxcShimStatus::OpenLibraryError => {
                Err(DxcLoaderError::OpenLibraryError)
            }
            sys::DxcShimStatus::GetCreateInstance2SymbolError => {
                Err(DxcLoaderError::GetCreateInstance2SymbolError)
            }
            _ => unreachable!(),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DxcCompilerCreationError {
    #[error("failed to get DxcCompiler3 instance")]
    GetDxcCompilerInstanceError,
    #[error("failed to get DxcUtils instance")]
    GetDxcUtilsInstanceError,
}

impl Drop for DxcCompiler {
    fn drop(&mut self) {
        unsafe { sys::dxc_compiler_release(self.inner) };
    }
}

pub trait DxcIncludeHandler {
    fn load_source(&self, filename: &str) -> Option<String>;
}

/// A string interner for include handler strings.
/// 
/// This is used to prevent shared heap ownership between the C shim and the Rust code. The 
/// [`dxc_include_handler_trampoline`] function uses this to ensure that the string is not freed too early.
struct DxcIncludeHandlerUserData<'a> {
    /// The user-provided include handler.
    include_handler: &'a dyn DxcIncludeHandler,
    
    /// The strings that have been interned.
    strings: Vec<CString>,
}

#[derive(thiserror::Error, Debug)]
#[error("compilation failed: {0}")]
pub struct DxcCompilationError(String);

pub struct DxcCompiler {
    _loader: Arc<DxcLoader>,
    inner: *mut sys::DxcShimCompiler,
}

// SAFETY: DxcCompiler is thread-safe.
// This is also a lie. Gotta fix this...
unsafe impl Send for DxcCompiler {}
unsafe impl Sync for DxcCompiler {}

impl DxcCompiler {
    pub fn new(loader: Arc<DxcLoader>) -> Result<Arc<Self>, DxcCompilerCreationError> {
        let mut inner = MaybeUninit::<*mut sys::DxcShimCompiler>::uninit();
        let status = unsafe { sys::dxc_create_compiler(loader.inner, inner.as_mut_ptr()) };

        match status {
            sys::DxcShimStatus::Ok => {
                let inner = unsafe { inner.assume_init() };
                Ok(Arc::new(Self { _loader: loader, inner }))
            }
            sys::DxcShimStatus::GetDxcCompilerInstanceError => {
                Err(DxcCompilerCreationError::GetDxcCompilerInstanceError)
            }
            sys::DxcShimStatus::GetDxcUtilsInstanceError => {
                Err(DxcCompilerCreationError::GetDxcUtilsInstanceError)
            }
            _ => unreachable!(),
        }
    }

    pub fn compile<'a>(&self, data: &str, include_handler: &'a dyn DxcIncludeHandler) -> Result<Vec<u8>, DxcCompilationError> {
        let data_cstr = CString::new(data).unwrap();

        let user_data = DxcIncludeHandlerUserData {
            include_handler,
            strings: Vec::new(),
        };

        let raw_result = unsafe { 
            sys::dxc_compile(
                self.inner, 
                data_cstr.as_ptr() as *const _, 
                Some(dxc_include_handler_trampoline as unsafe extern "C" fn(*const std::ffi::c_char, *mut std::ffi::c_void) -> *const std::ffi::c_char), 
                &user_data as *const _ as *mut std::ffi::c_void)
        };

        let result = if unsafe { sys::dxc_compilation_result_is_successful(raw_result) } {
            let mut bytecode = MaybeUninit::<*mut std::ffi::c_void>::uninit();
            let mut size = MaybeUninit::<usize>::uninit();

            unsafe { sys::dxc_compilation_result_get_bytecode(raw_result, bytecode.as_mut_ptr(), size.as_mut_ptr()) };
            let size = unsafe { size.assume_init() };
            let bytecode = unsafe { bytecode.assume_init() };

            let bytecode = unsafe { std::slice::from_raw_parts(bytecode as *const u8, size) };
            Ok(bytecode.to_vec())
        } else {
            let error_message_c = unsafe { sys::dxc_compilation_result_get_error_message(raw_result) };
            let error_message = unsafe { CStr::from_ptr(error_message_c) }.to_string_lossy().into_owned();
            Err(DxcCompilationError(error_message))
        };

        unsafe { sys::dxc_compilation_result_free(raw_result) };

        result
    }
}

unsafe extern "C" fn dxc_include_handler_trampoline(filename_cptr: *const std::ffi::c_char, user_data: *mut std::ffi::c_void) -> *const std::ffi::c_char {
    let filename = unsafe { CStr::from_ptr(filename_cptr) }.to_str().unwrap();

    let user_data = unsafe { &mut *(user_data as *mut DxcIncludeHandlerUserData<'_>) };
    
    match user_data.include_handler.load_source(filename) {
        Some(source) => {
            let source_cstr = CString::new(source).unwrap();
            let source_cstr_ptr = source_cstr.as_ptr();
            
            // Intern the string. The pointer will still be valid after that.
            //
            // All pointers will be dropped after [`DxcIncludeHandlerUserData`] is dropped after [`DxcCompiler::compile`] returns.
            user_data.strings.push(source_cstr);

            source_cstr_ptr as *const std::ffi::c_char
        },
        None => std::ptr::null(),
    }
}