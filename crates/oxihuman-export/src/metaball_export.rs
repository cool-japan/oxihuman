// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A single metaball element.
#[allow(dead_code)]
pub struct MetaballElement {
    pub element_type: u8,
    pub position: [f32; 3],
    pub radius: f32,
    pub stiffness: f32,
}

/// A metaball object export.
#[allow(dead_code)]
pub struct MetaballExport {
    pub name: String,
    pub resolution: f32,
    pub threshold: f32,
    pub elements: Vec<MetaballElement>,
}

/// Create a new metaball export.
#[allow(dead_code)]
pub fn new_metaball_export(name: &str) -> MetaballExport {
    MetaballExport {
        name: name.to_string(),
        resolution: 0.2,
        threshold: 0.6,
        elements: Vec::new(),
    }
}

/// Add a metaball element.
#[allow(dead_code)]
pub fn add_element(m: &mut MetaballExport, pos: [f32; 3], radius: f32) {
    m.elements.push(MetaballElement {
        element_type: 0,
        position: pos,
        radius,
        stiffness: 2.0,
    });
}

/// Count elements.
#[allow(dead_code)]
pub fn element_count(m: &MetaballExport) -> usize {
    m.elements.len()
}

/// Export to JSON.
#[allow(dead_code)]
pub fn export_metaball_to_json(m: &MetaballExport) -> String {
    let elems: String = m
        .elements
        .iter()
        .map(|e| {
            format!(
                r#"{{"type":{},"pos":[{},{},{}],"radius":{},"stiffness":{}}}"#,
                e.element_type,
                e.position[0],
                e.position[1],
                e.position[2],
                e.radius,
                e.stiffness
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"{{"name":"{}","resolution":{},"threshold":{},"elements":[{}]}}"#,
        m.name, m.resolution, m.threshold, elems
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_metaball_export_name() {
        let m = new_metaball_export("Meta");
        assert_eq!(m.name, "Meta");
    }

    #[test]
    fn test_new_metaball_export_empty_elements() {
        let m = new_metaball_export("M");
        assert!(m.elements.is_empty());
    }

    #[test]
    fn test_new_metaball_export_resolution() {
        let m = new_metaball_export("M");
        assert!((m.resolution - 0.2).abs() < 1e-5);
    }

    #[test]
    fn test_add_element_increases_count() {
        let mut m = new_metaball_export("M");
        add_element(&mut m, [0.0, 0.0, 0.0], 1.0);
        assert_eq!(element_count(&m), 1);
    }

    #[test]
    fn test_add_multiple_elements() {
        let mut m = new_metaball_export("M");
        add_element(&mut m, [0.0, 0.0, 0.0], 0.5);
        add_element(&mut m, [1.0, 0.0, 0.0], 0.75);
        assert_eq!(element_count(&m), 2);
    }

    #[test]
    fn test_add_element_radius() {
        let mut m = new_metaball_export("M");
        add_element(&mut m, [0.0, 0.0, 0.0], 1.5);
        assert!((m.elements[0].radius - 1.5).abs() < 1e-5);
    }

    #[test]
    fn test_export_metaball_to_json_name() {
        let m = new_metaball_export("MetaBall");
        let json = export_metaball_to_json(&m);
        assert!(json.contains("MetaBall"));
    }

    #[test]
    fn test_export_metaball_to_json_with_elements() {
        let mut m = new_metaball_export("M");
        add_element(&mut m, [0.0, 1.0, 0.0], 0.5);
        let json = export_metaball_to_json(&m);
        assert!(json.contains("radius"));
    }

    #[test]
    fn test_element_count_helper() {
        let mut m = new_metaball_export("M");
        add_element(&mut m, [0.0, 0.0, 0.0], 1.0);
        assert_eq!(element_count(&m), 1);
    }

    #[test]
    fn test_export_json_structure() {
        let m = new_metaball_export("M");
        let json = export_metaball_to_json(&m);
        assert!(json.starts_with('{') && json.ends_with('}'));
    }
}
