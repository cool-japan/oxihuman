// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Full-featured URL parser supporting RFC 3986, IPv6, userinfo, percent-encoding,
//! relative URL resolution, query-string parsing, punycode/IDNA basics, and
//! URL normalization.

#![allow(dead_code)]

use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// Percent encoding / decoding  (RFC 3986)
// ---------------------------------------------------------------------------

/// Characters that are *unreserved* per RFC 3986 and never need encoding.
fn is_unreserved(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'.' || b == b'_' || b == b'~'
}

/// Percent-encode a byte slice, encoding every byte that is not *unreserved*.
pub fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for &b in input.as_bytes() {
        if is_unreserved(b) {
            out.push(b as char);
        } else {
            out.push('%');
            out.push(to_hex_upper(b >> 4));
            out.push(to_hex_upper(b & 0x0F));
        }
    }
    out
}

/// Percent-encode only characters that are invalid in a path segment.
/// Keeps `/`, `@`, `:` and sub-delimiters intact.
pub fn percent_encode_path(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for &b in input.as_bytes() {
        if is_unreserved(b)
            || b == b'/'
            || b == b':'
            || b == b'@'
            || b == b'!'
            || b == b'$'
            || b == b'&'
            || b == b'\''
            || b == b'('
            || b == b')'
            || b == b'*'
            || b == b'+'
            || b == b','
            || b == b';'
            || b == b'='
        {
            out.push(b as char);
        } else {
            out.push('%');
            out.push(to_hex_upper(b >> 4));
            out.push(to_hex_upper(b & 0x0F));
        }
    }
    out
}

/// Percent-encode for query strings -- space becomes `+` in
/// `application/x-www-form-urlencoded`; other specials are encoded.
pub fn percent_encode_query_component(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for &b in input.as_bytes() {
        if is_unreserved(b) {
            out.push(b as char);
        } else if b == b' ' {
            out.push('+');
        } else {
            out.push('%');
            out.push(to_hex_upper(b >> 4));
            out.push(to_hex_upper(b & 0x0F));
        }
    }
    out
}

/// Decode percent-encoded strings. Also converts `+` to space when
/// `plus_as_space` is true (form-urlencoded).
pub fn percent_decode(input: &str, plus_as_space: bool) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Some(val) = hex_pair(bytes[i + 1], bytes[i + 2]) {
                out.push(val);
                i += 3;
                continue;
            }
        }
        if plus_as_space && bytes[i] == b'+' {
            out.push(b' ');
        } else {
            out.push(bytes[i]);
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn to_hex_upper(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        _ => (b'A' + nibble - 10) as char,
    }
}

