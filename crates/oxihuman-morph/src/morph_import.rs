// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// An imported morph target with per-vertex offsets.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ImportedMorph {
    pub name: String,
    pub offsets: Vec<[f32; 3]>,
    pub vertex_count: usize,
}

/// Parse a simple CSV string ("name,idx,x,y,z" per line) into an ImportedMorph.
/// Returns None if the CSV is empty or malformed.
#[allow(dead_code)]
pub fn parse_morph_csv(csv: &str) -> Option<ImportedMorph> {
    let mut name = String::new();
    let mut offsets: Vec<[f32; 3]> = Vec::new();
    for line in csv.lines() {
        let parts: Vec<&str> = line.splitn(5, ',').collect();
        if parts.len() < 5 {
            continue;
        }
        if name.is_empty() {
            name = parts[0].trim().to_string();
        }
        let x = parts[2].trim().parse::<f32>().ok()?;
        let y = parts[3].trim().parse::<f32>().ok()?;
        let z = parts[4].trim().parse::<f32>().ok()?;
        offsets.push([x, y, z]);
    }
    if name.is_empty() {
        return None;
    }
    let vc = offsets.len();
    Some(ImportedMorph { name, offsets, vertex_count: vc })
}

/// Build an ImportedMorph directly from an offset list.
#[allow(dead_code)]
pub fn morph_from_offsets(name: &str, offsets: Vec<[f32; 3]>) -> ImportedMorph {
    let vc = offsets.len();
    ImportedMorph { name: name.to_string(), offsets, vertex_count: vc }
}

/// Return the maximum offset magnitude across all vertices.
#[allow(dead_code)]
pub fn morph_max_offset(m: &ImportedMorph) -> f32 {
    m.offsets
        .iter()
        .map(|o| (o[0] * o[0] + o[1] * o[1] + o[2] * o[2]).sqrt())
        .fold(0.0f32, f32::max)
}

/// Return the number of vertices in the morph.
#[allow(dead_code)]
pub fn morph_vertex_count(m: &ImportedMorph) -> usize {
    m.vertex_count
}

/// Validate that the morph vertex count matches the mesh's vertex count.
#[allow(dead_code)]
pub fn validate_morph(m: &ImportedMorph, mesh_verts: usize) -> bool {
    m.vertex_count == mesh_verts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn morph_from_offsets_sets_fields() {
        let m = morph_from_offsets("test", vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
        assert_eq!(m.name, "test");
        assert_eq!(m.vertex_count, 2);
    }

    #[test]
    fn morph_vertex_count_matches() {
        let m = morph_from_offsets("m", vec![[0.0; 3]; 5]);
        assert_eq!(morph_vertex_count(&m), 5);
    }

    #[test]
    fn morph_max_offset_zero() {
        let m = morph_from_offsets("m", vec![[0.0; 3]; 3]);
        assert!((morph_max_offset(&m)).abs() < 1e-6);
    }

    #[test]
    fn morph_max_offset_nonzero() {
        let m = morph_from_offsets("m", vec![[1.0, 0.0, 0.0], [0.0, 2.0, 0.0]]);
        assert!((morph_max_offset(&m) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn validate_morph_correct() {
        let m = morph_from_offsets("m", vec![[0.0; 3]; 4]);
        assert!(validate_morph(&m, 4));
    }

    #[test]
    fn validate_morph_wrong_count() {
        let m = morph_from_offsets("m", vec![[0.0; 3]; 4]);
        assert!(!validate_morph(&m, 3));
    }

    #[test]
    fn parse_morph_csv_valid() {
        let csv = "smile,0,0.1,0.2,0.3\nsmile,1,0.4,0.5,0.6";
        let m = parse_morph_csv(csv).expect("should succeed");
        assert_eq!(m.name, "smile");
        assert_eq!(m.vertex_count, 2);
    }

    #[test]
    fn parse_morph_csv_empty_returns_none() {
        assert!(parse_morph_csv("").is_none());
    }

    #[test]
    fn parse_morph_csv_bad_float_returns_none() {
        let csv = "m,0,abc,0.0,0.0";
        assert!(parse_morph_csv(csv).is_none());
    }

    #[test]
    fn morph_from_offsets_empty() {
        let m = morph_from_offsets("empty", vec![]);
        assert_eq!(m.vertex_count, 0);
    }
}
