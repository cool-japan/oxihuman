// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Character classification utilities.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum CharClass {
    Alpha,
    Digit,
    AlphaNum,
    Whitespace,
    Punctuation,
    Other,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClassifierConfig {
    pub locale_aware: bool,
}

#[allow(dead_code)]
pub fn default_classifier_config() -> ClassifierConfig {
    ClassifierConfig {
        locale_aware: false,
    }
}

#[allow(dead_code)]
pub fn classify_char(c: char) -> CharClass {
    if c.is_ascii_alphabetic() {
        CharClass::Alpha
    } else if c.is_ascii_digit() {
        CharClass::Digit
    } else if c.is_ascii_alphanumeric() {
        CharClass::AlphaNum
    } else if c.is_whitespace() {
        CharClass::Whitespace
    } else if c.is_ascii_punctuation() {
        CharClass::Punctuation
    } else {
        CharClass::Other
    }
}

#[allow(dead_code)]
pub fn is_alpha(c: char) -> bool {
    c.is_ascii_alphabetic()
}

#[allow(dead_code)]
pub fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

#[allow(dead_code)]
pub fn is_alnum(c: char) -> bool {
    c.is_ascii_alphanumeric()
}

#[allow(dead_code)]
pub fn is_whitespace(c: char) -> bool {
    c.is_whitespace()
}

#[allow(dead_code)]
pub fn is_punctuation(c: char) -> bool {
    c.is_ascii_punctuation()
}

#[allow(dead_code)]
pub fn to_ascii_lower(c: char) -> char {
    c.to_ascii_lowercase()
}

#[allow(dead_code)]
pub fn to_ascii_upper(c: char) -> char {
    c.to_ascii_uppercase()
}

#[allow(dead_code)]
pub fn classify_str(s: &str) -> Vec<CharClass> {
    s.chars().map(classify_char).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_classifier_config();
        assert!(!cfg.locale_aware);
    }

    #[test]
    fn test_classify_alpha() {
        assert_eq!(classify_char('a'), CharClass::Alpha);
        assert_eq!(classify_char('Z'), CharClass::Alpha);
    }

    #[test]
    fn test_classify_digit() {
        assert_eq!(classify_char('0'), CharClass::Digit);
        assert_eq!(classify_char('9'), CharClass::Digit);
    }

    #[test]
    fn test_classify_whitespace() {
        assert_eq!(classify_char(' '), CharClass::Whitespace);
        assert_eq!(classify_char('\t'), CharClass::Whitespace);
    }

    #[test]
    fn test_classify_punctuation() {
        assert_eq!(classify_char('!'), CharClass::Punctuation);
        assert_eq!(classify_char('.'), CharClass::Punctuation);
    }

    #[test]
    fn test_is_alpha_digit() {
        assert!(is_alpha('a'));
        assert!(!is_alpha('1'));
        assert!(is_digit('5'));
        assert!(!is_digit('a'));
    }

    #[test]
    fn test_is_alnum() {
        assert!(is_alnum('a'));
        assert!(is_alnum('3'));
        assert!(!is_alnum('!'));
    }

    #[test]
    fn test_to_ascii_lower_upper() {
        assert_eq!(to_ascii_lower('A'), 'a');
        assert_eq!(to_ascii_upper('a'), 'A');
        assert_eq!(to_ascii_lower('z'), 'z');
    }

    #[test]
    fn test_classify_str() {
        let classes = classify_str("a1 !");
        assert_eq!(classes[0], CharClass::Alpha);
        assert_eq!(classes[1], CharClass::Digit);
        assert_eq!(classes[2], CharClass::Whitespace);
        assert_eq!(classes[3], CharClass::Punctuation);
    }
}
