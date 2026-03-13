// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! GLTF KHR_physics_rigid_bodies extension stubs.

#[allow(dead_code)]
/// Shape type for a physics rigid body.
#[derive(Debug, Clone, PartialEq)]
pub struct PhysicsShape {
    /// "box", "sphere", "capsule", or "convex"
    pub shape_type: String,
    pub size: [f32; 3],
    pub radius: f32,
    pub height: f32,
}

#[allow(dead_code)]
/// Descriptor for a rigid body node.
#[derive(Debug, Clone, PartialEq)]
pub struct RigidBodyDescriptor {
    pub mass: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub is_trigger: bool,
    pub shape: PhysicsShape,
}

#[allow(dead_code)]
/// Descriptor for a physics joint between two nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct PhysicsJointDescriptor {
    /// "hinge", "ball", "slider", "fixed"
    pub joint_type: String,
    pub node_a: usize,
    pub node_b: usize,
    pub limits: Option<[f32; 2]>,
}

#[allow(dead_code)]
/// A full physics scene: rigid bodies + joints.
#[derive(Debug, Clone, Default)]
pub struct GltfPhysicsScene {
    /// (node_index, descriptor) pairs
    pub rigid_bodies: Vec<(usize, RigidBodyDescriptor)>,
    pub joints: Vec<PhysicsJointDescriptor>,
}

// ── Shape constructors ────────────────────────────────────────────────────────

/// Build a box shape with the given half extents.
pub fn build_box_shape(half_extents: [f32; 3]) -> PhysicsShape {
    PhysicsShape {
        shape_type: "box".to_string(),
        size: half_extents,
        radius: 0.0,
        height: 0.0,
    }
}

/// Build a sphere shape with the given radius.
pub fn build_sphere_shape(radius: f32) -> PhysicsShape {
    PhysicsShape {
        shape_type: "sphere".to_string(),
        size: [radius, radius, radius],
        radius,
        height: 0.0,
    }
}

/// Build a capsule shape with the given radius and height.
pub fn build_capsule_shape(radius: f32, height: f32) -> PhysicsShape {
    PhysicsShape {
        shape_type: "capsule".to_string(),
        size: [radius, height, radius],
        radius,
        height,
    }
}

// ── Body constructors ─────────────────────────────────────────────────────────

/// Create a default dynamic rigid body with the given mass.
pub fn default_rigid_body(mass: f32) -> RigidBodyDescriptor {
    RigidBodyDescriptor {
        mass,
        linear_damping: 0.05,
        angular_damping: 0.05,
        is_trigger: false,
        shape: build_box_shape([0.5, 0.5, 0.5]),
    }
}

/// Create a kinematic (static) body with mass = 0.
pub fn kinematic_body() -> RigidBodyDescriptor {
    RigidBodyDescriptor {
        mass: 0.0,
        linear_damping: 0.0,
        angular_damping: 0.0,
        is_trigger: false,
        shape: build_box_shape([1.0, 0.1, 1.0]),
    }
}

// ── Scene queries ─────────────────────────────────────────────────────────────

/// Return the number of rigid bodies in the scene.
pub fn rigid_body_count(scene: &GltfPhysicsScene) -> usize {
    scene.rigid_bodies.len()
}

/// Return the number of joints in the scene.
pub fn joint_count(scene: &GltfPhysicsScene) -> usize {
    scene.joints.len()
}

// ── Validation ────────────────────────────────────────────────────────────────

