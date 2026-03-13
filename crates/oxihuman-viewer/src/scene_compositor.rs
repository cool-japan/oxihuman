// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Multi-layer scene compositing for the viewer.

#[allow(dead_code)]
pub enum BlendOp {
    Over,
    Add,
    Multiply,
    Screen,
    Mask,
}

#[allow(dead_code)]
pub struct CompositeLayer {
    pub name: String,
    pub visible: bool,
    pub opacity: f32,
    pub blend: BlendOp,
    pub z_order: i32,
    pub pixels: Vec<[f32; 4]>,
    pub width: u32,
    pub height: u32,
}

#[allow(dead_code)]
pub struct SceneCompositor {
    pub layers: Vec<CompositeLayer>,
    pub output_width: u32,
    pub output_height: u32,
    pub background_color: [f32; 4],
}

#[allow(dead_code)]
pub fn new_scene_compositor(width: u32, height: u32) -> SceneCompositor {
    SceneCompositor {
        layers: Vec::new(),
        output_width: width,
        output_height: height,
        background_color: [0.0, 0.0, 0.0, 1.0],
    }
}

#[allow(dead_code)]
pub fn add_layer(comp: &mut SceneCompositor, layer: CompositeLayer) {
    comp.layers.push(layer);
    // Keep layers sorted by z_order ascending (lowest rendered first)
    comp.layers.sort_by_key(|l| l.z_order);
}

#[allow(dead_code)]
pub fn remove_layer(comp: &mut SceneCompositor, name: &str) -> bool {
    let before = comp.layers.len();
    comp.layers.retain(|l| l.name != name);
    comp.layers.len() < before
}

#[allow(dead_code)]
pub fn composite_all(comp: &SceneCompositor) -> Vec<[f32; 4]> {
    let n_pixels = (comp.output_width * comp.output_height) as usize;
    let mut result = vec![comp.background_color; n_pixels];

    for layer in comp.layers.iter().filter(|l| l.visible) {
        if layer.pixels.len() != n_pixels {
            continue;
        }
        let blended = composite_two(&result, &layer.pixels, &layer.blend, layer.opacity);
        result = blended;
    }
    result
}

#[allow(dead_code)]
pub fn composite_two(a: &[[f32; 4]], b: &[[f32; 4]], op: &BlendOp, opacity: f32) -> Vec<[f32; 4]> {
    let len = a.len().min(b.len());
    (0..len)
        .map(|i| blend_pixel(a[i], b[i], op, opacity))
        .collect()
}

#[allow(dead_code)]
pub fn blend_pixel(a: [f32; 4], b: [f32; 4], op: &BlendOp, opacity: f32) -> [f32; 4] {
    let alpha_b = b[3] * opacity.clamp(0.0, 1.0);
    match op {
        BlendOp::Over => {
            // Standard Porter-Duff "over"
            let out_a = alpha_b + a[3] * (1.0 - alpha_b);
            if out_a < 1e-9 {
                return [0.0, 0.0, 0.0, 0.0];
            }
            let r = (b[0] * alpha_b + a[0] * a[3] * (1.0 - alpha_b)) / out_a;
            let g = (b[1] * alpha_b + a[1] * a[3] * (1.0 - alpha_b)) / out_a;
            let bl = (b[2] * alpha_b + a[2] * a[3] * (1.0 - alpha_b)) / out_a;
            [r, g, bl, out_a]
        }
        BlendOp::Add => {
            let scale = alpha_b;
            [
                (a[0] + b[0] * scale).min(1.0),
                (a[1] + b[1] * scale).min(1.0),
                (a[2] + b[2] * scale).min(1.0),
                (a[3] + alpha_b).min(1.0),
            ]
        }
        BlendOp::Multiply => {
            let t = alpha_b;
            let r = a[0] * b[0] * t + a[0] * (1.0 - t);
            let g = a[1] * b[1] * t + a[1] * (1.0 - t);
            let bl = a[2] * b[2] * t + a[2] * (1.0 - t);
            [r, g, bl, a[3]]
        }
        BlendOp::Screen => {
            let t = alpha_b;
            let r = (1.0 - (1.0 - a[0]) * (1.0 - b[0])) * t + a[0] * (1.0 - t);
            let g = (1.0 - (1.0 - a[1]) * (1.0 - b[1])) * t + a[1] * (1.0 - t);
            let bl = (1.0 - (1.0 - a[2]) * (1.0 - b[2])) * t + a[2] * (1.0 - t);
            [r, g, bl, a[3]]
        }
        BlendOp::Mask => {
            // Use b's alpha as a mask for a
            let mask = alpha_b;
            [a[0] * mask, a[1] * mask, a[2] * mask, a[3] * mask]
        }
    }
}

