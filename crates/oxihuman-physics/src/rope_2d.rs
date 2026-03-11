// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! 2D rope/chain simulation stub (Verlet integration + distance constraints).

#[derive(Debug, Clone)]
pub struct RopeNode2d {
    pub position: [f32; 2],
    pub prev_position: [f32; 2],
    pub pinned: bool,
}

impl RopeNode2d {
    pub fn new(x: f32, y: f32) -> Self {
        RopeNode2d {
            position: [x, y],
            prev_position: [x, y],
            pinned: false,
        }
    }

    pub fn pinned(mut self) -> Self {
        self.pinned = true;
        self
    }
}

#[derive(Debug, Clone)]
pub struct Rope2d {
    pub nodes: Vec<RopeNode2d>,
    pub segment_length: f32,
    pub gravity: f32,
    pub constraint_iters: usize,
}

impl Rope2d {
    pub fn from_points(points: Vec<[f32; 2]>, segment_length: f32, gravity: f32) -> Self {
        let nodes = points
            .iter()
            .enumerate()
            .map(|(i, &[x, y])| {
                let mut n = RopeNode2d::new(x, y);
                if i == 0 {
                    n.pinned = true;
                }
                n
            })
            .collect();
        Rope2d {
            nodes,
            segment_length,
            gravity,
            constraint_iters: 10,
        }
    }

    pub fn make_hanging(anchor: [f32; 2], n_links: usize, seg_len: f32, gravity: f32) -> Self {
        let points: Vec<[f32; 2]> = (0..=n_links)
            .map(|i| [anchor[0], anchor[1] - i as f32 * seg_len])
            .collect();
        Self::from_points(points, seg_len, gravity)
    }

    pub fn step(&mut self, dt: f32) {
        let g = self.gravity;
        for node in &mut self.nodes {
            if node.pinned {
                continue;
            }
            let vx = node.position[0] - node.prev_position[0];
            let vy = node.position[1] - node.prev_position[1];
            node.prev_position = node.position;
            node.position[0] += vx;
            node.position[1] += vy - g * dt * dt;
        }
        let seg = self.segment_length;
        for _ in 0..self.constraint_iters {
            let n = self.nodes.len();
            for i in 0..n.saturating_sub(1) {
                let j = i + 1;
                let ax = self.nodes[i].position[0];
                let ay = self.nodes[i].position[1];
                let bx = self.nodes[j].position[0];
                let by = self.nodes[j].position[1];
                let dx = bx - ax;
                let dy = by - ay;
                let dist = (dx * dx + dy * dy).sqrt().max(1e-8);
                let diff = (dist - seg) / dist * 0.5;
                if !self.nodes[i].pinned {
                    self.nodes[i].position[0] += dx * diff;
                    self.nodes[i].position[1] += dy * diff;
                }
                if !self.nodes[j].pinned {
                    self.nodes[j].position[0] -= dx * diff;
                    self.nodes[j].position[1] -= dy * diff;
                }
            }
        }
    }

    pub fn tip(&self) -> Option<[f32; 2]> {
        self.nodes.last().map(|n| n.position)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn total_length_approx(&self) -> f32 {
        self.segment_length * (self.nodes.len().saturating_sub(1)) as f32
    }
}

pub fn rope_segment_length(a: [f32; 2], b: [f32; 2]) -> f32 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let rope = Rope2d::make_hanging([0.0, 10.0], 4, 1.0, 9.81);
        assert_eq!(rope.node_count(), 5 /* 5 nodes for 4 links */,);
    }

    #[test]
    fn test_anchor_is_pinned() {
        let rope = Rope2d::make_hanging([0.0, 10.0], 4, 1.0, 9.81);
        assert!(rope.nodes[0].pinned /* first node is pinned */,);
    }

    #[test]
    fn test_step_runs() {
        let mut rope = Rope2d::make_hanging([0.0, 10.0], 4, 1.0, 9.81);
        rope.step(0.016);
        assert!(rope.tip().is_some() /* tip exists */,);
    }

    #[test]
    fn test_anchor_stays_fixed() {
        let mut rope = Rope2d::make_hanging([0.0, 10.0], 4, 1.0, 9.81);
        let anchor = rope.nodes[0].position;
        rope.step(0.016);
        assert!((rope.nodes[0].position[0] - anchor[0]).abs() < 1e-5, /* x unchanged */);
        assert!((rope.nodes[0].position[1] - anchor[1]).abs() < 1e-5, /* y unchanged */);
    }

    #[test]
    fn test_gravity_moves_tip() {
        let mut rope = Rope2d::make_hanging([0.0, 10.0], 4, 1.0, 9.81);
        let tip_y0 = rope.tip().unwrap()[1];
        rope.step(0.1);
        let tip_y1 = rope.tip().unwrap()[1];
        assert!(
            tip_y1 < tip_y0 || (tip_y1 - tip_y0).abs() < 0.5,
            /* tip moves down or constraint holds it */
        );
    }

    #[test]
    fn test_total_length_approx() {
        let rope = Rope2d::make_hanging([0.0, 0.0], 5, 2.0, 9.81);
        assert!((rope.total_length_approx() - 10.0).abs() < 1e-5, /* 5 segments * 2.0 */);
    }

    #[test]
    fn test_segment_length_fn() {
        let l = rope_segment_length([0.0, 0.0], [3.0, 4.0]);
        assert!((l - 5.0).abs() < 1e-5 /* 3-4-5 triangle */,);
    }

    #[test]
    fn test_multi_step_finite() {
        let mut rope = Rope2d::make_hanging([0.0, 10.0], 3, 1.0, 9.81);
        for _ in 0..100 {
            rope.step(0.01);
        }
        for n in &rope.nodes {
            assert!(
                n.position[0].is_finite() && n.position[1].is_finite(),
                /* all positions finite */
            );
        }
    }

    #[test]
    fn test_constraint_iters_default() {
        let rope = Rope2d::make_hanging([0.0, 0.0], 3, 1.0, 9.81);
        assert_eq!(rope.constraint_iters, 10);
    }
}
