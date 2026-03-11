// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct DensePoseResult {
    pub i_map: Vec<u8>,
    pub u_map: Vec<f32>,
    pub v_map: Vec<f32>,
    pub width: u32,
    pub height: u32,
}

pub fn new_dense_pose_result(w: u32, h: u32) -> DensePoseResult {
    let n = (w * h) as usize;
    DensePoseResult {
        i_map: vec![0; n],
        u_map: vec![0.0; n],
        v_map: vec![0.0; n],
        width: w,
        height: h,
    }
}

pub fn dense_pose_set(r: &mut DensePoseResult, x: u32, y: u32, i: u8, u: f32, v: f32) {
    if x < r.width && y < r.height {
        let idx = (y * r.width + x) as usize;
        r.i_map[idx] = i;
        r.u_map[idx] = u;
        r.v_map[idx] = v;
    }
}

pub fn dense_pose_get(r: &DensePoseResult, x: u32, y: u32) -> (u8, f32, f32) {
    let idx = (y * r.width + x) as usize;
    (r.i_map[idx], r.u_map[idx], r.v_map[idx])
}

pub fn dense_pose_coverage(r: &DensePoseResult) -> f32 {
    let total = r.i_map.len();
    if total == 0 {
        return 0.0;
    }
    let nonzero = r.i_map.iter().filter(|&&v| v > 0).count();
    nonzero as f32 / total as f32
}

pub fn dense_pose_to_bytes(r: &DensePoseResult) -> Vec<u8> {
    let mut out = Vec::with_capacity(r.i_map.len() * 9);
    for (i, (&imap, (&u, &v))) in r
        .i_map
        .iter()
        .zip(r.u_map.iter().zip(r.v_map.iter()))
        .enumerate()
    {
        let _ = i;
        out.push(imap);
        out.extend_from_slice(&u.to_le_bytes());
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_dense_pose_result() {
        /* correct size */
        let r = new_dense_pose_result(4, 4);
        assert_eq!(r.i_map.len(), 16);
    }

    #[test]
    fn test_dense_pose_set_get() {
        /* set and get */
        let mut r = new_dense_pose_result(8, 8);
        dense_pose_set(&mut r, 2, 3, 5, 0.4, 0.6);
        let (i, u, v) = dense_pose_get(&r, 2, 3);
        assert_eq!(i, 5);
        assert!((u - 0.4).abs() < 1e-5);
        assert!((v - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_dense_pose_coverage_zero() {
        /* empty = 0 coverage */
        let r = new_dense_pose_result(4, 4);
        assert!((dense_pose_coverage(&r)).abs() < 1e-6);
    }

    #[test]
    fn test_dense_pose_coverage_partial() {
        /* one set pixel */
        let mut r = new_dense_pose_result(2, 2);
        dense_pose_set(&mut r, 0, 0, 1, 0.5, 0.5);
        assert!((dense_pose_coverage(&r) - 0.25).abs() < 1e-5);
    }

    #[test]
    fn test_dense_pose_to_bytes_size() {
        /* bytes size = n * 9 */
        let r = new_dense_pose_result(2, 2);
        let bytes = dense_pose_to_bytes(&r);
        assert_eq!(bytes.len(), 4 * 9);
    }
}
