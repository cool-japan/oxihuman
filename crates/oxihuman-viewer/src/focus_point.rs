//! Camera focus point management for depth-of-field and look-at targeting.

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum FocusMode {
    Fixed,
    FollowObject,
    AutoDepth,
}

#[allow(dead_code)]
pub struct FocusPoint {
    pub position: [f32; 3],
    pub distance: f32,
    pub mode: FocusMode,
    pub dof_near: f32,
    pub dof_far: f32,
    pub dof_strength: f32,
    pub enabled: bool,
}

#[allow(dead_code)]
pub struct FocusHistory {
    pub points: Vec<[f32; 3]>,
    pub max_points: usize,
}

#[allow(dead_code)]
pub fn new_focus_point(pos: [f32; 3]) -> FocusPoint {
    FocusPoint {
        position: pos,
        distance: 1.0,
        mode: FocusMode::Fixed,
        dof_near: 0.5,
        dof_far: 0.5,
        dof_strength: 1.0,
        enabled: false,
    }
}

#[allow(dead_code)]
pub fn update_focus_distance(fp: &mut FocusPoint, camera_pos: [f32; 3]) {
    let dx = fp.position[0] - camera_pos[0];
    let dy = fp.position[1] - camera_pos[1];
    let dz = fp.position[2] - camera_pos[2];
    fp.distance = (dx * dx + dy * dy + dz * dz).sqrt();
}

#[allow(dead_code)]
pub fn set_focus_position(fp: &mut FocusPoint, pos: [f32; 3]) {
    fp.position = pos;
}

#[allow(dead_code)]
pub fn dof_near_plane(fp: &FocusPoint) -> f32 {
    fp.distance - fp.dof_near
}

#[allow(dead_code)]
pub fn dof_far_plane(fp: &FocusPoint) -> f32 {
    fp.distance + fp.dof_far
}

#[allow(dead_code)]
pub fn focus_in_range(fp: &FocusPoint, distance: f32) -> bool {
    distance >= dof_near_plane(fp) && distance <= dof_far_plane(fp)
}

#[allow(dead_code)]
pub fn set_focus_mode(fp: &mut FocusPoint, mode: FocusMode) {
    fp.mode = mode;
}

#[allow(dead_code)]
pub fn enable_dof(fp: &mut FocusPoint) {
    fp.enabled = true;
}

#[allow(dead_code)]
pub fn disable_dof(fp: &mut FocusPoint) {
    fp.enabled = false;
}

/// Lerp position toward target using speed * dt.
#[allow(dead_code)]
pub fn smooth_focus_to(fp: &mut FocusPoint, target: [f32; 3], speed: f32, dt: f32) {
    let t = (speed * dt).clamp(0.0, 1.0);
    fp.position = [
        fp.position[0] + (target[0] - fp.position[0]) * t,
        fp.position[1] + (target[1] - fp.position[1]) * t,
        fp.position[2] + (target[2] - fp.position[2]) * t,
    ];
}

#[allow(dead_code)]
pub fn new_focus_history(max: usize) -> FocusHistory {
    FocusHistory {
        points: Vec::new(),
        max_points: max,
    }
}

#[allow(dead_code)]
pub fn push_focus(hist: &mut FocusHistory, pos: [f32; 3]) {
    hist.points.push(pos);
    while hist.points.len() > hist.max_points {
        hist.points.remove(0);
    }
}

