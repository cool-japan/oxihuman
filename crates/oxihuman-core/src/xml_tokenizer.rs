// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! XML SAX-style tokenizer stub.

/// A single XML token.
#[derive(Debug, Clone, PartialEq)]
pub enum XmlToken {
    /// `<?xml ... ?>`
    Declaration(String),
    /// `<tag attr="v">`
    StartTag {
        name: String,
        attrs: Vec<(String, String)>,
    },
    /// `</tag>`
    EndTag(String),
    /// `<tag/>`
    EmptyTag {
        name: String,
        attrs: Vec<(String, String)>,
    },
    /// Text content between tags.
    Text(String),
    /// `<!-- comment -->`
    Comment(String),
    /// `<![CDATA[...]]>`
    CData(String),
}

/// XML tokenizer error.
#[derive(Debug, Clone, PartialEq)]
pub struct XmlError {
    pub position: usize,
    pub message: String,
}

impl std::fmt::Display for XmlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "XML error at position {}: {}",
            self.position, self.message
        )
    }
}

/// A SAX-style XML tokenizer.
#[derive(Debug)]
pub struct XmlTokenizer {
    input: Vec<char>,
    pos: usize,
}

impl XmlTokenizer {
    /// Create a new tokenizer for the given input.
    pub fn new(input: &str) -> Self {
        XmlTokenizer {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    /// Return `true` if all input has been consumed.
    pub fn is_done(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Return current position.
    pub fn position(&self) -> usize {
        self.pos
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.input.get(self.pos).copied();
        self.pos += 1;
        ch
    }

    fn consume_until(&mut self, stop: &str) -> String {
        let stop_chars: Vec<char> = stop.chars().collect();
        let mut buf = String::new();
        while self.pos + stop_chars.len() <= self.input.len() {
            let window: Vec<char> = self.input[self.pos..self.pos + stop_chars.len()].to_vec();
            if window == stop_chars {
                self.pos += stop_chars.len();
                break;
            }
            buf.push(self.input[self.pos]);
            self.pos += 1;
        }
        buf
    }

    /// Collect all tokens from the input.
    pub fn tokenize(&mut self) -> Result<Vec<XmlToken>, XmlError> {
        let mut tokens = vec![];
        while !self.is_done() {
            if self.peek() == Some('<') {
                self.pos += 1; /* consume '<' */
                if self.pos >= self.input.len() {
                    break;
                }
                /* comment */
                if self.input[self.pos..].starts_with(&['!', '-', '-']) {
                    self.pos += 3;
                    let text = self.consume_until("-->");
                    tokens.push(XmlToken::Comment(text));
                /* CDATA */
                } else if self.input[self.pos..]
                    .starts_with(&['!', '[', 'C', 'D', 'A', 'T', 'A', '['])
                {
                    self.pos += 8;
                    let text = self.consume_until("]]>");
                    tokens.push(XmlToken::CData(text));
                /* declaration */
                } else if self.peek() == Some('?') {
                    self.pos += 1;
                    let text = self.consume_until("?>");
                    tokens.push(XmlToken::Declaration(text));
                /* end tag */
                } else if self.peek() == Some('/') {
                    self.pos += 1;
                    let name: String = self.input[self.pos..]
                        .iter()
                        .take_while(|&&c| c != '>')
                        .collect();
                    self.pos += name.len() + 1;
                    tokens.push(XmlToken::EndTag(name.trim().to_string()));
                } else {
                    /* start or empty tag — stub: read until '>' */
                    let raw: String = self.input[self.pos..]
                        .iter()
                        .take_while(|&&c| c != '>')
                        .collect();
                    self.pos += raw.len() + 1;
                    let is_empty = raw.ends_with('/');
                    let raw = raw.trim_end_matches('/').trim();
                    let mut parts = raw.splitn(2, char::is_whitespace);
                    let name = parts.next().unwrap_or("").to_string();
                    /* attrs stub — skip parsing */
                    let attrs = vec![];
                    if is_empty {
                        tokens.push(XmlToken::EmptyTag { name, attrs });
                    } else {
                        tokens.push(XmlToken::StartTag { name, attrs });
                    }
                }
            } else {
                /* text content */
                let text: String = self.input[self.pos..]
                    .iter()
                    .take_while(|&&c| c != '<')
                    .collect();
                self.pos += text.len();
                if !text.is_empty() {
                    tokens.push(XmlToken::Text(text));
                }
            }
        }
        Ok(tokens)
    }
}

/// Count tokens of a specific variant name in a list.
pub fn count_start_tags(tokens: &[XmlToken]) -> usize {
    tokens
        .iter()
        .filter(|t| matches!(t, XmlToken::StartTag { .. }))
        .count()
}

/// Count end tags.
pub fn count_end_tags(tokens: &[XmlToken]) -> usize {
    tokens
        .iter()
        .filter(|t| matches!(t, XmlToken::EndTag(_)))
        .count()
}

/// Collect all text content strings.
pub fn collect_text(tokens: &[XmlToken]) -> Vec<&str> {
    tokens
        .iter()
        .filter_map(|t| {
            if let XmlToken::Text(s) = t {
                Some(s.as_str())
            } else {
                None
            }
        })
        .collect()
}

/// Return `true` if the token list represents a well-formed document (stub check).
pub fn is_balanced(tokens: &[XmlToken]) -> bool {
    count_start_tags(tokens) == count_end_tags(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        /* empty input produces no tokens */
        let mut tok = XmlTokenizer::new("");
        assert!(tok.tokenize().expect("should succeed").is_empty());
    }

    #[test]
    fn test_simple_element() {
        /* start + text + end */
        let mut tok = XmlTokenizer::new("<root>hello</root>");
        let tokens = tok.tokenize().expect("should succeed");
        assert!(count_start_tags(&tokens) > 0);
        assert!(count_end_tags(&tokens) > 0);
    }

    #[test]
    fn test_balanced() {
        /* balanced tag check */
        let mut tok = XmlTokenizer::new("<a>text</a>");
        let tokens = tok.tokenize().expect("should succeed");
        assert!(is_balanced(&tokens));
    }

    #[test]
    fn test_comment_token() {
        /* comment produces Comment token */
        let mut tok = XmlTokenizer::new("<!-- hi -->");
        let tokens = tok.tokenize().expect("should succeed");
        assert!(matches!(tokens.first(), Some(XmlToken::Comment(_))));
    }

    #[test]
    fn test_empty_tag() {
        /* self-closing tag */
        let mut tok = XmlTokenizer::new("<br/>");
        let tokens = tok.tokenize().expect("should succeed");
        assert!(matches!(tokens.first(), Some(XmlToken::EmptyTag { .. })));
    }

    #[test]
    fn test_text_collection() {
        /* collect_text extracts text nodes */
        let mut tok = XmlTokenizer::new("<x>world</x>");
        let tokens = tok.tokenize().expect("should succeed");
        let texts = collect_text(&tokens);
        assert!(!texts.is_empty());
    }

    #[test]
    fn test_declaration_token() {
        /* XML declaration */
        let mut tok = XmlTokenizer::new("<?xml version=\"1.0\"?>");
        let tokens = tok.tokenize().expect("should succeed");
        assert!(matches!(tokens.first(), Some(XmlToken::Declaration(_))));
    }

    #[test]
    fn test_count_start_end_symmetry() {
        /* nested tags counted correctly */
        let mut tok = XmlTokenizer::new("<a><b></b></a>");
        let tokens = tok.tokenize().expect("should succeed");
        assert_eq!(count_start_tags(&tokens), count_end_tags(&tokens));
    }

    #[test]
    fn test_position_advances() {
        /* position moves forward as tokens are consumed */
        let mut tok = XmlTokenizer::new("<tag/>");
        tok.tokenize().expect("should succeed");
        assert!(tok.position() > 0);
    }

    #[test]
    fn test_is_done_after_all_input() {
        /* is_done returns true after tokenizing */
        let mut tok = XmlTokenizer::new("<x/>");
        tok.tokenize().expect("should succeed");
        assert!(tok.is_done());
    }
}
