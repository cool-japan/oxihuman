//! Typed signal/slot publish-subscribe bus for decoupled component communication.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SignalConfig {
    pub max_subscribers: usize,
    pub max_queue_depth: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SignalMessage {
    pub topic: String,
    pub payload: Vec<u8>,
    pub sender_id: u32,
    pub timestamp_ms: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Subscription {
    pub id: u64,
    pub topic: String,
    pub subscriber_id: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SignalBus {
    pub config: SignalConfig,
    pub subscriptions: Vec<Subscription>,
    pub queue: Vec<SignalMessage>,
    pub next_sub_id: u64,
}

#[allow(dead_code)]
pub fn default_signal_config() -> SignalConfig {
    SignalConfig {
        max_subscribers: 256,
        max_queue_depth: 1024,
    }
}

#[allow(dead_code)]
pub fn new_signal_bus(cfg: SignalConfig) -> SignalBus {
    SignalBus {
        config: cfg,
        subscriptions: Vec::new(),
        queue: Vec::new(),
        next_sub_id: 1,
    }
}

/// Subscribe to a topic. Returns the subscription id.
#[allow(dead_code)]
pub fn subscribe(bus: &mut SignalBus, topic: &str, subscriber_id: u32) -> u64 {
    let id = bus.next_sub_id;
    bus.next_sub_id += 1;
    bus.subscriptions.push(Subscription {
        id,
        topic: topic.to_string(),
        subscriber_id,
    });
    id
}

/// Unsubscribe by subscription id. Returns true if found and removed.
#[allow(dead_code)]
pub fn unsubscribe(bus: &mut SignalBus, sub_id: u64) -> bool {
    if let Some(pos) = bus.subscriptions.iter().position(|s| s.id == sub_id) {
        bus.subscriptions.remove(pos);
        true
    } else {
        false
    }
}

/// Publish a message to the bus queue.
#[allow(dead_code)]
pub fn publish(bus: &mut SignalBus, topic: &str, payload: Vec<u8>, sender_id: u32, ts: u64) {
    if bus.queue.len() >= bus.config.max_queue_depth {
        return;
    }
    bus.queue.push(SignalMessage {
        topic: topic.to_string(),
        payload,
        sender_id,
        timestamp_ms: ts,
    });
}

/// Drain all messages for a specific topic from the queue.
#[allow(dead_code)]
pub fn drain_messages(bus: &mut SignalBus, topic: &str) -> Vec<SignalMessage> {
    let mut drained = Vec::new();
    let mut remaining = Vec::new();
    for msg in bus.queue.drain(..) {
        if msg.topic == topic {
            drained.push(msg);
        } else {
            remaining.push(msg);
        }
    }
    bus.queue = remaining;
    drained
}

#[allow(dead_code)]
pub fn subscriber_count(bus: &SignalBus, topic: &str) -> usize {
    bus.subscriptions.iter().filter(|s| s.topic == topic).count()
}

#[allow(dead_code)]
pub fn queue_depth(bus: &SignalBus) -> usize {
    bus.queue.len()
}

#[allow(dead_code)]
pub fn signal_bus_to_json(bus: &SignalBus) -> String {
    format!(
        r#"{{"subscription_count":{},"queue_depth":{}}}"#,
        bus.subscriptions.len(),
        bus.queue.len()
    )
}

#[allow(dead_code)]
pub fn clear_queue(bus: &mut SignalBus) {
    bus.queue.clear();
}

#[allow(dead_code)]
pub fn subscription_count(bus: &SignalBus) -> usize {
    bus.subscriptions.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_signal_config() {
        let cfg = default_signal_config();
        assert_eq!(cfg.max_subscribers, 256);
        assert_eq!(cfg.max_queue_depth, 1024);
    }

    #[test]
    fn test_subscribe_and_count() {
        let cfg = default_signal_config();
        let mut bus = new_signal_bus(cfg);
        subscribe(&mut bus, "morph.changed", 1);
        subscribe(&mut bus, "morph.changed", 2);
        assert_eq!(subscriber_count(&bus, "morph.changed"), 2);
        assert_eq!(subscriber_count(&bus, "other"), 0);
    }

    #[test]
    fn test_unsubscribe() {
        let cfg = default_signal_config();
        let mut bus = new_signal_bus(cfg);
        let sid = subscribe(&mut bus, "topic", 42);
        assert_eq!(subscription_count(&bus), 1);
        let removed = unsubscribe(&mut bus, sid);
        assert!(removed);
        assert_eq!(subscription_count(&bus), 0);
        let not_removed = unsubscribe(&mut bus, 9999);
        assert!(!not_removed);
    }

    #[test]
    fn test_publish_and_queue_depth() {
        let cfg = default_signal_config();
        let mut bus = new_signal_bus(cfg);
        publish(&mut bus, "topic", vec![1, 2, 3], 1, 100);
        publish(&mut bus, "topic", vec![4, 5], 1, 200);
        assert_eq!(queue_depth(&bus), 2);
    }

    #[test]
    fn test_drain_messages() {
        let cfg = default_signal_config();
        let mut bus = new_signal_bus(cfg);
        publish(&mut bus, "a", vec![1], 1, 10);
        publish(&mut bus, "b", vec![2], 1, 20);
        publish(&mut bus, "a", vec![3], 1, 30);
        let msgs = drain_messages(&mut bus, "a");
        assert_eq!(msgs.len(), 2);
        assert_eq!(queue_depth(&bus), 1);
    }

    #[test]
    fn test_clear_queue() {
        let cfg = default_signal_config();
        let mut bus = new_signal_bus(cfg);
        publish(&mut bus, "x", vec![], 0, 0);
        publish(&mut bus, "y", vec![], 0, 0);
        clear_queue(&mut bus);
        assert_eq!(queue_depth(&bus), 0);
    }

    #[test]
    fn test_max_queue_depth_enforced() {
        let cfg = SignalConfig { max_subscribers: 10, max_queue_depth: 2 };
        let mut bus = new_signal_bus(cfg);
        publish(&mut bus, "t", vec![], 0, 0);
        publish(&mut bus, "t", vec![], 0, 0);
        publish(&mut bus, "t", vec![], 0, 0); // should be dropped
        assert_eq!(queue_depth(&bus), 2);
    }

    #[test]
    fn test_signal_bus_to_json() {
        let cfg = default_signal_config();
        let mut bus = new_signal_bus(cfg);
        subscribe(&mut bus, "topic", 1);
        let j = signal_bus_to_json(&bus);
        assert!(j.contains("subscription_count"));
        assert!(j.contains("queue_depth"));
    }
}
