// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;

/// Direction of bulge per influenced vertex
pub enum BulgeDirection {
    /// Use vertex normal as bulge direction
    VertexNormal,
    /// Fixed world-space direction
    Fixed([f32; 3]),
    /// Radially outward from a center axis point
    RadialFrom([f32; 3]),
}

/// A muscle definition: a named region that bulges when a joint flexes
pub struct Muscle {
    pub name: String,
    /// Which joint drives this muscle
    pub joint_name: String,
    /// Flex angle that produces maximum bulge (radians, e.g., PI/2)
    pub max_flex_angle: f32,
    /// Peak bulge amplitude (world units)
    pub bulge_amplitude: f32,
    /// Vertex influences: (vertex_index, weight 0..1)
    pub influences: Vec<(u32, f32)>,
    /// Direction of bulge per influenced vertex (outward normal override)
    pub bulge_direction: BulgeDirection,
}

impl Muscle {
    pub fn new(name: impl Into<String>, joint_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            joint_name: joint_name.into(),
            max_flex_angle: FRAC_PI_2,
            bulge_amplitude: 0.02,
            influences: Vec::new(),
            bulge_direction: BulgeDirection::VertexNormal,
        }
    }

    pub fn with_influences(mut self, influences: Vec<(u32, f32)>) -> Self {
        self.influences = influences;
        self
    }

    pub fn with_amplitude(mut self, amp: f32) -> Self {
        self.bulge_amplitude = amp;
        self
    }

    pub fn with_max_flex(mut self, angle: f32) -> Self {
        self.max_flex_angle = angle;
        self
    }

    /// Compute bulge weight `[0,1]` from joint angle using smoothstep
    pub fn bulge_weight(&self, joint_angle: f32) -> f32 {
        let t = if self.max_flex_angle == 0.0 {
            0.0
        } else {
            (joint_angle / self.max_flex_angle).clamp(0.0, 1.0)
        };
        // smoothstep: t * t * (3 - 2*t)
        t * t * (3.0 - 2.0 * t)
    }

    /// Compute per-vertex displacements for a given joint angle and mesh normals
    pub fn compute_displacements(
        &self,
        joint_angle: f32,
        positions: &[[f32; 3]],
        normals: &[[f32; 3]],
    ) -> Vec<(u32, [f32; 3])> {
        let bw = self.bulge_weight(joint_angle);
        let mut result = Vec::with_capacity(self.influences.len());

        for &(vid, weight) in &self.influences {
            let w = bw * weight;
            let idx = vid as usize;

            let dir = match &self.bulge_direction {
                BulgeDirection::VertexNormal => {
                    if idx < normals.len() {
                        normalize(normals[idx])
                    } else {
                        [0.0, 1.0, 0.0]
                    }
                }
                BulgeDirection::Fixed(d) => normalize(*d),
                BulgeDirection::RadialFrom(center) => {
                    if idx < positions.len() {
                        let p = positions[idx];
                        let v = [p[0] - center[0], p[1] - center[1], p[2] - center[2]];
                        normalize(v)
                    } else {
                        [0.0, 1.0, 0.0]
                    }
                }
            };

            let disp = [
                dir[0] * w * self.bulge_amplitude,
                dir[1] * w * self.bulge_amplitude,
                dir[2] * w * self.bulge_amplitude,
            ];
            result.push((vid, disp));
        }

        result
    }
}

/// Normalize a 3D vector; returns [0,1,0] for zero-length vectors
fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

/// Simulate an isolated muscle (unit test helper)
pub struct MuscleSimulator {
    pub muscles: Vec<Muscle>,
}

impl MuscleSimulator {
    pub fn new() -> Self {
        Self {
            muscles: Vec::new(),
        }
    }

    pub fn add_muscle(&mut self, muscle: Muscle) {
        self.muscles.push(muscle);
    }

    pub fn muscle_count(&self) -> usize {
        self.muscles.len()
    }

    pub fn muscles_for_joint(&self, joint: &str) -> Vec<&Muscle> {
        self.muscles
            .iter()
            .filter(|m| m.joint_name == joint)
            .collect()
    }

