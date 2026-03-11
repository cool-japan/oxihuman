//! Advanced pose interpolation: SQUAD, cubic Hermite, tension-continuity-bias (TCB).

#[allow(dead_code)]
pub struct PoseKey {
    pub time: f32,
    pub pose: Vec<f32>, // flat joint rotations
    pub in_tangent: Vec<f32>,
    pub out_tangent: Vec<f32>,
}

#[allow(dead_code)]
pub struct PoseCurve {
    pub keys: Vec<PoseKey>,
    pub interpolation: InterpMode,
}

#[allow(dead_code)]
pub enum InterpMode {
    Linear,
    Cubic,
    Squad,
    Tcb,
}

#[allow(dead_code)]
pub struct TcbParams {
    pub tension: f32,
    pub continuity: f32,
    pub bias: f32,
}

// ── Core lerp ────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn lerp_poses(a: &[f32], b: &[f32], t: f32) -> Vec<f32> {
    let len = a.len().min(b.len());
    (0..len).map(|i| a[i] + (b[i] - a[i]) * t).collect()
}

// ── Cubic Hermite ─────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn cubic_hermite_interp(p0: f32, p1: f32, m0: f32, m1: f32, t: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    (2.0 * t3 - 3.0 * t2 + 1.0) * p0
        + (t3 - 2.0 * t2 + t) * m0
        + (-2.0 * t3 + 3.0 * t2) * p1
        + (t3 - t2) * m1
}

#[allow(dead_code)]
pub fn cubic_hermite_pose(a: &[f32], b: &[f32], ta: &[f32], tb: &[f32], t: f32) -> Vec<f32> {
    let len = a.len().min(b.len()).min(ta.len()).min(tb.len());
    (0..len)
        .map(|i| cubic_hermite_interp(a[i], b[i], ta[i], tb[i], t))
        .collect()
}

// ── Quaternion utilities ───────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn quat_dot(a: [f32; 4], b: [f32; 4]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]
}

#[allow(dead_code)]
pub fn normalize_quat(q: [f32; 4]) -> [f32; 4] {
    let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if len < 1e-9 {
        [0.0, 0.0, 0.0, 1.0]
    } else {
        [q[0] / len, q[1] / len, q[2] / len, q[3] / len]
    }
}

#[allow(dead_code)]
pub fn quat_slerp_interp(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    let mut dot = quat_dot(a, b);
    let bq = if dot < 0.0 {
        dot = -dot;
        [-b[0], -b[1], -b[2], -b[3]]
    } else {
        b
    };

    if dot > 0.9995 {
        let r = [
            a[0] + t * (bq[0] - a[0]),
            a[1] + t * (bq[1] - a[1]),
            a[2] + t * (bq[2] - a[2]),
            a[3] + t * (bq[3] - a[3]),
        ];
        normalize_quat(r)
    } else {
        let theta_0 = dot.acos();
        let theta = theta_0 * t;
        let sin_theta = theta.sin();
        let sin_theta_0 = theta_0.sin();
        let s0 = (theta_0 * (1.0 - t)).sin() / sin_theta_0;
        let s1 = sin_theta / sin_theta_0;
        normalize_quat([
            s0 * a[0] + s1 * bq[0],
            s0 * a[1] + s1 * bq[1],
            s0 * a[2] + s1 * bq[2],
            s0 * a[3] + s1 * bq[3],
        ])
    }
}

#[allow(dead_code)]
pub fn quat_multiply(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    // [x,y,z,w] convention
    let (ax, ay, az, aw) = (a[0], a[1], a[2], a[3]);
    let (bx, by, bz, bw) = (b[0], b[1], b[2], b[3]);
    [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ]
}

fn quat_conjugate(q: [f32; 4]) -> [f32; 4] {
    [-q[0], -q[1], -q[2], q[3]]
}

fn quat_log(q: [f32; 4]) -> [f32; 4] {
    let nq = normalize_quat(q);
    let w = nq[3].clamp(-1.0, 1.0);
    let theta = w.acos();
    let sin_theta = theta.sin();
    if sin_theta.abs() < 1e-9 {
        [0.0, 0.0, 0.0, 0.0]
    } else {
        let s = theta / sin_theta;
        [nq[0] * s, nq[1] * s, nq[2] * s, 0.0]
    }
}

