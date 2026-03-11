#![allow(dead_code)]
//! Render layer sort: sorts render layers by priority.

/// A named render layer with a priority.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct RenderLayer {
    name: String,
    priority: i32,
}

/// A sorted collection of render layers.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderLayerSort {
    layers: Vec<RenderLayer>,
}

/// Create a new empty layer sort.
#[allow(dead_code)]
pub fn new_layer_sort() -> RenderLayerSort {
    RenderLayerSort {
        layers: Vec::new(),
    }
}

/// Add a layer.
#[allow(dead_code)]
pub fn add_layer_ls(sort: &mut RenderLayerSort, name: &str, priority: i32) {
    sort.layers.push(RenderLayer {
        name: name.to_string(),
        priority,
    });
}

/// Sort layers by priority (ascending).
#[allow(dead_code)]
pub fn sort_layers(sort: &mut RenderLayerSort) {
    sort.layers.sort_by_key(|l| l.priority);
}

/// Return the number of layers.
#[allow(dead_code)]
pub fn layer_count_rls(sort: &RenderLayerSort) -> usize {
    sort.layers.len()
}

/// Return the name of the layer at `index`.
#[allow(dead_code)]
pub fn layer_at_rls(sort: &RenderLayerSort, index: usize) -> &str {
    sort.layers.get(index).map_or("", |l| &l.name)
}

/// Return the priority of the layer at `index`.
#[allow(dead_code)]
pub fn layer_priority_rls(sort: &RenderLayerSort, index: usize) -> i32 {
    sort.layers.get(index).map_or(0, |l| l.priority)
}

/// Serialize to JSON-like string.
#[allow(dead_code)]
pub fn layers_to_json(sort: &RenderLayerSort) -> String {
    let entries: Vec<String> = sort
        .layers
        .iter()
        .map(|l| format!("{{\"name\":\"{}\",\"priority\":{}}}", l.name, l.priority))
        .collect();
    format!("{{\"layers\":[{}]}}", entries.join(","))
}

/// Clear all layers.
#[allow(dead_code)]
pub fn clear_layers_rls(sort: &mut RenderLayerSort) {
    sort.layers.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_sort() {
        let s = new_layer_sort();
        assert_eq!(layer_count_rls(&s), 0);
    }

    #[test]
    fn test_add_layer() {
        let mut s = new_layer_sort();
        add_layer_ls(&mut s, "opaque", 0);
        assert_eq!(layer_count_rls(&s), 1);
    }

    #[test]
    fn test_sort_layers() {
        let mut s = new_layer_sort();
        add_layer_ls(&mut s, "transparent", 10);
        add_layer_ls(&mut s, "opaque", 0);
        sort_layers(&mut s);
        assert_eq!(layer_at_rls(&s, 0), "opaque");
    }

    #[test]
    fn test_layer_at() {
        let mut s = new_layer_sort();
        add_layer_ls(&mut s, "bg", 0);
        assert_eq!(layer_at_rls(&s, 0), "bg");
        assert_eq!(layer_at_rls(&s, 99), "");
    }

    #[test]
    fn test_layer_priority() {
        let mut s = new_layer_sort();
        add_layer_ls(&mut s, "fg", 5);
        assert_eq!(layer_priority_rls(&s, 0), 5);
    }

    #[test]
    fn test_to_json() {
        let s = new_layer_sort();
        let json = layers_to_json(&s);
        assert!(json.contains("\"layers\":[]"));
    }

    #[test]
    fn test_clear() {
        let mut s = new_layer_sort();
        add_layer_ls(&mut s, "a", 0);
        clear_layers_rls(&mut s);
        assert_eq!(layer_count_rls(&s), 0);
    }

    #[test]
    fn test_multiple_layers() {
        let mut s = new_layer_sort();
        add_layer_ls(&mut s, "a", 2);
        add_layer_ls(&mut s, "b", 1);
        add_layer_ls(&mut s, "c", 3);
        sort_layers(&mut s);
        assert_eq!(layer_at_rls(&s, 0), "b");
        assert_eq!(layer_at_rls(&s, 2), "c");
    }

    #[test]
    fn test_priority_out_of_range() {
        let s = new_layer_sort();
        assert_eq!(layer_priority_rls(&s, 0), 0);
    }

    #[test]
    fn test_sort_empty() {
        let mut s = new_layer_sort();
        sort_layers(&mut s);
        assert_eq!(layer_count_rls(&s), 0);
    }
}
