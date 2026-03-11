// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Particle trail/ribbon renderer.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TrailConfig {
    pub max_points: usize,
    pub fade_time: f32,
    pub width: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TrailPoint {
    pub position: [f32; 3],
    pub time: f32,
    pub width: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParticleTrail {
    pub points: Vec<TrailPoint>,
    pub config: TrailConfig,
}

#[allow(dead_code)]
pub fn default_trail_config() -> TrailConfig {
    TrailConfig {
        max_points: 64,
        fade_time: 1.0,
        width: 0.1,
    }
}

#[allow(dead_code)]
pub fn new_particle_trail() -> ParticleTrail {
    ParticleTrail {
        points: Vec::new(),
        config: default_trail_config(),
    }
}

#[allow(dead_code)]
pub fn trail_add_point(trail: &mut ParticleTrail, position: [f32; 3], time: f32) {
    let width = trail.config.width;
    trail.points.push(TrailPoint {
        position,
        time,
        width,
    });
    if trail.points.len() > trail.config.max_points {
        trail.points.remove(0);
    }
}

#[allow(dead_code)]
pub fn trail_update(trail: &mut ParticleTrail, dt: f32) {
    for p in &mut trail.points {
        p.time -= dt;
    }
    trail.points.retain(|p| p.time > 0.0);
}

#[allow(dead_code)]
pub fn trail_point_count(trail: &ParticleTrail) -> usize {
    trail.points.len()
}

#[allow(dead_code)]
pub fn trail_clear(trail: &mut ParticleTrail) {
    trail.points.clear();
}

#[allow(dead_code)]
pub fn trail_get_point(trail: &ParticleTrail, index: usize) -> Option<&TrailPoint> {
    trail.points.get(index)
}

#[allow(dead_code)]
pub fn trail_length(trail: &ParticleTrail) -> f32 {
    if trail.points.len() < 2 {
        return 0.0;
    }
    let mut total = 0.0_f32;
    for i in 1..trail.points.len() {
        let a = &trail.points[i - 1].position;
        let b = &trail.points[i].position;
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let dz = b[2] - a[2];
        total += (dx * dx + dy * dy + dz * dz).sqrt();
    }
    total
}

#[allow(dead_code)]
pub fn trail_to_json(trail: &ParticleTrail) -> String {
    format!(r#"{{"point_count":{}}}"#, trail.points.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_trail_config();
        assert_eq!(cfg.max_points, 64);
        assert!((cfg.fade_time - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_trail_empty() {
        let t = new_particle_trail();
        assert_eq!(trail_point_count(&t), 0);
    }

    #[test]
    fn test_add_point() {
        let mut t = new_particle_trail();
        trail_add_point(&mut t, [0.0, 0.0, 0.0], 1.0);
        assert_eq!(trail_point_count(&t), 1);
    }

    #[test]
    fn test_max_points_limit() {
        let mut t = new_particle_trail();
        t.config.max_points = 3;
        for i in 0..5 {
            trail_add_point(&mut t, [i as f32, 0.0, 0.0], 1.0);
        }
        assert_eq!(trail_point_count(&t), 3);
    }

    #[test]
    fn test_update_removes_expired() {
        let mut t = new_particle_trail();
        trail_add_point(&mut t, [0.0, 0.0, 0.0], 0.5);
        trail_add_point(&mut t, [1.0, 0.0, 0.0], 2.0);
        trail_update(&mut t, 1.0);
        assert_eq!(trail_point_count(&t), 1);
    }

    #[test]
    fn test_clear() {
        let mut t = new_particle_trail();
        trail_add_point(&mut t, [0.0, 0.0, 0.0], 1.0);
        trail_clear(&mut t);
        assert_eq!(trail_point_count(&t), 0);
    }

    #[test]
    fn test_get_point() {
        let mut t = new_particle_trail();
        trail_add_point(&mut t, [1.0, 2.0, 3.0], 1.0);
        let p = trail_get_point(&t, 0);
        assert!(p.is_some());
        assert!((p.unwrap().position[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_length_two_points() {
        let mut t = new_particle_trail();
        trail_add_point(&mut t, [0.0, 0.0, 0.0], 1.0);
        trail_add_point(&mut t, [3.0, 4.0, 0.0], 1.0);
        let len = trail_length(&t);
        assert!((len - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        let t = new_particle_trail();
        let j = trail_to_json(&t);
        assert!(j.contains("point_count"));
    }
}
