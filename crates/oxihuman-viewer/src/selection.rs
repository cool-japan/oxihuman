// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Selection buffer and hover state for the interactive viewer.

// ── Types ─────────────────────────────────────────────────────────────────────

/// The kind of entity being selected.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SelectionKind {
    Vertex,
    Edge,
    Face,
    Object,
    Bone,
}

/// A single selected entity.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectionItem {
    pub kind: SelectionKind,
    pub index: usize,
    pub object_id: usize,
}

/// Buffer tracking current selection and hover state.
#[derive(Debug, Clone, Default)]
pub struct SelectionBuffer {
    pub items: Vec<SelectionItem>,
    pub hover: Option<SelectionItem>,
}

// ── SelectionBuffer impl ──────────────────────────────────────────────────────

impl SelectionBuffer {
    /// Create an empty selection buffer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an item to the selection (does not deduplicate).
    pub fn select(&mut self, item: SelectionItem) {
        self.items.push(item);
    }

    /// Remove the first item matching `kind` and `index`.
    /// Returns `true` if an item was removed.
    pub fn deselect(&mut self, kind: SelectionKind, index: usize) -> bool {
        if let Some(pos) = self
            .items
            .iter()
            .position(|it| it.kind == kind && it.index == index)
        {
            self.items.remove(pos);
            true
        } else {
            false
        }
    }

    /// Clear the entire selection.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Set the hover item (or `None` to clear hover).
    pub fn set_hover(&mut self, item: Option<SelectionItem>) {
        self.hover = item;
    }

    /// Get a reference to the current hover item.
    pub fn hover(&self) -> Option<&SelectionItem> {
        self.hover.as_ref()
    }

    /// Return `true` if an item with `kind` and `index` is currently selected.
    pub fn is_selected(&self, kind: SelectionKind, index: usize) -> bool {
        self.items
            .iter()
            .any(|it| it.kind == kind && it.index == index)
    }

    /// Return all selected indices for the given `kind`.
    pub fn selected_indices(&self, kind: SelectionKind) -> Vec<usize> {
        self.items
            .iter()
            .filter(|it| it.kind == kind)
            .map(|it| it.index)
            .collect()
    }

    /// Return the total number of selected items.
    pub fn count(&self) -> usize {
        self.items.len()
    }

    /// Select all items in the index range `[start, end)` for the given kind / object.
    pub fn select_range(
        &mut self,
        kind: SelectionKind,
        object_id: usize,
        start: usize,
        end: usize,
    ) {
        for idx in start..end {
            self.items.push(SelectionItem {
                kind: kind.clone(),
                index: idx,
                object_id,
            });
        }
    }

    /// Invert the selection for the given kind / object over the range `[0, total)`.
    ///
    /// Items that are currently selected are deselected; items that are not
    /// selected are added to the selection.
    pub fn invert_selection(&mut self, kind: SelectionKind, object_id: usize, total: usize) {
        let currently: std::collections::HashSet<usize> = self
            .items
            .iter()
            .filter(|it| it.kind == kind && it.object_id == object_id)
            .map(|it| it.index)
            .collect();

        // Remove all existing items for this kind/object
        self.items
            .retain(|it| !(it.kind == kind && it.object_id == object_id));

        // Add items that were NOT selected
        for idx in 0..total {
            if !currently.contains(&idx) {
                self.items.push(SelectionItem {
                    kind: kind.clone(),
                    index: idx,
                    object_id,
                });
            }
        }
    }
}

// ── Raycast ───────────────────────────────────────────────────────────────────

/// Möller–Trumbore ray–triangle intersection.
/// Returns the distance `t` along the ray if it hits, else `None`.
fn ray_tri_intersect(
    origin: [f32; 3],
    dir: [f32; 3],
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
) -> Option<f32> {
    const EPSILON: f32 = 1e-8;

    let e1 = sub3(v1, v0);
    let e2 = sub3(v2, v0);
    let h = cross3(dir, e2);
    let a = dot3(e1, h);

    if a.abs() < EPSILON {
        return None; // Ray is parallel
    }

    let f = 1.0 / a;
    let s = sub3(origin, v0);
    let u = f * dot3(s, h);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let q = cross3(s, e1);
    let v = f * dot3(dir, q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t = f * dot3(e2, q);
    if t > EPSILON {
        Some(t)
    } else {
        None
    }
}

/// Cast a ray against a triangle mesh and return the index of the nearest hit face.
///
/// `positions` — vertex positions; `tris` — triangles as `[v0, v1, v2]` index triples.
/// Returns `None` if no triangle is hit.
pub fn raycast_select(
    ray_origin: [f32; 3],
    ray_dir: [f32; 3],
    positions: &[[f32; 3]],
    tris: &[[u32; 3]],
) -> Option<usize> {
    let mut nearest_t = f32::INFINITY;
    let mut nearest_face = None;

    for (face_idx, tri) in tris.iter().enumerate() {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;
        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
            continue;
        }
        if let Some(t) = ray_tri_intersect(
            ray_origin,
            ray_dir,
            positions[i0],
            positions[i1],
            positions[i2],
        ) {
            if t < nearest_t {
                nearest_t = t;
                nearest_face = Some(face_idx);
            }
        }
    }

    nearest_face
}