fn hex_pair(hi: u8, lo: u8) -> Option<u8> {
    let h = hex_val(hi)?;
    let l = hex_val(lo)?;
    Some((h << 4) | l)
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'A'..=b'F' => Some(b - b'A' + 10),
        b'a'..=b'f' => Some(b - b'a' + 10),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Punycode / IDNA (basic)
// ---------------------------------------------------------------------------

const PUNYCODE_BASE: u32 = 36;
const PUNYCODE_TMIN: u32 = 1;
const PUNYCODE_TMAX: u32 = 26;
const PUNYCODE_SKEW: u32 = 38;
const PUNYCODE_DAMP: u32 = 700;
const PUNYCODE_INITIAL_BIAS: u32 = 72;
const PUNYCODE_INITIAL_N: u32 = 0x80;

fn punycode_adapt(mut delta: u32, num_points: u32, first_time: bool) -> u32 {
    delta = if first_time {
        delta / PUNYCODE_DAMP
    } else {
        delta / 2
    };
    delta += delta / num_points;
    let mut k = 0u32;
    while delta > ((PUNYCODE_BASE - PUNYCODE_TMIN) * PUNYCODE_TMAX) / 2 {
        delta /= PUNYCODE_BASE - PUNYCODE_TMIN;
        k += PUNYCODE_BASE;
    }
    k + ((PUNYCODE_BASE - PUNYCODE_TMIN + 1) * delta) / (delta + PUNYCODE_SKEW)
}

fn punycode_encode_digit(d: u32) -> Option<char> {
    if d < 26 {
        Some((b'a' + d as u8) as char)
    } else if d < 36 {
        Some((b'0' + (d as u8 - 26)) as char)
    } else {
        None
    }
}

fn punycode_decode_digit(c: u8) -> Option<u32> {
    match c {
        b'a'..=b'z' => Some((c - b'a') as u32),
        b'A'..=b'Z' => Some((c - b'A') as u32),
        b'0'..=b'9' => Some((c - b'0' + 26) as u32),
        _ => None,
    }
}

/// Encode a Unicode label to Punycode (without the `xn--` prefix).
pub fn punycode_encode(input: &str) -> Option<String> {
    let codepoints: Vec<u32> = input.chars().map(|c| c as u32).collect();
    let mut output = String::new();

    for &cp in &codepoints {
        if cp < 0x80 {
            output.push(char::from(cp as u8));
        }
    }
    let mut handled = output.len() as u32;
    let basic_len = handled;

    if basic_len > 0 && handled < codepoints.len() as u32 {
        output.push('-');
    }

    let mut n = PUNYCODE_INITIAL_N;
    let mut delta: u32 = 0;
    let mut bias = PUNYCODE_INITIAL_BIAS;

    while (handled as usize) < codepoints.len() {
        let m = codepoints.iter().copied().filter(|&cp| cp >= n).min()?;

        delta = delta.checked_add((m - n).checked_mul(handled + 1)?)?;
        n = m;

        for &cp in &codepoints {
            if cp < n {
                delta = delta.checked_add(1)?;
            } else if cp == n {
                let mut q = delta;
                let mut k = PUNYCODE_BASE;
                loop {
                    let t = if k <= bias {
                        PUNYCODE_TMIN
                    } else if k >= bias + PUNYCODE_TMAX {
                        PUNYCODE_TMAX
                    } else {
                        k - bias
                    };
                    if q < t {
                        break;
                    }
                    let digit = t + ((q - t) % (PUNYCODE_BASE - t));
                    output.push(punycode_encode_digit(digit)?);
                    q = (q - t) / (PUNYCODE_BASE - t);
                    k += PUNYCODE_BASE;
                }
                output.push(punycode_encode_digit(q)?);
                bias = punycode_adapt(delta, handled + 1, handled == basic_len);
                delta = 0;
                handled += 1;
            }
        }
        delta += 1;
        n += 1;
    }
    Some(output)
}

/// Decode a Punycode string (without `xn--` prefix) to Unicode.
pub fn punycode_decode(input: &str) -> Option<String> {
    let bytes = input.as_bytes();
    let mut output: Vec<u32> = Vec::new();

    let basic_end = input.rfind('-').unwrap_or_default();
    for &b in &bytes[..basic_end] {
        if b >= 0x80 {
            return None;
        }
        output.push(b as u32);
    }

    let mut n = PUNYCODE_INITIAL_N;
    let mut i: u32 = 0;
    let mut bias = PUNYCODE_INITIAL_BIAS;

    let mut idx = if basic_end > 0 { basic_end + 1 } else { 0 };

    while idx < bytes.len() {
        let old_i = i;
        let mut w: u32 = 1;
        let mut k = PUNYCODE_BASE;

        loop {
            if idx >= bytes.len() {
                return None;
            }
            let digit = punycode_decode_digit(bytes[idx])?;
            idx += 1;
            i = i.checked_add(digit.checked_mul(w)?)?;
            let t = if k <= bias {
                PUNYCODE_TMIN
            } else if k >= bias + PUNYCODE_TMAX {
                PUNYCODE_TMAX
            } else {
                k - bias
            };
            if digit < t {
                break;
            }
            w = w.checked_mul(PUNYCODE_BASE - t)?;
            k += PUNYCODE_BASE;
        }

        let out_len = (output.len() as u32) + 1;
        bias = punycode_adapt(i - old_i, out_len, old_i == 0);
        n = n.checked_add(i / out_len)?;
        i %= out_len;

        output.insert(i as usize, n);
        i += 1;
    }

    output.iter().map(|&cp| char::from_u32(cp)).collect()
}

/// Convert an internationalized domain label to its ACE form (`xn--...`).
pub fn domain_to_ascii(domain: &str) -> String {
    domain
        .split('.')
        .map(|label| {
            if label.is_ascii() {
                label.to_ascii_lowercase()
            } else {
                match punycode_encode(&label.to_lowercase()) {
                    Some(encoded) => format!("xn--{}", encoded),
                    None => label.to_ascii_lowercase(),
                }
            }
        })
        .collect::<Vec<_>>()
        .join(".")
}

/// Convert an ACE domain back to Unicode (best effort).
pub fn domain_to_unicode(domain: &str) -> String {
    domain
        .split('.')
        .map(|label| {
            if let Some(stripped) = label.strip_prefix("xn--") {
                match punycode_decode(stripped) {
                    Some(decoded) => decoded,
                    None => label.to_string(),
                }
            } else {
                label.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(".")
}

// ---------------------------------------------------------------------------
// Core URL data structure
// ---------------------------------------------------------------------------

/// Parsed components of a URL.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UrlParts {
    pub scheme: String,
    pub host: String,
    pub port: Option<u16>,
    pub path: String,
    pub query: String,
    pub fragment: String,
    /// Optional userinfo component (`user` or `user:password`).
    pub userinfo: Option<String>,
}

impl fmt::Display for UrlParts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", url_to_string(self))
    }
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

/// Parse a URL string into its component parts.
/// Returns `None` if the URL is not recognizable.
pub fn parse_url(url: &str) -> Option<UrlParts> {
    let scheme_end = url.find("://")?;
    let scheme = url[..scheme_end].to_ascii_lowercase();

    if scheme.is_empty() || !scheme.as_bytes()[0].is_ascii_alphabetic() {
        return None;
    }
    if !scheme
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'+' || b == b'-' || b == b'.')
    {
        return None;
    }

    let rest = &url[scheme_end + 3..];

    let (rest, fragment) = if let Some(pos) = rest.find('#') {
        (&rest[..pos], rest[pos + 1..].to_string())
    } else {
        (rest, String::new())
    };

    let (rest, query) = if let Some(pos) = rest.find('?') {
        (&rest[..pos], rest[pos + 1..].to_string())
    } else {
        (rest, String::new())
    };

    let (authority, path) = if let Some(pos) = rest.find('/') {
        (&rest[..pos], rest[pos..].to_string())
    } else {
        (rest, "/".to_string())
    };

    let (userinfo, host_port) = if let Some(at_pos) = authority.rfind('@') {
        (
            Some(authority[..at_pos].to_string()),
            &authority[at_pos + 1..],
        )
    } else {
        (None, authority)
    };

    let (host, port) = parse_host_port(host_port);

    Some(UrlParts {
        scheme,
        host,
        port,
        path,
        query,
        fragment,
        userinfo,
    })
}

/// Parse the host-and-port portion, handling IPv6 bracket notation.
fn parse_host_port(input: &str) -> (String, Option<u16>) {
    if input.starts_with('[') {
        if let Some(bracket_end) = input.find(']') {
            let addr = input[1..bracket_end].to_string();
            let after = &input[bracket_end + 1..];
            let port = if let Some(stripped) = after.strip_prefix(':') {
                stripped.parse::<u16>().ok()
            } else {
                None
            };
            (addr, port)
        } else {
            (input.to_string(), None)
        }
    } else if let Some(colon_pos) = input.rfind(':') {
        let port_str = &input[colon_pos + 1..];
        if let Ok(p) = port_str.parse::<u16>() {
            (input[..colon_pos].to_string(), Some(p))
        } else {
            (input.to_string(), None)
        }
    } else {
        (input.to_string(), None)
    }
}

// ---------------------------------------------------------------------------
// Serialization
// ---------------------------------------------------------------------------

/// Convert `UrlParts` back to a URL string.
pub fn url_to_string(url: &UrlParts) -> String {
    let mut s = format!("{}://", url.scheme);

    if let Some(ref info) = url.userinfo {
        s.push_str(info);
        s.push('@');
    }

    if url.host.contains(':') {
        s.push('[');
        s.push_str(&url.host);
        s.push(']');
    } else {
        s.push_str(&url.host);
    }

    if let Some(port) = url.port {
        s.push_str(&format!(":{}", port));
    }
    s.push_str(&url.path);
    if !url.query.is_empty() {
        s.push('?');
        s.push_str(&url.query);
    }
    if !url.fragment.is_empty() {
        s.push('#');
        s.push_str(&url.fragment);
    }
    s
}

// ---------------------------------------------------------------------------
// Query-string helpers
// ---------------------------------------------------------------------------

/// Get the value of a query parameter by key (first occurrence).
pub fn url_query_param(url: &UrlParts, key: &str) -> Option<String> {
    if url.query.is_empty() {
        return None;
    }
    for pair in url.query.split('&') {
        if pair.is_empty() {
            continue;
        }
        if let Some(eq_pos) = pair.find('=') {
            let k = percent_decode(&pair[..eq_pos], true);
            if k == key {
                return Some(percent_decode(&pair[eq_pos + 1..], true));
            }
        } else {
            let k = percent_decode(pair, true);
            if k == key {
                return Some(String::new());
            }
        }
    }
    None
}

/// Parse the query string into a list of key-value pairs.
/// Percent-decoding is applied; `+` is treated as space.
pub fn parse_query_string(query: &str) -> Vec<(String, String)> {
    if query.is_empty() {
        return Vec::new();
    }
    query
        .split('&')
        .filter(|s| !s.is_empty())
        .map(|pair| {
            if let Some(eq) = pair.find('=') {
                (
                    percent_decode(&pair[..eq], true),
                    percent_decode(&pair[eq + 1..], true),
                )
            } else {
                (percent_decode(pair, true), String::new())
            }
        })
        .collect()
}

/// Parse query string into a `HashMap`. If duplicate keys exist the last
/// value wins.
pub fn parse_query_string_map(query: &str) -> HashMap<String, String> {
    parse_query_string(query).into_iter().collect()
}

/// Build a query string from key-value pairs with percent-encoding.
pub fn build_query_string(params: &[(impl AsRef<str>, impl AsRef<str>)]) -> String {
    params
        .iter()
        .map(|(k, v)| {
            format!(
                "{}={}",
                percent_encode_query_component(k.as_ref()),
                percent_encode_query_component(v.as_ref())
            )
        })
        .collect::<Vec<_>>()
        .join("&")
}

// ---------------------------------------------------------------------------
// URL predicates
// ---------------------------------------------------------------------------

/// Check if a string looks like an absolute URL (has a scheme).
pub fn is_absolute_url(s: &str) -> bool {
    if let Some(pos) = s.find("://") {
        let scheme = &s[..pos];
        !scheme.is_empty()
            && scheme.as_bytes()[0].is_ascii_alphabetic()
            && scheme
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'+' || b == b'-' || b == b'.')
    } else {
        false
    }
}

