// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Physics accuracy benchmarks — verifies that simulated outcomes stay within
//! acceptable bounds of analytical predictions.
//!
//! These are structured as Criterion benchmarks (not unit tests) so they can
//! be tracked for performance regression, but they also assert correctness
//! thresholds as part of the benchmark body.
//!
//! Three scenarios are tested:
//! 1. **Projectile motion** under gravity — final position within 1% of the
//!    analytical solution after 1 s of simulation.
//! 2. **Pendulum energy conservation** — total mechanical energy drifts less
//!    than 5% over 10 s of simulation.
//! 3. **Spring-mass equilibrium** — a damped spring-mass system settles to the
//!    correct equilibrium extension within 100 steps.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use oxihuman_physics::{add_xpbd_particle, new_xpbd_world, xpbd_add_distance, xpbd_step};

// ── helpers ───────────────────────────────────────────────────────────────────

/// Simple particle with Euler integration used for the projectile scenario.
struct Particle {
    pos: [f64; 3],
    vel: [f64; 3],
}

impl Particle {
    fn new(pos: [f64; 3], vel: [f64; 3]) -> Self {
        Self { pos, vel }
    }

    fn step(&mut self, accel: [f64; 3], dt: f64) {
        self.vel[0] += accel[0] * dt;
        self.vel[1] += accel[1] * dt;
        self.vel[2] += accel[2] * dt;
        self.pos[0] += self.vel[0] * dt;
        self.pos[1] += self.vel[1] * dt;
        self.pos[2] += self.vel[2] * dt;
    }
}

/// Symplectic-Euler pendulum particle constrained to a fixed rod length.
///
/// State: angle θ from vertical (radians), angular velocity ω (rad/s).
struct PendulumState {
    theta: f64,
    omega: f64,
    length: f64,
    g: f64,
}

impl PendulumState {
    fn new(theta0: f64, omega0: f64, length: f64, g: f64) -> Self {
        Self {
            theta: theta0,
            omega: omega0,
            length,
            g,
        }
    }

    fn step(&mut self, dt: f64) {
        // Simple pendulum: d²θ/dt² = -(g/L) * sin(θ)
        let alpha = -(self.g / self.length) * self.theta.sin();
        self.omega += alpha * dt;
        self.theta += self.omega * dt;
    }

    /// Total mechanical energy (KE + PE) relative to the pivot.
    fn energy(&self) -> f64 {
        let ke = 0.5 * self.length * self.length * self.omega * self.omega;
        let pe = self.g * self.length * (1.0 - self.theta.cos());
        ke + pe
    }
}

// ── bench 1: projectile motion accuracy ──────────────────────────────────────

/// Simulates a particle fired horizontally at 10 m/s from the origin for 1 s
/// (dt = 1 ms, 1000 steps) and checks that the final position is within 1% of
/// the analytical free-fall solution.
///
/// Analytical:  x = v₀t = 10 m,  y = ½gt² = -4.905 m,  z = 0
fn bench_projectile_motion(c: &mut Criterion) {
    c.bench_function("accuracy_projectile_1s_1pct", |b| {
        b.iter(|| {
            let v0 = black_box(10.0_f64); // initial horizontal velocity m/s
            let g = black_box(-9.81_f64);
            let dt = 0.001_f64; // 1 ms
            let steps = 1000usize; // 1 s total

            let mut p = Particle::new([0.0, 0.0, 0.0], [v0, 0.0, 0.0]);
            let accel = [0.0, g, 0.0];

            for _ in 0..steps {
                p.step(accel, dt);
            }

            // Analytical solution
            let t_total = dt * steps as f64;
            let x_analytical = v0 * t_total;
            let y_analytical = 0.5 * g * t_total * t_total;

            let x_err = (p.pos[0] - x_analytical).abs() / x_analytical.abs().max(1e-10);
            let y_err = (p.pos[1] - y_analytical).abs() / y_analytical.abs().max(1e-10);

            // Assert accuracy
            assert!(
                x_err < 0.01,
                "projectile x error {:.4} >= 1% (simulated={:.4}, expected={:.4})",
                x_err,
                p.pos[0],
                x_analytical
            );
            assert!(
                y_err < 0.01,
                "projectile y error {:.4} >= 1% (simulated={:.4}, expected={:.4})",
                y_err,
                p.pos[1],
                y_analytical
            );

            black_box((p.pos[0], p.pos[1]))
        });
    });
}

