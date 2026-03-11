// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A named variable in the scripting environment.
#[allow(dead_code)]
pub struct ScriptVar {
    pub name: String,
    pub value: f64,
}

/// A simple scripting environment with a variable store and call stack depth.
#[allow(dead_code)]
pub struct ScriptEnv {
    pub vars: Vec<ScriptVar>,
    pub call_depth: usize,
    pub max_depth: usize,
}

/// Create a new `ScriptEnv` with a default max call depth of 64.
#[allow(dead_code)]
pub fn new_script_env() -> ScriptEnv {
    ScriptEnv {
        vars: Vec::new(),
        call_depth: 0,
        max_depth: 64,
    }
}

/// Set a variable value (creates it if not present).
#[allow(dead_code)]
pub fn env_set(env: &mut ScriptEnv, name: &str, val: f64) {
    if let Some(v) = env.vars.iter_mut().find(|v| v.name == name) {
        v.value = val;
    } else {
        env.vars.push(ScriptVar { name: name.to_string(), value: val });
    }
}

/// Get a variable value by name.
#[allow(dead_code)]
pub fn env_get(env: &ScriptEnv, name: &str) -> Option<f64> {
    env.vars.iter().find(|v| v.name == name).map(|v| v.value)
}

/// Push a call frame. Returns false if max depth exceeded.
#[allow(dead_code)]
pub fn push_frame(env: &mut ScriptEnv) -> bool {
    if env.call_depth >= env.max_depth {
        return false;
    }
    env.call_depth += 1;
    true
}

/// Pop a call frame (minimum 0).
#[allow(dead_code)]
pub fn pop_frame(env: &mut ScriptEnv) {
    if env.call_depth > 0 {
        env.call_depth -= 1;
    }
}

/// Return the current call depth.
#[allow(dead_code)]
pub fn call_depth(env: &ScriptEnv) -> usize {
    env.call_depth
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_env_is_empty() {
        let env = new_script_env();
        assert!(env.vars.is_empty());
        assert_eq!(call_depth(&env), 0);
    }

    #[test]
    fn set_and_get_variable() {
        let mut env = new_script_env();
        env_set(&mut env, "x", 2.5);
        let v = env_get(&env, "x").unwrap();
        assert!((v - 2.5).abs() < 1e-9);
    }

    #[test]
    fn get_missing_variable() {
        let env = new_script_env();
        assert!(env_get(&env, "z").is_none());
    }

    #[test]
    fn set_updates_existing() {
        let mut env = new_script_env();
        env_set(&mut env, "n", 1.0);
        env_set(&mut env, "n", 2.0);
        assert_eq!(env_get(&env, "n"), Some(2.0));
        assert_eq!(env.vars.len(), 1);
    }

    #[test]
    fn push_frame_increments_depth() {
        let mut env = new_script_env();
        assert!(push_frame(&mut env));
        assert_eq!(call_depth(&env), 1);
    }

    #[test]
    fn pop_frame_decrements_depth() {
        let mut env = new_script_env();
        push_frame(&mut env);
        pop_frame(&mut env);
        assert_eq!(call_depth(&env), 0);
    }

    #[test]
    fn pop_does_not_go_below_zero() {
        let mut env = new_script_env();
        pop_frame(&mut env);
        assert_eq!(call_depth(&env), 0);
    }

    #[test]
    fn push_frame_respects_max_depth() {
        let mut env = new_script_env();
        env.max_depth = 2;
        assert!(push_frame(&mut env));
        assert!(push_frame(&mut env));
        assert!(!push_frame(&mut env));
        assert_eq!(call_depth(&env), 2);
    }

    #[test]
    fn multiple_variables_independent() {
        let mut env = new_script_env();
        env_set(&mut env, "a", 1.0);
        env_set(&mut env, "b", 2.0);
        assert_eq!(env_get(&env, "a"), Some(1.0));
        assert_eq!(env_get(&env, "b"), Some(2.0));
    }

    #[test]
    fn default_max_depth_is_64() {
        let env = new_script_env();
        assert_eq!(env.max_depth, 64);
    }
}
