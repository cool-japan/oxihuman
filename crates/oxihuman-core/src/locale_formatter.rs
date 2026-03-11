// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Locale-aware number and date formatter stub.

#[derive(Debug, Clone, PartialEq)]
pub enum LocaleId {
    EnUs,
    JaJp,
    DeDe,
    FrFr,
    ZhCn,
}

#[derive(Debug, Clone)]
pub struct LocaleFormatter {
    pub locale: LocaleId,
    pub decimal_sep: char,
    pub thousands_sep: char,
    pub date_format: String,
}

impl LocaleFormatter {
    pub fn new(locale: LocaleId) -> Self {
        let (decimal_sep, thousands_sep, date_format) = match &locale {
            LocaleId::EnUs => ('.', ',', "MM/DD/YYYY".to_string()),
            LocaleId::JaJp => ('.', ',', "YYYY年MM月DD日".to_string()),
            LocaleId::DeDe => (',', '.', "DD.MM.YYYY".to_string()),
            LocaleId::FrFr => (',', ' ', "DD/MM/YYYY".to_string()),
            LocaleId::ZhCn => ('.', ',', "YYYY-MM-DD".to_string()),
        };
        LocaleFormatter {
            locale,
            decimal_sep,
            thousands_sep,
            date_format,
        }
    }

    pub fn format_number(&self, value: f64, decimals: u8) -> String {
        format_number_locale(value, decimals, self.decimal_sep, self.thousands_sep)
    }

    pub fn format_date(&self, year: i32, month: u8, day: u8) -> String {
        format_date_locale(year, month, day, &self.date_format)
    }
}

pub fn format_number_locale(
    value: f64,
    decimals: u8,
    decimal_sep: char,
    thousands_sep: char,
) -> String {
    let factor = 10_f64.powi(decimals as i32);
    let rounded = (value * factor).round() / factor;
    let int_part = rounded.abs() as u64;
    let frac_part = ((rounded.abs() - int_part as f64) * factor).round() as u64;
    let sign = if value < 0.0 { "-" } else { "" };
    let int_str = format_thousands(int_part, thousands_sep);
    if decimals == 0 {
        format!("{}{}", sign, int_str)
    } else {
        format!(
            "{}{}{}{:0>width$}",
            sign,
            int_str,
            decimal_sep,
            frac_part,
            width = decimals as usize
        )
    }
}

pub fn format_thousands(mut n: u64, sep: char) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let mut digits: Vec<char> = Vec::new();
    let mut count = 0u32;
    while n > 0 {
        if count > 0 && count.is_multiple_of(3) {
            digits.push(sep);
        }
        digits.push(char::from_digit((n % 10) as u32, 10).unwrap_or('0'));
        n /= 10;
        count += 1;
    }
    digits.iter().rev().collect()
}

pub fn format_date_locale(year: i32, month: u8, day: u8, fmt: &str) -> String {
    fmt.replace("YYYY", &format!("{:04}", year))
        .replace("MM", &format!("{:02}", month))
        .replace("DD", &format!("{:02}", day))
}

pub fn locale_currency_symbol(locale: &LocaleId) -> &'static str {
    match locale {
        LocaleId::EnUs => "$",
        LocaleId::JaJp => "¥",
        LocaleId::DeDe | LocaleId::FrFr => "€",
        LocaleId::ZhCn => "¥",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number_en_us() {
        let fmt = LocaleFormatter::new(LocaleId::EnUs);
        let result = fmt.format_number(1234567.89, 2);
        assert!(result.contains('.') /* decimal separator present */,);
        assert!(result.contains(',') /* thousands separator present */,);
    }

    #[test]
    fn test_format_number_de() {
        let fmt = LocaleFormatter::new(LocaleId::DeDe);
        let result = fmt.format_number(1234.5, 2);
        assert!(result.contains(',') /* German decimal sep */,);
    }

    #[test]
    fn test_format_date_ja() {
        let fmt = LocaleFormatter::new(LocaleId::JaJp);
        let result = fmt.format_date(2026, 3, 7);
        assert!(result.contains("2026") /* year present */,);
        assert!(result.contains("03") /* month present */,);
    }

    #[test]
    fn test_format_date_us() {
        let fmt = LocaleFormatter::new(LocaleId::EnUs);
        let result = fmt.format_date(2026, 3, 7);
        assert_eq!(result, "03/07/2026");
    }

    #[test]
    fn test_format_thousands_zero() {
        assert_eq!(format_thousands(0, ','), "0");
    }

    #[test]
    fn test_format_thousands_small() {
        assert_eq!(format_thousands(999, ','), "999");
    }

    #[test]
    fn test_format_thousands_large() {
        let s = format_thousands(1_000_000, ',');
        assert_eq!(s, "1,000,000");
    }

    #[test]
    fn test_currency_symbol() {
        assert_eq!(locale_currency_symbol(&LocaleId::EnUs), "$");
        assert_eq!(locale_currency_symbol(&LocaleId::DeDe), "€");
    }

    #[test]
    fn test_negative_number() {
        let fmt = LocaleFormatter::new(LocaleId::EnUs);
        let result = fmt.format_number(-42.5, 1);
        assert!(result.starts_with('-') /* negative sign present */,);
    }
}
