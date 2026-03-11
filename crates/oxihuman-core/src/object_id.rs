// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A unique object identifier.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectId {
    pub id: u64,
}

/// Sequential ID generator.
#[allow(dead_code)]
pub struct ObjectIdGen {
    pub next: u64,
}

/// Create a new ID generator starting at 1 (0 is reserved for null).
#[allow(dead_code)]
pub fn new_id_gen() -> ObjectIdGen {
    ObjectIdGen { next: 1 }
}

/// Generate the next unique `ObjectId`.
#[allow(dead_code)]
pub fn next_id(gen: &mut ObjectIdGen) -> ObjectId {
    let id = gen.next;
    gen.next += 1;
    ObjectId { id }
}

/// Return the raw u64 value of an `ObjectId`.
#[allow(dead_code)]
pub fn id_value(oid: &ObjectId) -> u64 {
    oid.id
}

/// Returns true if the ID represents the null (invalid) object.
#[allow(dead_code)]
pub fn id_is_null(oid: &ObjectId) -> bool {
    oid.id == 0
}

/// Return the null (invalid) `ObjectId`.
#[allow(dead_code)]
pub fn null_id() -> ObjectId {
    ObjectId { id: 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_id_is_one() {
        let mut gen = new_id_gen();
        let id = next_id(&mut gen);
        assert_eq!(id_value(&id), 1);
    }

    #[test]
    fn ids_are_sequential() {
        let mut gen = new_id_gen();
        let a = next_id(&mut gen);
        let b = next_id(&mut gen);
        assert_eq!(id_value(&b), id_value(&a) + 1);
    }

    #[test]
    fn null_id_is_zero() {
        let nid = null_id();
        assert_eq!(id_value(&nid), 0);
    }

    #[test]
    fn null_id_is_null() {
        let nid = null_id();
        assert!(id_is_null(&nid));
    }

    #[test]
    fn generated_id_is_not_null() {
        let mut gen = new_id_gen();
        let id = next_id(&mut gen);
        assert!(!id_is_null(&id));
    }

    #[test]
    fn ten_sequential_ids() {
        let mut gen = new_id_gen();
        for i in 1u64..=10 {
            let id = next_id(&mut gen);
            assert_eq!(id_value(&id), i);
        }
    }

    #[test]
    fn ids_are_unique() {
        let mut gen = new_id_gen();
        let a = next_id(&mut gen);
        let b = next_id(&mut gen);
        assert_ne!(a, b);
    }

    #[test]
    fn null_id_not_equal_to_generated() {
        let mut gen = new_id_gen();
        let id = next_id(&mut gen);
        assert_ne!(id, null_id());
    }

    #[test]
    fn id_gen_starts_at_one() {
        let gen = new_id_gen();
        assert_eq!(gen.next, 1);
    }

    #[test]
    fn id_value_matches_struct_field() {
        let id = ObjectId { id: 42 };
        assert_eq!(id_value(&id), 42);
    }
}
