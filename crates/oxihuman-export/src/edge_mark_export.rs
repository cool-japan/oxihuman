// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Marked edge export: seam, sharp, crease and freestyle markings.

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EdgeFlags {
    pub is_seam: bool,
    pub is_sharp: bool,
    pub is_crease: bool,
    pub is_freestyle: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MarkedEdge {
    pub v0: u32,
    pub v1: u32,
    pub flags: EdgeFlags,
    pub crease_value: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeMarkExport {
    pub edges: Vec<MarkedEdge>,
}

#[allow(dead_code)]
pub fn new_edge_mark_export() -> EdgeMarkExport {
    EdgeMarkExport { edges: Vec::new() }
}

#[allow(dead_code)]
pub fn add_marked_edge(exp: &mut EdgeMarkExport, v0: u32, v1: u32, flags: EdgeFlags) {
    exp.edges.push(MarkedEdge {
        v0,
        v1,
        flags,
        crease_value: 0.0,
    });
}

#[allow(dead_code)]
pub fn marked_edge_count(exp: &EdgeMarkExport) -> usize {
    exp.edges.len()
}

#[allow(dead_code)]
pub fn seam_edge_count_em(exp: &EdgeMarkExport) -> usize {
    exp.edges.iter().filter(|e| e.flags.is_seam).count()
}

#[allow(dead_code)]
pub fn sharp_edge_count_em(exp: &EdgeMarkExport) -> usize {
    exp.edges.iter().filter(|e| e.flags.is_sharp).count()
}

#[allow(dead_code)]
pub fn crease_edge_count_em(exp: &EdgeMarkExport) -> usize {
    exp.edges.iter().filter(|e| e.flags.is_crease).count()
}

#[allow(dead_code)]
pub fn set_crease_value(exp: &mut EdgeMarkExport, v0: u32, v1: u32, value: f32) {
    for e in &mut exp.edges {
        let (ea, eb) = if e.v0 < e.v1 {
            (e.v0, e.v1)
        } else {
            (e.v1, e.v0)
        };
        let (qa, qb) = if v0 < v1 { (v0, v1) } else { (v1, v0) };
        if ea == qa && eb == qb {
            e.crease_value = value.clamp(0.0, 1.0);
        }
    }
}

#[allow(dead_code)]
pub fn edge_mark_to_json(exp: &EdgeMarkExport) -> String {
    format!(
        "{{\"edge_count\":{},\"seams\":{},\"sharp\":{}}}",
        marked_edge_count(exp),
        seam_edge_count_em(exp),
        sharp_edge_count_em(exp),
    )
}

#[allow(dead_code)]
pub fn avg_crease_value(exp: &EdgeMarkExport) -> f32 {
    let creased: Vec<f32> = exp
        .edges
        .iter()
        .filter(|e| e.flags.is_crease)
        .map(|e| e.crease_value)
        .collect();
    if creased.is_empty() {
        return 0.0;
    }
    creased.iter().sum::<f32>() / creased.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let exp = new_edge_mark_export();
        assert_eq!(marked_edge_count(&exp), 0);
    }

    #[test]
    fn test_add_edge() {
        let mut exp = new_edge_mark_export();
        add_marked_edge(
            &mut exp,
            0,
            1,
            EdgeFlags {
                is_seam: true,
                ..Default::default()
            },
        );
        assert_eq!(marked_edge_count(&exp), 1);
    }

    #[test]
    fn test_seam_count() {
        let mut exp = new_edge_mark_export();
        add_marked_edge(
            &mut exp,
            0,
            1,
            EdgeFlags {
                is_seam: true,
                ..Default::default()
            },
        );
        add_marked_edge(&mut exp, 1, 2, EdgeFlags::default());
        assert_eq!(seam_edge_count_em(&exp), 1);
    }

    #[test]
    fn test_sharp_count() {
        let mut exp = new_edge_mark_export();
        add_marked_edge(
            &mut exp,
            0,
            1,
            EdgeFlags {
                is_sharp: true,
                ..Default::default()
            },
        );
        assert_eq!(sharp_edge_count_em(&exp), 1);
    }

    #[test]
    fn test_crease_count() {
        let mut exp = new_edge_mark_export();
        add_marked_edge(
            &mut exp,
            0,
            1,
            EdgeFlags {
                is_crease: true,
                ..Default::default()
            },
        );
        assert_eq!(crease_edge_count_em(&exp), 1);
    }

    #[test]
    fn test_set_crease_value() {
        let mut exp = new_edge_mark_export();
        add_marked_edge(
            &mut exp,
            0,
            1,
            EdgeFlags {
                is_crease: true,
                ..Default::default()
            },
        );
        set_crease_value(&mut exp, 0, 1, 0.8);
        assert!((exp.edges[0].crease_value - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_json_output() {
        let exp = new_edge_mark_export();
        let j = edge_mark_to_json(&exp);
        assert!(j.contains("edge_count"));
    }

    #[test]
    fn test_avg_crease_empty() {
        let exp = new_edge_mark_export();
        assert!((avg_crease_value(&exp)).abs() < 1e-6);
    }

    #[test]
    fn test_avg_crease_value() {
        let mut exp = new_edge_mark_export();
        add_marked_edge(
            &mut exp,
            0,
            1,
            EdgeFlags {
                is_crease: true,
                ..Default::default()
            },
        );
        set_crease_value(&mut exp, 0, 1, 1.0);
        assert!((avg_crease_value(&exp) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_default_crease_zero() {
        let mut exp = new_edge_mark_export();
        add_marked_edge(&mut exp, 5, 6, EdgeFlags::default());
        assert!((exp.edges[0].crease_value).abs() < 1e-6);
    }
}
