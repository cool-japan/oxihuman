#![allow(dead_code)]

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderPipelineCache {
    cache: HashMap<String, String>,
    hits: u64,
    misses: u64,
}

#[allow(dead_code)]
pub fn new_pipeline_cache() -> RenderPipelineCache {
    RenderPipelineCache { cache: HashMap::new(), hits: 0, misses: 0 }
}

#[allow(dead_code)]
pub fn cache_pipeline(c: &mut RenderPipelineCache, key: &str, data: &str) {
    c.cache.insert(key.to_string(), data.to_string());
}

#[allow(dead_code)]
pub fn get_cached_pipeline(c: &mut RenderPipelineCache, key: &str) -> Option<String> {
    if let Some(v) = c.cache.get(key) { c.hits += 1; Some(v.clone()) }
    else { c.misses += 1; None }
}

#[allow(dead_code)]
pub fn pipeline_cache_count(c: &RenderPipelineCache) -> usize { c.cache.len() }

#[allow(dead_code)]
pub fn pipeline_cache_hit_rate(c: &RenderPipelineCache) -> f32 {
    let total = c.hits + c.misses;
    if total == 0 { 0.0 } else { c.hits as f32 / total as f32 }
}

#[allow(dead_code)]
pub fn evict_pipeline(c: &mut RenderPipelineCache, key: &str) { c.cache.remove(key); }

#[allow(dead_code)]
pub fn pipeline_cache_to_json(c: &RenderPipelineCache) -> String {
    format!("{{\"count\":{},\"hit_rate\":{:.4}}}", c.cache.len(), pipeline_cache_hit_rate(c))
}

#[allow(dead_code)]
pub fn clear_pipeline_cache(c: &mut RenderPipelineCache) { c.cache.clear(); c.hits = 0; c.misses = 0; }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let c = new_pipeline_cache(); assert_eq!(pipeline_cache_count(&c), 0); }
    #[test] fn test_cache() { let mut c = new_pipeline_cache(); cache_pipeline(&mut c, "k", "v"); assert_eq!(pipeline_cache_count(&c), 1); }
    #[test] fn test_get_hit() { let mut c = new_pipeline_cache(); cache_pipeline(&mut c, "k", "v"); assert_eq!(get_cached_pipeline(&mut c, "k"), Some("v".to_string())); }
    #[test] fn test_get_miss() { let mut c = new_pipeline_cache(); assert_eq!(get_cached_pipeline(&mut c, "x"), None); }
    #[test] fn test_hit_rate() { let mut c = new_pipeline_cache(); cache_pipeline(&mut c, "k", "v"); get_cached_pipeline(&mut c, "k"); assert!((pipeline_cache_hit_rate(&c) - 1.0).abs() < 1e-6); }
    #[test] fn test_evict() { let mut c = new_pipeline_cache(); cache_pipeline(&mut c, "k", "v"); evict_pipeline(&mut c, "k"); assert_eq!(pipeline_cache_count(&c), 0); }
    #[test] fn test_json() { let c = new_pipeline_cache(); assert!(pipeline_cache_to_json(&c).contains("count")); }
    #[test] fn test_clear() { let mut c = new_pipeline_cache(); cache_pipeline(&mut c, "k", "v"); clear_pipeline_cache(&mut c); assert_eq!(pipeline_cache_count(&c), 0); }
    #[test] fn test_hit_rate_zero() { let c = new_pipeline_cache(); assert!((pipeline_cache_hit_rate(&c)).abs() < 1e-6); }
    #[test] fn test_overwrite() { let mut c = new_pipeline_cache(); cache_pipeline(&mut c, "k", "a"); cache_pipeline(&mut c, "k", "b"); assert_eq!(get_cached_pipeline(&mut c, "k"), Some("b".to_string())); }
}
