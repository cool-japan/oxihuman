//! Motion trail rendering (ghost/afterimage effect).

#[allow(dead_code)]
pub struct TrailPoint {
    pub position: [f32; 3],
    pub time: f32,
    pub color: [f32; 4],
    pub width: f32,
}

#[allow(dead_code)]
pub struct MotionTrail {
    pub points: Vec<TrailPoint>,
    pub max_points: usize,
    pub fade_duration: f32,
    pub width_taper: bool,
    pub color_taper: bool,
    pub enabled: bool,
}

#[allow(dead_code)]
pub struct TrailSegment {
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub color: [f32; 4],
    pub width: f32,
}

#[allow(dead_code)]
pub fn new_motion_trail(max_points: usize, fade: f32) -> MotionTrail {
    MotionTrail {
        points: Vec::with_capacity(max_points),
        max_points,
        fade_duration: fade,
        width_taper: true,
        color_taper: true,
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn add_trail_point(trail: &mut MotionTrail, pos: [f32; 3], color: [f32; 4], time: f32) {
    if !trail.enabled {
        return;
    }
    trail.points.push(TrailPoint {
        position: pos,
        time,
        color,
        width: 1.0,
    });
    // Trim to max_points (remove oldest)
    while trail.points.len() > trail.max_points {
        trail.points.remove(0);
    }
}

/// Remove points whose age (current_time - point.time) exceeds fade_duration.
#[allow(dead_code)]
pub fn update_trail(trail: &mut MotionTrail, current_time: f32) {
    let fade = trail.fade_duration;
    trail.points.retain(|p| (current_time - p.time) <= fade);
}

/// Build trail segments from consecutive point pairs.
#[allow(dead_code)]
pub fn trail_segments(trail: &MotionTrail) -> Vec<TrailSegment> {
    if trail.points.len() < 2 {
        return Vec::new();
    }
    let mut segments = Vec::with_capacity(trail.points.len() - 1);
    for i in 0..trail.points.len() - 1 {
        let a = &trail.points[i];
        let b = &trail.points[i + 1];
        segments.push(TrailSegment {
            start: a.position,
            end: b.position,
            color: a.color,
            width: a.width,
        });
    }
    segments
}

#[allow(dead_code)]
pub fn trail_point_count(trail: &MotionTrail) -> usize {
    trail.points.len()
}

/// Sum of Euclidean distances between consecutive points.
#[allow(dead_code)]
pub fn trail_length(trail: &MotionTrail) -> f32 {
    if trail.points.len() < 2 {
        return 0.0;
    }
    let mut total = 0.0_f32;
    for i in 0..trail.points.len() - 1 {
        let a = trail.points[i].position;
        let b = trail.points[i + 1].position;
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let dz = b[2] - a[2];
        total += (dx * dx + dy * dy + dz * dz).sqrt();
    }
    total
}

/// Returns the faded color for the point at `idx`, fading alpha by age.
#[allow(dead_code)]
pub fn trail_color_at(trail: &MotionTrail, idx: usize, current_time: f32) -> [f32; 4] {
    if idx >= trail.points.len() {
        return [0.0, 0.0, 0.0, 0.0];
    }
    let p = &trail.points[idx];
    let mut color = p.color;
    if trail.color_taper && trail.fade_duration > 0.0 {
        let age = (current_time - p.time).max(0.0);
        let fade_factor = 1.0 - (age / trail.fade_duration).clamp(0.0, 1.0);
        color[3] *= fade_factor;
    }
    color
}

/// Returns the tapered width for the point at `idx` (narrows toward older points).
#[allow(dead_code)]
pub fn trail_width_at(trail: &MotionTrail, idx: usize) -> f32 {
    if !trail.width_taper || trail.points.is_empty() {
        return trail.points.get(idx).map(|p| p.width).unwrap_or(1.0);
    }
    let n = trail.points.len();
    if idx >= n {
        return 0.0;
    }
    let base_width = trail.points[idx].width;
    // Newest = index n-1, oldest = index 0
    let t = idx as f32 / (n - 1).max(1) as f32;
    base_width * t
}

#[allow(dead_code)]
pub fn clear_trail(trail: &mut MotionTrail) {
    trail.points.clear();
}

/// Returns the axis-aligned bounding box (min, max) of all trail points.
#[allow(dead_code)]
pub fn trail_aabb(trail: &MotionTrail) -> ([f32; 3], [f32; 3]) {
    if trail.points.is_empty() {
        return ([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
    }
    let mut min = trail.points[0].position;
    let mut max = trail.points[0].position;
    for p in trail.points.iter() {
        for i in 0..3 {
            if p.position[i] < min[i] {
                min[i] = p.position[i];
            }
            if p.position[i] > max[i] {
                max[i] = p.position[i];
            }
        }
    }
    (min, max)
}

#[allow(dead_code)]
pub fn oldest_trail_point(trail: &MotionTrail) -> Option<&TrailPoint> {
    trail.points.first()
}

#[allow(dead_code)]
pub fn newest_trail_point(trail: &MotionTrail) -> Option<&TrailPoint> {
    trail.points.last()
}

#[allow(dead_code)]
pub fn trail_enabled(trail: &MotionTrail) -> bool {
    trail.enabled
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_trail() -> MotionTrail {
        new_motion_trail(10, 5.0)
    }

    #[test]
    fn test_new_motion_trail() {
        let t = make_trail();
        assert_eq!(t.max_points, 10);
        assert!((t.fade_duration - 5.0).abs() < f32::EPSILON);
        assert!(t.enabled);
        assert!(t.points.is_empty());
    }

    #[test]
    fn test_add_trail_point() {
        let mut t = make_trail();
        add_trail_point(&mut t, [1.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0], 0.0);
        assert_eq!(trail_point_count(&t), 1);
    }

    #[test]
    fn test_update_trail_removes_old() {
        let mut t = make_trail();
        add_trail_point(&mut t, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0], 0.0);
        add_trail_point(&mut t, [1.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0], 10.0);
        // At current_time = 20.0, first point age = 20 > fade_duration=5
        // second point age = 10 > 5 too
        update_trail(&mut t, 20.0);
        assert_eq!(trail_point_count(&t), 0);
    }

    #[test]
    fn test_update_trail_keeps_recent() {
        let mut t = make_trail();
        add_trail_point(&mut t, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0], 0.0);
        add_trail_point(&mut t, [1.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0], 2.0);
        // current_time = 4.0: point0 age=4 <= 5, point1 age=2 <= 5
        update_trail(&mut t, 4.0);
        assert_eq!(trail_point_count(&t), 2);
    }

    #[test]
    fn test_trail_segments() {
        let mut t = make_trail();
        add_trail_point(&mut t, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0], 0.0);
        add_trail_point(&mut t, [1.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0], 1.0);
        add_trail_point(&mut t, [2.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0], 2.0);
        let segs = trail_segments(&t);
        assert_eq!(segs.len(), 2);
    }

    #[test]
    fn test_trail_segments_single_point() {
        let mut t = make_trail();
        add_trail_point(&mut t, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0], 0.0);
        let segs = trail_segments(&t);
        assert!(segs.is_empty());
    }

    #[test]
    fn test_trail_length() {
        let mut t = make_trail();
        add_trail_point(&mut t, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0], 0.0);
        add_trail_point(&mut t, [3.0, 4.0, 0.0], [1.0, 1.0, 1.0, 1.0], 1.0);
        let len = trail_length(&t);
        assert!((len - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_trail_length_empty() {
        let t = make_trail();
        assert!((trail_length(&t)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_trail_color_at_fades() {
        let mut t = make_trail();
        // point at time=0, fade_duration=5; at current_time=2.5, age=2.5, fade_factor=0.5
        add_trail_point(&mut t, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0], 0.0);
        let color = trail_color_at(&t, 0, 2.5);
        assert!((color[3] - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_trail_color_at_out_of_range() {
        let t = make_trail();
        let color = trail_color_at(&t, 99, 0.0);
        assert!((color[3]).abs() < f32::EPSILON);
    }

    #[test]
    fn test_trail_width_at() {
        let mut t = make_trail();
        t.width_taper = true;
        add_trail_point(&mut t, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0], 0.0);
        add_trail_point(&mut t, [1.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0], 1.0);
        // idx=0 is oldest: t=0.0, width=0
        let w0 = trail_width_at(&t, 0);
        assert!(w0 < 1.0);
        // idx=1 is newest: t=1.0, width=base_width*1.0
        let w1 = trail_width_at(&t, 1);
        assert!((w1 - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_clear_trail() {
        let mut t = make_trail();
        add_trail_point(&mut t, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0], 0.0);
        clear_trail(&mut t);
        assert_eq!(trail_point_count(&t), 0);
    }

    #[test]
    fn test_trail_aabb() {
        let mut t = make_trail();
        add_trail_point(&mut t, [1.0, 2.0, 3.0], [1.0, 1.0, 1.0, 1.0], 0.0);
        add_trail_point(&mut t, [4.0, 0.0, 5.0], [1.0, 1.0, 1.0, 1.0], 1.0);
        let (min, max) = trail_aabb(&t);
        assert!((min[0] - 1.0).abs() < f32::EPSILON);
        assert!((min[1] - 0.0).abs() < f32::EPSILON);
        assert!((max[0] - 4.0).abs() < f32::EPSILON);
        assert!((max[2] - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_oldest_trail_point() {
        let mut t = make_trail();
        add_trail_point(&mut t, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0], 1.0);
        add_trail_point(&mut t, [1.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0], 2.0);
        let oldest = oldest_trail_point(&t);
        assert!(oldest.is_some());
        assert!((oldest.unwrap().time - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_newest_trail_point() {
        let mut t = make_trail();
        add_trail_point(&mut t, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0], 1.0);
        add_trail_point(&mut t, [1.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0], 2.0);
        let newest = newest_trail_point(&t);
        assert!(newest.is_some());
        assert!((newest.unwrap().time - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_trail_enabled() {
        let t = make_trail();
        assert!(trail_enabled(&t));
    }
}
