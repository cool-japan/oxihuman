// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Indirect draw command buffer management for GPU-driven rendering.

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct DrawIndirectCommand {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub first_instance: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct DrawIndexedIndirectCommand {
    pub index_count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub vertex_offset: i32,
    pub first_instance: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IndirectBuffer {
    pub commands: Vec<DrawIndirectCommand>,
    pub max_commands: usize,
}

#[allow(dead_code)]
pub fn new_indirect_buffer(max: usize) -> IndirectBuffer {
    IndirectBuffer {
        commands: Vec::with_capacity(max),
        max_commands: max,
    }
}

#[allow(dead_code)]
pub fn add_draw_command(buf: &mut IndirectBuffer, cmd: DrawIndirectCommand) -> bool {
    if buf.commands.len() >= buf.max_commands {
        return false;
    }
    buf.commands.push(cmd);
    true
}

#[allow(dead_code)]
pub fn clear_indirect_buffer(buf: &mut IndirectBuffer) {
    buf.commands.clear();
}

#[allow(dead_code)]
pub fn command_count(buf: &IndirectBuffer) -> usize {
    buf.commands.len()
}

#[allow(dead_code)]
pub fn total_vertex_count(buf: &IndirectBuffer) -> u64 {
    buf.commands.iter().map(|c| c.vertex_count as u64 * c.instance_count as u64).sum()
}

#[allow(dead_code)]
pub fn total_instance_count(buf: &IndirectBuffer) -> u64 {
    buf.commands.iter().map(|c| c.instance_count as u64).sum()
}

#[allow(dead_code)]
pub fn new_draw_command(vertex_count: u32, instance_count: u32) -> DrawIndirectCommand {
    DrawIndirectCommand {
        vertex_count,
        instance_count,
        first_vertex: 0,
        first_instance: 0,
    }
}

#[allow(dead_code)]
pub fn new_indexed_command(index_count: u32, instance_count: u32) -> DrawIndexedIndirectCommand {
    DrawIndexedIndirectCommand {
        index_count,
        instance_count,
        first_index: 0,
        vertex_offset: 0,
        first_instance: 0,
    }
}

#[allow(dead_code)]
pub fn indirect_buffer_to_json(buf: &IndirectBuffer) -> String {
    format!(
        r#"{{"count":{},"max":{},"total_verts":{}}}"#,
        command_count(buf), buf.max_commands, total_vertex_count(buf)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer() {
        let b = new_indirect_buffer(16);
        assert_eq!(b.max_commands, 16);
        assert!(b.commands.is_empty());
    }

    #[test]
    fn test_add_command() {
        let mut b = new_indirect_buffer(10);
        let cmd = new_draw_command(36, 1);
        assert!(add_draw_command(&mut b, cmd));
        assert_eq!(command_count(&b), 1);
    }

    #[test]
    fn test_add_over_limit() {
        let mut b = new_indirect_buffer(1);
        assert!(add_draw_command(&mut b, new_draw_command(3, 1)));
        assert!(!add_draw_command(&mut b, new_draw_command(3, 1)));
    }

    #[test]
    fn test_clear() {
        let mut b = new_indirect_buffer(10);
        add_draw_command(&mut b, new_draw_command(3, 1));
        clear_indirect_buffer(&mut b);
        assert!(b.commands.is_empty());
    }

    #[test]
    fn test_total_vertex_count() {
        let mut b = new_indirect_buffer(10);
        add_draw_command(&mut b, new_draw_command(100, 2));
        add_draw_command(&mut b, new_draw_command(50, 3));
        assert_eq!(total_vertex_count(&b), 350);
    }

    #[test]
    fn test_total_instance_count() {
        let mut b = new_indirect_buffer(10);
        add_draw_command(&mut b, new_draw_command(10, 5));
        add_draw_command(&mut b, new_draw_command(10, 3));
        assert_eq!(total_instance_count(&b), 8);
    }

    #[test]
    fn test_new_indexed() {
        let cmd = new_indexed_command(36, 4);
        assert_eq!(cmd.index_count, 36);
        assert_eq!(cmd.instance_count, 4);
    }

    #[test]
    fn test_to_json() {
        let b = new_indirect_buffer(10);
        let j = indirect_buffer_to_json(&b);
        assert!(j.contains("count"));
    }

    #[test]
    fn test_empty_totals() {
        let b = new_indirect_buffer(10);
        assert_eq!(total_vertex_count(&b), 0);
        assert_eq!(total_instance_count(&b), 0);
    }

    #[test]
    fn test_draw_command_fields() {
        let cmd = DrawIndirectCommand {
            vertex_count: 12,
            instance_count: 1,
            first_vertex: 5,
            first_instance: 0,
        };
        assert_eq!(cmd.first_vertex, 5);
    }
}
