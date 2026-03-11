#![allow(dead_code)]

/// Byte order conversion utilities.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrderType {
    LittleEndian,
    BigEndian,
}

/// Converts u16 to little-endian bytes.
#[allow(dead_code)]
pub fn u16_to_le(value: u16) -> [u8; 2] {
    value.to_le_bytes()
}

/// Converts u16 to big-endian bytes.
#[allow(dead_code)]
pub fn u16_to_be(value: u16) -> [u8; 2] {
    value.to_be_bytes()
}

/// Converts u32 to little-endian bytes.
#[allow(dead_code)]
pub fn u32_to_le(value: u32) -> [u8; 4] {
    value.to_le_bytes()
}

/// Converts u32 to big-endian bytes.
#[allow(dead_code)]
pub fn u32_to_be(value: u32) -> [u8; 4] {
    value.to_be_bytes()
}

/// Converts f32 to little-endian bytes.
#[allow(dead_code)]
pub fn f32_to_le_bytes(value: f32) -> [u8; 4] {
    value.to_le_bytes()
}

/// Converts little-endian bytes to f32.
#[allow(dead_code)]
pub fn f32_from_le_bytes(bytes: [u8; 4]) -> f32 {
    f32::from_le_bytes(bytes)
}

/// Converts little-endian bytes to u16.
#[allow(dead_code)]
pub fn u16_from_le(bytes: [u8; 2]) -> u16 {
    u16::from_le_bytes(bytes)
}

/// Converts little-endian bytes to u32.
#[allow(dead_code)]
pub fn u32_from_le(bytes: [u8; 4]) -> u32 {
    u32::from_le_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u16_le_roundtrip() {
        let val: u16 = 0x1234;
        let bytes = u16_to_le(val);
        assert_eq!(u16_from_le(bytes), val);
    }

    #[test]
    fn test_u16_be() {
        let val: u16 = 0x1234;
        let bytes = u16_to_be(val);
        assert_eq!(bytes[0], 0x12);
        assert_eq!(bytes[1], 0x34);
    }

    #[test]
    fn test_u32_le_roundtrip() {
        let val: u32 = 0xDEAD_BEEF;
        let bytes = u32_to_le(val);
        assert_eq!(u32_from_le(bytes), val);
    }

    #[test]
    fn test_u32_be() {
        let val: u32 = 0x0102_0304;
        let bytes = u32_to_be(val);
        assert_eq!(bytes, [0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_f32_le_roundtrip() {
        let val: f32 = 2.78;
        let bytes = f32_to_le_bytes(val);
        let result = f32_from_le_bytes(bytes);
        assert!((result - val).abs() < f32::EPSILON);
    }

    #[test]
    fn test_u16_le_zero() {
        let bytes = u16_to_le(0);
        assert_eq!(bytes, [0, 0]);
    }

    #[test]
    fn test_u32_le_zero() {
        let bytes = u32_to_le(0);
        assert_eq!(bytes, [0, 0, 0, 0]);
    }

    #[test]
    fn test_f32_le_zero() {
        let bytes = f32_to_le_bytes(0.0);
        let result = f32_from_le_bytes(bytes);
        assert!((result).abs() < f32::EPSILON);
    }

    #[test]
    fn test_byte_order_type() {
        let le = ByteOrderType::LittleEndian;
        let be = ByteOrderType::BigEndian;
        assert_ne!(le, be);
    }

    #[test]
    fn test_u16_max() {
        let val = u16::MAX;
        let bytes = u16_to_le(val);
        assert_eq!(u16_from_le(bytes), val);
    }
}
