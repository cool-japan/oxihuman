// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! HTTP/1.1 request/response parser stub.

/// An HTTP method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Patch,
    Other(String),
}

impl HttpMethod {
    /// Parse from a string slice.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_ascii_uppercase().as_str() {
            "GET" => Self::Get,
            "POST" => Self::Post,
            "PUT" => Self::Put,
            "DELETE" => Self::Delete,
            "HEAD" => Self::Head,
            "OPTIONS" => Self::Options,
            "PATCH" => Self::Patch,
            other => Self::Other(other.to_string()),
        }
    }
}

/// An HTTP header (name + value pair).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpHeader {
    pub name: String,
    pub value: String,
}

/// A parsed HTTP/1.1 request.
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub version: String,
    pub headers: Vec<HttpHeader>,
    pub body: Vec<u8>,
}

/// A parsed HTTP/1.1 response.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub version: String,
    pub status_code: u16,
    pub reason: String,
    pub headers: Vec<HttpHeader>,
    pub body: Vec<u8>,
}

/// HTTP parse error.
#[derive(Debug, Clone, PartialEq)]
pub enum HttpError {
    MalformedRequestLine,
    MalformedStatusLine,
    MalformedHeader(String),
    InvalidStatusCode(String),
    UnexpectedEnd,
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedRequestLine => write!(f, "malformed HTTP request line"),
            Self::MalformedStatusLine => write!(f, "malformed HTTP status line"),
            Self::MalformedHeader(s) => write!(f, "malformed HTTP header: {s}"),
            Self::InvalidStatusCode(s) => write!(f, "invalid HTTP status code: {s}"),
            Self::UnexpectedEnd => write!(f, "unexpected end of HTTP message"),
        }
    }
}

/// Parse an HTTP/1.1 request from raw bytes.
pub fn parse_request(raw: &[u8]) -> Result<HttpRequest, HttpError> {
    let text = std::str::from_utf8(raw).map_err(|_| HttpError::MalformedRequestLine)?;
    let mut lines = text.split("\r\n");
    let request_line = lines.next().ok_or(HttpError::UnexpectedEnd)?;
    let mut parts = request_line.splitn(3, ' ');
    let method = parts
        .next()
        .ok_or(HttpError::MalformedRequestLine)
        .map(HttpMethod::from_str)?;
    let path = parts
        .next()
        .ok_or(HttpError::MalformedRequestLine)?
        .to_string();
    let version = parts
        .next()
        .ok_or(HttpError::MalformedRequestLine)?
        .to_string();
    let mut headers = vec![];
    for line in lines.by_ref() {
        if line.is_empty() {
            break;
        }
        let mut h = line.splitn(2, ':');
        let name = h.next().unwrap_or("").trim().to_string();
        let value = h.next().unwrap_or("").trim().to_string();
        headers.push(HttpHeader { name, value });
    }
    Ok(HttpRequest {
        method,
        path,
        version,
        headers,
        body: vec![],
    })
}

/// Parse an HTTP/1.1 response from raw bytes.
pub fn parse_response(raw: &[u8]) -> Result<HttpResponse, HttpError> {
    let text = std::str::from_utf8(raw).map_err(|_| HttpError::MalformedStatusLine)?;
    let mut lines = text.split("\r\n");
    let status_line = lines.next().ok_or(HttpError::UnexpectedEnd)?;
    let mut parts = status_line.splitn(3, ' ');
    let version = parts
        .next()
        .ok_or(HttpError::MalformedStatusLine)?
        .to_string();
    let code_str = parts.next().ok_or(HttpError::MalformedStatusLine)?;
    let status_code = code_str
        .parse::<u16>()
        .map_err(|_| HttpError::InvalidStatusCode(code_str.to_string()))?;
    let reason = parts.next().unwrap_or("").to_string();
    let mut headers = vec![];
    for line in lines.by_ref() {
        if line.is_empty() {
            break;
        }
        let mut h = line.splitn(2, ':');
        let name = h.next().unwrap_or("").trim().to_string();
        let value = h.next().unwrap_or("").trim().to_string();
        headers.push(HttpHeader { name, value });
    }
    Ok(HttpResponse {
        version,
        status_code,
        reason,
        headers,
        body: vec![],
    })
}

/// Look up a header value by name (case-insensitive).
pub fn find_header<'a>(headers: &'a [HttpHeader], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|h| h.name.eq_ignore_ascii_case(name))
        .map(|h| h.value.as_str())
}

/// Return `true` if the request uses HTTP/1.1.
pub fn is_http11(req: &HttpRequest) -> bool {
    req.version == "HTTP/1.1"
}

/// Return the Content-Length header value if present.
pub fn content_length(headers: &[HttpHeader]) -> Option<usize> {
    find_header(headers, "content-length").and_then(|v| v.trim().parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_get_request() {
        /* basic GET request parsed */
        let raw = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
        let req = parse_request(raw).expect("should succeed");
        assert_eq!(req.method, HttpMethod::Get);
        assert_eq!(req.path, "/");
    }

    #[test]
    fn test_parse_response_200() {
        /* 200 OK response parsed */
        let raw = b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";
        let resp = parse_response(raw).expect("should succeed");
        assert_eq!(resp.status_code, 200);
    }

    #[test]
    fn test_find_header_case_insensitive() {
        /* header lookup is case-insensitive */
        let headers = vec![HttpHeader {
            name: "Content-Type".to_string(),
            value: "text/html".to_string(),
        }];
        assert_eq!(find_header(&headers, "content-type"), Some("text/html"));
    }

    #[test]
    fn test_is_http11_true() {
        /* HTTP/1.1 version detected */
        let raw = b"GET / HTTP/1.1\r\n\r\n";
        let req = parse_request(raw).expect("should succeed");
        assert!(is_http11(&req));
    }

    #[test]
    fn test_content_length_header() {
        /* content_length parses header value */
        let headers = vec![HttpHeader {
            name: "Content-Length".to_string(),
            value: "42".to_string(),
        }];
        assert_eq!(content_length(&headers), Some(42));
    }

    #[test]
    fn test_method_post() {
        /* POST method parsed */
        let raw = b"POST /data HTTP/1.1\r\n\r\n";
        let req = parse_request(raw).expect("should succeed");
        assert_eq!(req.method, HttpMethod::Post);
    }

    #[test]
    fn test_invalid_status_code() {
        /* non-numeric status code returns error */
        let raw = b"HTTP/1.1 OK notanumber\r\n\r\n";
        assert!(parse_response(raw).is_err());
    }

    #[test]
    fn test_multiple_headers() {
        /* multiple headers parsed */
        let raw = b"GET / HTTP/1.1\r\nHost: x\r\nAccept: */*\r\n\r\n";
        let req = parse_request(raw).expect("should succeed");
        assert_eq!(req.headers.len(), 2);
    }

    #[test]
    fn test_find_header_missing() {
        /* missing header returns None */
        let headers: Vec<HttpHeader> = vec![];
        assert!(find_header(&headers, "x-custom").is_none());
    }

    #[test]
    fn test_parse_response_404() {
        /* 404 status code parsed */
        let raw = b"HTTP/1.1 404 Not Found\r\n\r\n";
        let resp = parse_response(raw).expect("should succeed");
        assert_eq!(resp.status_code, 404);
        assert_eq!(resp.reason, "Not Found");
    }
}
