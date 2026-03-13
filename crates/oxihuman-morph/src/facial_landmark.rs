#![allow(dead_code)]
//! Facial landmark positions for morph calibration.

/// A single facial landmark with name and 3-D position.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FacialLandmark {
    /// Landmark name (e.g. "nose_tip").
    pub name: String,
    /// 3-D position [x, y, z].
    pub position: [f32; 3],
    /// Vertex index on the mesh (if mapped).
    pub vertex_index: Option<usize>,
}

/// A set of facial landmarks.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LandmarkSet {
    /// The landmarks in this set.
    pub landmarks: Vec<FacialLandmark>,
    /// Human-readable label.
    pub label: String,
}

/// Create a new empty [`LandmarkSet`].
#[allow(dead_code)]
pub fn new_landmark_set(label: &str) -> LandmarkSet {
    LandmarkSet {
        landmarks: Vec::new(),
        label: label.to_string(),
    }
}

/// Add a landmark.
#[allow(dead_code)]
pub fn add_landmark(set: &mut LandmarkSet, name: &str, position: [f32; 3]) {
    set.landmarks.push(FacialLandmark {
        name: name.to_string(),
        position,
        vertex_index: None,
    });
}

/// Return the number of landmarks.
#[allow(dead_code)]
pub fn landmark_count(set: &LandmarkSet) -> usize {
    set.landmarks.len()
}

/// Get a landmark by index.
#[allow(dead_code)]
pub fn landmark_at(set: &LandmarkSet, index: usize) -> Option<&FacialLandmark> {
    set.landmarks.get(index)
}

/// Find a landmark by name.
#[allow(dead_code)]
pub fn landmark_by_name<'a>(set: &'a LandmarkSet, name: &str) -> Option<&'a FacialLandmark> {
    set.landmarks.iter().find(|l| l.name == name)
}

/// Compute Euclidean distance between two landmarks by index.
#[allow(dead_code)]
pub fn landmark_distance(set: &LandmarkSet, a: usize, b: usize) -> f32 {
    let la = match set.landmarks.get(a) {
        Some(l) => l,
        None => return 0.0,
    };
    let lb = match set.landmarks.get(b) {
        Some(l) => l,
        None => return 0.0,
    };
    let dx = la.position[0] - lb.position[0];
    let dy = la.position[1] - lb.position[1];
    let dz = la.position[2] - lb.position[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Serialize landmarks to a JSON-like string.
#[allow(dead_code)]
pub fn landmarks_to_json(set: &LandmarkSet) -> String {
    let mut s = format!("{{\"label\":\"{}\",\"landmarks\":[", set.label);
    for (i, l) in set.landmarks.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"name\":\"{}\",\"pos\":[{:.4},{:.4},{:.4}]}}",
            l.name, l.position[0], l.position[1], l.position[2]
        ));
    }
    s.push_str("]}");
    s
}

/// Validate landmarks: check for duplicates and NaN positions. Returns true if valid.
#[allow(dead_code)]
pub fn validate_landmarks(set: &LandmarkSet) -> bool {
    for l in &set.landmarks {
        if l.position.iter().any(|v| v.is_nan()) {
            return false;
        }
    }
    // Check for duplicate names
    for (i, a) in set.landmarks.iter().enumerate() {
        for b in set.landmarks.iter().skip(i + 1) {
            if a.name == b.name {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_landmark_set() {
        let s = new_landmark_set("face");
        assert_eq!(s.label, "face");
        assert_eq!(landmark_count(&s), 0);
    }

    #[test]
    fn test_add_landmark() {
        let mut s = new_landmark_set("face");
        add_landmark(&mut s, "nose_tip", [0.0, 0.5, 0.1]);
        assert_eq!(landmark_count(&s), 1);
    }

    #[test]
    fn test_landmark_at() {
        let mut s = new_landmark_set("face");
        add_landmark(&mut s, "chin", [0.0, -0.5, 0.0]);
        let l = landmark_at(&s, 0).expect("should succeed");
        assert_eq!(l.name, "chin");
    }

    #[test]
    fn test_landmark_at_out_of_bounds() {
        let s = new_landmark_set("face");
        assert!(landmark_at(&s, 0).is_none());
    }

    #[test]
    fn test_landmark_by_name() {
        let mut s = new_landmark_set("face");
        add_landmark(&mut s, "left_eye", [0.3, 0.4, 0.05]);
        assert!(landmark_by_name(&s, "left_eye").is_some());
        assert!(landmark_by_name(&s, "right_eye").is_none());
    }

    #[test]
    fn test_landmark_distance() {
        let mut s = new_landmark_set("face");
        add_landmark(&mut s, "a", [0.0, 0.0, 0.0]);
        add_landmark(&mut s, "b", [3.0, 4.0, 0.0]);
        assert!((landmark_distance(&s, 0, 1) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_landmark_distance_missing() {
        let s = new_landmark_set("face");
        assert!((landmark_distance(&s, 0, 1) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_landmarks_to_json() {
        let mut s = new_landmark_set("face");
        add_landmark(&mut s, "tip", [1.0, 2.0, 3.0]);
        let json = landmarks_to_json(&s);
        assert!(json.contains("tip"));
    }

    #[test]
    fn test_validate_landmarks_valid() {
        let mut s = new_landmark_set("face");
        add_landmark(&mut s, "a", [0.0, 0.0, 0.0]);
        add_landmark(&mut s, "b", [1.0, 1.0, 1.0]);
        assert!(validate_landmarks(&s));
    }

    #[test]
    fn test_validate_landmarks_duplicate() {
        let mut s = new_landmark_set("face");
        add_landmark(&mut s, "a", [0.0, 0.0, 0.0]);
        add_landmark(&mut s, "a", [1.0, 1.0, 1.0]);
        assert!(!validate_landmarks(&s));
    }
}
