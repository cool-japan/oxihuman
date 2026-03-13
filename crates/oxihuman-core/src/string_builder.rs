// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Efficient string builder.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StringBuilderConfig {
    pub initial_capacity: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StringBuilder {
    pub buf: String,
}

#[allow(dead_code)]
pub fn default_string_builder_config() -> StringBuilderConfig {
    StringBuilderConfig { initial_capacity: 64 }
}

#[allow(dead_code)]
pub fn new_string_builder(config: &StringBuilderConfig) -> StringBuilder {
    StringBuilder { buf: String::with_capacity(config.initial_capacity) }
}

#[allow(dead_code)]
pub fn sb_append(sb: &mut StringBuilder, s: &str) {
    sb.buf.push_str(s);
}

#[allow(dead_code)]
pub fn sb_append_char(sb: &mut StringBuilder, c: char) {
    sb.buf.push(c);
}

#[allow(dead_code)]
pub fn sb_append_line(sb: &mut StringBuilder, s: &str) {
    sb.buf.push_str(s);
    sb.buf.push('\n');
}

#[allow(dead_code)]
pub fn sb_build(sb: &StringBuilder) -> String {
    sb.buf.clone()
}

#[allow(dead_code)]
pub fn sb_clear(sb: &mut StringBuilder) {
    sb.buf.clear();
}

#[allow(dead_code)]
pub fn sb_len(sb: &StringBuilder) -> usize {
    sb.buf.len()
}

#[allow(dead_code)]
pub fn sb_is_empty(sb: &StringBuilder) -> bool {
    sb.buf.is_empty()
}

#[allow(dead_code)]
pub fn sb_capacity(sb: &StringBuilder) -> usize {
    sb.buf.capacity()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_string_builder_config();
        assert_eq!(cfg.initial_capacity, 64);
    }

    #[test]
    fn test_new_string_builder() {
        let cfg = default_string_builder_config();
        let sb = new_string_builder(&cfg);
        assert!(sb_is_empty(&sb));
    }

    #[test]
    fn test_sb_append() {
        let cfg = default_string_builder_config();
        let mut sb = new_string_builder(&cfg);
        sb_append(&mut sb, "hello");
        assert_eq!(sb_build(&sb), "hello");
    }

    #[test]
    fn test_sb_append_char() {
        let cfg = default_string_builder_config();
        let mut sb = new_string_builder(&cfg);
        sb_append_char(&mut sb, 'X');
        assert_eq!(sb_build(&sb), "X");
    }

    #[test]
    fn test_sb_append_line() {
        let cfg = default_string_builder_config();
        let mut sb = new_string_builder(&cfg);
        sb_append_line(&mut sb, "line");
        assert_eq!(sb_build(&sb), "line\n");
    }

    #[test]
    fn test_sb_len() {
        let cfg = default_string_builder_config();
        let mut sb = new_string_builder(&cfg);
        sb_append(&mut sb, "abc");
        assert_eq!(sb_len(&sb), 3);
    }

    #[test]
    fn test_sb_clear() {
        let cfg = default_string_builder_config();
        let mut sb = new_string_builder(&cfg);
        sb_append(&mut sb, "data");
        sb_clear(&mut sb);
        assert!(sb_is_empty(&sb));
    }

    #[test]
    fn test_sb_capacity() {
        let cfg = StringBuilderConfig { initial_capacity: 128 };
        let sb = new_string_builder(&cfg);
        assert!(sb_capacity(&sb) >= 128);
    }

    #[test]
    fn test_multiple_appends() {
        let cfg = default_string_builder_config();
        let mut sb = new_string_builder(&cfg);
        sb_append(&mut sb, "a");
        sb_append(&mut sb, "b");
        sb_append(&mut sb, "c");
        assert_eq!(sb_build(&sb), "abc");
    }
}
