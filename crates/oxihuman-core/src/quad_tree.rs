// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! 2D quadtree spatial index for point/rect queries.

#![allow(dead_code)]

/// Axis-aligned bounding box in 2D.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb2 {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl Aabb2 {
    #[allow(dead_code)]
    pub fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Aabb2 {
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }

    #[allow(dead_code)]
    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }

    #[allow(dead_code)]
    pub fn intersects(&self, other: &Aabb2) -> bool {
        self.min_x <= other.max_x
            && self.max_x >= other.min_x
            && self.min_y <= other.max_y
            && self.max_y >= other.min_y
    }

    #[allow(dead_code)]
    pub fn center(&self) -> (f32, f32) {
        (
            (self.min_x + self.max_x) * 0.5,
            (self.min_y + self.max_y) * 0.5,
        )
    }
}

/// A point stored in the quadtree with an associated integer id.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct QtPoint {
    pub x: f32,
    pub y: f32,
    pub id: u32,
}

/// Quadtree node.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct QuadTree {
    pub bounds: Aabb2,
    pub depth: u32,
    pub max_depth: u32,
    pub capacity: usize,
    pub points: Vec<QtPoint>,
    pub children: Option<Box<[QuadTree; 4]>>,
}

/// Create a new quadtree with given bounds, max depth, and node capacity.
#[allow(dead_code)]
pub fn new_quad_tree(bounds: Aabb2, max_depth: u32, capacity: usize) -> QuadTree {
    QuadTree {
        bounds,
        depth: 0,
        max_depth,
        capacity,
        points: Vec::new(),
        children: None,
    }
}

fn subdivide(qt: &mut QuadTree) {
    let (cx, cy) = qt.bounds.center();
    let b = &qt.bounds;
    let depth = qt.depth + 1;
    let max_depth = qt.max_depth;
    let capacity = qt.capacity;
    qt.children = Some(Box::new([
        QuadTree {
            bounds: Aabb2::new(b.min_x, b.min_y, cx, cy),
            depth,
            max_depth,
            capacity,
            points: Vec::new(),
            children: None,
        },
        QuadTree {
            bounds: Aabb2::new(cx, b.min_y, b.max_x, cy),
            depth,
            max_depth,
            capacity,
            points: Vec::new(),
            children: None,
        },
        QuadTree {
            bounds: Aabb2::new(b.min_x, cy, cx, b.max_y),
            depth,
            max_depth,
            capacity,
            points: Vec::new(),
            children: None,
        },
        QuadTree {
            bounds: Aabb2::new(cx, cy, b.max_x, b.max_y),
            depth,
            max_depth,
            capacity,
            points: Vec::new(),
            children: None,
        },
    ]));
}

/// Insert a point into the quadtree. Returns false if the point is out of bounds.
#[allow(dead_code)]
pub fn qt_insert(qt: &mut QuadTree, p: QtPoint) -> bool {
    if !qt.bounds.contains_point(p.x, p.y) {
        return false;
    }
    if let Some(ref mut children) = qt.children {
        for child in children.iter_mut() {
            if child.bounds.contains_point(p.x, p.y) {
                return qt_insert(child, p);
            }
        }
        return false;
    }
    qt.points.push(p);
    if qt.points.len() > qt.capacity && qt.depth < qt.max_depth {
        subdivide(qt);
        let pts: Vec<QtPoint> = qt.points.drain(..).collect();
        for pt in pts {
            if let Some(ref mut children) = qt.children {
                for child in children.iter_mut() {
                    if child.bounds.contains_point(pt.x, pt.y) {
                        qt_insert(child, pt);
                        break;
                    }
                }
            }
        }
    }
    true
}

/// Query all points within the given rectangle.
#[allow(dead_code)]
pub fn qt_query_rect(qt: &QuadTree, rect: &Aabb2) -> Vec<QtPoint> {
    let mut result = Vec::new();
    qt_query_rect_into(qt, rect, &mut result);
    result
}

fn qt_query_rect_into(qt: &QuadTree, rect: &Aabb2, out: &mut Vec<QtPoint>) {
    if !qt.bounds.intersects(rect) {
        return;
    }
    for &p in &qt.points {
        if rect.contains_point(p.x, p.y) {
            out.push(p);
        }
    }
    if let Some(ref children) = qt.children {
        for child in children.iter() {
            qt_query_rect_into(child, rect, out);
        }
    }
}

