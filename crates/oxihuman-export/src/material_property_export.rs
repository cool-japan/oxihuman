#![allow(dead_code)]
//! Material property export.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MaterialPropertyExport { properties: Vec<(String, String, String)> }

#[allow(dead_code)]
pub fn export_material_property(name: &str, prop_type: &str, value: &str) -> MaterialPropertyExport {
    MaterialPropertyExport { properties: vec![(name.to_string(), prop_type.to_string(), value.to_string())] }
}
#[allow(dead_code)]
pub fn property_name_mpe2(m: &MaterialPropertyExport, idx: usize) -> &str { m.properties.get(idx).map_or("", |p| &p.0) }
#[allow(dead_code)]
pub fn property_type_mpe2(m: &MaterialPropertyExport, idx: usize) -> &str { m.properties.get(idx).map_or("", |p| &p.1) }
#[allow(dead_code)]
pub fn property_value_mpe2(m: &MaterialPropertyExport, idx: usize) -> &str { m.properties.get(idx).map_or("", |p| &p.2) }
#[allow(dead_code)]
pub fn property_to_json_mpe2(m: &MaterialPropertyExport) -> String {
    let ps: Vec<String> = m.properties.iter().map(|(n,t,v)| format!("{{\"name\":\"{}\",\"type\":\"{}\",\"value\":\"{}\"}}", n, t, v)).collect();
    format!("{{\"properties\":[{}]}}", ps.join(","))
}
#[allow(dead_code)]
pub fn property_count_mpe2(m: &MaterialPropertyExport) -> usize { m.properties.len() }
#[allow(dead_code)]
pub fn property_export_size(m: &MaterialPropertyExport) -> usize { m.properties.iter().map(|(n,t,v)| n.len()+t.len()+v.len()).sum() }
#[allow(dead_code)]
pub fn validate_material_property(m: &MaterialPropertyExport) -> bool { m.properties.iter().all(|(n,_,_)| !n.is_empty()) }

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> MaterialPropertyExport { export_material_property("albedo", "color", "1.0,0.0,0.0") }
    #[test] fn test_export() { let m = data(); assert_eq!(property_count_mpe2(&m), 1); }
    #[test] fn test_name() { let m = data(); assert_eq!(property_name_mpe2(&m, 0), "albedo"); }
    #[test] fn test_type() { let m = data(); assert_eq!(property_type_mpe2(&m, 0), "color"); }
    #[test] fn test_value() { let m = data(); assert!(property_value_mpe2(&m, 0).contains("1.0")); }
    #[test] fn test_json() { let m = data(); assert!(property_to_json_mpe2(&m).contains("albedo")); }
    #[test] fn test_count() { let m = data(); assert_eq!(property_count_mpe2(&m), 1); }
    #[test] fn test_size() { let m = data(); assert!(property_export_size(&m) > 0); }
    #[test] fn test_validate() { let m = data(); assert!(validate_material_property(&m)); }
    #[test] fn test_oob() { let m = data(); assert_eq!(property_name_mpe2(&m, 5), ""); }
    #[test] fn test_invalid() { let m = export_material_property("", "t", "v"); assert!(!validate_material_property(&m)); }
}
