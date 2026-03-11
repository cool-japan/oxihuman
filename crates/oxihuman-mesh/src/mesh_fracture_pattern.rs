//! Generate procedural Voronoi fracture cell patterns on a mesh surface.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FracturePatternConfig {
    pub seed_count: usize,
    pub domain_min: [f32; 2],
    pub domain_max: [f32; 2],
    pub lcg_seed: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FractureCell {
    pub index: usize,
    pub centroid: [f32; 2],
    pub vertices: Vec<[f32; 2]>,
    pub area: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FracturePatternResult {
    pub cells: Vec<FractureCell>,
    pub seed_points: Vec<[f32; 2]>,
    pub valid: bool,
}

fn lcg_next(state: &mut u64) -> f32 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    let bits = ((*state >> 33) as u32) as f64;
    (bits / (u32::MAX as f64)) as f32
}

fn lcg_in_range(state: &mut u64, lo: f32, hi: f32) -> f32 {
    lo + lcg_next(state) * (hi - lo)
}

fn polygon_area(verts: &[[f32; 2]]) -> f32 {
    if verts.len() < 3 {
        return 0.0;
    }
    let mut area = 0.0f32;
    let n = verts.len();
    for i in 0..n {
        let j = (i + 1) % n;
        area += verts[i][0] * verts[j][1];
        area -= verts[j][0] * verts[i][1];
    }
    (area * 0.5).abs()
}

fn polygon_centroid(verts: &[[f32; 2]]) -> [f32; 2] {
    if verts.is_empty() {
        return [0.0; 2];
    }
    let sum: [f32; 2] = verts.iter().fold([0.0; 2], |acc, v| [acc[0] + v[0], acc[1] + v[1]]);
    let n = verts.len() as f32;
    [sum[0] / n, sum[1] / n]
}

/// Build approximate Voronoi cells by sampling a grid and assigning each sample
/// to its nearest seed, then collecting per-cell sample positions as pseudo-vertices.
fn build_voronoi_cells(seeds: &[[f32; 2]], config: &FracturePatternConfig) -> Vec<FractureCell> {
    let grid = 32usize;
    let dx = (config.domain_max[0] - config.domain_min[0]) / grid as f32;
    let dy = (config.domain_max[1] - config.domain_min[1]) / grid as f32;

    let mut cell_verts: Vec<Vec<[f32; 2]>> = vec![Vec::new(); seeds.len()];

    for gy in 0..grid {
        for gx in 0..grid {
            let px = config.domain_min[0] + (gx as f32 + 0.5) * dx;
            let py = config.domain_min[1] + (gy as f32 + 0.5) * dy;

            let mut best = 0usize;
            let mut best_dist = f32::MAX;
            for (idx, s) in seeds.iter().enumerate() {
                let d = (px - s[0]).powi(2) + (py - s[1]).powi(2);
                if d < best_dist {
                    best_dist = d;
                    best = idx;
                }
            }
            cell_verts[best].push([px, py]);
        }
    }

    cell_verts
        .into_iter()
        .enumerate()
        .map(|(i, verts)| {
            let centroid = polygon_centroid(&verts);
            let area = polygon_area(&verts);
            FractureCell { index: i, centroid, vertices: verts, area }
        })
        .collect()
}

#[allow(dead_code)]
pub fn default_fracture_pattern_config() -> FracturePatternConfig {
    FracturePatternConfig {
        seed_count: 8,
        domain_min: [0.0, 0.0],
        domain_max: [1.0, 1.0],
        lcg_seed: 42,
    }
}

#[allow(dead_code)]
pub fn generate_fracture_pattern(config: &FracturePatternConfig) -> FracturePatternResult {
    let mut state = config.lcg_seed;
    let seeds: Vec<[f32; 2]> = (0..config.seed_count)
        .map(|_| {
            [
                lcg_in_range(&mut state, config.domain_min[0], config.domain_max[0]),
                lcg_in_range(&mut state, config.domain_min[1], config.domain_max[1]),
            ]
        })
        .collect();

    let cells = build_voronoi_cells(&seeds, config);
    let valid = fracture_pattern_validate_cells(&cells);

    FracturePatternResult { cells, seed_points: seeds, valid }
}

fn fracture_pattern_validate_cells(cells: &[FractureCell]) -> bool {
    !cells.is_empty() && cells.iter().all(|c| c.area >= 0.0)
}

#[allow(dead_code)]
pub fn fracture_cell_count(result: &FracturePatternResult) -> usize {
    result.cells.len()
}

