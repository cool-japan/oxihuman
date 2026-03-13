// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/* Gas constant in J/(mol·K) */
const R_GAS: f32 = 8.314;

pub struct Reaction {
    pub rate_constant: f32,
    pub activation_energy_j: f32,
    pub pre_exponential: f32,
    pub temperature_k: f32,
}

pub fn new_reaction(ea: f32, a: f32, temp_k: f32) -> Reaction {
    let k = a * (-ea / (R_GAS * temp_k)).exp();
    Reaction {
        rate_constant: k,
        activation_energy_j: ea,
        pre_exponential: a,
        temperature_k: temp_k,
    }
}

pub fn reaction_rate_constant(r: &Reaction) -> f32 {
    r.pre_exponential * (-r.activation_energy_j / (R_GAS * r.temperature_k)).exp()
}

pub fn reaction_rate(r: &Reaction, concentration: f32, order: u32) -> f32 {
    let k = reaction_rate_constant(r);
    k * concentration.powi(order as i32)
}

pub fn reaction_half_life(r: &Reaction, order: u32) -> f32 {
    let k = reaction_rate_constant(r);
    if k < 1e-20 {
        return f32::INFINITY;
    }
    match order {
        1 => 2_f32.ln() / k,
        /* second order: t_half = 1 / (k * [A]_0), but [A]_0 unknown, return 1/k */
        2 => 1.0 / k,
        _ => 2_f32.ln() / k,
    }
}

pub fn reaction_set_temperature(r: &mut Reaction, t: f32) {
    r.temperature_k = t;
    r.rate_constant = reaction_rate_constant(r);
}

pub fn reaction_activation_energy(r: &Reaction) -> f32 {
    r.activation_energy_j
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_reaction() {
        /* create reaction with Arrhenius params */
        let r = new_reaction(50_000.0, 1e10, 300.0);
        assert!(r.rate_constant > 0.0);
    }

    #[test]
    fn test_reaction_rate_constant_positive() {
        /* k is always positive */
        let r = new_reaction(40_000.0, 1e8, 350.0);
        assert!(reaction_rate_constant(&r) > 0.0);
    }

    #[test]
    fn test_reaction_rate_order_1() {
        /* first order rate = k * [A] */
        let r = new_reaction(0.0, 2.0, 300.0);
        let rate = reaction_rate(&r, 1.0, 1);
        assert!((rate - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_reaction_half_life_first_order() {
        /* t_half = ln(2) / k */
        let r = new_reaction(0.0, 1.0, 300.0);
        let t = reaction_half_life(&r, 1);
        assert!((t - 2_f32.ln()).abs() < 1e-5);
    }

    #[test]
    fn test_reaction_set_temperature() {
        /* higher temperature increases k */
        let mut r = new_reaction(50_000.0, 1e10, 300.0);
        let k_low = reaction_rate_constant(&r);
        reaction_set_temperature(&mut r, 400.0);
        let k_high = reaction_rate_constant(&r);
        assert!(k_high > k_low);
    }

    #[test]
    fn test_reaction_activation_energy() {
        /* Ea stored correctly */
        let r = new_reaction(75_000.0, 1.0, 300.0);
        assert!((reaction_activation_energy(&r) - 75_000.0).abs() < 1.0);
    }

    #[test]
    fn test_reaction_rate_order_0() {
        /* zeroth order rate = k * [A]^0 = k */
        let r = new_reaction(0.0, 3.0, 300.0);
        let rate = reaction_rate(&r, 5.0, 0);
        assert!((rate - 3.0).abs() < 1e-4);
    }
}
