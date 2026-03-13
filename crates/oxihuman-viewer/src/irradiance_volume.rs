// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! IrradianceVolume — probe-based indirect lighting volume.

#![allow(dead_code)]

/// A single irradiance probe storing SH coefficients (9×RGB = 27 values).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IrradianceProbe {
    pub position: [f32; 3],
    /// 9 SH bands × RGB (stub: stored flat, 27 f32).
    pub sh: [f32; 27],
}

/// A 3-D grid of irradiance probes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IrradianceVolume {
    pub probes: Vec<IrradianceProbe>,
    pub min_bound: [f32; 3],
    pub max_bound: [f32; 3],
    pub spacing: f32,
}

/// Create an empty `IrradianceVolume`.
#[allow(dead_code)]
pub fn new_irradiance_volume(
    min_bound: [f32; 3],
    max_bound: [f32; 3],
    spacing: f32,
) -> IrradianceVolume {
    IrradianceVolume {
        probes: Vec::new(),
        min_bound,
        max_bound,
        spacing,
    }
}

/// Return the number of probes.
#[allow(dead_code)]
pub fn probe_count(vol: &IrradianceVolume) -> usize {
    vol.probes.len()
}

/// Return a reference to the probe at `index`.
#[allow(dead_code)]
pub fn probe_at(vol: &IrradianceVolume, index: usize) -> Option<&IrradianceProbe> {
    vol.probes.get(index)
}

/// Sample the irradiance (first SH band, diffuse colour) at world-space position.
/// Uses nearest-probe lookup.
#[allow(dead_code)]
pub fn sample_irradiance(vol: &IrradianceVolume, pos: [f32; 3]) -> [f32; 3] {
    if vol.probes.is_empty() {
        return [0.0; 3];
    }
    let nearest = vol
        .probes
        .iter()
        .min_by(|a, b| {
            let da = dist_sq(a.position, pos);
            let db = dist_sq(b.position, pos);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
;
    let Some(nearest) = nearest else {
        return [0.0; 3];
    };
    [nearest.sh[0], nearest.sh[1], nearest.sh[2]]
}

fn dist_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

/// Alias for `sample_irradiance`.
#[allow(dead_code)]
pub fn irradiance_at_world_pos(vol: &IrradianceVolume, pos: [f32; 3]) -> [f32; 3] {
    sample_irradiance(vol, pos)
}

/// Return the volume bounding box `[min, max]`.
#[allow(dead_code)]
pub fn volume_bounds(vol: &IrradianceVolume) -> ([f32; 3], [f32; 3]) {
    (vol.min_bound, vol.max_bound)
}

/// Return the probe spacing.
#[allow(dead_code)]
pub fn probe_spacing(vol: &IrradianceVolume) -> f32 {
    vol.spacing
}

/// Add a probe at the given position with zero SH.
#[allow(dead_code)]
pub fn add_probe(vol: &mut IrradianceVolume, position: [f32; 3]) {
    vol.probes.push(IrradianceProbe {
        position,
        sh: [0.0; 27],
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vol() -> IrradianceVolume {
        new_irradiance_volume([-5.0; 3], [5.0; 3], 1.0)
    }

    #[test]
    fn test_new_volume_empty() {
        let v = make_vol();
        assert_eq!(probe_count(&v), 0);
    }

    #[test]
    fn test_add_probe_increments_count() {
        let mut v = make_vol();
        add_probe(&mut v, [0.0, 0.0, 0.0]);
        assert_eq!(probe_count(&v), 1);
    }

    #[test]
    fn test_probe_at_some() {
        let mut v = make_vol();
        add_probe(&mut v, [1.0, 0.0, 0.0]);
        assert!(probe_at(&v, 0).is_some());
    }

    #[test]
    fn test_probe_at_none() {
        let v = make_vol();
        assert!(probe_at(&v, 0).is_none());
    }

    #[test]
    fn test_sample_irradiance_empty() {
        let v = make_vol();
        let ir = sample_irradiance(&v, [0.0; 3]);
        assert!(ir.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn test_irradiance_at_world_pos_same_as_sample() {
        let mut v = make_vol();
        add_probe(&mut v, [0.0; 3]);
        let a = sample_irradiance(&v, [0.0; 3]);
        let b = irradiance_at_world_pos(&v, [0.0; 3]);
        assert_eq!(a, b);
    }

    #[test]
    fn test_volume_bounds() {
        let v = make_vol();
        let (min, max) = volume_bounds(&v);
        assert!((min[0] - (-5.0)).abs() < 1e-6);
        assert!((max[0] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_probe_spacing() {
        let v = make_vol();
        assert!((probe_spacing(&v) - 1.0).abs() < 1e-6);
    }
}
