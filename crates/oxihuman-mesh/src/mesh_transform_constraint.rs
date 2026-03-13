// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! General transform constraint — maps source transform channels to target channels.

/// Which transform channel to read as the driving input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformChannel {
    LocationX,
    LocationY,
    LocationZ,
    RotationX,
    RotationY,
    RotationZ,
    ScaleX,
    ScaleY,
    ScaleZ,
}

/// Transform constraint descriptor.
#[derive(Debug, Clone)]
pub struct TransformConstraint {
    pub source_channel: TransformChannel,
    pub target_channel: TransformChannel,
    pub from_min: f32,
    pub from_max: f32,
    pub to_min: f32,
    pub to_max: f32,
    pub influence: f32,
    pub label: String,
}

/// Create a transform constraint with default mapping range `[-1, 1] -> [-1, 1]`.
pub fn new_transform_constraint(
    source: TransformChannel,
    target: TransformChannel,
    label: &str,
) -> TransformConstraint {
    TransformConstraint {
        source_channel: source,
        target_channel: target,
        from_min: -1.0,
        from_max: 1.0,
        to_min: -1.0,
        to_max: 1.0,
        influence: 1.0,
        label: label.to_owned(),
    }
}

/// Map `value` from the source range to the target range (linear).
pub fn map_value(c: &TransformConstraint, value: f32) -> f32 {
    let from_range = c.from_max - c.from_min;
    if from_range.abs() < 1e-8 {
        return c.to_min;
    }
    let t = (value - c.from_min) / from_range;
    let t = t.clamp(0.0, 1.0);
    let mapped = c.to_min + t * (c.to_max - c.to_min);
    mapped * c.influence
}

/// Validate that from_max > from_min and to_max > to_min.
pub fn validate_transform_constraint(c: &TransformConstraint) -> bool {
    c.from_max > c.from_min && c.to_max > c.to_min
}

/// Return a human-readable string for a channel.
pub fn channel_name(ch: TransformChannel) -> &'static str {
    match ch {
        TransformChannel::LocationX => "loc_x",
        TransformChannel::LocationY => "loc_y",
        TransformChannel::LocationZ => "loc_z",
        TransformChannel::RotationX => "rot_x",
        TransformChannel::RotationY => "rot_y",
        TransformChannel::RotationZ => "rot_z",
        TransformChannel::ScaleX => "scale_x",
        TransformChannel::ScaleY => "scale_y",
        TransformChannel::ScaleZ => "scale_z",
    }
}

/// Serialize to JSON-style string.
pub fn transform_constraint_to_json(c: &TransformConstraint) -> String {
    format!(
        r#"{{"label":"{}", "source":"{}", "target":"{}", "from":[{:.4},{:.4}], "to":[{:.4},{:.4}], "influence":{:.4}}}"#,
        c.label,
        channel_name(c.source_channel),
        channel_name(c.target_channel),
        c.from_min,
        c.from_max,
        c.to_min,
        c.to_max,
        c.influence
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_value_at_from_min_returns_to_min() {
        /* input at from_min should map to to_min */
        let mut c =
            new_transform_constraint(TransformChannel::LocationX, TransformChannel::ScaleY, "t");
        c.from_min = 0.0;
        c.from_max = 1.0;
        c.to_min = 0.0;
        c.to_max = 10.0;
        assert!((map_value(&c, 0.0) - 0.0).abs() < 1e-5);
    }

    #[test]
    fn map_value_at_from_max_returns_to_max() {
        /* input at from_max maps to to_max */
        let mut c =
            new_transform_constraint(TransformChannel::LocationX, TransformChannel::ScaleY, "t");
        c.from_min = 0.0;
        c.from_max = 1.0;
        c.to_min = 0.0;
        c.to_max = 10.0;
        assert!((map_value(&c, 1.0) - 10.0).abs() < 1e-4);
    }

    #[test]
    fn map_value_zero_influence_yields_zero() {
        /* influence 0 returns 0 */
        let mut c = new_transform_constraint(
            TransformChannel::RotationX,
            TransformChannel::LocationZ,
            "t",
        );
        c.from_min = 0.0;
        c.from_max = 1.0;
        c.to_min = 0.0;
        c.to_max = 5.0;
        c.influence = 0.0;
        assert!((map_value(&c, 0.5) - 0.0).abs() < 1e-5);
    }

    #[test]
    fn validate_default_constraint_is_valid() {
        /* default ranges [-1,1] -> [-1,1] should validate */
        let c = new_transform_constraint(TransformChannel::ScaleX, TransformChannel::ScaleX, "t");
        assert!(validate_transform_constraint(&c));
    }

    #[test]
    fn validate_inverted_range_fails() {
        /* from_min > from_max should fail validation */
        let mut c =
            new_transform_constraint(TransformChannel::ScaleX, TransformChannel::ScaleX, "t");
        c.from_min = 1.0;
        c.from_max = -1.0;
        assert!(!validate_transform_constraint(&c));
    }

    #[test]
    fn channel_name_location_x() {
        /* LocationX maps to "loc_x" */
        assert_eq!(channel_name(TransformChannel::LocationX), "loc_x");
    }

    #[test]
    fn channel_name_scale_z() {
        /* ScaleZ maps to "scale_z" */
        assert_eq!(channel_name(TransformChannel::ScaleZ), "scale_z");
    }

    #[test]
    fn json_contains_label() {
        /* JSON output includes label */
        let c = new_transform_constraint(
            TransformChannel::RotationY,
            TransformChannel::ScaleX,
            "myTfm",
        );
        assert!(transform_constraint_to_json(&c).contains("myTfm"));
    }

    #[test]
    fn default_influence_is_one() {
        /* default influence is 1 */
        let c = new_transform_constraint(
            TransformChannel::LocationY,
            TransformChannel::LocationY,
            "t",
        );
        assert!((c.influence - 1.0).abs() < 1e-6);
    }
}
