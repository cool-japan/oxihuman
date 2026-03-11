#![allow(dead_code)]
//! Vertex morph target export.

/// Vertex morph export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct VertexMorphExport {
    pub targets: Vec<MorphTarget>,
}

/// A morph target.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct MorphTarget {
    pub position_deltas: Vec<[f32; 3]>,
    pub normal_deltas: Vec<[f32; 3]>,
}

/// Export vertex morphs.
#[allow(dead_code)]
pub fn export_vertex_morphs(targets: Vec<MorphTarget>) -> VertexMorphExport {
    VertexMorphExport { targets }
}

/// Get morph target count.
#[allow(dead_code)]
pub fn morph_target_count_vme(e: &VertexMorphExport) -> usize {
    e.targets.len()
}

/// Get position deltas for a target.
#[allow(dead_code)]
pub fn morph_position_deltas(e: &VertexMorphExport, index: usize) -> &[[f32; 3]] {
    if index < e.targets.len() {
        &e.targets[index].position_deltas
    } else {
        &[]
    }
}

/// Get normal deltas for a target.
#[allow(dead_code)]
pub fn morph_normal_deltas(e: &VertexMorphExport, index: usize) -> &[[f32; 3]] {
    if index < e.targets.len() {
        &e.targets[index].normal_deltas
    } else {
        &[]
    }
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn morph_to_json(e: &VertexMorphExport) -> String {
    format!("{{\"target_count\":{}}}", e.targets.len())
}

/// Serialize to bytes.
#[allow(dead_code)]
pub fn morph_to_bytes(e: &VertexMorphExport) -> Vec<u8> {
    let mut bytes = Vec::new();
    for t in &e.targets {
        for d in &t.position_deltas {
            for v in d {
                bytes.extend_from_slice(&v.to_le_bytes());
            }
        }
    }
    bytes
}

/// Get export size estimate.
#[allow(dead_code)]
pub fn morph_export_size(e: &VertexMorphExport) -> usize {
    e.targets
        .iter()
        .map(|t| (t.position_deltas.len() + t.normal_deltas.len()) * 12)
        .sum()
}

/// Validate vertex morphs.
#[allow(dead_code)]
pub fn validate_vertex_morphs(e: &VertexMorphExport) -> bool {
    e.targets
        .iter()
        .all(|t| t.position_deltas.len() == t.normal_deltas.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_target(n: usize) -> MorphTarget {
        MorphTarget {
            position_deltas: vec![[0.1, 0.0, 0.0]; n],
            normal_deltas: vec![[0.0, 0.0, 0.0]; n],
        }
    }

    #[test]
    fn test_export_vertex_morphs() {
        let e = export_vertex_morphs(vec![make_target(3)]);
        assert_eq!(e.targets.len(), 1);
    }

    #[test]
    fn test_morph_target_count() {
        let e = export_vertex_morphs(vec![make_target(3), make_target(3)]);
        assert_eq!(morph_target_count_vme(&e), 2);
    }

    #[test]
    fn test_morph_position_deltas() {
        let e = export_vertex_morphs(vec![make_target(2)]);
        assert_eq!(morph_position_deltas(&e, 0).len(), 2);
        assert!(morph_position_deltas(&e, 5).is_empty());
    }

    #[test]
    fn test_morph_normal_deltas() {
        let e = export_vertex_morphs(vec![make_target(3)]);
        assert_eq!(morph_normal_deltas(&e, 0).len(), 3);
    }

    #[test]
    fn test_morph_to_json() {
        let e = export_vertex_morphs(vec![make_target(1)]);
        let j = morph_to_json(&e);
        assert!(j.contains("target_count"));
    }

    #[test]
    fn test_morph_to_bytes() {
        let e = export_vertex_morphs(vec![make_target(1)]);
        let b = morph_to_bytes(&e);
        assert_eq!(b.len(), 12); // 1 vertex * 3 floats * 4 bytes
    }

    #[test]
    fn test_morph_export_size() {
        let e = export_vertex_morphs(vec![make_target(2)]);
        assert_eq!(morph_export_size(&e), 2 * 12 + 2 * 12);
    }

    #[test]
    fn test_validate_ok() {
        let e = export_vertex_morphs(vec![make_target(3)]);
        assert!(validate_vertex_morphs(&e));
    }

    #[test]
    fn test_validate_mismatch() {
        let e = export_vertex_morphs(vec![MorphTarget {
            position_deltas: vec![[0.0; 3]; 3],
            normal_deltas: vec![[0.0; 3]; 2],
        }]);
        assert!(!validate_vertex_morphs(&e));
    }

    #[test]
    fn test_morph_empty() {
        let e = export_vertex_morphs(vec![]);
        assert_eq!(morph_target_count_vme(&e), 0);
        assert!(validate_vertex_morphs(&e));
    }
}
