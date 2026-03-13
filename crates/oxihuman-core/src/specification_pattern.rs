// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct Spec {
    pub name: String,
    pub predicate_desc: String,
}

pub fn new_spec(name: &str, desc: &str) -> Spec {
    Spec {
        name: name.to_string(),
        predicate_desc: desc.to_string(),
    }
}

pub fn spec_and(a: &Spec, b: &Spec) -> Spec {
    Spec {
        name: format!("({} AND {})", a.name, b.name),
        predicate_desc: format!("({}) AND ({})", a.predicate_desc, b.predicate_desc),
    }
}

pub fn spec_or(a: &Spec, b: &Spec) -> Spec {
    Spec {
        name: format!("({} OR {})", a.name, b.name),
        predicate_desc: format!("({}) OR ({})", a.predicate_desc, b.predicate_desc),
    }
}

pub fn spec_not(a: &Spec) -> Spec {
    Spec {
        name: format!("NOT({})", a.name),
        predicate_desc: format!("NOT({})", a.predicate_desc),
    }
}

pub fn spec_range(lo: i64, hi: i64) -> Spec {
    Spec {
        name: format!("range({},{})", lo, hi),
        predicate_desc: format!("value in [{},{}]", lo, hi),
    }
}

pub fn spec_name(s: &Spec) -> &str {
    &s.name
}

pub fn spec_satisfies_range(s: &Spec, val: i64, lo: i64, hi: i64) -> bool {
    let _ = s;
    val >= lo && val <= hi
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_spec() {
        /* create a spec with name and desc */
        let s = new_spec("positive", "value > 0");
        assert_eq!(spec_name(&s), "positive");
    }

    #[test]
    fn test_spec_and() {
        /* AND composition includes both names */
        let a = new_spec("A", "a");
        let b = new_spec("B", "b");
        let c = spec_and(&a, &b);
        assert!(c.name.contains("AND"));
    }

    #[test]
    fn test_spec_or() {
        /* OR composition includes both names */
        let a = new_spec("A", "a");
        let b = new_spec("B", "b");
        let c = spec_or(&a, &b);
        assert!(c.name.contains("OR"));
    }

    #[test]
    fn test_spec_not() {
        /* NOT composition wraps name */
        let a = new_spec("A", "a");
        let c = spec_not(&a);
        assert!(c.name.contains("NOT"));
    }

    #[test]
    fn test_spec_range() {
        /* range spec has proper name */
        let s = spec_range(0, 100);
        assert!(s.name.contains("range"));
    }

    #[test]
    fn test_spec_satisfies_range() {
        /* check value in range */
        let s = spec_range(0, 10);
        assert!(spec_satisfies_range(&s, 5, 0, 10));
        assert!(!spec_satisfies_range(&s, 11, 0, 10));
    }
}
