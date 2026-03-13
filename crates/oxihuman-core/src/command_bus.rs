// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

use std::collections::HashMap;

// ─── Core types ───────────────────────────────────────────────────────────────

/// Result returned by command execution or undo.
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
}

/// Shared mutable state that commands operate on.
#[derive(Debug, Clone)]
pub struct CommandState {
    pub params: HashMap<String, f64>,
    pub flags: HashMap<String, bool>,
    /// Log of executed command descriptions.
    pub history: Vec<String>,
}

/// A reversible command that operates on `CommandState`.
pub trait Command: std::fmt::Debug {
    fn execute(&self, state: &mut CommandState) -> CommandResult;
    fn undo(&self, state: &mut CommandState) -> CommandResult;
    fn description(&self) -> &str;
}

/// The command bus with undo/redo stacks.
pub struct CommandBus {
    pub undo_stack: Vec<Box<dyn Command>>,
    pub redo_stack: Vec<Box<dyn Command>>,
    pub max_history: usize,
    pub state: CommandState,
}

// ─── Built-in commands ────────────────────────────────────────────────────────

/// Set a numeric parameter, recording the previous value for undo.
#[derive(Debug)]
pub struct SetParamCommand {
    pub key: String,
    pub value: f64,
    pub old_value: f64,
}

impl Command for SetParamCommand {
    fn execute(&self, state: &mut CommandState) -> CommandResult {
        state.params.insert(self.key.clone(), self.value);
        CommandResult {
            success: true,
            message: format!("set {} = {}", self.key, self.value),
        }
    }

    fn undo(&self, state: &mut CommandState) -> CommandResult {
        state.params.insert(self.key.clone(), self.old_value);
        CommandResult {
            success: true,
            message: format!("undo set {} -> {}", self.key, self.old_value),
        }
    }

    fn description(&self) -> &str {
        "SetParamCommand"
    }
}

/// Set a boolean flag, recording the previous value for undo.
#[derive(Debug)]
pub struct SetFlagCommand {
    pub key: String,
    pub value: bool,
    pub old_value: bool,
}

impl Command for SetFlagCommand {
    fn execute(&self, state: &mut CommandState) -> CommandResult {
        state.flags.insert(self.key.clone(), self.value);
        CommandResult {
            success: true,
            message: format!("set flag {} = {}", self.key, self.value),
        }
    }

    fn undo(&self, state: &mut CommandState) -> CommandResult {
        state.flags.insert(self.key.clone(), self.old_value);
        CommandResult {
            success: true,
            message: format!("undo flag {} -> {}", self.key, self.old_value),
        }
    }

    fn description(&self) -> &str {
        "SetFlagCommand"
    }
}

/// Execute multiple commands atomically (undo reverses all in reverse order).
#[derive(Debug)]
pub struct BatchCommand {
    pub commands: Vec<Box<dyn Command>>,
    pub name: String,
}

impl Command for BatchCommand {
    fn execute(&self, state: &mut CommandState) -> CommandResult {
        let mut all_ok = true;
        let mut msgs = Vec::new();
        for cmd in &self.commands {
            let r = cmd.execute(state);
            if !r.success {
                all_ok = false;
            }
            msgs.push(r.message);
        }
        CommandResult {
            success: all_ok,
            message: msgs.join("; "),
        }
    }

    fn undo(&self, state: &mut CommandState) -> CommandResult {
        let mut all_ok = true;
        let mut msgs = Vec::new();
        for cmd in self.commands.iter().rev() {
            let r = cmd.undo(state);
            if !r.success {
                all_ok = false;
            }
            msgs.push(r.message);
        }
        CommandResult {
            success: all_ok,
            message: msgs.join("; "),
        }
    }

    fn description(&self) -> &str {
        &self.name
    }
}

// ─── Construction ─────────────────────────────────────────────────────────────

pub fn new_command_state() -> CommandState {
    CommandState {
        params: HashMap::new(),
        flags: HashMap::new(),
        history: Vec::new(),
    }
}

