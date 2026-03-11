// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

/// Export edge selection data (seams, creases, sharp edges).
#[allow(dead_code)]
pub struct SelectedEdge {
    pub v0: u32,
    pub v1: u32,
    pub flags: EdgeFlags,
}

#[allow(dead_code)]
pub struct EdgeFlags {
    pub is_seam: bool,
    pub is_sharp: bool,
    pub is_crease: bool,
    pub crease_value: f32,
}

#[allow(dead_code)]
pub struct EdgeSelectExport {
    pub edges: Vec<SelectedEdge>,
}

#[allow(dead_code)]
pub fn new_edge_select_export() -> EdgeSelectExport {
    EdgeSelectExport { edges: vec![] }
}

#[allow(dead_code)]
pub fn add_selected_edge(export: &mut EdgeSelectExport, edge: SelectedEdge) {
    export.edges.push(edge);
}

#[allow(dead_code)]
pub fn selected_edge_count(export: &EdgeSelectExport) -> usize {
    export.edges.len()
}

#[allow(dead_code)]
pub fn seam_count(export: &EdgeSelectExport) -> usize {
    export.edges.iter().filter(|e| e.flags.is_seam).count()
}

#[allow(dead_code)]
pub fn sharp_count(export: &EdgeSelectExport) -> usize {
    export.edges.iter().filter(|e| e.flags.is_sharp).count()
}

#[allow(dead_code)]
pub fn crease_count(export: &EdgeSelectExport) -> usize {
    export.edges.iter().filter(|e| e.flags.is_crease).count()
}

#[allow(dead_code)]
pub fn average_crease_value(export: &EdgeSelectExport) -> f32 {
    let crease_edges: Vec<f32> = export
        .edges
        .iter()
        .filter(|e| e.flags.is_crease)
        .map(|e| e.flags.crease_value)
        .collect();
    if crease_edges.is_empty() {
        return 0.0;
    }
    crease_edges.iter().sum::<f32>() / crease_edges.len() as f32
}

#[allow(dead_code)]
pub fn edge_select_to_json(export: &EdgeSelectExport) -> String {
    format!(
        "{{\"total\":{},\"seams\":{},\"sharp\":{},\"crease\":{}}}",
        export.edges.len(),
        seam_count(export),
        sharp_count(export),
        crease_count(export)
    )
}

#[allow(dead_code)]
pub fn build_edge_index(export: &EdgeSelectExport) -> HashMap<(u32, u32), usize> {
    let mut map = HashMap::new();
    for (i, e) in export.edges.iter().enumerate() {
        let key = if e.v0 < e.v1 {
            (e.v0, e.v1)
        } else {
            (e.v1, e.v0)
        };
        map.insert(key, i);
    }
    map
}

#[allow(dead_code)]
pub fn has_edge(export: &EdgeSelectExport, v0: u32, v1: u32) -> bool {
    let key = if v0 < v1 { (v0, v1) } else { (v1, v0) };
    export.edges.iter().any(|e| {
        let ek = if e.v0 < e.v1 {
            (e.v0, e.v1)
        } else {
            (e.v1, e.v0)
        };
        ek == key
    })
}

#[allow(dead_code)]
pub fn validate_edge_export(export: &EdgeSelectExport) -> bool {
    export
        .edges
        .iter()
        .all(|e| e.v0 != e.v1 && (0.0..=1.0).contains(&e.flags.crease_value))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_export() -> EdgeSelectExport {
        let mut e = new_edge_select_export();
        add_selected_edge(
            &mut e,
            SelectedEdge {
                v0: 0,
                v1: 1,
                flags: EdgeFlags {
                    is_seam: true,
                    is_sharp: false,
                    is_crease: false,
                    crease_value: 0.0,
                },
            },
        );
        add_selected_edge(
            &mut e,
            SelectedEdge {
                v0: 1,
                v1: 2,
                flags: EdgeFlags {
                    is_seam: false,
                    is_sharp: true,
                    is_crease: true,
                    crease_value: 0.8,
                },
            },
        );
        e
    }

    #[test]
    fn test_selected_edge_count() {
        let e = sample_export();
        assert_eq!(selected_edge_count(&e), 2);
    }

    #[test]
    fn test_seam_count() {
        let e = sample_export();
        assert_eq!(seam_count(&e), 1);
    }

    #[test]
    fn test_sharp_count() {
        let e = sample_export();
        assert_eq!(sharp_count(&e), 1);
    }

    #[test]
    fn test_crease_count() {
        let e = sample_export();
        assert_eq!(crease_count(&e), 1);
    }

    #[test]
    fn test_average_crease_value() {
        let e = sample_export();
        assert!((average_crease_value(&e) - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_has_edge() {
        let e = sample_export();
        assert!(has_edge(&e, 0, 1));
        assert!(!has_edge(&e, 3, 4));
    }

    #[test]
    fn test_validate_export() {
        let e = sample_export();
        assert!(validate_edge_export(&e));
    }

    #[test]
    fn test_to_json() {
        let e = sample_export();
        let j = edge_select_to_json(&e);
        assert!(j.contains("seams"));
    }

    #[test]
    fn test_build_edge_index() {
        let e = sample_export();
        let idx = build_edge_index(&e);
        assert!(idx.contains_key(&(0, 1)));
    }

    #[test]
    fn test_empty_average_crease() {
        let e = new_edge_select_export();
        assert_eq!(average_crease_value(&e), 0.0);
    }
}
