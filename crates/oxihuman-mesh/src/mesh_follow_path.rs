// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Follow-path curve constraint for mesh — projects mesh origin along a 3-D curve.

/// A single control point on the follow path.
#[derive(Debug, Clone)]
pub struct PathControlPoint {
    pub position: [f32; 3],
}

/// A follow-path constraint configuration.
#[derive(Debug, Clone)]
pub struct FollowPathConstraint {
    pub control_points: Vec<PathControlPoint>,
    pub offset: f32,
    pub follow_curve: bool,
    pub label: String,
}

/// Create a new follow-path constraint.
pub fn new_follow_path(label: &str) -> FollowPathConstraint {
    FollowPathConstraint {
        control_points: Vec::new(),
        offset: 0.0,
        follow_curve: true,
        label: label.to_owned(),
    }
}

/// Append a control point to the path.
pub fn add_path_point(fpc: &mut FollowPathConstraint, pos: [f32; 3]) {
    fpc.control_points.push(PathControlPoint { position: pos });
}

/// Number of control points.
pub fn path_point_count(fpc: &FollowPathConstraint) -> usize {
    fpc.control_points.len()
}

fn seg_length(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Compute total arc length of the path.
pub fn path_arc_length(fpc: &FollowPathConstraint) -> f32 {
    let pts = &fpc.control_points;
    if pts.len() < 2 {
        return 0.0;
    }
    pts.windows(2)
        .map(|w| seg_length(w[0].position, w[1].position))
        .sum()
}

/// Evaluate the position on the path at normalised parameter `t` in `[0, 1]`.
pub fn evaluate_path(fpc: &FollowPathConstraint, t: f32) -> Option<[f32; 3]> {
    let pts = &fpc.control_points;
    if pts.is_empty() {
        return None;
    }
    if pts.len() == 1 {
        return Some(pts[0].position);
    }
    let t = t.clamp(0.0, 1.0);
    let n = pts.len() - 1;
    let scaled = t * n as f32;
    let i = (scaled as usize).min(n.saturating_sub(1));
    let local_t = scaled - i as f32;
    let a = pts[i].position;
    let b = pts[i + 1].position;
    Some([
        a[0] + (b[0] - a[0]) * local_t,
        a[1] + (b[1] - a[1]) * local_t,
        a[2] + (b[2] - a[2]) * local_t,
    ])
}

/// Serialize the constraint to a JSON-style string.
pub fn follow_path_to_json(fpc: &FollowPathConstraint) -> String {
    format!(
        r#"{{"label":"{}", "offset":{:.4}, "follow_curve":{}, "point_count":{}}}"#,
        fpc.label,
        fpc.offset,
        fpc.follow_curve,
        fpc.control_points.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_constraint_has_no_points() {
        /* fresh constraint has zero control points */
        let f = new_follow_path("rail");
        assert_eq!(path_point_count(&f), 0);
    }

    #[test]
    fn add_point_increases_count() {
        /* adding a point increments count to 1 */
        let mut f = new_follow_path("rail");
        add_path_point(&mut f, [0.0, 0.0, 0.0]);
        assert_eq!(path_point_count(&f), 1);
    }

    #[test]
    fn arc_length_zero_for_single_point() {
        /* single-point path has zero length */
        let mut f = new_follow_path("r");
        add_path_point(&mut f, [0.0, 0.0, 0.0]);
        assert_eq!(path_arc_length(&f), 0.0);
    }

    #[test]
    fn arc_length_two_points_correct() {
        /* distance from (0,0,0) to (3,4,0) is 5 */
        let mut f = new_follow_path("r");
        add_path_point(&mut f, [0.0, 0.0, 0.0]);
        add_path_point(&mut f, [3.0, 4.0, 0.0]);
        assert!((path_arc_length(&f) - 5.0).abs() < 1e-4);
    }

    #[test]
    fn evaluate_path_at_zero_returns_first_point() {
        /* t=0 should return the start */
        let mut f = new_follow_path("r");
        add_path_point(&mut f, [1.0, 2.0, 3.0]);
        add_path_point(&mut f, [4.0, 5.0, 6.0]);
        let p = evaluate_path(&f, 0.0).expect("should succeed");
        assert!((p[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn evaluate_path_at_one_returns_last_point() {
        /* t=1 should return the end */
        let mut f = new_follow_path("r");
        add_path_point(&mut f, [0.0, 0.0, 0.0]);
        add_path_point(&mut f, [10.0, 0.0, 0.0]);
        let p = evaluate_path(&f, 1.0).expect("should succeed");
        assert!((p[0] - 10.0).abs() < 1e-4);
    }

    #[test]
    fn evaluate_empty_path_returns_none() {
        /* evaluating with no points gives None */
        let f = new_follow_path("r");
        assert!(evaluate_path(&f, 0.5).is_none());
    }

    #[test]
    fn json_contains_label() {
        /* JSON output should include the label field */
        let f = new_follow_path("track1");
        let j = follow_path_to_json(&f);
        assert!(j.contains("track1"));
    }

    #[test]
    fn default_follow_curve_is_true() {
        /* follow_curve defaults to true */
        let f = new_follow_path("r");
        assert!(f.follow_curve);
    }

    #[test]
    fn arc_length_empty_is_zero() {
        /* empty path has zero arc length */
        let f = new_follow_path("r");
        assert_eq!(path_arc_length(&f), 0.0);
    }
}
