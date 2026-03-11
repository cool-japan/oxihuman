//! Procedural noise-based mesh deformation.
//!
//! Uses a deterministic hash-based value noise (no external rand dependency)
//! to displace mesh vertices along their normals.  The noise is fully
//! reproducible given the same seed and configuration.

// ── public structs ────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// Configuration for a noise-based mesh deformation pass.
pub struct NoiseDeformConfig {
    /// Spatial frequency (scale) of the noise pattern.
    pub frequency: f32,
    /// Maximum displacement amplitude along the vertex normal.
    pub amplitude: f32,
    /// Number of octaves for turbulence mode (1 = single-octave value noise).
    pub octaves: u32,
    /// Lacunarity – frequency multiplier per octave.
    pub lacunarity: f32,
    /// Persistence – amplitude multiplier per octave.
    pub persistence: f32,
    /// Reproducible seed.
    pub seed: u32,
    /// Whether to use turbulence (multi-octave) instead of single-octave noise.
    pub use_turbulence: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// Result of a noise deformation pass.
pub struct NoiseDeformResult {
    /// Deformed vertex positions.
    pub vertices: Vec<[f32; 3]>,
    /// Per-vertex displacement magnitude.
    pub displacements: Vec<f32>,
}

// ── public functions ──────────────────────────────────────────────────────────

#[allow(dead_code)]
/// Returns a [`NoiseDeformConfig`] with sensible defaults.
pub fn default_noise_deform_config() -> NoiseDeformConfig {
    NoiseDeformConfig {
        frequency: 1.0,
        amplitude: 0.1,
        octaves: 4,
        lacunarity: 2.0,
        persistence: 0.5,
        seed: 42,
        use_turbulence: false,
    }
}

#[allow(dead_code)]
/// Deterministic 3-D value noise in [0, 1].
///
/// Uses integer hashing (no external RNG) so the output is fully reproducible.
pub fn value_noise_3d(x: f32, y: f32, z: f32, seed: u32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let zi = z.floor() as i32;
    let xf = x - x.floor();
    let yf = y - y.floor();
    let zf = z - z.floor();

    // Smooth step
    let ux = smooth(xf);
    let uy = smooth(yf);
    let uz = smooth(zf);

    let v000 = hash_f(xi, yi, zi, seed);
    let v100 = hash_f(xi + 1, yi, zi, seed);
    let v010 = hash_f(xi, yi + 1, zi, seed);
    let v110 = hash_f(xi + 1, yi + 1, zi, seed);
    let v001 = hash_f(xi, yi, zi + 1, seed);
    let v101 = hash_f(xi + 1, yi, zi + 1, seed);
    let v011 = hash_f(xi, yi + 1, zi + 1, seed);
    let v111 = hash_f(xi + 1, yi + 1, zi + 1, seed);

    let x00 = lerp(v000, v100, ux);
    let x10 = lerp(v010, v110, ux);
    let x01 = lerp(v001, v101, ux);
    let x11 = lerp(v011, v111, ux);

    let y0 = lerp(x00, x10, uy);
    let y1 = lerp(x01, x11, uy);

    lerp(y0, y1, uz)
}

#[allow(dead_code)]
/// Multi-octave turbulence noise.  Returns a value roughly in [0, 1].
pub fn turbulence_noise(x: f32, y: f32, z: f32, octaves: u32, seed: u32) -> f32 {
    let mut value = 0.0_f32;
    let mut amp = 1.0_f32;
    let mut freq = 1.0_f32;
    let mut max_amp = 0.0_f32;

    for oct in 0..octaves {
        value += value_noise_3d(x * freq, y * freq, z * freq, seed.wrapping_add(oct * 31)) * amp;
        max_amp += amp;
        amp *= 0.5;
        freq *= 2.0;
    }

    if max_amp > 0.0 {
        value / max_amp
    } else {
        0.0
    }
}

#[allow(dead_code)]
/// Deforms `verts` along `normals` using the noise configuration, returning a
/// [`NoiseDeformResult`].
pub fn noise_deform_mesh(
    verts: &[[f32; 3]],
    normals: &[[f32; 3]],
    cfg: &NoiseDeformConfig,
) -> NoiseDeformResult {
    let n = verts.len().min(normals.len());
    let mut out_verts = Vec::with_capacity(n);
    let mut displacements = Vec::with_capacity(n);

    for i in 0..n {
        let v = verts[i];
        let nm = normals[i];
        let noise_val = sample_noise(v, cfg);
        // Map [0,1] -> [-1,1] displacement
        let disp = (noise_val * 2.0 - 1.0) * cfg.amplitude;
        let displaced = [
            v[0] + nm[0] * disp,
            v[1] + nm[1] * disp,
            v[2] + nm[2] * disp,
        ];
        out_verts.push(displaced);
        displacements.push(disp.abs());
    }

    NoiseDeformResult {
        vertices: out_verts,
        displacements,
    }
}

#[allow(dead_code)]
/// In-place version of [`noise_deform_mesh`] — mutates `verts` directly.
pub fn noise_deform_inplace(
    verts: &mut [[f32; 3]],
    normals: &[[f32; 3]],
    cfg: &NoiseDeformConfig,
) {
    let n = verts.len().min(normals.len());
    for i in 0..n {
        let v = verts[i];
        let nm = normals[i];
        let noise_val = sample_noise(v, cfg);
        let disp = (noise_val * 2.0 - 1.0) * cfg.amplitude;
        verts[i] = [
            v[0] + nm[0] * disp,
            v[1] + nm[1] * disp,
            v[2] + nm[2] * disp,
        ];
    }
}

#[allow(dead_code)]
/// Returns the number of vertices in the result.
pub fn noise_deform_vertex_count(result: &NoiseDeformResult) -> usize {
    result.vertices.len()
}

#[allow(dead_code)]
/// Returns the maximum per-vertex displacement in the result.
pub fn noise_deform_max_displacement(result: &NoiseDeformResult) -> f32 {
    result
        .displacements
        .iter()
        .cloned()
        .fold(0.0_f32, f32::max)
}

#[allow(dead_code)]
/// Sets the noise spatial frequency in `cfg`.
pub fn set_noise_frequency(cfg: &mut NoiseDeformConfig, freq: f32) {
    cfg.frequency = freq;
}

#[allow(dead_code)]
/// Sets the noise displacement amplitude in `cfg`.
pub fn set_noise_amplitude(cfg: &mut NoiseDeformConfig, amp: f32) {
    cfg.amplitude = amp;
}

// ── private helpers ───────────────────────────────────────────────────────────

/// Hash three integers + seed into a float in [0, 1].
fn hash_f(x: i32, y: i32, z: i32, seed: u32) -> f32 {
    let mut h = seed;
    h ^= (x.wrapping_mul(1619)) as u32;
    h ^= (y.wrapping_mul(31337)) as u32;
    h ^= (z.wrapping_mul(6971)) as u32;
    h = h.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    // Extract bits [16..31] for better distribution
    (((h >> 16) & 0xFFFF) as f32) / 65535.0
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn sample_noise(v: [f32; 3], cfg: &NoiseDeformConfig) -> f32 {
    let sx = v[0] * cfg.frequency;
    let sy = v[1] * cfg.frequency;
    let sz = v[2] * cfg.frequency;
    if cfg.use_turbulence {
        turbulence_noise(sx, sy, sz, cfg.octaves, cfg.seed)
    } else {
        value_noise_3d(sx, sy, sz, cfg.seed)
    }
}

// ── unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_noise_in_range() {
        for ix in 0..5_i32 {
            for iy in 0..5_i32 {
                let v = value_noise_3d(ix as f32 * 0.37, iy as f32 * 0.53, 0.1, 42);
                assert!((0.0..=1.0).contains(&v), "value_noise out of range: {v}");
            }
        }
    }

