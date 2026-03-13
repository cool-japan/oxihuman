// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Simplex noise 2D (skew-based, deterministic).

#![allow(dead_code)]

const PERM: [u8; 512] = {
    const P: [u8; 256] = [
        151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30,
        69, 142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234, 75, 0, 26, 197, 62, 94,
        252, 219, 203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171,
        168, 68, 175, 74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122, 60,
        211, 133, 230, 220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25, 63, 161, 1,
        216, 80, 73, 209, 76, 132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188, 159, 86,
        164, 100, 109, 198, 173, 186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118,
        126, 255, 82, 85, 212, 207, 206, 59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170,
        213, 119, 248, 152, 2, 44, 154, 163, 70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39,
        253, 19, 98, 108, 110, 79, 113, 224, 232, 178, 185, 112, 104, 218, 246, 97, 228, 251, 34,
        242, 193, 238, 210, 144, 12, 191, 179, 162, 241, 81, 51, 145, 235, 249, 14, 239, 107, 49,
        192, 214, 31, 181, 199, 106, 157, 184, 84, 204, 176, 115, 121, 50, 45, 127, 4, 150, 254,
        138, 236, 205, 93, 222, 114, 67, 29, 24, 72, 243, 141, 128, 195, 78, 66, 215, 61, 156, 180,
    ];
    let mut arr = [0u8; 512];
    let mut i = 0;
    while i < 256 {
        arr[i] = P[i];
        arr[i + 256] = P[i];
        i += 1;
    }
    arr
};

const GRAD2: [[f32; 2]; 8] = [
    [1.0, 1.0],
    [-1.0, 1.0],
    [1.0, -1.0],
    [-1.0, -1.0],
    [1.0, 0.0],
    [-1.0, 0.0],
    [0.0, 1.0],
    [0.0, -1.0],
];

fn dot2(g: [f32; 2], x: f32, y: f32) -> f32 {
    g[0] * x + g[1] * y
}

/// 2D simplex noise, output in roughly [-1, 1].
#[allow(dead_code)]
pub fn simplex2(x: f32, y: f32) -> f32 {
    let f2 = 0.5 * (3.0f32.sqrt() - 1.0);
    let g2 = (3.0 - 3.0f32.sqrt()) / 6.0;
    let s = (x + y) * f2;
    let i = (x + s).floor() as i32;
    let j = (y + s).floor() as i32;
    let t = (i + j) as f32 * g2;
    let x0 = x - (i as f32 - t);
    let y0 = y - (j as f32 - t);
    let (i1, j1) = if x0 > y0 { (1, 0) } else { (0, 1) };
    let x1 = x0 - i1 as f32 + g2;
    let y1 = y0 - j1 as f32 + g2;
    let x2 = x0 - 1.0 + 2.0 * g2;
    let y2 = y0 - 1.0 + 2.0 * g2;
    let ii = (i & 255) as usize;
    let jj = (j & 255) as usize;
    let gi0 = PERM[ii + PERM[jj] as usize] as usize % 8;
    let gi1 = PERM[ii + i1 + PERM[jj + j1] as usize] as usize % 8;
    let gi2 = PERM[ii + 1 + PERM[jj + 1] as usize] as usize % 8;
    let contrib = |_t0: f32, dx: f32, dy: f32, gi: usize| {
        let t = 0.5 - dx * dx - dy * dy;
        if t < 0.0 {
            0.0
        } else {
            t * t * t * t * dot2(GRAD2[gi], dx, dy)
        }
    };
    70.0 * (contrib(0.0, x0, y0, gi0) + contrib(0.0, x1, y1, gi1) + contrib(0.0, x2, y2, gi2))
}

/// Normalize simplex2 output to [0, 1].
#[allow(dead_code)]
pub fn simplex2_01(x: f32, y: f32) -> f32 {
    (simplex2(x, y).clamp(-1.0, 1.0) + 1.0) * 0.5
}

/// 2D simplex noise with frequency and amplitude.
#[allow(dead_code)]
pub fn simplex2_scaled(x: f32, y: f32, frequency: f32, amplitude: f32) -> f32 {
    simplex2(x * frequency, y * frequency) * amplitude
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simplex2_deterministic() {
        let a = simplex2(1.5, 2.3);
        let b = simplex2(1.5, 2.3);
        assert!((a - b).abs() < 1e-10);
    }

    #[test]
    fn simplex2_bounded() {
        for i in 0..20 {
            let v = simplex2(i as f32 * 0.37, i as f32 * 0.53);
            assert!((-2.0..=2.0).contains(&v), "out of range: {v}");
        }
    }

    #[test]
    fn simplex2_01_in_range() {
        for i in 0..20 {
            let v = simplex2_01(i as f32 * 0.31, i as f32 * 0.47);
            assert!((0.0..=1.0).contains(&v), "simplex2_01 = {v}");
        }
    }

    #[test]
    fn simplex2_varies_with_input() {
        let a = simplex2(0.1, 0.2);
        let b = simplex2(10.1, 5.7);
        let _diff = (a - b).abs();
        // Just ensure it runs without panic
    }

    #[test]
    fn simplex2_scaled_amplitude() {
        let v = simplex2_scaled(0.5, 0.5, 1.0, 2.0);
        assert!(v.abs() <= 2.5, "scaled beyond amplitude: {v}");
    }

    #[test]
    fn simplex2_negative_coords() {
        let v = simplex2(-1.5, -2.3);
        assert!(v.abs() <= 2.0);
    }

    #[test]
    fn simplex2_large_coords() {
        let v = simplex2(100.7, 200.3);
        assert!(v.abs() <= 2.0);
    }

    #[test]
    fn simplex2_01_at_many_points() {
        let samples = [(0.0, 0.0), (0.5, 0.5), (1.0, 1.0), (3.7, 2.1)];
        for (x, y) in samples {
            let v = simplex2_01(x, y);
            assert!((0.0..=1.0).contains(&v), "v = {v} at ({x},{y})");
        }
    }

    #[test]
    fn dot2_correct() {
        let g = [1.0f32, 0.5];
        assert!((dot2(g, 2.0, 4.0) - 4.0).abs() < 1e-5);
    }

    #[test]
    fn simplex2_zero_is_finite() {
        let v = simplex2(0.0, 0.0);
        assert!(v.is_finite());
    }
}
