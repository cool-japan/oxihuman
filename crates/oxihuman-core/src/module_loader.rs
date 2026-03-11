//! Dynamic module loader stub — registry of named modules with load/unload lifecycle.
//! Provides a deterministic in-memory module registry for testing and integration.

use std::collections::HashMap;

// ── Types ─────────────────────────────────────────────────────────────────────

/// Lifecycle status of a registered module.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleStatus {
    /// Module is registered but not loaded.
    Unloaded,
    /// Module is currently being loaded (transition state).
    Loading,
    /// Module is fully loaded and active.
    Loaded,
    /// Module encountered an error during load.
    Error,
}

/// Registry entry for a single module.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ModuleEntry {
    /// Name of the module.
    pub name: String,
    /// Version string (e.g. `"1.0.0"`).
    pub version: String,
    /// Current status.
    pub status: ModuleStatus,
}

/// Configuration for the module loader.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ModuleLoaderConfig {
    /// Maximum number of modules that can be registered.
    pub max_modules: usize,
    /// Whether loading a module that is already loaded should be a no-op.
    pub allow_reload: bool,
}

/// Runtime state of the module loader.
#[allow(dead_code)]
pub struct ModuleLoader {
    /// Active configuration.
    pub config: ModuleLoaderConfig,
    /// Map from module name to entry.
    pub modules: HashMap<String, ModuleEntry>,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Return a sensible default `ModuleLoaderConfig`.
#[allow(dead_code)]
pub fn default_module_loader_config() -> ModuleLoaderConfig {
    ModuleLoaderConfig {
        max_modules: 256,
        allow_reload: true,
    }
}

/// Construct a fresh `ModuleLoader` from the given config.
#[allow(dead_code)]
pub fn new_module_loader(cfg: &ModuleLoaderConfig) -> ModuleLoader {
    ModuleLoader {
        config: cfg.clone(),
        modules: HashMap::new(),
    }
}

/// Register a module by name and version.  If the module is already registered
/// its entry is left unchanged.
#[allow(dead_code)]
pub fn register_module(loader: &mut ModuleLoader, name: &str, version: &str) {
    loader.modules.entry(name.to_string()).or_insert(ModuleEntry {
        name: name.to_string(),
        version: version.to_string(),
        status: ModuleStatus::Unloaded,
    });
}

/// Load a module by name.  Returns `true` on success, `false` if the module is
/// not registered or was already loaded and `allow_reload` is false.
#[allow(dead_code)]
pub fn load_module(loader: &mut ModuleLoader, name: &str) -> bool {
    if let Some(entry) = loader.modules.get_mut(name) {
        if entry.status == ModuleStatus::Loaded && !loader.config.allow_reload {
            return false;
        }
        entry.status = ModuleStatus::Loading;
        // Simulate successful load.
        entry.status = ModuleStatus::Loaded;
        true
    } else {
        false
    }
}

/// Unload a module by name.  Returns `true` if the module was loaded and is now
/// unloaded, `false` otherwise.
#[allow(dead_code)]
pub fn unload_module(loader: &mut ModuleLoader, name: &str) -> bool {
    if let Some(entry) = loader.modules.get_mut(name) {
        if entry.status == ModuleStatus::Loaded {
            entry.status = ModuleStatus::Unloaded;
            return true;
        }
    }
    false
}

/// Query the status of a module.  Returns `ModuleStatus::Unloaded` if the
/// module is not registered.
#[allow(dead_code)]
pub fn module_status(loader: &ModuleLoader, name: &str) -> ModuleStatus {
    loader
        .modules
        .get(name)
        .map(|e| e.status)
        .unwrap_or(ModuleStatus::Unloaded)
}

/// Return the total number of registered modules.
#[allow(dead_code)]
pub fn module_count(loader: &ModuleLoader) -> usize {
    loader.modules.len()
}

/// Return the names of all currently loaded modules.
#[allow(dead_code)]
pub fn loaded_modules(loader: &ModuleLoader) -> Vec<&str> {
    loader
        .modules
        .values()
        .filter(|e| e.status == ModuleStatus::Loaded)
        .map(|e| e.name.as_str())
        .collect()
}

/// Human-readable name for a `ModuleStatus`.
#[allow(dead_code)]
pub fn module_status_name(status: ModuleStatus) -> &'static str {
    match status {
        ModuleStatus::Unloaded => "unloaded",
        ModuleStatus::Loading => "loading",
        ModuleStatus::Loaded => "loaded",
        ModuleStatus::Error => "error",
    }
}

