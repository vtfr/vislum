#pragma once

#include <dxc/dxcapi.h>
#include <string>

inline std::string extract_utf8_error_message(CComPtr<IDxcBlobEncoding> errorMessage) {
    BOOL known;
    UINT32 codePage;
    if (FAILED(errorMessage->GetEncoding(&known, &codePage))) {
      return "Unknown error. Failed to retrieve encoding";
    }

    auto buffer_ptr = errorMessage->GetBufferPointer();
    auto buffer_size = errorMessage->GetBufferSize();

    // On Linux, we can always assume UTF-8.
    auto bytes = reinterpret_cast<const char*>(buffer_ptr);
    auto len = buffer_size;

    return std::string(bytes, len);
}