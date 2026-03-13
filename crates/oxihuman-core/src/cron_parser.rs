// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cron expression parser and next-fire-time calculator stub.

#[derive(Debug, Clone, PartialEq)]
pub struct CronExpr {
    pub minute: CronField,
    pub hour: CronField,
    pub day_of_month: CronField,
    pub month: CronField,
    pub day_of_week: CronField,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CronField {
    Wildcard,
    Value(u8),
    List(Vec<u8>),
    Range(u8, u8),
    Step(u8),
}

impl CronField {
    pub fn matches(&self, v: u8) -> bool {
        match self {
            CronField::Wildcard => true,
            CronField::Value(x) => *x == v,
            CronField::List(xs) => xs.contains(&v),
            CronField::Range(lo, hi) => v >= *lo && v <= *hi,
            CronField::Step(s) => *s > 0 && v.is_multiple_of(*s),
        }
    }
}

pub fn parse_cron(s: &str) -> Option<CronExpr> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 5 {
        return None;
    }
    Some(CronExpr {
        minute: parse_field(parts[0])?,
        hour: parse_field(parts[1])?,
        day_of_month: parse_field(parts[2])?,
        month: parse_field(parts[3])?,
        day_of_week: parse_field(parts[4])?,
    })
}

fn parse_field(s: &str) -> Option<CronField> {
    if s == "*" {
        return Some(CronField::Wildcard);
    }
    if let Some(step_str) = s.strip_prefix("*/") {
        let step: u8 = step_str.parse().ok()?;
        return Some(CronField::Step(step));
    }
    if s.contains('-') {
        let parts: Vec<&str> = s.splitn(2, '-').collect();
        let lo: u8 = parts[0].parse().ok()?;
        let hi: u8 = parts[1].parse().ok()?;
        return Some(CronField::Range(lo, hi));
    }
    if s.contains(',') {
        let vals: Option<Vec<u8>> = s.split(',').map(|x| x.parse::<u8>().ok()).collect();
        return Some(CronField::List(vals?));
    }
    let v: u8 = s.parse().ok()?;
    Some(CronField::Value(v))
}

/// Check if a given (minute, hour, day, month, weekday) fires.
pub fn cron_matches(
    expr: &CronExpr,
    minute: u8,
    hour: u8,
    day: u8,
    month: u8,
    weekday: u8,
) -> bool {
    expr.minute.matches(minute)
        && expr.hour.matches(hour)
        && expr.day_of_month.matches(day)
        && expr.month.matches(month)
        && expr.day_of_week.matches(weekday)
}

/// Describes a cron schedule in human-readable form.
pub fn describe_cron(expr: &CronExpr) -> String {
    format!(
        "min={:?} hr={:?} dom={:?} mon={:?} dow={:?}",
        expr.minute, expr.hour, expr.day_of_month, expr.month, expr.day_of_week
    )
}

pub fn cron_is_wildcard_all(expr: &CronExpr) -> bool {
    expr.minute == CronField::Wildcard
        && expr.hour == CronField::Wildcard
        && expr.day_of_month == CronField::Wildcard
        && expr.month == CronField::Wildcard
        && expr.day_of_week == CronField::Wildcard
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_wildcard_all() {
        let expr = parse_cron("* * * * *").expect("should succeed");
        assert!(cron_is_wildcard_all(&expr) /* all wildcards */,);
    }

    #[test]
    fn test_parse_specific() {
        let expr = parse_cron("30 8 * * 1").expect("should succeed");
        assert_eq!(expr.minute, CronField::Value(30));
        assert_eq!(expr.hour, CronField::Value(8));
    }

    #[test]
    fn test_matches_wildcard() {
        let expr = parse_cron("* * * * *").expect("should succeed");
        assert!(cron_matches(&expr, 0, 0, 1, 1, 0), /* wildcard matches anything */);
    }

    #[test]
    fn test_matches_value() {
        let expr = parse_cron("30 8 * * *").expect("should succeed");
        assert!(cron_matches(&expr, 30, 8, 1, 1, 0));
        assert!(!cron_matches(&expr, 31, 8, 1, 1, 0));
    }

    #[test]
    fn test_parse_range() {
        let expr = parse_cron("0 9-17 * * *").expect("should succeed");
        assert_eq!(expr.hour, CronField::Range(9, 17));
        assert!(expr.hour.matches(12) /* 12 is in 9-17 */,);
        assert!(!expr.hour.matches(8) /* 8 not in 9-17 */,);
    }

    #[test]
    fn test_parse_step() {
        let expr = parse_cron("*/15 * * * *").expect("should succeed");
        assert_eq!(expr.minute, CronField::Step(15));
        assert!(expr.minute.matches(0));
        assert!(expr.minute.matches(15));
        assert!(!expr.minute.matches(7));
    }

    #[test]
    fn test_parse_list() {
        let expr = parse_cron("0 8,12,18 * * *").expect("should succeed");
        assert!(expr.hour.matches(8));
        assert!(expr.hour.matches(12));
        assert!(!expr.hour.matches(10));
    }

    #[test]
    fn test_invalid_too_few_fields() {
        assert!(parse_cron("* * *").is_none() /* only 3 fields */,);
    }

    #[test]
    fn test_describe_cron() {
        let expr = parse_cron("* * * * *").expect("should succeed");
        let desc = describe_cron(&expr);
        assert!(desc.contains("Wildcard"), /* description includes Wildcard */);
    }
}
