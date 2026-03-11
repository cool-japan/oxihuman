// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Ptex per-face texture coordinate storage stub.

/// Per-face Ptex resolution descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PtexFaceRes {
    pub log2_u: u8,
    pub log2_v: u8,
}

impl PtexFaceRes {
    /// Create a new resolution descriptor.
    pub fn new(log2_u: u8, log2_v: u8) -> Self {
        Self { log2_u, log2_v }
    }

    /// Return u resolution (number of texels in u direction).
    pub fn res_u(&self) -> u32 {
        1 << self.log2_u as u32
    }

    /// Return v resolution (number of texels in v direction).
    pub fn res_v(&self) -> u32 {
        1 << self.log2_v as u32
    }

    /// Return total texel count for this face.
    pub fn total_texels(&self) -> u32 {
        self.res_u() * self.res_v()
    }
}

/// Ptex data for a single face (RGBA per texel, stub storage).
#[derive(Debug, Clone)]
pub struct PtexFaceData {
    pub res: PtexFaceRes,
    pub texels: Vec<[u8; 4]>,
}

impl PtexFaceData {
    /// Create a new face data block filled with a constant color.
    pub fn new_filled(res: PtexFaceRes, color: [u8; 4]) -> Self {
        let n = res.total_texels() as usize;
        Self {
            res,
            texels: vec![color; n],
        }
    }

    /// Sample a texel at integer coordinates (clamp to border).
    pub fn sample(&self, u: u32, v: u32) -> [u8; 4] {
        let u = u.min(self.res.res_u().saturating_sub(1));
        let v = v.min(self.res.res_v().saturating_sub(1));
        let idx = (v * self.res.res_u() + u) as usize;
        self.texels.get(idx).copied().unwrap_or([0; 4])
    }
}

/// Ptex texture for a mesh: one PtexFaceData per face.
#[derive(Debug, Clone)]
pub struct PtexTexture {
    pub faces: Vec<PtexFaceData>,
}

impl PtexTexture {
    /// Create a new Ptex texture with the given per-face resolutions.
    pub fn new(resolutions: &[PtexFaceRes]) -> Self {
        let faces = resolutions
            .iter()
            .map(|&r| PtexFaceData::new_filled(r, [128, 128, 128, 255]))
            .collect();
        Self { faces }
    }

    /// Return the number of faces.
    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    /// Total texels across all faces.
    pub fn total_texels(&self) -> u64 {
        self.faces.iter().map(|f| f.texels.len() as u64).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn basic_res() -> PtexFaceRes {
        PtexFaceRes::new(2, 2) /* 4x4 = 16 texels */
    }

    #[test]
    fn test_res_u_v() {
        /* log2_u=2 should give res_u=4 */
        let r = basic_res();
        assert_eq!(r.res_u(), 4);
        assert_eq!(r.res_v(), 4);
    }

    #[test]
    fn test_total_texels() {
        /* 4x4 face has 16 texels */
        let r = basic_res();
        assert_eq!(r.total_texels(), 16);
    }

    #[test]
    fn test_face_data_fill() {
        /* face data fills with given color */
        let fd = PtexFaceData::new_filled(basic_res(), [255, 0, 0, 255]);
        assert_eq!(fd.texels.len(), 16);
        assert_eq!(fd.texels[0], [255, 0, 0, 255]);
    }

    #[test]
    fn test_sample_in_bounds() {
        /* sampling returns correct texel */
        let fd = PtexFaceData::new_filled(basic_res(), [10, 20, 30, 255]);
        assert_eq!(fd.sample(1, 1), [10, 20, 30, 255]);
    }

    #[test]
    fn test_sample_clamped() {
        /* sampling out-of-bounds clamps to border */
        let fd = PtexFaceData::new_filled(basic_res(), [5, 5, 5, 255]);
        let clamped = fd.sample(100, 100);
        assert_eq!(clamped, [5, 5, 5, 255]);
    }

    #[test]
    fn test_ptex_texture_face_count() {
        /* texture has correct face count */
        let res = vec![basic_res(), basic_res(), PtexFaceRes::new(1, 1)];
        let tex = PtexTexture::new(&res);
        assert_eq!(tex.face_count(), 3);
    }

    #[test]
    fn test_ptex_texture_total_texels() {
        /* total texels sums correctly */
        let res = vec![PtexFaceRes::new(2, 2), PtexFaceRes::new(1, 1)];
        let tex = PtexTexture::new(&res);
        assert_eq!(tex.total_texels(), 16 + 4);
    }

    #[test]
    fn test_empty_texture() {
        /* empty texture has zero faces */
        let tex = PtexTexture::new(&[]);
        assert_eq!(tex.face_count(), 0);
        assert_eq!(tex.total_texels(), 0);
    }
}
