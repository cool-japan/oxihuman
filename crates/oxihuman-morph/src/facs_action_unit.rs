// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Facial Action Coding System (FACS) action unit support.

/// Identifier for a FACS action unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AuId(pub u8);

/// A single FACS action unit with its current activation weight.
#[derive(Debug, Clone)]
pub struct ActionUnit {
    pub id: AuId,
    pub label: String,
    pub weight: f32,
}

/// Collection of active action units for a frame.
#[derive(Debug, Clone, Default)]
pub struct FacsFrame {
    pub units: Vec<ActionUnit>,
}

impl ActionUnit {
    /// Create a new action unit with zero weight.
    pub fn new(id: u8, label: impl Into<String>) -> Self {
        Self { id: AuId(id), label: label.into(), weight: 0.0 }
    }

    /// Set the activation weight, clamped to [0, 1].
    pub fn set_weight(&mut self, w: f32) {
        self.weight = w.clamp(0.0, 1.0);
    }
}

impl FacsFrame {
    /// Add or update an action unit by ID.
    pub fn set(&mut self, id: u8, weight: f32) {
        let w = weight.clamp(0.0, 1.0);
        if let Some(au) = self.units.iter_mut().find(|a| a.id.0 == id) {
            au.weight = w;
        } else {
            let mut au = ActionUnit::new(id, format!("AU{id:02}"));
            au.weight = w;
            self.units.push(au);
        }
    }

    /// Retrieve the weight for an action unit, returning 0.0 if absent.
    pub fn get(&self, id: u8) -> f32 {
        self.units.iter().find(|a| a.id.0 == id).map(|a| a.weight).unwrap_or(0.0)
    }

    /// Clear all action units.
    pub fn clear(&mut self) {
        self.units.clear();
    }
}

/// Blend two FACS frames together by weight.
pub fn blend_facs_frames(a: &FacsFrame, b: &FacsFrame, t: f32) -> FacsFrame {
    let t = t.clamp(0.0, 1.0);
    let mut ids: Vec<u8> = a.units.iter().map(|u| u.id.0).collect();
    for u in &b.units {
        if !ids.contains(&u.id.0) {
            ids.push(u.id.0);
        }
    }
    let mut out = FacsFrame::default();
    for id in ids {
        let wa = a.get(id);
        let wb = b.get(id);
        out.set(id, wa + (wb - wa) * t);
    }
    out
}

/// Return the IDs of all action units with weight above the threshold.
pub fn active_units(frame: &FacsFrame, threshold: f32) -> Vec<u8> {
    frame.units.iter().filter(|a| a.weight > threshold).map(|a| a.id.0).collect()
}

/// Scale all weights in a frame by a factor.
pub fn scale_frame(frame: &mut FacsFrame, factor: f32) {
    for au in &mut frame.units {
        au.weight = (au.weight * factor).clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get() {
        /* setting and getting a unit should round-trip */
        let mut f = FacsFrame::default();
        f.set(1, 0.7);
        assert!((f.get(1) - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_missing_returns_zero() {
        /* absent unit should return 0.0 */
        let f = FacsFrame::default();
        assert_eq!(f.get(99), 0.0);
    }

    #[test]
    fn test_clamp_weight() {
        /* weight above 1 should be clamped */
        let mut f = FacsFrame::default();
        f.set(4, 2.5);
        assert!((f.get(4) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_clear() {
        /* clear should remove all units */
        let mut f = FacsFrame::default();
        f.set(1, 0.5);
        f.clear();
        assert!(f.units.is_empty());
    }

    #[test]
    fn test_blend_frames() {
        /* blend at 0.5 should average weights */
        let mut a = FacsFrame::default();
        a.set(1, 0.0);
        let mut b = FacsFrame::default();
        b.set(1, 1.0);
        let blended = blend_facs_frames(&a, &b, 0.5);
        assert!((blended.get(1) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_active_units() {
        /* only units above threshold should appear */
        let mut f = FacsFrame::default();
        f.set(1, 0.8);
        f.set(2, 0.1);
        let active = active_units(&f, 0.5);
        assert!(active.contains(&1));
        assert!(!active.contains(&2));
    }

    #[test]
    fn test_scale_frame() {
        /* scaling should multiply weights */
        let mut f = FacsFrame::default();
        f.set(6, 0.5);
        scale_frame(&mut f, 2.0);
        assert!((f.get(6) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_action_unit_new() {
        /* new unit should start at zero */
        let au = ActionUnit::new(12, "AU12");
        assert_eq!(au.weight, 0.0);
        assert_eq!(au.id.0, 12);
    }

    #[test]
    fn test_set_updates_existing() {
        /* second set on same id should update not duplicate */
        let mut f = FacsFrame::default();
        f.set(7, 0.3);
        f.set(7, 0.9);
        assert_eq!(f.units.len(), 1);
        assert!((f.get(7) - 0.9).abs() < 1e-6);
    }
}
