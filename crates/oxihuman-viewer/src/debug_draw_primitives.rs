// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Immediate-mode debug drawing for lines, boxes, spheres, and text.

// ── Enums ─────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum DebugPrimitive {
    Line {
        start: [f32; 3],
        end: [f32; 3],
    },
    Box {
        center: [f32; 3],
        half_extents: [f32; 3],
    },
    Sphere {
        center: [f32; 3],
        radius: f32,
    },
    Text {
        position: [f32; 3],
        content: String,
    },
}

// ── Structs ───────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DebugDrawConfig {
    pub line_width: f32,
    pub default_color: [f32; 4],
    pub depth_test: bool,
    pub max_primitives: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DebugPrimDrawList {
    pub primitives: Vec<DebugPrimitive>,
    pub colors: Vec<[f32; 4]>,
    pub config: DebugDrawConfig,
}

// ── Functions ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn default_debug_draw_config() -> DebugDrawConfig {
    DebugDrawConfig {
        line_width: 1.0,
        default_color: [1.0, 1.0, 0.0, 1.0],
        depth_test: true,
        max_primitives: 4096,
    }
}

#[allow(dead_code)]
pub fn new_debug_draw_list(cfg: DebugDrawConfig) -> DebugPrimDrawList {
    DebugPrimDrawList {
        primitives: Vec::new(),
        colors: Vec::new(),
        config: cfg,
    }
}

#[allow(dead_code)]
pub fn draw_line(
    list: &mut DebugPrimDrawList,
    start: [f32; 3],
    end: [f32; 3],
    color: [f32; 4],
) {
    if list.primitives.len() >= list.config.max_primitives {
        return;
    }
    list.primitives.push(DebugPrimitive::Line { start, end });
    list.colors.push(color);
}

#[allow(dead_code)]
pub fn draw_box(
    list: &mut DebugPrimDrawList,
    center: [f32; 3],
    half: [f32; 3],
    color: [f32; 4],
) {
    if list.primitives.len() >= list.config.max_primitives {
        return;
    }
    list.primitives.push(DebugPrimitive::Box {
        center,
        half_extents: half,
    });
    list.colors.push(color);
}

#[allow(dead_code)]
pub fn draw_sphere(
    list: &mut DebugPrimDrawList,
    center: [f32; 3],
    radius: f32,
    color: [f32; 4],
) {
    if list.primitives.len() >= list.config.max_primitives {
        return;
    }
    list.primitives.push(DebugPrimitive::Sphere { center, radius });
    list.colors.push(color);
}

#[allow(dead_code)]
pub fn draw_text(
    list: &mut DebugPrimDrawList,
    pos: [f32; 3],
    text: &str,
    color: [f32; 4],
) {
    if list.primitives.len() >= list.config.max_primitives {
        return;
    }
    list.primitives.push(DebugPrimitive::Text {
        position: pos,
        content: text.to_string(),
    });
    list.colors.push(color);
}

#[allow(dead_code)]
pub fn clear_debug_list(list: &mut DebugPrimDrawList) {
    list.primitives.clear();
    list.colors.clear();
}

#[allow(dead_code)]
pub fn primitive_count(list: &DebugPrimDrawList) -> usize {
    list.primitives.len()
}

#[allow(dead_code)]
pub fn debug_prim_draw_to_json(list: &DebugPrimDrawList) -> String {
    let entries: Vec<String> = list
        .primitives
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let color = list.colors.get(i).copied().unwrap_or([1.0, 1.0, 1.0, 1.0]);
            let color_str = format!(
                "[{},{},{},{}]",
                color[0], color[1], color[2], color[3]
            );
            match p {
                DebugPrimitive::Line { start, end } => format!(
                    r#"{{"type":"line","start":[{},{},{}],"end":[{},{},{}],"color":{}}}"#,
                    start[0], start[1], start[2],
                    end[0], end[1], end[2],
                    color_str
                ),
                DebugPrimitive::Box { center, half_extents } => format!(
                    r#"{{"type":"box","center":[{},{},{}],"half":[{},{},{}],"color":{}}}"#,
                    center[0], center[1], center[2],
                    half_extents[0], half_extents[1], half_extents[2],
                    color_str
                ),
                DebugPrimitive::Sphere { center, radius } => format!(
                    r#"{{"type":"sphere","center":[{},{},{}],"radius":{},"color":{}}}"#,
                    center[0], center[1], center[2],
                    radius,
                    color_str
                ),
                DebugPrimitive::Text { position, content } => format!(
                    r#"{{"type":"text","position":[{},{},{}],"content":"{}","color":{}}}"#,
                    position[0], position[1], position[2],
                    content,
                    color_str
                ),
            }
        })
        .collect();
    format!(r#"{{"primitives":[{}]}}"#, entries.join(","))
}

