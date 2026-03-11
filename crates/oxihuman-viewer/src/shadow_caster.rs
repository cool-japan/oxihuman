#![allow(dead_code)]
//! Shadow casting configuration and stub rendering.

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ShadowConfig {
    pub map_size: u32,
    pub bias: f32,
    pub near: f32,
    pub far: f32,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ShadowCaster {
    config: ShadowConfig,
    cast_count: u32,
}

#[allow(dead_code)]
pub fn new_shadow_caster(config: ShadowConfig) -> ShadowCaster {
    ShadowCaster {
        config,
        cast_count: 0,
    }
}

#[allow(dead_code)]
pub fn default_shadow_config() -> ShadowConfig {
    ShadowConfig {
        map_size: 2048,
        bias: 0.005,
        near: 0.1,
        far: 100.0,
    }
}

#[allow(dead_code)]
pub fn shadow_map_size(sc: &ShadowCaster) -> u32 {
    sc.config.map_size
}

#[allow(dead_code)]
pub fn shadow_bias(sc: &ShadowCaster) -> f32 {
    sc.config.bias
}

#[allow(dead_code)]
pub fn shadow_near(sc: &ShadowCaster) -> f32 {
    sc.config.near
}

#[allow(dead_code)]
pub fn shadow_far(sc: &ShadowCaster) -> f32 {
    sc.config.far
}

#[allow(dead_code)]
pub fn cast_shadow_stub(sc: &mut ShadowCaster) -> u32 {
    sc.cast_count += 1;
    sc.cast_count
}

#[allow(dead_code)]
pub fn shadow_to_json(sc: &ShadowCaster) -> String {
    format!(
        "{{\"map_size\":{},\"bias\":{},\"near\":{},\"far\":{},\"casts\":{}}}",
        sc.config.map_size, sc.config.bias, sc.config.near, sc.config.far, sc.cast_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_shadow_caster() {
        let sc = new_shadow_caster(default_shadow_config());
        assert_eq!(shadow_map_size(&sc), 2048);
    }

    #[test]
    fn test_default_shadow_config() {
        let c = default_shadow_config();
        assert_eq!(c.map_size, 2048);
        assert!((c.bias - 0.005).abs() < 1e-6);
    }

    #[test]
    fn test_shadow_map_size() {
        let sc = new_shadow_caster(ShadowConfig {
            map_size: 4096,
            bias: 0.01,
            near: 0.5,
            far: 200.0,
        });
        assert_eq!(shadow_map_size(&sc), 4096);
    }

    #[test]
    fn test_shadow_bias() {
        let sc = new_shadow_caster(default_shadow_config());
        assert!((shadow_bias(&sc) - 0.005).abs() < 1e-6);
    }

    #[test]
    fn test_shadow_near() {
        let sc = new_shadow_caster(default_shadow_config());
        assert!((shadow_near(&sc) - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_shadow_far() {
        let sc = new_shadow_caster(default_shadow_config());
        assert!((shadow_far(&sc) - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_cast_shadow_stub() {
        let mut sc = new_shadow_caster(default_shadow_config());
        assert_eq!(cast_shadow_stub(&mut sc), 1);
        assert_eq!(cast_shadow_stub(&mut sc), 2);
    }

    #[test]
    fn test_shadow_to_json() {
        let sc = new_shadow_caster(default_shadow_config());
        let json = shadow_to_json(&sc);
        assert!(json.contains("\"map_size\":2048"));
    }

    #[test]
    fn test_custom_config() {
        let cfg = ShadowConfig {
            map_size: 1024,
            bias: 0.001,
            near: 1.0,
            far: 50.0,
        };
        let sc = new_shadow_caster(cfg);
        assert_eq!(shadow_map_size(&sc), 1024);
        assert!((shadow_far(&sc) - 50.0).abs() < 1e-6);
    }

    #[test]
    fn test_initial_cast_count() {
        let sc = new_shadow_caster(default_shadow_config());
        let json = shadow_to_json(&sc);
        assert!(json.contains("\"casts\":0"));
    }
}
