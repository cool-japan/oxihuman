// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct DisplacementPreview {
    pub strength: f32,
    pub midlevel: f32,
    pub show_vectors: bool,
}

pub fn new_displacement_preview(strength: f32) -> DisplacementPreview {
    DisplacementPreview {
        strength,
        midlevel: 0.5,
        show_vectors: false,
    }
}

pub fn displacement_preview_color(height: f32, midlevel: f32) -> [f32; 3] {
    let delta = height - midlevel;
    if delta >= 0.0 {
        [delta.min(1.0), 0.0, 0.0]
    } else {
        [0.0, 0.0, (-delta).min(1.0)]
    }
}

pub fn displacement_preview_vector(
    pos: [f32; 3],
    normal: [f32; 3],
    height: f32,
    params: &DisplacementPreview,
) -> [f32; 3] {
    let offset = (height - params.midlevel) * params.strength;
    [
        pos[0] + normal[0] * offset,
        pos[1] + normal[1] * offset,
        pos[2] + normal[2] * offset,
    ]
}

pub fn displacement_magnitude_color(magnitude: f32, max: f32) -> [f32; 3] {
    let t = if max < 1e-9 {
        0.0
    } else {
        (magnitude / max).clamp(0.0, 1.0)
    };
    [t, 1.0 - t, 0.0]
}

pub fn displacement_is_elevated(height: f32, midlevel: f32) -> bool {
    height > midlevel
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_displacement_preview() {
        /* midlevel defaults to 0.5 */
        let p = new_displacement_preview(1.0);
        assert!((p.midlevel - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_displacement_preview_color_elevated() {
        /* height > midlevel -> red */
        let c = displacement_preview_color(0.8, 0.5);
        assert!(c[0] > 0.0 && c[2] < 1e-6);
    }

    #[test]
    fn test_displacement_preview_color_sunken() {
        /* height < midlevel -> blue */
        let c = displacement_preview_color(0.2, 0.5);
        assert!(c[2] > 0.0 && c[0] < 1e-6);
    }

    #[test]
    fn test_displacement_magnitude_color() {
        /* magnitude=max -> [1,0,0] */
        let c = displacement_magnitude_color(1.0, 1.0);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_displacement_is_elevated() {
        /* height > midlevel is elevated */
        assert!(displacement_is_elevated(0.7, 0.5));
        assert!(!displacement_is_elevated(0.3, 0.5));
    }
}
