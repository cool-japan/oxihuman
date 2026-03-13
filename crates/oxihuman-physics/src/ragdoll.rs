//! Ragdoll physics from skeleton bones.

// ---------------------------------------------------------------------------
// Structures
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub struct RagdollBone {
    pub id: u32,
    pub name: String,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub orientation: [f32; 4], // quaternion xyzw
    pub angular_velocity: [f32; 3],
    pub mass: f32,
    pub capsule_radius: f32,
    pub capsule_length: f32,
    pub parent_id: Option<u32>,
}

#[allow(dead_code)]
pub struct RagdollJoint {
    pub bone_a: u32,
    pub bone_b: u32,
    pub twist_limit_deg: f32,
    pub swing_limit_deg: f32,
}

#[allow(dead_code)]
pub struct Ragdoll {
    pub bones: Vec<RagdollBone>,
    pub joints: Vec<RagdollJoint>,
    pub gravity: [f32; 3],
    pub active: bool,
    pub next_id: u32,
}

// ---------------------------------------------------------------------------
// Constructors
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn new_ragdoll() -> Ragdoll {
    Ragdoll {
        bones: Vec::new(),
        joints: Vec::new(),
        gravity: [0.0, -9.81, 0.0],
        active: false,
        next_id: 0,
    }
}

/// Add a bone to the ragdoll.  Returns the new bone's ID.
#[allow(dead_code)]
pub fn add_ragdoll_bone(
    ragdoll: &mut Ragdoll,
    name: &str,
    pos: [f32; 3],
    mass: f32,
    capsule_radius: f32,
    capsule_length: f32,
    parent: Option<u32>,
) -> u32 {
    let id = ragdoll.next_id;
    ragdoll.next_id += 1;
    ragdoll.bones.push(RagdollBone {
        id,
        name: name.to_string(),
        position: pos,
        velocity: [0.0; 3],
        orientation: [0.0, 0.0, 0.0, 1.0],
        angular_velocity: [0.0; 3],
        mass,
        capsule_radius,
        capsule_length,
        parent_id: parent,
    });
    id
}

/// Add a joint between two bones.
#[allow(dead_code)]
pub fn add_ragdoll_joint(
    ragdoll: &mut Ragdoll,
    bone_a: u32,
    bone_b: u32,
    twist_deg: f32,
    swing_deg: f32,
) {
    ragdoll.joints.push(RagdollJoint {
        bone_a,
        bone_b,
        twist_limit_deg: twist_deg,
        swing_limit_deg: swing_deg,
    });
}

// ---------------------------------------------------------------------------
// Simulation
// ---------------------------------------------------------------------------

/// Integrate each active bone under gravity.
#[allow(dead_code)]
pub fn simulate_ragdoll(ragdoll: &mut Ragdoll, dt: f32) {
    if !ragdoll.active {
        return;
    }
    let g = ragdoll.gravity;
    for bone in ragdoll.bones.iter_mut() {
        bone.velocity[0] += g[0] * dt;
        bone.velocity[1] += g[1] * dt;
        bone.velocity[2] += g[2] * dt;
        bone.position[0] += bone.velocity[0] * dt;
        bone.position[1] += bone.velocity[1] * dt;
        bone.position[2] += bone.velocity[2] * dt;
    }
}

/// Apply an impulse (change in momentum) to a bone identified by id.
#[allow(dead_code)]
pub fn apply_impulse_to_bone(ragdoll: &mut Ragdoll, bone_id: u32, impulse: [f32; 3]) {
    if let Some(bone) = ragdoll.bones.iter_mut().find(|b| b.id == bone_id) {
        let inv_mass = if bone.mass > 1e-12 {
            1.0 / bone.mass
        } else {
            0.0
        };
        bone.velocity[0] += impulse[0] * inv_mass;
        bone.velocity[1] += impulse[1] * inv_mass;
        bone.velocity[2] += impulse[2] * inv_mass;
    }
}

/// Look up a bone by ID.
#[allow(dead_code)]
pub fn get_ragdoll_bone(ragdoll: &Ragdoll, id: u32) -> Option<&RagdollBone> {
    ragdoll.bones.iter().find(|b| b.id == id)
}

