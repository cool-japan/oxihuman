#![allow(dead_code)]

use std::collections::HashMap;

/// Simulates facial muscle activations.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceMuscleSim {
    tensions: HashMap<String, f32>,
    time: f32,
}

#[allow(dead_code)]
pub fn new_face_muscle_sim() -> FaceMuscleSim {
    FaceMuscleSim { tensions: HashMap::new(), time: 0.0 }
}

#[allow(dead_code)]
pub fn activate_face_muscle(sim: &mut FaceMuscleSim, name: &str, intensity: f32) {
    let val = intensity.clamp(0.0, 1.0);
    sim.tensions.insert(name.to_string(), val);
}

#[allow(dead_code)]
pub fn face_muscle_tension(sim: &FaceMuscleSim, name: &str) -> f32 {
    sim.tensions.get(name).copied().unwrap_or(0.0)
}

#[allow(dead_code)]
pub fn face_muscle_count(sim: &FaceMuscleSim) -> usize {
    sim.tensions.len()
}

#[allow(dead_code)]
pub fn relax_face_muscles(sim: &mut FaceMuscleSim, decay: f32) {
    let factor = (1.0 - decay).max(0.0);
    for v in sim.tensions.values_mut() {
        *v *= factor;
    }
}

#[allow(dead_code)]
pub fn face_muscle_to_params(sim: &FaceMuscleSim) -> Vec<(String, f32)> {
    let mut result: Vec<(String, f32)> = sim.tensions.iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

#[allow(dead_code)]
pub fn face_sim_step(sim: &mut FaceMuscleSim, dt: f32) {
    sim.time += dt;
}

#[allow(dead_code)]
pub fn face_sim_reset(sim: &mut FaceMuscleSim) {
    sim.tensions.clear();
    sim.time = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let s = new_face_muscle_sim();
        assert_eq!(face_muscle_count(&s), 0);
    }

    #[test]
    fn test_activate() {
        let mut s = new_face_muscle_sim();
        activate_face_muscle(&mut s, "zygomatic", 0.8);
        assert!((face_muscle_tension(&s, "zygomatic") - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_tension_missing() {
        let s = new_face_muscle_sim();
        assert!((face_muscle_tension(&s, "none")).abs() < 1e-6);
    }

    #[test]
    fn test_clamp() {
        let mut s = new_face_muscle_sim();
        activate_face_muscle(&mut s, "x", 2.0);
        assert!((face_muscle_tension(&s, "x") - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_relax() {
        let mut s = new_face_muscle_sim();
        activate_face_muscle(&mut s, "a", 1.0);
        relax_face_muscles(&mut s, 0.5);
        assert!((face_muscle_tension(&s, "a") - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_params() {
        let mut s = new_face_muscle_sim();
        activate_face_muscle(&mut s, "b", 0.3);
        activate_face_muscle(&mut s, "a", 0.7);
        let p = face_muscle_to_params(&s);
        assert_eq!(p[0].0, "a");
    }

    #[test]
    fn test_step() {
        let mut s = new_face_muscle_sim();
        face_sim_step(&mut s, 0.016);
        assert!(s.time > 0.0);
    }

    #[test]
    fn test_reset() {
        let mut s = new_face_muscle_sim();
        activate_face_muscle(&mut s, "x", 1.0);
        face_sim_step(&mut s, 1.0);
        face_sim_reset(&mut s);
        assert_eq!(face_muscle_count(&s), 0);
        assert!((s.time).abs() < 1e-6);
    }

    #[test]
    fn test_count() {
        let mut s = new_face_muscle_sim();
        activate_face_muscle(&mut s, "a", 0.5);
        activate_face_muscle(&mut s, "b", 0.5);
        assert_eq!(face_muscle_count(&s), 2);
    }

    #[test]
    fn test_relax_full() {
        let mut s = new_face_muscle_sim();
        activate_face_muscle(&mut s, "a", 1.0);
        relax_face_muscles(&mut s, 1.0);
        assert!((face_muscle_tension(&s, "a")).abs() < 1e-6);
    }
}
