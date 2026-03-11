// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Particle system render debug view.

/// Particle render display mode.
#[derive(Debug, Clone, PartialEq)]
pub enum ParticleDisplayMode {
    Points,
    Circles,
    Objects,
    None,
}

/// Particle render debug state.
#[derive(Debug, Clone)]
pub struct ParticleRenderView {
    pub display_mode: ParticleDisplayMode,
    pub particle_count: u32,
    pub display_percentage: f32,
    pub show_velocity: bool,
    pub velocity_scale: f32,
}

impl Default for ParticleRenderView {
    fn default() -> Self {
        Self {
            display_mode: ParticleDisplayMode::Points,
            particle_count: 0,
            display_percentage: 100.0,
            show_velocity: false,
            velocity_scale: 1.0,
        }
    }
}

/// Create a new ParticleRenderView.
pub fn new_particle_render_view() -> ParticleRenderView {
    ParticleRenderView::default()
}

/// Set display percentage (0.0–100.0).
pub fn particle_render_set_percentage(view: &mut ParticleRenderView, pct: f32) {
    view.display_percentage = pct.clamp(0.0, 100.0);
}

/// Set display mode.
pub fn particle_render_set_mode(view: &mut ParticleRenderView, mode: ParticleDisplayMode) {
    view.display_mode = mode;
}

/// Toggle velocity vector display.
pub fn particle_render_show_velocity(view: &mut ParticleRenderView, show: bool) {
    view.show_velocity = show;
}

/// Set velocity arrow scale.
pub fn particle_render_set_velocity_scale(view: &mut ParticleRenderView, scale: f32) {
    view.velocity_scale = scale.clamp(0.0, 100.0);
}

/// Compute displayed particle count based on percentage.
pub fn particle_render_displayed_count(view: &ParticleRenderView) -> u32 {
    ((view.particle_count as f32) * view.display_percentage / 100.0) as u32
}

/// Serialize to JSON.
pub fn particle_render_to_json(view: &ParticleRenderView) -> String {
    format!(
        r#"{{"count":{},"pct":{},"show_vel":{},"vel_scale":{}}}"#,
        view.particle_count, view.display_percentage, view.show_velocity, view.velocity_scale,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let v = new_particle_render_view();
        assert_eq!(v.particle_count, 0 /* default */);
    }

    #[test]
    fn test_pct_clamp_high() {
        let mut v = new_particle_render_view();
        particle_render_set_percentage(&mut v, 200.0);
        assert!((v.display_percentage - 100.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_pct_clamp_low() {
        let mut v = new_particle_render_view();
        particle_render_set_percentage(&mut v, -10.0);
        assert!((v.display_percentage - 0.0).abs() < 1e-6 /* clamped to 0 */);
    }

    #[test]
    fn test_set_mode_circles() {
        let mut v = new_particle_render_view();
        particle_render_set_mode(&mut v, ParticleDisplayMode::Circles);
        assert_eq!(
            v.display_mode,
            ParticleDisplayMode::Circles /* stored */
        );
    }

    #[test]
    fn test_show_velocity() {
        let mut v = new_particle_render_view();
        particle_render_show_velocity(&mut v, true);
        assert!(v.show_velocity /* enabled */);
    }

    #[test]
    fn test_velocity_scale() {
        let mut v = new_particle_render_view();
        particle_render_set_velocity_scale(&mut v, 2.5);
        assert!((v.velocity_scale - 2.5).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_displayed_count_half() {
        let mut v = new_particle_render_view();
        v.particle_count = 1000;
        particle_render_set_percentage(&mut v, 50.0);
        assert_eq!(particle_render_displayed_count(&v), 500 /* half */);
    }

    #[test]
    fn test_displayed_count_zero_pct() {
        let mut v = new_particle_render_view();
        v.particle_count = 1000;
        particle_render_set_percentage(&mut v, 0.0);
        assert_eq!(particle_render_displayed_count(&v), 0 /* none shown */);
    }

    #[test]
    fn test_json_keys() {
        let v = new_particle_render_view();
        let j = particle_render_to_json(&v);
        assert!(j.contains("count") /* key */);
    }
}
