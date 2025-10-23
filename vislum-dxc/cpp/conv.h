#pragma once

#include <codecvt>
#include <locale>
#include <string>

// Converts a UTF-8 string to a UTF-16 string.
inline std::wstring utf8_to_utf16(const char* s) {
    std::wstring_convert<std::codecvt_utf8_utf16<wchar_t>> conv;
    return conv.from_bytes(s);
}

inline std::string utf16_to_utf8(const wchar_t* s) {
    std::wstring_convert<std::codecvt_utf8_utf16<wchar_t>> conv;
    return conv.to_bytes(s);
}