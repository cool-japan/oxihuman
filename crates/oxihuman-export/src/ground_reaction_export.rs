// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct GroundReactionForce {
    pub time_s: Vec<f32>,
    pub force_n: Vec<[f32; 3]>,
    pub cop: Vec<[f32; 2]>,
}

pub fn new_ground_reaction() -> GroundReactionForce {
    GroundReactionForce {
        time_s: Vec::new(),
        force_n: Vec::new(),
        cop: Vec::new(),
    }
}

pub fn grf_push(g: &mut GroundReactionForce, t: f32, force: [f32; 3], cop: [f32; 2]) {
    g.time_s.push(t);
    g.force_n.push(force);
    g.cop.push(cop);
}

pub fn grf_peak_vertical(g: &GroundReactionForce) -> f32 {
    g.force_n.iter().map(|f| f[1]).fold(0.0f32, f32::max)
}

pub fn grf_impulse(g: &GroundReactionForce) -> [f32; 3] {
    if g.time_s.len() < 2 {
        return [0.0; 3];
    }
    let dt = g.time_s.last().copied().unwrap_or(0.0) - g.time_s.first().copied().unwrap_or(0.0);
    let n = g.force_n.len() as f32;
    let mean_f = [
        g.force_n.iter().map(|f| f[0]).sum::<f32>() / n,
        g.force_n.iter().map(|f| f[1]).sum::<f32>() / n,
        g.force_n.iter().map(|f| f[2]).sum::<f32>() / n,
    ];
    [mean_f[0] * dt, mean_f[1] * dt, mean_f[2] * dt]
}

pub fn grf_duration_s(g: &GroundReactionForce) -> f32 {
    if g.time_s.len() < 2 {
        return 0.0;
    }
    g.time_s.last().copied().unwrap_or(0.0) - g.time_s.first().copied().unwrap_or(0.0)
}

pub fn grf_to_csv(g: &GroundReactionForce) -> String {
    let mut s = String::from("time_s,fx,fy,fz,cop_x,cop_y\n");
    for i in 0..g.time_s.len() {
        let f = g.force_n[i];
        let c = g.cop[i];
        s.push_str(&format!(
            "{:.4},{:.4},{:.4},{:.4},{:.4},{:.4}\n",
            g.time_s[i], f[0], f[1], f[2], c[0], c[1]
        ));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ground_reaction() {
        /* starts empty */
        let g = new_ground_reaction();
        assert!(g.time_s.is_empty());
    }

    #[test]
    fn test_grf_push() {
        /* push adds sample */
        let mut g = new_ground_reaction();
        grf_push(&mut g, 0.0, [0.0, 700.0, 0.0], [0.0, 0.0]);
        assert_eq!(g.time_s.len(), 1);
    }

    #[test]
    fn test_grf_peak_vertical() {
        /* finds max vertical force */
        let mut g = new_ground_reaction();
        grf_push(&mut g, 0.0, [0.0, 500.0, 0.0], [0.0, 0.0]);
        grf_push(&mut g, 1.0, [0.0, 900.0, 0.0], [0.0, 0.0]);
        assert!((grf_peak_vertical(&g) - 900.0).abs() < 1e-5);
    }

    #[test]
    fn test_grf_impulse_empty() {
        /* empty returns zero impulse */
        let g = new_ground_reaction();
        let imp = grf_impulse(&g);
        assert_eq!(imp, [0.0f32; 3]);
    }

    #[test]
    fn test_grf_duration_s() {
        /* duration computed correctly */
        let mut g = new_ground_reaction();
        grf_push(&mut g, 0.0, [0.0; 3], [0.0; 2]);
        grf_push(&mut g, 1.5, [0.0; 3], [0.0; 2]);
        assert!((grf_duration_s(&g) - 1.5).abs() < 1e-5);
    }

    #[test]
    fn test_grf_to_csv() {
        /* csv header present */
        let g = new_ground_reaction();
        let csv = grf_to_csv(&g);
        assert!(csv.contains("time_s"));
    }
}
