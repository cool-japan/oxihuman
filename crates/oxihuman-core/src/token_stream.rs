// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Simple token stream for lexing / parsing utilities.

/// A single token produced by the lexer.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub pos: usize,
}

/// Token category.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Ident,
    Number,
    Punct,
    Str,
    Eof,
}

/// Token stream backed by a Vec.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TokenStream {
    tokens: Vec<Token>,
    cursor: usize,
}

/// Create a new empty `TokenStream`.
#[allow(dead_code)]
pub fn new_token_stream() -> TokenStream {
    TokenStream {
        tokens: Vec::new(),
        cursor: 0,
    }
}

/// Push a token onto the stream.
#[allow(dead_code)]
pub fn tks_push(ts: &mut TokenStream, kind: TokenKind, text: &str, pos: usize) {
    ts.tokens.push(Token {
        kind,
        text: text.to_string(),
        pos,
    });
}

/// Peek at the current token without consuming.
#[allow(dead_code)]
pub fn tks_peek(ts: &TokenStream) -> Option<&Token> {
    ts.tokens.get(ts.cursor)
}

/// Consume and return the next token.
#[allow(dead_code)]
pub fn tks_next(ts: &mut TokenStream) -> Option<Token> {
    if ts.cursor < ts.tokens.len() {
        let tok = ts.tokens[ts.cursor].clone();
        ts.cursor += 1;
        Some(tok)
    } else {
        None
    }
}

/// Whether the stream is exhausted.
#[allow(dead_code)]
pub fn tks_is_empty(ts: &TokenStream) -> bool {
    ts.cursor >= ts.tokens.len()
}

/// Number of remaining tokens.
#[allow(dead_code)]
pub fn tks_remaining(ts: &TokenStream) -> usize {
    ts.tokens.len().saturating_sub(ts.cursor)
}

/// Rewind to the beginning.
#[allow(dead_code)]
pub fn tks_rewind(ts: &mut TokenStream) {
    ts.cursor = 0;
}

/// Total token count (including consumed).
#[allow(dead_code)]
pub fn tks_total(ts: &TokenStream) -> usize {
    ts.tokens.len()
}

/// Skip tokens while predicate holds.
#[allow(dead_code)]
pub fn tks_skip_while(ts: &mut TokenStream, pred: impl Fn(&Token) -> bool) {
    while ts.cursor < ts.tokens.len() && pred(&ts.tokens[ts.cursor]) {
        ts.cursor += 1;
    }
}

/// Collect all remaining tokens as a Vec.
#[allow(dead_code)]
pub fn tks_drain(ts: &mut TokenStream) -> Vec<Token> {
    let rest = ts.tokens[ts.cursor..].to_vec();
    ts.cursor = ts.tokens.len();
    rest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_stream() {
        let ts = new_token_stream();
        assert!(tks_is_empty(&ts));
        assert_eq!(tks_remaining(&ts), 0);
    }

    #[test]
    fn test_push_and_peek() {
        let mut ts = new_token_stream();
        tks_push(&mut ts, TokenKind::Ident, "hello", 0);
        let tok = tks_peek(&ts).expect("should succeed");
        assert_eq!(tok.text, "hello".to_string());
    }

    #[test]
    fn test_next_consumes() {
        let mut ts = new_token_stream();
        tks_push(&mut ts, TokenKind::Number, "42", 0);
        let tok = tks_next(&mut ts).expect("should succeed");
        assert_eq!(tok.kind, TokenKind::Number);
        assert!(tks_is_empty(&ts));
    }

    #[test]
    fn test_remaining_decreases() {
        let mut ts = new_token_stream();
        tks_push(&mut ts, TokenKind::Ident, "a", 0);
        tks_push(&mut ts, TokenKind::Ident, "b", 1);
        assert_eq!(tks_remaining(&ts), 2);
        tks_next(&mut ts);
        assert_eq!(tks_remaining(&ts), 1);
    }

    #[test]
    fn test_rewind() {
        let mut ts = new_token_stream();
        tks_push(&mut ts, TokenKind::Ident, "x", 0);
        tks_next(&mut ts);
        tks_rewind(&mut ts);
        assert_eq!(tks_remaining(&ts), 1);
    }

    #[test]
    fn test_total_count() {
        let mut ts = new_token_stream();
        tks_push(&mut ts, TokenKind::Ident, "a", 0);
        tks_push(&mut ts, TokenKind::Ident, "b", 0);
        assert_eq!(tks_total(&ts), 2);
    }

    #[test]
    fn test_skip_while() {
        let mut ts = new_token_stream();
        tks_push(&mut ts, TokenKind::Ident, "a", 0);
        tks_push(&mut ts, TokenKind::Ident, "b", 1);
        tks_push(&mut ts, TokenKind::Number, "1", 2);
        tks_skip_while(&mut ts, |t| t.kind == TokenKind::Ident);
        let tok = tks_peek(&ts).expect("should succeed");
        assert_eq!(tok.kind, TokenKind::Number);
    }

    #[test]
    fn test_drain() {
        let mut ts = new_token_stream();
        tks_push(&mut ts, TokenKind::Punct, ";", 0);
        tks_push(&mut ts, TokenKind::Eof, "", 1);
        let drained = tks_drain(&mut ts);
        assert_eq!(drained.len(), 2);
        assert!(tks_is_empty(&ts));
    }

    #[test]
    fn test_next_none_on_empty() {
        let mut ts = new_token_stream();
        assert!(tks_next(&mut ts).is_none());
    }
}
