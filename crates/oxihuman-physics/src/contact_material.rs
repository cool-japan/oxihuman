//! Material contact properties table (friction/restitution pairs).

#[allow(dead_code)]
#[derive(Clone)]
pub struct ContactProps {
    pub static_friction: f32,
    pub dynamic_friction: f32,
    pub restitution: f32,
    pub rolling_friction: f32,
}

#[allow(dead_code)]
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PhysicsMaterial {
    Metal,
    Wood,
    Rubber,
    Glass,
    Concrete,
    Cloth,
    Foam,
    Stone,
    Ice,
    Mud,
}

#[allow(dead_code)]
pub struct ContactMaterialTable {
    pub entries: Vec<(PhysicsMaterial, PhysicsMaterial, ContactProps)>,
}

#[allow(dead_code)]
pub fn default_contact_props() -> ContactProps {
    ContactProps {
        static_friction: 0.5,
        dynamic_friction: 0.4,
        restitution: 0.3,
        rolling_friction: 0.01,
    }
}

#[allow(dead_code)]
pub fn new_contact_table() -> ContactMaterialTable {
    ContactMaterialTable {
        entries: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_contact_pair(
    table: &mut ContactMaterialTable,
    a: PhysicsMaterial,
    b: PhysicsMaterial,
    props: ContactProps,
) {
    table.entries.push((a, b, props));
}

/// Look up contact properties for a material pair; falls back to defaults.
#[allow(dead_code)]
pub fn lookup_contact(
    table: &ContactMaterialTable,
    a: &PhysicsMaterial,
    b: &PhysicsMaterial,
) -> ContactProps {
    for (ma, mb, props) in &table.entries {
        if (ma == a && mb == b) || (ma == b && mb == a) {
            return props.clone();
        }
    }
    // Fall back to combined individual material properties
    ContactProps {
        static_friction: combine_friction(material_friction(a), material_friction(b)),
        dynamic_friction: combine_friction(material_friction(a), material_friction(b)) * 0.9,
        restitution: combine_restitution(material_restitution(a), material_restitution(b)),
        rolling_friction: 0.01,
    }
}

/// Build a default table with common material pairs.
#[allow(dead_code)]
pub fn default_material_table() -> ContactMaterialTable {
    let mut table = new_contact_table();

    // Metal-Metal
    add_contact_pair(
        &mut table,
        PhysicsMaterial::Metal,
        PhysicsMaterial::Metal,
        ContactProps {
            static_friction: 0.6,
            dynamic_friction: 0.4,
            restitution: 0.2,
            rolling_friction: 0.002,
        },
    );
    // Rubber-Concrete
    add_contact_pair(
        &mut table,
        PhysicsMaterial::Rubber,
        PhysicsMaterial::Concrete,
        ContactProps {
            static_friction: 0.9,
            dynamic_friction: 0.8,
            restitution: 0.4,
            rolling_friction: 0.02,
        },
    );
    // Ice-Ice
    add_contact_pair(
        &mut table,
        PhysicsMaterial::Ice,
        PhysicsMaterial::Ice,
        ContactProps {
            static_friction: 0.05,
            dynamic_friction: 0.03,
            restitution: 0.1,
            rolling_friction: 0.001,
        },
    );
    // Wood-Wood
    add_contact_pair(
        &mut table,
        PhysicsMaterial::Wood,
        PhysicsMaterial::Wood,
        ContactProps {
            static_friction: 0.4,
            dynamic_friction: 0.3,
            restitution: 0.3,
            rolling_friction: 0.01,
        },
    );
    // Glass-Glass
    add_contact_pair(
        &mut table,
        PhysicsMaterial::Glass,
        PhysicsMaterial::Glass,
        ContactProps {
            static_friction: 0.4,
            dynamic_friction: 0.35,
            restitution: 0.55,
            rolling_friction: 0.003,
        },
    );
    // Stone-Stone
    add_contact_pair(
        &mut table,
        PhysicsMaterial::Stone,
        PhysicsMaterial::Stone,
        ContactProps {
            static_friction: 0.7,
            dynamic_friction: 0.6,
            restitution: 0.15,
            rolling_friction: 0.01,
        },
    );

    table
}

/// Restitution coefficient for a single material.
#[allow(dead_code)]
pub fn material_restitution(mat: &PhysicsMaterial) -> f32 {
    match mat {
        PhysicsMaterial::Metal => 0.2,
        PhysicsMaterial::Wood => 0.3,
        PhysicsMaterial::Rubber => 0.8,
        PhysicsMaterial::Glass => 0.55,
        PhysicsMaterial::Concrete => 0.1,
        PhysicsMaterial::Cloth => 0.05,
        PhysicsMaterial::Foam => 0.1,
        PhysicsMaterial::Stone => 0.15,
        PhysicsMaterial::Ice => 0.1,
        PhysicsMaterial::Mud => 0.02,
    }
}

/// Friction coefficient for a single material.
#[allow(dead_code)]
pub fn material_friction(mat: &PhysicsMaterial) -> f32 {
    match mat {
        PhysicsMaterial::Metal => 0.5,
        PhysicsMaterial::Wood => 0.4,
        PhysicsMaterial::Rubber => 0.9,
        PhysicsMaterial::Glass => 0.35,
        PhysicsMaterial::Concrete => 0.7,
        PhysicsMaterial::Cloth => 0.6,
        PhysicsMaterial::Foam => 0.55,
        PhysicsMaterial::Stone => 0.65,
        PhysicsMaterial::Ice => 0.05,
        PhysicsMaterial::Mud => 0.8,
    }
}

/// Geometric mean: sqrt(a * b).
#[allow(dead_code)]
pub fn combine_restitution(a: f32, b: f32) -> f32 {
    (a * b).sqrt()
}

/// Geometric mean of friction: sqrt(a * b).
#[allow(dead_code)]
pub fn combine_friction(a: f32, b: f32) -> f32 {
    (a * b).sqrt()
}

#[allow(dead_code)]
pub fn physics_material_name(mat: &PhysicsMaterial) -> &'static str {
    match mat {
        PhysicsMaterial::Metal => "Metal",
        PhysicsMaterial::Wood => "Wood",
        PhysicsMaterial::Rubber => "Rubber",
        PhysicsMaterial::Glass => "Glass",
        PhysicsMaterial::Concrete => "Concrete",
        PhysicsMaterial::Cloth => "Cloth",
        PhysicsMaterial::Foam => "Foam",
        PhysicsMaterial::Stone => "Stone",
        PhysicsMaterial::Ice => "Ice",
        PhysicsMaterial::Mud => "Mud",
    }
}

#[allow(dead_code)]
pub fn all_physics_materials() -> Vec<PhysicsMaterial> {
    vec![
        PhysicsMaterial::Metal,
        PhysicsMaterial::Wood,
        PhysicsMaterial::Rubber,
        PhysicsMaterial::Glass,
        PhysicsMaterial::Concrete,
        PhysicsMaterial::Cloth,
        PhysicsMaterial::Foam,
        PhysicsMaterial::Stone,
        PhysicsMaterial::Ice,
        PhysicsMaterial::Mud,
    ]
}

#[allow(dead_code)]
pub fn contact_pair_count(table: &ContactMaterialTable) -> usize {
    table.entries.len()
}

/// Density in kg/m³.
#[allow(dead_code)]
pub fn material_density(mat: &PhysicsMaterial) -> f32 {
    match mat {
        PhysicsMaterial::Metal => 7800.0,
        PhysicsMaterial::Wood => 600.0,
        PhysicsMaterial::Rubber => 1200.0,
        PhysicsMaterial::Glass => 2500.0,
        PhysicsMaterial::Concrete => 2300.0,
        PhysicsMaterial::Cloth => 300.0,
        PhysicsMaterial::Foam => 50.0,
        PhysicsMaterial::Stone => 2700.0,
        PhysicsMaterial::Ice => 917.0,
        PhysicsMaterial::Mud => 1800.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_contact_props() {
        let p = default_contact_props();
        assert!(p.static_friction >= 0.0);
        assert!(p.restitution >= 0.0);
    }

    #[test]
    fn test_new_contact_table_empty() {
        let table = new_contact_table();
        assert_eq!(contact_pair_count(&table), 0);
    }

    #[test]
    fn test_add_contact_pair() {
        let mut table = new_contact_table();
        add_contact_pair(
            &mut table,
            PhysicsMaterial::Metal,
            PhysicsMaterial::Wood,
            default_contact_props(),
        );
        assert_eq!(contact_pair_count(&table), 1);
    }

    #[test]
    fn test_lookup_contact_found() {
        let mut table = new_contact_table();
        let props = ContactProps {
            static_friction: 0.7,
            dynamic_friction: 0.6,
            restitution: 0.25,
            rolling_friction: 0.01,
        };
        add_contact_pair(
            &mut table,
            PhysicsMaterial::Metal,
            PhysicsMaterial::Wood,
            props,
        );
        let found = lookup_contact(&table, &PhysicsMaterial::Metal, &PhysicsMaterial::Wood);
        assert!((found.static_friction - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_lookup_contact_symmetric() {
        let mut table = new_contact_table();
        let props = ContactProps {
            static_friction: 0.7,
            dynamic_friction: 0.6,
            restitution: 0.25,
            rolling_friction: 0.01,
        };
        add_contact_pair(
            &mut table,
            PhysicsMaterial::Metal,
            PhysicsMaterial::Wood,
            props,
        );
        // Should also find it when order is reversed
        let found = lookup_contact(&table, &PhysicsMaterial::Wood, &PhysicsMaterial::Metal);
        assert!((found.static_friction - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_lookup_contact_fallback() {
        let table = new_contact_table();
        let found = lookup_contact(&table, &PhysicsMaterial::Rubber, &PhysicsMaterial::Ice);
        // Should return something sensible from material properties
        assert!(found.restitution >= 0.0);
    }

    #[test]
    fn test_default_material_table() {
        let table = default_material_table();
        assert!(contact_pair_count(&table) > 0);
    }

    #[test]
    fn test_material_restitution_rubber_high() {
        let r = material_restitution(&PhysicsMaterial::Rubber);
        assert!(r > 0.5, "Rubber should be bouncy");
    }

    #[test]
    fn test_material_friction_ice_low() {
        let f = material_friction(&PhysicsMaterial::Ice);
        assert!(f < 0.1, "Ice should have very low friction");
    }

    #[test]
    fn test_combine_restitution_geometric_mean() {
        let combined = combine_restitution(0.25, 1.0);
        assert!((combined - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_combine_friction_geometric_mean() {
        let combined = combine_friction(0.25, 1.0);
        assert!((combined - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_physics_material_name() {
        assert_eq!(physics_material_name(&PhysicsMaterial::Metal), "Metal");
        assert_eq!(physics_material_name(&PhysicsMaterial::Ice), "Ice");
        assert_eq!(physics_material_name(&PhysicsMaterial::Mud), "Mud");
    }

    #[test]
    fn test_all_physics_materials_count() {
        let mats = all_physics_materials();
        assert_eq!(mats.len(), 10);
    }

    #[test]
    fn test_material_density_metal_heavy() {
        let d = material_density(&PhysicsMaterial::Metal);
        assert!(d > 7000.0, "Metal should be dense");
    }

    #[test]
    fn test_material_density_foam_light() {
        let d = material_density(&PhysicsMaterial::Foam);
        assert!(d < 100.0, "Foam should be light");
    }

    #[test]
    fn test_contact_pair_count_after_default_table() {
        let table = default_material_table();
        assert!(contact_pair_count(&table) >= 6);
    }
}
