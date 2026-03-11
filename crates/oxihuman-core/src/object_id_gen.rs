#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Sequential object ID generator.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ObjectIdGen {
    current: u64,
    seed: u64,
    name: String,
    count: u64,
}

#[allow(dead_code)]
pub fn new_object_id_gen(name: &str, seed: u64) -> ObjectIdGen {
    ObjectIdGen {
        current: seed,
        seed,
        name: name.to_string(),
        count: 0,
    }
}

#[allow(dead_code)]
pub fn next_id(gen: &mut ObjectIdGen) -> u64 {
    let id = gen.current;
    gen.current += 1;
    gen.count += 1;
    id
}

#[allow(dead_code)]
pub fn current_id(gen: &ObjectIdGen) -> u64 {
    gen.current
}

#[allow(dead_code)]
pub fn reset_id_gen(gen: &mut ObjectIdGen) {
    gen.current = gen.seed;
    gen.count = 0;
}

#[allow(dead_code)]
pub fn id_gen_count(gen: &ObjectIdGen) -> u64 {
    gen.count
}

#[allow(dead_code)]
pub fn id_gen_to_json(gen: &ObjectIdGen) -> String {
    format!(
        r#"{{"name":"{}","current":{},"seed":{},"count":{}}}"#,
        gen.name, gen.current, gen.seed, gen.count
    )
}

#[allow(dead_code)]
pub fn id_gen_seed(gen: &ObjectIdGen) -> u64 {
    gen.seed
}

#[allow(dead_code)]
pub fn id_gen_name(gen: &ObjectIdGen) -> &str {
    &gen.name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_gen() {
        let g = new_object_id_gen("test", 0);
        assert_eq!(current_id(&g), 0);
    }

    #[test]
    fn test_next_id() {
        let mut g = new_object_id_gen("test", 0);
        assert_eq!(next_id(&mut g), 0);
        assert_eq!(next_id(&mut g), 1);
    }

    #[test]
    fn test_count() {
        let mut g = new_object_id_gen("test", 0);
        next_id(&mut g);
        next_id(&mut g);
        assert_eq!(id_gen_count(&g), 2);
    }

    #[test]
    fn test_reset() {
        let mut g = new_object_id_gen("test", 10);
        next_id(&mut g);
        reset_id_gen(&mut g);
        assert_eq!(current_id(&g), 10);
        assert_eq!(id_gen_count(&g), 0);
    }

    #[test]
    fn test_seed() {
        let g = new_object_id_gen("test", 42);
        assert_eq!(id_gen_seed(&g), 42);
    }

    #[test]
    fn test_name() {
        let g = new_object_id_gen("my_gen", 0);
        assert_eq!(id_gen_name(&g), "my_gen");
    }

    #[test]
    fn test_to_json() {
        let g = new_object_id_gen("j", 0);
        let json = id_gen_to_json(&g);
        assert!(json.contains("\"name\":\"j\""));
    }

    #[test]
    fn test_sequential() {
        let mut g = new_object_id_gen("seq", 100);
        assert_eq!(next_id(&mut g), 100);
        assert_eq!(next_id(&mut g), 101);
        assert_eq!(next_id(&mut g), 102);
    }

    #[test]
    fn test_current_after_next() {
        let mut g = new_object_id_gen("test", 0);
        next_id(&mut g);
        assert_eq!(current_id(&g), 1);
    }

    #[test]
    fn test_seed_start() {
        let mut g = new_object_id_gen("test", 50);
        assert_eq!(next_id(&mut g), 50);
    }
}
