//! Render layer stack export (distinct from render_layer_export).
#![allow(dead_code)]

/// A single render layer.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct RenderLayer2 {
    pub name: String,
    pub opacity: f32,
    pub blend_mode: String,
    pub visible: bool,
}

/// A stack of render layers.
#[allow(dead_code)]
pub struct RenderLayerStack2 {
    pub layers: Vec<RenderLayer2>,
}

/// Create a new render layer.
#[allow(dead_code)]
pub fn new_render_layer2(name: &str, opacity: f32, blend_mode: &str) -> RenderLayer2 {
    RenderLayer2 { name: name.to_string(), opacity: opacity.clamp(0.0, 1.0), blend_mode: blend_mode.to_string(), visible: true }
}

/// Add a render layer to the stack.
#[allow(dead_code)]
pub fn add_render_layer2(stack: &mut RenderLayerStack2, layer: RenderLayer2) {
    stack.layers.push(layer);
}

/// Export the layer stack to JSON.
#[allow(dead_code)]
pub fn export_layer_stack2(stack: &RenderLayerStack2) -> String {
    let layers: Vec<String> = stack.layers.iter().map(|l| {
        format!(r#"{{"name":"{}","opacity":{},"blend":"{}","visible":{}}}"#, l.name, l.opacity, l.blend_mode, l.visible)
    }).collect();
    format!("[{}]", layers.join(","))
}

/// Get layer count.
#[allow(dead_code)]
pub fn layer2_count(stack: &RenderLayerStack2) -> usize { stack.layers.len() }

/// Get layer name at index.
#[allow(dead_code)]
pub fn layer2_name(stack: &RenderLayerStack2, i: usize) -> &str {
    stack.layers.get(i).map(|l| l.name.as_str()).unwrap_or("")
}

/// Get layer opacity at index.
#[allow(dead_code)]
pub fn layer2_opacity(stack: &RenderLayerStack2, i: usize) -> f32 {
    stack.layers.get(i).map(|l| l.opacity).unwrap_or(0.0)
}

/// Get layer blend mode at index.
#[allow(dead_code)]
pub fn layer2_blend_mode(stack: &RenderLayerStack2, i: usize) -> &str {
    stack.layers.get(i).map(|l| l.blend_mode.as_str()).unwrap_or("normal")
}

/// Convert layer stack to JSON.
#[allow(dead_code)]
pub fn layer_stack2_to_json(stack: &RenderLayerStack2) -> String { export_layer_stack2(stack) }

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_stack() -> RenderLayerStack2 { RenderLayerStack2 { layers: Vec::new() } }

    #[test]
    fn test_new_render_layer() {
        let l = new_render_layer2("diffuse", 1.0, "normal");
        assert_eq!(l.name, "diffuse");
    }

    #[test]
    fn test_add_layer() {
        let mut s = empty_stack();
        add_render_layer2(&mut s, new_render_layer2("ambient", 0.5, "add"));
        assert_eq!(layer2_count(&s), 1);
    }

    #[test]
    fn test_layer_name() {
        let mut s = empty_stack();
        add_render_layer2(&mut s, new_render_layer2("spec", 1.0, "multiply"));
        assert_eq!(layer2_name(&s, 0), "spec");
    }

    #[test]
    fn test_layer_opacity() {
        let mut s = empty_stack();
        add_render_layer2(&mut s, new_render_layer2("ao", 0.7, "normal"));
        assert!((layer2_opacity(&s, 0) - 0.7).abs() < 1e-5);
    }

    #[test]
    fn test_layer_blend_mode() {
        let mut s = empty_stack();
        add_render_layer2(&mut s, new_render_layer2("shadow", 1.0, "multiply"));
        assert_eq!(layer2_blend_mode(&s, 0), "multiply");
    }

    #[test]
    fn test_export_layer_stack_json() {
        let s = empty_stack();
        let j = export_layer_stack2(&s);
        assert_eq!(j, "[]");
    }

    #[test]
    fn test_layer_name_oob() {
        let s = empty_stack();
        assert_eq!(layer2_name(&s, 99), "");
    }

    #[test]
    fn test_layer_opacity_clamped() {
        let l = new_render_layer2("x", 2.0, "normal");
        assert!((l.opacity - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_layer_stack_to_json_with_layer() {
        let mut s = empty_stack();
        add_render_layer2(&mut s, new_render_layer2("color", 1.0, "normal"));
        let j = layer_stack2_to_json(&s);
        assert!(j.contains("color"));
    }
}
