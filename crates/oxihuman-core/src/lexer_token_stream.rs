// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Lexer token stream abstraction.
//!
//! Provides a position-tracking, peekable stream over a sequence of lexer tokens
//! with support for mark/restore checkpointing, lookahead, and batch consumption.

/// A basic token produced by a lexer.
#[derive(Debug, Clone, PartialEq)]
pub struct LexToken {
    pub kind: LexTokenKind,
    pub text: String,
    pub line: usize,
    pub col: usize,
}

impl LexToken {
    pub fn new(kind: LexTokenKind, text: &str, line: usize, col: usize) -> Self {
        Self {
            kind,
            text: text.to_string(),
            line,
            col,
        }
    }

    pub fn is_eof(&self) -> bool {
        self.kind == LexTokenKind::Eof
    }
}

/// Categories of lexer tokens.
#[derive(Debug, Clone, PartialEq)]
pub enum LexTokenKind {
    Word,
    Number,
    Punctuation,
    Whitespace,
    Newline,
    Eof,
    Custom(String),
}

/// A position-tracking stream over a `Vec<LexToken>`.
#[derive(Debug, Clone)]
pub struct LexerStream {
    tokens: Vec<LexToken>,
    pos: usize,
    mark: Option<usize>,
}

impl LexerStream {
    pub fn new(tokens: Vec<LexToken>) -> Self {
        Self {
            tokens,
            pos: 0,
            mark: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    pub fn remaining(&self) -> usize {
        self.tokens.len().saturating_sub(self.pos)
    }

    pub fn current_pos(&self) -> usize {
        self.pos
    }

    pub fn peek(&self) -> Option<&LexToken> {
        self.tokens.get(self.pos)
    }

    pub fn peek_nth(&self, n: usize) -> Option<&LexToken> {
        self.tokens.get(self.pos + n)
    }

    pub fn next_token(&mut self) -> Option<&LexToken> {
        let t = self.tokens.get(self.pos);
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    pub fn skip(&mut self, n: usize) {
        self.pos = (self.pos + n).min(self.tokens.len());
    }

    pub fn set_mark(&mut self) {
        self.mark = Some(self.pos);
    }

    pub fn restore_mark(&mut self) {
        if let Some(m) = self.mark {
            self.pos = m;
        }
    }

    pub fn clear_mark(&mut self) {
        self.mark = None;
    }

    /// Consume tokens while the predicate holds.
    pub fn consume_while(&mut self, pred: impl Fn(&LexToken) -> bool) -> Vec<&LexToken> {
        let mut result = Vec::new();
        while let Some(t) = self.tokens.get(self.pos) {
            if pred(t) {
                result.push(t);
                self.pos += 1;
            } else {
                break;
            }
        }
        result
    }

    pub fn total(&self) -> usize {
        self.tokens.len()
    }

    /// Return all remaining tokens as a slice without consuming.
    pub fn peek_rest(&self) -> &[LexToken] {
        &self.tokens[self.pos..]
    }
}

/// Build a `LexerStream` from a string by naive whitespace-based tokenization.
pub fn lex_string(text: &str) -> LexerStream {
    let mut tokens: Vec<LexToken> = Vec::new();
    let mut line = 1usize;
    let mut col = 1usize;

    for word in text.split_whitespace() {
        let kind = if word.chars().all(|c| c.is_ascii_digit()) {
            LexTokenKind::Number
        } else if word.chars().all(|c| c.is_alphanumeric() || c == '_') {
            LexTokenKind::Word
        } else {
            LexTokenKind::Punctuation
        };
        tokens.push(LexToken::new(kind, word, line, col));
        col += word.len() + 1;
        if word.contains('\n') {
            line += 1;
            col = 1;
        }
    }
    tokens.push(LexToken::new(LexTokenKind::Eof, "", line, col));
    LexerStream::new(tokens)
}

/// Count tokens of a specific kind.
pub fn count_tokens_of_kind(stream: &LexerStream, kind: &LexTokenKind) -> usize {
    stream.tokens.iter().filter(|t| &t.kind == kind).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_produces_eof() {
        let s = lex_string("hello world");
        assert!(s.tokens.last().map(|t| t.is_eof()).unwrap_or(false));
    }

    #[test]
    fn test_lex_word_count() {
        let s = lex_string("one two three");
        assert_eq!(count_tokens_of_kind(&s, &LexTokenKind::Word), 3);
    }

    #[test]
    fn test_peek_does_not_advance() {
        let mut s = lex_string("a b");
        let first = s.peek().unwrap().text.clone();
        let _ = s.peek();
        let next = s.next_token().unwrap().text.clone();
        assert_eq!(first, next);
    }

    #[test]
    fn test_skip_advances() {
        let mut s = lex_string("a b c");
        s.skip(1);
        assert_eq!(s.current_pos(), 1);
    }

    #[test]
    fn test_mark_restore() {
        let mut s = lex_string("a b c");
        s.next_token();
        s.set_mark();
        s.next_token();
        s.restore_mark();
        assert_eq!(s.current_pos(), 1);
    }

    #[test]
    fn test_remaining_decreases() {
        let mut s = lex_string("a b");
        let before = s.remaining();
        s.next_token();
        assert!(s.remaining() < before);
    }

    #[test]
    fn test_peek_nth() {
        let s = lex_string("x y z");
        assert_eq!(s.peek_nth(1).map(|t| t.text.as_str()), Some("y"));
    }

    #[test]
    fn test_consume_while() {
        let mut s = lex_string("1 2 3 word");
        let nums = s.consume_while(|t| t.kind == LexTokenKind::Number);
        assert_eq!(nums.len(), 3);
    }

    #[test]
    fn test_is_empty_after_all_consumed() {
        let mut s = lex_string("a");
        /* Consume word + EOF */
        while !s.is_empty() {
            s.next_token();
        }
        assert!(s.is_empty());
    }
}
