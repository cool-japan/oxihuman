#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct TrackedLandmark {
    name: String,
    position: [f32; 3],
    prev_position: [f32; 3],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyLandmarkTracker {
    landmarks: Vec<TrackedLandmark>,
    dt: f32,
}

#[allow(dead_code)]
pub fn new_landmark_tracker() -> BodyLandmarkTracker {
    BodyLandmarkTracker { landmarks: Vec::new(), dt: 1.0 / 60.0 }
}

#[allow(dead_code)]
pub fn track_landmark(tracker: &mut BodyLandmarkTracker, name: &str, pos: [f32; 3]) {
    if let Some(lm) = tracker.landmarks.iter_mut().find(|l| l.name == name) {
        lm.prev_position = lm.position;
        lm.position = pos;
    } else {
        tracker.landmarks.push(TrackedLandmark { name: name.to_string(), position: pos, prev_position: pos });
    }
}

#[allow(dead_code)]
pub fn landmark_position_blt(tracker: &BodyLandmarkTracker, name: &str) -> [f32; 3] {
    tracker.landmarks.iter().find(|l| l.name == name).map(|l| l.position).unwrap_or([0.0; 3])
}

#[allow(dead_code)]
pub fn landmark_count_blt(tracker: &BodyLandmarkTracker) -> usize { tracker.landmarks.len() }

#[allow(dead_code)]
pub fn landmark_velocity(tracker: &BodyLandmarkTracker, name: &str) -> [f32; 3] {
    tracker.landmarks.iter().find(|l| l.name == name).map(|l| {
        let inv = 1.0 / tracker.dt;
        [(l.position[0] - l.prev_position[0]) * inv,
         (l.position[1] - l.prev_position[1]) * inv,
         (l.position[2] - l.prev_position[2]) * inv]
    }).unwrap_or([0.0; 3])
}

#[allow(dead_code)]
pub fn tracker_to_json(tracker: &BodyLandmarkTracker) -> String {
    format!("{{\"landmark_count\":{}}}", tracker.landmarks.len())
}

#[allow(dead_code)]
pub fn tracker_clear(tracker: &mut BodyLandmarkTracker) { tracker.landmarks.clear(); }

#[allow(dead_code)]
pub fn tracker_update(tracker: &mut BodyLandmarkTracker, dt: f32) { tracker.dt = dt.max(1e-6); }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let t = new_landmark_tracker(); assert_eq!(landmark_count_blt(&t), 0); }
    #[test] fn test_track() { let mut t = new_landmark_tracker(); track_landmark(&mut t, "nose", [0.0, 1.0, 0.0]); assert_eq!(landmark_count_blt(&t), 1); }
    #[test] fn test_position() { let mut t = new_landmark_tracker(); track_landmark(&mut t, "a", [1.0, 2.0, 3.0]); assert!((landmark_position_blt(&t, "a")[0] - 1.0).abs() < 1e-6); }
    #[test] fn test_missing() { let t = new_landmark_tracker(); assert!((landmark_position_blt(&t, "x")[0]).abs() < 1e-6); }
    #[test] fn test_velocity() { let mut t = new_landmark_tracker(); track_landmark(&mut t, "a", [0.0, 0.0, 0.0]); track_landmark(&mut t, "a", [1.0, 0.0, 0.0]); let v = landmark_velocity(&t, "a"); assert!(v[0] > 0.0); }
    #[test] fn test_json() { let t = new_landmark_tracker(); assert!(tracker_to_json(&t).contains("landmark_count")); }
    #[test] fn test_clear() { let mut t = new_landmark_tracker(); track_landmark(&mut t, "a", [0.0; 3]); tracker_clear(&mut t); assert_eq!(landmark_count_blt(&t), 0); }
    #[test] fn test_update() { let mut t = new_landmark_tracker(); tracker_update(&mut t, 0.033); assert!((t.dt - 0.033).abs() < 1e-6); }
    #[test] fn test_velocity_missing() { let t = new_landmark_tracker(); let v = landmark_velocity(&t, "x"); assert!((v[0]).abs() < 1e-6); }
    #[test] fn test_update_position() { let mut t = new_landmark_tracker(); track_landmark(&mut t, "a", [1.0, 0.0, 0.0]); track_landmark(&mut t, "a", [2.0, 0.0, 0.0]); assert!((landmark_position_blt(&t, "a")[0] - 2.0).abs() < 1e-6); }
}
