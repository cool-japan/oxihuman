#![allow(dead_code)]
//! Animation node export.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnimNodeExport { name: String, node_type: String, children: Vec<String>, weight: f32 }

#[allow(dead_code)]
pub fn export_anim_node(name: &str, node_type: &str) -> AnimNodeExport {
    AnimNodeExport { name: name.to_string(), node_type: node_type.to_string(), children: Vec::new(), weight: 1.0 }
}
#[allow(dead_code)]
pub fn node_name_ane(m: &AnimNodeExport) -> &str { &m.name }
#[allow(dead_code)]
pub fn node_type_ane(m: &AnimNodeExport) -> &str { &m.node_type }
#[allow(dead_code)]
pub fn node_children_ane(m: &AnimNodeExport) -> &[String] { &m.children }
#[allow(dead_code)]
pub fn node_to_json_ane(m: &AnimNodeExport) -> String {
    let cs: Vec<String> = m.children.iter().map(|c| format!("\"{}\"",c)).collect();
    format!("{{\"name\":\"{}\",\"type\":\"{}\",\"children\":[{}],\"weight\":{:.4}}}", m.name, m.node_type, cs.join(","), m.weight)
}
#[allow(dead_code)]
pub fn node_weight_ane(m: &AnimNodeExport) -> f32 { m.weight }
#[allow(dead_code)]
pub fn node_export_size_ane(m: &AnimNodeExport) -> usize { m.name.len() + m.node_type.len() + m.children.iter().map(|c| c.len()).sum::<usize>() + 4 }
#[allow(dead_code)]
pub fn validate_anim_node(m: &AnimNodeExport) -> bool { !m.name.is_empty() && !m.node_type.is_empty() }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_export() { let m = export_anim_node("root","blend"); assert_eq!(node_name_ane(&m), "root"); }
    #[test] fn test_name() { let m = export_anim_node("n","t"); assert_eq!(node_name_ane(&m), "n"); }
    #[test] fn test_type() { let m = export_anim_node("n","blend"); assert_eq!(node_type_ane(&m), "blend"); }
    #[test] fn test_children() { let m = export_anim_node("n","t"); assert!(node_children_ane(&m).is_empty()); }
    #[test] fn test_json() { let m = export_anim_node("n","t"); assert!(node_to_json_ane(&m).contains("\"n\"")); }
    #[test] fn test_weight() { let m = export_anim_node("n","t"); assert!((node_weight_ane(&m) - 1.0).abs() < 1e-6); }
    #[test] fn test_size() { let m = export_anim_node("n","t"); assert!(node_export_size_ane(&m) > 0); }
    #[test] fn test_validate() { let m = export_anim_node("n","t"); assert!(validate_anim_node(&m)); }
    #[test] fn test_invalid_name() { let m = export_anim_node("","t"); assert!(!validate_anim_node(&m)); }
    #[test] fn test_invalid_type() { let m = export_anim_node("n",""); assert!(!validate_anim_node(&m)); }
}
