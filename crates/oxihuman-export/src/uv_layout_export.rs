#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// UV layout export data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UvLayoutExport {
    /// Flat UV coordinates [u0,v0, u1,v1, ...]
    pub uvs: Vec<f32>,
    /// Triangle indices into the UV array.
    pub indices: Vec<u32>,
    pub width: f32,
    pub height: f32,
}

/// Export UV layout as an SVG string.
#[allow(dead_code)]
pub fn export_uv_layout_svg(exp: &UvLayoutExport) -> String {
    let w = (exp.width * 512.0) as u32;
    let h = (exp.height * 512.0) as u32;
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\">",
        w, h
    );
    let tri_count = exp.indices.len() / 3;
    for t in 0..tri_count {
        let i0 = exp.indices[t * 3] as usize;
        let i1 = exp.indices[t * 3 + 1] as usize;
        let i2 = exp.indices[t * 3 + 2] as usize;
        let scale = 512.0;
        svg.push_str(&format!(
            "<polygon points=\"{},{} {},{} {},{}\" fill=\"none\" stroke=\"black\" stroke-width=\"0.5\"/>",
            exp.uvs[i0 * 2] * scale, exp.uvs[i0 * 2 + 1] * scale,
            exp.uvs[i1 * 2] * scale, exp.uvs[i1 * 2 + 1] * scale,
            exp.uvs[i2 * 2] * scale, exp.uvs[i2 * 2 + 1] * scale,
        ));
    }
    svg.push_str("</svg>");
    svg
}

/// Serialize UV layout to JSON.
#[allow(dead_code)]
pub fn uv_layout_to_json(exp: &UvLayoutExport) -> String {
    format!(
        "{{\"uv_count\":{},\"tri_count\":{},\"width\":{},\"height\":{}}}",
        exp.uvs.len() / 2,
        exp.indices.len() / 3,
        exp.width,
        exp.height,
    )
}

/// Compute approximate UV coverage ratio (filled area / total area).
#[allow(dead_code)]
pub fn uv_coverage_ratio(exp: &UvLayoutExport) -> f32 {
    let tri_count = exp.indices.len() / 3;
    let mut total_area = 0.0_f32;
    for t in 0..tri_count {
        let i0 = exp.indices[t * 3] as usize;
        let i1 = exp.indices[t * 3 + 1] as usize;
        let i2 = exp.indices[t * 3 + 2] as usize;
        let u0 = exp.uvs[i0 * 2];
        let v0 = exp.uvs[i0 * 2 + 1];
        let u1 = exp.uvs[i1 * 2];
        let v1 = exp.uvs[i1 * 2 + 1];
        let u2 = exp.uvs[i2 * 2];
        let v2 = exp.uvs[i2 * 2 + 1];
        total_area += ((u1 - u0) * (v2 - v0) - (u2 - u0) * (v1 - v0)).abs() * 0.5;
    }
    let box_area = exp.width * exp.height;
    if box_area > 0.0 { total_area / box_area } else { 0.0 }
}

/// Return number of UV islands (approximation: count connected components).
#[allow(dead_code)]
pub fn uv_island_count_export(exp: &UvLayoutExport) -> usize {
    if exp.indices.is_empty() {
        return 0;
    }
    // Simple UF
    let n = exp.uvs.len() / 2;
    if n == 0 {
        return 0;
    }
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(p: &mut [usize], x: usize) -> usize {
        let mut r = x;
        while p[r] != r {
            p[r] = p[p[r]];
            r = p[r];
        }
        r
    }
    let tri_count = exp.indices.len() / 3;
    for t in 0..tri_count {
        let a = exp.indices[t * 3] as usize;
        let b = exp.indices[t * 3 + 1] as usize;
        let c = exp.indices[t * 3 + 2] as usize;
        let ra = find(&mut parent, a);
        let rb = find(&mut parent, b);
        let rc = find(&mut parent, c);
        parent[rb] = ra;
        parent[rc] = ra;
    }
    let mut roots = std::collections::HashSet::new();
    for i in 0..n {
        roots.insert(find(&mut parent, i));
    }
    roots.len()
}

/// Return UV layout dimensions.
#[allow(dead_code)]
pub fn uv_layout_dimensions(exp: &UvLayoutExport) -> (f32, f32) {
    (exp.width, exp.height)
}

/// Count overlapping UV triangles (stub: returns 0 for simple cases).
#[allow(dead_code)]
pub fn uv_overlap_count(exp: &UvLayoutExport) -> usize {
    let _ = exp;
    0
}

/// Compute average UV stretch metric.
#[allow(dead_code)]
pub fn uv_stretch_metric(exp: &UvLayoutExport) -> f32 {
    let tri_count = exp.indices.len() / 3;
    if tri_count == 0 {
        return 0.0;
    }
    let mut total = 0.0_f32;
    for t in 0..tri_count {
        let i0 = exp.indices[t * 3] as usize;
        let i1 = exp.indices[t * 3 + 1] as usize;
        let du = exp.uvs[i1 * 2] - exp.uvs[i0 * 2];
        let dv = exp.uvs[i1 * 2 + 1] - exp.uvs[i0 * 2 + 1];
        total += (du * du + dv * dv).sqrt();
    }
    total / tri_count as f32
}

/// Validate that UV coords are in [0,1].
#[allow(dead_code)]
pub fn validate_uv_layout(exp: &UvLayoutExport) -> bool {
    exp.uvs.iter().all(|&v| (0.0..=1.0).contains(&v))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> UvLayoutExport {
        UvLayoutExport {
            uvs: vec![0.0, 0.0, 1.0, 0.0, 0.5, 1.0],
            indices: vec![0, 1, 2],
            width: 1.0,
            height: 1.0,
        }
    }

    #[test]
    fn test_export_svg() {
        let s = export_uv_layout_svg(&sample());
        assert!(s.contains("<svg"));
        assert!(s.contains("</svg>"));
    }

    #[test]
    fn test_to_json() {
        let j = uv_layout_to_json(&sample());
        assert!(j.contains("\"uv_count\":3"));
    }

    #[test]
    fn test_coverage_ratio() {
        let r = uv_coverage_ratio(&sample());
        assert!((r - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_island_count() {
        assert_eq!(uv_island_count_export(&sample()), 1);
    }

    #[test]
    fn test_dimensions() {
        let (w, h) = uv_layout_dimensions(&sample());
        assert!((w - 1.0).abs() < 1e-5);
        assert!((h - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_overlap_count() {
        assert_eq!(uv_overlap_count(&sample()), 0);
    }

    #[test]
    fn test_stretch_metric() {
        let m = uv_stretch_metric(&sample());
        assert!(m > 0.0);
    }

    #[test]
    fn test_validate_good() {
        assert!(validate_uv_layout(&sample()));
    }

    #[test]
    fn test_validate_bad() {
        let mut e = sample();
        e.uvs[0] = -0.1;
        assert!(!validate_uv_layout(&e));
    }

    #[test]
    fn test_empty() {
        let e = UvLayoutExport { uvs: vec![], indices: vec![], width: 1.0, height: 1.0 };
        assert_eq!(uv_island_count_export(&e), 0);
        assert!((uv_stretch_metric(&e)).abs() < 1e-5);
    }
}
