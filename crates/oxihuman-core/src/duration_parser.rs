// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! ISO 8601 duration string parser.

#[derive(Debug, Clone, PartialEq, Default)]
pub struct IsoDuration {
    pub years: u32,
    pub months: u32,
    pub weeks: u32,
    pub days: u32,
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
}

impl IsoDuration {
    pub fn new() -> Self {
        IsoDuration::default()
    }

    /// Approximate total seconds (ignoring calendar months/years for simplicity).
    pub fn approx_seconds(&self) -> u64 {
        let days_total = self.years as u64 * 365
            + self.months as u64 * 30
            + self.weeks as u64 * 7
            + self.days as u64;
        days_total * 86400
            + self.hours as u64 * 3600
            + self.minutes as u64 * 60
            + self.seconds as u64
    }

    pub fn is_zero(&self) -> bool {
        self.approx_seconds() == 0
    }
}

/// Parse an ISO 8601 duration string like `P1Y2M3DT4H5M6S`.
pub fn parse_duration(s: &str) -> Option<IsoDuration> {
    let s = s.trim();
    if !s.starts_with('P') {
        return None;
    }
    let mut dur = IsoDuration::default();
    let inner = &s[1..];
    let (date_part, time_part) = if let Some(t_pos) = inner.find('T') {
        (&inner[..t_pos], Some(&inner[t_pos + 1..]))
    } else {
        (inner, None)
    };
    parse_date_part(date_part, &mut dur)?;
    if let Some(tp) = time_part {
        parse_time_part(tp, &mut dur)?;
    }
    Some(dur)
}

fn parse_date_part(s: &str, dur: &mut IsoDuration) -> Option<()> {
    let mut buf = String::new();
    for ch in s.chars() {
        if ch.is_ascii_digit() {
            buf.push(ch);
        } else {
            let n: u32 = buf.parse().ok()?;
            buf.clear();
            match ch {
                'Y' => dur.years = n,
                'M' => dur.months = n,
                'W' => dur.weeks = n,
                'D' => dur.days = n,
                _ => return None,
            }
        }
    }
    Some(())
}

fn parse_time_part(s: &str, dur: &mut IsoDuration) -> Option<()> {
    let mut buf = String::new();
    for ch in s.chars() {
        if ch.is_ascii_digit() {
            buf.push(ch);
        } else {
            let n: u32 = buf.parse().ok()?;
            buf.clear();
            match ch {
                'H' => dur.hours = n,
                'M' => dur.minutes = n,
                'S' => dur.seconds = n,
                _ => return None,
            }
        }
    }
    Some(())
}

pub fn duration_to_string(d: &IsoDuration) -> String {
    let mut s = String::from("P");
    if d.years > 0 {
        s.push_str(&format!("{}Y", d.years));
    }
    if d.months > 0 {
        s.push_str(&format!("{}M", d.months));
    }
    if d.weeks > 0 {
        s.push_str(&format!("{}W", d.weeks));
    }
    if d.days > 0 {
        s.push_str(&format!("{}D", d.days));
    }
    if d.hours > 0 || d.minutes > 0 || d.seconds > 0 {
        s.push('T');
        if d.hours > 0 {
            s.push_str(&format!("{}H", d.hours));
        }
        if d.minutes > 0 {
            s.push_str(&format!("{}M", d.minutes));
        }
        if d.seconds > 0 {
            s.push_str(&format!("{}S", d.seconds));
        }
    }
    if s == "P" {
        s.push_str("0D");
    }
    s
}

pub fn add_durations(a: &IsoDuration, b: &IsoDuration) -> IsoDuration {
    IsoDuration {
        years: a.years + b.years,
        months: a.months + b.months,
        weeks: a.weeks + b.weeks,
        days: a.days + b.days,
        hours: a.hours + b.hours,
        minutes: a.minutes + b.minutes,
        seconds: a.seconds + b.seconds,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full() {
        let d = parse_duration("P1Y2M3DT4H5M6S").unwrap();
        assert_eq!(d.years, 1);
        assert_eq!(d.months, 2);
        assert_eq!(d.days, 3);
        assert_eq!(d.hours, 4);
        assert_eq!(d.minutes, 5);
        assert_eq!(d.seconds, 6);
    }

    #[test]
    fn test_parse_date_only() {
        let d = parse_duration("P10D").unwrap();
        assert_eq!(d.days, 10);
        assert_eq!(d.hours, 0);
    }

    #[test]
    fn test_parse_time_only() {
        let d = parse_duration("PT30M").unwrap();
        assert_eq!(d.minutes, 30);
    }

    #[test]
    fn test_invalid_no_p() {
        assert!(parse_duration("1Y2M").is_none(), /* must start with P */);
    }

    #[test]
    fn test_approx_seconds_one_day() {
        let d = parse_duration("P1D").unwrap();
        assert_eq!(d.approx_seconds(), 86400);
    }

    #[test]
    fn test_is_zero() {
        let d = IsoDuration::default();
        assert!(d.is_zero() /* default is zero */,);
    }

    #[test]
    fn test_to_string_roundtrip() {
        let d = parse_duration("P1Y3DT2H").unwrap();
        let s = duration_to_string(&d);
        let d2 = parse_duration(&s).unwrap();
        assert_eq!(d, d2);
    }

    #[test]
    fn test_add_durations() {
        let a = parse_duration("P1DT1H").unwrap();
        let b = parse_duration("P2DT2H").unwrap();
        let c = add_durations(&a, &b);
        assert_eq!(c.days, 3);
        assert_eq!(c.hours, 3);
    }

    #[test]
    fn test_parse_week() {
        let d = parse_duration("P2W").unwrap();
        assert_eq!(d.weeks, 2);
        assert_eq!(d.approx_seconds(), 2 * 7 * 86400);
    }
}
