// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Skin sliding deformation model for anatomical simulation.

/// A skin sliding layer attached to underlying tissue.
#[derive(Debug, Clone)]
pub struct SkinLayer {
    /// Position of the skin vertex (world space).
    pub pos: [f32; 3],
    /// Underlying tissue (bone/muscle) reference position.
    pub ref_pos: [f32; 3],
    /// Sliding stiffness (how tightly skin adheres).
    pub stiffness: f32,
    /// Current slide offset.
    pub offset: [f32; 3],
}

impl SkinLayer {
    pub fn new(pos: [f32; 3], stiffness: f32) -> Self {
        SkinLayer {
            pos,
            ref_pos: pos,
            stiffness,
            offset: [0.0; 3],
        }
    }
}

/// Create a new skin layer node.
pub fn new_skin_layer(pos: [f32; 3], stiffness: f32) -> SkinLayer {
    SkinLayer::new(pos, stiffness)
}

/// Apply a sliding force given tissue displacement `tissue_disp`.
#[allow(clippy::needless_range_loop)]
pub fn skin_apply_slide(s: &mut SkinLayer, tissue_disp: [f32; 3], dt: f32, damping: f32) {
    /* The skin lags behind tissue movement — spring+damper */
    for k in 0..3 {
        let target = s.ref_pos[k] + tissue_disp[k];
        let spring_f = s.stiffness * (target - s.pos[k]);
        let damp_f = -damping * (s.pos[k] - (s.ref_pos[k] + s.offset[k]));
        s.pos[k] += (spring_f + damp_f) * dt;
        s.offset[k] = s.pos[k] - s.ref_pos[k];
    }
}

/// Return the magnitude of the slide offset.
pub fn skin_offset_magnitude(s: &SkinLayer) -> f32 {
    let o = s.offset;
    (o[0] * o[0] + o[1] * o[1] + o[2] * o[2]).sqrt()
}

/// Reset skin position to reference.
pub fn skin_reset(s: &mut SkinLayer) {
    s.pos = s.ref_pos;
    s.offset = [0.0; 3];
}

/// Return the sliding stiffness.
pub fn skin_stiffness(s: &SkinLayer) -> f32 {
    s.stiffness
}

/// Update the reference position (when underlying tissue moves).
pub fn skin_update_ref(s: &mut SkinLayer, new_ref: [f32; 3]) {
    s.ref_pos = new_ref;
}

/// Compute the adhesion energy (½ k x²).
pub fn skin_adhesion_energy(s: &SkinLayer) -> f32 {
    let mag2 = s.offset[0] * s.offset[0] + s.offset[1] * s.offset[1] + s.offset[2] * s.offset[2];
    0.5 * s.stiffness * mag2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_skin_zero_offset() {
        let s = new_skin_layer([0.0, 0.0, 0.0], 100.0);
        assert!(skin_offset_magnitude(&s) < 1e-6);
    }

    #[test]
    fn test_apply_slide_moves_skin() {
        let mut s = new_skin_layer([0.0, 0.0, 0.0], 100.0);
        skin_apply_slide(&mut s, [0.1, 0.0, 0.0], 0.01, 1.0);
        assert!(s.pos[0] > 0.0);
    }

    #[test]
    fn test_skin_reset_clears_offset() {
        let mut s = new_skin_layer([0.0, 0.0, 0.0], 100.0);
        skin_apply_slide(&mut s, [0.5, 0.0, 0.0], 0.1, 0.1);
        skin_reset(&mut s);
        assert!(skin_offset_magnitude(&s) < 1e-6);
    }

    #[test]
    fn test_stiffness_getter() {
        let s = new_skin_layer([0.0, 0.0, 0.0], 250.0);
        assert!((skin_stiffness(&s) - 250.0).abs() < 1e-5);
    }

    #[test]
    fn test_update_ref_changes_reference() {
        let mut s = new_skin_layer([0.0, 0.0, 0.0], 100.0);
        skin_update_ref(&mut s, [1.0, 0.0, 0.0]);
        assert!((s.ref_pos[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_adhesion_energy_zero_at_rest() {
        let s = new_skin_layer([0.0, 0.0, 0.0], 100.0);
        assert!(skin_adhesion_energy(&s) < 1e-6);
    }

    #[test]
    fn test_skin_converges_to_tissue() {
        /* stiffness=50 keeps forward Euler stable: 50*0.01=0.5 < 2 */
        let mut s = new_skin_layer([0.0, 0.0, 0.0], 50.0);
        for _ in 0..200 {
            skin_apply_slide(&mut s, [1.0, 0.0, 0.0], 0.01, 5.0);
        }
        /* skin should be close to tissue + slide position */
        assert!(s.pos[0] > 0.5);
    }

    #[test]
    fn test_offset_magnitude_positive_after_slide() {
        let mut s = new_skin_layer([0.0, 0.0, 0.0], 100.0);
        skin_apply_slide(&mut s, [0.5, 0.0, 0.0], 0.1, 0.1);
        assert!(skin_offset_magnitude(&s) > 0.0);
    }

    #[test]
    fn test_adhesion_energy_positive_when_offset() {
        let mut s = new_skin_layer([0.0, 0.0, 0.0], 100.0);
        skin_apply_slide(&mut s, [0.5, 0.0, 0.0], 0.1, 0.1);
        assert!(skin_adhesion_energy(&s) > 0.0);
    }
}
