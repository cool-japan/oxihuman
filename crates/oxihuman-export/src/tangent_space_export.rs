#![allow(dead_code)]
//! Export tangent space data.

/// Tangent space export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct TangentSpaceExport {
    pub tangents: Vec<[f32; 4]>,
}

/// Export tangent space for a mesh.
#[allow(dead_code)]
pub fn export_tangent_space(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    uvs: &[[f32; 2]],
    indices: &[[u32; 3]],
) -> TangentSpaceExport {
    let mut tangents = vec![[1.0f32, 0.0, 0.0, 1.0]; positions.len()];
    for tri in indices {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;
        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
            continue;
        }
        if i0 >= uvs.len() || i1 >= uvs.len() || i2 >= uvs.len() {
            continue;
        }
        let dp1 = [positions[i1][0] - positions[i0][0], positions[i1][1] - positions[i0][1], positions[i1][2] - positions[i0][2]];
        let dp2 = [positions[i2][0] - positions[i0][0], positions[i2][1] - positions[i0][1], positions[i2][2] - positions[i0][2]];
        let duv1 = [uvs[i1][0] - uvs[i0][0], uvs[i1][1] - uvs[i0][1]];
        let duv2 = [uvs[i2][0] - uvs[i0][0], uvs[i2][1] - uvs[i0][1]];
        let r = duv1[0] * duv2[1] - duv1[1] * duv2[0];
        if r.abs() < 1e-12 {
            continue;
        }
        let inv_r = 1.0 / r;
        let t = [
            inv_r * (duv2[1] * dp1[0] - duv1[1] * dp2[0]),
            inv_r * (duv2[1] * dp1[1] - duv1[1] * dp2[1]),
            inv_r * (duv2[1] * dp1[2] - duv1[1] * dp2[2]),
        ];
        let len = (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt();
        if len > 1e-12 {
            let tn = [t[0] / len, t[1] / len, t[2] / len];
            // Compute bitangent sign
            let n = normals.get(i0).copied().unwrap_or([0.0, 0.0, 1.0]);
            let bt = [
                n[1] * tn[2] - n[2] * tn[1],
                n[2] * tn[0] - n[0] * tn[2],
                n[0] * tn[1] - n[1] * tn[0],
            ];
            let b_expected = [
                inv_r * (duv1[0] * dp2[0] - duv2[0] * dp1[0]),
                inv_r * (duv1[0] * dp2[1] - duv2[0] * dp1[1]),
                inv_r * (duv1[0] * dp2[2] - duv2[0] * dp1[2]),
            ];
            let sign = if bt[0] * b_expected[0] + bt[1] * b_expected[1] + bt[2] * b_expected[2] < 0.0 {
                -1.0
            } else {
                1.0
            };
            for &vi in tri {
                tangents[vi as usize] = [tn[0], tn[1], tn[2], sign];
            }
        }
    }
    TangentSpaceExport { tangents }
}

/// Return the number of tangents.
#[allow(dead_code)]
pub fn tangent_count(export: &TangentSpaceExport) -> usize {
    export.tangents.len()
}

/// Convert tangents to bytes.
#[allow(dead_code)]
pub fn tangent_to_bytes(export: &TangentSpaceExport) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(export.tangents.len() * 16);
    for t in &export.tangents {
        for &v in t {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
    }
    bytes
}

/// Return the tangent format string.
#[allow(dead_code)]
pub fn tangent_format() -> &'static str {
    "FLOAT4"
}

/// Compute bitangent sign from a tangent.
#[allow(dead_code)]
pub fn bitangent_sign(tangent: [f32; 4]) -> f32 {
    tangent[3]
}

/// Validate that tangents are unit length.
#[allow(dead_code)]
pub fn validate_tangents(export: &TangentSpaceExport) -> bool {
    export.tangents.iter().all(|t| {
        let len = (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt();
        (len - 1.0).abs() < 0.1
    })
}

/// Return the export size in bytes.
#[allow(dead_code)]
pub fn tangent_export_size(export: &TangentSpaceExport) -> usize {
    export.tangents.len() * 16
}

/// Convert tangent data to a JSON string.
#[allow(dead_code)]
pub fn tangent_to_json(export: &TangentSpaceExport) -> String {
    format!("{{\"tangent_count\":{},\"format\":\"FLOAT4\"}}", export.tangents.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::type_complexity)]
    fn sample() -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<[u32; 3]>) {
        let p = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let n = vec![[0.0, 0.0, 1.0]; 3];
        let uv = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let i = vec![[0u32, 1, 2]];
        (p, n, uv, i)
    }

    #[test]
    fn test_export_tangent_space() {
        let (p, n, uv, i) = sample();
        let exp = export_tangent_space(&p, &n, &uv, &i);
        assert_eq!(tangent_count(&exp), 3);
    }

    #[test]
    fn test_tangent_to_bytes() {
        let (p, n, uv, i) = sample();
        let exp = export_tangent_space(&p, &n, &uv, &i);
        let bytes = tangent_to_bytes(&exp);
        assert_eq!(bytes.len(), 48);
    }

    #[test]
    fn test_tangent_format() {
        assert_eq!(tangent_format(), "FLOAT4");
    }

    #[test]
    fn test_bitangent_sign() {
        assert!((bitangent_sign([1.0, 0.0, 0.0, -1.0]) + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_validate_tangents() {
        let (p, n, uv, i) = sample();
        let exp = export_tangent_space(&p, &n, &uv, &i);
        assert!(validate_tangents(&exp));
    }

    #[test]
    fn test_tangent_export_size() {
        let (p, n, uv, i) = sample();
        let exp = export_tangent_space(&p, &n, &uv, &i);
        assert_eq!(tangent_export_size(&exp), 48);
    }

    #[test]
    fn test_tangent_to_json() {
        let (p, n, uv, i) = sample();
        let exp = export_tangent_space(&p, &n, &uv, &i);
        let j = tangent_to_json(&exp);
        assert!(j.contains("tangent_count"));
    }

    #[test]
    fn test_tangent_count_empty() {
        let exp = TangentSpaceExport { tangents: vec![] };
        assert_eq!(tangent_count(&exp), 0);
    }

    #[test]
    fn test_validate_empty_tangents() {
        let exp = TangentSpaceExport { tangents: vec![] };
        assert!(validate_tangents(&exp));
    }

    #[test]
    fn test_tangent_export_empty_size() {
        let exp = TangentSpaceExport { tangents: vec![] };
        assert_eq!(tangent_export_size(&exp), 0);
    }
}
