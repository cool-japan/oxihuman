#![allow(dead_code)]
//! Viewport camera with view/projection matrices.

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ViewportCamera {
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub up: [f32; 3],
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

#[allow(dead_code)]
pub fn new_viewport_camera(position: [f32; 3], target: [f32; 3], fov: f32, aspect: f32) -> ViewportCamera {
    ViewportCamera {
        position,
        target,
        up: [0.0, 1.0, 0.0],
        fov,
        aspect,
        near: 0.1,
        far: 1000.0,
    }
}

#[allow(dead_code)]
pub fn camera_view_matrix(c: &ViewportCamera) -> [f32; 16] {
    let f = normalize(sub(c.target, c.position));
    let s = normalize(cross(f, c.up));
    let u = cross(s, f);
    [
        s[0], u[0], -f[0], 0.0,
        s[1], u[1], -f[1], 0.0,
        s[2], u[2], -f[2], 0.0,
        -dot(s, c.position), -dot(u, c.position), dot(f, c.position), 1.0,
    ]
}

#[allow(dead_code)]
pub fn camera_proj_matrix(c: &ViewportCamera) -> [f32; 16] {
    let f = 1.0 / (c.fov * 0.5).tan();
    let nf = 1.0 / (c.near - c.far);
    [
        f / c.aspect, 0.0, 0.0, 0.0,
        0.0, f, 0.0, 0.0,
        0.0, 0.0, (c.far + c.near) * nf, -1.0,
        0.0, 0.0, 2.0 * c.far * c.near * nf, 0.0,
    ]
}

#[allow(dead_code)]
pub fn camera_view_proj(c: &ViewportCamera) -> [f32; 16] {
    let v = camera_view_matrix(c);
    let p = camera_proj_matrix(c);
    mat4_mul(&p, &v)
}

#[allow(dead_code)]
pub fn camera_position_vp(c: &ViewportCamera) -> [f32; 3] {
    c.position
}

#[allow(dead_code)]
pub fn camera_forward(c: &ViewportCamera) -> [f32; 3] {
    normalize(sub(c.target, c.position))
}

#[allow(dead_code)]
pub fn camera_right(c: &ViewportCamera) -> [f32; 3] {
    let f = camera_forward(c);
    normalize(cross(f, c.up))
}

#[allow(dead_code)]
pub fn camera_up_dir(c: &ViewportCamera) -> [f32; 3] {
    c.up
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-9 {
        return [0.0, 0.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

fn mat4_mul(a: &[f32; 16], b: &[f32; 16]) -> [f32; 16] {
    let mut r = [0.0_f32; 16];
    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                r[i * 4 + j] += a[i * 4 + k] * b[k * 4 + j];
            }
        }
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_viewport_camera() {
        let c = new_viewport_camera([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], 1.0, 1.777);
        assert!((c.position[2] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_camera_position_vp() {
        let c = new_viewport_camera([1.0, 2.0, 3.0], [0.0, 0.0, 0.0], 1.0, 1.0);
        let p = camera_position_vp(&c);
        assert!((p[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_camera_forward() {
        let c = new_viewport_camera([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], 1.0, 1.0);
        let f = camera_forward(&c);
        assert!(f[2] < 0.0);
    }

    #[test]
    fn test_camera_right() {
        let c = new_viewport_camera([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], 1.0, 1.0);
        let r = camera_right(&c);
        assert!((r[0] - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_camera_up_dir() {
        let c = new_viewport_camera([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], 1.0, 1.0);
        let u = camera_up_dir(&c);
        assert!((u[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_camera_view_matrix() {
        let c = new_viewport_camera([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], 1.0, 1.0);
        let m = camera_view_matrix(&c);
        assert!((m[15] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_camera_proj_matrix() {
        let c = new_viewport_camera([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], 1.0, 1.0);
        let m = camera_proj_matrix(&c);
        assert!(m[0].abs() > 0.0);
    }

    #[test]
    fn test_camera_view_proj() {
        let c = new_viewport_camera([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], 1.0, 1.0);
        let vp = camera_view_proj(&c);
        // Just ensure it doesn't crash and produces something
        assert!(vp.iter().any(|v| v.abs() > 0.0));
    }

    #[test]
    fn test_normalize_helper() {
        let v = normalize([3.0, 0.0, 0.0]);
        assert!((v[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_near_far() {
        let c = new_viewport_camera([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], 1.0, 1.0);
        assert!((c.near - 0.1).abs() < 1e-6);
        assert!((c.far - 1000.0).abs() < 1e-6);
    }
}
