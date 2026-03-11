#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutMode { Single, SplitH, SplitV, Quad }

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ViewportLayout {
    mode: LayoutMode,
    width: u32,
    height: u32,
}

#[allow(dead_code)]
pub fn new_viewport_layout(width: u32, height: u32) -> ViewportLayout {
    ViewportLayout { mode: LayoutMode::Single, width, height }
}

#[allow(dead_code)]
pub fn set_layout_mode(layout: &mut ViewportLayout, mode: LayoutMode) { layout.mode = mode; }

#[allow(dead_code)]
pub fn layout_viewport_count(layout: &ViewportLayout) -> usize {
    match layout.mode { LayoutMode::Single => 1, LayoutMode::SplitH | LayoutMode::SplitV => 2, LayoutMode::Quad => 4 }
}

#[allow(dead_code)]
pub fn viewport_rect_at(layout: &ViewportLayout, idx: usize) -> [u32; 4] {
    let (w, h) = (layout.width, layout.height);
    match layout.mode {
        LayoutMode::Single => [0, 0, w, h],
        LayoutMode::SplitH => if idx == 0 { [0, 0, w / 2, h] } else { [w / 2, 0, w / 2, h] },
        LayoutMode::SplitV => if idx == 0 { [0, 0, w, h / 2] } else { [0, h / 2, w, h / 2] },
        LayoutMode::Quad => {
            let hw = w / 2; let hh = h / 2;
            match idx { 0 => [0, 0, hw, hh], 1 => [hw, 0, hw, hh], 2 => [0, hh, hw, hh], _ => [hw, hh, hw, hh] }
        }
    }
}

#[allow(dead_code)]
pub fn layout_to_json(layout: &ViewportLayout) -> String {
    format!("{{\"mode\":\"{}\",\"viewports\":{}}}", layout_mode_name(layout), layout_viewport_count(layout))
}

#[allow(dead_code)]
pub fn layout_mode_name(layout: &ViewportLayout) -> &str {
    match layout.mode { LayoutMode::Single => "single", LayoutMode::SplitH => "split_h", LayoutMode::SplitV => "split_v", LayoutMode::Quad => "quad" }
}

#[allow(dead_code)]
pub fn layout_clear(layout: &mut ViewportLayout) { layout.mode = LayoutMode::Single; }

#[allow(dead_code)]
pub fn layout_resize(layout: &mut ViewportLayout, w: u32, h: u32) { layout.width = w; layout.height = h; }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let l = new_viewport_layout(800, 600); assert_eq!(layout_viewport_count(&l), 1); }
    #[test] fn test_single() { let l = new_viewport_layout(800, 600); let r = viewport_rect_at(&l, 0); assert_eq!(r, [0, 0, 800, 600]); }
    #[test] fn test_split_h() { let mut l = new_viewport_layout(800, 600); set_layout_mode(&mut l, LayoutMode::SplitH); assert_eq!(layout_viewport_count(&l), 2); }
    #[test] fn test_split_v() { let mut l = new_viewport_layout(800, 600); set_layout_mode(&mut l, LayoutMode::SplitV); assert_eq!(layout_viewport_count(&l), 2); }
    #[test] fn test_quad() { let mut l = new_viewport_layout(800, 600); set_layout_mode(&mut l, LayoutMode::Quad); assert_eq!(layout_viewport_count(&l), 4); }
    #[test] fn test_mode_name() { let l = new_viewport_layout(100, 100); assert_eq!(layout_mode_name(&l), "single"); }
    #[test] fn test_json() { let l = new_viewport_layout(100, 100); assert!(layout_to_json(&l).contains("mode")); }
    #[test] fn test_clear() { let mut l = new_viewport_layout(100, 100); set_layout_mode(&mut l, LayoutMode::Quad); layout_clear(&mut l); assert_eq!(layout_viewport_count(&l), 1); }
    #[test] fn test_resize() { let mut l = new_viewport_layout(100, 100); layout_resize(&mut l, 200, 300); let r = viewport_rect_at(&l, 0); assert_eq!(r[2], 200); }
    #[test] fn test_quad_rect() { let mut l = new_viewport_layout(800, 600); set_layout_mode(&mut l, LayoutMode::Quad); let r = viewport_rect_at(&l, 3); assert_eq!(r[0], 400); }
}
