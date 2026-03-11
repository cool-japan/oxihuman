// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct CmuFrame {
    pub joint_angles_deg: Vec<f32>,
    pub timestamp_ms: u64,
}

pub struct CmuMotion {
    pub subject: u32,
    pub action: u32,
    pub frames: Vec<CmuFrame>,
}

pub fn new_cmu_motion(subject: u32, action: u32) -> CmuMotion {
    CmuMotion {
        subject,
        action,
        frames: Vec::new(),
    }
}

pub fn cmu_push_frame(m: &mut CmuMotion, angles: Vec<f32>, ts: u64) {
    m.frames.push(CmuFrame {
        joint_angles_deg: angles,
        timestamp_ms: ts,
    });
}

pub fn cmu_frame_count(m: &CmuMotion) -> usize {
    m.frames.len()
}

pub fn cmu_duration_s(m: &CmuMotion) -> f32 {
    if m.frames.is_empty() {
        return 0.0;
    }
    let last_ts = m.frames.last().map_or(0, |f| f.timestamp_ms);
    let first_ts = m.frames[0].timestamp_ms;
    (last_ts - first_ts) as f32 / 1000.0
}

pub fn cmu_to_csv(m: &CmuMotion) -> String {
    let mut lines = vec![format!("subject,action,timestamp_ms,angles")];
    for f in &m.frames {
        let angles: Vec<_> = f.joint_angles_deg.iter().map(|v| v.to_string()).collect();
        lines.push(format!(
            "{},{},{},{}",
            m.subject,
            m.action,
            f.timestamp_ms,
            angles.join(";")
        ));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cmu_motion() {
        /* construction */
        let m = new_cmu_motion(1, 2);
        assert_eq!(m.subject, 1);
        assert_eq!(m.action, 2);
    }

    #[test]
    fn test_cmu_push_frame() {
        /* push frame increases count */
        let mut m = new_cmu_motion(1, 1);
        cmu_push_frame(&mut m, vec![0.0; 62], 0);
        assert_eq!(cmu_frame_count(&m), 1);
    }

    #[test]
    fn test_cmu_duration_empty() {
        /* no frames => 0s */
        let m = new_cmu_motion(1, 1);
        assert!((cmu_duration_s(&m)).abs() < 1e-6);
    }

    #[test]
    fn test_cmu_duration_two_frames() {
        /* 1000ms = 1s */
        let mut m = new_cmu_motion(1, 1);
        cmu_push_frame(&mut m, vec![0.0; 4], 0);
        cmu_push_frame(&mut m, vec![0.0; 4], 1000);
        assert!((cmu_duration_s(&m) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_cmu_to_csv_header() {
        /* csv has header */
        let m = new_cmu_motion(1, 1);
        let csv = cmu_to_csv(&m);
        assert!(csv.contains("subject"));
    }
}
