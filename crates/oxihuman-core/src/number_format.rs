// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Format numbers with separators, precision, and SI prefixes.

#![allow(dead_code)]

/// Format an integer with thousands separator.
#[allow(dead_code)]
pub fn format_int_sep(n: i64, sep: char) -> String {
    let neg = n < 0;
    let digits = format!("{}", n.unsigned_abs());
    let mut out = String::new();
    let len = digits.len();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            out.push(sep);
        }
        out.push(ch);
    }
    if neg {
        format!("-{}", out)
    } else {
        out
    }
}

/// Format a float with given decimal places.
#[allow(dead_code)]
pub fn format_float(x: f64, decimals: usize) -> String {
    format!("{:.prec$}", x, prec = decimals)
}

/// Format a float with thousands separator in the integer part.
#[allow(dead_code)]
pub fn format_float_sep(x: f64, decimals: usize, sep: char) -> String {
    let neg = x < 0.0;
    let abs_x = x.abs();
    let int_part = abs_x.floor() as i64;
    let frac = abs_x - int_part as f64;
    let int_str = format_int_sep(int_part, sep);
    let frac_str = if decimals > 0 {
        let factor = 10f64.powi(decimals as i32);
        let frac_int = (frac * factor).round() as u64;
        format!(".{:0>width$}", frac_int, width = decimals)
    } else {
        String::new()
    };
    if neg {
        format!("-{}{}", int_str, frac_str)
    } else {
        format!("{}{}", int_str, frac_str)
    }
}

/// Format a number with an SI prefix (k, M, G, T, m, μ, n).
#[allow(dead_code)]
pub fn format_si(x: f64, decimals: usize) -> String {
    let (value, prefix) = if x.abs() >= 1e12 {
        (x / 1e12, "T")
    } else if x.abs() >= 1e9 {
        (x / 1e9, "G")
    } else if x.abs() >= 1e6 {
        (x / 1e6, "M")
    } else if x.abs() >= 1e3 {
        (x / 1e3, "k")
    } else if x.abs() >= 1.0 || x == 0.0 {
        (x, "")
    } else if x.abs() >= 1e-3 {
        (x * 1e3, "m")
    } else if x.abs() >= 1e-6 {
        (x * 1e6, "u")
    } else {
        (x * 1e9, "n")
    };
    format!("{:.prec$}{}", value, prefix, prec = decimals)
}

/// Format a percentage (0.0–1.0 -> "X.Y%").
#[allow(dead_code)]
pub fn format_percent(ratio: f64, decimals: usize) -> String {
    format!("{:.prec$}%", ratio * 100.0, prec = decimals)
}

/// Format bytes with B, KB, MB, GB suffix.
#[allow(dead_code)]
pub fn format_bytes(n: u64) -> String {
    if n >= 1_073_741_824 {
        format!("{:.2} GB", n as f64 / 1_073_741_824.0)
    } else if n >= 1_048_576 {
        format!("{:.2} MB", n as f64 / 1_048_576.0)
    } else if n >= 1_024 {
        format!("{:.2} KB", n as f64 / 1_024.0)
    } else {
        format!("{} B", n)
    }
}

/// Left-pad a string to the given width with a fill character.
#[allow(dead_code)]
pub fn pad_left(s: &str, width: usize, fill: char) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        let pad: String = std::iter::repeat_n(fill, width - s.len()).collect();
        format!("{}{}", pad, s)
    }
}

/// Right-pad a string.
#[allow(dead_code)]
pub fn pad_right(s: &str, width: usize, fill: char) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        let pad: String = std::iter::repeat_n(fill, width - s.len()).collect();
        format!("{}{}", s, pad)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_int_sep_basic() {
        assert_eq!(format_int_sep(1_000_000, ','), "1,000,000");
    }

    #[test]
    fn format_int_sep_negative() {
        assert_eq!(format_int_sep(-1234, ','), "-1,234");
    }

    #[test]
    fn format_int_sep_small() {
        assert_eq!(format_int_sep(42, ','), "42");
    }

    #[test]
    fn format_float_precision() {
        // 1.234 -> "1.23" at 2 decimal places
        assert_eq!(format_float(1.234, 2), "1.23");
    }

    #[test]
    fn format_si_kilo() {
        let s = format_si(1500.0, 1);
        assert!(s.contains('k'));
        assert!(s.contains("1.5"));
    }

    #[test]
    fn format_si_mega() {
        let s = format_si(2_000_000.0, 0);
        assert!(s.contains('M'));
    }

    #[test]
    fn format_percent_basic() {
        let s = format_percent(0.75, 1);
        assert_eq!(s, "75.0%");
    }

    #[test]
    fn format_bytes_gb() {
        let s = format_bytes(2_147_483_648);
        assert!(s.contains("GB"));
    }

    #[test]
    fn pad_left_basic() {
        assert_eq!(pad_left("42", 5, '0'), "00042");
    }

    #[test]
    fn format_float_sep_basic() {
        let s = format_float_sep(1234567.89, 2, ',');
        assert!(s.contains(','));
        assert!(s.contains('.'));
    }
}
