// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Minimal PEG-style parser combinator.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ParseNode {
    Literal(String),
    Sequence(Vec<ParseNode>),
    Choice(Box<ParseNode>),
    Empty,
}

#[allow(dead_code)]
pub type ParseResult<'a> = Option<(ParseNode, &'a str)>;

/// Match a literal string at the start of input.
#[allow(dead_code)]
pub fn parse_literal<'a>(input: &'a str, lit: &str) -> ParseResult<'a> {
    if let Some(rest) = input.strip_prefix(lit) {
        Some((ParseNode::Literal(lit.to_string()), rest))
    } else {
        None
    }
}

/// Skip leading whitespace.
#[allow(dead_code)]
pub fn skip_whitespace(input: &str) -> &str {
    input.trim_start()
}

/// Match a decimal integer.
#[allow(dead_code)]
pub fn parse_integer(input: &str) -> ParseResult<'_> {
    let s = skip_whitespace(input);
    let end = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    if end == 0 {
        return None;
    }
    Some((ParseNode::Literal(s[..end].to_string()), &s[end..]))
}

/// Match a sequence of alphanumeric+underscore chars.
#[allow(dead_code)]
pub fn parse_ident(input: &str) -> ParseResult<'_> {
    let s = skip_whitespace(input);
    let end = s
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(s.len());
    if end == 0 {
        return None;
    }
    Some((ParseNode::Literal(s[..end].to_string()), &s[end..]))
}

/// Match an optional rule (returns Empty if fails).
#[allow(dead_code)]
pub fn parse_opt<'a, F>(input: &'a str, f: F) -> (ParseNode, &'a str)
where
    F: Fn(&'a str) -> ParseResult<'a>,
{
    if let Some((n, rest)) = f(input) {
        (n, rest)
    } else {
        (ParseNode::Empty, input)
    }
}

/// Try left then right, return first success.
#[allow(dead_code)]
pub fn parse_choice<'a, F, G>(input: &'a str, f: F, g: G) -> ParseResult<'a>
where
    F: Fn(&'a str) -> ParseResult<'a>,
    G: Fn(&'a str) -> ParseResult<'a>,
{
    f(input).or_else(|| g(input))
}

/// Parse zero or more idents separated by a delimiter.
#[allow(dead_code)]
pub fn parse_list<'a>(input: &'a str, delim: &str) -> (Vec<String>, &'a str) {
    let mut items = Vec::new();
    let mut cur = skip_whitespace(input);
    while let Some((ParseNode::Literal(s), rest)) = parse_ident(cur) {
        items.push(s);
        let rest2 = skip_whitespace(rest);
        if let Some(after_delim) = rest2.strip_prefix(delim) {
            cur = skip_whitespace(after_delim);
        } else {
            cur = rest2;
            break;
        }
    }
    (items, cur)
}

/// Estimate parse depth/complexity of a string (for diagnostics).
#[allow(dead_code)]
pub fn parse_depth(input: &str) -> usize {
    let mut depth = 0usize;
    let mut max_depth = 0usize;
    for c in input.chars() {
        match c {
            '(' | '[' | '{' => {
                depth += 1;
                if depth > max_depth {
                    max_depth = depth;
                }
            }
            ')' | ']' | '}' => {
                depth = depth.saturating_sub(1);
            }
            _ => {}
        }
    }
    max_depth
}

#[allow(dead_code)]
pub fn node_text(n: &ParseNode) -> Option<&str> {
    if let ParseNode::Literal(s) = n {
        Some(s.as_str())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_literal_success() {
        let (node, rest) = parse_literal("hello world", "hello").unwrap();
        assert_eq!(node_text(&node), Some("hello"));
        assert_eq!(rest, " world");
    }
    #[test]
    fn test_parse_literal_fail() {
        assert!(parse_literal("world", "hello").is_none());
    }
    #[test]
    fn test_parse_integer() {
        let (node, rest) = parse_integer("123abc").unwrap();
        assert_eq!(node_text(&node), Some("123"));
        assert_eq!(rest, "abc");
    }
    #[test]
    fn test_parse_ident() {
        let (node, rest) = parse_ident("foo_bar 42").unwrap();
        assert_eq!(node_text(&node), Some("foo_bar"));
        assert_eq!(rest, " 42");
    }
    #[test]
    fn test_parse_opt_success() {
        let (node, _rest) = parse_opt("hello", |i| parse_literal(i, "hello"));
        assert_ne!(node, ParseNode::Empty);
    }
    #[test]
    fn test_parse_opt_fail() {
        let (node, rest) = parse_opt("world", |i| parse_literal(i, "hello"));
        assert_eq!(node, ParseNode::Empty);
        assert_eq!(rest, "world");
    }
    #[test]
    fn test_parse_choice() {
        let r = parse_choice("42xyz", |i| parse_integer(i), |i| parse_ident(i));
        assert!(r.is_some_and(|(n, _)| node_text(&n) == Some("42")));
    }
    #[test]
    fn test_parse_list() {
        let (items, _) = parse_list("a, b, c", ",");
        assert_eq!(
            items,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }
    #[test]
    fn test_parse_depth() {
        assert_eq!(parse_depth("((()))"), 3);
        assert_eq!(parse_depth("[]{}"), 1);
        assert_eq!(parse_depth("no brackets"), 0);
    }
    #[test]
    fn test_skip_whitespace() {
        assert_eq!(skip_whitespace("   hello"), "hello");
    }
}
