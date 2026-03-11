#![allow(dead_code)]

//! Draw call batching by material and pipeline.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DrawCall {
    pub mesh_id: u32,
    pub material_id: u32,
    pub pipeline_id: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub index_count: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DrawBatch {
    pub material_id: u32,
    pub pipeline_id: u32,
    pub calls: Vec<DrawCall>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct DrawBatcher {
    pub batches: Vec<DrawBatch>,
}

#[allow(dead_code)]
pub fn new_draw_batcher() -> DrawBatcher {
    DrawBatcher { batches: Vec::new() }
}

#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
pub fn db_submit(
    batcher: &mut DrawBatcher,
    mesh_id: u32,
    material_id: u32,
    pipeline_id: u32,
    instance_count: u32,
    first_index: u32,
    index_count: u32,
) {
    let call = DrawCall {
        mesh_id,
        material_id,
        pipeline_id,
        instance_count,
        first_index,
        index_count,
    };
    if let Some(batch) = batcher
        .batches
        .iter_mut()
        .find(|b| b.material_id == material_id && b.pipeline_id == pipeline_id)
    {
        batch.calls.push(call);
    } else {
        batcher.batches.push(DrawBatch {
            material_id,
            pipeline_id,
            calls: vec![call],
        });
    }
}

#[allow(dead_code)]
pub fn db_batch_count(batcher: &DrawBatcher) -> usize {
    batcher.batches.len()
}

#[allow(dead_code)]
pub fn db_total_draw_calls(batcher: &DrawBatcher) -> usize {
    batcher.batches.iter().map(|b| b.calls.len()).sum()
}

#[allow(dead_code)]
pub fn db_clear(batcher: &mut DrawBatcher) {
    batcher.batches.clear();
}

#[allow(dead_code)]
pub fn db_calls_for_material(batcher: &DrawBatcher, material_id: u32) -> usize {
    batcher
        .batches
        .iter()
        .filter(|b| b.material_id == material_id)
        .map(|b| b.calls.len())
        .sum()
}

#[allow(dead_code)]
pub fn db_to_json(batcher: &DrawBatcher) -> String {
    format!(
        "{{\"batch_count\":{},\"total_calls\":{}}}",
        batcher.batches.len(),
        db_total_draw_calls(batcher)
    )
}

#[allow(dead_code)]
pub fn db_total_instance_count(batcher: &DrawBatcher) -> u32 {
    batcher
        .batches
        .iter()
        .flat_map(|b| b.calls.iter())
        .map(|c| c.instance_count)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_batcher() {
        let b = new_draw_batcher();
        assert_eq!(db_batch_count(&b), 0);
    }

    #[test]
    fn test_submit_creates_batch() {
        let mut b = new_draw_batcher();
        db_submit(&mut b, 0, 1, 2, 1, 0, 36);
        assert_eq!(db_batch_count(&b), 1);
    }

    #[test]
    fn test_same_material_batched() {
        let mut b = new_draw_batcher();
        db_submit(&mut b, 0, 1, 2, 1, 0, 36);
        db_submit(&mut b, 1, 1, 2, 1, 0, 36);
        assert_eq!(db_batch_count(&b), 1);
        assert_eq!(db_total_draw_calls(&b), 2);
    }

    #[test]
    fn test_different_material_new_batch() {
        let mut b = new_draw_batcher();
        db_submit(&mut b, 0, 1, 2, 1, 0, 36);
        db_submit(&mut b, 1, 3, 2, 1, 0, 36);
        assert_eq!(db_batch_count(&b), 2);
    }

    #[test]
    fn test_clear() {
        let mut b = new_draw_batcher();
        db_submit(&mut b, 0, 1, 2, 1, 0, 36);
        db_clear(&mut b);
        assert_eq!(db_batch_count(&b), 0);
    }

    #[test]
    fn test_calls_for_material() {
        let mut b = new_draw_batcher();
        db_submit(&mut b, 0, 1, 1, 1, 0, 36);
        db_submit(&mut b, 1, 1, 1, 1, 0, 36);
        db_submit(&mut b, 2, 2, 1, 1, 0, 36);
        assert_eq!(db_calls_for_material(&b, 1), 2);
    }

    #[test]
    fn test_total_draw_calls() {
        let mut b = new_draw_batcher();
        db_submit(&mut b, 0, 1, 1, 1, 0, 36);
        db_submit(&mut b, 1, 2, 1, 1, 0, 36);
        db_submit(&mut b, 2, 3, 1, 1, 0, 36);
        assert_eq!(db_total_draw_calls(&b), 3);
    }

    #[test]
    fn test_total_instance_count() {
        let mut b = new_draw_batcher();
        db_submit(&mut b, 0, 1, 1, 10, 0, 36);
        db_submit(&mut b, 1, 2, 1, 5, 0, 36);
        assert_eq!(db_total_instance_count(&b), 15);
    }

    #[test]
    fn test_to_json() {
        let b = new_draw_batcher();
        let json = db_to_json(&b);
        assert!(json.contains("batch_count"));
    }

    #[test]
    fn test_different_pipeline_new_batch() {
        let mut b = new_draw_batcher();
        db_submit(&mut b, 0, 1, 1, 1, 0, 36);
        db_submit(&mut b, 1, 1, 2, 1, 0, 36);
        assert_eq!(db_batch_count(&b), 2);
    }
}
