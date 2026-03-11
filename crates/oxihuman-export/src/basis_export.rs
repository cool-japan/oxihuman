// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]

/// Basis vector export for mesh coordinate frames.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BasisExport {
    pub tangents: Vec<[f32; 3]>,
    pub bitangents: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
}

#[allow(dead_code)]
impl BasisExport {
    /// Create from separate basis vectors.
    pub fn new(tangents: Vec<[f32; 3]>, bitangents: Vec<[f32; 3]>, normals: Vec<[f32; 3]>) -> Self {
        Self { tangents, bitangents, normals }
    }

    /// Number of basis frames.
    pub fn count(&self) -> usize {
        self.normals.len()
    }

    /// Check if all vectors are normalized.
    pub fn is_normalized(&self) -> bool {
        self.normals.iter().all(|n| {
            let len = (n[0]*n[0]+n[1]*n[1]+n[2]*n[2]).sqrt();
            (len - 1.0).abs() < 0.01
        })
    }

    /// Export to bytes (flat f32 layout: T, B, N per vertex).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for i in 0..self.count() {
            for v in [&self.tangents[i], &self.bitangents[i], &self.normals[i]] {
                for &f in v {
                    bytes.extend_from_slice(&f.to_le_bytes());
                }
            }
        }
        bytes
    }

    /// Size in bytes.
    pub fn byte_size(&self) -> usize {
        self.count() * 9 * 4
    }

    /// Validate orthogonality.
    pub fn is_orthogonal(&self) -> bool {
        for i in 0..self.count() {
            let dot = self.tangents[i][0]*self.normals[i][0]
                + self.tangents[i][1]*self.normals[i][1]
                + self.tangents[i][2]*self.normals[i][2];
            if dot.abs() > 0.01 { return false; }
        }
        true
    }
}

/// Export basis vectors to JSON string.
#[allow(dead_code)]
pub fn export_basis_json(basis: &BasisExport) -> String {
    format!(
        "{{\"count\":{},\"byte_size\":{},\"normalized\":{},\"orthogonal\":{}}}",
        basis.count(), basis.byte_size(), basis.is_normalized(), basis.is_orthogonal()
    )
}

/// Create identity basis (all Z-up).
#[allow(dead_code)]
pub fn identity_basis(count: usize) -> BasisExport {
    BasisExport {
        tangents: vec![[1.0, 0.0, 0.0]; count],
        bitangents: vec![[0.0, 1.0, 0.0]; count],
        normals: vec![[0.0, 0.0, 1.0]; count],
    }
}

/// Validate basis export data.
#[allow(dead_code)]
pub fn validate_basis(basis: &BasisExport) -> bool {
    basis.tangents.len() == basis.normals.len() && basis.bitangents.len() == basis.normals.len()
}

/// Compute basis from normals (tangent from cross with up).
#[allow(dead_code)]
pub fn basis_from_normals(normals: &[[f32; 3]]) -> BasisExport {
    let up = [0.0f32, 1.0, 0.0];
    let mut tangents = Vec::new();
    let mut bitangents = Vec::new();
    for n in normals {
        let t = cross(&up, n);
        let len = (t[0]*t[0]+t[1]*t[1]+t[2]*t[2]).sqrt();
        let t = if len > 1e-6 { [t[0]/len, t[1]/len, t[2]/len] } else { [1.0, 0.0, 0.0] };
        let b = cross(n, &t);
        tangents.push(t);
        bitangents.push(b);
    }
    BasisExport { tangents, bitangents, normals: normals.to_vec() }
}

fn cross(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_basis() {
        let b = identity_basis(3);
        assert_eq!(b.count(), 3);
        assert!(b.is_normalized());
    }

    #[test]
    fn test_orthogonal() {
        let b = identity_basis(2);
        assert!(b.is_orthogonal());
    }

    #[test]
    fn test_byte_size() {
        let b = identity_basis(1);
        assert_eq!(b.byte_size(), 36);
    }

    #[test]
    fn test_to_bytes() {
        let b = identity_basis(1);
        let bytes = b.to_bytes();
        assert_eq!(bytes.len(), 36);
    }

    #[test]
    fn test_validate() {
        let b = identity_basis(2);
        assert!(validate_basis(&b));
    }

    #[test]
    fn test_from_normals() {
        let normals = vec![[0.0, 0.0, 1.0]];
        let b = basis_from_normals(&normals);
        assert_eq!(b.count(), 1);
    }

    #[test]
    fn test_export_json() {
        let b = identity_basis(1);
        let json = export_basis_json(&b);
        assert!(json.contains("count"));
    }

    #[test]
    fn test_empty() {
        let b = identity_basis(0);
        assert_eq!(b.count(), 0);
    }

    #[test]
    fn test_not_normalized() {
        let b = BasisExport::new(
            vec![[2.0, 0.0, 0.0]],
            vec![[0.0, 2.0, 0.0]],
            vec![[0.0, 0.0, 2.0]],
        );
        assert!(!b.is_normalized());
    }

    #[test]
    fn test_validate_mismatched() {
        let b = BasisExport::new(vec![[1.0,0.0,0.0]], vec![], vec![[0.0,0.0,1.0]]);
        assert!(!validate_basis(&b));
    }
}
