//! Debug console stub — stores and queries log messages with severity levels.

/// Severity level for a console entry.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConsoleSeverity {
    Debug,
    Info,
    Warn,
    Error,
}

/// A single log entry in the debug console.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ConsoleEntry {
    pub severity: ConsoleSeverity,
    pub message: String,
    pub index: usize,
}

/// Configuration for the debug console.
#[allow(dead_code)]
pub struct DebugConsoleConfig {
    /// Maximum number of entries to retain.
    pub max_entries: usize,
}

/// The debug console — stores log entries with severity.
#[allow(dead_code)]
pub struct DebugConsole {
    pub entries: Vec<ConsoleEntry>,
    pub config: DebugConsoleConfig,
    next_index: usize,
}

/// Returns a default `DebugConsoleConfig`.
#[allow(dead_code)]
pub fn default_debug_console_config() -> DebugConsoleConfig {
    DebugConsoleConfig { max_entries: 1000 }
}

/// Creates a new `DebugConsole` from a config.
#[allow(dead_code)]
pub fn new_debug_console(cfg: &DebugConsoleConfig) -> DebugConsole {
    DebugConsole {
        entries: Vec::new(),
        config: DebugConsoleConfig {
            max_entries: cfg.max_entries,
        },
        next_index: 0,
    }
}

/// Appends a log entry with the given severity and message.
#[allow(dead_code)]
pub fn console_log(console: &mut DebugConsole, severity: ConsoleSeverity, message: &str) {
    let idx = console.next_index;
    console.next_index += 1;
    console.entries.push(ConsoleEntry {
        severity,
        message: message.to_string(),
        index: idx,
    });
    if console.entries.len() > console.config.max_entries {
        let overflow = console.entries.len() - console.config.max_entries;
        console.entries.drain(0..overflow);
    }
}

/// Returns the total number of entries in the console.
#[allow(dead_code)]
pub fn console_entry_count(console: &DebugConsole) -> usize {
    console.entries.len()
}

/// Returns all entries matching the given severity.
#[allow(dead_code)]
pub fn console_entries_by_severity(
    console: &DebugConsole,
    severity: ConsoleSeverity,
) -> Vec<&ConsoleEntry> {
    console
        .entries
        .iter()
        .filter(|e| e.severity == severity)
        .collect()
}

/// Clears all log entries from the console.
#[allow(dead_code)]
pub fn console_clear(console: &mut DebugConsole) {
    console.entries.clear();
}

/// Returns the last entry in the console, or `None` if empty.
#[allow(dead_code)]
pub fn console_last_entry(console: &DebugConsole) -> Option<&ConsoleEntry> {
    console.entries.last()
}

/// Returns the count of entries with `ConsoleSeverity::Error`.
#[allow(dead_code)]
pub fn console_error_count(console: &DebugConsole) -> usize {
    console
        .entries
        .iter()
        .filter(|e| e.severity == ConsoleSeverity::Error)
        .count()
}

/// Formats all console entries as a single multi-line string.
#[allow(dead_code)]
pub fn console_to_string(console: &DebugConsole) -> String {
    console
        .entries
        .iter()
        .map(|e| format!("[{}] {}", severity_name(e.severity), e.message))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Returns the human-readable name for a `ConsoleSeverity`.
#[allow(dead_code)]
pub fn severity_name(s: ConsoleSeverity) -> &'static str {
    match s {
        ConsoleSeverity::Debug => "DEBUG",
        ConsoleSeverity::Info => "INFO",
        ConsoleSeverity::Warn => "WARN",
        ConsoleSeverity::Error => "ERROR",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_console() -> DebugConsole {
        let cfg = default_debug_console_config();
        new_debug_console(&cfg)
    }

    #[test]
    fn test_new_console_empty() {
        let c = make_console();
        assert_eq!(console_entry_count(&c), 0);
        assert!(console_last_entry(&c).is_none());
    }

    #[test]
    fn test_console_log_info() {
        let mut c = make_console();
        console_log(&mut c, ConsoleSeverity::Info, "hello");
        assert_eq!(console_entry_count(&c), 1);
        let last = console_last_entry(&c).expect("should succeed");
        assert_eq!(last.message, "hello");
        assert_eq!(last.severity, ConsoleSeverity::Info);
    }

    #[test]
    fn test_console_entries_by_severity() {
        let mut c = make_console();
        console_log(&mut c, ConsoleSeverity::Info, "info msg");
        console_log(&mut c, ConsoleSeverity::Error, "error msg");
        console_log(&mut c, ConsoleSeverity::Warn, "warn msg");
        let errors = console_entries_by_severity(&c, ConsoleSeverity::Error);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "error msg");
    }

    #[test]
    fn test_console_clear() {
        let mut c = make_console();
        console_log(&mut c, ConsoleSeverity::Debug, "d1");
        console_log(&mut c, ConsoleSeverity::Debug, "d2");
        console_clear(&mut c);
        assert_eq!(console_entry_count(&c), 0);
        assert!(console_last_entry(&c).is_none());
    }

    #[test]
    fn test_console_error_count() {
        let mut c = make_console();
        console_log(&mut c, ConsoleSeverity::Info, "i1");
        console_log(&mut c, ConsoleSeverity::Error, "e1");
        console_log(&mut c, ConsoleSeverity::Error, "e2");
        assert_eq!(console_error_count(&c), 2);
    }

    #[test]
    fn test_console_to_string() {
        let mut c = make_console();
        console_log(&mut c, ConsoleSeverity::Info, "startup");
        console_log(&mut c, ConsoleSeverity::Warn, "low memory");
        let s = console_to_string(&c);
        assert!(s.contains("[INFO] startup"));
        assert!(s.contains("[WARN] low memory"));
    }

    #[test]
    fn test_severity_name() {
        assert_eq!(severity_name(ConsoleSeverity::Debug), "DEBUG");
        assert_eq!(severity_name(ConsoleSeverity::Info), "INFO");
        assert_eq!(severity_name(ConsoleSeverity::Warn), "WARN");
        assert_eq!(severity_name(ConsoleSeverity::Error), "ERROR");
    }

    #[test]
    fn test_max_entries_trimmed() {
        let cfg = DebugConsoleConfig { max_entries: 3 };
        let mut c = new_debug_console(&cfg);
        for i in 0..5 {
            console_log(&mut c, ConsoleSeverity::Info, &format!("msg {}", i));
        }
        assert_eq!(console_entry_count(&c), 3);
        // Oldest entries dropped — last entry should be msg 4
        assert_eq!(
            console_last_entry(&c).expect("should succeed").message,
            "msg 4"
        );
    }

    #[test]
    fn test_console_last_entry_multiple() {
        let mut c = make_console();
        console_log(&mut c, ConsoleSeverity::Info, "first");
        console_log(&mut c, ConsoleSeverity::Error, "last");
        let last = console_last_entry(&c).expect("should succeed");
        assert_eq!(last.message, "last");
        assert_eq!(last.severity, ConsoleSeverity::Error);
    }

    #[test]
    fn test_entries_by_severity_empty_result() {
        let mut c = make_console();
        console_log(&mut c, ConsoleSeverity::Info, "info only");
        let debug_entries = console_entries_by_severity(&c, ConsoleSeverity::Debug);
        assert!(debug_entries.is_empty());
    }
}
