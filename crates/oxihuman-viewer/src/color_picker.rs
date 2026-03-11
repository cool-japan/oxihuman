//! Color picker widget state and color space conversions.

// ── Legacy API (ColorPickerState / ColorSpace) ─────────────────────────────

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ColorSpace {
    Rgb,
    Hsv,
    Hsl,
    Linear,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct ColorPickerState {
    pub rgb: [f32; 4], // RGBA [0,1]
    pub hsv: [f32; 3], // HSV [0-360, 0-1, 0-1]
    pub hex: u32,      // 0xRRGGBBAA
    pub mode: ColorSpace,
    pub alpha_enabled: bool,
}

/// Internal helper: rgb -> [h, s, v] array form.
fn rgb_to_hsv_arr(r: f32, g: f32, b: f32) -> [f32; 3] {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let v = max;
    let s = if max > 1e-6 { delta / max } else { 0.0 };
    let h = if delta < 1e-6 {
        0.0
    } else if (max - r).abs() < 1e-6 {
        60.0 * (((g - b) / delta) % 6.0)
    } else if (max - g).abs() < 1e-6 {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    let h = if h < 0.0 { h + 360.0 } else { h };
    [h, s, v]
}

/// Internal helper: [h,s,v] -> [r,g,b].
fn hsv_arr_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    if s < 1e-6 {
        return [v, v, v];
    }
    let hh = (h % 360.0) / 60.0;
    let i = hh.floor() as i32;
    let f = hh - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    match i % 6 {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        _ => [v, p, q],
    }
}

#[allow(dead_code)]
pub fn new_color_picker(r: f32, g: f32, b: f32, a: f32) -> ColorPickerState {
    let hsv = rgb_to_hsv_arr(r, g, b);
    let hex = rgb_to_hex(r, g, b, a);
    ColorPickerState {
        rgb: [r, g, b, a],
        hsv,
        hex,
        mode: ColorSpace::Rgb,
        alpha_enabled: true,
    }
}

#[allow(dead_code)]
pub fn rgb_to_hex(r: f32, g: f32, b: f32, a: f32) -> u32 {
    let ri = (r.clamp(0.0, 1.0) * 255.0).round() as u32;
    let gi = (g.clamp(0.0, 1.0) * 255.0).round() as u32;
    let bi = (b.clamp(0.0, 1.0) * 255.0).round() as u32;
    let ai = (a.clamp(0.0, 1.0) * 255.0).round() as u32;
    (ri << 24) | (gi << 16) | (bi << 8) | ai
}

#[allow(dead_code)]
pub fn hex_to_rgb(hex: u32) -> [f32; 4] {
    let r = ((hex >> 24) & 0xFF) as f32 / 255.0;
    let g = ((hex >> 16) & 0xFF) as f32 / 255.0;
    let b = ((hex >> 8) & 0xFF) as f32 / 255.0;
    let a = (hex & 0xFF) as f32 / 255.0;
    [r, g, b, a]
}

/// sRGB gamma decode: x^2.2
#[allow(dead_code)]
pub fn rgb_to_linear(r: f32, g: f32, b: f32) -> [f32; 3] {
    [r.powf(2.2), g.powf(2.2), b.powf(2.2)]
}

/// Gamma encode: x^(1/2.2)
#[allow(dead_code)]
pub fn linear_to_rgb(r: f32, g: f32, b: f32) -> [f32; 3] {
    let inv_gamma = 1.0 / 2.2;
    [r.powf(inv_gamma), g.powf(inv_gamma), b.powf(inv_gamma)]
}

#[allow(dead_code)]
pub fn set_rgb(picker: &mut ColorPickerState, r: f32, g: f32, b: f32) {
    picker.rgb[0] = r;
    picker.rgb[1] = g;
    picker.rgb[2] = b;
    picker.hsv = rgb_to_hsv_arr(r, g, b);
    picker.hex = rgb_to_hex(r, g, b, picker.rgb[3]);
}

#[allow(dead_code)]
pub fn set_hsv(picker: &mut ColorPickerState, h: f32, s: f32, v: f32) {
    picker.hsv = [h, s, v];
    let rgb = hsv_arr_to_rgb(h, s, v);
    picker.rgb[0] = rgb[0];
    picker.rgb[1] = rgb[1];
    picker.rgb[2] = rgb[2];
    picker.hex = rgb_to_hex(rgb[0], rgb[1], rgb[2], picker.rgb[3]);
}

#[allow(dead_code)]
pub fn set_alpha(picker: &mut ColorPickerState, a: f32) {
    picker.rgb[3] = a.clamp(0.0, 1.0);
    picker.hex = rgb_to_hex(picker.rgb[0], picker.rgb[1], picker.rgb[2], picker.rgb[3]);
}

#[allow(dead_code)]
pub fn set_hex(picker: &mut ColorPickerState, hex: u32) {
    picker.hex = hex;
    let rgba = hex_to_rgb(hex);
    picker.rgb = rgba;
    picker.hsv = rgb_to_hsv_arr(rgba[0], rgba[1], rgba[2]);
}

#[allow(dead_code)]
pub fn lerp_color(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

#[allow(dead_code)]
pub fn color_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dr = a[0] - b[0];
    let dg = a[1] - b[1];
    let db = a[2] - b[2];
    (dr * dr + dg * dg + db * db).sqrt()
}

/// Rotate hue 180 degrees.
#[allow(dead_code)]
pub fn complementary_color(rgb: [f32; 3]) -> [f32; 3] {
    let [h, s, v] = rgb_to_hsv_arr(rgb[0], rgb[1], rgb[2]);
    let h2 = (h + 180.0) % 360.0;
    hsv_arr_to_rgb(h2, s, v)
}

// ── Spec API: ColorPickerConfig / HsvColor / ColorPickerStateNew ───────────

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ColorPickerConfig {
    pub show_alpha: bool,
    pub hdr: bool,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HsvColor {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}

/// Typed picker state for the spec API.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ColorPickerStateNew {
    pub config: ColorPickerConfig,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    pub hsv: HsvColor,
}

#[allow(dead_code)]
pub fn default_color_picker_config() -> ColorPickerConfig {
    ColorPickerConfig {
        show_alpha: true,
        hdr: false,
    }
}

#[allow(dead_code)]
pub fn new_color_picker_state(cfg: &ColorPickerConfig) -> ColorPickerStateNew {
    ColorPickerStateNew {
        config: cfg.clone(),
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
        hsv: HsvColor {
            h: 0.0,
            s: 0.0,
            v: 1.0,
        },
    }
}

#[allow(dead_code)]
pub fn hsv_to_rgb(hsv: HsvColor) -> [f32; 3] {
    hsv_arr_to_rgb(hsv.h, hsv.s, hsv.v)
}

#[allow(dead_code)]
pub fn rgb_to_hsv(r: f32, g: f32, b: f32) -> HsvColor {
    let arr = rgb_to_hsv_arr(r, g, b);
    HsvColor {
        h: arr[0],
        s: arr[1],
        v: arr[2],
    }
}

#[allow(dead_code)]
pub fn color_picker_set_rgb(state: &mut ColorPickerStateNew, r: f32, g: f32, b: f32) {
    state.r = r.clamp(0.0, 1.0);
    state.g = g.clamp(0.0, 1.0);
    state.b = b.clamp(0.0, 1.0);
    let arr = rgb_to_hsv_arr(state.r, state.g, state.b);
    state.hsv = HsvColor {
        h: arr[0],
        s: arr[1],
        v: arr[2],
    };
}

#[allow(dead_code)]
pub fn color_picker_set_hsv(state: &mut ColorPickerStateNew, h: f32, s: f32, v: f32) {
    state.hsv = HsvColor { h, s, v };
    let rgb = hsv_arr_to_rgb(h, s, v);
    state.r = rgb[0];
    state.g = rgb[1];
    state.b = rgb[2];
}

#[allow(dead_code)]
pub fn color_picker_rgb(state: &ColorPickerStateNew) -> [f32; 3] {
    [state.r, state.g, state.b]
}

#[allow(dead_code)]
pub fn color_picker_hsv(state: &ColorPickerStateNew) -> HsvColor {
    state.hsv
}

#[allow(dead_code)]
pub fn color_picker_hex_string(state: &ColorPickerStateNew) -> String {
    let r = (state.r.clamp(0.0, 1.0) * 255.0).round() as u8;
    let g = (state.g.clamp(0.0, 1.0) * 255.0).round() as u8;
    let b = (state.b.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

#[allow(dead_code)]
pub fn color_picker_reset(state: &mut ColorPickerStateNew) {
    state.r = 1.0;
    state.g = 1.0;
    state.b = 1.0;
    state.a = 1.0;
    state.hsv = HsvColor {
        h: 0.0,
        s: 0.0,
        v: 1.0,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_color_picker() {
        let p = new_color_picker(1.0, 0.0, 0.0, 1.0);
        assert!((p.rgb[0] - 1.0).abs() < 1e-5);
        assert!((p.rgb[3] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_rgb_to_hsv_red() {
        let h = rgb_to_hsv(1.0, 0.0, 0.0);
        assert!((h.h - 0.0).abs() < 1.0 || (h.h - 360.0).abs() < 1.0);
        assert!((h.s - 1.0).abs() < 1e-4);
        assert!((h.v - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_rgb_to_hsv_green() {
        let h = rgb_to_hsv(0.0, 1.0, 0.0);
        assert!((h.h - 120.0).abs() < 1.0);
        assert!((h.s - 1.0).abs() < 1e-4);
        assert!((h.v - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_hsv_to_rgb_round_trip() {
        let orig = [0.5_f32, 0.7, 0.3];
        let hsv = rgb_to_hsv(orig[0], orig[1], orig[2]);
        let back = hsv_to_rgb(hsv);
        assert!((back[0] - orig[0]).abs() < 1e-4);
        assert!((back[1] - orig[1]).abs() < 1e-4);
        assert!((back[2] - orig[2]).abs() < 1e-4);
    }

    #[test]
    fn test_hex_round_trip() {
        let hex = rgb_to_hex(1.0, 0.5, 0.25, 1.0);
        let back = hex_to_rgb(hex);
        assert!((back[0] - 1.0).abs() < 0.01);
        assert!((back[1] - 0.5).abs() < 0.01);
        assert!((back[2] - 0.25).abs() < 0.01);
        assert!((back[3] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_set_rgb() {
        let mut p = new_color_picker(1.0, 0.0, 0.0, 1.0);
        set_rgb(&mut p, 0.0, 1.0, 0.0);
        assert!((p.rgb[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_set_hsv() {
        let mut p = new_color_picker(1.0, 0.0, 0.0, 1.0);
        set_hsv(&mut p, 120.0, 1.0, 1.0); // green
        assert!((p.rgb[1] - 1.0).abs() < 1e-4);
        assert!((p.rgb[0]).abs() < 1e-4);
    }

    #[test]
    fn test_set_alpha() {
        let mut p = new_color_picker(1.0, 0.0, 0.0, 1.0);
        set_alpha(&mut p, 0.5);
        assert!((p.rgb[3] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_set_hex() {
        let mut p = new_color_picker(0.0, 0.0, 0.0, 1.0);
        let hex = rgb_to_hex(1.0, 0.0, 0.0, 1.0);
        set_hex(&mut p, hex);
        assert!((p.rgb[0] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_lerp_color() {
        let a = [0.0_f32; 4];
        let b = [1.0_f32; 4];
        let mid = lerp_color(a, b, 0.5);
        for &v in &mid {
            assert!((v - 0.5).abs() < 1e-5);
        }
    }

    #[test]
    fn test_color_distance_same() {
        let c = [0.5_f32, 0.5, 0.5];
        assert!(color_distance(c, c) < 1e-5);
    }

    #[test]
    fn test_color_distance_different() {
        let a = [1.0_f32, 0.0, 0.0];
        let b = [0.0_f32, 0.0, 0.0];
        assert!((color_distance(a, b) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_complementary_red_is_cyan() {
        let red = [1.0_f32, 0.0, 0.0];
        let comp = complementary_color(red);
        // Cyan: R=0, G=1, B=1
        assert!(comp[0] < 0.1);
        assert!(comp[1] > 0.9);
        assert!(comp[2] > 0.9);
    }

    #[test]
    fn test_rgb_to_linear() {
        let lin = rgb_to_linear(1.0, 1.0, 1.0);
        assert!((lin[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_linear_to_rgb() {
        let rgb = linear_to_rgb(1.0, 1.0, 1.0);
        assert!((rgb[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_rgb_linear_round_trip() {
        let r = 0.5_f32;
        let lin = rgb_to_linear(r, 0.0, 0.0);
        let back = linear_to_rgb(lin[0], lin[1], lin[2]);
        assert!((back[0] - r).abs() < 1e-4);
    }

    // ── Spec API tests ─────────────────────────────────────────────────────

    #[test]
    fn test_default_color_picker_config() {
        let cfg = default_color_picker_config();
        assert!(cfg.show_alpha);
        assert!(!cfg.hdr);
    }

    #[test]
    fn test_new_color_picker_state_white() {
        let cfg = default_color_picker_config();
        let s = new_color_picker_state(&cfg);
        assert!((s.r - 1.0).abs() < 1e-5);
        assert!((s.g - 1.0).abs() < 1e-5);
        assert!((s.b - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_color_picker_set_rgb_clamps() {
        let cfg = default_color_picker_config();
        let mut s = new_color_picker_state(&cfg);
        color_picker_set_rgb(&mut s, 2.0, -0.5, 0.5);
        assert!((s.r - 1.0).abs() < 1e-5);
        assert!((s.g - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_color_picker_set_hsv_green() {
        let cfg = default_color_picker_config();
        let mut s = new_color_picker_state(&cfg);
        color_picker_set_hsv(&mut s, 120.0, 1.0, 1.0);
        let rgb = color_picker_rgb(&s);
        assert!(rgb[1] > 0.99);
        assert!(rgb[0] < 0.01);
    }

    #[test]
    fn test_color_picker_hsv_roundtrip() {
        let cfg = default_color_picker_config();
        let mut s = new_color_picker_state(&cfg);
        color_picker_set_rgb(&mut s, 0.5, 0.3, 0.8);
        let hsv = color_picker_hsv(&s);
        let rgb_back = hsv_to_rgb(hsv);
        assert!((rgb_back[0] - 0.5).abs() < 1e-4);
        assert!((rgb_back[1] - 0.3).abs() < 1e-4);
        assert!((rgb_back[2] - 0.8).abs() < 1e-4);
    }

    #[test]
    fn test_color_picker_hex_string_red() {
        let cfg = default_color_picker_config();
        let mut s = new_color_picker_state(&cfg);
        color_picker_set_rgb(&mut s, 1.0, 0.0, 0.0);
        let hex = color_picker_hex_string(&s);
        assert_eq!(hex, "#FF0000");
    }

    #[test]
    fn test_color_picker_reset() {
        let cfg = default_color_picker_config();
        let mut s = new_color_picker_state(&cfg);
        color_picker_set_rgb(&mut s, 0.0, 0.0, 0.0);
        color_picker_reset(&mut s);
        let rgb = color_picker_rgb(&s);
        assert!((rgb[0] - 1.0).abs() < 1e-5);
    }
}
