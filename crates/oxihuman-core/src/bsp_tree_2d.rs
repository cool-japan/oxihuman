// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Binary Space Partitioning (BSP) tree for 2D polygon splitting.

#![allow(dead_code)]

/// A 2D line used as a BSP splitting plane: ax + by + c = 0.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct BspLine {
    pub a: f32,
    pub b: f32,
    pub c: f32,
}

/// Evaluate which side of the line a point is on.
/// Returns positive if in front, negative if behind, ~0 if on line.
#[allow(dead_code)]
pub fn bsp_line_side(line: &BspLine, p: [f32; 2]) -> f32 {
    line.a * p[0] + line.b * p[1] + line.c
}

/// A polygon in the BSP tree.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BspPolygon {
    pub vertices: Vec<[f32; 2]>,
    pub label: String,
}

/// A node in the BSP tree.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum BspNode {
    Leaf(Vec<BspPolygon>),
    Branch {
        splitter: BspLine,
        front: Box<BspNode>,
        back: Box<BspNode>,
    },
}

/// A 2D BSP tree.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct BspTree2D {
    root: Option<Box<BspNode>>,
}

/// Create a new empty BSP tree.
#[allow(dead_code)]
pub fn new_bsp_tree() -> BspTree2D {
    BspTree2D::default()
}

/// Split a polygon by a line into front and back parts.
/// Returns (front_vertices, back_vertices).
#[allow(dead_code)]
pub fn bsp_split_polygon(poly: &[[f32; 2]], line: &BspLine) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let mut front = Vec::new();
    let mut back = Vec::new();
    let n = poly.len();
    if n == 0 {
        return (front, back);
    }
    for i in 0..n {
        let curr = poly[i];
        let next = poly[(i + 1) % n];
        let d_curr = bsp_line_side(line, curr);
        let d_next = bsp_line_side(line, next);
        if d_curr >= 0.0 {
            front.push(curr);
        } else {
            back.push(curr);
        }
        // Check for crossing
        if (d_curr > 0.0 && d_next < 0.0) || (d_curr < 0.0 && d_next > 0.0) {
            let t = d_curr / (d_curr - d_next);
            let ix = curr[0] + t * (next[0] - curr[0]);
            let iy = curr[1] + t * (next[1] - curr[1]);
            front.push([ix, iy]);
            back.push([ix, iy]);
        }
    }
    (front, back)
}

/// Build a BSP tree from a list of polygons using the first polygon's edge as the splitter.
#[allow(dead_code)]
pub fn bsp_build(polygons: Vec<BspPolygon>) -> BspNode {
    if polygons.is_empty() {
        return BspNode::Leaf(Vec::new());
    }
    if polygons.len() == 1 {
        return BspNode::Leaf(polygons);
    }
    // Use first polygon's first edge as splitter.
    let splitter = {
        let verts = &polygons[0].vertices;
        if verts.len() < 2 {
            return BspNode::Leaf(polygons);
        }
        let a = verts[0];
        let b = verts[1];
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let len = (dx * dx + dy * dy).sqrt().max(1e-10);
        BspLine {
            a: -dy / len,
            b: dx / len,
            c: (dy * a[0] - dx * a[1]) / len,
        }
    };

    let mut front_polys = Vec::new();
    let mut back_polys = Vec::new();
    let input_count = polygons.len();

    for poly in polygons {
        let (f, b) = bsp_split_polygon(&poly.vertices, &splitter);
        let f_ok = f.len() >= 3;
        let b_ok = b.len() >= 3;
        if f_ok {
            front_polys.push(BspPolygon {
                vertices: f,
                label: poly.label.clone(),
            });
        }
        if b_ok {
            back_polys.push(BspPolygon {
                vertices: b,
                label: poly.label.clone(),
            });
        }
        if !f_ok && !b_ok {
            front_polys.push(poly);
        }
    }

    // Guard against infinite recursion: if no actual splitting occurred,
    // return a leaf with all polygons.
    if front_polys.len() == input_count && back_polys.is_empty() {
        return BspNode::Leaf(front_polys);
    }
    if back_polys.len() == input_count && front_polys.is_empty() {
        return BspNode::Leaf(back_polys);
    }

    BspNode::Branch {
        splitter,
        front: Box::new(bsp_build(front_polys)),
        back: Box::new(bsp_build(back_polys)),
    }
}

/// Count the total number of polygons in a BSP tree.
#[allow(dead_code)]
pub fn bsp_polygon_count(node: &BspNode) -> usize {
    match node {
        BspNode::Leaf(polys) => polys.len(),
        BspNode::Branch { front, back, .. } => bsp_polygon_count(front) + bsp_polygon_count(back),
    }
}

