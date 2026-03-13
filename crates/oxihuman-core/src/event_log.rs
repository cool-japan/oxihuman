//! Event logging and replay system.

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    fn severity(&self) -> u8 {
        match self {
            LogLevel::Trace => 0,
            LogLevel::Debug => 1,
            LogLevel::Info => 2,
            LogLevel::Warn => 3,
            LogLevel::Error => 4,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct LogEvent {
    pub id: u64,
    pub level: LogLevel,
    pub category: String,
    pub message: String,
    pub timestamp: u64,
    pub data: Vec<(String, String)>,
}

#[allow(dead_code)]
pub struct EventLog {
    pub events: Vec<LogEvent>,
    pub max_events: usize,
    pub tick: u64,
    pub next_id: u64,
    pub enabled: bool,
}

#[allow(dead_code)]
pub fn new_event_log(max_events: usize) -> EventLog {
    EventLog {
        events: Vec::new(),
        max_events,
        tick: 0,
        next_id: 1,
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn log_event(log: &mut EventLog, level: LogLevel, category: &str, msg: &str) {
    log_with_data(log, level, category, msg, Vec::new());
}

#[allow(dead_code)]
pub fn log_with_data(
    log: &mut EventLog,
    level: LogLevel,
    category: &str,
    msg: &str,
    data: Vec<(String, String)>,
) {
    if !log.enabled {
        return;
    }
    log.tick += 1;
    let event = LogEvent {
        id: log.next_id,
        level,
        category: category.to_string(),
        message: msg.to_string(),
        timestamp: log.tick,
        data,
    };
    log.next_id += 1;
    log.events.push(event);
    trim_log(log);
}

#[allow(dead_code)]
pub fn filter_by_level(log: &EventLog, min_level: LogLevel) -> Vec<&LogEvent> {
    let min_sev = min_level.severity();
    log.events
        .iter()
        .filter(|e| e.level.severity() >= min_sev)
        .collect()
}

#[allow(dead_code)]
pub fn filter_by_category<'a>(log: &'a EventLog, category: &str) -> Vec<&'a LogEvent> {
    log.events
        .iter()
        .filter(|e| e.category == category)
        .collect()
}

#[allow(dead_code)]
pub fn event_count(log: &EventLog) -> usize {
    log.events.len()
}

#[allow(dead_code)]
pub fn clear_log(log: &mut EventLog) {
    log.events.clear();
}

#[allow(dead_code)]
pub fn last_event(log: &EventLog) -> Option<&LogEvent> {
    log.events.last()
}

#[allow(dead_code)]
pub fn events_since(log: &EventLog, tick: u64) -> Vec<&LogEvent> {
    log.events.iter().filter(|e| e.timestamp > tick).collect()
}

#[allow(dead_code)]
pub fn error_count(log: &EventLog) -> usize {
    log.events
        .iter()
        .filter(|e| e.level == LogLevel::Error)
        .count()
}

#[allow(dead_code)]
pub fn warn_count(log: &EventLog) -> usize {
    log.events
        .iter()
        .filter(|e| e.level == LogLevel::Warn)
        .count()
}

#[allow(dead_code)]
pub fn serialize_log_json(log: &EventLog) -> String {
    let mut parts: Vec<String> = Vec::new();
    for e in &log.events {
        let data_parts: Vec<String> = e
            .data
            .iter()
            .map(|(k, v)| format!("{{\"key\":\"{}\",\"val\":\"{}\"}}", k, v))
            .collect();
        let data_json = format!("[{}]", data_parts.join(","));
        parts.push(format!(
            "{{\"id\":{},\"level\":\"{}\",\"category\":\"{}\",\"message\":\"{}\",\"timestamp\":{},\"data\":{}}}",
            e.id,
            e.level.as_str(),
            e.category,
            e.message,
            e.timestamp,
            data_json
        ));
    }
    format!("[{}]", parts.join(","))
}

#[allow(dead_code)]
pub fn trim_log(log: &mut EventLog) {
    if log.max_events == 0 {
        return;
    }
    while log.events.len() > log.max_events {
        log.events.remove(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_event_log() {
        let log = new_event_log(100);
        assert!(log.events.is_empty());
        assert_eq!(log.max_events, 100);
        assert!(log.enabled);
    }

    #[test]
    fn test_log_event_adds_event() {
        let mut log = new_event_log(100);
        log_event(&mut log, LogLevel::Info, "test", "hello");
        assert_eq!(event_count(&log), 1);
    }

    #[test]
    fn test_filter_by_level_info_up() {
        let mut log = new_event_log(100);
        log_event(&mut log, LogLevel::Debug, "cat", "debug msg");
        log_event(&mut log, LogLevel::Info, "cat", "info msg");
        log_event(&mut log, LogLevel::Error, "cat", "error msg");
        let filtered = filter_by_level(&log, LogLevel::Info);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_by_category() {
        let mut log = new_event_log(100);
        log_event(&mut log, LogLevel::Info, "mesh", "msg1");
        log_event(&mut log, LogLevel::Info, "morph", "msg2");
        log_event(&mut log, LogLevel::Info, "mesh", "msg3");
        let filtered = filter_by_category(&log, "mesh");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_error_count() {
        let mut log = new_event_log(100);
        log_event(&mut log, LogLevel::Info, "c", "m");
        log_event(&mut log, LogLevel::Error, "c", "e1");
        log_event(&mut log, LogLevel::Error, "c", "e2");
        assert_eq!(error_count(&log), 2);
    }

    #[test]
    fn test_warn_count() {
        let mut log = new_event_log(100);
        log_event(&mut log, LogLevel::Warn, "c", "w1");
        log_event(&mut log, LogLevel::Info, "c", "i");
        assert_eq!(warn_count(&log), 1);
    }

    #[test]
    fn test_clear_log() {
        let mut log = new_event_log(100);
        log_event(&mut log, LogLevel::Info, "c", "m");
        log_event(&mut log, LogLevel::Info, "c", "m");
        clear_log(&mut log);
        assert_eq!(event_count(&log), 0);
    }

    #[test]
    fn test_last_event() {
        let mut log = new_event_log(100);
        log_event(&mut log, LogLevel::Info, "c", "first");
        log_event(&mut log, LogLevel::Error, "c", "last");
        let last = last_event(&log).expect("should succeed");
        assert_eq!(last.message, "last");
    }

    #[test]
    fn test_last_event_empty() {
        let log = new_event_log(100);
        assert!(last_event(&log).is_none());
    }

    #[test]
    fn test_events_since() {
        let mut log = new_event_log(100);
        log_event(&mut log, LogLevel::Info, "c", "m1");
        let tick_after_first = log.tick;
        log_event(&mut log, LogLevel::Info, "c", "m2");
        log_event(&mut log, LogLevel::Info, "c", "m3");
        let since = events_since(&log, tick_after_first);
        assert_eq!(since.len(), 2);
    }

    #[test]
    fn test_trim_enforces_max() {
        let mut log = new_event_log(3);
        for i in 0..6 {
            log_event(&mut log, LogLevel::Info, "c", &format!("msg{}", i));
        }
        assert!(log.events.len() <= 3);
    }

    #[test]
    fn test_serialize_non_empty() {
        let mut log = new_event_log(100);
        log_event(&mut log, LogLevel::Info, "cat", "hello world");
        let json = serialize_log_json(&log);
        assert!(json.contains("hello world"));
        assert!(json.contains("INFO"));
        assert!(json.starts_with('['));
        assert!(json.ends_with(']'));
    }

    #[test]
    fn test_log_with_data() {
        let mut log = new_event_log(100);
        log_with_data(
            &mut log,
            LogLevel::Debug,
            "sys",
            "test",
            vec![("key1".to_string(), "val1".to_string())],
        );
        let e = last_event(&log).expect("should succeed");
        assert_eq!(e.data.len(), 1);
        assert_eq!(e.data[0].0, "key1");
    }

    #[test]
    fn test_ids_increment() {
        let mut log = new_event_log(100);
        log_event(&mut log, LogLevel::Info, "c", "m1");
        log_event(&mut log, LogLevel::Info, "c", "m2");
        assert!(log.events[1].id > log.events[0].id);
    }

    #[test]
    fn test_disabled_log_ignores_events() {
        let mut log = new_event_log(100);
        log.enabled = false;
        log_event(&mut log, LogLevel::Error, "c", "msg");
        assert_eq!(event_count(&log), 0);
    }
}
