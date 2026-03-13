// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Draw-indirect v2 helpers (GPU-driven rendering argument buffers).

/// A single indexed draw-indirect argument.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DrawIndexedIndirect {
    pub index_count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub base_vertex: i32,
    pub first_instance: u32,
}

/// A single non-indexed draw-indirect argument.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DrawIndirect {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub first_instance: u32,
}

/// Buffer holding many indirect draw arguments.
#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct DrawIndirectBuffer {
    pub indexed: Vec<DrawIndexedIndirect>,
    pub unindexed: Vec<DrawIndirect>,
}

#[allow(dead_code)]
pub fn new_draw_indirect_buffer() -> DrawIndirectBuffer {
    DrawIndirectBuffer::default()
}

#[allow(dead_code)]
pub fn di_push_indexed(buf: &mut DrawIndirectBuffer, cmd: DrawIndexedIndirect) {
    buf.indexed.push(cmd);
}

#[allow(dead_code)]
pub fn di_push_unindexed(buf: &mut DrawIndirectBuffer, cmd: DrawIndirect) {
    buf.unindexed.push(cmd);
}

#[allow(dead_code)]
pub fn di_clear(buf: &mut DrawIndirectBuffer) {
    buf.indexed.clear();
    buf.unindexed.clear();
}

#[allow(dead_code)]
pub fn di_indexed_count(buf: &DrawIndirectBuffer) -> usize {
    buf.indexed.len()
}

#[allow(dead_code)]
pub fn di_total_instances(buf: &DrawIndirectBuffer) -> u64 {
    let a: u64 = buf.indexed.iter().map(|c| c.instance_count as u64).sum();
    let b: u64 = buf.unindexed.iter().map(|c| c.instance_count as u64).sum();
    a + b
}

#[allow(dead_code)]
pub fn di_total_index_count(buf: &DrawIndirectBuffer) -> u64 {
    buf.indexed.iter().map(|c| c.index_count as u64).sum()
}

#[allow(dead_code)]
pub fn di_indexed_byte_size(buf: &DrawIndirectBuffer) -> usize {
    buf.indexed.len() * 20 // 5 x u32
}

#[allow(dead_code)]
pub fn di_to_json(buf: &DrawIndirectBuffer) -> String {
    format!(
        "{{\"indexed\":{},\"unindexed\":{},\"total_instances\":{}}}",
        buf.indexed.len(),
        buf.unindexed.len(),
        di_total_instances(buf)
    )
}

#[allow(dead_code)]
pub fn di_cull_zero_instance(buf: &mut DrawIndirectBuffer) {
    buf.indexed.retain(|c| c.instance_count > 0);
    buf.unindexed.retain(|c| c.instance_count > 0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty() {
        let b = new_draw_indirect_buffer();
        assert_eq!(di_indexed_count(&b), 0);
    }

    #[test]
    fn push_indexed() {
        let mut b = new_draw_indirect_buffer();
        di_push_indexed(
            &mut b,
            DrawIndexedIndirect {
                index_count: 3,
                instance_count: 1,
                ..Default::default()
            },
        );
        assert_eq!(di_indexed_count(&b), 1);
    }

    #[test]
    fn clear_empties() {
        let mut b = new_draw_indirect_buffer();
        di_push_indexed(&mut b, DrawIndexedIndirect::default());
        di_clear(&mut b);
        assert_eq!(di_indexed_count(&b), 0);
    }

    #[test]
    fn total_instances_sum() {
        let mut b = new_draw_indirect_buffer();
        di_push_indexed(
            &mut b,
            DrawIndexedIndirect {
                instance_count: 3,
                ..Default::default()
            },
        );
        di_push_unindexed(
            &mut b,
            DrawIndirect {
                instance_count: 2,
                ..Default::default()
            },
        );
        assert_eq!(di_total_instances(&b), 5);
    }

    #[test]
    fn total_index_count() {
        let mut b = new_draw_indirect_buffer();
        di_push_indexed(
            &mut b,
            DrawIndexedIndirect {
                index_count: 6,
                instance_count: 1,
                ..Default::default()
            },
        );
        assert_eq!(di_total_index_count(&b), 6);
    }

    #[test]
    fn byte_size() {
        let mut b = new_draw_indirect_buffer();
        di_push_indexed(&mut b, DrawIndexedIndirect::default());
        assert_eq!(di_indexed_byte_size(&b), 20);
    }

    #[test]
    fn cull_zero_instance() {
        let mut b = new_draw_indirect_buffer();
        di_push_indexed(
            &mut b,
            DrawIndexedIndirect {
                instance_count: 0,
                ..Default::default()
            },
        );
        di_push_indexed(
            &mut b,
            DrawIndexedIndirect {
                instance_count: 1,
                ..Default::default()
            },
        );
        di_cull_zero_instance(&mut b);
        assert_eq!(di_indexed_count(&b), 1);
    }

    #[test]
    fn json_has_indexed() {
        assert!(di_to_json(&new_draw_indirect_buffer()).contains("indexed"));
    }

    #[test]
    fn push_unindexed() {
        let mut b = new_draw_indirect_buffer();
        di_push_unindexed(
            &mut b,
            DrawIndirect {
                vertex_count: 4,
                instance_count: 1,
                ..Default::default()
            },
        );
        assert_eq!(b.unindexed.len(), 1);
    }

    #[test]
    fn total_instances_empty_zero() {
        assert_eq!(di_total_instances(&new_draw_indirect_buffer()), 0);
    }
}