    /// Apply all muscle bulges for given joint angles, return new vertex positions
    pub fn apply(
        &self,
        positions: &[[f32; 3]],
        normals: &[[f32; 3]],
        joint_angles: &HashMap<String, f32>,
    ) -> Vec<[f32; 3]> {
        let mut result = positions.to_vec();

        for muscle in &self.muscles {
            let angle = match joint_angles.get(&muscle.joint_name) {
                Some(&a) => a,
                None => continue,
            };

            let displacements = muscle.compute_displacements(angle, positions, normals);
            for (vid, disp) in displacements {
                let idx = vid as usize;
                if idx < result.len() {
                    result[idx][0] += disp[0];
                    result[idx][1] += disp[1];
                    result[idx][2] += disp[2];
                }
            }
        }

        result
    }
}

impl Default for MuscleSimulator {
    fn default() -> Self {
        Self::new()
    }
}

/// Preset muscle definition for a bicep
pub fn bicep_muscle(joint_name: impl Into<String>) -> Muscle {
    Muscle {
        name: "bicep".to_string(),
        joint_name: joint_name.into(),
        max_flex_angle: FRAC_PI_2,
        bulge_amplitude: 0.02,
        influences: Vec::new(),
        bulge_direction: BulgeDirection::VertexNormal,
    }
}

/// Preset muscle definition for a quadricep
pub fn quadricep_muscle(joint_name: impl Into<String>) -> Muscle {
    Muscle {
        name: "quadricep".to_string(),
        joint_name: joint_name.into(),
        max_flex_angle: FRAC_PI_2,
        bulge_amplitude: 0.025,
        influences: Vec::new(),
        bulge_direction: BulgeDirection::VertexNormal,
    }
}

/// Preset muscle definition for a calf muscle
pub fn calf_muscle(joint_name: impl Into<String>) -> Muscle {
    Muscle {
        name: "calf".to_string(),
        joint_name: joint_name.into(),
        max_flex_angle: FRAC_PI_2,
        bulge_amplitude: 0.018,
        influences: Vec::new(),
        bulge_direction: BulgeDirection::VertexNormal,
    }
}

