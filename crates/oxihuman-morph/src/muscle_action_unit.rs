//! FACS-like Action Units for facial expression control.

#[allow(dead_code)]
pub struct ActionUnit {
    pub au_code: u32,
    pub name: String,
    pub intensity: f32,
    pub morph_targets: Vec<(String, f32)>,
    pub bilateral: bool,
}

#[allow(dead_code)]
pub struct AuSet {
    pub units: Vec<ActionUnit>,
    pub name: String,
}

#[allow(dead_code)]
pub struct AuFrame {
    pub time: f32,
    pub au_intensities: Vec<(u32, f32)>,
}

#[allow(dead_code)]
pub fn standard_facs_set() -> AuSet {
    let mut set = new_au_set("FACS Standard");
    let au_defs: &[(u32, &str, bool)] = &[
        (1, "Inner Brow Raise", true),
        (2, "Outer Brow Raise", true),
        (4, "Brow Lowerer", true),
        (5, "Upper Lid Raiser", true),
        (6, "Cheek Raiser", true),
        (7, "Lid Tightener", true),
        (9, "Nose Wrinkler", true),
        (10, "Upper Lip Raiser", false),
        (12, "Lip Corner Puller", true),
        (15, "Lip Corner Depressor", true),
        (17, "Chin Raiser", false),
        (20, "Lip Stretcher", true),
        (23, "Lip Tightener", false),
        (25, "Lips Part", false),
        (26, "Jaw Drop", false),
    ];
    for &(code, name, bilateral) in au_defs {
        let targets = vec![(format!("au_{code}"), 1.0_f32)];
        add_action_unit(
            &mut set,
            ActionUnit {
                au_code: code,
                name: name.to_string(),
                intensity: 0.0,
                morph_targets: targets,
                bilateral,
            },
        );
    }
    set
}

#[allow(dead_code)]
pub fn new_au_set(name: &str) -> AuSet {
    AuSet {
        units: Vec::new(),
        name: name.to_string(),
    }
}

#[allow(dead_code)]
pub fn add_action_unit(set: &mut AuSet, au: ActionUnit) {
    set.units.push(au);
}

#[allow(dead_code)]
pub fn get_au(set: &AuSet, code: u32) -> Option<&ActionUnit> {
    set.units.iter().find(|u| u.au_code == code)
}

