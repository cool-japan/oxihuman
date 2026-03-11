// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A single draw call descriptor.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DrawCallDesc {
    pub program_id: u32,
    pub vbo_id: u32,
    pub ibo_id: u32,
    pub index_count: u32,
    pub base_vertex: i32,
    pub instance_count: u32,
    pub visible: bool,
}

/// An ordered list of draw calls.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct DrawList {
    pub calls: Vec<DrawCallDesc>,
}

/// Create a draw call descriptor.
#[allow(dead_code)]
pub fn new_draw_call(prog: u32, vbo: u32, ibo: u32, idx_count: u32) -> DrawCallDesc {
    DrawCallDesc {
        program_id: prog,
        vbo_id: vbo,
        ibo_id: ibo,
        index_count: idx_count,
        base_vertex: 0,
        instance_count: 1,
        visible: true,
    }
}

/// Create a new empty draw list.
#[allow(dead_code)]
pub fn new_draw_list() -> DrawList {
    DrawList { calls: Vec::new() }
}

/// Append a draw call to the list.
#[allow(dead_code)]
pub fn add_draw_call(list: &mut DrawList, call: DrawCallDesc) {
    list.calls.push(call);
}

/// Count draw calls where visible == true.
#[allow(dead_code)]
pub fn visible_call_count(list: &DrawList) -> usize {
    list.calls.iter().filter(|c| c.visible).count()
}

/// Total triangle count across all visible draw calls.
#[allow(dead_code)]
pub fn total_triangle_count(list: &DrawList) -> u64 {
    list.calls
        .iter()
        .filter(|c| c.visible)
        .map(|c| (c.index_count as u64 / 3) * c.instance_count as u64)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_draw_list_empty() {
        let dl = new_draw_list();
        assert_eq!(visible_call_count(&dl), 0);
    }

    #[test]
    fn add_draw_call_visible() {
        let mut dl = new_draw_list();
        let dc = new_draw_call(1, 2, 3, 6);
        add_draw_call(&mut dl, dc);
        assert_eq!(visible_call_count(&dl), 1);
    }

    #[test]
    fn invisible_call_not_counted() {
        let mut dl = new_draw_list();
        let mut dc = new_draw_call(1, 2, 3, 6);
        dc.visible = false;
        add_draw_call(&mut dl, dc);
        assert_eq!(visible_call_count(&dl), 0);
    }

    #[test]
    fn total_triangle_count_one_call() {
        let mut dl = new_draw_list();
        add_draw_call(&mut dl, new_draw_call(1, 2, 3, 12)); // 4 tris
        assert_eq!(total_triangle_count(&dl), 4);
    }

    #[test]
    fn total_triangle_count_instanced() {
        let mut dl = new_draw_list();
        let mut dc = new_draw_call(1, 2, 3, 6); // 2 tris
        dc.instance_count = 3;
        add_draw_call(&mut dl, dc);
        assert_eq!(total_triangle_count(&dl), 6); // 2 * 3
    }

    #[test]
    fn draw_call_defaults() {
        let dc = new_draw_call(10, 20, 30, 9);
        assert_eq!(dc.base_vertex, 0);
        assert_eq!(dc.instance_count, 1);
        assert!(dc.visible);
    }

    #[test]
    fn program_id_stored() {
        let dc = new_draw_call(55, 1, 2, 3);
        assert_eq!(dc.program_id, 55);
    }

    #[test]
    fn vbo_ibo_ids_stored() {
        let dc = new_draw_call(1, 11, 22, 3);
        assert_eq!(dc.vbo_id, 11);
        assert_eq!(dc.ibo_id, 22);
    }

    #[test]
    fn invisible_calls_excluded_from_triangles() {
        let mut dl = new_draw_list();
        let mut dc = new_draw_call(1, 1, 1, 30);
        dc.visible = false;
        add_draw_call(&mut dl, dc);
        assert_eq!(total_triangle_count(&dl), 0);
    }

    #[test]
    fn multiple_calls_total_triangles() {
        let mut dl = new_draw_list();
        add_draw_call(&mut dl, new_draw_call(1, 1, 1, 6));  // 2 tris
        add_draw_call(&mut dl, new_draw_call(1, 1, 1, 9));  // 3 tris
        assert_eq!(total_triangle_count(&dl), 5);
    }
}
