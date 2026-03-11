// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Indirect draw command buffer management — structures and utilities for
//! GPU-driven rendering with indirect draw calls.

/// A single indirect draw command (matches `DrawIndirectArgs` on GPU).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrawIndirectCommand {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub first_instance: u32,
}

/// Indexed indirect draw command (matches `DrawIndexedIndirectArgs`).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrawIndexedIndirectCommand {
    pub index_count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub vertex_offset: i32,
    pub first_instance: u32,
}

/// Buffer of indirect commands.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IndirectDrawBuffer {
    pub commands: Vec<DrawIndexedIndirectCommand>,
    /// Byte stride between commands.
    pub stride: u32,
}

impl Default for IndirectDrawBuffer {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            stride: 20, // 5 * u32
        }
    }
}

impl IndirectDrawBuffer {
    /// Add a draw command.
    #[allow(dead_code)]
    pub fn push(&mut self, cmd: DrawIndexedIndirectCommand) {
        self.commands.push(cmd);
    }

    /// Number of draw commands.
    #[allow(dead_code)]
    pub fn count(&self) -> u32 {
        self.commands.len() as u32
    }

    /// Total buffer size in bytes.
    #[allow(dead_code)]
    pub fn byte_size(&self) -> u32 {
        self.commands.len() as u32 * self.stride
    }

    /// Clear all commands.
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// Sort commands by index count (larger first for better GPU utilisation).
    #[allow(dead_code)]
    pub fn sort_by_index_count(&mut self) {
        self.commands.sort_by(|a, b| b.index_count.cmp(&a.index_count));
    }

    /// Merge consecutive commands that reference the same vertex buffer.
    #[allow(dead_code)]
    pub fn merge_compatible(&mut self) {
        if self.commands.len() < 2 {
            return;
        }
        let mut merged = Vec::with_capacity(self.commands.len());
        merged.push(self.commands[0]);

        for cmd in &self.commands[1..] {
            let Some(last) = merged.last_mut() else { continue; };
            if last.vertex_offset == cmd.vertex_offset
                && last.first_index + last.index_count == cmd.first_index
                && last.first_instance == cmd.first_instance
            {
                last.index_count += cmd.index_count;
                last.instance_count = last.instance_count.max(cmd.instance_count);
            } else {
                merged.push(*cmd);
            }
        }
        self.commands = merged;
    }

    /// Serialise commands to a byte buffer (little-endian).
    #[allow(dead_code)]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.commands.len() * 20);
        for cmd in &self.commands {
            buf.extend_from_slice(&cmd.index_count.to_le_bytes());
            buf.extend_from_slice(&cmd.instance_count.to_le_bytes());
            buf.extend_from_slice(&cmd.first_index.to_le_bytes());
            buf.extend_from_slice(&cmd.vertex_offset.to_le_bytes());
            buf.extend_from_slice(&cmd.first_instance.to_le_bytes());
        }
        buf
    }
}

/// Total triangle count across all commands.
#[allow(dead_code)]
pub fn total_triangles(buffer: &IndirectDrawBuffer) -> u64 {
    buffer.commands.iter().map(|c| (c.index_count / 3) as u64 * c.instance_count as u64).sum()
}

/// Total vertex count across all commands.
#[allow(dead_code)]
pub fn total_vertices(buffer: &IndirectDrawBuffer) -> u64 {
    buffer.commands.iter().map(|c| c.index_count as u64 * c.instance_count as u64).sum()
}

/// Create a single-draw command for a mesh.
#[allow(dead_code)]
pub fn single_draw(index_count: u32, vertex_offset: i32) -> DrawIndexedIndirectCommand {
    DrawIndexedIndirectCommand {
        index_count,
        instance_count: 1,
        first_index: 0,
        vertex_offset,
        first_instance: 0,
    }
}

/// Create an instanced draw command.
#[allow(dead_code)]
pub fn instanced_draw(index_count: u32, instance_count: u32) -> DrawIndexedIndirectCommand {
    DrawIndexedIndirectCommand {
        index_count,
        instance_count,
        first_index: 0,
        vertex_offset: 0,
        first_instance: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_buffer() {
        let b = IndirectDrawBuffer::default();
        assert_eq!(b.count(), 0);
        assert_eq!(b.byte_size(), 0);
    }

    #[test]
    fn test_push_and_count() {
        let mut b = IndirectDrawBuffer::default();
        b.push(single_draw(36, 0));
        b.push(single_draw(24, 100));
        assert_eq!(b.count(), 2);
    }

    #[test]
    fn test_byte_size() {
        let mut b = IndirectDrawBuffer::default();
        b.push(single_draw(36, 0));
        assert_eq!(b.byte_size(), 20);
    }

    #[test]
    fn test_clear() {
        let mut b = IndirectDrawBuffer::default();
        b.push(single_draw(36, 0));
        b.clear();
        assert_eq!(b.count(), 0);
    }

    #[test]
    fn test_sort_by_index_count() {
        let mut b = IndirectDrawBuffer::default();
        b.push(single_draw(10, 0));
        b.push(single_draw(100, 0));
        b.push(single_draw(50, 0));
        b.sort_by_index_count();
        assert_eq!(b.commands[0].index_count, 100);
        assert_eq!(b.commands[2].index_count, 10);
    }

    #[test]
    fn test_to_bytes_length() {
        let mut b = IndirectDrawBuffer::default();
        b.push(single_draw(36, 0));
        let bytes = b.to_bytes();
        assert_eq!(bytes.len(), 20);
    }

    #[test]
    fn test_total_triangles() {
        let mut b = IndirectDrawBuffer::default();
        b.push(single_draw(36, 0)); // 12 triangles
        b.push(single_draw(6, 0));  // 2 triangles
        assert_eq!(total_triangles(&b), 14);
    }

    #[test]
    fn test_instanced_draw() {
        let cmd = instanced_draw(36, 10);
        assert_eq!(cmd.instance_count, 10);
    }

    #[test]
    fn test_merge_compatible_adjacent() {
        let mut b = IndirectDrawBuffer::default();
        b.push(DrawIndexedIndirectCommand {
            index_count: 36, instance_count: 1, first_index: 0, vertex_offset: 0, first_instance: 0,
        });
        b.push(DrawIndexedIndirectCommand {
            index_count: 24, instance_count: 1, first_index: 36, vertex_offset: 0, first_instance: 0,
        });
        b.merge_compatible();
        assert_eq!(b.count(), 1);
        assert_eq!(b.commands[0].index_count, 60);
    }

    #[test]
    fn test_merge_compatible_incompatible() {
        let mut b = IndirectDrawBuffer::default();
        b.push(DrawIndexedIndirectCommand {
            index_count: 36, instance_count: 1, first_index: 0, vertex_offset: 0, first_instance: 0,
        });
        b.push(DrawIndexedIndirectCommand {
            index_count: 24, instance_count: 1, first_index: 0, vertex_offset: 100, first_instance: 0,
        });
        b.merge_compatible();
        assert_eq!(b.count(), 2);
    }
}
