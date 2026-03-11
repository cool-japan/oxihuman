// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Debug visualization primitives: wireframe overlays, normals, bounding boxes, joints.

use oxihuman_physics::BodyProxies;

// ── Data structures ───────────────────────────────────────────────────────────

/// A line segment drawn for debug purposes.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct DebugLine {
    pub start: [f32; 3],
    pub end: [f32; 3],
    /// RGBA colour.
    pub color: [f32; 4],
    pub thickness: f32,
}

/// A sphere (wire or solid) drawn for debug purposes.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct DebugSphere {
    pub center: [f32; 3],
    pub radius: f32,
    /// RGBA colour.
    pub color: [f32; 4],
    pub filled: bool,
}

/// An arrow (line + head) drawn for debug purposes.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct DebugArrow {
    pub origin: [f32; 3],
    /// Direction pre-scaled by desired length.
    pub direction: [f32; 3],
    /// RGBA colour.
    pub color: [f32; 4],
}

/// A collection of debug primitives to be rendered in a single frame.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct DebugDrawList {
    pub lines: Vec<DebugLine>,
    pub spheres: Vec<DebugSphere>,
    pub arrows: Vec<DebugArrow>,
    /// World-space text labels: (text, position).
    pub text_labels: Vec<(String, [f32; 3])>,
}

impl DebugDrawList {
    /// Create a new, empty draw list.
    #[allow(dead_code)]
    pub fn new() -> Self {
        DebugDrawList::default()
    }

    /// Add a line segment with default thickness 1.0.
    #[allow(dead_code)]
    pub fn add_line(&mut self, start: [f32; 3], end: [f32; 3], color: [f32; 4]) {
        self.lines.push(DebugLine {
            start,
            end,
            color,
            thickness: 1.0,
        });
    }

    /// Add a wireframe sphere.
    #[allow(dead_code)]
    pub fn add_sphere(&mut self, center: [f32; 3], radius: f32, color: [f32; 4]) {
        self.spheres.push(DebugSphere {
            center,
            radius,
            color,
            filled: false,
        });
    }

    /// Add a directional arrow.
    #[allow(dead_code)]
    pub fn add_arrow(&mut self, origin: [f32; 3], dir: [f32; 3], color: [f32; 4]) {
        self.arrows.push(DebugArrow {
            origin,
            direction: dir,
            color,
        });
    }

    /// Add a world-space text label.
    #[allow(dead_code)]
    pub fn add_label(&mut self, text: &str, pos: [f32; 3]) {
        self.text_labels.push((text.to_string(), pos));
    }

    /// Remove all primitives from the list.
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.lines.clear();
        self.spheres.clear();
        self.arrows.clear();
        self.text_labels.clear();
    }

    /// Total count of all primitives (lines + spheres + arrows + labels).
    #[allow(dead_code)]
    pub fn total_primitives(&self) -> usize {
        self.lines.len() + self.spheres.len() + self.arrows.len() + self.text_labels.len()
    }
}

// ── Free functions ────────────────────────────────────────────────────────────

/// Add an arrow for each vertex normal, scaled by `scale`.
#[allow(dead_code)]
pub fn draw_mesh_normals(
    list: &mut DebugDrawList,
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    scale: f32,
    color: [f32; 4],
) {
    let n = positions.len().min(normals.len());
    for i in 0..n {
        let dir = [
            normals[i][0] * scale,
            normals[i][1] * scale,
            normals[i][2] * scale,
        ];
        list.add_arrow(positions[i], dir, color);
    }
}

/// Add 12 line segments forming the edges of an axis-aligned bounding box.
#[allow(dead_code)]
pub fn draw_aabb(list: &mut DebugDrawList, min: [f32; 3], max: [f32; 3], color: [f32; 4]) {
    // 8 corners
    let c = [
        [min[0], min[1], min[2]], // 0
        [max[0], min[1], min[2]], // 1
        [max[0], max[1], min[2]], // 2
        [min[0], max[1], min[2]], // 3
        [min[0], min[1], max[2]], // 4
        [max[0], min[1], max[2]], // 5
        [max[0], max[1], max[2]], // 6
        [min[0], max[1], max[2]], // 7
    ];
    // 12 edges
    let edges = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0), // bottom face
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4), // top face
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7), // verticals
    ];
    for (a, b) in edges {
        list.add_line(c[a], c[b], color);
    }
}

