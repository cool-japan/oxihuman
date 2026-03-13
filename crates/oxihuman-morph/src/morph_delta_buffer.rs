#![allow(dead_code)]

/// Buffer of morph delta entries.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphDeltaBuffer {
    deltas: Vec<[f32; 3]>,
}

#[allow(dead_code)]
pub fn new_delta_buffer() -> MorphDeltaBuffer { MorphDeltaBuffer { deltas: Vec::new() } }

#[allow(dead_code)]
pub fn add_delta_db(buf: &mut MorphDeltaBuffer, dx: f32, dy: f32, dz: f32) {
    buf.deltas.push([dx, dy, dz]);
}

#[allow(dead_code)]
pub fn apply_delta_buffer(buf: &MorphDeltaBuffer, positions: &mut [[f32; 3]], weight: f32) {
    for (i, d) in buf.deltas.iter().enumerate() {
        if i < positions.len() {
            positions[i][0] += d[0] * weight;
            positions[i][1] += d[1] * weight;
            positions[i][2] += d[2] * weight;
        }
    }
}

#[allow(dead_code)]
pub fn delta_count_db(buf: &MorphDeltaBuffer) -> usize { buf.deltas.len() }

#[allow(dead_code)]
pub fn delta_at_db(buf: &MorphDeltaBuffer, idx: usize) -> Option<[f32; 3]> {
    buf.deltas.get(idx).copied()
}

#[allow(dead_code)]
pub fn delta_buffer_to_bytes(buf: &MorphDeltaBuffer) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(buf.deltas.len() * 12);
    for d in &buf.deltas {
        for c in d { bytes.extend_from_slice(&c.to_le_bytes()); }
    }
    bytes
}

#[allow(dead_code)]
pub fn clear_delta_buffer(buf: &mut MorphDeltaBuffer) { buf.deltas.clear(); }

#[allow(dead_code)]
pub fn delta_buffer_size(buf: &MorphDeltaBuffer) -> usize { buf.deltas.len() * 12 }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { assert_eq!(delta_count_db(&new_delta_buffer()), 0); }
    #[test] fn test_add() {
        let mut b = new_delta_buffer();
        add_delta_db(&mut b, 1.0, 2.0, 3.0);
        assert_eq!(delta_count_db(&b), 1);
    }
    #[test] fn test_at() {
        let mut b = new_delta_buffer();
        add_delta_db(&mut b, 1.0, 0.0, 0.0);
        let d = delta_at_db(&b, 0).expect("should succeed");
        assert!((d[0] - 1.0).abs() < 1e-6);
    }
    #[test] fn test_at_oob() { assert!(delta_at_db(&new_delta_buffer(), 0).is_none()); }
    #[test] fn test_apply() {
        let mut b = new_delta_buffer();
        add_delta_db(&mut b, 1.0, 0.0, 0.0);
        let mut pos = [[0.0f32; 3]];
        apply_delta_buffer(&b, &mut pos, 0.5);
        assert!((pos[0][0] - 0.5).abs() < 1e-6);
    }
    #[test] fn test_to_bytes() {
        let mut b = new_delta_buffer();
        add_delta_db(&mut b, 1.0, 2.0, 3.0);
        assert_eq!(delta_buffer_to_bytes(&b).len(), 12);
    }
    #[test] fn test_clear() {
        let mut b = new_delta_buffer();
        add_delta_db(&mut b, 1.0, 0.0, 0.0);
        clear_delta_buffer(&mut b);
        assert_eq!(delta_count_db(&b), 0);
    }
    #[test] fn test_size() {
        let mut b = new_delta_buffer();
        add_delta_db(&mut b, 0.0, 0.0, 0.0);
        assert_eq!(delta_buffer_size(&b), 12);
    }
    #[test] fn test_apply_empty() {
        let b = new_delta_buffer();
        let mut pos = [[1.0f32; 3]];
        apply_delta_buffer(&b, &mut pos, 1.0);
        assert!((pos[0][0] - 1.0).abs() < 1e-6);
    }
    #[test] fn test_multiple_deltas() {
        let mut b = new_delta_buffer();
        add_delta_db(&mut b, 1.0, 0.0, 0.0);
        add_delta_db(&mut b, 0.0, 1.0, 0.0);
        assert_eq!(delta_count_db(&b), 2);
    }
}