    #[test]
    fn test_value_noise_deterministic() {
        let a = value_noise_3d(1.5, 2.3, 0.7, 99);
        let b = value_noise_3d(1.5, 2.3, 0.7, 99);
        assert_eq!(a, b);
    }

    #[test]
    fn test_turbulence_in_range() {
        let v = turbulence_noise(0.5, 0.5, 0.5, 4, 7);
        assert!((0.0..=1.0).contains(&v), "turbulence out of range: {v}");
    }

    #[test]
    fn test_noise_deform_mesh_length() {
        let verts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let normals = vec![[0.0, 1.0, 0.0], [0.0, 1.0, 0.0]];
        let cfg = default_noise_deform_config();
        let result = noise_deform_mesh(&verts, &normals, &cfg);
        assert_eq!(noise_deform_vertex_count(&result), 2);
        assert_eq!(result.displacements.len(), 2);
    }

    #[test]
    fn test_noise_deform_displacement_bounded() {
        let verts = vec![[0.5, 0.5, 0.5]];
        let normals = vec![[0.0, 1.0, 0.0]];
        let mut cfg = default_noise_deform_config();
        cfg.amplitude = 0.2;
        let result = noise_deform_mesh(&verts, &normals, &cfg);
        assert!(
            result.displacements[0] <= 0.2 + 1e-6,
            "displacement exceeds amplitude"
        );
    }

    #[test]
    fn test_noise_deform_inplace_matches_result() {
        let verts_orig = vec![[0.3, 0.1, 0.9]];
        let normals = vec![[0.0, 1.0, 0.0]];
        let cfg = default_noise_deform_config();

        let result = noise_deform_mesh(&verts_orig, &normals, &cfg);

        let mut verts_ip = verts_orig.clone();
        noise_deform_inplace(&mut verts_ip, &normals, &cfg);

        assert!((result.vertices[0][1] - verts_ip[0][1]).abs() < 1e-6);
    }

    #[test]
    fn test_set_noise_frequency() {
        let mut cfg = default_noise_deform_config();
        set_noise_frequency(&mut cfg, std::f32::consts::PI);
        assert!((cfg.frequency - std::f32::consts::PI).abs() < 1e-6);
    }

    #[test]
    fn test_set_noise_amplitude() {
        let mut cfg = default_noise_deform_config();
        set_noise_amplitude(&mut cfg, 0.75);
        assert!((cfg.amplitude - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_noise_deform_max_displacement() {
        let verts = vec![[0.0, 0.0, 0.0], [10.0, 10.0, 10.0]];
        let normals = vec![[0.0, 1.0, 0.0]; 2];
        let cfg = default_noise_deform_config();
        let result = noise_deform_mesh(&verts, &normals, &cfg);
        let max = noise_deform_max_displacement(&result);
        for &d in &result.displacements {
            assert!(d <= max + 1e-6);
        }
    }
}
