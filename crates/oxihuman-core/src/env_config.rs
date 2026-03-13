// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Environment variable config reader stub (no real env reads in tests).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EnvEntry {
    pub key: String,
    pub value: String,
    pub default: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EnvConfig {
    pub entries: Vec<EnvEntry>,
}

#[allow(dead_code)]
pub fn new_env_config() -> EnvConfig {
    EnvConfig { entries: Vec::new() }
}

#[allow(dead_code)]
pub fn env_register(cfg: &mut EnvConfig, key: &str, default: &str) {
    if !cfg.entries.iter().any(|e| e.key == key) {
        cfg.entries.push(EnvEntry {
            key: key.to_string(),
            value: default.to_string(),
            default: default.to_string(),
        });
    }
}

#[allow(dead_code)]
pub fn env_set(cfg: &mut EnvConfig, key: &str, val: &str) {
    if let Some(entry) = cfg.entries.iter_mut().find(|e| e.key == key) {
        entry.value = val.to_string();
    }
}

#[allow(dead_code)]
pub fn env_get<'a>(cfg: &'a EnvConfig, key: &str) -> &'a str {
    cfg.entries
        .iter()
        .find(|e| e.key == key)
        .map(|e| e.value.as_str())
        .unwrap_or("")
}

#[allow(dead_code)]
pub fn env_has_override(cfg: &EnvConfig, key: &str) -> bool {
    cfg.entries
        .iter()
        .find(|e| e.key == key)
        .map(|e| e.value != e.default)
        .unwrap_or(false)
}

#[allow(dead_code)]
pub fn env_reset(cfg: &mut EnvConfig, key: &str) {
    if let Some(entry) = cfg.entries.iter_mut().find(|e| e.key == key) {
        entry.value = entry.default.clone();
    }
}

#[allow(dead_code)]
pub fn env_reset_all(cfg: &mut EnvConfig) {
    for entry in &mut cfg.entries {
        entry.value = entry.default.clone();
    }
}

#[allow(dead_code)]
pub fn env_count(cfg: &EnvConfig) -> usize {
    cfg.entries.len()
}

#[allow(dead_code)]
pub fn env_to_json(cfg: &EnvConfig) -> String {
    let entries: Vec<String> = cfg
        .entries
        .iter()
        .map(|e| format!("\"{}\":\"{}\"", e.key, e.value))
        .collect();
    format!("{{{}}}", entries.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let cfg = new_env_config();
        assert_eq!(env_count(&cfg), 0);
    }

    #[test]
    fn test_register_and_get() {
        let mut cfg = new_env_config();
        env_register(&mut cfg, "LOG_LEVEL", "info");
        assert_eq!(env_get(&cfg, "LOG_LEVEL"), "info");
    }

    #[test]
    fn test_set_override() {
        let mut cfg = new_env_config();
        env_register(&mut cfg, "DEBUG", "false");
        env_set(&mut cfg, "DEBUG", "true");
        assert_eq!(env_get(&cfg, "DEBUG"), "true");
        assert!(env_has_override(&cfg, "DEBUG"));
    }

    #[test]
    fn test_reset() {
        let mut cfg = new_env_config();
        env_register(&mut cfg, "FOO", "default");
        env_set(&mut cfg, "FOO", "custom");
        env_reset(&mut cfg, "FOO");
        assert_eq!(env_get(&cfg, "FOO"), "default");
        assert!(!env_has_override(&cfg, "FOO"));
    }

    #[test]
    fn test_reset_all() {
        let mut cfg = new_env_config();
        env_register(&mut cfg, "A", "1");
        env_register(&mut cfg, "B", "2");
        env_set(&mut cfg, "A", "100");
        env_set(&mut cfg, "B", "200");
        env_reset_all(&mut cfg);
        assert_eq!(env_get(&cfg, "A"), "1");
        assert_eq!(env_get(&cfg, "B"), "2");
    }

    #[test]
    fn test_count() {
        let mut cfg = new_env_config();
        env_register(&mut cfg, "X", "x");
        env_register(&mut cfg, "Y", "y");
        assert_eq!(env_count(&cfg), 2);
    }

    #[test]
    fn test_get_missing_key() {
        let cfg = new_env_config();
        assert_eq!(env_get(&cfg, "MISSING"), "");
    }

    #[test]
    fn test_no_duplicate_register() {
        let mut cfg = new_env_config();
        env_register(&mut cfg, "KEY", "v1");
        env_register(&mut cfg, "KEY", "v2");
        assert_eq!(env_count(&cfg), 1);
        assert_eq!(env_get(&cfg, "KEY"), "v1");
    }

    #[test]
    fn test_to_json() {
        let mut cfg = new_env_config();
        env_register(&mut cfg, "K", "V");
        let json = env_to_json(&cfg);
        assert!(json.contains("\"K\":\"V\""));
    }
}