/// Reload a module: unload then load again.  Returns `true` on success.
#[allow(dead_code)]
pub fn reload_module(loader: &mut ModuleLoader, name: &str) -> bool {
    if loader.modules.contains_key(name) {
        // Mark unloaded first, then load.
        if let Some(entry) = loader.modules.get_mut(name) {
            entry.status = ModuleStatus::Unloaded;
        }
        load_module(loader, name)
    } else {
        false
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_loader() -> ModuleLoader {
        new_module_loader(&default_module_loader_config())
    }

    #[test]
    fn test_default_config() {
        let cfg = default_module_loader_config();
        assert!(cfg.max_modules > 0);
        assert!(cfg.allow_reload);
    }

    #[test]
    fn test_new_loader_empty() {
        let loader = make_loader();
        assert_eq!(module_count(&loader), 0);
    }

    #[test]
    fn test_register_module() {
        let mut loader = make_loader();
        register_module(&mut loader, "renderer", "1.0.0");
        assert_eq!(module_count(&loader), 1);
        assert_eq!(module_status(&loader, "renderer"), ModuleStatus::Unloaded);
    }

    #[test]
    fn test_register_module_idempotent() {
        let mut loader = make_loader();
        register_module(&mut loader, "audio", "1.0.0");
        register_module(&mut loader, "audio", "2.0.0"); // second call must not overwrite
        assert_eq!(module_count(&loader), 1);
    }

    #[test]
    fn test_load_module_success() {
        let mut loader = make_loader();
        register_module(&mut loader, "physics", "1.0.0");
        let ok = load_module(&mut loader, "physics");
        assert!(ok);
        assert_eq!(module_status(&loader, "physics"), ModuleStatus::Loaded);
    }

    #[test]
    fn test_load_module_unknown_returns_false() {
        let mut loader = make_loader();
        let ok = load_module(&mut loader, "nonexistent");
        assert!(!ok);
    }

    #[test]
    fn test_unload_module_success() {
        let mut loader = make_loader();
        register_module(&mut loader, "scripting", "1.0.0");
        load_module(&mut loader, "scripting");
        let ok = unload_module(&mut loader, "scripting");
        assert!(ok);
        assert_eq!(module_status(&loader, "scripting"), ModuleStatus::Unloaded);
    }

    #[test]
    fn test_unload_already_unloaded_returns_false() {
        let mut loader = make_loader();
        register_module(&mut loader, "ui", "1.0.0");
        let ok = unload_module(&mut loader, "ui");
        assert!(!ok);
    }

    #[test]
    fn test_loaded_modules_list() {
        let mut loader = make_loader();
        register_module(&mut loader, "a", "1.0.0");
        register_module(&mut loader, "b", "1.0.0");
        load_module(&mut loader, "a");
        let loaded = loaded_modules(&loader);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0], "a");
    }

    #[test]
    fn test_module_status_name() {
        assert_eq!(module_status_name(ModuleStatus::Unloaded), "unloaded");
        assert_eq!(module_status_name(ModuleStatus::Loading), "loading");
        assert_eq!(module_status_name(ModuleStatus::Loaded), "loaded");
        assert_eq!(module_status_name(ModuleStatus::Error), "error");
    }

    #[test]
    fn test_reload_module() {
        let mut loader = make_loader();
        register_module(&mut loader, "assets", "1.0.0");
        load_module(&mut loader, "assets");
        let ok = reload_module(&mut loader, "assets");
        assert!(ok);
        assert_eq!(module_status(&loader, "assets"), ModuleStatus::Loaded);
    }

    #[test]
    fn test_reload_unregistered_module_returns_false() {
        let mut loader = make_loader();
        let ok = reload_module(&mut loader, "ghost");
        assert!(!ok);
    }

    #[test]
    fn test_status_unknown_module_is_unloaded() {
        let loader = make_loader();
        assert_eq!(module_status(&loader, "unknown"), ModuleStatus::Unloaded);
    }
}
