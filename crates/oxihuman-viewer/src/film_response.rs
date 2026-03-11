// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Film response — filmic tone-mapping curves and color response emulation.

use std::f32::consts::E;

/// Available filmic tone mapping operators.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilmicOperator {
    Reinhard,
    AcesFilm,
    HejlBurgessDawson,
    Uncharted2,
    Linear,
}

/// Film response configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FilmResponseConfig {
    pub operator: FilmicOperator,
    pub exposure: f32,
    pub gamma: f32,
}

#[allow(dead_code)]
pub fn default_film_response() -> FilmResponseConfig {
    FilmResponseConfig {
        operator: FilmicOperator::AcesFilm,
        exposure: 1.0,
        gamma: 2.2,
    }
}

#[allow(dead_code)]
pub fn fr_set_exposure(cfg: &mut FilmResponseConfig, v: f32) {
    cfg.exposure = v.clamp(0.01, 100.0);
}

#[allow(dead_code)]
pub fn fr_set_gamma(cfg: &mut FilmResponseConfig, v: f32) {
    cfg.gamma = v.clamp(1.0, 3.0);
}

#[allow(dead_code)]
pub fn fr_set_operator(cfg: &mut FilmResponseConfig, op: FilmicOperator) {
    cfg.operator = op;
}

#[allow(dead_code)]
pub fn fr_reinhard(x: f32) -> f32 {
    x / (1.0 + x)
}

#[allow(dead_code)]
pub fn fr_aces(x: f32) -> f32 {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    ((x * (a * x + b)) / (x * (c * x + d) + e)).clamp(0.0, 1.0)
}

/// Hejl/Burgess-Dawson filmic curve.
#[allow(dead_code)]
pub fn fr_hejl(x: f32) -> f32 {
    let x = (x - 0.004).max(0.0);
    (x * (6.2 * x + 0.5)) / (x * (6.2 * x + 1.7) + 0.06)
}

#[allow(dead_code)]
pub fn fr_apply_exposure(cfg: &FilmResponseConfig, linear: f32) -> f32 {
    linear * cfg.exposure
}

#[allow(dead_code)]
pub fn fr_tonemap(cfg: &FilmResponseConfig, linear: f32) -> f32 {
    let v = linear * cfg.exposure;
    match cfg.operator {
        FilmicOperator::Reinhard => fr_reinhard(v),
        FilmicOperator::AcesFilm => fr_aces(v),
        FilmicOperator::HejlBurgessDawson => fr_hejl(v),
        FilmicOperator::Uncharted2 => fr_reinhard(v * 0.9),
        FilmicOperator::Linear => v.clamp(0.0, 1.0),
    }
}

#[allow(dead_code)]
pub fn fr_gamma_correct(v: f32, gamma: f32) -> f32 {
    v.clamp(0.0, 1.0).powf(1.0 / gamma)
}

#[allow(dead_code)]
pub fn fr_process(cfg: &FilmResponseConfig, linear: f32) -> f32 {
    let mapped = fr_tonemap(cfg, linear);
    fr_gamma_correct(mapped, cfg.gamma)
}

#[allow(dead_code)]
pub fn fr_ev_stops(cfg: &FilmResponseConfig) -> f32 {
    cfg.exposure.log2()
}

#[allow(dead_code)]
pub fn fr_to_json(cfg: &FilmResponseConfig) -> String {
    let op = match cfg.operator {
        FilmicOperator::Reinhard => "reinhard",
        FilmicOperator::AcesFilm => "aces",
        FilmicOperator::HejlBurgessDawson => "hejl",
        FilmicOperator::Uncharted2 => "uncharted2",
        FilmicOperator::Linear => "linear",
    };
    format!(
        r#"{{"operator":"{}","exposure":{:.4},"gamma":{:.4}}}"#,
        op, cfg.exposure, cfg.gamma
    )
}

/// Suppress unused import warning on `E`.
fn _use_e() -> f32 {
    E
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_aces() {
        let cfg = default_film_response();
        assert_eq!(cfg.operator, FilmicOperator::AcesFilm);
    }

    #[test]
    fn set_exposure_clamps() {
        let mut cfg = default_film_response();
        fr_set_exposure(&mut cfg, 0.0);
        assert!(cfg.exposure > 0.0);
    }

    #[test]
    fn reinhard_half_at_one() {
        assert!((fr_reinhard(1.0) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn aces_range() {
        let v = fr_aces(1.0);
        assert!((0.0..=1.0).contains(&v));
    }

    #[test]
    fn tonemap_linear_clamps_at_exposure() {
        let mut cfg = default_film_response();
        fr_set_operator(&mut cfg, FilmicOperator::Linear);
        fr_set_exposure(&mut cfg, 1.0);
        let v = fr_tonemap(&cfg, 2.0);
        assert!((v - 1.0).abs() < 1e-6);
    }

    #[test]
    fn gamma_correct_identity_at_one() {
        assert!((fr_gamma_correct(1.0, 1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn process_dark_stays_dark() {
        let cfg = default_film_response();
        let out = fr_process(&cfg, 0.0);
        assert!(out < 0.01);
    }

    #[test]
    fn ev_stops_at_one_exposure() {
        let mut cfg = default_film_response();
        fr_set_exposure(&mut cfg, 1.0);
        assert!(fr_ev_stops(&cfg).abs() < 1e-5);
    }

    #[test]
    fn to_json_fields() {
        let cfg = default_film_response();
        let j = fr_to_json(&cfg);
        assert!(j.contains("operator"));
        assert!(j.contains("gamma"));
    }
}
