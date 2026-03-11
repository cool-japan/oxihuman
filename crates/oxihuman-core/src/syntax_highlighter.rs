// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Syntax highlight token classifier stub.
//!
//! Given a stream of tokens (string slices) and a language hint, assigns
//! a `HighlightKind` to each token for downstream rendering.

/// A syntax highlighting category.
#[derive(Debug, Clone, PartialEq)]
pub enum HighlightKind {
    Keyword,
    Identifier,
    Literal,
    StringLit,
    Comment,
    Punctuation,
    Operator,
    Number,
    Whitespace,
    Unknown,
}

/// A token with an associated highlight kind.
#[derive(Debug, Clone)]
pub struct HighlightToken {
    pub text: String,
    pub kind: HighlightKind,
}

/// Supported language modes for highlighting.
#[derive(Debug, Clone, PartialEq)]
pub enum Language {
    Rust,
    Python,
    Json,
    Plain,
}

/// Configuration for the syntax highlighter.
#[derive(Debug, Clone)]
pub struct HighlighterConfig {
    pub language: Language,
    pub case_sensitive_keywords: bool,
}

impl Default for HighlighterConfig {
    fn default() -> Self {
        Self {
            language: Language::Plain,
            case_sensitive_keywords: true,
        }
    }
}

static RUST_KEYWORDS: &[&str] = &[
    "fn", "let", "mut", "pub", "use", "mod", "struct", "enum", "impl", "trait", "if", "else",
    "match", "return", "for", "while", "loop", "in", "as", "where", "type", "const", "static",
    "self", "Self", "super", "crate", "async", "await", "move",
];

static PYTHON_KEYWORDS: &[&str] = &[
    "def", "class", "import", "from", "return", "if", "elif", "else", "for", "while", "in", "not",
    "and", "or", "with", "as", "pass", "break", "continue", "try", "except", "finally", "lambda",
    "yield", "None", "True", "False",
];

/// Classify a single token string given the language config.
pub fn classify_token(token: &str, cfg: &HighlighterConfig) -> HighlightKind {
    let keywords: &[&str] = match cfg.language {
        Language::Rust => RUST_KEYWORDS,
        Language::Python => PYTHON_KEYWORDS,
        Language::Json | Language::Plain => &[],
    };

    if keywords.contains(&token) {
        return HighlightKind::Keyword;
    }
    if token.starts_with("//") || token.starts_with('#') {
        return HighlightKind::Comment;
    }
    if (token.starts_with('"') && token.ends_with('"'))
        || (token.starts_with('\'') && token.ends_with('\''))
    {
        return HighlightKind::StringLit;
    }
    if token
        .chars()
        .all(|c| c.is_ascii_digit() || c == '.' || c == '_')
        && !token.is_empty()
    {
        return HighlightKind::Number;
    }
    if token.chars().all(char::is_whitespace) && !token.is_empty() {
        return HighlightKind::Whitespace;
    }
    if token.chars().all(|c| c.is_alphanumeric() || c == '_') && !token.is_empty() {
        return HighlightKind::Identifier;
    }
    if token.len() == 1 && "{}[]();,.<>".contains(token) {
        return HighlightKind::Punctuation;
    }
    if token.len() == 1 && "+-*/=!&|^~%".contains(token) {
        return HighlightKind::Operator;
    }
    HighlightKind::Unknown
}

/// Highlight a list of tokens, returning a `HighlightToken` per entry.
pub fn highlight_tokens(tokens: &[&str], cfg: &HighlighterConfig) -> Vec<HighlightToken> {
    tokens
        .iter()
        .map(|&t| HighlightToken {
            text: t.to_string(),
            kind: classify_token(t, cfg),
        })
        .collect()
}

/// Count tokens of a given kind in the result.
pub fn count_kind(tokens: &[HighlightToken], kind: &HighlightKind) -> usize {
    tokens.iter().filter(|t| &t.kind == kind).count()
}

/// Return a simple ANSI-colored representation (stub — only marks keywords).
pub fn to_ansi_string(tokens: &[HighlightToken]) -> String {
    let mut out = String::new();
    for t in tokens {
        if t.kind == HighlightKind::Keyword {
            out.push_str("\x1b[1;34m");
            out.push_str(&t.text);
            out.push_str("\x1b[0m");
        } else {
            out.push_str(&t.text);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rust_cfg() -> HighlighterConfig {
        HighlighterConfig {
            language: Language::Rust,
            case_sensitive_keywords: true,
        }
    }

    #[test]
    fn test_keyword_classified() {
        assert_eq!(classify_token("fn", &rust_cfg()), HighlightKind::Keyword);
    }

    #[test]
    fn test_identifier_classified() {
        assert_eq!(
            classify_token("my_var", &rust_cfg()),
            HighlightKind::Identifier
        );
    }

    #[test]
    fn test_number_classified() {
        assert_eq!(classify_token("42", &rust_cfg()), HighlightKind::Number);
    }

    #[test]
    fn test_comment_classified() {
        assert_eq!(
            classify_token("// a comment", &rust_cfg()),
            HighlightKind::Comment
        );
    }

    #[test]
    fn test_string_classified() {
        assert_eq!(
            classify_token("\"hello\"", &rust_cfg()),
            HighlightKind::StringLit
        );
    }

    #[test]
    fn test_highlight_tokens_count() {
        let tokens = ["fn", "main", "(", ")"];
        let cfg = rust_cfg();
        let ht = highlight_tokens(&tokens, &cfg);
        assert_eq!(ht.len(), 4);
    }

    #[test]
    fn test_count_kind() {
        let tokens = ["fn", "let", "x"];
        let cfg = rust_cfg();
        let ht = highlight_tokens(&tokens, &cfg);
        assert_eq!(count_kind(&ht, &HighlightKind::Keyword), 2);
    }

    #[test]
    fn test_ansi_string_contains_escape() {
        let tokens = ["fn"];
        let cfg = rust_cfg();
        let ht = highlight_tokens(&tokens, &cfg);
        let s = to_ansi_string(&ht);
        assert!(s.contains("\x1b["));
    }

    #[test]
    fn test_python_keyword() {
        let cfg = HighlighterConfig {
            language: Language::Python,
            case_sensitive_keywords: true,
        };
        assert_eq!(classify_token("def", &cfg), HighlightKind::Keyword);
    }
}
