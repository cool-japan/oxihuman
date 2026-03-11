//! Log sink/router — receives log entries and dispatches them to registered sinks
//! (console, buffer, or null).

/// Severity level for a log entry.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Finest-grained diagnostic information.
    Trace,
    /// Debug-level information.
    Debug,
    /// Informational messages.
    Info,
    /// Warning conditions.
    Warn,
    /// Error conditions.
    Error,
}

/// A single log entry stored in a buffer sink.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Severity of this entry.
    pub level: LogLevel,
    /// Log message text.
    pub message: String,
    /// Monotonic or wall-clock timestamp supplied by the caller.
    pub timestamp: f64,
}

/// Destination type for the log sink.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SinkType {
    /// Print to standard output (simulated — entries are also buffered).
    Console,
    /// Store all entries in an in-memory buffer.
    Buffer,
    /// Discard all entries.
    Null,
}

/// A log sink that receives and optionally stores log entries.
#[allow(dead_code)]
pub struct LogSink {
    /// What kind of sink this is.
    pub sink_type: SinkType,
    /// Minimum level to accept; entries below this level are discarded.
    pub min_level: LogLevel,
    /// Buffered entries (non-empty only for Console and Buffer sinks).
    entries: Vec<LogEntry>,
}

/// Creates a new `LogSink` of the given type with minimum level `Trace` (accept all).
#[allow(dead_code)]
pub fn new_log_sink(sink_type: SinkType) -> LogSink {
    LogSink {
        sink_type,
        min_level: LogLevel::Trace,
        entries: Vec::new(),
    }
}

/// Writes a log entry to the sink.
/// Entries below `sink.min_level` are silently dropped.
/// `Null` sinks discard all entries.
#[allow(dead_code)]
pub fn log_sink_write(sink: &mut LogSink, level: LogLevel, message: &str, timestamp: f64) {
    if sink.sink_type == SinkType::Null {
        return;
    }
    if level < sink.min_level {
        return;
    }
    sink.entries.push(LogEntry {
        level,
        message: message.to_string(),
        timestamp,
    });
}

/// Returns the number of entries currently stored in the sink.
#[allow(dead_code)]
pub fn log_sink_entry_count(sink: &LogSink) -> usize {
    sink.entries.len()
}

/// Returns a slice of all stored log entries.
#[allow(dead_code)]
pub fn log_sink_entries(sink: &LogSink) -> &[LogEntry] {
    &sink.entries
}

/// Clears all stored entries from the sink.
#[allow(dead_code)]
pub fn log_sink_clear(sink: &mut LogSink) {
    sink.entries.clear();
}

/// Returns the numeric value for a `LogLevel` (Trace=0 … Error=4).
#[allow(dead_code)]
pub fn log_level_value(level: LogLevel) -> u8 {
    match level {
        LogLevel::Trace => 0,
        LogLevel::Debug => 1,
        LogLevel::Info => 2,
        LogLevel::Warn => 3,
        LogLevel::Error => 4,
    }
}

/// Returns the human-readable name of a `LogLevel`.
#[allow(dead_code)]
pub fn log_level_name(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Trace => "TRACE",
        LogLevel::Debug => "DEBUG",
        LogLevel::Info => "INFO",
        LogLevel::Warn => "WARN",
        LogLevel::Error => "ERROR",
    }
}

/// Returns the human-readable name of a `SinkType`.
#[allow(dead_code)]
pub fn sink_type_name(t: SinkType) -> &'static str {
    match t {
        SinkType::Console => "console",
        SinkType::Buffer => "buffer",
        SinkType::Null => "null",
    }
}

/// Sets the minimum accepted log level for the sink.
/// Entries below this level will be dropped on the next write.
#[allow(dead_code)]
pub fn set_log_min_level(sink: &mut LogSink, min_level: LogLevel) {
    sink.min_level = min_level;
}

