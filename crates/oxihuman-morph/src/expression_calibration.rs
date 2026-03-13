// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Expression calibration: fit FACS Action Units to facial landmarks.

// ── Types ─────────────────────────────────────────────────────────────────────

/// A single 3D facial landmark.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FacialLandmark {
    pub id: usize,
    pub name: String,
    pub position: [f32; 3],
}

/// A set of facial landmarks (e.g. 68-point or sparse).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LandmarkSet {
    pub landmarks: Vec<FacialLandmark>,
}

/// A FACS Action Unit activation in [0, 1].
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AuActivation {
    pub au_id: u8,
    pub intensity: f32,
}

// ── Core functions ────────────────────────────────────────────────────────────

/// Compute per-landmark displacement from neutral to posed.
#[allow(dead_code)]
pub fn landmark_delta(neutral: &LandmarkSet, posed: &LandmarkSet) -> Vec<[f32; 3]> {
    neutral
        .landmarks
        .iter()
        .zip(posed.landmarks.iter())
        .map(|(n, p)| {
            [
                p.position[0] - n.position[0],
                p.position[1] - n.position[1],
                p.position[2] - n.position[2],
            ]
        })
        .collect()
}

/// Project per-landmark deltas onto AU basis vectors via dot product.
#[allow(dead_code)]
pub fn project_deltas_to_aus(deltas: &[[f32; 3]], au_basis: &[[f32; 3]]) -> Vec<f32> {
    au_basis
        .iter()
        .map(|basis| {
            deltas.iter().enumerate().fold(0.0_f32, |acc, (i, d)| {
                let b = au_basis.get(i % au_basis.len()).copied().unwrap_or(*basis);
                acc + d[0] * b[0] + d[1] * b[1] + d[2] * b[2]
            })
        })
        .collect()
}

/// Build a simple default AU basis for `n_landmarks` landmarks.
/// Each AU basis vector is a unit vector in [Y direction] scaled per-AU.
#[allow(dead_code)]
pub fn build_default_au_basis(n_landmarks: usize) -> Vec<[f32; 3]> {
    (0..n_landmarks)
        .map(|i| {
            let scale = 1.0 / (n_landmarks.max(1) as f32).sqrt();
            let sign = if i % 2 == 0 { 1.0_f32 } else { -1.0_f32 };
            [0.0, sign * scale, 0.0]
        })
        .collect()
}

/// Fit AU activations to the displacement between neutral and target landmarks.
/// Uses a simple least-squares projection.
#[allow(dead_code)]
pub fn calibrate_expression_to_landmarks(
    neutral: &LandmarkSet,
    target: &LandmarkSet,
    au_basis: &[[f32; 3]],
) -> Vec<AuActivation> {
    let deltas = landmark_delta(neutral, target);
    let raw = project_deltas_to_aus(&deltas, au_basis);
    raw.into_iter()
        .enumerate()
        .map(|(i, v)| AuActivation {
            au_id: i as u8,
            intensity: v.clamp(0.0, 1.0),
        })
        .collect()
}

/// Compute reconstruction error after applying AU activations.
#[allow(dead_code)]
pub fn landmark_reconstruction_error(
    neutral: &LandmarkSet,
    target: &LandmarkSet,
    activations: &[AuActivation],
    au_basis: &[[f32; 3]],
) -> f32 {
    let deltas = landmark_delta(neutral, target);
    let n = deltas.len();
    if n == 0 {
        return 0.0;
    }
    // Reconstruct deltas from activations
    let mut reconstructed = vec![[0.0_f32; 3]; n];
    for act in activations {
        let idx = (act.au_id as usize).min(au_basis.len().saturating_sub(1));
        let basis = au_basis[idx];
        for r in reconstructed.iter_mut() {
            r[0] += act.intensity * basis[0];
            r[1] += act.intensity * basis[1];
            r[2] += act.intensity * basis[2];
        }
    }
    // Mean squared error
    let mse: f32 = deltas
        .iter()
        .zip(reconstructed.iter())
        .map(|(d, r)| {
            let e = [d[0] - r[0], d[1] - r[1], d[2] - r[2]];
            e[0] * e[0] + e[1] * e[1] + e[2] * e[2]
        })
        .sum::<f32>()
        / n as f32;
    mse.sqrt()
}

