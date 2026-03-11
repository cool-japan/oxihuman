// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Corner smoothing by averaging vertex positions with neighbors.


#[allow(dead_code)]
pub fn corner_angle(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = [b[0]-a[0], b[1]-a[1], b[2]-a[2]];
    let ac = [c[0]-a[0], c[1]-a[1], c[2]-a[2]];
    let dot = ab[0]*ac[0]+ab[1]*ac[1]+ab[2]*ac[2];
    let la = (ab[0]*ab[0]+ab[1]*ab[1]+ab[2]*ab[2]).sqrt();
    let lb = (ac[0]*ac[0]+ac[1]*ac[1]+ac[2]*ac[2]).sqrt();
    if la < 1e-12 || lb < 1e-12 { return 0.0; }
    (dot / (la * lb)).clamp(-1.0, 1.0).acos()
}

#[allow(dead_code)]
pub fn is_sharp_corner(angle: f32, threshold: f32) -> bool {
    angle < threshold
}

#[allow(dead_code)]
pub fn smooth_corners(positions: &mut [[f32; 3]], triangles: &[[u32; 3]], threshold: f32, factor: f32) {
    let n = positions.len();
    let mut accum = vec![[0.0f64; 3]; n];
    let mut counts = vec![0u32; n];
    for tri in triangles {
        let indices = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        #[allow(clippy::needless_range_loop)]
        for k in 0..3 {
            let a = positions[indices[k]];
            let b = positions[indices[(k+1)%3]];
            let c = positions[indices[(k+2)%3]];
            let angle = corner_angle(a, b, c);
            if is_sharp_corner(angle, threshold) {
                let mid = [(b[0]+c[0])*0.5, (b[1]+c[1])*0.5, (b[2]+c[2])*0.5];
                accum[indices[k]][0] += mid[0] as f64;
                accum[indices[k]][1] += mid[1] as f64;
                accum[indices[k]][2] += mid[2] as f64;
                counts[indices[k]] += 1;
            }
        }
    }
    let f = factor.clamp(0.0, 1.0);
    #[allow(clippy::needless_range_loop)]
    for i in 0..n {
        if counts[i] > 0 {
            let avg = [(accum[i][0] / counts[i] as f64) as f32, (accum[i][1] / counts[i] as f64) as f32, (accum[i][2] / counts[i] as f64) as f32];
            positions[i][0] += (avg[0] - positions[i][0]) * f;
            positions[i][1] += (avg[1] - positions[i][1]) * f;
            positions[i][2] += (avg[2] - positions[i][2]) * f;
        }
    }
}

#[allow(dead_code)]
pub fn count_sharp_corners(positions: &[[f32; 3]], triangles: &[[u32; 3]], threshold: f32) -> usize {
    let mut count = 0;
    for tri in triangles {
        for k in 0..3 {
            let a = positions[tri[k] as usize];
            let b = positions[tri[(k+1)%3] as usize];
            let c = positions[tri[(k+2)%3] as usize];
            if is_sharp_corner(corner_angle(a, b, c), threshold) { count += 1; }
        }
    }
    count
}

#[allow(dead_code)]
pub fn corner_smooth_to_json(sharp: usize, total: usize) -> String {
    format!("{{\"sharp\":{},\"total\":{}}}", sharp, total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn tri() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        (vec![[0.0,0.0,0.0],[2.0,0.0,0.0],[0.0,2.0,0.0]], vec![[0,1,2]])
    }

    #[test] fn test_corner_angle() { let a = corner_angle([0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]); assert!((a - PI/2.0).abs() < 1e-4); }
    #[test] fn test_is_sharp() { assert!(is_sharp_corner(0.5, 1.0)); }
    #[test] fn test_not_sharp() { assert!(!is_sharp_corner(2.0, 1.0)); }
    #[test] fn test_smooth_runs() { let(mut p,t)=tri(); smooth_corners(&mut p,&t,PI/3.0,0.5); assert_eq!(p.len(),3); }
    #[test] fn test_count_sharp() { let(p,t)=tri(); let c=count_sharp_corners(&p,&t,PI/3.0); assert!(c>0); }
    #[test] fn test_zero_factor() { let(mut p,t)=tri(); let orig=p.clone(); smooth_corners(&mut p,&t,PI,0.0); assert!((p[0][0]-orig[0][0]).abs()<1e-6); }
    #[test] fn test_to_json() { assert!(corner_smooth_to_json(2,6).contains("sharp")); }
    #[test] fn test_degenerate() { assert!((corner_angle([0.0,0.0,0.0],[0.0,0.0,0.0],[1.0,0.0,0.0])).abs() < 1e-4); }
    #[test] fn test_empty() { smooth_corners(&mut [],&[],PI,0.5); }
    #[test] fn test_right_angle() { let a = corner_angle([0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]); assert!((a - PI/2.0).abs() < 0.01); }
}
