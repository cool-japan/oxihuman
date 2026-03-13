// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Escape and unescape utilities for HTML entities and JSON strings.

#![allow(dead_code)]

/// Escape a string as HTML, replacing `&`, `<`, `>`, `"`, `'`.
#[allow(dead_code)]
pub fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            other => out.push(other),
        }
    }
    out
}

/// Unescape HTML entities back to characters.
#[allow(dead_code)]
pub fn html_unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while !rest.is_empty() {
        if let Some(idx) = rest.find('&') {
            out.push_str(&rest[..idx]);
            rest = &rest[idx..];
            if rest.starts_with("&amp;") {
                out.push('&');
                rest = &rest[5..];
            } else if rest.starts_with("&lt;") {
                out.push('<');
                rest = &rest[4..];
            } else if rest.starts_with("&gt;") {
                out.push('>');
                rest = &rest[4..];
            } else if rest.starts_with("&quot;") {
                out.push('"');
                rest = &rest[6..];
            } else if rest.starts_with("&#39;") {
                out.push('\'');
                rest = &rest[5..];
            } else {
                out.push('&');
                rest = &rest[1..];
            }
        } else {
            out.push_str(rest);
            break;
        }
    }
    out
}

/// Escape a string as a JSON string value (without surrounding quotes).
#[allow(dead_code)]
pub fn json_string_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\x08' => out.push_str("\\b"),
            '\x0c' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            other => out.push(other),
        }
    }
    out
}

/// Unescape a JSON-escaped string (no surrounding quotes).
#[allow(dead_code)]
pub fn json_string_unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('"') => out.push('"'),
            Some('\\') => out.push('\\'),
            Some('/') => out.push('/'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('b') => out.push('\x08'),
            Some('f') => out.push('\x0c'),
            Some('u') => {
                let hex: String = (0..4).filter_map(|_| chars.next()).collect();
                if let Ok(code) = u32::from_str_radix(&hex, 16) {
                    if let Some(c) = char::from_u32(code) {
                        out.push(c);
                    }
                }
            }
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => break,
        }
    }
    out
}

/// Escape special URL characters (basic percent-encoding for spaces and common chars).
#[allow(dead_code)]
pub fn url_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            ' ' => out.push_str("%20"),
            '&' => out.push_str("%26"),
            '=' => out.push_str("%3D"),
            '+' => out.push_str("%2B"),
            '#' => out.push_str("%23"),
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_escape_ampersand() {
        assert_eq!(html_escape("a&b"), "a&amp;b");
    }

    #[test]
    fn html_escape_lt_gt() {
        assert_eq!(html_escape("<div>"), "&lt;div&gt;");
    }

    #[test]
    fn html_escape_roundtrip() {
        let s = "Hello <World> & \"test\" 'x'";
        let e = html_escape(s);
        let d = html_unescape(&e);
        assert_eq!(d, s);
    }

    #[test]
    fn html_unescape_no_entities() {
        assert_eq!(html_unescape("hello"), "hello");
    }

    #[test]
    fn json_escape_quotes() {
        let s = r#"say "hi""#;
        let e = json_string_escape(s);
        assert!(e.contains("\\\""));
    }

    #[test]
    fn json_escape_newline() {
        let e = json_string_escape("line1\nline2");
        assert!(e.contains("\\n"));
    }

    #[test]
    fn json_roundtrip() {
        let s = "tab\there\nnewline\\backslash\"quote";
        let e = json_string_escape(s);
        let d = json_string_unescape(&e);
        assert_eq!(d, s);
    }

    #[test]
    fn json_escape_empty() {
        assert_eq!(json_string_escape(""), "");
        assert_eq!(json_string_unescape(""), "");
    }

    #[test]
    fn url_escape_space() {
        assert_eq!(url_escape("hello world"), "hello%20world");
    }

    #[test]
    fn url_escape_ampersand() {
        assert_eq!(url_escape("a&b"), "a%26b");
    }
}