#[allow(dead_code)]
pub fn fracture_cell_centroid(result: &FracturePatternResult, index: usize) -> Option<[f32; 2]> {
    result.cells.get(index).map(|c| c.centroid)
}

#[allow(dead_code)]
pub fn fracture_cell_vertices(result: &FracturePatternResult, index: usize) -> &[[f32; 2]] {
    result.cells.get(index).map(|c| c.vertices.as_slice()).unwrap_or(&[])
}

#[allow(dead_code)]
pub fn fracture_pattern_to_json(result: &FracturePatternResult) -> String {
    format!(
        "{{\"cell_count\":{},\"seed_count\":{},\"valid\":{}}}",
        result.cells.len(),
        result.seed_points.len(),
        result.valid
    )
}

#[allow(dead_code)]
pub fn fracture_seed_points(result: &FracturePatternResult) -> &[[f32; 2]] {
    &result.seed_points
}

#[allow(dead_code)]
pub fn fracture_pattern_validate(result: &FracturePatternResult) -> bool {
    result.valid
}

#[allow(dead_code)]
pub fn fracture_cell_area(result: &FracturePatternResult, index: usize) -> f32 {
    result.cells.get(index).map(|c| c.area).unwrap_or(0.0)
}

#[allow(dead_code)]
pub fn fracture_pattern_clear(result: &mut FracturePatternResult) {
    result.cells.clear();
    result.seed_points.clear();
    result.valid = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_fracture_pattern_config();
        assert_eq!(cfg.seed_count, 8);
        assert_eq!(cfg.lcg_seed, 42);
    }

    #[test]
    fn test_generate_cell_count() {
        let cfg = default_fracture_pattern_config();
        let res = generate_fracture_pattern(&cfg);
        assert_eq!(fracture_cell_count(&res), cfg.seed_count);
    }

    #[test]
    fn test_seed_points_count() {
        let cfg = default_fracture_pattern_config();
        let res = generate_fracture_pattern(&cfg);
        assert_eq!(fracture_seed_points(&res).len(), cfg.seed_count);
    }

    #[test]
    fn test_validate() {
        let cfg = default_fracture_pattern_config();
        let res = generate_fracture_pattern(&cfg);
        assert!(fracture_pattern_validate(&res));
    }

    #[test]
    fn test_centroid_in_domain() {
        let cfg = default_fracture_pattern_config();
        let res = generate_fracture_pattern(&cfg);
        for i in 0..fracture_cell_count(&res) {
            if let Some(c) = fracture_cell_centroid(&res, i) {
                assert!(c[0] >= cfg.domain_min[0] && c[0] <= cfg.domain_max[0]);
                assert!(c[1] >= cfg.domain_min[1] && c[1] <= cfg.domain_max[1]);
            }
        }
    }

    #[test]
    fn test_cell_area_nonneg() {
        let cfg = default_fracture_pattern_config();
        let res = generate_fracture_pattern(&cfg);
        for i in 0..fracture_cell_count(&res) {
            assert!(fracture_cell_area(&res, i) >= 0.0);
        }
    }

    #[test]
    fn test_to_json_contains_fields() {
        let cfg = default_fracture_pattern_config();
        let res = generate_fracture_pattern(&cfg);
        let json = fracture_pattern_to_json(&res);
        assert!(json.contains("cell_count"));
        assert!(json.contains("seed_count"));
        assert!(json.contains("valid"));
    }

    #[test]
    fn test_clear() {
        let cfg = default_fracture_pattern_config();
        let mut res = generate_fracture_pattern(&cfg);
        fracture_pattern_clear(&mut res);
        assert_eq!(fracture_cell_count(&res), 0);
        assert!(!fracture_pattern_validate(&res));
    }

    #[test]
    fn test_deterministic() {
        let cfg = default_fracture_pattern_config();
        let r1 = generate_fracture_pattern(&cfg);
        let r2 = generate_fracture_pattern(&cfg);
        assert_eq!(r1.seed_points.len(), r2.seed_points.len());
        for (a, b) in r1.seed_points.iter().zip(r2.seed_points.iter()) {
            assert!((a[0] - b[0]).abs() < 1e-6);
            assert!((a[1] - b[1]).abs() < 1e-6);
        }
    }

    #[test]
    fn test_vertices_non_empty() {
        let cfg = default_fracture_pattern_config();
        let res = generate_fracture_pattern(&cfg);
        let total_verts: usize = (0..fracture_cell_count(&res))
            .map(|i| fracture_cell_vertices(&res, i).len())
            .sum();
        assert!(total_verts > 0);
    }
}
