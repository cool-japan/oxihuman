// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

fn spread_2d(mut x: u32) -> u32 {
    x &= 0x0000_FFFF;
    x = (x | (x << 8)) & 0x00FF_00FF;
    x = (x | (x << 4)) & 0x0F0F_0F0F;
    x = (x | (x << 2)) & 0x3333_3333;
    x = (x | (x << 1)) & 0x5555_5555;
    x
}

fn compact_2d(mut x: u32) -> u32 {
    x &= 0x5555_5555;
    x = (x | (x >> 1)) & 0x3333_3333;
    x = (x | (x >> 2)) & 0x0F0F_0F0F;
    x = (x | (x >> 4)) & 0x00FF_00FF;
    x = (x | (x >> 8)) & 0x0000_FFFF;
    x
}

pub fn morton_encode_2d(x: u16, y: u16) -> u32 {
    spread_2d(x as u32) | (spread_2d(y as u32) << 1)
}

pub fn morton_decode_2d(m: u32) -> (u16, u16) {
    (compact_2d(m) as u16, compact_2d(m >> 1) as u16)
}

/* clean 3-D Morton using bit interleaving */
fn interleave3(mut n: u64) -> u64 {
    n &= 0x1FFFFF;
    n = (n | n << 32) & 0x1F00000000FFFF;
    n = (n | n << 16) & 0x1F0000FF0000FF;
    n = (n | n << 8) & 0x100F00F00F00F00F;
    n = (n | n << 4) & 0x10C30C30C30C30C3;
    n = (n | n << 2) & 0x1249249249249249;
    n
}

fn deinterleave3(mut n: u64) -> u64 {
    n &= 0x1249249249249249;
    n = (n | n >> 2) & 0x10C30C30C30C30C3;
    n = (n | n >> 4) & 0x100F00F00F00F00F;
    n = (n | n >> 8) & 0x1F0000FF0000FF;
    n = (n | n >> 16) & 0x1F00000000FFFF;
    n = (n | n >> 32) & 0x1FFFFF;
    n
}

pub fn morton_encode_3d(x: u16, y: u16, z: u16) -> u64 {
    interleave3(x as u64) | (interleave3(y as u64) << 1) | (interleave3(z as u64) << 2)
}

pub fn morton_decode_3d(m: u64) -> (u16, u16, u16) {
    (
        deinterleave3(m) as u16,
        deinterleave3(m >> 1) as u16,
        deinterleave3(m >> 2) as u16,
    )
}

pub fn morton_neighbor(m: u32, dx: i32, dy: i32) -> u32 {
    let (x, y) = morton_decode_2d(m);
    let nx = (x as i32).wrapping_add(dx) as u16;
    let ny = (y as i32).wrapping_add(dy) as u16;
    morton_encode_2d(nx, ny)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_origin_2d() {
        /* (0,0) encodes to 0 */
        assert_eq!(morton_encode_2d(0, 0), 0);
    }

    #[test]
    fn roundtrip_2d() {
        /* encode-decode roundtrip */
        let (x, y) = (5u16, 9u16);
        let m = morton_encode_2d(x, y);
        assert_eq!(morton_decode_2d(m), (x, y));
    }

    #[test]
    fn encode_origin_3d() {
        /* (0,0,0) encodes to 0 */
        assert_eq!(morton_encode_3d(0, 0, 0), 0);
    }

    #[test]
    fn roundtrip_3d() {
        /* encode-decode roundtrip 3D */
        let (x, y, z) = (3u16, 5u16, 7u16);
        let m = morton_encode_3d(x, y, z);
        assert_eq!(morton_decode_3d(m), (x, y, z));
    }

    #[test]
    fn neighbor_dx() {
        /* neighbor with dx=1 advances x by 1 */
        let m = morton_encode_2d(2, 3);
        let n = morton_neighbor(m, 1, 0);
        assert_eq!(morton_decode_2d(n), (3, 3));
    }

    #[test]
    fn neighbor_dy() {
        /* neighbor with dy=1 advances y by 1 */
        let m = morton_encode_2d(2, 3);
        let n = morton_neighbor(m, 0, 1);
        assert_eq!(morton_decode_2d(n), (2, 4));
    }

    #[test]
    fn x1_encodes_correctly_2d() {
        /* x=1,y=0 → bit 0 set */
        assert_eq!(morton_encode_2d(1, 0), 1);
    }
}
