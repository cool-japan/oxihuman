// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// An impulse applied to a pair of bodies at a contact point.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct ImpulsePair {
    pub body_a: u32,
    pub body_b: u32,
    pub impulse: [f32; 3],
    pub contact_point: [f32; 3],
    pub normal: [f32; 3],
    pub applied: bool,
}

#[allow(dead_code)]
fn v3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
fn v3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[allow(dead_code)]
fn v3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[allow(dead_code)]
fn v3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[allow(dead_code)]
fn v3_len(v: [f32; 3]) -> f32 {
    v3_dot(v, v).sqrt()
}

#[allow(dead_code)]
impl ImpulsePair {
    pub fn new(body_a: u32, body_b: u32, impulse: [f32; 3]) -> Self {
        Self {
            body_a,
            body_b,
            impulse,
            contact_point: [0.0; 3],
            normal: [0.0, 1.0, 0.0],
            applied: false,
        }
    }

    pub fn with_contact(mut self, point: [f32; 3], normal: [f32; 3]) -> Self {
        self.contact_point = point;
        self.normal = normal;
        self
    }

    pub fn from_collision(
        body_a: u32,
        body_b: u32,
        relative_velocity: [f32; 3],
        normal: [f32; 3],
        restitution: f32,
        inv_mass_sum: f32,
    ) -> Self {
        let vn = v3_dot(relative_velocity, normal);
        let j = if inv_mass_sum > f32::EPSILON {
            -(1.0 + restitution) * vn / inv_mass_sum
        } else {
            0.0
        };
        Self {
            body_a,
            body_b,
            impulse: v3_scale(normal, j),
            contact_point: [0.0; 3],
            normal,
            applied: false,
        }
    }

    pub fn magnitude(&self) -> f32 {
        v3_len(self.impulse)
    }

    pub fn normal_component(&self) -> f32 {
        v3_dot(self.impulse, self.normal)
    }

    pub fn tangential_impulse(&self) -> [f32; 3] {
        let n_comp = v3_scale(self.normal, self.normal_component());
        v3_sub(self.impulse, n_comp)
    }

    pub fn tangential_magnitude(&self) -> f32 {
        v3_len(self.tangential_impulse())
    }

    pub fn impulse_for_a(&self) -> [f32; 3] {
        self.impulse
    }

    pub fn impulse_for_b(&self) -> [f32; 3] {
        v3_scale(self.impulse, -1.0)
    }

    pub fn mark_applied(&mut self) {
        self.applied = true;
    }

    pub fn is_applied(&self) -> bool {
        self.applied
    }

    pub fn involves(&self, body: u32) -> bool {
        self.body_a == body || self.body_b == body
    }

    pub fn is_separating(&self) -> bool {
        self.normal_component() > 0.0
    }
}

/// A collection of impulse pairs.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ImpulsePairBuffer {
    pairs: Vec<ImpulsePair>,
}

impl Default for ImpulsePairBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl ImpulsePairBuffer {
    pub fn new() -> Self {
        Self { pairs: Vec::new() }
    }

    pub fn push(&mut self, pair: ImpulsePair) {
        self.pairs.push(pair);
    }

    pub fn count(&self) -> usize {
        self.pairs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    pub fn clear(&mut self) {
        self.pairs.clear();
    }

    pub fn total_magnitude(&self) -> f32 {
        self.pairs.iter().map(|p| p.magnitude()).sum()
    }

    pub fn pairs(&self) -> &[ImpulsePair] {
        &self.pairs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let ip = ImpulsePair::new(0, 1, [10.0, 0.0, 0.0]);
        assert_eq!(ip.body_a, 0);
        assert!(!ip.is_applied());
    }

    #[test]
    fn test_magnitude() {
        let ip = ImpulsePair::new(0, 1, [3.0, 4.0, 0.0]);
        assert!((ip.magnitude() - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_impulse_for_b_negated() {
        let ip = ImpulsePair::new(0, 1, [5.0, 0.0, 0.0]);
        let b = ip.impulse_for_b();
        assert!((b[0] + 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_normal_component() {
        let ip = ImpulsePair::new(0, 1, [0.0, 10.0, 0.0]).with_contact([0.0; 3], [0.0, 1.0, 0.0]);
        assert!((ip.normal_component() - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_tangential_impulse() {
        let ip = ImpulsePair::new(0, 1, [5.0, 10.0, 0.0]).with_contact([0.0; 3], [0.0, 1.0, 0.0]);
        let t = ip.tangential_impulse();
        assert!((t[0] - 5.0).abs() < f32::EPSILON);
        assert!((t[1] - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_from_collision() {
        let ip = ImpulsePair::from_collision(0, 1, [0.0, -5.0, 0.0], [0.0, 1.0, 0.0], 0.5, 1.0);
        assert!(ip.impulse[1] > 0.0); // separating impulse
    }

    #[test]
    fn test_mark_applied() {
        let mut ip = ImpulsePair::new(0, 1, [1.0; 3]);
        ip.mark_applied();
        assert!(ip.is_applied());
    }

    #[test]
    fn test_involves() {
        let ip = ImpulsePair::new(2, 5, [0.0; 3]);
        assert!(ip.involves(2));
        assert!(ip.involves(5));
        assert!(!ip.involves(3));
    }

    #[test]
    fn test_buffer() {
        let mut buf = ImpulsePairBuffer::new();
        buf.push(ImpulsePair::new(0, 1, [1.0, 0.0, 0.0]));
        buf.push(ImpulsePair::new(1, 2, [0.0, 2.0, 0.0]));
        assert_eq!(buf.count(), 2);
    }

    #[test]
    fn test_buffer_total_magnitude() {
        let mut buf = ImpulsePairBuffer::new();
        buf.push(ImpulsePair::new(0, 1, [3.0, 4.0, 0.0]));
        buf.push(ImpulsePair::new(1, 2, [0.0, 0.0, 5.0]));
        assert!((buf.total_magnitude() - 10.0).abs() < 1e-5);
    }
}
