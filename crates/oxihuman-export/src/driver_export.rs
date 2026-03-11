// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Driver/expression export (animation drivers).

/* ── legacy API (kept for backward compat) ── */

#[derive(Debug, Clone, PartialEq)]
pub enum DriverType {
    Linear,
    Polynomial,
    Scripted,
}

#[derive(Debug, Clone)]
pub struct DriverExport {
    pub name: String,
    pub target_prop: String,
    pub driver_type: DriverType,
    pub coefficients: Vec<f32>,
}

pub fn new_driver_export(name: &str, target_prop: &str, driver_type: DriverType) -> DriverExport {
    DriverExport {
        name: name.to_string(),
        target_prop: target_prop.to_string(),
        driver_type,
        coefficients: Vec::new(),
    }
}

pub fn driver_evaluate(driver: &DriverExport, input: f32) -> f32 {
    match driver.driver_type {
        DriverType::Scripted => 0.0,
        DriverType::Linear => {
            let c0 = driver.coefficients.first().copied().unwrap_or(0.0);
            let c1 = driver.coefficients.get(1).copied().unwrap_or(1.0);
            c0 + c1 * input
        }
        DriverType::Polynomial => driver
            .coefficients
            .iter()
            .enumerate()
            .fold(0.0f32, |acc, (i, &c)| acc + c * input.powi(i as i32)),
    }
}

pub fn driver_add_coefficient(driver: &mut DriverExport, coeff: f32) {
    driver.coefficients.push(coeff);
}

pub fn driver_coefficient_count(driver: &DriverExport) -> usize {
    driver.coefficients.len()
}

pub fn driver_to_json_legacy(driver: &DriverExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"type\":\"{}\",\"coefficients\":{}}}",
        driver.name,
        driver_type_name(driver),
        driver.coefficients.len()
    )
}

pub fn driver_validate(driver: &DriverExport) -> bool {
    !driver.name.is_empty() && !driver.target_prop.is_empty()
}

pub fn driver_type_name(driver: &DriverExport) -> &'static str {
    match driver.driver_type {
        DriverType::Linear => "Linear",
        DriverType::Polynomial => "Polynomial",
        DriverType::Scripted => "Scripted",
    }
}

/* ── spec functions (wave 150B) ── */

/// Spec-style driver data.
#[derive(Debug, Clone)]
pub struct DriverData {
    pub name: String,
    pub expression: String,
    pub target_prop: String,
    pub variables: Vec<String>,
}

/// Create a new `DriverData`.
pub fn new_driver_data(name: &str, expression: &str, target_prop: &str) -> DriverData {
    DriverData {
        name: name.to_string(),
        expression: expression.to_string(),
        target_prop: target_prop.to_string(),
        variables: Vec::new(),
    }
}

/// Push a variable name.
pub fn driver_push_variable(d: &mut DriverData, var: &str) {
    d.variables.push(var.to_string());
}

/// Serialize to JSON.
pub fn driver_to_json(d: &DriverData) -> String {
    format!(
        "{{\"name\":\"{}\",\"expression\":\"{}\",\"target\":\"{}\",\"vars\":{}}}",
        d.name,
        d.expression,
        d.target_prop,
        d.variables.len()
    )
}

/// Number of variables.
pub fn driver_variable_count(d: &DriverData) -> usize {
    d.variables.len()
}

/// Returns true if the expression is non-empty.
pub fn driver_has_expression(d: &DriverData) -> bool {
    !d.expression.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_driver_data() {
        let d = new_driver_data("drv", "x*2", "loc.x");
        assert_eq!(d.name, "drv");
    }

    #[test]
    fn test_driver_push_variable() {
        let mut d = new_driver_data("d", "e", "p");
        driver_push_variable(&mut d, "var1");
        assert_eq!(driver_variable_count(&d), 1);
    }

    #[test]
    fn test_driver_to_json() {
        let d = new_driver_data("d", "x+1", "rot.y");
        let j = driver_to_json(&d);
        assert!(j.contains("x+1"));
    }

    #[test]
    fn test_driver_has_expression() {
        let d = new_driver_data("d", "x", "p");
        assert!(driver_has_expression(&d));
    }

    #[test]
    fn test_driver_no_expression() {
        let d = new_driver_data("d", "", "p");
        assert!(!driver_has_expression(&d));
    }
}
