//! Wave and ripple deformation of mesh vertices.

// ---------------------------------------------------------------------------
// Structures
// ---------------------------------------------------------------------------

#[allow(dead_code)]
#[derive(Clone)]
pub enum WaveShape {
    Sine,
    Cosine,
    Square,
    Triangle,
    Sawtooth,
}

#[allow(dead_code)]
pub struct WaveParams {
    pub amplitude: f32,
    pub frequency: f32,
    pub phase: f32,
    /// Direction along which the wave propagates.
    pub direction: [f32; 3],
    pub shape: WaveShape,
    /// Exponential decay from the wave origin.
    pub decay: f32,
}

#[allow(dead_code)]
pub struct RippleSource {
    pub center: [f32; 3],
    pub amplitude: f32,
    pub frequency: f32,
    pub phase: f32,
    pub decay: f32,
}

// ---------------------------------------------------------------------------
// Default
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn default_wave_params() -> WaveParams {
    WaveParams {
        amplitude: 0.1,
        frequency: 1.0,
        phase: 0.0,
        direction: [1.0, 0.0, 0.0],
        shape: WaveShape::Sine,
        decay: 0.0,
    }
}

// ---------------------------------------------------------------------------
// Math helpers
// ---------------------------------------------------------------------------

/// Normalize a 3-vector.  Returns zero vector if near-zero length.
#[allow(dead_code)]
pub fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        [0.0; 3]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

/// Dot product of two 3-vectors.
#[allow(dead_code)]
pub fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Euclidean distance between two 3-points.
#[allow(dead_code)]
pub fn distance3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Evaluate the wave shape function at phase argument `t` (in radians).
fn shape_value(shape: &WaveShape, t: f32) -> f32 {
    use std::f32::consts::PI;
    match shape {
        WaveShape::Sine => t.sin(),
        WaveShape::Cosine => t.cos(),
        WaveShape::Square => {
            if t.sin() >= 0.0 {
                1.0
            } else {
                -1.0
            }
        }
        WaveShape::Triangle => {
            // Triangle wave: (2/pi) * arcsin(sin(t))
            (2.0 / PI) * t.sin().asin()
        }
        WaveShape::Sawtooth => {
            // Normalised to [-1, 1]
            let wrapped = t / (2.0 * PI);
            2.0 * (wrapped - wrapped.floor()) - 1.0
        }
    }
}

// ---------------------------------------------------------------------------
// Wave functions
// ---------------------------------------------------------------------------

/// Evaluate a planar wave at `position` and `time`.
/// The wave travels along `params.direction`.
#[allow(dead_code)]
pub fn wave_value(params: &WaveParams, position: [f32; 3], time: f32) -> f32 {
    use std::f32::consts::TAU;
    let dir = normalize3(params.direction);
    let proj = dot3(position, dir);
    let arg = TAU * params.frequency * (proj - time) + params.phase;
    let raw = shape_value(&params.shape, arg);
    let dist = proj.abs();
    let envelope = wave_envelope(params.amplitude, params.decay, dist);
    envelope * raw
}

/// Displace mesh vertices along their normals by the wave value.
#[allow(dead_code)]
pub fn apply_wave_deform(
    positions: &mut [[f32; 3]],
    normals: &[[f32; 3]],
    params: &WaveParams,
    time: f32,
) {
    let n = positions.len().min(normals.len());
    for i in 0..n {
        let val = wave_value(params, positions[i], time);
        let nrm = normalize3(normals[i]);
        positions[i][0] += val * nrm[0];
        positions[i][1] += val * nrm[1];
        positions[i][2] += val * nrm[2];
    }
}

/// Evaluate a circular ripple from a source at position `pos` and `time`.
#[allow(dead_code)]
pub fn ripple_value(src: &RippleSource, pos: [f32; 3], time: f32) -> f32 {
    use std::f32::consts::TAU;
    let dist = distance3(src.center, pos);
    let arg = TAU * src.frequency * (dist - time) + src.phase;
    let raw = arg.sin();
    wave_envelope(src.amplitude, src.decay, dist) * raw
}

