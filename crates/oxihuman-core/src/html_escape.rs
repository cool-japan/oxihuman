// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! HTML entity escaping/unescaping utilities.

/// Escape a string for safe embedding in HTML content.
pub fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            other => out.push(other),
        }
    }
    out
}

/// Unescape common named and numeric HTML entities.
pub fn html_unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(amp_pos) = rest.find('&') {
        out.push_str(&rest[..amp_pos]);
        rest = &rest[amp_pos..];
        if let Some(semi_pos) = rest.find(';') {
            let entity = &rest[1..semi_pos]; /* between & and ; */
            let replacement = match entity {
                "amp" => "&",
                "lt" => "<",
                "gt" => ">",
                "quot" => "\"",
                "#x27" | "apos" => "'",
                "#x26" => "&",
                "#60" => "<",
                "#62" => ">",
                "#34" => "\"",
                _ => {
                    /* unknown entity: pass through as-is */
                    out.push_str(&rest[..semi_pos + 1]);
                    rest = &rest[semi_pos + 1..];
                    continue;
                }
            };
            out.push_str(replacement);
            rest = &rest[semi_pos + 1..];
        } else {
            /* no closing semicolon */
            out.push('&');
            rest = &rest[1..];
        }
    }
    out.push_str(rest);
    out
}

/// Return true if the string contains any characters that must be escaped.
pub fn html_needs_escape(s: &str) -> bool {
    s.chars().any(|c| matches!(c, '&' | '<' | '>' | '"' | '\''))
}

/// Escape a string for use in an HTML attribute value (double-quoted).
pub fn html_escape_attr(s: &str) -> String {
    /* same as html_escape but always escapes apostrophes too */
    html_escape(s)
}

/// Verify round-trip escape/unescape.
pub fn html_roundtrip_ok(s: &str) -> bool {
    html_unescape(&html_escape(s)) == s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_amp() {
        /* ampersand escapes */
        assert_eq!(html_escape("a&b"), "a&amp;b");
    }

    #[test]
    fn test_escape_lt_gt() {
        /* angle brackets escape */
        assert_eq!(html_escape("<div>"), "&lt;div&gt;");
    }

    #[test]
    fn test_escape_quote() {
        /* double quote escapes */
        assert_eq!(html_escape("say \"hi\""), "say &quot;hi&quot;");
    }

    #[test]
    fn test_escape_no_change() {
        /* safe string unchanged */
        assert_eq!(html_escape("hello world"), "hello world");
    }

    #[test]
    fn test_unescape_amp() {
        /* unescape &amp; */
        assert_eq!(html_unescape("a&amp;b"), "a&b");
    }

    #[test]
    fn test_unescape_lt_gt() {
        /* unescape angle brackets */
        assert_eq!(html_unescape("&lt;div&gt;"), "<div>");
    }

    #[test]
    fn test_needs_escape_true() {
        /* angle bracket triggers escape check */
        assert!(html_needs_escape("<script>"));
    }

    #[test]
    fn test_needs_escape_false() {
        /* plain text does not need escaping */
        assert!(!html_needs_escape("hello"));
    }

    #[test]
    fn test_roundtrip_complex() {
        /* complex string round-trips */
        let s = "<h1>Hello & \"World\"!</h1>";
        assert!(html_roundtrip_ok(s));
    }
}
