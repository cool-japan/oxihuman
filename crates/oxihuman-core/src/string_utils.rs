// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub fn str_capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub fn str_snake_to_camel(s: &str) -> String {
    s.split('_')
        .enumerate()
        .map(|(i, part)| {
            if i == 0 {
                part.to_string()
            } else {
                str_capitalize(part)
            }
        })
        .collect()
}

pub fn str_camel_to_snake(s: &str) -> String {
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            out.push('_');
        }
        out.push(ch.to_lowercase().next().unwrap_or(ch));
    }
    out
}

pub fn str_repeat(s: &str, n: usize) -> String {
    s.repeat(n)
}

pub fn str_pad_left(s: &str, width: usize, ch: char) -> String {
    let len = s.chars().count();
    if len >= width {
        s.to_string()
    } else {
        let pad: String = std::iter::repeat_n(ch, width - len).collect();
        pad + s
    }
}

pub fn str_pad_right(s: &str, width: usize, ch: char) -> String {
    let len = s.chars().count();
    if len >= width {
        s.to_string()
    } else {
        let pad: String = std::iter::repeat_n(ch, width - len).collect();
        s.to_string() + &pad
    }
}

pub fn str_truncate(s: &str, max_len: usize) -> &str {
    let mut idx = 0;
    for (i, _) in s.char_indices().take(max_len) {
        idx = i;
        if s[i..].chars().next().map_or(0, |c| c.len_utf8()) + i > s.len() {
            break;
        }
    }
    let char_count = s.chars().count();
    if char_count <= max_len {
        s
    } else {
        let byte_idx = s
            .char_indices()
            .nth(max_len)
            .map(|(i, _)| i)
            .unwrap_or(s.len());
        let _ = idx;
        &s[..byte_idx]
    }
}

pub fn str_count_char(s: &str, ch: char) -> usize {
    s.chars().filter(|&c| c == ch).count()
}

pub fn str_reverse(s: &str) -> String {
    s.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize() {
        /* basic capitalization */
        assert_eq!(str_capitalize("hello"), "Hello");
        assert_eq!(str_capitalize(""), "");
    }

    #[test]
    fn test_snake_to_camel() {
        /* snake_case to camelCase */
        assert_eq!(str_snake_to_camel("hello_world"), "helloWorld");
        assert_eq!(str_snake_to_camel("foo_bar_baz"), "fooBarBaz");
    }

    #[test]
    fn test_camel_to_snake() {
        /* camelCase to snake_case */
        assert_eq!(str_camel_to_snake("helloWorld"), "hello_world");
        assert_eq!(str_camel_to_snake("fooBarBaz"), "foo_bar_baz");
    }

    #[test]
    fn test_repeat() {
        /* repeat string n times */
        assert_eq!(str_repeat("ab", 3), "ababab");
        assert_eq!(str_repeat("x", 0), "");
    }

    #[test]
    fn test_pad_left() {
        /* pad left with char */
        assert_eq!(str_pad_left("hi", 5, '-'), "---hi");
        assert_eq!(str_pad_left("hello", 3, '-'), "hello");
    }

    #[test]
    fn test_pad_right() {
        /* pad right with char */
        assert_eq!(str_pad_right("hi", 5, '.'), "hi...");
    }

    #[test]
    fn test_truncate() {
        /* truncate to max_len chars */
        assert_eq!(str_truncate("hello", 3), "hel");
        assert_eq!(str_truncate("hi", 10), "hi");
    }

    #[test]
    fn test_count_char() {
        /* count occurrences of char */
        assert_eq!(str_count_char("hello", 'l'), 2);
        assert_eq!(str_count_char("abc", 'z'), 0);
    }
}
