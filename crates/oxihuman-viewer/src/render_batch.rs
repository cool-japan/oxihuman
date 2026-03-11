#![allow(dead_code)]
//! Batches draw calls by material for efficient rendering.

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct BatchEntry {
    pub material_id: u32,
    pub vertex_count: u32,
    pub index_count: u32,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct RenderBatch {
    entries: Vec<BatchEntry>,
}

#[allow(dead_code)]
pub fn new_render_batch() -> RenderBatch {
    RenderBatch {
        entries: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_batch_entry(b: &mut RenderBatch, material_id: u32, vertex_count: u32, index_count: u32) {
    b.entries.push(BatchEntry {
        material_id,
        vertex_count,
        index_count,
    });
}

#[allow(dead_code)]
pub fn batch_entry_count(b: &RenderBatch) -> usize {
    b.entries.len()
}

#[allow(dead_code)]
pub fn batch_total_vertices(b: &RenderBatch) -> u64 {
    b.entries.iter().map(|e| e.vertex_count as u64).sum()
}

#[allow(dead_code)]
pub fn batch_total_indices(b: &RenderBatch) -> u64 {
    b.entries.iter().map(|e| e.index_count as u64).sum()
}

#[allow(dead_code)]
pub fn sort_batch_by_material(b: &mut RenderBatch) {
    b.entries.sort_by_key(|e| e.material_id);
}

#[allow(dead_code)]
pub fn flush_render_batch(b: &mut RenderBatch) -> Vec<BatchEntry> {
    let result = b.entries.clone();
    b.entries.clear();
    result
}

#[allow(dead_code)]
pub fn batch_is_empty_rb(b: &RenderBatch) -> bool {
    b.entries.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_render_batch() {
        let b = new_render_batch();
        assert!(batch_is_empty_rb(&b));
    }

    #[test]
    fn test_add_batch_entry() {
        let mut b = new_render_batch();
        add_batch_entry(&mut b, 0, 100, 300);
        assert_eq!(batch_entry_count(&b), 1);
    }

    #[test]
    fn test_batch_total_vertices() {
        let mut b = new_render_batch();
        add_batch_entry(&mut b, 0, 100, 300);
        add_batch_entry(&mut b, 1, 200, 600);
        assert_eq!(batch_total_vertices(&b), 300);
    }

    #[test]
    fn test_batch_total_indices() {
        let mut b = new_render_batch();
        add_batch_entry(&mut b, 0, 100, 300);
        add_batch_entry(&mut b, 1, 200, 600);
        assert_eq!(batch_total_indices(&b), 900);
    }

    #[test]
    fn test_sort_batch_by_material() {
        let mut b = new_render_batch();
        add_batch_entry(&mut b, 2, 10, 30);
        add_batch_entry(&mut b, 0, 20, 60);
        sort_batch_by_material(&mut b);
        assert_eq!(b.entries[0].material_id, 0);
    }

    #[test]
    fn test_flush_render_batch() {
        let mut b = new_render_batch();
        add_batch_entry(&mut b, 0, 10, 30);
        let flushed = flush_render_batch(&mut b);
        assert_eq!(flushed.len(), 1);
        assert!(batch_is_empty_rb(&b));
    }

    #[test]
    fn test_batch_is_empty_rb() {
        let b = new_render_batch();
        assert!(batch_is_empty_rb(&b));
    }

    #[test]
    fn test_batch_entry_count() {
        let mut b = new_render_batch();
        for i in 0..5 {
            add_batch_entry(&mut b, i, 10, 30);
        }
        assert_eq!(batch_entry_count(&b), 5);
    }

    #[test]
    fn test_empty_totals() {
        let b = new_render_batch();
        assert_eq!(batch_total_vertices(&b), 0);
        assert_eq!(batch_total_indices(&b), 0);
    }

    #[test]
    fn test_flush_empty() {
        let mut b = new_render_batch();
        let flushed = flush_render_batch(&mut b);
        assert!(flushed.is_empty());
    }
}