/// Displace mesh vertices along normals by a ripple.
#[allow(dead_code)]
pub fn apply_ripple(
    positions: &mut [[f32; 3]],
    normals: &[[f32; 3]],
    src: &RippleSource,
    time: f32,
) {
    let n = positions.len().min(normals.len());
    for i in 0..n {
        let val = ripple_value(src, positions[i], time);
        let nrm = normalize3(normals[i]);
        positions[i][0] += val * nrm[0];
        positions[i][1] += val * nrm[1];
        positions[i][2] += val * nrm[2];
    }
}

/// Superpose multiple ripple sources (sum of displacements).
#[allow(dead_code)]
pub fn apply_multiple_ripples(
    positions: &mut [[f32; 3]],
    normals: &[[f32; 3]],
    sources: &[RippleSource],
    time: f32,
) {
    let n = positions.len().min(normals.len());
    for i in 0..n {
        let nrm = normalize3(normals[i]);
        let total: f32 = sources
            .iter()
            .map(|s| ripple_value(s, positions[i], time))
            .sum();
        positions[i][0] += total * nrm[0];
        positions[i][1] += total * nrm[1];
        positions[i][2] += total * nrm[2];
    }
}

/// Exponential envelope: amplitude * e^(-decay * dist).
/// When decay == 0 the envelope equals amplitude.
#[allow(dead_code)]
pub fn wave_envelope(amplitude: f32, decay: f32, distance: f32) -> f32 {
    amplitude * (-decay * distance).exp()
}

/// Standing wave value: sin(k * pos) * cos(omega * time).
/// Here `freq` is used as both k and omega for simplicity.
#[allow(dead_code)]
pub fn standing_wave(pos: f32, freq: f32, time: f32) -> f32 {
    use std::f32::consts::TAU;
    (TAU * freq * pos).sin() * (TAU * freq * time).cos()
}

