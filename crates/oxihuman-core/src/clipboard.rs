//! Clipboard manager for copy/paste operations with typed content.

/// Types of content that can be stored in the clipboard.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum ClipboardContent {
    /// Plain text string.
    Text(String),
    /// Named parameter values (key, value pairs).
    Parameters(Vec<(String, f32)>),
    /// Pose data: name and joint rotations as flat f32 array.
    Pose(String, Vec<f32>),
    /// RGBA color.
    Color([f32; 4]),
}

/// A single clipboard entry with metadata.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ClipboardEntry {
    /// The stored content.
    pub content: ClipboardContent,
    /// Monotonic sequence number.
    pub sequence: u64,
    /// Optional label/description.
    pub label: String,
}

/// Clipboard manager with history support.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Clipboard {
    /// Current clipboard content (most recently copied).
    current: Option<ClipboardEntry>,
    /// History of past clipboard entries (most recent first).
    history: Vec<ClipboardEntry>,
    /// Maximum history entries to keep.
    max_history: usize,
    /// Running sequence counter.
    next_seq: u64,
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

/// Create a new empty clipboard with the given history capacity.
#[allow(dead_code)]
pub fn new_clipboard(max_history: usize) -> Clipboard {
    Clipboard {
        current: None,
        history: Vec::new(),
        max_history,
        next_seq: 0,
    }
}

// ---------------------------------------------------------------------------
// Core operations
// ---------------------------------------------------------------------------

/// Copy content to the clipboard with an optional label.
#[allow(dead_code)]
pub fn copy_to_clipboard(cb: &mut Clipboard, content: ClipboardContent, label: &str) {
    // Push current to history if present.
    if let Some(prev) = cb.current.take() {
        cb.history.insert(0, prev);
        if cb.history.len() > cb.max_history {
            cb.history.truncate(cb.max_history);
        }
    }
    let entry = ClipboardEntry {
        content,
        sequence: cb.next_seq,
        label: label.to_string(),
    };
    cb.next_seq += 1;
    cb.current = Some(entry);
}

/// Paste (clone) the current clipboard content, if any.
#[allow(dead_code)]
pub fn paste_from_clipboard(cb: &Clipboard) -> Option<ClipboardContent> {
    cb.current.as_ref().map(|e| e.content.clone())
}

/// Check whether the clipboard has any content.
#[allow(dead_code)]
pub fn clipboard_has_content(cb: &Clipboard) -> bool {
    cb.current.is_some()
}

/// Return the type name of the current clipboard content.
#[allow(dead_code)]
pub fn clipboard_content_type(cb: &Clipboard) -> Option<&'static str> {
    cb.current.as_ref().map(|e| match &e.content {
        ClipboardContent::Text(_) => "Text",
        ClipboardContent::Parameters(_) => "Parameters",
        ClipboardContent::Pose(_, _) => "Pose",
        ClipboardContent::Color(_) => "Color",
    })
}

/// Clear the clipboard (current + history).
#[allow(dead_code)]
pub fn clear_clipboard(cb: &mut Clipboard) {
    cb.current = None;
    cb.history.clear();
}

/// Return the number of entries in the clipboard history (not counting current).
#[allow(dead_code)]
pub fn clipboard_history_count(cb: &Clipboard) -> usize {
    cb.history.len()
}

/// Get a history entry by index (0 = most recently replaced).
#[allow(dead_code)]
pub fn get_history_entry(cb: &Clipboard, index: usize) -> Option<&ClipboardEntry> {
    cb.history.get(index)
}

