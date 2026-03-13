// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A snapshot of a cloth simulation particle.
#[allow(dead_code)]
#[derive(Clone)]
pub struct ClothParticleState {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub pinned: bool,
}

/// A cloth simulation state export.
#[allow(dead_code)]
pub struct ClothSimStateExport {
    pub frame: usize,
    pub time: f32,
    pub particles: Vec<ClothParticleState>,
}

/// Create a new cloth sim state.
#[allow(dead_code)]
pub fn new_cloth_sim_state(frame: usize, time: f32) -> ClothSimStateExport {
    ClothSimStateExport {
        frame,
        time,
        particles: Vec::new(),
    }
}

/// Add a particle.
#[allow(dead_code)]
pub fn add_cloth_particle_state(
    export: &mut ClothSimStateExport,
    pos: [f32; 3],
    vel: [f32; 3],
    pinned: bool,
) {
    export.particles.push(ClothParticleState {
        position: pos,
        velocity: vel,
        pinned,
    });
}

/// Count particles.
#[allow(dead_code)]
pub fn particle_count_css(export: &ClothSimStateExport) -> usize {
    export.particles.len()
}

/// Count pinned particles.
#[allow(dead_code)]
pub fn pinned_count_css(export: &ClothSimStateExport) -> usize {
    export.particles.iter().filter(|p| p.pinned).count()
}

/// Average speed of particles.
#[allow(dead_code)]
pub fn avg_speed_css(export: &ClothSimStateExport) -> f32 {
    if export.particles.is_empty() {
        return 0.0;
    }
    let sum: f32 = export
        .particles
        .iter()
        .map(|p| {
            let v = p.velocity;
            (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
        })
        .sum();
    sum / export.particles.len() as f32
}

/// Max speed.
#[allow(dead_code)]
pub fn max_speed_css(export: &ClothSimStateExport) -> f32 {
    export
        .particles
        .iter()
        .map(|p| {
            let v = p.velocity;
            (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
        })
        .fold(0.0_f32, f32::max)
}

/// Flatten positions to flat array.
#[allow(dead_code)]
pub fn flat_positions_css(export: &ClothSimStateExport) -> Vec<f32> {
    export.particles.iter().flat_map(|p| p.position).collect()
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn cloth_sim_state_to_json(export: &ClothSimStateExport) -> String {
    format!(
        r#"{{"frame":{},"time":{:.4},"particles":{},"pinned":{}}}"#,
        export.frame,
        export.time,
        export.particles.len(),
        pinned_count_css(export)
    )
}

/// Validate particle count matches expected.
#[allow(dead_code)]
pub fn validate_cloth_sim_state(export: &ClothSimStateExport, expected: usize) -> bool {
    export.particles.len() == expected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_count() {
        let mut e = new_cloth_sim_state(0, 0.0);
        add_cloth_particle_state(&mut e, [0.0; 3], [0.0; 3], false);
        assert_eq!(particle_count_css(&e), 1);
    }

    #[test]
    fn pinned_count() {
        let mut e = new_cloth_sim_state(0, 0.0);
        add_cloth_particle_state(&mut e, [0.0; 3], [0.0; 3], true);
        add_cloth_particle_state(&mut e, [0.0; 3], [0.0; 3], false);
        assert_eq!(pinned_count_css(&e), 1);
    }

    #[test]
    fn avg_speed_zero() {
        let mut e = new_cloth_sim_state(0, 0.0);
        add_cloth_particle_state(&mut e, [0.0; 3], [0.0; 3], false);
        assert!((avg_speed_css(&e) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn avg_speed_known() {
        let mut e = new_cloth_sim_state(0, 0.0);
        add_cloth_particle_state(&mut e, [0.0; 3], [3.0, 4.0, 0.0], false);
        assert!((avg_speed_css(&e) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn max_speed() {
        let mut e = new_cloth_sim_state(0, 0.0);
        add_cloth_particle_state(&mut e, [0.0; 3], [1.0, 0.0, 0.0], false);
        add_cloth_particle_state(&mut e, [0.0; 3], [3.0, 4.0, 0.0], false);
        assert!((max_speed_css(&e) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn flat_positions_size() {
        let mut e = new_cloth_sim_state(0, 0.0);
        add_cloth_particle_state(&mut e, [1.0, 2.0, 3.0], [0.0; 3], false);
        assert_eq!(flat_positions_css(&e).len(), 3);
    }

    #[test]
    fn json_has_frame() {
        let e = new_cloth_sim_state(5, 0.1);
        let j = cloth_sim_state_to_json(&e);
        assert!(j.contains("\"frame\":5"));
    }

    #[test]
    fn validate_count() {
        let mut e = new_cloth_sim_state(0, 0.0);
        add_cloth_particle_state(&mut e, [0.0; 3], [0.0; 3], false);
        assert!(validate_cloth_sim_state(&e, 1));
    }

    #[test]
    fn empty_avg_speed() {
        let e = new_cloth_sim_state(0, 0.0);
        assert!((avg_speed_css(&e) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn time_stored() {
        let e = new_cloth_sim_state(0, 1.5);
        assert!((e.time - 1.5).abs() < 1e-4);
    }
}
