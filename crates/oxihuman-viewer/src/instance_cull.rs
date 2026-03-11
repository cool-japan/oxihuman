// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Instance-level frustum and occlusion culling helpers.

/// An axis-aligned bounding box.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct Aabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl Aabb {
    #[allow(dead_code)]
    pub fn center(&self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }
    #[allow(dead_code)]
    pub fn extents(&self) -> [f32; 3] {
        [
            (self.max[0] - self.min[0]) * 0.5,
            (self.max[1] - self.min[1]) * 0.5,
            (self.max[2] - self.min[2]) * 0.5,
        ]
    }
}

/// A culling entry.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct CullEntry {
    pub id: u32,
    pub bounds: Aabb,
    pub visible: bool,
}

/// Cull manager.
#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct InstanceCull {
    pub entries: Vec<CullEntry>,
}

#[allow(dead_code)]
pub fn new_instance_cull() -> InstanceCull {
    InstanceCull::default()
}

#[allow(dead_code)]
pub fn ic_register(cull: &mut InstanceCull, id: u32, bounds: Aabb) {
    cull.entries.push(CullEntry {
        id,
        bounds,
        visible: true,
    });
}

#[allow(dead_code)]
pub fn ic_remove(cull: &mut InstanceCull, id: u32) {
    cull.entries.retain(|e| e.id != id);
}

#[allow(dead_code)]
pub fn ic_set_visible(cull: &mut InstanceCull, id: u32, v: bool) {
    if let Some(e) = cull.entries.iter_mut().find(|e| e.id == id) {
        e.visible = v;
    }
}

#[allow(dead_code)]
pub fn ic_visible_count(cull: &InstanceCull) -> usize {
    cull.entries.iter().filter(|e| e.visible).count()
}

/// Simple sphere-frustum cull (placeholder: cull by distance from origin).
#[allow(dead_code)]
pub fn ic_cull_by_distance(cull: &mut InstanceCull, max_dist: f32) {
    for e in &mut cull.entries {
        let c = e.bounds.center();
        let d = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
        e.visible = d <= max_dist;
    }
}

#[allow(dead_code)]
pub fn ic_clear(cull: &mut InstanceCull) {
    cull.entries.clear();
}

#[allow(dead_code)]
pub fn ic_count(cull: &InstanceCull) -> usize {
    cull.entries.len()
}

#[allow(dead_code)]
pub fn ic_to_json(cull: &InstanceCull) -> String {
    format!(
        "{{\"total\":{},\"visible\":{}}}",
        ic_count(cull),
        ic_visible_count(cull)
    )
}

#[allow(dead_code)]
pub fn ic_aabb_volume(b: &Aabb) -> f32 {
    let e = b.extents();
    8.0 * e[0] * e[1] * e[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_aabb(s: f32) -> Aabb {
        Aabb {
            min: [-s, -s, -s],
            max: [s, s, s],
        }
    }

    #[test]
    fn new_empty() {
        assert_eq!(ic_count(&new_instance_cull()), 0);
    }

    #[test]
    fn register() {
        let mut c = new_instance_cull();
        ic_register(&mut c, 1, mk_aabb(1.0));
        assert_eq!(ic_count(&c), 1);
    }

    #[test]
    fn remove() {
        let mut c = new_instance_cull();
        ic_register(&mut c, 1, mk_aabb(1.0));
        ic_remove(&mut c, 1);
        assert_eq!(ic_count(&c), 0);
    }

    #[test]
    fn set_not_visible() {
        let mut c = new_instance_cull();
        ic_register(&mut c, 1, mk_aabb(1.0));
        ic_set_visible(&mut c, 1, false);
        assert_eq!(ic_visible_count(&c), 0);
    }

    #[test]
    fn cull_by_distance() {
        let mut c = new_instance_cull();
        ic_register(
            &mut c,
            1,
            Aabb {
                min: [100.0, 0.0, 0.0],
                max: [102.0, 2.0, 2.0],
            },
        );
        ic_register(&mut c, 2, mk_aabb(0.5));
        ic_cull_by_distance(&mut c, 10.0);
        assert_eq!(ic_visible_count(&c), 1);
    }

    #[test]
    fn clear_empties() {
        let mut c = new_instance_cull();
        ic_register(&mut c, 1, mk_aabb(1.0));
        ic_clear(&mut c);
        assert_eq!(ic_count(&c), 0);
    }

    #[test]
    fn aabb_center() {
        let b = Aabb {
            min: [0.0, 0.0, 0.0],
            max: [2.0, 2.0, 2.0],
        };
        let ctr = b.center();
        assert!((ctr[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn aabb_volume() {
        let b = mk_aabb(1.0);
        assert!((ic_aabb_volume(&b) - 8.0).abs() < 1e-4);
    }

    #[test]
    fn json_has_total() {
        assert!(ic_to_json(&new_instance_cull()).contains("total"));
    }

    #[test]
    fn all_visible_by_default() {
        let mut c = new_instance_cull();
        ic_register(&mut c, 1, mk_aabb(1.0));
        ic_register(&mut c, 2, mk_aabb(2.0));
        assert_eq!(ic_visible_count(&c), 2);
    }
}
