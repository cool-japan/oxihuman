// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::HashMap;

/// Blend-shape parameter state. All values in [0.0, 1.0].
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ParamState {
    pub height: f32,
    pub weight: f32,
    pub muscle: f32,
    pub age: f32,
    /// Extensible extra parameters.
    pub extra: HashMap<String, f32>,
}

impl Default for ParamState {
    fn default() -> Self {
        ParamState {
            height: 0.5,
            weight: 0.5,
            muscle: 0.5,
            age: 0.5,
            extra: HashMap::new(),
        }
    }
}

impl ParamState {
    pub fn new(height: f32, weight: f32, muscle: f32, age: f32) -> Self {
        ParamState {
            height,
            weight,
            muscle,
            age,
            extra: HashMap::new(),
        }
    }

    /// Get any parameter by name (including the built-ins).
    pub fn get(&self, key: &str) -> Option<f32> {
        match key {
            "height" => Some(self.height),
            "weight" => Some(self.weight),
            "muscle" => Some(self.muscle),
            "age" => Some(self.age),
            other => self.extra.get(other).copied(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_params_are_midpoint() {
        let p = ParamState::default();
        assert!((p.height - 0.5).abs() < 1e-6);
        assert!((p.weight - 0.5).abs() < 1e-6);
    }

    #[test]
    fn get_by_name() {
        let mut p = ParamState::default();
        p.extra.insert("bmi".to_string(), 0.3);
        assert_eq!(p.get("height"), Some(0.5));
        assert_eq!(p.get("bmi"), Some(0.3));
        assert_eq!(p.get("missing"), None);
    }
}
