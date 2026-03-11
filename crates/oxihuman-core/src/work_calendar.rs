// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Business day calculator stub.

use crate::calendar_util::{date_to_julian_day, day_of_week, julian_day_to_date};

#[derive(Debug, Clone, Default)]
pub struct WorkCalendar {
    /// Set of (year, month, day) tuples representing holidays/non-working days.
    pub holidays: Vec<(i32, u8, u8)>,
    /// Work week: set of weekdays (0=Sun, 1=Mon, …, 6=Sat) that are working days.
    pub work_days: Vec<u8>,
}

impl WorkCalendar {
    pub fn new_mon_fri() -> Self {
        WorkCalendar {
            holidays: Vec::new(),
            work_days: vec![1, 2, 3, 4, 5],
        }
    }

    pub fn add_holiday(&mut self, year: i32, month: u8, day: u8) {
        self.holidays.push((year, month, day));
    }

    pub fn is_working_day(&self, year: i32, month: u8, day: u8) -> bool {
        is_business_day(year, month, day, &self.work_days, &self.holidays)
    }

    pub fn next_business_day(&self, year: i32, month: u8, day: u8) -> (i32, u8, u8) {
        next_bday(year, month, day, &self.work_days, &self.holidays)
    }

    pub fn business_days_between(&self, y1: i32, m1: u8, d1: u8, y2: i32, m2: u8, d2: u8) -> u32 {
        count_bdays(y1, m1, d1, y2, m2, d2, &self.work_days, &self.holidays)
    }
}

pub fn is_business_day(
    year: i32,
    month: u8,
    day: u8,
    work_days: &[u8],
    holidays: &[(i32, u8, u8)],
) -> bool {
    let dow = day_of_week(year, month, day);
    if !work_days.contains(&dow) {
        return false;
    }
    !holidays.contains(&(year, month, day))
}

pub fn next_bday(
    year: i32,
    month: u8,
    day: u8,
    work_days: &[u8],
    holidays: &[(i32, u8, u8)],
) -> (i32, u8, u8) {
    let mut jdn = date_to_julian_day(year, month, day) + 1;
    loop {
        let d = julian_day_to_date(jdn);
        if is_business_day(d.year, d.month, d.day, work_days, holidays) {
            return (d.year, d.month, d.day);
        }
        jdn += 1;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn count_bdays(
    y1: i32,
    m1: u8,
    d1: u8,
    y2: i32,
    m2: u8,
    d2: u8,
    work_days: &[u8],
    holidays: &[(i32, u8, u8)],
) -> u32 {
    let start = date_to_julian_day(y1, m1, d1);
    let end = date_to_julian_day(y2, m2, d2);
    if end <= start {
        return 0;
    }
    (start..end)
        .filter(|&jdn| {
            let d = julian_day_to_date(jdn);
            is_business_day(d.year, d.month, d.day, work_days, holidays)
        })
        .count() as u32
}

pub fn add_business_days(
    year: i32,
    month: u8,
    day: u8,
    n: u32,
    work_days: &[u8],
    holidays: &[(i32, u8, u8)],
) -> (i32, u8, u8) {
    let mut jdn = date_to_julian_day(year, month, day);
    let mut remaining = n;
    while remaining > 0 {
        jdn += 1;
        let d = julian_day_to_date(jdn);
        if is_business_day(d.year, d.month, d.day, work_days, holidays) {
            remaining = remaining.saturating_sub(1);
        }
    }
    let d = julian_day_to_date(jdn);
    (d.year, d.month, d.day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monday_is_working() {
        let cal = WorkCalendar::new_mon_fri();
        /* 2026-03-09 is a Monday */
        assert!(cal.is_working_day(2026, 3, 9), /* Monday = working day */);
    }

    #[test]
    fn test_saturday_not_working() {
        let cal = WorkCalendar::new_mon_fri();
        /* 2026-03-07 is a Saturday */
        assert!(!cal.is_working_day(2026, 3, 7), /* Saturday = non-working */);
    }

    #[test]
    fn test_holiday_not_working() {
        let mut cal = WorkCalendar::new_mon_fri();
        cal.add_holiday(2026, 3, 9);
        assert!(!cal.is_working_day(2026, 3, 9) /* holiday added */,);
    }

    #[test]
    fn test_next_business_day_from_friday() {
        let cal = WorkCalendar::new_mon_fri();
        /* 2026-03-06 is a Friday; next business day = Monday 2026-03-09 */
        let (y, m, d) = cal.next_business_day(2026, 3, 6);
        assert_eq!((y, m, d), (2026, 3, 9));
    }

    #[test]
    fn test_business_days_one_week() {
        let cal = WorkCalendar::new_mon_fri();
        /* Mon-Fri = 5 business days in week */
        let n = cal.business_days_between(2026, 3, 9, 2026, 3, 13);
        assert_eq!(n, 4 /* Mon through Thu exclusive of end */,);
    }

    #[test]
    fn test_add_business_days() {
        let cal = WorkCalendar::new_mon_fri();
        let (y, m, d) = add_business_days(2026, 3, 9, 5, &cal.work_days, &cal.holidays);
        /* 5 bdays from Monday = next Monday */
        assert_eq!((y, m, d), (2026, 3, 16));
    }

    #[test]
    fn test_bdays_zero() {
        let cal = WorkCalendar::new_mon_fri();
        assert_eq!(cal.business_days_between(2026, 3, 9, 2026, 3, 9), 0);
    }

    #[test]
    fn test_add_zero_bdays() {
        let cal = WorkCalendar::new_mon_fri();
        let (y, m, d) = add_business_days(2026, 3, 9, 0, &cal.work_days, &cal.holidays);
        assert_eq!((y, m, d), (2026, 3, 9));
    }
}
