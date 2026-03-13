#![allow(dead_code)]

//! GPU timestamp query for profiling.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TimestampEntry {
    pub label: String,
    pub timestamp_ns: u64,
    pub resolved: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TimestampQuerySet {
    pub entries: Vec<TimestampEntry>,
    pub frame: u64,
    pub tick_freq_hz: u64,
}

#[allow(dead_code)]
pub fn new_timestamp_query_set(tick_freq_hz: u64) -> TimestampQuerySet {
    TimestampQuerySet {
        entries: Vec::new(),
        frame: 0,
        tick_freq_hz,
    }
}

#[allow(dead_code)]
pub fn tqs_record(set: &mut TimestampQuerySet, label: &str) {
    set.entries.push(TimestampEntry {
        label: label.to_string(),
        timestamp_ns: 0,
        resolved: false,
    });
}

#[allow(dead_code)]
pub fn tqs_resolve(set: &mut TimestampQuerySet, label: &str, ticks: u64) {
    if let Some(e) = set.entries.iter_mut().find(|e| e.label == label && !e.resolved) {
        e.timestamp_ns = if set.tick_freq_hz > 0 {
            (ticks as u128 * 1_000_000_000 / set.tick_freq_hz as u128) as u64
        } else {
            ticks
        };
        e.resolved = true;
    }
}

#[allow(dead_code)]
pub fn tqs_duration_ns(set: &TimestampQuerySet, start: &str, end: &str) -> Option<u64> {
    let t_start = set.entries.iter().find(|e| e.label == start && e.resolved)?.timestamp_ns;
    let t_end = set.entries.iter().find(|e| e.label == end && e.resolved)?.timestamp_ns;
    if t_end >= t_start { Some(t_end - t_start) } else { None }
}

#[allow(dead_code)]
pub fn tqs_entry_count(set: &TimestampQuerySet) -> usize {
    set.entries.len()
}

#[allow(dead_code)]
pub fn tqs_resolved_count(set: &TimestampQuerySet) -> usize {
    set.entries.iter().filter(|e| e.resolved).count()
}

#[allow(dead_code)]
pub fn tqs_advance_frame(set: &mut TimestampQuerySet) {
    set.entries.clear();
    set.frame += 1;
}

#[allow(dead_code)]
pub fn tqs_clear(set: &mut TimestampQuerySet) {
    set.entries.clear();
}

#[allow(dead_code)]
pub fn tqs_to_json(set: &TimestampQuerySet) -> String {
    format!(
        "{{\"frame\":{},\"entry_count\":{},\"resolved_count\":{}}}",
        set.frame,
        set.entries.len(),
        tqs_resolved_count(set)
    )
}

#[allow(dead_code)]
pub fn tqs_duration_ms(set: &TimestampQuerySet, start: &str, end: &str) -> Option<f32> {
    tqs_duration_ns(set, start, end).map(|ns| ns as f32 / 1_000_000.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_set() {
        let s = new_timestamp_query_set(1_000_000_000);
        assert_eq!(tqs_entry_count(&s), 0);
    }

    #[test]
    fn test_record() {
        let mut s = new_timestamp_query_set(1_000_000_000);
        tqs_record(&mut s, "frame_start");
        assert_eq!(tqs_entry_count(&s), 1);
    }

    #[test]
    fn test_resolve() {
        let mut s = new_timestamp_query_set(1_000_000_000);
        tqs_record(&mut s, "frame_start");
        tqs_resolve(&mut s, "frame_start", 1_000_000_000);
        assert_eq!(tqs_resolved_count(&s), 1);
    }

    #[test]
    fn test_resolved_timestamp_ns() {
        let mut s = new_timestamp_query_set(1_000_000_000);
        tqs_record(&mut s, "t");
        tqs_resolve(&mut s, "t", 2_000_000_000);
        let entry = s.entries.iter().find(|e| e.label == "t").expect("should succeed");
        assert_eq!(entry.timestamp_ns, 2_000_000_000);
    }

    #[test]
    fn test_duration_ns() {
        let mut s = new_timestamp_query_set(1_000_000_000);
        tqs_record(&mut s, "start");
        tqs_record(&mut s, "end");
        tqs_resolve(&mut s, "start", 0);
        tqs_resolve(&mut s, "end", 16_000_000);
        let dur = tqs_duration_ns(&s, "start", "end");
        assert!(dur.is_some());
    }

    #[test]
    fn test_duration_ms() {
        let mut s = new_timestamp_query_set(1_000_000_000);
        tqs_record(&mut s, "a");
        tqs_record(&mut s, "b");
        tqs_resolve(&mut s, "a", 0);
        tqs_resolve(&mut s, "b", 16_000_000);
        let ms = tqs_duration_ms(&s, "a", "b");
        assert!(ms.is_some());
        assert!(ms.expect("should succeed") > 0.0);
    }

    #[test]
    fn test_advance_frame() {
        let mut s = new_timestamp_query_set(1_000_000_000);
        tqs_record(&mut s, "t");
        tqs_advance_frame(&mut s);
        assert_eq!(tqs_entry_count(&s), 0);
        assert_eq!(s.frame, 1);
    }

    #[test]
    fn test_clear() {
        let mut s = new_timestamp_query_set(1_000_000_000);
        tqs_record(&mut s, "t");
        tqs_clear(&mut s);
        assert_eq!(tqs_entry_count(&s), 0);
    }

    #[test]
    fn test_to_json() {
        let s = new_timestamp_query_set(1_000_000_000);
        let json = tqs_to_json(&s);
        assert!(json.contains("frame"));
    }

    #[test]
    fn test_duration_missing_label() {
        let s = new_timestamp_query_set(1_000_000_000);
        assert!(tqs_duration_ns(&s, "missing_a", "missing_b").is_none());
    }
}
