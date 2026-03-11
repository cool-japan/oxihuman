//! Continuous collision detection (CCD) via AABB sweep tests.
//!
//! Computes the time-of-impact (TOI) between two axis-aligned bounding boxes
//! (AABBs) moving with constant velocities over a given timestep `dt`.
//! The algorithm is based on the separating-axis sweep approach: for each axis
//! find the interval during which the AABBs overlap, then intersect all three
//! intervals to find the earliest contact time.

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Configuration for the CCD sweep solver.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CcdConfig {
    /// Small tolerance added to AABB extents to avoid floating-point gaps.
    pub skin_width: f32,
    /// If the computed TOI is greater than `dt`, the result is "no hit".
    pub max_dt: f32,
}

/// An axis-aligned bounding box used for CCD.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CcdAabb {
    /// Minimum corner (inclusive) of the box.
    pub min: [f32; 3],
    /// Maximum corner (inclusive) of the box.
    pub max: [f32; 3],
}

/// Result of a single CCD sweep query.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CcdSweepResult {
    /// Time of first contact in `[0, dt]`, or `None` if no collision occurs.
    pub toi: Option<f32>,
    /// Axis-aligned normal at the contact point, or `None` if no collision.
    pub normal: Option<[f32; 3]>,
    /// `true` when the bodies were already overlapping at `t = 0`.
    pub initially_overlapping: bool,
}

// ---------------------------------------------------------------------------
// Public functions
// ---------------------------------------------------------------------------

/// Returns the default CCD configuration.
#[allow(dead_code)]
pub fn default_ccd_config() -> CcdConfig {
    CcdConfig { skin_width: 1e-4, max_dt: 1.0 }
}

/// Performs a swept AABB-vs-AABB continuous collision test.
///
/// Both AABBs move at constant velocities `vel_a` / `vel_b` over time `dt`.
/// Returns a [`CcdSweepResult`] with the time-of-impact and contact normal.
#[allow(dead_code)]
pub fn ccd_sweep(
    a: &CcdAabb,
    vel_a: [f32; 3],
    b: &CcdAabb,
    vel_b: [f32; 3],
    dt: f32,
    cfg: &CcdConfig,
) -> CcdSweepResult {
    // Relative velocity of B with respect to A.
    let rel_vel = [vel_b[0] - vel_a[0], vel_b[1] - vel_a[1], vel_b[2] - vel_a[2]];

    let initially_overlapping = aabb_overlap_ccd(a, b);

    let mut t_enter: f32 = 0.0;
    let mut t_exit: f32 = dt;
    let mut contact_axis: usize = 0;
    let mut contact_sign: f32 = 0.0;

    for (axis, &v) in rel_vel.iter().enumerate() {
        let a_min = a.min[axis] - cfg.skin_width;
        let a_max = a.max[axis] + cfg.skin_width;
        let b_min = b.min[axis] - cfg.skin_width;
        let b_max = b.max[axis] + cfg.skin_width;
        // v: relative velocity of B relative to A on this axis

        // Gap between boxes on this axis.
        // Positive gap_lo => B's min is to the right of A's max (B is right of A).
        let gap_lo = b_min - a_max; // gap when B is to the right (+) of A
        let gap_hi = a_min - b_max; // gap when A is to the right (+) of B

        if gap_lo > 0.0 {
            // Separated: B is to the right of A.
            if v >= 0.0 {
                // Moving apart or stationary — no collision on this axis.
                return CcdSweepResult { toi: None, normal: None, initially_overlapping };
            }
            let t0 = gap_lo / (-v);
            let t1 = -gap_hi / v; // = (a_min - b_max) / (-v) when b approaches
            // Recompute t1 properly: overlap ends when B_min overtakes A_min from left.
            // Use: overlap window is [-gap_hi / (-v) ... gap_lo / (-v)] → handled below.
            let t1_corr = if v.abs() > 1e-12 { (gap_lo - (a_max - a_min + b_max - b_min)) / (-v) } else { f32::INFINITY };
            let _ = t1; // suppress unused warning
            if t0 > t_enter {
                t_enter = t0;
                contact_axis = axis;
                contact_sign = -1.0; // B hits A from the right → normal points in -axis
            }
            t_exit = t_exit.min(t1_corr.max(t0));
        } else if gap_hi > 0.0 {
            // Separated: A is to the right of B.
            if v <= 0.0 {
                return CcdSweepResult { toi: None, normal: None, initially_overlapping };
            }
            let t0 = gap_hi / v;
            let t1_corr = if v.abs() > 1e-12 { t0 + (a_max - a_min + b_max - b_min) / v } else { f32::INFINITY };
            if t0 > t_enter {
                t_enter = t0;
                contact_axis = axis;
                contact_sign = 1.0;
            }
            t_exit = t_exit.min(t1_corr);
        }
        // else: already overlapping on this axis — t_enter stays at 0.

        if t_enter > t_exit {
            return CcdSweepResult { toi: None, normal: None, initially_overlapping };
        }
    }

    if t_enter > dt + 1e-9 {
        return CcdSweepResult { toi: None, normal: None, initially_overlapping };
    }

    let toi = t_enter.clamp(0.0, dt);
    let mut normal = [0.0f32; 3];
    normal[contact_axis] = contact_sign;

    CcdSweepResult { toi: Some(toi), normal: Some(normal), initially_overlapping }
}

