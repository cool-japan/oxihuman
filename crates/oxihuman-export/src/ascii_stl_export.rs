// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! ASCII STL format export.

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-10 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

#[derive(Debug, Clone)]
pub struct AsciiStlOptions {
    pub solid_name: String,
    pub precision: usize,
}

impl Default for AsciiStlOptions {
    fn default() -> Self {
        AsciiStlOptions {
            solid_name: "mesh".to_string(),
            precision: 6,
        }
    }
}

pub fn render_ascii_stl(positions: &[[f32; 3]], indices: &[u32], opts: &AsciiStlOptions) -> String {
    let mut s = format!("solid {}\n", opts.solid_name);
    for tri in indices.chunks(3) {
        if tri.len() < 3 {
            continue;
        }
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if a >= positions.len() || b >= positions.len() || c >= positions.len() {
            continue;
        }
        let v0 = positions[a];
        let v1 = positions[b];
        let v2 = positions[c];
        let ab = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let ac = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let n = normalize3([
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ]);
        let p = opts.precision;
        s.push_str(&format!(
            "  facet normal {:.p$} {:.p$} {:.p$}\n",
            n[0],
            n[1],
            n[2],
            p = p
        ));
        s.push_str("    outer loop\n");
        s.push_str(&format!(
            "      vertex {:.p$} {:.p$} {:.p$}\n",
            v0[0],
            v0[1],
            v0[2],
            p = p
        ));
        s.push_str(&format!(
            "      vertex {:.p$} {:.p$} {:.p$}\n",
            v1[0],
            v1[1],
            v1[2],
            p = p
        ));
        s.push_str(&format!(
            "      vertex {:.p$} {:.p$} {:.p$}\n",
            v2[0],
            v2[1],
            v2[2],
            p = p
        ));
        s.push_str("    endloop\n  endfacet\n");
    }
    s.push_str(&format!("endsolid {}\n", opts.solid_name));
    s
}

pub fn export_ascii_stl(
    positions: &[[f32; 3]],
    indices: &[u32],
    opts: &AsciiStlOptions,
) -> Vec<u8> {
    render_ascii_stl(positions, indices, opts).into_bytes()
}

pub fn count_ascii_stl_triangles(stl_text: &str) -> usize {
    stl_text
        .lines()
        .filter(|l| l.trim().starts_with("facet normal"))
        .count()
}

pub fn validate_ascii_stl(stl_text: &str) -> bool {
    stl_text.trim_start().starts_with("solid") && stl_text.trim_end().ends_with("endsolid mesh")
        || stl_text.contains("endsolid")
}

pub fn default_ascii_stl_options() -> AsciiStlOptions {
    AsciiStlOptions::default()
}

pub fn ascii_stl_size_bytes(positions: &[[f32; 3]], indices: &[u32]) -> usize {
    render_ascii_stl(positions, indices, &AsciiStlOptions::default()).len()
}

pub fn parse_ascii_stl_vertices(stl_text: &str) -> Vec<[f32; 3]> {
    let mut verts = Vec::new();
    for line in stl_text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("vertex ") {
            let parts: Vec<f32> = rest
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            if parts.len() == 3 {
                verts.push([parts[0], parts[1], parts[2]]);
            }
        }
    }
    verts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_tri() -> (Vec<[f32; 3]>, Vec<u32>) {
        (
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![0u32, 1, 2],
        )
    }

    #[test]
    fn test_render_ascii_stl_solid() {
        let (pos, idx) = simple_tri();
        let s = render_ascii_stl(&pos, &idx, &default_ascii_stl_options());
        assert!(s.starts_with("solid"));
    }

    #[test]
    fn test_count_ascii_stl_triangles() {
        let (pos, idx) = simple_tri();
        let s = render_ascii_stl(&pos, &idx, &default_ascii_stl_options());
        assert_eq!(count_ascii_stl_triangles(&s), 1);
    }

    #[test]
    fn test_export_ascii_stl_nonempty() {
        let (pos, idx) = simple_tri();
        let bytes = export_ascii_stl(&pos, &idx, &default_ascii_stl_options());
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_validate_ascii_stl() {
        let (pos, idx) = simple_tri();
        let s = render_ascii_stl(&pos, &idx, &default_ascii_stl_options());
        assert!(validate_ascii_stl(&s));
    }

    #[test]
    fn test_parse_ascii_stl_vertices() {
        let (pos, idx) = simple_tri();
        let s = render_ascii_stl(&pos, &idx, &default_ascii_stl_options());
        let verts = parse_ascii_stl_vertices(&s);
        assert_eq!(verts.len(), 3);
    }

    #[test]
    fn test_ascii_stl_size_bytes() {
        let (pos, idx) = simple_tri();
        assert!(ascii_stl_size_bytes(&pos, &idx) > 0);
    }

    #[test]
    fn test_render_endsolid() {
        let (pos, idx) = simple_tri();
        let s = render_ascii_stl(&pos, &idx, &default_ascii_stl_options());
        assert!(s.contains("endsolid"));
    }

    #[test]
    fn test_custom_solid_name() {
        let opts = AsciiStlOptions {
            solid_name: "custom".to_string(),
            precision: 4,
        };
        let (pos, idx) = simple_tri();
        let s = render_ascii_stl(&pos, &idx, &opts);
        assert!(s.contains("custom"));
    }
}
