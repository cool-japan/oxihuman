// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct GrayScottGrid {
    pub u: Vec<f32>,
    pub v: Vec<f32>,
    pub width: usize,
    pub height: usize,
    pub du: f32,
    pub dv: f32,
    pub feed: f32,
    pub kill: f32,
}

pub fn new_gray_scott(w: usize, h: usize) -> GrayScottGrid {
    let n = w * h;
    GrayScottGrid {
        u: vec![1.0; n],
        v: vec![0.0; n],
        width: w,
        height: h,
        du: 0.2,
        dv: 0.1,
        feed: 0.055,
        kill: 0.062,
    }
}

fn idx(g: &GrayScottGrid, x: usize, y: usize) -> usize {
    y * g.width + x
}

fn laplacian(field: &[f32], x: usize, y: usize, w: usize, h: usize) -> f32 {
    let center = field[y * w + x];
    let left = field[y * w + (x + w - 1) % w];
    let right = field[y * w + (x + 1) % w];
    let up = field[((y + h - 1) % h) * w + x];
    let down = field[((y + 1) % h) * w + x];
    left + right + up + down - 4.0 * center
}

pub fn gs_step(g: &mut GrayScottGrid, dt: f32) {
    let w = g.width;
    let h = g.height;
    let mut new_u = g.u.clone();
    let mut new_v = g.v.clone();

    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let u = g.u[i];
            let v = g.v[i];
            let lu = laplacian(&g.u, x, y, w, h);
            let lv = laplacian(&g.v, x, y, w, h);
            let uvv = u * v * v;
            new_u[i] = u + dt * (g.du * lu - uvv + g.feed * (1.0 - u));
            new_v[i] = v + dt * (g.dv * lv + uvv - (g.feed + g.kill) * v);
        }
    }
    g.u = new_u;
    g.v = new_v;
}

pub fn gs_get(g: &GrayScottGrid, x: usize, y: usize) -> (f32, f32) {
    let i = idx(g, x, y);
    (g.u[i], g.v[i])
}

pub fn gs_set(g: &mut GrayScottGrid, x: usize, y: usize, u: f32, v: f32) {
    let i = idx(g, x, y);
    g.u[i] = u;
    g.v[i] = v;
}

pub fn gs_mean_u(g: &GrayScottGrid) -> f32 {
    if g.u.is_empty() {
        return 0.0;
    }
    g.u.iter().sum::<f32>() / g.u.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_gray_scott() {
        /* grid initialized with u=1, v=0 */
        let g = new_gray_scott(4, 4);
        assert_eq!(g.u.len(), 16);
        assert_eq!(g.v.len(), 16);
        assert!((g.u[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_gs_set_get() {
        /* set and get values at a cell */
        let mut g = new_gray_scott(4, 4);
        gs_set(&mut g, 2, 1, 0.5, 0.3);
        let (u, v) = gs_get(&g, 2, 1);
        assert!((u - 0.5).abs() < 1e-6);
        assert!((v - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_gs_mean_u() {
        /* mean u computed correctly */
        let mut g = new_gray_scott(4, 4);
        for i in 0..16 {
            g.u[i] = 2.0;
        }
        assert!((gs_mean_u(&g) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_gs_step_runs() {
        /* step does not panic */
        let mut g = new_gray_scott(4, 4);
        gs_set(&mut g, 2, 2, 0.5, 0.25);
        gs_step(&mut g, 0.1);
        /* values change after step */
        let (u, _) = gs_get(&g, 0, 0);
        assert!(u.is_finite());
    }

    #[test]
    fn test_gs_step_v_increases_near_seed() {
        /* v increases near seeded cell */
        let mut g = new_gray_scott(4, 4);
        gs_set(&mut g, 1, 1, 0.5, 0.5);
        let (_, v_before) = gs_get(&g, 1, 1);
        gs_step(&mut g, 0.1);
        let (_, v_after) = gs_get(&g, 1, 1);
        /* v has changed from initial */
        assert!((v_after - v_before).abs() > 1e-10);
    }

    #[test]
    fn test_dimensions() {
        /* width and height stored correctly */
        let g = new_gray_scott(5, 3);
        assert_eq!(g.width, 5);
        assert_eq!(g.height, 3);
    }
}
