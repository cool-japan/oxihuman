//! Export log tracking.
#![allow(dead_code)]

/// A single log entry for an export operation.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ExportLogEntry2 {
    pub message: String,
    pub is_error: bool,
    pub timestamp_ms: u64,
}

/// An export log.
#[allow(dead_code)]
pub struct ExportLog2 {
    pub entries: Vec<ExportLogEntry2>,
}

/// Create a new empty export log.
#[allow(dead_code)]
pub fn new_export_log2() -> ExportLog2 {
    ExportLog2 { entries: Vec::new() }
}

/// Log a start event.
#[allow(dead_code)]
pub fn log2_start(log: &mut ExportLog2, path: &str) {
    log.entries.push(ExportLogEntry2 {
        message: format!("Start: {}", path),
        is_error: false,
        timestamp_ms: 0,
    });
}

/// Log a finish event.
#[allow(dead_code)]
pub fn log2_finish(log: &mut ExportLog2, path: &str) {
    log.entries.push(ExportLogEntry2 {
        message: format!("Finish: {}", path),
        is_error: false,
        timestamp_ms: 1,
    });
}

/// Log an error.
#[allow(dead_code)]
pub fn log2_error(log: &mut ExportLog2, msg: &str) {
    log.entries.push(ExportLogEntry2 {
        message: format!("Error: {}", msg),
        is_error: true,
        timestamp_ms: 0,
    });
}

/// Get the number of log entries.
#[allow(dead_code)]
pub fn log2_len(log: &ExportLog2) -> usize {
    log.entries.len()
}

/// Get entry at index.
#[allow(dead_code)]
pub fn log2_entry_at(log: &ExportLog2, i: usize) -> Option<&ExportLogEntry2> {
    log.entries.get(i)
}

/// Convert log to string.
#[allow(dead_code)]
pub fn export_log2_to_string(log: &ExportLog2) -> String {
    log.entries.iter().map(|e| e.message.clone()).collect::<Vec<_>>().join("\n")
}

/// Clear all log entries.
#[allow(dead_code)]
pub fn clear_log2(log: &mut ExportLog2) {
    log.entries.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_log_empty() {
        let l = new_export_log2();
        assert_eq!(log2_len(&l), 0);
    }

    #[test]
    fn test_log_start() {
        let mut l = new_export_log2();
        log2_start(&mut l, "/out/a.glb");
        assert_eq!(log2_len(&l), 1);
    }

    #[test]
    fn test_log_finish() {
        let mut l = new_export_log2();
        log2_finish(&mut l, "/out/a.glb");
        let e = log2_entry_at(&l, 0).unwrap();
        assert!(e.message.contains("Finish"));
    }

    #[test]
    fn test_log_error() {
        let mut l = new_export_log2();
        log2_error(&mut l, "file not found");
        let e = log2_entry_at(&l, 0).unwrap();
        assert!(e.is_error);
    }

    #[test]
    fn test_log_entry_at_oob() {
        let l = new_export_log2();
        assert!(log2_entry_at(&l, 0).is_none());
    }

    #[test]
    fn test_export_log_to_string() {
        let mut l = new_export_log2();
        log2_start(&mut l, "/a");
        let s = export_log2_to_string(&l);
        assert!(s.contains("Start"));
    }

    #[test]
    fn test_clear_log() {
        let mut l = new_export_log2();
        log2_start(&mut l, "/x");
        clear_log2(&mut l);
        assert_eq!(log2_len(&l), 0);
    }

    #[test]
    fn test_multiple_entries() {
        let mut l = new_export_log2();
        log2_start(&mut l, "/a");
        log2_finish(&mut l, "/a");
        assert_eq!(log2_len(&l), 2);
    }

    #[test]
    fn test_log_entry_struct() {
        let e = ExportLogEntry2 { message: "ok".to_string(), is_error: false, timestamp_ms: 100 };
        assert_eq!(e.timestamp_ms, 100);
    }
}
