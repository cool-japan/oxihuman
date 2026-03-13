// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Skin cluster export: joint influence data per vertex.

/// A single joint influence.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JointInfluence {
    pub joint_index: u32,
    pub weight: f32,
}

/// Skin cluster export data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SkinClusterExport {
    pub name: String,
    pub joint_names: Vec<String>,
    pub vertex_influences: Vec<Vec<JointInfluence>>,
    pub max_influences: usize,
}

#[allow(dead_code)]
pub fn new_skin_cluster_export(name: &str, max_inf: usize) -> SkinClusterExport {
    SkinClusterExport {
        name: name.to_string(),
        joint_names: Vec::new(),
        vertex_influences: Vec::new(),
        max_influences: max_inf,
    }
}

#[allow(dead_code)]
pub fn sc_add_joint(e: &mut SkinClusterExport, name: &str) -> u32 {
    let idx = e.joint_names.len() as u32;
    e.joint_names.push(name.to_string());
    idx
}

#[allow(dead_code)]
pub fn sc_joint_count(e: &SkinClusterExport) -> usize {
    e.joint_names.len()
}

#[allow(dead_code)]
pub fn sc_add_vertex(e: &mut SkinClusterExport, influences: &[(u32, f32)]) {
    let mut infs: Vec<JointInfluence> = influences
        .iter()
        .map(|&(ji, w)| JointInfluence {
            joint_index: ji,
            weight: w,
        })
        .collect();
    infs.sort_by(|a, b| {
        b.weight
            .partial_cmp(&a.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    infs.truncate(e.max_influences);
    e.vertex_influences.push(infs);
}

#[allow(dead_code)]
pub fn sc_vertex_count(e: &SkinClusterExport) -> usize {
    e.vertex_influences.len()
}

#[allow(dead_code)]
pub fn sc_get_influences(e: &SkinClusterExport, vertex: usize) -> Option<&[JointInfluence]> {
    e.vertex_influences.get(vertex).map(|v| v.as_slice())
}

#[allow(dead_code)]
pub fn sc_normalize_weights(e: &mut SkinClusterExport) {
    for infs in &mut e.vertex_influences {
        let total: f32 = infs.iter().map(|i| i.weight).sum();
        if total > 1e-12 {
            for inf in infs.iter_mut() {
                inf.weight /= total;
            }
        }
    }
}

#[allow(dead_code)]
pub fn sc_max_influence_count(e: &SkinClusterExport) -> usize {
    e.vertex_influences
        .iter()
        .map(|v| v.len())
        .max()
        .unwrap_or(0)
}

#[allow(dead_code)]
pub fn sc_validate(e: &SkinClusterExport) -> bool {
    let jc = e.joint_names.len() as u32;
    e.vertex_influences.iter().all(|infs| {
        infs.iter()
            .all(|i| i.joint_index < jc && (0.0..=1.0).contains(&i.weight))
    })
}

#[allow(dead_code)]
pub fn skin_cluster_to_json(e: &SkinClusterExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"joints\":{},\"vertices\":{},\"max_influences\":{}}}",
        e.name,
        e.joint_names.len(),
        e.vertex_influences.len(),
        e.max_influences
    )
}

#[allow(dead_code)]
pub fn sc_to_csv(e: &SkinClusterExport) -> String {
    let mut s = String::from("vertex,joint_index,weight\n");
    for (vi, infs) in e.vertex_influences.iter().enumerate() {
        for inf in infs {
            s.push_str(&format!("{},{},{:.6}\n", vi, inf.joint_index, inf.weight));
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let e = new_skin_cluster_export("body", 4);
        assert_eq!(e.max_influences, 4);
        assert!(e.joint_names.is_empty());
    }

    #[test]
    fn test_add_joint() {
        let mut e = new_skin_cluster_export("body", 4);
        let idx = sc_add_joint(&mut e, "spine");
        assert_eq!(idx, 0);
        assert_eq!(sc_joint_count(&e), 1);
    }

    #[test]
    fn test_add_vertex() {
        let mut e = new_skin_cluster_export("body", 4);
        sc_add_joint(&mut e, "a");
        sc_add_vertex(&mut e, &[(0, 1.0)]);
        assert_eq!(sc_vertex_count(&e), 1);
    }

    #[test]
    fn test_get_influences() {
        let mut e = new_skin_cluster_export("body", 4);
        sc_add_joint(&mut e, "a");
        sc_add_joint(&mut e, "b");
        sc_add_vertex(&mut e, &[(0, 0.3), (1, 0.7)]);
        let infs = sc_get_influences(&e, 0).expect("should succeed");
        assert_eq!(infs.len(), 2);
        assert!((infs[0].weight - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_truncate_influences() {
        let mut e = new_skin_cluster_export("body", 2);
        for i in 0..5 {
            sc_add_joint(&mut e, &format!("j{}", i));
        }
        sc_add_vertex(
            &mut e,
            &[(0, 0.1), (1, 0.2), (2, 0.3), (3, 0.25), (4, 0.15)],
        );
        let infs = sc_get_influences(&e, 0).expect("should succeed");
        assert_eq!(infs.len(), 2);
    }

    #[test]
    fn test_normalize() {
        let mut e = new_skin_cluster_export("body", 4);
        sc_add_joint(&mut e, "a");
        sc_add_joint(&mut e, "b");
        sc_add_vertex(&mut e, &[(0, 2.0), (1, 3.0)]);
        sc_normalize_weights(&mut e);
        let infs = sc_get_influences(&e, 0).expect("should succeed");
        let total: f32 = infs.iter().map(|i| i.weight).sum();
        assert!((total - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_max_influence_count() {
        let mut e = new_skin_cluster_export("body", 4);
        sc_add_joint(&mut e, "a");
        sc_add_joint(&mut e, "b");
        sc_add_vertex(&mut e, &[(0, 1.0)]);
        sc_add_vertex(&mut e, &[(0, 0.5), (1, 0.5)]);
        assert_eq!(sc_max_influence_count(&e), 2);
    }

    #[test]
    fn test_validate_ok() {
        let mut e = new_skin_cluster_export("body", 4);
        sc_add_joint(&mut e, "a");
        sc_add_vertex(&mut e, &[(0, 0.5)]);
        assert!(sc_validate(&e));
    }

    #[test]
    fn test_validate_bad_index() {
        let mut e = new_skin_cluster_export("body", 4);
        sc_add_joint(&mut e, "a");
        e.vertex_influences.push(vec![JointInfluence {
            joint_index: 99,
            weight: 1.0,
        }]);
        assert!(!sc_validate(&e));
    }

    #[test]
    fn test_to_json() {
        let e = new_skin_cluster_export("body", 4);
        assert!(skin_cluster_to_json(&e).contains("\"name\":\"body\""));
    }

    #[test]
    fn test_to_csv() {
        let mut e = new_skin_cluster_export("body", 4);
        sc_add_joint(&mut e, "a");
        sc_add_vertex(&mut e, &[(0, 0.5)]);
        let csv = sc_to_csv(&e);
        assert!(csv.contains("vertex,joint_index,weight"));
        assert!(csv.contains("0,0,"));
    }
}
