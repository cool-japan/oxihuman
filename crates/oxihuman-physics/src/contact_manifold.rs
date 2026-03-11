//! Contact manifold — collection of contact points between two colliding bodies.
//!
//! A manifold groups up to `max_contacts` contact points and provides helpers
//! for computing average normals, maximum penetration depth, and reducing
//! the point set when the limit is exceeded.

// ---------------------------------------------------------------------------
// Config / structs
// ---------------------------------------------------------------------------

/// Configuration for a [`ContactManifold`].
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ManifoldConfig {
    /// Maximum number of contact points to retain.
    pub max_contacts: usize,
    /// Minimum penetration depth to keep a contact.
    pub min_depth: f32,
}

/// A single contact point.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct ContactPoint {
    /// World-space position of the contact.
    pub position: [f32; 3],
    /// Contact normal pointing from body B toward body A.
    pub normal: [f32; 3],
    /// Penetration depth (positive = overlapping).
    pub depth: f32,
}

/// A contact manifold holding contact points between two bodies.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ContactManifold {
    /// ID of the first body.
    pub body_a: u32,
    /// ID of the second body.
    pub body_b: u32,
    /// Contact points.
    pub contacts: Vec<ContactPoint>,
    /// Configuration.
    pub cfg: ManifoldConfig,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Returns a [`ManifoldConfig`] with sensible defaults.
#[allow(dead_code)]
pub fn default_manifold_config() -> ManifoldConfig {
    ManifoldConfig { max_contacts: 4, min_depth: 0.0 }
}

/// Create a new empty [`ContactManifold`] for the given body pair.
#[allow(dead_code)]
pub fn new_contact_manifold(body_a: u32, body_b: u32, cfg: &ManifoldConfig) -> ContactManifold {
    ContactManifold {
        body_a,
        body_b,
        contacts: Vec::new(),
        cfg: cfg.clone(),
    }
}

/// Add a contact point to the manifold, then reduce if over capacity.
#[allow(dead_code)]
pub fn manifold_add_contact(
    manifold: &mut ContactManifold,
    point: [f32; 3],
    normal: [f32; 3],
    depth: f32,
) {
    if depth < manifold.cfg.min_depth {
        return;
    }
    let cp = ContactPoint { position: point, normal, depth };
    manifold.contacts.push(cp);
    if manifold.contacts.len() > manifold.cfg.max_contacts {
        manifold_reduce(manifold);
    }
}

/// Returns the number of contact points.
#[allow(dead_code)]
pub fn manifold_contact_count(manifold: &ContactManifold) -> usize {
    manifold.contacts.len()
}

/// Returns the average contact normal (unit length), or zero if no contacts.
#[allow(dead_code)]
pub fn manifold_average_normal(manifold: &ContactManifold) -> [f32; 3] {
    if manifold.contacts.is_empty() {
        return [0.0, 0.0, 0.0];
    }
    let mut sum = [0.0f32; 3];
    for cp in &manifold.contacts {
        sum[0] += cp.normal[0];
        sum[1] += cp.normal[1];
        sum[2] += cp.normal[2];
    }
    let len = (sum[0] * sum[0] + sum[1] * sum[1] + sum[2] * sum[2]).sqrt();
    if len < 1e-10 {
        return [0.0, 0.0, 0.0];
    }
    [sum[0] / len, sum[1] / len, sum[2] / len]
}

/// Returns the maximum penetration depth across all contact points.
#[allow(dead_code)]
pub fn manifold_max_depth(manifold: &ContactManifold) -> f32 {
    manifold.contacts.iter().map(|cp| cp.depth).fold(f32::NEG_INFINITY, f32::max)
}

