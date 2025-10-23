#include <dlfcn.h>
#include <dxc/dxcapi.h>
#include <stdlib.h>
#include <string.h>

#include "conv.h"

enum Status {
  DXC_SHIM_STATUS_OK = 0,
  DXC_SHIM_STATUS_FAILED_TO_OPEN_LIBRARY = 1,
  DXC_SHIM_STATUS_FAILED_TO_CREATE_INSTANCE = 2,
  DXC_SHIM_STATUS_FAILED_TO_CREATE_COMPILER = 3,
  DXC_SHIM_STATUS_FAILED_TO_CREATE_UTILS = 4,
  DXC_SHIM_STATUS_FAILED_TO_COMPILE_UNKNOWN = 5,
  DXC_SHIM_STATUS_FAILED_TO_COMPILE = 6,
};

struct Loader {
  void *handle;
  DxcCreateInstance2Proc createInstance2;
};

struct Compiler {
  CComPtr<IDxcCompiler3> compiler;
  CComPtr<IDxcUtils> utils;
};

struct CompilationResult {
  bool successful;
  std::string error_message;
  CComPtr<IDxcBlob> bytecode;

  inline static CompilationResult* success(CComPtr<IDxcBlob> bytecode) {
    return new CompilationResult(true, std::string(), bytecode);
  }

  inline static CompilationResult* failure(CComPtr<IDxcBlobEncoding> errorBlob) {
    auto error_message = extract_utf8_error_message(errorBlob);

    return new CompilationResult(false, std::move(error_message), CComPtr<IDxcBlob>());
  }

private:
    CompilationResult(
      bool successful, 
      std::string&& errorMessage, 
      CComPtr<IDxcBlob> bytecode) 
      : successful(successful)
      , error_message(std::move(errorMessage))
      , bytecode(bytecode) {}
};

class IncludeHandler : public IDxcIncludeHandler {
public:
  virtual ~IncludeHandler() = default;

  // IUnknown methods
  HRESULT STDMETHODCALLTYPE QueryInterface(REFIID riid, void **ppvObject) override {
    if (riid == __uuidof(IUnknown) || riid == __uuidof(IDxcIncludeHandler)) {
      *ppvObject = static_cast<IDxcIncludeHandler*>(this);
      AddRef();
      return S_OK;
    }
    *ppvObject = nullptr;
    return E_NOINTERFACE;
  }

  ULONG STDMETHODCALLTYPE AddRef() override {
    return __sync_add_and_fetch(&m_refCount, 1);
  }

  ULONG STDMETHODCALLTYPE Release() override {
    ULONG refCount = __sync_sub_and_fetch(&m_refCount, 1);
    if (refCount == 0) {
      delete this;
    }
    return refCount;
  }

  // IDxcIncludeHandler methods
  HRESULT STDMETHODCALLTYPE LoadSource(LPCWSTR /*pFilename*/, IDxcBlob ** /*ppIncludeSource*/) override {
    // For now, return E_NOTIMPL to indicate no custom include handling
    // This can be extended later to handle custom include resolution
    return E_NOTIMPL;
  }

private:
  LONG m_refCount = 1;
};

extern "C" {
  // Opens the loader.
  Status dxc_shim_loader_open(Loader **loader);
  
  // Closes the loader.
  void dxc_shim_loader_close(Loader *loader);

  // Creates a compiler.
  Status dxc_shim_create_compiler(Loader *loader, Compiler **compiler);
  
  // Compiles a shader.
  CompilationResult* dxc_shim_compile(Compiler *compiler, const char *data);
  
  // Returns whether a compilation was successful.
  bool dxc_shim_compilation_result_is_successful(CompilationResult *result);
  
  // Returns the error message of a compilation.
  //
  // Returns NULL if the compilation was successful.
  char* dxc_shim_compilation_result_get_error_message(CompilationResult *result);

  // Returns the bytecode of a compilation.
  //
  // Size is the size of the bytecode in bytes. If the compilation failed, the size will be set to 0.
  void dxc_shim_compilation_result_get_bytecode(CompilationResult *result, void **bytecode, size_t *size);

  // Frees the result.
  void dxc_shim_compilation_result_free(CompilationResult *result);
} // extern "C"

Status dxc_shim_loader_open(Loader **loader) {
  void *handle = dlopen("libdxcompiler.so", RTLD_NOW);
  if (handle == NULL) {
    return DXC_SHIM_STATUS_FAILED_TO_OPEN_LIBRARY;
  }

  *loader = new Loader();
  (*loader)->handle = handle;
  (*loader)->createInstance2 = (DxcCreateInstance2Proc)dlsym(handle, "DxcCreateInstance2");

  return DXC_SHIM_STATUS_OK;
}

void dxc_shim_loader_close(Loader *loader) {
  dlclose(loader->handle);
  delete loader;
}

Status dxc_shim_create_compiler(Loader *loader, Compiler **compiler) {
    *compiler = new Compiler();

    HRESULT hr;
    hr = loader->createInstance2(NULL, CLSID_DxcCompiler, IID_PPV_ARGS(&(*compiler)->compiler));
    if (FAILED(hr)) {
        delete *compiler;
        return DXC_SHIM_STATUS_FAILED_TO_CREATE_COMPILER;
    }

    hr = loader->createInstance2(NULL, CLSID_DxcUtils, IID_PPV_ARGS(&(*compiler)->utils));
    if (FAILED(hr)) {
        delete *compiler;
        return DXC_SHIM_STATUS_FAILED_TO_CREATE_UTILS;
    }

    return DXC_SHIM_STATUS_OK;
}

CompilationResult* dxc_shim_compile(
  Compiler *compiler, 
  const char *data
) {
    DxcBuffer buffer = {
        .Ptr = data,
        .Size = strlen(data),
        .Encoding = CP_UTF8,
    };

    HRESULT hr;
    CComPtr<IDxcResult> dxcResult;

    CComPtr<IDxcIncludeHandler> includeHandler = new IncludeHandler();

    LPCWSTR args[] = {
      L"-spirv",
      L"-fspv-target-env=vulkan1.3",
      L"-E", L"main",
      L"-T", L"vs_6_5"
    };

    UINT32 argCount = sizeof(args) / sizeof(args[0]);

    hr = compiler->compiler->Compile(&buffer, args, argCount, includeHandler, IID_PPV_ARGS(&dxcResult));
    if (FAILED(hr)) {
        return CompilationResult::failure(nullptr);
    }

    dxcResult->GetStatus(&hr);
    if (FAILED(hr)) {
      CComPtr<IDxcBlobEncoding> error_blob;
      dxcResult->GetErrorBuffer(&error_blob);

      return CompilationResult::failure(error_blob);
    }

    CComPtr<IDxcBlob> bytecode;
    dxcResult->GetResult(&bytecode);

    return CompilationResult::success(bytecode);
}

bool dxc_shim_compilation_result_is_successful(CompilationResult *result) {
    return result->successful;
}

char* dxc_shim_compilation_result_get_error_message(CompilationResult *result) {
    return (char*)result->error_message.c_str();
}

void dxc_shim_compilation_result_get_bytecode(CompilationResult *result, void **bytecode, size_t *size) {
    *bytecode = result->bytecode->GetBufferPointer();
    *size = result->bytecode->GetBufferSize();
}

void dxc_shim_compilation_result_free(CompilationResult *result) {
    delete result;
}
