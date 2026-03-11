// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A matrix that determines which collision layers can interact.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollisionLayerMatrix {
    layers: u32,
    matrix: Vec<bool>,
    layer_names: Vec<String>,
}

#[allow(dead_code)]
impl CollisionLayerMatrix {
    pub fn new(layers: u32) -> Self {
        let n = layers as usize;
        Self {
            layers,
            matrix: vec![false; n * n],
            layer_names: (0..n).map(|i| format!("layer_{i}")).collect(),
        }
    }

    pub fn all_enabled(layers: u32) -> Self {
        let n = layers as usize;
        Self {
            layers,
            matrix: vec![true; n * n],
            layer_names: (0..n).map(|i| format!("layer_{i}")).collect(),
        }
    }

    fn idx(&self, a: u32, b: u32) -> usize {
        (a as usize) * (self.layers as usize) + (b as usize)
    }

    pub fn set_interaction(&mut self, a: u32, b: u32, enabled: bool) {
        if a < self.layers && b < self.layers {
            let i1 = self.idx(a, b);
            let i2 = self.idx(b, a);
            self.matrix[i1] = enabled;
            self.matrix[i2] = enabled;
        }
    }

    pub fn can_interact(&self, a: u32, b: u32) -> bool {
        if a >= self.layers || b >= self.layers {
            return false;
        }
        self.matrix[self.idx(a, b)]
    }

    pub fn set_layer_name(&mut self, layer: u32, name: &str) {
        if (layer as usize) < self.layer_names.len() {
            self.layer_names[layer as usize] = name.to_string();
        }
    }

    pub fn layer_name(&self, layer: u32) -> Option<&str> {
        self.layer_names.get(layer as usize).map(|s| s.as_str())
    }

    pub fn layer_count(&self) -> u32 {
        self.layers
    }

    pub fn interaction_count(&self) -> usize {
        let mut count = 0;
        for a in 0..self.layers {
            for b in a..self.layers {
                if self.can_interact(a, b) {
                    count += 1;
                }
            }
        }
        count
    }

    pub fn disable_all(&mut self) {
        self.matrix.fill(false);
    }

    pub fn enable_all(&mut self) {
        self.matrix.fill(true);
    }

    pub fn enable_self_interaction(&mut self) {
        for i in 0..self.layers {
            self.set_interaction(i, i, true);
        }
    }

    pub fn interacting_layers(&self, layer: u32) -> Vec<u32> {
        (0..self.layers)
            .filter(|&b| self.can_interact(layer, b))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_all_disabled() {
        let m = CollisionLayerMatrix::new(4);
        assert!(!m.can_interact(0, 1));
    }

    #[test]
    fn test_all_enabled() {
        let m = CollisionLayerMatrix::all_enabled(4);
        assert!(m.can_interact(0, 1));
        assert!(m.can_interact(3, 2));
    }

    #[test]
    fn test_set_interaction() {
        let mut m = CollisionLayerMatrix::new(4);
        m.set_interaction(0, 1, true);
        assert!(m.can_interact(0, 1));
        assert!(m.can_interact(1, 0));
    }

    #[test]
    fn test_symmetric() {
        let mut m = CollisionLayerMatrix::new(4);
        m.set_interaction(2, 3, true);
        assert!(m.can_interact(3, 2));
    }

    #[test]
    fn test_out_of_bounds() {
        let m = CollisionLayerMatrix::new(4);
        assert!(!m.can_interact(10, 0));
    }

    #[test]
    fn test_layer_name() {
        let mut m = CollisionLayerMatrix::new(4);
        m.set_layer_name(0, "default");
        assert_eq!(m.layer_name(0), Some("default"));
    }

    #[test]
    fn test_interaction_count() {
        let mut m = CollisionLayerMatrix::new(3);
        m.set_interaction(0, 1, true);
        m.set_interaction(1, 2, true);
        assert_eq!(m.interaction_count(), 2);
    }

    #[test]
    fn test_disable_all() {
        let mut m = CollisionLayerMatrix::all_enabled(4);
        m.disable_all();
        assert!(!m.can_interact(0, 1));
    }

    #[test]
    fn test_self_interaction() {
        let mut m = CollisionLayerMatrix::new(3);
        m.enable_self_interaction();
        assert!(m.can_interact(0, 0));
        assert!(m.can_interact(1, 1));
        assert!(!m.can_interact(0, 1));
    }

    #[test]
    fn test_interacting_layers() {
        let mut m = CollisionLayerMatrix::new(4);
        m.set_interaction(0, 1, true);
        m.set_interaction(0, 3, true);
        let layers = m.interacting_layers(0);
        assert_eq!(layers, vec![1, 3]);
    }
}
