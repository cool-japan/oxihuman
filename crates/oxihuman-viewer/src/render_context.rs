#![allow(dead_code)]

/// Render context holding viewport state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderContext {
    width: u32,
    height: u32,
    clear_color: [f32; 3],
    frame_number: u64,
}

#[allow(dead_code)]
pub fn new_render_context(width: u32, height: u32) -> RenderContext {
    RenderContext { width, height, clear_color: [0.0, 0.0, 0.0], frame_number: 0 }
}

#[allow(dead_code)]
pub fn context_width(ctx: &RenderContext) -> u32 { ctx.width }

#[allow(dead_code)]
pub fn context_height(ctx: &RenderContext) -> u32 { ctx.height }

#[allow(dead_code)]
pub fn context_aspect(ctx: &RenderContext) -> f32 {
    if ctx.height == 0 { return 1.0; }
    ctx.width as f32 / ctx.height as f32
}

#[allow(dead_code)]
pub fn context_clear_color(ctx: &RenderContext) -> [f32; 3] { ctx.clear_color }

#[allow(dead_code)]
pub fn context_to_json(ctx: &RenderContext) -> String {
    format!("{{\"width\":{},\"height\":{},\"frame\":{}}}", ctx.width, ctx.height, ctx.frame_number)
}

#[allow(dead_code)]
pub fn context_reset(ctx: &mut RenderContext) {
    ctx.frame_number = 0;
    ctx.clear_color = [0.0, 0.0, 0.0];
}

#[allow(dead_code)]
pub fn context_frame_number(ctx: &RenderContext) -> u64 { ctx.frame_number }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let c = new_render_context(800, 600); assert_eq!(context_width(&c), 800); }
    #[test] fn test_height() { assert_eq!(context_height(&new_render_context(800, 600)), 600); }
    #[test] fn test_aspect() { assert!((context_aspect(&new_render_context(800, 400)) - 2.0).abs() < 1e-6); }
    #[test] fn test_aspect_zero() { assert!((context_aspect(&new_render_context(800, 0)) - 1.0).abs() < 1e-6); }
    #[test] fn test_clear_color() { let c = new_render_context(1, 1); assert!((context_clear_color(&c)[0]).abs() < 1e-6); }
    #[test] fn test_frame_number() { assert_eq!(context_frame_number(&new_render_context(1, 1)), 0); }
    #[test] fn test_to_json() { assert!(context_to_json(&new_render_context(1, 1)).contains("width")); }
    #[test] fn test_reset() {
        let mut c = new_render_context(1, 1);
        c.frame_number = 100;
        context_reset(&mut c);
        assert_eq!(context_frame_number(&c), 0);
    }
    #[test] fn test_large_resolution() { let c = new_render_context(3840, 2160); assert_eq!(context_width(&c), 3840); }
    #[test] fn test_aspect_square() { assert!((context_aspect(&new_render_context(100, 100)) - 1.0).abs() < 1e-6); }
}