#[allow(dead_code)]
pub fn move_layer_to_top(comp: &mut SceneCompositor, name: &str) -> bool {
    let max_z = comp.layers.iter().map(|l| l.z_order).max().unwrap_or(0);
    if let Some(layer) = comp.layers.iter_mut().find(|l| l.name == name) {
        layer.z_order = max_z + 1;
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn set_layer_opacity(comp: &mut SceneCompositor, name: &str, opacity: f32) -> bool {
    if let Some(layer) = comp.layers.iter_mut().find(|l| l.name == name) {
        layer.opacity = opacity.clamp(0.0, 1.0);
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn layer_count(comp: &SceneCompositor) -> usize {
    comp.layers.len()
}

#[allow(dead_code)]
pub fn visible_layer_count(comp: &SceneCompositor) -> usize {
    comp.layers.iter().filter(|l| l.visible).count()
}

#[allow(dead_code)]
pub fn to_u8_pixels(pixels: &[[f32; 4]]) -> Vec<u8> {
    let mut out = Vec::with_capacity(pixels.len() * 4);
    for p in pixels {
        out.push((p[0].clamp(0.0, 1.0) * 255.0).round() as u8);
        out.push((p[1].clamp(0.0, 1.0) * 255.0).round() as u8);
        out.push((p[2].clamp(0.0, 1.0) * 255.0).round() as u8);
        out.push((p[3].clamp(0.0, 1.0) * 255.0).round() as u8);
    }
    out
}

#[allow(dead_code)]
pub fn compositor_to_json(comp: &SceneCompositor) -> String {
    let mut out = format!(
        r#"{{"width":{},"height":{},"layer_count":{},"layers":["#,
        comp.output_width,
        comp.output_height,
        comp.layers.len()
    );
    for (i, l) in comp.layers.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let blend_name = match l.blend {
            BlendOp::Over => "over",
            BlendOp::Add => "add",
            BlendOp::Multiply => "multiply",
            BlendOp::Screen => "screen",
            BlendOp::Mask => "mask",
        };
        out.push_str(&format!(
            r#"{{"name":"{}","visible":{},"opacity":{},"blend":"{}","z_order":{}}}"#,
            l.name.replace('"', "\\\""),
            l.visible,
            l.opacity,
            blend_name,
            l.z_order
        ));
    }
    out.push_str("]}");
    out
}

#[allow(dead_code)]
fn make_solid_layer(name: &str, w: u32, h: u32, color: [f32; 4], z: i32) -> CompositeLayer {
    let n = (w * h) as usize;
    CompositeLayer {
        name: name.to_string(),
        visible: true,
        opacity: 1.0,
        blend: BlendOp::Over,
        z_order: z,
        pixels: vec![color; n],
        width: w,
        height: h,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_compositor() {
        let c = new_scene_compositor(100, 100);
        assert_eq!(c.output_width, 100);
        assert_eq!(c.output_height, 100);
        assert!(c.layers.is_empty());
    }

    #[test]
    fn test_add_layer() {
        let mut c = new_scene_compositor(4, 4);
        let layer = make_solid_layer("base", 4, 4, [1.0, 0.0, 0.0, 1.0], 0);
        add_layer(&mut c, layer);
        assert_eq!(layer_count(&c), 1);
    }

    #[test]
    fn test_remove_layer_existing() {
        let mut c = new_scene_compositor(4, 4);
        add_layer(&mut c, make_solid_layer("base", 4, 4, [1.0; 4], 0));
        assert!(remove_layer(&mut c, "base"));
        assert_eq!(layer_count(&c), 0);
    }

    #[test]
    fn test_remove_layer_missing() {
        let mut c = new_scene_compositor(4, 4);
        assert!(!remove_layer(&mut c, "nonexistent"));
    }

    #[test]
    fn test_composite_all_single_layer() {
        let mut c = new_scene_compositor(2, 2);
        let color = [0.5, 0.5, 0.5, 1.0];
        add_layer(&mut c, make_solid_layer("l1", 2, 2, color, 0));
        let out = composite_all(&c);
        assert_eq!(out.len(), 4);
        // Over a black background with alpha=1, should be the layer color
        assert!((out[0][0] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_blend_pixel_over() {
        let a = [1.0f32, 0.0, 0.0, 1.0]; // red opaque background
        let b = [0.0f32, 0.0, 1.0, 0.5]; // blue half-alpha
        let r = blend_pixel(a, b, &BlendOp::Over, 1.0);
        // result should be a blend of red and blue
        assert!(r[0] > 0.0 && r[2] > 0.0);
        assert!((r[3] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_blend_pixel_add() {
        let a = [0.2f32, 0.2, 0.2, 1.0];
        let b = [0.3f32, 0.3, 0.3, 1.0];
        let r = blend_pixel(a, b, &BlendOp::Add, 1.0);
        assert!((r[0] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_blend_pixel_multiply() {
        let a = [1.0f32, 1.0, 1.0, 1.0];
        let b = [0.5f32, 0.5, 0.5, 1.0];
        let r = blend_pixel(a, b, &BlendOp::Multiply, 1.0);
        assert!((r[0] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_blend_pixel_mask() {
        let a = [1.0f32, 0.0, 0.0, 1.0];
        let b = [0.0f32, 0.0, 0.0, 0.5]; // mask alpha = 0.5
        let r = blend_pixel(a, b, &BlendOp::Mask, 1.0);
        assert!((r[0] - 0.5).abs() < 0.01);
        assert!((r[3] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_move_layer_to_top() {
        let mut c = new_scene_compositor(4, 4);
        add_layer(&mut c, make_solid_layer("a", 4, 4, [1.0; 4], 0));
        add_layer(&mut c, make_solid_layer("b", 4, 4, [1.0; 4], 1));
        assert!(move_layer_to_top(&mut c, "a"));
        let az = c.layers.iter().find(|l| l.name == "a").expect("should succeed").z_order;
        let bz = c.layers.iter().find(|l| l.name == "b").expect("should succeed").z_order;
        assert!(az > bz);
    }

    #[test]
    fn test_set_layer_opacity() {
        let mut c = new_scene_compositor(4, 4);
        add_layer(&mut c, make_solid_layer("l", 4, 4, [1.0; 4], 0));
        assert!(set_layer_opacity(&mut c, "l", 0.5));
        let op = c.layers.iter().find(|l| l.name == "l").expect("should succeed").opacity;
        assert!((op - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_visible_layer_count() {
        let mut c = new_scene_compositor(4, 4);
        add_layer(&mut c, make_solid_layer("a", 4, 4, [1.0; 4], 0));
        let mut b = make_solid_layer("b", 4, 4, [1.0; 4], 1);
        b.visible = false;
        add_layer(&mut c, b);
        assert_eq!(visible_layer_count(&c), 1);
    }

    #[test]
    fn test_to_u8_pixels() {
        let pixels = vec![[1.0f32, 0.0, 0.5, 1.0]];
        let bytes = to_u8_pixels(&pixels);
        assert_eq!(bytes[0], 255);
        assert_eq!(bytes[1], 0);
        assert_eq!(bytes[3], 255);
    }

    #[test]
    fn test_compositor_to_json() {
        let mut c = new_scene_compositor(100, 100);
        add_layer(&mut c, make_solid_layer("layer1", 100, 100, [1.0; 4], 0));
        let json = compositor_to_json(&c);
        assert!(json.contains("\"layer1\""));
        assert!(json.contains("\"width\":100"));
    }

    #[test]
    fn test_composite_two_length() {
        let a = vec![[1.0f32, 0.0, 0.0, 1.0]; 10];
        let b = vec![[0.0f32, 1.0, 0.0, 1.0]; 10];
        let r = composite_two(&a, &b, &BlendOp::Over, 1.0);
        assert_eq!(r.len(), 10);
    }
}
