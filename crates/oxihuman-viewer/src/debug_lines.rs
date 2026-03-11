// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Debug line renderer (for physics/skeleton visualization).

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DebugLine {
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub color: [f32; 4],
    pub lifetime: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct DebugLineRenderer {
    lines: Vec<DebugLine>,
}

#[allow(dead_code)]
pub fn new_debug_line_renderer() -> DebugLineRenderer {
    DebugLineRenderer::default()
}

#[allow(dead_code)]
pub fn dl_add_line(renderer: &mut DebugLineRenderer, start: [f32; 3], end: [f32; 3], color: [f32; 4]) {
    renderer.lines.push(DebugLine { start, end, color, lifetime: f32::INFINITY });
}

#[allow(dead_code)]
pub fn dl_add_line_timed(renderer: &mut DebugLineRenderer, start: [f32; 3], end: [f32; 3], color: [f32; 4], lifetime: f32) {
    renderer.lines.push(DebugLine { start, end, color, lifetime });
}

#[allow(dead_code)]
pub fn dl_update(renderer: &mut DebugLineRenderer, dt: f32) {
    for line in &mut renderer.lines {
        if line.lifetime.is_finite() {
            line.lifetime -= dt;
        }
    }
    renderer.lines.retain(|l| l.lifetime > 0.0);
}

#[allow(dead_code)]
pub fn dl_count(renderer: &DebugLineRenderer) -> usize {
    renderer.lines.len()
}

#[allow(dead_code)]
pub fn dl_clear(renderer: &mut DebugLineRenderer) {
    renderer.lines.clear();
}

#[allow(dead_code)]
pub fn dl_get(renderer: &DebugLineRenderer, index: usize) -> Option<&DebugLine> {
    renderer.lines.get(index)
}

#[allow(dead_code)]
pub fn dl_expired_count(renderer: &DebugLineRenderer) -> usize {
    renderer.lines.iter().filter(|l| l.lifetime <= 0.0).count()
}

#[allow(dead_code)]
pub fn dl_to_json(renderer: &DebugLineRenderer) -> String {
    format!(r#"{{"line_count":{}}}"#, renderer.lines.len())
}

// ── DebugLines (new API) ──────────────────────────────────────────────────────

#[allow(dead_code)]
pub struct DebugLines {
    pub lines: Vec<DebugLine>,
}

#[allow(dead_code)]
pub fn new_debug_lines() -> DebugLines {
    DebugLines { lines: Vec::new() }
}

#[allow(dead_code)]
pub fn dl_add(lines: &mut DebugLines, start: [f32; 3], end: [f32; 3], color: [f32; 4], duration: f32) {
    lines.lines.push(DebugLine { start, end, color, lifetime: duration });
}

#[allow(dead_code)]
pub fn dl_update_lines(lines: &mut DebugLines, dt: f32) {
    for l in lines.lines.iter_mut() {
        l.lifetime -= dt;
    }
    lines.lines.retain(|l| l.lifetime > 0.0);
}

#[allow(dead_code)]
pub fn dl_line_count(lines: &DebugLines) -> usize {
    lines.lines.len()
}

#[allow(dead_code)]
pub fn dl_clear_lines(lines: &mut DebugLines) {
    lines.lines.clear();
}

#[allow(dead_code)]
pub fn dl_total_length(lines: &DebugLines) -> f32 {
    lines.lines.iter().map(|l| {
        let dx = l.end[0] - l.start[0];
        let dy = l.end[1] - l.start[1];
        let dz = l.end[2] - l.start[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_renderer_empty() {
        let r = new_debug_line_renderer();
        assert_eq!(dl_count(&r), 0);
    }

    #[test]
    fn test_add_line() {
        let mut r = new_debug_line_renderer();
        dl_add_line(&mut r, [0.0; 3], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(dl_count(&r), 1);
    }

    #[test]
    fn test_add_line_timed() {
        let mut r = new_debug_line_renderer();
        dl_add_line_timed(&mut r, [0.0; 3], [1.0; 3], [1.0; 4], 0.5);
        assert_eq!(dl_count(&r), 1);
    }

    #[test]
    fn test_update_removes_expired() {
        let mut r = new_debug_line_renderer();
        dl_add_line_timed(&mut r, [0.0; 3], [1.0; 3], [1.0; 4], 0.3);
        dl_add_line(&mut r, [0.0; 3], [1.0; 3], [0.0, 1.0, 0.0, 1.0]);
        dl_update(&mut r, 1.0);
        assert_eq!(dl_count(&r), 1);
    }

    #[test]
    fn test_clear() {
        let mut r = new_debug_line_renderer();
        dl_add_line(&mut r, [0.0; 3], [1.0; 3], [1.0; 4]);
        dl_clear(&mut r);
        assert_eq!(dl_count(&r), 0);
    }

    #[test]
    fn test_get_line() {
        let mut r = new_debug_line_renderer();
        dl_add_line(&mut r, [1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [1.0; 4]);
        let line = dl_get(&r, 0);
        assert!(line.is_some());
        assert!((line.unwrap().start[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_expired_count_initial() {
        let r = new_debug_line_renderer();
        assert_eq!(dl_expired_count(&r), 0);
    }

    #[test]
    fn test_to_json() {
        let r = new_debug_line_renderer();
        let j = dl_to_json(&r);
        assert!(j.contains("line_count"));
    }

    #[test]
    fn test_infinite_lifetime_not_removed() {
        let mut r = new_debug_line_renderer();
        dl_add_line(&mut r, [0.0; 3], [1.0; 3], [1.0; 4]);
        dl_update(&mut r, 1000.0);
        assert_eq!(dl_count(&r), 1);
    }

    /* DebugLines new API tests */
    #[test]
    fn test_dl_add() {
        let mut dl = new_debug_lines();
        dl_add(&mut dl, [0.0; 3], [1.0; 3], [1.0; 4], 1.0);
        assert_eq!(dl_line_count(&dl), 1);
    }

    #[test]
    fn test_dl_line_count() {
        let mut dl = new_debug_lines();
        dl_add(&mut dl, [0.0; 3], [1.0; 3], [1.0; 4], 1.0);
        dl_add(&mut dl, [0.0; 3], [2.0; 3], [0.0; 4], 2.0);
        assert_eq!(dl_line_count(&dl), 2);
    }

    #[test]
    fn test_dl_update_removes_expired() {
        let mut dl = new_debug_lines();
        dl_add(&mut dl, [0.0; 3], [1.0; 3], [1.0; 4], 0.5);
        dl_update_lines(&mut dl, 1.0);
        assert_eq!(dl_line_count(&dl), 0);
    }

    #[test]
    fn test_dl_update_keeps_non_expired() {
        let mut dl = new_debug_lines();
        dl_add(&mut dl, [0.0; 3], [1.0; 3], [1.0; 4], 2.0);
        dl_update_lines(&mut dl, 0.5);
        assert_eq!(dl_line_count(&dl), 1);
    }

    #[test]
    fn test_dl_clear() {
        let mut dl = new_debug_lines();
        dl_add(&mut dl, [0.0; 3], [1.0; 3], [1.0; 4], 1.0);
        dl_clear_lines(&mut dl);
        assert_eq!(dl_line_count(&dl), 0);
    }

    #[test]
    fn test_dl_total_length() {
        let mut dl = new_debug_lines();
        dl_add(&mut dl, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0; 4], 1.0);
        assert!((dl_total_length(&dl) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_dl_total_length_empty() {
        let dl = new_debug_lines();
        assert_eq!(dl_total_length(&dl), 0.0);
    }

    #[test]
    fn test_dl_partial_removal() {
        let mut dl = new_debug_lines();
        dl_add(&mut dl, [0.0; 3], [1.0; 3], [1.0; 4], 0.1);
        dl_add(&mut dl, [0.0; 3], [2.0; 3], [1.0; 4], 5.0);
        dl_update_lines(&mut dl, 1.0);
        assert_eq!(dl_line_count(&dl), 1);
    }
}
