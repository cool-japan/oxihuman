// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Command dispatcher: name → handler index mapping.

#![allow(dead_code)]

/// A registered command entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CommandEntry {
    pub name: String,
    pub handler_id: u32,
    pub description: String,
}

/// Dispatcher that maps command names to handler ids.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct CommandDispatcher {
    pub commands: Vec<CommandEntry>,
}

/// Create a new empty dispatcher.
#[allow(dead_code)]
pub fn new_dispatcher() -> CommandDispatcher {
    CommandDispatcher::default()
}

/// Register a command with a name, handler id and description.
#[allow(dead_code)]
pub fn register_command(d: &mut CommandDispatcher, name: &str, handler_id: u32, desc: &str) {
    d.commands.push(CommandEntry {
        name: name.to_string(),
        handler_id,
        description: desc.to_string(),
    });
}

/// Find a command by name, returning a reference to its entry.
#[allow(dead_code)]
pub fn find_command<'a>(d: &'a CommandDispatcher, name: &str) -> Option<&'a CommandEntry> {
    d.commands.iter().find(|e| e.name == name)
}

/// Return the number of registered commands.
#[allow(dead_code)]
pub fn command_count(d: &CommandDispatcher) -> usize {
    d.commands.len()
}

/// Return the names of all registered commands.
#[allow(dead_code)]
pub fn list_commands(d: &CommandDispatcher) -> Vec<&str> {
    d.commands.iter().map(|e| e.name.as_str()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_dispatcher_empty() {
        let d = new_dispatcher();
        assert_eq!(command_count(&d), 0);
    }

    #[test]
    fn test_register_single_command() {
        let mut d = new_dispatcher();
        register_command(&mut d, "render", 1, "Render the scene");
        assert_eq!(command_count(&d), 1);
    }

    #[test]
    fn test_find_existing_command() {
        let mut d = new_dispatcher();
        register_command(&mut d, "save", 42, "Save the file");
        let entry = find_command(&d, "save");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().handler_id, 42);
    }

    #[test]
    fn test_find_missing_command() {
        let d = new_dispatcher();
        assert!(find_command(&d, "nonexistent").is_none());
    }

    #[test]
    fn test_list_commands_empty() {
        let d = new_dispatcher();
        assert!(list_commands(&d).is_empty());
    }

    #[test]
    fn test_list_commands_multiple() {
        let mut d = new_dispatcher();
        register_command(&mut d, "a", 1, "");
        register_command(&mut d, "b", 2, "");
        let names = list_commands(&d);
        assert!(names.contains(&"a"));
        assert!(names.contains(&"b"));
    }

    #[test]
    fn test_command_description_stored() {
        let mut d = new_dispatcher();
        register_command(&mut d, "undo", 5, "Undo last action");
        let e = find_command(&d, "undo").unwrap();
        assert_eq!(e.description, "Undo last action");
    }

    #[test]
    fn test_multiple_commands_distinct() {
        let mut d = new_dispatcher();
        register_command(&mut d, "x", 10, "X");
        register_command(&mut d, "y", 20, "Y");
        assert_eq!(find_command(&d, "x").unwrap().handler_id, 10);
        assert_eq!(find_command(&d, "y").unwrap().handler_id, 20);
    }

    #[test]
    fn test_command_count_increments() {
        let mut d = new_dispatcher();
        for i in 0..5 {
            register_command(&mut d, &format!("cmd{i}"), i as u32, "");
        }
        assert_eq!(command_count(&d), 5);
    }
}
