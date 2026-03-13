// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Decal batch renderer — groups decals into draw batches for efficient GPU submission.

/// A single decal instance.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct DecalInstance {
    pub transform: [[f32; 4]; 4],
    pub atlas_uv: [f32; 4],
    pub opacity: f32,
    pub layer: u32,
    pub enabled: bool,
}

impl Default for DecalInstance {
    fn default() -> Self {
        Self {
            transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            atlas_uv: [0.0, 0.0, 1.0, 1.0],
            opacity: 1.0,
            layer: 0,
            enabled: true,
        }
    }
}

/// Batch configuration.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct DecalBatchConfig {
    pub max_per_batch: usize,
    pub sort_by_layer: bool,
}

impl Default for DecalBatchConfig {
    fn default() -> Self {
        Self {
            max_per_batch: 256,
            sort_by_layer: true,
        }
    }
}

/// Decal batch manager.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct DecalBatch {
    pub config: DecalBatchConfig,
    pub instances: Vec<DecalInstance>,
}

/// Create new batch.
#[allow(dead_code)]
pub fn new_decal_batch(cfg: DecalBatchConfig) -> DecalBatch {
    DecalBatch {
        config: cfg,
        instances: Vec::new(),
    }
}

/// Add a decal instance.
#[allow(dead_code)]
pub fn add_decal_instance(b: &mut DecalBatch, inst: DecalInstance) -> Option<usize> {
    if b.instances.len() >= b.config.max_per_batch {
        return None;
    }
    let idx = b.instances.len();
    b.instances.push(inst);
    Some(idx)
}

/// Remove by index.
#[allow(dead_code)]
pub fn remove_decal_instance(b: &mut DecalBatch, idx: usize) {
    if idx < b.instances.len() {
        b.instances.remove(idx);
    }
}

/// Instance count.
#[allow(dead_code)]
pub fn decal_instance_count(b: &DecalBatch) -> usize {
    b.instances.len()
}

/// Enabled instance count.
#[allow(dead_code)]
pub fn enabled_decal_count(b: &DecalBatch) -> usize {
    b.instances.iter().filter(|i| i.enabled).count()
}

/// Sort instances by layer.
#[allow(dead_code)]
pub fn sort_decals_by_layer(b: &mut DecalBatch) {
    b.instances.sort_by_key(|i| i.layer);
}

/// Compute number of sub-batches needed.
#[allow(dead_code)]
pub fn sub_batch_count(b: &DecalBatch) -> usize {
    if b.instances.is_empty() {
        return 0;
    }
    b.instances.len().div_ceil(b.config.max_per_batch)
}

/// Clear all instances.
#[allow(dead_code)]
pub fn clear_decal_batch(b: &mut DecalBatch) {
    b.instances.clear();
}

/// Export to JSON-like string.
#[allow(dead_code)]
pub fn decal_batch_to_json(b: &DecalBatch) -> String {
    format!(
        r#"{{"count":{},"enabled":{}}}"#,
        b.instances.len(),
        enabled_decal_count(b)
    )
}

/// Memory estimate in bytes.
#[allow(dead_code)]
pub fn decal_batch_memory_bytes(b: &DecalBatch) -> usize {
    b.instances.len() * std::mem::size_of::<DecalInstance>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_batch_empty() {
        let b = new_decal_batch(DecalBatchConfig::default());
        assert_eq!(decal_instance_count(&b), 0);
    }

    #[test]
    fn add_instance() {
        let mut b = new_decal_batch(DecalBatchConfig::default());
        let idx = add_decal_instance(&mut b, DecalInstance::default());
        assert_eq!(idx, Some(0));
    }

    #[test]
    fn capacity_limit() {
        let mut b = new_decal_batch(DecalBatchConfig {
            max_per_batch: 2,
            ..Default::default()
        });
        add_decal_instance(&mut b, DecalInstance::default());
        add_decal_instance(&mut b, DecalInstance::default());
        let r = add_decal_instance(&mut b, DecalInstance::default());
        assert!(r.is_none());
    }

    #[test]
    fn remove_instance() {
        let mut b = new_decal_batch(DecalBatchConfig::default());
        add_decal_instance(&mut b, DecalInstance::default());
        remove_decal_instance(&mut b, 0);
        assert_eq!(decal_instance_count(&b), 0);
    }

    #[test]
    fn enabled_count() {
        let mut b = new_decal_batch(DecalBatchConfig::default());
        add_decal_instance(
            &mut b,
            DecalInstance {
                enabled: true,
                ..Default::default()
            },
        );
        add_decal_instance(
            &mut b,
            DecalInstance {
                enabled: false,
                ..Default::default()
            },
        );
        assert_eq!(enabled_decal_count(&b), 1);
    }

    #[test]
    fn sort_by_layer() {
        let mut b = new_decal_batch(DecalBatchConfig::default());
        add_decal_instance(
            &mut b,
            DecalInstance {
                layer: 3,
                ..Default::default()
            },
        );
        add_decal_instance(
            &mut b,
            DecalInstance {
                layer: 1,
                ..Default::default()
            },
        );
        sort_decals_by_layer(&mut b);
        assert_eq!(b.instances[0].layer, 1);
    }

    #[test]
    fn sub_batch_count_zero_when_empty() {
        let b = new_decal_batch(DecalBatchConfig::default());
        assert_eq!(sub_batch_count(&b), 0);
    }

    #[test]
    fn memory_bytes_positive() {
        let mut b = new_decal_batch(DecalBatchConfig::default());
        add_decal_instance(&mut b, DecalInstance::default());
        assert!(decal_batch_memory_bytes(&b) > 0);
    }

    #[test]
    fn json_contains_count() {
        let b = new_decal_batch(DecalBatchConfig::default());
        assert!(decal_batch_to_json(&b).contains("count"));
    }

    #[test]
    fn clear_empties() {
        let mut b = new_decal_batch(DecalBatchConfig::default());
        add_decal_instance(&mut b, DecalInstance::default());
        clear_decal_batch(&mut b);
        assert_eq!(decal_instance_count(&b), 0);
    }
}
