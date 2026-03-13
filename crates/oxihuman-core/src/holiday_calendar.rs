// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Holiday calendar stub.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HolidayDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl HolidayDate {
    pub fn new(year: i32, month: u8, day: u8) -> Self {
        HolidayDate { year, month, day }
    }
}

#[derive(Debug, Clone)]
pub struct Holiday {
    pub date: HolidayDate,
    pub name: String,
    pub region: String,
}

impl Holiday {
    pub fn new(year: i32, month: u8, day: u8, name: &str, region: &str) -> Self {
        Holiday {
            date: HolidayDate::new(year, month, day),
            name: name.to_string(),
            region: region.to_string(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct HolidayCalendar {
    pub holidays: Vec<Holiday>,
}

impl HolidayCalendar {
    pub fn new() -> Self {
        HolidayCalendar::default()
    }

    pub fn add(&mut self, holiday: Holiday) {
        self.holidays.push(holiday);
    }

    pub fn is_holiday(&self, year: i32, month: u8, day: u8) -> bool {
        is_holiday_in(&self.holidays, year, month, day)
    }

    pub fn holidays_in_month(&self, year: i32, month: u8) -> Vec<&Holiday> {
        self.holidays
            .iter()
            .filter(|h| h.date.year == year && h.date.month == month)
            .collect()
    }

    pub fn count(&self) -> usize {
        self.holidays.len()
    }
}

pub fn is_holiday_in(holidays: &[Holiday], year: i32, month: u8, day: u8) -> bool {
    let target = HolidayDate::new(year, month, day);
    holidays.iter().any(|h| h.date == target)
}

pub fn jp_national_holidays(year: i32) -> HolidayCalendar {
    let mut cal = HolidayCalendar::new();
    let fixed = [
        (1, 1, "元日"),
        (2, 11, "建国記念の日"),
        (2, 23, "天皇誕生日"),
        (4, 29, "昭和の日"),
        (5, 3, "憲法記念日"),
        (5, 4, "みどりの日"),
        (5, 5, "こどもの日"),
        (8, 11, "山の日"),
        (11, 3, "文化の日"),
        (11, 23, "勤労感謝の日"),
    ];
    for (m, d, name) in fixed {
        cal.add(Holiday::new(year, m, d, name, "JP"));
    }
    cal
}

pub fn us_federal_holidays(year: i32) -> HolidayCalendar {
    let mut cal = HolidayCalendar::new();
    let fixed = [
        (1, 1, "New Year's Day"),
        (7, 4, "Independence Day"),
        (11, 11, "Veterans Day"),
        (12, 25, "Christmas Day"),
    ];
    for (m, d, name) in fixed {
        cal.add(Holiday::new(year, m, d, name, "US"));
    }
    cal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jp_holidays_has_entries() {
        let cal = jp_national_holidays(2026);
        assert!(!cal.holidays.is_empty() /* JP calendar has holidays */,);
    }

    #[test]
    fn test_new_years_day_jp() {
        let cal = jp_national_holidays(2026);
        assert!(cal.is_holiday(2026, 1, 1) /* New Year is holiday */,);
    }

    #[test]
    fn test_non_holiday() {
        let cal = jp_national_holidays(2026);
        assert!(!cal.is_holiday(2026, 3, 7), /* March 7 is not a JP holiday */);
    }

    #[test]
    fn test_us_holidays() {
        let cal = us_federal_holidays(2026);
        assert!(cal.is_holiday(2026, 7, 4) /* Independence Day */,);
    }

    #[test]
    fn test_holidays_in_month() {
        let cal = jp_national_holidays(2026);
        let may = cal.holidays_in_month(2026, 5);
        assert_eq!(may.len(), 3 /* May has 3 JP holidays */,);
    }

    #[test]
    fn test_count() {
        let cal = jp_national_holidays(2026);
        assert_eq!(cal.count(), 10);
    }

    #[test]
    fn test_add_custom() {
        let mut cal = HolidayCalendar::new();
        cal.add(Holiday::new(2026, 6, 15, "Custom Day", "XX"));
        assert!(cal.is_holiday(2026, 6, 15) /* custom holiday */,);
    }

    #[test]
    fn test_empty_calendar() {
        let cal = HolidayCalendar::new();
        assert_eq!(cal.count(), 0);
    }

    #[test]
    fn test_is_holiday_in_slice() {
        let holidays = vec![Holiday::new(2026, 1, 1, "Test", "XX")];
        assert!(is_holiday_in(&holidays, 2026, 1, 1));
        assert!(!is_holiday_in(&holidays, 2026, 1, 2));
    }
}
