// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export IK/FK blend weight data per bone chain.

/// Blend mode.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlendMode {
    FullIk,
    FullFk,
    Blended(f32),
}

/// IK/FK blend record for a single chain.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IkFkBlendRecord {
    pub chain_name: String,
    pub blend: BlendMode,
}

/// Export of IK/FK blend data.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct IkFkBlendExport {
    pub records: Vec<IkFkBlendRecord>,
}

/// Create a new export.
#[allow(dead_code)]
pub fn new_ik_fk_blend_export() -> IkFkBlendExport {
    IkFkBlendExport::default()
}

/// Add a record.
#[allow(dead_code)]
pub fn add_record(export: &mut IkFkBlendExport, chain_name: &str, blend: BlendMode) {
    export.records.push(IkFkBlendRecord {
        chain_name: chain_name.to_string(),
        blend,
    });
}

/// Get the IK influence weight (0=FK, 1=IK) for a chain.
#[allow(dead_code)]
pub fn ik_weight(record: &IkFkBlendRecord) -> f32 {
    match record.blend {
        BlendMode::FullIk => 1.0,
        BlendMode::FullFk => 0.0,
        BlendMode::Blended(w) => w.clamp(0.0, 1.0),
    }
}

/// Find a record by chain name.
#[allow(dead_code)]
pub fn find_record<'a>(export: &'a IkFkBlendExport, name: &str) -> Option<&'a IkFkBlendRecord> {
    export.records.iter().find(|r| r.chain_name == name)
}

/// Set blend for a named chain (adds if not present).
#[allow(dead_code)]
pub fn set_blend(export: &mut IkFkBlendExport, name: &str, blend: BlendMode) {
    if let Some(r) = export.records.iter_mut().find(|r| r.chain_name == name) {
        r.blend = blend;
    } else {
        add_record(export, name, blend);
    }
}

/// Count chains with IK weight above threshold.
#[allow(dead_code)]
pub fn ik_dominant_count(export: &IkFkBlendExport, threshold: f32) -> usize {
    export
        .records
        .iter()
        .filter(|r| ik_weight(r) > threshold)
        .count()
}

/// Serialise to flat buffer: [ik_weight per chain].
#[allow(dead_code)]
pub fn serialise_ik_fk(export: &IkFkBlendExport) -> Vec<f32> {
    export.records.iter().map(ik_weight).collect()
}

/// Check that all IK weights are in [0, 1].
#[allow(dead_code)]
pub fn all_weights_valid(export: &IkFkBlendExport) -> bool {
    export
        .records
        .iter()
        .all(|r| (0.0..=1.0).contains(&ik_weight(r)))
}

/// Average IK weight across all chains.
#[allow(dead_code)]
pub fn average_ik_weight(export: &IkFkBlendExport) -> f32 {
    if export.records.is_empty() {
        return 0.0;
    }
    export.records.iter().map(ik_weight).sum::<f32>() / export.records.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn basic_export() -> IkFkBlendExport {
        let mut e = new_ik_fk_blend_export();
        add_record(&mut e, "arm_l", BlendMode::FullIk);
        add_record(&mut e, "arm_r", BlendMode::FullFk);
        add_record(&mut e, "leg_l", BlendMode::Blended(0.7));
        e
    }

    #[test]
    fn test_ik_weight_full_ik() {
        let r = IkFkBlendRecord {
            chain_name: "x".to_string(),
            blend: BlendMode::FullIk,
        };
        assert!((ik_weight(&r) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_ik_weight_full_fk() {
        let r = IkFkBlendRecord {
            chain_name: "x".to_string(),
            blend: BlendMode::FullFk,
        };
        assert!((ik_weight(&r) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_ik_weight_blended() {
        let r = IkFkBlendRecord {
            chain_name: "x".to_string(),
            blend: BlendMode::Blended(0.4),
        };
        assert!((ik_weight(&r) - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_find_record_found() {
        let e = basic_export();
        assert!(find_record(&e, "arm_l").is_some());
    }

    #[test]
    fn test_find_record_not_found() {
        let e = basic_export();
        assert!(find_record(&e, "spine").is_none());
    }

    #[test]
    fn test_set_blend_existing() {
        let mut e = basic_export();
        set_blend(&mut e, "arm_l", BlendMode::FullFk);
        assert!((ik_weight(find_record(&e, "arm_l").unwrap()) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_blend_new() {
        let mut e = basic_export();
        set_blend(&mut e, "spine", BlendMode::Blended(0.5));
        assert!(find_record(&e, "spine").is_some());
    }

    #[test]
    fn test_ik_dominant_count() {
        let e = basic_export();
        assert_eq!(ik_dominant_count(&e, 0.5), 2); // arm_l(1.0), leg_l(0.7)
    }

    #[test]
    fn test_serialise_length() {
        let e = basic_export();
        assert_eq!(serialise_ik_fk(&e).len(), 3);
    }

    #[test]
    fn test_all_weights_valid() {
        let e = basic_export();
        assert!(all_weights_valid(&e));
    }
}
