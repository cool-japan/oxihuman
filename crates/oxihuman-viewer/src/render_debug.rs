// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Debug rendering primitives (wireframe, normals, bounding boxes, etc.).

#[allow(dead_code)]
pub struct DebugLine {
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub color: [f32; 4],
    pub width: f32,
}

#[allow(dead_code)]
pub struct DebugPoint {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub size: f32,
}

#[allow(dead_code)]
pub struct DebugText {
    pub position: [f32; 3],
    pub text: String,
    pub color: [f32; 4],
    pub scale: f32,
}

#[allow(dead_code)]
pub struct DebugDraw {
    pub lines: Vec<DebugLine>,
    pub points: Vec<DebugPoint>,
    pub texts: Vec<DebugText>,
    pub enabled: bool,
}

#[allow(dead_code)]
pub fn new_debug_draw() -> DebugDraw {
    DebugDraw {
        lines: Vec::new(),
        points: Vec::new(),
        texts: Vec::new(),
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn draw_line(dd: &mut DebugDraw, start: [f32; 3], end: [f32; 3], color: [f32; 4]) {
    if !dd.enabled {
        return;
    }
    dd.lines.push(DebugLine {
        start,
        end,
        color,
        width: 1.0,
    });
}

#[allow(dead_code)]
pub fn draw_point(dd: &mut DebugDraw, pos: [f32; 3], color: [f32; 4], size: f32) {
    if !dd.enabled {
        return;
    }
    dd.points.push(DebugPoint {
        position: pos,
        color,
        size,
    });
}

#[allow(dead_code)]
pub fn draw_text(dd: &mut DebugDraw, pos: [f32; 3], text: &str, color: [f32; 4]) {
    if !dd.enabled {
        return;
    }
    dd.texts.push(DebugText {
        position: pos,
        text: text.to_string(),
        color,
        scale: 1.0,
    });
}

/// Draw 12 edges of an axis-aligned bounding box.
#[allow(dead_code)]
pub fn draw_aabb(dd: &mut DebugDraw, min: [f32; 3], max: [f32; 3], color: [f32; 4]) {
    if !dd.enabled {
        return;
    }
    let [x0, y0, z0] = min;
    let [x1, y1, z1] = max;

    // 8 corners
    let corners = [
        [x0, y0, z0],
        [x1, y0, z0],
        [x1, y1, z0],
        [x0, y1, z0], // front face
        [x0, y0, z1],
        [x1, y0, z1],
        [x1, y1, z1],
        [x0, y1, z1], // back face
    ];

    // 12 edges
    let edges: [(usize, usize); 12] = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0), // front
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4), // back
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7), // sides
    ];

    for (a, b) in &edges {
        draw_line(dd, corners[*a], corners[*b], color);
    }
}

/// Draw 3 great circles of a sphere (one per axis), each with `segments` segments.
#[allow(dead_code)]
pub fn draw_sphere_wireframe(
    dd: &mut DebugDraw,
    center: [f32; 3],
    radius: f32,
    color: [f32; 4],
    segments: u32,
) {
    if !dd.enabled {
        return;
    }
    let n = segments as usize;
    for circle in 0..3usize {
        for i in 0..n {
            let t0 = i as f32 / n as f32 * std::f32::consts::TAU;
            let t1 = (i + 1) as f32 / n as f32 * std::f32::consts::TAU;

            let (s0, c0) = t0.sin_cos();
            let (s1, c1) = t1.sin_cos();

            let (p0, p1) = match circle {
                0 => (
                    [center[0] + radius * c0, center[1] + radius * s0, center[2]],
                    [center[0] + radius * c1, center[1] + radius * s1, center[2]],
                ),
                1 => (
                    [center[0] + radius * c0, center[1], center[2] + radius * s0],
                    [center[0] + radius * c1, center[1], center[2] + radius * s1],
                ),
                _ => (
                    [center[0], center[1] + radius * c0, center[2] + radius * s0],
                    [center[0], center[1] + radius * c1, center[2] + radius * s1],
                ),
            };

            draw_line(dd, p0, p1, color);
        }
    }
}

#[allow(dead_code)]
pub fn draw_normal_vectors(
    dd: &mut DebugDraw,
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    scale: f32,
    color: [f32; 4],
) {
    if !dd.enabled {
        return;
    }
    for (pos, nor) in positions.iter().zip(normals.iter()) {
        let end = [
            pos[0] + nor[0] * scale,
            pos[1] + nor[1] * scale,
            pos[2] + nor[2] * scale,
        ];
        draw_line(dd, *pos, end, color);
    }
}

#[allow(dead_code)]
pub fn draw_skeleton(
    dd: &mut DebugDraw,
    positions: &[[f32; 3]],
    parent_indices: &[Option<usize>],
    color: [f32; 4],
) {
    if !dd.enabled {
        return;
    }
    for (i, parent_opt) in parent_indices.iter().enumerate() {
        if let Some(parent) = parent_opt {
            if i < positions.len() && *parent < positions.len() {
                draw_line(dd, positions[i], positions[*parent], color);
            }
        }
    }
}

