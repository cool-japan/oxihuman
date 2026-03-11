// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export fluid simulation data (particles, velocity fields).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FluidParticle {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub density: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FluidSimExport {
    pub frame: u32,
    pub particles: Vec<FluidParticle>,
    pub time_step: f32,
}

#[allow(dead_code)]
pub fn new_fluid_sim_export(frame: u32, dt: f32) -> FluidSimExport {
    FluidSimExport { frame, particles: Vec::new(), time_step: dt.max(1e-6) }
}

#[allow(dead_code)]
pub fn fse_add_particle(fse: &mut FluidSimExport, pos: [f32; 3], vel: [f32; 3], density: f32) {
    fse.particles.push(FluidParticle { position: pos, velocity: vel, density });
}

#[allow(dead_code)]
pub fn fse_particle_count(fse: &FluidSimExport) -> usize { fse.particles.len() }

#[allow(dead_code)]
pub fn fse_avg_density(fse: &FluidSimExport) -> f32 {
    if fse.particles.is_empty() { return 0.0; }
    fse.particles.iter().map(|p| p.density).sum::<f32>() / fse.particles.len() as f32
}

#[allow(dead_code)]
pub fn fse_max_speed(fse: &FluidSimExport) -> f32 {
    fse.particles.iter().map(|p| {
        let v = p.velocity;
        (v[0]*v[0]+v[1]*v[1]+v[2]*v[2]).sqrt()
    }).fold(0.0f32, f32::max)
}

#[allow(dead_code)]
pub fn fse_bounding_box(fse: &FluidSimExport) -> ([f32; 3], [f32; 3]) {
    if fse.particles.is_empty() { return ([0.0; 3], [0.0; 3]); }
    let mut mn = [f32::MAX; 3]; let mut mx = [f32::MIN; 3];
    for p in &fse.particles {
        for i in 0..3 { mn[i] = mn[i].min(p.position[i]); mx[i] = mx[i].max(p.position[i]); }
    }
    (mn, mx)
}

#[allow(dead_code)]
pub fn fse_validate(fse: &FluidSimExport) -> bool {
    fse.time_step > 0.0 && fse.particles.iter().all(|p| p.density >= 0.0)
}

#[allow(dead_code)]
pub fn fse_to_json(fse: &FluidSimExport) -> String {
    format!("{{\"frame\":{},\"particles\":{},\"dt\":{:.6}}}", fse.frame, fse.particles.len(), fse.time_step)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> FluidSimExport {
        let mut f = new_fluid_sim_export(5, 0.016);
        fse_add_particle(&mut f, [0.0,0.0,0.0], [1.0,0.0,0.0], 1000.0);
        fse_add_particle(&mut f, [1.0,0.0,0.0], [0.0,1.0,0.0], 1000.0);
        f
    }

    #[test] fn test_new() { let f = new_fluid_sim_export(0, 0.01); assert_eq!(fse_particle_count(&f), 0); }
    #[test] fn test_add() { assert_eq!(fse_particle_count(&sample()), 2); }
    #[test] fn test_avg_density() { assert!((fse_avg_density(&sample()) - 1000.0).abs() < 1e-3); }
    #[test] fn test_max_speed() { assert!((fse_max_speed(&sample()) - 1.0).abs() < 1e-5); }
    #[test] fn test_bbox() { let (mn, mx) = fse_bounding_box(&sample()); assert!((mn[0]).abs() < 1e-6); assert!((mx[0] - 1.0).abs() < 1e-6); }
    #[test] fn test_validate() { assert!(fse_validate(&sample())); }
    #[test] fn test_to_json() { assert!(fse_to_json(&sample()).contains("frame")); }
    #[test] fn test_frame() { assert_eq!(sample().frame, 5); }
    #[test] fn test_empty_avg() { let f = new_fluid_sim_export(0, 0.01); assert!((fse_avg_density(&f)).abs() < 1e-6); }
    #[test] fn test_empty_bbox() { let f = new_fluid_sim_export(0, 0.01); let (mn, mx) = fse_bounding_box(&f); assert!((mn[0]).abs() < 1e-6); assert!((mx[0]).abs() < 1e-6); }
}
