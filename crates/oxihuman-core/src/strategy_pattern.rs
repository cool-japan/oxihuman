// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct SortStrategy {
    pub name: String,
    pub ascending: bool,
}

pub fn new_sort_strategy(name: &str, ascending: bool) -> SortStrategy {
    SortStrategy {
        name: name.to_string(),
        ascending,
    }
}

pub fn sort_apply_f32(s: &SortStrategy, data: &mut [f32]) {
    if s.ascending {
        data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    } else {
        data.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    }
}

pub fn sort_apply_str(s: &SortStrategy, data: &mut [String]) {
    if s.ascending {
        data.sort();
    } else {
        data.sort_by(|a, b| b.cmp(a));
    }
}

pub fn sort_is_ascending(s: &SortStrategy) -> bool {
    s.ascending
}

pub fn sort_strategy_name(s: &SortStrategy) -> &str {
    &s.name
}

pub fn sort_reverse(s: &mut SortStrategy) {
    s.ascending = !s.ascending;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_f32_ascending() {
        /* sort floats ascending */
        let s = new_sort_strategy("asc", true);
        let mut v = vec![3.0f32, 1.0, 2.0];
        sort_apply_f32(&s, &mut v);
        assert_eq!(v, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_sort_f32_descending() {
        /* sort floats descending */
        let s = new_sort_strategy("desc", false);
        let mut v = vec![3.0f32, 1.0, 2.0];
        sort_apply_f32(&s, &mut v);
        assert_eq!(v, vec![3.0, 2.0, 1.0]);
    }

    #[test]
    fn test_sort_str_ascending() {
        /* sort strings ascending */
        let s = new_sort_strategy("alpha", true);
        let mut v = vec![
            "banana".to_string(),
            "apple".to_string(),
            "cherry".to_string(),
        ];
        sort_apply_str(&s, &mut v);
        assert_eq!(v[0], "apple");
    }

    #[test]
    fn test_sort_reverse() {
        /* reverse toggles direction */
        let mut s = new_sort_strategy("asc", true);
        sort_reverse(&mut s);
        assert!(!sort_is_ascending(&s));
    }

    #[test]
    fn test_sort_strategy_name() {
        /* name returned correctly */
        let s = new_sort_strategy("mySort", true);
        assert_eq!(sort_strategy_name(&s), "mySort");
    }

    #[test]
    fn test_sort_is_ascending() {
        /* ascending flag checked correctly */
        let s = new_sort_strategy("desc", false);
        assert!(!sort_is_ascending(&s));
    }
}
