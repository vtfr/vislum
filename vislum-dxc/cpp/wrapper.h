#pragma once

#define __EMULATE_UUID

#include "conv.h"
#include <cstdint>
#include <dxc/WinAdapter.h>
#include <dxc/dxcapi.h>
#include <exception>
#include <dlfcn.h>
#include <vector>
#include <string>
#include <atomic>

enum class DxcShimStatus: uint8_t {
  Ok = 0,
  OpenLibraryError = 2,
  GetCreateInstance2SymbolError = 2,
  GetDxcCompilerInstanceError = 3,
  GetDxcUtilsInstanceError = 4,
};

class DxcShimException : public std::exception {
public:
  DxcShimException(DxcShimStatus status) : m_status(status) {}

  inline DxcShimStatus getStatus() const {
    return m_status;
  }

private:
  DxcShimStatus m_status;
};

class DxcShimLoader {
public:
  explicit DxcShimLoader() {
    m_handle = dlopen("libdxcompiler.so", RTLD_NOW);
    if (m_handle == NULL) {
      throw DxcShimException(DxcShimStatus::OpenLibraryError);
    }

    m_createInstance2 = (DxcCreateInstance2Proc)dlsym(m_handle, "DxcCreateInstance2");
    if (m_createInstance2 == NULL) {
      dlclose(m_handle);
      throw DxcShimException(DxcShimStatus::GetCreateInstance2SymbolError);
    }
  }

  inline DxcCreateInstance2Proc getCreateInstance2Proc() const {
    return m_createInstance2;
  }

  ~DxcShimLoader() {
    dlclose(m_handle);
  }

private:
  void *m_handle;
  DxcCreateInstance2Proc m_createInstance2;
};

class DxcShimCompilationResult {
public:
  inline bool isSuccessful() const {
    return m_isSuccessful;
  }

  inline std::string const& getErrorMessage() const {
    return m_errorMessage;
  }

  inline std::vector<uint8_t> const& getBytecode() const {
    return m_bytecode;
  }

  inline static DxcShimCompilationResult* success(std::vector<uint8_t> bytecode) {
    return new DxcShimCompilationResult(true, std::string(), std::move(bytecode));
  }

  inline static DxcShimCompilationResult* failure(std::string errorMessage) {
    return new DxcShimCompilationResult(false, std::move(errorMessage), std::vector<uint8_t>());
  }

private:
  inline explicit DxcShimCompilationResult(
    bool isSuccessful, 
    std::string&& errorMessage, 
    std::vector<uint8_t>&& bytecode) 
        : m_isSuccessful(isSuccessful)
        , m_errorMessage(std::move(errorMessage))
        , m_bytecode(std::move(bytecode)) { }

  bool m_isSuccessful;
  std::string m_errorMessage;
  std::vector<uint8_t> m_bytecode;
};

typedef char* (*DxcShimUserCallback)(const char* filename, void* userData);

class DxcShimIncludeHandler : public IDxcIncludeHandler {
public:  
  inline DxcShimIncludeHandler(CComPtr<IDxcUtils>& utils, DxcShimUserCallback userCallback, void* userData) 
    : m_utils(utils)
    , m_userCallback(userCallback)
    , m_userData(userData) {}

  // IUnknown methods
  HRESULT STDMETHODCALLTYPE QueryInterface(REFIID riid, void **ppvObject) override {
    if (riid == __uuidof(IUnknown) || riid == __uuidof(IDxcIncludeHandler)) {
      *ppvObject = this;
      AddRef();
      return S_OK;
    }
    *ppvObject = nullptr;
    return E_NOINTERFACE;
  }

  ULONG STDMETHODCALLTYPE AddRef() override {
    return m_refCount++;
  }

  ULONG STDMETHODCALLTYPE Release() override {
    ULONG refCount = m_refCount--;
    if (refCount == 0) {
      delete this;
    }
    return refCount;
  }

  // IDxcIncludeHandler methods
  HRESULT STDMETHODCALLTYPE LoadSource(LPCWSTR wideFilename, IDxcBlob** ppIncludeSource) override {
    std::string filename = utf16_to_utf8(wideFilename);

    char* source = m_userCallback(filename.c_str(), m_userData);
    if (source == nullptr) {
      return E_FAIL;
    }

    CComPtr<IDxcBlobEncoding> sourceBlob;
    HRESULT hr = m_utils->CreateBlob(source, strlen(source), CP_UTF8, &sourceBlob);
    if (FAILED(hr)) {
      return hr;
    }

    *ppIncludeSource = sourceBlob.Detach();
    return S_OK;
  }

private:
  CComPtr<IDxcUtils>& m_utils;
  DxcShimUserCallback m_userCallback;
  void* m_userData;
  std::atomic<ULONG> m_refCount {1u};
};

class DxcShimCompiler {
public:
  DxcShimCompiler(DxcShimLoader const&loader) {
    HRESULT hr;
    hr = loader.getCreateInstance2Proc()(NULL, CLSID_DxcCompiler, IID_PPV_ARGS(&m_compiler));
    if (FAILED(hr)) {
      throw DxcShimException(DxcShimStatus::GetDxcCompilerInstanceError);
    }

    hr = loader.getCreateInstance2Proc()(NULL, CLSID_DxcUtils, IID_PPV_ARGS(&m_utils));
    if (FAILED(hr)) {
      throw DxcShimException(DxcShimStatus::GetDxcUtilsInstanceError);
    }
  }

  inline DxcShimCompilationResult* compile(const char* data, DxcShimUserCallback userCallback, void* userData) {
    CComPtr<IDxcResult> dxcResult;

    DxcBuffer buffer = {
      .Ptr = data,
      .Size = strlen(data),
      .Encoding = CP_UTF8,
    };

    LPCWSTR args[] = {
      L"-spirv",
      L"-fspv-target-env=vulkan1.3",
      L"-E", L"main",
      L"-T", L"vs_6_5"
    };

    UINT32 argCount = sizeof(args) / sizeof(args[0]);

    CComPtr<IDxcIncludeHandler> includeHandler; 
    if (userCallback != nullptr) {
      includeHandler = new DxcShimIncludeHandler(m_utils, userCallback, userData);
    }

    m_compiler->Compile(&buffer, args, argCount, includeHandler, IID_PPV_ARGS(&dxcResult));
    
    HRESULT hr;
    dxcResult->GetStatus(&hr);
    if (FAILED(hr)) {
        CComPtr<IDxcBlobEncoding> errorBlob;
        dxcResult->GetErrorBuffer(&errorBlob);

        BOOL known;
        UINT32 codePage;
        errorBlob->GetEncoding(&known, &codePage);

        // If the encoding is UTF-8, return the error message as a UTF-8 string.
        if (codePage == CP_UTF8) {
          std::string message = static_cast<LPSTR>(errorBlob->GetBufferPointer());
          return DxcShimCompilationResult::failure(message);
        } 
          
        // Assume UTF-16 if the encoding.
        std::string message = utf16_to_utf8((LPWSTR)errorBlob->GetBufferPointer());
        return DxcShimCompilationResult::failure(std::move(message));
    }

    CComPtr<IDxcBlob> bytecode;
    dxcResult->GetResult(&bytecode);

    auto ptr = static_cast<uint8_t*>(bytecode->GetBufferPointer());
    auto size = static_cast<size_t>(bytecode->GetBufferSize());

    std::vector<uint8_t> bytecodeData(ptr, ptr + size);
    return DxcShimCompilationResult::success(std::move(bytecodeData));
  }

private:
  CComPtr<IDxcCompiler3> m_compiler;
  CComPtr<IDxcUtils> m_utils;
};