pub fn new_command_bus(max_history: usize) -> CommandBus {
    CommandBus {
        undo_stack: Vec::new(),
        redo_stack: Vec::new(),
        max_history,
        state: new_command_state(),
    }
}

// ─── Operations ───────────────────────────────────────────────────────────────

/// Execute a command and push it onto the undo stack.
/// Clears the redo stack (standard undo/redo semantics).
pub fn execute_command(bus: &mut CommandBus, cmd: Box<dyn Command>) -> CommandResult {
    let result = cmd.execute(&mut bus.state);
    if result.success {
        bus.state.history.push(cmd.description().to_string());
        bus.redo_stack.clear();
        bus.undo_stack.push(cmd);
        // Trim undo stack to max_history
        if bus.undo_stack.len() > bus.max_history {
            bus.undo_stack.remove(0);
        }
    }
    result
}

/// Undo the last command and push it onto the redo stack.
pub fn undo_last(bus: &mut CommandBus) -> Option<CommandResult> {
    let cmd = bus.undo_stack.pop()?;
    let result = cmd.undo(&mut bus.state);
    bus.redo_stack.push(cmd);
    Some(result)
}

/// Redo the last undone command and push it back onto the undo stack.
pub fn redo_last(bus: &mut CommandBus) -> Option<CommandResult> {
    let cmd = bus.redo_stack.pop()?;
    let result = cmd.execute(&mut bus.state);
    bus.undo_stack.push(cmd);
    Some(result)
}

pub fn undo_count(bus: &CommandBus) -> usize {
    bus.undo_stack.len()
}

pub fn redo_count(bus: &CommandBus) -> usize {
    bus.redo_stack.len()
}

pub fn clear_history(bus: &mut CommandBus) {
    bus.undo_stack.clear();
    bus.redo_stack.clear();
}

