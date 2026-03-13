// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Viewport annotation system (labels, arrows, measurements, highlights).

// ── Types ─────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Clone)]
pub enum AnnotationType {
    Label(String),
    Arrow,
    Measurement,
    Highlight,
    BoundingBox,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Annotation {
    pub id: u64,
    pub annotation_type: AnnotationType,
    pub world_position: [f32; 3],
    pub screen_offset: [f32; 2],
    pub color: [f32; 4],
    pub visible: bool,
    pub scale: f32,
}

#[allow(dead_code)]
pub struct MeasurementAnnotation {
    pub id: u64,
    pub point_a: [f32; 3],
    pub point_b: [f32; 3],
    pub label: String,
    pub color: [f32; 4],
}

#[allow(dead_code)]
pub struct AnnotationLayer {
    pub name: String,
    pub annotations: Vec<Annotation>,
    pub measurements: Vec<MeasurementAnnotation>,
    pub visible: bool,
    next_id: u64,
}

// ── Functions ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn new_layer(name: &str) -> AnnotationLayer {
    AnnotationLayer {
        name: name.to_string(),
        annotations: Vec::new(),
        measurements: Vec::new(),
        visible: true,
        next_id: 1,
    }
}

fn alloc_id(layer: &mut AnnotationLayer) -> u64 {
    let id = layer.next_id;
    layer.next_id += 1;
    id
}

#[allow(dead_code)]
pub fn add_label(layer: &mut AnnotationLayer, pos: [f32; 3], text: &str, color: [f32; 4]) -> u64 {
    let id = alloc_id(layer);
    layer.annotations.push(Annotation {
        id,
        annotation_type: AnnotationType::Label(text.to_string()),
        world_position: pos,
        screen_offset: [0.0, 0.0],
        color,
        visible: true,
        scale: 1.0,
    });
    id
}

#[allow(dead_code)]
pub fn add_arrow(layer: &mut AnnotationLayer, pos: [f32; 3], color: [f32; 4]) -> u64 {
    let id = alloc_id(layer);
    layer.annotations.push(Annotation {
        id,
        annotation_type: AnnotationType::Arrow,
        world_position: pos,
        screen_offset: [0.0, 0.0],
        color,
        visible: true,
        scale: 1.0,
    });
    id
}

#[allow(dead_code)]
pub fn add_measurement(layer: &mut AnnotationLayer, a: [f32; 3], b: [f32; 3], label: &str) -> u64 {
    let id = alloc_id(layer);
    layer.measurements.push(MeasurementAnnotation {
        id,
        point_a: a,
        point_b: b,
        label: label.to_string(),
        color: [1.0, 1.0, 0.0, 1.0],
    });
    id
}

#[allow(dead_code)]
pub fn add_highlight(layer: &mut AnnotationLayer, pos: [f32; 3], color: [f32; 4]) -> u64 {
    let id = alloc_id(layer);
    layer.annotations.push(Annotation {
        id,
        annotation_type: AnnotationType::Highlight,
        world_position: pos,
        screen_offset: [0.0, 0.0],
        color,
        visible: true,
        scale: 1.0,
    });
    id
}

#[allow(dead_code)]
pub fn remove_annotation(layer: &mut AnnotationLayer, id: u64) -> bool {
    let before = layer.annotations.len() + layer.measurements.len();
    layer.annotations.retain(|a| a.id != id);
    layer.measurements.retain(|m| m.id != id);
    let after = layer.annotations.len() + layer.measurements.len();
    after < before
}

#[allow(dead_code)]
pub fn get_annotation(layer: &AnnotationLayer, id: u64) -> Option<&Annotation> {
    layer.annotations.iter().find(|a| a.id == id)
}

#[allow(dead_code)]
pub fn visible_annotations(layer: &AnnotationLayer) -> Vec<&Annotation> {
    layer.annotations.iter().filter(|a| a.visible).collect()
}

#[allow(dead_code)]
pub fn set_layer_visible(layer: &mut AnnotationLayer, visible: bool) {
    layer.visible = visible;
}

#[allow(dead_code)]
pub fn project_to_screen(world_pos: [f32; 3], view_proj: &[[f32; 4]; 4]) -> [f32; 2] {
    // Homogeneous transform: clip = view_proj * [x, y, z, 1]
    let x = world_pos[0];
    let y = world_pos[1];
    let z = world_pos[2];
    let w_in = 1.0f32;

    // view_proj is column-major: m[col][row]
    let cx =
        view_proj[0][0] * x + view_proj[1][0] * y + view_proj[2][0] * z + view_proj[3][0] * w_in;
    let cy =
        view_proj[0][1] * x + view_proj[1][1] * y + view_proj[2][1] * z + view_proj[3][1] * w_in;
    let cw =
        view_proj[0][3] * x + view_proj[1][3] * y + view_proj[2][3] * z + view_proj[3][3] * w_in;

    if cw.abs() < 1e-9 {
        return [0.0, 0.0];
    }
    // NDC to [0,1]
    [(cx / cw + 1.0) * 0.5, (cy / cw + 1.0) * 0.5]
}

