// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export per-face color data (RGBA).

/// RGBA color stored as f32 per channel.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct FaceColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl FaceColor {
    #[allow(dead_code)]
    pub fn white() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }
    #[allow(dead_code)]
    pub fn black() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
}

/// Per-face color export.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct FaceColorExport {
    pub colors: Vec<FaceColor>,
}

/// Create a face color export with a given number of faces.
#[allow(dead_code)]
pub fn new_face_color_export(face_count: usize, default_color: FaceColor) -> FaceColorExport {
    FaceColorExport {
        colors: vec![default_color; face_count],
    }
}

/// Set a face color.
#[allow(dead_code)]
pub fn set_face_color(export: &mut FaceColorExport, face_idx: usize, color: FaceColor) {
    if face_idx < export.colors.len() {
        export.colors[face_idx] = color;
    }
}

/// Get a face color.
#[allow(dead_code)]
pub fn get_face_color(export: &FaceColorExport, face_idx: usize) -> Option<FaceColor> {
    export.colors.get(face_idx).copied()
}

/// Convert face color to RGBA u8.
#[allow(dead_code)]
pub fn color_to_u8(c: FaceColor) -> [u8; 4] {
    [
        (c.r.clamp(0.0, 1.0) * 255.0) as u8,
        (c.g.clamp(0.0, 1.0) * 255.0) as u8,
        (c.b.clamp(0.0, 1.0) * 255.0) as u8,
        (c.a.clamp(0.0, 1.0) * 255.0) as u8,
    ]
}

/// Serialise all face colors to a flat f32 buffer (RGBA interleaved).
#[allow(dead_code)]
pub fn serialise_face_colors(export: &FaceColorExport) -> Vec<f32> {
    export
        .colors
        .iter()
        .flat_map(|c| [c.r, c.g, c.b, c.a])
        .collect()
}

/// Compute the average color across all faces.
#[allow(dead_code)]
pub fn average_color(export: &FaceColorExport) -> Option<FaceColor> {
    if export.colors.is_empty() {
        return None;
    }
    let n = export.colors.len() as f32;
    let sum = export.colors.iter().fold([0.0_f32; 4], |acc, c| {
        [acc[0] + c.r, acc[1] + c.g, acc[2] + c.b, acc[3] + c.a]
    });
    Some(FaceColor {
        r: sum[0] / n,
        g: sum[1] / n,
        b: sum[2] / n,
        a: sum[3] / n,
    })
}

/// Count faces that are non-transparent (alpha > threshold).
#[allow(dead_code)]
pub fn opaque_face_count(export: &FaceColorExport, threshold: f32) -> usize {
    export.colors.iter().filter(|c| c.a > threshold).count()
}

/// Fill all faces with a single color.
#[allow(dead_code)]
pub fn fill_all(export: &mut FaceColorExport, color: FaceColor) {
    for c in &mut export.colors {
        *c = color;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_face_color_export() {
        let e = new_face_color_export(3, FaceColor::white());
        assert_eq!(e.colors.len(), 3);
    }

    #[test]
    fn test_set_get_face_color() {
        let mut e = new_face_color_export(2, FaceColor::white());
        set_face_color(&mut e, 0, FaceColor::black());
        let c = get_face_color(&e, 0).unwrap();
        assert!((c.r - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_get_out_of_bounds() {
        let e = new_face_color_export(1, FaceColor::white());
        assert!(get_face_color(&e, 99).is_none());
    }

    #[test]
    fn test_color_to_u8_white() {
        let bytes = color_to_u8(FaceColor::white());
        assert_eq!(bytes, [255, 255, 255, 255]);
    }

    #[test]
    fn test_color_to_u8_black() {
        let bytes = color_to_u8(FaceColor::black());
        assert_eq!(bytes[0], 0);
        assert_eq!(bytes[3], 255);
    }

    #[test]
    fn test_serialise_length() {
        let e = new_face_color_export(3, FaceColor::white());
        assert_eq!(serialise_face_colors(&e).len(), 12);
    }

    #[test]
    fn test_average_color_white() {
        let e = new_face_color_export(4, FaceColor::white());
        let avg = average_color(&e).unwrap();
        assert!((avg.r - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_average_color_empty() {
        let e = FaceColorExport { colors: vec![] };
        assert!(average_color(&e).is_none());
    }

    #[test]
    fn test_opaque_face_count() {
        let mut e = new_face_color_export(3, FaceColor::white());
        set_face_color(
            &mut e,
            1,
            FaceColor {
                a: 0.0,
                ..FaceColor::black()
            },
        );
        assert_eq!(opaque_face_count(&e, 0.5), 2);
    }

    #[test]
    fn test_fill_all() {
        let mut e = new_face_color_export(2, FaceColor::white());
        fill_all(&mut e, FaceColor::black());
        assert!((e.colors[0].r - 0.0).abs() < 1e-6);
    }
}
