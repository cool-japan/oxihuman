// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct StressLine {
    pub points: Vec<[f32; 3]>,
    pub principal_value: f32,
    pub is_compressive: bool,
}

pub fn new_stress_line(principal: f32, compressive: bool) -> StressLine {
    StressLine {
        points: vec![],
        principal_value: principal,
        is_compressive: compressive,
    }
}

pub fn stress_line_push(l: &mut StressLine, p: [f32; 3]) {
    l.points.push(p);
}

pub fn stress_line_length(l: &StressLine) -> f32 {
    if l.points.len() < 2 {
        return 0.0;
    }
    let mut total = 0.0_f32;
    for i in 1..l.points.len() {
        let a = l.points[i - 1];
        let b = l.points[i];
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let dz = b[2] - a[2];
        total += (dx * dx + dy * dy + dz * dz).sqrt();
    }
    total
}

pub fn stress_line_point_count(l: &StressLine) -> usize {
    l.points.len()
}

/// Tension = red, compression = blue.
pub fn stress_line_color(l: &StressLine) -> [f32; 3] {
    if l.is_compressive {
        [0.0, 0.0, 1.0]
    } else {
        [1.0, 0.0, 0.0]
    }
}

pub fn stress_line_is_critical(l: &StressLine, threshold: f32) -> bool {
    l.principal_value.abs() >= threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stress_line() {
        /* construction */
        let l = new_stress_line(5.0, false);
        assert!((l.principal_value - 5.0).abs() < 1e-6);
        assert!(!l.is_compressive);
    }

    #[test]
    fn test_push_and_count() {
        /* push adds points */
        let mut l = new_stress_line(1.0, true);
        stress_line_push(&mut l, [0.0, 0.0, 0.0]);
        stress_line_push(&mut l, [1.0, 0.0, 0.0]);
        assert_eq!(stress_line_point_count(&l), 2);
    }

    #[test]
    fn test_length() {
        /* two points 1 apart */
        let mut l = new_stress_line(1.0, false);
        stress_line_push(&mut l, [0.0, 0.0, 0.0]);
        stress_line_push(&mut l, [1.0, 0.0, 0.0]);
        assert!((stress_line_length(&l) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_color_compressive() {
        /* compressive => blue */
        let l = new_stress_line(1.0, true);
        let c = stress_line_color(&l);
        assert!((c[2] - 1.0).abs() < 1e-6);
        assert!(c[0] < 1e-6);
    }

    #[test]
    fn test_color_tension() {
        /* tensile => red */
        let l = new_stress_line(1.0, false);
        let c = stress_line_color(&l);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_is_critical() {
        /* above threshold */
        let l = new_stress_line(10.0, false);
        assert!(stress_line_is_critical(&l, 5.0));
        assert!(!stress_line_is_critical(&l, 20.0));
    }
}
