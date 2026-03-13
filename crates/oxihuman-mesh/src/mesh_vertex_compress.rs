#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Vertex data compression/decompression utilities.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexCompress {
    pub data: Vec<u8>,
    pub original_count: usize,
    pub format: String,
}

#[allow(dead_code)]
pub fn compress_positions(positions: &[[f32; 3]]) -> Vec<i16> {
    let mut result = Vec::with_capacity(positions.len() * 3);
    for p in positions {
        for &v in p {
            result.push((v.clamp(-32.0, 32.0) * 1000.0) as i16);
        }
    }
    result
}

#[allow(dead_code)]
pub fn compress_normals(normals: &[[f32; 3]]) -> Vec<i8> {
    let mut result = Vec::with_capacity(normals.len() * 3);
    for n in normals {
        for &v in n {
            result.push((v.clamp(-1.0, 1.0) * 127.0) as i8);
        }
    }
    result
}

#[allow(dead_code)]
pub fn compress_uvs(uvs: &[[f32; 2]]) -> Vec<u16> {
    let mut result = Vec::with_capacity(uvs.len() * 2);
    for uv in uvs {
        for &v in uv {
            result.push((v.clamp(0.0, 1.0) * 65535.0) as u16);
        }
    }
    result
}

#[allow(dead_code)]
pub fn decompress_positions(compressed: &[i16]) -> Vec<[f32; 3]> {
    let mut result = Vec::with_capacity(compressed.len() / 3);
    for chunk in compressed.chunks(3) {
        if chunk.len() == 3 {
            result.push([
                chunk[0] as f32 / 1000.0,
                chunk[1] as f32 / 1000.0,
                chunk[2] as f32 / 1000.0,
            ]);
        }
    }
    result
}

#[allow(dead_code)]
pub fn decompress_normals(compressed: &[i8]) -> Vec<[f32; 3]> {
    let mut result = Vec::with_capacity(compressed.len() / 3);
    for chunk in compressed.chunks(3) {
        if chunk.len() == 3 {
            result.push([
                chunk[0] as f32 / 127.0,
                chunk[1] as f32 / 127.0,
                chunk[2] as f32 / 127.0,
            ]);
        }
    }
    result
}

#[allow(dead_code)]
pub fn compress_ratio_vc(original_bytes: usize, compressed_bytes: usize) -> f32 {
    if original_bytes == 0 {
        return 0.0;
    }
    compressed_bytes as f32 / original_bytes as f32
}

#[allow(dead_code)]
pub fn compressed_size_vc(positions: &[[f32; 3]], normals: &[[f32; 3]], uvs: &[[f32; 2]]) -> usize {
    positions.len() * 3 * 2 + normals.len() * 3 + uvs.len() * 2 * 2
}

#[allow(dead_code)]
pub fn validate_compressed(compressed_pos: &[i16], compressed_nrm: &[i8]) -> bool {
    compressed_pos.len().is_multiple_of(3) && compressed_nrm.len().is_multiple_of(3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_positions() {
        let pos = vec![[1.0, 2.0, 3.0]];
        let c = compress_positions(&pos);
        assert_eq!(c.len(), 3);
        assert_eq!(c[0], 1000);
    }

    #[test]
    fn test_decompress_positions() {
        let pos = vec![[1.0, 2.0, 3.0]];
        let c = compress_positions(&pos);
        let d = decompress_positions(&c);
        assert!((d[0][0] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_compress_normals() {
        let nrm = vec![[0.0, 1.0, 0.0]];
        let c = compress_normals(&nrm);
        assert_eq!(c.len(), 3);
        assert_eq!(c[1], 127);
    }

    #[test]
    fn test_decompress_normals() {
        let nrm = vec![[0.0, 1.0, 0.0]];
        let c = compress_normals(&nrm);
        let d = decompress_normals(&c);
        assert!((d[0][1] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_compress_uvs() {
        let uvs = vec![[0.5, 0.5]];
        let c = compress_uvs(&uvs);
        assert_eq!(c.len(), 2);
    }

    #[test]
    fn test_compress_ratio() {
        let r = compress_ratio_vc(100, 50);
        assert!((r - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_compress_ratio_zero() {
        assert!((compress_ratio_vc(0, 50)).abs() < 1e-6);
    }

    #[test]
    fn test_compressed_size() {
        let s = compressed_size_vc(&[[0.0; 3]], &[[0.0; 3]], &[[0.0; 2]]);
        assert!(s > 0);
    }

    #[test]
    fn test_validate_compressed() {
        assert!(validate_compressed(&[0, 0, 0], &[0, 0, 0]));
        assert!(!validate_compressed(&[0, 0], &[0, 0, 0]));
    }

    #[test]
    fn test_clamp_positions() {
        let pos = vec![[100.0, -100.0, 0.0]];
        let c = compress_positions(&pos);
        let d = decompress_positions(&c);
        assert!((d[0][0] - 32.0).abs() < 0.1);
    }
}
