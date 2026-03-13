// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Discrete Morse theory analysis of a scalar field on a mesh.
#[allow(dead_code)]
pub enum CriticalPointType {
    Minimum,
    Saddle,
    Maximum,
}

#[allow(dead_code)]
pub struct CriticalPoint {
    pub vertex_idx: usize,
    pub point_type: CriticalPointType,
    pub value: f32,
}

#[allow(dead_code)]
pub struct MorseResult {
    pub critical_points: Vec<CriticalPoint>,
    pub minima_count: usize,
    pub saddle_count: usize,
    pub maxima_count: usize,
}

/// Build vertex adjacency from triangle mesh.
fn build_adjacency(n_verts: usize, indices: &[u32]) -> Vec<Vec<usize>> {
    let mut adj = vec![vec![]; n_verts];
    for chunk in indices.chunks(3) {
        if chunk.len() == 3 {
            let (a, b, c) = (chunk[0] as usize, chunk[1] as usize, chunk[2] as usize);
            if a < n_verts && b < n_verts && c < n_verts {
                if !adj[a].contains(&b) {
                    adj[a].push(b);
                }
                if !adj[a].contains(&c) {
                    adj[a].push(c);
                }
                if !adj[b].contains(&a) {
                    adj[b].push(a);
                }
                if !adj[b].contains(&c) {
                    adj[b].push(c);
                }
                if !adj[c].contains(&a) {
                    adj[c].push(a);
                }
                if !adj[c].contains(&b) {
                    adj[c].push(b);
                }
            }
        }
    }
    adj
}

/// Classify each vertex as minimum, saddle, or maximum based on scalar field.
#[allow(dead_code)]
pub fn compute_morse_critical_points(
    scalar_field: &[f32],
    positions: &[[f32; 3]],
    indices: &[u32],
) -> MorseResult {
    let n = scalar_field.len().min(positions.len());
    if n == 0 {
        return MorseResult {
            critical_points: vec![],
            minima_count: 0,
            saddle_count: 0,
            maxima_count: 0,
        };
    }
    let adj = build_adjacency(n, indices);
    let mut critical_points = Vec::new();
    let mut minima_count = 0;
    let mut saddle_count = 0;
    let mut maxima_count = 0;

    for v in 0..n {
        if adj[v].is_empty() {
            continue;
        }
        let val = scalar_field[v];
        let neighbors: Vec<f32> = adj[v]
            .iter()
            .filter(|&&nb| nb < n)
            .map(|&nb| scalar_field[nb])
            .collect();
        if neighbors.is_empty() {
            continue;
        }
        let all_lower = neighbors.iter().all(|&nv| nv < val);
        let all_higher = neighbors.iter().all(|&nv| nv > val);
        if all_lower {
            maxima_count += 1;
            critical_points.push(CriticalPoint {
                vertex_idx: v,
                point_type: CriticalPointType::Maximum,
                value: val,
            });
        } else if all_higher {
            minima_count += 1;
            critical_points.push(CriticalPoint {
                vertex_idx: v,
                point_type: CriticalPointType::Minimum,
                value: val,
            });
        } else {
            // Simplified saddle detection: mixed neighbors
            let lower_count = neighbors.iter().filter(|&&nv| nv < val).count();
            let higher_count = neighbors.iter().filter(|&&nv| nv > val).count();
            if lower_count > 0 && higher_count > 0 {
                saddle_count += 1;
                critical_points.push(CriticalPoint {
                    vertex_idx: v,
                    point_type: CriticalPointType::Saddle,
                    value: val,
                });
            }
        }
    }

    MorseResult {
        critical_points,
        minima_count,
        saddle_count,
        maxima_count,
    }
}

#[allow(dead_code)]
pub fn critical_point_count(r: &MorseResult) -> usize {
    r.critical_points.len()
}

