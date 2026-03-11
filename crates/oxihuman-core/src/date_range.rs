// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Date range iterator and overlap check.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateRange {
    /// Inclusive start (Julian Day Number).
    pub start_jdn: i64,
    /// Inclusive end (Julian Day Number).
    pub end_jdn: i64,
}

impl DateRange {
    pub fn new(start_jdn: i64, end_jdn: i64) -> Self {
        DateRange { start_jdn, end_jdn }
    }

    pub fn from_ymd(sy: i32, sm: u8, sd: u8, ey: i32, em: u8, ed: u8) -> Self {
        DateRange::new(ymd_to_jdn(sy, sm, sd), ymd_to_jdn(ey, em, ed))
    }

    pub fn duration_days(&self) -> i64 {
        (self.end_jdn - self.start_jdn).max(0)
    }

    pub fn contains_jdn(&self, jdn: i64) -> bool {
        jdn >= self.start_jdn && jdn <= self.end_jdn
    }

    pub fn is_empty(&self) -> bool {
        self.start_jdn > self.end_jdn
    }
}

pub fn ymd_to_jdn(year: i32, month: u8, day: u8) -> i64 {
    let a = (14 - month as i32) / 12;
    let y = year + 4800 - a;
    let m = month as i32 + 12 * a - 3;
    day as i64 + (153 * m as i64 + 2) / 5 + 365 * y as i64 + y as i64 / 4 - y as i64 / 100
        + y as i64 / 400
        - 32045
}

pub fn ranges_overlap(a: &DateRange, b: &DateRange) -> bool {
    a.start_jdn <= b.end_jdn && b.start_jdn <= a.end_jdn
}

pub fn range_intersection(a: &DateRange, b: &DateRange) -> Option<DateRange> {
    let start = a.start_jdn.max(b.start_jdn);
    let end = a.end_jdn.min(b.end_jdn);
    if start <= end {
        Some(DateRange::new(start, end))
    } else {
        None
    }
}

pub fn range_union(a: &DateRange, b: &DateRange) -> DateRange {
    DateRange::new(a.start_jdn.min(b.start_jdn), a.end_jdn.max(b.end_jdn))
}

/// Iterate over JDN values in a range, collecting them.
pub fn range_jdn_list(r: &DateRange) -> Vec<i64> {
    (r.start_jdn..=r.end_jdn).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_days() {
        let r = DateRange::from_ymd(2026, 1, 1, 2026, 1, 31);
        assert_eq!(r.duration_days(), 30);
    }

    #[test]
    fn test_contains() {
        let r = DateRange::from_ymd(2026, 1, 1, 2026, 12, 31);
        let mid = ymd_to_jdn(2026, 6, 15);
        assert!(r.contains_jdn(mid) /* mid year is inside */,);
    }

    #[test]
    fn test_not_contains() {
        let r = DateRange::from_ymd(2026, 1, 1, 2026, 6, 30);
        let after = ymd_to_jdn(2026, 7, 1);
        assert!(!r.contains_jdn(after) /* July 1 is outside */,);
    }

    #[test]
    fn test_overlap_true() {
        let a = DateRange::from_ymd(2026, 1, 1, 2026, 6, 30);
        let b = DateRange::from_ymd(2026, 4, 1, 2026, 9, 30);
        assert!(ranges_overlap(&a, &b) /* overlapping ranges */,);
    }

    #[test]
    fn test_overlap_false() {
        let a = DateRange::from_ymd(2026, 1, 1, 2026, 3, 31);
        let b = DateRange::from_ymd(2026, 4, 1, 2026, 6, 30);
        assert!(!ranges_overlap(&a, &b) /* non-overlapping ranges */,);
    }

    #[test]
    fn test_intersection() {
        let a = DateRange::from_ymd(2026, 1, 1, 2026, 6, 30);
        let b = DateRange::from_ymd(2026, 3, 1, 2026, 9, 30);
        let inter = range_intersection(&a, &b).unwrap();
        assert_eq!(inter.start_jdn, ymd_to_jdn(2026, 3, 1));
        assert_eq!(inter.end_jdn, ymd_to_jdn(2026, 6, 30));
    }

    #[test]
    fn test_intersection_none() {
        let a = DateRange::from_ymd(2026, 1, 1, 2026, 3, 31);
        let b = DateRange::from_ymd(2026, 4, 1, 2026, 6, 30);
        assert!(range_intersection(&a, &b).is_none(), /* no intersection */);
    }

    #[test]
    fn test_union() {
        let a = DateRange::from_ymd(2026, 1, 1, 2026, 6, 30);
        let b = DateRange::from_ymd(2026, 3, 1, 2026, 9, 30);
        let u = range_union(&a, &b);
        assert_eq!(u.start_jdn, ymd_to_jdn(2026, 1, 1));
        assert_eq!(u.end_jdn, ymd_to_jdn(2026, 9, 30));
    }

    #[test]
    fn test_range_jdn_list() {
        let r = DateRange::new(0, 4);
        let list = range_jdn_list(&r);
        assert_eq!(list.len(), 5);
        assert_eq!(list[0], 0);
        assert_eq!(list[4], 4);
    }
}
