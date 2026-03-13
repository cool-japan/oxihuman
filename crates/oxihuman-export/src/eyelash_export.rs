// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct Eyelash {
    pub root_pos: [f32; 3],
    pub tip_pos: [f32; 3],
    pub diameter_um: f32,
    pub curl_angle_deg: f32,
    pub pigment: f32,
}

pub fn new_eyelash(root: [f32; 3], tip: [f32; 3]) -> Eyelash {
    Eyelash {
        root_pos: root,
        tip_pos: tip,
        diameter_um: 80.0,
        curl_angle_deg: 30.0,
        pigment: 1.0,
    }
}

pub fn eyelash_length(e: &Eyelash) -> f32 {
    let d = [
        e.tip_pos[0] - e.root_pos[0],
        e.tip_pos[1] - e.root_pos[1],
        e.tip_pos[2] - e.root_pos[2],
    ];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

pub fn eyelash_to_csv_line(e: &Eyelash) -> String {
    format!(
        "{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.2},{:.2},{:.4}",
        e.root_pos[0],
        e.root_pos[1],
        e.root_pos[2],
        e.tip_pos[0],
        e.tip_pos[1],
        e.tip_pos[2],
        e.diameter_um,
        e.curl_angle_deg,
        e.pigment
    )
}

pub fn eyelashes_to_csv(lashes: &[Eyelash]) -> String {
    let mut out = String::from("rx,ry,rz,tx,ty,tz,diameter_um,curl_deg,pigment\n");
    for l in lashes {
        out.push_str(&eyelash_to_csv_line(l));
        out.push('\n');
    }
    out
}

pub fn eyelash_mean_length(lashes: &[Eyelash]) -> f32 {
    if lashes.is_empty() {
        return 0.0;
    }
    lashes.iter().map(eyelash_length).sum::<f32>() / lashes.len() as f32
}

pub fn eyelash_count(lashes: &[Eyelash]) -> usize {
    lashes.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_eyelash() {
        /* construction */
        let e = new_eyelash([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((e.diameter_um - 80.0).abs() < 1e-6);
    }

    #[test]
    fn test_eyelash_length() {
        /* unit length */
        let e = new_eyelash([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!((eyelash_length(&e) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_csv_line() {
        /* CSV format */
        let e = new_eyelash([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
        let line = eyelash_to_csv_line(&e);
        assert!(line.contains("1.0000"));
    }

    #[test]
    fn test_eyelashes_to_csv_header() {
        /* CSV has header */
        let lashes: Vec<Eyelash> = vec![];
        let csv = eyelashes_to_csv(&lashes);
        assert!(csv.contains("diameter_um"));
    }

    #[test]
    fn test_mean_length() {
        /* mean of two equal lengths */
        let lashes = vec![
            new_eyelash([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]),
            new_eyelash([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]),
        ];
        assert!((eyelash_mean_length(&lashes) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_count() {
        /* count */
        let lashes = vec![new_eyelash([0.0; 3], [1.0, 0.0, 0.0]); 7];
        assert_eq!(eyelash_count(&lashes), 7);
    }
}
