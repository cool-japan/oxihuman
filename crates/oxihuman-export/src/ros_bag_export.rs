// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! ROS bag v2 file header export stub.

pub const ROS_BAG_MAGIC: &[u8; 13] = b"#ROSBAG V2.0\n";

/// ROS bag header record fields.
#[allow(dead_code)]
pub struct RosBagHeader {
    pub index_pos: u64,
    pub conn_count: u32,
    pub chunk_count: u32,
    pub compression: String,
}

impl Default for RosBagHeader {
    fn default() -> Self {
        Self {
            index_pos: 0,
            conn_count: 0,
            chunk_count: 0,
            compression: String::from("none"),
        }
    }
}

/// A ROS bag connection record.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RosBagConnection {
    pub conn_id: u32,
    pub topic: String,
    pub msg_type: String,
}

/// A ROS bag stub (in-memory).
#[allow(dead_code)]
pub struct RosBag {
    pub header: RosBagHeader,
    pub connections: Vec<RosBagConnection>,
    pub message_count: u64,
}

impl RosBag {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            header: RosBagHeader::default(),
            connections: Vec::new(),
            message_count: 0,
        }
    }
}

impl Default for RosBag {
    fn default() -> Self {
        Self::new()
    }
}

/// Add a connection to the bag.
#[allow(dead_code)]
pub fn add_connection(bag: &mut RosBag, topic: &str, msg_type: &str) -> u32 {
    let id = bag.connections.len() as u32;
    bag.connections.push(RosBagConnection {
        conn_id: id,
        topic: topic.to_string(),
        msg_type: msg_type.to_string(),
    });
    bag.header.conn_count = bag.connections.len() as u32;
    id
}

/// Build the ROS bag file header bytes (magic + stub header record).
#[allow(dead_code)]
pub fn build_ros_bag_header_bytes(bag: &RosBag) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(ROS_BAG_MAGIC);
    let header_fields = format!(
        "index_pos={},conn_count={},chunk_count={},compression={}",
        bag.header.index_pos, bag.header.conn_count, bag.header.chunk_count, bag.header.compression
    );
    let field_bytes = header_fields.as_bytes();
    out.extend_from_slice(&(field_bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(field_bytes);
    out
}

/// Number of connections in the bag.
#[allow(dead_code)]
pub fn connection_count(bag: &RosBag) -> usize {
    bag.connections.len()
}

/// Find connection by topic.
#[allow(dead_code)]
pub fn find_connection_by_topic<'a>(bag: &'a RosBag, topic: &str) -> Option<&'a RosBagConnection> {
    bag.connections.iter().find(|c| c.topic == topic)
}

/// Connection topics list.
#[allow(dead_code)]
pub fn connection_topics(bag: &RosBag) -> Vec<&str> {
    bag.connections.iter().map(|c| c.topic.as_str()).collect()
}

/// Increment message counter.
#[allow(dead_code)]
pub fn record_message(bag: &mut RosBag) {
    bag.message_count += 1;
}

/// Check if a topic exists.
#[allow(dead_code)]
pub fn has_topic(bag: &RosBag, topic: &str) -> bool {
    bag.connections.iter().any(|c| c.topic == topic)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_starts_with_magic() {
        let bag = RosBag::new();
        let bytes = build_ros_bag_header_bytes(&bag);
        assert_eq!(&bytes[0..13], ROS_BAG_MAGIC);
    }

    #[test]
    fn add_connection_increments_count() {
        let mut bag = RosBag::new();
        add_connection(&mut bag, "/pose", "geometry_msgs/Pose");
        assert_eq!(connection_count(&bag), 1);
    }

    #[test]
    fn find_connection_by_topic_some() {
        let mut bag = RosBag::new();
        add_connection(&mut bag, "/pose", "geometry_msgs/Pose");
        let c = find_connection_by_topic(&bag, "/pose");
        assert!(c.is_some());
    }

    #[test]
    fn find_connection_by_topic_none() {
        let bag = RosBag::new();
        assert!(find_connection_by_topic(&bag, "/missing").is_none());
    }

    #[test]
    fn has_topic_true() {
        let mut bag = RosBag::new();
        add_connection(&mut bag, "/scan", "sensor_msgs/LaserScan");
        assert!(has_topic(&bag, "/scan"));
    }

    #[test]
    fn has_topic_false() {
        let bag = RosBag::new();
        assert!(!has_topic(&bag, "/nonexistent"));
    }

    #[test]
    fn record_message_increments() {
        let mut bag = RosBag::new();
        record_message(&mut bag);
        record_message(&mut bag);
        assert_eq!(bag.message_count, 2);
    }

    #[test]
    fn connection_topics_list() {
        let mut bag = RosBag::new();
        add_connection(&mut bag, "/a", "std_msgs/String");
        add_connection(&mut bag, "/b", "std_msgs/Int32");
        let topics = connection_topics(&bag);
        assert!(topics.contains(&"/a"));
        assert!(topics.contains(&"/b"));
    }

    #[test]
    fn header_bytes_nonempty() {
        let bag = RosBag::new();
        let bytes = build_ros_bag_header_bytes(&bag);
        assert!(bytes.len() > 13);
    }

    #[test]
    fn default_header_compression_none() {
        let h = RosBagHeader::default();
        assert_eq!(h.compression, "none");
    }
}
