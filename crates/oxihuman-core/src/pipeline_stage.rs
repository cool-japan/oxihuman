#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Pipeline stage execution model.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum StageResult {
    Pending,
    Success,
    Failed(String),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PipelineStage {
    name: String,
    complete: bool,
    result: StageResult,
    dependencies: Vec<String>,
}

#[allow(dead_code)]
pub fn new_pipeline_stage(name: &str) -> PipelineStage {
    PipelineStage {
        name: name.to_string(),
        complete: false,
        result: StageResult::Pending,
        dependencies: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn execute_stage(stage: &mut PipelineStage) -> &StageResult {
    if !stage.complete {
        stage.result = StageResult::Success;
        stage.complete = true;
    }
    &stage.result
}

#[allow(dead_code)]
pub fn stage_name_ps(stage: &PipelineStage) -> &str {
    &stage.name
}

#[allow(dead_code)]
pub fn stage_is_complete(stage: &PipelineStage) -> bool {
    stage.complete
}

#[allow(dead_code)]
pub fn stage_result(stage: &PipelineStage) -> &StageResult {
    &stage.result
}

#[allow(dead_code)]
pub fn stage_to_json(stage: &PipelineStage) -> String {
    let result_str = match &stage.result {
        StageResult::Pending => "pending",
        StageResult::Success => "success",
        StageResult::Failed(_) => "failed",
    };
    format!(
        r#"{{"name":"{}","complete":{},"result":"{}"}}"#,
        stage.name, stage.complete, result_str
    )
}

#[allow(dead_code)]
pub fn stage_reset(stage: &mut PipelineStage) {
    stage.complete = false;
    stage.result = StageResult::Pending;
}

#[allow(dead_code)]
pub fn stage_dependencies(stage: &PipelineStage) -> &[String] {
    &stage.dependencies
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stage() {
        let s = new_pipeline_stage("build");
        assert_eq!(stage_name_ps(&s), "build");
        assert!(!stage_is_complete(&s));
    }

    #[test]
    fn test_execute_stage() {
        let mut s = new_pipeline_stage("build");
        execute_stage(&mut s);
        assert!(stage_is_complete(&s));
        assert_eq!(*stage_result(&s), StageResult::Success);
    }

    #[test]
    fn test_stage_reset() {
        let mut s = new_pipeline_stage("build");
        execute_stage(&mut s);
        stage_reset(&mut s);
        assert!(!stage_is_complete(&s));
        assert_eq!(*stage_result(&s), StageResult::Pending);
    }

    #[test]
    fn test_stage_to_json() {
        let s = new_pipeline_stage("test");
        let json = stage_to_json(&s);
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("\"result\":\"pending\""));
    }

    #[test]
    fn test_stage_to_json_after_exec() {
        let mut s = new_pipeline_stage("test");
        execute_stage(&mut s);
        let json = stage_to_json(&s);
        assert!(json.contains("\"result\":\"success\""));
    }

    #[test]
    fn test_dependencies_empty() {
        let s = new_pipeline_stage("x");
        assert!(stage_dependencies(&s).is_empty());
    }

    #[test]
    fn test_double_execute() {
        let mut s = new_pipeline_stage("x");
        execute_stage(&mut s);
        execute_stage(&mut s);
        assert!(stage_is_complete(&s));
    }

    #[test]
    fn test_stage_result_pending() {
        let s = new_pipeline_stage("x");
        assert_eq!(*stage_result(&s), StageResult::Pending);
    }

    #[test]
    fn test_failed_result() {
        let r = StageResult::Failed("oops".to_string());
        assert_eq!(r, StageResult::Failed("oops".to_string()));
    }

    #[test]
    fn test_stage_name() {
        let s = new_pipeline_stage("deploy");
        assert_eq!(stage_name_ps(&s), "deploy");
    }
}
