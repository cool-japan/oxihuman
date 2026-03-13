// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! CommandBuffer — GPU command buffer abstraction (stub).

#![allow(dead_code)]

/// Types of render commands.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum RenderCommand {
    Draw { vertex_count: u32 },
    DrawIndexed { index_count: u32 },
    SetPipeline { pipeline_id: u32 },
    SetBindGroup { group: u32 },
    CopyBuffer { src: u32, dst: u32, size: u32 },
}

/// A buffer of render commands.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct CommandBuffer {
    commands: Vec<RenderCommand>,
}

/// Create a new empty command buffer.
#[allow(dead_code)]
pub fn new_command_buffer() -> CommandBuffer {
    CommandBuffer::default()
}

/// Push a command onto the buffer.
#[allow(dead_code)]
pub fn push_command(buf: &mut CommandBuffer, cmd: RenderCommand) {
    buf.commands.push(cmd);
}

/// Number of commands in the buffer.
#[allow(dead_code)]
pub fn command_count(buf: &CommandBuffer) -> usize {
    buf.commands.len()
}

/// Execute all commands (stub — returns number executed).
#[allow(dead_code)]
pub fn execute_commands_stub(buf: &CommandBuffer) -> usize {
    buf.commands.len()
}

/// Get a command at an index.
#[allow(dead_code)]
pub fn command_at(buf: &CommandBuffer, index: usize) -> Option<&RenderCommand> {
    buf.commands.get(index)
}

/// Clear all commands.
#[allow(dead_code)]
pub fn clear_commands(buf: &mut CommandBuffer) {
    buf.commands.clear();
}

/// Estimated byte size of the command buffer.
#[allow(dead_code)]
pub fn command_buffer_size(buf: &CommandBuffer) -> usize {
    buf.commands.len() * std::mem::size_of::<RenderCommand>()
}

/// Return a human-readable name for the command type.
#[allow(dead_code)]
pub fn command_type_name(cmd: &RenderCommand) -> &'static str {
    match cmd {
        RenderCommand::Draw { .. } => "Draw",
        RenderCommand::DrawIndexed { .. } => "DrawIndexed",
        RenderCommand::SetPipeline { .. } => "SetPipeline",
        RenderCommand::SetBindGroup { .. } => "SetBindGroup",
        RenderCommand::CopyBuffer { .. } => "CopyBuffer",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_command_buffer() {
        let buf = new_command_buffer();
        assert_eq!(command_count(&buf), 0);
    }

    #[test]
    fn test_push_command() {
        let mut buf = new_command_buffer();
        push_command(&mut buf, RenderCommand::Draw { vertex_count: 3 });
        assert_eq!(command_count(&buf), 1);
    }

    #[test]
    fn test_command_count() {
        let mut buf = new_command_buffer();
        push_command(&mut buf, RenderCommand::Draw { vertex_count: 3 });
        push_command(&mut buf, RenderCommand::SetPipeline { pipeline_id: 1 });
        assert_eq!(command_count(&buf), 2);
    }

    #[test]
    fn test_execute_commands_stub() {
        let mut buf = new_command_buffer();
        push_command(&mut buf, RenderCommand::Draw { vertex_count: 3 });
        assert_eq!(execute_commands_stub(&buf), 1);
    }

    #[test]
    fn test_command_at() {
        let mut buf = new_command_buffer();
        push_command(&mut buf, RenderCommand::Draw { vertex_count: 42 });
        let cmd = command_at(&buf, 0).expect("should succeed");
        assert_eq!(cmd, &RenderCommand::Draw { vertex_count: 42 });
    }

    #[test]
    fn test_command_at_none() {
        let buf = new_command_buffer();
        assert!(command_at(&buf, 0).is_none());
    }

    #[test]
    fn test_clear_commands() {
        let mut buf = new_command_buffer();
        push_command(&mut buf, RenderCommand::Draw { vertex_count: 3 });
        clear_commands(&mut buf);
        assert_eq!(command_count(&buf), 0);
    }

    #[test]
    fn test_command_buffer_size() {
        let mut buf = new_command_buffer();
        push_command(&mut buf, RenderCommand::Draw { vertex_count: 3 });
        assert!(command_buffer_size(&buf) > 0);
    }

    #[test]
    fn test_command_type_name_draw() {
        let cmd = RenderCommand::Draw { vertex_count: 1 };
        assert_eq!(command_type_name(&cmd), "Draw");
    }

    #[test]
    fn test_command_type_name_copy() {
        let cmd = RenderCommand::CopyBuffer { src: 0, dst: 1, size: 64 };
        assert_eq!(command_type_name(&cmd), "CopyBuffer");
    }
}
