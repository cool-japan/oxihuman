#![allow(dead_code)]
//! Export world transform data.

/// World transform export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct WorldTransformExport {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

/// Export a world transform.
#[allow(dead_code)]
pub fn export_world_transform(position: [f32; 3], rotation: [f32; 4], scale: [f32; 3]) -> WorldTransformExport {
    WorldTransformExport { position, rotation, scale }
}

/// Get the position component.
#[allow(dead_code)]
pub fn transform_position(export: &WorldTransformExport) -> [f32; 3] {
    export.position
}

/// Get the rotation component (quaternion xyzw).
#[allow(dead_code)]
pub fn transform_rotation(export: &WorldTransformExport) -> [f32; 4] {
    export.rotation
}

/// Get the scale component.
#[allow(dead_code)]
pub fn transform_scale_export(export: &WorldTransformExport) -> [f32; 3] {
    export.scale
}

/// Convert the transform to a 4x4 matrix (column-major).
#[allow(dead_code)]
pub fn transform_to_matrix(export: &WorldTransformExport) -> [f32; 16] {
    let q = export.rotation;
    let s = export.scale;
    let t = export.position;
    let x2 = q[0] + q[0];
    let y2 = q[1] + q[1];
    let z2 = q[2] + q[2];
    let xx = q[0] * x2;
    let xy = q[0] * y2;
    let xz = q[0] * z2;
    let yy = q[1] * y2;
    let yz = q[1] * z2;
    let zz = q[2] * z2;
    let wx = q[3] * x2;
    let wy = q[3] * y2;
    let wz = q[3] * z2;
    [
        (1.0 - (yy + zz)) * s[0], (xy + wz) * s[0], (xz - wy) * s[0], 0.0,
        (xy - wz) * s[1], (1.0 - (xx + zz)) * s[1], (yz + wx) * s[1], 0.0,
        (xz + wy) * s[2], (yz - wx) * s[2], (1.0 - (xx + yy)) * s[2], 0.0,
        t[0], t[1], t[2], 1.0,
    ]
}

/// Convert transform to JSON.
#[allow(dead_code)]
pub fn transform_to_json(export: &WorldTransformExport) -> String {
    format!(
        "{{\"position\":[{:.4},{:.4},{:.4}],\"rotation\":[{:.4},{:.4},{:.4},{:.4}],\"scale\":[{:.4},{:.4},{:.4}]}}",
        export.position[0], export.position[1], export.position[2],
        export.rotation[0], export.rotation[1], export.rotation[2], export.rotation[3],
        export.scale[0], export.scale[1], export.scale[2]
    )
}

/// Check if transform is identity.
#[allow(dead_code)]
pub fn transform_is_identity(export: &WorldTransformExport) -> bool {
    let pos_zero = export.position.iter().all(|&v| v.abs() < 1e-6);
    let rot_identity = (export.rotation[0].abs() < 1e-6)
        && (export.rotation[1].abs() < 1e-6)
        && (export.rotation[2].abs() < 1e-6)
        && ((export.rotation[3] - 1.0).abs() < 1e-6);
    let scale_one = export.scale.iter().all(|&v| (v - 1.0).abs() < 1e-6);
    pos_zero && rot_identity && scale_one
}

/// Validate transform data.
#[allow(dead_code)]
pub fn validate_transform(export: &WorldTransformExport) -> bool {
    let all_finite = export.position.iter().all(|v| v.is_finite())
        && export.rotation.iter().all(|v| v.is_finite())
        && export.scale.iter().all(|v| v.is_finite());
    let quat_len = (export.rotation.iter().map(|v| v * v).sum::<f32>()).sqrt();
    all_finite && (quat_len - 1.0).abs() < 0.1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity_transform() -> WorldTransformExport {
        export_world_transform([0.0; 3], [0.0, 0.0, 0.0, 1.0], [1.0, 1.0, 1.0])
    }

    #[test]
    fn test_export_world_transform() {
        let t = identity_transform();
        assert_eq!(t.position, [0.0; 3]);
    }

    #[test]
    fn test_transform_position() {
        let t = identity_transform();
        assert_eq!(transform_position(&t), [0.0; 3]);
    }

    #[test]
    fn test_transform_rotation() {
        let t = identity_transform();
        assert_eq!(transform_rotation(&t), [0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_transform_scale() {
        let t = identity_transform();
        assert_eq!(transform_scale_export(&t), [1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_transform_to_matrix_identity() {
        let t = identity_transform();
        let m = transform_to_matrix(&t);
        assert!((m[0] - 1.0).abs() < 1e-4);
        assert!((m[5] - 1.0).abs() < 1e-4);
        assert!((m[10] - 1.0).abs() < 1e-4);
        assert!((m[15] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_transform_to_json() {
        let t = identity_transform();
        let j = transform_to_json(&t);
        assert!(j.contains("position"));
        assert!(j.contains("rotation"));
    }

    #[test]
    fn test_transform_is_identity() {
        assert!(transform_is_identity(&identity_transform()));
    }

    #[test]
    fn test_transform_is_not_identity() {
        let t = export_world_transform([1.0, 0.0, 0.0], [0.0, 0.0, 0.0, 1.0], [1.0, 1.0, 1.0]);
        assert!(!transform_is_identity(&t));
    }

    #[test]
    fn test_validate_transform() {
        assert!(validate_transform(&identity_transform()));
    }

    #[test]
    fn test_validate_transform_bad() {
        let t = export_world_transform([0.0; 3], [0.0; 4], [1.0; 3]);
        assert!(!validate_transform(&t));
    }
}