/// Returns all stored entries whose level is exactly `level`.
#[allow(dead_code)]
pub fn log_sink_filter_by_level(sink: &LogSink, level: LogLevel) -> Vec<&LogEntry> {
    sink.entries.iter().filter(|e| e.level == level).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_buffer_sink() -> LogSink {
        new_log_sink(SinkType::Buffer)
    }

    #[test]
    fn test_new_sink_empty() {
        let sink = make_buffer_sink();
        assert_eq!(log_sink_entry_count(&sink), 0);
        assert!(log_sink_entries(&sink).is_empty());
    }

    #[test]
    fn test_write_and_count() {
        let mut sink = make_buffer_sink();
        log_sink_write(&mut sink, LogLevel::Info, "hello", 0.0);
        log_sink_write(&mut sink, LogLevel::Warn, "careful", 1.0);
        assert_eq!(log_sink_entry_count(&sink), 2);
    }

    #[test]
    fn test_null_sink_discards_all() {
        let mut sink = new_log_sink(SinkType::Null);
        log_sink_write(&mut sink, LogLevel::Error, "ignored", 0.0);
        assert_eq!(log_sink_entry_count(&sink), 0);
    }

    #[test]
    fn test_min_level_filter() {
        let mut sink = make_buffer_sink();
        set_log_min_level(&mut sink, LogLevel::Warn);
        log_sink_write(&mut sink, LogLevel::Trace, "too low", 0.0);
        log_sink_write(&mut sink, LogLevel::Info, "still low", 1.0);
        log_sink_write(&mut sink, LogLevel::Warn, "accepted", 2.0);
        log_sink_write(&mut sink, LogLevel::Error, "accepted too", 3.0);
        assert_eq!(log_sink_entry_count(&sink), 2);
    }

    #[test]
    fn test_log_sink_clear() {
        let mut sink = make_buffer_sink();
        log_sink_write(&mut sink, LogLevel::Debug, "msg", 0.0);
        log_sink_clear(&mut sink);
        assert_eq!(log_sink_entry_count(&sink), 0);
    }

    #[test]
    fn test_log_level_values_ordered() {
        assert!(log_level_value(LogLevel::Trace) < log_level_value(LogLevel::Debug));
        assert!(log_level_value(LogLevel::Debug) < log_level_value(LogLevel::Info));
        assert!(log_level_value(LogLevel::Info) < log_level_value(LogLevel::Warn));
        assert!(log_level_value(LogLevel::Warn) < log_level_value(LogLevel::Error));
    }

    #[test]
    fn test_log_level_names() {
        assert_eq!(log_level_name(LogLevel::Trace), "TRACE");
        assert_eq!(log_level_name(LogLevel::Error), "ERROR");
    }

    #[test]
    fn test_sink_type_names() {
        assert_eq!(sink_type_name(SinkType::Console), "console");
        assert_eq!(sink_type_name(SinkType::Buffer), "buffer");
        assert_eq!(sink_type_name(SinkType::Null), "null");
    }

    #[test]
    fn test_filter_by_level() {
        let mut sink = make_buffer_sink();
        log_sink_write(&mut sink, LogLevel::Info, "a", 0.0);
        log_sink_write(&mut sink, LogLevel::Warn, "b", 1.0);
        log_sink_write(&mut sink, LogLevel::Info, "c", 2.0);
        let infos = log_sink_filter_by_level(&sink, LogLevel::Info);
        assert_eq!(infos.len(), 2);
        let warns = log_sink_filter_by_level(&sink, LogLevel::Warn);
        assert_eq!(warns.len(), 1);
    }

    #[test]
    fn test_entries_slice_content() {
        let mut sink = make_buffer_sink();
        log_sink_write(&mut sink, LogLevel::Error, "boom", 42.0);
        let entries = log_sink_entries(&sink);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].message, "boom");
        assert!((entries[0].timestamp - 42.0).abs() < 1e-9);
    }
}
