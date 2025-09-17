use regex::Regex;
use thiserror::Error;
use std::sync::LazyLock;

// Static regex patterns compiled once
static INCLUDE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"#include\s+"([^"]+)""#).unwrap()
});

static IFDEF_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"#ifdef\s+([A-Z_]+)").unwrap()
});


/// Attempts to parse an include directive from the line.
///
/// Returns:
/// - `Ok(Some(path))`: The line is a valid include directive.
/// - `Ok(None)`: The line is not an include directive.
/// - `Err(InvalidIncludeDirectiveError)`: The line is an invalid include directive.
fn parse_include(line: &str) -> Option<&str> {
    match INCLUDE_REGEX.captures(line) {
        Some(caps) => Some(caps.get(1).unwrap().as_str()),
        None => None
    }
}

/// Attempts to parse an "#ifdef" directive from the line.
///
/// Returns:
/// - `Ok(Some(identifier))`: The line is a valid "#ifdef" directive.
/// - `Ok(None)`: The line is not an "#ifdef" directive.
/// - `Err(InvalidIfDefDirectiveError)`: The line is an invalid "#ifdef" directive.
fn parse_ifdef(line: &str) -> Option<&str> {
    match IFDEF_REGEX.captures(line) {
        Some(caps) => Some(caps.get(1).unwrap().as_str()),
        None => None
    }
}

/// Checks if the line is a "#endif" directive.
fn is_endif(line: &str) -> bool {
    line.trim() == "#endif"
}

/// Checks if the line is a "#else" directive.
fn is_else(line: &str) -> bool {
    line.trim() == "#else"
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Directive<'a> {
    Include(&'a str),
    Ifdef(&'a str),
    Else,
    Endif,
    Raw(&'a str),
}

impl<'a> Directive<'a> {
    pub fn parse(line: &'a str) -> Self {
        if is_endif(line) {
            return Directive::Endif;
        }

        if is_else(line) {
            return Directive::Else;
        }

        match parse_include(line) {
            Some(include_path) => return Directive::Include(include_path),
            None => {},
        }

        match parse_ifdef(line) {
            Some(identifier) => return Directive::Ifdef(identifier),
            None => {},
        }

        Directive::Raw(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_include() {
        let result = parse_include(r#"#include "test.wgsl""#);
        assert_eq!(result, Some("test.wgsl"));
    }

    #[test]
    fn test_not_include() {
        let result = parse_include("some other line");
        assert_eq!(result, None);
    }

    #[test]
    fn test_invalid_include_no_quotes() {
        let result = parse_include("#include test.wgsl");
        assert_eq!(result, None);
    }

    #[test]
    fn test_invalid_include_missing_end_quote() {
        let result = parse_include(r#"#include "test.wgsl"#);
        assert_eq!(result, None);
    }

    #[test]
    fn test_invalid_include_missing_start_quote() {
        let result = parse_include(r#"#include test.wgsl""#);
        assert_eq!(result, None);
    }

    // Tests for maybe_parse_ifdef
    #[test]
    fn test_valid_ifdef() {
        let result = parse_ifdef("#ifdef INSTANCED");
        assert_eq!(result, Some("INSTANCED"));
    }

    #[test]
    fn test_not_ifdef() {
        let result = parse_ifdef("some other line");
        assert_eq!(result, None);
    }

    #[test]
    fn test_invalid_ifdef_lowercase() {
        let result = parse_ifdef("#ifdef debug");
        assert_eq!(result, None);
    }

    // Tests for is_endif
    #[test]
    fn test_valid_endif() {
        assert!(is_endif("#endif"));
    }

    #[test]
    fn test_endif_with_whitespace() {
        assert!(is_endif("  #endif  "));
    }

    #[test]
    fn test_not_endif() {
        assert!(!is_endif("#ifdef DEBUG"));
    }
}