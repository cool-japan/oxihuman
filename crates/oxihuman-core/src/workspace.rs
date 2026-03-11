// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Project workspace management.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WorkspaceConfig {
    pub name: String,
    pub version: String,
    pub author: String,
    pub output_dir: String,
}

impl WorkspaceConfig {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            author: String::new(),
            output_dir: "output".to_string(),
        }
    }

    pub fn to_json(&self) -> String {
        format!(
            r#"{{"name":"{}","version":"{}","author":"{}","output_dir":"{}"}}"#,
            self.name, self.version, self.author, self.output_dir
        )
    }

    /// Parse a minimal JSON config (best-effort).
    pub fn from_json(s: &str) -> Result<Self, String> {
        let name = extract_json_str(s, "name").unwrap_or_default();
        let version = extract_json_str(s, "version").unwrap_or_else(|| "0.1.0".to_string());
        let author = extract_json_str(s, "author").unwrap_or_default();
        let output_dir = extract_json_str(s, "output_dir").unwrap_or_else(|| "output".to_string());
        if name.is_empty() {
            return Err("missing name field".to_string());
        }
        Ok(Self {
            name,
            version,
            author,
            output_dir,
        })
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AssetEntry {
    pub path: String,
    pub kind: String,
    pub hash: String,
    pub size_bytes: usize,
}

impl AssetEntry {
    pub fn new(path: &str, kind: &str, hash: &str, size_bytes: usize) -> Self {
        Self {
            path: path.to_string(),
            kind: kind.to_string(),
            hash: hash.to_string(),
            size_bytes,
        }
    }

    pub fn to_json(&self) -> String {
        format!(
            r#"{{"path":"{}","kind":"{}","hash":"{}","size_bytes":{}}}"#,
            self.path, self.kind, self.hash, self.size_bytes
        )
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Workspace {
    pub config: WorkspaceConfig,
    pub assets: Vec<AssetEntry>,
    pub dirty: bool,
}

impl Workspace {
    pub fn new(name: &str) -> Self {
        Self {
            config: WorkspaceConfig::new(name),
            assets: Vec::new(),
            dirty: false,
        }
    }

    /// Add or update an asset. Sets dirty = true.
    pub fn add_asset(&mut self, path: &str, kind: &str, hash: &str, size_bytes: usize) {
        // replace if path already exists
        if let Some(e) = self.assets.iter_mut().find(|a| a.path == path) {
            e.kind = kind.to_string();
            e.hash = hash.to_string();
            e.size_bytes = size_bytes;
        } else {
            self.assets
                .push(AssetEntry::new(path, kind, hash, size_bytes));
        }
        self.dirty = true;
    }

    /// Remove an asset. Returns true if it was found and removed.
    pub fn remove_asset(&mut self, path: &str) -> bool {
        let before = self.assets.len();
        self.assets.retain(|a| a.path != path);
        let removed = self.assets.len() < before;
        if removed {
            self.dirty = true;
        }
        removed
    }

    /// Find an asset by path.
    pub fn find_asset(&self, path: &str) -> Option<&AssetEntry> {
        self.assets.iter().find(|a| a.path == path)
    }

    /// All assets of a given kind.
    pub fn assets_by_kind(&self, kind: &str) -> Vec<&AssetEntry> {
        self.assets.iter().filter(|a| a.kind == kind).collect()
    }

    /// Sum of all asset sizes.
    pub fn total_size(&self) -> usize {
        self.assets.iter().map(|a| a.size_bytes).sum()
    }

    /// Number of assets.
    pub fn asset_count(&self) -> usize {
        self.assets.len()
    }

    /// Serialize workspace to JSON.
    pub fn to_json(&self) -> String {
        let asset_jsons: Vec<String> = self.assets.iter().map(|a| a.to_json()).collect();
        format!(
            r#"{{"config":{},"dirty":{},"assets":[{}]}}"#,
            self.config.to_json(),
            self.dirty,
            asset_jsons.join(",")
        )
    }

    /// Deserialize workspace from JSON (best-effort).
    pub fn from_json(s: &str) -> Result<Workspace, String> {
        // Extract the config sub-object
        let config_str =
            extract_json_object(s, "config").ok_or_else(|| "missing config object".to_string())?;
        let config = WorkspaceConfig::from_json(&config_str)?;
        // Parse assets array (simplified: each asset is an object {...})
        let assets = parse_asset_array(s);
        let dirty_str = extract_json_str(s, "dirty").unwrap_or_else(|| "false".to_string());
        let dirty = dirty_str == "true";
        Ok(Workspace {
            config,
            assets,
            dirty,
        })
    }

    /// Mark workspace as clean (dirty = false).
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }
}

/// Default workspace configuration with sensible defaults.
pub fn default_workspace_config() -> WorkspaceConfig {
    WorkspaceConfig {
        name: "default_project".to_string(),
        version: "0.1.0".to_string(),
        author: "Unknown".to_string(),
        output_dir: "dist".to_string(),
    }
}

/// Human-readable summary of a workspace.
pub fn workspace_summary(ws: &Workspace) -> String {
    format!(
        "Workspace '{}' v{} — {} assets, {} bytes total, dirty={}",
        ws.config.name,
        ws.config.version,
        ws.asset_count(),
        ws.total_size(),
        ws.dirty
    )
}

// ─── Minimal JSON helpers ────────────────────────────────────────────────────

/// Extract a string value for a key: "key":"value" or "key":value (booleans).
fn extract_json_str(s: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\":", key);
    let start = s.find(&pattern)? + pattern.len();
    let rest = s[start..].trim_start();
    if rest.starts_with('"') {
        // quoted string
        let inner = rest.strip_prefix('"')?;
        let end = inner.find('"')?;
        Some(inner[..end].to_string())
    } else {
        // unquoted token (number / boolean)
        let end = rest.find([',', '}', ']']).unwrap_or(rest.len());
        Some(rest[..end].trim().to_string())
    }
}

/// Extract a JSON object value for a key (returns the raw {...} substring).
fn extract_json_object(s: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\":", key);
    let start = s.find(&pattern)? + pattern.len();
    let rest = s[start..].trim_start();
    if !rest.starts_with('{') {
        return None;
    }
    let mut depth = 0usize;
    let mut end = 0;
    for (i, c) in rest.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = i + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    Some(rest[..end].to_string())
}