/// Validate the physics scene. Returns a list of error strings (empty = valid).
pub fn validate_physics_scene(scene: &GltfPhysicsScene) -> Vec<String> {
    let mut errors = Vec::new();
    for (i, (node_idx, body)) in scene.rigid_bodies.iter().enumerate() {
        if body.mass < 0.0 {
            errors.push(format!(
                "rigid_body[{}] node {}: negative mass {}",
                i, node_idx, body.mass
            ));
        }
        if body.linear_damping < 0.0 {
            errors.push(format!("rigid_body[{}]: negative linear_damping", i));
        }
        if body.angular_damping < 0.0 {
            errors.push(format!("rigid_body[{}]: negative angular_damping", i));
        }
        let valid_types = ["box", "sphere", "capsule", "convex"];
        if !valid_types.contains(&body.shape.shape_type.as_str()) {
            errors.push(format!(
                "rigid_body[{}]: unknown shape_type '{}'",
                i, body.shape.shape_type
            ));
        }
    }
    for (i, joint) in scene.joints.iter().enumerate() {
        if joint.node_a == joint.node_b {
            errors.push(format!("joint[{}]: node_a == node_b ({})", i, joint.node_a));
        }
        if let Some(lim) = joint.limits {
            if lim[0] > lim[1] {
                errors.push(format!("joint[{}]: limits[0] > limits[1]", i));
            }
        }
    }
    errors
}

// ── JSON serialisation ────────────────────────────────────────────────────────

