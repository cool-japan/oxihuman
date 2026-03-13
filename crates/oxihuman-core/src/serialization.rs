//! Generic binary and JSON serialization utilities.

#[allow(dead_code)]
pub struct BinWriter {
    pub buf: Vec<u8>,
}

#[allow(dead_code)]
pub struct BinReader<'a> {
    pub buf: &'a [u8],
    pub pos: usize,
}

#[allow(dead_code)]
pub struct JsonBuilder {
    pub buf: String,
    pub indent: usize,
    pub first_item: bool,
}

// ── BinWriter ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn new_bin_writer() -> BinWriter {
    BinWriter { buf: Vec::new() }
}

#[allow(dead_code)]
pub fn write_u8(w: &mut BinWriter, v: u8) {
    w.buf.push(v);
}

#[allow(dead_code)]
pub fn write_u16_le(w: &mut BinWriter, v: u16) {
    w.buf.extend_from_slice(&v.to_le_bytes());
}

#[allow(dead_code)]
pub fn write_u32_le(w: &mut BinWriter, v: u32) {
    w.buf.extend_from_slice(&v.to_le_bytes());
}

#[allow(dead_code)]
pub fn write_f32_le(w: &mut BinWriter, v: f32) {
    w.buf.extend_from_slice(&v.to_le_bytes());
}

#[allow(dead_code)]
pub fn write_bytes(w: &mut BinWriter, data: &[u8]) {
    w.buf.extend_from_slice(data);
}

/// Write a length-prefixed string: u32 length (LE) followed by UTF-8 bytes.
#[allow(dead_code)]
pub fn write_str(w: &mut BinWriter, s: &str) {
    let bytes = s.as_bytes();
    write_u32_le(w, bytes.len() as u32);
    write_bytes(w, bytes);
}

// ── BinReader ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn new_bin_reader(buf: &[u8]) -> BinReader<'_> {
    BinReader { buf, pos: 0 }
}

