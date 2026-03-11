//! Color space conversions (sRGB, linear, HSL, HSV, CIE Lab).

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// An RGB color in sRGB space, components in [0, 1].
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorRgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

/// A color in HSL (hue, saturation, lightness) space.
/// Hue is in degrees [0, 360), saturation and lightness in [0, 1].
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorHsl {
    pub h: f32,
    pub s: f32,
    pub l: f32,
}

/// A color in HSV (hue, saturation, value) space.
/// Hue is in degrees [0, 360), saturation and value in [0, 1].
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorHsv {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}

/// A color in CIE L*a*b* space (approximate).
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorLab {
    pub l: f32,
    pub a: f32,
    pub b: f32,
}

// ---------------------------------------------------------------------------
// sRGB <-> Linear
// ---------------------------------------------------------------------------

/// Convert a single sRGB component to linear.
#[allow(dead_code)]
pub fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Convert a single linear component to sRGB.
#[allow(dead_code)]
pub fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

// ---------------------------------------------------------------------------
// RGB <-> HSL
// ---------------------------------------------------------------------------

/// Convert an sRGB color to HSL.
#[allow(dead_code)]
pub fn rgb_to_hsl(rgb: &ColorRgb) -> ColorHsl {
    let r = rgb.r;
    let g = rgb.g;
    let b = rgb.b;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) * 0.5;

    if (max - min).abs() < 1e-7 {
        return ColorHsl { h: 0.0, s: 0.0, l };
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if (max - r).abs() < 1e-7 {
        let mut hh = (g - b) / d;
        if g < b {
            hh += 6.0;
        }
        hh
    } else if (max - g).abs() < 1e-7 {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };

    ColorHsl { h: h * 60.0, s, l }
}

