// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Message routing table — maps message types to handler identifiers.

/// A routing rule entry.
#[derive(Clone, Debug)]
pub struct RouteEntry {
    pub message_type: String,
    pub handler_id: String,
    pub priority: i32,
    pub enabled: bool,
}

/// A message to be routed.
#[derive(Clone, Debug)]
pub struct RoutableMessage {
    pub message_type: String,
    pub payload: String,
    pub source: String,
}

/// Routing configuration.
#[derive(Clone, Debug)]
pub struct MessageRouterConfig {
    pub default_handler: Option<String>,
    pub max_routes: usize,
}

impl Default for MessageRouterConfig {
    fn default() -> Self {
        Self {
            default_handler: None,
            max_routes: 256,
        }
    }
}

/// The message router that maps types to handlers.
pub struct MessageRouter {
    pub config: MessageRouterConfig,
    routes: Vec<RouteEntry>,
}

/// Creates a new message router.
pub fn new_router(config: MessageRouterConfig) -> MessageRouter {
    MessageRouter {
        config,
        routes: Vec::new(),
    }
}

/// Adds a route, returning false if the table is full.
pub fn add_route(router: &mut MessageRouter, entry: RouteEntry) -> bool {
    if router.routes.len() >= router.config.max_routes {
        return false;
    }
    router.routes.push(entry);
    true
}

/// Removes all routes for a given message type.
pub fn remove_routes_for(router: &mut MessageRouter, message_type: &str) -> usize {
    let before = router.routes.len();
    router.routes.retain(|r| r.message_type != message_type);
    before.saturating_sub(router.routes.len())
}

/// Routes a message, returning the handler ID with the highest priority.
pub fn route_message<'a>(router: &'a MessageRouter, msg: &RoutableMessage) -> Option<&'a str> {
    let mut best: Option<&RouteEntry> = None;
    for entry in router
        .routes
        .iter()
        .filter(|e| e.enabled && e.message_type == msg.message_type)
    {
        match best {
            None => best = Some(entry),
            Some(b) if entry.priority > b.priority => best = Some(entry),
            _ => {}
        }
    }
    best.map(|e| e.handler_id.as_str())
        .or(router.config.default_handler.as_deref())
}

/// Enables or disables all routes for a given handler.
pub fn set_handler_enabled(router: &mut MessageRouter, handler_id: &str, enabled: bool) {
    for r in router
        .routes
        .iter_mut()
        .filter(|r| r.handler_id == handler_id)
    {
        r.enabled = enabled;
    }
}

/// Returns all handler IDs registered in the router.
pub fn all_handler_ids(router: &MessageRouter) -> Vec<String> {
    let mut ids: Vec<String> = router.routes.iter().map(|r| r.handler_id.clone()).collect();
    ids.sort();
    ids.dedup();
    ids
}

impl MessageRouter {
    /// Creates a new router with default config.
    pub fn new(config: MessageRouterConfig) -> Self {
        new_router(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_router() -> MessageRouter {
        new_router(MessageRouterConfig::default())
    }

    fn entry(msg_type: &str, handler: &str, priority: i32) -> RouteEntry {
        RouteEntry {
            message_type: msg_type.into(),
            handler_id: handler.into(),
            priority,
            enabled: true,
        }
    }

    fn msg(msg_type: &str) -> RoutableMessage {
        RoutableMessage {
            message_type: msg_type.into(),
            payload: "{}".into(),
            source: "test".into(),
        }
    }

    #[test]
    fn test_add_and_route() {
        let mut r = make_router();
        add_route(&mut r, entry("click", "ui_handler", 10));
        assert_eq!(route_message(&r, &msg("click")), Some("ui_handler"));
    }

    #[test]
    fn test_higher_priority_wins() {
        let mut r = make_router();
        add_route(&mut r, entry("ev", "low_handler", 1));
        add_route(&mut r, entry("ev", "high_handler", 10));
        assert_eq!(route_message(&r, &msg("ev")), Some("high_handler"));
    }

    #[test]
    fn test_unknown_type_uses_default_handler() {
        let mut r = new_router(MessageRouterConfig {
            default_handler: Some("fallback".into()),
            max_routes: 64,
        });
        add_route(&mut r, entry("known", "h", 1));
        assert_eq!(route_message(&r, &msg("unknown")), Some("fallback"));
    }

    #[test]
    fn test_unknown_type_without_default_returns_none() {
        let r = make_router();
        assert!(route_message(&r, &msg("notype")).is_none());
    }

    #[test]
    fn test_remove_routes_for() {
        let mut r = make_router();
        add_route(&mut r, entry("t1", "h1", 1));
        add_route(&mut r, entry("t1", "h2", 2));
        add_route(&mut r, entry("t2", "h3", 1));
        let removed = remove_routes_for(&mut r, "t1");
        assert_eq!(removed, 2);
        assert!(route_message(&r, &msg("t1")).is_none());
    }

    #[test]
    fn test_disabled_route_skipped() {
        let mut r = make_router();
        add_route(&mut r, entry("ping", "handler_a", 5));
        set_handler_enabled(&mut r, "handler_a", false);
        assert!(route_message(&r, &msg("ping")).is_none());
    }

    #[test]
    fn test_all_handler_ids_unique() {
        let mut r = make_router();
        add_route(&mut r, entry("a", "h1", 1));
        add_route(&mut r, entry("b", "h1", 1));
        add_route(&mut r, entry("c", "h2", 1));
        let ids = all_handler_ids(&r);
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn test_capacity_limit() {
        let mut r = new_router(MessageRouterConfig {
            default_handler: None,
            max_routes: 2,
        });
        add_route(&mut r, entry("a", "h", 1));
        add_route(&mut r, entry("b", "h", 1));
        let ok = add_route(&mut r, entry("c", "h", 1));
        assert!(!ok);
    }

    #[test]
    fn test_enable_handler_after_disable() {
        let mut r = make_router();
        add_route(&mut r, entry("e", "he", 5));
        set_handler_enabled(&mut r, "he", false);
        set_handler_enabled(&mut r, "he", true);
        assert_eq!(route_message(&r, &msg("e")), Some("he"));
    }
}
