// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

/// A physics material combining surface and mass properties.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct PhysicsMaterial {
    pub name: String,
    pub density: f32,
    pub static_friction: f32,
    pub dynamic_friction: f32,
    pub restitution: f32,
    pub is_trigger: bool,
}

/// Create a default physics material (generic solid).
#[allow(dead_code)]
pub fn default_physics_material() -> PhysicsMaterial {
    PhysicsMaterial {
        name: "default".to_string(),
        density: 1000.0,
        static_friction: 0.5,
        dynamic_friction: 0.4,
        restitution: 0.3,
        is_trigger: false,
    }
}

/// Wood material: medium density, medium friction.
#[allow(dead_code)]
pub fn physics_material_wood() -> PhysicsMaterial {
    PhysicsMaterial {
        name: "wood".to_string(),
        density: 600.0,
        static_friction: 0.4,
        dynamic_friction: 0.35,
        restitution: 0.2,
        is_trigger: false,
    }
}

/// Rubber material: low density, high friction, high restitution.
#[allow(dead_code)]
pub fn physics_material_rubber() -> PhysicsMaterial {
    PhysicsMaterial {
        name: "rubber".to_string(),
        density: 1200.0,
        static_friction: 0.8,
        dynamic_friction: 0.7,
        restitution: 0.7,
        is_trigger: false,
    }
}

/// Ice material: very low friction, medium restitution.
#[allow(dead_code)]
pub fn physics_material_ice() -> PhysicsMaterial {
    PhysicsMaterial {
        name: "ice".to_string(),
        density: 917.0,
        static_friction: 0.05,
        dynamic_friction: 0.03,
        restitution: 0.1,
        is_trigger: false,
    }
}

/// Compute the combined restitution of two materials (geometric mean).
#[allow(dead_code)]
pub fn combined_restitution(a: &PhysicsMaterial, b: &PhysicsMaterial) -> f32 {
    (a.restitution * b.restitution).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_material_name() {
        let m = default_physics_material();
        assert_eq!(m.name, "default");
    }

    #[test]
    fn wood_lower_friction_than_rubber() {
        let w = physics_material_wood();
        let r = physics_material_rubber();
        assert!(w.static_friction < r.static_friction);
    }

    #[test]
    fn ice_lowest_friction() {
        let ice = physics_material_ice();
        let wood = physics_material_wood();
        assert!(ice.static_friction < wood.static_friction);
    }

    #[test]
    fn combined_restitution_geometric_mean() {
        let a = PhysicsMaterial {
            name: "a".to_string(),
            density: 1.0,
            static_friction: 0.5,
            dynamic_friction: 0.4,
            restitution: 0.4,
            is_trigger: false,
        };
        let b = PhysicsMaterial {
            name: "b".to_string(),
            density: 1.0,
            static_friction: 0.5,
            dynamic_friction: 0.4,
            restitution: 0.9,
            is_trigger: false,
        };
        let r = combined_restitution(&a, &b);
        let expected = (0.4f32 * 0.9).sqrt();
        assert!((r - expected).abs() < 1e-5);
    }

    #[test]
    fn combined_restitution_same_material() {
        let m = physics_material_rubber();
        let r = combined_restitution(&m, &m);
        assert!((r - m.restitution).abs() < 1e-5);
    }

    #[test]
    fn not_a_trigger_by_default() {
        let m = default_physics_material();
        assert!(!m.is_trigger);
    }

    #[test]
    fn rubber_higher_restitution_than_wood() {
        let w = physics_material_wood();
        let r = physics_material_rubber();
        assert!(r.restitution > w.restitution);
    }

    #[test]
    fn density_positive() {
        assert!(default_physics_material().density > 0.0);
        assert!(physics_material_wood().density > 0.0);
        assert!(physics_material_rubber().density > 0.0);
        assert!(physics_material_ice().density > 0.0);
    }

    #[test]
    fn restitution_in_range() {
        let m = default_physics_material();
        assert!((0.0..=1.0).contains(&m.restitution));
    }

    #[test]
    fn friction_nonnegative() {
        let m = physics_material_ice();
        assert!(m.static_friction >= 0.0);
        assert!(m.dynamic_friction >= 0.0);
    }
}