/// Count the depth of the BSP tree.
#[allow(dead_code)]
pub fn bsp_depth(node: &BspNode) -> usize {
    match node {
        BspNode::Leaf(_) => 0,
        BspNode::Branch { front, back, .. } => 1 + bsp_depth(front).max(bsp_depth(back)),
    }
}

/// Collect all polygons from a BSP tree in front-to-back order.
#[allow(dead_code)]
pub fn bsp_collect_polygons(node: &BspNode) -> Vec<&BspPolygon> {
    let mut result = Vec::new();
    bsp_collect_recursive(node, &mut result);
    result
}

fn bsp_collect_recursive<'a>(node: &'a BspNode, out: &mut Vec<&'a BspPolygon>) {
    match node {
        BspNode::Leaf(polys) => {
            for p in polys {
                out.push(p);
            }
        }
        BspNode::Branch { front, back, .. } => {
            bsp_collect_recursive(front, out);
            bsp_collect_recursive(back, out);
        }
    }
}

/// Check if a BSP tree is a leaf.
#[allow(dead_code)]
pub fn bsp_is_leaf(node: &BspNode) -> bool {
    matches!(node, BspNode::Leaf(_))
}

/// Set the BSP root.
#[allow(dead_code)]
pub fn bsp_set_root(tree: &mut BspTree2D, node: BspNode) {
    tree.root = Some(Box::new(node));
}

/// Get BSP root reference.
#[allow(dead_code)]
pub fn bsp_get_root(tree: &BspTree2D) -> Option<&BspNode> {
    tree.root.as_deref()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_quad(x: f32, y: f32, w: f32, h: f32, label: &str) -> BspPolygon {
        BspPolygon {
            vertices: vec![[x, y], [x + w, y], [x + w, y + h], [x, y + h]],
            label: label.to_string(),
        }
    }

    #[test]
    fn test_new_tree_empty() {
        let tree = new_bsp_tree();
        assert!(bsp_get_root(&tree).is_none());
    }

    #[test]
    fn test_build_single_polygon() {
        let poly = make_quad(0.0, 0.0, 1.0, 1.0, "a");
        let node = bsp_build(vec![poly]);
        assert!(bsp_is_leaf(&node));
    }

    #[test]
    fn test_build_two_polygons() {
        let p1 = make_quad(-1.0, -1.0, 2.0, 2.0, "a");
        let p2 = make_quad(-1.0, 1.0, 2.0, 2.0, "b");
        let node = bsp_build(vec![p1, p2]);
        // May be leaf or branch depending on splitter; just check it builds
        let count = bsp_polygon_count(&node);
        assert!(count >= 1);
    }

    #[test]
    fn test_bsp_line_side() {
        let line = BspLine {
            a: 1.0,
            b: 0.0,
            c: 0.0,
        };
        assert!(bsp_line_side(&line, [1.0, 0.0]) > 0.0);
        assert!(bsp_line_side(&line, [-1.0, 0.0]) < 0.0);
    }

    #[test]
    fn test_split_polygon_horizontal() {
        let line = BspLine {
            a: 0.0,
            b: 1.0,
            c: -0.5,
        };
        let poly = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let (f, b) = bsp_split_polygon(&poly, &line);
        assert!(!f.is_empty());
        assert!(!b.is_empty());
    }

    #[test]
    fn test_collect_polygons() {
        let p1 = make_quad(0.0, 0.0, 1.0, 1.0, "a");
        let p2 = make_quad(2.0, 0.0, 1.0, 1.0, "b");
        let node = bsp_build(vec![p1, p2]);
        let collected = bsp_collect_polygons(&node);
        assert!(!collected.is_empty());
    }

    #[test]
    fn test_bsp_depth_single() {
        let poly = make_quad(0.0, 0.0, 1.0, 1.0, "a");
        let node = bsp_build(vec![poly]);
        assert_eq!(bsp_depth(&node), 0);
    }

    #[test]
    fn test_bsp_set_and_get_root() {
        let mut tree = new_bsp_tree();
        let poly = make_quad(0.0, 0.0, 1.0, 1.0, "a");
        let node = bsp_build(vec![poly]);
        bsp_set_root(&mut tree, node);
        assert!(bsp_get_root(&tree).is_some());
    }

    #[test]
    fn test_empty_build() {
        let node = bsp_build(vec![]);
        assert!(bsp_is_leaf(&node));
        assert_eq!(bsp_polygon_count(&node), 0);
    }
}