/// Extracts the time-of-impact from a sweep result.
#[allow(dead_code)]
pub fn ccd_hit_time(result: &CcdSweepResult) -> Option<f32> {
    result.toi
}

/// Extracts the contact normal from a sweep result.
#[allow(dead_code)]
pub fn ccd_hit_normal(result: &CcdSweepResult) -> Option<[f32; 3]> {
    result.normal
}

/// Returns a new `CcdAabb` displaced by `vel * t`.
#[allow(dead_code)]
pub fn ccd_aabb_at_time(aabb: &CcdAabb, vel: [f32; 3], t: f32) -> CcdAabb {
    CcdAabb {
        min: [aabb.min[0] + vel[0] * t, aabb.min[1] + vel[1] * t, aabb.min[2] + vel[2] * t],
        max: [aabb.max[0] + vel[0] * t, aabb.max[1] + vel[1] * t, aabb.max[2] + vel[2] * t],
    }
}

/// Returns `true` if two AABBs currently overlap (at rest, `t = 0`).
#[allow(dead_code)]
pub fn aabb_overlap_ccd(a: &CcdAabb, b: &CcdAabb) -> bool {
    a.min[0] <= b.max[0]
        && a.max[0] >= b.min[0]
        && a.min[1] <= b.max[1]
        && a.max[1] >= b.min[1]
        && a.min[2] <= b.max[2]
        && a.max[2] >= b.min[2]
}

/// Returns `true` if the result indicates tunneling (bodies overlapping at `t = 0`
/// with no finite TOI — i.e. they were interpenetrating from the start).
#[allow(dead_code)]
pub fn ccd_is_tunneling(result: &CcdSweepResult) -> bool {
    result.initially_overlapping && result.toi.is_none()
}