#[allow(dead_code)]
pub fn set_au_intensity(set: &mut AuSet, code: u32, intensity: f32) -> bool {
    let clamped = intensity.clamp(0.0, 1.0);
    if let Some(au) = set.units.iter_mut().find(|u| u.au_code == code) {
        au.intensity = clamped;
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn evaluate_au_set(set: &AuSet) -> Vec<(String, f32)> {
    let mut contributions: std::collections::HashMap<String, f32> =
        std::collections::HashMap::new();
    for au in &set.units {
        if au.intensity <= 0.0 {
            continue;
        }
        for (target_name, weight_mult) in &au.morph_targets {
            let entry = contributions.entry(target_name.clone()).or_insert(0.0);
            *entry += au.intensity * weight_mult;
        }
    }
    let mut result: Vec<(String, f32)> = contributions.into_iter().collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    for (_, v) in &mut result {
        *v = v.clamp(0.0, 1.0);
    }
    result
}

#[allow(dead_code)]
pub fn au_to_emotion(set: &AuSet) -> &'static str {
    let active: Vec<u32> = set
        .units
        .iter()
        .filter(|u| u.intensity > 0.5)
        .map(|u| u.au_code)
        .collect();

    let has = |code: u32| active.contains(&code);

    if has(1) && has(4) && has(15) {
        "sadness"
    } else if has(2) && has(5) && has(20) && has(26) {
        "fear"
    } else if has(4) && has(5) && has(23) && has(25) {
        "anger"
    } else if has(1) && has(2) && has(5) && has(26) {
        "surprise"
    } else if has(9) && has(15) && has(16) {
        "disgust"
    } else if has(6) && has(12) {
        "happiness"
    } else {
        "neutral"
    }
}

#[allow(dead_code)]
pub fn au_count(set: &AuSet) -> usize {
    set.units.len()
}

#[allow(dead_code)]
pub fn active_aus(set: &AuSet) -> Vec<&ActionUnit> {
    set.units.iter().filter(|u| u.intensity > 0.01).collect()
}

#[allow(dead_code)]
pub fn reset_all_aus(set: &mut AuSet) {
    for au in &mut set.units {
        au.intensity = 0.0;
    }
}

#[allow(dead_code)]
pub fn blend_au_frames(a: &AuFrame, b: &AuFrame, t: f32) -> AuFrame {
    let t = t.clamp(0.0, 1.0);
    let mut map: std::collections::HashMap<u32, f32> = std::collections::HashMap::new();
    for &(code, intensity) in &a.au_intensities {
        map.insert(code, intensity * (1.0 - t));
    }
    for &(code, intensity) in &b.au_intensities {
        let entry = map.entry(code).or_insert(0.0);
        *entry += intensity * t;
    }
    let mut intensities: Vec<(u32, f32)> = map.into_iter().collect();
    intensities.sort_by_key(|&(code, _)| code);
    AuFrame {
        time: a.time * (1.0 - t) + b.time * t,
        au_intensities: intensities,
    }
}

#[allow(dead_code)]
pub fn au_frame_from_set(set: &AuSet, time: f32) -> AuFrame {
    let au_intensities = set.units.iter().map(|u| (u.au_code, u.intensity)).collect();
    AuFrame {
        time,
        au_intensities,
    }
}

#[allow(dead_code)]
pub fn apply_au_frame(set: &mut AuSet, frame: &AuFrame) {
    for &(code, intensity) in &frame.au_intensities {
        set_au_intensity(set, code, intensity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_facs_set_has_15_aus() {
        let set = standard_facs_set();
        assert_eq!(au_count(&set), 15);
    }

    #[test]
    fn test_new_au_set() {
        let set = new_au_set("Test Set");
        assert_eq!(set.name, "Test Set");
        assert_eq!(au_count(&set), 0);
    }

    #[test]
    fn test_add_action_unit() {
        let mut set = new_au_set("Test");
        add_action_unit(
            &mut set,
            ActionUnit {
                au_code: 1,
                name: "Inner Brow Raise".to_string(),
                intensity: 0.0,
                morph_targets: vec![("brow_raise".to_string(), 1.0)],
                bilateral: true,
            },
        );
        assert_eq!(au_count(&set), 1);
    }

    #[test]
    fn test_get_au() {
        let set = standard_facs_set();
        let au = get_au(&set, 12);
        assert!(au.is_some());
        assert_eq!(au.unwrap().au_code, 12);
    }

    #[test]
    fn test_get_au_not_found() {
        let set = standard_facs_set();
        assert!(get_au(&set, 99).is_none());
    }

    #[test]
    fn test_set_au_intensity() {
        let mut set = standard_facs_set();
        let result = set_au_intensity(&mut set, 12, 0.8);
        assert!(result);
        let au = get_au(&set, 12).unwrap();
        assert!((au.intensity - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_au_intensity_clamps() {
        let mut set = standard_facs_set();
        set_au_intensity(&mut set, 1, 2.0);
        assert!((get_au(&set, 1).unwrap().intensity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_au_intensity_not_found() {
        let mut set = standard_facs_set();
        assert!(!set_au_intensity(&mut set, 99, 0.5));
    }

    #[test]
    fn test_evaluate_au_set() {
        let mut set = standard_facs_set();
        set_au_intensity(&mut set, 12, 0.5);
        let result = evaluate_au_set(&set);
        assert!(!result.is_empty());
        let au12 = result.iter().find(|(name, _)| name == "au_12");
        assert!(au12.is_some());
        assert!((au12.unwrap().1 - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_au_to_emotion_happiness() {
        let mut set = standard_facs_set();
        set_au_intensity(&mut set, 6, 1.0);
        set_au_intensity(&mut set, 12, 1.0);
        assert_eq!(au_to_emotion(&set), "happiness");
    }

    #[test]
    fn test_au_to_emotion_neutral() {
        let set = standard_facs_set();
        assert_eq!(au_to_emotion(&set), "neutral");
    }

    #[test]
    fn test_active_aus() {
        let mut set = standard_facs_set();
        set_au_intensity(&mut set, 1, 0.5);
        set_au_intensity(&mut set, 12, 0.0);
        let active = active_aus(&set);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].au_code, 1);
    }

    #[test]
    fn test_reset_all_aus() {
        let mut set = standard_facs_set();
        set_au_intensity(&mut set, 1, 0.8);
        set_au_intensity(&mut set, 12, 0.6);
        reset_all_aus(&mut set);
        assert!(active_aus(&set).is_empty());
    }

    #[test]
    fn test_au_frame_from_set() {
        let mut set = standard_facs_set();
        set_au_intensity(&mut set, 12, 0.7);
        let frame = au_frame_from_set(&set, 1.5);
        assert!((frame.time - 1.5).abs() < 1e-6);
        let au12_entry = frame.au_intensities.iter().find(|&&(code, _)| code == 12);
        assert!(au12_entry.is_some());
        assert!((au12_entry.unwrap().1 - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_apply_au_frame() {
        let mut set = standard_facs_set();
        set_au_intensity(&mut set, 12, 0.7);
        let frame = au_frame_from_set(&set, 0.0);
        let mut set2 = standard_facs_set();
        apply_au_frame(&mut set2, &frame);
        assert!((get_au(&set2, 12).unwrap().intensity - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_blend_au_frames() {
        let mut set_a = standard_facs_set();
        set_au_intensity(&mut set_a, 12, 0.0);
        let frame_a = au_frame_from_set(&set_a, 0.0);

        let mut set_b = standard_facs_set();
        set_au_intensity(&mut set_b, 12, 1.0);
        let frame_b = au_frame_from_set(&set_b, 1.0);

        let blended = blend_au_frames(&frame_a, &frame_b, 0.5);
        let au12 = blended.au_intensities.iter().find(|&&(code, _)| code == 12);
        assert!(au12.is_some());
        assert!((au12.unwrap().1 - 0.5).abs() < 1e-6);
        assert!((blended.time - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_au_count() {
        let set = standard_facs_set();
        assert_eq!(au_count(&set), 15);
    }
}
