#![allow(dead_code)]

//! Render pipeline v2 configuration (passes, targets).

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineStage {
    Geometry,
    Lighting,
    PostProcess,
    Ui,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderPassConfig {
    pub name: String,
    pub stage: PipelineStage,
    pub enabled: bool,
    pub target_index: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderPipelineV2 {
    pub name: String,
    pub passes: Vec<RenderPassConfig>,
    pub msaa_samples: u32,
    pub hdr_enabled: bool,
}

#[allow(dead_code)]
pub fn new_render_pipeline_v2(name: &str) -> RenderPipelineV2 {
    RenderPipelineV2 {
        name: name.to_string(),
        passes: Vec::new(),
        msaa_samples: 1,
        hdr_enabled: false,
    }
}

#[allow(dead_code)]
pub fn rpv2_add_pass(pipeline: &mut RenderPipelineV2, name: &str, stage: PipelineStage, target: u32) {
    pipeline.passes.push(RenderPassConfig {
        name: name.to_string(),
        stage,
        enabled: true,
        target_index: target,
    });
}

#[allow(dead_code)]
pub fn rpv2_enable_pass(pipeline: &mut RenderPipelineV2, name: &str, enabled: bool) {
    if let Some(p) = pipeline.passes.iter_mut().find(|p| p.name == name) {
        p.enabled = enabled;
    }
}

#[allow(dead_code)]
pub fn rpv2_pass_count(pipeline: &RenderPipelineV2) -> usize {
    pipeline.passes.len()
}

#[allow(dead_code)]
pub fn rpv2_active_pass_count(pipeline: &RenderPipelineV2) -> usize {
    pipeline.passes.iter().filter(|p| p.enabled).count()
}

#[allow(dead_code)]
pub fn rpv2_remove_pass(pipeline: &mut RenderPipelineV2, name: &str) {
    pipeline.passes.retain(|p| p.name != name);
}

#[allow(dead_code)]
pub fn rpv2_set_msaa(pipeline: &mut RenderPipelineV2, samples: u32) {
    pipeline.msaa_samples = samples.max(1);
}

#[allow(dead_code)]
pub fn rpv2_set_hdr(pipeline: &mut RenderPipelineV2, enabled: bool) {
    pipeline.hdr_enabled = enabled;
}

#[allow(dead_code)]
pub fn rpv2_clear(pipeline: &mut RenderPipelineV2) {
    pipeline.passes.clear();
}

#[allow(dead_code)]
pub fn rpv2_to_json(pipeline: &RenderPipelineV2) -> String {
    format!(
        "{{\"name\":\"{}\",\"pass_count\":{},\"msaa_samples\":{},\"hdr\":{}}}",
        pipeline.name,
        pipeline.passes.len(),
        pipeline.msaa_samples,
        pipeline.hdr_enabled
    )
}

#[allow(dead_code)]
pub fn rpv2_has_pass(pipeline: &RenderPipelineV2, name: &str) -> bool {
    pipeline.passes.iter().any(|p| p.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pipeline() {
        let p = new_render_pipeline_v2("forward");
        assert_eq!(rpv2_pass_count(&p), 0);
    }

    #[test]
    fn test_add_pass() {
        let mut p = new_render_pipeline_v2("deferred");
        rpv2_add_pass(&mut p, "gbuffer", PipelineStage::Geometry, 0);
        assert_eq!(rpv2_pass_count(&p), 1);
    }

    #[test]
    fn test_enable_disable_pass() {
        let mut p = new_render_pipeline_v2("fwd");
        rpv2_add_pass(&mut p, "main", PipelineStage::Lighting, 0);
        rpv2_enable_pass(&mut p, "main", false);
        assert_eq!(rpv2_active_pass_count(&p), 0);
    }

    #[test]
    fn test_remove_pass() {
        let mut p = new_render_pipeline_v2("fwd");
        rpv2_add_pass(&mut p, "pass1", PipelineStage::PostProcess, 0);
        rpv2_remove_pass(&mut p, "pass1");
        assert!(!rpv2_has_pass(&p, "pass1"));
    }

    #[test]
    fn test_set_msaa() {
        let mut p = new_render_pipeline_v2("fwd");
        rpv2_set_msaa(&mut p, 4);
        assert_eq!(p.msaa_samples, 4);
    }

    #[test]
    fn test_msaa_min_one() {
        let mut p = new_render_pipeline_v2("fwd");
        rpv2_set_msaa(&mut p, 0);
        assert_eq!(p.msaa_samples, 1);
    }

    #[test]
    fn test_set_hdr() {
        let mut p = new_render_pipeline_v2("fwd");
        rpv2_set_hdr(&mut p, true);
        assert!(p.hdr_enabled);
    }

    #[test]
    fn test_clear() {
        let mut p = new_render_pipeline_v2("fwd");
        rpv2_add_pass(&mut p, "a", PipelineStage::Ui, 0);
        rpv2_clear(&mut p);
        assert_eq!(rpv2_pass_count(&p), 0);
    }

    #[test]
    fn test_to_json() {
        let p = new_render_pipeline_v2("forward");
        let json = rpv2_to_json(&p);
        assert!(json.contains("\"name\":\"forward\""));
    }

    #[test]
    fn test_active_pass_count_multiple() {
        let mut p = new_render_pipeline_v2("fwd");
        rpv2_add_pass(&mut p, "a", PipelineStage::Geometry, 0);
        rpv2_add_pass(&mut p, "b", PipelineStage::Lighting, 1);
        rpv2_enable_pass(&mut p, "a", false);
        assert_eq!(rpv2_active_pass_count(&p), 1);
    }
}
