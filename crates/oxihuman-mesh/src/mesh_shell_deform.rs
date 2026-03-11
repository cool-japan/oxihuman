// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Parameters for shell deformation.
#[allow(dead_code)]
pub struct ShellDeformParams {
    pub thickness: f32,
    pub stiffness: f32,
    pub gravity: f32,
}

impl Default for ShellDeformParams {
    fn default() -> Self {
        Self {
            thickness: 0.01,
            stiffness: 1.0,
            gravity: 9.8,
        }
    }
}

/// Shell deformation result.
#[allow(dead_code)]
pub struct ShellDeformResult {
    pub positions: Vec<[f32; 3]>,
    pub max_displacement: f32,
}

/// Compute per-vertex normals for shell offset.
#[allow(dead_code)]
pub fn vertex_normals_shell(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0_f32; 3]; positions.len()];
    let nf = indices.len() / 3;
    for fi in 0..nf {
        let a = positions[indices[fi * 3] as usize];
        let b = positions[indices[fi * 3 + 1] as usize];
        let c = positions[indices[fi * 3 + 2] as usize];
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        for vi in [fi * 3, fi * 3 + 1, fi * 3 + 2] {
            let idx = indices[vi] as usize;
            normals[idx][0] += n[0];
            normals[idx][1] += n[1];
            normals[idx][2] += n[2];
        }
    }
    for n in &mut normals {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len > 1e-9 {
            n[0] /= len;
            n[1] /= len;
            n[2] /= len;
        }
    }
    normals
}

/// Apply simple shell offset deformation.
#[allow(dead_code)]
pub fn shell_deform(
    positions: &[[f32; 3]],
    indices: &[u32],
    params: &ShellDeformParams,
) -> ShellDeformResult {
    let normals = vertex_normals_shell(positions, indices);
    let mut out = positions.to_vec();
    let mut max_disp = 0.0_f32;
    for (i, p) in out.iter_mut().enumerate() {
        let n = normals[i];
        let disp = params.thickness;
        p[0] += n[0] * disp;
        p[1] += n[1] * disp;
        p[2] += n[2] * disp;
        max_disp = max_disp.max(disp);
    }
    ShellDeformResult {
        positions: out,
        max_displacement: max_disp,
    }
}

/// Compute average shell thickness displacement.
#[allow(dead_code)]
pub fn avg_shell_displacement(original: &[[f32; 3]], deformed: &[[f32; 3]]) -> f32 {
    if original.is_empty() {
        return 0.0;
    }
    let sum: f32 = original
        .iter()
        .zip(deformed.iter())
        .map(|(a, b)| {
            let dx = a[0] - b[0];
            let dy = a[1] - b[1];
            let dz = a[2] - b[2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .sum();
    sum / original.len() as f32
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn shell_deform_to_json(result: &ShellDeformResult) -> String {
    format!(
        r#"{{"vertices":{},"max_displacement":{:.6}}}"#,
        result.positions.len(),
        result.max_displacement
    )
}

/// Validate shell result (same vertex count).
#[allow(dead_code)]
pub fn shell_deform_valid(original: &[[f32; 3]], result: &ShellDeformResult) -> bool {
    original.len() == result.positions.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_tri() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        (pos, vec![0_u32, 1, 2])
    }

    #[test]
    fn vertex_count_preserved() {
        let (pos, idx) = flat_tri();
        let r = shell_deform(&pos, &idx, &ShellDeformParams::default());
        assert!(shell_deform_valid(&pos, &r));
    }

    #[test]
    fn displacement_nonzero() {
        let (pos, idx) = flat_tri();
        let p = ShellDeformParams {
            thickness: 0.1,
            ..Default::default()
        };
        let r = shell_deform(&pos, &idx, &p);
        let avg = avg_shell_displacement(&pos, &r.positions);
        assert!(avg > 0.0);
    }

    #[test]
    fn max_displacement_positive() {
        let (pos, idx) = flat_tri();
        let r = shell_deform(&pos, &idx, &ShellDeformParams::default());
        assert!(r.max_displacement >= 0.0);
    }

    #[test]
    fn json_has_vertices() {
        let (pos, idx) = flat_tri();
        let r = shell_deform(&pos, &idx, &ShellDeformParams::default());
        let j = shell_deform_to_json(&r);
        assert!(j.contains("\"vertices\":3"));
    }

    #[test]
    fn normals_computed() {
        let (pos, idx) = flat_tri();
        let n = vertex_normals_shell(&pos, &idx);
        assert_eq!(n.len(), pos.len());
    }

    #[test]
    fn zero_thickness_no_move() {
        let (pos, idx) = flat_tri();
        let p = ShellDeformParams {
            thickness: 0.0,
            stiffness: 1.0,
            gravity: 0.0,
        };
        let r = shell_deform(&pos, &idx, &p);
        let avg = avg_shell_displacement(&pos, &r.positions);
        assert!(avg < 1e-6);
    }

    #[test]
    fn default_params() {
        let p = ShellDeformParams::default();
        assert!(p.thickness > 0.0);
    }

    #[test]
    fn avg_displacement_symmetric() {
        let pos = vec![[0.0_f32; 3]; 3];
        let deformed = vec![[1.0_f32; 3]; 3];
        let avg = avg_shell_displacement(&pos, &deformed);
        assert!((avg - 3.0_f32.sqrt()).abs() < 1e-5);
    }

    #[test]
    fn empty_shell() {
        let r = shell_deform(&[], &[], &ShellDeformParams::default());
        assert_eq!(r.positions.len(), 0);
    }

    #[test]
    fn large_thickness() {
        let (pos, idx) = flat_tri();
        let p = ShellDeformParams {
            thickness: 100.0,
            stiffness: 1.0,
            gravity: 0.0,
        };
        let r = shell_deform(&pos, &idx, &p);
        assert!(r.max_displacement > 0.0);
    }
}
