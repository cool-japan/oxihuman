#![allow(dead_code)]
//! Edge marking utilities.

/// Mark type for an edge.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarkType {
    Seam,
    Sharp,
    Crease,
    Boundary,
}

/// A single edge mark.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeMark {
    pub edge: [u32; 2],
    pub mark_type: MarkType,
}

/// Edge mark set.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeMarkSet {
    pub marks: Vec<EdgeMark>,
}

/// Create a new edge mark set.
#[allow(dead_code)]
pub fn new_edge_mark_set() -> EdgeMarkSet {
    EdgeMarkSet { marks: Vec::new() }
}

/// Mark an edge.
#[allow(dead_code)]
pub fn mark_edge(set: &mut EdgeMarkSet, edge: [u32; 2], mark_type: MarkType) {
    // Remove existing mark for this edge
    set.marks.retain(|m| m.edge != edge);
    set.marks.push(EdgeMark { edge, mark_type });
}

/// Unmark an edge.
#[allow(dead_code)]
pub fn unmark_edge(set: &mut EdgeMarkSet, edge: [u32; 2]) {
    set.marks.retain(|m| m.edge != edge);
}

/// Check if an edge is marked.
#[allow(dead_code)]
pub fn is_edge_marked(set: &EdgeMarkSet, edge: [u32; 2]) -> bool {
    set.marks.iter().any(|m| m.edge == edge)
}

/// Count marked edges.
#[allow(dead_code)]
pub fn marked_count(set: &EdgeMarkSet) -> usize {
    set.marks.len()
}

/// Get mark type for an edge.
#[allow(dead_code)]
pub fn mark_type_at(set: &EdgeMarkSet, edge: [u32; 2]) -> Option<MarkType> {
    set.marks.iter().find(|m| m.edge == edge).map(|m| m.mark_type)
}

/// Clear all marks.
#[allow(dead_code)]
pub fn clear_marks(set: &mut EdgeMarkSet) {
    set.marks.clear();
}

/// Serialize marks to JSON.
#[allow(dead_code)]
pub fn marks_to_json(set: &EdgeMarkSet) -> String {
    format!("{{\"mark_count\":{}}}", set.marks.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_edge_mark_set() {
        let s = new_edge_mark_set();
        assert!(s.marks.is_empty());
    }

    #[test]
    fn test_mark_edge() {
        let mut s = new_edge_mark_set();
        mark_edge(&mut s, [0, 1], MarkType::Seam);
        assert_eq!(s.marks.len(), 1);
    }

    #[test]
    fn test_mark_edge_replace() {
        let mut s = new_edge_mark_set();
        mark_edge(&mut s, [0, 1], MarkType::Seam);
        mark_edge(&mut s, [0, 1], MarkType::Sharp);
        assert_eq!(s.marks.len(), 1);
        assert_eq!(s.marks[0].mark_type, MarkType::Sharp);
    }

    #[test]
    fn test_unmark_edge() {
        let mut s = new_edge_mark_set();
        mark_edge(&mut s, [0, 1], MarkType::Seam);
        unmark_edge(&mut s, [0, 1]);
        assert!(s.marks.is_empty());
    }

    #[test]
    fn test_is_edge_marked() {
        let mut s = new_edge_mark_set();
        mark_edge(&mut s, [0, 1], MarkType::Seam);
        assert!(is_edge_marked(&s, [0, 1]));
        assert!(!is_edge_marked(&s, [2, 3]));
    }

    #[test]
    fn test_marked_count() {
        let mut s = new_edge_mark_set();
        mark_edge(&mut s, [0, 1], MarkType::Seam);
        mark_edge(&mut s, [2, 3], MarkType::Sharp);
        assert_eq!(marked_count(&s), 2);
    }

    #[test]
    fn test_mark_type_at() {
        let mut s = new_edge_mark_set();
        mark_edge(&mut s, [0, 1], MarkType::Crease);
        assert_eq!(mark_type_at(&s, [0, 1]), Some(MarkType::Crease));
        assert_eq!(mark_type_at(&s, [5, 6]), None);
    }

    #[test]
    fn test_clear_marks() {
        let mut s = new_edge_mark_set();
        mark_edge(&mut s, [0, 1], MarkType::Seam);
        clear_marks(&mut s);
        assert!(s.marks.is_empty());
    }

    #[test]
    fn test_marks_to_json() {
        let s = new_edge_mark_set();
        let j = marks_to_json(&s);
        assert!(j.contains("mark_count"));
    }

    #[test]
    fn test_unmark_nonexistent() {
        let mut s = new_edge_mark_set();
        unmark_edge(&mut s, [0, 1]);
        assert!(s.marks.is_empty());
    }
}