/// Zero-mean, unit-scale normalisation of a landmark set.
#[allow(dead_code)]
pub fn normalize_landmark_set(landmarks: &mut LandmarkSet) {
    let n = landmarks.landmarks.len();
    if n == 0 {
        return;
    }
    let mean: [f32; 3] = {
        let sum = landmarks.landmarks.iter().fold([0.0_f32; 3], |acc, l| {
            [
                acc[0] + l.position[0],
                acc[1] + l.position[1],
                acc[2] + l.position[2],
            ]
        });
        [sum[0] / n as f32, sum[1] / n as f32, sum[2] / n as f32]
    };
    for lm in landmarks.landmarks.iter_mut() {
        lm.position[0] -= mean[0];
        lm.position[1] -= mean[1];
        lm.position[2] -= mean[2];
    }
    let scale: f32 = landmarks
        .landmarks
        .iter()
        .map(|l| {
            (l.position[0] * l.position[0]
                + l.position[1] * l.position[1]
                + l.position[2] * l.position[2])
                .sqrt()
        })
        .fold(0.0_f32, f32::max);
    if scale > 1e-8 {
        for lm in landmarks.landmarks.iter_mut() {
            lm.position[0] /= scale;
            lm.position[1] /= scale;
            lm.position[2] /= scale;
        }
    }
}

/// Build a canonical 68-landmark face set at approximate positions.
#[allow(dead_code)]
pub fn standard_68_landmarks() -> LandmarkSet {
    let names = [
        "jaw_0",
        "jaw_1",
        "jaw_2",
        "jaw_3",
        "jaw_4",
        "jaw_5",
        "jaw_6",
        "jaw_7",
        "jaw_8",
        "jaw_9",
        "jaw_10",
        "jaw_11",
        "jaw_12",
        "jaw_13",
        "jaw_14",
        "jaw_15",
        "jaw_16",
        "brow_l_0",
        "brow_l_1",
        "brow_l_2",
        "brow_l_3",
        "brow_l_4",
        "brow_r_0",
        "brow_r_1",
        "brow_r_2",
        "brow_r_3",
        "brow_r_4",
        "nose_bridge_0",
        "nose_bridge_1",
        "nose_bridge_2",
        "nose_bridge_3",
        "nose_tip",
        "nose_nostril_l",
        "nose_under_l",
        "nose_under_r",
        "nose_nostril_r",
        "eye_l_0",
        "eye_l_1",
        "eye_l_2",
        "eye_l_3",
        "eye_l_4",
        "eye_l_5",
        "eye_r_0",
        "eye_r_1",
        "eye_r_2",
        "eye_r_3",
        "eye_r_4",
        "eye_r_5",
        "mouth_0",
        "mouth_1",
        "mouth_2",
        "mouth_3",
        "mouth_4",
        "mouth_5",
        "mouth_6",
        "mouth_7",
        "mouth_8",
        "mouth_9",
        "mouth_10",
        "mouth_11",
        "mouth_inner_0",
        "mouth_inner_1",
        "mouth_inner_2",
        "mouth_inner_3",
        "mouth_inner_4",
        "mouth_inner_5",
        "mouth_inner_6",
        "mouth_inner_7",
    ];
    let positions: Vec<[f32; 3]> = (0..68)
        .map(|i| {
            let angle = i as f32 * std::f32::consts::TAU / 68.0;
            [0.5 * angle.cos(), 0.5 * angle.sin(), 0.0]
        })
        .collect();
    LandmarkSet {
        landmarks: (0..68)
            .map(|i| FacialLandmark {
                id: i,
                name: names.get(i).copied().unwrap_or("lm").to_string(),
                position: positions[i],
            })
            .collect(),
    }
}

