// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct EntityId {
    pub kind: String,
    pub value: u64,
}

pub fn new_entity_id(kind: &str, value: u64) -> EntityId {
    EntityId {
        kind: kind.to_string(),
        value,
    }
}

pub fn entity_id_to_string(id: &EntityId) -> String {
    format!("{}:{}", id.kind, id.value)
}

pub fn entity_id_parse(s: &str) -> Option<EntityId> {
    let (kind, val_str) = s.split_once(':')?;
    let value: u64 = val_str.parse().ok()?;
    Some(EntityId {
        kind: kind.to_string(),
        value,
    })
}

pub fn entity_id_is_same_kind(a: &EntityId, b: &EntityId) -> bool {
    a.kind == b.kind
}

pub fn entity_id_value(id: &EntityId) -> u64 {
    id.value
}

pub fn entity_id_kind(id: &EntityId) -> &str {
    &id.kind
}

pub fn entity_id_nil(kind: &str) -> EntityId {
    EntityId {
        kind: kind.to_string(),
        value: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_entity_id() {
        /* create entity id with kind and value */
        let id = new_entity_id("user", 42);
        assert_eq!(entity_id_kind(&id), "user");
        assert_eq!(entity_id_value(&id), 42);
    }

    #[test]
    fn test_to_string() {
        /* serializes to kind:value format */
        let id = new_entity_id("order", 99);
        assert_eq!(entity_id_to_string(&id), "order:99");
    }

    #[test]
    fn test_parse() {
        /* parse kind:value string */
        let id = entity_id_parse("item:123").expect("should succeed");
        assert_eq!(entity_id_kind(&id), "item");
        assert_eq!(entity_id_value(&id), 123);
    }

    #[test]
    fn test_parse_invalid() {
        /* invalid format returns None */
        assert!(entity_id_parse("nodots").is_none());
        assert!(entity_id_parse("a:notanumber").is_none());
    }

    #[test]
    fn test_same_kind() {
        /* same kind check */
        let a = new_entity_id("user", 1);
        let b = new_entity_id("user", 2);
        let c = new_entity_id("order", 1);
        assert!(entity_id_is_same_kind(&a, &b));
        assert!(!entity_id_is_same_kind(&a, &c));
    }

    #[test]
    fn test_nil() {
        /* nil entity has value 0 */
        let id = entity_id_nil("product");
        assert_eq!(entity_id_value(&id), 0);
    }

    #[test]
    fn test_roundtrip() {
        /* parse(to_string()) roundtrip */
        let id = new_entity_id("thing", 7);
        let s = entity_id_to_string(&id);
        let id2 = entity_id_parse(&s).expect("should succeed");
        assert_eq!(entity_id_value(&id2), 7);
    }
}