#[allow(dead_code)]
pub fn measurement_length(m: &MeasurementAnnotation) -> f32 {
    let d = [
        m.point_b[0] - m.point_a[0],
        m.point_b[1] - m.point_a[1],
        m.point_b[2] - m.point_a[2],
    ];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

#[allow(dead_code)]
pub fn annotation_count(layer: &AnnotationLayer) -> usize {
    layer.annotations.len() + layer.measurements.len()
}

#[allow(dead_code)]
pub fn clear_layer(layer: &mut AnnotationLayer) {
    layer.annotations.clear();
    layer.measurements.clear();
}

#[allow(dead_code)]
pub fn measurement_midpoint(m: &MeasurementAnnotation) -> [f32; 3] {
    [
        (m.point_a[0] + m.point_b[0]) * 0.5,
        (m.point_a[1] + m.point_b[1]) * 0.5,
        (m.point_a[2] + m.point_b[2]) * 0.5,
    ]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_layer() {
        let layer = new_layer("Test");
        assert_eq!(layer.name, "Test");
        assert!(layer.annotations.is_empty());
        assert!(layer.measurements.is_empty());
        assert!(layer.visible);
    }

    #[test]
    fn test_add_label_and_get() {
        let mut layer = new_layer("L");
        let id = add_label(&mut layer, [1.0, 2.0, 3.0], "Hello", [1.0; 4]);
        let ann = get_annotation(&layer, id);
        assert!(ann.is_some());
        let a = ann.expect("should succeed");
        assert_eq!(a.id, id);
        assert_eq!(a.world_position, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_add_arrow() {
        let mut layer = new_layer("L");
        let id = add_arrow(&mut layer, [0.0; 3], [1.0, 0.0, 0.0, 1.0]);
        assert!(get_annotation(&layer, id).is_some());
    }

    #[test]
    fn test_add_measurement() {
        let mut layer = new_layer("L");
        let id = add_measurement(&mut layer, [0.0; 3], [3.0, 4.0, 0.0], "dist");
        // measurements don't appear in get_annotation
        assert_eq!(layer.measurements.len(), 1);
        assert_eq!(layer.measurements[0].id, id);
    }

    #[test]
    fn test_add_highlight() {
        let mut layer = new_layer("L");
        let id = add_highlight(&mut layer, [0.0; 3], [0.0, 1.0, 0.0, 1.0]);
        assert!(get_annotation(&layer, id).is_some());
    }

    #[test]
    fn test_remove_annotation() {
        let mut layer = new_layer("L");
        let id = add_label(&mut layer, [0.0; 3], "Test", [1.0; 4]);
        let removed = remove_annotation(&mut layer, id);
        assert!(removed);
        assert!(get_annotation(&layer, id).is_none());
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut layer = new_layer("L");
        assert!(!remove_annotation(&mut layer, 999));
    }

    #[test]
    fn test_visible_annotations_filter() {
        let mut layer = new_layer("L");
        let id1 = add_label(&mut layer, [0.0; 3], "A", [1.0; 4]);
        let id2 = add_label(&mut layer, [0.0; 3], "B", [1.0; 4]);
        // Hide second annotation
        if let Some(a) = layer.annotations.iter_mut().find(|a| a.id == id2) {
            a.visible = false;
        }
        let visible = visible_annotations(&layer);
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].id, id1);
    }

    #[test]
    fn test_measurement_length() {
        let m = MeasurementAnnotation {
            id: 1,
            point_a: [0.0, 0.0, 0.0],
            point_b: [3.0, 4.0, 0.0],
            label: "d".to_string(),
            color: [1.0; 4],
        };
        assert!((measurement_length(&m) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_measurement_midpoint() {
        let m = MeasurementAnnotation {
            id: 1,
            point_a: [0.0, 0.0, 0.0],
            point_b: [2.0, 4.0, 6.0],
            label: "m".to_string(),
            color: [1.0; 4],
        };
        let mid = measurement_midpoint(&m);
        assert!((mid[0] - 1.0).abs() < 1e-5);
        assert!((mid[1] - 2.0).abs() < 1e-5);
        assert!((mid[2] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_clear_layer() {
        let mut layer = new_layer("L");
        add_label(&mut layer, [0.0; 3], "A", [1.0; 4]);
        add_measurement(&mut layer, [0.0; 3], [1.0; 3], "m");
        clear_layer(&mut layer);
        assert_eq!(annotation_count(&layer), 0);
    }

    #[test]
    fn test_annotation_count() {
        let mut layer = new_layer("L");
        add_label(&mut layer, [0.0; 3], "A", [1.0; 4]);
        add_measurement(&mut layer, [0.0; 3], [1.0; 3], "m");
        assert_eq!(annotation_count(&layer), 2);
    }

    #[test]
    fn test_set_layer_visible() {
        let mut layer = new_layer("L");
        set_layer_visible(&mut layer, false);
        assert!(!layer.visible);
        set_layer_visible(&mut layer, true);
        assert!(layer.visible);
    }

    #[test]
    fn test_project_to_screen_identity() {
        // Identity matrix (column-major)
        let vp: [[f32; 4]; 4] = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let screen = project_to_screen([0.0, 0.0, 0.0], &vp);
        assert!((screen[0] - 0.5).abs() < 1e-5);
        assert!((screen[1] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_ids_unique() {
        let mut layer = new_layer("L");
        let id1 = add_label(&mut layer, [0.0; 3], "A", [1.0; 4]);
        let id2 = add_label(&mut layer, [0.0; 3], "B", [1.0; 4]);
        let id3 = add_arrow(&mut layer, [0.0; 3], [1.0; 4]);
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
    }
}
