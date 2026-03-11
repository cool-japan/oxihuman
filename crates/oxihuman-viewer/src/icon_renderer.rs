// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Icon/glyph rendering data for UI.

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum IconType {
    Arrow,
    Circle,
    Cross,
    Check,
    Star,
    Triangle,
    Pencil,
    Trash,
    Eye,
    Lock,
    Folder,
    File,
}

#[allow(dead_code)]
pub struct IconGlyph {
    pub icon_type: IconType,
    pub vertices: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub filled: bool,
}

#[allow(dead_code)]
pub struct IconAtlas {
    pub icons: Vec<(IconType, IconGlyph)>,
    pub cell_size: f32,
}

#[allow(dead_code)]
pub struct RenderedIcon {
    pub icon_type: IconType,
    pub position: [f32; 2],
    pub size: f32,
    pub color: [f32; 4],
    pub rotation: f32,
}

#[allow(dead_code)]
pub fn icon_glyph_circle(radius: f32, segments: u32) -> IconGlyph {
    let n = segments.max(3) as usize;
    let mut vertices = Vec::with_capacity(n);
    let mut indices = Vec::new();
    use std::f32::consts::TAU;
    for i in 0..n {
        let angle = (i as f32 / n as f32) * TAU;
        vertices.push([radius * angle.cos(), radius * angle.sin()]);
    }
    for i in 0..(n as u32 - 1) {
        indices.push(0);
        indices.push(i);
        indices.push(i + 1);
    }
    if n >= 2 {
        indices.push(0);
        indices.push(n as u32 - 1);
        indices.push(0);
    }
    IconGlyph {
        icon_type: IconType::Circle,
        vertices,
        indices,
        filled: true,
    }
}

#[allow(dead_code)]
pub fn icon_glyph_cross(size: f32) -> IconGlyph {
    let h = size * 0.5;
    let t = size * 0.15;
    let vertices = vec![
        [-t, -h],
        [t, -h],
        [t, -t],
        [h, -t],
        [h, t],
        [t, t],
        [t, h],
        [-t, h],
        [-t, t],
        [-h, t],
        [-h, -t],
        [-t, -t],
    ];
    let mut indices = Vec::new();
    let n = vertices.len() as u32;
    for i in 1..(n - 1) {
        indices.push(0);
        indices.push(i);
        indices.push(i + 1);
    }
    IconGlyph {
        icon_type: IconType::Cross,
        vertices,
        indices,
        filled: true,
    }
}

#[allow(dead_code)]
pub fn icon_glyph_arrow(size: f32) -> IconGlyph {
    let h = size * 0.5;
    let head = size * 0.35;
    let shaft = size * 0.15;
    let vertices = vec![
        [0.0, h],
        [head, 0.0],
        [shaft, 0.0],
        [shaft, -h],
        [-shaft, -h],
        [-shaft, 0.0],
        [-head, 0.0],
    ];
    let mut indices = Vec::new();
    let n = vertices.len() as u32;
    for i in 1..(n - 1) {
        indices.push(0);
        indices.push(i);
        indices.push(i + 1);
    }
    IconGlyph {
        icon_type: IconType::Arrow,
        vertices,
        indices,
        filled: true,
    }
}

#[allow(dead_code)]
pub fn icon_glyph_check(size: f32) -> IconGlyph {
    let s = size * 0.5;
    let vertices = vec![
        [-s, 0.0],
        [-s * 0.2, -s * 0.5],
        [s, s * 0.6],
        [s * 0.7, s * 0.9],
        [-s * 0.2, s * 0.1],
        [-s * 0.6, -s * 0.2],
    ];
    let indices = vec![0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 5];
    IconGlyph {
        icon_type: IconType::Check,
        vertices,
        indices,
        filled: true,
    }
}

fn make_generic_glyph(icon_type: IconType, size: f32) -> IconGlyph {
    let h = size * 0.5;
    let vertices = vec![[0.0, h], [h, -h], [-h, -h]];
    let indices = vec![0, 1, 2];
    IconGlyph {
        icon_type,
        vertices,
        indices,
        filled: true,
    }
}

#[allow(dead_code)]
pub fn build_icon_atlas(cell_size: f32) -> IconAtlas {
    let types = all_icon_types();
    let mut icons = Vec::with_capacity(types.len());
    for &t in &types {
        let glyph = match t {
            IconType::Circle => icon_glyph_circle(cell_size * 0.4, 16),
            IconType::Cross => icon_glyph_cross(cell_size * 0.8),
            IconType::Arrow => icon_glyph_arrow(cell_size * 0.8),
            IconType::Check => icon_glyph_check(cell_size * 0.8),
            _ => make_generic_glyph(t, cell_size * 0.8),
        };
        icons.push((t, glyph));
    }
    IconAtlas { icons, cell_size }
}

#[allow(dead_code)]
pub fn get_icon_glyph(atlas: &IconAtlas, icon_type: IconType) -> Option<&IconGlyph> {
    atlas
        .icons
        .iter()
        .find(|(t, _)| *t == icon_type)
        .map(|(_, g)| g)
}

#[allow(dead_code)]
pub fn render_icon(icon: &IconGlyph, pos: [f32; 2], size: f32, color: [f32; 4]) -> RenderedIcon {
    RenderedIcon {
        icon_type: icon.icon_type,
        position: pos,
        size,
        color,
        rotation: 0.0,
    }
}