/// Convert an HSL color to sRGB.
#[allow(dead_code)]
pub fn hsl_to_rgb(hsl: &ColorHsl) -> ColorRgb {
    let h = hsl.h;
    let s = hsl.s;
    let l = hsl.l;

    if s.abs() < 1e-7 {
        return ColorRgb { r: l, g: l, b: l };
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    let r = hue_to_rgb(p, q, h / 360.0 + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h / 360.0);
    let b = hue_to_rgb(p, q, h / 360.0 - 1.0 / 3.0);

    ColorRgb { r, g, b }
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 0.5 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

// ---------------------------------------------------------------------------
// RGB <-> HSV
// ---------------------------------------------------------------------------

/// Convert an sRGB color to HSV.
#[allow(dead_code)]
pub fn rgb_to_hsv(rgb: &ColorRgb) -> ColorHsv {
    let r = rgb.r;
    let g = rgb.g;
    let b = rgb.b;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;

    let v = max;
    let s = if max.abs() < 1e-7 { 0.0 } else { d / max };

    if d.abs() < 1e-7 {
        return ColorHsv { h: 0.0, s: 0.0, v };
    }

    let h = if (max - r).abs() < 1e-7 {
        let mut hh = (g - b) / d;
        if g < b {
            hh += 6.0;
        }
        hh
    } else if (max - g).abs() < 1e-7 {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };

    ColorHsv { h: h * 60.0, s, v }
}

/// Convert an HSV color to sRGB.
#[allow(dead_code)]
pub fn hsv_to_rgb(hsv: &ColorHsv) -> ColorRgb {
    let h = hsv.h;
    let s = hsv.s;
    let v = hsv.v;

    if s.abs() < 1e-7 {
        return ColorRgb { r: v, g: v, b: v };
    }

    let hh = (h % 360.0) / 60.0;
    let i = hh.floor() as i32;
    let f = hh - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    match i % 6 {
        0 => ColorRgb { r: v, g: t, b: p },
        1 => ColorRgb { r: q, g: v, b: p },
        2 => ColorRgb { r: p, g: v, b: t },
        3 => ColorRgb { r: p, g: q, b: v },
        4 => ColorRgb { r: t, g: p, b: v },
        _ => ColorRgb { r: v, g: p, b: q },
    }
}

// ---------------------------------------------------------------------------
// RGB <-> CIE Lab (approximate, D65 illuminant)
// ---------------------------------------------------------------------------

/// Convert an sRGB color to CIE L*a*b* (approximate, D65 reference white).
#[allow(dead_code)]
pub fn rgb_to_lab(rgb: &ColorRgb) -> ColorLab {
    // sRGB -> linear
    let rl = srgb_to_linear(rgb.r);
    let gl = srgb_to_linear(rgb.g);
    let bl = srgb_to_linear(rgb.b);

    // Linear RGB -> XYZ (sRGB matrix, D65)
    let x = 0.4124564 * rl + 0.3575761 * gl + 0.1804375 * bl;
    let y = 0.2126729 * rl + 0.7151522 * gl + 0.0721750 * bl;
    let z = 0.019_333_9 * rl + 0.119_192 * gl + 0.950_304_1 * bl;

    // D65 reference white
    let xn = 0.95047;
    let yn = 1.00000;
    let zn = 1.08883;

    let fx = lab_f(x / xn);
    let fy = lab_f(y / yn);
    let fz = lab_f(z / zn);

    ColorLab {
        l: 116.0 * fy - 16.0,
        a: 500.0 * (fx - fy),
        b: 200.0 * (fy - fz),
    }
}

/// Convert a CIE L*a*b* color to sRGB (approximate, D65 reference white).
#[allow(dead_code)]
pub fn lab_to_rgb(lab: &ColorLab) -> ColorRgb {
    let fy = (lab.l + 16.0) / 116.0;
    let fx = lab.a / 500.0 + fy;
    let fz = fy - lab.b / 200.0;

    let xn = 0.95047_f32;
    let yn = 1.00000_f32;
    let zn = 1.08883_f32;

    let x = xn * lab_f_inv(fx);
    let y = yn * lab_f_inv(fy);
    let z = zn * lab_f_inv(fz);

    // XYZ -> linear RGB
    let rl = 3.2404542 * x - 1.5371385 * y - 0.4985314 * z;
    let gl = -0.969_266 * x + 1.876_010_8 * y + 0.041_556 * z;
    let bl = 0.0556434 * x - 0.2040259 * y + 1.0572252 * z;

    ColorRgb {
        r: linear_to_srgb(rl.clamp(0.0, 1.0)),
        g: linear_to_srgb(gl.clamp(0.0, 1.0)),
        b: linear_to_srgb(bl.clamp(0.0, 1.0)),
    }
}

fn lab_f(t: f32) -> f32 {
    let delta: f32 = 6.0 / 29.0;
    if t > delta * delta * delta {
        t.cbrt()
    } else {
        t / (3.0 * delta * delta) + 4.0 / 29.0
    }
}

fn lab_f_inv(t: f32) -> f32 {
    let delta: f32 = 6.0 / 29.0;
    if t > delta {
        t * t * t
    } else {
        3.0 * delta * delta * (t - 4.0 / 29.0)
    }
}

// ---------------------------------------------------------------------------
// Distance / Utility
// ---------------------------------------------------------------------------

/// CIE76 Delta E color distance between two Lab colors.
#[allow(dead_code)]
pub fn color_distance_lab(a: &ColorLab, b: &ColorLab) -> f32 {
    let dl = a.l - b.l;
    let da = a.a - b.a;
    let db = a.b - b.b;
    (dl * dl + da * da + db * db).sqrt()
}

/// Linearly interpolate between two RGB colors.
#[allow(dead_code)]
pub fn lerp_rgb(a: &ColorRgb, b: &ColorRgb, t: f32) -> ColorRgb {
    let t = t.clamp(0.0, 1.0);
    ColorRgb {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
    }
}

/// Linearly interpolate between two HSL colors (shortest hue path).
#[allow(dead_code)]
pub fn lerp_hsl(a: &ColorHsl, b: &ColorHsl, t: f32) -> ColorHsl {
    let t = t.clamp(0.0, 1.0);
    let mut dh = b.h - a.h;
    if dh > 180.0 {
        dh -= 360.0;
    }
    if dh < -180.0 {
        dh += 360.0;
    }
    let mut h = a.h + dh * t;
    if h < 0.0 {
        h += 360.0;
    }
    if h >= 360.0 {
        h -= 360.0;
    }
    ColorHsl {
        h,
        s: a.s + (b.s - a.s) * t,
        l: a.l + (b.l - a.l) * t,
    }
}

/// Clamp all RGB components to [0, 1].
#[allow(dead_code)]
pub fn clamp_color(c: &ColorRgb) -> ColorRgb {
    ColorRgb {
        r: c.r.clamp(0.0, 1.0),
        g: c.g.clamp(0.0, 1.0),
        b: c.b.clamp(0.0, 1.0),
    }
}

/// Approximate color temperature (Kelvin) to sRGB.
/// Uses Tanner Helland's algorithm as a fast approximation.
#[allow(dead_code)]
pub fn color_temperature_to_rgb(kelvin: f32) -> ColorRgb {
    let temp = kelvin / 100.0;

    let r = if temp <= 66.0 {
        1.0
    } else {
        let x = temp - 60.0;
        (329.698_73 * x.powf(-0.133_204_76) / 255.0).clamp(0.0, 1.0)
    };

    let g = if temp <= 66.0 {
        let x = temp;
        (99.470_8 * x.ln() - 161.119_57).clamp(0.0, 255.0) / 255.0
    } else {
        let x = temp - 60.0;
        (288.122_17 * x.powf(-0.075_514_85) / 255.0).clamp(0.0, 1.0)
    };

    let b = if temp >= 66.0 {
        1.0
    } else if temp <= 19.0 {
        0.0
    } else {
        let x = temp - 10.0;
        (138.517_73 * x.ln() - 305.044_8).clamp(0.0, 255.0) / 255.0
    };

    ColorRgb { r, g, b }
}

/// Compute the relative luminance of an sRGB color (ITU-R BT.709).
#[allow(dead_code)]
pub fn luminance(rgb: &ColorRgb) -> f32 {
    0.2126 * srgb_to_linear(rgb.r) + 0.7152 * srgb_to_linear(rgb.g) + 0.0722 * srgb_to_linear(rgb.b)
}

#[allow(dead_code)]
pub fn rgb_to_hsv_f(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let hsv = rgb_to_hsv(&ColorRgb { r, g, b });
    (hsv.h, hsv.s, hsv.v)
}

#[allow(dead_code)]
pub fn hsv_to_rgb_f(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let rgb = hsv_to_rgb(&ColorHsv { h, s, v });
    (rgb.r, rgb.g, rgb.b)
}

#[allow(dead_code)]
pub fn rgb_to_srgb(c: f32) -> f32 {
    linear_to_srgb(c)
}

#[allow(dead_code)]
pub fn rgb_to_luma(r: f32, g: f32, b: f32) -> f32 {
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

#[allow(dead_code)]
pub fn lch_chroma(_l: f32, c: f32, _h: f32) -> f32 {
    c
}

#[allow(dead_code)]
pub fn rgb_to_oklab(r: f32, g: f32, b: f32) -> [f32; 3] {
    let l = rgb_to_luma(r, g, b);
    [l, 0.0, 0.0]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() < eps
    }

    #[test]
    fn test_srgb_to_linear_zero() {
        assert_eq!(srgb_to_linear(0.0), 0.0);
    }

    #[test]
    fn test_srgb_to_linear_one() {
        assert!(approx_eq(srgb_to_linear(1.0), 1.0, 1e-5));
    }

    #[test]
    fn test_linear_to_srgb_roundtrip() {
        for &v in &[0.0_f32, 0.1, 0.5, 0.9, 1.0] {
            let lin = srgb_to_linear(v);
            let back = linear_to_srgb(lin);
            assert!(approx_eq(v, back, 1e-4), "failed roundtrip for {v}");
        }
    }

    #[test]
    fn test_rgb_to_hsl_gray() {
        let gray = ColorRgb {
            r: 0.5,
            g: 0.5,
            b: 0.5,
        };
        let hsl = rgb_to_hsl(&gray);
        assert!(approx_eq(hsl.s, 0.0, 1e-5));
        assert!(approx_eq(hsl.l, 0.5, 1e-5));
    }

    #[test]
    fn test_rgb_to_hsl_red() {
        let red = ColorRgb {
            r: 1.0,
            g: 0.0,
            b: 0.0,
        };
        let hsl = rgb_to_hsl(&red);
        assert!(approx_eq(hsl.h, 0.0, 1e-3));
        assert!(approx_eq(hsl.s, 1.0, 1e-5));
        assert!(approx_eq(hsl.l, 0.5, 1e-5));
    }

    #[test]
    fn test_hsl_to_rgb_roundtrip() {
        let orig = ColorRgb {
            r: 0.2,
            g: 0.6,
            b: 0.8,
        };
        let hsl = rgb_to_hsl(&orig);
        let back = hsl_to_rgb(&hsl);
        assert!(approx_eq(orig.r, back.r, 1e-4));
        assert!(approx_eq(orig.g, back.g, 1e-4));
        assert!(approx_eq(orig.b, back.b, 1e-4));
    }

    #[test]
    fn test_rgb_to_hsv_black() {
        let black = ColorRgb {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let hsv = rgb_to_hsv(&black);
        assert!(approx_eq(hsv.v, 0.0, 1e-5));
    }

    #[test]
    fn test_hsv_to_rgb_roundtrip() {
        let orig = ColorRgb {
            r: 0.3,
            g: 0.7,
            b: 0.5,
        };
        let hsv = rgb_to_hsv(&orig);
        let back = hsv_to_rgb(&hsv);
        assert!(approx_eq(orig.r, back.r, 1e-4));
        assert!(approx_eq(orig.g, back.g, 1e-4));
        assert!(approx_eq(orig.b, back.b, 1e-4));
    }

    #[test]
    fn test_rgb_to_lab_white() {
        let white = ColorRgb {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        };
        let lab = rgb_to_lab(&white);
        assert!(approx_eq(lab.l, 100.0, 0.5));
        assert!(approx_eq(lab.a, 0.0, 0.5));
        assert!(approx_eq(lab.b, 0.0, 0.5));
    }

    #[test]
    fn test_rgb_to_lab_black() {
        let black = ColorRgb {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let lab = rgb_to_lab(&black);
        assert!(approx_eq(lab.l, 0.0, 0.5));
    }

    #[test]
    fn test_lab_roundtrip() {
        let orig = ColorRgb {
            r: 0.4,
            g: 0.6,
            b: 0.3,
        };
        let lab = rgb_to_lab(&orig);
        let back = lab_to_rgb(&lab);
        assert!(approx_eq(orig.r, back.r, 0.02));
        assert!(approx_eq(orig.g, back.g, 0.02));
        assert!(approx_eq(orig.b, back.b, 0.02));
    }

    #[test]
    fn test_color_distance_lab_same() {
        let c = ColorLab {
            l: 50.0,
            a: 10.0,
            b: -20.0,
        };
        assert!(approx_eq(color_distance_lab(&c, &c), 0.0, 1e-6));
    }

    #[test]
    fn test_color_distance_lab_different() {
        let a = ColorLab {
            l: 50.0,
            a: 0.0,
            b: 0.0,
        };
        let b = ColorLab {
            l: 60.0,
            a: 0.0,
            b: 0.0,
        };
        assert!(approx_eq(color_distance_lab(&a, &b), 10.0, 1e-5));
    }

    #[test]
    fn test_lerp_rgb() {
        let a = ColorRgb {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let b = ColorRgb {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        };
        let mid = lerp_rgb(&a, &b, 0.5);
        assert!(approx_eq(mid.r, 0.5, 1e-6));
    }

    #[test]
    fn test_lerp_hsl_shortest_path() {
        let a = ColorHsl {
            h: 350.0,
            s: 1.0,
            l: 0.5,
        };
        let b = ColorHsl {
            h: 10.0,
            s: 1.0,
            l: 0.5,
        };
        let mid = lerp_hsl(&a, &b, 0.5);
        // Should wrap around 0, giving hue ~0 or ~360
        assert!(mid.h < 20.0 || mid.h > 340.0);
    }

    #[test]
    fn test_clamp_color() {
        let c = ColorRgb {
            r: -0.1,
            g: 1.5,
            b: 0.5,
        };
        let clamped = clamp_color(&c);
        assert_eq!(clamped.r, 0.0);
        assert_eq!(clamped.g, 1.0);
        assert_eq!(clamped.b, 0.5);
    }

    #[test]
    fn test_color_temperature_daylight() {
        let c = color_temperature_to_rgb(6500.0);
        // Daylight should be roughly white-ish
        assert!(c.r > 0.8);
        assert!(c.g > 0.8);
        assert!(c.b > 0.8);
    }

    #[test]
    fn test_color_temperature_warm() {
        let warm = color_temperature_to_rgb(2700.0);
        let cool = color_temperature_to_rgb(10000.0);
        // Warm should have more red relative to blue
        assert!(warm.r > warm.b);
        // Cool should have more blue
        assert!(cool.b > cool.r || approx_eq(cool.b, cool.r, 0.15));
    }

    #[test]
    fn test_luminance_black() {
        let black = ColorRgb {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        assert!(approx_eq(luminance(&black), 0.0, 1e-6));
    }

    #[test]
    fn test_luminance_white() {
        let white = ColorRgb {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        };
        assert!(approx_eq(luminance(&white), 1.0, 0.01));
    }
}
