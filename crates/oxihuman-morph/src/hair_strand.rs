//! Hair strand simulation morphs for procedural hair control.

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Hair profile categories.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HairProfile {
    Straight,
    Wavy,
    Curly,
    Coily,
}

/// Configuration for hair strand generation.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct HairStrandConfig {
    /// Number of points per strand.
    pub point_count: usize,
    /// Default strand length.
    pub length: f32,
    /// Gravity strength.
    pub gravity: f32,
    /// Curl frequency for wavy/curly/coily profiles.
    pub curl_freq: f32,
    /// Curl amplitude for wavy/curly/coily profiles.
    pub curl_amp: f32,
}

/// A single hair strand represented as a sequence of 3D points.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct HairStrand {
    /// Points along the strand: `[x, y, z, x, y, z, ...]`.
    pub points: Vec<f32>,
    /// Hair profile type.
    pub profile: HairProfile,
    /// Original strand length (sum of segment lengths at generation).
    pub rest_length: f32,
}

/// Axis-aligned bounding box result.
#[allow(dead_code)]
pub type BoundingBox = ([f32; 3], [f32; 3]);

/// Simple LCG pseudo-random number generator (no external crate).
struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed.wrapping_add(1))
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }

    fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 11) as f32 / (1u64 << 53) as f32
    }
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

/// Create a default strand configuration.
#[allow(dead_code)]
pub fn default_strand_config() -> HairStrandConfig {
    HairStrandConfig {
        point_count: 16,
        length: 0.3,
        gravity: 9.81,
        curl_freq: 4.0,
        curl_amp: 0.01,
    }
}

/// Create a new hair strand with no points.
#[allow(dead_code)]
pub fn new_hair_strand(profile: HairProfile) -> HairStrand {
    HairStrand {
        points: Vec::new(),
        profile,
        rest_length: 0.0,
    }
}

// ---------------------------------------------------------------------------
// Generation
// ---------------------------------------------------------------------------

/// Generate strand points from a root position, direction, and profile.
/// Uses an LCG seeded by the root position hash to add curl variation.
#[allow(dead_code)]
pub fn generate_strand_points(
    root: [f32; 3],
    direction: [f32; 3],
    config: &HairStrandConfig,
    profile: HairProfile,
) -> HairStrand {
    let n = config.point_count.max(2);
    let seg_len = config.length / (n - 1) as f32;

    // Normalize direction
    let dlen =
        (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2])
            .sqrt()
            .max(1e-12);
    let dir = [
        direction[0] / dlen,
        direction[1] / dlen,
        direction[2] / dlen,
    ];

    // Build a tangent for curl offset
    let tangent = if dir[1].abs() < 0.9 {
        // cross(dir, up)
        let cx = dir[2];
        let cy = 0.0;
        let cz = -dir[0];
        let cl = (cx * cx + cy * cy + cz * cz).sqrt().max(1e-12);
        [cx / cl, cy / cl, cz / cl]
    } else {
        let cx = 0.0;
        let cy = -dir[2];
        let cz = dir[1];
        let cl = (cx * cx + cy * cy + cz * cz).sqrt().max(1e-12);
        [cx / cl, cy / cl, cz / cl]
    };

    let (freq, amp) = curl_params(profile, config);

    let seed = (root[0].to_bits() as u64)
        .wrapping_add(root[1].to_bits() as u64)
        .wrapping_add(root[2].to_bits() as u64);
    let mut rng = Lcg::new(seed);
    let phase = rng.next_f32() * std::f32::consts::TAU;

    let mut points = Vec::with_capacity(n * 3);
    let mut px = root[0];
    let mut py = root[1];
    let mut pz = root[2];
    points.push(px);
    points.push(py);
    points.push(pz);

    for i in 1..n {
        let t = i as f32 / (n - 1) as f32;
        let angle = t * freq * std::f32::consts::TAU + phase;
        let curl_offset = angle.sin() * amp;

        px += dir[0] * seg_len + tangent[0] * curl_offset;
        py += dir[1] * seg_len + tangent[1] * curl_offset;
        pz += dir[2] * seg_len + tangent[2] * curl_offset;

        points.push(px);
        points.push(py);
        points.push(pz);
    }

    let rest = compute_rest_length(&points);

    HairStrand {
        points,
        profile,
        rest_length: rest,
    }
}

fn curl_params(profile: HairProfile, config: &HairStrandConfig) -> (f32, f32) {
    match profile {
        HairProfile::Straight => (0.0, 0.0),
        HairProfile::Wavy => (config.curl_freq * 0.5, config.curl_amp * 0.5),
        HairProfile::Curly => (config.curl_freq, config.curl_amp),
        HairProfile::Coily => (config.curl_freq * 2.0, config.curl_amp * 1.5),
    }
}

