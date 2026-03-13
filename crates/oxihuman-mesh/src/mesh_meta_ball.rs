// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Metaball implicit surface mesh — evaluates a metaball isosurface field.

/// A single metaball element.
#[derive(Debug, Clone)]
pub struct MetaBall {
    pub center: [f32; 3],
    pub radius: f32,
    pub strength: f32,
    pub label: String,
}

/// Collection of metaballs forming one implicit surface.
#[derive(Debug, Default)]
pub struct MetaBallField {
    pub balls: Vec<MetaBall>,
    pub iso_threshold: f32,
}

/// Create a new metaball field with a given iso threshold.
pub fn new_meta_ball_field(iso_threshold: f32) -> MetaBallField {
    MetaBallField {
        balls: Vec::new(),
        iso_threshold: iso_threshold.max(0.0),
    }
}

/// Add a metaball to the field.
pub fn add_meta_ball(
    field: &mut MetaBallField,
    center: [f32; 3],
    radius: f32,
    strength: f32,
    label: &str,
) {
    field.balls.push(MetaBall {
        center,
        radius: radius.max(1e-6),
        strength,
        label: label.to_owned(),
    });
}

/// Number of metaballs.
pub fn meta_ball_count(field: &MetaBallField) -> usize {
    field.balls.len()
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// Wyvill-style falloff function for one metaball at `query`.
fn ball_potential(ball: &MetaBall, query: [f32; 3]) -> f32 {
    let r = dist3(ball.center, query);
    let n = r / ball.radius;
    if n >= 1.0 {
        return 0.0;
    }
    let n2 = n * n;
    ball.strength * (1.0 - n2).powi(2)
}

/// Evaluate the combined field potential at a query point.
pub fn evaluate_field(field: &MetaBallField, query: [f32; 3]) -> f32 {
    field.balls.iter().map(|b| ball_potential(b, query)).sum()
}

/// Is the query point inside the isosurface?
pub fn is_inside_surface(field: &MetaBallField, query: [f32; 3]) -> bool {
    evaluate_field(field, query) >= field.iso_threshold
}

/// Average radius over all metaballs.
pub fn average_meta_radius(field: &MetaBallField) -> f32 {
    if field.balls.is_empty() {
        return 0.0;
    }
    let sum: f32 = field.balls.iter().map(|b| b.radius).sum();
    sum / field.balls.len() as f32
}

/// Serialize to JSON-style string.
pub fn meta_ball_field_to_json(field: &MetaBallField) -> String {
    format!(
        r#"{{"iso_threshold":{:.4}, "ball_count":{}}}"#,
        field.iso_threshold,
        field.balls.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_field_has_no_balls() {
        /* fresh field has no metaballs */
        let f = new_meta_ball_field(0.5);
        assert_eq!(meta_ball_count(&f), 0);
    }

    #[test]
    fn add_ball_increments_count() {
        /* adding a ball should increase count */
        let mut f = new_meta_ball_field(0.5);
        add_meta_ball(&mut f, [0.0; 3], 1.0, 1.0, "b0");
        assert_eq!(meta_ball_count(&f), 1);
    }

    #[test]
    fn field_potential_at_center_is_max() {
        /* potential at ball center should equal strength */
        let mut f = new_meta_ball_field(0.5);
        add_meta_ball(&mut f, [0.0; 3], 1.0, 2.0, "b0");
        let v = evaluate_field(&f, [0.0; 3]);
        assert!((v - 2.0).abs() < 1e-5);
    }

    #[test]
    fn field_potential_outside_radius_is_zero() {
        /* potential beyond radius should be zero */
        let mut f = new_meta_ball_field(0.5);
        add_meta_ball(&mut f, [0.0; 3], 1.0, 1.0, "b0");
        let v = evaluate_field(&f, [5.0, 0.0, 0.0]);
        assert!(v < 1e-8);
    }

    #[test]
    fn is_inside_surface_at_center() {
        /* center of strong ball is inside surface */
        let mut f = new_meta_ball_field(0.5);
        add_meta_ball(&mut f, [0.0; 3], 1.0, 1.0, "b0");
        assert!(is_inside_surface(&f, [0.0; 3]));
    }

    #[test]
    fn is_inside_surface_far_point_false() {
        /* far point is outside the surface */
        let mut f = new_meta_ball_field(0.5);
        add_meta_ball(&mut f, [0.0; 3], 1.0, 1.0, "b0");
        assert!(!is_inside_surface(&f, [10.0, 0.0, 0.0]));
    }

    #[test]
    fn average_radius_empty_is_zero() {
        /* empty field average radius is zero */
        let f = new_meta_ball_field(0.5);
        assert_eq!(average_meta_radius(&f), 0.0);
    }

    #[test]
    fn average_radius_correct() {
        /* average of radii 2 and 4 is 3 */
        let mut f = new_meta_ball_field(0.5);
        add_meta_ball(&mut f, [0.0; 3], 2.0, 1.0, "a");
        add_meta_ball(&mut f, [0.0; 3], 4.0, 1.0, "b");
        assert!((average_meta_radius(&f) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn json_contains_ball_count() {
        /* JSON should include ball_count field */
        let mut f = new_meta_ball_field(0.5);
        add_meta_ball(&mut f, [0.0; 3], 1.0, 1.0, "b");
        let j = meta_ball_field_to_json(&f);
        assert!(j.contains("ball_count"));
    }
}
