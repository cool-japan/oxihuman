// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Generic undo/redo command history stack.

#[allow(dead_code)]
pub struct UndoCommand {
    pub id: u64,
    pub name: String,
    pub data: Vec<u8>,
    pub after: Vec<u8>,
}

#[allow(dead_code)]
pub struct UndoStack {
    pub history: Vec<UndoCommand>,
    pub future: Vec<UndoCommand>,
    pub max_depth: usize,
    pub next_id: u64,
}

#[allow(dead_code)]
pub fn new_undo_stack(max_depth: usize) -> UndoStack {
    UndoStack {
        history: Vec::new(),
        future: Vec::new(),
        max_depth,
        next_id: 0,
    }
}

#[allow(dead_code)]
pub fn push_command(stack: &mut UndoStack, name: &str, before: Vec<u8>, after: Vec<u8>) {
    stack.future.clear();
    let id = stack.next_id;
    stack.next_id += 1;
    stack.history.push(UndoCommand {
        id,
        name: name.to_string(),
        data: before,
        after,
    });
    if stack.max_depth > 0 && stack.history.len() > stack.max_depth {
        let overflow = stack.history.len() - stack.max_depth;
        stack.history.drain(0..overflow);
    }
}

#[allow(dead_code)]
pub fn undo(stack: &mut UndoStack) -> Option<&UndoCommand> {
    let cmd = stack.history.pop()?;
    stack.future.push(cmd);
    stack.future.last()
}

#[allow(dead_code)]
pub fn redo(stack: &mut UndoStack) -> Option<&UndoCommand> {
    let cmd = stack.future.pop()?;
    stack.history.push(cmd);
    stack.history.last()
}

#[allow(dead_code)]
pub fn can_undo(stack: &UndoStack) -> bool {
    !stack.history.is_empty()
}

#[allow(dead_code)]
pub fn can_redo(stack: &UndoStack) -> bool {
    !stack.future.is_empty()
}

#[allow(dead_code)]
pub fn clear_undo_history(stack: &mut UndoStack) {
    stack.history.clear();
    stack.future.clear();
}

#[allow(dead_code)]
pub fn history_depth(stack: &UndoStack) -> usize {
    stack.history.len()
}

#[allow(dead_code)]
pub fn future_depth(stack: &UndoStack) -> usize {
    stack.future.len()
}

#[allow(dead_code)]
pub fn peek_undo(stack: &UndoStack) -> Option<&UndoCommand> {
    stack.history.last()
}

#[allow(dead_code)]
pub fn peek_redo(stack: &UndoStack) -> Option<&UndoCommand> {
    stack.future.last()
}

#[allow(dead_code)]
pub fn command_names(stack: &UndoStack) -> Vec<&str> {
    stack.history.iter().map(|c| c.name.as_str()).collect()
}

#[allow(dead_code)]
pub fn truncate_history(stack: &mut UndoStack, keep: usize) {
    if stack.history.len() > keep {
        let drain_count = stack.history.len() - keep;
        stack.history.drain(0..drain_count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_then_undo_returns_command() {
        let mut stack = new_undo_stack(10);
        push_command(&mut stack, "move", vec![1], vec![2]);
        let cmd = undo(&mut stack);
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap().name, "move");
    }

    #[test]
    fn test_redo_after_undo() {
        let mut stack = new_undo_stack(10);
        push_command(&mut stack, "action", vec![0], vec![1]);
        undo(&mut stack);
        let redone = redo(&mut stack);
        assert!(redone.is_some());
        assert_eq!(redone.unwrap().name, "action");
    }

    #[test]
    fn test_can_undo_after_push() {
        let mut stack = new_undo_stack(10);
        assert!(!can_undo(&stack));
        push_command(&mut stack, "a", vec![], vec![]);
        assert!(can_undo(&stack));
    }

    #[test]
    fn test_can_redo_after_undo() {
        let mut stack = new_undo_stack(10);
        push_command(&mut stack, "a", vec![], vec![]);
        assert!(!can_redo(&stack));
        undo(&mut stack);
        assert!(can_redo(&stack));
    }

    #[test]
    fn test_clear_history() {
        let mut stack = new_undo_stack(10);
        push_command(&mut stack, "a", vec![], vec![]);
        push_command(&mut stack, "b", vec![], vec![]);
        clear_undo_history(&mut stack);
        assert!(!can_undo(&stack));
        assert!(!can_redo(&stack));
    }

    #[test]
    fn test_max_depth_enforced() {
        let mut stack = new_undo_stack(3);
        push_command(&mut stack, "a", vec![], vec![]);
        push_command(&mut stack, "b", vec![], vec![]);
        push_command(&mut stack, "c", vec![], vec![]);
        push_command(&mut stack, "d", vec![], vec![]);
        assert_eq!(history_depth(&stack), 3);
    }

    #[test]
    fn test_redo_cleared_on_new_push() {
        let mut stack = new_undo_stack(10);
        push_command(&mut stack, "a", vec![], vec![]);
        undo(&mut stack);
        assert!(can_redo(&stack));
        push_command(&mut stack, "b", vec![], vec![]);
        assert!(!can_redo(&stack));
    }

    #[test]
    fn test_peek_undo_does_not_pop() {
        let mut stack = new_undo_stack(10);
        push_command(&mut stack, "peek_test", vec![], vec![]);
        let _ = peek_undo(&stack);
        assert!(can_undo(&stack));
        assert_eq!(history_depth(&stack), 1);
    }

    #[test]
    fn test_peek_redo_does_not_pop() {
        let mut stack = new_undo_stack(10);
        push_command(&mut stack, "a", vec![], vec![]);
        undo(&mut stack);
        let _ = peek_redo(&stack);
        assert!(can_redo(&stack));
        assert_eq!(future_depth(&stack), 1);
    }

    #[test]
    fn test_history_depth() {
        let mut stack = new_undo_stack(10);
        assert_eq!(history_depth(&stack), 0);
        push_command(&mut stack, "a", vec![], vec![]);
        assert_eq!(history_depth(&stack), 1);
    }

    #[test]
    fn test_future_depth() {
        let mut stack = new_undo_stack(10);
        push_command(&mut stack, "a", vec![], vec![]);
        push_command(&mut stack, "b", vec![], vec![]);
        undo(&mut stack);
        assert_eq!(future_depth(&stack), 1);
    }

    #[test]
    fn test_command_names() {
        let mut stack = new_undo_stack(10);
        push_command(&mut stack, "first", vec![], vec![]);
        push_command(&mut stack, "second", vec![], vec![]);
        let names = command_names(&stack);
        assert_eq!(names, vec!["first", "second"]);
    }

    #[test]
    fn test_truncate_history() {
        let mut stack = new_undo_stack(10);
        push_command(&mut stack, "a", vec![], vec![]);
        push_command(&mut stack, "b", vec![], vec![]);
        push_command(&mut stack, "c", vec![], vec![]);
        truncate_history(&mut stack, 2);
        assert_eq!(history_depth(&stack), 2);
    }
}
