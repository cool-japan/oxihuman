// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct JointTorque {
    pub joint_name: String,
    pub time_s: Vec<f32>,
    pub torque_nm: Vec<[f32; 3]>,
}

pub fn new_joint_torque(name: &str) -> JointTorque {
    JointTorque {
        joint_name: name.to_string(),
        time_s: Vec::new(),
        torque_nm: Vec::new(),
    }
}

pub fn torque_push(j: &mut JointTorque, t: f32, torque: [f32; 3]) {
    j.time_s.push(t);
    j.torque_nm.push(torque);
}

pub fn torque_peak(j: &JointTorque) -> f32 {
    j.torque_nm
        .iter()
        .map(|t| (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt())
        .fold(0.0f32, f32::max)
}

pub fn torque_mean_magnitude(j: &JointTorque) -> f32 {
    if j.torque_nm.is_empty() {
        return 0.0;
    }
    let sum: f32 = j
        .torque_nm
        .iter()
        .map(|t| (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt())
        .sum();
    sum / j.torque_nm.len() as f32
}

pub fn torque_duration_s(j: &JointTorque) -> f32 {
    if j.time_s.len() < 2 {
        return 0.0;
    }
    j.time_s.last().copied().unwrap_or(0.0) - j.time_s.first().copied().unwrap_or(0.0)
}

pub fn torque_to_csv(j: &JointTorque) -> String {
    let mut s = format!("joint,{}\ntime_s,tx,ty,tz\n", j.joint_name);
    for (t, torq) in j.time_s.iter().zip(j.torque_nm.iter()) {
        s.push_str(&format!(
            "{:.4},{:.4},{:.4},{:.4}\n",
            t, torq[0], torq[1], torq[2]
        ));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_joint_torque() {
        /* name stored */
        let j = new_joint_torque("knee");
        assert_eq!(j.joint_name, "knee");
    }

    #[test]
    fn test_torque_push() {
        /* push increments count */
        let mut j = new_joint_torque("hip");
        torque_push(&mut j, 0.0, [1.0, 0.0, 0.0]);
        assert_eq!(j.time_s.len(), 1);
    }

    #[test]
    fn test_torque_peak() {
        /* finds max magnitude */
        let mut j = new_joint_torque("ankle");
        torque_push(&mut j, 0.0, [0.0, 0.0, 0.0]);
        torque_push(&mut j, 1.0, [3.0, 4.0, 0.0]);
        assert!((torque_peak(&j) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_torque_mean_magnitude_empty() {
        /* empty returns zero */
        let j = new_joint_torque("shoulder");
        assert_eq!(torque_mean_magnitude(&j), 0.0);
    }

    #[test]
    fn test_torque_duration_s() {
        /* duration is time span */
        let mut j = new_joint_torque("elbow");
        torque_push(&mut j, 0.0, [0.0; 3]);
        torque_push(&mut j, 3.0, [0.0; 3]);
        assert!((torque_duration_s(&j) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_torque_to_csv() {
        /* csv contains joint name */
        let mut j = new_joint_torque("wrist");
        torque_push(&mut j, 0.0, [1.0, 0.0, 0.0]);
        let csv = torque_to_csv(&j);
        assert!(csv.contains("wrist"));
    }
}
