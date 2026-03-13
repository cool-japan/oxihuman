// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Annotation layer management for viewport markups.
//!
//! Supports text labels, arrows, circles, boxes, and freehand paths as
//! lightweight 2-D annotations rendered on top of the viewer.

#![allow(dead_code)]

// ── Enums ─────────────────────────────────────────────────────────────────────

/// Shape or style of an annotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotationType {
    /// A text label anchored to a point.
    Text,
    /// A directional arrow between two points.
    Arrow,
    /// A circle defined by center and radius.
    Circle,
    /// An axis-aligned bounding box.
    Box,
    /// A freehand polyline path.
    Freehand,
}

// ── Structs ───────────────────────────────────────────────────────────────────

/// A single viewport annotation.
#[derive(Debug, Clone)]
pub struct Annotation {
    /// Unique identifier within the layer.
    pub id: u64,
    /// Annotation shape type.
    pub annotation_type: AnnotationType,
    /// 2-D position `[x, y]` (viewport pixels or normalized coordinates).
    pub position: [f32; 2],
    /// Optional second point (arrow head, box corner, etc.).
    pub end_point: Option<[f32; 2]>,
    /// Text content (used when `annotation_type == AnnotationType::Text`).
    pub text: String,
    /// RGBA color `[r, g, b, a]` in `0.0..=1.0`.
    pub color: [f32; 4],
    /// Stroke thickness in pixels.
    pub thickness: f32,
    /// Radius (used when `annotation_type == AnnotationType::Circle`).
    pub radius: f32,
    /// Additional polyline points (used for `Freehand`).
    pub points: Vec<[f32; 2]>,
}

/// A named layer that holds multiple [`Annotation`]s.
#[derive(Debug, Clone)]
pub struct AnnotationLayer {
    /// Layer name.
    pub name: String,
    /// Whether the layer and all its annotations are rendered.
    pub visible: bool,
    /// All annotations in this layer.
    pub annotations: Vec<Annotation>,
    /// Internal counter for generating unique annotation IDs.
    next_id: u64,
}

// ── Type aliases ──────────────────────────────────────────────────────────────

/// Axis-aligned bounding box `[min_x, min_y, max_x, max_y]`.
pub type AnnotationAabb = [f32; 4];

/// Result type for annotation operations.
pub type AnnotationResult = Result<(), String>;

// ── Functions ─────────────────────────────────────────────────────────────────

/// Construct a new, empty, visible [`AnnotationLayer`] with the given name.
#[allow(dead_code)]
pub fn new_annotation_layer(name: &str) -> AnnotationLayer {
    AnnotationLayer {
        name: name.to_string(),
        visible: true,
        annotations: Vec::new(),
        next_id: 1,
    }
}

/// Add `annotation` to `layer`, assigning it a unique ID.
/// Returns the assigned ID.
#[allow(dead_code)]
pub fn add_annotation(layer: &mut AnnotationLayer, mut annotation: Annotation) -> u64 {
    let id = layer.next_id;
    annotation.id = id;
    layer.next_id += 1;
    layer.annotations.push(annotation);
    id
}

/// Remove the annotation with `id` from `layer`.
/// Returns `Ok(())` if found and removed, `Err` otherwise.
#[allow(dead_code)]
pub fn remove_annotation(layer: &mut AnnotationLayer, id: u64) -> AnnotationResult {
    let before = layer.annotations.len();
    layer.annotations.retain(|a| a.id != id);
    if layer.annotations.len() < before {
        Ok(())
    } else {
        Err(format!("annotation id={} not found", id))
    }
}

/// Return the number of annotations in `layer`.
#[allow(dead_code)]
pub fn annotation_count(layer: &AnnotationLayer) -> usize {
    layer.annotations.len()
}

/// Return whether `layer` is visible.
#[allow(dead_code)]
pub fn layer_visible(layer: &AnnotationLayer) -> bool {
    layer.visible
}

