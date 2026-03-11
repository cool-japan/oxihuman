#![allow(dead_code)]
//! Render viewport: defines a rectangular viewport region.

/// A render viewport.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderViewport {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

/// Create a new render viewport.
#[allow(dead_code)]
pub fn new_render_viewport(x: u32, y: u32, width: u32, height: u32) -> RenderViewport {
    RenderViewport { x, y, width, height }
}

/// Return the X origin.
#[allow(dead_code)]
pub fn viewport_x(vp: &RenderViewport) -> u32 {
    vp.x
}

/// Return the Y origin.
#[allow(dead_code)]
pub fn viewport_y(vp: &RenderViewport) -> u32 {
    vp.y
}

/// Return the width.
#[allow(dead_code)]
pub fn viewport_width_rv(vp: &RenderViewport) -> u32 {
    vp.width
}

/// Return the height.
#[allow(dead_code)]
pub fn viewport_height_rv(vp: &RenderViewport) -> u32 {
    vp.height
}

/// Return the aspect ratio (width / height). Returns 1.0 if height is 0.
#[allow(dead_code)]
pub fn viewport_aspect_rv(vp: &RenderViewport) -> f32 {
    if vp.height == 0 {
        return 1.0;
    }
    vp.width as f32 / vp.height as f32
}

/// Serialize to JSON-like string.
#[allow(dead_code)]
pub fn viewport_to_json(vp: &RenderViewport) -> String {
    format!(
        "{{\"x\":{},\"y\":{},\"width\":{},\"height\":{}}}",
        vp.x, vp.y, vp.width, vp.height
    )
}

/// Check if a point (px, py) is within the viewport.
#[allow(dead_code)]
pub fn viewport_contains_point_rv(vp: &RenderViewport, px: u32, py: u32) -> bool {
    (vp.x..=vp.x + vp.width).contains(&px) && (vp.y..=vp.y + vp.height).contains(&py)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_viewport() {
        let vp = new_render_viewport(0, 0, 800, 600);
        assert_eq!(viewport_width_rv(&vp), 800);
    }

    #[test]
    fn test_x_y() {
        let vp = new_render_viewport(10, 20, 100, 100);
        assert_eq!(viewport_x(&vp), 10);
        assert_eq!(viewport_y(&vp), 20);
    }

    #[test]
    fn test_width_height() {
        let vp = new_render_viewport(0, 0, 1920, 1080);
        assert_eq!(viewport_width_rv(&vp), 1920);
        assert_eq!(viewport_height_rv(&vp), 1080);
    }

    #[test]
    fn test_aspect_ratio() {
        let vp = new_render_viewport(0, 0, 1920, 1080);
        let aspect = viewport_aspect_rv(&vp);
        assert!((aspect - 1920.0 / 1080.0).abs() < 0.01);
    }

    #[test]
    fn test_aspect_zero_height() {
        let vp = new_render_viewport(0, 0, 100, 0);
        assert!((viewport_aspect_rv(&vp) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let vp = new_render_viewport(0, 0, 800, 600);
        let json = viewport_to_json(&vp);
        assert!(json.contains("\"width\":800"));
    }

    #[test]
    fn test_contains_point_inside() {
        let vp = new_render_viewport(10, 10, 100, 100);
        assert!(viewport_contains_point_rv(&vp, 50, 50));
    }

    #[test]
    fn test_contains_point_outside() {
        let vp = new_render_viewport(10, 10, 100, 100);
        assert!(!viewport_contains_point_rv(&vp, 5, 5));
    }

    #[test]
    fn test_contains_point_boundary() {
        let vp = new_render_viewport(0, 0, 100, 100);
        assert!(viewport_contains_point_rv(&vp, 0, 0));
        assert!(viewport_contains_point_rv(&vp, 100, 100));
    }

    #[test]
    fn test_square_viewport() {
        let vp = new_render_viewport(0, 0, 512, 512);
        assert!((viewport_aspect_rv(&vp) - 1.0).abs() < 1e-6);
    }
}