/// Very small asset-array parser: finds each `{...}` inside `"assets":[...]`.
fn parse_asset_array(s: &str) -> Vec<AssetEntry> {
    let mut assets = Vec::new();
    let marker = "\"assets\":[";
    let Some(start) = s.find(marker) else {
        return assets;
    };
    let rest = &s[start + marker.len()..];
    // collect all top-level { } objects
    let mut depth = 0i32;
    let mut obj_start = None;
    for (i, c) in rest.char_indices() {
        match c {
            '{' => {
                if depth == 0 {
                    obj_start = Some(i);
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    if let Some(s_start) = obj_start {
                        let obj_str = &rest[s_start..=i];
                        let path = extract_json_str(obj_str, "path").unwrap_or_default();
                        let kind = extract_json_str(obj_str, "kind").unwrap_or_default();
                        let hash = extract_json_str(obj_str, "hash").unwrap_or_default();
                        let size: usize = extract_json_str(obj_str, "size_bytes")
                            .and_then(|v| v.parse().ok())
                            .unwrap_or(0);
                        if !path.is_empty() {
                            assets.push(AssetEntry::new(&path, &kind, &hash, size));
                        }
                        obj_start = None;
                    }
                }
            }
            ']' if depth == 0 => break,
            _ => {}
        }
    }
    assets
}

// ─── Tests ───────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_workspace_is_clean() {
        let ws = Workspace::new("test");
        assert!(!ws.dirty);
    }

    #[test]
    fn new_workspace_name_stored() {
        let ws = Workspace::new("myproject");
        assert_eq!(ws.config.name, "myproject");
    }

    #[test]
    fn add_asset_increases_count() {
        let mut ws = Workspace::new("proj");
        ws.add_asset("mesh.glb", "mesh", "abc123", 1024);
        assert_eq!(ws.asset_count(), 1);
    }

    #[test]
    fn add_asset_sets_dirty() {
        let mut ws = Workspace::new("proj");
        ws.add_asset("mesh.glb", "mesh", "abc123", 1024);
        assert!(ws.dirty);
    }

    #[test]
    fn remove_asset_returns_true_when_found() {
        let mut ws = Workspace::new("proj");
        ws.add_asset("mesh.glb", "mesh", "abc123", 1024);
        assert!(ws.remove_asset("mesh.glb"));
        assert_eq!(ws.asset_count(), 0);
    }

    #[test]
    fn remove_asset_returns_false_when_missing() {
        let mut ws = Workspace::new("proj");
        assert!(!ws.remove_asset("nonexistent.glb"));
    }

    #[test]
    fn find_asset_returns_correct_entry() {
        let mut ws = Workspace::new("proj");
        ws.add_asset("tex.png", "texture", "deadbeef", 512);
        let found = ws.find_asset("tex.png");
        assert!(found.is_some());
        assert_eq!(found.unwrap().kind, "texture");
    }

    #[test]
    fn find_asset_returns_none_for_missing() {
        let ws = Workspace::new("proj");
        assert!(ws.find_asset("ghost.png").is_none());
    }

    #[test]
    fn assets_by_kind_filters_correctly() {
        let mut ws = Workspace::new("proj");
        ws.add_asset("a.glb", "mesh", "h1", 100);
        ws.add_asset("b.glb", "mesh", "h2", 200);
        ws.add_asset("c.png", "texture", "h3", 50);
        let meshes = ws.assets_by_kind("mesh");
        assert_eq!(meshes.len(), 2);
    }

    #[test]
    fn total_size_sums_correctly() {
        let mut ws = Workspace::new("proj");
        ws.add_asset("a", "mesh", "h1", 100);
        ws.add_asset("b", "mesh", "h2", 200);
        assert_eq!(ws.total_size(), 300);
    }

    #[test]
    fn mark_clean_resets_dirty() {
        let mut ws = Workspace::new("proj");
        ws.add_asset("a", "mesh", "h1", 1);
        assert!(ws.dirty);
        ws.mark_clean();
        assert!(!ws.dirty);
    }

    #[test]
    fn to_json_round_trip() {
        let mut ws = Workspace::new("roundtrip");
        ws.add_asset("mesh.glb", "mesh", "cafebabe", 4096);
        let json = ws.to_json();
        let ws2 = Workspace::from_json(&json).expect("round-trip parse failed");
        assert_eq!(ws2.config.name, "roundtrip");
        assert_eq!(ws2.asset_count(), 1);
        assert_eq!(ws2.find_asset("mesh.glb").unwrap().size_bytes, 4096);
    }

    #[test]
    fn from_json_error_on_missing_name() {
        let result = Workspace::from_json(r#"{"config":{"version":"0.1.0"},"assets":[]}"#);
        assert!(result.is_err());
    }

    #[test]
    fn workspace_summary_contains_name() {
        let ws = Workspace::new("showcase");
        let s = workspace_summary(&ws);
        assert!(s.contains("showcase"));
    }

    #[test]
    fn default_workspace_config_has_name() {
        let cfg = default_workspace_config();
        assert!(!cfg.name.is_empty());
    }
}
