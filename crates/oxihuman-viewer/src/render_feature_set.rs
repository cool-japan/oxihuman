#![allow(dead_code)]

use std::collections::HashSet;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderFeatureSet {
    features: HashSet<String>,
}

#[allow(dead_code)]
pub fn new_feature_set() -> RenderFeatureSet {
    RenderFeatureSet { features: HashSet::new() }
}

#[allow(dead_code)]
pub fn enable_feature_fs(fs: &mut RenderFeatureSet, name: &str) { fs.features.insert(name.to_string()); }

#[allow(dead_code)]
pub fn disable_feature_fs(fs: &mut RenderFeatureSet, name: &str) { fs.features.remove(name); }

#[allow(dead_code)]
pub fn is_enabled_fs(fs: &RenderFeatureSet, name: &str) -> bool { fs.features.contains(name) }

#[allow(dead_code)]
pub fn feature_count_fs(fs: &RenderFeatureSet) -> usize { fs.features.len() }

#[allow(dead_code)]
pub fn features_to_vec(fs: &RenderFeatureSet) -> Vec<String> { fs.features.iter().cloned().collect() }

#[allow(dead_code)]
pub fn feature_set_to_json(fs: &RenderFeatureSet) -> String {
    format!("{{\"count\":{}}}", fs.features.len())
}

#[allow(dead_code)]
pub fn clear_feature_set(fs: &mut RenderFeatureSet) { fs.features.clear(); }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let f = new_feature_set(); assert_eq!(feature_count_fs(&f), 0); }
    #[test] fn test_enable() { let mut f = new_feature_set(); enable_feature_fs(&mut f, "ssao"); assert!(is_enabled_fs(&f, "ssao")); }
    #[test] fn test_disable() { let mut f = new_feature_set(); enable_feature_fs(&mut f, "x"); disable_feature_fs(&mut f, "x"); assert!(!is_enabled_fs(&f, "x")); }
    #[test] fn test_count() { let mut f = new_feature_set(); enable_feature_fs(&mut f, "a"); enable_feature_fs(&mut f, "b"); assert_eq!(feature_count_fs(&f), 2); }
    #[test] fn test_to_vec() { let mut f = new_feature_set(); enable_feature_fs(&mut f, "x"); assert_eq!(features_to_vec(&f).len(), 1); }
    #[test] fn test_json() { let f = new_feature_set(); assert!(feature_set_to_json(&f).contains("count")); }
    #[test] fn test_clear() { let mut f = new_feature_set(); enable_feature_fs(&mut f, "a"); clear_feature_set(&mut f); assert_eq!(feature_count_fs(&f), 0); }
    #[test] fn test_not_enabled() { let f = new_feature_set(); assert!(!is_enabled_fs(&f, "z")); }
    #[test] fn test_double_enable() { let mut f = new_feature_set(); enable_feature_fs(&mut f, "x"); enable_feature_fs(&mut f, "x"); assert_eq!(feature_count_fs(&f), 1); }
    #[test] fn test_disable_missing() { let mut f = new_feature_set(); disable_feature_fs(&mut f, "x"); assert_eq!(feature_count_fs(&f), 0); }
}
