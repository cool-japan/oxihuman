// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A single draw submission.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct DrawItem {
    mesh_id: u32,
    material_id: u32,
    vertex_count: usize,
    sort_key: u64,
}

/// A collection of draw submissions.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderSubmission {
    items: Vec<DrawItem>,
}

/// Create a new empty render submission.
#[allow(dead_code)]
pub fn new_render_submission() -> RenderSubmission {
    RenderSubmission { items: Vec::new() }
}

/// Submit a draw call.
#[allow(dead_code)]
pub fn submit_draw(
    sub: &mut RenderSubmission,
    mesh_id: u32,
    material_id: u32,
    vertex_count: usize,
) {
    let sort_key = ((material_id as u64) << 32) | (mesh_id as u64);
    sub.items.push(DrawItem {
        mesh_id,
        material_id,
        vertex_count,
        sort_key,
    });
}

/// Return the number of submissions.
#[allow(dead_code)]
pub fn submission_count(sub: &RenderSubmission) -> usize {
    sub.items.len()
}

/// Sort submissions by sort key.
#[allow(dead_code)]
pub fn submission_sort(sub: &mut RenderSubmission) {
    sub.items.sort_by_key(|i| i.sort_key);
}

/// Flush all submissions (clear the list).
#[allow(dead_code)]
pub fn submission_flush(sub: &mut RenderSubmission) {
    sub.items.clear();
}

/// Return the total vertex count across all submissions.
#[allow(dead_code)]
pub fn submission_total_vertices(sub: &RenderSubmission) -> usize {
    sub.items.iter().map(|i| i.vertex_count).sum()
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn submission_to_json(sub: &RenderSubmission) -> String {
    format!(
        "{{\"count\":{},\"total_verts\":{}}}",
        sub.items.len(),
        submission_total_vertices(sub)
    )
}

/// Clear all submissions.
#[allow(dead_code)]
pub fn submission_clear(sub: &mut RenderSubmission) {
    sub.items.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty() {
        let s = new_render_submission();
        assert_eq!(submission_count(&s), 0);
    }

    #[test]
    fn submit_increments() {
        let mut s = new_render_submission();
        submit_draw(&mut s, 0, 0, 100);
        assert_eq!(submission_count(&s), 1);
    }

    #[test]
    fn total_vertices() {
        let mut s = new_render_submission();
        submit_draw(&mut s, 0, 0, 100);
        submit_draw(&mut s, 1, 0, 200);
        assert_eq!(submission_total_vertices(&s), 300);
    }

    #[test]
    fn sort_does_not_panic() {
        let mut s = new_render_submission();
        submit_draw(&mut s, 1, 2, 100);
        submit_draw(&mut s, 0, 1, 200);
        submission_sort(&mut s);
        assert_eq!(submission_count(&s), 2);
    }

    #[test]
    fn flush_clears() {
        let mut s = new_render_submission();
        submit_draw(&mut s, 0, 0, 100);
        submission_flush(&mut s);
        assert_eq!(submission_count(&s), 0);
    }

    #[test]
    fn to_json() {
        let s = new_render_submission();
        let j = submission_to_json(&s);
        assert!(j.contains("\"count\":0"));
    }

    #[test]
    fn clear_works() {
        let mut s = new_render_submission();
        submit_draw(&mut s, 0, 0, 50);
        submission_clear(&mut s);
        assert_eq!(submission_count(&s), 0);
    }

    #[test]
    fn multiple_draws() {
        let mut s = new_render_submission();
        for i in 0..10u32 {
            submit_draw(&mut s, i, 0, 100);
        }
        assert_eq!(submission_count(&s), 10);
    }

    #[test]
    fn total_vertices_empty() {
        let s = new_render_submission();
        assert_eq!(submission_total_vertices(&s), 0);
    }

    #[test]
    fn json_with_data() {
        let mut s = new_render_submission();
        submit_draw(&mut s, 0, 0, 500);
        let j = submission_to_json(&s);
        assert!(j.contains("500"));
    }
}