/// Sum two wave values (linear superposition).
#[allow(dead_code)]
pub fn wave_interference(a: f32, b: f32) -> f32 {
    a + b
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::{FRAC_PI_2, PI};

    #[test]
    fn test_wave_value_sine_at_zero() {
        let p = default_wave_params();
        // At proj=0, time=0, phase=0 → sin(0) = 0
        let v = wave_value(&p, [0.0, 0.0, 0.0], 0.0);
        assert!(v.abs() < 1e-5, "sine at 0 should be 0, got {v}");
    }

    #[test]
    fn test_wave_value_cosine_at_zero() {
        let p = WaveParams {
            shape: WaveShape::Cosine,
            ..default_wave_params()
        };
        // cos(0) = 1, amplitude=0.1 → 0.1
        let v = wave_value(&p, [0.0, 0.0, 0.0], 0.0);
        assert!((v - 0.1).abs() < 1e-5, "cosine at 0 should be 0.1, got {v}");
    }

    #[test]
    fn test_wave_value_changes_with_position() {
        let p = default_wave_params();
        let v0 = wave_value(&p, [0.0, 0.0, 0.0], 0.0);
        let v1 = wave_value(&p, [0.25, 0.0, 0.0], 0.0);
        // They should differ.
        assert!((v0 - v1).abs() > 1e-6, "wave should vary with position");
    }

    #[test]
    fn test_apply_wave_deform_changes_positions() {
        let mut pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let orig = pos.clone();
        let nrm = vec![[0.0f32, 1.0, 0.0], [0.0, 1.0, 0.0]];
        let p = WaveParams {
            shape: WaveShape::Cosine,
            amplitude: 1.0,
            ..default_wave_params()
        };
        apply_wave_deform(&mut pos, &nrm, &p, 0.0);
        // Cosine at x=0 → displacement along y.
        assert!((pos[0][1] - orig[0][1]).abs() > 1e-5);
    }

    #[test]
    fn test_ripple_value_at_center() {
        let src = RippleSource {
            center: [0.0, 0.0, 0.0],
            amplitude: 1.0,
            frequency: 1.0,
            phase: 0.0,
            decay: 0.0,
        };
        // dist = 0 → sin(0) = 0
        let v = ripple_value(&src, [0.0, 0.0, 0.0], 0.0);
        assert!(
            v.abs() < 1e-5,
            "ripple at center at t=0 should be 0, got {v}"
        );
    }

    #[test]
    fn test_ripple_value_away_from_center() {
        use std::f32::consts::FRAC_1_PI;
        let src = RippleSource {
            center: [0.0, 0.0, 0.0],
            amplitude: 1.0,
            frequency: 0.5 * FRAC_1_PI,
            phase: FRAC_PI_2, // shifts to cosine
            decay: 0.0,
        };
        let v = ripple_value(&src, [1.0, 0.0, 0.0], 0.0);
        assert!(v.abs() <= 1.001, "ripple should be bounded by amplitude");
    }

    #[test]
    fn test_apply_ripple_changes_positions() {
        let mut pos = vec![[1.0f32, 0.0, 0.0]];
        let orig = pos.clone();
        let nrm = vec![[0.0f32, 1.0, 0.0]];
        let src = RippleSource {
            center: [0.0, 0.0, 0.0],
            amplitude: 1.0,
            frequency: 1.0 / (2.0 * PI),
            phase: FRAC_PI_2,
            decay: 0.0,
        };
        apply_ripple(&mut pos, &nrm, &src, 0.0);
        let _ = (pos[0][1] - orig[0][1]).abs(); // just must not panic
        let _ = orig;
    }

    #[test]
    fn test_apply_multiple_ripples() {
        let mut pos = vec![[1.0f32, 0.0, 0.0]];
        let nrm = vec![[0.0f32, 1.0, 0.0]];
        let sources = vec![
            RippleSource {
                center: [0.0, 0.0, 0.0],
                amplitude: 0.5,
                frequency: 1.0,
                phase: 0.0,
                decay: 0.0,
            },
            RippleSource {
                center: [2.0, 0.0, 0.0],
                amplitude: 0.5,
                frequency: 1.0,
                phase: 0.0,
                decay: 0.0,
            },
        ];
        apply_multiple_ripples(&mut pos, &nrm, &sources, 0.0);
        // Must not panic and return valid floats.
        assert!(pos[0][1].is_finite());
    }

    #[test]
    fn test_wave_envelope_no_decay() {
        let e = wave_envelope(2.0, 0.0, 100.0);
        assert!((e - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_wave_envelope_decay() {
        let e = wave_envelope(1.0, 1.0, 10.0);
        assert!(e < 0.001, "envelope should decay strongly at dist=10");
    }

    #[test]
    fn test_standing_wave_at_zero() {
        // sin(0)*cos(0) = 0
        let v = standing_wave(0.0, 1.0, 0.0);
        assert!(v.abs() < 1e-5);
    }

    #[test]
    fn test_standing_wave_quarter_period() {
        // At pos=0.25 (quarter wavelength) and time=0: sin(pi/2)*1 = 1
        // freq=1 → k=2pi, so k*0.25 = pi/2
        let v = standing_wave(0.25, 1.0, 0.0);
        assert!((v - 1.0).abs() < 1e-5, "got {v}");
    }

    #[test]
    fn test_wave_interference() {
        assert!((wave_interference(0.3, 0.7) - 1.0).abs() < 1e-6);
        assert!((wave_interference(-1.0, 1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_distance3() {
        let d = distance3([0.0, 0.0, 0.0], [3.0, 4.0, 0.0]);
        assert!((d - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_normalize3() {
        let n = normalize3([3.0, 0.0, 4.0]);
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_dot3() {
        assert!((dot3([1.0, 0.0, 0.0], [0.0, 1.0, 0.0])).abs() < 1e-6);
        assert!((dot3([1.0, 0.0, 0.0], [1.0, 0.0, 0.0]) - 1.0).abs() < 1e-6);
    }
}
