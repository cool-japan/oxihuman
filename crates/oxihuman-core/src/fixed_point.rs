// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A 16.16 fixed-point number for deterministic arithmetic.

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Fixed32(i32);

#[allow(dead_code)]
const FRAC_BITS: i32 = 16;
#[allow(dead_code)]
const SCALE: i32 = 1 << FRAC_BITS;

#[allow(dead_code)]
impl Fixed32 {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(SCALE);
    pub const HALF: Self = Self(SCALE / 2);

    pub fn from_int(v: i32) -> Self {
        Self(v.saturating_mul(SCALE))
    }

    pub fn from_f32(v: f32) -> Self {
        Self((v * SCALE as f32) as i32)
    }

    pub fn to_f32(self) -> f32 {
        self.0 as f32 / SCALE as f32
    }

    pub fn to_int(self) -> i32 {
        self.0 >> FRAC_BITS
    }

    pub fn raw(self) -> i32 {
        self.0
    }

    pub fn from_raw(raw: i32) -> Self {
        Self(raw)
    }

    pub fn saturating_add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }

    pub fn saturating_sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }

    pub fn fixed_mul(self, rhs: Self) -> Self {
        let wide = self.0 as i64 * rhs.0 as i64;
        Self((wide >> FRAC_BITS) as i32)
    }

    pub fn checked_div(self, rhs: Self) -> Option<Self> {
        if rhs.0 == 0 {
            return None;
        }
        let wide = (self.0 as i64) << FRAC_BITS;
        Some(Self((wide / rhs.0 as i64) as i32))
    }

    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }

    pub fn negate(self) -> Self {
        Self(-self.0)
    }

    pub fn floor(self) -> Self {
        Self(self.0 & !(SCALE - 1))
    }

    pub fn ceil(self) -> Self {
        Self((self.0 + SCALE - 1) & !(SCALE - 1))
    }

    pub fn frac(self) -> Self {
        Self(self.0 & (SCALE - 1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_int() {
        let f = Fixed32::from_int(5);
        assert_eq!(f.to_int(), 5);
    }

    #[test]
    fn test_from_f32() {
        let f = Fixed32::from_f32(1.5);
        assert!((f.to_f32() - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_add() {
        let a = Fixed32::from_int(3);
        let b = Fixed32::from_int(4);
        assert_eq!(a.saturating_add(b).to_int(), 7);
    }

    #[test]
    fn test_sub() {
        let a = Fixed32::from_int(10);
        let b = Fixed32::from_int(3);
        assert_eq!(a.saturating_sub(b).to_int(), 7);
    }

    #[test]
    fn test_mul() {
        let a = Fixed32::from_int(3);
        let b = Fixed32::from_f32(2.5);
        let r = a.fixed_mul(b);
        assert!((r.to_f32() - 7.5).abs() < 0.01);
    }

    #[test]
    fn test_div() {
        let a = Fixed32::from_int(10);
        let b = Fixed32::from_int(4);
        let r = a.checked_div(b).unwrap();
        assert!((r.to_f32() - 2.5).abs() < 0.01);
    }

    #[test]
    fn test_div_by_zero() {
        assert!(Fixed32::ONE.checked_div(Fixed32::ZERO).is_none());
    }

    #[test]
    fn test_abs_neg() {
        let f = Fixed32::from_int(-5);
        assert_eq!(f.abs().to_int(), 5);
        assert_eq!(Fixed32::from_int(3).negate().to_int(), -3);
    }

    #[test]
    fn test_floor_ceil() {
        let f = Fixed32::from_f32(2.7);
        assert_eq!(f.floor().to_int(), 2);
        assert_eq!(f.ceil().to_int(), 3);
    }

    #[test]
    fn test_constants() {
        assert_eq!(Fixed32::ZERO.to_int(), 0);
        assert_eq!(Fixed32::ONE.to_int(), 1);
        assert!((Fixed32::HALF.to_f32() - 0.5).abs() < 0.001);
    }
}
