#![allow(dead_code)]

//! Scene light export.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SceneLightExport {
    pub light_type: String,
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32,
}

#[allow(dead_code)]
pub fn export_scene_light(light_type: &str, color: [f32; 3], intensity: f32, range: f32) -> SceneLightExport {
    SceneLightExport { light_type: light_type.to_string(), color, intensity, range }
}

#[allow(dead_code)]
pub fn light_type_sle(l: &SceneLightExport) -> &str { &l.light_type }

#[allow(dead_code)]
pub fn light_color_sle(l: &SceneLightExport) -> [f32; 3] { l.color }

#[allow(dead_code)]
pub fn light_intensity_sle(l: &SceneLightExport) -> f32 { l.intensity }

#[allow(dead_code)]
pub fn light_range_sle(l: &SceneLightExport) -> f32 { l.range }

#[allow(dead_code)]
pub fn light_to_json_sle(l: &SceneLightExport) -> String {
    format!("{{\"type\":\"{}\",\"color\":[{:.3},{:.3},{:.3}],\"intensity\":{:.4},\"range\":{:.4}}}", l.light_type, l.color[0], l.color[1], l.color[2], l.intensity, l.range)
}

#[allow(dead_code)]
pub fn light_export_size(l: &SceneLightExport) -> usize { l.light_type.len() + 20 }

#[allow(dead_code)]
pub fn validate_scene_light(l: &SceneLightExport) -> bool {
    l.intensity >= 0.0 && l.range > 0.0 && !l.light_type.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export() { let l = export_scene_light("point", [1.0,1.0,1.0], 1.0, 10.0); assert_eq!(light_type_sle(&l), "point"); }
    #[test]
    fn test_color() { let l = export_scene_light("p", [1.0,0.5,0.0], 1.0, 10.0); assert!((light_color_sle(&l)[1] - 0.5).abs() < 1e-6); }
    #[test]
    fn test_intensity() { let l = export_scene_light("p", [1.0;3], 5.0, 10.0); assert!((light_intensity_sle(&l) - 5.0).abs() < 1e-6); }
    #[test]
    fn test_range() { let l = export_scene_light("p", [1.0;3], 1.0, 25.0); assert!((light_range_sle(&l) - 25.0).abs() < 1e-6); }
    #[test]
    fn test_to_json() { let l = export_scene_light("spot", [1.0;3], 1.0, 10.0); assert!(light_to_json_sle(&l).contains("\"type\":\"spot\"")); }
    #[test]
    fn test_export_size() { let l = export_scene_light("p", [1.0;3], 1.0, 10.0); assert!(light_export_size(&l) > 0); }
    #[test]
    fn test_validate() { assert!(validate_scene_light(&export_scene_light("p", [1.0;3], 1.0, 10.0))); }
    #[test]
    fn test_validate_bad_range() { assert!(!validate_scene_light(&export_scene_light("p", [1.0;3], 1.0, 0.0))); }
    #[test]
    fn test_validate_empty_type() { assert!(!validate_scene_light(&export_scene_light("", [1.0;3], 1.0, 10.0))); }
    #[test]
    fn test_directional() { let l = export_scene_light("directional", [1.0;3], 2.0, 100.0); assert_eq!(light_type_sle(&l), "directional"); }
}
