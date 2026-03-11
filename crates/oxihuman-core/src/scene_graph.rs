//! Scene graph for hierarchical node management and transforms.

#[allow(dead_code)]
pub type NodeId = u32;

#[allow(dead_code)]
pub type VisitorFn = fn(NodeId);

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Transform {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl Transform {
    #[allow(dead_code)]
    pub fn identity() -> Self {
        Transform {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SceneNode {
    pub id: NodeId,
    pub name: String,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub local_transform: Transform,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SceneGraph {
    pub nodes: Vec<SceneNode>,
    pub next_id: NodeId,
}

#[allow(dead_code)]
pub fn new_scene_graph() -> SceneGraph {
    SceneGraph {
        nodes: Vec::new(),
        next_id: 0,
    }
}

#[allow(dead_code)]
pub fn add_node(graph: &mut SceneGraph, name: &str, parent: Option<NodeId>) -> NodeId {
    let id = graph.next_id;
    graph.next_id += 1;
    if let Some(pid) = parent {
        if let Some(pnode) = graph.nodes.iter_mut().find(|n| n.id == pid) {
            pnode.children.push(id);
        }
    }
    graph.nodes.push(SceneNode {
        id,
        name: name.to_string(),
        parent,
        children: Vec::new(),
        local_transform: Transform::identity(),
    });
    id
}

#[allow(dead_code)]
pub fn remove_node(graph: &mut SceneGraph, id: NodeId) {
    if let Some(idx) = graph.nodes.iter().position(|n| n.id == id) {
        let parent = graph.nodes[idx].parent;
        if let Some(pid) = parent {
            if let Some(pnode) = graph.nodes.iter_mut().find(|n| n.id == pid) {
                pnode.children.retain(|&c| c != id);
            }
        }
        graph.nodes.remove(idx);
    }
}

#[allow(dead_code)]
pub fn detach_node(graph: &mut SceneGraph, id: NodeId) {
    if let Some(node) = graph.nodes.iter_mut().find(|n| n.id == id) {
        node.parent = None;
    }
}

#[allow(dead_code)]
pub fn set_parent(graph: &mut SceneGraph, id: NodeId, parent: Option<NodeId>) {
    detach_node(graph, id);
    if let Some(node) = graph.nodes.iter_mut().find(|n| n.id == id) {
        node.parent = parent;
    }
    if let Some(pid) = parent {
        let has_child = graph
            .nodes
            .iter()
            .find(|n| n.id == pid)
            .map(|n| n.children.contains(&id))
            .unwrap_or(false);
        if !has_child {
            if let Some(pnode) = graph.nodes.iter_mut().find(|n| n.id == pid) {
                pnode.children.push(id);
            }
        }
    }
}

#[allow(dead_code)]
pub fn find_node_by_name<'a>(graph: &'a SceneGraph, name: &str) -> Option<&'a SceneNode> {
    graph.nodes.iter().find(|n| n.name == name)
}

#[allow(dead_code)]
pub fn node_count(graph: &SceneGraph) -> usize {
    graph.nodes.len()
}

#[allow(dead_code)]
pub fn node_children(graph: &SceneGraph, id: NodeId) -> Vec<NodeId> {
    graph
        .nodes
        .iter()
        .find(|n| n.id == id)
        .map(|n| n.children.clone())
        .unwrap_or_default()
}

#[allow(dead_code)]
pub fn node_local_transform(graph: &SceneGraph, id: NodeId) -> Option<&Transform> {
    graph.nodes.iter().find(|n| n.id == id).map(|n| &n.local_transform)
}

#[allow(dead_code)]
pub fn node_world_transform(graph: &SceneGraph, id: NodeId) -> Transform {
    // Simple: walk up the hierarchy and combine positions (no full matrix math).
    let mut pos = [0.0f32; 3];
    let mut current_id = Some(id);
    while let Some(cid) = current_id {
        if let Some(node) = graph.nodes.iter().find(|n| n.id == cid) {
            for (p, t) in pos.iter_mut().zip(node.local_transform.position.iter()) {
                *p += t;
            }
            current_id = node.parent;
        } else {
            break;
        }
    }
    Transform {
        position: pos,
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: [1.0, 1.0, 1.0],
    }
}

#[allow(dead_code)]
pub fn set_local_transform(graph: &mut SceneGraph, id: NodeId, t: Transform) {
    if let Some(node) = graph.nodes.iter_mut().find(|n| n.id == id) {
        node.local_transform = t;
    }
}

#[allow(dead_code)]
pub fn graph_depth(graph: &SceneGraph, id: NodeId) -> usize {
    let mut depth = 0usize;
    let mut current_id = Some(id);
    while let Some(cid) = current_id {
        if let Some(node) = graph.nodes.iter().find(|n| n.id == cid) {
            current_id = node.parent;
            if current_id.is_some() {
                depth += 1;
            }
        } else {
            break;
        }
    }
    depth
}

#[allow(dead_code)]
pub fn traverse_depth_first(graph: &SceneGraph, start: NodeId, visitor: VisitorFn) {
    visitor(start);
    let children = node_children(graph, start);
    for child in children {
        traverse_depth_first(graph, child, visitor);
    }
}

#[allow(dead_code)]
pub fn scene_graph_to_json(graph: &SceneGraph) -> String {
    let nodes_json: Vec<String> = graph
        .nodes
        .iter()
        .map(|n| {
            let parent_str = match n.parent {
                Some(p) => format!("{}", p),
                None => "null".to_string(),
            };
            format!(r#"{{"id":{},"name":"{}","parent":{}}}"#, n.id, n.name, parent_str)
        })
        .collect();
    format!(
        r#"{{"node_count":{},"nodes":[{}]}}"#,
        graph.nodes.len(),
        nodes_json.join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_graph_empty() {
        let g = new_scene_graph();
        assert_eq!(node_count(&g), 0);
    }

    #[test]
    fn test_add_node() {
        let mut g = new_scene_graph();
        let id = add_node(&mut g, "root", None);
        assert_eq!(node_count(&g), 1);
        assert_eq!(id, 0);
    }

    #[test]
    fn test_find_by_name() {
        let mut g = new_scene_graph();
        add_node(&mut g, "hip", None);
        let n = find_node_by_name(&g, "hip");
        assert!(n.is_some());
        assert!(find_node_by_name(&g, "spine").is_none());
    }

    #[test]
    fn test_remove_node() {
        let mut g = new_scene_graph();
        let id = add_node(&mut g, "temp", None);
        remove_node(&mut g, id);
        assert_eq!(node_count(&g), 0);
    }

    #[test]
    fn test_to_json() {
        let mut g = new_scene_graph();
        add_node(&mut g, "root", None);
        let j = scene_graph_to_json(&g);
        assert!(j.contains("\"node_count\":1"));
    }
}
