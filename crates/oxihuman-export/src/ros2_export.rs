// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! ROS2 message stub export — serialises mesh data into ROS2-compatible message stubs.

/// Supported ROS2 QoS reliability policies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ros2Reliability {
    Reliable,
    BestEffort,
}

/// A single ROS2 message field stub.
#[derive(Debug, Clone)]
pub struct Ros2Field {
    pub name: String,
    pub type_str: String,
    pub value: String,
}

/// A ROS2 message stub.
#[derive(Debug, Clone)]
pub struct Ros2Message {
    pub topic: String,
    pub msg_type: String,
    pub reliability: Ros2Reliability,
    pub fields: Vec<Ros2Field>,
}

/// A ROS2 export session containing multiple messages.
#[derive(Debug, Default)]
pub struct Ros2Export {
    pub messages: Vec<Ros2Message>,
}

/// Create a new ROS2 export session.
pub fn new_ros2_export() -> Ros2Export {
    Ros2Export::default()
}

/// Add a new message stub.
pub fn add_ros2_message(
    export: &mut Ros2Export,
    topic: &str,
    msg_type: &str,
    reliability: Ros2Reliability,
) {
    export.messages.push(Ros2Message {
        topic: topic.to_owned(),
        msg_type: msg_type.to_owned(),
        reliability,
        fields: Vec::new(),
    });
}

/// Add a field to the last message.
pub fn add_ros2_field(export: &mut Ros2Export, name: &str, type_str: &str, value: &str) {
    if let Some(msg) = export.messages.last_mut() {
        msg.fields.push(Ros2Field {
            name: name.to_owned(),
            type_str: type_str.to_owned(),
            value: value.to_owned(),
        });
    }
}

/// Number of messages in the export.
pub fn ros2_message_count(export: &Ros2Export) -> usize {
    export.messages.len()
}

/// Find a message by topic name.
pub fn find_ros2_message<'a>(export: &'a Ros2Export, topic: &str) -> Option<&'a Ros2Message> {
    export.messages.iter().find(|m| m.topic == topic)
}

/// Total number of fields across all messages.
pub fn total_ros2_fields(export: &Ros2Export) -> usize {
    export.messages.iter().map(|m| m.fields.len()).sum()
}

/// Render a message stub as a YAML-like string.
pub fn render_ros2_message(msg: &Ros2Message) -> String {
    let fields: Vec<String> = msg
        .fields
        .iter()
        .map(|f| format!("  {}: {}", f.name, f.value))
        .collect();
    format!(
        "topic: {}\ntype: {}\n{}",
        msg.topic,
        msg.msg_type,
        fields.join("\n")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_is_empty() {
        /* fresh export has no messages */
        let e = new_ros2_export();
        assert_eq!(ros2_message_count(&e), 0);
    }

    #[test]
    fn add_message_increments_count() {
        /* adding a message increases count by 1 */
        let mut e = new_ros2_export();
        add_ros2_message(
            &mut e,
            "/scan",
            "sensor_msgs/LaserScan",
            Ros2Reliability::Reliable,
        );
        assert_eq!(ros2_message_count(&e), 1);
    }

    #[test]
    fn add_field_increases_field_count() {
        /* adding a field to a message increases total count */
        let mut e = new_ros2_export();
        add_ros2_message(
            &mut e,
            "/odom",
            "nav_msgs/Odometry",
            Ros2Reliability::BestEffort,
        );
        add_ros2_field(&mut e, "x", "float64", "0.0");
        assert_eq!(total_ros2_fields(&e), 1);
    }

    #[test]
    fn find_message_by_topic() {
        /* find_ros2_message returns the correct message */
        let mut e = new_ros2_export();
        add_ros2_message(
            &mut e,
            "/pose",
            "geometry_msgs/Pose",
            Ros2Reliability::Reliable,
        );
        let m = find_ros2_message(&e, "/pose").expect("should succeed");
        assert_eq!(m.topic, "/pose");
    }

    #[test]
    fn find_message_missing_returns_none() {
        /* querying a non-existent topic yields None */
        let e = new_ros2_export();
        assert!(find_ros2_message(&e, "/nonexistent").is_none());
    }

    #[test]
    fn reliability_best_effort_stored() {
        /* BestEffort reliability should be stored */
        let mut e = new_ros2_export();
        add_ros2_message(
            &mut e,
            "/img",
            "sensor_msgs/Image",
            Ros2Reliability::BestEffort,
        );
        assert_eq!(e.messages[0].reliability, Ros2Reliability::BestEffort);
    }

    #[test]
    fn render_message_contains_topic() {
        /* rendered string should contain the topic */
        let msg = Ros2Message {
            topic: "/test".into(),
            msg_type: "std_msgs/String".into(),
            reliability: Ros2Reliability::Reliable,
            fields: vec![],
        };
        let s = render_ros2_message(&msg);
        assert!(s.contains("/test"));
    }

    #[test]
    fn total_fields_zero_when_no_fields() {
        /* total fields is zero before any fields are added */
        let mut e = new_ros2_export();
        add_ros2_message(&mut e, "/a", "std_msgs/Bool", Ros2Reliability::Reliable);
        assert_eq!(total_ros2_fields(&e), 0);
    }

    #[test]
    fn multiple_messages_counted() {
        /* adding two messages gives count 2 */
        let mut e = new_ros2_export();
        add_ros2_message(&mut e, "/a", "t", Ros2Reliability::Reliable);
        add_ros2_message(&mut e, "/b", "t", Ros2Reliability::Reliable);
        assert_eq!(ros2_message_count(&e), 2);
    }
}