/// Build the KHR_physics_rigid_bodies extension JSON string.
pub fn build_physics_extension_json(scene: &GltfPhysicsScene) -> String {
    let mut bodies_arr = String::from("[");
    for (i, (node_idx, body)) in scene.rigid_bodies.iter().enumerate() {
        if i > 0 {
            bodies_arr.push(',');
        }
        let shape_json = match body.shape.shape_type.as_str() {
            "sphere" => format!(
                r#"{{"type":"sphere","sphere":{{"radius":{}}}}}"#,
                body.shape.radius
            ),
            "capsule" => format!(
                r#"{{"type":"capsule","capsule":{{"radius":{},"height":{}}}}}"#,
                body.shape.radius, body.shape.height
            ),
            _ => format!(
                r#"{{"type":"box","box":{{"size":[{},{},{}]}}}}"#,
                body.shape.size[0], body.shape.size[1], body.shape.size[2]
            ),
        };
        bodies_arr.push_str(&format!(
            r#"{{"node":{},"mass":{},"linearDamping":{},"angularDamping":{},"isTrigger":{},"shape":{}}}"#,
            node_idx,
            body.mass,
            body.linear_damping,
            body.angular_damping,
            body.is_trigger,
            shape_json
        ));
    }
    bodies_arr.push(']');

    let mut joints_arr = String::from("[");
    for (i, joint) in scene.joints.iter().enumerate() {
        if i > 0 {
            joints_arr.push(',');
        }
        let limits_str = match joint.limits {
            Some(l) => format!(r#","limits":[{},{}]"#, l[0], l[1]),
            None => String::new(),
        };
        joints_arr.push_str(&format!(
            r#"{{"type":"{}","nodeA":{},"nodeB":{}{}}}"#,
            joint.joint_type, joint.node_a, joint.node_b, limits_str
        ));
    }
    joints_arr.push(']');

    format!(
        r#"{{"KHR_physics_rigid_bodies":{{"rigidBodies":{},"joints":{}}}}}"#,
        bodies_arr, joints_arr
    )
}

/// Inject the KHR_physics_rigid_bodies extension into an existing GLTF JSON string.
///
/// This is a stub that appends the physics data to the JSON. A full
/// implementation would parse and merge the JSON objects properly.
pub fn embed_physics_in_gltf(gltf_json: &str, physics: &GltfPhysicsScene) -> String {
    let physics_json = build_physics_extension_json(physics);
    // Stub: append physics block as a comment-style annotation inside the JSON.
    // In production this would use a proper JSON merge.
    if gltf_json.trim_end().ends_with('}') {
        let trimmed = gltf_json.trim_end().trim_end_matches('}');
        format!(r#"{}, "extensions": {}}}"#, trimmed, physics_json)
    } else {
        format!("{}\n/* physics: {} */", gltf_json, physics_json)
    }
}

// ── Biped ragdoll preset ──────────────────────────────────────────────────────

/// Build a standard biped ragdoll physics scene with `n_joints` joints.
///
/// The biped uses capsule bodies for limbs and box bodies for torso/head.
pub fn biped_physics_scene(n_joints: usize) -> GltfPhysicsScene {
    // Standard biped nodes: hips, spine, chest, neck, head,
    // upper_arm_L/R, lower_arm_L/R, hand_L/R,
    // upper_leg_L/R, lower_leg_L/R, foot_L/R  (15 bodies)
    let body_defs: &[(&str, f32, &str, f32, f32)] = &[
        // (name, mass, shape, radius_or_half, height)
        ("hips", 8.0, "box", 0.15, 0.0),
        ("spine", 6.0, "capsule", 0.10, 0.25),
        ("chest", 8.0, "box", 0.18, 0.0),
        ("neck", 1.5, "capsule", 0.05, 0.10),
        ("head", 5.0, "sphere", 0.12, 0.0),
        ("upper_arm_L", 2.0, "capsule", 0.05, 0.28),
        ("upper_arm_R", 2.0, "capsule", 0.05, 0.28),
        ("lower_arm_L", 1.5, "capsule", 0.04, 0.25),
        ("lower_arm_R", 1.5, "capsule", 0.04, 0.25),
        ("hand_L", 0.5, "box", 0.05, 0.0),
        ("hand_R", 0.5, "box", 0.05, 0.0),
        ("upper_leg_L", 5.0, "capsule", 0.07, 0.38),
        ("upper_leg_R", 5.0, "capsule", 0.07, 0.38),
        ("lower_leg_L", 3.0, "capsule", 0.05, 0.36),
        ("lower_leg_R", 3.0, "capsule", 0.05, 0.36),
    ];

    let mut rigid_bodies = Vec::new();
    for (idx, &(_, mass, shape_type, r, h)) in body_defs.iter().enumerate() {
        let shape = match shape_type {
            "sphere" => build_sphere_shape(r),
            "capsule" => build_capsule_shape(r, h),
            _ => build_box_shape([r, r * 0.6, r]),
        };
        rigid_bodies.push((
            idx,
            RigidBodyDescriptor {
                mass,
                linear_damping: 0.05,
                angular_damping: 0.08,
                is_trigger: false,
                shape,
            },
        ));
    }

    // Build joints between adjacent body parts (ball joints for a ragdoll)
    let joint_pairs: &[(usize, usize, &str)] = &[
        (0, 1, "ball"),    // hips → spine
        (1, 2, "ball"),    // spine → chest
        (2, 3, "ball"),    // chest → neck
        (3, 4, "ball"),    // neck → head
        (2, 5, "ball"),    // chest → upper_arm_L
        (2, 6, "ball"),    // chest → upper_arm_R
        (5, 7, "hinge"),   // upper_arm_L → lower_arm_L
        (6, 8, "hinge"),   // upper_arm_R → lower_arm_R
        (7, 9, "ball"),    // lower_arm_L → hand_L
        (8, 10, "ball"),   // lower_arm_R → hand_R
        (0, 11, "ball"),   // hips → upper_leg_L
        (0, 12, "ball"),   // hips → upper_leg_R
        (11, 13, "hinge"), // upper_leg_L → lower_leg_L
        (12, 14, "hinge"), // upper_leg_R → lower_leg_R
    ];

    let actual_joints = n_joints.min(joint_pairs.len());
    let joints: Vec<PhysicsJointDescriptor> = joint_pairs[..actual_joints]
        .iter()
        .map(|&(a, b, jt)| PhysicsJointDescriptor {
            joint_type: jt.to_string(),
            node_a: a,
            node_b: b,
            limits: None,
        })
        .collect();

    GltfPhysicsScene {
        rigid_bodies,
        joints,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_box_shape_fields() {
        let s = build_box_shape([1.0, 2.0, 3.0]);
        assert_eq!(s.shape_type, "box");
        assert_eq!(s.size, [1.0, 2.0, 3.0]);
        assert!((s.radius - 0.0).abs() < 1e-6);
    }

    #[test]
    fn build_sphere_shape_fields() {
        let s = build_sphere_shape(0.5);
        assert_eq!(s.shape_type, "sphere");
        assert!((s.radius - 0.5).abs() < 1e-6);
    }

    #[test]
    fn build_capsule_shape_fields() {
        let s = build_capsule_shape(0.1, 0.8);
        assert_eq!(s.shape_type, "capsule");
        assert!((s.radius - 0.1).abs() < 1e-6);
        assert!((s.height - 0.8).abs() < 1e-6);
    }

    #[test]
    fn default_rigid_body_mass() {
        let b = default_rigid_body(5.0);
        assert!((b.mass - 5.0).abs() < 1e-6);
        assert!(!b.is_trigger);
    }

    #[test]
    fn kinematic_body_mass_zero() {
        let b = kinematic_body();
        assert!((b.mass - 0.0).abs() < 1e-6);
    }

    #[test]
    fn rigid_body_count_correct() {
        let mut scene = GltfPhysicsScene::default();
        scene.rigid_bodies.push((0, default_rigid_body(1.0)));
        scene.rigid_bodies.push((1, default_rigid_body(2.0)));
        assert_eq!(rigid_body_count(&scene), 2);
    }

    #[test]
    fn joint_count_correct() {
        let mut scene = GltfPhysicsScene::default();
        scene.joints.push(PhysicsJointDescriptor {
            joint_type: "ball".to_string(),
            node_a: 0,
            node_b: 1,
            limits: None,
        });
        assert_eq!(joint_count(&scene), 1);
    }

    #[test]
    fn build_physics_extension_json_contains_key() {
        let scene = GltfPhysicsScene::default();
        let json = build_physics_extension_json(&scene);
        assert!(json.contains("KHR_physics_rigid_bodies"));
    }

    #[test]
    fn build_physics_extension_json_with_body() {
        let mut scene = GltfPhysicsScene::default();
        scene.rigid_bodies.push((0, default_rigid_body(10.0)));
        let json = build_physics_extension_json(&scene);
        assert!(json.contains("KHR_physics_rigid_bodies"));
        assert!(json.contains("rigidBodies"));
    }

    #[test]
    fn validate_empty_scene_no_errors() {
        let scene = GltfPhysicsScene::default();
        let errs = validate_physics_scene(&scene);
        assert!(
            errs.is_empty(),
            "empty scene should have no errors: {:?}",
            errs
        );
    }

    #[test]
    fn validate_negative_mass_error() {
        let mut scene = GltfPhysicsScene::default();
        let mut body = default_rigid_body(-1.0);
        body.mass = -1.0;
        scene.rigid_bodies.push((0, body));
        let errs = validate_physics_scene(&scene);
        assert!(!errs.is_empty());
        assert!(errs[0].contains("negative mass"));
    }

    #[test]
    fn validate_same_node_joint_error() {
        let mut scene = GltfPhysicsScene::default();
        scene.joints.push(PhysicsJointDescriptor {
            joint_type: "ball".to_string(),
            node_a: 3,
            node_b: 3,
            limits: None,
        });
        let errs = validate_physics_scene(&scene);
        assert!(!errs.is_empty());
    }

    #[test]
    fn biped_physics_scene_has_joints() {
        let scene = biped_physics_scene(14);
        assert!(!scene.joints.is_empty());
        assert!(!scene.rigid_bodies.is_empty());
    }

    #[test]
    fn biped_physics_scene_joint_count_capped() {
        let scene = biped_physics_scene(5);
        assert_eq!(joint_count(&scene), 5);
    }

    #[test]
    fn embed_physics_in_gltf_roundtrip() {
        let scene = GltfPhysicsScene::default();
        let base = r#"{"asset":{"version":"2.0"}}"#;
        let result = embed_physics_in_gltf(base, &scene);
        assert!(result.contains("KHR_physics_rigid_bodies"));
    }
}
