//! Scene outliner UI — hierarchical list of scene nodes with selection and visibility state.

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct OutlinerNode {
    pub id: u64,
    pub name: String,
    pub parent_id: Option<u64>,
    pub visible: bool,
    pub selected: bool,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct SceneOutlinerConfig {
    pub max_nodes: usize,
    pub single_selection: bool,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct SceneOutliner {
    pub config: SceneOutlinerConfig,
    pub nodes: Vec<OutlinerNode>,
    pub selected_id: Option<u64>,
}

#[allow(dead_code)]
pub fn default_scene_outliner_config() -> SceneOutlinerConfig {
    SceneOutlinerConfig {
        max_nodes: 4096,
        single_selection: true,
    }
}

#[allow(dead_code)]
pub fn new_scene_outliner(cfg: &SceneOutlinerConfig) -> SceneOutliner {
    SceneOutliner {
        config: cfg.clone(),
        nodes: Vec::new(),
        selected_id: None,
    }
}

#[allow(dead_code)]
pub fn outliner_add_node(
    outliner: &mut SceneOutliner,
    id: u64,
    name: &str,
    parent_id: Option<u64>,
) {
    if outliner.nodes.iter().any(|n| n.id == id) {
        return;
    }
    outliner.nodes.push(OutlinerNode {
        id,
        name: name.to_string(),
        parent_id,
        visible: true,
        selected: false,
    });
}

#[allow(dead_code)]
pub fn outliner_select_node(outliner: &mut SceneOutliner, id: u64) {
    if outliner.config.single_selection {
        for node in outliner.nodes.iter_mut() {
            node.selected = node.id == id;
        }
    } else if let Some(node) = outliner.nodes.iter_mut().find(|n| n.id == id) {
        node.selected = true;
    }
    if outliner.nodes.iter().any(|n| n.id == id) {
        outliner.selected_id = Some(id);
    }
}

#[allow(dead_code)]
pub fn outliner_toggle_visibility(outliner: &mut SceneOutliner, id: u64) {
    if let Some(node) = outliner.nodes.iter_mut().find(|n| n.id == id) {
        node.visible = !node.visible;
    }
}

#[allow(dead_code)]
pub fn outliner_node_count(outliner: &SceneOutliner) -> usize {
    outliner.nodes.len()
}

#[allow(dead_code)]
pub fn outliner_selected_node(outliner: &SceneOutliner) -> Option<u64> {
    outliner.selected_id
}

#[allow(dead_code)]
pub fn outliner_is_visible(outliner: &SceneOutliner, id: u64) -> bool {
    outliner
        .nodes
        .iter()
        .find(|n| n.id == id)
        .map(|n| n.visible)
        .unwrap_or(false)
}

#[allow(dead_code)]
pub fn outliner_children(outliner: &SceneOutliner, parent_id: u64) -> Vec<u64> {
    outliner
        .nodes
        .iter()
        .filter(|n| n.parent_id == Some(parent_id))
        .map(|n| n.id)
        .collect()
}

#[allow(dead_code)]
pub fn outliner_clear(outliner: &mut SceneOutliner) {
    outliner.nodes.clear();
    outliner.selected_id = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_outliner() -> SceneOutliner {
        let cfg = default_scene_outliner_config();
        new_scene_outliner(&cfg)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_scene_outliner_config();
        assert_eq!(cfg.max_nodes, 4096);
        assert!(cfg.single_selection);
    }

    #[test]
    fn test_new_outliner_empty() {
        let o = make_outliner();
        assert_eq!(outliner_node_count(&o), 0);
        assert!(outliner_selected_node(&o).is_none());
    }

    #[test]
    fn test_add_node() {
        let mut o = make_outliner();
        outliner_add_node(&mut o, 1, "Root", None);
        assert_eq!(outliner_node_count(&o), 1);
    }

    #[test]
    fn test_add_duplicate_id_ignored() {
        let mut o = make_outliner();
        outliner_add_node(&mut o, 1, "Root", None);
        outliner_add_node(&mut o, 1, "Root2", None);
        assert_eq!(outliner_node_count(&o), 1);
    }

    #[test]
    fn test_select_node() {
        let mut o = make_outliner();
        outliner_add_node(&mut o, 10, "Mesh", None);
        outliner_select_node(&mut o, 10);
        assert_eq!(outliner_selected_node(&o), Some(10));
    }

    #[test]
    fn test_single_selection_clears_previous() {
        let mut o = make_outliner();
        outliner_add_node(&mut o, 1, "A", None);
        outliner_add_node(&mut o, 2, "B", None);
        outliner_select_node(&mut o, 1);
        outliner_select_node(&mut o, 2);
        // node 1 should no longer be selected
        assert!(!o.nodes.iter().find(|n| n.id == 1).unwrap().selected);
        assert!(o.nodes.iter().find(|n| n.id == 2).unwrap().selected);
    }

    #[test]
    fn test_toggle_visibility() {
        let mut o = make_outliner();
        outliner_add_node(&mut o, 5, "Light", None);
        assert!(outliner_is_visible(&o, 5));
        outliner_toggle_visibility(&mut o, 5);
        assert!(!outliner_is_visible(&o, 5));
        outliner_toggle_visibility(&mut o, 5);
        assert!(outliner_is_visible(&o, 5));
    }

    #[test]
    fn test_outliner_children() {
        let mut o = make_outliner();
        outliner_add_node(&mut o, 1, "Root", None);
        outliner_add_node(&mut o, 2, "Child1", Some(1));
        outliner_add_node(&mut o, 3, "Child2", Some(1));
        let children = outliner_children(&o, 1);
        assert_eq!(children.len(), 2);
        assert!(children.contains(&2));
        assert!(children.contains(&3));
    }

    #[test]
    fn test_outliner_clear() {
        let mut o = make_outliner();
        outliner_add_node(&mut o, 1, "X", None);
        outliner_select_node(&mut o, 1);
        outliner_clear(&mut o);
        assert_eq!(outliner_node_count(&o), 0);
        assert!(outliner_selected_node(&o).is_none());
    }

    #[test]
    fn test_is_visible_missing_node() {
        let o = make_outliner();
        assert!(!outliner_is_visible(&o, 999));
    }

    #[test]
    fn test_children_of_leaf_empty() {
        let mut o = make_outliner();
        outliner_add_node(&mut o, 1, "Leaf", None);
        let children = outliner_children(&o, 1);
        assert!(children.is_empty());
    }
}
