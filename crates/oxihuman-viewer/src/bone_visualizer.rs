//! Skeleton bone visualization — renders bone sticks and joint spheres from transform data.
//!
//! Converts an array of joint world-space positions and a parent index array into
//! [`BoneStick`] and [`JointSphere`] primitives ready for GPU submission.

// ── Types ─────────────────────────────────────────────────────────────────────

/// Configuration for bone visualization.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoneVisConfig {
    /// RGB colour of bone sticks (0.0–1.0 per channel).
    pub bone_color: [f32; 3],
    /// Radius of joint spheres.
    pub joint_radius: f32,
    /// Whether joint spheres are rendered.
    pub show_joints: bool,
    /// Whether the visualizer is active.
    pub enabled: bool,
    /// Line width for bone sticks.
    pub line_width: f32,
}

/// A single bone "stick" — a line segment from child to parent joint.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct BoneStick {
    /// World-space position of the child joint (the bone's own joint).
    pub child_pos: [f32; 3],
    /// World-space position of the parent joint.
    pub parent_pos: [f32; 3],
    /// Index of the child joint.
    pub child_idx: usize,
    /// Index of the parent joint (`usize::MAX` for root bones).
    pub parent_idx: usize,
}

/// A sphere primitive centred on a joint position.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct JointSphere {
    /// World-space centre of the sphere.
    pub position: [f32; 3],
    /// Sphere radius.
    pub radius: f32,
    /// Index of the joint this sphere represents.
    pub joint_idx: usize,
}

/// Aggregated draw-call data produced by [`bone_vis_draw_call`].
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoneVisDrawCall {
    /// All bone sticks to draw.
    pub sticks: Vec<BoneStick>,
    /// All joint spheres to draw.
    pub spheres: Vec<JointSphere>,
    /// Configuration snapshot used to produce this draw-call.
    pub config: BoneVisConfig,
}

// ── Functions ─────────────────────────────────────────────────────────────────

/// Returns a default [`BoneVisConfig`] (orange bones, 0.05 joint radius).
#[allow(dead_code)]
pub fn default_bone_vis_config() -> BoneVisConfig {
    BoneVisConfig {
        bone_color: [1.0, 0.6, 0.1],
        joint_radius: 0.05,
        show_joints: true,
        enabled: true,
        line_width: 2.0,
    }
}

/// Builds [`BoneStick`]s from world-space joint `positions` and `parents` (-1 for root).
#[allow(dead_code)]
pub fn build_bone_sticks(positions: &[[f32; 3]], parents: &[i32]) -> Vec<BoneStick> {
    let len = positions.len().min(parents.len());
    let mut sticks = Vec::new();

    for i in 0..len {
        let p = parents[i];
        if p < 0 {
            // Root joint — no bone stick
            continue;
        }
        let parent_idx = p as usize;
        if parent_idx >= positions.len() {
            continue;
        }
        sticks.push(BoneStick {
            child_pos: positions[i],
            parent_pos: positions[parent_idx],
            child_idx: i,
            parent_idx,
        });
    }

    sticks
}

/// Builds [`JointSphere`]s for every joint position using the radius from `cfg`.
#[allow(dead_code)]
pub fn build_joint_spheres(positions: &[[f32; 3]], cfg: &BoneVisConfig) -> Vec<JointSphere> {
    positions
        .iter()
        .enumerate()
        .map(|(i, &pos)| JointSphere {
            position: pos,
            radius: cfg.joint_radius,
            joint_idx: i,
        })
        .collect()
}

/// Builds a complete [`BoneVisDrawCall`] from joint positions, parent indices, and a config.
#[allow(dead_code)]
pub fn bone_vis_draw_call(
    positions: &[[f32; 3]],
    parents: &[i32],
    cfg: &BoneVisConfig,
) -> BoneVisDrawCall {
    let sticks = build_bone_sticks(positions, parents);
    let spheres = if cfg.show_joints {
        build_joint_spheres(positions, cfg)
    } else {
        Vec::new()
    };
    BoneVisDrawCall {
        sticks,
        spheres,
        config: cfg.clone(),
    }
}

/// Returns the number of bone sticks in a draw-call.
#[allow(dead_code)]
pub fn bone_count(call: &BoneVisDrawCall) -> usize {
    call.sticks.len()
}

