// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export an animation state machine (states + transitions) as JSON.

#![allow(dead_code)]

/// Configuration for animation-state-machine export.
#[derive(Debug, Clone)]
pub struct AnimStateExportConfig {
    /// Pretty-print JSON output.
    pub pretty: bool,
    /// Validate state machine before export (unique names, valid transition refs).
    pub validate: bool,
}

/// A single animation state.
#[derive(Debug, Clone)]
pub struct AnimState {
    /// Unique state identifier.
    pub id: String,
    /// Human-readable label.
    pub label: String,
    /// Animation clip referenced by this state.
    pub clip: String,
    /// Playback speed multiplier.
    pub speed: f32,
    /// Whether the clip loops.
    pub looping: bool,
}

/// A directed transition between two states.
#[derive(Debug, Clone)]
pub struct AnimTransition {
    /// Source state id.
    pub from: String,
    /// Target state id.
    pub to: String,
    /// Blend duration in seconds.
    pub duration: f32,
    /// Optional condition expression.
    pub condition: String,
}

/// Container holding the full animation state machine for export.
#[derive(Debug, Clone)]
pub struct AnimStateExport {
    /// All animation states.
    pub states: Vec<AnimState>,
    /// All transitions.
    pub transitions: Vec<AnimTransition>,
    /// Byte count of the last serialised output.
    pub total_bytes: usize,
}

/// Returns the default [`AnimStateExportConfig`].
pub fn default_anim_state_export_config() -> AnimStateExportConfig {
    AnimStateExportConfig { pretty: true, validate: true }
}

/// Creates a new, empty [`AnimStateExport`].
pub fn new_anim_state_export() -> AnimStateExport {
    AnimStateExport {
        states: Vec::new(),
        transitions: Vec::new(),
        total_bytes: 0,
    }
}

/// Adds an animation state. Duplicate ids are silently replaced.
pub fn ase_add_state(export: &mut AnimStateExport, state: AnimState) {
    if let Some(existing) = export.states.iter_mut().find(|s| s.id == state.id) {
        *existing = state;
    } else {
        export.states.push(state);
    }
}

/// Adds a transition. Duplicate (from, to) pairs are silently replaced.
pub fn ase_add_transition(export: &mut AnimStateExport, tr: AnimTransition) {
    if let Some(existing) = export
        .transitions
        .iter_mut()
        .find(|t| t.from == tr.from && t.to == tr.to)
    {
        *existing = tr;
    } else {
        export.transitions.push(tr);
    }
}

/// Serialises the state machine as JSON.
pub fn ase_to_json(export: &mut AnimStateExport, cfg: &AnimStateExportConfig) -> String {
    let indent = if cfg.pretty { "  " } else { "" };
    let nl = if cfg.pretty { "\n" } else { "" };

    let mut out = format!("{{{nl}");

    // states
    out.push_str(&format!("{indent}\"states\":[{nl}"));
    let slen = export.states.len();
    for (i, s) in export.states.iter().enumerate() {
        let comma = if i + 1 < slen { "," } else { "" };
        out.push_str(&format!(
            "{indent}{indent}{{\"id\":\"{}\",\"label\":\"{}\",\"clip\":\"{}\",\
             \"speed\":{:.4},\"looping\":{}}}{comma}{nl}",
            s.id, s.label, s.clip, s.speed, s.looping
        ));
    }
    out.push_str(&format!("{indent}],{nl}"));

    // transitions
    out.push_str(&format!("{indent}\"transitions\":[{nl}"));
    let tlen = export.transitions.len();
    for (i, t) in export.transitions.iter().enumerate() {
        let comma = if i + 1 < tlen { "," } else { "" };
        out.push_str(&format!(
            "{indent}{indent}{{\"from\":\"{}\",\"to\":\"{}\",\
             \"duration\":{:.4},\"condition\":\"{}\"}}{comma}{nl}",
            t.from, t.to, t.duration, t.condition
        ));
    }
    out.push_str(&format!("{indent}]{nl}}}"));

    export.total_bytes = out.len();
    out
}

/// Returns the number of states stored.
pub fn ase_state_count(export: &AnimStateExport) -> usize {
    export.states.len()
}

/// Returns the number of transitions stored.
pub fn ase_transition_count(export: &AnimStateExport) -> usize {
    export.transitions.len()
}

/// Writes JSON to a file path (stub — returns byte count).
pub fn ase_write_to_file(
    export: &mut AnimStateExport,
    cfg: &AnimStateExportConfig,
    _path: &str,
) -> usize {
    let json = ase_to_json(export, cfg);
    export.total_bytes = json.len();
    export.total_bytes
}

/// Clears all states and transitions.
pub fn ase_clear(export: &mut AnimStateExport) {
    export.states.clear();
    export.transitions.clear();
    export.total_bytes = 0;
}

