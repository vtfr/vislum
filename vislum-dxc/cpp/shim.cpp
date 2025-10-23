#include "wrapper.h"

extern "C" {
  // Opens the loader.
  DxcShimStatus dxc_loader_open(DxcShimLoader **loader);
  
  // Closes the loader.
  void dxc_loader_close(DxcShimLoader *loader);

  // Creates a compiler.
  DxcShimStatus dxc_create_compiler(DxcShimLoader *loader, DxcShimCompiler **compiler);

  // Releases the compiler.
  void dxc_compiler_release(DxcShimCompiler *compiler);
  
  // Compiles a shader.
  DxcShimCompilationResult* dxc_compile(DxcShimCompiler *compiler, const char *data, DxcShimUserCallback userCallback, void* userData);
  
  // Returns whether a compilation was successful.
  bool dxc_compilation_result_is_successful(DxcShimCompilationResult *result);
  
  // Returns the error message of a compilation.
  //
  // Returns NULL if the compilation was successful.
  char* dxc_compilation_result_get_error_message(DxcShimCompilationResult *result);

  // Returns the bytecode of a compilation.
  //
  // Size is the size of the bytecode in bytes. If the compilation failed, the size will be set to 0.
  void dxc_compilation_result_get_bytecode(DxcShimCompilationResult *result, void **bytecode, size_t *size);

  // Frees the result.
  void dxc_compilation_result_free(DxcShimCompilationResult *result);
} // extern "C"

DxcShimStatus dxc_loader_open(DxcShimLoader **loader) {
  try {
    *loader = new DxcShimLoader();
    return DxcShimStatus::Ok;
  } catch (const DxcShimException &e) {
    return e.getStatus();
  }
}

void dxc_loader_close(DxcShimLoader *loader) {
  delete loader;
}

DxcShimStatus dxc_create_compiler(DxcShimLoader *loader, DxcShimCompiler **compiler) {
  try {
    *compiler = new DxcShimCompiler(*loader);
    return DxcShimStatus::Ok;
  } catch (const DxcShimException &e) {
    return e.getStatus();
  }
}

void dxc_compiler_release(DxcShimCompiler *compiler) {
  delete compiler;
}

DxcShimCompilationResult* dxc_compile(DxcShimCompiler *compiler, const char *data, DxcShimUserCallback userCallback, void* userData) {
  return compiler->compile(data, userCallback, userData);
}

bool dxc_compilation_result_is_successful(DxcShimCompilationResult *result) {
    return result->isSuccessful();
}

char* dxc_compilation_result_get_error_message(DxcShimCompilationResult *result) {
    return (char*)result->getErrorMessage().c_str();
}

void dxc_compilation_result_get_bytecode(DxcShimCompilationResult *result, void **bytecode, size_t *size) {
    *bytecode = (void*)result->getBytecode().data();
    *size = result->getBytecode().size();
}

void dxc_compilation_result_free(DxcShimCompilationResult *result) {
    delete result;
}
