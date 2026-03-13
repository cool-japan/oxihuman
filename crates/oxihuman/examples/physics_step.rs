// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Physics step example: build a 3×3 cloth grid, run 10 simulation steps,
//! and print the final Y position of the free corner particle.
//!
//! The top row of particles is pinned so the cloth hangs under gravity.
//! All parameters are deterministic literals — no random input.

use anyhow::{anyhow, Result};
use oxihuman::physics::{ClothParticle, ClothSim, Spring, SpringKind};

/// Build a flat N×N cloth grid in the XZ plane at height `y`.
///
/// Grid layout (N=3, indices left-to-right, top-to-bottom):
/// ```text
///  0 — 1 — 2   (top row, y = 1.0, will be pinned)
///  |   |   |
///  3 — 4 — 5
///  |   |   |
///  6 — 7 — 8   (bottom row, y = 0.0, free)
/// ```
fn build_grid_cloth(n: usize, spacing: f32, height: f32) -> ClothSim {
    let mut particles: Vec<ClothParticle> = Vec::with_capacity(n * n);

    for row in 0..n {
        for col in 0..n {
            let x = col as f32 * spacing;
            let y = height - row as f32 * spacing;
            let z = 0.0_f32;
            let mut p = ClothParticle::new([x, y, z], 1.0);
            // Pin the entire top row
            if row == 0 {
                p = p.pinned();
            }
            particles.push(p);
        }
    }

    let mut springs: Vec<Spring> = Vec::new();

    // Structural springs: horizontal and vertical edges
    for row in 0..n {
        for col in 0..n {
            let idx = row * n + col;
            // Right neighbour
            if col + 1 < n {
                let right = row * n + col + 1;
                let rest = spacing;
                springs.push(Spring {
                    a: idx,
                    b: right,
                    rest_length: rest,
                    stiffness: 0.9,
                    kind: SpringKind::Structural,
                });
            }
            // Down neighbour
            if row + 1 < n {
                let down = (row + 1) * n + col;
                let rest = spacing;
                springs.push(Spring {
                    a: idx,
                    b: down,
                    rest_length: rest,
                    stiffness: 0.9,
                    kind: SpringKind::Structural,
                });
            }
            // Diagonal bending spring (skip-one down-right)
            if col + 1 < n && row + 1 < n {
                let diag = (row + 1) * n + col + 1;
                let rest = spacing * std::f32::consts::SQRT_2;
                springs.push(Spring {
                    a: idx,
                    b: diag,
                    rest_length: rest,
                    stiffness: 0.5,
                    kind: SpringKind::Bending,
                });
            }
        }
    }

    ClothSim {
        particles,
        springs,
        gravity: [0.0, -9.81, 0.0],
        damping: 0.99,
    }
}

fn main() -> Result<()> {
    const GRID: usize = 3;
    const SPACING: f32 = 0.5;
    const INITIAL_HEIGHT: f32 = 1.0;
    const DT: f32 = 0.016; // ~60 Hz
    const STEPS: usize = 10;

    let mut sim = build_grid_cloth(GRID, SPACING, INITIAL_HEIGHT);

    println!(
        "Cloth grid: {}×{} = {} particles, {} springs",
        GRID,
        GRID,
        sim.particles.len(),
        sim.springs.len()
    );

    // Free bottom-right corner index
    let corner_idx = GRID * GRID - 1;
    let initial_y = sim.particles[corner_idx].position[1];
    println!(
        "Corner particle [{}] initial Y = {:.4}",
        corner_idx, initial_y
    );

    // Run simulation steps
    for step in 0..STEPS {
        sim.step(DT, 4);
        let y = sim
            .particles
            .get(corner_idx)
            .ok_or_else(|| anyhow!("corner particle index {} out of range", corner_idx))?
            .position[1];
        println!("  step {:2} → corner Y = {:.4}", step + 1, y);
    }

    let final_y = sim
        .particles
        .get(corner_idx)
        .ok_or_else(|| anyhow!("corner particle index {} out of range", corner_idx))?
        .position[1];

    println!(
        "\nFinal corner Y position after {} steps: {:.6}",
        STEPS, final_y
    );
    assert!(
        final_y < initial_y,
        "corner should fall under gravity (initial={initial_y:.4}, final={final_y:.4})"
    );
    println!(
        "Gravity check passed: particle fell {:.4} units.",
        initial_y - final_y
    );

    Ok(())
}