// ---------------------------------------------------------------------------
// Path manipulation
// ---------------------------------------------------------------------------

/// Remove `.` and `..` segments from a path (RFC 3986 section 5.2.4).
pub fn normalize_path(path: &str) -> String {
    let mut segments: Vec<&str> = Vec::new();
    let has_leading_slash = path.starts_with('/');
    let has_trailing_slash = path.ends_with('/') && path.len() > 1;

    for seg in path.split('/') {
        match seg {
            "." | "" => {}
            ".." => {
                segments.pop();
            }
            _ => segments.push(seg),
        }
    }

    let mut result = if has_leading_slash {
        "/".to_string()
    } else {
        String::new()
    };
    result.push_str(&segments.join("/"));
    if has_trailing_slash && !result.ends_with('/') {
        result.push('/');
    }
    if result.is_empty() {
        "/".to_string()
    } else {
        result
    }
}

/// Join a base path with a relative reference, resolving `.` and `..`.
pub fn join_path(base: &str, relative: &str) -> String {
    if relative.starts_with('/') {
        return normalize_path(relative);
    }
    let base_dir = if let Some(pos) = base.rfind('/') {
        &base[..=pos]
    } else {
        "/"
    };
    let merged = format!("{}{}", base_dir, relative);
    normalize_path(&merged)
}

