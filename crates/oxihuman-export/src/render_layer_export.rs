// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Render layer/pass export.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum RenderPassType {
    Beauty,
    Diffuse,
    Specular,
    Shadow,
    Ao,
    Depth,
    Normal,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderLayer {
    pub name: String,
    pub passes: Vec<RenderPassType>,
    pub samples: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderLayerExport {
    pub layers: Vec<RenderLayer>,
}

#[allow(dead_code)]
pub fn new_render_layer_export() -> RenderLayerExport {
    RenderLayerExport { layers: Vec::new() }
}

#[allow(dead_code)]
pub fn rl_add_layer(export: &mut RenderLayerExport, layer: RenderLayer) {
    export.layers.push(layer);
}

#[allow(dead_code)]
pub fn rl_layer_count(export: &RenderLayerExport) -> usize {
    export.layers.len()
}

#[allow(dead_code)]
pub fn rl_get_layer<'a>(export: &'a RenderLayerExport, name: &str) -> Option<&'a RenderLayer> {
    export.layers.iter().find(|l| l.name == name)
}

#[allow(dead_code)]
pub fn rl_remove_layer(export: &mut RenderLayerExport, name: &str) {
    export.layers.retain(|l| l.name != name);
}

#[allow(dead_code)]
pub fn rl_total_passes(export: &RenderLayerExport) -> usize {
    export.layers.iter().map(|l| l.passes.len()).sum()
}

#[allow(dead_code)]
pub fn rl_pass_type_name(pass: &RenderPassType) -> &'static str {
    match pass {
        RenderPassType::Beauty => "beauty",
        RenderPassType::Diffuse => "diffuse",
        RenderPassType::Specular => "specular",
        RenderPassType::Shadow => "shadow",
        RenderPassType::Ao => "ao",
        RenderPassType::Depth => "depth",
        RenderPassType::Normal => "normal",
    }
}

#[allow(dead_code)]
pub fn rl_to_json(export: &RenderLayerExport) -> String {
    format!(
        "{{\"layer_count\":{},\"total_passes\":{}}}",
        export.layers.len(),
        rl_total_passes(export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_layer(name: &str) -> RenderLayer {
        RenderLayer {
            name: name.to_string(),
            passes: vec![RenderPassType::Beauty, RenderPassType::Diffuse],
            samples: 128,
        }
    }

    #[test]
    fn test_new_export() {
        let exp = new_render_layer_export();
        assert_eq!(rl_layer_count(&exp), 0);
    }

    #[test]
    fn test_add_layer() {
        let mut exp = new_render_layer_export();
        rl_add_layer(&mut exp, make_layer("main"));
        assert_eq!(rl_layer_count(&exp), 1);
    }

    #[test]
    fn test_get_layer() {
        let mut exp = new_render_layer_export();
        rl_add_layer(&mut exp, make_layer("bg"));
        assert!(rl_get_layer(&exp, "bg").is_some());
        assert!(rl_get_layer(&exp, "fg").is_none());
    }

    #[test]
    fn test_remove_layer() {
        let mut exp = new_render_layer_export();
        rl_add_layer(&mut exp, make_layer("a"));
        rl_add_layer(&mut exp, make_layer("b"));
        rl_remove_layer(&mut exp, "a");
        assert_eq!(rl_layer_count(&exp), 1);
    }

    #[test]
    fn test_total_passes() {
        let mut exp = new_render_layer_export();
        rl_add_layer(&mut exp, make_layer("l1")); // 2 passes
        rl_add_layer(&mut exp, make_layer("l2")); // 2 passes
        assert_eq!(rl_total_passes(&exp), 4);
    }

    #[test]
    fn test_pass_type_name() {
        assert_eq!(rl_pass_type_name(&RenderPassType::Beauty), "beauty");
        assert_eq!(rl_pass_type_name(&RenderPassType::Ao), "ao");
    }

    #[test]
    fn test_to_json() {
        let exp = new_render_layer_export();
        let j = rl_to_json(&exp);
        assert!(j.contains("layer_count"));
    }

    #[test]
    fn test_pass_eq() {
        assert_eq!(RenderPassType::Depth, RenderPassType::Depth);
        assert_ne!(RenderPassType::Shadow, RenderPassType::Normal);
    }
}
