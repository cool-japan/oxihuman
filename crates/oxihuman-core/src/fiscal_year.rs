// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Fiscal year period calculator stub.

/// Fiscal year configuration: defines the start month and day of each FY.
#[derive(Debug, Clone, PartialEq)]
pub struct FiscalYearConfig {
    /// Month (1-12) when the fiscal year starts.
    pub start_month: u8,
    /// Day (1-31) when the fiscal year starts.
    pub start_day: u8,
}

impl FiscalYearConfig {
    pub fn new(start_month: u8, start_day: u8) -> Self {
        FiscalYearConfig {
            start_month,
            start_day,
        }
    }

    /// Calendar year starting on Jan 1 (US GAAP / IFRS default).
    pub fn calendar_year() -> Self {
        FiscalYearConfig::new(1, 1)
    }

    /// Japanese fiscal year: April 1.
    pub fn japan() -> Self {
        FiscalYearConfig::new(4, 1)
    }

    /// UK fiscal year: April 6.
    pub fn uk() -> Self {
        FiscalYearConfig::new(4, 6)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FiscalPeriod {
    pub fiscal_year: i32,
    pub quarter: u8,
    pub start_month: u8,
    pub end_month: u8,
}

impl FiscalPeriod {
    pub fn label(&self) -> String {
        format!("FY{}Q{}", self.fiscal_year, self.quarter)
    }
}

/// Determine the fiscal year number for a calendar date.
pub fn fiscal_year_of(cal_year: i32, cal_month: u8, config: &FiscalYearConfig) -> i32 {
    if cal_month < config.start_month || (cal_month == config.start_month && config.start_day > 1) {
        cal_year - 1
    } else {
        cal_year
    }
}

/// Get the start and end calendar months of a fiscal quarter.
pub fn fiscal_quarter(cal_year: i32, cal_month: u8, config: &FiscalYearConfig) -> FiscalPeriod {
    let fy = fiscal_year_of(cal_year, cal_month, config);
    let months_since_start = if cal_month >= config.start_month {
        cal_month - config.start_month
    } else {
        12 - config.start_month + cal_month
    };
    let quarter = months_since_start / 3 + 1;
    let q_start_offset = (quarter - 1) * 3;
    let start_m = (config.start_month as u32 + q_start_offset as u32 - 1) % 12 + 1;
    let end_m = (start_m + 2 - 1) % 12 + 1;
    FiscalPeriod {
        fiscal_year: fy,
        quarter,
        start_month: start_m as u8,
        end_month: end_m as u8,
    }
}

/// Count the number of calendar months in a fiscal year (always 12).
pub fn fiscal_year_months() -> u8 {
    12
}

/// List all 4 quarters for a given fiscal year.
pub fn fiscal_year_quarters(fy: i32, config: &FiscalYearConfig) -> Vec<FiscalPeriod> {
    (0u8..4)
        .map(|q| {
            let m_offset = config.start_month as u32 + q as u32 * 3;
            let cal_m = ((m_offset - 1) % 12 + 1) as u8;
            let cal_y = if m_offset > 12 { fy + 1 } else { fy };
            fiscal_quarter(cal_y, cal_m, config)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calendar_year() {
        let cfg = FiscalYearConfig::calendar_year();
        assert_eq!(fiscal_year_of(2026, 3, &cfg), 2026);
    }

    #[test]
    fn test_japan_fy_before_april() {
        let cfg = FiscalYearConfig::japan();
        assert_eq!(
            fiscal_year_of(2026, 3, &cfg),
            2025, /* March is in FY2025 */
        );
    }

    #[test]
    fn test_japan_fy_in_april() {
        let cfg = FiscalYearConfig::japan();
        assert_eq!(
            fiscal_year_of(2026, 4, &cfg),
            2026, /* April starts FY2026 */
        );
    }

    #[test]
    fn test_quarter_label() {
        let cfg = FiscalYearConfig::calendar_year();
        let p = fiscal_quarter(2026, 1, &cfg);
        assert_eq!(p.label(), "FY2026Q1");
    }

    #[test]
    fn test_q1_q2_calendar() {
        let cfg = FiscalYearConfig::calendar_year();
        let q1 = fiscal_quarter(2026, 2, &cfg);
        let q2 = fiscal_quarter(2026, 5, &cfg);
        assert_eq!(q1.quarter, 1);
        assert_eq!(q2.quarter, 2);
    }

    #[test]
    fn test_fiscal_year_quarters_count() {
        let cfg = FiscalYearConfig::calendar_year();
        let qs = fiscal_year_quarters(2026, &cfg);
        assert_eq!(qs.len(), 4 /* always 4 quarters */,);
    }

    #[test]
    fn test_fiscal_year_months() {
        assert_eq!(fiscal_year_months(), 12);
    }

    #[test]
    fn test_japan_q1() {
        let cfg = FiscalYearConfig::japan();
        let p = fiscal_quarter(2026, 4, &cfg);
        assert_eq!(p.quarter, 1 /* April = Q1 in Japanese FY */,);
    }

    #[test]
    fn test_uk_config() {
        let cfg = FiscalYearConfig::uk();
        assert_eq!(cfg.start_month, 4);
        assert_eq!(cfg.start_day, 6);
    }
}
