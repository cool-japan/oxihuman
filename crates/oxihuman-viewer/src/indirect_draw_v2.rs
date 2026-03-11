#![allow(dead_code)]

//! Indirect draw argument buffer v2.

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct DrawIndexedIndirectArg {
    pub index_count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub vertex_offset: i32,
    pub first_instance: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IndirectDrawBufferV2 {
    pub args: Vec<DrawIndexedIndirectArg>,
    pub capacity: u32,
    pub generation: u32,
}

#[allow(dead_code)]
pub fn new_indirect_draw_buffer_v2(capacity: u32) -> IndirectDrawBufferV2 {
    IndirectDrawBufferV2 {
        args: Vec::with_capacity(capacity as usize),
        capacity,
        generation: 0,
    }
}

#[allow(dead_code)]
pub fn idb2_push(
    buf: &mut IndirectDrawBufferV2,
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    vertex_offset: i32,
    first_instance: u32,
) -> bool {
    if buf.args.len() >= buf.capacity as usize {
        return false;
    }
    buf.args.push(DrawIndexedIndirectArg {
        index_count,
        instance_count,
        first_index,
        vertex_offset,
        first_instance,
    });
    true
}

#[allow(dead_code)]
pub fn idb2_count(buf: &IndirectDrawBufferV2) -> usize {
    buf.args.len()
}

#[allow(dead_code)]
pub fn idb2_clear(buf: &mut IndirectDrawBufferV2) {
    buf.args.clear();
    buf.generation += 1;
}

#[allow(dead_code)]
pub fn idb2_is_full(buf: &IndirectDrawBufferV2) -> bool {
    buf.args.len() >= buf.capacity as usize
}

#[allow(dead_code)]
pub fn idb2_total_instances(buf: &IndirectDrawBufferV2) -> u32 {
    buf.args.iter().map(|a| a.instance_count).sum()
}

#[allow(dead_code)]
pub fn idb2_total_indices(buf: &IndirectDrawBufferV2) -> u32 {
    buf.args.iter().map(|a| a.index_count * a.instance_count).sum()
}

#[allow(dead_code)]
pub fn idb2_patch_instance_count(buf: &mut IndirectDrawBufferV2, slot: usize, count: u32) {
    if let Some(arg) = buf.args.get_mut(slot) {
        arg.instance_count = count;
    }
}

#[allow(dead_code)]
pub fn idb2_to_json(buf: &IndirectDrawBufferV2) -> String {
    format!(
        "{{\"count\":{},\"capacity\":{},\"generation\":{}}}",
        buf.args.len(),
        buf.capacity,
        buf.generation
    )
}

#[allow(dead_code)]
pub fn idb2_remaining(buf: &IndirectDrawBufferV2) -> u32 {
    buf.capacity.saturating_sub(buf.args.len() as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer() {
        let b = new_indirect_draw_buffer_v2(16);
        assert_eq!(idb2_count(&b), 0);
        assert_eq!(b.capacity, 16);
    }

    #[test]
    fn test_push() {
        let mut b = new_indirect_draw_buffer_v2(16);
        let ok = idb2_push(&mut b, 36, 1, 0, 0, 0);
        assert!(ok);
        assert_eq!(idb2_count(&b), 1);
    }

    #[test]
    fn test_push_full() {
        let mut b = new_indirect_draw_buffer_v2(1);
        idb2_push(&mut b, 36, 1, 0, 0, 0);
        let ok = idb2_push(&mut b, 36, 1, 0, 0, 0);
        assert!(!ok);
    }

    #[test]
    fn test_is_full() {
        let mut b = new_indirect_draw_buffer_v2(1);
        idb2_push(&mut b, 36, 1, 0, 0, 0);
        assert!(idb2_is_full(&b));
    }

    #[test]
    fn test_clear() {
        let mut b = new_indirect_draw_buffer_v2(8);
        idb2_push(&mut b, 36, 1, 0, 0, 0);
        idb2_clear(&mut b);
        assert_eq!(idb2_count(&b), 0);
    }

    #[test]
    fn test_generation_increments_on_clear() {
        let mut b = new_indirect_draw_buffer_v2(8);
        let gen0 = b.generation;
        idb2_clear(&mut b);
        assert!(b.generation > gen0);
    }

    #[test]
    fn test_total_instances() {
        let mut b = new_indirect_draw_buffer_v2(8);
        idb2_push(&mut b, 36, 3, 0, 0, 0);
        idb2_push(&mut b, 36, 2, 0, 0, 0);
        assert_eq!(idb2_total_instances(&b), 5);
    }

    #[test]
    fn test_patch_instance_count() {
        let mut b = new_indirect_draw_buffer_v2(8);
        idb2_push(&mut b, 36, 1, 0, 0, 0);
        idb2_patch_instance_count(&mut b, 0, 10);
        assert_eq!(b.args[0].instance_count, 10);
    }

    #[test]
    fn test_remaining() {
        let mut b = new_indirect_draw_buffer_v2(4);
        idb2_push(&mut b, 36, 1, 0, 0, 0);
        assert_eq!(idb2_remaining(&b), 3);
    }

    #[test]
    fn test_to_json() {
        let b = new_indirect_draw_buffer_v2(32);
        let json = idb2_to_json(&b);
        assert!(json.contains("capacity"));
    }
}
