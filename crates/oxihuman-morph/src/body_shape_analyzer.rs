#![allow(dead_code)]
//! Body shape analyzer: analyzes body parameters to classify body type and compute scores.

use std::collections::HashMap;

/// Analysis results for a body shape.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyShapeAnalyzer {
    pub body_type: String,
    pub symmetry: f32,
    pub proportion: f32,
    pub muscularity: f32,
    pub bmi: f32,
}

/// Analyze body shape from a set of measurements.
#[allow(dead_code)]
pub fn analyze_body_shape(
    height: f32,
    weight: f32,
    shoulder_w: f32,
    hip_w: f32,
) -> BodyShapeAnalyzer {
    let bmi = if height > 0.0 {
        weight / (height * height)
    } else {
        0.0
    };
    let ratio = if hip_w > 0.0 {
        shoulder_w / hip_w
    } else {
        1.0
    };
    let body_type = if ratio > 1.2 {
        "inverted_triangle"
    } else if ratio < 0.9 {
        "pear"
    } else {
        "balanced"
    };
    BodyShapeAnalyzer {
        body_type: body_type.to_string(),
        symmetry: 0.95,
        proportion: ratio.clamp(0.0, 2.0) / 2.0,
        muscularity: 0.5,
        bmi,
    }
}

/// Return the body type classification string.
#[allow(dead_code)]
pub fn body_type_classification(analyzer: &BodyShapeAnalyzer) -> &str {
    &analyzer.body_type
}

/// Return the symmetry score.
#[allow(dead_code)]
pub fn symmetry_score_bsa(analyzer: &BodyShapeAnalyzer) -> f32 {
    analyzer.symmetry
}

/// Return the proportion score.
#[allow(dead_code)]
pub fn proportion_score_bsa(analyzer: &BodyShapeAnalyzer) -> f32 {
    analyzer.proportion
}

/// Return the muscularity score.
#[allow(dead_code)]
pub fn muscularity_score(analyzer: &BodyShapeAnalyzer) -> f32 {
    analyzer.muscularity
}

/// Return the BMI estimate.
#[allow(dead_code)]
pub fn bmi_estimate(analyzer: &BodyShapeAnalyzer) -> f32 {
    analyzer.bmi
}

/// Serialize to JSON-like string.
#[allow(dead_code)]
pub fn analyzer_to_json(analyzer: &BodyShapeAnalyzer) -> String {
    format!(
        "{{\"body_type\":\"{}\",\"symmetry\":{},\"proportion\":{},\"muscularity\":{},\"bmi\":{}}}",
        analyzer.body_type, analyzer.symmetry, analyzer.proportion, analyzer.muscularity, analyzer.bmi
    )
}

/// Analyze from a parameter map (expects keys: height, weight, shoulder_width, hip_width).
#[allow(dead_code)]
pub fn analyze_from_params(params: &HashMap<String, f32>) -> BodyShapeAnalyzer {
    let h = params.get("height").copied().unwrap_or(1.7);
    let w = params.get("weight").copied().unwrap_or(70.0);
    let sw = params.get("shoulder_width").copied().unwrap_or(0.45);
    let hw = params.get("hip_width").copied().unwrap_or(0.4);
    analyze_body_shape(h, w, sw, hw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_balanced() {
        let a = analyze_body_shape(1.75, 70.0, 0.45, 0.45);
        assert_eq!(body_type_classification(&a), "balanced");
    }

    #[test]
    fn test_analyze_inverted_triangle() {
        let a = analyze_body_shape(1.75, 70.0, 0.6, 0.4);
        assert_eq!(body_type_classification(&a), "inverted_triangle");
    }

    #[test]
    fn test_analyze_pear() {
        let a = analyze_body_shape(1.75, 70.0, 0.3, 0.5);
        assert_eq!(body_type_classification(&a), "pear");
    }

    #[test]
    fn test_bmi() {
        let a = analyze_body_shape(1.75, 70.0, 0.45, 0.45);
        let expected = 70.0 / (1.75 * 1.75);
        assert!((bmi_estimate(&a) - expected).abs() < 0.1);
    }

    #[test]
    fn test_symmetry_score() {
        let a = analyze_body_shape(1.75, 70.0, 0.45, 0.45);
        assert!(symmetry_score_bsa(&a) > 0.0);
    }

    #[test]
    fn test_proportion_score() {
        let a = analyze_body_shape(1.75, 70.0, 0.45, 0.45);
        assert!((0.0..=1.0).contains(&proportion_score_bsa(&a)));
    }

    #[test]
    fn test_muscularity() {
        let a = analyze_body_shape(1.75, 70.0, 0.45, 0.45);
        assert!((muscularity_score(&a) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let a = analyze_body_shape(1.75, 70.0, 0.45, 0.45);
        let json = analyzer_to_json(&a);
        assert!(json.contains("\"body_type\""));
    }

    #[test]
    fn test_from_params() {
        let mut params = HashMap::new();
        params.insert("height".to_string(), 1.8);
        params.insert("weight".to_string(), 80.0);
        params.insert("shoulder_width".to_string(), 0.5);
        params.insert("hip_width".to_string(), 0.42);
        let a = analyze_from_params(&params);
        assert!(bmi_estimate(&a) > 0.0);
    }

    #[test]
    fn test_zero_height() {
        let a = analyze_body_shape(0.0, 70.0, 0.45, 0.45);
        assert!((bmi_estimate(&a) - 0.0).abs() < 1e-6);
    }
}
