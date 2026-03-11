#![allow(dead_code)]
//! Skin cluster for skeletal mesh binding.

/// A single joint influence on a vertex.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JointInfluence {
    pub joint_index: u32,
    pub weight: f32,
}

/// A skin cluster containing per-vertex joint influences.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SkinCluster {
    pub influences: Vec<Vec<JointInfluence>>,
    pub joint_names: Vec<String>,
}

/// Create a new skin cluster for a given vertex count.
#[allow(dead_code)]
pub fn new_skin_cluster(vertex_count: usize, joint_names: Vec<String>) -> SkinCluster {
    SkinCluster {
        influences: vec![Vec::new(); vertex_count],
        joint_names,
    }
}

/// Add an influence to a vertex.
#[allow(dead_code)]
pub fn add_influence(cluster: &mut SkinCluster, vertex: usize, joint_index: u32, weight: f32) {
    if vertex < cluster.influences.len() {
        cluster.influences[vertex].push(JointInfluence { joint_index, weight });
    }
}

/// Get influences at a vertex.
#[allow(dead_code)]
pub fn influences_at_vertex(cluster: &SkinCluster, vertex: usize) -> &[JointInfluence] {
    if vertex < cluster.influences.len() {
        &cluster.influences[vertex]
    } else {
        &[]
    }
}

/// Get maximum number of influences on any single vertex.
#[allow(dead_code)]
pub fn max_influences(cluster: &SkinCluster) -> usize {
    cluster.influences.iter().map(|v| v.len()).max().unwrap_or(0)
}

/// Return joint count.
#[allow(dead_code)]
pub fn joint_count_sc(cluster: &SkinCluster) -> usize {
    cluster.joint_names.len()
}

/// Normalize all weights per vertex so they sum to 1.0.
#[allow(dead_code)]
pub fn normalize_skin_weights(cluster: &mut SkinCluster) {
    for infs in &mut cluster.influences {
        let sum: f32 = infs.iter().map(|i| i.weight).sum();
        if sum > 0.0 {
            for inf in infs.iter_mut() {
                inf.weight /= sum;
            }
        }
    }
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn skin_cluster_to_json(cluster: &SkinCluster) -> String {
    format!(
        "{{\"vertex_count\":{},\"joint_count\":{},\"max_influences\":{}}}",
        cluster.influences.len(),
        cluster.joint_names.len(),
        max_influences(cluster)
    )
}

/// Return vertex count of the skin cluster.
#[allow(dead_code)]
pub fn skin_cluster_vertex_count(cluster: &SkinCluster) -> usize {
    cluster.influences.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_skin_cluster() {
        let sc = new_skin_cluster(4, vec!["bone0".into(), "bone1".into()]);
        assert_eq!(skin_cluster_vertex_count(&sc), 4);
    }

    #[test]
    fn test_add_influence() {
        let mut sc = new_skin_cluster(2, vec!["j0".into()]);
        add_influence(&mut sc, 0, 0, 1.0);
        assert_eq!(influences_at_vertex(&sc, 0).len(), 1);
    }

    #[test]
    fn test_max_influences() {
        let mut sc = new_skin_cluster(2, vec!["j0".into(), "j1".into()]);
        add_influence(&mut sc, 0, 0, 0.5);
        add_influence(&mut sc, 0, 1, 0.5);
        add_influence(&mut sc, 1, 0, 1.0);
        assert_eq!(max_influences(&sc), 2);
    }

    #[test]
    fn test_joint_count() {
        let sc = new_skin_cluster(1, vec!["a".into(), "b".into(), "c".into()]);
        assert_eq!(joint_count_sc(&sc), 3);
    }

    #[test]
    fn test_normalize_skin_weights() {
        let mut sc = new_skin_cluster(1, vec!["j0".into(), "j1".into()]);
        add_influence(&mut sc, 0, 0, 3.0);
        add_influence(&mut sc, 0, 1, 1.0);
        normalize_skin_weights(&mut sc);
        let infs = influences_at_vertex(&sc, 0);
        let sum: f32 = infs.iter().map(|i| i.weight).sum();
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_skin_cluster_to_json() {
        let sc = new_skin_cluster(2, vec!["j0".into()]);
        let j = skin_cluster_to_json(&sc);
        assert!(j.contains("\"vertex_count\":2"));
    }

    #[test]
    fn test_influences_at_oob() {
        let sc = new_skin_cluster(1, vec![]);
        assert!(influences_at_vertex(&sc, 10).is_empty());
    }

    #[test]
    fn test_add_influence_oob() {
        let mut sc = new_skin_cluster(1, vec!["j0".into()]);
        add_influence(&mut sc, 100, 0, 1.0); // should not panic
        assert_eq!(skin_cluster_vertex_count(&sc), 1);
    }

    #[test]
    fn test_empty_cluster() {
        let sc = new_skin_cluster(0, vec![]);
        assert_eq!(max_influences(&sc), 0);
    }

    #[test]
    fn test_normalize_zero_weights() {
        let mut sc = new_skin_cluster(1, vec!["j0".into()]);
        add_influence(&mut sc, 0, 0, 0.0);
        normalize_skin_weights(&mut sc);
        assert_eq!(influences_at_vertex(&sc, 0)[0].weight, 0.0);
    }
}
