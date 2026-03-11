// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Render pass descriptor.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum RenderPassKindV3 {
    Geometry,
    Lighting,
    PostProcess,
    Shadow,
    UI,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderPassV3 {
    pub name: String,
    pub kind: RenderPassKindV3,
    pub enabled: bool,
    pub order: i32,
}

#[allow(dead_code)]
pub fn new_render_pass_v3(name: &str, kind: RenderPassKindV3, order: i32) -> RenderPassV3 {
    RenderPassV3 { name: name.to_string(), kind, enabled: true, order }
}

#[allow(dead_code)]
pub fn rpv3_enable(pass: &mut RenderPassV3) {
    pass.enabled = true;
}

#[allow(dead_code)]
pub fn rpv3_disable(pass: &mut RenderPassV3) {
    pass.enabled = false;
}

#[allow(dead_code)]
pub fn rpv3_is_enabled(pass: &RenderPassV3) -> bool {
    pass.enabled
}

#[allow(dead_code)]
pub fn rpv3_order(pass: &RenderPassV3) -> i32 {
    pass.order
}

#[allow(dead_code)]
pub fn rpv3_kind_name(pass: &RenderPassV3) -> &'static str {
    match pass.kind {
        RenderPassKindV3::Geometry => "Geometry",
        RenderPassKindV3::Lighting => "Lighting",
        RenderPassKindV3::PostProcess => "PostProcess",
        RenderPassKindV3::Shadow => "Shadow",
        RenderPassKindV3::UI => "UI",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_enabled_by_default() {
        let p = new_render_pass_v3("shadow", RenderPassKindV3::Shadow, 0);
        assert!(rpv3_is_enabled(&p));
    }

    #[test]
    fn test_disable() {
        let mut p = new_render_pass_v3("geometry", RenderPassKindV3::Geometry, 1);
        rpv3_disable(&mut p);
        assert!(!rpv3_is_enabled(&p));
    }

    #[test]
    fn test_enable_after_disable() {
        let mut p = new_render_pass_v3("ui", RenderPassKindV3::UI, 10);
        rpv3_disable(&mut p);
        rpv3_enable(&mut p);
        assert!(rpv3_is_enabled(&p));
    }

    #[test]
    fn test_order() {
        let p = new_render_pass_v3("lighting", RenderPassKindV3::Lighting, 5);
        assert_eq!(rpv3_order(&p), 5);
    }

    #[test]
    fn test_kind_name_geometry() {
        let p = new_render_pass_v3("g", RenderPassKindV3::Geometry, 0);
        assert_eq!(rpv3_kind_name(&p), "Geometry");
    }

    #[test]
    fn test_kind_name_shadow() {
        let p = new_render_pass_v3("s", RenderPassKindV3::Shadow, 0);
        assert_eq!(rpv3_kind_name(&p), "Shadow");
    }

    #[test]
    fn test_kind_name_post_process() {
        let p = new_render_pass_v3("pp", RenderPassKindV3::PostProcess, 0);
        assert_eq!(rpv3_kind_name(&p), "PostProcess");
    }

    #[test]
    fn test_kind_name_ui() {
        let p = new_render_pass_v3("ui", RenderPassKindV3::UI, 99);
        assert_eq!(rpv3_kind_name(&p), "UI");
    }
}