/// Number of bones.
#[allow(dead_code)]
pub fn ragdoll_bone_count(ragdoll: &Ragdoll) -> usize {
    ragdoll.bones.len()
}

/// Number of joints.
#[allow(dead_code)]
pub fn ragdoll_joint_count(ragdoll: &Ragdoll) -> usize {
    ragdoll.joints.len()
}

/// Centre of mass of all bones (mass-weighted average position).
#[allow(dead_code)]
pub fn ragdoll_center_of_mass(ragdoll: &Ragdoll) -> [f32; 3] {
    let total = ragdoll_total_mass(ragdoll);
    if total < 1e-12 {
        return [0.0; 3];
    }
    let mut com = [0.0f32; 3];
    for bone in &ragdoll.bones {
        com[0] += bone.position[0] * bone.mass;
        com[1] += bone.position[1] * bone.mass;
        com[2] += bone.position[2] * bone.mass;
    }
    [com[0] / total, com[1] / total, com[2] / total]
}

/// Sum of all bone masses.
#[allow(dead_code)]
pub fn ragdoll_total_mass(ragdoll: &Ragdoll) -> f32 {
    ragdoll.bones.iter().map(|b| b.mass).sum()
}

/// Mark ragdoll as active (physics simulation enabled).
#[allow(dead_code)]
pub fn activate_ragdoll(ragdoll: &mut Ragdoll) {
    ragdoll.active = true;
}

/// Mark ragdoll as inactive.
#[allow(dead_code)]
pub fn deactivate_ragdoll(ragdoll: &mut Ragdoll) {
    ragdoll.active = false;
}

// ---------------------------------------------------------------------------
// Default humanoid ragdoll
// ---------------------------------------------------------------------------

