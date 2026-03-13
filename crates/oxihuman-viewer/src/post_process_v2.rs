// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Post-process effect chain v2.

#[allow(dead_code)]
pub enum PostEffectKind {
    Bloom,
    SSAO,
    MotionBlur,
    ColorGrade,
    Vignette,
    ToneMap,
}

#[allow(dead_code)]
pub struct PostEffect {
    pub kind: PostEffectKind,
    pub enabled: bool,
    pub intensity: f32,
}

#[allow(dead_code)]
pub struct PostProcessChainV2 {
    pub effects: Vec<PostEffect>,
}

#[allow(dead_code)]
pub fn new_post_process_chain_v2() -> PostProcessChainV2 {
    PostProcessChainV2 { effects: Vec::new() }
}

#[allow(dead_code)]
pub fn ppv2_add_effect(chain: &mut PostProcessChainV2, kind: PostEffectKind, intensity: f32) {
    chain.effects.push(PostEffect { kind, enabled: true, intensity });
}

#[allow(dead_code)]
pub fn ppv2_enable_effect(chain: &mut PostProcessChainV2, idx: usize) {
    if idx < chain.effects.len() {
        chain.effects[idx].enabled = true;
    }
}

#[allow(dead_code)]
pub fn ppv2_disable_effect(chain: &mut PostProcessChainV2, idx: usize) {
    if idx < chain.effects.len() {
        chain.effects[idx].enabled = false;
    }
}

#[allow(dead_code)]
pub fn ppv2_effect_count(chain: &PostProcessChainV2) -> usize {
    chain.effects.len()
}

#[allow(dead_code)]
pub fn ppv2_active_count(chain: &PostProcessChainV2) -> usize {
    chain.effects.iter().filter(|e| e.enabled).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_effect() {
        let mut c = new_post_process_chain_v2();
        ppv2_add_effect(&mut c, PostEffectKind::Bloom, 1.0);
        assert_eq!(ppv2_effect_count(&c), 1);
    }

    #[test]
    fn test_effect_count() {
        let mut c = new_post_process_chain_v2();
        ppv2_add_effect(&mut c, PostEffectKind::Bloom, 1.0);
        ppv2_add_effect(&mut c, PostEffectKind::SSAO, 0.5);
        assert_eq!(ppv2_effect_count(&c), 2);
    }

    #[test]
    fn test_disable_effect() {
        let mut c = new_post_process_chain_v2();
        ppv2_add_effect(&mut c, PostEffectKind::Vignette, 0.5);
        ppv2_disable_effect(&mut c, 0);
        assert_eq!(ppv2_active_count(&c), 0);
    }

    #[test]
    fn test_enable_effect() {
        let mut c = new_post_process_chain_v2();
        ppv2_add_effect(&mut c, PostEffectKind::ToneMap, 1.0);
        ppv2_disable_effect(&mut c, 0);
        ppv2_enable_effect(&mut c, 0);
        assert_eq!(ppv2_active_count(&c), 1);
    }

    #[test]
    fn test_active_count_all_enabled() {
        let mut c = new_post_process_chain_v2();
        ppv2_add_effect(&mut c, PostEffectKind::Bloom, 1.0);
        ppv2_add_effect(&mut c, PostEffectKind::MotionBlur, 0.5);
        assert_eq!(ppv2_active_count(&c), 2);
    }

    #[test]
    fn test_active_count_none() {
        let mut c = new_post_process_chain_v2();
        ppv2_add_effect(&mut c, PostEffectKind::ColorGrade, 1.0);
        ppv2_disable_effect(&mut c, 0);
        assert_eq!(ppv2_active_count(&c), 0);
    }

    #[test]
    fn test_enable_out_of_bounds_safe() {
        let mut c = new_post_process_chain_v2();
        ppv2_enable_effect(&mut c, 99); /* should not panic */
    }

    #[test]
    fn test_disable_out_of_bounds_safe() {
        let mut c = new_post_process_chain_v2();
        ppv2_disable_effect(&mut c, 99); /* should not panic */
    }
}
