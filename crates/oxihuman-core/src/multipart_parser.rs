// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Multipart/form-data parser stub — splits parts by boundary.

/// A single multipart form part.
#[derive(Clone, Debug, PartialEq)]
pub struct MultipartPart {
    pub name: Option<String>,
    pub filename: Option<String>,
    pub content_type: String,
    pub data: Vec<u8>,
}

/// Result of parsing a multipart body.
#[derive(Clone, Debug, PartialEq)]
pub struct MultipartBody {
    pub boundary: String,
    pub parts: Vec<MultipartPart>,
}

/// Errors from multipart parsing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MultipartError {
    MissingBoundary,
    MalformedPart,
    EmptyBody,
}

impl std::fmt::Display for MultipartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MultipartError::MissingBoundary => write!(f, "missing boundary"),
            MultipartError::MalformedPart => write!(f, "malformed part"),
            MultipartError::EmptyBody => write!(f, "empty body"),
        }
    }
}

/// Extracts the boundary from a Content-Type header value.
pub fn extract_boundary(content_type: &str) -> Option<String> {
    content_type.split(';').find_map(|seg| {
        let seg = seg.trim();
        seg.strip_prefix("boundary=")
            .map(|b| b.trim_matches('"').to_owned())
    })
}

/// Parses a raw multipart body string given a boundary.
pub fn parse_multipart(body: &str, boundary: &str) -> Result<MultipartBody, MultipartError> {
    if body.is_empty() {
        return Err(MultipartError::EmptyBody);
    }
    if boundary.is_empty() {
        return Err(MultipartError::MissingBoundary);
    }
    let delimiter = format!("--{}", boundary);
    let mut parts = Vec::new();

    for raw_part in body.split(&delimiter) {
        let part = raw_part.trim();
        if part.is_empty() || part == "--" {
            continue;
        }
        let parsed = parse_single_part(part)?;
        parts.push(parsed);
    }

    Ok(MultipartBody {
        boundary: boundary.into(),
        parts,
    })
}

fn parse_single_part(raw: &str) -> Result<MultipartPart, MultipartError> {
    /* find blank line separating headers from body */
    let sep = if raw.contains("\r\n\r\n") {
        "\r\n\r\n"
    } else {
        "\n\n"
    };
    let idx = raw.find(sep).ok_or(MultipartError::MalformedPart)?;
    let header_section = &raw[..idx];
    let body_section = &raw[idx + sep.len()..];

    let mut name = None;
    let mut filename = None;
    let mut content_type = "text/plain".to_owned();

    for line in header_section.lines() {
        let lower = line.to_lowercase();
        if lower.starts_with("content-disposition:") {
            name = extract_field(line, "name");
            filename = extract_field(line, "filename");
        } else if lower.starts_with("content-type:") {
            content_type = line
                .split_once(':')
                .map(|x| x.1)
                .unwrap_or("")
                .trim()
                .to_owned();
        }
    }

    Ok(MultipartPart {
        name,
        filename,
        content_type,
        data: body_section.as_bytes().to_vec(),
    })
}

fn extract_field(header_line: &str, field: &str) -> Option<String> {
    let pattern = format!("{}=\"", field);
    let start = header_line.find(&pattern)? + pattern.len();
    let end = header_line[start..].find('"')? + start;
    Some(header_line[start..end].to_owned())
}

/// Returns the total byte size of all parts.
pub fn total_body_bytes(body: &MultipartBody) -> usize {
    body.parts.iter().map(|p| p.data.len()).sum()
}

/// Finds a part by its name field.
pub fn find_part_by_name<'a>(body: &'a MultipartBody, name: &str) -> Option<&'a MultipartPart> {
    body.parts.iter().find(|p| p.name.as_deref() == Some(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_boundary_from_content_type() {
        let ct = "multipart/form-data; boundary=abc123";
        assert_eq!(extract_boundary(ct), Some("abc123".into()));
    }

    #[test]
    fn test_extract_boundary_quoted() {
        let ct = "multipart/form-data; boundary=\"mybound\"";
        assert_eq!(extract_boundary(ct), Some("mybound".into()));
    }

    #[test]
    fn test_parse_empty_body_error() {
        assert_eq!(parse_multipart("", "bound"), Err(MultipartError::EmptyBody));
    }

    #[test]
    fn test_parse_empty_boundary_error() {
        assert_eq!(
            parse_multipart("data", ""),
            Err(MultipartError::MissingBoundary)
        );
    }

    #[test]
    fn test_parse_single_part() {
        let body = "--bound\nContent-Disposition: form-data; name=\"field\"\n\nhello\n--bound--";
        let result = parse_multipart(body, "bound").expect("should succeed");
        assert_eq!(result.parts.len(), 1);
    }

    #[test]
    fn test_find_part_by_name() {
        let body = "--b\nContent-Disposition: form-data; name=\"username\"\n\nalice\n--b--";
        let result = parse_multipart(body, "b").expect("should succeed");
        let part = find_part_by_name(&result, "username");
        assert!(part.is_some());
    }

    #[test]
    fn test_total_body_bytes_sums_parts() {
        let body = MultipartBody {
            boundary: "b".into(),
            parts: vec![
                MultipartPart {
                    name: None,
                    filename: None,
                    content_type: "text/plain".into(),
                    data: vec![1, 2, 3],
                },
                MultipartPart {
                    name: None,
                    filename: None,
                    content_type: "text/plain".into(),
                    data: vec![4, 5],
                },
            ],
        };
        assert_eq!(total_body_bytes(&body), 5);
    }

    #[test]
    fn test_boundary_appears_in_parsed_body() {
        let body = "--mybound\nContent-Disposition: form-data; name=\"x\"\n\nval\n--mybound--";
        let result = parse_multipart(body, "mybound").expect("should succeed");
        assert_eq!(result.boundary, "mybound");
    }

    #[test]
    fn test_no_boundary_in_content_type_returns_none() {
        let ct = "application/json";
        assert!(extract_boundary(ct).is_none());
    }
}
