use regex::Regex;
use thiserror::Error;
use std::sync::LazyLock;

#[derive(Debug, PartialEq, Eq)]
pub struct InvalidIncludeDirectiveError;

#[derive(Debug, PartialEq, Eq)]
pub struct InvalidIfDefDirectiveError;

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
fn maybe_parse_include(line: &str) -> Result<Option<&str>, InvalidIncludeDirectiveError> {
    if let Some(caps) = INCLUDE_REGEX.captures(line) {
        Ok(Some(caps.get(1).unwrap().as_str()))
    } else if line.trim().starts_with("#include") {
        Err(InvalidIncludeDirectiveError)
    } else {
        Ok(None)
    }
}

/// Attempts to parse an "#ifdef" directive from the line.
///
/// Returns:
/// - `Ok(Some(identifier))`: The line is a valid "#ifdef" directive.
/// - `Ok(None)`: The line is not an "#ifdef" directive.
/// - `Err(InvalidIfDefDirectiveError)`: The line is an invalid "#ifdef" directive.
fn maybe_parse_ifdef(line: &str) -> Result<Option<&str>, InvalidIfDefDirectiveError> {
    if let Some(caps) = IFDEF_REGEX.captures(line) {
        Ok(Some(caps.get(1).unwrap().as_str()))
    } else if line.trim().starts_with("#ifdef") {
        Err(InvalidIfDefDirectiveError)
    } else {
        Ok(None)
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

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DirectiveParseError {
    #[error("invalid #include directive")]
    InvalidIncludeDirective,

    #[error("invalid #ifdef directive")]
    InvalidIfDefDirective,
}

impl<'a> Directive<'a> {
    pub fn parse(line: &'a str) -> Result<Self, DirectiveParseError> {
        if is_endif(line) {
            return Ok(Directive::Endif);
        }

        if is_else(line) {
            return Ok(Directive::Else);
        }

        match maybe_parse_include(line) {
            Ok(Some(include_path)) => return Ok(Directive::Include(include_path)),
            Ok(None) => {},
            Err(_) => return Err(DirectiveParseError::InvalidIncludeDirective),
        }

        match maybe_parse_ifdef(line) {
            Ok(Some(identifier)) => return Ok(Directive::Ifdef(identifier)),
            Ok(None) => {},
            Err(_) => return Err(DirectiveParseError::InvalidIfDefDirective),
        }

        Ok(Directive::Raw(line))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_include() {
        let result = maybe_parse_include(r#"#include "test.wgsl""#);
        assert_eq!(result, Ok(Some("test.wgsl")));
    }

    #[test]
    fn test_not_include() {
        let result = maybe_parse_include("some other line");
        assert_eq!(result, Ok(None));
    }

    #[test]
    fn test_invalid_include_no_quotes() {
        let result = maybe_parse_include("#include test.wgsl");
        assert_eq!(result, Err(InvalidIncludeDirectiveError));
    }

    #[test]
    fn test_invalid_include_missing_end_quote() {
        let result = maybe_parse_include(r#"#include "test.wgsl"#);
        assert_eq!(result, Err(InvalidIncludeDirectiveError));
    }

    #[test]
    fn test_invalid_include_missing_start_quote() {
        let result = maybe_parse_include(r#"#include test.wgsl""#);
        assert_eq!(result, Err(InvalidIncludeDirectiveError));
    }

    // Tests for maybe_parse_ifdef
    #[test]
    fn test_valid_ifdef() {
        let result = maybe_parse_ifdef("#ifdef INSTANCED");
        assert_eq!(result, Ok(Some("INSTANCED")));
    }

    #[test]
    fn test_not_ifdef() {
        let result = maybe_parse_ifdef("some other line");
        assert_eq!(result, Ok(None));
    }

    #[test]
    fn test_invalid_ifdef_lowercase() {
        let result = maybe_parse_ifdef("#ifdef debug");
        assert_eq!(result, Err(InvalidIfDefDirectiveError));
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