#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayerMask(u32);

#[allow(dead_code)]
pub fn new_layer_mask() -> LayerMask {
    LayerMask(0)
}

#[allow(dead_code)]
pub fn layer_set_mask(mask: &mut LayerMask, layer: u32) {
    if layer < 32 {
        mask.0 |= 1u32 << layer;
    }
}

#[allow(dead_code)]
pub fn layer_clear_mask(mask: &mut LayerMask, layer: u32) {
    if layer < 32 {
        mask.0 &= !(1u32 << layer);
    }
}

#[allow(dead_code)]
pub fn layer_test_mask(mask: &LayerMask, layer: u32) -> bool {
    if layer < 32 {
        (mask.0 >> layer) & 1 == 1
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn layers_collide(a: &LayerMask, b: &LayerMask) -> bool {
    (a.0 & b.0) != 0
}

#[allow(dead_code)]
pub fn layer_mask_to_u32(mask: &LayerMask) -> u32 {
    mask.0
}

#[allow(dead_code)]
pub fn layer_mask_from_u32(v: u32) -> LayerMask {
    LayerMask(v)
}

#[allow(dead_code)]
pub fn layer_mask_all() -> LayerMask {
    LayerMask(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = new_layer_mask();
        assert_eq!(layer_mask_to_u32(&m), 0);
    }

    #[test]
    fn test_set() {
        let mut m = new_layer_mask();
        layer_set_mask(&mut m, 0);
        assert!(layer_test_mask(&m, 0));
    }

    #[test]
    fn test_clear() {
        let mut m = new_layer_mask();
        layer_set_mask(&mut m, 3);
        layer_clear_mask(&mut m, 3);
        assert!(!layer_test_mask(&m, 3));
    }

    #[test]
    fn test_collide() {
        let mut a = new_layer_mask();
        let mut b = new_layer_mask();
        layer_set_mask(&mut a, 1);
        layer_set_mask(&mut b, 1);
        assert!(layers_collide(&a, &b));
    }

    #[test]
    fn test_no_collide() {
        let mut a = new_layer_mask();
        let mut b = new_layer_mask();
        layer_set_mask(&mut a, 0);
        layer_set_mask(&mut b, 1);
        assert!(!layers_collide(&a, &b));
    }

    #[test]
    fn test_from_u32() {
        let m = layer_mask_from_u32(0xFF);
        assert!(layer_test_mask(&m, 7));
        assert!(!layer_test_mask(&m, 8));
    }

    #[test]
    fn test_all() {
        let m = layer_mask_all();
        assert!(layer_test_mask(&m, 0));
        assert!(layer_test_mask(&m, 31));
    }

    #[test]
    fn test_out_of_bounds() {
        let m = new_layer_mask();
        assert!(!layer_test_mask(&m, 32));
    }

    #[test]
    fn test_multiple_layers() {
        let mut m = new_layer_mask();
        layer_set_mask(&mut m, 0);
        layer_set_mask(&mut m, 5);
        layer_set_mask(&mut m, 10);
        assert_eq!(layer_mask_to_u32(&m), (1 << 0) | (1 << 5) | (1 << 10));
    }

    #[test]
    fn test_to_u32() {
        let mut m = new_layer_mask();
        layer_set_mask(&mut m, 0);
        assert_eq!(layer_mask_to_u32(&m), 1);
    }
}
