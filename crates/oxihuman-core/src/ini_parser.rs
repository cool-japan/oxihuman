//! Minimal INI configuration file parser.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IniSection {
    pub name: String,
    pub entries: Vec<(String, String)>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IniConfig {
    pub sections: Vec<IniSection>,
    pub default_section: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IniParseError {
    pub line: usize,
    pub message: String,
}

#[allow(dead_code)]
pub fn parse_ini(text: &str) -> Result<IniConfig, IniParseError> {
    let mut cfg = IniConfig {
        sections: Vec::new(),
        default_section: String::from("default"),
    };
    let mut current_section = String::from("default");

    for (line_num, raw_line) in text.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            if !line.ends_with(']') {
                return Err(IniParseError {
                    line: line_num + 1,
                    message: format!("Malformed section header: {}", line),
                });
            }
            current_section = line[1..line.len() - 1].trim().to_string();
            if !cfg.sections.iter().any(|s| s.name == current_section) {
                cfg.sections.push(IniSection {
                    name: current_section.clone(),
                    entries: Vec::new(),
                });
            }
        } else if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim().to_string();
            let value = line[eq_pos + 1..].trim().to_string();
            if key.is_empty() {
                return Err(IniParseError {
                    line: line_num + 1,
                    message: String::from("Empty key"),
                });
            }
            // Find or create section
            if let Some(sec) = cfg.sections.iter_mut().find(|s| s.name == current_section) {
                sec.entries.push((key, value));
            } else {
                cfg.sections.push(IniSection {
                    name: current_section.clone(),
                    entries: vec![(key, value)],
                });
            }
        } else {
            return Err(IniParseError {
                line: line_num + 1,
                message: format!("Invalid line: {}", line),
            });
        }
    }
    Ok(cfg)
}

#[allow(dead_code)]
pub fn ini_get<'a>(cfg: &'a IniConfig, section: &str, key: &str) -> Option<&'a str> {
    cfg.sections
        .iter()
        .find(|s| s.name == section)
        .and_then(|s| s.entries.iter().find(|(k, _)| k == key))
        .map(|(_, v)| v.as_str())
}

#[allow(dead_code)]
pub fn ini_get_or<'a>(cfg: &'a IniConfig, section: &str, key: &str, default: &'a str) -> &'a str {
    ini_get(cfg, section, key).unwrap_or(default)
}

#[allow(dead_code)]
pub fn ini_set(cfg: &mut IniConfig, section: &str, key: &str, value: &str) {
    if let Some(sec) = cfg.sections.iter_mut().find(|s| s.name == section) {
        if let Some(entry) = sec.entries.iter_mut().find(|(k, _)| k == key) {
            entry.1 = value.to_string();
        } else {
            sec.entries.push((key.to_string(), value.to_string()));
        }
    } else {
        cfg.sections.push(IniSection {
            name: section.to_string(),
            entries: vec![(key.to_string(), value.to_string())],
        });
    }
}

#[allow(dead_code)]
pub fn ini_to_string(cfg: &IniConfig) -> String {
    let mut out = String::new();
    for sec in &cfg.sections {
        out.push_str(&format!("[{}]\n", sec.name));
        for (k, v) in &sec.entries {
            out.push_str(&format!("{} = {}\n", k, v));
        }
        out.push('\n');
    }
    out
}

#[allow(dead_code)]
pub fn ini_section_count(cfg: &IniConfig) -> usize {
    cfg.sections.len()
}

#[allow(dead_code)]
pub fn ini_key_count(cfg: &IniConfig, section: &str) -> usize {
    cfg.sections
        .iter()
        .find(|s| s.name == section)
        .map(|s| s.entries.len())
        .unwrap_or(0)
}

#[allow(dead_code)]
pub fn ini_has_section(cfg: &IniConfig, section: &str) -> bool {
    cfg.sections.iter().any(|s| s.name == section)
}

#[allow(dead_code)]
pub fn ini_has_key(cfg: &IniConfig, section: &str, key: &str) -> bool {
    cfg.sections
        .iter()
        .find(|s| s.name == section)
        .map(|s| s.entries.iter().any(|(k, _)| k == key))
        .unwrap_or(false)
}

