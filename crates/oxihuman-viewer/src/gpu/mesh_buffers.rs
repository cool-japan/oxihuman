// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! CPU-side mesh buffer ready to be uploaded to the GPU.

/// CPU-side mesh buffer ready to be uploaded to the GPU.
///
/// Binary layout expected by [`MeshUploadBuffer::from_raw_bytes`]:
/// ```text
/// [version: u32][n_verts: u32][n_idx: u32]
/// [positions: n_verts x 3 x f32]
/// [normals:   n_verts x 3 x f32]
/// [uvs:       n_verts x 2 x f32]
/// [indices:   n_idx  x u32]
/// ```
#[derive(Debug, Clone, Default)]
pub struct MeshUploadBuffer {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    /// Monotonically increasing timestamp (e.g. frame count or epoch ms).
    pub timestamp: u64,
}

impl MeshUploadBuffer {
    /// Parse the WasmEngine binary buffer format.
    ///
    /// Returns `None` if the buffer is too short or the version is unsupported.
    pub fn from_raw_bytes(bytes: &[u8]) -> Option<Self> {
        const SUPPORTED_VERSION: u32 = 1;
        if bytes.len() < 12 {
            return None;
        }

        let version = read_u32_le(bytes, 0)?;
        if version != SUPPORTED_VERSION {
            return None;
        }
        let n_verts = read_u32_le(bytes, 4)? as usize;
        let n_idx = read_u32_le(bytes, 8)? as usize;

        let pos_bytes = n_verts * 3 * 4;
        let nrm_bytes = n_verts * 3 * 4;
        let uv_bytes = n_verts * 2 * 4;
        let idx_bytes = n_idx * 4;
        let total_needed = 12 + pos_bytes + nrm_bytes + uv_bytes + idx_bytes;

        if bytes.len() < total_needed {
            return None;
        }

        let mut offset = 12usize;

        let mut positions = Vec::with_capacity(n_verts);
        for _ in 0..n_verts {
            let x = f32::from_le_bytes(bytes[offset..offset + 4].try_into().ok()?);
            let y = f32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().ok()?);
            let z = f32::from_le_bytes(bytes[offset + 8..offset + 12].try_into().ok()?);
            positions.push([x, y, z]);
            offset += 12;
        }

        let mut normals = Vec::with_capacity(n_verts);
        for _ in 0..n_verts {
            let x = f32::from_le_bytes(bytes[offset..offset + 4].try_into().ok()?);
            let y = f32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().ok()?);
            let z = f32::from_le_bytes(bytes[offset + 8..offset + 12].try_into().ok()?);
            normals.push([x, y, z]);
            offset += 12;
        }

        let mut uvs = Vec::with_capacity(n_verts);
        for _ in 0..n_verts {
            let u = f32::from_le_bytes(bytes[offset..offset + 4].try_into().ok()?);
            let v = f32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().ok()?);
            uvs.push([u, v]);
            offset += 8;
        }

        let mut indices = Vec::with_capacity(n_idx);
        for _ in 0..n_idx {
            let i = read_u32_le(bytes, offset)?;
            indices.push(i);
            offset += 4;
        }

        Some(MeshUploadBuffer {
            positions,
            normals,
            uvs,
            indices,
            timestamp: 0,
        })
    }

    /// Returns `true` if the buffer contains at least one triangle and all indices are in range.
    pub fn is_valid(&self) -> bool {
        if self.positions.is_empty() || self.indices.len() < 3 {
            return false;
        }
        let n = self.positions.len() as u32;
        self.indices.iter().all(|&i| i < n)
    }
}

fn read_u32_le(bytes: &[u8], offset: usize) -> Option<u32> {
    let b: [u8; 4] = bytes.get(offset..offset + 4)?.try_into().ok()?;
    Some(u32::from_le_bytes(b))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_valid_buffer_bytes() -> Vec<u8> {
        let mut b = Vec::<u8>::new();
        b.extend_from_slice(&1u32.to_le_bytes()); // version
        b.extend_from_slice(&3u32.to_le_bytes()); // n_verts
        b.extend_from_slice(&3u32.to_le_bytes()); // n_idx
                                                  // positions: 3 verts x 3 floats
        for i in 0u32..9 {
            b.extend_from_slice(&(i as f32).to_le_bytes());
        }
        // normals: 3 verts x 3 floats
        for _ in 0..9 {
            b.extend_from_slice(&0f32.to_le_bytes());
        }
        // uvs: 3 verts x 2 floats
        for _ in 0..6 {
            b.extend_from_slice(&0f32.to_le_bytes());
        }
        // indices: 0, 1, 2
        for i in 0u32..3 {
            b.extend_from_slice(&i.to_le_bytes());
        }
        b
    }

    #[test]
    fn mesh_upload_buffer_parse_valid() {
        let bytes = make_valid_buffer_bytes();
        let buf = MeshUploadBuffer::from_raw_bytes(&bytes).expect("should parse valid bytes");
        assert_eq!(buf.positions.len(), 3);
        assert_eq!(buf.normals.len(), 3);
        assert_eq!(buf.uvs.len(), 3);
        assert_eq!(buf.indices, vec![0, 1, 2]);
    }

    #[test]
    fn mesh_upload_buffer_parse_too_short() {
        assert!(MeshUploadBuffer::from_raw_bytes(&[0u8; 5]).is_none());
    }

    #[test]
    fn mesh_upload_buffer_parse_wrong_version() {
        let mut bytes = make_valid_buffer_bytes();
        bytes[0] = 99;
        assert!(MeshUploadBuffer::from_raw_bytes(&bytes).is_none());
    }

    #[test]
    fn mesh_upload_buffer_is_valid_true() {
        let bytes = make_valid_buffer_bytes();
        let buf = MeshUploadBuffer::from_raw_bytes(&bytes).expect("should succeed");
        assert!(buf.is_valid());
    }

    #[test]
    fn mesh_upload_buffer_is_valid_empty() {
        let buf = MeshUploadBuffer::default();
        assert!(!buf.is_valid());
    }

    #[test]
    fn mesh_upload_buffer_is_valid_out_of_range_index() {
        let bytes = make_valid_buffer_bytes();
        let mut buf = MeshUploadBuffer::from_raw_bytes(&bytes).expect("should succeed");
        buf.indices.push(999);
        assert!(!buf.is_valid());
    }
}
