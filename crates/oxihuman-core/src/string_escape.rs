#![allow(dead_code)]

/// Escapes a string for use in JSON.
#[allow(dead_code)]
pub fn escape_json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            _ => out.push(c),
        }
    }
    out
}

/// Unescapes a JSON-escaped string.
#[allow(dead_code)]
pub fn unescape_json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('u') => {
                    let hex: String = chars.by_ref().take(4).collect();
                    if let Ok(code) = u32::from_str_radix(&hex, 16) {
                        if let Some(ch) = char::from_u32(code) {
                            out.push(ch);
                        }
                    }
                }
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Escapes HTML special characters.
#[allow(dead_code)]
pub fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// Unescapes HTML entities.
#[allow(dead_code)]
pub fn unescape_html(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

/// Percent-encodes a URL string.
#[allow(dead_code)]
pub fn escape_url(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push_str(&format!("%{b:02X}"));
            }
        }
    }
    out
}

/// Decodes a percent-encoded URL string.
#[allow(dead_code)]
pub fn unescape_url(s: &str) -> String {
    let mut out = String::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = &s[i + 1..i + 3];
            if let Ok(val) = u8::from_str_radix(hex, 16) {
                out.push(val as char);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

/// Returns true if the string contains characters that need JSON escaping.
#[allow(dead_code)]
pub fn needs_escaping(s: &str) -> bool {
    s.chars()
        .any(|c| c == '"' || c == '\\' || (c as u32) < 0x20)
}

/// Returns the length of the JSON-escaped version of the string.
#[allow(dead_code)]
pub fn escaped_length(s: &str) -> usize {
    escape_json_string(s).len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_json() {
        assert_eq!(escape_json_string("hello"), "hello");
        assert_eq!(escape_json_string("he\"llo"), "he\\\"llo");
    }

    #[test]
    fn test_unescape_json() {
        assert_eq!(unescape_json_string("he\\\"llo"), "he\"llo");
        assert_eq!(unescape_json_string("a\\nb"), "a\nb");
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<b>hi</b>"), "&lt;b&gt;hi&lt;/b&gt;");
    }

    #[test]
    fn test_unescape_html() {
        assert_eq!(unescape_html("&lt;b&gt;"), "<b>");
    }

    #[test]
    fn test_escape_url() {
        assert_eq!(escape_url("hello world"), "hello%20world");
    }

    #[test]
    fn test_unescape_url() {
        assert_eq!(unescape_url("hello%20world"), "hello world");
    }

    #[test]
    fn test_needs_escaping() {
        assert!(!needs_escaping("hello"));
        assert!(needs_escaping("he\"llo"));
        assert!(needs_escaping("tab\there"));
    }

    #[test]
    fn test_escaped_length() {
        assert_eq!(escaped_length("abc"), 3);
        assert!(escaped_length("a\"b") > 3);
    }

    #[test]
    fn test_roundtrip_json() {
        let orig = "line1\nline2\ttab\"quote\\back";
        let escaped = escape_json_string(orig);
        let unescaped = unescape_json_string(&escaped);
        assert_eq!(unescaped, orig);
    }

    #[test]
    fn test_roundtrip_url() {
        let orig = "key=val&foo=bar baz";
        let escaped = escape_url(orig);
        let unescaped = unescape_url(&escaped);
        assert_eq!(unescaped, orig);
    }
}
