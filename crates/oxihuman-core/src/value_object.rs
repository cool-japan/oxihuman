// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct Money {
    pub amount_cents: i64,
    pub currency: String,
}

pub struct Percentage {
    pub value: f32,
}

pub fn new_money(cents: i64, currency: &str) -> Money {
    Money {
        amount_cents: cents,
        currency: currency.to_string(),
    }
}

pub fn money_add(a: &Money, b: &Money) -> Option<Money> {
    if a.currency != b.currency {
        return None;
    }
    Some(Money {
        amount_cents: a.amount_cents + b.amount_cents,
        currency: a.currency.clone(),
    })
}

pub fn money_to_string(m: &Money) -> String {
    let dollars = m.amount_cents / 100;
    let cents = (m.amount_cents % 100).unsigned_abs();
    format!("{}.{:02} {}", dollars, cents, m.currency)
}

pub fn new_percentage(v: f32) -> Percentage {
    Percentage { value: v }
}

pub fn percentage_clamp(p: &mut Percentage) {
    p.value = p.value.clamp(0.0, 100.0);
}

pub fn percentage_to_fraction(p: &Percentage) -> f32 {
    p.value / 100.0
}

pub fn percentage_from_fraction(f: f32) -> Percentage {
    Percentage { value: f * 100.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_money() {
        /* create money value */
        let m = new_money(1050, "USD");
        assert_eq!(m.amount_cents, 1050);
    }

    #[test]
    fn test_money_add_same_currency() {
        /* add same currency gives sum */
        let a = new_money(100, "EUR");
        let b = new_money(250, "EUR");
        let r = money_add(&a, &b).unwrap();
        assert_eq!(r.amount_cents, 350);
    }

    #[test]
    fn test_money_add_diff_currency() {
        /* different currencies returns None */
        let a = new_money(100, "USD");
        let b = new_money(100, "EUR");
        assert!(money_add(&a, &b).is_none());
    }

    #[test]
    fn test_money_to_string() {
        /* formats as dollars.cents CURRENCY */
        let m = new_money(1099, "USD");
        let s = money_to_string(&m);
        assert!(s.contains("10.99"));
        assert!(s.contains("USD"));
    }

    #[test]
    fn test_percentage_clamp() {
        /* clamps to 0-100 */
        let mut p = new_percentage(150.0);
        percentage_clamp(&mut p);
        assert_eq!(p.value, 100.0);
    }

    #[test]
    fn test_percentage_to_fraction() {
        /* 50% = 0.5 */
        let p = new_percentage(50.0);
        assert!((percentage_to_fraction(&p) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_percentage_from_fraction() {
        /* 0.25 = 25% */
        let p = percentage_from_fraction(0.25);
        assert!((p.value - 25.0).abs() < 1e-5);
    }
}