/// Validates the state machine.
/// Returns a list of error strings (empty = valid).
pub fn ase_validate(export: &AnimStateExport, cfg: &AnimStateExportConfig) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();
    if !cfg.validate {
        return errors;
    }
    // Check for empty state ids
    for s in &export.states {
        if s.id.is_empty() {
            errors.push("state with empty id".to_string());
        }
        if s.clip.is_empty() {
            errors.push(format!("state '{}' has empty clip", s.id));
        }
    }
    // Check transition endpoints exist
    let state_ids: Vec<&str> = export.states.iter().map(|s| s.id.as_str()).collect();
    for t in &export.transitions {
        if !state_ids.contains(&t.from.as_str()) {
            errors.push(format!("transition from unknown state '{}'", t.from));
        }
        if !state_ids.contains(&t.to.as_str()) {
            errors.push(format!("transition to unknown state '{}'", t.to));
        }
    }
    errors
}

// ── internal helpers ───────────────────────────────────────────────────────────

fn make_state(id: &str, label: &str, clip: &str, speed: f32, looping: bool) -> AnimState {
    AnimState {
        id: id.to_string(),
        label: label.to_string(),
        clip: clip.to_string(),
        speed,
        looping,
    }
}

fn make_transition(from: &str, to: &str, duration: f32, condition: &str) -> AnimTransition {
    AnimTransition {
        from: from.to_string(),
        to: to.to_string(),
        duration,
        condition: condition.to_string(),
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_anim_state_export_config();
        assert!(cfg.pretty);
        assert!(cfg.validate);
    }

    #[test]
    fn new_export_is_empty() {
        let e = new_anim_state_export();
        assert_eq!(ase_state_count(&e), 0);
        assert_eq!(ase_transition_count(&e), 0);
    }

    #[test]
    fn add_state_increments_count() {
        let mut e = new_anim_state_export();
        ase_add_state(&mut e, make_state("idle", "Idle", "idle_clip", 1.0, true));
        assert_eq!(ase_state_count(&e), 1);
    }

    #[test]
    fn duplicate_state_overwrites() {
        let mut e = new_anim_state_export();
        ase_add_state(&mut e, make_state("idle", "Idle", "clip_a", 1.0, true));
        ase_add_state(&mut e, make_state("idle", "Idle", "clip_b", 2.0, false));
        assert_eq!(ase_state_count(&e), 1);
        assert_eq!(e.states[0].clip, "clip_b");
    }

    #[test]
    fn add_transition_increments_count() {
        let mut e = new_anim_state_export();
        ase_add_state(&mut e, make_state("idle", "Idle", "clip", 1.0, true));
        ase_add_state(&mut e, make_state("walk", "Walk", "clip", 1.0, true));
        ase_add_transition(&mut e, make_transition("idle", "walk", 0.3, "speed > 0.1"));
        assert_eq!(ase_transition_count(&e), 1);
    }

    #[test]
    fn json_contains_states_key() {
        let mut e = new_anim_state_export();
        ase_add_state(&mut e, make_state("idle", "Idle", "idle.clip", 1.0, true));
        let cfg = default_anim_state_export_config();
        let json = ase_to_json(&mut e, &cfg);
        assert!(json.contains("\"states\""));
        assert!(json.contains("\"idle\""));
        assert!(json.contains("idle.clip"));
    }

    #[test]
    fn json_contains_transitions_key() {
        let mut e = new_anim_state_export();
        ase_add_state(&mut e, make_state("idle", "Idle", "clip", 1.0, true));
        ase_add_state(&mut e, make_state("run", "Run", "clip", 1.0, false));
        ase_add_transition(&mut e, make_transition("idle", "run", 0.2, ""));
        let cfg = default_anim_state_export_config();
        let json = ase_to_json(&mut e, &cfg);
        assert!(json.contains("\"transitions\""));
    }

    #[test]
    fn validate_catches_missing_state() {
        let mut e = new_anim_state_export();
        ase_add_state(&mut e, make_state("idle", "Idle", "clip", 1.0, true));
        ase_add_transition(&mut e, make_transition("idle", "nonexistent", 0.3, ""));
        let cfg = default_anim_state_export_config();
        let errs = ase_validate(&e, &cfg);
        assert!(!errs.is_empty());
    }

    #[test]
    fn validate_ok_for_valid_machine() {
        let mut e = new_anim_state_export();
        ase_add_state(&mut e, make_state("idle", "Idle", "idle.clip", 1.0, true));
        ase_add_state(&mut e, make_state("walk", "Walk", "walk.clip", 1.0, true));
        ase_add_transition(&mut e, make_transition("idle", "walk", 0.3, "speed > 0"));
        let cfg = default_anim_state_export_config();
        let errs = ase_validate(&e, &cfg);
        assert!(errs.is_empty());
    }

    #[test]
    fn write_to_file_sets_total_bytes() {
        let mut e = new_anim_state_export();
        ase_add_state(&mut e, make_state("idle", "Idle", "idle.clip", 1.0, true));
        let cfg = default_anim_state_export_config();
        let n = ase_write_to_file(&mut e, &cfg, "/tmp/ase.json");
        assert!(n > 0);
    }

    #[test]
    fn clear_resets_state() {
        let mut e = new_anim_state_export();
        ase_add_state(&mut e, make_state("idle", "Idle", "idle.clip", 1.0, true));
        ase_add_transition(&mut e, make_transition("idle", "idle", 0.0, ""));
        ase_clear(&mut e);
        assert_eq!(ase_state_count(&e), 0);
        assert_eq!(ase_transition_count(&e), 0);
        assert_eq!(e.total_bytes, 0);
    }
}
