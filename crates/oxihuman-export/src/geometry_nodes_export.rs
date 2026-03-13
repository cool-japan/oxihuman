//! Geometry node graph export (Blender-style procedural geometry graph description).

#[allow(dead_code)]
#[derive(Clone)]
pub enum GeoNodeType {
    MeshPrimitive,
    Transform,
    JoinGeometry,
    SeparateGeometry,
    Attribute,
    Math,
    Compare,
    Switch,
    Output,
}

impl GeoNodeType {
    fn type_name(&self) -> &'static str {
        match self {
            GeoNodeType::MeshPrimitive => "MeshPrimitive",
            GeoNodeType::Transform => "Transform",
            GeoNodeType::JoinGeometry => "JoinGeometry",
            GeoNodeType::SeparateGeometry => "SeparateGeometry",
            GeoNodeType::Attribute => "Attribute",
            GeoNodeType::Math => "Math",
            GeoNodeType::Compare => "Compare",
            GeoNodeType::Switch => "Switch",
            GeoNodeType::Output => "Output",
        }
    }

    fn discriminant(&self) -> u8 {
        match self {
            GeoNodeType::MeshPrimitive => 0,
            GeoNodeType::Transform => 1,
            GeoNodeType::JoinGeometry => 2,
            GeoNodeType::SeparateGeometry => 3,
            GeoNodeType::Attribute => 4,
            GeoNodeType::Math => 5,
            GeoNodeType::Compare => 6,
            GeoNodeType::Switch => 7,
            GeoNodeType::Output => 8,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct GeoNodeSocket {
    pub name: String,
    pub socket_type: String,
    pub default_value: String,
}

#[allow(dead_code)]
pub struct GeoNode {
    pub id: u32,
    pub name: String,
    pub node_type: GeoNodeType,
    pub inputs: Vec<GeoNodeSocket>,
    pub outputs: Vec<GeoNodeSocket>,
    pub position: [f32; 2],
}

#[allow(dead_code)]
pub struct GeoNodeLink {
    pub from_node: u32,
    pub from_socket: usize,
    pub to_node: u32,
    pub to_socket: usize,
}

#[allow(dead_code)]
pub struct GeoNodeGraph {
    pub name: String,
    pub nodes: Vec<GeoNode>,
    pub links: Vec<GeoNodeLink>,
    pub next_id: u32,
}

#[allow(dead_code)]
pub fn new_geo_graph(name: &str) -> GeoNodeGraph {
    GeoNodeGraph {
        name: name.to_string(),
        nodes: Vec::new(),
        links: Vec::new(),
        next_id: 1,
    }
}

#[allow(dead_code)]
pub fn add_geo_node(graph: &mut GeoNodeGraph, name: &str, node_type: GeoNodeType) -> u32 {
    let id = graph.next_id;
    graph.next_id += 1;
    let x = id as f32 * 200.0;
    graph.nodes.push(GeoNode {
        id,
        name: name.to_string(),
        node_type,
        inputs: Vec::new(),
        outputs: Vec::new(),
        position: [x, 0.0],
    });
    id
}

#[allow(dead_code)]
pub fn add_geo_link(
    graph: &mut GeoNodeGraph,
    from: u32,
    from_sock: usize,
    to: u32,
    to_sock: usize,
) {
    graph.links.push(GeoNodeLink {
        from_node: from,
        from_socket: from_sock,
        to_node: to,
        to_socket: to_sock,
    });
}

#[allow(dead_code)]
pub fn get_geo_node(graph: &GeoNodeGraph, id: u32) -> Option<&GeoNode> {
    graph.nodes.iter().find(|n| n.id == id)
}

#[allow(dead_code)]
pub fn geo_node_count(graph: &GeoNodeGraph) -> usize {
    graph.nodes.len()
}

#[allow(dead_code)]
pub fn geo_link_count(graph: &GeoNodeGraph) -> usize {
    graph.links.len()
}

#[allow(dead_code)]
pub fn export_geo_graph_json(graph: &GeoNodeGraph) -> String {
    let nodes_json: Vec<String> = graph
        .nodes
        .iter()
        .map(|n| {
            format!(
                "{{\"id\":{},\"name\":\"{}\",\"type\":\"{}\"}}",
                n.id,
                n.name,
                n.node_type.type_name()
            )
        })
        .collect();
    let links_json: Vec<String> = graph
        .links
        .iter()
        .map(|l| {
            format!(
                "{{\"from\":{},\"from_socket\":{},\"to\":{},\"to_socket\":{}}}",
                l.from_node, l.from_socket, l.to_node, l.to_socket
            )
        })
        .collect();
    format!(
        "{{\"name\":\"{}\",\"nodes\":[{}],\"links\":[{}]}}",
        graph.name,
        nodes_json.join(","),
        links_json.join(",")
    )
}

#[allow(dead_code)]
pub fn export_geo_graph_python(graph: &GeoNodeGraph) -> String {
    let mut lines = Vec::new();
    lines.push("import bpy".to_string());
    lines.push(format!("# Geometry node graph: {}", graph.name));
    lines.push("node_tree = bpy.context.object.modifiers['GeometryNodes'].node_group".to_string());
    lines.push("nodes = node_tree.nodes".to_string());
    lines.push("links = node_tree.links".to_string());
    lines.push("nodes.clear()".to_string());
    lines.push(String::new());

    let mut node_var_names: std::collections::HashMap<u32, String> =
        std::collections::HashMap::new();
    for node in &graph.nodes {
        let var_name = format!("node_{}", node.id);
        node_var_names.insert(node.id, var_name.clone());
        let btype = match node.node_type {
            GeoNodeType::MeshPrimitive => "GeometryNodeMeshCube",
            GeoNodeType::Transform => "GeometryNodeTransform",
            GeoNodeType::JoinGeometry => "GeometryNodeJoinGeometry",
            GeoNodeType::SeparateGeometry => "GeometryNodeSeparateGeometry",
            GeoNodeType::Attribute => "GeometryNodeInputNamedAttribute",
            GeoNodeType::Math => "ShaderNodeMath",
            GeoNodeType::Compare => "FunctionNodeCompare",
            GeoNodeType::Switch => "GeometryNodeSwitch",
            GeoNodeType::Output => "NodeGroupOutput",
        };
        lines.push(format!("{var_name} = nodes.new(type='{btype}')"));
        lines.push(format!(
            "{var_name}.location = ({}, {})",
            node.position[0], node.position[1]
        ));
        lines.push(format!("{var_name}.label = \"{}\"", node.name));
        lines.push(String::new());
    }

    for link in &graph.links {
        if let (Some(from_var), Some(to_var)) = (
            node_var_names.get(&link.from_node),
            node_var_names.get(&link.to_node),
        ) {
            lines.push(format!(
                "links.new({from_var}.outputs[{}], {to_var}.inputs[{}])",
                link.from_socket, link.to_socket
            ));
        }
    }

    lines.join("\n")
}

#[allow(dead_code)]
pub fn find_output_node(graph: &GeoNodeGraph) -> Option<&GeoNode> {
    graph
        .nodes
        .iter()
        .find(|n| n.node_type.discriminant() == GeoNodeType::Output.discriminant())
}

#[allow(dead_code)]
pub fn nodes_of_type<'a>(graph: &'a GeoNodeGraph, node_type: &GeoNodeType) -> Vec<&'a GeoNode> {
    let disc = node_type.discriminant();
    graph
        .nodes
        .iter()
        .filter(|n| n.node_type.discriminant() == disc)
        .collect()
}

