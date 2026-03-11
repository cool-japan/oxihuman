// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Decal projection renderer.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DecalConfig {
    pub size: [f32; 2],
    pub depth: f32,
    pub fade_angle: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Decal {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub texture_id: u32,
    pub config: DecalConfig,
    pub visible: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct DecalRenderer {
    decals: Vec<Decal>,
}

#[allow(dead_code)]
pub fn default_decal_config() -> DecalConfig {
    DecalConfig { size: [1.0, 1.0], depth: 0.1, fade_angle: 0.5 }
}

#[allow(dead_code)]
pub fn new_decal_renderer() -> DecalRenderer {
    DecalRenderer::default()
}

#[allow(dead_code)]
pub fn dr_add_decal(renderer: &mut DecalRenderer, decal: Decal) {
    renderer.decals.push(decal);
}

#[allow(dead_code)]
pub fn dr_remove_decal(renderer: &mut DecalRenderer, index: usize) {
    if index < renderer.decals.len() {
        renderer.decals.remove(index);
    }
}

#[allow(dead_code)]
pub fn dr_count(renderer: &DecalRenderer) -> usize {
    renderer.decals.len()
}

#[allow(dead_code)]
pub fn dr_clear(renderer: &mut DecalRenderer) {
    renderer.decals.clear();
}

#[allow(dead_code)]
pub fn dr_get(renderer: &DecalRenderer, index: usize) -> Option<&Decal> {
    renderer.decals.get(index)
}

#[allow(dead_code)]
pub fn dr_set_visible(renderer: &mut DecalRenderer, index: usize, visible: bool) {
    if let Some(d) = renderer.decals.get_mut(index) {
        d.visible = visible;
    }
}

#[allow(dead_code)]
pub fn dr_to_json(renderer: &DecalRenderer) -> String {
    format!(r#"{{"decal_count":{}}}"#, renderer.decals.len())
}

// ── New types required by task ─────────────────────────────────────────────

/// A fixed-capacity pool of decals (alias wrapper around `DecalRenderer`).
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct DecalPool {
    decals: Vec<Decal>,
    capacity: usize,
}

/// Create a new `DecalPool` with the given capacity.
#[allow(dead_code)]
pub fn new_decal_pool(capacity: usize) -> DecalPool {
    DecalPool { decals: Vec::with_capacity(capacity), capacity }
}

/// Add a decal to the pool (up to capacity).
#[allow(dead_code)]
pub fn add_decal(pool: &mut DecalPool, decal: Decal) -> bool {
    if pool.decals.len() < pool.capacity || pool.capacity == 0 {
        pool.decals.push(decal);
        true
    } else {
        false
    }
}

/// Return the number of decals in the pool.
#[allow(dead_code)]
pub fn decal_count(pool: &DecalPool) -> usize {
    pool.decals.len()
}

/// Return a reference to the decal at `index`.
#[allow(dead_code)]
pub fn decal_at(pool: &DecalPool, index: usize) -> Option<&Decal> {
    pool.decals.get(index)
}

/// Return an axis-aligned bounds `[min, max]` for a decal (simple centred box).
#[allow(dead_code)]
pub fn decal_bounds(decal: &Decal) -> [[f32; 3]; 2] {
    let hw = decal.config.size[0] / 2.0;
    let hh = decal.config.size[1] / 2.0;
    let hd = decal.config.depth / 2.0;
    let p = decal.position;
    [[p[0] - hw, p[1] - hh, p[2] - hd], [p[0] + hw, p[1] + hh, p[2] + hd]]
}

/// Project a world-space point onto the decal surface (stub returns uv=[0,0]).
#[allow(dead_code)]
pub fn project_decal(_decal: &Decal, _point: [f32; 3]) -> [f32; 2] {
    [0.0, 0.0]
}

/// Return the opacity of a decal (1.0 if visible, else 0.0).
#[allow(dead_code)]
pub fn decal_opacity(decal: &Decal) -> f32 {
    if decal.visible { 1.0 } else { 0.0 }
}

/// Remove the decal at `index` from the pool.
#[allow(dead_code)]
pub fn remove_decal(pool: &mut DecalPool, index: usize) {
    if index < pool.decals.len() {
        pool.decals.remove(index);
    }
}

fn make_test_decal() -> Decal {
    Decal {
        position: [0.0, 0.0, 0.0],
        normal: [0.0, 1.0, 0.0],
        texture_id: 0,
        config: default_decal_config(),
        visible: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_decal_config();
        assert!((cfg.size[0] - 1.0).abs() < 1e-6);
        assert!((cfg.depth - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_new_renderer_empty() {
        let r = new_decal_renderer();
        assert_eq!(dr_count(&r), 0);
    }

    #[test]
    fn test_add_decal() {
        let mut r = new_decal_renderer();
        dr_add_decal(&mut r, make_test_decal());
        assert_eq!(dr_count(&r), 1);
    }

    #[test]
    fn test_remove_decal() {
        let mut r = new_decal_renderer();
        dr_add_decal(&mut r, make_test_decal());
        dr_remove_decal(&mut r, 0);
        assert_eq!(dr_count(&r), 0);
    }

    #[test]
    fn test_clear() {
        let mut r = new_decal_renderer();
        dr_add_decal(&mut r, make_test_decal());
        dr_add_decal(&mut r, make_test_decal());
        dr_clear(&mut r);
        assert_eq!(dr_count(&r), 0);
    }

    #[test]
    fn test_get_decal() {
        let mut r = new_decal_renderer();
        dr_add_decal(&mut r, make_test_decal());
        let d = dr_get(&r, 0);
        assert!(d.is_some());
        assert!(dr_get(&r, 99).is_none());
    }

    #[test]
    fn test_set_visible() {
        let mut r = new_decal_renderer();
        dr_add_decal(&mut r, make_test_decal());
        dr_set_visible(&mut r, 0, false);
        assert!(!dr_get(&r, 0).unwrap().visible);
    }

    #[test]
    fn test_to_json() {
        let r = new_decal_renderer();
        let j = dr_to_json(&r);
        assert!(j.contains("decal_count"));
    }

    #[test]
    fn test_new_decal_pool_empty() {
        let pool = new_decal_pool(10);
        assert_eq!(decal_count(&pool), 0);
    }

    #[test]
    fn test_add_decal_pool() {
        let mut pool = new_decal_pool(5);
        add_decal(&mut pool, make_test_decal());
        assert_eq!(decal_count(&pool), 1);
    }

    #[test]
    fn test_decal_at_some() {
        let mut pool = new_decal_pool(5);
        add_decal(&mut pool, make_test_decal());
        assert!(decal_at(&pool, 0).is_some());
    }

    #[test]
    fn test_decal_at_none() {
        let pool = new_decal_pool(5);
        assert!(decal_at(&pool, 0).is_none());
    }

    #[test]
    fn test_decal_opacity_visible() {
        let d = make_test_decal();
        assert!((decal_opacity(&d) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_decal_opacity_hidden() {
        let mut d = make_test_decal();
        d.visible = false;
        assert!((decal_opacity(&d)).abs() < 1e-6);
    }

    #[test]
    fn test_remove_decal_from_pool() {
        let mut pool = new_decal_pool(5);
        add_decal(&mut pool, make_test_decal());
        remove_decal(&mut pool, 0);
        assert_eq!(decal_count(&pool), 0);
    }

    #[test]
    fn test_decal_bounds_nonzero_size() {
        let d = make_test_decal();
        let bounds = decal_bounds(&d);
        assert!(bounds[1][0] > bounds[0][0]);
    }
}
