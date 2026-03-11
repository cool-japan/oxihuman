// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export geometry warp / space-warp deformation data.

/// A warp keyframe: a displacement field at a given time.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WarpKeyframe {
    pub time: f32,
    pub displacements: Vec<[f32; 3]>,
}

/// Geometry warp export.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct GeoWarpExport {
    pub name: String,
    pub vertex_count: usize,
    pub keyframes: Vec<WarpKeyframe>,
}

/// Create a new geo warp export.
#[allow(dead_code)]
pub fn new_geo_warp(name: &str, vertex_count: usize) -> GeoWarpExport {
    GeoWarpExport {
        name: name.to_string(),
        vertex_count,
        keyframes: vec![],
    }
}

/// Add a keyframe.
#[allow(dead_code)]
pub fn add_warp_keyframe(export: &mut GeoWarpExport, time: f32, displacements: Vec<[f32; 3]>) {
    export.keyframes.push(WarpKeyframe {
        time,
        displacements,
    });
}

/// Duration from first to last keyframe.
#[allow(dead_code)]
pub fn warp_duration(export: &GeoWarpExport) -> f32 {
    if export.keyframes.len() < 2 {
        return 0.0;
    }
    let first = export
        .keyframes
        .iter()
        .map(|k| k.time)
        .fold(f32::MAX, f32::min);
    let last = export
        .keyframes
        .iter()
        .map(|k| k.time)
        .fold(f32::MIN, f32::max);
    last - first
}

/// Interpolate displacement at a given time (linear between keyframes).
#[allow(dead_code)]
pub fn interpolate_warp(export: &GeoWarpExport, time: f32) -> Option<Vec<[f32; 3]>> {
    let kfs = &export.keyframes;
    if kfs.is_empty() {
        return None;
    }
    // clamp to range
    if time <= kfs[0].time {
        return Some(kfs[0].displacements.clone());
    }
    if time >= kfs[kfs.len() - 1].time {
        return Some(kfs[kfs.len() - 1].displacements.clone());
    }
    for i in 0..kfs.len() - 1 {
        let k0 = &kfs[i];
        let k1 = &kfs[i + 1];
        if time >= k0.time && time <= k1.time {
            let t = (time - k0.time) / (k1.time - k0.time);
            let out: Vec<[f32; 3]> = k0
                .displacements
                .iter()
                .zip(k1.displacements.iter())
                .map(|(a, b)| {
                    [
                        a[0] * (1.0 - t) + b[0] * t,
                        a[1] * (1.0 - t) + b[1] * t,
                        a[2] * (1.0 - t) + b[2] * t,
                    ]
                })
                .collect();
            return Some(out);
        }
    }
    None
}

/// Maximum displacement magnitude at a given keyframe index.
#[allow(dead_code)]
pub fn max_displacement(export: &GeoWarpExport, kf_idx: usize) -> f32 {
    export
        .keyframes
        .get(kf_idx)
        .map(|k| {
            k.displacements
                .iter()
                .map(|d| (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt())
                .fold(0.0_f32, f32::max)
        })
        .unwrap_or(0.0)
}

/// Serialise a single keyframe to a flat f32 buffer.
#[allow(dead_code)]
pub fn serialise_keyframe(kf: &WarpKeyframe) -> Vec<f32> {
    let mut buf = vec![kf.time];
    for d in &kf.displacements {
        buf.extend_from_slice(d);
    }
    buf
}

/// Zero-displacement field for `n` vertices.
#[allow(dead_code)]
pub fn zero_displacement(n: usize) -> Vec<[f32; 3]> {
    vec![[0.0; 3]; n]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_kf_export() -> GeoWarpExport {
        let mut e = new_geo_warp("warp", 2);
        add_warp_keyframe(&mut e, 0.0, vec![[0.0; 3]; 2]);
        add_warp_keyframe(&mut e, 1.0, vec![[1.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        e
    }

    #[test]
    fn test_new_geo_warp() {
        let e = new_geo_warp("x", 4);
        assert_eq!(e.vertex_count, 4);
    }

    #[test]
    fn test_add_keyframe() {
        let e = two_kf_export();
        assert_eq!(e.keyframes.len(), 2);
    }

    #[test]
    fn test_warp_duration() {
        let e = two_kf_export();
        assert!((warp_duration(&e) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_warp_duration_empty() {
        let e = new_geo_warp("e", 0);
        assert_eq!(warp_duration(&e), 0.0);
    }

    #[test]
    fn test_interpolate_at_start() {
        let e = two_kf_export();
        let d = interpolate_warp(&e, 0.0).unwrap();
        assert!((d[0][0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_interpolate_midpoint() {
        let e = two_kf_export();
        let d = interpolate_warp(&e, 0.5).unwrap();
        assert!((d[0][0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_interpolate_empty_returns_none() {
        let e = new_geo_warp("e", 2);
        assert!(interpolate_warp(&e, 0.5).is_none());
    }

    #[test]
    fn test_max_displacement() {
        let e = two_kf_export();
        assert!((max_displacement(&e, 1) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_serialise_keyframe_length() {
        let kf = WarpKeyframe {
            time: 0.0,
            displacements: vec![[0.0; 3]; 3],
        };
        let buf = serialise_keyframe(&kf);
        assert_eq!(buf.len(), 1 + 9);
    }

    #[test]
    fn test_zero_displacement() {
        let d = zero_displacement(5);
        assert_eq!(d.len(), 5);
        assert_eq!(d[0], [0.0; 3]);
    }
}
