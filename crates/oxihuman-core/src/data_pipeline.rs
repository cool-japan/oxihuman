//! Data processing pipeline with sequential stages.

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum StageStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct PipelineStage {
    pub id: u32,
    pub name: String,
    pub status: StageStatus,
    pub input_keys: Vec<String>,
    pub output_keys: Vec<String>,
    pub duration_ms: Option<u64>,
    pub error: Option<String>,
}

#[allow(dead_code)]
pub struct DataPipeline {
    pub name: String,
    pub stages: Vec<PipelineStage>,
    pub context: Vec<(String, String)>,
    pub next_id: u32,
    pub current_stage: usize,
}

#[allow(dead_code)]
pub fn new_pipeline(name: &str) -> DataPipeline {
    DataPipeline {
        name: name.to_string(),
        stages: Vec::new(),
        context: Vec::new(),
        next_id: 0,
        current_stage: 0,
    }
}

#[allow(dead_code)]
pub fn add_stage(
    pipeline: &mut DataPipeline,
    name: &str,
    inputs: Vec<String>,
    outputs: Vec<String>,
) -> u32 {
    let id = pipeline.next_id;
    pipeline.next_id += 1;
    pipeline.stages.push(PipelineStage {
        id,
        name: name.to_string(),
        status: StageStatus::Pending,
        input_keys: inputs,
        output_keys: outputs,
        duration_ms: None,
        error: None,
    });
    id
}

#[allow(dead_code)]
pub fn set_context_value(pipeline: &mut DataPipeline, key: &str, value: &str) {
    for entry in &mut pipeline.context {
        if entry.0 == key {
            entry.1 = value.to_string();
            return;
        }
    }
    pipeline.context.push((key.to_string(), value.to_string()));
}

#[allow(dead_code)]
pub fn get_context_value<'a>(pipeline: &'a DataPipeline, key: &str) -> Option<&'a str> {
    pipeline
        .context
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

#[allow(dead_code)]
pub fn advance_stage(pipeline: &mut DataPipeline) {
    if pipeline.current_stage < pipeline.stages.len() {
        pipeline.current_stage += 1;
    }
}

#[allow(dead_code)]
pub fn mark_stage_complete(pipeline: &mut DataPipeline, id: u32, duration_ms: u64) {
    for stage in &mut pipeline.stages {
        if stage.id == id {
            stage.status = StageStatus::Completed;
            stage.duration_ms = Some(duration_ms);
            return;
        }
    }
}

#[allow(dead_code)]
pub fn mark_stage_failed(pipeline: &mut DataPipeline, id: u32, error: &str) {
    for stage in &mut pipeline.stages {
        if stage.id == id {
            stage.status = StageStatus::Failed;
            stage.error = Some(error.to_string());
            return;
        }
    }
}

#[allow(dead_code)]
pub fn mark_stage_skipped(pipeline: &mut DataPipeline, id: u32) {
    for stage in &mut pipeline.stages {
        if stage.id == id {
            stage.status = StageStatus::Skipped;
            return;
        }
    }
}

#[allow(dead_code)]
pub fn pipeline_progress(pipeline: &DataPipeline) -> f32 {
    let total = pipeline.stages.len();
    if total == 0 {
        return 1.0;
    }
    let done = completed_stage_count(pipeline);
    done as f32 / total as f32
}

#[allow(dead_code)]
pub fn stage_count(pipeline: &DataPipeline) -> usize {
    pipeline.stages.len()
}

#[allow(dead_code)]
pub fn completed_stage_count(pipeline: &DataPipeline) -> usize {
    pipeline
        .stages
        .iter()
        .filter(|s| s.status == StageStatus::Completed)
        .count()
}

#[allow(dead_code)]
pub fn failed_stages(pipeline: &DataPipeline) -> Vec<&PipelineStage> {
    pipeline
        .stages
        .iter()
        .filter(|s| s.status == StageStatus::Failed)
        .collect()
}

#[allow(dead_code)]
pub fn reset_pipeline(pipeline: &mut DataPipeline) {
    for stage in &mut pipeline.stages {
        stage.status = StageStatus::Pending;
        stage.duration_ms = None;
        stage.error = None;
    }
    pipeline.current_stage = 0;
}