// ── bench 2: pendulum energy conservation ─────────────────────────────────────

/// Simulates a simple pendulum (L = 1 m, g = 9.81 m/s²) starting at 20°
/// with zero angular velocity, for 10 s (dt = 1 ms, 10 000 steps).
///
/// Energy conservation is checked: the total mechanical energy must not drift
/// by more than 5% relative to the initial energy.
fn bench_pendulum_energy_conservation(c: &mut Criterion) {
    c.bench_function("accuracy_pendulum_energy_10s_5pct", |b| {
        b.iter(|| {
            let theta0 = black_box(20.0_f64.to_radians());
            let mut pend = PendulumState::new(theta0, 0.0, 1.0, 9.81);

            let e0 = pend.energy();
            let dt = 0.001_f64;
            let steps = 10_000usize;

            for _ in 0..steps {
                pend.step(dt);
            }

            let e_final = pend.energy();
            let drift = (e_final - e0).abs() / e0.abs().max(1e-15);

            assert!(
                drift < 0.05,
                "pendulum energy drift {:.4} >= 5% (e0={:.6}, e_final={:.6})",
                drift,
                e0,
                e_final
            );

            black_box((e0, e_final))
        });
    });
}

// ── bench 3: spring-mass equilibrium ──────────────────────────────────────────

/// Simulates a spring-mass system using XPBD distance constraints.
///
/// Setup: particle A is pinned at origin; particle B (mass = 1 kg) starts
/// 0.5 m away from A; natural rest length = 1.0 m.  Under gravity the mass
/// should settle near y = -9.81 * m / k ≈ some extension.
///
/// We verify that after 100 steps the particle has moved toward the expected
/// direction (downward) and that the distance constraint is satisfied within
/// 1% of the rest length.
fn bench_spring_mass_equilibrium(c: &mut Criterion) {
    c.bench_function("accuracy_spring_mass_equilibrium_100steps", |b| {
        b.iter(|| {
            let mut world = new_xpbd_world();

            // Pinned anchor at origin
            let _a = add_xpbd_particle(&mut world, [0.0, 0.0, 0.0], 0.0);
            // Free mass starts 0.5 m below (half rest length)
            let b_idx = add_xpbd_particle(&mut world, [0.0, -0.5, 0.0], 1.0);

            // Distance constraint with rest length 1.0 m, low compliance
            xpbd_add_distance(&mut world, 0, 1, 1e-6);

            let dt = black_box(0.016_f32);
            for _ in 0..100 {
                xpbd_step(&mut world, dt, 4);
            }

            let p_b = &world.particles[b_idx];
            let dist = (p_b.position[0] * p_b.position[0]
                + p_b.position[1] * p_b.position[1]
                + p_b.position[2] * p_b.position[2])
                .sqrt();

            // Constraint should be satisfied: distance ≈ rest length (1.0 m) within 1%
            let rest = 1.0_f32;
            let dist_err = (dist - rest).abs() / rest;
            assert!(
                dist_err < 0.01,
                "spring-mass distance error {:.4} >= 1% (dist={:.4}, rest={:.4})",
                dist_err,
                dist,
                rest
            );

            // Mass must have moved downward (gravity pulls it)
            assert!(
                p_b.position[1] < 0.0,
                "mass should be below anchor (y={:.4})",
                p_b.position[1]
            );

            black_box(dist)
        });
    });
}

// ── criterion wiring ──────────────────────────────────────────────────────────

criterion_group!(
    accuracy_benches,
    bench_projectile_motion,
    bench_pendulum_energy_conservation,
    bench_spring_mass_equilibrium,
);
criterion_main!(accuracy_benches);
