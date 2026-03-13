//! Point set operations on mesh vertices.
#![allow(dead_code)]

/// A point set for spatial queries.
#[allow(dead_code)]
pub struct PointSet {
    pub points: Vec<[f32; 3]>,
}

/// Create a new, empty point set.
#[allow(dead_code)]
pub fn new_point_set() -> PointSet {
    PointSet { points: Vec::new() }
}

/// Add a point to the set.
#[allow(dead_code)]
pub fn add_point(set: &mut PointSet, p: [f32; 3]) {
    set.points.push(p);
}

/// Find the nearest point to a query point. Returns its index.
#[allow(dead_code)]
pub fn nearest_point(set: &PointSet, q: [f32; 3]) -> Option<usize> {
    if set.points.is_empty() { return None; }
    let mut best = 0;
    let mut best_dist = f32::MAX;
    for (i, &p) in set.points.iter().enumerate() {
        let dx = p[0]-q[0]; let dy = p[1]-q[1]; let dz = p[2]-q[2];
        let d = dx*dx + dy*dy + dz*dz;
        if d < best_dist { best_dist = d; best = i; }
    }
    Some(best)
}

/// Return the number of points in the set.
#[allow(dead_code)]
pub fn point_count(set: &PointSet) -> usize {
    set.points.len()
}

/// Compute the centroid of all points.
#[allow(dead_code)]
pub fn centroid(set: &PointSet) -> [f32; 3] {
    let n = set.points.len();
    if n == 0 { return [0.0; 3]; }
    let sum = set.points.iter().fold([0.0f32;3], |acc, p| [acc[0]+p[0], acc[1]+p[1], acc[2]+p[2]]);
    [sum[0]/n as f32, sum[1]/n as f32, sum[2]/n as f32]
}

/// Find all points within `radius` of query point `q`.
#[allow(dead_code)]
pub fn points_in_radius(set: &PointSet, q: [f32; 3], radius: f32) -> Vec<usize> {
    let r2 = radius * radius;
    set.points.iter().enumerate().filter_map(|(i, &p)| {
        let dx = p[0]-q[0]; let dy = p[1]-q[1]; let dz = p[2]-q[2];
        if dx*dx+dy*dy+dz*dz <= r2 { Some(i) } else { None }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_set() -> PointSet {
        let mut s = new_point_set();
        add_point(&mut s, [0.0, 0.0, 0.0]);
        add_point(&mut s, [1.0, 0.0, 0.0]);
        add_point(&mut s, [0.0, 1.0, 0.0]);
        s
    }

    #[test]
    fn test_new_point_set_empty() {
        let s = new_point_set();
        assert_eq!(point_count(&s), 0);
    }

    #[test]
    fn test_add_point() {
        let mut s = new_point_set();
        add_point(&mut s, [1.0, 2.0, 3.0]);
        assert_eq!(point_count(&s), 1);
    }

    #[test]
    fn test_nearest_point_empty() {
        let s = new_point_set();
        assert!(nearest_point(&s, [0.0,0.0,0.0]).is_none());
    }

    #[test]
    fn test_nearest_point() {
        let s = sample_set();
        let idx = nearest_point(&s, [0.9, 0.0, 0.0]).expect("should succeed");
        assert_eq!(idx, 1);
    }

    #[test]
    fn test_nearest_point_origin() {
        let s = sample_set();
        let idx = nearest_point(&s, [0.0,0.0,0.0]).expect("should succeed");
        assert_eq!(idx, 0);
    }

    #[test]
    fn test_centroid() {
        let s = sample_set();
        let c = centroid(&s);
        assert!((c[0] - 1.0/3.0).abs() < 1e-5);
    }

    #[test]
    fn test_centroid_empty() {
        let s = new_point_set();
        let c = centroid(&s);
        assert!((c[0]).abs() < 1e-5);
    }

    #[test]
    fn test_points_in_radius() {
        let s = sample_set();
        let r = points_in_radius(&s, [0.0,0.0,0.0], 0.5);
        assert!(r.contains(&0));
        assert!(!r.contains(&1));
    }

    #[test]
    fn test_points_in_radius_all() {
        let s = sample_set();
        let r = points_in_radius(&s, [0.0,0.0,0.0], 10.0);
        assert_eq!(r.len(), 3);
    }
}
