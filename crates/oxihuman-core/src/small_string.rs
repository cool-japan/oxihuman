#![allow(dead_code)]

const SMALL_CAP: usize = 32;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SmallString {
    data: [u8; SMALL_CAP],
    len: usize,
}

#[allow(dead_code)]
pub fn new_small_string(s: &str) -> SmallString {
    let mut ss = SmallString {
        data: [0u8; SMALL_CAP],
        len: 0,
    };
    let bytes = s.as_bytes();
    let copy_len = bytes.len().min(SMALL_CAP);
    ss.data[..copy_len].copy_from_slice(&bytes[..copy_len]);
    ss.len = copy_len;
    ss
}

#[allow(dead_code)]
pub fn small_string_push(ss: &mut SmallString, ch: char) -> bool {
    let mut buf = [0u8; 4];
    let encoded = ch.encode_utf8(&mut buf);
    if ss.len + encoded.len() <= SMALL_CAP {
        ss.data[ss.len..ss.len + encoded.len()].copy_from_slice(encoded.as_bytes());
        ss.len += encoded.len();
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn small_string_len(ss: &SmallString) -> usize {
    ss.len
}

#[allow(dead_code)]
pub fn small_string_as_str(ss: &SmallString) -> &str {
    std::str::from_utf8(&ss.data[..ss.len]).unwrap_or("")
}

#[allow(dead_code)]
pub fn small_string_clear(ss: &mut SmallString) {
    ss.len = 0;
}

#[allow(dead_code)]
pub fn small_string_is_empty(ss: &SmallString) -> bool {
    ss.len == 0
}

#[allow(dead_code)]
pub fn small_string_capacity() -> usize {
    SMALL_CAP
}

#[allow(dead_code)]
pub fn small_string_truncate(ss: &mut SmallString, new_len: usize) {
    if new_len < ss.len {
        // Ensure we truncate at a valid UTF-8 boundary
        let s = std::str::from_utf8(&ss.data[..ss.len]).unwrap_or("");
        let mut valid_len = 0;
        for (i, _) in s.char_indices() {
            if i >= new_len {
                break;
            }
            valid_len = i + s[i..].chars().next().map_or(0, |c| c.len_utf8());
        }
        ss.len = valid_len.min(new_len);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let ss = new_small_string("hello");
        assert_eq!(small_string_as_str(&ss), "hello");
    }

    #[test]
    fn test_len() {
        let ss = new_small_string("abc");
        assert_eq!(small_string_len(&ss), 3);
    }

    #[test]
    fn test_push() {
        let mut ss = new_small_string("hi");
        assert!(small_string_push(&mut ss, '!'));
        assert_eq!(small_string_as_str(&ss), "hi!");
    }

    #[test]
    fn test_push_full() {
        let mut ss = new_small_string("");
        for _ in 0..SMALL_CAP {
            small_string_push(&mut ss, 'x');
        }
        assert!(!small_string_push(&mut ss, 'y'));
    }

    #[test]
    fn test_clear() {
        let mut ss = new_small_string("data");
        small_string_clear(&mut ss);
        assert!(small_string_is_empty(&ss));
    }

    #[test]
    fn test_is_empty() {
        let ss = new_small_string("");
        assert!(small_string_is_empty(&ss));
    }

    #[test]
    fn test_capacity() {
        assert_eq!(small_string_capacity(), SMALL_CAP);
    }

    #[test]
    fn test_truncate() {
        let mut ss = new_small_string("abcdef");
        small_string_truncate(&mut ss, 3);
        assert_eq!(small_string_as_str(&ss), "abc");
    }

    #[test]
    fn test_truncate_noop() {
        let mut ss = new_small_string("ab");
        small_string_truncate(&mut ss, 10);
        assert_eq!(small_string_len(&ss), 2);
    }

    #[test]
    fn test_empty_new() {
        let ss = new_small_string("");
        assert_eq!(small_string_len(&ss), 0);
    }
}