#[allow(dead_code)]
pub fn is_debug_full(list: &DebugPrimDrawList) -> bool {
    list.primitives.len() >= list.config.max_primitives
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_debug_draw_config();
        assert!((cfg.line_width - 1.0).abs() < 1e-6);
        assert!(cfg.depth_test);
        assert_eq!(cfg.max_primitives, 4096);
    }

    #[test]
    fn new_list_is_empty() {
        let cfg = default_debug_draw_config();
        let list = new_debug_draw_list(cfg);
        assert_eq!(primitive_count(&list), 0);
        assert!(!is_debug_full(&list));
    }

    #[test]
    fn draw_line_adds_primitive() {
        let cfg = default_debug_draw_config();
        let mut list = new_debug_draw_list(cfg);
        draw_line(&mut list, [0.0; 3], [1.0; 3], [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(primitive_count(&list), 1);
    }

    #[test]
    fn draw_box_adds_primitive() {
        let cfg = default_debug_draw_config();
        let mut list = new_debug_draw_list(cfg);
        draw_box(&mut list, [0.0; 3], [0.5; 3], [0.0, 1.0, 0.0, 1.0]);
        assert_eq!(primitive_count(&list), 1);
    }

    #[test]
    fn draw_sphere_adds_primitive() {
        let cfg = default_debug_draw_config();
        let mut list = new_debug_draw_list(cfg);
        draw_sphere(&mut list, [0.0; 3], 1.0, [0.0, 0.0, 1.0, 1.0]);
        assert_eq!(primitive_count(&list), 1);
    }

    #[test]
    fn draw_text_adds_primitive() {
        let cfg = default_debug_draw_config();
        let mut list = new_debug_draw_list(cfg);
        draw_text(&mut list, [1.0, 2.0, 3.0], "hello", [1.0; 4]);
        assert_eq!(primitive_count(&list), 1);
    }

    #[test]
    fn clear_debug_list_empties() {
        let cfg = default_debug_draw_config();
        let mut list = new_debug_draw_list(cfg);
        draw_line(&mut list, [0.0; 3], [1.0; 3], [1.0; 4]);
        draw_sphere(&mut list, [0.0; 3], 0.5, [1.0; 4]);
        clear_debug_list(&mut list);
        assert_eq!(primitive_count(&list), 0);
    }

    #[test]
    fn max_primitives_cap_respected() {
        let cfg = DebugDrawConfig {
            line_width: 1.0,
            default_color: [1.0; 4],
            depth_test: false,
            max_primitives: 2,
        };
        let mut list = new_debug_draw_list(cfg);
        draw_line(&mut list, [0.0; 3], [1.0; 3], [1.0; 4]);
        draw_line(&mut list, [0.0; 3], [2.0; 3], [1.0; 4]);
        draw_line(&mut list, [0.0; 3], [3.0; 3], [1.0; 4]); // should be dropped
        assert_eq!(primitive_count(&list), 2);
        assert!(is_debug_full(&list));
    }

    #[test]
    fn json_output_contains_type() {
        let cfg = default_debug_draw_config();
        let mut list = new_debug_draw_list(cfg);
        draw_line(&mut list, [0.0; 3], [1.0; 3], [1.0; 4]);
        let json = debug_prim_draw_to_json(&list);
        assert!(json.contains("line"));
        assert!(json.contains("primitives"));
    }
}