/// Add one line per joint to its parent (skipping root joints with no parent).
#[allow(dead_code)]
pub fn draw_skeleton(
    list: &mut DebugDrawList,
    joint_positions: &[[f32; 3]],
    parent_indices: &[Option<usize>],
    color: [f32; 4],
) {
    let n = joint_positions.len().min(parent_indices.len());
    for i in 0..n {
        if let Some(parent) = parent_indices[i] {
            if parent < joint_positions.len() {
                list.add_line(joint_positions[i], joint_positions[parent], color);
            }
        }
    }
}

/// Add wireframe debug spheres for every sphere and capsule proxy in `proxies`.
///
/// For capsules, spheres are placed at both endpoints.
#[allow(dead_code)]
pub fn draw_physics_proxies_debug(
    list: &mut DebugDrawList,
    proxies: &BodyProxies,
    color: [f32; 4],
) {
    for sphere in &proxies.spheres {
        list.add_sphere(sphere.center, sphere.radius, color);
    }
    for capsule in &proxies.capsules {
        list.add_sphere(capsule.center_a, capsule.radius, color);
        list.add_sphere(capsule.center_b, capsule.radius, color);
        list.add_line(capsule.center_a, capsule.center_b, color);
    }
}

/// Serialize the draw list to a compact JSON string.
#[allow(dead_code)]
pub fn debug_draw_to_json(list: &DebugDrawList) -> String {
    let lines: Vec<String> = list
        .lines
        .iter()
        .map(|l| {
            format!(
                r#"{{"start":[{},{},{}],"end":[{},{},{}],"color":[{},{},{},{}],"thickness":{}}}"#,
                l.start[0],
                l.start[1],
                l.start[2],
                l.end[0],
                l.end[1],
                l.end[2],
                l.color[0],
                l.color[1],
                l.color[2],
                l.color[3],
                l.thickness
            )
        })
        .collect();
    let spheres: Vec<String> = list
        .spheres
        .iter()
        .map(|s| {
            format!(
                r#"{{"center":[{},{},{}],"radius":{},"color":[{},{},{},{}],"filled":{}}}"#,
                s.center[0],
                s.center[1],
                s.center[2],
                s.radius,
                s.color[0],
                s.color[1],
                s.color[2],
                s.color[3],
                s.filled
            )
        })
        .collect();
    let arrows: Vec<String> = list
        .arrows
        .iter()
        .map(|a| {
            format!(
                r#"{{"origin":[{},{},{}],"direction":[{},{},{}],"color":[{},{},{},{}]}}"#,
                a.origin[0],
                a.origin[1],
                a.origin[2],
                a.direction[0],
                a.direction[1],
                a.direction[2],
                a.color[0],
                a.color[1],
                a.color[2],
                a.color[3]
            )
        })
        .collect();
    let labels: Vec<String> = list
        .text_labels
        .iter()
        .map(|(text, pos)| {
            format!(
                r#"{{"text":"{}","pos":[{},{},{}]}}"#,
                json_escape(text),
                pos[0],
                pos[1],
                pos[2]
            )
        })
        .collect();

    format!(
        r#"{{"lines":[{}],"spheres":[{}],"arrows":[{}],"labels":[{}]}}"#,
        lines.join(","),
        spheres.join(","),
        arrows.join(","),
        labels.join(",")
    )
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_physics::{BodyProxies, CapsuleProxy, SphereProxy};

    // 1. new() is empty
    #[test]
    fn new_is_empty() {
        let list = DebugDrawList::new();
        assert_eq!(list.total_primitives(), 0);
    }

    // 2. add_line increments line count
    #[test]
    fn add_line_increments_count() {
        let mut list = DebugDrawList::new();
        list.add_line([0.0; 3], [1.0; 3], [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(list.lines.len(), 1);
        assert_eq!(list.total_primitives(), 1);
    }

    // 3. add_sphere increments sphere count
    #[test]
    fn add_sphere_increments_count() {
        let mut list = DebugDrawList::new();
        list.add_sphere([0.0; 3], 1.0, [0.0, 1.0, 0.0, 1.0]);
        assert_eq!(list.spheres.len(), 1);
    }

    // 4. add_arrow increments arrow count
    #[test]
    fn add_arrow_increments_count() {
        let mut list = DebugDrawList::new();
        list.add_arrow([0.0; 3], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0, 1.0]);
        assert_eq!(list.arrows.len(), 1);
    }

    // 5. add_label increments label count
    #[test]
    fn add_label_increments_count() {
        let mut list = DebugDrawList::new();
        list.add_label("test", [0.0, 1.0, 0.0]);
        assert_eq!(list.text_labels.len(), 1);
        assert_eq!(list.text_labels[0].0, "test");
    }

    // 6. clear() resets everything
    #[test]
    fn clear_resets_all() {
        let mut list = DebugDrawList::new();
        list.add_line([0.0; 3], [1.0; 3], [1.0; 4]);
        list.add_sphere([0.0; 3], 1.0, [1.0; 4]);
        list.add_arrow([0.0; 3], [1.0, 0.0, 0.0], [1.0; 4]);
        list.add_label("lbl", [0.0; 3]);
        list.clear();
        assert_eq!(list.total_primitives(), 0);
    }

    // 7. total_primitives sums all types
    #[test]
    fn total_primitives_sum() {
        let mut list = DebugDrawList::new();
        list.add_line([0.0; 3], [1.0; 3], [1.0; 4]);
        list.add_line([0.0; 3], [2.0; 3], [1.0; 4]);
        list.add_sphere([0.0; 3], 0.5, [1.0; 4]);
        list.add_arrow([0.0; 3], [0.0, 1.0, 0.0], [1.0; 4]);
        list.add_label("a", [0.0; 3]);
        assert_eq!(list.total_primitives(), 5);
    }

    // 8. draw_aabb adds exactly 12 lines
    #[test]
    fn draw_aabb_adds_12_lines() {
        let mut list = DebugDrawList::new();
        draw_aabb(&mut list, [-1.0; 3], [1.0; 3], [1.0, 1.0, 0.0, 1.0]);
        assert_eq!(list.lines.len(), 12);
    }

    // 9. draw_mesh_normals adds n arrows for n vertices
    #[test]
    fn draw_mesh_normals_adds_n_arrows() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let normals = vec![[0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0]];
        let mut list = DebugDrawList::new();
        draw_mesh_normals(&mut list, &positions, &normals, 0.1, [0.0, 0.0, 1.0, 1.0]);
        assert_eq!(list.arrows.len(), 3);
    }

    // 10. draw_skeleton adds n-1 lines when all joints have parents except root
    #[test]
    fn draw_skeleton_adds_n_minus_1_lines() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 2.0, 0.0],
            [0.0, 3.0, 0.0],
        ];
        let parents: Vec<Option<usize>> = vec![None, Some(0), Some(1), Some(2)];
        let mut list = DebugDrawList::new();
        draw_skeleton(&mut list, &positions, &parents, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(list.lines.len(), 3); // 4 joints - 1 (root has no parent)
    }

    // 11. debug_draw_to_json produces non-empty JSON
    #[test]
    fn debug_draw_to_json_non_empty() {
        let mut list = DebugDrawList::new();
        list.add_line([0.0; 3], [1.0; 3], [1.0; 4]);
        let json = debug_draw_to_json(&list);
        assert!(!json.is_empty());
        assert!(json.contains("lines"));
    }

    // 12. debug_draw_to_json empty list produces valid structure
    #[test]
    fn debug_draw_to_json_empty() {
        let list = DebugDrawList::new();
        let json = debug_draw_to_json(&list);
        assert!(json.contains("lines"));
        assert!(json.contains("spheres"));
    }

    // 13. draw_physics_proxies_debug runs without panic
    #[test]
    fn draw_physics_proxies_debug_no_panic() {
        let mut proxies = BodyProxies::new();
        proxies
            .spheres
            .push(SphereProxy::new([0.0, 1.0, 0.0], 0.1, "head"));
        proxies.capsules.push(CapsuleProxy::new(
            [0.0, 0.5, 0.0],
            [0.0, 1.0, 0.0],
            0.15,
            "torso",
        ));
        let mut list = DebugDrawList::new();
        draw_physics_proxies_debug(&mut list, &proxies, [0.0, 1.0, 0.0, 1.0]);
        // 1 sphere proxy + 2 endpoint spheres + 1 capsule line = 4 total primitives
        assert_eq!(list.spheres.len(), 3);
        assert_eq!(list.lines.len(), 1);
    }

    // 14. draw_aabb covers correct corners — check one diagonal
    #[test]
    fn draw_aabb_min_max_in_lines() {
        let min = [0.0f32; 3];
        let max = [2.0f32; 3];
        let mut list = DebugDrawList::new();
        draw_aabb(&mut list, min, max, [1.0; 4]);
        // At least one line must start or end at the min corner
        let has_min = list.lines.iter().any(|l| l.start == min || l.end == min);
        assert!(has_min);
    }
}