fn quat_exp(q: [f32; 4]) -> [f32; 4] {
    let theta = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt();
    let sin_theta = theta.sin();
    let cos_theta = theta.cos();
    if theta < 1e-9 {
        normalize_quat([0.0, 0.0, 0.0, cos_theta])
    } else {
        let s = sin_theta / theta;
        normalize_quat([q[0] * s, q[1] * s, q[2] * s, cos_theta])
    }
}

// ── SQUAD ─────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn squad_intermediate(q_prev: [f32; 4], q_curr: [f32; 4], q_next: [f32; 4]) -> [f32; 4] {
    // s_i = q_i * exp(-(log(q_i^-1 * q_{i+1}) + log(q_i^-1 * q_{i-1})) / 4)
    let q_inv = quat_conjugate(q_curr);
    let log1 = quat_log(quat_multiply(q_inv, q_next));
    let log2 = quat_log(quat_multiply(q_inv, q_prev));
    let sum = [
        -(log1[0] + log2[0]) / 4.0,
        -(log1[1] + log2[1]) / 4.0,
        -(log1[2] + log2[2]) / 4.0,
        -(log1[3] + log2[3]) / 4.0,
    ];
    normalize_quat(quat_multiply(q_curr, quat_exp(sum)))
}

#[allow(dead_code)]
pub fn squad_quat(q0: [f32; 4], q1: [f32; 4], s0: [f32; 4], s1: [f32; 4], t: f32) -> [f32; 4] {
    let slerp_q = quat_slerp_interp(q0, q1, t);
    let slerp_s = quat_slerp_interp(s0, s1, t);
    quat_slerp_interp(slerp_q, slerp_s, 2.0 * t * (1.0 - t))
}

// ── TCB tangents ──────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn tcb_tangents(keys: &[PoseKey], idx: usize, params: &TcbParams) -> (Vec<f32>, Vec<f32>) {
    let n = keys.len();
    if n == 0 {
        return (Vec::new(), Vec::new());
    }
    let dim = keys[idx].pose.len();
    let (tc, c, b) = (params.tension, params.continuity, params.bias);

    if n == 1 || idx == 0 {
        return (Vec::new(), Vec::new());
    }

    let prev = if idx > 0 { idx - 1 } else { 0 };
    let next = if idx + 1 < n { idx + 1 } else { idx };

    // TCB incoming tangent: (1-t)(1+c)(1+b)/2 * (p[i]-p[i-1]) + (1-t)(1-c)(1-b)/2 * (p[i+1]-p[i])
    // TCB outgoing tangent: (1-t)(1+c)(1-b)/2 * (p[i]-p[i-1]) + (1-t)(1-c)(1+b)/2 * (p[i+1]-p[i])
    let a_in = (1.0 - tc) * (1.0 + c) * (1.0 + b) / 2.0;
    let b_in = (1.0 - tc) * (1.0 - c) * (1.0 - b) / 2.0;
    let a_out = (1.0 - tc) * (1.0 + c) * (1.0 - b) / 2.0;
    let b_out = (1.0 - tc) * (1.0 - c) * (1.0 + b) / 2.0;

    let in_t: Vec<f32> = (0..dim)
        .map(|d| {
            let dp = keys[idx].pose[d] - keys[prev].pose[d];
            let dn = keys[next].pose[d] - keys[idx].pose[d];
            a_in * dp + b_in * dn
        })
        .collect();
    let out_t: Vec<f32> = (0..dim)
        .map(|d| {
            let dp = keys[idx].pose[d] - keys[prev].pose[d];
            let dn = keys[next].pose[d] - keys[idx].pose[d];
            a_out * dp + b_out * dn
        })
        .collect();

    (in_t, out_t)
}

// ── Curve operations ──────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn sample_pose_curve(curve: &PoseCurve, time: f32) -> Vec<f32> {
    let keys = &curve.keys;
    if keys.is_empty() {
        return Vec::new();
    }
    if keys.len() == 1 {
        return keys[0].pose.clone();
    }

    // Clamp to range
    if time <= keys[0].time {
        return keys[0].pose.clone();
    }
    if time >= keys[keys.len() - 1].time {
        return keys[keys.len() - 1].pose.clone();
    }

    // Find bracket
    let idx = keys
        .windows(2)
        .position(|w| time >= w[0].time && time < w[1].time)
        .unwrap_or(0);

    let k0 = &keys[idx];
    let k1 = &keys[idx + 1];
    let dt = k1.time - k0.time;
    let t = if dt.abs() < 1e-9 {
        0.0
    } else {
        (time - k0.time) / dt
    };

    match curve.interpolation {
        InterpMode::Linear => lerp_poses(&k0.pose, &k1.pose, t),
        InterpMode::Cubic => {
            cubic_hermite_pose(&k0.pose, &k1.pose, &k0.out_tangent, &k1.in_tangent, t)
        }
        InterpMode::Squad | InterpMode::Tcb => {
            cubic_hermite_pose(&k0.pose, &k1.pose, &k0.out_tangent, &k1.in_tangent, t)
        }
    }
}

