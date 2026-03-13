// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Calendar date arithmetic: Julian day, days-in-month, etc.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl CalDate {
    pub fn new(year: i32, month: u8, day: u8) -> Self {
        CalDate { year, month, day }
    }

    pub fn to_julian_day(&self) -> i64 {
        date_to_julian_day(self.year, self.month, self.day)
    }

    pub fn days_in_month(&self) -> u8 {
        days_in_month(self.year, self.month)
    }

    pub fn is_leap_year(&self) -> bool {
        is_leap_year(self.year)
    }
}

pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

pub fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Compute Julian Day Number (JDN) for a proleptic Gregorian calendar date.
pub fn date_to_julian_day(year: i32, month: u8, day: u8) -> i64 {
    let a = (14 - month as i32) / 12;
    let y = year + 4800 - a;
    let m = month as i32 + 12 * a - 3;
    day as i64 + (153 * m as i64 + 2) / 5 + 365 * y as i64 + y as i64 / 4 - y as i64 / 100
        + y as i64 / 400
        - 32045
}

/// Reconstruct a date from a Julian Day Number.
pub fn julian_day_to_date(jdn: i64) -> CalDate {
    let a = jdn + 32044;
    let b = (4 * a + 3) / 146097;
    let c = a - (146097 * b) / 4;
    let d = (4 * c + 3) / 1461;
    let e = c - (1461 * d) / 4;
    let m = (5 * e + 2) / 153;
    let day = (e - (153 * m + 2) / 5 + 1) as u8;
    let month = (m + 3 - 12 * (m / 10)) as u8;
    let year = (100 * b + d - 4800 + m / 10) as i32;
    CalDate { year, month, day }
}

/// Day-of-week: 0=Sunday, 1=Monday, …, 6=Saturday.
pub fn day_of_week(year: i32, month: u8, day: u8) -> u8 {
    let jdn = date_to_julian_day(year, month, day);
    ((jdn + 1) % 7) as u8
}

/// Days between two dates (b - a).
pub fn days_between(a: &CalDate, b: &CalDate) -> i64 {
    b.to_julian_day() - a.to_julian_day()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leap_year() {
        assert!(is_leap_year(2000) /* 2000 is leap */,);
        assert!(!is_leap_year(1900) /* 1900 is not leap */,);
        assert!(is_leap_year(2024) /* 2024 is leap */,);
    }

    #[test]
    fn test_days_in_feb_leap() {
        assert_eq!(days_in_month(2024, 2), 29);
    }

    #[test]
    fn test_days_in_feb_non_leap() {
        assert_eq!(days_in_month(2023, 2), 28);
    }

    #[test]
    fn test_days_in_january() {
        assert_eq!(days_in_month(2026, 1), 31);
    }

    #[test]
    fn test_julian_roundtrip() {
        let date = CalDate::new(2026, 3, 7);
        let jdn = date.to_julian_day();
        let back = julian_day_to_date(jdn);
        assert_eq!(back, date);
    }

    #[test]
    fn test_julian_epoch() {
        /* J2000.0 epoch = 2000-01-01 → JDN 2451545 */
        let jdn = date_to_julian_day(2000, 1, 1);
        assert_eq!(jdn, 2451545);
    }

    #[test]
    fn test_day_of_week() {
        /* 2026-03-07 is a Saturday (6) */
        let dow = day_of_week(2026, 3, 7);
        assert_eq!(dow, 6);
    }

    #[test]
    fn test_days_between() {
        let a = CalDate::new(2026, 1, 1);
        let b = CalDate::new(2026, 1, 31);
        assert_eq!(days_between(&a, &b), 30);
    }

    #[test]
    fn test_caldate_methods() {
        let d = CalDate::new(2024, 2, 29);
        assert!(d.is_leap_year() /* 2024 leap */,);
        assert_eq!(d.days_in_month(), 29);
    }
}