#[allow(dead_code)]
pub fn remove_geo_node(graph: &mut GeoNodeGraph, id: u32) -> bool {
    let before = graph.nodes.len();
    graph.nodes.retain(|n| n.id != id);
    graph.links.retain(|l| l.from_node != id && l.to_node != id);
    graph.nodes.len() < before
}

#[allow(dead_code)]
pub fn validate_geo_graph(graph: &GeoNodeGraph) -> Vec<String> {
    let mut issues = Vec::new();
    let node_ids: Vec<u32> = graph.nodes.iter().map(|n| n.id).collect();

    for link in &graph.links {
        if !node_ids.contains(&link.from_node) {
            issues.push(format!(
                "Dangling link: from_node {} not found",
                link.from_node
            ));
        }
        if !node_ids.contains(&link.to_node) {
            issues.push(format!("Dangling link: to_node {} not found", link.to_node));
        }
    }

    if find_output_node(graph).is_none() {
        issues.push("No Output node found in graph".to_string());
    }

    issues
}

#[allow(dead_code)]
pub fn default_output_node(graph: &mut GeoNodeGraph) -> u32 {
    add_geo_node(graph, "Group Output", GeoNodeType::Output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_geo_graph() {
        let graph = new_geo_graph("MyGraph");
        assert_eq!(graph.name, "MyGraph");
        assert!(graph.nodes.is_empty());
        assert!(graph.links.is_empty());
        assert_eq!(graph.next_id, 1);
    }

    #[test]
    fn test_add_geo_node() {
        let mut graph = new_geo_graph("G");
        let id = add_geo_node(&mut graph, "Transform", GeoNodeType::Transform);
        assert_eq!(id, 1);
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].name, "Transform");
    }

    #[test]
    fn test_add_geo_link() {
        let mut graph = new_geo_graph("G");
        let a = add_geo_node(&mut graph, "A", GeoNodeType::MeshPrimitive);
        let b = add_geo_node(&mut graph, "B", GeoNodeType::Output);
        add_geo_link(&mut graph, a, 0, b, 0);
        assert_eq!(graph.links.len(), 1);
        assert_eq!(graph.links[0].from_node, a);
        assert_eq!(graph.links[0].to_node, b);
    }

    #[test]
    fn test_get_geo_node_found() {
        let mut graph = new_geo_graph("G");
        let id = add_geo_node(&mut graph, "Math", GeoNodeType::Math);
        let node = get_geo_node(&graph, id);
        assert!(node.is_some());
        assert_eq!(node.expect("should succeed").name, "Math");
    }

    #[test]
    fn test_get_geo_node_not_found() {
        let graph = new_geo_graph("G");
        assert!(get_geo_node(&graph, 99).is_none());
    }

    #[test]
    fn test_geo_node_count() {
        let mut graph = new_geo_graph("G");
        assert_eq!(geo_node_count(&graph), 0);
        add_geo_node(&mut graph, "A", GeoNodeType::Math);
        add_geo_node(&mut graph, "B", GeoNodeType::Math);
        assert_eq!(geo_node_count(&graph), 2);
    }

    #[test]
    fn test_geo_link_count() {
        let mut graph = new_geo_graph("G");
        let a = add_geo_node(&mut graph, "A", GeoNodeType::MeshPrimitive);
        let b = add_geo_node(&mut graph, "B", GeoNodeType::Output);
        assert_eq!(geo_link_count(&graph), 0);
        add_geo_link(&mut graph, a, 0, b, 0);
        assert_eq!(geo_link_count(&graph), 1);
    }

    #[test]
    fn test_export_geo_graph_json_non_empty() {
        let mut graph = new_geo_graph("TestGraph");
        add_geo_node(&mut graph, "Node1", GeoNodeType::MeshPrimitive);
        let json = export_geo_graph_json(&graph);
        assert!(!json.is_empty());
        assert!(json.contains("TestGraph"));
        assert!(json.contains("MeshPrimitive"));
    }

    #[test]
    fn test_export_geo_graph_python_non_empty() {
        let mut graph = new_geo_graph("PyGraph");
        add_geo_node(&mut graph, "Cube", GeoNodeType::MeshPrimitive);
        let py = export_geo_graph_python(&graph);
        assert!(!py.is_empty());
        assert!(py.contains("import bpy"));
    }

    #[test]
    fn test_find_output_node_none() {
        let mut graph = new_geo_graph("G");
        add_geo_node(&mut graph, "Math", GeoNodeType::Math);
        assert!(find_output_node(&graph).is_none());
    }

    #[test]
    fn test_find_output_node_found() {
        let mut graph = new_geo_graph("G");
        add_geo_node(&mut graph, "Output", GeoNodeType::Output);
        let result = find_output_node(&graph);
        assert!(result.is_some());
    }

    #[test]
    fn test_remove_geo_node() {
        let mut graph = new_geo_graph("G");
        let a = add_geo_node(&mut graph, "A", GeoNodeType::Math);
        let b = add_geo_node(&mut graph, "B", GeoNodeType::Output);
        add_geo_link(&mut graph, a, 0, b, 0);
        let removed = remove_geo_node(&mut graph, a);
        assert!(removed);
        assert_eq!(graph.nodes.len(), 1);
        assert!(graph.links.is_empty());
    }

    #[test]
    fn test_remove_geo_node_not_found() {
        let mut graph = new_geo_graph("G");
        add_geo_node(&mut graph, "A", GeoNodeType::Math);
        let removed = remove_geo_node(&mut graph, 999);
        assert!(!removed);
    }

    #[test]
    fn test_validate_geo_graph_no_output_warning() {
        let mut graph = new_geo_graph("G");
        add_geo_node(&mut graph, "Math", GeoNodeType::Math);
        let issues = validate_geo_graph(&graph);
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.contains("Output")));
    }

    #[test]
    fn test_validate_geo_graph_passes_with_output() {
        let mut graph = new_geo_graph("G");
        default_output_node(&mut graph);
        let issues = validate_geo_graph(&graph);
        assert!(issues.is_empty(), "Issues: {issues:?}");
    }

    #[test]
    fn test_nodes_of_type() {
        let mut graph = new_geo_graph("G");
        add_geo_node(&mut graph, "M1", GeoNodeType::Math);
        add_geo_node(&mut graph, "M2", GeoNodeType::Math);
        add_geo_node(&mut graph, "Out", GeoNodeType::Output);
        let math_nodes = nodes_of_type(&graph, &GeoNodeType::Math);
        assert_eq!(math_nodes.len(), 2);
    }

    #[test]
    fn test_default_output_node() {
        let mut graph = new_geo_graph("G");
        let id = default_output_node(&mut graph);
        assert!(get_geo_node(&graph, id).is_some());
        assert!(find_output_node(&graph).is_some());
    }
}