/// Set the visibility of `layer`.
#[allow(dead_code)]
pub fn set_layer_visible(layer: &mut AnnotationLayer, visible: bool) {
    layer.visible = visible;
}

/// Remove all annotations from `layer`.
#[allow(dead_code)]
pub fn clear_annotations(layer: &mut AnnotationLayer) {
    layer.annotations.clear();
}

/// Return the name of `layer`.
#[allow(dead_code)]
pub fn layer_name(layer: &AnnotationLayer) -> &str {
    &layer.name
}

/// Set the name of `layer`.
#[allow(dead_code)]
pub fn set_layer_name(layer: &mut AnnotationLayer, name: &str) {
    layer.name = name.to_string();
}

/// Return a human-readable name for an [`AnnotationType`].
#[allow(dead_code)]
pub fn annotation_type_name(t: AnnotationType) -> &'static str {
    match t {
        AnnotationType::Text => "text",
        AnnotationType::Arrow => "arrow",
        AnnotationType::Circle => "circle",
        AnnotationType::Box => "box",
        AnnotationType::Freehand => "freehand",
    }
}

/// Find the first annotation within `radius` pixels of `point`, returning a reference.
#[allow(dead_code)]
pub fn find_annotation_at(layer: &AnnotationLayer, point: [f32; 2], radius: f32) -> Option<&Annotation> {
    layer.annotations.iter().find(|a| {
        let dx = a.position[0] - point[0];
        let dy = a.position[1] - point[1];
        (dx * dx + dy * dy).sqrt() <= radius
    })
}

/// Compute the axis-aligned bounding box `[min_x, min_y, max_x, max_y]` of all
/// annotation positions in `layer`.  Returns `None` if the layer is empty.
#[allow(dead_code)]
pub fn annotation_bounding_box(layer: &AnnotationLayer) -> Option<AnnotationAabb> {
    if layer.annotations.is_empty() {
        return None;
    }
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;

    for a in &layer.annotations {
        let points_to_check: Vec<[f32; 2]> = {
            let mut pts = vec![a.position];
            if let Some(ep) = a.end_point {
                pts.push(ep);
            }
            pts.extend_from_slice(&a.points);
            pts
        };
        for p in points_to_check {
            if p[0] < min_x { min_x = p[0]; }
            if p[1] < min_y { min_y = p[1]; }
            if p[0] > max_x { max_x = p[0]; }
            if p[1] > max_y { max_y = p[1]; }
        }
    }
    Some([min_x, min_y, max_x, max_y])
}

/// Move the annotation with `id` by `delta` pixels.
/// Returns `Err` if the annotation is not found.
#[allow(dead_code)]
pub fn move_annotation(layer: &mut AnnotationLayer, id: u64, delta: [f32; 2]) -> AnnotationResult {
    match layer.annotations.iter_mut().find(|a| a.id == id) {
        Some(a) => {
            a.position[0] += delta[0];
            a.position[1] += delta[1];
            if let Some(ref mut ep) = a.end_point {
                ep[0] += delta[0];
                ep[1] += delta[1];
            }
            for p in &mut a.points {
                p[0] += delta[0];
                p[1] += delta[1];
            }
            Ok(())
        }
        None => Err(format!("annotation id={} not found", id)),
    }
}

/// Serialize `layer` to a compact JSON string.
#[allow(dead_code)]
pub fn annotation_layer_to_json(layer: &AnnotationLayer) -> String {
    let annotations_json: Vec<String> = layer
        .annotations
        .iter()
        .map(|a| {
            let type_name = annotation_type_name(a.annotation_type);
            let ep = match a.end_point {
                Some(p) => format!("[{},{}]", p[0], p[1]),
                None => "null".to_string(),
            };
            format!(
                r#"{{"id":{},"type":"{}","position":[{},{}],"end_point":{},"text":"{}","color":[{},{},{},{}],"thickness":{},"radius":{}}}"#,
                a.id,
                type_name,
                a.position[0], a.position[1],
                ep,
                a.text,
                a.color[0], a.color[1], a.color[2], a.color[3],
                a.thickness,
                a.radius,
            )
        })
        .collect();
    format!(
        r#"{{"name":"{}","visible":{},"annotations":[{}]}}"#,
        layer.name,
        layer.visible,
        annotations_json.join(",")
    )
}

