// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Light cookie — projects a texture mask through a spotlight or directional light.

use std::f32::consts::FRAC_PI_4;

/// Cookie projection type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum CookieProjection {
    Spot,
    Directional,
    Point,
}

/// Light cookie descriptor.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct LightCookie {
    pub texture_id: u32,
    pub projection: CookieProjection,
    pub intensity: f32,
    pub tiling: [f32; 2],
    pub offset: [f32; 2],
    pub angle_deg: f32,
    pub enabled: bool,
}

impl Default for LightCookie {
    fn default() -> Self {
        Self {
            texture_id: 0,
            projection: CookieProjection::Spot,
            intensity: 1.0,
            tiling: [1.0, 1.0],
            offset: [0.0, 0.0],
            angle_deg: 45.0,
            enabled: true,
        }
    }
}

/// Cookie manager.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct LightCookieManager {
    pub cookies: Vec<LightCookie>,
    pub max_cookies: usize,
}

/// Create new manager.
#[allow(dead_code)]
pub fn new_cookie_manager(max_cookies: usize) -> LightCookieManager {
    LightCookieManager {
        cookies: Vec::new(),
        max_cookies,
    }
}

/// Add a cookie.
#[allow(dead_code)]
pub fn add_cookie(m: &mut LightCookieManager, cookie: LightCookie) -> Option<usize> {
    if m.cookies.len() >= m.max_cookies {
        return None;
    }
    let idx = m.cookies.len();
    m.cookies.push(cookie);
    Some(idx)
}

/// Remove cookie by index.
#[allow(dead_code)]
pub fn remove_cookie(m: &mut LightCookieManager, idx: usize) {
    if idx < m.cookies.len() {
        m.cookies.remove(idx);
    }
}

/// Cookie count.
#[allow(dead_code)]
pub fn cookie_count(m: &LightCookieManager) -> usize {
    m.cookies.len()
}

/// Enabled cookie count.
#[allow(dead_code)]
pub fn enabled_cookie_count(m: &LightCookieManager) -> usize {
    m.cookies.iter().filter(|c| c.enabled).count()
}

/// Compute cone half-angle in radians from degree field.
#[allow(dead_code)]
pub fn cone_half_angle_rad(cookie: &LightCookie) -> f32 {
    cookie.angle_deg.to_radians() * 0.5
}

/// UV transform for spot projection using FRAC_PI_4 as reference half-angle.
#[allow(dead_code)]
pub fn spot_uv_scale(cookie: &LightCookie) -> f32 {
    let half = cone_half_angle_rad(cookie);
    // normalize so 45-degree cone maps to 1.0
    half / FRAC_PI_4
}

/// Sample cookie attenuation for a given UV.
#[allow(dead_code)]
pub fn sample_cookie_uv(uv: [f32; 2], cookie: &LightCookie) -> f32 {
    if !cookie.enabled {
        return 1.0;
    }
    let u = uv[0] * cookie.tiling[0] + cookie.offset[0];
    let v = uv[1] * cookie.tiling[1] + cookie.offset[1];
    // bilinear stub: simple checkerboard pattern for testing
    let inside = (0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v);
    if inside {
        cookie.intensity
    } else {
        0.0
    }
}

/// Export to JSON-like string.
#[allow(dead_code)]
pub fn cookie_to_json(c: &LightCookie) -> String {
    format!(
        r#"{{"texture_id":{},"angle_deg":{:.2},"enabled":{}}}"#,
        c.texture_id, c.angle_deg, c.enabled
    )
}

/// Clear all cookies.
#[allow(dead_code)]
pub fn clear_cookies(m: &mut LightCookieManager) {
    m.cookies.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_manager_empty() {
        let m = new_cookie_manager(16);
        assert_eq!(cookie_count(&m), 0);
    }

    #[test]
    fn add_cookie_ok() {
        let mut m = new_cookie_manager(16);
        assert!(add_cookie(&mut m, LightCookie::default()).is_some());
    }

    #[test]
    fn capacity_limit() {
        let mut m = new_cookie_manager(1);
        add_cookie(&mut m, LightCookie::default());
        assert!(add_cookie(&mut m, LightCookie::default()).is_none());
    }

    #[test]
    fn remove_cookie_clears() {
        let mut m = new_cookie_manager(16);
        add_cookie(&mut m, LightCookie::default());
        remove_cookie(&mut m, 0);
        assert_eq!(cookie_count(&m), 0);
    }

    #[test]
    fn enabled_count() {
        let mut m = new_cookie_manager(16);
        add_cookie(
            &mut m,
            LightCookie {
                enabled: true,
                ..Default::default()
            },
        );
        add_cookie(
            &mut m,
            LightCookie {
                enabled: false,
                ..Default::default()
            },
        );
        assert_eq!(enabled_cookie_count(&m), 1);
    }

    #[test]
    fn cone_half_angle_45deg() {
        let c = LightCookie {
            angle_deg: 90.0,
            ..Default::default()
        };
        let h = cone_half_angle_rad(&c);
        assert!((h - FRAC_PI_4).abs() < 1e-5);
    }

    #[test]
    fn spot_uv_scale_identity() {
        let c = LightCookie {
            angle_deg: 90.0,
            ..Default::default()
        };
        let scale = spot_uv_scale(&c);
        assert!((scale - 1.0).abs() < 1e-5);
    }

    #[test]
    fn sample_inside_returns_intensity() {
        let c = LightCookie {
            intensity: 0.7,
            ..Default::default()
        };
        let v = sample_cookie_uv([0.5, 0.5], &c);
        assert!((v - 0.7).abs() < 1e-6);
    }

    #[test]
    fn sample_outside_returns_zero() {
        let c = LightCookie::default();
        let v = sample_cookie_uv([1.5, 0.5], &c);
        assert!(v.abs() < 1e-6);
    }

    #[test]
    fn json_contains_texture_id() {
        let c = LightCookie {
            texture_id: 5,
            ..Default::default()
        };
        assert!(cookie_to_json(&c).contains("5"));
    }
}
