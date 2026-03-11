#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Command {
    name: String,
    executed: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CommandInvoker {
    history: Vec<Command>,
    undo_stack: Vec<Command>,
    redo_stack: Vec<Command>,
}

#[allow(dead_code)]
pub fn new_command_invoker() -> CommandInvoker {
    CommandInvoker {
        history: Vec::new(),
        undo_stack: Vec::new(),
        redo_stack: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn execute_command(inv: &mut CommandInvoker, name: &str) {
    let cmd = Command {
        name: name.to_string(),
        executed: true,
    };
    inv.undo_stack.push(cmd.clone());
    inv.history.push(cmd);
    inv.redo_stack.clear();
}

#[allow(dead_code)]
pub fn undo_command(inv: &mut CommandInvoker) -> bool {
    if let Some(mut cmd) = inv.undo_stack.pop() {
        cmd.executed = false;
        inv.redo_stack.push(cmd);
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn redo_command(inv: &mut CommandInvoker) -> bool {
    if let Some(mut cmd) = inv.redo_stack.pop() {
        cmd.executed = true;
        inv.undo_stack.push(cmd);
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn command_count(inv: &CommandInvoker) -> usize {
    inv.history.len()
}

#[allow(dead_code)]
pub fn can_undo_cmd(inv: &CommandInvoker) -> bool {
    !inv.undo_stack.is_empty()
}

#[allow(dead_code)]
pub fn can_redo_cmd(inv: &CommandInvoker) -> bool {
    !inv.redo_stack.is_empty()
}

#[allow(dead_code)]
pub fn invoker_clear(inv: &mut CommandInvoker) {
    inv.history.clear();
    inv.undo_stack.clear();
    inv.redo_stack.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_invoker() {
        let inv = new_command_invoker();
        assert_eq!(command_count(&inv), 0);
    }

    #[test]
    fn test_execute() {
        let mut inv = new_command_invoker();
        execute_command(&mut inv, "move");
        assert_eq!(command_count(&inv), 1);
    }

    #[test]
    fn test_undo() {
        let mut inv = new_command_invoker();
        execute_command(&mut inv, "scale");
        assert!(undo_command(&mut inv));
        assert!(!can_undo_cmd(&inv));
    }

    #[test]
    fn test_redo() {
        let mut inv = new_command_invoker();
        execute_command(&mut inv, "rotate");
        undo_command(&mut inv);
        assert!(redo_command(&mut inv));
    }

    #[test]
    fn test_undo_empty() {
        let mut inv = new_command_invoker();
        assert!(!undo_command(&mut inv));
    }

    #[test]
    fn test_redo_empty() {
        let mut inv = new_command_invoker();
        assert!(!redo_command(&mut inv));
    }

    #[test]
    fn test_can_undo() {
        let mut inv = new_command_invoker();
        assert!(!can_undo_cmd(&inv));
        execute_command(&mut inv, "delete");
        assert!(can_undo_cmd(&inv));
    }

    #[test]
    fn test_can_redo() {
        let mut inv = new_command_invoker();
        execute_command(&mut inv, "insert");
        assert!(!can_redo_cmd(&inv));
        undo_command(&mut inv);
        assert!(can_redo_cmd(&inv));
    }

    #[test]
    fn test_clear() {
        let mut inv = new_command_invoker();
        execute_command(&mut inv, "a");
        execute_command(&mut inv, "b");
        invoker_clear(&mut inv);
        assert_eq!(command_count(&inv), 0);
        assert!(!can_undo_cmd(&inv));
    }

    #[test]
    fn test_execute_clears_redo() {
        let mut inv = new_command_invoker();
        execute_command(&mut inv, "x");
        undo_command(&mut inv);
        execute_command(&mut inv, "y");
        assert!(!can_redo_cmd(&inv));
    }
}