// ── Helper ────────────────────────────────────────────────────────────────────

/// Construct a basic [`Annotation`] with default styling (id will be assigned by `add_annotation`).
#[allow(dead_code)]
pub fn new_annotation(annotation_type: AnnotationType, position: [f32; 2], text: &str) -> Annotation {
    Annotation {
        id: 0,
        annotation_type,
        position,
        end_point: None,
        text: text.to_string(),
        color: [1.0, 1.0, 0.0, 1.0],
        thickness: 2.0,
        radius: 10.0,
        points: Vec::new(),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_layer() -> AnnotationLayer {
        new_annotation_layer("Viewport Notes")
    }

    fn make_text_annotation(x: f32, y: f32) -> Annotation {
        new_annotation(AnnotationType::Text, [x, y], "hello")
    }

    fn make_arrow_annotation() -> Annotation {
        let mut a = new_annotation(AnnotationType::Arrow, [0.0, 0.0], "");
        a.end_point = Some([100.0, 100.0]);
        a
    }

    #[test]
    fn test_new_annotation_layer_defaults() {
        let layer = make_layer();
        assert_eq!(layer.name, "Viewport Notes");
        assert!(layer.visible);
        assert!(layer.annotations.is_empty());
    }

    #[test]
    fn test_add_annotation_assigns_id() {
        let mut layer = make_layer();
        let id = add_annotation(&mut layer, make_text_annotation(10.0, 20.0));
        assert_eq!(id, 1);
        let id2 = add_annotation(&mut layer, make_text_annotation(30.0, 40.0));
        assert_eq!(id2, 2);
    }

    #[test]
    fn test_annotation_count() {
        let mut layer = make_layer();
        add_annotation(&mut layer, make_text_annotation(0.0, 0.0));
        add_annotation(&mut layer, make_text_annotation(1.0, 1.0));
        assert_eq!(annotation_count(&layer), 2);
    }

    #[test]
    fn test_remove_annotation_ok() {
        let mut layer = make_layer();
        let id = add_annotation(&mut layer, make_text_annotation(0.0, 0.0));
        assert!(remove_annotation(&mut layer, id).is_ok());
        assert_eq!(annotation_count(&layer), 0);
    }

    #[test]
    fn test_remove_annotation_not_found() {
        let mut layer = make_layer();
        assert!(remove_annotation(&mut layer, 999).is_err());
    }

    #[test]
    fn test_layer_visible_default() {
        let layer = make_layer();
        assert!(layer_visible(&layer));
    }

    #[test]
    fn test_set_layer_visible() {
        let mut layer = make_layer();
        set_layer_visible(&mut layer, false);
        assert!(!layer_visible(&layer));
        set_layer_visible(&mut layer, true);
        assert!(layer_visible(&layer));
    }

    #[test]
    fn test_clear_annotations() {
        let mut layer = make_layer();
        add_annotation(&mut layer, make_text_annotation(0.0, 0.0));
        add_annotation(&mut layer, make_text_annotation(1.0, 1.0));
        clear_annotations(&mut layer);
        assert_eq!(annotation_count(&layer), 0);
    }

    #[test]
    fn test_layer_name_get_set() {
        let mut layer = make_layer();
        assert_eq!(layer_name(&layer), "Viewport Notes");
        set_layer_name(&mut layer, "Measurements");
        assert_eq!(layer_name(&layer), "Measurements");
    }

    #[test]
    fn test_annotation_type_name_all() {
        assert_eq!(annotation_type_name(AnnotationType::Text), "text");
        assert_eq!(annotation_type_name(AnnotationType::Arrow), "arrow");
        assert_eq!(annotation_type_name(AnnotationType::Circle), "circle");
        assert_eq!(annotation_type_name(AnnotationType::Box), "box");
        assert_eq!(annotation_type_name(AnnotationType::Freehand), "freehand");
    }

    #[test]
    fn test_find_annotation_at_found() {
        let mut layer = make_layer();
        add_annotation(&mut layer, make_text_annotation(50.0, 50.0));
        let found = find_annotation_at(&layer, [52.0, 52.0], 5.0);
        assert!(found.is_some());
    }

    #[test]
    fn test_find_annotation_at_not_found() {
        let mut layer = make_layer();
        add_annotation(&mut layer, make_text_annotation(50.0, 50.0));
        let found = find_annotation_at(&layer, [200.0, 200.0], 5.0);
        assert!(found.is_none());
    }

    #[test]
    fn test_annotation_bounding_box_empty() {
        let layer = make_layer();
        assert!(annotation_bounding_box(&layer).is_none());
    }

    #[test]
    fn test_annotation_bounding_box_single() {
        let mut layer = make_layer();
        add_annotation(&mut layer, make_text_annotation(10.0, 20.0));
        let bbox = annotation_bounding_box(&layer).expect("should succeed");
        assert_eq!(bbox[0], 10.0);
        assert_eq!(bbox[1], 20.0);
        assert_eq!(bbox[2], 10.0);
        assert_eq!(bbox[3], 20.0);
    }

    #[test]
    fn test_annotation_bounding_box_multiple() {
        let mut layer = make_layer();
        add_annotation(&mut layer, make_text_annotation(0.0, 0.0));
        add_annotation(&mut layer, make_text_annotation(100.0, 200.0));
        let bbox = annotation_bounding_box(&layer).expect("should succeed");
        assert_eq!(bbox[0], 0.0);
        assert_eq!(bbox[1], 0.0);
        assert_eq!(bbox[2], 100.0);
        assert_eq!(bbox[3], 200.0);
    }

    #[test]
    fn test_move_annotation_ok() {
        let mut layer = make_layer();
        let id = add_annotation(&mut layer, make_text_annotation(10.0, 20.0));
        assert!(move_annotation(&mut layer, id, [5.0, -5.0]).is_ok());
        let a = layer.annotations.iter().find(|a| a.id == id).expect("should succeed");
        assert!((a.position[0] - 15.0).abs() < 1e-5);
        assert!((a.position[1] - 15.0).abs() < 1e-5);
    }

    #[test]
    fn test_move_annotation_not_found() {
        let mut layer = make_layer();
        assert!(move_annotation(&mut layer, 999, [1.0, 1.0]).is_err());
    }

    #[test]
    fn test_annotation_layer_to_json_contains_name() {
        let mut layer = make_layer();
        add_annotation(&mut layer, make_text_annotation(0.0, 0.0));
        let json = annotation_layer_to_json(&layer);
        assert!(json.contains("\"name\":\"Viewport Notes\""));
    }

    #[test]
    fn test_annotation_layer_to_json_contains_type() {
        let mut layer = make_layer();
        add_annotation(&mut layer, make_arrow_annotation());
        let json = annotation_layer_to_json(&layer);
        assert!(json.contains("\"type\":\"arrow\""));
    }

    #[test]
    fn test_move_annotation_also_moves_end_point() {
        let mut layer = make_layer();
        let id = add_annotation(&mut layer, make_arrow_annotation());
        move_annotation(&mut layer, id, [10.0, 10.0]).expect("should succeed");
        let a = layer.annotations.iter().find(|a| a.id == id).expect("should succeed");
        if let Some(ep) = a.end_point {
            assert!((ep[0] - 110.0).abs() < 1e-5);
            assert!((ep[1] - 110.0).abs() < 1e-5);
        } else {
            panic!("end_point should be Some");
        }
    }
}