/// Euler characteristic from Morse theory: minima - saddles + maxima
#[allow(dead_code)]
pub fn morse_euler_characteristic(r: &MorseResult) -> i32 {
    r.minima_count as i32 - r.saddle_count as i32 + r.maxima_count as i32
}

#[allow(dead_code)]
pub fn morse_result_to_json(r: &MorseResult) -> String {
    format!(
        "{{\"minima\":{},\"saddles\":{},\"maxima\":{},\"total\":{}}}",
        r.minima_count,
        r.saddle_count,
        r.maxima_count,
        r.critical_points.len()
    )
}

#[allow(dead_code)]
pub fn find_global_minimum(scalar_field: &[f32]) -> Option<usize> {
    if scalar_field.is_empty() {
        return None;
    }
    let mut best = 0;
    for (i, &v) in scalar_field.iter().enumerate() {
        if v < scalar_field[best] {
            best = i;
        }
    }
    Some(best)
}

#[allow(dead_code)]
pub fn find_global_maximum(scalar_field: &[f32]) -> Option<usize> {
    if scalar_field.is_empty() {
        return None;
    }
    let mut best = 0;
    for (i, &v) in scalar_field.iter().enumerate() {
        if v > scalar_field[best] {
            best = i;
        }
    }
    Some(best)
}

#[allow(dead_code)]
pub fn field_range(scalar_field: &[f32]) -> (f32, f32) {
    if scalar_field.is_empty() {
        return (0.0, 0.0);
    }
    let mut mn = scalar_field[0];
    let mut mx = scalar_field[0];
    for &v in scalar_field {
        if v < mn {
            mn = v;
        }
        if v > mx {
            mx = v;
        }
    }
    (mn, mx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 1, 2, 3];
        (pos, idx)
    }

    #[test]
    fn test_empty_field() {
        let r = compute_morse_critical_points(&[], &[], &[]);
        assert_eq!(r.critical_points.len(), 0);
    }

    #[test]
    fn test_monotone_field_has_extrema() {
        let (pos, idx) = line_mesh();
        let field = vec![0.0, 1.0, 2.0, 3.0];
        let r = compute_morse_critical_points(&field, &pos, &idx);
        assert!(r.minima_count > 0 || r.maxima_count > 0);
    }

    #[test]
    fn test_critical_point_count_fn() {
        let (pos, idx) = line_mesh();
        let field = vec![1.0, 3.0, 0.0, 2.0];
        let r = compute_morse_critical_points(&field, &pos, &idx);
        assert_eq!(critical_point_count(&r), r.critical_points.len());
    }

    #[test]
    fn test_global_minimum() {
        let f = vec![3.0, 1.0, 2.0];
        assert_eq!(find_global_minimum(&f), Some(1));
    }

    #[test]
    fn test_global_maximum() {
        let f = vec![1.0, 4.0, 2.0];
        assert_eq!(find_global_maximum(&f), Some(1));
    }

    #[test]
    fn test_field_range() {
        let f = vec![1.0, 5.0, 3.0, 2.0];
        let (mn, mx) = field_range(&f);
        assert!((mn - 1.0).abs() < 1e-6);
        assert!((mx - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_empty_global_min() {
        assert!(find_global_minimum(&[]).is_none());
    }

    #[test]
    fn test_to_json() {
        let (pos, idx) = line_mesh();
        let field = vec![0.0, 2.0, 1.0, 3.0];
        let r = compute_morse_critical_points(&field, &pos, &idx);
        let j = morse_result_to_json(&r);
        assert!(j.contains("minima"));
        assert!(j.contains("maxima"));
    }

    #[test]
    fn test_morse_euler_characteristic() {
        let r = MorseResult {
            critical_points: vec![],
            minima_count: 2,
            saddle_count: 1,
            maxima_count: 1,
        };
        assert_eq!(morse_euler_characteristic(&r), 2);
    }

    #[test]
    fn test_field_range_empty() {
        let (mn, mx) = field_range(&[]);
        assert_eq!(mn, 0.0);
        assert_eq!(mx, 0.0);
    }
}
