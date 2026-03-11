// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

/// Feature flags for renderer.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeatureFlag {
    Shadows,
    Ssao,
    Bloom,
    Fxaa,
    Hdr,
    Msaa,
}

/// Manages enabled render features.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderFeature {
    features: HashMap<FeatureFlag, bool>,
}

#[allow(dead_code)]
pub fn new_render_feature() -> RenderFeature {
    RenderFeature {
        features: HashMap::new(),
    }
}

#[allow(dead_code)]
pub fn enable_feature(rf: &mut RenderFeature, flag: FeatureFlag) {
    rf.features.insert(flag, true);
}

#[allow(dead_code)]
pub fn disable_feature(rf: &mut RenderFeature, flag: FeatureFlag) {
    rf.features.insert(flag, false);
}

#[allow(dead_code)]
pub fn is_feature_enabled(rf: &RenderFeature, flag: FeatureFlag) -> bool {
    rf.features.get(&flag).copied().unwrap_or(false)
}

#[allow(dead_code)]
pub fn feature_count_rf(rf: &RenderFeature) -> usize {
    rf.features.values().filter(|&&v| v).count()
}

#[allow(dead_code)]
pub fn feature_name(flag: FeatureFlag) -> &'static str {
    match flag {
        FeatureFlag::Shadows => "shadows",
        FeatureFlag::Ssao => "ssao",
        FeatureFlag::Bloom => "bloom",
        FeatureFlag::Fxaa => "fxaa",
        FeatureFlag::Hdr => "hdr",
        FeatureFlag::Msaa => "msaa",
    }
}

#[allow(dead_code)]
pub fn features_to_json(rf: &RenderFeature) -> String {
    let e: Vec<String> = rf
        .features
        .iter()
        .map(|(k, v)| format!("\"{}\":{}", feature_name(*k), v))
        .collect();
    format!("{{{}}}", e.join(","))
}

#[allow(dead_code)]
pub fn clear_features(rf: &mut RenderFeature) {
    rf.features.clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new() {
        assert_eq!(feature_count_rf(&new_render_feature()), 0);
    }
    #[test]
    fn test_enable() {
        let mut f = new_render_feature();
        enable_feature(&mut f, FeatureFlag::Shadows);
        assert!(is_feature_enabled(&f, FeatureFlag::Shadows));
    }
    #[test]
    fn test_disable() {
        let mut f = new_render_feature();
        enable_feature(&mut f, FeatureFlag::Bloom);
        disable_feature(&mut f, FeatureFlag::Bloom);
        assert!(!is_feature_enabled(&f, FeatureFlag::Bloom));
    }
    #[test]
    fn test_count() {
        let mut f = new_render_feature();
        enable_feature(&mut f, FeatureFlag::Ssao);
        enable_feature(&mut f, FeatureFlag::Hdr);
        assert_eq!(feature_count_rf(&f), 2);
    }
    #[test]
    fn test_name() {
        assert_eq!(feature_name(FeatureFlag::Fxaa), "fxaa");
    }
    #[test]
    fn test_not_enabled() {
        assert!(!is_feature_enabled(
            &new_render_feature(),
            FeatureFlag::Msaa
        ));
    }
    #[test]
    fn test_clear() {
        let mut f = new_render_feature();
        enable_feature(&mut f, FeatureFlag::Shadows);
        clear_features(&mut f);
        assert_eq!(feature_count_rf(&f), 0);
    }
    #[test]
    fn test_to_json() {
        let mut f = new_render_feature();
        enable_feature(&mut f, FeatureFlag::Bloom);
        assert!(features_to_json(&f).contains("bloom"));
    }
    #[test]
    fn test_enable_twice() {
        let mut f = new_render_feature();
        enable_feature(&mut f, FeatureFlag::Shadows);
        enable_feature(&mut f, FeatureFlag::Shadows);
        assert_eq!(feature_count_rf(&f), 1);
    }
    #[test]
    fn test_msaa_name() {
        assert_eq!(feature_name(FeatureFlag::Msaa), "msaa");
    }
}
