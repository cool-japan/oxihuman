#![allow(dead_code)]
//! Export normal data.

/// Normal export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct NormalExport {
    pub normals: Vec<[f32; 3]>,
}

/// Export normals from mesh data.
#[allow(dead_code)]
pub fn export_normals(normals: &[[f32; 3]]) -> NormalExport {
    NormalExport { normals: normals.to_vec() }
}

/// Return the normal format string.
#[allow(dead_code)]
pub fn normal_format() -> &'static str {
    "FLOAT3"
}

/// Convert normals to bytes.
#[allow(dead_code)]
pub fn normal_to_bytes(export: &NormalExport) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(export.normals.len() * 12);
    for n in &export.normals {
        for &v in n {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
    }
    bytes
}

/// Return normal count.
#[allow(dead_code)]
pub fn normal_count_export(export: &NormalExport) -> usize {
    export.normals.len()
}

/// Encode normal to octahedral 16-bit representation.
#[allow(dead_code)]
pub fn normal_to_oct16(n: [f32; 3]) -> [i16; 2] {
    let inv_l1 = 1.0 / (n[0].abs() + n[1].abs() + n[2].abs()).max(1e-12);
    let mut ox = n[0] * inv_l1;
    let mut oy = n[1] * inv_l1;
    if n[2] < 0.0 {
        let new_ox = (1.0 - oy.abs()) * if ox >= 0.0 { 1.0 } else { -1.0 };
        let new_oy = (1.0 - ox.abs()) * if oy >= 0.0 { 1.0 } else { -1.0 };
        ox = new_ox;
        oy = new_oy;
    }
    [
        (ox.clamp(-1.0, 1.0) * 32767.0) as i16,
        (oy.clamp(-1.0, 1.0) * 32767.0) as i16,
    ]
}

/// Decode octahedral 16-bit to normal.
#[allow(dead_code)]
pub fn oct16_to_normal(oct: [i16; 2]) -> [f32; 3] {
    let ox = oct[0] as f32 / 32767.0;
    let oy = oct[1] as f32 / 32767.0;
    let oz = 1.0 - ox.abs() - oy.abs();
    let (fx, fy) = if oz < 0.0 {
        (
            (1.0 - oy.abs()) * if ox >= 0.0 { 1.0 } else { -1.0 },
            (1.0 - ox.abs()) * if oy >= 0.0 { 1.0 } else { -1.0 },
        )
    } else {
        (ox, oy)
    };
    let len = (fx * fx + fy * fy + oz * oz).sqrt();
    if len < 1e-12 {
        [0.0, 0.0, 1.0]
    } else {
        [fx / len, fy / len, oz / len]
    }
}

/// Validate that all normals are unit length.
#[allow(dead_code)]
pub fn validate_normals(export: &NormalExport) -> bool {
    export.normals.iter().all(|n| {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        (len - 1.0).abs() < 0.01
    })
}

/// Return the export size in bytes.
#[allow(dead_code)]
pub fn normal_export_size(export: &NormalExport) -> usize {
    export.normals.len() * 12
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_normals() {
        let n = vec![[0.0, 0.0, 1.0]];
        let exp = export_normals(&n);
        assert_eq!(normal_count_export(&exp), 1);
    }

    #[test]
    fn test_normal_format() {
        assert_eq!(normal_format(), "FLOAT3");
    }

    #[test]
    fn test_normal_to_bytes() {
        let exp = export_normals(&[[0.0, 0.0, 1.0]]);
        let bytes = normal_to_bytes(&exp);
        assert_eq!(bytes.len(), 12);
    }

    #[test]
    fn test_normal_count() {
        let exp = export_normals(&[[0.0; 3]; 5]);
        assert_eq!(normal_count_export(&exp), 5);
    }

    #[test]
    fn test_oct16_roundtrip() {
        let n = [0.0f32, 0.0, 1.0];
        let oct = normal_to_oct16(n);
        let decoded = oct16_to_normal(oct);
        assert!((decoded[2] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_oct16_roundtrip_x() {
        let n = [1.0f32, 0.0, 0.0];
        let oct = normal_to_oct16(n);
        let decoded = oct16_to_normal(oct);
        assert!((decoded[0] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_validate_normals() {
        let exp = export_normals(&[[0.0, 0.0, 1.0], [1.0, 0.0, 0.0]]);
        assert!(validate_normals(&exp));
    }

    #[test]
    fn test_validate_normals_bad() {
        let exp = export_normals(&[[0.0, 0.0, 2.0]]);
        assert!(!validate_normals(&exp));
    }

    #[test]
    fn test_normal_export_size() {
        let exp = export_normals(&[[0.0; 3]; 3]);
        assert_eq!(normal_export_size(&exp), 36);
    }

    #[test]
    fn test_empty_normals() {
        let exp = export_normals(&[]);
        assert_eq!(normal_count_export(&exp), 0);
        assert!(validate_normals(&exp));
    }
}