/// Computes the resolved position (centre of `aabb` after moving to TOI, or
/// moved by full `dt` if no collision).
#[allow(dead_code)]
pub fn ccd_resolve_position(aabb: &CcdAabb, vel: [f32; 3], result: &CcdSweepResult) -> [f32; 3] {
    let t = result.toi.unwrap_or(1.0);
    [
        (aabb.min[0] + aabb.max[0]) * 0.5 + vel[0] * t,
        (aabb.min[1] + aabb.max[1]) * 0.5 + vel[1] * t,
        (aabb.min[2] + aabb.max[2]) * 0.5 + vel[2] * t,
    ]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> CcdConfig {
        default_ccd_config()
    }

    fn unit_box(cx: f32, cy: f32, cz: f32) -> CcdAabb {
        CcdAabb {
            min: [cx - 0.5, cy - 0.5, cz - 0.5],
            max: [cx + 0.5, cy + 0.5, cz + 0.5],
        }
    }

    /// Two stationary separated boxes — no collision.
    #[test]
    fn test_stationary_no_collision() {
        let a = unit_box(0.0, 0.0, 0.0);
        let b = unit_box(5.0, 0.0, 0.0);
        let result = ccd_sweep(&a, [0.0; 3], &b, [0.0; 3], 1.0, &cfg());
        assert!(ccd_hit_time(&result).is_none());
    }

    /// Box B moving toward A along X — should collide.
    #[test]
    fn test_head_on_collision_x() {
        let a = unit_box(0.0, 0.0, 0.0);
        let b = unit_box(3.0, 0.0, 0.0);
        // B moves left at speed 4; should reach A well within dt = 1.
        let result = ccd_sweep(&a, [0.0; 3], &b, [-4.0, 0.0, 0.0], 1.0, &cfg());
        assert!(ccd_hit_time(&result).is_some());
        let t = ccd_hit_time(&result).unwrap();
        assert!((0.0..=1.0).contains(&t), "TOI out of range: {t}");
    }

    /// Boxes moving apart — no collision.
    #[test]
    fn test_moving_apart_no_collision() {
        let a = unit_box(0.0, 0.0, 0.0);
        let b = unit_box(3.0, 0.0, 0.0);
        let result = ccd_sweep(&a, [-1.0, 0.0, 0.0], &b, [1.0, 0.0, 0.0], 1.0, &cfg());
        assert!(ccd_hit_time(&result).is_none());
    }

    /// Already-overlapping boxes — `initially_overlapping` is true.
    #[test]
    fn test_initially_overlapping() {
        let a = unit_box(0.0, 0.0, 0.0);
        let b = unit_box(0.3, 0.0, 0.0); // overlapping
        assert!(aabb_overlap_ccd(&a, &b));
        let result = ccd_sweep(&a, [0.0; 3], &b, [0.0; 3], 1.0, &cfg());
        assert!(result.initially_overlapping);
    }

    /// `ccd_aabb_at_time` displaces correctly.
    #[test]
    fn test_aabb_at_time() {
        let a = unit_box(0.0, 0.0, 0.0);
        let moved = ccd_aabb_at_time(&a, [1.0, 2.0, 3.0], 2.0);
        assert!((moved.min[0] - 1.5).abs() < 1e-6);
        assert!((moved.min[1] - 3.5).abs() < 1e-6);
        assert!((moved.min[2] - 5.5).abs() < 1e-6);
    }

    /// `aabb_overlap_ccd` returns false for clearly separated boxes.
    #[test]
    fn test_aabb_no_overlap() {
        let a = unit_box(0.0, 0.0, 0.0);
        let b = unit_box(10.0, 0.0, 0.0);
        assert!(!aabb_overlap_ccd(&a, &b));
    }

    /// `ccd_resolve_position` with no collision returns position at t = 1.
    #[test]
    fn test_resolve_position_no_hit() {
        let a = unit_box(0.0, 0.0, 0.0);
        let result = CcdSweepResult { toi: None, normal: None, initially_overlapping: false };
        let pos = ccd_resolve_position(&a, [2.0, 0.0, 0.0], &result);
        // centre starts at 0, moves +2 over t=1
        assert!((pos[0] - 2.0).abs() < 1e-6);
    }

    /// Perpendicular motion — no collision on X, boxes separated in X.
    #[test]
    fn test_perpendicular_motion_no_collision() {
        let a = unit_box(0.0, 0.0, 0.0);
        let b = unit_box(3.0, 0.0, 0.0); // separated in X
        // B moves only in Y — never closes the X gap.
        let result = ccd_sweep(&a, [0.0; 3], &b, [0.0, 4.0, 0.0], 1.0, &cfg());
        assert!(ccd_hit_time(&result).is_none());
    }
}
