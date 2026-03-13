// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Max/MSP patch stub export (JSON-based maxpat format).

/// A Max/MSP object (box) in the patch.
#[derive(Debug, Clone)]
pub struct MaxBox {
    pub id: String,
    pub class_name: String,
    pub text: String,
    pub position: [f64; 2],
}

impl MaxBox {
    pub fn new(
        id: impl Into<String>,
        class_name: impl Into<String>,
        text: impl Into<String>,
        x: f64,
        y: f64,
    ) -> Self {
        Self {
            id: id.into(),
            class_name: class_name.into(),
            text: text.into(),
            position: [x, y],
        }
    }
}

/// A connection between two Max boxes.
#[derive(Debug, Clone)]
pub struct MaxPatchCord {
    pub src_id: String,
    pub src_outlet: usize,
    pub dst_id: String,
    pub dst_inlet: usize,
}

/// A Max/MSP patch.
#[derive(Debug, Clone, Default)]
pub struct MaxPatch {
    pub boxes: Vec<MaxBox>,
    pub patchcords: Vec<MaxPatchCord>,
    pub description: String,
}

impl MaxPatch {
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            boxes: Vec::new(),
            patchcords: Vec::new(),
        }
    }

    pub fn add_box(&mut self, b: MaxBox) {
        self.boxes.push(b);
    }

    pub fn connect(
        &mut self,
        src_id: impl Into<String>,
        src_outlet: usize,
        dst_id: impl Into<String>,
        dst_inlet: usize,
    ) {
        self.patchcords.push(MaxPatchCord {
            src_id: src_id.into(),
            src_outlet,
            dst_id: dst_id.into(),
            dst_inlet,
        });
    }
}

/// Generate a minimal Max/MSP patch JSON string (maxpat format stub).
pub fn generate_maxpat_json(patch: &MaxPatch) -> String {
    let mut json = String::new();
    json.push_str("{\n    \"patcher\": {\n");
    json.push_str(&format!(
        "        \"description\": \"{}\",\n",
        patch.description
    ));
    json.push_str("        \"boxes\": [\n");
    for (i, b) in patch.boxes.iter().enumerate() {
        let comma = if i + 1 < patch.boxes.len() { "," } else { "" };
        json.push_str(&format!(
            "            {{\"box\": {{\"id\": \"{}\", \"maxclass\": \"{}\", \"text\": \"{}\", \"patching_rect\": [{}, {}, 100, 25]}}}}{}\n",
            b.id, b.class_name, b.text, b.position[0], b.position[1], comma
        ));
    }
    json.push_str("        ],\n");
    json.push_str("        \"lines\": [\n");
    for (i, cord) in patch.patchcords.iter().enumerate() {
        let comma = if i + 1 < patch.patchcords.len() {
            ","
        } else {
            ""
        };
        json.push_str(&format!(
            "            {{\"patchline\": {{\"source\": [\"{}\", {}], \"destination\": [\"{}\", {}]}}}}{}\n",
            cord.src_id, cord.src_outlet, cord.dst_id, cord.dst_inlet, comma
        ));
    }
    json.push_str("        ]\n");
    json.push_str("    }\n}\n");
    json
}

/// Count boxes in a patch.
pub fn count_max_boxes(patch: &MaxPatch) -> usize {
    patch.boxes.len()
}

/// Count connections in a patch.
pub fn count_patchcords(patch: &MaxPatch) -> usize {
    patch.patchcords.len()
}

/// Build a minimal sine oscillator Max patch.
pub fn sine_osc_max_patch() -> MaxPatch {
    let mut patch = MaxPatch::new("Sine oscillator example");
    patch.add_box(MaxBox::new("b1", "cycle~", "cycle~ 440", 100.0, 100.0));
    patch.add_box(MaxBox::new("b2", "dac~", "dac~", 100.0, 200.0));
    patch.connect("b1", 0, "b2", 0);
    patch.connect("b1", 0, "b2", 1);
    patch
}

/// Validate that a JSON string looks like a maxpat.
pub fn is_valid_maxpat(json: &str) -> bool {
    json.contains("\"patcher\"") && json.contains("\"boxes\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_box_position() {
        let b = MaxBox::new("x", "object", "text", 10.0, 20.0);
        assert_eq!(b.position, [10.0, 20.0] /* correct position */);
    }

    #[test]
    fn test_add_box_count() {
        let mut patch = MaxPatch::new("test");
        patch.add_box(MaxBox::new("b1", "cycle~", "cycle~ 440", 0.0, 0.0));
        assert_eq!(count_max_boxes(&patch), 1 /* one box */);
    }

    #[test]
    fn test_connect_count() {
        let mut patch = MaxPatch::new("test");
        patch.connect("b1", 0, "b2", 0);
        assert_eq!(count_patchcords(&patch), 1 /* one connection */);
    }

    #[test]
    fn test_generate_maxpat_json_valid() {
        let patch = MaxPatch::new("test");
        let json = generate_maxpat_json(&patch);
        assert!(is_valid_maxpat(&json) /* valid maxpat JSON */);
    }

    #[test]
    fn test_sine_osc_patch_boxes() {
        let patch = sine_osc_max_patch();
        assert_eq!(count_max_boxes(&patch), 2 /* cycle~ and dac~ */);
    }

    #[test]
    fn test_sine_osc_patch_connections() {
        let patch = sine_osc_max_patch();
        assert_eq!(count_patchcords(&patch), 2 /* two connections */);
    }

    #[test]
    fn test_generated_json_contains_cycle() {
        let patch = sine_osc_max_patch();
        let json = generate_maxpat_json(&patch);
        assert!(json.contains("cycle~") /* oscillator present */);
    }

    #[test]
    fn test_is_valid_maxpat_false() {
        assert!(!is_valid_maxpat("{\"not\": \"maxpat\"}") /* missing keys */);
    }

    #[test]
    fn test_description_in_json() {
        let patch = MaxPatch::new("my patch");
        let json = generate_maxpat_json(&patch);
        assert!(json.contains("my patch") /* description included */);
    }
}
