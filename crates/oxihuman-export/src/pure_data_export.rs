// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Pure Data (.pd) patch stub export.

/// A Pd object (box).
#[derive(Debug, Clone)]
pub struct PdObject {
    pub x: i32,
    pub y: i32,
    pub class_name: String,
    pub args: Vec<String>,
}

impl PdObject {
    pub fn new(x: i32, y: i32, class_name: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            x,
            y,
            class_name: class_name.into(),
            args,
        }
    }

    pub fn to_pd_line(&self) -> String {
        /* #X obj x y class_name args; */
        if self.args.is_empty() {
            format!("#X obj {} {} {};", self.x, self.y, self.class_name)
        } else {
            format!(
                "#X obj {} {} {} {};",
                self.x,
                self.y,
                self.class_name,
                self.args.join(" ")
            )
        }
    }
}

/// A Pd message box.
#[derive(Debug, Clone)]
pub struct PdMessage {
    pub x: i32,
    pub y: i32,
    pub content: String,
}

impl PdMessage {
    pub fn new(x: i32, y: i32, content: impl Into<String>) -> Self {
        Self {
            x,
            y,
            content: content.into(),
        }
    }

    pub fn to_pd_line(&self) -> String {
        format!("#X msg {} {} {};", self.x, self.y, self.content)
    }
}

/// A connection between two Pd objects.
#[derive(Debug, Clone)]
pub struct PdConnect {
    pub src_idx: usize,
    pub src_outlet: usize,
    pub dst_idx: usize,
    pub dst_inlet: usize,
}

/// A Pure Data patch.
#[derive(Debug, Clone, Default)]
pub struct PdPatch {
    pub objects: Vec<PdObject>,
    pub messages: Vec<PdMessage>,
    pub connections: Vec<PdConnect>,
    pub canvas_w: i32,
    pub canvas_h: i32,
}

impl PdPatch {
    pub fn new(canvas_w: i32, canvas_h: i32) -> Self {
        Self {
            canvas_w,
            canvas_h,
            ..Default::default()
        }
    }

    pub fn add_object(&mut self, obj: PdObject) -> usize {
        self.objects.push(obj);
        self.objects.len() - 1
    }

    pub fn add_message(&mut self, msg: PdMessage) {
        self.messages.push(msg);
    }

    pub fn connect(&mut self, src_idx: usize, src_outlet: usize, dst_idx: usize, dst_inlet: usize) {
        self.connections.push(PdConnect {
            src_idx,
            src_outlet,
            dst_idx,
            dst_inlet,
        });
    }
}

/// Export a PdPatch to a .pd text file string.
pub fn export_pd_patch(patch: &PdPatch) -> String {
    let mut lines = Vec::new();
    /* Canvas header */
    lines.push(format!(
        "#N canvas 0 0 {} {} 12;",
        patch.canvas_w, patch.canvas_h
    ));
    for obj in &patch.objects {
        lines.push(obj.to_pd_line());
    }
    for msg in &patch.messages {
        lines.push(msg.to_pd_line());
    }
    for conn in &patch.connections {
        lines.push(format!(
            "#X connect {} {} {} {};",
            conn.src_idx, conn.src_outlet, conn.dst_idx, conn.dst_inlet
        ));
    }
    lines.join("\n") + "\n"
}

/// Count objects in a patch.
pub fn count_pd_objects(patch: &PdPatch) -> usize {
    patch.objects.len()
}

/// Build a minimal Pd sine oscillator patch.
pub fn sine_osc_pd_patch() -> PdPatch {
    let mut patch = PdPatch::new(640, 480);
    let osc_idx = patch.add_object(PdObject::new(100, 100, "osc~", vec!["440".to_string()]));
    let dac_idx = patch.add_object(PdObject::new(100, 200, "dac~", vec![]));
    patch.connect(osc_idx, 0, dac_idx, 0);
    patch.connect(osc_idx, 0, dac_idx, 1);
    patch
}

/// Validate that a string looks like a Pd patch.
pub fn is_valid_pd_patch(src: &str) -> bool {
    src.contains("#N canvas") && src.contains("#X")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pd_object_line_no_args() {
        let obj = PdObject::new(10, 20, "print", vec![]);
        let line = obj.to_pd_line();
        assert!(line.contains("#X obj") /* obj keyword */);
        assert!(line.contains("print") /* class name */);
    }

    #[test]
    fn test_pd_object_line_with_args() {
        let obj = PdObject::new(0, 0, "osc~", vec!["440".to_string()]);
        let line = obj.to_pd_line();
        assert!(line.contains("440") /* frequency arg */);
    }

    #[test]
    fn test_pd_message_line() {
        let msg = PdMessage::new(50, 50, "bang");
        let line = msg.to_pd_line();
        assert!(line.contains("#X msg") /* message keyword */);
        assert!(line.contains("bang") /* content */);
    }

    #[test]
    fn test_export_pd_patch_canvas() {
        let patch = PdPatch::new(640, 480);
        let src = export_pd_patch(&patch);
        assert!(src.contains("#N canvas") /* canvas header */);
    }

    #[test]
    fn test_count_pd_objects() {
        let mut patch = PdPatch::new(640, 480);
        patch.add_object(PdObject::new(0, 0, "osc~", vec![]));
        assert_eq!(count_pd_objects(&patch), 1 /* one object */);
    }

    #[test]
    fn test_sine_osc_pd_patch() {
        let patch = sine_osc_pd_patch();
        assert_eq!(count_pd_objects(&patch), 2 /* osc~ and dac~ */);
    }

    #[test]
    fn test_export_contains_connect() {
        let patch = sine_osc_pd_patch();
        let src = export_pd_patch(&patch);
        assert!(src.contains("#X connect") /* connection lines */);
    }

    #[test]
    fn test_is_valid_pd_patch() {
        /* patch with at least one object produces both #N canvas and #X */
        let patch = sine_osc_pd_patch();
        let src = export_pd_patch(&patch);
        assert!(is_valid_pd_patch(&src) /* valid patch */);
    }

    #[test]
    fn test_is_valid_pd_patch_false() {
        assert!(!is_valid_pd_patch("not a patch") /* invalid */);
    }
}
