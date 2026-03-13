// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Service discovery registry stub — register and resolve service endpoints.

/// Metadata describing a service instance.
#[derive(Clone, Debug)]
pub struct ServiceInstance {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub version: String,
    pub healthy: bool,
    pub tags: Vec<String>,
}

/// Configuration for the service registry.
#[derive(Clone, Debug)]
pub struct ServiceRegistryConfig {
    pub max_instances_per_service: usize,
}

impl Default for ServiceRegistryConfig {
    fn default() -> Self {
        Self {
            max_instances_per_service: 16,
        }
    }
}

/// An in-memory service registry.
pub struct ServiceRegistry {
    pub config: ServiceRegistryConfig,
    instances: Vec<ServiceInstance>,
}

/// Creates a new service registry.
pub fn new_registry(config: ServiceRegistryConfig) -> ServiceRegistry {
    ServiceRegistry {
        config,
        instances: Vec::new(),
    }
}

/// Registers a service instance, returning false if the service is at capacity.
pub fn register_instance(reg: &mut ServiceRegistry, instance: ServiceInstance) -> bool {
    let count = reg
        .instances
        .iter()
        .filter(|i| i.name == instance.name)
        .count();
    if count >= reg.config.max_instances_per_service {
        return false;
    }
    reg.instances.push(instance);
    true
}

/// Deregisters an instance by name and host+port combination.
pub fn deregister_instance(reg: &mut ServiceRegistry, name: &str, host: &str, port: u16) -> bool {
    let before = reg.instances.len();
    reg.instances
        .retain(|i| !(i.name == name && i.host == host && i.port == port));
    reg.instances.len() < before
}

/// Resolves healthy instances of a service by name.
pub fn resolve_service<'a>(reg: &'a ServiceRegistry, name: &str) -> Vec<&'a ServiceInstance> {
    reg.instances
        .iter()
        .filter(|i| i.name == name && i.healthy)
        .collect()
}

/// Marks all instances of a service as healthy or unhealthy.
pub fn set_service_health(reg: &mut ServiceRegistry, name: &str, healthy: bool) {
    for inst in reg.instances.iter_mut().filter(|i| i.name == name) {
        inst.healthy = healthy;
    }
}

/// Returns the total number of registered instances.
pub fn total_instance_count(reg: &ServiceRegistry) -> usize {
    reg.instances.len()
}

impl ServiceRegistry {
    /// Creates a new registry with default config.
    pub fn new(config: ServiceRegistryConfig) -> Self {
        new_registry(config)
    }
}

fn make_instance(name: &str, host: &str, port: u16) -> ServiceInstance {
    ServiceInstance {
        name: name.into(),
        host: host.into(),
        port,
        version: "1.0".into(),
        healthy: true,
        tags: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_reg() -> ServiceRegistry {
        new_registry(ServiceRegistryConfig::default())
    }

    #[test]
    fn test_register_and_count() {
        let mut reg = make_reg();
        let ok = register_instance(&mut reg, make_instance("svc-a", "host1", 8080));
        assert!(ok);
        assert_eq!(total_instance_count(&reg), 1);
    }

    #[test]
    fn test_resolve_healthy_instance() {
        let mut reg = make_reg();
        register_instance(&mut reg, make_instance("svc-b", "host2", 9000));
        let found = resolve_service(&reg, "svc-b");
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn test_unhealthy_instance_not_resolved() {
        let mut reg = make_reg();
        let mut inst = make_instance("svc-c", "h", 80);
        inst.healthy = false;
        register_instance(&mut reg, inst);
        assert!(resolve_service(&reg, "svc-c").is_empty());
    }

    #[test]
    fn test_deregister_removes_instance() {
        let mut reg = make_reg();
        register_instance(&mut reg, make_instance("svc-d", "h", 80));
        let removed = deregister_instance(&mut reg, "svc-d", "h", 80);
        assert!(removed);
        assert_eq!(total_instance_count(&reg), 0);
    }

    #[test]
    fn test_deregister_nonexistent_returns_false() {
        let mut reg = make_reg();
        assert!(!deregister_instance(&mut reg, "none", "h", 80));
    }

    #[test]
    fn test_set_service_health_to_unhealthy() {
        let mut reg = make_reg();
        register_instance(&mut reg, make_instance("svc-e", "h", 80));
        set_service_health(&mut reg, "svc-e", false);
        assert!(resolve_service(&reg, "svc-e").is_empty());
    }

    #[test]
    fn test_capacity_limit_enforced() {
        let mut reg = new_registry(ServiceRegistryConfig {
            max_instances_per_service: 2,
        });
        register_instance(&mut reg, make_instance("svc-f", "h1", 1));
        register_instance(&mut reg, make_instance("svc-f", "h2", 2));
        let ok = register_instance(&mut reg, make_instance("svc-f", "h3", 3));
        assert!(!ok);
    }

    #[test]
    fn test_multiple_services_independent() {
        let mut reg = make_reg();
        register_instance(&mut reg, make_instance("alpha", "h1", 80));
        register_instance(&mut reg, make_instance("beta", "h2", 80));
        assert_eq!(resolve_service(&reg, "alpha").len(), 1);
        assert_eq!(resolve_service(&reg, "beta").len(), 1);
    }

    #[test]
    fn test_resolve_returns_empty_for_unknown_service() {
        let reg = make_reg();
        assert!(resolve_service(&reg, "unknown").is_empty());
    }
}
