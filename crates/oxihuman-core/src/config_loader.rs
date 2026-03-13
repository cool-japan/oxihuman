// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A single key-value configuration entry.
#[allow(dead_code)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
}

/// A configuration loaded from key=value text.
#[allow(dead_code)]
pub struct Config {
    pub entries: Vec<ConfigEntry>,
}

/// Load a `Config` from a `key=value` text (one per line, `#` lines are comments).
#[allow(dead_code)]
pub fn load_config(text: &str) -> Config {
    let mut entries = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(eq) = line.find('=') {
            let key = line[..eq].trim().to_string();
            let value = line[eq + 1..].trim().to_string();
            if !key.is_empty() {
                entries.push(ConfigEntry { key, value });
            }
        }
    }
    Config { entries }
}

/// Get the value for a key as a string slice.
#[allow(dead_code)]
pub fn config_get<'a>(cfg: &'a Config, key: &str) -> Option<&'a str> {
    cfg.entries
        .iter()
        .rev()
        .find(|e| e.key == key)
        .map(|e| e.value.as_str())
}

/// Get the value for a key parsed as `i64`.
#[allow(dead_code)]
pub fn config_get_int(cfg: &Config, key: &str) -> Option<i64> {
    config_get(cfg, key)?.parse::<i64>().ok()
}

/// Get the value for a key parsed as `f64`.
#[allow(dead_code)]
pub fn config_get_float(cfg: &Config, key: &str) -> Option<f64> {
    config_get(cfg, key)?.parse::<f64>().ok()
}

/// Set (or append) a key-value pair.
#[allow(dead_code)]
pub fn config_set(cfg: &mut Config, key: &str, val: &str) {
    if let Some(entry) = cfg.entries.iter_mut().find(|e| e.key == key) {
        entry.value = val.to_string();
    } else {
        cfg.entries.push(ConfigEntry {
            key: key.to_string(),
            value: val.to_string(),
        });
    }
}

/// Return the number of entries.
#[allow(dead_code)]
pub fn config_entry_count(cfg: &Config) -> usize {
    cfg.entries.len()
}

#[allow(dead_code)]
pub struct ConfigLoader {
    pub sections: std::collections::HashMap<String, std::collections::HashMap<String, String>>,
}

#[allow(dead_code)]
pub fn new_config_loader() -> ConfigLoader {
    ConfigLoader { sections: std::collections::HashMap::new() }
}

#[allow(dead_code)]
pub fn cl_set(loader: &mut ConfigLoader, section: &str, key: &str, value: &str) {
    loader
        .sections
        .entry(section.to_string())
        .or_default()
        .insert(key.to_string(), value.to_string());
}

#[allow(dead_code)]
pub fn cl_get<'a>(loader: &'a ConfigLoader, section: &str, key: &str) -> Option<&'a str> {
    loader.sections.get(section)?.get(key).map(|s| s.as_str())
}

#[allow(dead_code)]
pub fn cl_get_f64(loader: &ConfigLoader, section: &str, key: &str) -> Option<f64> {
    cl_get(loader, section, key)?.parse::<f64>().ok()
}

#[allow(dead_code)]
pub fn cl_section_count(loader: &ConfigLoader) -> usize {
    loader.sections.len()
}

#[allow(dead_code)]
pub fn cl_key_count(loader: &ConfigLoader, section: &str) -> usize {
    loader.sections.get(section).map_or(0, |m| m.len())
}

#[allow(dead_code)]
pub fn cl_clear(loader: &mut ConfigLoader) {
    loader.sections.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_simple_entry() {
        let cfg = load_config("name=Alice");
        assert_eq!(config_get(&cfg, "name"), Some("Alice"));
    }

    #[test]
    fn load_ignores_comments() {
        let cfg = load_config("# comment\nkey=val");
        assert_eq!(config_entry_count(&cfg), 1);
    }

    #[test]
    fn load_ignores_empty_lines() {
        let cfg = load_config("\n\nkey=val\n\n");
        assert_eq!(config_entry_count(&cfg), 1);
    }

    #[test]
    fn get_missing_key_returns_none() {
        let cfg = load_config("x=1");
        assert!(config_get(&cfg, "y").is_none());
    }

    #[test]
    fn get_int_parses_correctly() {
        let cfg = load_config("count=42");
        assert_eq!(config_get_int(&cfg, "count"), Some(42));
    }

    #[test]
    fn get_float_parses_correctly() {
        let cfg = load_config("ratio=0.5");
        let v = config_get_float(&cfg, "ratio").expect("should succeed");
        assert!((v - 0.5).abs() < 1e-9);
    }

    #[test]
    fn config_set_new_entry() {
        let mut cfg = load_config("");
        config_set(&mut cfg, "mode", "fast");
        assert_eq!(config_get(&cfg, "mode"), Some("fast"));
    }

    #[test]
    fn config_set_updates_existing() {
        let mut cfg = load_config("level=1");
        config_set(&mut cfg, "level", "5");
        assert_eq!(config_get(&cfg, "level"), Some("5"));
    }

    #[test]
    fn entry_count_correct() {
        let cfg = load_config("a=1\nb=2\nc=3");
        assert_eq!(config_entry_count(&cfg), 3);
    }

    #[test]
    fn whitespace_trimmed() {
        let cfg = load_config("  key  =  value  ");
        assert_eq!(config_get(&cfg, "key"), Some("value"));
    }
}
