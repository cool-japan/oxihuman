// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! UUID stub (deterministic from seed, no real randomness).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UuidConfig {
    pub version: u8,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Uuid {
    pub bytes: [u8; 16],
}

#[allow(dead_code)]
pub fn default_uuid_config() -> UuidConfig {
    UuidConfig { version: 4 }
}

/// Generate a deterministic UUID from a u64 seed using a simple LCG.
#[allow(dead_code)]
pub fn uuid_from_seed(seed: u64) -> Uuid {
    // LCG to fill 16 bytes
    let mut state = seed;
    let mut bytes = [0u8; 16];
    for b in &mut bytes {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *b = (state >> 56) as u8;
    }
    // Set version 4 bits
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    // Set variant bits
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid { bytes }
}

#[allow(dead_code)]
pub fn uuid_to_string(u: &Uuid) -> String {
    let b = &u.bytes;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        b[0], b[1], b[2], b[3],
        b[4], b[5],
        b[6], b[7],
        b[8], b[9],
        b[10], b[11], b[12], b[13], b[14], b[15]
    )
}

#[allow(dead_code)]
pub fn uuid_from_str(s: &str) -> Option<Uuid> {
    // Expect format: 8-4-4-4-12
    let s = s.replace('-', "");
    if s.len() != 32 {
        return None;
    }
    let mut bytes = [0u8; 16];
    for (i, b) in bytes.iter_mut().enumerate() {
        *b = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(Uuid { bytes })
}

#[allow(dead_code)]
pub fn uuid_is_nil(u: &Uuid) -> bool {
    u.bytes == [0u8; 16]
}

#[allow(dead_code)]
pub fn uuid_nil() -> Uuid {
    Uuid { bytes: [0u8; 16] }
}

#[allow(dead_code)]
pub fn uuid_bytes(u: &Uuid) -> &[u8; 16] {
    &u.bytes
}

#[allow(dead_code)]
pub fn uuid_version(u: &Uuid) -> u8 {
    (u.bytes[6] >> 4) & 0x0f
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_uuid_config();
        assert_eq!(cfg.version, 4);
    }

    #[test]
    fn test_uuid_from_seed_deterministic() {
        let u1 = uuid_from_seed(42);
        let u2 = uuid_from_seed(42);
        assert_eq!(u1, u2);
    }

    #[test]
    fn test_uuid_from_seed_different_seeds() {
        let u1 = uuid_from_seed(1);
        let u2 = uuid_from_seed(2);
        assert_ne!(u1, u2);
    }

    #[test]
    fn test_uuid_to_string_format() {
        let u = uuid_from_seed(0);
        let s = uuid_to_string(&u);
        // format: 8-4-4-4-12 with dashes
        let parts: Vec<&str> = s.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[1].len(), 4);
        assert_eq!(parts[2].len(), 4);
        assert_eq!(parts[3].len(), 4);
        assert_eq!(parts[4].len(), 12);
    }

    #[test]
    fn test_uuid_roundtrip() {
        let u = uuid_from_seed(12345);
        let s = uuid_to_string(&u);
        let u2 = uuid_from_str(&s).unwrap();
        assert_eq!(u, u2);
    }

    #[test]
    fn test_uuid_nil() {
        let n = uuid_nil();
        assert!(uuid_is_nil(&n));
    }

    #[test]
    fn test_uuid_from_seed_not_nil() {
        let u = uuid_from_seed(999);
        assert!(!uuid_is_nil(&u));
    }

    #[test]
    fn test_uuid_version() {
        let u = uuid_from_seed(7);
        assert_eq!(uuid_version(&u), 4);
    }

    #[test]
    fn test_uuid_bytes_len() {
        let u = uuid_from_seed(100);
        assert_eq!(uuid_bytes(&u).len(), 16);
    }
}