fn compute_rest_length(points: &[f32]) -> f32 {
    let n = points.len() / 3;
    if n < 2 {
        return 0.0;
    }
    let mut total = 0.0_f32;
    for i in 1..n {
        let dx = points[i * 3] - points[(i - 1) * 3];
        let dy = points[i * 3 + 1] - points[(i - 1) * 3 + 1];
        let dz = points[i * 3 + 2] - points[(i - 1) * 3 + 2];
        total += (dx * dx + dy * dy + dz * dz).sqrt();
    }
    total
}

// ---------------------------------------------------------------------------
// Physics / Modification
// ---------------------------------------------------------------------------

/// Apply gravity to a strand, displacing each point downward proportional
/// to its distance from root.
#[allow(dead_code)]
pub fn apply_gravity_to_strand(strand: &mut HairStrand, gravity: f32, dt: f32) {
    let n = strand.points.len() / 3;
    if n < 2 {
        return;
    }
    for i in 1..n {
        let factor = i as f32 / (n - 1) as f32;
        strand.points[i * 3 + 1] -= gravity * dt * factor;
    }
}

// ---------------------------------------------------------------------------
// Queries
// ---------------------------------------------------------------------------

/// Return the total length of the strand (sum of segment lengths).
#[allow(dead_code)]
pub fn strand_length(strand: &HairStrand) -> f32 {
    compute_rest_length(&strand.points)
}

/// Return the number of control points in the strand.
#[allow(dead_code)]
pub fn strand_point_count(strand: &HairStrand) -> usize {
    strand.points.len() / 3
}

/// Set the strand's profile type.
#[allow(dead_code)]
pub fn set_strand_profile(strand: &mut HairStrand, profile: HairProfile) {
    strand.profile = profile;
}

/// Return the curl frequency for a given profile using the supplied config.
#[allow(dead_code)]
pub fn curl_frequency(profile: HairProfile, config: &HairStrandConfig) -> f32 {
    curl_params(profile, config).0
}

/// Return the curl amplitude for a given profile using the supplied config.
#[allow(dead_code)]
pub fn curl_amplitude(profile: HairProfile, config: &HairStrandConfig) -> f32 {
    curl_params(profile, config).1
}

/// Compute the tangent vector at parameter `t` (0..1) along the strand.
/// Returns `[0, 0, 0]` for empty or single-point strands.
#[allow(dead_code)]
pub fn strand_tangent_at(strand: &HairStrand, t: f32) -> [f32; 3] {
    let n = strand.points.len() / 3;
    if n < 2 {
        return [0.0, 0.0, 0.0];
    }
    let t = t.clamp(0.0, 1.0);
    let seg = (t * (n - 1) as f32).min((n - 2) as f32);
    let idx = seg as usize;
    let dx = strand.points[(idx + 1) * 3] - strand.points[idx * 3];
    let dy = strand.points[(idx + 1) * 3 + 1] - strand.points[idx * 3 + 1];
    let dz = strand.points[(idx + 1) * 3 + 2] - strand.points[idx * 3 + 2];
    let len = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-12);
    [dx / len, dy / len, dz / len]
}

/// Blend two strands point-by-point. If they have different point counts,
/// use the shorter count.
#[allow(dead_code)]
pub fn blend_strands(a: &HairStrand, b: &HairStrand, t: f32) -> HairStrand {
    let t = t.clamp(0.0, 1.0);
    let count = a.points.len().min(b.points.len());
    let mut points = Vec::with_capacity(count);
    for i in 0..count {
        points.push(a.points[i] + (b.points[i] - a.points[i]) * t);
    }
    let profile = if t < 0.5 { a.profile } else { b.profile };
    let rest = compute_rest_length(&points);
    HairStrand {
        points,
        profile,
        rest_length: rest,
    }
}

