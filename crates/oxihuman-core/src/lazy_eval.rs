// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct LazyValue<T: Clone> {
    pub value: Option<T>,
}

impl<T: Clone> LazyValue<T> {
    pub fn new() -> Self {
        LazyValue { value: None }
    }
}

impl<T: Clone> Default for LazyValue<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_lazy_f32() -> LazyValue<f32> {
    LazyValue::new()
}

pub fn lazy_get_or_compute(lv: &mut LazyValue<f32>, compute: impl Fn() -> f32) -> f32 {
    if lv.value.is_none() {
        lv.value = Some(compute());
    }
    lv.value.unwrap_or_default()
}

pub fn lazy_invalidate(lv: &mut LazyValue<f32>) {
    lv.value = None;
}

pub fn lazy_is_computed(lv: &LazyValue<f32>) -> bool {
    lv.value.is_some()
}

pub fn lazy_set(lv: &mut LazyValue<f32>, val: f32) {
    lv.value = Some(val);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_not_computed() {
        /* new lazy value is not yet computed */
        let lv = new_lazy_f32();
        assert!(!lazy_is_computed(&lv));
    }

    #[test]
    fn test_get_or_compute_runs_fn() {
        /* get_or_compute calls fn on first access */
        let mut lv = new_lazy_f32();
        let val = lazy_get_or_compute(&mut lv, || 42.0);
        assert!((val - 42.0).abs() < 1e-6);
        assert!(lazy_is_computed(&lv));
    }

    #[test]
    fn test_get_or_compute_caches() {
        /* fn is called only once */
        let mut lv = new_lazy_f32();
        lazy_get_or_compute(&mut lv, || 10.0);
        let val2 = lazy_get_or_compute(&mut lv, || 99.0);
        assert!((val2 - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_invalidate() {
        /* invalidate clears the cached value */
        let mut lv = new_lazy_f32();
        lazy_set(&mut lv, 5.0);
        lazy_invalidate(&mut lv);
        assert!(!lazy_is_computed(&lv));
    }

    #[test]
    fn test_set() {
        /* lazy_set stores a value directly */
        let mut lv = new_lazy_f32();
        lazy_set(&mut lv, 3.0);
        assert!(lazy_is_computed(&lv));
        assert!((lv.value.expect("should succeed") - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_default() {
        /* Default impl works */
        let lv: LazyValue<f32> = LazyValue::default();
        assert!(lv.value.is_none());
    }
}
