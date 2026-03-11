// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[allow(dead_code)]
pub struct UuidGen {
    pub counter: u64,
    pub prefix: String,
}

#[allow(dead_code)]
pub fn new_uuid_gen(prefix: &str) -> UuidGen {
    UuidGen { counter: 0, prefix: prefix.to_string() }
}

#[allow(dead_code)]
pub fn ug_next(gen: &mut UuidGen) -> String {
    let id = format!("{}-{:016x}", gen.prefix, gen.counter);
    gen.counter += 1;
    id
}

#[allow(dead_code)]
pub fn ug_parse_counter(id: &str, prefix: &str) -> Option<u64> {
    let expected_prefix = format!("{}-", prefix);
    if !id.starts_with(&expected_prefix) {
        return None;
    }
    let hex_part = &id[expected_prefix.len()..];
    u64::from_str_radix(hex_part, 16).ok()
}

#[allow(dead_code)]
pub fn ug_count(gen: &UuidGen) -> u64 {
    gen.counter
}

#[allow(dead_code)]
pub fn ug_reset(gen: &mut UuidGen) {
    gen.counter = 0;
}

#[allow(dead_code)]
pub fn ug_peek_next(gen: &UuidGen) -> String {
    format!("{}-{:016x}", gen.prefix, gen.counter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_format() {
        let mut gen = new_uuid_gen("item");
        let id = ug_next(&mut gen);
        assert!(id.starts_with("item-"));
        assert_eq!(id.len(), "item-".len() + 16);
    }

    #[test]
    fn test_parse_counter() {
        let mut gen = new_uuid_gen("obj");
        let id = ug_next(&mut gen); /* counter was 0 before ug_next */
        let c = ug_parse_counter(&id, "obj").unwrap();
        assert_eq!(c, 0);
    }

    #[test]
    fn test_count_increments() {
        let mut gen = new_uuid_gen("x");
        assert_eq!(ug_count(&gen), 0);
        ug_next(&mut gen);
        assert_eq!(ug_count(&gen), 1);
        ug_next(&mut gen);
        assert_eq!(ug_count(&gen), 2);
    }

    #[test]
    fn test_reset() {
        let mut gen = new_uuid_gen("t");
        ug_next(&mut gen);
        ug_next(&mut gen);
        ug_reset(&mut gen);
        assert_eq!(ug_count(&gen), 0);
    }

    #[test]
    fn test_peek_matches_next() {
        let mut gen = new_uuid_gen("p");
        let peek = ug_peek_next(&gen);
        let next = ug_next(&mut gen);
        assert_eq!(peek, next);
    }

    #[test]
    fn test_parse_counter_wrong_prefix() {
        let r = ug_parse_counter("item-0000000000000001", "obj");
        assert!(r.is_none());
    }

    #[test]
    fn test_sequential_ids_differ() {
        let mut gen = new_uuid_gen("seq");
        let id1 = ug_next(&mut gen);
        let id2 = ug_next(&mut gen);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_peek_does_not_advance() {
        let gen = new_uuid_gen("q");
        let _p1 = ug_peek_next(&gen);
        let _p2 = ug_peek_next(&gen);
        assert_eq!(ug_count(&gen), 0);
    }
}
