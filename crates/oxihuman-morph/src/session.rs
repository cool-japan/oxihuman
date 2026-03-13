// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Morph session serialization.
//!
//! A `MorphSession` captures the full state needed to reproduce a morphed mesh:
//! the parameter values, which targets were loaded, and where they came from.
//!
//! Privacy note: sessions store only normalized parameters (no raw geometry).

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::params::ParamState;

/// A saved morph session — everything needed to recreate a body configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphSession {
    /// Session format version.
    pub version: String,
    /// Body morphing parameters.
    pub params: SessionParams,
    /// Optional: directory that was used to load targets.
    pub targets_dir: Option<PathBuf>,
    /// Names of targets that were loaded (for documentation / reload).
    pub loaded_target_names: Vec<String>,
    /// Optional human-readable label for this session.
    pub label: Option<String>,
}

/// Serializable parameter state (mirrors ParamState but serde-friendly).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionParams {
    pub height: f32,
    pub weight: f32,
    pub muscle: f32,
    pub age: f32,
    #[serde(default)]
    pub extra: HashMap<String, f32>,
}

impl From<&ParamState> for SessionParams {
    fn from(p: &ParamState) -> Self {
        SessionParams {
            height: p.height,
            weight: p.weight,
            muscle: p.muscle,
            age: p.age,
            extra: p.extra.clone(),
        }
    }
}

impl From<SessionParams> for ParamState {
    fn from(s: SessionParams) -> Self {
        ParamState {
            height: s.height,
            weight: s.weight,
            muscle: s.muscle,
            age: s.age,
            extra: s.extra,
        }
    }
}

impl MorphSession {
    /// Create a new session from the current engine state.
    pub fn new(params: &ParamState) -> Self {
        MorphSession {
            version: "0.1.0".to_string(),
            params: SessionParams::from(params),
            targets_dir: None,
            loaded_target_names: Vec::new(),
            label: None,
        }
    }

    /// Set the label for this session.
    pub fn with_label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    /// Set the targets directory.
    pub fn with_targets_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.targets_dir = Some(dir.into());
        self
    }

    /// Add a target name to the session's loaded-targets list.
    pub fn add_target_name(&mut self, name: &str) {
        if !self.loaded_target_names.contains(&name.to_string()) {
            self.loaded_target_names.push(name.to_string());
        }
    }

    /// Reconstruct a `ParamState` from the session.
    pub fn to_param_state(&self) -> ParamState {
        ParamState {
            height: self.params.height,
            weight: self.params.weight,
            muscle: self.params.muscle,
            age: self.params.age,
            extra: self.params.extra.clone(),
        }
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Deserialize from JSON string.
    pub fn from_json(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }

    /// Save session to a JSON file.
    pub fn save(&self, path: &Path) -> Result<()> {
        let json = self.to_json()?;
        std::fs::write(path, json)
            .with_context(|| format!("saving session to {}", path.display()))?;
        Ok(())
    }

    /// Load session from a JSON file.
    pub fn load(path: &Path) -> Result<Self> {
        let json = std::fs::read_to_string(path)
            .with_context(|| format!("reading session from {}", path.display()))?;
        Self::from_json(&json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::ParamState;

    fn sample_session() -> MorphSession {
        let mut session = MorphSession::new(&ParamState::new(0.7, 0.3, 0.8, 0.2));
        session.add_target_name("height");
        session.add_target_name("muscle");
        session.loaded_target_names.push("weight".to_string());
        session
    }

    #[test]
    fn session_round_trip_json() {
        let session = sample_session();
        let json = session.to_json().expect("should succeed");
        let restored = MorphSession::from_json(&json).expect("should succeed");
        assert!((restored.params.height - 0.7).abs() < 1e-5);
        assert!((restored.params.muscle - 0.8).abs() < 1e-5);
        assert_eq!(restored.loaded_target_names.len(), 3);
    }

    #[test]
    fn session_to_param_state() {
        let session = sample_session();
        let p = session.to_param_state();
        assert!((p.height - 0.7).abs() < 1e-5);
        assert!((p.age - 0.2).abs() < 1e-5);
    }

    #[test]
    fn session_save_load_file() {
        let session = sample_session()
            .with_label("test session")
            .with_targets_dir("/tmp/targets");
        let path = std::path::PathBuf::from("/tmp/test_oxihuman_session.json");
        session.save(&path).expect("should succeed");
        let loaded = MorphSession::load(&path).expect("should succeed");
        assert_eq!(loaded.label, Some("test session".to_string()));
        assert!((loaded.params.weight - 0.3).abs() < 1e-5);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn add_target_name_deduplicates() {
        let mut session = MorphSession::new(&ParamState::default());
        session.add_target_name("height");
        session.add_target_name("height"); // duplicate
        assert_eq!(session.loaded_target_names.len(), 1);
    }

    #[test]
    fn session_with_extra_params() {
        let mut p = ParamState::default();
        p.extra.insert("expression".to_string(), 0.4);
        let session = MorphSession::new(&p);
        let json = session.to_json().expect("should succeed");
        let restored = MorphSession::from_json(&json).expect("should succeed");
        assert_eq!(restored.params.extra.get("expression").copied(), Some(0.4));
    }

    #[test]
    fn from_param_state_conversion() {
        let p = ParamState::new(0.1, 0.2, 0.3, 0.4);
        let sp = SessionParams::from(&p);
        let p2: ParamState = sp.into();
        assert!((p2.height - 0.1).abs() < 1e-5);
        assert!((p2.age - 0.4).abs() < 1e-5);
    }
}
