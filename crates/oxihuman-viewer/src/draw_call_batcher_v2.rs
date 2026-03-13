// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Draw call batching for reduced API overhead.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DrawCallV2 {
    pub mesh_id: u32,
    pub material_id: u32,
    pub instance_count: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DrawCallBatcherV2 {
    pub calls: Vec<DrawCallV2>,
    pub max_instances: u32,
}

#[allow(dead_code)]
pub fn new_draw_call_batcher_v2(max_instances: u32) -> DrawCallBatcherV2 {
    DrawCallBatcherV2 { calls: Vec::new(), max_instances }
}

#[allow(dead_code)]
pub fn dcbv2_add(batcher: &mut DrawCallBatcherV2, mesh_id: u32, material_id: u32) {
    if dcbv2_can_batch(batcher, mesh_id, material_id) {
        if let Some(last) = batcher.calls.last_mut() {
            last.instance_count += 1;
            return;
        }
    }
    batcher.calls.push(DrawCallV2 { mesh_id, material_id, instance_count: 1 });
}

#[allow(dead_code)]
pub fn dcbv2_call_count(batcher: &DrawCallBatcherV2) -> usize {
    batcher.calls.len()
}

#[allow(dead_code)]
pub fn dcbv2_total_instances(batcher: &DrawCallBatcherV2) -> u32 {
    batcher.calls.iter().map(|c| c.instance_count).sum()
}

#[allow(dead_code)]
pub fn dcbv2_flush(batcher: &mut DrawCallBatcherV2) -> Vec<DrawCallV2> {
    let mut out = Vec::new();
    std::mem::swap(&mut batcher.calls, &mut out);
    out
}

#[allow(dead_code)]
pub fn dcbv2_can_batch(batcher: &DrawCallBatcherV2, mesh_id: u32, material_id: u32) -> bool {
    if let Some(last) = batcher.calls.last() {
        last.mesh_id == mesh_id
            && last.material_id == material_id
            && last.instance_count < batcher.max_instances
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_creates_call() {
        let mut b = new_draw_call_batcher_v2(100);
        dcbv2_add(&mut b, 1, 1);
        assert_eq!(dcbv2_call_count(&b), 1);
    }

    #[test]
    fn test_same_mesh_material_batches() {
        let mut b = new_draw_call_batcher_v2(100);
        dcbv2_add(&mut b, 1, 1);
        dcbv2_add(&mut b, 1, 1);
        assert_eq!(dcbv2_call_count(&b), 1);
        assert_eq!(dcbv2_total_instances(&b), 2);
    }

    #[test]
    fn test_different_material_new_call() {
        let mut b = new_draw_call_batcher_v2(100);
        dcbv2_add(&mut b, 1, 1);
        dcbv2_add(&mut b, 1, 2);
        assert_eq!(dcbv2_call_count(&b), 2);
    }

    #[test]
    fn test_total_instances() {
        let mut b = new_draw_call_batcher_v2(100);
        dcbv2_add(&mut b, 1, 1);
        dcbv2_add(&mut b, 1, 1);
        dcbv2_add(&mut b, 2, 1);
        assert_eq!(dcbv2_total_instances(&b), 3);
    }

    #[test]
    fn test_flush_clears() {
        let mut b = new_draw_call_batcher_v2(100);
        dcbv2_add(&mut b, 1, 1);
        let flushed = dcbv2_flush(&mut b);
        assert_eq!(flushed.len(), 1);
        assert_eq!(dcbv2_call_count(&b), 0);
    }

    #[test]
    fn test_can_batch_false_empty() {
        let b = new_draw_call_batcher_v2(100);
        assert!(!dcbv2_can_batch(&b, 1, 1));
    }

    #[test]
    fn test_max_instances_exceeded() {
        let mut b = new_draw_call_batcher_v2(2);
        dcbv2_add(&mut b, 1, 1);
        dcbv2_add(&mut b, 1, 1);
        dcbv2_add(&mut b, 1, 1); /* third should create new call */
        assert_eq!(dcbv2_call_count(&b), 2);
    }

    #[test]
    fn test_can_batch_true_when_possible() {
        let mut b = new_draw_call_batcher_v2(10);
        dcbv2_add(&mut b, 3, 5);
        assert!(dcbv2_can_batch(&b, 3, 5));
    }
}
