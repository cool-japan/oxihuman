//! Plugin registration and lifecycle API.

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum PluginState {
    Unloaded,
    Loaded,
    Active,
    Error(String),
}

#[allow(dead_code)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: [u32; 3], // major, minor, patch
    pub author: String,
    pub description: String,
    pub dependencies: Vec<String>,
}

#[allow(dead_code)]
pub struct Plugin {
    pub metadata: PluginMetadata,
    pub state: PluginState,
    pub load_order: u32,
}

#[allow(dead_code)]
pub struct PluginApiRegistry {
    pub plugins: Vec<Plugin>,
    pub next_order: u32,
}

#[allow(dead_code)]
pub fn new_registry() -> PluginApiRegistry {
    PluginApiRegistry {
        plugins: Vec::new(),
        next_order: 0,
    }
}

#[allow(dead_code)]
pub fn register_plugin(registry: &mut PluginApiRegistry, meta: PluginMetadata) -> usize {
    let order = registry.next_order;
    registry.next_order += 1;
    let plugin = Plugin {
        metadata: meta,
        state: PluginState::Loaded,
        load_order: order,
    };
    registry.plugins.push(plugin);
    registry.plugins.len() - 1
}

#[allow(dead_code)]
pub fn get_plugin<'a>(registry: &'a PluginApiRegistry, id: &str) -> Option<&'a Plugin> {
    registry.plugins.iter().find(|p| p.metadata.id == id)
}

