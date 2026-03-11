//! Structured logging system with levels and categories.

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Log severity levels.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

impl LogLevel {
    fn as_str(self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

/// A single log entry.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct LogEntry {
    /// Entry sequence number.
    pub id: u64,
    /// Severity level.
    pub level: LogLevel,
    /// Category tag (e.g. "render", "physics", "io").
    pub category: String,
    /// The log message.
    pub message: String,
}

/// A structured logger that stores entries in memory.
#[allow(dead_code)]
pub struct Logger {
    /// All recorded entries.
    pub entries: Vec<LogEntry>,
    /// Minimum level to accept (entries below this are ignored).
    pub min_level: LogLevel,
    /// Auto-incrementing ID counter.
    next_id: u64,
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

/// Create a new logger with the given minimum level.
#[allow(dead_code)]
pub fn new_logger(min_level: LogLevel) -> Logger {
    Logger {
        entries: Vec::new(),
        min_level,
        next_id: 1,
    }
}

// ---------------------------------------------------------------------------
// Logging functions
// ---------------------------------------------------------------------------

/// Log a message at the specified level and category.
/// Entries below the logger's minimum level are silently ignored.
#[allow(dead_code)]
pub fn log_message(logger: &mut Logger, level: LogLevel, category: &str, message: &str) {
    if level < logger.min_level {
        return;
    }
    let entry = LogEntry {
        id: logger.next_id,
        level,
        category: category.to_string(),
        message: message.to_string(),
    };
    logger.next_id += 1;
    logger.entries.push(entry);
}

/// Log a trace-level message.
#[allow(dead_code)]
pub fn log_trace(logger: &mut Logger, category: &str, message: &str) {
    log_message(logger, LogLevel::Trace, category, message);
}

/// Log a debug-level message.
#[allow(dead_code)]
pub fn log_debug(logger: &mut Logger, category: &str, message: &str) {
    log_message(logger, LogLevel::Debug, category, message);
}

/// Log an info-level message.
#[allow(dead_code)]
pub fn log_info(logger: &mut Logger, category: &str, message: &str) {
    log_message(logger, LogLevel::Info, category, message);
}

/// Log a warn-level message.
#[allow(dead_code)]
pub fn log_warn(logger: &mut Logger, category: &str, message: &str) {
    log_message(logger, LogLevel::Warn, category, message);
}

/// Log an error-level message.
#[allow(dead_code)]
pub fn log_error(logger: &mut Logger, category: &str, message: &str) {
    log_message(logger, LogLevel::Error, category, message);
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Change the minimum log level (entries below this are dropped).
#[allow(dead_code)]
pub fn set_min_level(logger: &mut Logger, level: LogLevel) {
    logger.min_level = level;
}

// ---------------------------------------------------------------------------
// Queries
// ---------------------------------------------------------------------------

/// Return the total number of stored entries.
#[allow(dead_code)]
pub fn entry_count(logger: &Logger) -> usize {
    logger.entries.len()
}

/// Return entries matching a specific level.
#[allow(dead_code)]
pub fn entries_by_level(logger: &Logger, level: LogLevel) -> Vec<&LogEntry> {
    logger.entries.iter().filter(|e| e.level == level).collect()
}

/// Clear all stored log entries.
#[allow(dead_code)]
pub fn clear_log(logger: &mut Logger) {
    logger.entries.clear();
}

/// Serialize the log to a JSON string.
#[allow(dead_code)]
pub fn logger_to_json(logger: &Logger) -> String {
    let mut buf = String::from("[");
    for (i, entry) in logger.entries.iter().enumerate() {
        if i > 0 {
            buf.push(',');
        }
        buf.push_str(&format!(
            r#"{{"id":{},"level":"{}","category":"{}","message":"{}"}}"#,
            entry.id,
            entry.level.as_str(),
            entry.category,
            entry.message
        ));
    }
    buf.push(']');
    buf
}

/// Return entries matching a specific category.
#[allow(dead_code)]
pub fn filter_by_category<'a>(logger: &'a Logger, category: &str) -> Vec<&'a LogEntry> {
    logger
        .entries
        .iter()
        .filter(|e| e.category == category)
        .collect()
}

/// Return the last `n` entries (or fewer if the log is smaller).
#[allow(dead_code)]
pub fn last_n_entries(logger: &Logger, n: usize) -> Vec<&LogEntry> {
    let len = logger.entries.len();
    let start = len.saturating_sub(n);
    logger.entries[start..].iter().collect()
}

/// Check whether the log contains any error-level entries.
#[allow(dead_code)]
pub fn has_errors(logger: &Logger) -> bool {
    logger.entries.iter().any(|e| e.level == LogLevel::Error)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_logger() {
        let l = new_logger(LogLevel::Info);
        assert_eq!(entry_count(&l), 0);
        assert_eq!(l.min_level, LogLevel::Info);
    }

    #[test]
    fn test_log_message_above_min() {
        let mut l = new_logger(LogLevel::Debug);
        log_message(&mut l, LogLevel::Info, "cat", "hello");
        assert_eq!(entry_count(&l), 1);
    }

    #[test]
    fn test_log_message_below_min_ignored() {
        let mut l = new_logger(LogLevel::Warn);
        log_message(&mut l, LogLevel::Debug, "cat", "dropped");
        assert_eq!(entry_count(&l), 0);
    }

    #[test]
    fn test_log_trace() {
        let mut l = new_logger(LogLevel::Trace);
        log_trace(&mut l, "test", "trace msg");
        assert_eq!(entry_count(&l), 1);
        assert_eq!(l.entries[0].level, LogLevel::Trace);
    }

    #[test]
    fn test_log_debug() {
        let mut l = new_logger(LogLevel::Trace);
        log_debug(&mut l, "test", "debug msg");
        assert_eq!(l.entries[0].level, LogLevel::Debug);
    }

    #[test]
    fn test_log_info() {
        let mut l = new_logger(LogLevel::Trace);
        log_info(&mut l, "test", "info msg");
        assert_eq!(l.entries[0].level, LogLevel::Info);
    }

    #[test]
    fn test_log_warn() {
        let mut l = new_logger(LogLevel::Trace);
        log_warn(&mut l, "test", "warn msg");
        assert_eq!(l.entries[0].level, LogLevel::Warn);
    }

    #[test]
    fn test_log_error() {
        let mut l = new_logger(LogLevel::Trace);
        log_error(&mut l, "test", "error msg");
        assert_eq!(l.entries[0].level, LogLevel::Error);
    }

    #[test]
    fn test_set_min_level() {
        let mut l = new_logger(LogLevel::Trace);
        log_trace(&mut l, "a", "ok");
        assert_eq!(entry_count(&l), 1);
        set_min_level(&mut l, LogLevel::Error);
        log_trace(&mut l, "a", "dropped");
        assert_eq!(entry_count(&l), 1);
    }

    #[test]
    fn test_entries_by_level() {
        let mut l = new_logger(LogLevel::Trace);
        log_info(&mut l, "a", "i1");
        log_warn(&mut l, "a", "w1");
        log_info(&mut l, "a", "i2");
        assert_eq!(entries_by_level(&l, LogLevel::Info).len(), 2);
        assert_eq!(entries_by_level(&l, LogLevel::Warn).len(), 1);
    }

    #[test]
    fn test_clear_log() {
        let mut l = new_logger(LogLevel::Trace);
        log_info(&mut l, "a", "msg");
        clear_log(&mut l);
        assert_eq!(entry_count(&l), 0);
    }

    #[test]
    fn test_logger_to_json() {
        let mut l = new_logger(LogLevel::Trace);
        log_info(&mut l, "render", "frame done");
        let json = logger_to_json(&l);
        assert!(json.contains("render"));
        assert!(json.contains("frame done"));
        assert!(json.starts_with('['));
        assert!(json.ends_with(']'));
    }

    #[test]
    fn test_filter_by_category() {
        let mut l = new_logger(LogLevel::Trace);
        log_info(&mut l, "render", "r1");
        log_info(&mut l, "physics", "p1");
        log_info(&mut l, "render", "r2");
        assert_eq!(filter_by_category(&l, "render").len(), 2);
        assert_eq!(filter_by_category(&l, "physics").len(), 1);
        assert_eq!(filter_by_category(&l, "audio").len(), 0);
    }

    #[test]
    fn test_last_n_entries() {
        let mut l = new_logger(LogLevel::Trace);
        for i in 0..10 {
            log_info(&mut l, "t", &format!("msg{i}"));
        }
        let last3 = last_n_entries(&l, 3);
        assert_eq!(last3.len(), 3);
        assert!(last3[0].message.contains("msg7"));
    }

    #[test]
    fn test_last_n_entries_more_than_available() {
        let mut l = new_logger(LogLevel::Trace);
        log_info(&mut l, "t", "only one");
        let result = last_n_entries(&l, 100);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_has_errors_false() {
        let mut l = new_logger(LogLevel::Trace);
        log_info(&mut l, "a", "fine");
        log_warn(&mut l, "a", "warning");
        assert!(!has_errors(&l));
    }

    #[test]
    fn test_has_errors_true() {
        let mut l = new_logger(LogLevel::Trace);
        log_error(&mut l, "a", "bad");
        assert!(has_errors(&l));
    }
}
