// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! MIME content negotiation — selects the best media type from Accept headers.

/// A parsed MIME type with optional quality factor.
#[derive(Clone, Debug)]
pub struct MediaType {
    pub mime: String,
    pub quality: f32,
}

/// Result of content negotiation.
#[derive(Clone, Debug)]
pub struct NegotiationResult {
    pub selected: String,
    pub quality: f32,
}

/// Parses an Accept header value into a list of media types with quality factors.
pub fn parse_accept_header(accept: &str) -> Vec<MediaType> {
    let mut types: Vec<MediaType> = accept
        .split(',')
        .filter_map(|part| {
            let part = part.trim();
            if part.is_empty() {
                return None;
            }
            let mut segments = part.splitn(2, ";q=");
            let mime = segments.next()?.trim().to_lowercase();
            let quality: f32 = segments
                .next()
                .and_then(|q| q.trim().parse().ok())
                .unwrap_or(1.0);
            Some(MediaType { mime, quality })
        })
        .collect();
    /* sort highest quality first */
    types.sort_by(|a, b| {
        b.quality
            .partial_cmp(&a.quality)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    types
}

/// Selects the best matching media type from the server's offered types.
pub fn negotiate<'a>(
    accept_types: &[MediaType],
    offered: &'a [&'a str],
) -> Option<NegotiationResult> {
    let wildcard_any = ["*", "*"].join("/");
    let wildcard_suffix = ["/", "*"].join("");
    for mt in accept_types {
        let mime = mt.mime.as_str();
        if mime == wildcard_any {
            if let Some(&o) = offered.first() {
                return Some(NegotiationResult {
                    selected: o.into(),
                    quality: mt.quality,
                });
            }
        } else if let Some(prefix) = mime.strip_suffix(&wildcard_suffix) {
            /* wildcard subtype match — find type with matching prefix */
            if let Some(&o) = offered.iter().find(|&&off| off.starts_with(prefix)) {
                return Some(NegotiationResult {
                    selected: o.into(),
                    quality: mt.quality,
                });
            }
        } else if let Some(&o) = offered.iter().find(|&&off| off == mime) {
            return Some(NegotiationResult {
                selected: o.into(),
                quality: mt.quality,
            });
        }
    }
    None
}

/// Returns true if the given MIME type is a text format.
pub fn is_text_type(mime: &str) -> bool {
    mime.starts_with("text/") || mime == "application/json" || mime == "application/xml"
}

/// Returns the file extension typically associated with a MIME type.
pub fn mime_to_extension(mime: &str) -> Option<&'static str> {
    match mime {
        "text/html" => Some("html"),
        "text/plain" => Some("txt"),
        "application/json" => Some("json"),
        "application/xml" | "text/xml" => Some("xml"),
        "image/png" => Some("png"),
        "image/jpeg" => Some("jpg"),
        "application/octet-stream" => Some("bin"),
        _ => None,
    }
}

/// Returns the default quality for a given MIME type (stub heuristic).
pub fn default_quality(mime: &str) -> f32 {
    let wildcard_any = ["*", "*"].join("/");
    let wildcard_suffix = ["/", "*"].join("");
    if mime == wildcard_any {
        0.1
    } else if mime.ends_with(&wildcard_suffix) {
        0.5
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_accept_header_basic() {
        let types = parse_accept_header("text/html, application/json;q=0.9");
        assert_eq!(types[0].mime, "text/html");
        assert!((types[0].quality - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_parse_quality_factor() {
        let types = parse_accept_header("application/json;q=0.8");
        assert!((types[0].quality - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_negotiate_exact_match() {
        let accept = parse_accept_header("application/json");
        let offered = ["text/html", "application/json"];
        let result = negotiate(&accept, &offered).unwrap();
        assert_eq!(result.selected, "application/json");
    }

    #[test]
    fn test_negotiate_wildcard_any() {
        let wildcard = ["*", "*"].join("/");
        let accept = parse_accept_header(&wildcard);
        let offered = ["application/json"];
        let result = negotiate(&accept, &offered).unwrap();
        assert_eq!(result.selected, "application/json");
    }

    #[test]
    fn test_negotiate_subtype_wildcard() {
        let text_wildcard = ["text", "*"].join("/");
        let accept = parse_accept_header(&text_wildcard);
        let offered = ["text/plain", "application/json"];
        let result = negotiate(&accept, &offered).unwrap();
        assert_eq!(result.selected, "text/plain");
    }

    #[test]
    fn test_negotiate_no_match_returns_none() {
        let accept = parse_accept_header("image/png");
        let offered = ["text/html"];
        assert!(negotiate(&accept, &offered).is_none());
    }

    #[test]
    fn test_is_text_type() {
        assert!(is_text_type("text/html"));
        assert!(is_text_type("application/json"));
        assert!(!is_text_type("image/png"));
    }

    #[test]
    fn test_mime_to_extension() {
        assert_eq!(mime_to_extension("application/json"), Some("json"));
        assert_eq!(mime_to_extension("image/jpeg"), Some("jpg"));
        assert!(mime_to_extension("unknown/type").is_none());
    }

    #[test]
    fn test_default_quality_wildcard() {
        let wc_any = ["*", "*"].join("/");
        let wc_text = ["text", "*"].join("/");
        assert!((default_quality(&wc_any) - 0.1).abs() < 1e-5);
        assert!((default_quality(&wc_text) - 0.5).abs() < 1e-5);
        assert!((default_quality("text/html") - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_highest_quality_selected_first() {
        let accept = parse_accept_header("text/html;q=0.5, application/json;q=0.9");
        let offered = ["text/html", "application/json"];
        let result = negotiate(&accept, &offered).unwrap();
        assert_eq!(result.selected, "application/json");
    }
}
