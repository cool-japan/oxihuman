// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub fn hilbert_order_for_size(grid_size: u32) -> u32 {
    if grid_size <= 1 {
        return 0;
    }
    let mut n = grid_size - 1;
    let mut order = 0u32;
    while n > 0 {
        n >>= 1;
        order += 1;
    }
    order
}

pub fn hilbert_max_index(n: u32) -> u64 {
    if n == 0 {
        return 0;
    }
    (n as u64) * (n as u64) - 1
}

pub fn hilbert_xy_to_d(n: u32, mut x: u32, mut y: u32) -> u64 {
    let mut d = 0u64;
    let mut s = n / 2;
    while s > 0 {
        let rx = if x & s > 0 { 1u32 } else { 0 };
        let ry = if y & s > 0 { 1u32 } else { 0 };
        d += (s as u64) * (s as u64) * ((3 * rx) ^ ry) as u64;
        /* rotate */
        if ry == 0 {
            if rx == 1 {
                x = s.wrapping_sub(1).wrapping_sub(x);
                y = s.wrapping_sub(1).wrapping_sub(y);
            }
            std::mem::swap(&mut x, &mut y);
        }
        s /= 2;
    }
    d
}

pub fn hilbert_d_to_xy(n: u32, mut d: u64) -> (u32, u32) {
    let mut x = 0u32;
    let mut y = 0u32;
    let mut s = 1u32;
    while s < n {
        let rx = if (d & 2) > 0 { 1u32 } else { 0 };
        let ry = if (d & 1) ^ (rx as u64) > 0 { 1u32 } else { 0 };
        if ry == 0 {
            if rx == 1 {
                x = s.wrapping_sub(1).wrapping_sub(x);
                y = s.wrapping_sub(1).wrapping_sub(y);
            }
            std::mem::swap(&mut x, &mut y);
        }
        x += s * rx;
        y += s * ry;
        d >>= 2;
        s *= 2;
    }
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_is_zero() {
        /* (0,0) maps to curve index 0 */
        assert_eq!(hilbert_xy_to_d(4, 0, 0), 0);
    }

    #[test]
    fn roundtrip() {
        /* encode then decode returns (x,y) */
        let n = 4u32;
        for d in 0..16u64 {
            let (x, y) = hilbert_d_to_xy(n, d);
            let d2 = hilbert_xy_to_d(n, x, y);
            assert_eq!(d, d2, "roundtrip failed at d={d}");
        }
    }

    #[test]
    fn max_index() {
        /* max index for 4x4 grid is 15 */
        assert_eq!(hilbert_max_index(4), 15);
    }

    #[test]
    fn order_for_size_1() {
        /* grid 1 → order 0 */
        assert_eq!(hilbert_order_for_size(1), 0);
    }

    #[test]
    fn order_for_size_4() {
        /* grid 4 → order 2 */
        assert_eq!(hilbert_order_for_size(4), 2);
    }

    #[test]
    fn order_for_size_8() {
        /* grid 8 → order 3 */
        assert_eq!(hilbert_order_for_size(8), 3);
    }
}
