// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Decal fade-in/fade-out lifecycle controller.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecalFadePhase {
    In,
    Stable,
    Out,
    Done,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DecalFadeConfig {
    pub fade_in_duration: f32,
    pub stable_duration: f32,
    pub fade_out_duration: f32,
}

impl Default for DecalFadeConfig {
    fn default() -> Self {
        Self {
            fade_in_duration: 0.3,
            stable_duration: 2.0,
            fade_out_duration: 0.5,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DecalFadeState {
    pub elapsed: f32,
    pub phase: DecalFadePhase,
    pub config: DecalFadeConfig,
}

#[allow(dead_code)]
pub fn default_decal_fade_config() -> DecalFadeConfig {
    DecalFadeConfig::default()
}

#[allow(dead_code)]
pub fn new_decal_fade_state(config: DecalFadeConfig) -> DecalFadeState {
    DecalFadeState {
        elapsed: 0.0,
        phase: DecalFadePhase::In,
        config,
    }
}

#[allow(dead_code)]
pub fn df_advance(state: &mut DecalFadeState, dt: f32) {
    state.elapsed += dt;
    let cfg = &state.config;
    state.phase = match state.phase {
        DecalFadePhase::In if state.elapsed >= cfg.fade_in_duration => {
            state.elapsed -= cfg.fade_in_duration;
            DecalFadePhase::Stable
        }
        DecalFadePhase::Stable if state.elapsed >= cfg.stable_duration => {
            state.elapsed -= cfg.stable_duration;
            DecalFadePhase::Out
        }
        DecalFadePhase::Out if state.elapsed >= cfg.fade_out_duration => {
            state.elapsed = 0.0;
            DecalFadePhase::Done
        }
        p => p,
    };
}

#[allow(dead_code)]
pub fn df_reset(state: &mut DecalFadeState) {
    state.elapsed = 0.0;
    state.phase = DecalFadePhase::In;
}

#[allow(dead_code)]
pub fn df_is_done(state: &DecalFadeState) -> bool {
    state.phase == DecalFadePhase::Done
}

#[allow(dead_code)]
pub fn df_alpha(state: &DecalFadeState) -> f32 {
    match state.phase {
        DecalFadePhase::In => {
            (state.elapsed / state.config.fade_in_duration.max(1e-9)).clamp(0.0, 1.0)
        }
        DecalFadePhase::Stable => 1.0,
        DecalFadePhase::Out => {
            1.0 - (state.elapsed / state.config.fade_out_duration.max(1e-9)).clamp(0.0, 1.0)
        }
        DecalFadePhase::Done => 0.0,
    }
}

#[allow(dead_code)]
pub fn df_phase_angle_rad(state: &DecalFadeState) -> f32 {
    df_alpha(state) * PI * 0.5
}

#[allow(dead_code)]
pub fn df_total_duration(config: &DecalFadeConfig) -> f32 {
    config.fade_in_duration + config.stable_duration + config.fade_out_duration
}

#[allow(dead_code)]
pub fn df_to_json(state: &DecalFadeState) -> String {
    format!(
        "{{\"alpha\":{:.4},\"done\":{}}}",
        df_alpha(state),
        df_is_done(state)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn initial_phase_in() {
        assert_eq!(
            new_decal_fade_state(default_decal_fade_config()).phase,
            DecalFadePhase::In
        );
    }
    #[test]
    fn alpha_starts_at_zero() {
        let s = new_decal_fade_state(default_decal_fade_config());
        assert!(df_alpha(&s).abs() < 1e-6);
    }
    #[test]
    fn alpha_clamps_to_one() {
        let mut s = new_decal_fade_state(default_decal_fade_config());
        s.elapsed = 999.0;
        s.phase = DecalFadePhase::In;
        assert!((0.0..=1.0).contains(&df_alpha(&s)));
    }
    #[test]
    fn advance_progresses_phase() {
        let mut s = new_decal_fade_state(default_decal_fade_config());
        df_advance(&mut s, 1.0);
        assert_ne!(s.phase, DecalFadePhase::In);
    }
    #[test]
    fn stable_alpha_is_one() {
        let mut s = new_decal_fade_state(default_decal_fade_config());
        s.phase = DecalFadePhase::Stable;
        s.elapsed = 0.0;
        assert!((df_alpha(&s) - 1.0).abs() < 1e-5);
    }
    #[test]
    fn done_alpha_is_zero() {
        let mut s = new_decal_fade_state(default_decal_fade_config());
        s.phase = DecalFadePhase::Done;
        assert!(df_alpha(&s).abs() < 1e-6);
    }
    #[test]
    fn reset_goes_back_to_in() {
        let mut s = new_decal_fade_state(default_decal_fade_config());
        df_advance(&mut s, 10.0);
        df_reset(&mut s);
        assert_eq!(s.phase, DecalFadePhase::In);
    }
    #[test]
    fn is_done_after_full_cycle() {
        let mut s = new_decal_fade_state(default_decal_fade_config());
        /* advance multiple times to traverse In -> Stable -> Out -> Done */
        for _ in 0..10 {
            df_advance(&mut s, 1.0);
        }
        assert!(df_is_done(&s));
    }
    #[test]
    fn phase_angle_nonneg() {
        let s = new_decal_fade_state(default_decal_fade_config());
        assert!(df_phase_angle_rad(&s) >= 0.0);
    }
    #[test]
    fn to_json_has_alpha() {
        assert!(
            df_to_json(&new_decal_fade_state(default_decal_fade_config())).contains("\"alpha\"")
        );
    }
}