// ── Private math helpers ──────────────────────────────────────────────────────

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn vertex_item(idx: usize) -> SelectionItem {
        SelectionItem {
            kind: SelectionKind::Vertex,
            index: idx,
            object_id: 0,
        }
    }

    fn face_item(idx: usize) -> SelectionItem {
        SelectionItem {
            kind: SelectionKind::Face,
            index: idx,
            object_id: 0,
        }
    }

    #[test]
    fn select_and_is_selected() {
        let mut buf = SelectionBuffer::new();
        buf.select(vertex_item(5));
        assert!(buf.is_selected(SelectionKind::Vertex, 5));
        assert!(!buf.is_selected(SelectionKind::Vertex, 6));
    }

    #[test]
    fn deselect_found() {
        let mut buf = SelectionBuffer::new();
        buf.select(vertex_item(3));
        let removed = buf.deselect(SelectionKind::Vertex, 3);
        assert!(removed);
        assert!(!buf.is_selected(SelectionKind::Vertex, 3));
    }

    #[test]
    fn deselect_not_found() {
        let mut buf = SelectionBuffer::new();
        let removed = buf.deselect(SelectionKind::Vertex, 99);
        assert!(!removed);
    }

    #[test]
    fn clear_empties_buffer() {
        let mut buf = SelectionBuffer::new();
        buf.select(vertex_item(0));
        buf.select(vertex_item(1));
        buf.clear();
        assert_eq!(buf.count(), 0);
    }

    #[test]
    fn set_hover_and_get() {
        let mut buf = SelectionBuffer::new();
        assert!(buf.hover().is_none());
        buf.set_hover(Some(face_item(7)));
        assert_eq!(buf.hover().expect("should succeed").index, 7);
    }

    #[test]
    fn clear_hover() {
        let mut buf = SelectionBuffer::new();
        buf.set_hover(Some(face_item(2)));
        buf.set_hover(None);
        assert!(buf.hover().is_none());
    }

    #[test]
    fn selected_indices_by_kind() {
        let mut buf = SelectionBuffer::new();
        buf.select(vertex_item(10));
        buf.select(vertex_item(20));
        buf.select(face_item(5));
        let verts = buf.selected_indices(SelectionKind::Vertex);
        assert_eq!(verts.len(), 2);
        assert!(verts.contains(&10));
        assert!(verts.contains(&20));
    }

    #[test]
    fn count_is_correct() {
        let mut buf = SelectionBuffer::new();
        buf.select(vertex_item(0));
        buf.select(vertex_item(1));
        buf.select(face_item(0));
        assert_eq!(buf.count(), 3);
    }

    #[test]
    fn select_range_correct_size() {
        let mut buf = SelectionBuffer::new();
        buf.select_range(SelectionKind::Vertex, 0, 5, 10);
        assert_eq!(buf.count(), 5);
        assert!(buf.is_selected(SelectionKind::Vertex, 5));
        assert!(buf.is_selected(SelectionKind::Vertex, 9));
        assert!(!buf.is_selected(SelectionKind::Vertex, 10));
    }

    #[test]
    fn invert_selection_toggles_all() {
        let mut buf = SelectionBuffer::new();
        // Select vertices 0 and 1 out of [0,1,2,3]
        buf.select(vertex_item(0));
        buf.select(vertex_item(1));
        buf.invert_selection(SelectionKind::Vertex, 0, 4);
        // Now 2 and 3 should be selected, 0 and 1 not
        assert!(!buf.is_selected(SelectionKind::Vertex, 0));
        assert!(!buf.is_selected(SelectionKind::Vertex, 1));
        assert!(buf.is_selected(SelectionKind::Vertex, 2));
        assert!(buf.is_selected(SelectionKind::Vertex, 3));
    }

    #[test]
    fn raycast_hits_known_face() {
        // Triangle in the XY plane at z=0
        let positions: &[[f32; 3]] = &[[-1.0, -1.0, 0.0], [1.0, -1.0, 0.0], [0.0, 1.0, 0.0]];
        let tris: &[[u32; 3]] = &[[0, 1, 2]];
        // Ray from z=5 shooting toward -z
        let hit = raycast_select([0.0, 0.0, 5.0], [0.0, 0.0, -1.0], positions, tris);
        assert_eq!(hit, Some(0), "should hit face 0");
    }

    #[test]
    fn raycast_no_hit_for_ray_away() {
        let positions: &[[f32; 3]] = &[[-1.0, -1.0, 0.0], [1.0, -1.0, 0.0], [0.0, 1.0, 0.0]];
        let tris: &[[u32; 3]] = &[[0, 1, 2]];
        // Ray pointing in +z away from triangle
        let hit = raycast_select([0.0, 0.0, 5.0], [0.0, 0.0, 1.0], positions, tris);
        assert!(hit.is_none(), "ray away should not hit");
    }

    #[test]
    fn raycast_nearest_face_returned() {
        // Two triangles: one at z=-1 and one at z=-3 (ray from z=5)
        let positions: &[[f32; 3]] = &[
            // face 0 at z=-1
            [-1.0, -1.0, -1.0],
            [1.0, -1.0, -1.0],
            [0.0, 1.0, -1.0],
            // face 1 at z=-3
            [-1.0, -1.0, -3.0],
            [1.0, -1.0, -3.0],
            [0.0, 1.0, -3.0],
        ];
        let tris: &[[u32; 3]] = &[[0, 1, 2], [3, 4, 5]];
        let hit = raycast_select([0.0, 0.0, 5.0], [0.0, 0.0, -1.0], positions, tris);
        assert_eq!(hit, Some(0), "nearest face (face 0) should be returned");
    }

    #[test]
    fn empty_mesh_no_hit() {
        let hit = raycast_select([0.0, 0.0, 1.0], [0.0, 0.0, -1.0], &[], &[]);
        assert!(hit.is_none());
    }

    #[test]
    fn hover_kind_preserved() {
        let mut buf = SelectionBuffer::new();
        let item = SelectionItem {
            kind: SelectionKind::Bone,
            index: 3,
            object_id: 1,
        };
        buf.set_hover(Some(item));
        assert_eq!(buf.hover().expect("should succeed").kind, SelectionKind::Bone);
    }
}
