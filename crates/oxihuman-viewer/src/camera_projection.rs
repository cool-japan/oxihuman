#![allow(dead_code)]

/// Projection type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProjectionType { Perspective, Orthographic }

/// Camera projection configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraProjection {
    proj_type: ProjectionType,
    fov_deg: f32,
    near: f32,
    far: f32,
    aspect: f32,
    ortho_size: f32,
}

#[allow(dead_code)]
pub fn new_perspective(fov_deg: f32, aspect: f32, near: f32, far: f32) -> CameraProjection {
    CameraProjection { proj_type: ProjectionType::Perspective, fov_deg, near, far, aspect, ortho_size: 1.0 }
}

#[allow(dead_code)]
pub fn new_orthographic(size: f32, aspect: f32, near: f32, far: f32) -> CameraProjection {
    CameraProjection { proj_type: ProjectionType::Orthographic, fov_deg: 0.0, near, far, aspect, ortho_size: size }
}

#[allow(dead_code)]
pub fn projection_matrix(proj: &CameraProjection) -> [[f32; 4]; 4] {
    let mut m = [[0.0f32; 4]; 4];
    match proj.proj_type {
        ProjectionType::Perspective => {
            let f = 1.0 / (proj.fov_deg.to_radians() * 0.5).tan();
            let nf = 1.0 / (proj.near - proj.far);
            m[0][0] = f / proj.aspect;
            m[1][1] = f;
            m[2][2] = (proj.far + proj.near) * nf;
            m[2][3] = -1.0;
            m[3][2] = 2.0 * proj.far * proj.near * nf;
        }
        ProjectionType::Orthographic => {
            let r = proj.ortho_size * proj.aspect;
            let t = proj.ortho_size;
            m[0][0] = 1.0 / r;
            m[1][1] = 1.0 / t;
            m[2][2] = -2.0 / (proj.far - proj.near);
            m[3][2] = -(proj.far + proj.near) / (proj.far - proj.near);
            m[3][3] = 1.0;
        }
    }
    m
}

#[allow(dead_code)]
pub fn projection_type(proj: &CameraProjection) -> ProjectionType { proj.proj_type }

#[allow(dead_code)]
pub fn projection_fov(proj: &CameraProjection) -> f32 { proj.fov_deg }

#[allow(dead_code)]
pub fn projection_to_json(proj: &CameraProjection) -> String {
    let t = match proj.proj_type { ProjectionType::Perspective => "perspective", ProjectionType::Orthographic => "orthographic" };
    format!("{{\"type\":\"{}\",\"near\":{:.4},\"far\":{:.4}}}", t, proj.near, proj.far)
}

#[allow(dead_code)]
pub fn projection_near(proj: &CameraProjection) -> f32 { proj.near }

#[allow(dead_code)]
pub fn projection_far(proj: &CameraProjection) -> f32 { proj.far }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_perspective() { let p = new_perspective(60.0, 1.0, 0.1, 100.0); assert_eq!(projection_type(&p), ProjectionType::Perspective); }
    #[test] fn test_orthographic() { let p = new_orthographic(5.0, 1.0, 0.1, 100.0); assert_eq!(projection_type(&p), ProjectionType::Orthographic); }
    #[test] fn test_fov() { assert!((projection_fov(&new_perspective(60.0, 1.0, 0.1, 100.0)) - 60.0).abs() < 1e-6); }
    #[test] fn test_near() { assert!((projection_near(&new_perspective(60.0, 1.0, 0.1, 100.0)) - 0.1).abs() < 1e-6); }
    #[test] fn test_far() { assert!((projection_far(&new_perspective(60.0, 1.0, 0.1, 100.0)) - 100.0).abs() < 1e-6); }
    #[test] fn test_matrix_perspective() {
        let p = new_perspective(90.0, 1.0, 0.1, 100.0);
        let m = projection_matrix(&p);
        assert!(m[0][0].abs() > 0.0);
    }
    #[test] fn test_matrix_ortho() {
        let p = new_orthographic(5.0, 1.0, 0.1, 100.0);
        let m = projection_matrix(&p);
        assert!((m[3][3] - 1.0).abs() < 1e-6);
    }
    #[test] fn test_to_json() { assert!(projection_to_json(&new_perspective(60.0, 1.0, 0.1, 100.0)).contains("perspective")); }
    #[test] fn test_ortho_json() { assert!(projection_to_json(&new_orthographic(5.0, 1.0, 0.1, 100.0)).contains("orthographic")); }
    #[test] fn test_ortho_fov() { assert!((projection_fov(&new_orthographic(5.0, 1.0, 0.1, 100.0))).abs() < 1e-6); }
}
