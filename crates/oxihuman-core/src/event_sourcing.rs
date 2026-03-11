// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct EventStore {
    pub events: Vec<(String, u64, String)>,
}

pub fn new_event_store() -> EventStore {
    EventStore { events: Vec::new() }
}

pub fn es_append(s: &mut EventStore, agg_id: &str, version: u64, event_json: &str) {
    s.events
        .push((agg_id.to_string(), version, event_json.to_string()));
}

pub fn es_events_for<'a>(s: &'a EventStore, agg_id: &str) -> Vec<(u64, &'a str)> {
    s.events
        .iter()
        .filter(|(id, _, _)| id == agg_id)
        .map(|(_, v, j)| (*v, j.as_str()))
        .collect()
}

pub fn es_latest_version(s: &EventStore, agg_id: &str) -> u64 {
    s.events
        .iter()
        .filter(|(id, _, _)| id == agg_id)
        .map(|(_, v, _)| *v)
        .max()
        .unwrap_or(0)
}

pub fn es_total_events(s: &EventStore) -> usize {
    s.events.len()
}

pub fn es_replay(s: &EventStore, agg_id: &str) -> Vec<String> {
    let mut evs: Vec<(u64, String)> = s
        .events
        .iter()
        .filter(|(id, _, _)| id == agg_id)
        .map(|(_, v, j)| (*v, j.clone()))
        .collect();
    evs.sort_by_key(|(v, _)| *v);
    evs.into_iter().map(|(_, j)| j).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_and_total() {
        /* append events and count total */
        let mut s = new_event_store();
        es_append(&mut s, "agg1", 1, "{}");
        es_append(&mut s, "agg1", 2, "{}");
        assert_eq!(es_total_events(&s), 2);
    }

    #[test]
    fn test_events_for() {
        /* filter events by aggregate */
        let mut s = new_event_store();
        es_append(&mut s, "a", 1, "ev1");
        es_append(&mut s, "b", 1, "ev2");
        let evs = es_events_for(&s, "a");
        assert_eq!(evs.len(), 1);
        assert_eq!(evs[0].1, "ev1");
    }

    #[test]
    fn test_latest_version() {
        /* latest version is max version for aggregate */
        let mut s = new_event_store();
        es_append(&mut s, "agg", 1, "");
        es_append(&mut s, "agg", 3, "");
        es_append(&mut s, "agg", 2, "");
        assert_eq!(es_latest_version(&s, "agg"), 3);
    }

    #[test]
    fn test_latest_version_missing() {
        /* missing aggregate returns 0 */
        let s = new_event_store();
        assert_eq!(es_latest_version(&s, "missing"), 0);
    }

    #[test]
    fn test_replay_sorted() {
        /* replay returns events in version order */
        let mut s = new_event_store();
        es_append(&mut s, "agg", 2, "second");
        es_append(&mut s, "agg", 1, "first");
        let replayed = es_replay(&s, "agg");
        assert_eq!(replayed[0], "first");
        assert_eq!(replayed[1], "second");
    }

    #[test]
    fn test_empty_store() {
        /* empty store returns zero events */
        let s = new_event_store();
        assert_eq!(es_total_events(&s), 0);
    }

    #[test]
    fn test_multiple_aggregates() {
        /* multiple aggregates stored independently */
        let mut s = new_event_store();
        es_append(&mut s, "a", 1, "a1");
        es_append(&mut s, "b", 1, "b1");
        es_append(&mut s, "a", 2, "a2");
        assert_eq!(es_events_for(&s, "a").len(), 2);
        assert_eq!(es_events_for(&s, "b").len(), 1);
    }
}