/// Serialize the current clipboard content to a JSON string.
#[allow(dead_code)]
pub fn clipboard_to_json(cb: &Clipboard) -> String {
    match &cb.current {
        None => "{\"content\":null}".to_string(),
        Some(entry) => {
            let content_str = match &entry.content {
                ClipboardContent::Text(s) => format!("{{\"type\":\"Text\",\"value\":\"{}\"}}", s),
                ClipboardContent::Parameters(params) => {
                    let pairs: Vec<String> = params
                        .iter()
                        .map(|(k, v)| format!("[\"{}\",{:.6}]", k, v))
                        .collect();
                    format!(
                        "{{\"type\":\"Parameters\",\"value\":[{}]}}",
                        pairs.join(",")
                    )
                }
                ClipboardContent::Pose(name, vals) => {
                    let nums: Vec<String> = vals.iter().map(|v| format!("{:.6}", v)).collect();
                    format!(
                        "{{\"type\":\"Pose\",\"name\":\"{}\",\"values\":[{}]}}",
                        name,
                        nums.join(",")
                    )
                }
                ClipboardContent::Color(c) => {
                    format!(
                        "{{\"type\":\"Color\",\"rgba\":[{:.4},{:.4},{:.4},{:.4}]}}",
                        c[0], c[1], c[2], c[3]
                    )
                }
            };
            format!(
                "{{\"content\":{},\"label\":\"{}\",\"seq\":{}}}",
                content_str, entry.label, entry.sequence
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Convenience copy helpers
// ---------------------------------------------------------------------------

/// Copy a text string to the clipboard.
#[allow(dead_code)]
pub fn copy_text(cb: &mut Clipboard, text: &str) {
    copy_to_clipboard(cb, ClipboardContent::Text(text.to_string()), "text");
}

/// Copy parameter key/value pairs to the clipboard.
#[allow(dead_code)]
pub fn copy_parameters(cb: &mut Clipboard, params: &[(String, f32)]) {
    copy_to_clipboard(
        cb,
        ClipboardContent::Parameters(params.to_vec()),
        "parameters",
    );
}

/// Copy a pose to the clipboard.
#[allow(dead_code)]
pub fn copy_pose(cb: &mut Clipboard, name: &str, values: &[f32]) {
    copy_to_clipboard(
        cb,
        ClipboardContent::Pose(name.to_string(), values.to_vec()),
        "pose",
    );
}

/// Copy an RGBA color to the clipboard.
#[allow(dead_code)]
pub fn copy_color(cb: &mut Clipboard, color: [f32; 4]) {
    copy_to_clipboard(cb, ClipboardContent::Color(color), "color");
}

/// Undo the last paste by restoring the most recent history entry as current.
/// Returns `true` if an entry was restored.
#[allow(dead_code)]
pub fn undo_paste(cb: &mut Clipboard) -> bool {
    if cb.history.is_empty() {
        return false;
    }
    let restored = cb.history.remove(0);
    cb.current = Some(restored);
    true
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_clipboard_empty() {
        let cb = new_clipboard(10);
        assert!(!clipboard_has_content(&cb));
        assert_eq!(clipboard_history_count(&cb), 0);
    }

    #[test]
    fn test_copy_and_paste_text() {
        let mut cb = new_clipboard(10);
        copy_text(&mut cb, "hello");
        let content = paste_from_clipboard(&cb);
        assert_eq!(content, Some(ClipboardContent::Text("hello".to_string())));
    }

    #[test]
    fn test_clipboard_has_content() {
        let mut cb = new_clipboard(10);
        assert!(!clipboard_has_content(&cb));
        copy_text(&mut cb, "x");
        assert!(clipboard_has_content(&cb));
    }

    #[test]
    fn test_clipboard_content_type_text() {
        let mut cb = new_clipboard(10);
        copy_text(&mut cb, "abc");
        assert_eq!(clipboard_content_type(&cb), Some("Text"));
    }

    #[test]
    fn test_clipboard_content_type_color() {
        let mut cb = new_clipboard(10);
        copy_color(&mut cb, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(clipboard_content_type(&cb), Some("Color"));
    }

    #[test]
    fn test_clipboard_content_type_params() {
        let mut cb = new_clipboard(10);
        copy_parameters(&mut cb, &[("weight".to_string(), 0.5)]);
        assert_eq!(clipboard_content_type(&cb), Some("Parameters"));
    }

    #[test]
    fn test_clipboard_content_type_pose() {
        let mut cb = new_clipboard(10);
        copy_pose(&mut cb, "idle", &[0.0, 1.0, 0.0]);
        assert_eq!(clipboard_content_type(&cb), Some("Pose"));
    }

    #[test]
    fn test_clear_clipboard() {
        let mut cb = new_clipboard(10);
        copy_text(&mut cb, "data");
        clear_clipboard(&mut cb);
        assert!(!clipboard_has_content(&cb));
        assert_eq!(clipboard_history_count(&cb), 0);
    }

    #[test]
    fn test_history_builds_up() {
        let mut cb = new_clipboard(10);
        copy_text(&mut cb, "first");
        copy_text(&mut cb, "second");
        copy_text(&mut cb, "third");
        assert_eq!(clipboard_history_count(&cb), 2);
    }

    #[test]
    fn test_history_max_capacity() {
        let mut cb = new_clipboard(2);
        copy_text(&mut cb, "a");
        copy_text(&mut cb, "b");
        copy_text(&mut cb, "c");
        copy_text(&mut cb, "d");
        assert_eq!(clipboard_history_count(&cb), 2);
    }

    #[test]
    fn test_get_history_entry() {
        let mut cb = new_clipboard(10);
        copy_text(&mut cb, "first");
        copy_text(&mut cb, "second");
        let entry = get_history_entry(&cb, 0).expect("should succeed");
        assert_eq!(entry.content, ClipboardContent::Text("first".to_string()));
    }

    #[test]
    fn test_undo_paste() {
        let mut cb = new_clipboard(10);
        copy_text(&mut cb, "alpha");
        copy_text(&mut cb, "beta");
        let ok = undo_paste(&mut cb);
        assert!(ok);
        let content = paste_from_clipboard(&cb).expect("should succeed");
        assert_eq!(content, ClipboardContent::Text("alpha".to_string()));
    }

    #[test]
    fn test_undo_paste_empty() {
        let mut cb = new_clipboard(10);
        assert!(!undo_paste(&mut cb));
    }

    #[test]
    fn test_clipboard_to_json_empty() {
        let cb = new_clipboard(10);
        let json = clipboard_to_json(&cb);
        assert!(json.contains("null"));
    }

    #[test]
    fn test_clipboard_to_json_text() {
        let mut cb = new_clipboard(10);
        copy_text(&mut cb, "hello");
        let json = clipboard_to_json(&cb);
        assert!(json.contains("\"type\":\"Text\""));
        assert!(json.contains("hello"));
    }

    #[test]
    fn test_clipboard_to_json_color() {
        let mut cb = new_clipboard(10);
        copy_color(&mut cb, [1.0, 0.5, 0.25, 1.0]);
        let json = clipboard_to_json(&cb);
        assert!(json.contains("\"type\":\"Color\""));
    }

    #[test]
    fn test_paste_from_empty() {
        let cb = new_clipboard(10);
        assert_eq!(paste_from_clipboard(&cb), None);
    }
}
