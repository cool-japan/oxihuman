// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Draw list management for batched rendering.

/// Draw command type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DrawCommandType {
    Triangles,
    Lines,
    Points,
}

/// A single draw command.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DrawCommand {
    pub cmd_type: DrawCommandType,
    pub vertex_offset: u32,
    pub vertex_count: u32,
    pub index_offset: u32,
    pub index_count: u32,
    pub material_id: u32,
}

/// Draw list.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DrawList {
    pub commands: Vec<DrawCommand>,
}

/// Create a new draw command.
#[allow(dead_code)]
pub fn new_draw_command(cmd_type: DrawCommandType, vertex_count: u32, index_count: u32) -> DrawCommand {
    DrawCommand {
        cmd_type,
        vertex_offset: 0,
        vertex_count,
        index_offset: 0,
        index_count,
        material_id: 0,
    }
}

/// Create empty draw list.
#[allow(dead_code)]
pub fn new_draw_list() -> DrawList {
    DrawList { commands: Vec::new() }
}

/// Add command to list.
#[allow(dead_code)]
pub fn add_draw_command(list: &mut DrawList, cmd: DrawCommand) {
    list.commands.push(cmd);
}

/// Clear the draw list.
#[allow(dead_code)]
pub fn clear_draw_list(list: &mut DrawList) {
    list.commands.clear();
}

/// Command count.
#[allow(dead_code)]
pub fn draw_command_count(list: &DrawList) -> usize {
    list.commands.len()
}

/// Total vertex count across all commands.
#[allow(dead_code)]
pub fn total_vertices(list: &DrawList) -> u64 {
    list.commands.iter().map(|c| c.vertex_count as u64).sum()
}

/// Total index count.
#[allow(dead_code)]
pub fn total_indices(list: &DrawList) -> u64 {
    list.commands.iter().map(|c| c.index_count as u64).sum()
}

/// Sort by material for batching.
#[allow(dead_code)]
pub fn sort_by_material(list: &mut DrawList) {
    list.commands.sort_by_key(|c| c.material_id);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_command() {
        let c = new_draw_command(DrawCommandType::Triangles, 100, 300);
        assert_eq!(c.vertex_count, 100);
    }

    #[test]
    fn test_new_list() {
        let l = new_draw_list();
        assert!(l.commands.is_empty());
    }

    #[test]
    fn test_add_command() {
        let mut l = new_draw_list();
        add_draw_command(&mut l, new_draw_command(DrawCommandType::Triangles, 10, 30));
        assert_eq!(draw_command_count(&l), 1);
    }

    #[test]
    fn test_clear() {
        let mut l = new_draw_list();
        add_draw_command(&mut l, new_draw_command(DrawCommandType::Lines, 10, 0));
        clear_draw_list(&mut l);
        assert_eq!(draw_command_count(&l), 0);
    }

    #[test]
    fn test_total_vertices() {
        let mut l = new_draw_list();
        add_draw_command(&mut l, new_draw_command(DrawCommandType::Triangles, 10, 30));
        add_draw_command(&mut l, new_draw_command(DrawCommandType::Triangles, 20, 60));
        assert_eq!(total_vertices(&l), 30);
    }

    #[test]
    fn test_total_indices() {
        let mut l = new_draw_list();
        add_draw_command(&mut l, new_draw_command(DrawCommandType::Triangles, 10, 30));
        assert_eq!(total_indices(&l), 30);
    }

    #[test]
    fn test_sort_by_material() {
        let mut l = new_draw_list();
        let mut c1 = new_draw_command(DrawCommandType::Triangles, 10, 30);
        c1.material_id = 2;
        let mut c2 = new_draw_command(DrawCommandType::Triangles, 10, 30);
        c2.material_id = 1;
        add_draw_command(&mut l, c1);
        add_draw_command(&mut l, c2);
        sort_by_material(&mut l);
        assert!(l.commands[0].material_id <= l.commands[1].material_id);
    }

    #[test]
    fn test_command_type() {
        let c = new_draw_command(DrawCommandType::Points, 5, 0);
        assert_eq!(c.cmd_type, DrawCommandType::Points);
    }

    #[test]
    fn test_default_offsets() {
        let c = new_draw_command(DrawCommandType::Triangles, 10, 30);
        assert_eq!(c.vertex_offset, 0);
        assert_eq!(c.index_offset, 0);
    }

    #[test]
    fn test_empty_totals() {
        let l = new_draw_list();
        assert_eq!(total_vertices(&l), 0);
        assert_eq!(total_indices(&l), 0);
    }
}
