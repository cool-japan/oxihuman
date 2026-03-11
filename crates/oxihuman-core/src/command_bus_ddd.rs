// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct DddCommand {
    pub name: String,
    pub payload: String,
}

pub struct CommandLog {
    pub commands: Vec<DddCommand>,
}

pub fn new_ddd_command(name: &str, payload: &str) -> DddCommand {
    DddCommand {
        name: name.to_string(),
        payload: payload.to_string(),
    }
}

pub fn new_command_log() -> CommandLog {
    CommandLog {
        commands: Vec::new(),
    }
}

pub fn log_dispatch(l: &mut CommandLog, cmd: DddCommand) {
    l.commands.push(cmd);
}

pub fn log_command_count(l: &CommandLog) -> usize {
    l.commands.len()
}

pub fn log_commands_by_name<'a>(l: &'a CommandLog, name: &str) -> Vec<&'a DddCommand> {
    l.commands.iter().filter(|c| c.name == name).collect()
}

pub fn log_clear(l: &mut CommandLog) {
    l.commands.clear();
}

pub fn log_last_command(l: &CommandLog) -> Option<&DddCommand> {
    l.commands.last()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatch_and_count() {
        /* dispatch adds command and increments count */
        let mut l = new_command_log();
        log_dispatch(&mut l, new_ddd_command("Create", "{}"));
        assert_eq!(log_command_count(&l), 1);
    }

    #[test]
    fn test_commands_by_name() {
        /* filter commands by name */
        let mut l = new_command_log();
        log_dispatch(&mut l, new_ddd_command("Create", "a"));
        log_dispatch(&mut l, new_ddd_command("Delete", "b"));
        log_dispatch(&mut l, new_ddd_command("Create", "c"));
        assert_eq!(log_commands_by_name(&l, "Create").len(), 2);
    }

    #[test]
    fn test_log_clear() {
        /* clear removes all commands */
        let mut l = new_command_log();
        log_dispatch(&mut l, new_ddd_command("X", ""));
        log_clear(&mut l);
        assert_eq!(log_command_count(&l), 0);
    }

    #[test]
    fn test_log_last_command() {
        /* last command returned correctly */
        let mut l = new_command_log();
        log_dispatch(&mut l, new_ddd_command("First", ""));
        log_dispatch(&mut l, new_ddd_command("Last", ""));
        let last = log_last_command(&l).unwrap();
        assert_eq!(last.name, "Last");
    }

    #[test]
    fn test_empty_log() {
        /* empty log has no last command */
        let l = new_command_log();
        assert!(log_last_command(&l).is_none());
    }

    #[test]
    fn test_new_command_fields() {
        /* command fields set correctly */
        let cmd = new_ddd_command("Update", "data");
        assert_eq!(cmd.name, "Update");
        assert_eq!(cmd.payload, "data");
    }

    #[test]
    fn test_commands_by_name_none() {
        /* no match returns empty */
        let l = new_command_log();
        assert!(log_commands_by_name(&l, "X").is_empty());
    }
}
