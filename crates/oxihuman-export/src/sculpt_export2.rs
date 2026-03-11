//! Sculpt layer export.
#![allow(dead_code)]

/// A single sculpt layer.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct SculptLayer2 {
    pub name: String,
    pub deltas: Vec<[f32; 3]>,
    pub intensity: f32,
}

/// Sculpt export containing multiple layers.
#[allow(dead_code)]
pub struct SculptExport2 {
    pub layers: Vec<SculptLayer2>,
}

/// Create a new sculpt export.
#[allow(dead_code)]
pub fn new_sculpt_export2() -> SculptExport2 {
    SculptExport2 { layers: Vec::new() }
}

/// Add a sculpt layer.
#[allow(dead_code)]
pub fn add_sculpt_layer2(se: &mut SculptExport2, layer: SculptLayer2) {
    se.layers.push(layer);
}

/// Export sculpt layers to JSON.
#[allow(dead_code)]
pub fn export_sculpt_layers2(se: &SculptExport2) -> String {
    let layers: Vec<String> = se.layers.iter().map(|l| {
        format!(r#"{{"name":"{}","intensity":{}}}"#, l.name, l.intensity)
    }).collect();
    format!("[{}]", layers.join(","))
}

/// Get sculpt layer count.
#[allow(dead_code)]
pub fn sculpt2_layer_count(se: &SculptExport2) -> usize { se.layers.len() }

/// Get delta at vertex `v` in layer `l`.
#[allow(dead_code)]
pub fn sculpt2_delta_at(se: &SculptExport2, l: usize, v: usize) -> [f32; 3] {
    se.layers.get(l).and_then(|layer| layer.deltas.get(v)).copied().unwrap_or([0.0;3])
}

/// Get layer name.
#[allow(dead_code)]
pub fn sculpt2_layer_name(se: &SculptExport2, l: usize) -> &str {
    se.layers.get(l).map(|x| x.name.as_str()).unwrap_or("")
}

/// Convert sculpt to JSON.
#[allow(dead_code)]
pub fn sculpt2_to_json(se: &SculptExport2) -> String { export_sculpt_layers2(se) }

/// Get the RMS magnitude of a sculpt layer's deltas.
#[allow(dead_code)]
pub fn sculpt2_layer_magnitude(layer: &SculptLayer2) -> f32 {
    if layer.deltas.is_empty() { return 0.0; }
    let sum: f32 = layer.deltas.iter().map(|d| d[0]*d[0]+d[1]*d[1]+d[2]*d[2]).sum();
    (sum / layer.deltas.len() as f32).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_layer(name: &str) -> SculptLayer2 {
        SculptLayer2 { name: name.to_string(), deltas: vec![[0.1,0.0,0.0],[0.0,0.2,0.0]], intensity: 1.0 }
    }

    #[test]
    fn test_new_sculpt_export_empty() {
        let se = new_sculpt_export2();
        assert_eq!(sculpt2_layer_count(&se), 0);
    }

    #[test]
    fn test_add_layer() {
        let mut se = new_sculpt_export2();
        add_sculpt_layer2(&mut se, make_layer("detail"));
        assert_eq!(sculpt2_layer_count(&se), 1);
    }

    #[test]
    fn test_sculpt_layer_name() {
        let mut se = new_sculpt_export2();
        add_sculpt_layer2(&mut se, make_layer("wrinkle"));
        assert_eq!(sculpt2_layer_name(&se, 0), "wrinkle");
    }

    #[test]
    fn test_sculpt_delta_at() {
        let mut se = new_sculpt_export2();
        add_sculpt_layer2(&mut se, make_layer("bump"));
        let d = sculpt2_delta_at(&se, 0, 0);
        assert!((d[0] - 0.1).abs() < 1e-5);
    }

    #[test]
    fn test_export_sculpt_layers() {
        let mut se = new_sculpt_export2();
        add_sculpt_layer2(&mut se, make_layer("x"));
        let s = export_sculpt_layers2(&se);
        assert!(s.contains("name"));
    }

    #[test]
    fn test_sculpt_layer_magnitude() {
        let l = make_layer("m");
        let mag = sculpt2_layer_magnitude(&l);
        assert!(mag > 0.0);
    }

    #[test]
    fn test_sculpt_magnitude_empty() {
        let l = SculptLayer2 { name:"e".to_string(), deltas:vec![], intensity:1.0 };
        assert!((sculpt2_layer_magnitude(&l)).abs() < 1e-5);
    }

    #[test]
    fn test_sculpt_delta_oob() {
        let mut se = new_sculpt_export2();
        add_sculpt_layer2(&mut se, make_layer("a"));
        let d = sculpt2_delta_at(&se, 0, 100);
        assert!((d[0]).abs() < 1e-5);
    }

    #[test]
    fn test_sculpt_layer_name_oob() {
        let se = new_sculpt_export2();
        assert_eq!(sculpt2_layer_name(&se, 0), "");
    }
}
