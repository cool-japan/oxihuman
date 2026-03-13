// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! On-demand JSON morph target load/unload/weight methods for `WasmEngine`.

use crate::engine_core::WasmEngine;

impl WasmEngine {
    // -- On-demand target streaming --

    /// Parse a JSON target definition and load it under the given name with weight 0.
    ///
    /// Expected JSON format: `{"deltas":[[vid,dx,dy,dz],...]}`
    ///
    /// Returns `true` on success, `false` on parse error.
    pub fn load_target_from_json(&mut self, name: &str, json: &str) -> bool {
        let v: serde_json::Value = match serde_json::from_str(json) {
            Ok(v) => v,
            Err(_) => return false,
        };
        let Some(arr) = v.get("deltas").and_then(|d| d.as_array()) else {
            return false;
        };
        let mut deltas: Vec<(u32, f32, f32, f32)> = Vec::with_capacity(arr.len());
        for item in arr {
            let Some(tuple) = item.as_array() else {
                return false;
            };
            if tuple.len() != 4 {
                return false;
            }
            let vid = tuple[0].as_u64().unwrap_or(0) as u32;
            let dx = tuple[1].as_f64().unwrap_or(0.0) as f32;
            let dy = tuple[2].as_f64().unwrap_or(0.0) as f32;
            let dz = tuple[3].as_f64().unwrap_or(0.0) as f32;
            deltas.push((vid, dx, dy, dz));
        }
        self.json_targets.insert(name.to_string(), (deltas, 0.0));
        self.last_mesh = None;
        true
    }

    /// Remove a JSON-loaded target by name. Returns `true` if it was present.
    pub fn unload_target(&mut self, name: &str) -> bool {
        let removed = self.json_targets.remove(name).is_some();
        if removed {
            self.last_mesh = None;
        }
        removed
    }

    /// Return a JSON array of names of all currently JSON-loaded targets.
    pub fn get_loaded_target_names(&self) -> String {
        let items: Vec<String> = self
            .json_targets
            .keys()
            .map(|n| format!("\"{}\"", n.replace('\\', "\\\\").replace('"', "\\\"")))
            .collect();
        format!("[{}]", items.join(","))
    }

    /// Set the weight for a JSON-loaded target by name. Returns `true` if found.
    pub fn set_target_weight_by_name(&mut self, name: &str, weight: f32) -> bool {
        if let Some(entry) = self.json_targets.get_mut(name) {
            entry.1 = weight;
            self.last_mesh = None;
            true
        } else {
            false
        }
    }

    /// Get the weight of a JSON-loaded target by name. Returns `-1.0` if not found.
    pub fn get_target_weight_by_name(&self, name: &str) -> f32 {
        self.json_targets.get(name).map(|(_, w)| *w).unwrap_or(-1.0)
    }

    /// Return the number of JSON-loaded targets.
    pub fn loaded_target_count(&self) -> u32 {
        self.json_targets.len() as u32
    }
}