/// Draw triangle edges from an indexed mesh.
#[allow(dead_code)]
pub fn draw_wireframe(
    dd: &mut DebugDraw,
    positions: &[[f32; 3]],
    indices: &[u32],
    color: [f32; 4],
) {
    if !dd.enabled {
        return;
    }
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        if i0 < positions.len() && i1 < positions.len() && i2 < positions.len() {
            draw_line(dd, positions[i0], positions[i1], color);
            draw_line(dd, positions[i1], positions[i2], color);
            draw_line(dd, positions[i2], positions[i0], color);
        }
    }
}

#[allow(dead_code)]
pub fn clear_debug_draw(dd: &mut DebugDraw) {
    dd.lines.clear();
    dd.points.clear();
    dd.texts.clear();
}

#[allow(dead_code)]
pub fn line_count(dd: &DebugDraw) -> usize {
    dd.lines.len()
}

#[allow(dead_code)]
pub fn total_primitive_count(dd: &DebugDraw) -> usize {
    dd.lines.len() + dd.points.len() + dd.texts.len()
}

#[allow(dead_code)]
pub fn set_enabled(dd: &mut DebugDraw, enabled: bool) {
    dd.enabled = enabled;
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

    #[test]
    fn test_draw_line_adds_one() {
        let mut dd = new_debug_draw();
        draw_line(&mut dd, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], WHITE);
        assert_eq!(line_count(&dd), 1);
    }

    #[test]
    fn test_draw_aabb_adds_12_lines() {
        let mut dd = new_debug_draw();
        draw_aabb(&mut dd, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0], WHITE);
        assert_eq!(line_count(&dd), 12);
    }

    #[test]
    fn test_draw_sphere_wireframe_segments() {
        let segments = 16u32;
        let mut dd = new_debug_draw();
        draw_sphere_wireframe(&mut dd, [0.0, 0.0, 0.0], 1.0, WHITE, segments);
        assert_eq!(line_count(&dd), (segments * 3) as usize);
    }

    #[test]
    fn test_draw_normal_vectors_adds_n_lines() {
        let mut dd = new_debug_draw();
        let positions: Vec<[f32; 3]> = (0..5).map(|i| [i as f32, 0.0, 0.0]).collect();
        let normals: Vec<[f32; 3]> = vec![[0.0, 1.0, 0.0]; 5];
        draw_normal_vectors(&mut dd, &positions, &normals, 0.1, WHITE);
        assert_eq!(line_count(&dd), 5);
    }

    #[test]
    fn test_draw_wireframe_adds_triangle_count_times_3() {
        let mut dd = new_debug_draw();
        let positions: Vec<[f32; 3]> = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [1.5, 0.0, 0.0],
            [2.0, 1.0, 0.0],
        ];
        let indices: Vec<u32> = vec![0, 1, 2, 1, 3, 4];
        draw_wireframe(&mut dd, &positions, &indices, WHITE);
        assert_eq!(line_count(&dd), 6); // 2 triangles * 3
    }

    #[test]
    fn test_clear_resets_counts() {
        let mut dd = new_debug_draw();
        draw_line(&mut dd, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], WHITE);
        draw_point(&mut dd, [0.0, 0.0, 0.0], WHITE, 1.0);
        draw_text(&mut dd, [0.0, 0.0, 0.0], "hi", WHITE);
        clear_debug_draw(&mut dd);
        assert_eq!(total_primitive_count(&dd), 0);
    }

    #[test]
    fn test_total_primitive_count() {
        let mut dd = new_debug_draw();
        draw_line(&mut dd, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], WHITE);
        draw_point(&mut dd, [0.0, 0.0, 0.0], WHITE, 2.0);
        draw_text(&mut dd, [0.0, 0.0, 0.0], "test", WHITE);
        assert_eq!(total_primitive_count(&dd), 3);
    }

    #[test]
    fn test_draw_point_adds_one() {
        let mut dd = new_debug_draw();
        draw_point(&mut dd, [0.0, 0.0, 0.0], WHITE, 1.0);
        assert_eq!(dd.points.len(), 1);
    }

    #[test]
    fn test_draw_text_adds_one() {
        let mut dd = new_debug_draw();
        draw_text(&mut dd, [0.0, 0.0, 0.0], "hello", WHITE);
        assert_eq!(dd.texts.len(), 1);
    }

    #[test]
    fn test_set_enabled_false_no_lines() {
        let mut dd = new_debug_draw();
        set_enabled(&mut dd, false);
        draw_line(&mut dd, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], WHITE);
        assert_eq!(line_count(&dd), 0);
    }

    #[test]
    fn test_draw_skeleton_adds_lines() {
        let mut dd = new_debug_draw();
        let positions: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 2.0, 0.0]];
        let parents: Vec<Option<usize>> = vec![None, Some(0), Some(1)];
        draw_skeleton(&mut dd, &positions, &parents, WHITE);
        assert_eq!(line_count(&dd), 2);
    }

    #[test]
    fn test_new_debug_draw_empty() {
        let dd = new_debug_draw();
        assert_eq!(total_primitive_count(&dd), 0);
        assert!(dd.enabled);
    }

    #[test]
    fn test_line_count_direct() {
        let mut dd = new_debug_draw();
        for _ in 0..7 {
            draw_line(&mut dd, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], WHITE);
        }
        assert_eq!(line_count(&dd), 7);
    }
}
