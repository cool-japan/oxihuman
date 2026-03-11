//! Command history with undo/redo — stores named command records with metadata.

// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// A record of a single executed command.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct CommandRecord {
    /// Display name of the command.
    pub name: String,
    /// Timestamp when the command was executed (seconds since epoch or app start).
    pub timestamp: f64,
}

/// Configuration for the command history.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CommandHistoryConfig {
    /// Maximum number of undo steps kept (0 = unlimited).
    pub max_depth: usize,
}

/// Stores an undo/redo stack of named command records.
///
/// The stack is split at `cursor`:
///   - `entries[0..cursor]` are the undo-able commands (most-recent is `entries[cursor-1]`).
///   - `entries[cursor..]` are the redo-able commands (next redo is `entries[cursor]`).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CommandHistory {
    pub config: CommandHistoryConfig,
    pub entries: Vec<CommandRecord>,
    /// Points to the index past the last executed command.
    pub cursor: usize,
}

/// Returns a default `CommandHistoryConfig`.
#[allow(dead_code)]
pub fn default_command_history_config() -> CommandHistoryConfig {
    CommandHistoryConfig { max_depth: 0 }
}

/// Creates a new empty `CommandHistory`.
#[allow(dead_code)]
pub fn new_command_history(cfg: &CommandHistoryConfig) -> CommandHistory {
    CommandHistory {
        config: cfg.clone(),
        entries: Vec::new(),
        cursor: 0,
    }
}

/// Pushes a new command record, discarding any redo entries after the current cursor.
/// Trims the oldest entries if `max_depth` is set.
#[allow(dead_code)]
pub fn history_push(history: &mut CommandHistory, name: &str, timestamp: f64) {
    // Discard redo branch
    history.entries.truncate(history.cursor);
    history.entries.push(CommandRecord {
        name: name.to_string(),
        timestamp,
    });
    history.cursor = history.entries.len();

    // Trim oldest entries to respect max_depth
    let max = history.config.max_depth;
    if max > 0 && history.cursor > max {
        let trim = history.cursor - max;
        history.entries.drain(0..trim);
        history.cursor = history.entries.len();
    }
}

/// Undoes the last command: moves the cursor back one step and returns the record.
/// Returns `None` if there is nothing to undo.
#[allow(dead_code)]
pub fn history_undo(history: &mut CommandHistory) -> Option<CommandRecord> {
    if history.cursor == 0 {
        return None;
    }
    history.cursor -= 1;
    Some(history.entries[history.cursor].clone())
}

/// Redoes the next command: moves the cursor forward and returns the record.
/// Returns `None` if there is nothing to redo.
#[allow(dead_code)]
pub fn history_redo(history: &mut CommandHistory) -> Option<CommandRecord> {
    if history.cursor >= history.entries.len() {
        return None;
    }
    let record = history.entries[history.cursor].clone();
    history.cursor += 1;
    Some(record)
}

/// Returns `true` if there is at least one command that can be undone.
#[allow(dead_code)]
pub fn history_can_undo(history: &CommandHistory) -> bool {
    history.cursor > 0
}

/// Returns `true` if there is at least one command that can be redone.
#[allow(dead_code)]
pub fn history_can_redo(history: &CommandHistory) -> bool {
    history.cursor < history.entries.len()
}

/// Returns the total number of entries in the history (undo + redo combined).
#[allow(dead_code)]
pub fn history_entry_count(history: &CommandHistory) -> usize {
    history.entries.len()
}

/// Clears all entries and resets the cursor.
#[allow(dead_code)]
pub fn history_clear(history: &mut CommandHistory) {
    history.entries.clear();
    history.cursor = 0;
}

/// Returns a reference to the most recently executed command (the one at `cursor - 1`),
/// or `None` if the cursor is at the beginning.
#[allow(dead_code)]
pub fn history_current_command(history: &CommandHistory) -> Option<&CommandRecord> {
    if history.cursor == 0 {
        return None;
    }
    Some(&history.entries[history.cursor - 1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_command_history_config();
        assert_eq!(cfg.max_depth, 0);
    }

    #[test]
    fn test_new_history_empty() {
        let cfg = default_command_history_config();
        let history = new_command_history(&cfg);
        assert_eq!(history_entry_count(&history), 0);
        assert!(!history_can_undo(&history));
        assert!(!history_can_redo(&history));
    }

    #[test]
    fn test_push_and_can_undo() {
        let cfg = default_command_history_config();
        let mut history = new_command_history(&cfg);
        history_push(&mut history, "move", 1.0);
        assert!(history_can_undo(&history));
        assert!(!history_can_redo(&history));
        assert_eq!(history_entry_count(&history), 1);
    }

    #[test]
    fn test_undo_returns_record() {
        let cfg = default_command_history_config();
        let mut history = new_command_history(&cfg);
        history_push(&mut history, "scale", 2.0);
        let rec = history_undo(&mut history).unwrap();
        assert_eq!(rec.name, "scale");
        assert!((rec.timestamp - 2.0).abs() < 1e-9);
        assert!(!history_can_undo(&history));
        assert!(history_can_redo(&history));
    }

    #[test]
    fn test_redo() {
        let cfg = default_command_history_config();
        let mut history = new_command_history(&cfg);
        history_push(&mut history, "rotate", 3.0);
        history_undo(&mut history);
        let rec = history_redo(&mut history).unwrap();
        assert_eq!(rec.name, "rotate");
        assert!(history_can_undo(&history));
        assert!(!history_can_redo(&history));
    }

    #[test]
    fn test_push_discards_redo_branch() {
        let cfg = default_command_history_config();
        let mut history = new_command_history(&cfg);
        history_push(&mut history, "A", 1.0);
        history_push(&mut history, "B", 2.0);
        history_undo(&mut history); // cursor points back before B
        history_push(&mut history, "C", 3.0); // B is discarded
        assert_eq!(history_entry_count(&history), 2); // A and C
        assert!(!history_can_redo(&history));
    }

    #[test]
    fn test_max_depth() {
        let cfg = CommandHistoryConfig { max_depth: 3 };
        let mut history = new_command_history(&cfg);
        for i in 0..5_u32 {
            history_push(&mut history, &format!("cmd{i}"), f64::from(i));
        }
        assert_eq!(history_entry_count(&history), 3);
        assert_eq!(history.entries[0].name, "cmd2");
    }

    #[test]
    fn test_clear() {
        let cfg = default_command_history_config();
        let mut history = new_command_history(&cfg);
        history_push(&mut history, "x", 1.0);
        history_push(&mut history, "y", 2.0);
        history_clear(&mut history);
        assert_eq!(history_entry_count(&history), 0);
        assert!(!history_can_undo(&history));
        assert!(!history_can_redo(&history));
    }

    #[test]
    fn test_current_command() {
        let cfg = default_command_history_config();
        let mut history = new_command_history(&cfg);
        assert!(history_current_command(&history).is_none());
        history_push(&mut history, "paint", 5.0);
        assert_eq!(history_current_command(&history).unwrap().name, "paint");
        history_undo(&mut history);
        assert!(history_current_command(&history).is_none());
    }

    #[test]
    fn test_undo_none_when_empty() {
        let cfg = default_command_history_config();
        let mut history = new_command_history(&cfg);
        assert!(history_undo(&mut history).is_none());
    }

    #[test]
    fn test_redo_none_when_no_future() {
        let cfg = default_command_history_config();
        let mut history = new_command_history(&cfg);
        history_push(&mut history, "A", 0.0);
        assert!(history_redo(&mut history).is_none());
    }
}
