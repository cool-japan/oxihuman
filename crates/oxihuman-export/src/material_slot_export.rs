#![allow(dead_code)]

//! Material slot export for multi-material meshes.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MaterialSlotExport {
    pub slots: Vec<MatSlot>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MatSlot {
    pub name: String,
    pub shader: String,
    pub face_start: usize,
    pub face_end: usize,
}

#[allow(dead_code)]
pub fn export_material_slots(slots: Vec<MatSlot>) -> MaterialSlotExport {
    MaterialSlotExport { slots }
}

#[allow(dead_code)]
pub fn slot_count_mse(exp: &MaterialSlotExport) -> usize { exp.slots.len() }

#[allow(dead_code)]
pub fn slot_material_name(exp: &MaterialSlotExport, idx: usize) -> Option<&str> {
    exp.slots.get(idx).map(|s| s.name.as_str())
}

#[allow(dead_code)]
pub fn slot_to_json(exp: &MaterialSlotExport) -> String {
    let items: Vec<String> = exp.slots.iter().map(|s|
        format!("{{\"name\":\"{}\",\"shader\":\"{}\",\"faces\":[{},{}]}}", s.name, s.shader, s.face_start, s.face_end)
    ).collect();
    format!("{{\"slot_count\":{},\"slots\":[{}]}}", exp.slots.len(), items.join(","))
}

#[allow(dead_code)]
pub fn slot_face_range(exp: &MaterialSlotExport, idx: usize) -> Option<(usize, usize)> {
    exp.slots.get(idx).map(|s| (s.face_start, s.face_end))
}

#[allow(dead_code)]
pub fn slot_export_size(exp: &MaterialSlotExport) -> usize {
    exp.slots.iter().map(|s| s.name.len() + s.shader.len() + 16).sum()
}

#[allow(dead_code)]
pub fn validate_slots(exp: &MaterialSlotExport) -> bool {
    !exp.slots.is_empty() && exp.slots.iter().all(|s| !s.name.is_empty())
}

#[allow(dead_code)]
pub fn slot_shader(exp: &MaterialSlotExport, idx: usize) -> Option<&str> {
    exp.slots.get(idx).map(|s| s.shader.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ms(n: &str) -> MatSlot { MatSlot { name: n.into(), shader: "pbr".into(), face_start: 0, face_end: 10 } }

    #[test]
    fn test_export() { let e = export_material_slots(vec![ms("skin")]); assert_eq!(slot_count_mse(&e), 1); }
    #[test]
    fn test_name() { let e = export_material_slots(vec![ms("skin")]); assert_eq!(slot_material_name(&e, 0), Some("skin")); }
    #[test]
    fn test_name_none() { let e = export_material_slots(vec![]); assert_eq!(slot_material_name(&e, 0), None); }
    #[test]
    fn test_to_json() { let e = export_material_slots(vec![ms("a")]); assert!(slot_to_json(&e).contains("\"slot_count\":1")); }
    #[test]
    fn test_face_range() { let e = export_material_slots(vec![ms("a")]); assert_eq!(slot_face_range(&e, 0), Some((0, 10))); }
    #[test]
    fn test_export_size() { let e = export_material_slots(vec![ms("ab")]); assert!(slot_export_size(&e) > 0); }
    #[test]
    fn test_validate() { assert!(validate_slots(&export_material_slots(vec![ms("a")]))); }
    #[test]
    fn test_validate_empty() { assert!(!validate_slots(&export_material_slots(vec![]))); }
    #[test]
    fn test_shader() { let e = export_material_slots(vec![ms("a")]); assert_eq!(slot_shader(&e, 0), Some("pbr")); }
    #[test]
    fn test_multiple() { let e = export_material_slots(vec![ms("a"),ms("b")]); assert_eq!(slot_count_mse(&e), 2); }
}
