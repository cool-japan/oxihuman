// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Render pass v2 — extended render pass descriptor with barriers and timestamps.

/// Pass type classification.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassTypeV2 {
    Geometry,
    Shadow,
    PostProcess,
    Compute,
    Blit,
}

/// A render pass v2 descriptor.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderPassV2 {
    pub name: String,
    pub pass_type: PassTypeV2,
    pub enabled: bool,
    pub timestamp_enabled: bool,
    pub draw_count: u32,
}

/// Ordered list of render passes.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct RenderPassListV2 {
    pub passes: Vec<RenderPassV2>,
}

#[allow(dead_code)]
pub fn new_render_pass_v2(name: &str, pass_type: PassTypeV2) -> RenderPassV2 {
    RenderPassV2 {
        name: name.to_string(),
        pass_type,
        enabled: true,
        timestamp_enabled: false,
        draw_count: 0,
    }
}

#[allow(dead_code)]
pub fn rpv2_pass_type_name(t: PassTypeV2) -> &'static str {
    match t {
        PassTypeV2::Geometry => "geometry",
        PassTypeV2::Shadow => "shadow",
        PassTypeV2::PostProcess => "post_process",
        PassTypeV2::Compute => "compute",
        PassTypeV2::Blit => "blit",
    }
}

#[allow(dead_code)]
pub fn rpv2_set_enabled(pass: &mut RenderPassV2, enabled: bool) {
    pass.enabled = enabled;
}

#[allow(dead_code)]
pub fn rpv2_set_timestamp(pass: &mut RenderPassV2, enabled: bool) {
    pass.timestamp_enabled = enabled;
}

#[allow(dead_code)]
pub fn rpv2_increment_draws(pass: &mut RenderPassV2, count: u32) {
    pass.draw_count += count;
}

#[allow(dead_code)]
pub fn rpl2_add(list: &mut RenderPassListV2, pass: RenderPassV2) {
    list.passes.push(pass);
}

#[allow(dead_code)]
pub fn rpl2_pass_count(list: &RenderPassListV2) -> usize {
    list.passes.len()
}

#[allow(dead_code)]
pub fn rpl2_enabled_count(list: &RenderPassListV2) -> usize {
    list.passes.iter().filter(|p| p.enabled).count()
}

#[allow(dead_code)]
pub fn rpl2_total_draws(list: &RenderPassListV2) -> u32 {
    list.passes.iter().map(|p| p.draw_count).sum()
}

#[allow(dead_code)]
pub fn rpl2_clear(list: &mut RenderPassListV2) {
    list.passes.clear();
}

#[allow(dead_code)]
pub fn rpl2_to_json(list: &RenderPassListV2) -> String {
    format!(
        r#"{{"pass_count":{},"enabled_count":{},"total_draws":{}}}"#,
        rpl2_pass_count(list),
        rpl2_enabled_count(list),
        rpl2_total_draws(list)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pass_enabled() {
        let p = new_render_pass_v2("geo", PassTypeV2::Geometry);
        assert!(p.enabled);
    }

    #[test]
    fn pass_type_name() {
        assert_eq!(rpv2_pass_type_name(PassTypeV2::Shadow), "shadow");
    }

    #[test]
    fn set_disabled() {
        let mut p = new_render_pass_v2("x", PassTypeV2::Blit);
        rpv2_set_enabled(&mut p, false);
        assert!(!p.enabled);
    }

    #[test]
    fn increment_draws() {
        let mut p = new_render_pass_v2("geo", PassTypeV2::Geometry);
        rpv2_increment_draws(&mut p, 10);
        assert_eq!(p.draw_count, 10);
    }

    #[test]
    fn list_empty() {
        let list = RenderPassListV2::default();
        assert_eq!(rpl2_pass_count(&list), 0);
    }

    #[test]
    fn list_add_pass() {
        let mut list = RenderPassListV2::default();
        rpl2_add(&mut list, new_render_pass_v2("geo", PassTypeV2::Geometry));
        assert_eq!(rpl2_pass_count(&list), 1);
    }

    #[test]
    fn list_enabled_count() {
        let mut list = RenderPassListV2::default();
        let mut p = new_render_pass_v2("x", PassTypeV2::Compute);
        rpv2_set_enabled(&mut p, false);
        rpl2_add(&mut list, p);
        rpl2_add(&mut list, new_render_pass_v2("y", PassTypeV2::Compute));
        assert_eq!(rpl2_enabled_count(&list), 1);
    }

    #[test]
    fn list_total_draws() {
        let mut list = RenderPassListV2::default();
        let mut p = new_render_pass_v2("geo", PassTypeV2::Geometry);
        rpv2_increment_draws(&mut p, 5);
        rpl2_add(&mut list, p);
        assert_eq!(rpl2_total_draws(&list), 5);
    }

    #[test]
    fn list_clear() {
        let mut list = RenderPassListV2::default();
        rpl2_add(&mut list, new_render_pass_v2("geo", PassTypeV2::Geometry));
        rpl2_clear(&mut list);
        assert_eq!(rpl2_pass_count(&list), 0);
    }

    #[test]
    fn to_json_fields() {
        let list = RenderPassListV2::default();
        let j = rpl2_to_json(&list);
        assert!(j.contains("pass_count"));
    }
}