#[allow(dead_code)]
pub fn compute_cubic_tangents(keys: &mut [PoseKey]) {
    let n = keys.len();
    if n < 2 {
        return;
    }

    // Catmull-Rom: compute tangents for each key
    let poses: Vec<Vec<f32>> = keys.iter().map(|k| k.pose.clone()).collect();

    for i in 0..n {
        let dim = poses[i].len();
        let tangent: Vec<f32> = (0..dim)
            .map(|d| {
                let prev = if i > 0 { poses[i - 1][d] } else { poses[i][d] };
                let next = if i + 1 < n {
                    poses[i + 1][d]
                } else {
                    poses[i][d]
                };
                0.5 * (next - prev)
            })
            .collect();
        keys[i].in_tangent = tangent.clone();
        keys[i].out_tangent = tangent;
    }
}

#[allow(dead_code)]
pub fn curve_duration(curve: &PoseCurve) -> f32 {
    if curve.keys.is_empty() {
        return 0.0;
    }
    let first = curve.keys[0].time;
    let last = curve.keys[curve.keys.len() - 1].time;
    last - first
}

#[allow(dead_code)]
pub fn add_pose_key(curve: &mut PoseCurve, key: PoseKey) {
    // Insert in sorted order by time
    let pos = curve.keys.partition_point(|k| k.time <= key.time);
    curve.keys.insert(pos, key);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id_quat() -> [f32; 4] {
        [0.0, 0.0, 0.0, 1.0]
    }

    #[test]
    fn test_lerp_at_t0() {
        let a = vec![0.0, 1.0, 2.0];
        let b = vec![1.0, 2.0, 3.0];
        let r = lerp_poses(&a, &b, 0.0);
        assert_eq!(r, a);
    }

    #[test]
    fn test_lerp_at_t1() {
        let a = vec![0.0, 1.0, 2.0];
        let b = vec![1.0, 2.0, 3.0];
        let r = lerp_poses(&a, &b, 1.0);
        assert_eq!(r, b);
    }

    #[test]
    fn test_lerp_at_half() {
        let a = vec![0.0, 0.0];
        let b = vec![2.0, 4.0];
        let r = lerp_poses(&a, &b, 0.5);
        assert!((r[0] - 1.0).abs() < 1e-6);
        assert!((r[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_cubic_hermite_at_t0() {
        let v = cubic_hermite_interp(1.0, 2.0, 0.0, 0.0, 0.0);
        assert!((v - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_cubic_hermite_at_t1() {
        let v = cubic_hermite_interp(1.0, 2.0, 0.0, 0.0, 1.0);
        assert!((v - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_cubic_hermite_pose_length() {
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![1.0, 1.0, 1.0];
        let ta = vec![0.5, 0.5, 0.5];
        let tb = vec![0.5, 0.5, 0.5];
        let r = cubic_hermite_pose(&a, &b, &ta, &tb, 0.5);
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn test_squad_returns_normalized() {
        let q0 = id_quat();
        let q1 = [0.0, 0.0, 0.707, 0.707];
        let s0 = squad_intermediate(q0, q0, q1);
        let s1 = squad_intermediate(q0, q1, q0);
        let result = squad_quat(q0, q1, s0, s1, 0.5);
        let len = result.iter().map(|v| v * v).sum::<f32>().sqrt();
        assert!((len - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_quat_slerp_at_endpoints() {
        let a = id_quat();
        // Use a properly normalized quaternion for b
        let b = [
            0.0_f32,
            0.0,
            std::f32::consts::FRAC_1_SQRT_2,
            std::f32::consts::FRAC_1_SQRT_2,
        ];
        let r0 = quat_slerp_interp(a, b, 0.0);
        let r1 = quat_slerp_interp(a, b, 1.0);
        for i in 0..4 {
            assert!((r0[i] - a[i]).abs() < 1e-3);
            assert!((r1[i] - b[i]).abs() < 1e-3);
        }
    }

    #[test]
    fn test_normalize_quat() {
        let q = [1.0, 0.0, 0.0, 0.0];
        let n = normalize_quat(q);
        let len: f32 = n.iter().map(|v| v * v).sum::<f32>().sqrt();
        assert!((len - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_quat_dot() {
        let q = id_quat();
        assert!((quat_dot(q, q) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_quat_multiply_identity() {
        let q = [0.0, 0.0, 0.5, 0.866];
        let id = id_quat();
        let r = quat_multiply(q, id);
        for i in 0..4 {
            assert!((r[i] - q[i]).abs() < 1e-5);
        }
    }

    #[test]
    fn test_sample_curve_before_start() {
        let key = PoseKey {
            time: 1.0,
            pose: vec![1.0, 2.0],
            in_tangent: vec![0.0, 0.0],
            out_tangent: vec![0.0, 0.0],
        };
        let curve = PoseCurve {
            keys: vec![key],
            interpolation: InterpMode::Linear,
        };
        let result = sample_pose_curve(&curve, 0.0);
        assert_eq!(result, vec![1.0, 2.0]);
    }

    #[test]
    fn test_sample_curve_linear() {
        let k0 = PoseKey {
            time: 0.0,
            pose: vec![0.0],
            in_tangent: vec![0.0],
            out_tangent: vec![0.0],
        };
        let k1 = PoseKey {
            time: 1.0,
            pose: vec![1.0],
            in_tangent: vec![0.0],
            out_tangent: vec![0.0],
        };
        let curve = PoseCurve {
            keys: vec![k0, k1],
            interpolation: InterpMode::Linear,
        };
        let r = sample_pose_curve(&curve, 0.5);
        assert!((r[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_curve_duration() {
        let k0 = PoseKey {
            time: 0.0,
            pose: vec![0.0],
            in_tangent: vec![],
            out_tangent: vec![],
        };
        let k1 = PoseKey {
            time: 2.0,
            pose: vec![1.0],
            in_tangent: vec![],
            out_tangent: vec![],
        };
        let curve = PoseCurve {
            keys: vec![k0, k1],
            interpolation: InterpMode::Linear,
        };
        assert!((curve_duration(&curve) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_add_pose_key_sorted() {
        let mut curve = PoseCurve {
            keys: Vec::new(),
            interpolation: InterpMode::Linear,
        };
        add_pose_key(
            &mut curve,
            PoseKey {
                time: 1.0,
                pose: vec![1.0],
                in_tangent: vec![],
                out_tangent: vec![],
            },
        );
        add_pose_key(
            &mut curve,
            PoseKey {
                time: 0.0,
                pose: vec![0.0],
                in_tangent: vec![],
                out_tangent: vec![],
            },
        );
        assert!(curve.keys[0].time <= curve.keys[1].time);
    }

    #[test]
    fn test_tcb_tangents_single_key() {
        let key = PoseKey {
            time: 0.0,
            pose: vec![1.0, 2.0],
            in_tangent: vec![],
            out_tangent: vec![],
        };
        let params = TcbParams {
            tension: 0.0,
            continuity: 0.0,
            bias: 0.0,
        };
        let (inn, out) = tcb_tangents(&[key], 0, &params);
        assert_eq!(inn.len(), 0);
        assert_eq!(out.len(), 0);
    }

    #[test]
    fn test_compute_cubic_tangents() {
        let mut keys = vec![
            PoseKey {
                time: 0.0,
                pose: vec![0.0, 0.0],
                in_tangent: vec![],
                out_tangent: vec![],
            },
            PoseKey {
                time: 1.0,
                pose: vec![1.0, 2.0],
                in_tangent: vec![],
                out_tangent: vec![],
            },
            PoseKey {
                time: 2.0,
                pose: vec![0.0, 0.0],
                in_tangent: vec![],
                out_tangent: vec![],
            },
        ];
        compute_cubic_tangents(&mut keys);
        // Middle key should have non-zero tangents
        assert_eq!(keys[1].in_tangent.len(), 2);
        assert_eq!(keys[1].out_tangent.len(), 2);
    }
}
