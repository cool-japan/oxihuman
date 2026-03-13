// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Timezone offset calculator stub.

#[derive(Debug, Clone, PartialEq)]
pub struct TimezoneOffset {
    /// Offset in minutes from UTC (positive = east).
    pub minutes: i32,
    /// Optional IANA-style name.
    pub name: String,
}

impl TimezoneOffset {
    pub fn new(minutes: i32, name: &str) -> Self {
        TimezoneOffset {
            minutes,
            name: name.to_string(),
        }
    }

    pub fn utc() -> Self {
        TimezoneOffset::new(0, "UTC")
    }

    pub fn hours(&self) -> f32 {
        self.minutes as f32 / 60.0
    }

    pub fn is_positive(&self) -> bool {
        self.minutes >= 0
    }

    pub fn format_offset(&self) -> String {
        format_offset(self.minutes)
    }
}

pub fn format_offset(minutes: i32) -> String {
    let sign = if minutes >= 0 { '+' } else { '-' };
    let abs_min = minutes.unsigned_abs();
    let h = abs_min / 60;
    let m = abs_min % 60;
    format!("{}{:02}:{:02}", sign, h, m)
}

pub fn parse_offset(s: &str) -> Option<TimezoneOffset> {
    let s = s.trim();
    if s == "UTC" || s == "Z" {
        return Some(TimezoneOffset::utc());
    }
    let (sign, rest) = if let Some(r) = s.strip_prefix('+') {
        (1i32, r)
    } else if let Some(r) = s.strip_prefix('-') {
        (-1i32, r)
    } else {
        return None;
    };
    let parts: Vec<&str> = rest.splitn(2, ':').collect();
    let hours: i32 = parts.first()?.parse().ok()?;
    let mins: i32 = if parts.len() > 1 {
        parts[1].parse().ok()?
    } else {
        0
    };
    Some(TimezoneOffset::new(sign * (hours * 60 + mins), s))
}

pub fn offset_difference(a: &TimezoneOffset, b: &TimezoneOffset) -> i32 {
    a.minutes - b.minutes
}

pub fn convert_utc_minutes(utc_minutes: i64, offset: &TimezoneOffset) -> i64 {
    utc_minutes + offset.minutes as i64
}

pub fn known_offsets() -> Vec<TimezoneOffset> {
    vec![
        TimezoneOffset::new(0, "UTC"),
        TimezoneOffset::new(540, "Asia/Tokyo"),
        TimezoneOffset::new(-300, "America/New_York"),
        TimezoneOffset::new(-480, "America/Los_Angeles"),
        TimezoneOffset::new(60, "Europe/Paris"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utc_offset() {
        let tz = TimezoneOffset::utc();
        assert_eq!(tz.minutes, 0);
        assert_eq!(tz.format_offset(), "+00:00");
    }

    #[test]
    fn test_tokyo_offset() {
        let tz = TimezoneOffset::new(540, "Asia/Tokyo");
        assert_eq!(tz.hours(), 9.0);
        assert_eq!(tz.format_offset(), "+09:00");
    }

    #[test]
    fn test_negative_offset() {
        let tz = TimezoneOffset::new(-300, "America/New_York");
        assert_eq!(tz.format_offset(), "-05:00");
        assert!(!tz.is_positive() /* negative offset */,);
    }

    #[test]
    fn test_parse_offset_utc() {
        let tz = parse_offset("UTC").expect("should succeed");
        assert_eq!(tz.minutes, 0);
    }

    #[test]
    fn test_parse_offset_positive() {
        let tz = parse_offset("+09:00").expect("should succeed");
        assert_eq!(tz.minutes, 540);
    }

    #[test]
    fn test_parse_offset_negative() {
        let tz = parse_offset("-05:00").expect("should succeed");
        assert_eq!(tz.minutes, -300);
    }

    #[test]
    fn test_offset_difference() {
        let a = TimezoneOffset::new(540, "JST");
        let b = TimezoneOffset::new(0, "UTC");
        assert_eq!(offset_difference(&a, &b), 540);
    }

    #[test]
    fn test_convert_utc() {
        let tz = TimezoneOffset::new(540, "JST");
        let local = convert_utc_minutes(0, &tz);
        assert_eq!(local, 540);
    }

    #[test]
    fn test_known_offsets_nonempty() {
        let list = known_offsets();
        assert!(!list.is_empty(), /* known offsets list should not be empty */);
    }
}