/// Query all points within a circle (center + radius).
#[allow(dead_code)]
pub fn qt_query_circle(qt: &QuadTree, cx: f32, cy: f32, r: f32) -> Vec<QtPoint> {
    let rect = Aabb2::new(cx - r, cy - r, cx + r, cy + r);
    let r2 = r * r;
    qt_query_rect(qt, &rect)
        .into_iter()
        .filter(|p| {
            let dx = p.x - cx;
            let dy = p.y - cy;
            dx * dx + dy * dy <= r2
        })
        .collect()
}

/// Count total points stored in the quadtree.
#[allow(dead_code)]
pub fn qt_count(qt: &QuadTree) -> usize {
    let mut n = qt.points.len();
    if let Some(ref children) = qt.children {
        for child in children.iter() {
            n += qt_count(child);
        }
    }
    n
}

/// Check if the quadtree is empty.
#[allow(dead_code)]
pub fn qt_is_empty(qt: &QuadTree) -> bool {
    qt_count(qt) == 0
}

/// Clear all points from the quadtree (preserves structure).
#[allow(dead_code)]
pub fn qt_clear(qt: &mut QuadTree) {
    qt.points.clear();
    qt.children = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_tree() -> QuadTree {
        let bounds = Aabb2::new(0.0, 0.0, 100.0, 100.0);
        new_quad_tree(bounds, 4, 4)
    }

    #[test]
    fn new_tree_is_empty() {
        let qt = default_tree();
        assert_eq!(qt_count(&qt), 0);
        assert!(qt_is_empty(&qt));
    }

    #[test]
    fn insert_and_count() {
        let mut qt = default_tree();
        assert!(qt_insert(
            &mut qt,
            QtPoint {
                x: 10.0,
                y: 10.0,
                id: 1
            }
        ));
        assert_eq!(qt_count(&qt), 1);
    }

    #[test]
    fn insert_out_of_bounds_rejected() {
        let mut qt = default_tree();
        assert!(!qt_insert(
            &mut qt,
            QtPoint {
                x: 200.0,
                y: 50.0,
                id: 99
            }
        ));
    }

    #[test]
    fn query_rect_finds_point() {
        let mut qt = default_tree();
        qt_insert(
            &mut qt,
            QtPoint {
                x: 50.0,
                y: 50.0,
                id: 7,
            },
        );
        let rect = Aabb2::new(40.0, 40.0, 60.0, 60.0);
        let found = qt_query_rect(&qt, &rect);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, 7);
    }

    #[test]
    fn query_rect_misses_point() {
        let mut qt = default_tree();
        qt_insert(
            &mut qt,
            QtPoint {
                x: 10.0,
                y: 10.0,
                id: 1,
            },
        );
        let rect = Aabb2::new(80.0, 80.0, 100.0, 100.0);
        let found = qt_query_rect(&qt, &rect);
        assert!(found.is_empty());
    }

    #[test]
    fn query_circle_finds_nearby() {
        let mut qt = default_tree();
        qt_insert(
            &mut qt,
            QtPoint {
                x: 50.0,
                y: 50.0,
                id: 42,
            },
        );
        let found = qt_query_circle(&qt, 50.0, 50.0, 5.0);
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn subdivide_on_capacity_overflow() {
        let bounds = Aabb2::new(0.0, 0.0, 100.0, 100.0);
        let mut qt = new_quad_tree(bounds, 4, 2);
        for i in 0..5u32 {
            qt_insert(
                &mut qt,
                QtPoint {
                    x: (i * 10 + 5) as f32,
                    y: (i * 10 + 5) as f32,
                    id: i,
                },
            );
        }
        assert_eq!(qt_count(&qt), 5);
    }

    #[test]
    fn clear_empties_tree() {
        let mut qt = default_tree();
        qt_insert(
            &mut qt,
            QtPoint {
                x: 20.0,
                y: 20.0,
                id: 5,
            },
        );
        qt_clear(&mut qt);
        assert!(qt_is_empty(&qt));
    }

    #[test]
    fn aabb2_intersects() {
        let a = Aabb2::new(0.0, 0.0, 10.0, 10.0);
        let b = Aabb2::new(5.0, 5.0, 15.0, 15.0);
        assert!(a.intersects(&b));
    }

    #[test]
    fn aabb2_no_intersect() {
        let a = Aabb2::new(0.0, 0.0, 5.0, 5.0);
        let b = Aabb2::new(10.0, 10.0, 20.0, 20.0);
        assert!(!a.intersects(&b));
    }
}
