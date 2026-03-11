// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Render command queue.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DrawCall {
    pub mesh_id: u32,
    pub material_id: u32,
    pub transform: [f32; 16],
    pub layer: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderQueue {
    calls: Vec<DrawCall>,
}

#[allow(dead_code)]
pub fn new_render_queue() -> RenderQueue {
    RenderQueue { calls: Vec::new() }
}

#[allow(dead_code)]
pub fn rq_push(rq: &mut RenderQueue, call: DrawCall) {
    rq.calls.push(call);
}

#[allow(dead_code)]
pub fn rq_sort_by_layer(rq: &mut RenderQueue) {
    rq.calls.sort_by_key(|c| c.layer);
}

#[allow(dead_code)]
pub fn rq_clear(rq: &mut RenderQueue) {
    rq.calls.clear();
}

#[allow(dead_code)]
pub fn rq_len(rq: &RenderQueue) -> usize {
    rq.calls.len()
}

#[allow(dead_code)]
pub fn rq_is_empty(rq: &RenderQueue) -> bool {
    rq.calls.is_empty()
}

#[allow(dead_code)]
pub fn rq_get(rq: &RenderQueue, index: usize) -> Option<&DrawCall> {
    rq.calls.get(index)
}

#[allow(dead_code)]
pub fn rq_drain(rq: &mut RenderQueue) -> Vec<DrawCall> {
    std::mem::take(&mut rq.calls)
}

#[allow(dead_code)]
pub fn rq_to_json(rq: &RenderQueue) -> String {
    let entries: Vec<String> = rq
        .calls
        .iter()
        .map(|c| {
            format!(
                r#"{{"mesh_id":{},"material_id":{},"layer":{}}}"#,
                c.mesh_id, c.material_id, c.layer
            )
        })
        .collect();
    format!("[{}]", entries.join(","))
}

fn make_identity_transform() -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_call(mesh_id: u32, layer: u32) -> DrawCall {
        DrawCall {
            mesh_id,
            material_id: 0,
            transform: make_identity_transform(),
            layer,
        }
    }

    #[test]
    fn test_new_queue_empty() {
        let rq = new_render_queue();
        assert!(rq_is_empty(&rq));
        assert_eq!(rq_len(&rq), 0);
    }

    #[test]
    fn test_push_increases_len() {
        let mut rq = new_render_queue();
        rq_push(&mut rq, make_call(1, 0));
        assert_eq!(rq_len(&rq), 1);
    }

    #[test]
    fn test_sort_by_layer() {
        let mut rq = new_render_queue();
        rq_push(&mut rq, make_call(1, 5));
        rq_push(&mut rq, make_call(2, 1));
        rq_push(&mut rq, make_call(3, 3));
        rq_sort_by_layer(&mut rq);
        assert_eq!(rq_get(&rq, 0).unwrap().layer, 1);
        assert_eq!(rq_get(&rq, 2).unwrap().layer, 5);
    }

    #[test]
    fn test_clear() {
        let mut rq = new_render_queue();
        rq_push(&mut rq, make_call(1, 0));
        rq_clear(&mut rq);
        assert!(rq_is_empty(&rq));
    }

    #[test]
    fn test_get_valid_index() {
        let mut rq = new_render_queue();
        rq_push(&mut rq, make_call(42, 0));
        assert_eq!(rq_get(&rq, 0).unwrap().mesh_id, 42);
    }

    #[test]
    fn test_get_out_of_bounds() {
        let rq = new_render_queue();
        assert!(rq_get(&rq, 0).is_none());
    }

    #[test]
    fn test_drain_empties_queue() {
        let mut rq = new_render_queue();
        rq_push(&mut rq, make_call(1, 0));
        rq_push(&mut rq, make_call(2, 1));
        let drained = rq_drain(&mut rq);
        assert_eq!(drained.len(), 2);
        assert!(rq_is_empty(&rq));
    }

    #[test]
    fn test_to_json_non_empty() {
        let mut rq = new_render_queue();
        rq_push(&mut rq, make_call(7, 2));
        let j = rq_to_json(&rq);
        assert!(j.contains("mesh_id"));
    }

    #[test]
    fn test_to_json_empty() {
        let rq = new_render_queue();
        assert_eq!(rq_to_json(&rq), "[]");
    }
}
