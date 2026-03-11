// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub fn koch_snowflake_points(iterations: u32) -> Vec<[f32; 2]> {
    let mut pts: Vec<[f32; 2]> = vec![[0.0, 0.0], [1.0, 0.0]];
    for _ in 0..iterations {
        let mut new_pts = Vec::new();
        for w in pts.windows(2) {
            let a = w[0];
            let b = w[1];
            let p1 = [(2.0 * a[0] + b[0]) / 3.0, (2.0 * a[1] + b[1]) / 3.0];
            let p3 = [(a[0] + 2.0 * b[0]) / 3.0, (a[1] + 2.0 * b[1]) / 3.0];
            let dx = p3[0] - p1[0];
            let dy = p3[1] - p1[1];
            let p2 = [
                p1[0] + dx * 0.5 - dy * 0.866025,
                p1[1] + dy * 0.5 + dx * 0.866025,
            ];
            new_pts.push(a);
            new_pts.push(p1);
            new_pts.push(p2);
            new_pts.push(p3);
        }
        new_pts.push(pts[pts.len() - 1]);
        pts = new_pts;
    }
    pts
}

pub fn koch_point_count(iterations: u32) -> usize {
    3 * 4_usize.pow(iterations) + 1
}

pub fn sierpinski_triangle_centroids(iterations: u32) -> Vec<[f32; 2]> {
    let mut tris: Vec<([f32; 2], [f32; 2], [f32; 2])> =
        vec![([0.0, 0.0], [1.0, 0.0], [0.5, 0.866025])];
    for _ in 0..iterations {
        let mut new_tris = Vec::new();
        for (a, b, c) in &tris {
            let ab = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
            let bc = [(b[0] + c[0]) * 0.5, (b[1] + c[1]) * 0.5];
            let ca = [(c[0] + a[0]) * 0.5, (c[1] + a[1]) * 0.5];
            new_tris.push((*a, ab, ca));
            new_tris.push((ab, *b, bc));
            new_tris.push((ca, bc, *c));
        }
        tris = new_tris;
    }
    tris.iter()
        .map(|(a, b, c)| [(a[0] + b[0] + c[0]) / 3.0, (a[1] + b[1] + c[1]) / 3.0])
        .collect()
}

pub fn fractal_dimension_estimate(iterations: u32, scale_factor: f32, count_ratio: f32) -> f32 {
    let _ = iterations;
    if scale_factor <= 0.0 || count_ratio <= 0.0 {
        return 0.0;
    }
    count_ratio.ln() / (1.0 / scale_factor).ln()
}

pub fn mandelbrot_escape_time(cx: f32, cy: f32, max_iter: u32) -> u32 {
    let mut zx = 0.0f32;
    let mut zy = 0.0f32;
    for i in 0..max_iter {
        let zx2 = zx * zx - zy * zy + cx;
        let zy2 = 2.0 * zx * zy + cy;
        zx = zx2;
        zy = zy2;
        if zx * zx + zy * zy > 4.0 {
            return i;
        }
    }
    max_iter
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_koch_points_iteration_0() {
        /* iteration 0 = 2 points */
        let pts = koch_snowflake_points(0);
        assert_eq!(pts.len(), 2);
    }

    #[test]
    fn test_koch_points_iteration_1() {
        /* iteration 1 = 5 points */
        let pts = koch_snowflake_points(1);
        assert_eq!(pts.len(), 5);
    }

    #[test]
    fn test_sierpinski_count() {
        /* iteration 0 = 1, iteration 1 = 3, iteration 2 = 9 */
        assert_eq!(sierpinski_triangle_centroids(0).len(), 1);
        assert_eq!(sierpinski_triangle_centroids(1).len(), 3);
    }

    #[test]
    fn test_fractal_dimension_estimate() {
        /* log(3)/log(2) ~ 1.585 for Sierpinski */
        let d = fractal_dimension_estimate(4, 0.5, 3.0);
        assert!((d - 1.585).abs() < 0.01);
    }

    #[test]
    fn test_mandelbrot_interior_point() {
        /* origin is in Mandelbrot set */
        let et = mandelbrot_escape_time(0.0, 0.0, 100);
        assert_eq!(et, 100);
    }
}
