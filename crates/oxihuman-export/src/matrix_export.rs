#![allow(dead_code)]
//! Export matrix data.

/// Matrix export data (4x4 column-major).
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct MatrixExport {
    pub data: [f32; 16],
}

/// Export a 4x4 matrix.
#[allow(dead_code)]
pub fn export_matrix4(data: [f32; 16]) -> MatrixExport {
    MatrixExport { data }
}

/// Convert matrix to bytes.
#[allow(dead_code)]
pub fn matrix_to_bytes(export: &MatrixExport) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(64);
    for &v in &export.data {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    bytes
}

/// Convert matrix to JSON.
#[allow(dead_code)]
pub fn matrix_to_json(export: &MatrixExport) -> String {
    let vals: Vec<String> = export.data.iter().map(|v| format!("{:.6}", v)).collect();
    format!("{{\"matrix\":[{}]}}", vals.join(","))
}

/// Transpose a matrix.
#[allow(dead_code)]
pub fn matrix_transpose(export: &MatrixExport) -> MatrixExport {
    let d = &export.data;
    MatrixExport {
        data: [
            d[0], d[4], d[8],  d[12],
            d[1], d[5], d[9],  d[13],
            d[2], d[6], d[10], d[14],
            d[3], d[7], d[11], d[15],
        ],
    }
}

/// Compute the determinant of a 4x4 matrix.
#[allow(dead_code)]
pub fn matrix_determinant_export(export: &MatrixExport) -> f32 {
    let m = &export.data;
    let s0 = m[0] * m[5] - m[4] * m[1];
    let s1 = m[0] * m[6] - m[4] * m[2];
    let s2 = m[0] * m[7] - m[4] * m[3];
    let s3 = m[1] * m[6] - m[5] * m[2];
    let s4 = m[1] * m[7] - m[5] * m[3];
    let s5 = m[2] * m[7] - m[6] * m[3];
    let c5 = m[10] * m[15] - m[14] * m[11];
    let c4 = m[9] * m[15] - m[13] * m[11];
    let c3 = m[9] * m[14] - m[13] * m[10];
    let c2 = m[8] * m[15] - m[12] * m[11];
    let c1 = m[8] * m[14] - m[12] * m[10];
    let c0 = m[8] * m[13] - m[12] * m[9];
    s0 * c5 - s1 * c4 + s2 * c3 + s3 * c2 - s4 * c1 + s5 * c0
}

/// Check if a matrix is the identity matrix.
#[allow(dead_code)]
pub fn matrix_is_identity(export: &MatrixExport) -> bool {
    let identity = [
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ];
    export.data.iter().zip(identity.iter()).all(|(a, b)| (a - b).abs() < 1e-6)
}

/// Return the export size in bytes.
#[allow(dead_code)]
pub fn matrix_export_size() -> usize {
    64
}

/// Validate that the matrix contains finite values.
#[allow(dead_code)]
pub fn validate_matrix(export: &MatrixExport) -> bool {
    export.data.iter().all(|v| v.is_finite())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity() -> MatrixExport {
        export_matrix4([
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ])
    }

    #[test]
    fn test_export_matrix4() {
        let m = identity();
        assert_eq!(m.data[0], 1.0);
    }

    #[test]
    fn test_matrix_to_bytes() {
        let m = identity();
        let bytes = matrix_to_bytes(&m);
        assert_eq!(bytes.len(), 64);
    }

    #[test]
    fn test_matrix_to_json() {
        let m = identity();
        let j = matrix_to_json(&m);
        assert!(j.contains("matrix"));
    }

    #[test]
    fn test_matrix_transpose() {
        let m = export_matrix4([
            1.0, 2.0, 3.0, 4.0,
            5.0, 6.0, 7.0, 8.0,
            9.0, 10.0, 11.0, 12.0,
            13.0, 14.0, 15.0, 16.0,
        ]);
        let t = matrix_transpose(&m);
        assert!((t.data[1] - 5.0).abs() < 1e-6);
        assert!((t.data[4] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_matrix_determinant_identity() {
        let m = identity();
        let det = matrix_determinant_export(&m);
        assert!((det - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_matrix_is_identity() {
        assert!(matrix_is_identity(&identity()));
    }

    #[test]
    fn test_matrix_is_not_identity() {
        let m = export_matrix4([2.0; 16]);
        assert!(!matrix_is_identity(&m));
    }

    #[test]
    fn test_matrix_export_size() {
        assert_eq!(matrix_export_size(), 64);
    }

    #[test]
    fn test_validate_matrix() {
        assert!(validate_matrix(&identity()));
    }

    #[test]
    fn test_validate_matrix_nan() {
        let m = export_matrix4([f32::NAN; 16]);
        assert!(!validate_matrix(&m));
    }
}