#[allow(dead_code)]
pub fn transform_icon(
    glyph: &IconGlyph,
    scale: f32,
    rotation: f32,
    translate: [f32; 2],
) -> IconGlyph {
    let cos_r = rotation.cos();
    let sin_r = rotation.sin();
    let vertices = glyph
        .vertices
        .iter()
        .map(|&[x, y]| {
            let sx = x * scale;
            let sy = y * scale;
            let rx = sx * cos_r - sy * sin_r + translate[0];
            let ry = sx * sin_r + sy * cos_r + translate[1];
            [rx, ry]
        })
        .collect();
    IconGlyph {
        icon_type: glyph.icon_type,
        vertices,
        indices: glyph.indices.clone(),
        filled: glyph.filled,
    }
}

#[allow(dead_code)]
pub fn icon_bounds(glyph: &IconGlyph) -> ([f32; 2], [f32; 2]) {
    if glyph.vertices.is_empty() {
        return ([0.0, 0.0], [0.0, 0.0]);
    }
    let mut min = glyph.vertices[0];
    let mut max = glyph.vertices[0];
    for &v in &glyph.vertices {
        if v[0] < min[0] {
            min[0] = v[0];
        }
        if v[1] < min[1] {
            min[1] = v[1];
        }
        if v[0] > max[0] {
            max[0] = v[0];
        }
        if v[1] > max[1] {
            max[1] = v[1];
        }
    }
    (min, max)
}

#[allow(dead_code)]
pub fn icon_vertex_count(glyph: &IconGlyph) -> usize {
    glyph.vertices.len()
}

#[allow(dead_code)]
pub fn icon_index_count(glyph: &IconGlyph) -> usize {
    glyph.indices.len()
}

#[allow(dead_code)]
pub fn all_icon_types() -> Vec<IconType> {
    vec![
        IconType::Arrow,
        IconType::Circle,
        IconType::Cross,
        IconType::Check,
        IconType::Star,
        IconType::Triangle,
        IconType::Pencil,
        IconType::Trash,
        IconType::Eye,
        IconType::Lock,
        IconType::Folder,
        IconType::File,
    ]
}

#[allow(dead_code)]
pub fn icon_type_name(icon: IconType) -> &'static str {
    match icon {
        IconType::Arrow => "arrow",
        IconType::Circle => "circle",
        IconType::Cross => "cross",
        IconType::Check => "check",
        IconType::Star => "star",
        IconType::Triangle => "triangle",
        IconType::Pencil => "pencil",
        IconType::Trash => "trash",
        IconType::Eye => "eye",
        IconType::Lock => "lock",
        IconType::Folder => "folder",
        IconType::File => "file",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_atlas_has_glyphs() {
        let atlas = build_icon_atlas(32.0);
        assert!(!atlas.icons.is_empty());
        assert_eq!(atlas.icons.len(), all_icon_types().len());
    }

    #[test]
    fn test_get_icon_glyph() {
        let atlas = build_icon_atlas(32.0);
        let glyph = get_icon_glyph(&atlas, IconType::Circle);
        assert!(glyph.is_some());
        assert_eq!(glyph.unwrap().icon_type, IconType::Circle);
    }

    #[test]
    fn test_get_icon_glyph_missing() {
        let empty = IconAtlas {
            icons: vec![],
            cell_size: 32.0,
        };
        assert!(get_icon_glyph(&empty, IconType::Arrow).is_none());
    }

    #[test]
    fn test_icon_glyph_circle_vertex_count() {
        let glyph = icon_glyph_circle(1.0, 16);
        assert_eq!(icon_vertex_count(&glyph), 16);
    }

    #[test]
    fn test_icon_glyph_cross_has_vertices() {
        let glyph = icon_glyph_cross(1.0);
        assert!(!glyph.vertices.is_empty());
        assert!(!glyph.indices.is_empty());
    }

    #[test]
    fn test_icon_glyph_arrow_has_vertices() {
        let glyph = icon_glyph_arrow(1.0);
        assert!(!glyph.vertices.is_empty());
    }

    #[test]
    fn test_icon_glyph_check_has_vertices() {
        let glyph = icon_glyph_check(1.0);
        assert!(!glyph.vertices.is_empty());
        assert!(!glyph.indices.is_empty());
    }

    #[test]
    fn test_render_icon() {
        let glyph = icon_glyph_circle(1.0, 8);
        let rendered = render_icon(&glyph, [10.0, 20.0], 32.0, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(rendered.position, [10.0, 20.0]);
        assert_eq!(rendered.size, 32.0);
        assert_eq!(rendered.icon_type, IconType::Circle);
    }

    #[test]
    fn test_transform_icon_changes_bounds() {
        let glyph = icon_glyph_circle(1.0, 8);
        let (min0, max0) = icon_bounds(&glyph);
        let transformed = transform_icon(&glyph, 2.0, 0.0, [5.0, 5.0]);
        let (min1, max1) = icon_bounds(&transformed);
        assert!(max1[0] > max0[0]);
        assert!(min1[0] != min0[0]); // translated
        let _ = (min1, max1);
    }

    #[test]
    fn test_icon_bounds() {
        let glyph = icon_glyph_circle(1.0, 8);
        let (min, max) = icon_bounds(&glyph);
        assert!(min[0] <= max[0]);
        assert!(min[1] <= max[1]);
    }

    #[test]
    fn test_all_icon_types_count() {
        let types = all_icon_types();
        assert_eq!(types.len(), 12);
    }

    #[test]
    fn test_icon_type_name_non_empty() {
        for t in all_icon_types() {
            assert!(!icon_type_name(t).is_empty());
        }
    }

    #[test]
    fn test_icon_index_count() {
        let glyph = icon_glyph_cross(1.0);
        assert!(icon_index_count(&glyph) > 0);
    }
}
