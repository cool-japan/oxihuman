// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export blend/morph weights per vertex.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendWeightEntry {
    pub target_name: String,
    pub weights: Vec<f32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendWeightExport {
    pub entries: Vec<BlendWeightEntry>,
}

#[allow(dead_code)]
pub fn new_blend_weight_export() -> BlendWeightExport {
    BlendWeightExport { entries: Vec::new() }
}

#[allow(dead_code)]
pub fn bwe_add(bwe: &mut BlendWeightExport, name: &str, weights: Vec<f32>) {
    bwe.entries.push(BlendWeightEntry { target_name: name.to_string(), weights });
}

#[allow(dead_code)]
pub fn bwe_target_count(bwe: &BlendWeightExport) -> usize { bwe.entries.len() }

#[allow(dead_code)]
pub fn bwe_find<'a>(bwe: &'a BlendWeightExport, name: &str) -> Option<&'a BlendWeightEntry> {
    bwe.entries.iter().find(|e| e.target_name == name)
}

#[allow(dead_code)]
pub fn bwe_max_weight(entry: &BlendWeightEntry) -> f32 {
    entry.weights.iter().copied().fold(0.0f32, f32::max)
}

#[allow(dead_code)]
pub fn bwe_nonzero_count(entry: &BlendWeightEntry) -> usize {
    entry.weights.iter().filter(|&&w| w.abs() > 1e-6).count()
}

#[allow(dead_code)]
pub fn bwe_validate(bwe: &BlendWeightExport) -> bool {
    !bwe.entries.is_empty() && bwe.entries.iter().all(|e| !e.weights.is_empty())
}

#[allow(dead_code)]
pub fn bwe_total_weights(bwe: &BlendWeightExport) -> usize {
    bwe.entries.iter().map(|e| e.weights.len()).sum()
}

#[allow(dead_code)]
pub fn bwe_to_json(bwe: &BlendWeightExport) -> String {
    let items: Vec<String> = bwe.entries.iter().map(|e| format!("{{\"target\":\"{}\",\"count\":{}}}", e.target_name, e.weights.len())).collect();
    format!("{{\"targets\":[{}]}}", items.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> BlendWeightExport {
        let mut b = new_blend_weight_export();
        bwe_add(&mut b, "smile", vec![0.0, 0.5, 1.0]);
        b
    }

    #[test] fn test_new() { assert_eq!(bwe_target_count(&new_blend_weight_export()), 0); }
    #[test] fn test_add() { assert_eq!(bwe_target_count(&sample()), 1); }
    #[test] fn test_find() { assert!(bwe_find(&sample(), "smile").is_some()); }
    #[test] fn test_find_missing() { assert!(bwe_find(&sample(), "nope").is_none()); }
    #[test] fn test_max_weight() { let s = sample(); assert!((bwe_max_weight(&s.entries[0]) - 1.0).abs() < 1e-6); }
    #[test] fn test_nonzero() { let s = sample(); assert_eq!(bwe_nonzero_count(&s.entries[0]), 2); }
    #[test] fn test_validate() { assert!(bwe_validate(&sample())); }
    #[test] fn test_total() { assert_eq!(bwe_total_weights(&sample()), 3); }
    #[test] fn test_to_json() { assert!(bwe_to_json(&sample()).contains("smile")); }
    #[test] fn test_empty_invalid() { assert!(!bwe_validate(&new_blend_weight_export())); }
}
