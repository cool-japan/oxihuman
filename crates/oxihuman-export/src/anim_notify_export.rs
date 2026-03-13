// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Animation notification event export.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnimNotify {
    pub name: String,
    pub time: f32,
    pub payload: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnimNotifyTrack {
    pub track_name: String,
    pub notifies: Vec<AnimNotify>,
}

#[allow(dead_code)]
pub fn new_notify_track(name: &str) -> AnimNotifyTrack {
    AnimNotifyTrack {
        track_name: name.to_string(),
        notifies: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_notify(track: &mut AnimNotifyTrack, name: &str, time: f32, payload: &str) {
    track.notifies.push(AnimNotify {
        name: name.to_string(),
        time,
        payload: payload.to_string(),
    });
}

#[allow(dead_code)]
pub fn notify_count(track: &AnimNotifyTrack) -> usize {
    track.notifies.len()
}

#[allow(dead_code)]
pub fn find_notify<'a>(track: &'a AnimNotifyTrack, name: &str) -> Option<&'a AnimNotify> {
    track.notifies.iter().find(|n| n.name == name)
}

#[allow(dead_code)]
pub fn notifies_in_range(track: &AnimNotifyTrack, start: f32, end: f32) -> Vec<&AnimNotify> {
    track
        .notifies
        .iter()
        .filter(|n| n.time >= start && n.time <= end)
        .collect()
}

#[allow(dead_code)]
pub fn track_duration(track: &AnimNotifyTrack) -> f32 {
    track
        .notifies
        .iter()
        .map(|n| n.time)
        .fold(0.0_f32, f32::max)
}

#[allow(dead_code)]
pub fn sort_notifies(track: &mut AnimNotifyTrack) {
    track.notifies.sort_by(|a, b| {
        a.time
            .partial_cmp(&b.time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

#[allow(dead_code)]
pub fn notify_track_to_json(track: &AnimNotifyTrack) -> String {
    format!(
        "{{\"track\":\"{}\",\"notify_count\":{}}}",
        track.track_name,
        track.notifies.len()
    )
}

#[allow(dead_code)]
pub fn clear_notifies(track: &mut AnimNotifyTrack) {
    track.notifies.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_track_empty() {
        let t = new_notify_track("events");
        assert_eq!(notify_count(&t), 0);
    }

    #[test]
    fn test_add_notify() {
        let mut t = new_notify_track("events");
        add_notify(&mut t, "hit", 0.5, "{}");
        assert_eq!(notify_count(&t), 1);
    }

    #[test]
    fn test_find_notify() {
        let mut t = new_notify_track("events");
        add_notify(&mut t, "footstep", 0.3, "left");
        assert!(find_notify(&t, "footstep").is_some());
    }

    #[test]
    fn test_find_missing() {
        let t = new_notify_track("events");
        assert!(find_notify(&t, "missing").is_none());
    }

    #[test]
    fn test_notifies_in_range() {
        let mut t = new_notify_track("events");
        add_notify(&mut t, "a", 0.1, "");
        add_notify(&mut t, "b", 0.5, "");
        add_notify(&mut t, "c", 0.9, "");
        let r = notifies_in_range(&t, 0.0, 0.6);
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn test_track_duration() {
        let mut t = new_notify_track("events");
        add_notify(&mut t, "end", 2.0, "");
        assert!((track_duration(&t) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_sort_notifies() {
        let mut t = new_notify_track("events");
        add_notify(&mut t, "b", 0.5, "");
        add_notify(&mut t, "a", 0.1, "");
        sort_notifies(&mut t);
        assert!((t.notifies[0].time - 0.1).abs() < 1e-5);
    }

    #[test]
    fn test_json_output() {
        let t = new_notify_track("events");
        let j = notify_track_to_json(&t);
        assert!(j.contains("events"));
    }

    #[test]
    fn test_clear_notifies() {
        let mut t = new_notify_track("events");
        add_notify(&mut t, "x", 0.0, "");
        clear_notifies(&mut t);
        assert_eq!(notify_count(&t), 0);
    }

    #[test]
    fn test_payload_stored() {
        let mut t = new_notify_track("events");
        add_notify(&mut t, "pickup", 1.0, "gold");
        let n = find_notify(&t, "pickup").expect("should succeed");
        assert_eq!(n.payload, "gold".to_string());
    }
}
