#![allow(dead_code)]

/// Types of physics events.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum PhysicsEvent {
    CollisionStart { body_a: u32, body_b: u32 },
    CollisionEnd { body_a: u32, body_b: u32 },
    BodySleep { body_id: u32 },
    BodyWake { body_id: u32 },
    JointBreak { joint_id: u32 },
}

/// Event bus for physics events.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PhysicsEventBus {
    events: Vec<PhysicsEvent>,
    subscriber_count: usize,
}

/// Creates a new physics event bus.
#[allow(dead_code)]
pub fn new_physics_event_bus() -> PhysicsEventBus {
    PhysicsEventBus {
        events: Vec::new(),
        subscriber_count: 0,
    }
}

/// Emits a physics event.
#[allow(dead_code)]
pub fn emit_physics_event(bus: &mut PhysicsEventBus, event: PhysicsEvent) {
    bus.events.push(event);
}

/// Subscribes to the event bus (increments subscriber count).
#[allow(dead_code)]
pub fn subscribe_physics(bus: &mut PhysicsEventBus) -> usize {
    bus.subscriber_count += 1;
    bus.subscriber_count
}

/// Returns the number of pending events.
#[allow(dead_code)]
pub fn event_count_phys(bus: &PhysicsEventBus) -> usize {
    bus.events.len()
}

/// Clears all events.
#[allow(dead_code)]
pub fn clear_physics_events(bus: &mut PhysicsEventBus) {
    bus.events.clear();
}

/// Drains all events into a Vec.
#[allow(dead_code)]
pub fn drain_physics_events(bus: &mut PhysicsEventBus) -> Vec<PhysicsEvent> {
    bus.events.drain(..).collect()
}

/// Returns true if there are subscribers.
#[allow(dead_code)]
pub fn has_subscribers(bus: &PhysicsEventBus) -> bool {
    bus.subscriber_count > 0
}

/// Returns the number of subscribers.
#[allow(dead_code)]
pub fn subscriber_count(bus: &PhysicsEventBus) -> usize {
    bus.subscriber_count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_bus() {
        let bus = new_physics_event_bus();
        assert_eq!(event_count_phys(&bus), 0);
    }

    #[test]
    fn test_emit() {
        let mut bus = new_physics_event_bus();
        emit_physics_event(&mut bus, PhysicsEvent::BodySleep { body_id: 1 });
        assert_eq!(event_count_phys(&bus), 1);
    }

    #[test]
    fn test_subscribe() {
        let mut bus = new_physics_event_bus();
        subscribe_physics(&mut bus);
        assert!(has_subscribers(&bus));
    }

    #[test]
    fn test_clear() {
        let mut bus = new_physics_event_bus();
        emit_physics_event(&mut bus, PhysicsEvent::BodyWake { body_id: 1 });
        clear_physics_events(&mut bus);
        assert_eq!(event_count_phys(&bus), 0);
    }

    #[test]
    fn test_drain() {
        let mut bus = new_physics_event_bus();
        emit_physics_event(&mut bus, PhysicsEvent::JointBreak { joint_id: 5 });
        let events = drain_physics_events(&mut bus);
        assert_eq!(events.len(), 1);
        assert_eq!(event_count_phys(&bus), 0);
    }

    #[test]
    fn test_collision_start() {
        let mut bus = new_physics_event_bus();
        emit_physics_event(&mut bus, PhysicsEvent::CollisionStart { body_a: 1, body_b: 2 });
        let events = drain_physics_events(&mut bus);
        assert_eq!(events[0], PhysicsEvent::CollisionStart { body_a: 1, body_b: 2 });
    }

    #[test]
    fn test_subscriber_count() {
        let mut bus = new_physics_event_bus();
        subscribe_physics(&mut bus);
        subscribe_physics(&mut bus);
        assert_eq!(subscriber_count(&bus), 2);
    }

    #[test]
    fn test_no_subscribers() {
        let bus = new_physics_event_bus();
        assert!(!has_subscribers(&bus));
    }

    #[test]
    fn test_multiple_events() {
        let mut bus = new_physics_event_bus();
        emit_physics_event(&mut bus, PhysicsEvent::BodySleep { body_id: 1 });
        emit_physics_event(&mut bus, PhysicsEvent::BodyWake { body_id: 2 });
        assert_eq!(event_count_phys(&bus), 2);
    }

    #[test]
    fn test_collision_end() {
        let mut bus = new_physics_event_bus();
        emit_physics_event(&mut bus, PhysicsEvent::CollisionEnd { body_a: 3, body_b: 4 });
        assert_eq!(event_count_phys(&bus), 1);
    }
}
