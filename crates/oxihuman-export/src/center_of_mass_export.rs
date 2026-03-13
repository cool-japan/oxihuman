// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct CenterOfMass {
    pub time_s: Vec<f32>,
    pub position: Vec<[f32; 3]>,
    pub velocity: Vec<[f32; 3]>,
}

pub fn new_center_of_mass() -> CenterOfMass {
    CenterOfMass {
        time_s: Vec::new(),
        position: Vec::new(),
        velocity: Vec::new(),
    }
}

pub fn com_push(c: &mut CenterOfMass, t: f32, pos: [f32; 3], vel: [f32; 3]) {
    c.time_s.push(t);
    c.position.push(pos);
    c.velocity.push(vel);
}

pub fn com_total_distance(c: &CenterOfMass) -> f32 {
    let mut total = 0.0f32;
    for i in 1..c.position.len() {
        let a = c.position[i - 1];
        let b = c.position[i];
        total += ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt();
    }
    total
}

pub fn com_mean_height(c: &CenterOfMass) -> f32 {
    if c.position.is_empty() {
        return 0.0;
    }
    c.position.iter().map(|p| p[1]).sum::<f32>() / c.position.len() as f32
}

pub fn com_duration_s(c: &CenterOfMass) -> f32 {
    if c.time_s.len() < 2 {
        return 0.0;
    }
    c.time_s.last().copied().unwrap_or(0.0) - c.time_s.first().copied().unwrap_or(0.0)
}

pub fn com_to_csv(c: &CenterOfMass) -> String {
    let mut s = String::from("time_s,px,py,pz,vx,vy,vz\n");
    for i in 0..c.time_s.len() {
        let p = c.position[i];
        let v = c.velocity[i];
        s.push_str(&format!(
            "{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4}\n",
            c.time_s[i], p[0], p[1], p[2], v[0], v[1], v[2]
        ));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_center_of_mass() {
        /* starts empty */
        let c = new_center_of_mass();
        assert!(c.time_s.is_empty());
    }

    #[test]
    fn test_com_push() {
        /* push adds sample */
        let mut c = new_center_of_mass();
        com_push(&mut c, 0.0, [0.0, 1.0, 0.0], [0.0; 3]);
        assert_eq!(c.time_s.len(), 1);
    }

    #[test]
    fn test_com_total_distance() {
        /* distance between two points */
        let mut c = new_center_of_mass();
        com_push(&mut c, 0.0, [0.0, 0.0, 0.0], [0.0; 3]);
        com_push(&mut c, 1.0, [3.0, 4.0, 0.0], [0.0; 3]);
        assert!((com_total_distance(&c) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_com_mean_height() {
        /* mean of y coords */
        let mut c = new_center_of_mass();
        com_push(&mut c, 0.0, [0.0, 2.0, 0.0], [0.0; 3]);
        com_push(&mut c, 1.0, [0.0, 4.0, 0.0], [0.0; 3]);
        assert!((com_mean_height(&c) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_com_duration_s() {
        /* duration = last - first time */
        let mut c = new_center_of_mass();
        com_push(&mut c, 0.0, [0.0; 3], [0.0; 3]);
        com_push(&mut c, 2.0, [0.0; 3], [0.0; 3]);
        assert!((com_duration_s(&c) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_com_to_csv() {
        /* csv has header */
        let c = new_center_of_mass();
        let csv = com_to_csv(&c);
        assert!(csv.contains("time_s"));
    }
}