/// Compute an axis-aligned bounding box for the strand.
/// Returns `([min_x, min_y, min_z], [max_x, max_y, max_z])`.
/// Returns zeros for empty strands.
#[allow(dead_code)]
pub fn strand_bounding_box(strand: &HairStrand) -> BoundingBox {
    let n = strand.points.len() / 3;
    if n == 0 {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut min = [strand.points[0], strand.points[1], strand.points[2]];
    let mut max = min;
    for i in 1..n {
        for j in 0..3 {
            let v = strand.points[i * 3 + j];
            if v < min[j] {
                min[j] = v;
            }
            if v > max[j] {
                max[j] = v;
            }
        }
    }
    (min, max)
}

/// Convert strand points to a flat vertex buffer (just returns a clone of
/// the internal points array for now; a production version would add normals).
#[allow(dead_code)]
pub fn strand_to_vertices(strand: &HairStrand) -> Vec<f32> {
    strand.points.clone()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_strand_config();
        assert!(cfg.point_count >= 2);
        assert!(cfg.length > 0.0);
    }

    #[test]
    fn test_new_strand_empty() {
        let s = new_hair_strand(HairProfile::Straight);
        assert!(s.points.is_empty());
        assert_eq!(s.profile, HairProfile::Straight);
    }

    #[test]
    fn test_generate_straight() {
        let cfg = default_strand_config();
        let s = generate_strand_points(
            [0.0, 0.0, 0.0],
            [0.0, -1.0, 0.0],
            &cfg,
            HairProfile::Straight,
        );
        assert_eq!(strand_point_count(&s), cfg.point_count);
        assert!(strand_length(&s) > 0.0);
    }

    #[test]
    fn test_generate_curly() {
        let cfg = default_strand_config();
        let s = generate_strand_points([0.0, 0.0, 0.0], [0.0, -1.0, 0.0], &cfg, HairProfile::Curly);
        assert_eq!(strand_point_count(&s), cfg.point_count);
    }

    #[test]
    fn test_generate_coily() {
        let cfg = default_strand_config();
        let s = generate_strand_points([0.0, 0.0, 0.0], [0.0, -1.0, 0.0], &cfg, HairProfile::Coily);
        assert_eq!(strand_point_count(&s), cfg.point_count);
    }

    #[test]
    fn test_gravity_displaces_downward() {
        let cfg = default_strand_config();
        let mut s = generate_strand_points(
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
            &cfg,
            HairProfile::Straight,
        );
        let y_before = s.points[s.points.len() - 2]; // last point Y
        apply_gravity_to_strand(&mut s, 9.81, 0.1);
        let y_after = s.points[s.points.len() - 2];
        assert!(y_after < y_before);
    }

    #[test]
    fn test_strand_length_positive() {
        let cfg = default_strand_config();
        let s = generate_strand_points(
            [0.0, 0.0, 0.0],
            [0.0, -1.0, 0.0],
            &cfg,
            HairProfile::Straight,
        );
        assert!(strand_length(&s) > 0.0);
    }

    #[test]
    fn test_strand_point_count() {
        let cfg = default_strand_config();
        let s = generate_strand_points([0.0, 0.0, 0.0], [0.0, -1.0, 0.0], &cfg, HairProfile::Wavy);
        assert_eq!(strand_point_count(&s), cfg.point_count);
    }

    #[test]
    fn test_set_profile() {
        let mut s = new_hair_strand(HairProfile::Straight);
        set_strand_profile(&mut s, HairProfile::Curly);
        assert_eq!(s.profile, HairProfile::Curly);
    }

    #[test]
    fn test_curl_frequency_straight_zero() {
        let cfg = default_strand_config();
        assert_eq!(curl_frequency(HairProfile::Straight, &cfg), 0.0);
    }

    #[test]
    fn test_curl_amplitude_curly_positive() {
        let cfg = default_strand_config();
        assert!(curl_amplitude(HairProfile::Curly, &cfg) > 0.0);
    }

    #[test]
    fn test_tangent_at_start() {
        let cfg = default_strand_config();
        let s = generate_strand_points(
            [0.0, 0.0, 0.0],
            [0.0, -1.0, 0.0],
            &cfg,
            HairProfile::Straight,
        );
        let tan = strand_tangent_at(&s, 0.0);
        // Should roughly point downward
        assert!(tan[1] < 0.0);
    }

    #[test]
    fn test_tangent_empty_strand() {
        let s = new_hair_strand(HairProfile::Straight);
        let tan = strand_tangent_at(&s, 0.5);
        assert_eq!(tan, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_blend_strands_at_zero() {
        let cfg = default_strand_config();
        let a = generate_strand_points(
            [0.0, 0.0, 0.0],
            [0.0, -1.0, 0.0],
            &cfg,
            HairProfile::Straight,
        );
        let b = generate_strand_points([1.0, 0.0, 0.0], [0.0, -1.0, 0.0], &cfg, HairProfile::Curly);
        let c = blend_strands(&a, &b, 0.0);
        assert_eq!(c.points[0], a.points[0]);
        assert_eq!(c.profile, HairProfile::Straight);
    }

    #[test]
    fn test_bounding_box_non_empty() {
        let cfg = default_strand_config();
        let s = generate_strand_points(
            [0.0, 0.0, 0.0],
            [0.0, -1.0, 0.0],
            &cfg,
            HairProfile::Straight,
        );
        let (min, max) = strand_bounding_box(&s);
        assert!(max[1] >= min[1]);
    }

    #[test]
    fn test_bounding_box_empty() {
        let s = new_hair_strand(HairProfile::Straight);
        let (min, max) = strand_bounding_box(&s);
        assert_eq!(min, [0.0, 0.0, 0.0]);
        assert_eq!(max, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_strand_to_vertices() {
        let cfg = default_strand_config();
        let s = generate_strand_points(
            [0.0, 0.0, 0.0],
            [0.0, -1.0, 0.0],
            &cfg,
            HairProfile::Straight,
        );
        let verts = strand_to_vertices(&s);
        assert_eq!(verts.len(), s.points.len());
    }
}