#[allow(dead_code)]
pub fn activate_plugin(registry: &mut PluginApiRegistry, id: &str) -> bool {
    if let Some(p) = registry.plugins.iter_mut().find(|p| p.metadata.id == id) {
        match p.state {
            PluginState::Loaded | PluginState::Unloaded => {
                p.state = PluginState::Active;
                true
            }
            _ => false,
        }
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn deactivate_plugin(registry: &mut PluginApiRegistry, id: &str) -> bool {
    if let Some(p) = registry.plugins.iter_mut().find(|p| p.metadata.id == id) {
        if p.state == PluginState::Active {
            p.state = PluginState::Loaded;
            true
        } else {
            false
        }
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn unload_plugin(registry: &mut PluginApiRegistry, id: &str) -> bool {
    if let Some(p) = registry.plugins.iter_mut().find(|p| p.metadata.id == id) {
        p.state = PluginState::Unloaded;
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn set_plugin_error(registry: &mut PluginApiRegistry, id: &str, msg: &str) {
    if let Some(p) = registry.plugins.iter_mut().find(|p| p.metadata.id == id) {
        p.state = PluginState::Error(msg.to_string());
    }
}

#[allow(dead_code)]
pub fn active_plugins(registry: &PluginApiRegistry) -> Vec<&Plugin> {
    registry
        .plugins
        .iter()
        .filter(|p| p.state == PluginState::Active)
        .collect()
}

#[allow(dead_code)]
pub fn plugin_count(registry: &PluginApiRegistry) -> usize {
    registry.plugins.len()
}

#[allow(dead_code)]
pub fn has_dependency(registry: &PluginApiRegistry, plugin_id: &str, dep_id: &str) -> bool {
    if let Some(p) = get_plugin(registry, plugin_id) {
        p.metadata.dependencies.iter().any(|d| d == dep_id)
    } else {
        false
    }
}

/// Topological sort of plugins by dependencies (Kahn's algorithm).
#[allow(dead_code)]
pub fn dependency_order(registry: &PluginApiRegistry) -> Vec<&str> {
    let n = registry.plugins.len();
    // Build adjacency: dep → dependent
    let mut in_degree = vec![0usize; n];
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];

    for (i, plugin) in registry.plugins.iter().enumerate() {
        for dep in &plugin.metadata.dependencies {
            if let Some(j) = registry.plugins.iter().position(|p| &p.metadata.id == dep) {
                adj[j].push(i);
                in_degree[i] += 1;
            }
        }
    }

    let mut queue: Vec<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
    let mut result = Vec::new();

    while !queue.is_empty() {
        let node = queue.remove(0);
        result.push(registry.plugins[node].metadata.id.as_str());
        for &next in &adj[node] {
            in_degree[next] -= 1;
            if in_degree[next] == 0 {
                queue.push(next);
            }
        }
    }

    result
}

#[allow(dead_code)]
pub fn plugin_version_string(plugin: &Plugin) -> String {
    let [major, minor, patch] = plugin.metadata.version;
    format!("{}.{}.{}", major, minor, patch)
}

#[allow(dead_code)]
pub fn check_dependencies_met(registry: &PluginApiRegistry, plugin_id: &str) -> bool {
    if let Some(plugin) = get_plugin(registry, plugin_id) {
        plugin.metadata.dependencies.iter().all(|dep| {
            if let Some(dep_plugin) = get_plugin(registry, dep) {
                dep_plugin.state == PluginState::Active || dep_plugin.state == PluginState::Loaded
            } else {
                false
            }
        })
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_meta(id: &str, deps: Vec<&str>) -> PluginMetadata {
        PluginMetadata {
            id: id.to_string(),
            name: format!("Plugin {}", id),
            version: [1, 0, 0],
            author: "test".to_string(),
            description: "test plugin".to_string(),
            dependencies: deps.into_iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn test_new_registry_empty() {
        let reg = new_registry();
        assert_eq!(reg.plugins.len(), 0);
        assert_eq!(reg.next_order, 0);
    }

    #[test]
    fn test_register_plugin() {
        let mut reg = new_registry();
        let idx = register_plugin(&mut reg, make_meta("foo", vec![]));
        assert_eq!(idx, 0);
        assert_eq!(reg.plugins.len(), 1);
        assert_eq!(reg.plugins[0].metadata.id, "foo");
        assert_eq!(reg.plugins[0].state, PluginState::Loaded);
    }

    #[test]
    fn test_get_plugin_found() {
        let mut reg = new_registry();
        register_plugin(&mut reg, make_meta("bar", vec![]));
        let p = get_plugin(&reg, "bar");
        assert!(p.is_some());
        assert_eq!(p.expect("should succeed").metadata.id, "bar");
    }

    #[test]
    fn test_get_plugin_not_found() {
        let reg = new_registry();
        assert!(get_plugin(&reg, "missing").is_none());
    }

    #[test]
    fn test_activate_plugin() {
        let mut reg = new_registry();
        register_plugin(&mut reg, make_meta("baz", vec![]));
        assert!(activate_plugin(&mut reg, "baz"));
        assert_eq!(
            get_plugin(&reg, "baz").expect("should succeed").state,
            PluginState::Active
        );
    }

    #[test]
    fn test_activate_plugin_missing() {
        let mut reg = new_registry();
        assert!(!activate_plugin(&mut reg, "ghost"));
    }

    #[test]
    fn test_deactivate_plugin() {
        let mut reg = new_registry();
        register_plugin(&mut reg, make_meta("qux", vec![]));
        activate_plugin(&mut reg, "qux");
        assert!(deactivate_plugin(&mut reg, "qux"));
        assert_eq!(
            get_plugin(&reg, "qux").expect("should succeed").state,
            PluginState::Loaded
        );
    }

    #[test]
    fn test_active_plugins_list() {
        let mut reg = new_registry();
        register_plugin(&mut reg, make_meta("a", vec![]));
        register_plugin(&mut reg, make_meta("b", vec![]));
        activate_plugin(&mut reg, "a");
        let active = active_plugins(&reg);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].metadata.id, "a");
    }

    #[test]
    fn test_set_plugin_error() {
        let mut reg = new_registry();
        register_plugin(&mut reg, make_meta("err_plugin", vec![]));
        set_plugin_error(&mut reg, "err_plugin", "init failed");
        let p = get_plugin(&reg, "err_plugin").expect("should succeed");
        assert!(matches!(&p.state, PluginState::Error(msg) if msg == "init failed"));
    }

    #[test]
    fn test_plugin_version_string() {
        let mut reg = new_registry();
        register_plugin(
            &mut reg,
            PluginMetadata {
                id: "ver".to_string(),
                name: "Ver".to_string(),
                version: [2, 3, 4],
                author: "test".to_string(),
                description: "".to_string(),
                dependencies: vec![],
            },
        );
        let p = get_plugin(&reg, "ver").expect("should succeed");
        assert_eq!(plugin_version_string(p), "2.3.4");
    }

    #[test]
    fn test_has_dependency_true() {
        let mut reg = new_registry();
        register_plugin(&mut reg, make_meta("dep_a", vec![]));
        register_plugin(&mut reg, make_meta("dep_b", vec!["dep_a"]));
        assert!(has_dependency(&reg, "dep_b", "dep_a"));
    }

    #[test]
    fn test_has_dependency_false() {
        let mut reg = new_registry();
        register_plugin(&mut reg, make_meta("solo", vec![]));
        assert!(!has_dependency(&reg, "solo", "nonexistent"));
    }

    #[test]
    fn test_dependency_order() {
        let mut reg = new_registry();
        register_plugin(&mut reg, make_meta("base", vec![]));
        register_plugin(&mut reg, make_meta("mid", vec!["base"]));
        register_plugin(&mut reg, make_meta("top", vec!["mid"]));
        let order = dependency_order(&reg);
        assert_eq!(order.len(), 3);
        let base_pos = order
            .iter()
            .position(|&s| s == "base")
            .expect("should succeed");
        let mid_pos = order
            .iter()
            .position(|&s| s == "mid")
            .expect("should succeed");
        let top_pos = order
            .iter()
            .position(|&s| s == "top")
            .expect("should succeed");
        assert!(base_pos < mid_pos);
        assert!(mid_pos < top_pos);
    }

    #[test]
    fn test_check_dependencies_met() {
        let mut reg = new_registry();
        register_plugin(&mut reg, make_meta("lib", vec![]));
        register_plugin(&mut reg, make_meta("app", vec!["lib"]));
        // lib is Loaded (not just unloaded), so deps are met
        assert!(check_dependencies_met(&reg, "app"));
    }

    #[test]
    fn test_plugin_count() {
        let mut reg = new_registry();
        register_plugin(&mut reg, make_meta("x", vec![]));
        register_plugin(&mut reg, make_meta("y", vec![]));
        assert_eq!(plugin_count(&reg), 2);
    }
}
