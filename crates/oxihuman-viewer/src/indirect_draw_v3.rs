// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Indirect draw command buffer.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IndirectDrawCmdV3 {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub first_instance: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IndirectDrawBufferV3 {
    pub commands: Vec<IndirectDrawCmdV3>,
}

#[allow(dead_code)]
pub fn new_indirect_draw_buffer_v3() -> IndirectDrawBufferV3 {
    IndirectDrawBufferV3 { commands: Vec::new() }
}

#[allow(clippy::too_many_arguments)]
#[allow(dead_code)]
pub fn idbv3_add_cmd(
    buf: &mut IndirectDrawBufferV3,
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
) {
    buf.commands.push(IndirectDrawCmdV3 { vertex_count, instance_count, first_vertex, first_instance });
}

#[allow(dead_code)]
pub fn idbv3_cmd_count(buf: &IndirectDrawBufferV3) -> usize {
    buf.commands.len()
}

#[allow(dead_code)]
pub fn idbv3_total_vertices(buf: &IndirectDrawBufferV3) -> u32 {
    buf.commands.iter().map(|c| c.vertex_count).sum()
}

#[allow(dead_code)]
pub fn idbv3_total_instances(buf: &IndirectDrawBufferV3) -> u32 {
    buf.commands.iter().map(|c| c.instance_count).sum()
}

#[allow(dead_code)]
pub fn idbv3_clear(buf: &mut IndirectDrawBufferV3) {
    buf.commands.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_cmd() {
        let mut b = new_indirect_draw_buffer_v3();
        idbv3_add_cmd(&mut b, 100, 1, 0, 0);
        assert_eq!(idbv3_cmd_count(&b), 1);
    }

    #[test]
    fn test_cmd_count_empty() {
        let b = new_indirect_draw_buffer_v3();
        assert_eq!(idbv3_cmd_count(&b), 0);
    }

    #[test]
    fn test_total_vertices() {
        let mut b = new_indirect_draw_buffer_v3();
        idbv3_add_cmd(&mut b, 100, 1, 0, 0);
        idbv3_add_cmd(&mut b, 200, 1, 0, 0);
        assert_eq!(idbv3_total_vertices(&b), 300);
    }

    #[test]
    fn test_total_instances() {
        let mut b = new_indirect_draw_buffer_v3();
        idbv3_add_cmd(&mut b, 100, 3, 0, 0);
        idbv3_add_cmd(&mut b, 200, 2, 0, 0);
        assert_eq!(idbv3_total_instances(&b), 5);
    }

    #[test]
    fn test_clear() {
        let mut b = new_indirect_draw_buffer_v3();
        idbv3_add_cmd(&mut b, 100, 1, 0, 0);
        idbv3_clear(&mut b);
        assert_eq!(idbv3_cmd_count(&b), 0);
    }

    #[test]
    fn test_total_vertices_empty() {
        let b = new_indirect_draw_buffer_v3();
        assert_eq!(idbv3_total_vertices(&b), 0);
    }

    #[test]
    fn test_total_instances_empty() {
        let b = new_indirect_draw_buffer_v3();
        assert_eq!(idbv3_total_instances(&b), 0);
    }

    #[test]
    fn test_multiple_cmds() {
        let mut b = new_indirect_draw_buffer_v3();
        for _ in 0..5 {
            idbv3_add_cmd(&mut b, 10, 1, 0, 0);
        }
        assert_eq!(idbv3_cmd_count(&b), 5);
    }
}