/// Sets the RGB bone color on `cfg`.
#[allow(dead_code)]
pub fn set_bone_color(cfg: &mut BoneVisConfig, r: f32, g: f32, b: f32) {
    cfg.bone_color = [r, g, b];
}

/// Sets the joint sphere radius on `cfg`.
#[allow(dead_code)]
pub fn set_joint_radius(cfg: &mut BoneVisConfig, radius: f32) {
    cfg.joint_radius = radius;
}

/// Toggles the visibility of joint spheres.
#[allow(dead_code)]
pub fn bone_vis_toggle_joints(cfg: &mut BoneVisConfig) {
    cfg.show_joints = !cfg.show_joints;
}

/// Returns whether the bone visualizer is enabled.
#[allow(dead_code)]
pub fn bone_vis_is_enabled(cfg: &BoneVisConfig) -> bool {
    cfg.enabled
}

/// Returns the [`BoneStick`] at `bone_idx` from `call`, or [`None`] if out of range.
#[allow(dead_code)]
pub fn selected_bone_draw_call(call: &BoneVisDrawCall, bone_idx: usize) -> Option<BoneStick> {
    call.sticks.get(bone_idx).cloned()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn chain_positions() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 2.0, 0.0]]
    }

    fn chain_parents() -> Vec<i32> {
        vec![-1, 0, 1]
    }

    #[test]
    fn default_config_enabled() {
        let cfg = default_bone_vis_config();
        assert!(bone_vis_is_enabled(&cfg));
        assert!(cfg.show_joints);
    }

    #[test]
    fn build_sticks_single_chain() {
        let pos = chain_positions();
        let par = chain_parents();
        let sticks = build_bone_sticks(&pos, &par);
        // Root has no parent → 2 sticks for a 3-joint chain
        assert_eq!(sticks.len(), 2);
    }

    #[test]
    fn build_sticks_root_only() {
        let pos = vec![[0.0_f32, 0.0, 0.0]];
        let par = vec![-1_i32];
        let sticks = build_bone_sticks(&pos, &par);
        assert_eq!(sticks.len(), 0);
    }

    #[test]
    fn build_joint_spheres_count() {
        let pos = chain_positions();
        let cfg = default_bone_vis_config();
        let spheres = build_joint_spheres(&pos, &cfg);
        assert_eq!(spheres.len(), 3);
    }

    #[test]
    fn joint_sphere_radius_matches_config() {
        let pos = vec![[1.0_f32, 2.0, 3.0]];
        let mut cfg = default_bone_vis_config();
        set_joint_radius(&mut cfg, 0.12);
        let spheres = build_joint_spheres(&pos, &cfg);
        assert!((spheres[0].radius - 0.12).abs() < 1e-6);
    }

    #[test]
    fn bone_vis_draw_call_bone_count() {
        let pos = chain_positions();
        let par = chain_parents();
        let cfg = default_bone_vis_config();
        let call = bone_vis_draw_call(&pos, &par, &cfg);
        assert_eq!(bone_count(&call), 2);
    }

    #[test]
    fn selected_bone_in_bounds() {
        let pos = chain_positions();
        let par = chain_parents();
        let cfg = default_bone_vis_config();
        let call = bone_vis_draw_call(&pos, &par, &cfg);
        assert!(selected_bone_draw_call(&call, 0).is_some());
    }

    #[test]
    fn selected_bone_out_of_bounds() {
        let pos = chain_positions();
        let par = chain_parents();
        let cfg = default_bone_vis_config();
        let call = bone_vis_draw_call(&pos, &par, &cfg);
        assert!(selected_bone_draw_call(&call, 99).is_none());
    }

    #[test]
    fn toggle_joints_hides_spheres() {
        let pos = chain_positions();
        let par = chain_parents();
        let mut cfg = default_bone_vis_config();
        bone_vis_toggle_joints(&mut cfg);
        let call = bone_vis_draw_call(&pos, &par, &cfg);
        assert_eq!(call.spheres.len(), 0);
    }

    #[test]
    fn set_bone_color_applies() {
        let mut cfg = default_bone_vis_config();
        set_bone_color(&mut cfg, 0.1, 0.2, 0.3);
        assert!((cfg.bone_color[0] - 0.1).abs() < 1e-6);
        assert!((cfg.bone_color[1] - 0.2).abs() < 1e-6);
        assert!((cfg.bone_color[2] - 0.3).abs() < 1e-6);
    }
}
