// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Thermal heat diffusion simulation for material temperature modeling.

// ── Structs ──────────────────────────────────────────────────────────────────

/// Configuration parameters for thermal simulation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ThermalConfig {
    pub conductivity: f32,
    pub specific_heat: f32,
    pub density: f32,
    pub ambient_temp: f32,
    pub dt: f32,
}

/// A single node in the thermal network.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ThermalNode {
    pub temperature: f32,
    pub heat_source: f32,
    pub mass: f32,
    pub neighbors: Vec<u32>,
}

/// A thermal system: a network of nodes sharing a config.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ThermalSystem {
    pub nodes: Vec<ThermalNode>,
    pub config: ThermalConfig,
    pub time: f32,
}

/// Snapshot of temperatures after a simulation step.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ThermalResult {
    pub temperatures: Vec<f32>,
    pub max_temp: f32,
    pub min_temp: f32,
    pub avg_temp: f32,
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Build a default `ThermalConfig` for steel-like material at room temperature.
#[allow(dead_code)]
pub fn default_thermal_config() -> ThermalConfig {
    ThermalConfig {
        conductivity: 50.0,  // W/(m·K) – steel
        specific_heat: 500.0, // J/(kg·K)
        density: 7800.0,      // kg/m³
        ambient_temp: 20.0,   // °C
        dt: 0.01,             // seconds
    }
}

/// Create a new `ThermalNode` with given temperature and mass.
#[allow(dead_code)]
pub fn new_thermal_node(temp: f32, mass: f32) -> ThermalNode {
    ThermalNode {
        temperature: temp,
        heat_source: 0.0,
        mass: mass.max(1e-9),
        neighbors: Vec::new(),
    }
}

/// Create a new empty `ThermalSystem` with the given config.
#[allow(dead_code)]
pub fn new_thermal_system(cfg: ThermalConfig) -> ThermalSystem {
    ThermalSystem {
        nodes: Vec::new(),
        config: cfg,
        time: 0.0,
    }
}

/// Add a node to the system.
#[allow(dead_code)]
pub fn add_thermal_node(sys: &mut ThermalSystem, node: ThermalNode) {
    sys.nodes.push(node);
}

/// Compute thermal flux from `from` to `to` in Watts.
///
/// `flux = conductivity * (T_from - T_to)`.
/// Scaled by area / distance (assumed unit here for simplicity).
#[allow(dead_code)]
pub fn thermal_flux(from: &ThermalNode, to: &ThermalNode, cfg: &ThermalConfig) -> f32 {
    cfg.conductivity * (from.temperature - to.temperature)
}

/// Step the entire thermal system by `config.dt` and return a snapshot.
#[allow(dead_code)]
pub fn step_thermal(sys: &mut ThermalSystem) -> ThermalResult {
    let n = sys.nodes.len();
    if n == 0 {
        return ThermalResult {
            temperatures: Vec::new(),
            max_temp: 0.0,
            min_temp: 0.0,
            avg_temp: 0.0,
        };
    }

    // Compute delta temperatures
    let mut delta = vec![0.0_f32; n];
    let dt = sys.config.dt;

    for (i, d) in delta.iter_mut().enumerate().take(n) {
        // Conduction from neighbors
        let neighbors = sys.nodes[i].neighbors.clone();
        for &nb in &neighbors {
            let nb = nb as usize;
            if nb < n {
                let flux = sys.config.conductivity
                    * (sys.nodes[nb].temperature - sys.nodes[i].temperature);
                *d += flux;
            }
        }
        // Convective cooling to ambient
        let cooling = sys.config.conductivity
            * 0.01
            * (sys.config.ambient_temp - sys.nodes[i].temperature);
        *d += cooling;
        // Internal heat source (W)
        *d += sys.nodes[i].heat_source;
    }

    // Apply delta: dT = Q * dt / (m * Cp)
    for (i, &d) in delta.iter().enumerate().take(n) {
        let cap = sys.nodes[i].mass * sys.config.specific_heat;
        sys.nodes[i].temperature += d * dt / cap.max(1e-12);
    }
    sys.time += dt;

    let temps: Vec<f32> = sys.nodes.iter().map(|nd| nd.temperature).collect();
    let max_temp = temps.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let min_temp = temps.iter().cloned().fold(f32::INFINITY, f32::min);
    let avg_temp = temps.iter().sum::<f32>() / temps.len() as f32;

    ThermalResult {
        temperatures: temps,
        max_temp,
        min_temp,
        avg_temp,
    }
}

/// Get the temperature of a node by index.
#[allow(dead_code)]
pub fn node_temperature(sys: &ThermalSystem, idx: usize) -> f32 {
    sys.nodes[idx].temperature
}

