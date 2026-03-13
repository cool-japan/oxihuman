// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! GPU occlusion query stub.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OcclusionQueryConfig {
    pub conservative: bool,
    pub threshold_pixels: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OcclusionQueryResult {
    pub object_id: u32,
    pub visible_samples: u32,
    pub is_visible: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OcclusionQueryBatch {
    cfg: OcclusionQueryConfig,
    results: Vec<OcclusionQueryResult>,
}

#[allow(dead_code)]
pub fn default_occlusion_query_config() -> OcclusionQueryConfig {
    OcclusionQueryConfig { conservative: false, threshold_pixels: 1 }
}

#[allow(dead_code)]
pub fn new_occlusion_batch(cfg: OcclusionQueryConfig) -> OcclusionQueryBatch {
    OcclusionQueryBatch { cfg, results: Vec::new() }
}

/// Submit a query for `object_id` with `visible_samples` (stub: immediate result).
#[allow(dead_code)]
pub fn oq_submit(batch: &mut OcclusionQueryBatch, object_id: u32, visible_samples: u32) {
    let is_visible = visible_samples >= batch.cfg.threshold_pixels;
    batch.results.push(OcclusionQueryResult { object_id, visible_samples, is_visible });
}

/// Collect all results (returns a clone of the internal list).
#[allow(dead_code)]
pub fn oq_collect(batch: &OcclusionQueryBatch) -> Vec<OcclusionQueryResult> {
    batch.results.clone()
}

/// Returns true if the given object_id was last seen visible.
#[allow(dead_code)]
pub fn oq_is_visible(batch: &OcclusionQueryBatch, object_id: u32) -> bool {
    batch
        .results
        .iter().rfind(|r| r.object_id == object_id)
        .map(|r| r.is_visible)
        .unwrap_or(false)
}

/// Count of visible objects in the batch.
#[allow(dead_code)]
pub fn oq_visible_count(batch: &OcclusionQueryBatch) -> usize {
    batch.results.iter().filter(|r| r.is_visible).count()
}

#[allow(dead_code)]
pub fn oq_clear(batch: &mut OcclusionQueryBatch) {
    batch.results.clear();
}

#[allow(dead_code)]
pub fn oq_to_json(batch: &OcclusionQueryBatch) -> String {
    let entries: Vec<String> = batch
        .results
        .iter()
        .map(|r| {
            format!(
                r#"{{"object_id":{},"visible_samples":{},"is_visible":{}}}"#,
                r.object_id, r.visible_samples, r.is_visible
            )
        })
        .collect();
    format!("[{}]", entries.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_occlusion_query_config();
        assert!(!cfg.conservative);
        assert_eq!(cfg.threshold_pixels, 1);
    }

    #[test]
    fn test_new_batch_empty() {
        let batch = new_occlusion_batch(default_occlusion_query_config());
        assert_eq!(oq_visible_count(&batch), 0);
    }

    #[test]
    fn test_submit_visible() {
        let mut batch = new_occlusion_batch(default_occlusion_query_config());
        oq_submit(&mut batch, 1, 100);
        assert!(oq_is_visible(&batch, 1));
    }

    #[test]
    fn test_submit_occluded() {
        let mut batch = new_occlusion_batch(default_occlusion_query_config());
        oq_submit(&mut batch, 2, 0);
        assert!(!oq_is_visible(&batch, 2));
    }

    #[test]
    fn test_visible_count() {
        let mut batch = new_occlusion_batch(default_occlusion_query_config());
        oq_submit(&mut batch, 1, 10);
        oq_submit(&mut batch, 2, 0);
        oq_submit(&mut batch, 3, 5);
        assert_eq!(oq_visible_count(&batch), 2);
    }

    #[test]
    fn test_collect_count() {
        let mut batch = new_occlusion_batch(default_occlusion_query_config());
        oq_submit(&mut batch, 1, 1);
        oq_submit(&mut batch, 2, 2);
        assert_eq!(oq_collect(&batch).len(), 2);
    }

    #[test]
    fn test_clear() {
        let mut batch = new_occlusion_batch(default_occlusion_query_config());
        oq_submit(&mut batch, 1, 5);
        oq_clear(&mut batch);
        assert_eq!(oq_visible_count(&batch), 0);
    }

    #[test]
    fn test_is_visible_unknown_object() {
        let batch = new_occlusion_batch(default_occlusion_query_config());
        assert!(!oq_is_visible(&batch, 99));
    }

    #[test]
    fn test_to_json_contains_object_id() {
        let mut batch = new_occlusion_batch(default_occlusion_query_config());
        oq_submit(&mut batch, 7, 3);
        let j = oq_to_json(&batch);
        assert!(j.contains("object_id"));
        assert!(j.contains('7'));
    }
}
