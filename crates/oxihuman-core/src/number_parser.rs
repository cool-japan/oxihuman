// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Simple number parsing utilities.

#![allow(dead_code)]

/// Parse a string to f32, returning None on failure.
#[allow(dead_code)]
pub fn parse_f32(s: &str) -> Option<f32> {
    s.trim().parse::<f32>().ok()
}

/// Parse a string to i32, returning None on failure.
#[allow(dead_code)]
pub fn parse_i32(s: &str) -> Option<i32> {
    s.trim().parse::<i32>().ok()
}

/// Parse a string to bool. Accepts "true"/"false" (case-insensitive), "1"/"0".
#[allow(dead_code)]
pub fn parse_bool(s: &str) -> Option<bool> {
    match s.trim().to_lowercase().as_str() {
        "true" | "1" | "yes" => Some(true),
        "false" | "0" | "no" => Some(false),
        _ => None,
    }
}

/// Parse a "x,y,z" formatted string to a [f32; 3] array.
#[allow(dead_code)]
pub fn parse_vec3(s: &str) -> Option<[f32; 3]> {
    let parts: Vec<&str> = s.splitn(3, ',').collect();
    if parts.len() != 3 {
        return None;
    }
    let x = parse_f32(parts[0])?;
    let y = parse_f32(parts[1])?;
    let z = parse_f32(parts[2])?;
    Some([x, y, z])
}

/// Format an f32 compactly (removes trailing zeros where possible).
#[allow(dead_code)]
pub fn format_f32_compact(v: f32) -> String {
    // Use standard display then trim trailing zeros after decimal point
    let s = format!("{v:.6}");
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    s.to_string()
}

/// Return true if the string represents an integer (no decimal point).
#[allow(dead_code)]
pub fn is_integer_str(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() {
        return false;
    }
    // Allow optional leading sign
    let digits = if t.starts_with('-') || t.starts_with('+') {
        &t[1..]
    } else {
        t
    };
    !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_f32_valid() {
        let v = parse_f32("1.5").expect("should succeed");
        assert!((v - 1.5f32).abs() < 1e-5);
    }

    #[test]
    fn test_parse_f32_invalid() {
        assert!(parse_f32("abc").is_none());
    }

    #[test]
    fn test_parse_i32_valid() {
        assert_eq!(parse_i32("42"), Some(42));
        assert_eq!(parse_i32("-7"), Some(-7));
    }

    #[test]
    fn test_parse_i32_invalid() {
        assert!(parse_i32("3.14").is_none());
    }

    #[test]
    fn test_parse_bool_variants() {
        assert_eq!(parse_bool("true"), Some(true));
        assert_eq!(parse_bool("False"), Some(false));
        assert_eq!(parse_bool("1"), Some(true));
        assert_eq!(parse_bool("0"), Some(false));
        assert_eq!(parse_bool("yes"), Some(true));
        assert_eq!(parse_bool("no"), Some(false));
    }

    #[test]
    fn test_parse_bool_invalid() {
        assert!(parse_bool("maybe").is_none());
    }

    #[test]
    fn test_parse_vec3_valid() {
        let v = parse_vec3("1.0,2.0,3.0").expect("should succeed");
        assert!((v[0] - 1.0f32).abs() < 1e-5);
        assert!((v[1] - 2.0f32).abs() < 1e-5);
        assert!((v[2] - 3.0f32).abs() < 1e-5);
    }

    #[test]
    fn test_parse_vec3_invalid_count() {
        assert!(parse_vec3("1.0,2.0").is_none());
    }

    #[test]
    fn test_is_integer_str_valid() {
        assert!(is_integer_str("42"));
        assert!(is_integer_str("-7"));
        assert!(is_integer_str("0"));
    }

    #[test]
    fn test_is_integer_str_invalid() {
        assert!(!is_integer_str("3.14"));
        assert!(!is_integer_str("abc"));
        assert!(!is_integer_str(""));
    }
}