#[allow(dead_code)]
pub fn read_u8(r: &mut BinReader<'_>) -> Option<u8> {
    if r.pos < r.buf.len() {
        let v = r.buf[r.pos];
        r.pos += 1;
        Some(v)
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn read_u16_le(r: &mut BinReader<'_>) -> Option<u16> {
    if r.pos + 2 <= r.buf.len() {
        let bytes = [r.buf[r.pos], r.buf[r.pos + 1]];
        r.pos += 2;
        Some(u16::from_le_bytes(bytes))
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn read_u32_le(r: &mut BinReader<'_>) -> Option<u32> {
    if r.pos + 4 <= r.buf.len() {
        let bytes = [
            r.buf[r.pos],
            r.buf[r.pos + 1],
            r.buf[r.pos + 2],
            r.buf[r.pos + 3],
        ];
        r.pos += 4;
        Some(u32::from_le_bytes(bytes))
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn read_f32_le(r: &mut BinReader<'_>) -> Option<f32> {
    if r.pos + 4 <= r.buf.len() {
        let bytes = [
            r.buf[r.pos],
            r.buf[r.pos + 1],
            r.buf[r.pos + 2],
            r.buf[r.pos + 3],
        ];
        r.pos += 4;
        Some(f32::from_le_bytes(bytes))
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn read_str(r: &mut BinReader<'_>) -> Option<String> {
    let len = read_u32_le(r)? as usize;
    if r.pos + len <= r.buf.len() {
        let bytes = &r.buf[r.pos..r.pos + len];
        r.pos += len;
        String::from_utf8(bytes.to_vec()).ok()
    } else {
        None
    }
}

// ── JsonBuilder ───────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn new_json_builder() -> JsonBuilder {
    JsonBuilder {
        buf: String::new(),
        indent: 0,
        first_item: true,
    }
}

fn maybe_comma(b: &mut JsonBuilder) {
    if !b.first_item {
        b.buf.push(',');
    }
    b.first_item = false;
}

#[allow(dead_code)]
pub fn json_key_str(b: &mut JsonBuilder, key: &str, val: &str) {
    maybe_comma(b);
    b.buf.push_str(&format!(r#""{}":"{}""#, key, val));
}

#[allow(dead_code)]
pub fn json_key_f32(b: &mut JsonBuilder, key: &str, val: f32) {
    maybe_comma(b);
    b.buf.push_str(&format!(r#""{}":{}"#, key, val));
}

#[allow(dead_code)]
pub fn json_key_u32(b: &mut JsonBuilder, key: &str, val: u32) {
    maybe_comma(b);
    b.buf.push_str(&format!(r#""{}":{}"#, key, val));
}

#[allow(dead_code)]
pub fn json_finalize(b: &mut JsonBuilder) -> String {
    format!("{{{}}}", b.buf)
}

#[allow(dead_code)]
pub fn f32_array_to_json(arr: &[f32]) -> String {
    let items: Vec<String> = arr.iter().map(|v| format!("{}", v)).collect();
    format!("[{}]", items.join(","))
}

#[allow(dead_code)]
pub fn u32_array_to_json(arr: &[u32]) -> String {
    let items: Vec<String> = arr.iter().map(|v| format!("{}", v)).collect();
    format!("[{}]", items.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── round-trip tests ────────────────────────────────────────────────────

    #[test]
    fn test_roundtrip_u8() {
        let mut w = new_bin_writer();
        write_u8(&mut w, 42);
        let mut r = new_bin_reader(&w.buf);
        assert_eq!(read_u8(&mut r), Some(42));
    }

    #[test]
    fn test_roundtrip_u8_max() {
        let mut w = new_bin_writer();
        write_u8(&mut w, 255);
        let mut r = new_bin_reader(&w.buf);
        assert_eq!(read_u8(&mut r), Some(255));
    }

    #[test]
    fn test_roundtrip_u16() {
        let mut w = new_bin_writer();
        write_u16_le(&mut w, 1234);
        let mut r = new_bin_reader(&w.buf);
        assert_eq!(read_u16_le(&mut r), Some(1234));
    }

    #[test]
    fn test_roundtrip_u16_max() {
        let mut w = new_bin_writer();
        write_u16_le(&mut w, 65535);
        let mut r = new_bin_reader(&w.buf);
        assert_eq!(read_u16_le(&mut r), Some(65535));
    }

    #[test]
    fn test_roundtrip_u32() {
        let mut w = new_bin_writer();
        write_u32_le(&mut w, 0xDEAD_BEEF);
        let mut r = new_bin_reader(&w.buf);
        assert_eq!(read_u32_le(&mut r), Some(0xDEAD_BEEF));
    }

    #[test]
    fn test_roundtrip_f32() {
        let mut w = new_bin_writer();
        write_f32_le(&mut w, std::f32::consts::PI);
        let mut r = new_bin_reader(&w.buf);
        let v = read_f32_le(&mut r).expect("should succeed");
        assert!((v - std::f32::consts::PI).abs() < 1e-6);
    }

    #[test]
    fn test_roundtrip_f32_negative() {
        let mut w = new_bin_writer();
        write_f32_le(&mut w, -1.5);
        let mut r = new_bin_reader(&w.buf);
        let v = read_f32_le(&mut r).expect("should succeed");
        assert!((v + 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_roundtrip_str() {
        let mut w = new_bin_writer();
        write_str(&mut w, "hello world");
        let mut r = new_bin_reader(&w.buf);
        assert_eq!(read_str(&mut r), Some("hello world".to_string()));
    }

    #[test]
    fn test_roundtrip_str_empty() {
        let mut w = new_bin_writer();
        write_str(&mut w, "");
        let mut r = new_bin_reader(&w.buf);
        assert_eq!(read_str(&mut r), Some("".to_string()));
    }

    #[test]
    fn test_roundtrip_multiple_values() {
        let mut w = new_bin_writer();
        write_u8(&mut w, 7);
        write_u32_le(&mut w, 42);
        write_f32_le(&mut w, 2.78);
        let mut r = new_bin_reader(&w.buf);
        assert_eq!(read_u8(&mut r), Some(7));
        assert_eq!(read_u32_le(&mut r), Some(42));
        let f = read_f32_le(&mut r).expect("should succeed");
        assert!((f - 2.78).abs() < 1e-4);
    }

    #[test]
    fn test_read_past_end_returns_none() {
        let mut r = new_bin_reader(&[]);
        assert_eq!(read_u8(&mut r), None);
        assert_eq!(read_u32_le(&mut r), None);
        assert_eq!(read_f32_le(&mut r), None);
    }

    // ── JSON builder tests ──────────────────────────────────────────────────

    #[test]
    fn test_json_builder_str() {
        let mut b = new_json_builder();
        json_key_str(&mut b, "name", "Alice");
        let out = json_finalize(&mut b);
        assert!(out.contains(r#""name":"Alice""#));
        assert!(out.starts_with('{'));
        assert!(out.ends_with('}'));
    }

    #[test]
    fn test_json_builder_u32() {
        let mut b = new_json_builder();
        json_key_u32(&mut b, "count", 42);
        let out = json_finalize(&mut b);
        assert!(out.contains(r#""count":42"#));
    }

    #[test]
    fn test_json_builder_f32() {
        let mut b = new_json_builder();
        json_key_f32(&mut b, "value", 1.5);
        let out = json_finalize(&mut b);
        assert!(out.contains(r#""value":1.5"#));
    }

    #[test]
    fn test_json_builder_multiple_keys() {
        let mut b = new_json_builder();
        json_key_str(&mut b, "a", "hello");
        json_key_u32(&mut b, "b", 99);
        let out = json_finalize(&mut b);
        assert!(out.contains(r#""a":"hello""#));
        assert!(out.contains(r#""b":99"#));
        assert!(out.contains(','));
    }

    #[test]
    fn test_f32_array_to_json() {
        let arr = [1.0f32, 2.0, 3.0];
        let json = f32_array_to_json(&arr);
        assert!(json.starts_with('['));
        assert!(json.ends_with(']'));
        assert!(json.contains("1"));
        assert!(json.contains("2"));
        assert!(json.contains("3"));
    }

    #[test]
    fn test_u32_array_to_json() {
        let arr = [10u32, 20, 30];
        let json = u32_array_to_json(&arr);
        assert_eq!(json, "[10,20,30]");
    }

    #[test]
    fn test_f32_array_empty() {
        let json = f32_array_to_json(&[]);
        assert_eq!(json, "[]");
    }
}
