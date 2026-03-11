#![allow(dead_code)]

/// Bind slot for textures.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BindSlot { Albedo, Normal, Roughness, Metallic, Emissive, Ao }

/// Manages texture binding to slots.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TextureBind {
    slots: Vec<(BindSlot, String)>,
}

#[allow(dead_code)]
pub fn new_texture_bind() -> TextureBind { TextureBind { slots: Vec::new() } }

#[allow(dead_code)]
pub fn bind_texture(tb: &mut TextureBind, slot: BindSlot, name: &str) {
    if let Some(e) = tb.slots.iter_mut().find(|(s, _)| *s == slot) {
        e.1 = name.to_string();
    } else {
        tb.slots.push((slot, name.to_string()));
    }
}

#[allow(dead_code)]
pub fn unbind_texture(tb: &mut TextureBind, slot: BindSlot) {
    tb.slots.retain(|(s, _)| *s != slot);
}

#[allow(dead_code)]
pub fn bound_texture_at(tb: &TextureBind, slot: BindSlot) -> Option<&str> {
    tb.slots.iter().find(|(s, _)| *s == slot).map(|(_, n)| n.as_str())
}

#[allow(dead_code)]
pub fn bind_count(tb: &TextureBind) -> usize { tb.slots.len() }

#[allow(dead_code)]
pub fn bind_slot_name(slot: BindSlot) -> &'static str {
    match slot { BindSlot::Albedo => "albedo", BindSlot::Normal => "normal", BindSlot::Roughness => "roughness", BindSlot::Metallic => "metallic", BindSlot::Emissive => "emissive", BindSlot::Ao => "ao" }
}

#[allow(dead_code)]
pub fn binds_to_json(tb: &TextureBind) -> String {
    let e: Vec<String> = tb.slots.iter().map(|(s, n)| format!("\"{}\":\"{}\"", bind_slot_name(*s), n)).collect();
    format!("{{{}}}", e.join(","))
}

#[allow(dead_code)]
pub fn clear_binds(tb: &mut TextureBind) { tb.slots.clear(); }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { assert_eq!(bind_count(&new_texture_bind()), 0); }
    #[test] fn test_bind() {
        let mut t = new_texture_bind();
        bind_texture(&mut t, BindSlot::Albedo, "diffuse.png");
        assert_eq!(bound_texture_at(&t, BindSlot::Albedo), Some("diffuse.png"));
    }
    #[test] fn test_unbind() {
        let mut t = new_texture_bind();
        bind_texture(&mut t, BindSlot::Normal, "n.png");
        unbind_texture(&mut t, BindSlot::Normal);
        assert!(bound_texture_at(&t, BindSlot::Normal).is_none());
    }
    #[test] fn test_count() {
        let mut t = new_texture_bind();
        bind_texture(&mut t, BindSlot::Albedo, "a"); bind_texture(&mut t, BindSlot::Normal, "n");
        assert_eq!(bind_count(&t), 2);
    }
    #[test] fn test_overwrite() {
        let mut t = new_texture_bind();
        bind_texture(&mut t, BindSlot::Albedo, "old");
        bind_texture(&mut t, BindSlot::Albedo, "new");
        assert_eq!(bound_texture_at(&t, BindSlot::Albedo), Some("new"));
        assert_eq!(bind_count(&t), 1);
    }
    #[test] fn test_slot_name() { assert_eq!(bind_slot_name(BindSlot::Roughness), "roughness"); }
    #[test] fn test_to_json() {
        let mut t = new_texture_bind();
        bind_texture(&mut t, BindSlot::Emissive, "e");
        assert!(binds_to_json(&t).contains("emissive"));
    }
    #[test] fn test_clear() {
        let mut t = new_texture_bind();
        bind_texture(&mut t, BindSlot::Ao, "ao");
        clear_binds(&mut t);
        assert_eq!(bind_count(&t), 0);
    }
    #[test] fn test_not_bound() { assert!(bound_texture_at(&new_texture_bind(), BindSlot::Metallic).is_none()); }
    #[test] fn test_unbind_missing() {
        let mut t = new_texture_bind();
        unbind_texture(&mut t, BindSlot::Albedo);
        assert_eq!(bind_count(&t), 0);
    }
}
