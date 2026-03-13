// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Packed bitfield reader/writer (operates on a u64).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BitfieldLayout {
    pub fields: Vec<(String, u8, u8)>, // (name, bit_offset, bit_width)
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Bitfield {
    pub value: u64,
    pub layout: BitfieldLayout,
}

#[allow(dead_code)]
pub fn new_bitfield_layout() -> BitfieldLayout {
    BitfieldLayout { fields: Vec::new() }
}

#[allow(dead_code)]
pub fn bf_add_field(layout: &mut BitfieldLayout, name: &str, offset: u8, width: u8) {
    layout.fields.push((name.to_string(), offset, width));
}

#[allow(dead_code)]
pub fn new_bitfield(layout: BitfieldLayout) -> Bitfield {
    Bitfield { value: 0, layout }
}

#[allow(dead_code)]
pub fn bf_get(bf: &Bitfield, name: &str) -> Option<u64> {
    bf.layout.fields.iter().find(|(n, _, _)| n == name).map(|&(_, offset, width)| {
        let mask = if width >= 64 { u64::MAX } else { (1u64 << width) - 1 };
        (bf.value >> offset) & mask
    })
}

#[allow(dead_code)]
pub fn bf_set(bf: &mut Bitfield, name: &str, val: u64) -> bool {
    if let Some(&(_, offset, width)) = bf.layout.fields.iter().find(|(n, _, _)| n == name) {
        let mask = if width >= 64 { u64::MAX } else { (1u64 << width) - 1 };
        bf.value &= !(mask << offset);
        bf.value |= (val & mask) << offset;
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn bf_get_raw(bf: &Bitfield) -> u64 {
    bf.value
}

#[allow(dead_code)]
pub fn bf_set_raw(bf: &mut Bitfield, v: u64) {
    bf.value = v;
}

#[allow(dead_code)]
pub fn bf_field_count(bf: &Bitfield) -> usize {
    bf.layout.fields.len()
}

#[allow(dead_code)]
pub fn bf_to_json(bf: &Bitfield) -> String {
    let fields: Vec<String> = bf
        .layout
        .fields
        .iter()
        .map(|(name, offset, width)| {
            let mask = if *width >= 64 { u64::MAX } else { (1u64 << width) - 1 };
            let val = (bf.value >> offset) & mask;
            format!("\"{}\":{}", name, val)
        })
        .collect();
    format!("{{\"raw\":{},{}}}", bf.value, fields.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_layout() {
        let layout = new_bitfield_layout();
        assert!(layout.fields.is_empty());
    }

    #[test]
    fn test_add_field() {
        let mut layout = new_bitfield_layout();
        bf_add_field(&mut layout, "flags", 0, 4);
        assert_eq!(layout.fields.len(), 1);
    }

    #[test]
    fn test_set_and_get() {
        let mut layout = new_bitfield_layout();
        bf_add_field(&mut layout, "mode", 0, 4);
        bf_add_field(&mut layout, "state", 4, 2);
        let mut bf = new_bitfield(layout);
        assert!(bf_set(&mut bf, "mode", 0b1010));
        assert_eq!(bf_get(&bf, "mode"), Some(0b1010));
        assert!(bf_set(&mut bf, "state", 0b11));
        assert_eq!(bf_get(&bf, "state"), Some(0b11));
    }

    #[test]
    fn test_get_missing() {
        let layout = new_bitfield_layout();
        let bf = new_bitfield(layout);
        assert_eq!(bf_get(&bf, "nonexistent"), None);
    }

    #[test]
    fn test_set_missing() {
        let mut layout = new_bitfield_layout();
        bf_add_field(&mut layout, "x", 0, 8);
        let mut bf = new_bitfield(layout);
        assert!(!bf_set(&mut bf, "missing", 1));
    }

    #[test]
    fn test_raw_access() {
        let layout = new_bitfield_layout();
        let mut bf = new_bitfield(layout);
        bf_set_raw(&mut bf, 0xDEAD);
        assert_eq!(bf_get_raw(&bf), 0xDEAD);
    }

    #[test]
    fn test_field_count() {
        let mut layout = new_bitfield_layout();
        bf_add_field(&mut layout, "a", 0, 4);
        bf_add_field(&mut layout, "b", 4, 4);
        let bf = new_bitfield(layout);
        assert_eq!(bf_field_count(&bf), 2);
    }

    #[test]
    fn test_to_json() {
        let mut layout = new_bitfield_layout();
        bf_add_field(&mut layout, "val", 0, 8);
        let mut bf = new_bitfield(layout);
        bf_set(&mut bf, "val", 42);
        let json = bf_to_json(&bf);
        assert!(json.contains("\"val\":42"));
    }

    #[test]
    fn test_fields_do_not_interfere() {
        let mut layout = new_bitfield_layout();
        bf_add_field(&mut layout, "lo", 0, 8);
        bf_add_field(&mut layout, "hi", 8, 8);
        let mut bf = new_bitfield(layout);
        bf_set(&mut bf, "lo", 0xFF);
        bf_set(&mut bf, "hi", 0xAB);
        assert_eq!(bf_get(&bf, "lo"), Some(0xFF));
        assert_eq!(bf_get(&bf, "hi"), Some(0xAB));
    }
}