/// Set the heat source power (Watts) on node `idx`.
#[allow(dead_code)]
pub fn set_node_heat_source(sys: &mut ThermalSystem, idx: usize, power: f32) {
    sys.nodes[idx].heat_source = power;
}

/// Serialize the system config and node count to JSON.
#[allow(dead_code)]
pub fn thermal_system_to_json(sys: &ThermalSystem) -> String {
    format!(
        "{{\"node_count\":{},\"time\":{:.4},\"conductivity\":{:.4},\"ambient_temp\":{:.2}}}",
        sys.nodes.len(),
        sys.time,
        sys.config.conductivity,
        sys.config.ambient_temp,
    )
}

/// Serialize a `ThermalResult` to JSON.
#[allow(dead_code)]
pub fn thermal_result_to_json(r: &ThermalResult) -> String {
    format!(
        "{{\"node_count\":{},\"max_temp\":{:.4},\"min_temp\":{:.4},\"avg_temp\":{:.4}}}",
        r.temperatures.len(),
        r.max_temp,
        r.min_temp,
        r.avg_temp,
    )
}

/// Estimate the equilibrium temperature as the weighted mean of node temperatures
/// considering heat sources and ambient cooling.
#[allow(dead_code)]
pub fn equilibrium_temperature(sys: &ThermalSystem) -> f32 {
    if sys.nodes.is_empty() {
        return sys.config.ambient_temp;
    }
    // Simple weighted average: nodes with heat sources are hotter at equilibrium
    let total_source: f32 = sys.nodes.iter().map(|n| n.heat_source).sum();
    if total_source.abs() < 1e-9 {
        return sys.config.ambient_temp;
    }
    // Equilibrium: T_eq = T_ambient + total_source / (k_conv * n)
    let n = sys.nodes.len() as f32;
    let k_conv = sys.config.conductivity * 0.01 * n;
    sys.config.ambient_temp + total_source / k_conv.max(1e-12)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_reasonable() {
        let cfg = default_thermal_config();
        assert!(cfg.conductivity > 0.0);
        assert!(cfg.specific_heat > 0.0);
        assert!(cfg.dt > 0.0);
    }

    #[test]
    fn new_node_has_given_temperature() {
        let node = new_thermal_node(100.0, 1.0);
        assert!((node.temperature - 100.0).abs() < 1e-6);
    }

    #[test]
    fn step_thermal_empty_system_ok() {
        let cfg = default_thermal_config();
        let mut sys = new_thermal_system(cfg);
        let result = step_thermal(&mut sys);
        assert_eq!(result.temperatures.len(), 0);
    }

    #[test]
    fn step_thermal_cools_hot_node_toward_ambient() {
        let cfg = default_thermal_config();
        let mut sys = new_thermal_system(cfg);
        let node = new_thermal_node(200.0, 1.0);
        add_thermal_node(&mut sys, node);
        let before = sys.nodes[0].temperature;
        step_thermal(&mut sys);
        let after = sys.nodes[0].temperature;
        // Hot node should cool towards ambient (20°C)
        assert!(after < before, "Hot node should cool; before={before}, after={after}");
    }

    #[test]
    fn thermal_flux_direction_correct() {
        let cfg = default_thermal_config();
        let hot = new_thermal_node(100.0, 1.0);
        let cold = new_thermal_node(20.0, 1.0);
        let flux = thermal_flux(&hot, &cold, &cfg);
        assert!(flux > 0.0, "Flux from hot to cold should be positive");
    }

    #[test]
    fn set_heat_source_raises_equilibrium() {
        let cfg = default_thermal_config();
        let mut sys = new_thermal_system(cfg);
        add_thermal_node(&mut sys, new_thermal_node(20.0, 1.0));
        set_node_heat_source(&mut sys, 0, 1000.0);
        let eq = equilibrium_temperature(&sys);
        assert!(eq > sys.config.ambient_temp, "Equilibrium should be above ambient with heat source");
    }

    #[test]
    fn thermal_result_to_json_fields() {
        let r = ThermalResult {
            temperatures: vec![100.0, 80.0, 60.0],
            max_temp: 100.0,
            min_temp: 60.0,
            avg_temp: 80.0,
        };
        let json = thermal_result_to_json(&r);
        assert!(json.contains("max_temp"));
        assert!(json.contains("min_temp"));
        assert!(json.contains("avg_temp"));
    }

    #[test]
    fn thermal_system_to_json_has_node_count() {
        let cfg = default_thermal_config();
        let mut sys = new_thermal_system(cfg);
        add_thermal_node(&mut sys, new_thermal_node(50.0, 1.0));
        add_thermal_node(&mut sys, new_thermal_node(30.0, 1.0));
        let json = thermal_system_to_json(&sys);
        assert!(json.contains("node_count"));
        assert!(json.contains('2'));
    }
}
