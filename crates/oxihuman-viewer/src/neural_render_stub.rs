// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub struct NeuralRenderConfig {
    pub latent_dim: usize,
    pub resolution: u32,
    pub num_layers: usize,
}

pub fn new_neural_render_config(res: u32) -> NeuralRenderConfig {
    NeuralRenderConfig {
        latent_dim: 64,
        resolution: res,
        num_layers: 8,
    }
}

pub fn neural_render_param_count(cfg: &NeuralRenderConfig) -> usize {
    cfg.latent_dim * cfg.num_layers * (cfg.resolution as usize)
}

pub fn neural_render_memory_mb(cfg: &NeuralRenderConfig) -> f32 {
    (neural_render_param_count(cfg) * 4) as f32 / (1024.0 * 1024.0)
}

pub fn neural_render_latent_size(cfg: &NeuralRenderConfig) -> usize {
    cfg.latent_dim
}

pub fn neural_render_is_valid(cfg: &NeuralRenderConfig) -> bool {
    cfg.latent_dim > 0 && cfg.resolution > 0 && cfg.num_layers > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_config() {
        /* resolution is set correctly */
        let cfg = new_neural_render_config(256);
        assert_eq!(cfg.resolution, 256);
    }

    #[test]
    fn test_is_valid() {
        /* default config is valid */
        let cfg = new_neural_render_config(128);
        assert!(neural_render_is_valid(&cfg));
    }

    #[test]
    fn test_latent_size() {
        /* latent size returns latent_dim */
        let cfg = new_neural_render_config(64);
        assert_eq!(neural_render_latent_size(&cfg), cfg.latent_dim);
    }

    #[test]
    fn test_param_count_positive() {
        /* param count is positive */
        let cfg = new_neural_render_config(64);
        assert!(neural_render_param_count(&cfg) > 0);
    }

    #[test]
    fn test_memory_mb_positive() {
        /* memory is positive */
        let cfg = new_neural_render_config(64);
        assert!(neural_render_memory_mb(&cfg) > 0.0);
    }
}
