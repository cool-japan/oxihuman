// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct Population {
    pub size: f32,
    pub carrying_capacity: f32,
    pub growth_rate: f32,
}

pub fn new_population(size: f32, capacity: f32, rate: f32) -> Population {
    Population {
        size,
        carrying_capacity: capacity,
        growth_rate: rate,
    }
}

pub fn population_step(p: &mut Population, dt: f32) {
    /* logistic ODE: dN/dt = r * N * (1 - N/K) */
    let dn = p.growth_rate * p.size * (1.0 - p.size / p.carrying_capacity);
    p.size = (p.size + dn * dt).max(0.0);
}

pub fn population_doubling_time(p: &Population) -> f32 {
    if p.growth_rate <= 0.0 {
        return f32::INFINITY;
    }
    2_f32.ln() / p.growth_rate
}

pub fn population_carrying_fraction(p: &Population) -> f32 {
    if p.carrying_capacity <= 0.0 {
        return 0.0;
    }
    p.size / p.carrying_capacity
}

pub fn population_is_growing(p: &Population) -> bool {
    p.size < p.carrying_capacity
}

pub fn population_equilibrium(p: &Population) -> f32 {
    p.carrying_capacity
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_population() {
        /* create population with size, capacity, rate */
        let p = new_population(100.0, 1000.0, 0.1);
        assert_eq!(p.size, 100.0);
        assert_eq!(p.carrying_capacity, 1000.0);
    }

    #[test]
    fn test_population_step_grows() {
        /* below carrying capacity, population grows */
        let mut p = new_population(100.0, 1000.0, 0.1);
        let size_before = p.size;
        population_step(&mut p, 1.0);
        assert!(p.size > size_before);
    }

    #[test]
    fn test_population_at_capacity() {
        /* at carrying capacity, growth is zero */
        let mut p = new_population(1000.0, 1000.0, 0.1);
        let size_before = p.size;
        population_step(&mut p, 1.0);
        assert!((p.size - size_before).abs() < 1e-4);
    }

    #[test]
    fn test_population_doubling_time() {
        /* doubling time = ln(2) / r */
        let p = new_population(100.0, 1000.0, 0.1);
        let dt = population_doubling_time(&p);
        assert!((dt - 2_f32.ln() / 0.1).abs() < 1e-5);
    }

    #[test]
    fn test_population_carrying_fraction() {
        /* fraction of capacity */
        let p = new_population(500.0, 1000.0, 0.1);
        assert!((population_carrying_fraction(&p) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_population_is_growing() {
        /* growing when below capacity */
        let p = new_population(500.0, 1000.0, 0.1);
        assert!(population_is_growing(&p));
    }

    #[test]
    fn test_population_equilibrium() {
        /* equilibrium is carrying capacity */
        let p = new_population(100.0, 1000.0, 0.1);
        assert_eq!(population_equilibrium(&p), 1000.0);
    }
}
