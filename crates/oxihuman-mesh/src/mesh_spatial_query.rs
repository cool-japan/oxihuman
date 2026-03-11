#![allow(dead_code)]
//! Spatial query structure for point lookups.

/// A query result.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct QueryResult {
    pub index: usize,
    pub distance: f32,
    pub position: [f32; 3],
}

/// A spatial query structure.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SpatialQuery {
    pub points: Vec<[f32; 3]>,
    pub built: bool,
}

/// Create a new spatial query.
#[allow(dead_code)]
pub fn new_spatial_query() -> SpatialQuery {
    SpatialQuery {
        points: Vec::new(),
        built: false,
    }
}

/// Build from points.
#[allow(dead_code)]
pub fn spatial_build(sq: &mut SpatialQuery, points: &[[f32; 3]]) {
    sq.points = points.to_vec();
    sq.built = true;
}

/// Query nearest point.
#[allow(dead_code)]
pub fn query_nearest(sq: &SpatialQuery, point: [f32; 3]) -> Option<QueryResult> {
    if !sq.built || sq.points.is_empty() {
        return None;
    }
    let mut best_idx = 0;
    let mut best_dist = f32::MAX;
    for (i, p) in sq.points.iter().enumerate() {
        let dx = p[0] - point[0];
        let dy = p[1] - point[1];
        let dz = p[2] - point[2];
        let d = (dx * dx + dy * dy + dz * dz).sqrt();
        if d < best_dist {
            best_dist = d;
            best_idx = i;
        }
    }
    Some(QueryResult {
        index: best_idx,
        distance: best_dist,
        position: sq.points[best_idx],
    })
}

/// Query all points within a sphere.
#[allow(dead_code)]
pub fn query_in_sphere(sq: &SpatialQuery, center: [f32; 3], radius: f32) -> Vec<QueryResult> {
    if !sq.built {
        return Vec::new();
    }
    let r2 = radius * radius;
    sq.points
        .iter()
        .enumerate()
        .filter_map(|(i, p)| {
            let dx = p[0] - center[0];
            let dy = p[1] - center[1];
            let dz = p[2] - center[2];
            let d2 = dx * dx + dy * dy + dz * dz;
            if d2 <= r2 {
                Some(QueryResult {
                    index: i,
                    distance: d2.sqrt(),
                    position: *p,
                })
            } else {
                None
            }
        })
        .collect()
}

/// Query all points within an AABB.
#[allow(dead_code)]
pub fn query_in_box(sq: &SpatialQuery, box_min: [f32; 3], box_max: [f32; 3]) -> Vec<QueryResult> {
    if !sq.built {
        return Vec::new();
    }
    sq.points
        .iter()
        .enumerate()
        .filter_map(|(i, p)| {
            if (box_min[0]..=box_max[0]).contains(&p[0])
                && (box_min[1]..=box_max[1]).contains(&p[1])
                && (box_min[2]..=box_max[2]).contains(&p[2])
            {
                Some(QueryResult {
                    index: i,
                    distance: 0.0,
                    position: *p,
                })
            } else {
                None
            }
        })
        .collect()
}

/// Return total point count.
#[allow(dead_code)]
pub fn query_count(sq: &SpatialQuery) -> usize {
    sq.points.len()
}

/// Query first point (by insertion order).
#[allow(dead_code)]
pub fn query_first(sq: &SpatialQuery) -> Option<QueryResult> {
    if sq.points.is_empty() {
        return None;
    }
    Some(QueryResult {
        index: 0,
        distance: 0.0,
        position: sq.points[0],
    })
}

/// Clear the spatial query.
#[allow(dead_code)]
pub fn spatial_clear(sq: &mut SpatialQuery) {
    sq.points.clear();
    sq.built = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_spatial_query() {
        let sq = new_spatial_query();
        assert_eq!(query_count(&sq), 0);
    }

    #[test]
    fn test_spatial_build() {
        let mut sq = new_spatial_query();
        spatial_build(&mut sq, &[[0.0; 3], [1.0, 0.0, 0.0]]);
        assert_eq!(query_count(&sq), 2);
    }

    #[test]
    fn test_query_nearest() {
        let mut sq = new_spatial_query();
        spatial_build(&mut sq, &[[0.0; 3], [10.0, 0.0, 0.0]]);
        let r = query_nearest(&sq, [1.0, 0.0, 0.0]).unwrap();
        assert_eq!(r.index, 0);
    }

    #[test]
    fn test_query_nearest_not_built() {
        let sq = new_spatial_query();
        assert!(query_nearest(&sq, [0.0; 3]).is_none());
    }

    #[test]
    fn test_query_in_sphere() {
        let mut sq = new_spatial_query();
        spatial_build(&mut sq, &[[0.0; 3], [10.0, 0.0, 0.0]]);
        let r = query_in_sphere(&sq, [0.0; 3], 1.0);
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn test_query_in_box() {
        let mut sq = new_spatial_query();
        spatial_build(&mut sq, &[[0.5, 0.5, 0.5], [10.0, 10.0, 10.0]]);
        let r = query_in_box(&sq, [0.0; 3], [1.0, 1.0, 1.0]);
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn test_query_first() {
        let mut sq = new_spatial_query();
        spatial_build(&mut sq, &[[5.0, 5.0, 5.0]]);
        let r = query_first(&sq).unwrap();
        assert_eq!(r.index, 0);
    }

    #[test]
    fn test_query_first_empty() {
        let sq = new_spatial_query();
        assert!(query_first(&sq).is_none());
    }

    #[test]
    fn test_spatial_clear() {
        let mut sq = new_spatial_query();
        spatial_build(&mut sq, &[[0.0; 3]]);
        spatial_clear(&mut sq);
        assert_eq!(query_count(&sq), 0);
        assert!(!sq.built);
    }

    #[test]
    fn test_query_in_sphere_empty() {
        let sq = new_spatial_query();
        assert!(query_in_sphere(&sq, [0.0; 3], 10.0).is_empty());
    }
}
