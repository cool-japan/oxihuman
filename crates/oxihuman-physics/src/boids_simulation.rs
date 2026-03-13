// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Craig Reynolds boids (flocking) simulation.

/// A single boid agent.
#[derive(Debug, Clone)]
pub struct Boid {
    pub pos: [f32; 2],
    pub vel: [f32; 2],
}

impl Boid {
    pub fn new(x: f32, y: f32, vx: f32, vy: f32) -> Self {
        Self {
            pos: [x, y],
            vel: [vx, vy],
        }
    }

    pub fn speed(&self) -> f32 {
        (self.vel[0] * self.vel[0] + self.vel[1] * self.vel[1]).sqrt()
    }
}

/// Boids simulation parameters.
#[derive(Debug, Clone)]
pub struct BoidsParams {
    pub max_speed: f32,
    pub min_speed: f32,
    pub perception_radius: f32,
    pub separation_radius: f32,
    pub align_weight: f32,
    pub cohesion_weight: f32,
    pub separation_weight: f32,
}

impl Default for BoidsParams {
    fn default() -> Self {
        Self {
            max_speed: 2.0,
            min_speed: 0.5,
            perception_radius: 50.0,
            separation_radius: 15.0,
            align_weight: 1.0,
            cohesion_weight: 0.8,
            separation_weight: 1.5,
        }
    }
}

fn dist2(a: &Boid, b: &Boid) -> f32 {
    let dx = a.pos[0] - b.pos[0];
    let dy = a.pos[1] - b.pos[1];
    dx * dx + dy * dy
}

fn normalize(v: [f32; 2]) -> [f32; 2] {
    let mag = (v[0] * v[0] + v[1] * v[1]).sqrt();
    if mag < 1e-6 {
        [0.0, 0.0]
    } else {
        [v[0] / mag, v[1] / mag]
    }
}

fn clamp_speed(vel: [f32; 2], min_s: f32, max_s: f32) -> [f32; 2] {
    let s = (vel[0] * vel[0] + vel[1] * vel[1]).sqrt();
    if s < 1e-6 {
        return vel;
    }
    let clamped = s.clamp(min_s, max_s);
    [vel[0] / s * clamped, vel[1] / s * clamped]
}

/// Boids flock simulation.
pub struct BoidsSimulation {
    pub boids: Vec<Boid>,
    pub params: BoidsParams,
}

impl BoidsSimulation {
    pub fn new(boids: Vec<Boid>, params: BoidsParams) -> Self {
        Self { boids, params }
    }

    #[allow(clippy::needless_range_loop)]
    pub fn step(&mut self, dt: f32) {
        let n = self.boids.len();
        let mut new_vels = vec![[0.0f32; 2]; n];

        for i in 0..n {
            let mut align = [0.0f32; 2];
            let mut cohesion = [0.0f32; 2];
            let mut separation = [0.0f32; 2];
            let mut neighbors = 0usize;
            let perc_r2 = self.params.perception_radius * self.params.perception_radius;
            let sep_r2 = self.params.separation_radius * self.params.separation_radius;

            for j in 0..n {
                if i == j {
                    continue;
                }
                let d2 = dist2(&self.boids[i], &self.boids[j]);
                if d2 < perc_r2 {
                    align[0] += self.boids[j].vel[0];
                    align[1] += self.boids[j].vel[1];
                    cohesion[0] += self.boids[j].pos[0];
                    cohesion[1] += self.boids[j].pos[1];
                    neighbors += 1;

                    if d2 < sep_r2 {
                        separation[0] -= self.boids[j].pos[0] - self.boids[i].pos[0];
                        separation[1] -= self.boids[j].pos[1] - self.boids[i].pos[1];
                    }
                }
            }

            let mut steer = self.boids[i].vel;
            if neighbors > 0 {
                let n_f = neighbors as f32;
                /* alignment */
                let align_n = normalize([align[0] / n_f, align[1] / n_f]);
                /* cohesion */
                let cx = cohesion[0] / n_f - self.boids[i].pos[0];
                let cy = cohesion[1] / n_f - self.boids[i].pos[1];
                let coh_n = normalize([cx, cy]);
                /* separation */
                let sep_n = normalize(separation);

                steer[0] += self.params.align_weight * align_n[0]
                    + self.params.cohesion_weight * coh_n[0]
                    + self.params.separation_weight * sep_n[0];
                steer[1] += self.params.align_weight * align_n[1]
                    + self.params.cohesion_weight * coh_n[1]
                    + self.params.separation_weight * sep_n[1];
            }
            new_vels[i] = clamp_speed(steer, self.params.min_speed, self.params.max_speed);
        }

        for (i, boid) in self.boids.iter_mut().enumerate() {
            boid.vel = new_vels[i];
            boid.pos[0] += boid.vel[0] * dt;
            boid.pos[1] += boid.vel[1] * dt;
        }
    }

