// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Minimal key=value / flag argument parser.

#![allow(dead_code)]

use std::collections::HashMap;

/// Parsed argument result.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedArgs {
    /// Named key=value arguments.
    pub kwargs: HashMap<String, String>,
    /// Boolean flags (present = true).
    pub flags: Vec<String>,
    /// Positional arguments.
    pub positional: Vec<String>,
}

/// Create an empty ParsedArgs.
#[allow(dead_code)]
pub fn new_parsed_args() -> ParsedArgs {
    ParsedArgs {
        kwargs: HashMap::new(),
        flags: Vec::new(),
        positional: Vec::new(),
    }
}

/// Parse a slice of argument strings.
/// Recognizes:
///   - `key=value` → kwargs
///   - `--flag` or `-flag` → flags (without dashes as key)
///   - anything else → positional
#[allow(dead_code)]
pub fn parse_args(args: &[&str]) -> ParsedArgs {
    let mut result = new_parsed_args();
    for &arg in args {
        if let Some(eq_pos) = arg.find('=') {
            let key = arg[..eq_pos].trim_start_matches('-').to_string();
            let val = arg[eq_pos + 1..].to_string();
            result.kwargs.insert(key, val);
        } else if let Some(stripped) = arg.strip_prefix("--") {
            result.flags.push(stripped.to_string());
        } else if let Some(stripped) = arg.strip_prefix('-') {
            if !stripped.is_empty() {
                result.flags.push(stripped.to_string());
            } else {
                result.positional.push(arg.to_string());
            }
        } else {
            result.positional.push(arg.to_string());
        }
    }
    result
}

/// Parse from a single space-separated command string.
#[allow(dead_code)]
pub fn parse_args_str(s: &str) -> ParsedArgs {
    let args: Vec<&str> = s.split_whitespace().collect();
    parse_args(&args)
}

/// Get a kwarg value, returning None if not present.
#[allow(dead_code)]
pub fn arg_get<'a>(args: &'a ParsedArgs, key: &str) -> Option<&'a str> {
    args.kwargs.get(key).map(|s| s.as_str())
}

/// Get a kwarg as f64.
#[allow(dead_code)]
pub fn arg_get_f64(args: &ParsedArgs, key: &str) -> Option<f64> {
    arg_get(args, key).and_then(|s| s.parse::<f64>().ok())
}

/// Get a kwarg as i64.
#[allow(dead_code)]
pub fn arg_get_i64(args: &ParsedArgs, key: &str) -> Option<i64> {
    arg_get(args, key).and_then(|s| s.parse::<i64>().ok())
}

/// Check if a flag is present.
#[allow(dead_code)]
pub fn arg_has_flag(args: &ParsedArgs, flag: &str) -> bool {
    args.flags.iter().any(|f| f == flag)
}

/// Return the number of positional arguments.
#[allow(dead_code)]
pub fn arg_positional_count(args: &ParsedArgs) -> usize {
    args.positional.len()
}

/// Get a kwarg with a default value if missing.
#[allow(dead_code)]
pub fn arg_get_or(args: &ParsedArgs, key: &str, default: &str) -> String {
    arg_get(args, key).unwrap_or(default).to_string()
}

/// Return all kwarg keys.
#[allow(dead_code)]
pub fn arg_keys(args: &ParsedArgs) -> Vec<&str> {
    args.kwargs.keys().map(|s| s.as_str()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_kwargs() {
        let args = parse_args(&["key=value", "n=42"]);
        assert_eq!(arg_get(&args, "key"), Some("value"));
        assert_eq!(arg_get(&args, "n"), Some("42"));
    }

    #[test]
    fn parse_flags_double_dash() {
        let args = parse_args(&["--verbose", "--debug"]);
        assert!(arg_has_flag(&args, "verbose"));
        assert!(arg_has_flag(&args, "debug"));
    }

    #[test]
    fn parse_flags_single_dash() {
        let args = parse_args(&["-v", "-q"]);
        assert!(arg_has_flag(&args, "v"));
        assert!(arg_has_flag(&args, "q"));
    }

    #[test]
    fn parse_positional() {
        let args = parse_args(&["file.txt", "output.txt"]);
        assert_eq!(arg_positional_count(&args), 2);
        assert_eq!(args.positional[0], "file.txt");
    }

    #[test]
    fn parse_mixed() {
        let args = parse_args(&["input.txt", "--verbose", "n=10"]);
        assert_eq!(arg_positional_count(&args), 1);
        assert!(arg_has_flag(&args, "verbose"));
        assert_eq!(arg_get_i64(&args, "n"), Some(10));
    }

    #[test]
    fn arg_get_f64_valid() {
        let args = parse_args(&["x=2.5"]);
        let v = arg_get_f64(&args, "x").unwrap();
        assert!((v - 2.5).abs() < 1e-9);
    }

    #[test]
    fn arg_get_or_default() {
        let args = new_parsed_args();
        assert_eq!(arg_get_or(&args, "missing", "default"), "default");
    }

    #[test]
    fn parse_from_str() {
        let args = parse_args_str("--verbose n=5 input.txt");
        assert!(arg_has_flag(&args, "verbose"));
        assert_eq!(arg_get_i64(&args, "n"), Some(5));
    }

    #[test]
    fn empty_args() {
        let args = parse_args(&[]);
        assert!(args.kwargs.is_empty());
        assert!(args.flags.is_empty());
        assert!(args.positional.is_empty());
    }

    #[test]
    fn flag_not_present() {
        let args = parse_args(&["--verbose"]);
        assert!(!arg_has_flag(&args, "quiet"));
    }
}
