// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

pub struct AgeSpot {
    pub center: [f32; 2],
    pub radius_mm: f32,
    pub darkness: f32,
}

pub fn new_age_spot(center: [f32; 2], radius: f32, darkness: f32) -> AgeSpot {
    AgeSpot {
        center,
        radius_mm: radius,
        darkness: darkness.clamp(0.0, 1.0),
    }
}

pub fn spot_area_mm2(s: &AgeSpot) -> f32 {
    PI * s.radius_mm * s.radius_mm
}

pub fn spot_to_csv_line(s: &AgeSpot) -> String {
    format!(
        "{:.4},{:.4},{:.4},{:.4}",
        s.center[0], s.center[1], s.radius_mm, s.darkness
    )
}

pub fn spots_to_csv(spots: &[AgeSpot]) -> String {
    let mut out = String::from("cx,cy,radius_mm,darkness\n");
    for s in spots {
        out.push_str(&spot_to_csv_line(s));
        out.push('\n');
    }
    out
}

pub fn spots_mean_darkness(spots: &[AgeSpot]) -> f32 {
    if spots.is_empty() {
        return 0.0;
    }
    spots.iter().map(|s| s.darkness).sum::<f32>() / spots.len() as f32
}

pub fn spots_count(spots: &[AgeSpot]) -> usize {
    spots.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_age_spot() {
        /* construction */
        let s = new_age_spot([1.0, 2.0], 3.0, 0.7);
        assert!((s.radius_mm - 3.0).abs() < 1e-6);
        assert!((s.darkness - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_area() {
        /* pi * r^2 */
        let s = new_age_spot([0.0, 0.0], 1.0, 0.5);
        assert!((spot_area_mm2(&s) - PI).abs() < 1e-5);
    }

    #[test]
    fn test_to_csv_line() {
        /* CSV line */
        let s = new_age_spot([1.5, 2.5], 3.0, 0.7);
        let line = spot_to_csv_line(&s);
        assert!(line.contains("1.5000"));
    }

    #[test]
    fn test_to_csv_header() {
        /* CSV has header */
        let csv = spots_to_csv(&[]);
        assert!(csv.contains("radius_mm"));
    }

    #[test]
    fn test_mean_darkness() {
        /* mean of 0.4 and 0.6 */
        let spots = vec![
            new_age_spot([0.0, 0.0], 1.0, 0.4),
            new_age_spot([1.0, 0.0], 1.0, 0.6),
        ];
        assert!((spots_mean_darkness(&spots) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_count() {
        /* count */
        let spots: Vec<AgeSpot> = (0..5).map(|_| new_age_spot([0.0, 0.0], 1.0, 0.5)).collect();
        assert_eq!(spots_count(&spots), 5);
    }
}
