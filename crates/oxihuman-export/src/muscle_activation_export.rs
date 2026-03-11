// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct MuscleActivation {
    pub muscle_name: String,
    pub time_s: Vec<f32>,
    pub activation: Vec<f32>,
}

pub fn new_muscle_activation(name: &str) -> MuscleActivation {
    MuscleActivation {
        muscle_name: name.to_string(),
        time_s: Vec::new(),
        activation: Vec::new(),
    }
}

pub fn activation_push(m: &mut MuscleActivation, t: f32, a: f32) {
    m.time_s.push(t);
    m.activation.push(a.clamp(0.0, 1.0));
}

pub fn activation_mean(m: &MuscleActivation) -> f32 {
    if m.activation.is_empty() {
        return 0.0;
    }
    m.activation.iter().sum::<f32>() / m.activation.len() as f32
}

pub fn activation_peak(m: &MuscleActivation) -> f32 {
    m.activation.iter().cloned().fold(0.0f32, f32::max)
}

pub fn activation_duration_s(m: &MuscleActivation) -> f32 {
    if m.time_s.len() < 2 {
        return 0.0;
    }
    m.time_s.last().copied().unwrap_or(0.0) - m.time_s.first().copied().unwrap_or(0.0)
}

pub fn activation_to_csv(m: &MuscleActivation) -> String {
    let mut s = format!("muscle,{}\ntime_s,activation\n", m.muscle_name);
    for (t, a) in m.time_s.iter().zip(m.activation.iter()) {
        s.push_str(&format!("{:.4},{:.4}\n", t, a));
    }
    s
}

pub fn activation_to_bytes(m: &MuscleActivation) -> Vec<u8> {
    let mut b = Vec::new();
    let n = m.time_s.len() as u32;
    b.extend_from_slice(&n.to_le_bytes());
    for &t in &m.time_s {
        b.extend_from_slice(&t.to_le_bytes());
    }
    for &a in &m.activation {
        b.extend_from_slice(&a.to_le_bytes());
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_muscle_activation() {
        /* name stored correctly */
        let m = new_muscle_activation("bicep");
        assert_eq!(m.muscle_name, "bicep");
        assert!(m.time_s.is_empty());
    }

    #[test]
    fn test_activation_push() {
        /* push adds samples */
        let mut m = new_muscle_activation("quad");
        activation_push(&mut m, 0.1, 0.8);
        assert_eq!(m.time_s.len(), 1);
    }

    #[test]
    fn test_activation_mean() {
        /* mean of [0, 1] is 0.5 */
        let mut m = new_muscle_activation("glute");
        activation_push(&mut m, 0.0, 0.0);
        activation_push(&mut m, 1.0, 1.0);
        assert!((activation_mean(&m) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_activation_peak() {
        /* peak finds max */
        let mut m = new_muscle_activation("hamstring");
        activation_push(&mut m, 0.0, 0.3);
        activation_push(&mut m, 0.5, 0.9);
        assert!((activation_peak(&m) - 0.9).abs() < 1e-5);
    }

    #[test]
    fn test_activation_duration_s() {
        /* duration is last-first time */
        let mut m = new_muscle_activation("calf");
        activation_push(&mut m, 0.0, 0.5);
        activation_push(&mut m, 2.5, 0.5);
        assert!((activation_duration_s(&m) - 2.5).abs() < 1e-5);
    }

    #[test]
    fn test_activation_to_csv() {
        /* csv contains muscle name */
        let mut m = new_muscle_activation("deltoid");
        activation_push(&mut m, 0.0, 0.5);
        let csv = activation_to_csv(&m);
        assert!(csv.contains("deltoid"));
    }

    #[test]
    fn test_activation_to_bytes() {
        /* bytes have correct length */
        let mut m = new_muscle_activation("trap");
        activation_push(&mut m, 0.0, 0.5);
        let b = activation_to_bytes(&m);
        /* 4 bytes for count + 4*1 for time + 4*1 for activation */
        assert_eq!(b.len(), 12);
    }
}
