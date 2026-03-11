#![allow(dead_code)]

/// A snapshot of a rigid body's state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyStateSnapshot {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub angular_velocity: [f32; 3],
}

/// Takes a snapshot of the body state.
#[allow(dead_code)]
pub fn take_body_snapshot(
    position: [f32; 3],
    velocity: [f32; 3],
    angular_velocity: [f32; 3],
) -> BodyStateSnapshot {
    BodyStateSnapshot {
        position,
        velocity,
        angular_velocity,
    }
}

/// Restores a body state from a snapshot, returns (position, velocity, angular_velocity).
#[allow(dead_code)]
pub fn restore_body_snapshot(snap: &BodyStateSnapshot) -> ([f32; 3], [f32; 3], [f32; 3]) {
    (snap.position, snap.velocity, snap.angular_velocity)
}

/// Returns the position from the snapshot.
#[allow(dead_code)]
pub fn snapshot_position(snap: &BodyStateSnapshot) -> [f32; 3] {
    snap.position
}

/// Returns the velocity from the snapshot.
#[allow(dead_code)]
pub fn snapshot_velocity(snap: &BodyStateSnapshot) -> [f32; 3] {
    snap.velocity
}

/// Returns the angular velocity from the snapshot.
#[allow(dead_code)]
pub fn snapshot_angular_velocity(snap: &BodyStateSnapshot) -> [f32; 3] {
    snap.angular_velocity
}

/// Serializes the snapshot to JSON.
#[allow(dead_code)]
pub fn snapshot_to_json(snap: &BodyStateSnapshot) -> String {
    format!(
        "{{\"position\":[{},{},{}],\"velocity\":[{},{},{}],\"angular_velocity\":[{},{},{}]}}",
        snap.position[0], snap.position[1], snap.position[2],
        snap.velocity[0], snap.velocity[1], snap.velocity[2],
        snap.angular_velocity[0], snap.angular_velocity[1], snap.angular_velocity[2],
    )
}

/// Returns the squared difference between two snapshots.
#[allow(dead_code)]
pub fn snapshot_diff_sq(a: &BodyStateSnapshot, b: &BodyStateSnapshot) -> f32 {
    let mut diff = 0.0f32;
    for i in 0..3 {
        diff += (a.position[i] - b.position[i]).powi(2);
        diff += (a.velocity[i] - b.velocity[i]).powi(2);
        diff += (a.angular_velocity[i] - b.angular_velocity[i]).powi(2);
    }
    diff
}

/// Returns true if two snapshots are approximately equal.
#[allow(dead_code)]
pub fn snapshots_equal(a: &BodyStateSnapshot, b: &BodyStateSnapshot, tolerance: f32) -> bool {
    snapshot_diff_sq(a, b) < tolerance * tolerance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_take_snapshot() {
        let snap = take_body_snapshot([1.0; 3], [2.0; 3], [3.0; 3]);
        assert!((snap.position[0] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_restore() {
        let snap = take_body_snapshot([1.0; 3], [2.0; 3], [3.0; 3]);
        let (p, v, av) = restore_body_snapshot(&snap);
        assert!((p[0] - 1.0).abs() < f32::EPSILON);
        assert!((v[0] - 2.0).abs() < f32::EPSILON);
        assert!((av[0] - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_position() {
        let snap = take_body_snapshot([5.0, 6.0, 7.0], [0.0; 3], [0.0; 3]);
        let p = snapshot_position(&snap);
        assert!((p[1] - 6.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_velocity() {
        let snap = take_body_snapshot([0.0; 3], [1.0, 2.0, 3.0], [0.0; 3]);
        let v = snapshot_velocity(&snap);
        assert!((v[2] - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_angular_velocity() {
        let snap = take_body_snapshot([0.0; 3], [0.0; 3], [4.0, 5.0, 6.0]);
        let av = snapshot_angular_velocity(&snap);
        assert!((av[0] - 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_to_json() {
        let snap = take_body_snapshot([1.0; 3], [0.0; 3], [0.0; 3]);
        let json = snapshot_to_json(&snap);
        assert!(json.contains("\"position\""));
    }

    #[test]
    fn test_diff_sq_zero() {
        let a = take_body_snapshot([1.0; 3], [2.0; 3], [3.0; 3]);
        let b = take_body_snapshot([1.0; 3], [2.0; 3], [3.0; 3]);
        assert!(snapshot_diff_sq(&a, &b).abs() < f32::EPSILON);
    }

    #[test]
    fn test_diff_sq_nonzero() {
        let a = take_body_snapshot([0.0; 3], [0.0; 3], [0.0; 3]);
        let b = take_body_snapshot([1.0; 3], [0.0; 3], [0.0; 3]);
        assert!(snapshot_diff_sq(&a, &b) > 0.0);
    }

    #[test]
    fn test_snapshots_equal() {
        let a = take_body_snapshot([1.0; 3], [2.0; 3], [3.0; 3]);
        let b = take_body_snapshot([1.0; 3], [2.0; 3], [3.0; 3]);
        assert!(snapshots_equal(&a, &b, 0.001));
    }

    #[test]
    fn test_snapshots_not_equal() {
        let a = take_body_snapshot([0.0; 3], [0.0; 3], [0.0; 3]);
        let b = take_body_snapshot([10.0; 3], [0.0; 3], [0.0; 3]);
        assert!(!snapshots_equal(&a, &b, 0.001));
    }
}
