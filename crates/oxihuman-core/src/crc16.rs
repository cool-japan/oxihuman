#![allow(dead_code)]

/// CRC-16 calculator (CRC-16/CCITT-FALSE polynomial 0x1021).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Crc16 {
    value: u16,
}

const CRC16_POLY: u16 = 0x1021;
const CRC16_INIT: u16 = 0xFFFF;

fn crc16_byte(crc: u16, byte: u8) -> u16 {
    let mut c = crc ^ ((byte as u16) << 8);
    for _ in 0..8 {
        if c & 0x8000 != 0 {
            c = (c << 1) ^ CRC16_POLY;
        } else {
            c <<= 1;
        }
    }
    c
}

/// Computes CRC-16 over a byte slice.
#[allow(dead_code)]
pub fn crc16_compute(data: &[u8]) -> Crc16 {
    let mut crc = CRC16_INIT;
    for &b in data {
        crc = crc16_byte(crc, b);
    }
    Crc16 { value: crc }
}

/// Updates an existing CRC-16 with additional data.
#[allow(dead_code)]
pub fn crc16_update(crc: &Crc16, data: &[u8]) -> Crc16 {
    let mut c = crc.value;
    for &b in data {
        c = crc16_byte(c, b);
    }
    Crc16 { value: c }
}

/// Returns the CRC-16 as a u16.
#[allow(dead_code)]
pub fn crc16_to_u16(crc: &Crc16) -> u16 {
    crc.value
}

/// Returns the CRC-16 as a hex string.
#[allow(dead_code)]
pub fn crc16_to_hex(crc: &Crc16) -> String {
    format!("{:04X}", crc.value)
}

/// Verifies data against an expected CRC-16 value.
#[allow(dead_code)]
pub fn crc16_verify(data: &[u8], expected: u16) -> bool {
    let crc = crc16_compute(data);
    crc.value == expected
}

/// Resets CRC-16 to initial value.
#[allow(dead_code)]
pub fn crc16_reset() -> Crc16 {
    Crc16 { value: CRC16_INIT }
}

/// Computes CRC-16 from raw bytes (alias).
#[allow(dead_code)]
pub fn crc16_from_bytes(data: &[u8]) -> Crc16 {
    crc16_compute(data)
}

/// Combines two CRC-16 values by XOR.
#[allow(dead_code)]
pub fn crc16_combine(a: &Crc16, b: &Crc16) -> Crc16 {
    Crc16 {
        value: a.value ^ b.value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc16_compute() {
        let crc = crc16_compute(b"hello");
        assert_ne!(crc16_to_u16(&crc), 0);
    }

    #[test]
    fn test_crc16_deterministic() {
        let a = crc16_compute(b"test data");
        let b = crc16_compute(b"test data");
        assert_eq!(crc16_to_u16(&a), crc16_to_u16(&b));
    }

    #[test]
    fn test_crc16_different_data() {
        let a = crc16_compute(b"abc");
        let b = crc16_compute(b"def");
        assert_ne!(crc16_to_u16(&a), crc16_to_u16(&b));
    }

    #[test]
    fn test_crc16_update() {
        let crc = crc16_compute(b"hello");
        let updated = crc16_update(&crc, b" world");
        assert_ne!(crc16_to_u16(&crc), crc16_to_u16(&updated));
    }

    #[test]
    fn test_crc16_to_hex() {
        let crc = crc16_compute(b"test");
        let hex = crc16_to_hex(&crc);
        assert_eq!(hex.len(), 4);
    }

    #[test]
    fn test_crc16_verify() {
        let crc = crc16_compute(b"verify me");
        let val = crc16_to_u16(&crc);
        assert!(crc16_verify(b"verify me", val));
        assert!(!crc16_verify(b"other", val));
    }

    #[test]
    fn test_crc16_reset() {
        let crc = crc16_reset();
        assert_eq!(crc16_to_u16(&crc), CRC16_INIT);
    }

    #[test]
    fn test_crc16_from_bytes() {
        let a = crc16_from_bytes(b"same");
        let b = crc16_compute(b"same");
        assert_eq!(crc16_to_u16(&a), crc16_to_u16(&b));
    }

    #[test]
    fn test_crc16_combine() {
        let a = crc16_compute(b"part1");
        let b = crc16_compute(b"part2");
        let c = crc16_combine(&a, &b);
        assert_eq!(crc16_to_u16(&c), crc16_to_u16(&a) ^ crc16_to_u16(&b));
    }

    #[test]
    fn test_crc16_empty() {
        let crc = crc16_compute(b"");
        assert_eq!(crc16_to_u16(&crc), CRC16_INIT);
    }
}