#[allow(dead_code)]
pub fn ini_remove_key(cfg: &mut IniConfig, section: &str, key: &str) -> bool {
    if let Some(sec) = cfg.sections.iter_mut().find(|s| s.name == section) {
        let before = sec.entries.len();
        sec.entries.retain(|(k, _)| k != key);
        sec.entries.len() < before
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let cfg = parse_ini("").unwrap();
        assert_eq!(ini_section_count(&cfg), 0);
    }

    #[test]
    fn test_parse_single_section() {
        let text = "[server]\nhost = localhost\nport = 8080\n";
        let cfg = parse_ini(text).unwrap();
        assert!(ini_has_section(&cfg, "server"));
        assert_eq!(ini_get(&cfg, "server", "host"), Some("localhost"));
        assert_eq!(ini_get(&cfg, "server", "port"), Some("8080"));
    }

    #[test]
    fn test_parse_multiple_sections() {
        let text = "[a]\nk1 = v1\n[b]\nk2 = v2\n";
        let cfg = parse_ini(text).unwrap();
        assert_eq!(ini_section_count(&cfg), 2);
        assert_eq!(ini_get(&cfg, "a", "k1"), Some("v1"));
        assert_eq!(ini_get(&cfg, "b", "k2"), Some("v2"));
    }

    #[test]
    fn test_ini_get_or_fallback() {
        let cfg = parse_ini("[s]\nkey = val\n").unwrap();
        let result = ini_get_or(&cfg, "s", "missing", "fallback");
        assert_eq!(result, "fallback");
    }

    #[test]
    fn test_ini_set_new_key() {
        let mut cfg = parse_ini("[s]\nk = v\n").unwrap();
        ini_set(&mut cfg, "s", "new_key", "new_val");
        assert_eq!(ini_get(&cfg, "s", "new_key"), Some("new_val"));
    }

    #[test]
    fn test_ini_set_update_existing() {
        let mut cfg = parse_ini("[s]\nk = old\n").unwrap();
        ini_set(&mut cfg, "s", "k", "new");
        assert_eq!(ini_get(&cfg, "s", "k"), Some("new"));
    }

    #[test]
    fn test_ini_to_string_roundtrip() {
        let text = "[section]\nfoo = bar\nbaz = 42\n\n";
        let cfg = parse_ini(text).unwrap();
        let out = ini_to_string(&cfg);
        let cfg2 = parse_ini(&out).unwrap();
        assert_eq!(ini_get(&cfg2, "section", "foo"), Some("bar"));
        assert_eq!(ini_get(&cfg2, "section", "baz"), Some("42"));
    }

    #[test]
    fn test_ini_remove_key() {
        let mut cfg = parse_ini("[s]\na = 1\nb = 2\n").unwrap();
        let removed = ini_remove_key(&mut cfg, "s", "a");
        assert!(removed);
        assert!(!ini_has_key(&cfg, "s", "a"));
        assert!(ini_has_key(&cfg, "s", "b"));
    }

    #[test]
    fn test_ini_remove_nonexistent_key() {
        let mut cfg = parse_ini("[s]\na = 1\n").unwrap();
        let removed = ini_remove_key(&mut cfg, "s", "z");
        assert!(!removed);
    }

    #[test]
    fn test_ini_key_count() {
        let cfg = parse_ini("[sec]\na = 1\nb = 2\nc = 3\n").unwrap();
        assert_eq!(ini_key_count(&cfg, "sec"), 3);
        assert_eq!(ini_key_count(&cfg, "nonexistent"), 0);
    }

    #[test]
    fn test_parse_comments_skipped() {
        let text = "; comment\n[s]\n# another\nk = v\n";
        let cfg = parse_ini(text).unwrap();
        assert_eq!(ini_key_count(&cfg, "s"), 1);
    }

    #[test]
    fn test_parse_invalid_line_error() {
        let result = parse_ini("[s]\nnot_valid_line\n");
        assert!(result.is_err());
    }

    #[test]
    fn test_ini_set_creates_section() {
        let mut cfg = parse_ini("").unwrap();
        ini_set(&mut cfg, "new_section", "key", "val");
        assert!(ini_has_section(&cfg, "new_section"));
        assert_eq!(ini_get(&cfg, "new_section", "key"), Some("val"));
    }
}