/// Split a path into its segments.
pub fn path_segments(path: &str) -> Vec<String> {
    path.split('/')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

// ---------------------------------------------------------------------------
// Relative URL resolution (RFC 3986 section 5)
// ---------------------------------------------------------------------------

/// Resolve a (possibly relative) URL reference against a base URL.
///
/// If `reference` is absolute it is returned as-is (parsed).
/// Otherwise the base URL is used to fill in missing components.
pub fn resolve_url(base: &UrlParts, reference: &str) -> Option<UrlParts> {
    if is_absolute_url(reference) {
        return parse_url(reference);
    }

    if reference.starts_with("//") {
        let full = format!("{}:{}", base.scheme, reference);
        return parse_url(&full);
    }

    if let Some(frag) = reference.strip_prefix('#') {
        let mut result = base.clone();
        result.fragment = frag.to_string();
        return Some(result);
    }

    if let Some(rest) = reference.strip_prefix('?') {
        let mut result = base.clone();
        let (q, f) = if let Some(hash) = rest.find('#') {
            (&rest[..hash], rest[hash + 1..].to_string())
        } else {
            (rest, String::new())
        };
        result.query = q.to_string();
        result.fragment = f;
        return Some(result);
    }

    let (rest, fragment) = if let Some(pos) = reference.find('#') {
        (&reference[..pos], reference[pos + 1..].to_string())
    } else {
        (reference, String::new())
    };

    let (rest, query) = if let Some(pos) = rest.find('?') {
        (&rest[..pos], rest[pos + 1..].to_string())
    } else {
        (rest, String::new())
    };

    let path = join_path(&base.path, rest);

    Some(UrlParts {
        scheme: base.scheme.clone(),
        host: base.host.clone(),
        port: base.port,
        path,
        query,
        fragment,
        userinfo: base.userinfo.clone(),
    })
}

// ---------------------------------------------------------------------------
// Normalization
// ---------------------------------------------------------------------------

/// Default port numbers for common schemes.
fn default_port(scheme: &str) -> Option<u16> {
    match scheme {
        "http" | "ws" => Some(80),
        "https" | "wss" => Some(443),
        "ftp" => Some(21),
        "ssh" => Some(22),
        _ => None,
    }
}

/// Normalize a URL:
/// * Lowercase scheme and host
/// * Remove default port
/// * Normalize path (remove `.` / `..`, collapse slashes)
/// * Uppercase percent-encoded triplets
/// * Decode unreserved percent-encoded characters
pub fn normalize_url(url: &mut UrlParts) {
    url.scheme = url.scheme.to_ascii_lowercase();
    url.host = url.host.to_ascii_lowercase();

    if let Some(port) = url.port {
        if default_port(&url.scheme) == Some(port) {
            url.port = None;
        }
    }

    url.path = normalize_path(&url.path);
    url.path = normalize_percent_encoding(&url.path);

    if !url.query.is_empty() {
        url.query = normalize_percent_encoding(&url.query);
    }
    if !url.fragment.is_empty() {
        url.fragment = normalize_percent_encoding(&url.fragment);
    }
}

/// Parse and normalize a URL in one shot.
pub fn parse_and_normalize_url(url: &str) -> Option<UrlParts> {
    let mut parts = parse_url(url)?;
    normalize_url(&mut parts);
    Some(parts)
}

/// Normalize percent-encoding: decode unreserved characters, uppercase
/// hex digits in remaining triplets.
fn normalize_percent_encoding(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Some(val) = hex_pair(bytes[i + 1], bytes[i + 2]) {
                if is_unreserved(val) {
                    out.push(val as char);
                } else {
                    out.push('%');
                    out.push(to_hex_upper(val >> 4));
                    out.push(to_hex_upper(val & 0x0F));
                }
                i += 3;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_http() {
        let u = parse_url("http://example.com/").expect("should succeed");
        assert_eq!(u.scheme, "http");
        assert_eq!(u.host, "example.com");
        assert_eq!(u.path, "/");
    }

    #[test]
    fn parse_with_port() {
        let u = parse_url("http://localhost:8080/api").expect("should succeed");
        assert_eq!(u.port, Some(8080));
        assert_eq!(u.host, "localhost");
    }

    #[test]
    fn parse_with_query() {
        let u = parse_url("https://example.com/search?q=hello&lang=en").expect("should succeed");
        assert_eq!(url_query_param(&u, "q"), Some("hello".to_string()));
        assert_eq!(url_query_param(&u, "lang"), Some("en".to_string()));
    }

    #[test]
    fn parse_with_fragment() {
        let u = parse_url("https://example.com/page#section1").expect("should succeed");
        assert_eq!(u.fragment, "section1");
    }

    #[test]
    fn missing_scheme_returns_none() {
        assert!(parse_url("example.com/page").is_none());
    }

    #[test]
    fn url_to_string_round_trip() {
        let original = "https://example.com:443/path?x=1";
        let u = parse_url(original).expect("should succeed");
        let s = url_to_string(&u);
        assert!(s.contains("https://"));
        assert!(s.contains("example.com"));
    }

    #[test]
    fn is_absolute_url_true() {
        assert!(is_absolute_url("http://example.com"));
    }

    #[test]
    fn is_absolute_url_false() {
        assert!(!is_absolute_url("/relative/path"));
    }

    #[test]
    fn query_param_missing_returns_none() {
        let u = parse_url("https://example.com/").expect("should succeed");
        assert_eq!(url_query_param(&u, "missing"), None);
    }

    #[test]
    fn https_scheme_parsed() {
        let u = parse_url("https://secure.example.com/").expect("should succeed");
        assert_eq!(u.scheme, "https");
    }

    // ---- Percent encoding/decoding ----

    #[test]
    fn percent_encode_basic() {
        assert_eq!(percent_encode("hello world"), "hello%20world");
        assert_eq!(percent_encode("a+b=c"), "a%2Bb%3Dc");
    }

    #[test]
    fn percent_decode_basic() {
        assert_eq!(percent_decode("hello%20world", false), "hello world");
        assert_eq!(percent_decode("hello+world", true), "hello world");
        assert_eq!(percent_decode("hello+world", false), "hello+world");
    }

    #[test]
    fn percent_roundtrip() {
        let original = "foo bar/baz?x=1&y=hello world";
        let encoded = percent_encode(original);
        let decoded = percent_decode(&encoded, false);
        assert_eq!(decoded, original);
    }

    #[test]
    fn percent_encode_path_keeps_slashes() {
        let p = percent_encode_path("/foo/bar baz/qux");
        assert!(p.contains('/'));
        assert!(p.contains("%20"));
    }

    // ---- IPv6 ----

    #[test]
    fn parse_ipv6_host() {
        let u = parse_url("http://[::1]:8080/path").expect("should succeed");
        assert_eq!(u.host, "::1");
        assert_eq!(u.port, Some(8080));
        assert_eq!(u.path, "/path");
    }

    #[test]
    fn parse_ipv6_no_port() {
        let u = parse_url("http://[2001:db8::1]/").expect("should succeed");
        assert_eq!(u.host, "2001:db8::1");
        assert_eq!(u.port, None);
    }

    #[test]
    fn url_to_string_ipv6() {
        let u = parse_url("http://[::1]:8080/").expect("should succeed");
        let s = url_to_string(&u);
        assert_eq!(s, "http://[::1]:8080/");
    }

    // ---- Userinfo ----

    #[test]
    fn parse_userinfo() {
        let u = parse_url("ftp://user:pass@ftp.example.com/pub").expect("should succeed");
        assert_eq!(u.userinfo, Some("user:pass".to_string()));
        assert_eq!(u.host, "ftp.example.com");
        assert_eq!(u.path, "/pub");
    }

    #[test]
    fn parse_userinfo_no_password() {
        let u = parse_url("http://admin@example.com/").expect("should succeed");
        assert_eq!(u.userinfo, Some("admin".to_string()));
        assert_eq!(u.host, "example.com");
    }

    #[test]
    fn url_to_string_with_userinfo() {
        let u = parse_url("http://user:pw@host.com/").expect("should succeed");
        let s = url_to_string(&u);
        assert_eq!(s, "http://user:pw@host.com/");
    }

    // ---- Relative URL resolution ----

    #[test]
    fn resolve_absolute_reference() {
        let base = parse_url("http://example.com/a/b").expect("should succeed");
        let r = resolve_url(&base, "https://other.com/c").expect("should succeed");
        assert_eq!(r.scheme, "https");
        assert_eq!(r.host, "other.com");
        assert_eq!(r.path, "/c");
    }

    #[test]
    fn resolve_relative_path() {
        let base = parse_url("http://example.com/a/b/c").expect("should succeed");
        let r = resolve_url(&base, "../d").expect("should succeed");
        assert_eq!(r.path, "/a/d");
        assert_eq!(r.host, "example.com");
    }

    #[test]
    fn resolve_absolute_path() {
        let base = parse_url("http://example.com/a/b/c").expect("should succeed");
        let r = resolve_url(&base, "/x/y").expect("should succeed");
        assert_eq!(r.path, "/x/y");
    }

    #[test]
    fn resolve_fragment_only() {
        let base = parse_url("http://example.com/a?q=1").expect("should succeed");
        let r = resolve_url(&base, "#frag").expect("should succeed");
        assert_eq!(r.path, "/a");
        assert_eq!(r.query, "q=1");
        assert_eq!(r.fragment, "frag");
    }

    #[test]
    fn resolve_query_only() {
        let base = parse_url("http://example.com/a").expect("should succeed");
        let r = resolve_url(&base, "?x=1").expect("should succeed");
        assert_eq!(r.path, "/a");
        assert_eq!(r.query, "x=1");
    }

    #[test]
    fn resolve_protocol_relative() {
        let base = parse_url("https://example.com/a").expect("should succeed");
        let r = resolve_url(&base, "//other.com/b").expect("should succeed");
        assert_eq!(r.scheme, "https");
        assert_eq!(r.host, "other.com");
        assert_eq!(r.path, "/b");
    }

    // ---- Query string parsing ----

    #[test]
    fn parse_query_string_basic() {
        let pairs = parse_query_string("a=1&b=2&c=hello+world");
        assert_eq!(
            pairs,
            vec![
                ("a".into(), "1".into()),
                ("b".into(), "2".into()),
                ("c".into(), "hello world".into()),
            ]
        );
    }

    #[test]
    fn parse_query_string_empty() {
        let pairs = parse_query_string("");
        assert!(pairs.is_empty());
    }

    #[test]
    fn parse_query_string_no_value() {
        let pairs = parse_query_string("key");
        assert_eq!(pairs, vec![("key".into(), String::new())]);
    }

    #[test]
    fn build_and_parse_query() {
        let params = [("name", "John Doe"), ("age", "30")];
        let qs = build_query_string(&params);
        let parsed = parse_query_string(&qs);
        assert_eq!(parsed[0], ("name".into(), "John Doe".into()));
        assert_eq!(parsed[1], ("age".into(), "30".into()));
    }

    #[test]
    fn parse_query_string_map_basic() {
        let map = parse_query_string_map("x=10&y=20");
        assert_eq!(map.get("x").map(|s| s.as_str()), Some("10"));
        assert_eq!(map.get("y").map(|s| s.as_str()), Some("20"));
    }

    // ---- Fragment handling ----

    #[test]
    fn fragment_with_query() {
        let u = parse_url("https://example.com/page?q=1#section").expect("should succeed");
        assert_eq!(u.query, "q=1");
        assert_eq!(u.fragment, "section");
    }

    #[test]
    fn fragment_empty() {
        let u = parse_url("https://example.com/page#").expect("should succeed");
        assert_eq!(u.fragment, "");
    }

    // ---- Punycode / IDNA ----

    #[test]
    fn punycode_encode_decode() {
        let encoded = punycode_encode("m\u{00FC}nchen").expect("should succeed");
        let decoded = punycode_decode(&encoded).expect("should succeed");
        assert_eq!(decoded, "m\u{00FC}nchen");
    }

    #[test]
    fn punycode_ascii_only() {
        // All-ASCII input: punycode output is just the ASCII chars (no non-basic
        // code points to encode).
        let encoded = punycode_encode("example").expect("should succeed");
        assert_eq!(encoded, "example");
        // Decoding with trailing delimiter (the standard separator) works.
        let decoded = punycode_decode("example-").expect("should succeed");
        assert_eq!(decoded, "example");
    }

    #[test]
    fn domain_to_ascii_basic() {
        let ascii = domain_to_ascii("m\u{00FC}nchen.de");
        assert!(ascii.starts_with("xn--"));
        assert!(ascii.ends_with(".de"));
    }

    #[test]
    fn domain_roundtrip() {
        let original = "m\u{00FC}nchen.de";
        let ascii = domain_to_ascii(original);
        let unicode = domain_to_unicode(&ascii);
        assert_eq!(unicode, original);
    }

    #[test]
    fn domain_to_ascii_plain() {
        assert_eq!(domain_to_ascii("example.com"), "example.com");
    }

    // ---- Normalization ----

    #[test]
    fn normalize_removes_default_port() {
        let mut u = parse_url("https://example.com:443/path").expect("should succeed");
        normalize_url(&mut u);
        assert_eq!(u.port, None);
    }

    #[test]
    fn normalize_keeps_non_default_port() {
        let mut u = parse_url("http://example.com:8080/path").expect("should succeed");
        normalize_url(&mut u);
        assert_eq!(u.port, Some(8080));
    }

    #[test]
    fn normalize_lowercases_scheme_and_host() {
        let mut u = parse_url("HTTP://EXAMPLE.COM/Path").expect("should succeed");
        normalize_url(&mut u);
        assert_eq!(u.scheme, "http");
        assert_eq!(u.host, "example.com");
    }

    #[test]
    fn normalize_path_dots() {
        let mut u = parse_url("http://example.com/a/b/../c/./d").expect("should succeed");
        normalize_url(&mut u);
        assert_eq!(u.path, "/a/c/d");
    }

    #[test]
    fn normalize_percent_encoding_uppercase() {
        let mut u = parse_url("http://example.com/a%2fb").expect("should succeed");
        normalize_url(&mut u);
        assert_eq!(u.path, "/a%2Fb");
    }

    #[test]
    fn normalize_percent_decodes_unreserved() {
        let mut u = parse_url("http://example.com/%61%62%63").expect("should succeed");
        normalize_url(&mut u);
        assert_eq!(u.path, "/abc");
    }

    #[test]
    fn parse_and_normalize_convenience() {
        let u = parse_and_normalize_url("HTTP://Example.COM:80/a/../b").expect("should succeed");
        assert_eq!(u.scheme, "http");
        assert_eq!(u.host, "example.com");
        assert_eq!(u.port, None);
        assert_eq!(u.path, "/b");
    }

    // ---- Path manipulation ----

    #[test]
    fn normalize_path_basic() {
        assert_eq!(normalize_path("/a/b/c"), "/a/b/c");
        assert_eq!(normalize_path("/a/./b/../c"), "/a/c");
        assert_eq!(normalize_path("/a/b/../../c"), "/c");
        assert_eq!(normalize_path("/"), "/");
    }

    #[test]
    fn join_path_relative() {
        assert_eq!(join_path("/a/b/c", "d"), "/a/b/d");
        assert_eq!(join_path("/a/b/c", "../d"), "/a/d");
        assert_eq!(join_path("/a/b/c", "../../d"), "/d");
    }

    #[test]
    fn join_path_absolute() {
        assert_eq!(join_path("/a/b/c", "/x/y"), "/x/y");
    }

    #[test]
    fn path_segments_basic() {
        assert_eq!(
            path_segments("/a/b/c"),
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
        assert!(path_segments("/").is_empty());
    }

    // ---- Scheme validation ----

    #[test]
    fn invalid_scheme_returns_none() {
        assert!(parse_url("123://example.com").is_none());
    }

    // ---- Encoded query params ----

    #[test]
    fn query_param_percent_encoded() {
        let u = parse_url("http://example.com/?name=John%20Doe").expect("should succeed");
        assert_eq!(url_query_param(&u, "name"), Some("John Doe".to_string()));
    }

    // ---- Display impl ----

    #[test]
    fn display_impl() {
        let u = parse_url("http://example.com/path?q=1#f").expect("should succeed");
        let s = format!("{}", u);
        assert_eq!(s, "http://example.com/path?q=1#f");
    }
}
