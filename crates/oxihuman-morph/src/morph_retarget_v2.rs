#![allow(dead_code)]

//! Morph retargeting between different mesh topologies.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexMapping {
    pub source_index: u32,
    pub target_index: u32,
    pub weight: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphRetargetV2 {
    pub source_vertex_count: u32,
    pub target_vertex_count: u32,
    pub mappings: Vec<VertexMapping>,
    pub tolerance: f32,
}

#[allow(dead_code)]
pub fn new_morph_retarget_v2(source_count: u32, target_count: u32) -> MorphRetargetV2 {
    MorphRetargetV2 {
        source_vertex_count: source_count,
        target_vertex_count: target_count,
        mappings: Vec::new(),
        tolerance: 1e-3,
    }
}

#[allow(dead_code)]
pub fn mrv2_add_mapping(retarget: &mut MorphRetargetV2, source: u32, target: u32, weight: f32) {
    retarget.mappings.push(VertexMapping {
        source_index: source,
        target_index: target,
        weight: weight.clamp(0.0, 1.0),
    });
}

#[allow(dead_code)]
pub fn mrv2_retarget(
    retarget: &MorphRetargetV2,
    source_deltas: &[[f32; 3]],
    target_deltas: &mut [[f32; 3]],
) {
    for d in target_deltas.iter_mut() {
        *d = [0.0; 3];
    }
    for mapping in &retarget.mappings {
        let si = mapping.source_index as usize;
        let ti = mapping.target_index as usize;
        if si < source_deltas.len() && ti < target_deltas.len() {
            for k in 0..3 {
                target_deltas[ti][k] += source_deltas[si][k] * mapping.weight;
            }
        }
    }
}

#[allow(dead_code)]
pub fn mrv2_mapping_count(retarget: &MorphRetargetV2) -> usize {
    retarget.mappings.len()
}

#[allow(dead_code)]
pub fn mrv2_clear_mappings(retarget: &mut MorphRetargetV2) {
    retarget.mappings.clear();
}

#[allow(dead_code)]
pub fn mrv2_coverage(retarget: &MorphRetargetV2) -> f32 {
    let mapped: std::collections::HashSet<u32> =
        retarget.mappings.iter().map(|m| m.target_index).collect();
    if retarget.target_vertex_count == 0 {
        return 0.0;
    }
    mapped.len() as f32 / retarget.target_vertex_count as f32
}

#[allow(dead_code)]
pub fn mrv2_to_json(retarget: &MorphRetargetV2) -> String {
    format!(
        "{{\"source_count\":{},\"target_count\":{},\"mapping_count\":{}}}",
        retarget.source_vertex_count,
        retarget.target_vertex_count,
        retarget.mappings.len()
    )
}

#[allow(dead_code)]
pub fn mrv2_set_tolerance(retarget: &mut MorphRetargetV2, tolerance: f32) {
    retarget.tolerance = tolerance.max(0.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_retarget() {
        let r = new_morph_retarget_v2(100, 80);
        assert_eq!(r.source_vertex_count, 100);
        assert_eq!(mrv2_mapping_count(&r), 0);
    }

    #[test]
    fn test_add_mapping() {
        let mut r = new_morph_retarget_v2(10, 10);
        mrv2_add_mapping(&mut r, 0, 0, 1.0);
        assert_eq!(mrv2_mapping_count(&r), 1);
    }

    #[test]
    fn test_retarget_applies_delta() {
        let mut r = new_morph_retarget_v2(1, 1);
        mrv2_add_mapping(&mut r, 0, 0, 1.0);
        let src = [[1.0_f32, 0.0, 0.0]];
        let mut dst = [[0.0_f32; 3]; 1];
        mrv2_retarget(&r, &src, &mut dst);
        assert!((dst[0][0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_retarget_weighted() {
        let mut r = new_morph_retarget_v2(1, 1);
        mrv2_add_mapping(&mut r, 0, 0, 0.5);
        let src = [[2.0_f32, 0.0, 0.0]];
        let mut dst = [[0.0_f32; 3]; 1];
        mrv2_retarget(&r, &src, &mut dst);
        assert!((dst[0][0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_clear_mappings() {
        let mut r = new_morph_retarget_v2(10, 10);
        mrv2_add_mapping(&mut r, 0, 0, 1.0);
        mrv2_clear_mappings(&mut r);
        assert_eq!(mrv2_mapping_count(&r), 0);
    }

    #[test]
    fn test_coverage_zero_on_empty() {
        let r = new_morph_retarget_v2(10, 10);
        assert!((mrv2_coverage(&r) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_coverage_full() {
        let mut r = new_morph_retarget_v2(2, 2);
        mrv2_add_mapping(&mut r, 0, 0, 1.0);
        mrv2_add_mapping(&mut r, 1, 1, 1.0);
        assert!((mrv2_coverage(&r) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_tolerance() {
        let mut r = new_morph_retarget_v2(10, 10);
        mrv2_set_tolerance(&mut r, 0.01);
        assert!((r.tolerance - 0.01).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let r = new_morph_retarget_v2(100, 80);
        let json = mrv2_to_json(&r);
        assert!(json.contains("source_count"));
    }

    #[test]
    fn test_out_of_bounds_safe() {
        let mut r = new_morph_retarget_v2(1, 1);
        mrv2_add_mapping(&mut r, 99, 99, 1.0);
        let src = [[1.0_f32; 3]; 1];
        let mut dst = [[0.0_f32; 3]; 1];
        mrv2_retarget(&r, &src, &mut dst);
        assert!((dst[0][0] - 0.0).abs() < 1e-6);
    }
}