/// Build a 15-bone humanoid ragdoll with reasonable proportions.
/// Bone layout (parent → child), exactly 15 bones:
///   hips → spine → chest → neck → head   (5)
///   chest → left_upper_arm → left_lower_arm   (2, shoulders merged into chest)
///   chest → right_upper_arm → right_lower_arm (2)
///   hips  → left_upper_leg  → left_lower_leg  → left_foot   (3)
///   hips  → right_upper_leg → right_lower_leg → right_foot  (3)
///   Total: 5 + 2 + 2 + 3 + 3 = 15
#[allow(dead_code)]
pub fn default_humanoid_ragdoll() -> Ragdoll {
    let mut r = new_ragdoll();

    // -- Torso (5 bones) --
    let hips = add_ragdoll_bone(&mut r, "hips", [0.0, 0.90, 0.0], 8.0, 0.12, 0.25, None);
    let spine = add_ragdoll_bone(
        &mut r,
        "spine",
        [0.0, 1.15, 0.0],
        6.0,
        0.10,
        0.25,
        Some(hips),
    );
    let chest = add_ragdoll_bone(
        &mut r,
        "chest",
        [0.0, 1.40, 0.0],
        7.0,
        0.13,
        0.25,
        Some(spine),
    );
    let neck = add_ragdoll_bone(
        &mut r,
        "neck",
        [0.0, 1.60, 0.0],
        2.0,
        0.06,
        0.12,
        Some(chest),
    );
    let head = add_ragdoll_bone(
        &mut r,
        "head",
        [0.0, 1.75, 0.0],
        5.0,
        0.10,
        0.20,
        Some(neck),
    );

    // -- Left arm (2 bones) --
    let l_upper_arm = add_ragdoll_bone(
        &mut r,
        "left_upper_arm",
        [-0.40, 1.30, 0.0],
        2.0,
        0.05,
        0.28,
        Some(chest),
    );
    let l_lower_arm = add_ragdoll_bone(
        &mut r,
        "left_lower_arm",
        [-0.40, 0.95, 0.0],
        1.5,
        0.04,
        0.26,
        Some(l_upper_arm),
    );

    // -- Right arm (2 bones) --
    let r_upper_arm = add_ragdoll_bone(
        &mut r,
        "right_upper_arm",
        [0.40, 1.30, 0.0],
        2.0,
        0.05,
        0.28,
        Some(chest),
    );
    let r_lower_arm = add_ragdoll_bone(
        &mut r,
        "right_lower_arm",
        [0.40, 0.95, 0.0],
        1.5,
        0.04,
        0.26,
        Some(r_upper_arm),
    );

    // -- Left leg (3 bones) --
    let l_upper_leg = add_ragdoll_bone(
        &mut r,
        "left_upper_leg",
        [-0.12, 0.60, 0.0],
        4.0,
        0.07,
        0.40,
        Some(hips),
    );
    let l_lower_leg = add_ragdoll_bone(
        &mut r,
        "left_lower_leg",
        [-0.12, 0.20, 0.0],
        3.0,
        0.05,
        0.38,
        Some(l_upper_leg),
    );
    let l_foot = add_ragdoll_bone(
        &mut r,
        "left_foot",
        [-0.12, 0.00, 0.0],
        1.0,
        0.05,
        0.18,
        Some(l_lower_leg),
    );

    // -- Right leg (3 bones) --
    let r_upper_leg = add_ragdoll_bone(
        &mut r,
        "right_upper_leg",
        [0.12, 0.60, 0.0],
        4.0,
        0.07,
        0.40,
        Some(hips),
    );
    let r_lower_leg = add_ragdoll_bone(
        &mut r,
        "right_lower_leg",
        [0.12, 0.20, 0.0],
        3.0,
        0.05,
        0.38,
        Some(r_upper_leg),
    );
    let r_foot = add_ragdoll_bone(
        &mut r,
        "right_foot",
        [0.12, 0.00, 0.0],
        1.0,
        0.05,
        0.18,
        Some(r_lower_leg),
    );

    // Joints
    add_ragdoll_joint(&mut r, hips, spine, 20.0, 30.0);
    add_ragdoll_joint(&mut r, spine, chest, 20.0, 30.0);
    add_ragdoll_joint(&mut r, chest, neck, 15.0, 30.0);
    add_ragdoll_joint(&mut r, neck, head, 30.0, 45.0);
    add_ragdoll_joint(&mut r, chest, l_upper_arm, 10.0, 90.0);
    add_ragdoll_joint(&mut r, l_upper_arm, l_lower_arm, 0.0, 145.0);
    add_ragdoll_joint(&mut r, chest, r_upper_arm, 10.0, 90.0);
    add_ragdoll_joint(&mut r, r_upper_arm, r_lower_arm, 0.0, 145.0);
    add_ragdoll_joint(&mut r, hips, l_upper_leg, 20.0, 100.0);
    add_ragdoll_joint(&mut r, l_upper_leg, l_lower_leg, 0.0, 145.0);
    add_ragdoll_joint(&mut r, l_lower_leg, l_foot, 30.0, 50.0);
    add_ragdoll_joint(&mut r, hips, r_upper_leg, 20.0, 100.0);
    add_ragdoll_joint(&mut r, r_upper_leg, r_lower_leg, 0.0, 145.0);
    add_ragdoll_joint(&mut r, r_lower_leg, r_foot, 30.0, 50.0);

    r
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ragdoll_empty() {
        let r = new_ragdoll();
        assert_eq!(ragdoll_bone_count(&r), 0);
        assert_eq!(ragdoll_joint_count(&r), 0);
        assert!(!r.active);
    }

    #[test]
    fn test_add_bone_count() {
        let mut r = new_ragdoll();
        add_ragdoll_bone(&mut r, "hip", [0.0, 0.9, 0.0], 8.0, 0.1, 0.3, None);
        add_ragdoll_bone(&mut r, "spine", [0.0, 1.2, 0.0], 6.0, 0.1, 0.3, Some(0));
        assert_eq!(ragdoll_bone_count(&r), 2);
    }

    #[test]
    fn test_add_joint_count() {
        let mut r = new_ragdoll();
        let a = add_ragdoll_bone(&mut r, "a", [0.0, 0.0, 0.0], 1.0, 0.1, 0.3, None);
        let b = add_ragdoll_bone(&mut r, "b", [0.0, 0.5, 0.0], 1.0, 0.1, 0.3, Some(a));
        add_ragdoll_joint(&mut r, a, b, 30.0, 60.0);
        assert_eq!(ragdoll_joint_count(&r), 1);
    }

    #[test]
    fn test_get_ragdoll_bone_found() {
        let mut r = new_ragdoll();
        let id = add_ragdoll_bone(&mut r, "test", [0.0; 3], 1.0, 0.1, 0.3, None);
        let bone = get_ragdoll_bone(&r, id);
        assert!(bone.is_some());
        assert_eq!(bone.expect("should succeed").name, "test");
    }

    #[test]
    fn test_get_ragdoll_bone_not_found() {
        let r = new_ragdoll();
        assert!(get_ragdoll_bone(&r, 999).is_none());
    }

    #[test]
    fn test_default_humanoid_has_15_bones() {
        let r = default_humanoid_ragdoll();
        assert_eq!(
            ragdoll_bone_count(&r),
            15,
            "expected 15 bones, got {}",
            ragdoll_bone_count(&r)
        );
    }

    #[test]
    fn test_simulate_ragdoll_inactive_no_move() {
        let mut r = new_ragdoll();
        add_ragdoll_bone(&mut r, "hip", [0.0, 1.0, 0.0], 8.0, 0.1, 0.3, None);
        let y_before = r.bones[0].position[1];
        simulate_ragdoll(&mut r, 0.1);
        assert!((r.bones[0].position[1] - y_before).abs() < 1e-9);
    }

    #[test]
    fn test_simulate_ragdoll_active_falls() {
        let mut r = new_ragdoll();
        add_ragdoll_bone(&mut r, "hip", [0.0, 10.0, 0.0], 8.0, 0.1, 0.3, None);
        activate_ragdoll(&mut r);
        for _ in 0..50 {
            simulate_ragdoll(&mut r, 0.05);
        }
        assert!(
            r.bones[0].position[1] < 10.0,
            "bone should fall under gravity"
        );
    }

    #[test]
    fn test_apply_impulse_to_bone() {
        let mut r = new_ragdoll();
        let id = add_ragdoll_bone(&mut r, "hip", [0.0; 3], 2.0, 0.1, 0.3, None);
        apply_impulse_to_bone(&mut r, id, [2.0, 0.0, 0.0]);
        let bone = get_ragdoll_bone(&r, id).expect("should succeed");
        // impulse = mass * delta_v → delta_v = 2/2 = 1
        assert!((bone.velocity[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_ragdoll_total_mass() {
        let mut r = new_ragdoll();
        add_ragdoll_bone(&mut r, "a", [0.0; 3], 3.0, 0.1, 0.3, None);
        add_ragdoll_bone(&mut r, "b", [0.0; 3], 7.0, 0.1, 0.3, None);
        assert!((ragdoll_total_mass(&r) - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_ragdoll_center_of_mass() {
        let mut r = new_ragdoll();
        add_ragdoll_bone(&mut r, "a", [0.0, 0.0, 0.0], 1.0, 0.1, 0.3, None);
        add_ragdoll_bone(&mut r, "b", [2.0, 0.0, 0.0], 1.0, 0.1, 0.3, None);
        let com = ragdoll_center_of_mass(&r);
        assert!((com[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_activate_deactivate() {
        let mut r = new_ragdoll();
        assert!(!r.active);
        activate_ragdoll(&mut r);
        assert!(r.active);
        deactivate_ragdoll(&mut r);
        assert!(!r.active);
    }

    #[test]
    fn test_default_humanoid_bone_ids_sequential() {
        let r = default_humanoid_ragdoll();
        for (i, bone) in r.bones.iter().enumerate() {
            assert_eq!(bone.id as usize, i);
        }
    }

    #[test]
    fn test_default_humanoid_joints_reference_valid_bones() {
        let r = default_humanoid_ragdoll();
        let ids: Vec<u32> = r.bones.iter().map(|b| b.id).collect();
        for joint in &r.joints {
            assert!(
                ids.contains(&joint.bone_a),
                "invalid bone_a {}",
                joint.bone_a
            );
            assert!(
                ids.contains(&joint.bone_b),
                "invalid bone_b {}",
                joint.bone_b
            );
        }
    }

    #[test]
    fn test_ragdoll_center_of_mass_empty() {
        let r = new_ragdoll();
        let com = ragdoll_center_of_mass(&r);
        assert_eq!(com, [0.0; 3]);
    }
}
