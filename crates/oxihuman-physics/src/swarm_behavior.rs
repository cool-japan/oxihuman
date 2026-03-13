// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct Boid {
    pub pos: [f32; 2],
    pub vel: [f32; 2],
}

pub fn new_boid(pos: [f32; 2], vel: [f32; 2]) -> Boid {
    Boid { pos, vel }
}

fn dist2(a: &Boid, b: &Boid) -> f32 {
    let dx = a.pos[0] - b.pos[0];
    let dy = a.pos[1] - b.pos[1];
    dx * dx + dy * dy
}

pub fn boid_separation(b: &Boid, neighbors: &[Boid], radius: f32) -> [f32; 2] {
    let mut fx = 0.0f32;
    let mut fy = 0.0f32;
    let r2 = radius * radius;
    for n in neighbors {
        let d2 = dist2(b, n);
        if d2 < r2 && d2 > 1e-10 {
            let d = d2.sqrt();
            fx += (b.pos[0] - n.pos[0]) / d;
            fy += (b.pos[1] - n.pos[1]) / d;
        }
    }
    [fx, fy]
}

pub fn boid_alignment(b: &Boid, neighbors: &[Boid], radius: f32) -> [f32; 2] {
    let mut avx = 0.0f32;
    let mut avy = 0.0f32;
    let mut count = 0usize;
    let r2 = radius * radius;
    for n in neighbors {
        if dist2(b, n) < r2 {
            avx += n.vel[0];
            avy += n.vel[1];
            count += 1;
        }
    }
    if count == 0 {
        return [0.0, 0.0];
    }
    let avx = avx / count as f32;
    let avy = avy / count as f32;
    [avx - b.vel[0], avy - b.vel[1]]
}

pub fn boid_cohesion(b: &Boid, neighbors: &[Boid], radius: f32) -> [f32; 2] {
    let mut cx = 0.0f32;
    let mut cy = 0.0f32;
    let mut count = 0usize;
    let r2 = radius * radius;
    for n in neighbors {
        if dist2(b, n) < r2 {
            cx += n.pos[0];
            cy += n.pos[1];
            count += 1;
        }
    }
    if count == 0 {
        return [0.0, 0.0];
    }
    cx /= count as f32;
    cy /= count as f32;
    [cx - b.pos[0], cy - b.pos[1]]
}

pub fn boid_step(b: &mut Boid, steer: [f32; 2], max_speed: f32, dt: f32) {
    b.vel[0] += steer[0] * dt;
    b.vel[1] += steer[1] * dt;
    let speed = boid_speed(b);
    if speed > max_speed {
        let scale = max_speed / speed;
        b.vel[0] *= scale;
        b.vel[1] *= scale;
    }
    b.pos[0] += b.vel[0] * dt;
    b.pos[1] += b.vel[1] * dt;
}

pub fn boid_speed(b: &Boid) -> f32 {
    (b.vel[0] * b.vel[0] + b.vel[1] * b.vel[1]).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_boid() {
        /* create boid with pos and vel */
        let b = new_boid([1.0, 2.0], [0.5, 0.0]);
        assert_eq!(b.pos, [1.0, 2.0]);
    }

    #[test]
    fn test_boid_speed() {
        /* compute speed from velocity */
        let b = new_boid([0.0, 0.0], [3.0, 4.0]);
        assert!((boid_speed(&b) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_boid_separation_repels() {
        /* nearby boid generates repulsion */
        let b = new_boid([0.0, 0.0], [0.0, 0.0]);
        let neighbors = vec![new_boid([0.3, 0.0], [0.0, 0.0])];
        let f = boid_separation(&b, &neighbors, 1.0);
        /* force should push b away from neighbor (negative x) */
        assert!(f[0] < 0.0);
    }

    #[test]
    fn test_boid_alignment_matches_neighbor() {
        /* alignment steers toward neighbor velocity */
        let b = new_boid([0.0, 0.0], [0.0, 0.0]);
        let neighbors = vec![new_boid([0.5, 0.0], [2.0, 0.0])];
        let f = boid_alignment(&b, &neighbors, 2.0);
        assert!(f[0] > 0.0);
    }

    #[test]
    fn test_boid_cohesion_toward_center() {
        /* cohesion steers toward neighbor center */
        let b = new_boid([0.0, 0.0], [0.0, 0.0]);
        let neighbors = vec![new_boid([2.0, 0.0], [0.0, 0.0])];
        let f = boid_cohesion(&b, &neighbors, 5.0);
        assert!(f[0] > 0.0);
    }

    #[test]
    fn test_boid_step_moves() {
        /* step updates position */
        let mut b = new_boid([0.0, 0.0], [1.0, 0.0]);
        boid_step(&mut b, [0.0, 0.0], 5.0, 0.1);
        assert!(b.pos[0] > 0.0);
    }

    #[test]
    fn test_boid_speed_clamped() {
        /* boid speed clamped to max_speed */
        let mut b = new_boid([0.0, 0.0], [0.0, 0.0]);
        boid_step(&mut b, [100.0, 0.0], 2.0, 1.0);
        assert!(boid_speed(&b) <= 2.0 + 1e-5);
    }
}
