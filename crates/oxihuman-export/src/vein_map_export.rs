// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct VeinMap {
    pub paths: Vec<Vec<[f32; 3]>>,
    pub widths: Vec<f32>,
    pub depths: Vec<f32>,
}

pub fn new_vein_map() -> VeinMap {
    VeinMap {
        paths: vec![],
        widths: vec![],
        depths: vec![],
    }
}

pub fn vein_add_path(m: &mut VeinMap, points: Vec<[f32; 3]>, width: f32, depth: f32) {
    m.paths.push(points);
    m.widths.push(width);
    m.depths.push(depth);
}

pub fn vein_path_count(m: &VeinMap) -> usize {
    m.paths.len()
}

pub fn vein_total_length(m: &VeinMap) -> f32 {
    m.paths.iter().map(|path| path_length(path)).sum()
}

fn path_length(pts: &[[f32; 3]]) -> f32 {
    if pts.len() < 2 {
        return 0.0;
    }
    let mut total = 0.0_f32;
    for i in 1..pts.len() {
        let a = pts[i - 1];
        let b = pts[i];
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let dz = b[2] - a[2];
        total += (dx * dx + dy * dy + dz * dz).sqrt();
    }
    total
}

pub fn vein_to_json(m: &VeinMap) -> String {
    let mut paths_json = String::new();
    for (i, (path, (&w, &d))) in m
        .paths
        .iter()
        .zip(m.widths.iter().zip(m.depths.iter()))
        .enumerate()
    {
        if i > 0 {
            paths_json.push(',');
        }
        let pts: Vec<String> = path
            .iter()
            .map(|p| format!("[{:.4},{:.4},{:.4}]", p[0], p[1], p[2]))
            .collect();
        paths_json.push_str(&format!(
            "{{\"width\":{w:.4},\"depth\":{d:.4},\"points\":[{}]}}",
            pts.join(",")
        ));
    }
    format!("{{\"veins\":[{paths_json}]}}")
}

pub fn vein_mean_depth(m: &VeinMap) -> f32 {
    if m.depths.is_empty() {
        return 0.0;
    }
    m.depths.iter().sum::<f32>() / m.depths.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_vein_map() {
        /* empty on construction */
        let m = new_vein_map();
        assert_eq!(vein_path_count(&m), 0);
    }

    #[test]
    fn test_add_path() {
        /* add and count */
        let mut m = new_vein_map();
        vein_add_path(&mut m, vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]], 0.5, 0.1);
        assert_eq!(vein_path_count(&m), 1);
    }

    #[test]
    fn test_total_length() {
        /* single path of length 1 */
        let mut m = new_vein_map();
        vein_add_path(&mut m, vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]], 0.5, 0.1);
        assert!((vein_total_length(&m) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        /* JSON contains veins key */
        let mut m = new_vein_map();
        vein_add_path(&mut m, vec![[0.0; 3]], 0.5, 0.1);
        let json = vein_to_json(&m);
        assert!(json.contains("veins"));
    }

    #[test]
    fn test_mean_depth() {
        /* mean depth */
        let mut m = new_vein_map();
        vein_add_path(&mut m, vec![], 0.5, 0.2);
        vein_add_path(&mut m, vec![], 0.5, 0.4);
        assert!((vein_mean_depth(&m) - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_empty_mean_depth() {
        /* empty => 0 */
        let m = new_vein_map();
        assert!((vein_mean_depth(&m)).abs() < 1e-6);
    }
}