    pub fn boid_count(&self) -> usize {
        self.boids.len()
    }

    pub fn average_speed(&self) -> f32 {
        if self.boids.is_empty() {
            return 0.0;
        }
        self.boids.iter().map(|b| b.speed()).sum::<f32>() / self.boids.len() as f32
    }

    pub fn center_of_mass(&self) -> [f32; 2] {
        if self.boids.is_empty() {
            return [0.0, 0.0];
        }
        let n = self.boids.len() as f32;
        let cx = self.boids.iter().map(|b| b.pos[0]).sum::<f32>() / n;
        let cy = self.boids.iter().map(|b| b.pos[1]).sum::<f32>() / n;
        [cx, cy]
    }
}

pub fn new_boids_simulation(boids: Vec<Boid>) -> BoidsSimulation {
    BoidsSimulation::new(boids, BoidsParams::default())
}

pub fn boids_step(sim: &mut BoidsSimulation, dt: f32) {
    sim.step(dt);
}

pub fn boids_count(sim: &BoidsSimulation) -> usize {
    sim.boid_count()
}

pub fn boids_avg_speed(sim: &BoidsSimulation) -> f32 {
    sim.average_speed()
}

pub fn boids_center(sim: &BoidsSimulation) -> [f32; 2] {
    sim.center_of_mass()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_boids(n: usize) -> Vec<Boid> {
        (0..n)
            .map(|i| Boid::new(i as f32 * 5.0, 0.0, 1.0, 0.0))
            .collect()
    }

    #[test]
    fn test_new_sim() {
        let sim = new_boids_simulation(make_boids(5));
        assert_eq!(boids_count(&sim), 5);
    }

    #[test]
    fn test_step_changes_positions() {
        let mut sim = new_boids_simulation(make_boids(3));
        let p0 = sim.boids[0].pos;
        boids_step(&mut sim, 1.0);
        assert!(sim.boids[0].pos[0] != p0[0] || sim.boids[0].pos[1] != p0[1]);
    }

    #[test]
    fn test_speed_clamped_after_step() {
        let mut sim = new_boids_simulation(make_boids(5));
        boids_step(&mut sim, 0.1);
        for b in &sim.boids {
            assert!(b.speed() <= sim.params.max_speed + 1e-5);
        }
    }

    #[test]
    fn test_center_of_mass() {
        let boids = vec![Boid::new(0.0, 0.0, 1.0, 0.0), Boid::new(2.0, 0.0, 1.0, 0.0)];
        let sim = new_boids_simulation(boids);
        let c = boids_center(&sim);
        assert!((c[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_average_speed() {
        let boids = vec![Boid::new(0.0, 0.0, 1.0, 0.0)];
        let sim = new_boids_simulation(boids);
        assert!((boids_avg_speed(&sim) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_empty_sim() {
        let sim = new_boids_simulation(vec![]);
        assert_eq!(boids_count(&sim), 0);
        assert_eq!(boids_avg_speed(&sim), 0.0);
    }

    #[test]
    fn test_many_steps_finite() {
        let mut sim = new_boids_simulation(make_boids(10));
        for _ in 0..100 {
            boids_step(&mut sim, 0.1);
        }
        for b in &sim.boids {
            assert!(b.pos[0].is_finite());
            assert!(b.pos[1].is_finite());
        }
    }

    #[test]
    fn test_single_boid_no_neighbors() {
        let mut sim = new_boids_simulation(vec![Boid::new(0.0, 0.0, 1.0, 0.0)]);
        boids_step(&mut sim, 1.0);
        /* single boid keeps moving in original direction */
        assert!(sim.boids[0].pos[0].is_finite());
    }

    #[test]
    fn test_speed_after_many_steps_reasonable() {
        let mut sim = new_boids_simulation(make_boids(8));
        for _ in 0..50 {
            boids_step(&mut sim, 0.1);
        }
        let avg = boids_avg_speed(&sim);
        assert!(avg >= 0.0 && avg <= sim.params.max_speed + 1e-4);
    }
}
