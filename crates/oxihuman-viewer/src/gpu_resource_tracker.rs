#![allow(dead_code)]
//! GPU resource tracker: tracks allocated GPU resources and their memory usage.

use std::collections::HashMap;

/// A tracked GPU resource.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct GpuResource {
    name: String,
    resource_type: String,
    memory_bytes: u64,
}

/// Tracks GPU resources.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GpuResourceTracker {
    resources: Vec<GpuResource>,
}

/// Create a new tracker.
#[allow(dead_code)]
pub fn new_gpu_tracker() -> GpuResourceTracker {
    GpuResourceTracker {
        resources: Vec::new(),
    }
}

/// Track a new resource.
#[allow(dead_code)]
pub fn track_resource(tracker: &mut GpuResourceTracker, name: &str, resource_type: &str, memory_bytes: u64) {
    tracker.resources.push(GpuResource {
        name: name.to_string(),
        resource_type: resource_type.to_string(),
        memory_bytes,
    });
}

/// Remove a resource by name.
#[allow(dead_code)]
pub fn untrack_resource(tracker: &mut GpuResourceTracker, name: &str) {
    tracker.resources.retain(|r| r.name != name);
}

/// Return the number of tracked resources.
#[allow(dead_code)]
pub fn tracked_count(tracker: &GpuResourceTracker) -> usize {
    tracker.resources.len()
}

/// Return total memory across all tracked resources.
#[allow(dead_code)]
pub fn resource_memory_total(tracker: &GpuResourceTracker) -> u64 {
    tracker.resources.iter().map(|r| r.memory_bytes).sum()
}

/// Return a map of resource type to count.
#[allow(dead_code)]
pub fn resource_by_type(tracker: &GpuResourceTracker) -> HashMap<String, usize> {
    let mut map = HashMap::new();
    for r in &tracker.resources {
        *map.entry(r.resource_type.clone()).or_insert(0) += 1;
    }
    map
}

/// Serialize to JSON-like string.
#[allow(dead_code)]
pub fn tracker_to_json(tracker: &GpuResourceTracker) -> String {
    let entries: Vec<String> = tracker
        .resources
        .iter()
        .map(|r| {
            format!(
                "{{\"name\":\"{}\",\"type\":\"{}\",\"memory\":{}}}",
                r.name, r.resource_type, r.memory_bytes
            )
        })
        .collect();
    format!("{{\"resources\":[{}]}}", entries.join(","))
}

/// Clear all tracked resources.
#[allow(dead_code)]
pub fn tracker_clear(tracker: &mut GpuResourceTracker) {
    tracker.resources.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tracker() {
        let t = new_gpu_tracker();
        assert_eq!(tracked_count(&t), 0);
    }

    #[test]
    fn test_track_resource() {
        let mut t = new_gpu_tracker();
        track_resource(&mut t, "tex0", "texture", 1024);
        assert_eq!(tracked_count(&t), 1);
    }

    #[test]
    fn test_untrack() {
        let mut t = new_gpu_tracker();
        track_resource(&mut t, "tex0", "texture", 1024);
        untrack_resource(&mut t, "tex0");
        assert_eq!(tracked_count(&t), 0);
    }

    #[test]
    fn test_memory_total() {
        let mut t = new_gpu_tracker();
        track_resource(&mut t, "a", "texture", 100);
        track_resource(&mut t, "b", "buffer", 200);
        assert_eq!(resource_memory_total(&t), 300);
    }

    #[test]
    fn test_by_type() {
        let mut t = new_gpu_tracker();
        track_resource(&mut t, "a", "texture", 100);
        track_resource(&mut t, "b", "texture", 200);
        track_resource(&mut t, "c", "buffer", 50);
        let map = resource_by_type(&t);
        assert_eq!(map["texture"], 2);
        assert_eq!(map["buffer"], 1);
    }

    #[test]
    fn test_to_json() {
        let t = new_gpu_tracker();
        let json = tracker_to_json(&t);
        assert!(json.contains("\"resources\":[]"));
    }

    #[test]
    fn test_clear() {
        let mut t = new_gpu_tracker();
        track_resource(&mut t, "x", "buffer", 50);
        tracker_clear(&mut t);
        assert_eq!(tracked_count(&t), 0);
    }

    #[test]
    fn test_untrack_nonexistent() {
        let mut t = new_gpu_tracker();
        untrack_resource(&mut t, "nope");
        assert_eq!(tracked_count(&t), 0);
    }

    #[test]
    fn test_empty_memory() {
        let t = new_gpu_tracker();
        assert_eq!(resource_memory_total(&t), 0);
    }

    #[test]
    fn test_multiple_same_name() {
        let mut t = new_gpu_tracker();
        track_resource(&mut t, "dup", "texture", 100);
        track_resource(&mut t, "dup", "texture", 200);
        assert_eq!(tracked_count(&t), 2);
        untrack_resource(&mut t, "dup");
        assert_eq!(tracked_count(&t), 0);
    }
}
