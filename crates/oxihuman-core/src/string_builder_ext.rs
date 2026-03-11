#![allow(dead_code)]

/// Extended string builder with line and capacity utilities.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StringBuilderExt {
    buffer: String,
}

/// Creates a new empty string builder.
#[allow(dead_code)]
pub fn new_string_builder_ext() -> StringBuilderExt {
    StringBuilderExt {
        buffer: String::new(),
    }
}

/// Appends a string slice.
#[allow(dead_code)]
pub fn append_str(sb: &mut StringBuilderExt, s: &str) {
    sb.buffer.push_str(s);
}

/// Appends a single character.
#[allow(dead_code)]
pub fn append_char(sb: &mut StringBuilderExt, c: char) {
    sb.buffer.push(c);
}

/// Appends a string followed by a newline.
#[allow(dead_code)]
pub fn append_line(sb: &mut StringBuilderExt, s: &str) {
    sb.buffer.push_str(s);
    sb.buffer.push('\n');
}

/// Returns the current length in bytes.
#[allow(dead_code)]
pub fn builder_len(sb: &StringBuilderExt) -> usize {
    sb.buffer.len()
}

/// Converts to a String.
#[allow(dead_code)]
pub fn builder_to_string(sb: &StringBuilderExt) -> String {
    sb.buffer.clone()
}

/// Clears the builder.
#[allow(dead_code)]
pub fn builder_clear(sb: &mut StringBuilderExt) {
    sb.buffer.clear();
}

/// Returns the capacity.
#[allow(dead_code)]
pub fn builder_capacity(sb: &StringBuilderExt) -> usize {
    sb.buffer.capacity()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let sb = new_string_builder_ext();
        assert_eq!(builder_len(&sb), 0);
    }

    #[test]
    fn test_append_str() {
        let mut sb = new_string_builder_ext();
        append_str(&mut sb, "hello");
        assert_eq!(builder_to_string(&sb), "hello");
    }

    #[test]
    fn test_append_char() {
        let mut sb = new_string_builder_ext();
        append_char(&mut sb, 'x');
        assert_eq!(builder_to_string(&sb), "x");
    }

    #[test]
    fn test_append_line() {
        let mut sb = new_string_builder_ext();
        append_line(&mut sb, "line1");
        assert_eq!(builder_to_string(&sb), "line1\n");
    }

    #[test]
    fn test_builder_len() {
        let mut sb = new_string_builder_ext();
        append_str(&mut sb, "abc");
        assert_eq!(builder_len(&sb), 3);
    }

    #[test]
    fn test_clear() {
        let mut sb = new_string_builder_ext();
        append_str(&mut sb, "stuff");
        builder_clear(&mut sb);
        assert_eq!(builder_len(&sb), 0);
    }

    #[test]
    fn test_capacity() {
        let sb = new_string_builder_ext();
        let _ = builder_capacity(&sb);
    }

    #[test]
    fn test_chained_appends() {
        let mut sb = new_string_builder_ext();
        append_str(&mut sb, "hello");
        append_char(&mut sb, ' ');
        append_str(&mut sb, "world");
        assert_eq!(builder_to_string(&sb), "hello world");
    }

    #[test]
    fn test_multiple_lines() {
        let mut sb = new_string_builder_ext();
        append_line(&mut sb, "a");
        append_line(&mut sb, "b");
        assert_eq!(builder_to_string(&sb), "a\nb\n");
    }

    #[test]
    fn test_empty_to_string() {
        let sb = new_string_builder_ext();
        assert_eq!(builder_to_string(&sb), "");
    }
}