/// Build a muscle from vertex group: all vertices within distance of center
pub fn muscle_from_region(
    name: impl Into<String>,
    joint_name: impl Into<String>,
    positions: &[[f32; 3]],
    center: [f32; 3],
    radius: f32,
    amplitude: f32,
) -> Muscle {
    let mut influences = Vec::new();

    for (i, pos) in positions.iter().enumerate() {
        let dx = pos[0] - center[0];
        let dy = pos[1] - center[1];
        let dz = pos[2] - center[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();

        if dist <= radius {
            // Weight by (1 - dist/radius) — linear falloff (Gaussian-like)
            let weight = if radius > 0.0 {
                (1.0 - dist / radius).clamp(0.0, 1.0)
            } else {
                1.0
            };
            influences.push((i as u32, weight));
        }
    }

    Muscle {
        name: name.into(),
        joint_name: joint_name.into(),
        max_flex_angle: FRAC_PI_2,
        bulge_amplitude: amplitude,
        influences,
        bulge_direction: BulgeDirection::RadialFrom(center),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::{FRAC_PI_2, PI};

    #[test]
    fn test_muscle_new() {
        let m = Muscle::new("bicep", "elbow");
        assert_eq!(m.name, "bicep");
        assert_eq!(m.joint_name, "elbow");
        assert!((m.max_flex_angle - FRAC_PI_2).abs() < 1e-6);
        assert!((m.bulge_amplitude - 0.02).abs() < 1e-6);
        assert!(m.influences.is_empty());
    }

    #[test]
    fn test_bulge_weight_zero_angle() {
        let m = Muscle::new("m", "j");
        let w = m.bulge_weight(0.0);
        assert!(
            w.abs() < 1e-6,
            "bulge weight at 0 angle should be 0, got {w}"
        );
    }

    #[test]
    fn test_bulge_weight_max_angle() {
        let m = Muscle::new("m", "j");
        let w = m.bulge_weight(FRAC_PI_2);
        assert!(
            (w - 1.0).abs() < 1e-6,
            "bulge weight at max_flex_angle should be 1, got {w}"
        );
    }

    #[test]
    fn test_bulge_weight_half() {
        let m = Muscle::new("m", "j");
        // At half of max_flex_angle, t = 0.5, smoothstep(0.5) = 0.5
        let w = m.bulge_weight(FRAC_PI_2 / 2.0);
        // smoothstep(0.5) = 0.5 * 0.5 * (3 - 2*0.5) = 0.25 * 2.0 = 0.5
        assert!(
            (w - 0.5).abs() < 1e-5,
            "bulge weight at half max angle should be 0.5, got {w}"
        );
    }

    #[test]
    fn test_bulge_weight_clamped() {
        let m = Muscle::new("m", "j");
        // Above max_flex_angle should clamp to 1.0
        let w_over = m.bulge_weight(PI * 10.0);
        assert!(
            (w_over - 1.0).abs() < 1e-6,
            "bulge weight above max should clamp to 1, got {w_over}"
        );
        // Below 0 should clamp to 0.0
        let w_under = m.bulge_weight(-1.0);
        assert!(
            w_under.abs() < 1e-6,
            "bulge weight below 0 should clamp to 0, got {w_under}"
        );
    }

    #[test]
    fn test_compute_displacements_vertex_normal() {
        let mut m = Muscle::new("m", "j");
        m.influences = vec![(0, 1.0)];
        m.bulge_amplitude = 1.0;
        m.bulge_direction = BulgeDirection::VertexNormal;

        let positions = vec![[0.0_f32, 0.0, 0.0]];
        let normals = vec![[0.0_f32, 1.0, 0.0]];

        // At max_flex_angle, w = 1.0, influence = 1.0, dir = (0,1,0)
        let disps = m.compute_displacements(FRAC_PI_2, &positions, &normals);
        assert_eq!(disps.len(), 1);
        let (vid, d) = disps[0];
        assert_eq!(vid, 0);
        assert!((d[0]).abs() < 1e-6);
        assert!(
            (d[1] - 1.0).abs() < 1e-6,
            "y displacement should be 1.0, got {}",
            d[1]
        );
        assert!((d[2]).abs() < 1e-6);
    }

    #[test]
    fn test_compute_displacements_fixed() {
        let mut m = Muscle::new("m", "j");
        m.influences = vec![(0, 1.0)];
        m.bulge_amplitude = 1.0;
        m.bulge_direction = BulgeDirection::Fixed([1.0, 0.0, 0.0]);

        let positions = vec![[0.0_f32, 0.0, 0.0]];
        let normals = vec![[0.0_f32, 1.0, 0.0]];

        let disps = m.compute_displacements(FRAC_PI_2, &positions, &normals);
        assert_eq!(disps.len(), 1);
        let (vid, d) = disps[0];
        assert_eq!(vid, 0);
        // Fixed direction [1,0,0], amplitude 1.0, full weight => x = 1.0
        assert!(
            (d[0] - 1.0).abs() < 1e-6,
            "x displacement should be 1.0, got {}",
            d[0]
        );
        assert!((d[1]).abs() < 1e-6);
        assert!((d[2]).abs() < 1e-6);
    }

    #[test]
    fn test_compute_displacements_radial() {
        let mut m = Muscle::new("m", "j");
        m.influences = vec![(0, 1.0)];
        m.bulge_amplitude = 1.0;
        m.bulge_direction = BulgeDirection::RadialFrom([0.0, 0.0, 0.0]);

        // Vertex at [1, 0, 0] from center [0,0,0] => radial dir = [1,0,0]
        let positions = vec![[1.0_f32, 0.0, 0.0]];
        let normals = vec![[0.0_f32, 1.0, 0.0]];

        let disps = m.compute_displacements(FRAC_PI_2, &positions, &normals);
        assert_eq!(disps.len(), 1);
        let (vid, d) = disps[0];
        assert_eq!(vid, 0);
        assert!(
            (d[0] - 1.0).abs() < 1e-6,
            "x displacement should be 1.0, got {}",
            d[0]
        );
        assert!((d[1]).abs() < 1e-6);
        assert!((d[2]).abs() < 1e-6);
    }

    #[test]
    fn test_simulator_add_muscle() {
        let mut sim = MuscleSimulator::new();
        assert_eq!(sim.muscle_count(), 0);
        sim.add_muscle(Muscle::new("bicep", "elbow"));
        sim.add_muscle(Muscle::new("tricep", "elbow"));
        assert_eq!(sim.muscle_count(), 2);
    }

    #[test]
    fn test_simulator_muscles_for_joint() {
        let mut sim = MuscleSimulator::new();
        sim.add_muscle(Muscle::new("bicep", "elbow"));
        sim.add_muscle(Muscle::new("tricep", "elbow"));
        sim.add_muscle(Muscle::new("quad", "knee"));

        let elbow_muscles = sim.muscles_for_joint("elbow");
        assert_eq!(elbow_muscles.len(), 2);

        let knee_muscles = sim.muscles_for_joint("knee");
        assert_eq!(knee_muscles.len(), 1);

        let hip_muscles = sim.muscles_for_joint("hip");
        assert!(hip_muscles.is_empty());
    }

    #[test]
    fn test_simulator_apply() {
        let mut sim = MuscleSimulator::new();
        let mut m = Muscle::new("bicep", "elbow");
        m.influences = vec![(0, 1.0)];
        m.bulge_amplitude = 1.0;
        m.bulge_direction = BulgeDirection::VertexNormal;
        sim.add_muscle(m);

        let positions = vec![[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let normals = vec![[0.0_f32, 1.0, 0.0], [0.0, 1.0, 0.0]];
        let mut joint_angles = HashMap::new();
        joint_angles.insert("elbow".to_string(), FRAC_PI_2);

        let result = sim.apply(&positions, &normals, &joint_angles);
        assert_eq!(result.len(), 2);
        // Vertex 0 should be displaced by 1.0 in y
        assert!(
            (result[0][1] - 1.0).abs() < 1e-5,
            "vertex 0 y should be ~1.0, got {}",
            result[0][1]
        );
        // Vertex 1 should be unchanged
        assert!((result[1][0] - 1.0).abs() < 1e-6);
        assert!((result[1][1]).abs() < 1e-6);
    }

    #[test]
    fn test_muscle_from_region() {
        // 5 vertices in a row along x
        let positions: Vec<[f32; 3]> = (0..5).map(|i| [i as f32 * 0.1, 0.0, 0.0]).collect();
        let center = [0.2_f32, 0.0, 0.0];
        let radius = 0.15;

        let m = muscle_from_region("test_muscle", "hip", &positions, center, radius, 0.05);
        assert_eq!(m.name, "test_muscle");
        assert_eq!(m.joint_name, "hip");
        assert!((m.bulge_amplitude - 0.05).abs() < 1e-6);

        // Vertices at x=0.1 (dist=0.1), x=0.2 (dist=0.0), x=0.3 (dist=0.1) are within radius=0.15
        // Vertices at x=0.0 (dist=0.2) and x=0.4 (dist=0.2) are outside
        assert!(!m.influences.is_empty());
        // Center vertex (x=0.2, dist=0) should have weight ~1.0
        let center_inf = m.influences.iter().find(|&&(vi, _)| vi == 2);
        assert!(center_inf.is_some());
        let (_, w) = center_inf.expect("should succeed");
        assert!(
            (*w - 1.0).abs() < 1e-5,
            "center vertex weight should be 1.0, got {w}"
        );
    }

    #[test]
    fn test_preset_muscles() {
        let bicep = bicep_muscle("elbow_L");
        assert_eq!(bicep.name, "bicep");
        assert_eq!(bicep.joint_name, "elbow_L");
        assert!((bicep.max_flex_angle - FRAC_PI_2).abs() < 1e-6);
        assert!((bicep.bulge_amplitude - 0.02).abs() < 1e-6);

        let quad = quadricep_muscle("knee_R");
        assert_eq!(quad.name, "quadricep");
        assert_eq!(quad.joint_name, "knee_R");
        assert!((quad.bulge_amplitude - 0.025).abs() < 1e-6);

        let calf = calf_muscle("ankle_L");
        assert_eq!(calf.name, "calf");
        assert_eq!(calf.joint_name, "ankle_L");
        assert!((calf.bulge_amplitude - 0.018).abs() < 1e-6);
    }
}
