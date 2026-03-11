//! Nose bridge width and height morphology controls.

#[allow(dead_code)]
pub struct NoseBridgeConfig {
    pub bridge_width: f32,
    pub bridge_height: f32,
    pub root_depth: f32,
    pub dorsal_hump: f32,
}

#[allow(dead_code)]
pub struct NoseBridgeState {
    pub width: f32,
    pub height: f32,
    pub root_depth: f32,
    pub hump: f32,
    pub saddle: f32,
}

#[allow(dead_code)]
pub struct NoseBridgeWeights {
    pub wide: f32,
    pub narrow: f32,
    pub high: f32,
    pub low: f32,
    pub humped: f32,
    pub saddled: f32,
}

#[allow(dead_code)]
pub fn default_nose_bridge_config() -> NoseBridgeConfig {
    NoseBridgeConfig {
        bridge_width: 0.5,
        bridge_height: 0.5,
        root_depth: 0.3,
        dorsal_hump: 0.0,
    }
}

#[allow(dead_code)]
pub fn new_nose_bridge_state() -> NoseBridgeState {
    NoseBridgeState {
        width: 0.5,
        height: 0.5,
        root_depth: 0.3,
        hump: 0.0,
        saddle: 0.0,
    }
}

#[allow(dead_code)]
pub fn set_bridge_width(state: &mut NoseBridgeState, width: f32) {
    state.width = width.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_bridge_height(state: &mut NoseBridgeState, height: f32) {
    state.height = height.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_dorsal_hump(state: &mut NoseBridgeState, hump: f32) {
    state.hump = hump.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_saddle_nose(state: &mut NoseBridgeState, saddle: f32) {
    state.saddle = saddle.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_nose_bridge_weights(
    state: &NoseBridgeState,
    cfg: &NoseBridgeConfig,
) -> NoseBridgeWeights {
    let pivot = cfg.bridge_width.clamp(0.0, 1.0);
    let wide = if state.width > pivot {
        ((state.width - pivot) / (1.0 - pivot + 1e-6)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let narrow = if state.width < pivot {
        ((pivot - state.width) / (pivot + 1e-6)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let h_pivot = cfg.bridge_height.clamp(0.0, 1.0);
    let high = if state.height > h_pivot {
        ((state.height - h_pivot) / (1.0 - h_pivot + 1e-6)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let low = if state.height < h_pivot {
        ((h_pivot - state.height) / (h_pivot + 1e-6)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let hump_scale = cfg.dorsal_hump.clamp(0.0, 1.0) + 0.5;
    let humped = (state.hump * hump_scale).clamp(0.0, 1.0);
    let saddled = state.saddle.clamp(0.0, 1.0);
    NoseBridgeWeights { wide, narrow, high, low, humped, saddled }
}

#[allow(dead_code)]
pub fn blend_nose_bridge(
    a: &NoseBridgeState,
    b: &NoseBridgeState,
    t: f32,
) -> NoseBridgeState {
    let t = t.clamp(0.0, 1.0);
    let u = 1.0 - t;
    NoseBridgeState {
        width: a.width * u + b.width * t,
        height: a.height * u + b.height * t,
        root_depth: a.root_depth * u + b.root_depth * t,
        hump: a.hump * u + b.hump * t,
        saddle: a.saddle * u + b.saddle * t,
    }
}

#[allow(dead_code)]
pub fn reset_nose_bridge(state: &mut NoseBridgeState) {
    state.width = 0.5;
    state.height = 0.5;
    state.root_depth = 0.3;
    state.hump = 0.0;
    state.saddle = 0.0;
}

#[allow(dead_code)]
pub fn nose_bridge_to_json(state: &NoseBridgeState) -> String {
    format!(
        r#"{{"width":{:.4},"height":{:.4},"root_depth":{:.4},"hump":{:.4},"saddle":{:.4}}}"#,
        state.width, state.height, state.root_depth, state.hump, state.saddle,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_nose_bridge_config();
        assert!((cfg.bridge_width - 0.5).abs() < 1e-5);
        assert!((cfg.bridge_height - 0.5).abs() < 1e-5);
        assert!(cfg.dorsal_hump.abs() < 1e-5);
    }

    #[test]
    fn test_new_state() {
        let s = new_nose_bridge_state();
        assert!((s.width - 0.5).abs() < 1e-5);
        assert!((s.height - 0.5).abs() < 1e-5);
        assert!(s.hump.abs() < 1e-5);
        assert!(s.saddle.abs() < 1e-5);
    }

    #[test]
    fn test_set_bridge_width() {
        let mut s = new_nose_bridge_state();
        set_bridge_width(&mut s, 0.8);
        assert!((s.width - 0.8).abs() < 1e-5);
        set_bridge_width(&mut s, 2.0);
        assert!((s.width - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_set_bridge_height() {
        let mut s = new_nose_bridge_state();
        set_bridge_height(&mut s, 0.2);
        assert!((s.height - 0.2).abs() < 1e-5);
        set_bridge_height(&mut s, -1.0);
        assert!(s.height.abs() < 1e-5);
    }

    #[test]
    fn test_set_dorsal_hump() {
        let mut s = new_nose_bridge_state();
        set_dorsal_hump(&mut s, 0.7);
        assert!((s.hump - 0.7).abs() < 1e-5);
    }

    #[test]
    fn test_set_saddle_nose() {
        let mut s = new_nose_bridge_state();
        set_saddle_nose(&mut s, 0.4);
        assert!((s.saddle - 0.4).abs() < 1e-5);
    }

    #[test]
    fn test_compute_weights_wide() {
        let mut s = new_nose_bridge_state();
        let cfg = default_nose_bridge_config();
        set_bridge_width(&mut s, 1.0);
        let w = compute_nose_bridge_weights(&s, &cfg);
        assert!(w.wide > 0.0);
        assert!(w.narrow < 1e-5);
    }

    #[test]
    fn test_compute_weights_narrow() {
        let mut s = new_nose_bridge_state();
        let cfg = default_nose_bridge_config();
        set_bridge_width(&mut s, 0.0);
        let w = compute_nose_bridge_weights(&s, &cfg);
        assert!(w.narrow > 0.0);
        assert!(w.wide < 1e-5);
    }

    #[test]
    fn test_blend_nose_bridge() {
        let a = new_nose_bridge_state();
        let mut b = new_nose_bridge_state();
        b.width = 1.0;
        let mid = blend_nose_bridge(&a, &b, 0.5);
        assert!((mid.width - 0.75).abs() < 1e-5);
    }

    #[test]
    fn test_reset_nose_bridge() {
        let mut s = new_nose_bridge_state();
        set_dorsal_hump(&mut s, 1.0);
        set_saddle_nose(&mut s, 1.0);
        reset_nose_bridge(&mut s);
        assert!(s.hump.abs() < 1e-5);
        assert!(s.saddle.abs() < 1e-5);
        assert!((s.width - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_nose_bridge_to_json() {
        let s = new_nose_bridge_state();
        let json = nose_bridge_to_json(&s);
        assert!(json.contains("width"));
        assert!(json.contains("hump"));
        assert!(json.contains("saddle"));
    }
}
