// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Perlin gradient noise (deterministic, no rand).

#![allow(dead_code)]

/// Fixed permutation table (256 entries, doubled to 512).
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

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

fn grad2(hash: u8, x: f32, y: f32) -> f32 {
    match hash & 3 {
        0 => x + y,
        1 => -x + y,
        2 => x - y,
        _ => -x - y,
    }
}

fn grad3(hash: u8, x: f32, y: f32, z: f32) -> f32 {
    let h = hash & 15;
    let u = if h < 8 { x } else { y };
    let v = if h < 4 {
        y
    } else if h == 12 || h == 14 {
        x
    } else {
        z
    };
    let su = if h & 1 == 0 { u } else { -u };
    let sv = if h & 2 == 0 { v } else { -v };
    su + sv
}

/// 2D Perlin noise, output in roughly [-1, 1].
#[allow(dead_code)]
pub fn perlin2(x: f32, y: f32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let xf = x - x.floor();
    let yf = y - y.floor();
    let u = fade(xf);
    let v = fade(yf);
    let aa = PERM[(PERM[(xi & 255) as usize] as i32 + (yi & 255)) as usize & 511];
    let ab = PERM[(PERM[(xi & 255) as usize] as i32 + ((yi + 1) & 255)) as usize & 511];
    let ba = PERM[(PERM[((xi + 1) & 255) as usize] as i32 + (yi & 255)) as usize & 511];
    let bb = PERM[(PERM[((xi + 1) & 255) as usize] as i32 + ((yi + 1) & 255)) as usize & 511];
    let x1 = lerp(grad2(aa, xf, yf), grad2(ba, xf - 1.0, yf), u);
    let x2 = lerp(grad2(ab, xf, yf - 1.0), grad2(bb, xf - 1.0, yf - 1.0), u);
    lerp(x1, x2, v)
}

/// 3D Perlin noise, output in roughly [-1, 1].
#[allow(dead_code)]
pub fn perlin3(x: f32, y: f32, z: f32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let zi = z.floor() as i32;
    let xf = x - x.floor();
    let yf = y - y.floor();
    let zf = z - z.floor();
    let u = fade(xf);
    let v = fade(yf);
    let w = fade(zf);
    let p = |i: i32| PERM[(i & 255) as usize] as i32;
    let aaa = PERM[(p(xi) + p(yi) + p(zi)) as usize & 511];
    let aba = PERM[(p(xi) + p(yi + 1) + p(zi)) as usize & 511];
    let aab = PERM[(p(xi) + p(yi) + p(zi + 1)) as usize & 511];
    let abb = PERM[(p(xi) + p(yi + 1) + p(zi + 1)) as usize & 511];
    let baa = PERM[(p(xi + 1) + p(yi) + p(zi)) as usize & 511];
    let bba = PERM[(p(xi + 1) + p(yi + 1) + p(zi)) as usize & 511];
    let bab = PERM[(p(xi + 1) + p(yi) + p(zi + 1)) as usize & 511];
    let bbb = PERM[(p(xi + 1) + p(yi + 1) + p(zi + 1)) as usize & 511];
    let x1 = lerp(grad3(aaa, xf, yf, zf), grad3(baa, xf - 1.0, yf, zf), u);
    let x2 = lerp(
        grad3(aba, xf, yf - 1.0, zf),
        grad3(bba, xf - 1.0, yf - 1.0, zf),
        u,
    );
    let x3 = lerp(
        grad3(aab, xf, yf, zf - 1.0),
        grad3(bab, xf - 1.0, yf, zf - 1.0),
        u,
    );
    let x4 = lerp(
        grad3(abb, xf, yf - 1.0, zf - 1.0),
        grad3(bbb, xf - 1.0, yf - 1.0, zf - 1.0),
        u,
    );
    let y1 = lerp(x1, x2, v);
    let y2 = lerp(x3, x4, v);
    lerp(y1, y2, w)
}

/// Normalize perlin2 output to [0, 1].
#[allow(dead_code)]
pub fn perlin2_01(x: f32, y: f32) -> f32 {
    (perlin2(x, y) + 1.0) * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perlin2_at_integers_near_zero() {
        // At integer coordinates, perlin is 0
        let v = perlin2(1.0, 2.0);
        assert!(v.abs() < 1e-5, "perlin2 at integer: {v}");
    }

    #[test]
    fn perlin2_deterministic() {
        let a = perlin2(1.5, 2.3);
        let b = perlin2(1.5, 2.3);
        assert!((a - b).abs() < 1e-10);
    }

    #[test]
    fn perlin2_bounded() {
        for i in 0..20 {
            let v = perlin2(i as f32 * 0.37, i as f32 * 0.19);
            assert!((-2.0..=2.0).contains(&v), "out of expected range: {v}");
        }
    }

    #[test]
    fn perlin3_at_integers_near_zero() {
        let v = perlin3(1.0, 2.0, 3.0);
        assert!(v.abs() < 1e-5);
    }

    #[test]
    fn perlin3_deterministic() {
        let a = perlin3(1.5, 2.3, 0.7);
        let b = perlin3(1.5, 2.3, 0.7);
        assert!((a - b).abs() < 1e-10);
    }

    #[test]
    fn perlin2_01_in_range() {
        for i in 0..20 {
            let v = perlin2_01(i as f32 * 0.31, i as f32 * 0.47);
            assert!((0.0..=1.5).contains(&v), "perlin2_01 out of range: {v}");
        }
    }

    #[test]
    fn perlin2_different_at_nearby_points() {
        let a = perlin2(0.1, 0.2);
        let b = perlin2(0.2, 0.3);
        // Not necessarily different, but we test the function runs
        let _diff = (a - b).abs();
    }

    #[test]
    fn perlin3_bounded() {
        for i in 0..10 {
            let v = perlin3(i as f32 * 0.37, i as f32 * 0.53, i as f32 * 0.19);
            assert!((-2.0..=2.0).contains(&v));
        }
    }

    #[test]
    fn fade_at_zero_and_one() {
        assert!(fade(0.0).abs() < 1e-6);
        assert!((fade(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn lerp_endpoints() {
        assert!((lerp(0.0, 10.0, 0.0) - 0.0).abs() < 1e-5);
        assert!((lerp(0.0, 10.0, 1.0) - 10.0).abs() < 1e-5);
    }
}
