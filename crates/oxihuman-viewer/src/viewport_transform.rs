#![allow(dead_code)]

/// Viewport transform for screen/NDC/world conversions.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ViewportTransform {
    width: f32,
    height: f32,
}

#[allow(dead_code)]
pub fn new_viewport_transform(width: f32, height: f32) -> ViewportTransform {
    ViewportTransform { width, height }
}

#[allow(dead_code)]
pub fn screen_to_ndc(vt: &ViewportTransform, x: f32, y: f32) -> [f32; 2] {
    let nx = if vt.width > 0.0 { x / vt.width * 2.0 - 1.0 } else { 0.0 };
    let ny = if vt.height > 0.0 { 1.0 - y / vt.height * 2.0 } else { 0.0 };
    [nx, ny]
}

#[allow(dead_code)]
pub fn ndc_to_screen(vt: &ViewportTransform, nx: f32, ny: f32) -> [f32; 2] {
    let x = (nx + 1.0) * 0.5 * vt.width;
    let y = (1.0 - ny) * 0.5 * vt.height;
    [x, y]
}

#[allow(dead_code)]
pub fn screen_to_world_stub(vt: &ViewportTransform, x: f32, y: f32) -> [f32; 3] {
    let ndc = screen_to_ndc(vt, x, y);
    [ndc[0], ndc[1], 0.0]
}

#[allow(dead_code)]
pub fn world_to_screen_stub(vt: &ViewportTransform, pos: [f32; 3]) -> [f32; 2] {
    ndc_to_screen(vt, pos[0], pos[1])
}

#[allow(dead_code)]
pub fn transform_point_vt(vt: &ViewportTransform, x: f32, y: f32) -> [f32; 2] {
    screen_to_ndc(vt, x, y)
}

#[allow(dead_code)]
pub fn transform_to_json(vt: &ViewportTransform) -> String {
    format!("{{\"width\":{:.2},\"height\":{:.2}}}", vt.width, vt.height)
}

#[allow(dead_code)]
pub fn transform_is_identity_vt(vt: &ViewportTransform) -> bool {
    (vt.width - 2.0).abs() < 1e-6 && (vt.height - 2.0).abs() < 1e-6
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let v = new_viewport_transform(800.0, 600.0); assert!((v.width - 800.0).abs() < 1e-6); }
    #[test] fn test_screen_to_ndc_center() {
        let v = new_viewport_transform(100.0, 100.0);
        let ndc = screen_to_ndc(&v, 50.0, 50.0);
        assert!((ndc[0]).abs() < 1e-6);
        assert!((ndc[1]).abs() < 1e-6);
    }
    #[test] fn test_screen_to_ndc_corners() {
        let v = new_viewport_transform(100.0, 100.0);
        let ndc = screen_to_ndc(&v, 0.0, 0.0);
        assert!((ndc[0] - (-1.0)).abs() < 1e-6);
        assert!((ndc[1] - 1.0).abs() < 1e-6);
    }
    #[test] fn test_ndc_to_screen() {
        let v = new_viewport_transform(100.0, 100.0);
        let s = ndc_to_screen(&v, 0.0, 0.0);
        assert!((s[0] - 50.0).abs() < 1e-6);
        assert!((s[1] - 50.0).abs() < 1e-6);
    }
    #[test] fn test_roundtrip() {
        let v = new_viewport_transform(800.0, 600.0);
        let ndc = screen_to_ndc(&v, 400.0, 300.0);
        let s = ndc_to_screen(&v, ndc[0], ndc[1]);
        assert!((s[0] - 400.0).abs() < 1e-3);
    }
    #[test] fn test_world_stub() {
        let v = new_viewport_transform(100.0, 100.0);
        let w = screen_to_world_stub(&v, 50.0, 50.0);
        assert!((w[2]).abs() < 1e-6);
    }
    #[test] fn test_to_json() { assert!(transform_to_json(&new_viewport_transform(1.0, 1.0)).contains("width")); }
    #[test] fn test_identity() { assert!(transform_is_identity_vt(&new_viewport_transform(2.0, 2.0))); }
    #[test] fn test_not_identity() { assert!(!transform_is_identity_vt(&new_viewport_transform(800.0, 600.0))); }
    #[test] fn test_transform_point() {
        let v = new_viewport_transform(100.0, 100.0);
        let p = transform_point_vt(&v, 50.0, 50.0);
        assert!((p[0]).abs() < 1e-6);
    }
}
