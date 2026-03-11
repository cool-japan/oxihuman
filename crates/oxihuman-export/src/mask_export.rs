// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Boolean mask export (vertex selection masks).

#![allow(dead_code)]

/// Configuration for mask export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MaskExportConfig {
    /// Name of the mask layer.
    pub name: String,
    /// Whether the mask is inverted on export.
    pub invert: bool,
}

/// A boolean vertex selection mask.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MaskExport {
    /// Mask configuration.
    pub config: MaskExportConfig,
    /// Per-vertex mask values.
    pub mask: Vec<bool>,
}

/// Returns the default [`MaskExportConfig`].
#[allow(dead_code)]
pub fn default_mask_export_config() -> MaskExportConfig {
    MaskExportConfig {
        name: "SelectionMask".to_string(),
        invert: false,
    }
}

/// Creates a new [`MaskExport`] with `vertex_count` entries, all set to `false`.
#[allow(dead_code)]
pub fn new_mask_export(config: MaskExportConfig, vertex_count: usize) -> MaskExport {
    MaskExport { config, mask: vec![false; vertex_count] }
}

/// Sets the mask value at `index`.
#[allow(dead_code)]
pub fn mask_set(export: &mut MaskExport, index: usize, value: bool) {
    if index < export.mask.len() {
        export.mask[index] = value;
    }
}

/// Returns the mask value at `index`, or `false` if out of range.
#[allow(dead_code)]
pub fn mask_get(export: &MaskExport, index: usize) -> bool {
    export.mask.get(index).copied().unwrap_or(false)
}

/// Returns the number of `true` entries.
#[allow(dead_code)]
pub fn mask_count_set(export: &MaskExport) -> usize {
    export.mask.iter().filter(|&&v| v).count()
}

/// Returns the number of `false` entries.
#[allow(dead_code)]
pub fn mask_count_clear(export: &MaskExport) -> usize {
    export.mask.iter().filter(|&&v| !v).count()
}

/// Inverts all mask values in-place.
#[allow(dead_code)]
pub fn mask_invert(export: &mut MaskExport) {
    for v in &mut export.mask {
        *v = !*v;
    }
}

/// Packs the mask as a byte array (1 bit per vertex, padded to bytes).
#[allow(dead_code)]
pub fn mask_to_bytes(export: &MaskExport) -> Vec<u8> {
    let n_bytes = export.mask.len().div_ceil(8);
    let mut bytes = vec![0u8; n_bytes];
    for (i, &val) in export.mask.iter().enumerate() {
        if val {
            bytes[i / 8] |= 1 << (i % 8);
        }
    }
    bytes
}

/// Serialises to a minimal JSON string.
#[allow(dead_code)]
pub fn mask_to_json(export: &MaskExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"vertex_count\":{},\"set_count\":{}}}",
        export.config.name,
        export.mask.len(),
        mask_count_set(export)
    )
}

/// Validates that the mask has at least one entry.
#[allow(dead_code)]
pub fn mask_validate(export: &MaskExport) -> bool {
    !export.mask.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_mask_export_config();
        assert_eq!(cfg.name, "SelectionMask");
        assert!(!cfg.invert);
    }

    #[test]
    fn test_new_mask_all_false() {
        let export = new_mask_export(default_mask_export_config(), 8);
        assert_eq!(mask_count_set(&export), 0);
        assert_eq!(mask_count_clear(&export), 8);
    }

    #[test]
    fn test_set_and_get() {
        let mut export = new_mask_export(default_mask_export_config(), 4);
        mask_set(&mut export, 2, true);
        assert!(mask_get(&export, 2));
        assert!(!mask_get(&export, 0));
    }

    #[test]
    fn test_count_set() {
        let mut export = new_mask_export(default_mask_export_config(), 4);
        mask_set(&mut export, 0, true);
        mask_set(&mut export, 3, true);
        assert_eq!(mask_count_set(&export), 2);
    }

    #[test]
    fn test_invert() {
        let mut export = new_mask_export(default_mask_export_config(), 4);
        mask_set(&mut export, 0, true);
        mask_invert(&mut export);
        assert!(!mask_get(&export, 0));
        assert!(mask_get(&export, 1));
    }

    #[test]
    fn test_to_bytes() {
        let mut export = new_mask_export(default_mask_export_config(), 8);
        mask_set(&mut export, 0, true);
        let bytes = mask_to_bytes(&export);
        assert_eq!(bytes[0] & 1, 1);
    }

    #[test]
    fn test_to_json() {
        let export = new_mask_export(default_mask_export_config(), 5);
        let json = mask_to_json(&export);
        assert!(json.contains("vertex_count"));
    }

    #[test]
    fn test_validate_nonempty() {
        let export = new_mask_export(default_mask_export_config(), 3);
        assert!(mask_validate(&export));
    }

    #[test]
    fn test_validate_empty() {
        let export = new_mask_export(default_mask_export_config(), 0);
        assert!(!mask_validate(&export));
    }
}
