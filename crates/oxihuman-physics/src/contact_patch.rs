//! Contact patch simulation for tire/ground or soft body contact.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ContactPatchConfig {
    pub patch_radius: f32,
    pub normal_stiffness: f32,
    pub friction_coefficient: f32,
    pub max_contacts: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ContactPoint {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub depth: f32,
    pub body_a: u32,
    pub body_b: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ContactPatch {
    pub contacts: Vec<ContactPoint>,
    pub patch_area: f32,
    pub avg_normal: [f32; 3],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PatchForceResult {
    pub normal_force: [f32; 3],
    pub friction_force: [f32; 3],
    pub total_force: [f32; 3],
}

#[allow(dead_code)]
pub fn default_contact_patch_config() -> ContactPatchConfig {
    ContactPatchConfig {
        patch_radius: 0.1,
        normal_stiffness: 1000.0,
        friction_coefficient: 0.6,
        max_contacts: 64,
    }
}

#[allow(dead_code)]
pub fn new_contact_point(
    pos: [f32; 3],
    normal: [f32; 3],
    depth: f32,
    a: u32,
    b: u32,
) -> ContactPoint {
    ContactPoint { position: pos, normal, depth, body_a: a, body_b: b }
}

/// Build a ContactPatch from a list of contact points, computing patch_area and avg_normal.
#[allow(dead_code)]
pub fn build_contact_patch(contacts: Vec<ContactPoint>) -> ContactPatch {
    if contacts.is_empty() {
        return ContactPatch {
            contacts,
            patch_area: 0.0,
            avg_normal: [0.0, 1.0, 0.0],
        };
    }
    let n = contacts.len() as f32;
    let mut avg_normal = [0.0f32; 3];
    for cp in &contacts {
        avg_normal[0] += cp.normal[0];
        avg_normal[1] += cp.normal[1];
        avg_normal[2] += cp.normal[2];
    }
    avg_normal[0] /= n;
    avg_normal[1] /= n;
    avg_normal[2] /= n;
    let len = (avg_normal[0].powi(2) + avg_normal[1].powi(2) + avg_normal[2].powi(2)).sqrt();
    if len > 1e-10 {
        avg_normal[0] /= len;
        avg_normal[1] /= len;
        avg_normal[2] /= len;
    }
    // Estimate patch area proportional to number of contacts (unit area per contact)
    let patch_area = std::f32::consts::PI * (contacts.len() as f32).sqrt() * 0.01;
    ContactPatch { contacts, patch_area, avg_normal }
}

/// Compute normal and friction forces for a contact patch.
///
/// Normal force: F_n = k * avg_depth * avg_normal
/// Friction force: opposes lateral velocity (velocity projected perpendicular to normal)
#[allow(dead_code)]
pub fn compute_patch_force(
    patch: &ContactPatch,
    cfg: &ContactPatchConfig,
    velocity: [f32; 3],
) -> PatchForceResult {
    if patch.contacts.is_empty() {
        return PatchForceResult {
            normal_force: [0.0; 3],
            friction_force: [0.0; 3],
            total_force: [0.0; 3],
        };
    }

    let avg_depth = patch_avg_depth(patch);
    let n = patch.avg_normal;

    // Normal force magnitude
    let fn_mag = cfg.normal_stiffness * avg_depth;
    let normal_force = [fn_mag * n[0], fn_mag * n[1], fn_mag * n[2]];

    // Lateral velocity (velocity minus normal component)
    let v_dot_n = velocity[0] * n[0] + velocity[1] * n[1] + velocity[2] * n[2];
    let v_lateral = [
        velocity[0] - v_dot_n * n[0],
        velocity[1] - v_dot_n * n[1],
        velocity[2] - v_dot_n * n[2],
    ];
    let v_lat_mag =
        (v_lateral[0].powi(2) + v_lateral[1].powi(2) + v_lateral[2].powi(2)).sqrt();

    let friction_force = if v_lat_mag > 1e-10 {
        let ff_mag = cfg.friction_coefficient * fn_mag;
        [
            -ff_mag * v_lateral[0] / v_lat_mag,
            -ff_mag * v_lateral[1] / v_lat_mag,
            -ff_mag * v_lateral[2] / v_lat_mag,
        ]
    } else {
        [0.0; 3]
    };

    let total_force = [
        normal_force[0] + friction_force[0],
        normal_force[1] + friction_force[1],
        normal_force[2] + friction_force[2],
    ];

    PatchForceResult { normal_force, friction_force, total_force }
}

#[allow(dead_code)]
pub fn patch_contact_count(patch: &ContactPatch) -> usize {
    patch.contacts.len()
}

#[allow(dead_code)]
pub fn patch_avg_depth(patch: &ContactPatch) -> f32 {
    if patch.contacts.is_empty() {
        return 0.0;
    }
    let sum: f32 = patch.contacts.iter().map(|cp| cp.depth).sum();
    sum / patch.contacts.len() as f32
}

#[allow(dead_code)]
pub fn patch_to_json(patch: &ContactPatch) -> String {
    format!(
        "{{\"contact_count\":{},\"patch_area\":{:.6},\"avg_normal\":[{:.4},{:.4},{:.4}]}}",
        patch.contacts.len(),
        patch.patch_area,
        patch.avg_normal[0],
        patch.avg_normal[1],
        patch.avg_normal[2]
    )
}

#[allow(dead_code)]
pub fn contact_point_to_json(cp: &ContactPoint) -> String {
    format!(
        "{{\"position\":[{:.4},{:.4},{:.4}],\"normal\":[{:.4},{:.4},{:.4}],\"depth\":{:.6},\"body_a\":{},\"body_b\":{}}}",
        cp.position[0], cp.position[1], cp.position[2],
        cp.normal[0], cp.normal[1], cp.normal[2],
        cp.depth, cp.body_a, cp.body_b
    )
}

#[allow(dead_code)]
pub fn patch_force_to_json(r: &PatchForceResult) -> String {
    format!(
        "{{\"normal_force\":[{:.4},{:.4},{:.4}],\"friction_force\":[{:.4},{:.4},{:.4}],\"total_force\":[{:.4},{:.4},{:.4}]}}",
        r.normal_force[0], r.normal_force[1], r.normal_force[2],
        r.friction_force[0], r.friction_force[1], r.friction_force[2],
        r.total_force[0], r.total_force[1], r.total_force[2]
    )
}

#[allow(dead_code)]
pub fn is_in_contact(patch: &ContactPatch) -> bool {
    !patch.contacts.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_contact(depth: f32) -> ContactPoint {
        new_contact_point([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], depth, 0, 1)
    }

    #[test]
    fn default_config_sane() {
        let cfg = default_contact_patch_config();
        assert!(cfg.patch_radius > 0.0);
        assert!(cfg.normal_stiffness > 0.0);
        assert!(cfg.friction_coefficient >= 0.0);
        assert!(cfg.max_contacts > 0);
    }

    #[test]
    fn build_patch_empty_no_contact() {
        let patch = build_contact_patch(vec![]);
        assert!(!is_in_contact(&patch));
        assert_eq!(patch_contact_count(&patch), 0);
    }

    #[test]
    fn build_patch_single_contact() {
        let contacts = vec![make_contact(0.01)];
        let patch = build_contact_patch(contacts);
        assert!(is_in_contact(&patch));
        assert_eq!(patch_contact_count(&patch), 1);
    }

    #[test]
    fn patch_avg_depth_correct() {
        let contacts = vec![make_contact(0.01), make_contact(0.03)];
        let patch = build_contact_patch(contacts);
        let depth = patch_avg_depth(&patch);
        assert!((depth - 0.02).abs() < 1e-5);
    }

    #[test]
    fn compute_patch_force_no_contacts_is_zero() {
        let patch = build_contact_patch(vec![]);
        let cfg = default_contact_patch_config();
        let result = compute_patch_force(&patch, &cfg, [0.0, 0.0, 0.0]);
        assert_eq!(result.normal_force, [0.0; 3]);
        assert_eq!(result.friction_force, [0.0; 3]);
        assert_eq!(result.total_force, [0.0; 3]);
    }

    #[test]
    fn compute_patch_force_normal_direction() {
        let contacts = vec![make_contact(0.01)];
        let patch = build_contact_patch(contacts);
        let cfg = default_contact_patch_config();
        let result = compute_patch_force(&patch, &cfg, [0.0, 0.0, 0.0]);
        // avg_normal is [0,1,0], so normal_force should be in Y direction
        assert!(result.normal_force[1] > 0.0);
        assert!(result.normal_force[0].abs() < 1e-10);
        assert!(result.normal_force[2].abs() < 1e-10);
    }

    #[test]
    fn compute_patch_force_friction_opposes_lateral() {
        let contacts = vec![make_contact(0.01)];
        let patch = build_contact_patch(contacts);
        let cfg = default_contact_patch_config();
        // lateral velocity in X
        let result = compute_patch_force(&patch, &cfg, [1.0, 0.0, 0.0]);
        // friction should be negative X
        assert!(result.friction_force[0] < 0.0);
    }

    #[test]
    fn patch_to_json_contains_contact_count() {
        let contacts = vec![make_contact(0.01), make_contact(0.02)];
        let patch = build_contact_patch(contacts);
        let json = patch_to_json(&patch);
        assert!(json.contains("\"contact_count\":2"));
    }

    #[test]
    fn contact_point_to_json_contains_depth() {
        let cp = make_contact(0.05);
        let json = contact_point_to_json(&cp);
        assert!(json.contains("\"depth\""));
        assert!(json.contains("\"body_a\":0"));
        assert!(json.contains("\"body_b\":1"));
    }

    #[test]
    fn patch_force_to_json_contains_fields() {
        let r = PatchForceResult {
            normal_force: [0.0, 10.0, 0.0],
            friction_force: [-3.0, 0.0, 0.0],
            total_force: [-3.0, 10.0, 0.0],
        };
        let json = patch_force_to_json(&r);
        assert!(json.contains("normal_force"));
        assert!(json.contains("friction_force"));
        assert!(json.contains("total_force"));
    }
}