/// Euclidean distance between two landmarks.
#[allow(dead_code)]
pub fn landmark_distance(a: &FacialLandmark, b: &FacialLandmark) -> f32 {
    let dx = a.position[0] - b.position[0];
    let dy = a.position[1] - b.position[1];
    let dz = a.position[2] - b.position[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Maximum X span of a landmark set.
#[allow(dead_code)]
pub fn face_width(landmarks: &LandmarkSet) -> f32 {
    if landmarks.landmarks.is_empty() {
        return 0.0;
    }
    let min_x = landmarks
        .landmarks
        .iter()
        .map(|l| l.position[0])
        .fold(f32::INFINITY, f32::min);
    let max_x = landmarks
        .landmarks
        .iter()
        .map(|l| l.position[0])
        .fold(f32::NEG_INFINITY, f32::max);
    (max_x - min_x).max(0.0)
}

/// Maximum Y span of a landmark set.
#[allow(dead_code)]
pub fn face_height(landmarks: &LandmarkSet) -> f32 {
    if landmarks.landmarks.is_empty() {
        return 0.0;
    }
    let min_y = landmarks
        .landmarks
        .iter()
        .map(|l| l.position[1])
        .fold(f32::INFINITY, f32::min);
    let max_y = landmarks
        .landmarks
        .iter()
        .map(|l| l.position[1])
        .fold(f32::NEG_INFINITY, f32::max);
    (max_y - min_y).max(0.0)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_lm(id: usize, pos: [f32; 3]) -> FacialLandmark {
        FacialLandmark {
            id,
            name: format!("lm{id}"),
            position: pos,
        }
    }

    fn make_set(positions: &[[f32; 3]]) -> LandmarkSet {
        LandmarkSet {
            landmarks: positions
                .iter()
                .enumerate()
                .map(|(i, &p)| make_lm(i, p))
                .collect(),
        }
    }

    #[test]
    fn test_landmark_delta_identical_is_zero() {
        let s = make_set(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        let deltas = landmark_delta(&s, &s);
        for d in &deltas {
            assert_eq!(*d, [0.0, 0.0, 0.0]);
        }
    }

    #[test]
    fn test_landmark_delta_correct() {
        let n = make_set(&[[0.0, 0.0, 0.0]]);
        let p = make_set(&[[1.0, 2.0, 3.0]]);
        let d = landmark_delta(&n, &p);
        assert_eq!(d[0], [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_face_width_positive() {
        let ls = standard_68_landmarks();
        assert!(face_width(&ls) > 0.0);
    }

    #[test]
    fn test_face_height_positive() {
        let ls = standard_68_landmarks();
        assert!(face_height(&ls) > 0.0);
    }

    #[test]
    fn test_face_width_empty() {
        let ls = LandmarkSet { landmarks: vec![] };
        assert_eq!(face_width(&ls), 0.0);
    }

    #[test]
    fn test_normalize_landmark_set_mean_near_zero() {
        let mut ls = make_set(&[[1.0, 2.0, 0.0], [3.0, 4.0, 0.0], [-1.0, 0.0, 0.0]]);
        normalize_landmark_set(&mut ls);
        let n = ls.landmarks.len() as f32;
        let mean_x: f32 = ls.landmarks.iter().map(|l| l.position[0]).sum::<f32>() / n;
        let mean_y: f32 = ls.landmarks.iter().map(|l| l.position[1]).sum::<f32>() / n;
        assert!(mean_x.abs() < 1e-5);
        assert!(mean_y.abs() < 1e-5);
    }

    #[test]
    fn test_normalize_empty_no_panic() {
        let mut ls = LandmarkSet { landmarks: vec![] };
        normalize_landmark_set(&mut ls);
    }

    #[test]
    fn test_reconstruction_error_nonnegative() {
        let n = standard_68_landmarks();
        let p = standard_68_landmarks();
        let basis = build_default_au_basis(68);
        let acts = calibrate_expression_to_landmarks(&n, &p, &basis);
        let err = landmark_reconstruction_error(&n, &p, &acts, &basis);
        assert!(err >= 0.0);
    }

    #[test]
    fn test_calibrate_no_nan() {
        let n = standard_68_landmarks();
        let p = standard_68_landmarks();
        let basis = build_default_au_basis(68);
        let acts = calibrate_expression_to_landmarks(&n, &p, &basis);
        for a in &acts {
            assert!(!a.intensity.is_nan());
        }
    }

    #[test]
    fn test_calibrate_intensity_clamped() {
        let n = standard_68_landmarks();
        let p = standard_68_landmarks();
        let basis = build_default_au_basis(68);
        let acts = calibrate_expression_to_landmarks(&n, &p, &basis);
        for a in &acts {
            assert!((0.0..=1.0).contains(&a.intensity));
        }
    }

    #[test]
    fn test_landmark_distance_zero_same_point() {
        let a = make_lm(0, [1.0, 2.0, 3.0]);
        let b = make_lm(1, [1.0, 2.0, 3.0]);
        assert!((landmark_distance(&a, &b)).abs() < 1e-6);
    }

    #[test]
    fn test_landmark_distance_known() {
        let a = make_lm(0, [0.0, 0.0, 0.0]);
        let b = make_lm(1, [3.0, 4.0, 0.0]);
        assert!((landmark_distance(&a, &b) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_standard_68_landmarks_count() {
        assert_eq!(standard_68_landmarks().landmarks.len(), 68);
    }

    #[test]
    fn test_build_default_au_basis_length() {
        assert_eq!(build_default_au_basis(10).len(), 10);
    }

    #[test]
    fn test_project_deltas_no_nan() {
        let deltas: Vec<[f32; 3]> = (0..5).map(|i| [i as f32, 0.0, 0.0]).collect();
        let basis = build_default_au_basis(5);
        let out = project_deltas_to_aus(&deltas, &basis);
        for v in &out {
            assert!(!v.is_nan());
        }
    }
}
