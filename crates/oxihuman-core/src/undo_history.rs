//! Undo/redo history stack for editor operations.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HistoryConfig {
    pub max_history: usize,
    pub merge_window_ms: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub id: u64,
    pub description: String,
    pub timestamp_ms: u64,
    pub data_size: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UndoHistory {
    pub config: HistoryConfig,
    pub entries: Vec<HistoryEntry>,
    pub cursor: usize,
    pub next_id: u64,
}

#[allow(dead_code)]
pub fn default_history_config() -> HistoryConfig {
    HistoryConfig {
        max_history: 100,
        merge_window_ms: 500,
    }
}

#[allow(dead_code)]
pub fn new_undo_history(cfg: HistoryConfig) -> UndoHistory {
    UndoHistory {
        config: cfg,
        entries: Vec::new(),
        cursor: 0,
        next_id: 1,
    }
}

/// Push a new history entry, truncating any redo entries above the cursor.
/// Returns the new entry's id.
#[allow(dead_code)]
pub fn push_history(h: &mut UndoHistory, desc: &str, ts: u64, size: usize) -> u64 {
    // Truncate redo stack
    if h.cursor < h.entries.len() {
        h.entries.truncate(h.cursor);
    }
    let id = h.next_id;
    h.next_id += 1;
    let entry = HistoryEntry {
        id,
        description: desc.to_string(),
        timestamp_ms: ts,
        data_size: size,
    };
    h.entries.push(entry);
    // Enforce max_history
    while h.entries.len() > h.config.max_history {
        h.entries.remove(0);
    }
    h.cursor = h.entries.len();
    id
}

/// Move cursor back one step and return the entry that was undone.
#[allow(dead_code)]
pub fn undo(h: &mut UndoHistory) -> Option<&HistoryEntry> {
    if h.cursor == 0 {
        return None;
    }
    h.cursor -= 1;
    h.entries.get(h.cursor)
}

/// Move cursor forward one step and return the entry that was redone.
#[allow(dead_code)]
pub fn redo(h: &mut UndoHistory) -> Option<&HistoryEntry> {
    if h.cursor >= h.entries.len() {
        return None;
    }
    let idx = h.cursor;
    h.cursor += 1;
    h.entries.get(idx)
}

#[allow(dead_code)]
pub fn can_undo(h: &UndoHistory) -> bool {
    h.cursor > 0
}

#[allow(dead_code)]
pub fn can_redo(h: &UndoHistory) -> bool {
    h.cursor < h.entries.len()
}

/// Number of entries accessible via undo (below cursor).
#[allow(dead_code)]
pub fn history_depth(h: &UndoHistory) -> usize {
    h.cursor
}

#[allow(dead_code)]
pub fn clear_history(h: &mut UndoHistory) {
    h.entries.clear();
    h.cursor = 0;
}

#[allow(dead_code)]
pub fn history_to_json(h: &UndoHistory) -> String {
    let entries: Vec<String> = h.entries.iter().map(|e| {
        format!(
            r#"{{"id":{},"description":"{}","timestamp_ms":{},"data_size":{}}}"#,
            e.id, e.description, e.timestamp_ms, e.data_size
        )
    }).collect();
    format!(
        r#"{{"cursor":{},"entry_count":{},"entries":[{}]}}"#,
        h.cursor,
        h.entries.len(),
        entries.join(",")
    )
}

#[allow(dead_code)]
pub fn current_entry(h: &UndoHistory) -> Option<&HistoryEntry> {
    if h.cursor == 0 {
        return None;
    }
    h.entries.get(h.cursor - 1)
}

#[allow(dead_code)]
pub fn history_total_size(h: &UndoHistory) -> usize {
    h.entries.iter().map(|e| e.data_size).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_history_config() {
        let cfg = default_history_config();
        assert_eq!(cfg.max_history, 100);
        assert_eq!(cfg.merge_window_ms, 500);
    }

    #[test]
    fn test_push_and_depth() {
        let cfg = default_history_config();
        let mut h = new_undo_history(cfg);
        push_history(&mut h, "op1", 1000, 64);
        push_history(&mut h, "op2", 2000, 128);
        assert_eq!(history_depth(&h), 2);
    }

    #[test]
    fn test_undo_redo() {
        let cfg = default_history_config();
        let mut h = new_undo_history(cfg);
        push_history(&mut h, "op1", 1000, 64);
        push_history(&mut h, "op2", 2000, 128);
        assert!(can_undo(&h));
        assert!(!can_redo(&h));
        let e = undo(&mut h).unwrap();
        assert_eq!(e.description, "op2");
        assert!(can_redo(&h));
        let e2 = redo(&mut h).unwrap();
        assert_eq!(e2.description, "op2");
    }

    #[test]
    fn test_push_truncates_redo() {
        let cfg = default_history_config();
        let mut h = new_undo_history(cfg);
        push_history(&mut h, "op1", 1000, 10);
        push_history(&mut h, "op2", 2000, 10);
        undo(&mut h);
        push_history(&mut h, "op3", 3000, 10);
        assert!(!can_redo(&h));
        assert_eq!(history_depth(&h), 2);
    }

    #[test]
    fn test_current_entry() {
        let cfg = default_history_config();
        let mut h = new_undo_history(cfg);
        assert!(current_entry(&h).is_none());
        push_history(&mut h, "a", 1, 4);
        let e = current_entry(&h).unwrap();
        assert_eq!(e.description, "a");
    }

    #[test]
    fn test_clear_history() {
        let cfg = default_history_config();
        let mut h = new_undo_history(cfg);
        push_history(&mut h, "x", 0, 0);
        clear_history(&mut h);
        assert_eq!(history_depth(&h), 0);
        assert!(!can_undo(&h));
    }

    #[test]
    fn test_history_total_size() {
        let cfg = default_history_config();
        let mut h = new_undo_history(cfg);
        push_history(&mut h, "a", 0, 100);
        push_history(&mut h, "b", 1, 200);
        assert_eq!(history_total_size(&h), 300);
    }

    #[test]
    fn test_max_history_enforced() {
        let cfg = HistoryConfig { max_history: 3, merge_window_ms: 0 };
        let mut h = new_undo_history(cfg);
        for i in 0..5u64 {
            push_history(&mut h, "op", i, 1);
        }
        assert_eq!(h.entries.len(), 3);
    }

    #[test]
    fn test_history_to_json() {
        let cfg = default_history_config();
        let mut h = new_undo_history(cfg);
        push_history(&mut h, "test_op", 999, 32);
        let j = history_to_json(&h);
        assert!(j.contains("test_op"));
        assert!(j.contains("cursor"));
    }
}
