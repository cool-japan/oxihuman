// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct PipelineStruct {
    pub stages: Vec<String>,
}

pub struct PipelineContextStruct {
    pub data: std::collections::HashMap<String, String>,
    pub stage: usize,
}

pub fn new_pipeline_struct(stages: Vec<&str>) -> PipelineStruct {
    PipelineStruct {
        stages: stages.into_iter().map(|s| s.to_string()).collect(),
    }
}

pub fn pipeline_stage_count(p: &PipelineStruct) -> usize {
    p.stages.len()
}

pub fn new_pipeline_context_struct() -> PipelineContextStruct {
    PipelineContextStruct {
        data: std::collections::HashMap::new(),
        stage: 0,
    }
}

pub fn context_set(c: &mut PipelineContextStruct, key: &str, val: &str) {
    c.data.insert(key.to_string(), val.to_string());
}

pub fn context_get<'a>(c: &'a PipelineContextStruct, key: &str) -> Option<&'a str> {
    c.data.get(key).map(|s| s.as_str())
}

pub fn context_advance(c: &mut PipelineContextStruct) {
    c.stage += 1;
}

pub fn context_stage(c: &PipelineContextStruct) -> usize {
    c.stage
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_stage_count() {
        /* count stages in pipeline */
        let p = new_pipeline_struct(vec!["parse", "validate", "execute"]);
        assert_eq!(pipeline_stage_count(&p), 3);
    }

    #[test]
    fn test_context_set_get() {
        /* set and get context values */
        let mut c = new_pipeline_context_struct();
        context_set(&mut c, "key", "value");
        assert_eq!(context_get(&c, "key"), Some("value"));
        assert_eq!(context_get(&c, "missing"), None);
    }

    #[test]
    fn test_context_advance() {
        /* advance stage index */
        let mut c = new_pipeline_context_struct();
        assert_eq!(context_stage(&c), 0);
        context_advance(&mut c);
        assert_eq!(context_stage(&c), 1);
    }

    #[test]
    fn test_empty_pipeline() {
        /* empty pipeline has zero stages */
        let p = new_pipeline_struct(vec![]);
        assert_eq!(pipeline_stage_count(&p), 0);
    }

    #[test]
    fn test_context_multiple_keys() {
        /* multiple key-value pairs stored */
        let mut c = new_pipeline_context_struct();
        context_set(&mut c, "a", "1");
        context_set(&mut c, "b", "2");
        assert_eq!(context_get(&c, "a"), Some("1"));
        assert_eq!(context_get(&c, "b"), Some("2"));
    }

    #[test]
    fn test_context_overwrite() {
        /* overwriting a key updates it */
        let mut c = new_pipeline_context_struct();
        context_set(&mut c, "x", "old");
        context_set(&mut c, "x", "new");
        assert_eq!(context_get(&c, "x"), Some("new"));
    }

    #[test]
    fn test_context_advance_multiple() {
        /* advance multiple times */
        let mut c = new_pipeline_context_struct();
        context_advance(&mut c);
        context_advance(&mut c);
        assert_eq!(context_stage(&c), 2);
    }
}
