#![allow(dead_code)]

//! Compressed morph delta (position + normal deltas).

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct MorphDeltaEntry {
    pub vertex_index: u32,
    pub position_delta: [f32; 3],
    pub normal_delta: [f32; 3],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphDeltaV2 {
    pub name: String,
    pub deltas: Vec<MorphDeltaEntry>,
    pub compressed: bool,
    pub scale: f32,
}

#[allow(dead_code)]
pub fn new_morph_delta_v2(name: &str) -> MorphDeltaV2 {
    MorphDeltaV2 {
        name: name.to_string(),
        deltas: Vec::new(),
        compressed: false,
        scale: 1.0,
    }
}

#[allow(dead_code)]
pub fn mdv2_add_delta(
    md: &mut MorphDeltaV2,
    vertex_index: u32,
    position_delta: [f32; 3],
    normal_delta: [f32; 3],
) {
    md.deltas.push(MorphDeltaEntry {
        vertex_index,
        position_delta,
        normal_delta,
    });
}

#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn mdv2_apply(md: &MorphDeltaV2, weight: f32, positions: &mut [[f32; 3]], normals: &mut [[f32; 3]]) {
    let effective = weight * md.scale;
    for entry in &md.deltas {
        let idx = entry.vertex_index as usize;
        if idx < positions.len() {
            for k in 0..3 {
                positions[idx][k] += entry.position_delta[k] * effective;
            }
        }
        if idx < normals.len() {
            for k in 0..3 {
                normals[idx][k] += entry.normal_delta[k] * effective;
            }
        }
    }
}

#[allow(dead_code)]
pub fn mdv2_delta_count(md: &MorphDeltaV2) -> usize {
    md.deltas.len()
}

#[allow(dead_code)]
pub fn mdv2_compress(md: &mut MorphDeltaV2) {
    md.compressed = true;
}

#[allow(dead_code)]
pub fn mdv2_magnitude(md: &MorphDeltaV2) -> f32 {
    md.deltas
        .iter()
        .map(|e| {
            let sq: f32 = e.position_delta.iter().map(|x| x * x).sum();
            sq.sqrt()
        })
        .fold(0.0_f32, f32::max)
}

#[allow(dead_code)]
pub fn mdv2_clear(md: &mut MorphDeltaV2) {
    md.deltas.clear();
    md.compressed = false;
}

#[allow(dead_code)]
pub fn mdv2_to_json(md: &MorphDeltaV2) -> String {
    format!(
        "{{\"name\":\"{}\",\"delta_count\":{},\"compressed\":{},\"scale\":{}}}",
        md.name,
        md.deltas.len(),
        md.compressed,
        md.scale
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_morph_delta_v2() {
        let md = new_morph_delta_v2("smile");
        assert_eq!(md.name, "smile");
        assert_eq!(mdv2_delta_count(&md), 0);
    }

    #[test]
    fn test_add_delta() {
        let mut md = new_morph_delta_v2("blink");
        mdv2_add_delta(&mut md, 0, [0.1, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(mdv2_delta_count(&md), 1);
    }

    #[test]
    fn test_apply_positions() {
        let mut md = new_morph_delta_v2("test");
        mdv2_add_delta(&mut md, 0, [1.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        let mut pos = [[0.0_f32; 3]; 2];
        let mut nrm = [[0.0_f32; 3]; 2];
        mdv2_apply(&md, 0.5, &mut pos, &mut nrm);
        assert!((pos[0][0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_apply_normals() {
        let mut md = new_morph_delta_v2("test");
        mdv2_add_delta(&mut md, 0, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let mut pos = [[0.0_f32; 3]; 1];
        let mut nrm = [[0.0_f32; 3]; 1];
        mdv2_apply(&md, 1.0, &mut pos, &mut nrm);
        assert!((nrm[0][1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_compress() {
        let mut md = new_morph_delta_v2("test");
        mdv2_compress(&mut md);
        assert!(md.compressed);
    }

    #[test]
    fn test_magnitude_zero_on_empty() {
        let md = new_morph_delta_v2("empty");
        assert!((mdv2_magnitude(&md) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_magnitude_nonzero() {
        let mut md = new_morph_delta_v2("test");
        mdv2_add_delta(&mut md, 0, [3.0, 4.0, 0.0], [0.0; 3]);
        let mag = mdv2_magnitude(&md);
        assert!((mag - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_clear() {
        let mut md = new_morph_delta_v2("test");
        mdv2_add_delta(&mut md, 0, [1.0, 0.0, 0.0], [0.0; 3]);
        mdv2_clear(&mut md);
        assert_eq!(mdv2_delta_count(&md), 0);
    }

    #[test]
    fn test_to_json() {
        let md = new_morph_delta_v2("jaw");
        let json = mdv2_to_json(&md);
        assert!(json.contains("\"name\":\"jaw\""));
    }

    #[test]
    fn test_apply_out_of_bounds_safe() {
        let mut md = new_morph_delta_v2("test");
        mdv2_add_delta(&mut md, 100, [1.0, 0.0, 0.0], [0.0; 3]);
        let mut pos = [[0.0_f32; 3]; 1];
        let mut nrm = [[0.0_f32; 3]; 1];
        mdv2_apply(&md, 1.0, &mut pos, &mut nrm);
        assert!((pos[0][0] - 0.0).abs() < 1e-6);
    }
}
