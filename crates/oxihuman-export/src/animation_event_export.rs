#![allow(dead_code)]

//! Animation event export.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnimEventExport {
    pub events: Vec<AnimEvent>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnimEvent {
    pub time: f32,
    pub name: String,
    pub data: String,
}

#[allow(dead_code)]
pub fn export_anim_events(events: Vec<AnimEvent>) -> AnimEventExport {
    AnimEventExport { events }
}

#[allow(dead_code)]
pub fn event_count_aee(exp: &AnimEventExport) -> usize { exp.events.len() }

#[allow(dead_code)]
pub fn event_time(exp: &AnimEventExport, idx: usize) -> Option<f32> {
    exp.events.get(idx).map(|e| e.time)
}

#[allow(dead_code)]
pub fn event_name_aee(exp: &AnimEventExport, idx: usize) -> Option<&str> {
    exp.events.get(idx).map(|e| e.name.as_str())
}

#[allow(dead_code)]
pub fn event_to_json(exp: &AnimEventExport) -> String {
    let items: Vec<String> = exp.events.iter().map(|e|
        format!("{{\"time\":{:.4},\"name\":\"{}\",\"data\":\"{}\"}}", e.time, e.name, e.data)
    ).collect();
    format!("{{\"event_count\":{},\"events\":[{}]}}", exp.events.len(), items.join(","))
}

#[allow(dead_code)]
pub fn event_data(exp: &AnimEventExport, idx: usize) -> Option<&str> {
    exp.events.get(idx).map(|e| e.data.as_str())
}

#[allow(dead_code)]
pub fn event_export_size(exp: &AnimEventExport) -> usize {
    exp.events.iter().map(|e| e.name.len() + e.data.len() + 4).sum()
}

#[allow(dead_code)]
pub fn validate_anim_events(exp: &AnimEventExport) -> bool {
    exp.events.iter().all(|e| e.time >= 0.0 && !e.name.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ev(t: f32) -> AnimEvent { AnimEvent { time: t, name: "footstep".into(), data: "left".into() } }

    #[test]
    fn test_export() { let e = export_anim_events(vec![ev(0.5)]); assert_eq!(event_count_aee(&e), 1); }
    #[test]
    fn test_time() { let e = export_anim_events(vec![ev(1.5)]); assert!((event_time(&e, 0).unwrap() - 1.5).abs() < 1e-6); }
    #[test]
    fn test_time_none() { let e = export_anim_events(vec![]); assert!(event_time(&e, 0).is_none()); }
    #[test]
    fn test_name() { let e = export_anim_events(vec![ev(0.0)]); assert_eq!(event_name_aee(&e, 0), Some("footstep")); }
    #[test]
    fn test_to_json() { let e = export_anim_events(vec![ev(0.0)]); assert!(event_to_json(&e).contains("\"event_count\":1")); }
    #[test]
    fn test_data() { let e = export_anim_events(vec![ev(0.0)]); assert_eq!(event_data(&e, 0), Some("left")); }
    #[test]
    fn test_export_size() { let e = export_anim_events(vec![ev(0.0)]); assert!(event_export_size(&e) > 0); }
    #[test]
    fn test_validate() { assert!(validate_anim_events(&export_anim_events(vec![ev(0.0)]))); }
    #[test]
    fn test_validate_empty() { assert!(validate_anim_events(&export_anim_events(vec![]))); }
    #[test]
    fn test_multiple() { let e = export_anim_events(vec![ev(0.0),ev(1.0),ev(2.0)]); assert_eq!(event_count_aee(&e), 3); }
}
