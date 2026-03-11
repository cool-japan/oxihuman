//! LOD morph — level-of-detail morph that applies full morph weight at close range
//! and reduces influence at distance using configurable falloff levels.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LodMorphConfig {
    pub base_weight: f32,
    pub falloff_exponent: f32,
    pub active: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LodLevel {
    pub distance: f32,
    pub weight_scale: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LodMorphResult {
    pub effective_weight: f32,
    pub level_index: usize,
    pub distance: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LodMorph {
    pub config: LodMorphConfig,
    pub levels: Vec<LodLevel>,
    pub current_distance: f32,
}

#[allow(dead_code)]
pub fn default_lod_morph_config() -> LodMorphConfig {
    LodMorphConfig {
        base_weight: 1.0,
        falloff_exponent: 2.0,
        active: true,
    }
}

#[allow(dead_code)]
pub fn new_lod_morph(config: LodMorphConfig) -> LodMorph {
    LodMorph {
        config,
        levels: Vec::new(),
        current_distance: 0.0,
    }
}

#[allow(dead_code)]
pub fn lod_morph_weight_at_distance(morph: &LodMorph, distance: f32) -> LodMorphResult {
    if !morph.config.active || morph.levels.is_empty() {
        return LodMorphResult {
            effective_weight: morph.config.base_weight,
            level_index: 0,
            distance,
        };
    }

    // Find the appropriate level (sorted by distance ascending)
    let mut level_index = 0usize;
    for (i, lv) in morph.levels.iter().enumerate() {
        if distance >= lv.distance {
            level_index = i;
        } else {
            break;
        }
    }

    let lv = &morph.levels[level_index];

    // Interpolate to next level if available
    let weight_scale = if level_index + 1 < morph.levels.len() {
        let next = &morph.levels[level_index + 1];
        let t = ((distance - lv.distance) / (next.distance - lv.distance)).clamp(0.0, 1.0);
        lv.weight_scale + t * (next.weight_scale - lv.weight_scale)
    } else {
        // Beyond last level: apply falloff
        let excess = (distance - lv.distance).max(0.0);
        let falloff = 1.0 / (1.0 + excess.powf(morph.config.falloff_exponent));
        lv.weight_scale * falloff
    };

    LodMorphResult {
        effective_weight: morph.config.base_weight * weight_scale,
        level_index,
        distance,
    }
}

#[allow(dead_code)]
pub fn lod_morph_add_level(morph: &mut LodMorph, distance: f32, weight_scale: f32) {
    morph.levels.push(LodLevel {
        distance,
        weight_scale: weight_scale.clamp(0.0, 1.0),
    });
    morph.levels.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(std::cmp::Ordering::Equal));
}

#[allow(dead_code)]
pub fn lod_morph_level_count(morph: &LodMorph) -> usize {
    morph.levels.len()
}

#[allow(dead_code)]
pub fn lod_morph_max_distance(morph: &LodMorph) -> f32 {
    morph
        .levels
        .iter()
        .map(|l| l.distance)
        .fold(f32::NEG_INFINITY, f32::max)
}

#[allow(dead_code)]
pub fn lod_morph_set_distance(morph: &mut LodMorph, distance: f32) {
    morph.current_distance = distance.max(0.0);
}

#[allow(dead_code)]
pub fn lod_morph_to_json(morph: &LodMorph) -> String {
    let levels: Vec<String> = morph
        .levels
        .iter()
        .map(|l| {
            format!(
                "{{\"distance\":{:.4},\"weight_scale\":{:.4}}}",
                l.distance, l.weight_scale
            )
        })
        .collect();
    format!(
        "{{\"base_weight\":{:.4},\"active\":{},\"current_distance\":{:.4},\"levels\":[{}]}}",
        morph.config.base_weight,
        morph.config.active,
        morph.current_distance,
        levels.join(",")
    )
}

#[allow(dead_code)]
pub fn lod_morph_reset(morph: &mut LodMorph) {
    morph.levels.clear();
    morph.current_distance = 0.0;
    morph.config.active = true;
}

#[allow(dead_code)]
pub fn lod_morph_is_active(morph: &LodMorph) -> bool {
    morph.config.active
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_morph() -> LodMorph {
        let cfg = default_lod_morph_config();
        let mut m = new_lod_morph(cfg);
        lod_morph_add_level(&mut m, 0.0, 1.0);
        lod_morph_add_level(&mut m, 10.0, 0.5);
        lod_morph_add_level(&mut m, 50.0, 0.1);
        m
    }

    #[test]
    fn test_default_config() {
        let c = default_lod_morph_config();
        assert!((c.base_weight - 1.0).abs() < 1e-6);
        assert!(c.active);
    }

    #[test]
    fn test_add_levels_sorted() {
        let m = make_morph();
        assert_eq!(lod_morph_level_count(&m), 3);
        assert!(m.levels[0].distance <= m.levels[1].distance);
    }

    #[test]
    fn test_weight_at_zero_distance() {
        let m = make_morph();
        let r = lod_morph_weight_at_distance(&m, 0.0);
        assert!((r.effective_weight - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_weight_decreases_with_distance() {
        let m = make_morph();
        let r_near = lod_morph_weight_at_distance(&m, 0.0);
        let r_far = lod_morph_weight_at_distance(&m, 50.0);
        assert!(r_far.effective_weight < r_near.effective_weight);
    }

    #[test]
    fn test_max_distance() {
        let m = make_morph();
        assert!((lod_morph_max_distance(&m) - 50.0).abs() < 1e-5);
    }

    #[test]
    fn test_set_distance() {
        let mut m = make_morph();
        lod_morph_set_distance(&mut m, 25.0);
        assert!((m.current_distance - 25.0).abs() < 1e-5);
    }

    #[test]
    fn test_is_active() {
        let m = make_morph();
        assert!(lod_morph_is_active(&m));
    }

    #[test]
    fn test_inactive_returns_base_weight() {
        let mut m = make_morph();
        m.config.active = false;
        let r = lod_morph_weight_at_distance(&m, 100.0);
        assert!((r.effective_weight - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_reset() {
        let mut m = make_morph();
        lod_morph_reset(&mut m);
        assert_eq!(lod_morph_level_count(&m), 0);
        assert!(lod_morph_is_active(&m));
    }

    #[test]
    fn test_to_json() {
        let m = make_morph();
        let j = lod_morph_to_json(&m);
        assert!(j.contains("levels"));
        assert!(j.contains("base_weight"));
    }
}
