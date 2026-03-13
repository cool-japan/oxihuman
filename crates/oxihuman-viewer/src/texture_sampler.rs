// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Texture wrap mode.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum WrapMode {
    Repeat,
    ClampToEdge,
    MirroredRepeat,
}

/// Texture filter mode.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum FilterMode {
    Nearest,
    Linear,
    Mipmap,
}

/// Texture sampler state descriptor.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TextureSampler {
    pub wrap_s: WrapMode,
    pub wrap_t: WrapMode,
    pub min_filter: FilterMode,
    pub mag_filter: FilterMode,
    pub anisotropy: f32,
}

/// Create a default sampler (linear, clamp, no anisotropy).
#[allow(dead_code)]
pub fn default_texture_sampler() -> TextureSampler {
    TextureSampler {
        wrap_s: WrapMode::ClampToEdge,
        wrap_t: WrapMode::ClampToEdge,
        min_filter: FilterMode::Linear,
        mag_filter: FilterMode::Linear,
        anisotropy: 1.0,
    }
}

/// Create a linear sampler with repeat wrap.
#[allow(dead_code)]
pub fn linear_sampler() -> TextureSampler {
    TextureSampler {
        wrap_s: WrapMode::Repeat,
        wrap_t: WrapMode::Repeat,
        min_filter: FilterMode::Linear,
        mag_filter: FilterMode::Linear,
        anisotropy: 1.0,
    }
}

/// Create a nearest-neighbor sampler.
#[allow(dead_code)]
pub fn nearest_sampler() -> TextureSampler {
    TextureSampler {
        wrap_s: WrapMode::ClampToEdge,
        wrap_t: WrapMode::ClampToEdge,
        min_filter: FilterMode::Nearest,
        mag_filter: FilterMode::Nearest,
        anisotropy: 1.0,
    }
}

/// Create a sampler with the given anisotropy level.
#[allow(dead_code)]
pub fn sampler_aniso(level: f32) -> TextureSampler {
    TextureSampler {
        wrap_s: WrapMode::Repeat,
        wrap_t: WrapMode::Repeat,
        min_filter: FilterMode::Mipmap,
        mag_filter: FilterMode::Linear,
        anisotropy: level.max(1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_sampler_clamp() {
        let s = default_texture_sampler();
        assert_eq!(s.wrap_s, WrapMode::ClampToEdge);
        assert_eq!(s.wrap_t, WrapMode::ClampToEdge);
    }

    #[test]
    fn default_sampler_linear_filter() {
        let s = default_texture_sampler();
        assert_eq!(s.min_filter, FilterMode::Linear);
        assert_eq!(s.mag_filter, FilterMode::Linear);
    }

    #[test]
    fn default_sampler_anisotropy_one() {
        let s = default_texture_sampler();
        assert!((s.anisotropy - 1.0).abs() < 1e-6);
    }

    #[test]
    fn linear_sampler_repeat_wrap() {
        let s = linear_sampler();
        assert_eq!(s.wrap_s, WrapMode::Repeat);
        assert_eq!(s.wrap_t, WrapMode::Repeat);
    }

    #[test]
    fn nearest_sampler_nearest_filter() {
        let s = nearest_sampler();
        assert_eq!(s.min_filter, FilterMode::Nearest);
        assert_eq!(s.mag_filter, FilterMode::Nearest);
    }

    #[test]
    fn sampler_aniso_mipmap_filter() {
        let s = sampler_aniso(8.0);
        assert_eq!(s.min_filter, FilterMode::Mipmap);
    }

    #[test]
    fn sampler_aniso_level_stored() {
        let s = sampler_aniso(16.0);
        assert!((s.anisotropy - 16.0).abs() < 1e-5);
    }

    #[test]
    fn sampler_aniso_clamps_to_one() {
        let s = sampler_aniso(0.0);
        assert!(s.anisotropy >= 1.0);
    }

    #[test]
    fn nearest_sampler_clamp_wrap() {
        let s = nearest_sampler();
        assert_eq!(s.wrap_s, WrapMode::ClampToEdge);
    }

    #[test]
    fn linear_sampler_linear_filters() {
        let s = linear_sampler();
        assert_eq!(s.min_filter, FilterMode::Linear);
    }
}
