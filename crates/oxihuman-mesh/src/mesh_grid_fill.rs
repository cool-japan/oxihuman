// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Grid fill: create a grid mesh between two edge loops.

/* ── legacy API (keep for existing lib.rs exports) ── */

#[derive(Debug, Clone)]
pub struct GridFillConfig {
    pub spans_u: usize,
    pub spans_v: usize,
    pub smooth: bool,
}

impl Default for GridFillConfig {
    fn default() -> Self {
        Self {
            spans_u: 4,
            spans_v: 4,
            smooth: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GridFillResult {
    pub positions: Vec<[f32; 3]>,
    pub quads: Vec<[usize; 4]>,
    pub u_count: usize,
    pub v_count: usize,
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

pub fn grid_fill(
    loop_a: &[[f32; 3]],
    loop_b: &[[f32; 3]],
    config: &GridFillConfig,
) -> GridFillResult {
    if loop_a.is_empty() || loop_a.len() != loop_b.len() {
        return GridFillResult {
            positions: vec![],
            quads: vec![],
            u_count: 0,
            v_count: 0,
        };
    }
    let n_a = loop_a.len();
    let u_steps = config.spans_u.max(1);
    let v_steps = config.spans_v.max(1);
    let mut positions: Vec<[f32; 3]> = Vec::new();
    for vi in 0..=v_steps {
        let tv = vi as f32 / v_steps as f32;
        for ui in 0..=u_steps {
            let tu = ui as f32 / u_steps as f32;
            let idx_a = (tu * (n_a - 1) as f32) as usize;
            let idx_b = (idx_a + 1).min(n_a - 1);
            let frac = tu * (n_a - 1) as f32 - idx_a as f32;
            let pa = lerp3(loop_a[idx_a], loop_a[idx_b], frac);
            let pb = lerp3(loop_b[idx_a], loop_b[idx_b], frac);
            positions.push(lerp3(pa, pb, tv));
        }
    }
    let row_len = u_steps + 1;
    let mut quads: Vec<[usize; 4]> = Vec::new();
    for vi in 0..v_steps {
        for ui in 0..u_steps {
            let i0 = vi * row_len + ui;
            let i1 = i0 + 1;
            let i2 = (vi + 1) * row_len + ui + 1;
            let i3 = (vi + 1) * row_len + ui;
            quads.push([i0, i1, i2, i3]);
        }
    }
    GridFillResult {
        positions,
        quads,
        u_count: u_steps + 1,
        v_count: v_steps + 1,
    }
}

pub fn vertex_count(result: &GridFillResult) -> usize {
    result.positions.len()
}

pub fn quad_count(result: &GridFillResult) -> usize {
    result.quads.len()
}

pub fn dimensions_match(result: &GridFillResult, u: usize, v: usize) -> bool {
    result.u_count == u && result.v_count == v
}

pub fn grid_quads_to_tris(result: &GridFillResult) -> Vec<[usize; 3]> {
    result
        .quads
        .iter()
        .flat_map(|&[a, b, c, d]| [[a, b, c], [a, c, d]])
        .collect()
}

pub fn grid_bounds(result: &GridFillResult) -> ([f32; 3], [f32; 3]) {
    let mut mn = [f32::MAX; 3];
    let mut mx = [f32::MIN; 3];
    for &p in &result.positions {
        for k in 0..3 {
            if p[k] < mn[k] {
                mn[k] = p[k];
            }
            if p[k] > mx[k] {
                mx[k] = p[k];
            }
        }
    }
    (mn, mx)
}

/* ── spec functions (wave 150B) ── */

/// Parameters for spec-style grid fill.
#[derive(Debug, Clone)]
pub struct GridFillParams {
    pub loop_a: Vec<[f32; 3]>,
    pub loop_b: Vec<[f32; 3]>,
    pub rows: usize,
}

/// Build a `GridFillParams`.
pub fn new_grid_fill(loop_a: Vec<[f32; 3]>, loop_b: Vec<[f32; 3]>, rows: usize) -> GridFillParams {
    GridFillParams {
        loop_a,
        loop_b,
        rows,
    }
}

/// Vertex count for a grid fill with n columns and `rows` row steps.
pub fn grid_fill_vertex_count(params: &GridFillParams) -> usize {
    let cols = params.loop_a.len();
    if cols == 0 {
        return 0;
    }
    let rows = params.rows.max(1);
    (rows + 1) * cols
}

/// Face (quad) count for a grid fill.
pub fn grid_fill_face_count(params: &GridFillParams) -> usize {
    let cols = params.loop_a.len();
    if cols < 2 {
        return 0;
    }
    let rows = params.rows.max(1);
    rows * (cols - 1)
}

/// Interpolated vertex at grid position (row, col).
pub fn grid_fill_vertex(params: &GridFillParams, row: usize, col: usize) -> [f32; 3] {
    let n = params.loop_a.len();
    if n == 0 || col >= n {
        return [0.0; 3];
    }
    let rows = params.rows.max(1);
    let t = row as f32 / rows as f32;
    let a = params.loop_a[col];
    let b = params.loop_b[col];
    lerp3(a, b, t)
}

/// Returns true if both loops are non-empty and equal length.
pub fn grid_fill_is_valid(params: &GridFillParams) -> bool {
    !params.loop_a.is_empty() && params.loop_a.len() == params.loop_b.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_line_loops() -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
        let a: Vec<[f32; 3]> = (0..5).map(|i| [i as f32, 0.0, 0.0]).collect();
        let b: Vec<[f32; 3]> = (0..5).map(|i| [i as f32, 1.0, 0.0]).collect();
        (a, b)
    }

    #[test]
    fn test_default_config() {
        let cfg = GridFillConfig::default();
        assert_eq!(cfg.spans_u, 4);
        assert_eq!(cfg.spans_v, 4);
    }

    #[test]
    fn test_vertex_count() {
        let (a, b) = two_line_loops();
        let cfg = GridFillConfig {
            spans_u: 4,
            spans_v: 4,
            smooth: false,
        };
        let result = grid_fill(&a, &b, &cfg);
        /* (4+1) * (4+1) = 25 */
        assert_eq!(vertex_count(&result), 25);
    }

    #[test]
    fn test_quad_count() {
        let (a, b) = two_line_loops();
        let cfg = GridFillConfig {
            spans_u: 4,
            spans_v: 4,
            smooth: false,
        };
        let result = grid_fill(&a, &b, &cfg);
        assert_eq!(quad_count(&result), 16);
    }

    #[test]
    fn test_dimensions_match() {
        let (a, b) = two_line_loops();
        let cfg = GridFillConfig {
            spans_u: 3,
            spans_v: 2,
            smooth: false,
        };
        let result = grid_fill(&a, &b, &cfg);
        assert!(dimensions_match(&result, 4, 3));
    }

    #[test]
    fn test_grid_fill_params_valid() {
        let (a, b) = two_line_loops();
        let p = new_grid_fill(a, b, 4);
        assert!(grid_fill_is_valid(&p));
    }

    #[test]
    fn test_grid_fill_vertex_count_spec() {
        /* 5 columns, 4 rows → 5*(4+1) = 25 */
        let (a, b) = two_line_loops();
        let p = new_grid_fill(a, b, 4);
        assert_eq!(grid_fill_vertex_count(&p), 25);
    }

    #[test]
    fn test_grid_fill_face_count_spec() {
        /* 5 cols, 4 rows → 4*4 = 16 */
        let (a, b) = two_line_loops();
        let p = new_grid_fill(a, b, 4);
        assert_eq!(grid_fill_face_count(&p), 16);
    }

    #[test]
    fn test_grid_fill_vertex_midpoint() {
        /* row=1 of 2, col=0 should be halfway between loop_a[0] and loop_b[0] */
        let a = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let b = vec![[0.0f32, 2.0, 0.0], [1.0, 2.0, 0.0]];
        let p = new_grid_fill(a, b, 2);
        let v = grid_fill_vertex(&p, 1, 0);
        assert!((v[1] - 1.0).abs() < 1e-5);
    }
}
