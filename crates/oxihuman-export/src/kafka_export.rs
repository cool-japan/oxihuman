// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Kafka message export stub — packages mesh data as Kafka producer record stubs.

/// A single Kafka producer record stub.
#[derive(Debug, Clone)]
pub struct KafkaRecord {
    pub topic: String,
    pub partition: Option<i32>,
    pub key: Option<String>,
    pub value: Vec<u8>,
    pub timestamp_ms: Option<i64>,
}

/// A Kafka export session.
#[derive(Debug, Default)]
pub struct KafkaExport {
    pub records: Vec<KafkaRecord>,
    pub bootstrap_servers: String,
}

/// Create a new Kafka export session.
pub fn new_kafka_export(bootstrap_servers: &str) -> KafkaExport {
    KafkaExport {
        records: Vec::new(),
        bootstrap_servers: bootstrap_servers.to_owned(),
    }
}

/// Add a record with a text value.
pub fn add_kafka_text(export: &mut KafkaExport, topic: &str, key: Option<&str>, value: &str) {
    export.records.push(KafkaRecord {
        topic: topic.to_owned(),
        partition: None,
        key: key.map(str::to_owned),
        value: value.as_bytes().to_vec(),
        timestamp_ms: None,
    });
}

/// Add a record with a binary value and explicit partition.
pub fn add_kafka_binary(export: &mut KafkaExport, topic: &str, partition: i32, data: Vec<u8>) {
    export.records.push(KafkaRecord {
        topic: topic.to_owned(),
        partition: Some(partition),
        key: None,
        value: data,
        timestamp_ms: None,
    });
}

/// Number of records.
pub fn kafka_record_count(export: &KafkaExport) -> usize {
    export.records.len()
}

/// Find the first record for a given topic.
pub fn find_kafka_record<'a>(export: &'a KafkaExport, topic: &str) -> Option<&'a KafkaRecord> {
    export.records.iter().find(|r| r.topic == topic)
}

/// Total value bytes across all records.
pub fn total_kafka_bytes(export: &KafkaExport) -> usize {
    export.records.iter().map(|r| r.value.len()).sum()
}

/// Count records that have an explicit partition.
pub fn partitioned_record_count(export: &KafkaExport) -> usize {
    export
        .records
        .iter()
        .filter(|r| r.partition.is_some())
        .count()
}

/// Serialize export metadata to JSON-style string.
pub fn kafka_export_to_json(export: &KafkaExport) -> String {
    format!(
        r#"{{"bootstrap":"{}","record_count":{},"total_bytes":{}}}"#,
        export.bootstrap_servers,
        kafka_record_count(export),
        total_kafka_bytes(export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_has_no_records() {
        /* fresh export has no records */
        let e = new_kafka_export("localhost:9092");
        assert_eq!(kafka_record_count(&e), 0);
    }

    #[test]
    fn add_text_increments_count() {
        /* adding a text record increments count */
        let mut e = new_kafka_export("localhost:9092");
        add_kafka_text(&mut e, "mesh", None, "data");
        assert_eq!(kafka_record_count(&e), 1);
    }

    #[test]
    fn find_record_by_topic() {
        /* find returns the correct record */
        let mut e = new_kafka_export("localhost:9092");
        add_kafka_text(&mut e, "poses", Some("k"), "val");
        assert!(find_kafka_record(&e, "poses").is_some());
    }

    #[test]
    fn find_missing_topic_returns_none() {
        /* missing topic returns None */
        let e = new_kafka_export("localhost:9092");
        assert!(find_kafka_record(&e, "nope").is_none());
    }

    #[test]
    fn total_bytes_counted() {
        /* value bytes should be counted */
        let mut e = new_kafka_export("localhost:9092");
        add_kafka_text(&mut e, "t", None, "hello");
        assert_eq!(total_kafka_bytes(&e), 5);
    }

    #[test]
    fn partitioned_record_count_correct() {
        /* only records with partitions are counted */
        let mut e = new_kafka_export("localhost:9092");
        add_kafka_binary(&mut e, "t", 0, vec![1, 2, 3]);
        add_kafka_text(&mut e, "t", None, "x");
        assert_eq!(partitioned_record_count(&e), 1);
    }

    #[test]
    fn json_contains_bootstrap() {
        /* JSON includes bootstrap servers */
        let e = new_kafka_export("kafka.example.com:9092");
        assert!(kafka_export_to_json(&e).contains("kafka.example.com"));
    }

    #[test]
    fn key_stored_when_provided() {
        /* key should be stored when given */
        let mut e = new_kafka_export("localhost:9092");
        add_kafka_text(&mut e, "t", Some("mykey"), "v");
        assert_eq!(e.records[0].key.as_deref(), Some("mykey"));
    }

    #[test]
    fn binary_record_partition_stored() {
        /* partition should be stored for binary records */
        let mut e = new_kafka_export("localhost:9092");
        add_kafka_binary(&mut e, "t", 3, vec![]);
        assert_eq!(e.records[0].partition, Some(3));
    }
}