/// Return descriptions of commands in the undo stack (oldest first).
pub fn command_descriptions(bus: &CommandBus) -> Vec<&str> {
    bus.undo_stack.iter().map(|c| c.description()).collect()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_bus_empty() {
        let bus = new_command_bus(10);
        assert_eq!(undo_count(&bus), 0);
        assert_eq!(redo_count(&bus), 0);
    }

    #[test]
    fn test_execute_set_param() {
        let mut bus = new_command_bus(10);
        let cmd = Box::new(SetParamCommand {
            key: "height".to_string(),
            value: 1.75,
            old_value: 0.0,
        });
        let result = execute_command(&mut bus, cmd);
        assert!(result.success);
        assert_eq!(
            *bus.state.params.get("height").expect("should succeed"),
            1.75
        );
    }

    #[test]
    fn test_undo_set_param() {
        let mut bus = new_command_bus(10);
        let cmd = Box::new(SetParamCommand {
            key: "age".to_string(),
            value: 30.0,
            old_value: 25.0,
        });
        execute_command(&mut bus, cmd);
        let r = undo_last(&mut bus).expect("should succeed");
        assert!(r.success);
        assert_eq!(*bus.state.params.get("age").expect("should succeed"), 25.0);
    }

    #[test]
    fn test_redo_after_undo() {
        let mut bus = new_command_bus(10);
        let cmd = Box::new(SetParamCommand {
            key: "x".to_string(),
            value: 5.0,
            old_value: 0.0,
        });
        execute_command(&mut bus, cmd);
        undo_last(&mut bus);
        let r = redo_last(&mut bus).expect("should succeed");
        assert!(r.success);
        assert_eq!(*bus.state.params.get("x").expect("should succeed"), 5.0);
    }

    #[test]
    fn test_redo_cleared_on_new_command() {
        let mut bus = new_command_bus(10);
        execute_command(
            &mut bus,
            Box::new(SetParamCommand {
                key: "a".to_string(),
                value: 1.0,
                old_value: 0.0,
            }),
        );
        undo_last(&mut bus);
        assert_eq!(redo_count(&bus), 1);
        execute_command(
            &mut bus,
            Box::new(SetParamCommand {
                key: "b".to_string(),
                value: 2.0,
                old_value: 0.0,
            }),
        );
        assert_eq!(redo_count(&bus), 0);
    }

    #[test]
    fn test_set_flag_command() {
        let mut bus = new_command_bus(10);
        let cmd = Box::new(SetFlagCommand {
            key: "visible".to_string(),
            value: true,
            old_value: false,
        });
        execute_command(&mut bus, cmd);
        assert!(*bus.state.flags.get("visible").expect("should succeed"));
    }

    #[test]
    fn test_undo_set_flag() {
        let mut bus = new_command_bus(10);
        execute_command(
            &mut bus,
            Box::new(SetFlagCommand {
                key: "f".to_string(),
                value: true,
                old_value: false,
            }),
        );
        undo_last(&mut bus);
        assert!(!*bus.state.flags.get("f").expect("should succeed"));
    }

    #[test]
    fn test_batch_command() {
        let mut bus = new_command_bus(10);
        let batch = Box::new(BatchCommand {
            name: "set_both".to_string(),
            commands: vec![
                Box::new(SetParamCommand {
                    key: "p1".to_string(),
                    value: 1.0,
                    old_value: 0.0,
                }),
                Box::new(SetParamCommand {
                    key: "p2".to_string(),
                    value: 2.0,
                    old_value: 0.0,
                }),
            ],
        });
        let r = execute_command(&mut bus, batch);
        assert!(r.success);
        assert_eq!(*bus.state.params.get("p1").expect("should succeed"), 1.0);
        assert_eq!(*bus.state.params.get("p2").expect("should succeed"), 2.0);
    }

    #[test]
    fn test_batch_undo() {
        let mut bus = new_command_bus(10);
        execute_command(
            &mut bus,
            Box::new(BatchCommand {
                name: "batch".to_string(),
                commands: vec![Box::new(SetParamCommand {
                    key: "q".to_string(),
                    value: 9.0,
                    old_value: 1.0,
                })],
            }),
        );
        undo_last(&mut bus);
        assert_eq!(*bus.state.params.get("q").expect("should succeed"), 1.0);
    }

    #[test]
    fn test_clear_history() {
        let mut bus = new_command_bus(10);
        execute_command(
            &mut bus,
            Box::new(SetParamCommand {
                key: "x".to_string(),
                value: 1.0,
                old_value: 0.0,
            }),
        );
        clear_history(&mut bus);
        assert_eq!(undo_count(&bus), 0);
        assert_eq!(redo_count(&bus), 0);
    }

    #[test]
    fn test_command_descriptions() {
        let mut bus = new_command_bus(10);
        execute_command(
            &mut bus,
            Box::new(SetParamCommand {
                key: "x".to_string(),
                value: 1.0,
                old_value: 0.0,
            }),
        );
        execute_command(
            &mut bus,
            Box::new(SetFlagCommand {
                key: "f".to_string(),
                value: true,
                old_value: false,
            }),
        );
        let descs = command_descriptions(&bus);
        assert_eq!(descs.len(), 2);
        assert!(descs.contains(&"SetParamCommand"));
        assert!(descs.contains(&"SetFlagCommand"));
    }

    #[test]
    fn test_undo_empty_returns_none() {
        let mut bus = new_command_bus(10);
        assert!(undo_last(&mut bus).is_none());
    }

    #[test]
    fn test_redo_empty_returns_none() {
        let mut bus = new_command_bus(10);
        assert!(redo_last(&mut bus).is_none());
    }

    #[test]
    fn test_max_history_trimmed() {
        let mut bus = new_command_bus(3);
        for i in 0..5 {
            execute_command(
                &mut bus,
                Box::new(SetParamCommand {
                    key: format!("p{}", i),
                    value: i as f64,
                    old_value: 0.0,
                }),
            );
        }
        assert_eq!(undo_count(&bus), 3);
    }

    #[test]
    fn test_state_history_log() {
        let mut bus = new_command_bus(10);
        execute_command(
            &mut bus,
            Box::new(SetParamCommand {
                key: "k".to_string(),
                value: 1.0,
                old_value: 0.0,
            }),
        );
        assert_eq!(bus.state.history.len(), 1);
        assert_eq!(bus.state.history[0], "SetParamCommand");
    }
}
