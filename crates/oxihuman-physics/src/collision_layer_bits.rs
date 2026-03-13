// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Collision layer/mask bit management.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollisionLayerBits {
    pub layer: u32,
    pub mask: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollisionLayerBitsConfig {
    pub num_layers: usize,
}

#[allow(dead_code)]
pub fn default_collision_layer_bits_config() -> CollisionLayerBitsConfig {
    CollisionLayerBitsConfig { num_layers: 32 }
}

#[allow(dead_code)]
pub fn new_collision_layer_bits(layer: u32, mask: u32) -> CollisionLayerBits {
    CollisionLayerBits { layer, mask }
}

#[allow(dead_code)]
pub fn cl_can_collide(a: &CollisionLayerBits, b: &CollisionLayerBits) -> bool {
    (a.mask & b.layer) != 0 && (b.mask & a.layer) != 0
}

#[allow(dead_code)]
pub fn cl_set_layer(cl: &mut CollisionLayerBits, layer: u32) {
    cl.layer = layer;
}

#[allow(dead_code)]
pub fn cl_set_mask(cl: &mut CollisionLayerBits, mask: u32) {
    cl.mask = mask;
}

#[allow(dead_code)]
pub fn cl_add_mask_bit(cl: &mut CollisionLayerBits, bit: u32) {
    cl.mask |= 1 << bit;
}

#[allow(dead_code)]
pub fn cl_remove_mask_bit(cl: &mut CollisionLayerBits, bit: u32) {
    cl.mask &= !(1 << bit);
}

#[allow(dead_code)]
pub fn cl_bits_to_json(cl: &CollisionLayerBits) -> String {
    format!("{{\"layer\":{},\"mask\":{}}}", cl.layer, cl.mask)
}

#[allow(dead_code)]
pub fn cl_layer_name(layer: u32) -> String {
    format!("layer_{}", layer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_collision_layer_bits_config();
        assert_eq!(cfg.num_layers, 32);
    }

    #[test]
    fn test_new_collision_layer() {
        let cl = new_collision_layer_bits(1, 0xFF);
        assert_eq!(cl.layer, 1);
        assert_eq!(cl.mask, 0xFF);
    }

    #[test]
    fn test_can_collide_mutual() {
        let a = new_collision_layer_bits(0b0001, 0b0010);
        let b = new_collision_layer_bits(0b0010, 0b0001);
        assert!(cl_can_collide(&a, &b));
    }

    #[test]
    fn test_cannot_collide() {
        let a = new_collision_layer_bits(0b0001, 0b0100);
        let b = new_collision_layer_bits(0b0010, 0b0001);
        assert!(!cl_can_collide(&a, &b));
    }

    #[test]
    fn test_set_layer() {
        let mut cl = new_collision_layer_bits(1, 0);
        cl_set_layer(&mut cl, 5);
        assert_eq!(cl.layer, 5);
    }

    #[test]
    fn test_add_mask_bit() {
        let mut cl = new_collision_layer_bits(1, 0);
        cl_add_mask_bit(&mut cl, 3);
        assert_eq!(cl.mask, 0b1000);
    }

    #[test]
    fn test_remove_mask_bit() {
        let mut cl = new_collision_layer_bits(1, 0b1111);
        cl_remove_mask_bit(&mut cl, 1);
        assert_eq!(cl.mask, 0b1101);
    }

    #[test]
    fn test_to_json() {
        let cl = new_collision_layer_bits(2, 4);
        let j = cl_bits_to_json(&cl);
        assert!(j.contains("\"layer\":2"));
    }

    #[test]
    fn test_layer_name() {
        let name = cl_layer_name(7);
        assert_eq!(name, "layer_7");
    }
}
