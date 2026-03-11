#![allow(dead_code)]
//! Renders a scene graph by traversing nodes.

#[allow(dead_code)]
#[derive(Clone, Debug)]
struct SceneNode {
    name: String,
    visible: bool,
    children: Vec<usize>,
    draw_calls: u32,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct SceneGraphRenderer {
    nodes: Vec<SceneNode>,
    total_draws: u32,
}

#[allow(dead_code)]
pub fn new_scene_graph_renderer() -> SceneGraphRenderer {
    SceneGraphRenderer {
        nodes: Vec::new(),
        total_draws: 0,
    }
}

#[allow(dead_code)]
pub fn submit_node(r: &mut SceneGraphRenderer, name: &str, visible: bool, draw_calls: u32) -> usize {
    let idx = r.nodes.len();
    r.nodes.push(SceneNode {
        name: name.to_string(),
        visible,
        children: Vec::new(),
        draw_calls,
    });
    idx
}

#[allow(dead_code)]
pub fn traverse_and_render(r: &mut SceneGraphRenderer) -> u32 {
    let mut total = 0_u32;
    for node in &r.nodes {
        if node.visible {
            total += node.draw_calls;
        }
    }
    r.total_draws = total;
    total
}

#[allow(dead_code)]
pub fn node_draw_count(r: &SceneGraphRenderer, index: usize) -> u32 {
    r.nodes.get(index).map(|n| n.draw_calls).unwrap_or(0)
}

#[allow(dead_code)]
pub fn total_draw_calls(r: &SceneGraphRenderer) -> u32 {
    r.total_draws
}

#[allow(dead_code)]
pub fn visible_node_count(r: &SceneGraphRenderer) -> usize {
    r.nodes.iter().filter(|n| n.visible).count()
}

#[allow(dead_code)]
pub fn scene_graph_stats(r: &SceneGraphRenderer) -> String {
    format!(
        "nodes={}, visible={}, draws={}",
        r.nodes.len(),
        visible_node_count(r),
        r.total_draws
    )
}

#[allow(dead_code)]
pub fn reset_graph_renderer(r: &mut SceneGraphRenderer) {
    r.nodes.clear();
    r.total_draws = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_scene_graph_renderer() {
        let r = new_scene_graph_renderer();
        assert_eq!(r.nodes.len(), 0);
    }

    #[test]
    fn test_submit_node() {
        let mut r = new_scene_graph_renderer();
        let idx = submit_node(&mut r, "mesh", true, 5);
        assert_eq!(idx, 0);
    }

    #[test]
    fn test_traverse_and_render() {
        let mut r = new_scene_graph_renderer();
        submit_node(&mut r, "a", true, 3);
        submit_node(&mut r, "b", true, 2);
        let draws = traverse_and_render(&mut r);
        assert_eq!(draws, 5);
    }

    #[test]
    fn test_invisible_nodes() {
        let mut r = new_scene_graph_renderer();
        submit_node(&mut r, "a", true, 3);
        submit_node(&mut r, "b", false, 10);
        let draws = traverse_and_render(&mut r);
        assert_eq!(draws, 3);
    }

    #[test]
    fn test_node_draw_count() {
        let mut r = new_scene_graph_renderer();
        submit_node(&mut r, "n", true, 7);
        assert_eq!(node_draw_count(&r, 0), 7);
    }

    #[test]
    fn test_visible_node_count() {
        let mut r = new_scene_graph_renderer();
        submit_node(&mut r, "a", true, 1);
        submit_node(&mut r, "b", false, 1);
        assert_eq!(visible_node_count(&r), 1);
    }

    #[test]
    fn test_scene_graph_stats() {
        let mut r = new_scene_graph_renderer();
        submit_node(&mut r, "a", true, 1);
        traverse_and_render(&mut r);
        let s = scene_graph_stats(&r);
        assert!(s.contains("nodes=1"));
    }

    #[test]
    fn test_reset_graph_renderer() {
        let mut r = new_scene_graph_renderer();
        submit_node(&mut r, "a", true, 1);
        reset_graph_renderer(&mut r);
        assert_eq!(r.nodes.len(), 0);
    }

    #[test]
    fn test_total_draw_calls() {
        let mut r = new_scene_graph_renderer();
        submit_node(&mut r, "a", true, 5);
        traverse_and_render(&mut r);
        assert_eq!(total_draw_calls(&r), 5);
    }

    #[test]
    fn test_node_draw_count_invalid() {
        let r = new_scene_graph_renderer();
        assert_eq!(node_draw_count(&r, 99), 0);
    }
}
