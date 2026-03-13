#![allow(dead_code)]
//! Render pipeline with configurable stages.

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct PipelineStage {
    pub name: String,
    pub order: u32,
    pub enabled: bool,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct RenderPipeline {
    pub name: String,
    stages: Vec<PipelineStage>,
}

#[allow(dead_code)]
pub fn new_render_pipeline(name: &str) -> RenderPipeline {
    RenderPipeline {
        name: name.to_string(),
        stages: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_pipeline_stage(p: &mut RenderPipeline, name: &str, order: u32) {
    p.stages.push(PipelineStage {
        name: name.to_string(),
        order,
        enabled: true,
    });
    p.stages.sort_by_key(|s| s.order);
}

#[allow(dead_code)]
pub fn stage_count(p: &RenderPipeline) -> usize {
    p.stages.len()
}

#[allow(dead_code)]
pub fn execute_pipeline(p: &RenderPipeline) -> Vec<&str> {
    p.stages
        .iter()
        .filter(|s| s.enabled)
        .map(|s| s.name.as_str())
        .collect()
}

#[allow(dead_code)]
pub fn pipeline_name(p: &RenderPipeline) -> &str {
    &p.name
}

#[allow(dead_code)]
pub fn stage_at(p: &RenderPipeline, index: usize) -> Option<&PipelineStage> {
    p.stages.get(index)
}

#[allow(dead_code)]
pub fn pipeline_to_json(p: &RenderPipeline) -> String {
    let stages: Vec<String> = p
        .stages
        .iter()
        .map(|s| format!("{{\"name\":\"{}\",\"order\":{}}}", s.name, s.order))
        .collect();
    format!(
        "{{\"name\":\"{}\",\"stages\":[{}]}}",
        p.name,
        stages.join(",")
    )
}

#[allow(dead_code)]
pub fn pipeline_clear(p: &mut RenderPipeline) {
    p.stages.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_render_pipeline() {
        let p = new_render_pipeline("main");
        assert_eq!(pipeline_name(&p), "main");
        assert_eq!(stage_count(&p), 0);
    }

    #[test]
    fn test_add_pipeline_stage() {
        let mut p = new_render_pipeline("p");
        add_pipeline_stage(&mut p, "depth", 0);
        assert_eq!(stage_count(&p), 1);
    }

    #[test]
    fn test_execute_pipeline() {
        let mut p = new_render_pipeline("p");
        add_pipeline_stage(&mut p, "depth", 0);
        add_pipeline_stage(&mut p, "color", 1);
        let order = execute_pipeline(&p);
        assert_eq!(order, vec!["depth", "color"]);
    }

    #[test]
    fn test_stage_at() {
        let mut p = new_render_pipeline("p");
        add_pipeline_stage(&mut p, "s", 0);
        assert_eq!(stage_at(&p, 0).expect("should succeed").name, "s");
        assert!(stage_at(&p, 5).is_none());
    }

    #[test]
    fn test_pipeline_to_json() {
        let p = new_render_pipeline("main");
        let json = pipeline_to_json(&p);
        assert!(json.contains("\"name\":\"main\""));
    }

    #[test]
    fn test_pipeline_clear() {
        let mut p = new_render_pipeline("p");
        add_pipeline_stage(&mut p, "s", 0);
        pipeline_clear(&mut p);
        assert_eq!(stage_count(&p), 0);
    }

    #[test]
    fn test_ordering() {
        let mut p = new_render_pipeline("p");
        add_pipeline_stage(&mut p, "b", 2);
        add_pipeline_stage(&mut p, "a", 1);
        assert_eq!(stage_at(&p, 0).expect("should succeed").name, "a");
    }

    #[test]
    fn test_pipeline_name() {
        let p = new_render_pipeline("forward");
        assert_eq!(pipeline_name(&p), "forward");
    }

    #[test]
    fn test_execute_empty() {
        let p = new_render_pipeline("p");
        assert!(execute_pipeline(&p).is_empty());
    }

    #[test]
    fn test_multiple_stages() {
        let mut p = new_render_pipeline("p");
        for i in 0..5 {
            add_pipeline_stage(&mut p, &format!("s{i}"), i);
        }
        assert_eq!(stage_count(&p), 5);
    }
}
