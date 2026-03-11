#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Simple whitespace/punctuation tokenizer.
//! kind: 0=word, 1=number, 2=punct, 3=whitespace

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: u8,
    pub text: String,
}

#[allow(dead_code)]
pub fn tokenize(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = text.char_indices().peekable();
    while let Some((i, ch)) = chars.next() {
        if ch.is_whitespace() {
            let mut s = ch.to_string();
            while let Some(&(_, nc)) = chars.peek() {
                if nc.is_whitespace() {
                    s.push(nc);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(Token { kind: 3, text: s });
        } else if ch.is_ascii_digit()
            || (ch == '-'
                && chars
                    .peek()
                    .map(|&(_, c)| c.is_ascii_digit())
                    .unwrap_or(false))
        {
            let mut s = ch.to_string();
            while let Some(&(_, nc)) = chars.peek() {
                if nc.is_ascii_digit() || nc == '.' {
                    s.push(nc);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(Token { kind: 1, text: s });
        } else if ch.is_alphabetic() || ch == '_' {
            let _ = i;
            let mut s = ch.to_string();
            while let Some(&(_, nc)) = chars.peek() {
                if nc.is_alphanumeric() || nc == '_' {
                    s.push(nc);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(Token { kind: 0, text: s });
        } else {
            tokens.push(Token {
                kind: 2,
                text: ch.to_string(),
            });
        }
    }
    tokens
}

#[allow(dead_code)]
pub fn token_words(tokens: &[Token]) -> Vec<&str> {
    tokens
        .iter()
        .filter(|t| t.kind == 0)
        .map(|t| t.text.as_str())
        .collect()
}

#[allow(dead_code)]
pub fn token_numbers(tokens: &[Token]) -> Vec<f64> {
    tokens
        .iter()
        .filter(|t| t.kind == 1)
        .filter_map(|t| t.text.parse::<f64>().ok())
        .collect()
}

#[allow(dead_code)]
pub fn token_count(tokens: &[Token]) -> usize {
    tokens.len()
}

#[allow(dead_code)]
pub fn is_numeric_token(t: &Token) -> bool {
    t.kind == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_words() {
        let tokens = tokenize("hello world");
        let words = token_words(&tokens);
        assert!(words.contains(&"hello"));
        assert!(words.contains(&"world"));
    }

    #[test]
    fn tokenize_numbers() {
        let tokens = tokenize("abc 42 xyz");
        let nums = token_numbers(&tokens);
        assert_eq!(nums.len(), 1);
        assert!((nums[0] - 42.0).abs() < 1e-9);
    }

    #[test]
    fn tokenize_punctuation() {
        let tokens = tokenize("a,b.c");
        let puncts: Vec<_> = tokens.iter().filter(|t| t.kind == 2).collect();
        assert!(!puncts.is_empty());
    }

    #[test]
    fn tokenize_whitespace() {
        let tokens = tokenize("  ");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, 3);
    }

    #[test]
    fn token_count_correct() {
        let tokens = tokenize("one 2 three");
        // one(word) + space + 2(num) + space + three(word) = 5
        assert_eq!(token_count(&tokens), 5);
    }

    #[test]
    fn is_numeric_token_true() {
        let t = Token {
            kind: 1,
            text: "3.14".to_string(),
        };
        assert!(is_numeric_token(&t));
    }

    #[test]
    fn is_numeric_token_false() {
        let t = Token {
            kind: 0,
            text: "hello".to_string(),
        };
        assert!(!is_numeric_token(&t));
    }

    #[test]
    fn tokenize_empty() {
        let tokens = tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn float_number() {
        // Use a simple integer-like float to avoid approx_constant lint
        let tokens = tokenize("2.5");
        let nums = token_numbers(&tokens);
        assert_eq!(nums.len(), 1);
        assert!((nums[0] - 2.5).abs() < 1e-5);
    }

    #[test]
    fn multiple_numbers() {
        let tokens = tokenize("1 22 333");
        let nums = token_numbers(&tokens);
        assert_eq!(nums.len(), 3);
    }
}