#[allow(dead_code)]
pub fn pipeline_to_json(pipeline: &DataPipeline) -> String {
    let mut s = String::new();
    s.push_str("{\"name\":\"");
    s.push_str(&pipeline.name);
    s.push_str("\",\"stages\":[");
    for (i, stage) in pipeline.stages.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str("{\"id\":");
        s.push_str(&stage.id.to_string());
        s.push_str(",\"name\":\"");
        s.push_str(&stage.name);
        s.push_str("\",\"status\":\"");
        let status_str = match &stage.status {
            StageStatus::Pending => "Pending",
            StageStatus::Running => "Running",
            StageStatus::Completed => "Completed",
            StageStatus::Failed => "Failed",
            StageStatus::Skipped => "Skipped",
        };
        s.push_str(status_str);
        s.push_str("\"}");
    }
    s.push_str("]}");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pipeline() {
        let p = new_pipeline("test");
        assert_eq!(p.name, "test");
        assert!(p.stages.is_empty());
        assert!(p.context.is_empty());
        assert_eq!(p.next_id, 0);
        assert_eq!(p.current_stage, 0);
    }

    #[test]
    fn test_add_stage() {
        let mut p = new_pipeline("pipe");
        let id = add_stage(&mut p, "load", vec![], vec!["data".to_string()]);
        assert_eq!(id, 0);
        assert_eq!(stage_count(&p), 1);
        let id2 = add_stage(&mut p, "process", vec!["data".to_string()], vec![]);
        assert_eq!(id2, 1);
        assert_eq!(stage_count(&p), 2);
    }

    #[test]
    fn test_set_get_context() {
        let mut p = new_pipeline("pipe");
        set_context_value(&mut p, "key1", "value1");
        assert_eq!(get_context_value(&p, "key1"), Some("value1"));
        assert_eq!(get_context_value(&p, "missing"), None);
    }

    #[test]
    fn test_set_context_overwrite() {
        let mut p = new_pipeline("pipe");
        set_context_value(&mut p, "k", "v1");
        set_context_value(&mut p, "k", "v2");
        assert_eq!(get_context_value(&p, "k"), Some("v2"));
        assert_eq!(p.context.len(), 1);
    }

    #[test]
    fn test_mark_stage_complete() {
        let mut p = new_pipeline("pipe");
        let id = add_stage(&mut p, "s1", vec![], vec![]);
        mark_stage_complete(&mut p, id, 100);
        assert_eq!(p.stages[0].status, StageStatus::Completed);
        assert_eq!(p.stages[0].duration_ms, Some(100));
    }

    #[test]
    fn test_mark_stage_failed() {
        let mut p = new_pipeline("pipe");
        let id = add_stage(&mut p, "s1", vec![], vec![]);
        mark_stage_failed(&mut p, id, "something went wrong");
        assert_eq!(p.stages[0].status, StageStatus::Failed);
        assert!(p.stages[0].error.is_some());
    }

    #[test]
    fn test_mark_stage_skipped() {
        let mut p = new_pipeline("pipe");
        let id = add_stage(&mut p, "s1", vec![], vec![]);
        mark_stage_skipped(&mut p, id);
        assert_eq!(p.stages[0].status, StageStatus::Skipped);
    }

    #[test]
    fn test_pipeline_progress_empty() {
        let p = new_pipeline("pipe");
        assert!((pipeline_progress(&p) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_pipeline_progress_partial() {
        let mut p = new_pipeline("pipe");
        let id0 = add_stage(&mut p, "s1", vec![], vec![]);
        let _id1 = add_stage(&mut p, "s2", vec![], vec![]);
        mark_stage_complete(&mut p, id0, 10);
        let prog = pipeline_progress(&p);
        assert!((prog - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_failed_stages() {
        let mut p = new_pipeline("pipe");
        let id0 = add_stage(&mut p, "s1", vec![], vec![]);
        let _id1 = add_stage(&mut p, "s2", vec![], vec![]);
        mark_stage_failed(&mut p, id0, "error");
        let failed = failed_stages(&p);
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0].id, id0);
    }

    #[test]
    fn test_reset_pipeline() {
        let mut p = new_pipeline("pipe");
        let id = add_stage(&mut p, "s1", vec![], vec![]);
        mark_stage_complete(&mut p, id, 50);
        advance_stage(&mut p);
        reset_pipeline(&mut p);
        assert_eq!(p.stages[0].status, StageStatus::Pending);
        assert_eq!(p.current_stage, 0);
    }

    #[test]
    fn test_advance_stage() {
        let mut p = new_pipeline("pipe");
        add_stage(&mut p, "s1", vec![], vec![]);
        add_stage(&mut p, "s2", vec![], vec![]);
        assert_eq!(p.current_stage, 0);
        advance_stage(&mut p);
        assert_eq!(p.current_stage, 1);
    }

    #[test]
    fn test_advance_stage_clamped() {
        let mut p = new_pipeline("pipe");
        add_stage(&mut p, "s1", vec![], vec![]);
        advance_stage(&mut p);
        advance_stage(&mut p); // Should not exceed stages.len()
        assert_eq!(p.current_stage, 1);
    }

    #[test]
    fn test_pipeline_to_json_non_empty() {
        let mut p = new_pipeline("mypipe");
        add_stage(&mut p, "load", vec![], vec![]);
        let json = pipeline_to_json(&p);
        assert!(json.contains("mypipe"));
        assert!(json.contains("load"));
    }

    #[test]
    fn test_completed_stage_count() {
        let mut p = new_pipeline("pipe");
        let id0 = add_stage(&mut p, "s1", vec![], vec![]);
        let id1 = add_stage(&mut p, "s2", vec![], vec![]);
        mark_stage_complete(&mut p, id0, 1);
        mark_stage_complete(&mut p, id1, 2);
        assert_eq!(completed_stage_count(&p), 2);
    }
}
