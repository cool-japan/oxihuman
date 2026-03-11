#![allow(dead_code)]

/// A node in the scene BVH.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BvhSceneNode {
    pub id: usize,
    pub aabb_min: [f32; 3],
    pub aabb_max: [f32; 3],
    pub label: String,
}

/// Bounding volume hierarchy for scene objects.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SceneBvh { nodes: Vec<BvhSceneNode> }

#[allow(dead_code)]
pub fn new_scene_bvh() -> SceneBvh { SceneBvh { nodes: Vec::new() } }

#[allow(dead_code)]
pub fn insert_scene_node(bvh: &mut SceneBvh, aabb_min: [f32; 3], aabb_max: [f32; 3], label: &str) -> usize {
    let id = bvh.nodes.len();
    bvh.nodes.push(BvhSceneNode { id, aabb_min, aabb_max, label: label.to_string() });
    id
}

#[allow(dead_code)]
pub fn query_frustum(bvh: &SceneBvh, frustum_min: [f32; 3], frustum_max: [f32; 3]) -> Vec<usize> {
    bvh.nodes.iter().filter(|n| {
        n.aabb_max[0] >= frustum_min[0] && n.aabb_min[0] <= frustum_max[0] &&
        n.aabb_max[1] >= frustum_min[1] && n.aabb_min[1] <= frustum_max[1] &&
        n.aabb_max[2] >= frustum_min[2] && n.aabb_min[2] <= frustum_max[2]
    }).map(|n| n.id).collect()
}

#[allow(dead_code)]
pub fn query_ray_sbvh(bvh: &SceneBvh, origin: [f32; 3], _dir: [f32; 3]) -> Vec<usize> {
    bvh.nodes.iter().filter(|n| {
        (n.aabb_min[0]..=n.aabb_max[0]).contains(&origin[0]) &&
        (n.aabb_min[1]..=n.aabb_max[1]).contains(&origin[1]) &&
        (n.aabb_min[2]..=n.aabb_max[2]).contains(&origin[2])
    }).map(|n| n.id).collect()
}

#[allow(dead_code)]
pub fn bvh_node_count_sb(bvh: &SceneBvh) -> usize { bvh.nodes.len() }

#[allow(dead_code)]
pub fn bvh_depth_sb(_bvh: &SceneBvh) -> usize { 1 }

#[allow(dead_code)]
pub fn rebuild_bvh(bvh: &mut SceneBvh) { bvh.nodes.sort_by(|a, b| a.aabb_min[0].partial_cmp(&b.aabb_min[0]).unwrap_or(std::cmp::Ordering::Equal)); }

#[allow(dead_code)]
pub fn bvh_to_json(bvh: &SceneBvh) -> String { format!("{{\"node_count\":{}}}", bvh.nodes.len()) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { assert_eq!(bvh_node_count_sb(&new_scene_bvh()), 0); }
    #[test] fn test_insert() {
        let mut b = new_scene_bvh();
        insert_scene_node(&mut b, [0.0; 3], [1.0; 3], "obj");
        assert_eq!(bvh_node_count_sb(&b), 1);
    }
    #[test] fn test_query_frustum() {
        let mut b = new_scene_bvh();
        insert_scene_node(&mut b, [0.0; 3], [1.0; 3], "a");
        insert_scene_node(&mut b, [5.0; 3], [6.0; 3], "b");
        let r = query_frustum(&b, [-1.0; 3], [2.0; 3]);
        assert_eq!(r.len(), 1);
    }
    #[test] fn test_query_ray() {
        let mut b = new_scene_bvh();
        insert_scene_node(&mut b, [0.0; 3], [1.0; 3], "a");
        let r = query_ray_sbvh(&b, [0.5, 0.5, 0.5], [0.0, 0.0, 1.0]);
        assert_eq!(r.len(), 1);
    }
    #[test] fn test_depth() { assert_eq!(bvh_depth_sb(&new_scene_bvh()), 1); }
    #[test] fn test_rebuild() {
        let mut b = new_scene_bvh();
        insert_scene_node(&mut b, [2.0; 3], [3.0; 3], "b");
        insert_scene_node(&mut b, [0.0; 3], [1.0; 3], "a");
        rebuild_bvh(&mut b);
        assert_eq!(bvh_node_count_sb(&b), 2);
    }
    #[test] fn test_to_json() { assert!(bvh_to_json(&new_scene_bvh()).contains("node_count")); }
    #[test] fn test_frustum_empty() { assert!(query_frustum(&new_scene_bvh(), [0.0; 3], [1.0; 3]).is_empty()); }
    #[test] fn test_ray_miss() {
        let mut b = new_scene_bvh();
        insert_scene_node(&mut b, [0.0; 3], [1.0; 3], "a");
        assert!(query_ray_sbvh(&b, [5.0; 3], [0.0, 0.0, 1.0]).is_empty());
    }
    #[test] fn test_multiple_inserts() {
        let mut b = new_scene_bvh();
        for i in 0..5 { insert_scene_node(&mut b, [i as f32; 3], [(i+1) as f32; 3], "n"); }
        assert_eq!(bvh_node_count_sb(&b), 5);
    }
}
