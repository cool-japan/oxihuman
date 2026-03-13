// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Billboard/sprite rendering.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SpriteConfig {
    pub size: [f32; 2],
    pub always_face_camera: bool,
    pub sort_by_depth: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Sprite {
    pub position: [f32; 3],
    pub texture_id: u32,
    pub color: [f32; 4],
    pub config: SpriteConfig,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SpriteRenderer {
    sprites: Vec<Sprite>,
}

#[allow(dead_code)]
pub fn default_sprite_config() -> SpriteConfig {
    SpriteConfig {
        size: [1.0, 1.0],
        always_face_camera: true,
        sort_by_depth: true,
    }
}

#[allow(dead_code)]
pub fn new_sprite_renderer() -> SpriteRenderer {
    SpriteRenderer { sprites: Vec::new() }
}

#[allow(dead_code)]
pub fn sprite_add(renderer: &mut SpriteRenderer, sprite: Sprite) {
    renderer.sprites.push(sprite);
}

/// Remove sprite at index. Returns true on success.
#[allow(dead_code)]
pub fn sprite_remove(renderer: &mut SpriteRenderer, index: usize) -> bool {
    if index < renderer.sprites.len() {
        renderer.sprites.remove(index);
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn sprite_count(renderer: &SpriteRenderer) -> usize {
    renderer.sprites.len()
}

#[allow(dead_code)]
pub fn sprite_clear(renderer: &mut SpriteRenderer) {
    renderer.sprites.clear();
}

#[allow(dead_code)]
pub fn sprite_get(renderer: &SpriteRenderer, index: usize) -> Option<&Sprite> {
    renderer.sprites.get(index)
}

#[allow(dead_code)]
pub fn sprite_to_json(renderer: &SpriteRenderer) -> String {
    let entries: Vec<String> = renderer
        .sprites
        .iter()
        .map(|s| {
            format!(
                r#"{{"texture_id":{},"pos":[{:.3},{:.3},{:.3}],"size":[{:.3},{:.3}]}}"#,
                s.texture_id,
                s.position[0], s.position[1], s.position[2],
                s.config.size[0], s.config.size[1]
            )
        })
        .collect();
    format!("[{}]", entries.join(","))
}

fn make_sprite(texture_id: u32, pos: [f32; 3]) -> Sprite {
    Sprite {
        position: pos,
        texture_id,
        color: [1.0, 1.0, 1.0, 1.0],
        config: default_sprite_config(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_sprite_config();
        assert!(cfg.always_face_camera);
        assert!(cfg.sort_by_depth);
        assert!((cfg.size[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_renderer_empty() {
        let r = new_sprite_renderer();
        assert_eq!(sprite_count(&r), 0);
    }

    #[test]
    fn test_add_sprite() {
        let mut r = new_sprite_renderer();
        sprite_add(&mut r, make_sprite(1, [0.0, 0.0, 0.0]));
        assert_eq!(sprite_count(&r), 1);
    }

    #[test]
    fn test_remove_valid() {
        let mut r = new_sprite_renderer();
        sprite_add(&mut r, make_sprite(1, [0.0, 0.0, 0.0]));
        sprite_add(&mut r, make_sprite(2, [1.0, 0.0, 0.0]));
        assert!(sprite_remove(&mut r, 0));
        assert_eq!(sprite_count(&r), 1);
    }

    #[test]
    fn test_remove_invalid() {
        let mut r = new_sprite_renderer();
        assert!(!sprite_remove(&mut r, 0));
    }

    #[test]
    fn test_get_valid() {
        let mut r = new_sprite_renderer();
        sprite_add(&mut r, make_sprite(42, [0.0, 1.0, 0.0]));
        assert_eq!(sprite_get(&r, 0).expect("should succeed").texture_id, 42);
    }

    #[test]
    fn test_get_invalid() {
        let r = new_sprite_renderer();
        assert!(sprite_get(&r, 0).is_none());
    }

    #[test]
    fn test_clear() {
        let mut r = new_sprite_renderer();
        sprite_add(&mut r, make_sprite(1, [0.0, 0.0, 0.0]));
        sprite_clear(&mut r);
        assert_eq!(sprite_count(&r), 0);
    }

    #[test]
    fn test_to_json_empty() {
        let r = new_sprite_renderer();
        assert_eq!(sprite_to_json(&r), "[]");
    }

    #[test]
    fn test_to_json_has_texture_id() {
        let mut r = new_sprite_renderer();
        sprite_add(&mut r, make_sprite(7, [0.0, 0.0, 0.0]));
        let j = sprite_to_json(&r);
        assert!(j.contains("texture_id"));
        assert!(j.contains('7'));
    }
}
