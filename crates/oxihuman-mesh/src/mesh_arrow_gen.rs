// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct ArrowParams {
    pub shaft_radius: f32,
    pub head_radius: f32,
    pub shaft_length: f32,
    pub head_length: f32,
    pub segments: u32,
}

pub fn new_arrow(shaft_len: f32, head_len: f32) -> ArrowParams {
    ArrowParams {
        shaft_radius: 0.05,
        head_radius: 0.15,
        shaft_length: shaft_len,
        head_length: head_len,
        segments: 16,
    }
}

pub fn arrow_total_length(p: &ArrowParams) -> f32 {
    p.shaft_length + p.head_length
}

pub fn arrow_vertex_count(p: &ArrowParams) -> usize {
    let s = p.segments as usize;
    s * 4 + 2
}

pub fn arrow_face_count(p: &ArrowParams) -> usize {
    let s = p.segments as usize;
    s * 4
}

pub fn arrow_tip_position(p: &ArrowParams, origin: [f32; 3], direction: [f32; 3]) -> [f32; 3] {
    let len = arrow_total_length(p);
    let dl =
        (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2])
            .sqrt()
            .max(1e-10);
    let dn = [direction[0] / dl, direction[1] / dl, direction[2] / dl];
    [
        origin[0] + dn[0] * len,
        origin[1] + dn[1] * len,
        origin[2] + dn[2] * len,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_arrow() {
        /* construction */
        let a = new_arrow(0.8, 0.2);
        assert!((a.shaft_length - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_arrow_total_length() {
        /* shaft + head */
        let a = new_arrow(0.8, 0.2);
        assert!((arrow_total_length(&a) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_arrow_tip_along_x() {
        /* tip should be origin + [len,0,0] for x direction */
        let a = new_arrow(0.8, 0.2);
        let tip = arrow_tip_position(&a, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!((tip[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_arrow_vertex_count() {
        /* > 0 */
        let a = new_arrow(0.8, 0.2);
        assert!(arrow_vertex_count(&a) > 0);
    }

    #[test]
    fn test_arrow_face_count() {
        /* > 0 */
        let a = new_arrow(0.8, 0.2);
        assert!(arrow_face_count(&a) > 0);
    }
}