#[allow(dead_code)]
pub fn average_focus(hist: &FocusHistory) -> [f32; 3] {
    if hist.points.is_empty() {
        return [0.0, 0.0, 0.0];
    }
    let n = hist.points.len() as f32;
    let sum = hist.points.iter().fold([0.0f32; 3], |acc, p| {
        [acc[0] + p[0], acc[1] + p[1], acc[2] + p[2]]
    });
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

#[allow(dead_code)]
pub fn focus_history_len(hist: &FocusHistory) -> usize {
    hist.points.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_focus_point() {
        let fp = new_focus_point([1.0, 2.0, 3.0]);
        assert_eq!(fp.position, [1.0, 2.0, 3.0]);
        assert!(!fp.enabled);
        assert_eq!(fp.mode, FocusMode::Fixed);
    }

    #[test]
    fn test_update_focus_distance() {
        let mut fp = new_focus_point([3.0, 0.0, 0.0]);
        update_focus_distance(&mut fp, [0.0, 0.0, 0.0]);
        assert!((fp.distance - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_set_focus_position() {
        let mut fp = new_focus_point([0.0, 0.0, 0.0]);
        set_focus_position(&mut fp, [5.0, 6.0, 7.0]);
        assert_eq!(fp.position, [5.0, 6.0, 7.0]);
    }

    #[test]
    fn test_dof_near_plane() {
        let mut fp = new_focus_point([0.0, 0.0, 0.0]);
        fp.distance = 10.0;
        fp.dof_near = 2.0;
        assert!((dof_near_plane(&fp) - 8.0).abs() < 1e-5);
    }

    #[test]
    fn test_dof_far_plane() {
        let mut fp = new_focus_point([0.0, 0.0, 0.0]);
        fp.distance = 10.0;
        fp.dof_far = 3.0;
        assert!((dof_far_plane(&fp) - 13.0).abs() < 1e-5);
    }

    #[test]
    fn test_focus_in_range_true() {
        let mut fp = new_focus_point([0.0, 0.0, 0.0]);
        fp.distance = 10.0;
        fp.dof_near = 2.0;
        fp.dof_far = 2.0;
        assert!(focus_in_range(&fp, 10.0));
    }

    #[test]
    fn test_focus_in_range_false() {
        let mut fp = new_focus_point([0.0, 0.0, 0.0]);
        fp.distance = 10.0;
        fp.dof_near = 1.0;
        fp.dof_far = 1.0;
        assert!(!focus_in_range(&fp, 5.0));
    }

    #[test]
    fn test_enable_disable_dof() {
        let mut fp = new_focus_point([0.0, 0.0, 0.0]);
        assert!(!fp.enabled);
        enable_dof(&mut fp);
        assert!(fp.enabled);
        disable_dof(&mut fp);
        assert!(!fp.enabled);
    }

    #[test]
    fn test_smooth_focus_to() {
        let mut fp = new_focus_point([0.0, 0.0, 0.0]);
        smooth_focus_to(&mut fp, [10.0, 0.0, 0.0], 1.0, 0.5);
        assert!(fp.position[0] > 0.0 && fp.position[0] < 10.0);
    }

    #[test]
    fn test_smooth_focus_to_full_step() {
        let mut fp = new_focus_point([0.0, 0.0, 0.0]);
        smooth_focus_to(&mut fp, [5.0, 0.0, 0.0], 10.0, 1.0);
        assert!((fp.position[0] - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_focus_history_push_and_len() {
        let mut hist = new_focus_history(5);
        push_focus(&mut hist, [1.0, 0.0, 0.0]);
        push_focus(&mut hist, [2.0, 0.0, 0.0]);
        assert_eq!(focus_history_len(&hist), 2);
    }

    #[test]
    fn test_focus_history_max_points() {
        let mut hist = new_focus_history(3);
        for i in 0..6 {
            push_focus(&mut hist, [i as f32, 0.0, 0.0]);
        }
        assert_eq!(focus_history_len(&hist), 3);
    }

    #[test]
    fn test_average_focus() {
        let mut hist = new_focus_history(10);
        push_focus(&mut hist, [0.0, 0.0, 0.0]);
        push_focus(&mut hist, [2.0, 0.0, 0.0]);
        push_focus(&mut hist, [4.0, 0.0, 0.0]);
        let avg = average_focus(&hist);
        assert!((avg[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_average_focus_empty() {
        let hist = new_focus_history(5);
        let avg = average_focus(&hist);
        assert_eq!(avg, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_set_focus_mode() {
        let mut fp = new_focus_point([0.0, 0.0, 0.0]);
        set_focus_mode(&mut fp, FocusMode::AutoDepth);
        assert_eq!(fp.mode, FocusMode::AutoDepth);
    }
}