/// Reduce the manifold to `max_contacts` by removing the shallowest contacts.
///
/// The point with maximum depth is always retained.  The rest are chosen to
/// maximise the spread (area of the convex hull of the retained set).  This
/// implementation uses a greedy approximation: after the deepest point, pick
/// the one furthest from all already-chosen points.
#[allow(dead_code)]
pub fn manifold_reduce(manifold: &mut ContactManifold) {
    let max_c = manifold.cfg.max_contacts;
    if manifold.contacts.len() <= max_c {
        return;
    }

    // Always keep the deepest contact
    let deepest_idx = manifold
        .contacts
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.depth.partial_cmp(&b.depth).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0);

    let mut kept: Vec<usize> = vec![deepest_idx];

    while kept.len() < max_c {
        let mut best_idx = 0usize;
        let mut best_dist = f32::NEG_INFINITY;
        for (i, cp) in manifold.contacts.iter().enumerate() {
            if kept.contains(&i) {
                continue;
            }
            // Min distance to any already-kept point
            let min_d = kept
                .iter()
                .map(|&k| {
                    let kp = manifold.contacts[k].position;
                    dist_sq(cp.position, kp)
                })
                .fold(f32::INFINITY, f32::min);
            if min_d > best_dist {
                best_dist = min_d;
                best_idx = i;
            }
        }
        kept.push(best_idx);
    }

    let new_contacts: Vec<ContactPoint> = kept.iter().map(|&i| manifold.contacts[i]).collect();
    manifold.contacts = new_contacts;
}

/// Returns true if the manifold has at least one contact point.
#[allow(dead_code)]
pub fn manifold_is_valid(manifold: &ContactManifold) -> bool {
    !manifold.contacts.is_empty()
}

/// Returns the body pair `(body_a, body_b)`.
#[allow(dead_code)]
pub fn manifold_bodies(manifold: &ContactManifold) -> (u32, u32) {
    (manifold.body_a, manifold.body_b)
}

/// Remove all contact points from the manifold.
#[allow(dead_code)]
pub fn manifold_clear(manifold: &mut ContactManifold) {
    manifold.contacts.clear();
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn dist_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_manifold() -> ContactManifold {
        let cfg = default_manifold_config();
        new_contact_manifold(1, 2, &cfg)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_manifold_config();
        assert_eq!(cfg.max_contacts, 4);
    }

    #[test]
    fn test_new_manifold_empty() {
        let m = make_manifold();
        assert_eq!(manifold_contact_count(&m), 0);
        assert!(!manifold_is_valid(&m));
    }

    #[test]
    fn test_add_contact() {
        let mut m = make_manifold();
        manifold_add_contact(&mut m, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.1);
        assert_eq!(manifold_contact_count(&m), 1);
        assert!(manifold_is_valid(&m));
    }

    #[test]
    fn test_manifold_bodies() {
        let cfg = default_manifold_config();
        let m = new_contact_manifold(7, 13, &cfg);
        assert_eq!(manifold_bodies(&m), (7, 13));
    }

    #[test]
    fn test_average_normal() {
        let mut m = make_manifold();
        manifold_add_contact(&mut m, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.1);
        manifold_add_contact(&mut m, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.2);
        let avg = manifold_average_normal(&m);
        assert!((avg[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_max_depth() {
        let mut m = make_manifold();
        manifold_add_contact(&mut m, [0.0; 3], [0.0, 1.0, 0.0], 0.1);
        manifold_add_contact(&mut m, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.5);
        assert!((manifold_max_depth(&m) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_reduce_keeps_max_contacts() {
        let cfg = ManifoldConfig { max_contacts: 2, min_depth: 0.0 };
        let mut m = new_contact_manifold(0, 1, &cfg);
        for i in 0..6u32 {
            manifold_add_contact(
                &mut m,
                [i as f32, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                0.1 * i as f32,
            );
        }
        assert!(manifold_contact_count(&m) <= 2);
    }

    #[test]
    fn test_clear() {
        let mut m = make_manifold();
        manifold_add_contact(&mut m, [0.0; 3], [0.0, 1.0, 0.0], 0.1);
        manifold_clear(&mut m);
        assert_eq!(manifold_contact_count(&m), 0);
    }

    #[test]
    fn test_average_normal_empty() {
        let m = make_manifold();
        assert_eq!(manifold_average_normal(&m), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_min_depth_filter() {
        let cfg = ManifoldConfig { max_contacts: 4, min_depth: 0.05 };
        let mut m = new_contact_manifold(0, 1, &cfg);
        manifold_add_contact(&mut m, [0.0; 3], [0.0, 1.0, 0.0], 0.01); // below threshold
        manifold_add_contact(&mut m, [0.0; 3], [0.0, 1.0, 0.0], 0.1);  // accepted
        assert_eq!(manifold_contact_count(&m), 1);
    }
}
