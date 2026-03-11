#![allow(dead_code)]

//! Pipeline statistics query (vertices, primitives, fragments).

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct PipelineStats {
    pub input_assembly_vertices: u64,
    pub input_assembly_primitives: u64,
    pub vertex_shader_invocations: u64,
    pub clipping_invocations: u64,
    pub clipping_primitives: u64,
    pub fragment_shader_invocations: u64,
    pub compute_shader_invocations: u64,
    pub frame: u64,
}

#[allow(dead_code)]
pub fn new_pipeline_stats() -> PipelineStats {
    PipelineStats::default()
}

#[allow(dead_code)]
pub fn ps_update(stats: &mut PipelineStats, vertices: u64, primitives: u64, fragments: u64) {
    stats.input_assembly_vertices += vertices;
    stats.input_assembly_primitives += primitives;
    stats.fragment_shader_invocations += fragments;
    stats.vertex_shader_invocations += vertices;
    stats.clipping_invocations += primitives;
    stats.clipping_primitives += primitives;
}

#[allow(dead_code)]
pub fn ps_reset(stats: &mut PipelineStats) {
    let frame = stats.frame;
    *stats = PipelineStats::default();
    stats.frame = frame;
}

#[allow(dead_code)]
pub fn ps_advance_frame(stats: &mut PipelineStats) {
    ps_reset(stats);
    stats.frame += 1;
}

#[allow(dead_code)]
pub fn ps_primitives_per_vertex(stats: &PipelineStats) -> f32 {
    if stats.input_assembly_vertices == 0 {
        return 0.0;
    }
    stats.input_assembly_primitives as f32 / stats.input_assembly_vertices as f32
}

#[allow(dead_code)]
pub fn ps_fragment_per_primitive(stats: &PipelineStats) -> f32 {
    if stats.clipping_primitives == 0 {
        return 0.0;
    }
    stats.fragment_shader_invocations as f32 / stats.clipping_primitives as f32
}

#[allow(dead_code)]
pub fn ps_clip_ratio(stats: &PipelineStats) -> f32 {
    if stats.clipping_invocations == 0 {
        return 0.0;
    }
    stats.clipping_primitives as f32 / stats.clipping_invocations as f32
}

#[allow(dead_code)]
pub fn ps_add_compute(stats: &mut PipelineStats, invocations: u64) {
    stats.compute_shader_invocations += invocations;
}

#[allow(dead_code)]
pub fn ps_to_json(stats: &PipelineStats) -> String {
    format!(
        "{{\"frame\":{},\"vertices\":{},\"primitives\":{},\"fragments\":{}}}",
        stats.frame,
        stats.input_assembly_vertices,
        stats.input_assembly_primitives,
        stats.fragment_shader_invocations
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stats() {
        let s = new_pipeline_stats();
        assert_eq!(s.input_assembly_vertices, 0);
    }

    #[test]
    fn test_update() {
        let mut s = new_pipeline_stats();
        ps_update(&mut s, 100, 33, 5000);
        assert_eq!(s.input_assembly_vertices, 100);
    }

    #[test]
    fn test_reset() {
        let mut s = new_pipeline_stats();
        ps_update(&mut s, 100, 33, 5000);
        ps_reset(&mut s);
        assert_eq!(s.input_assembly_vertices, 0);
    }

    #[test]
    fn test_advance_frame() {
        let mut s = new_pipeline_stats();
        ps_update(&mut s, 100, 33, 5000);
        ps_advance_frame(&mut s);
        assert_eq!(s.frame, 1);
        assert_eq!(s.input_assembly_vertices, 0);
    }

    #[test]
    fn test_primitives_per_vertex() {
        let mut s = new_pipeline_stats();
        ps_update(&mut s, 9, 3, 0);
        let ratio = ps_primitives_per_vertex(&s);
        assert!((ratio - 1.0 / 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_primitives_per_vertex_zero_vertices() {
        let s = new_pipeline_stats();
        assert!((ps_primitives_per_vertex(&s) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_fragment_per_primitive() {
        let mut s = new_pipeline_stats();
        ps_update(&mut s, 3, 1, 100);
        let ratio = ps_fragment_per_primitive(&s);
        assert!(ratio > 0.0);
    }

    #[test]
    fn test_add_compute() {
        let mut s = new_pipeline_stats();
        ps_add_compute(&mut s, 256);
        assert_eq!(s.compute_shader_invocations, 256);
    }

    #[test]
    fn test_clip_ratio() {
        let mut s = new_pipeline_stats();
        ps_update(&mut s, 3, 1, 100);
        let ratio = ps_clip_ratio(&s);
        assert!((ratio - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        let s = new_pipeline_stats();
        let json = ps_to_json(&s);
        assert!(json.contains("vertices"));
    }
}
