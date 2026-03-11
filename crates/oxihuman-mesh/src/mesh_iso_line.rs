//! Isovalue line extraction from mesh scalar fields.
#![allow(dead_code)]

/// An isovalue line composed of segments.
#[allow(dead_code)]
pub struct IsoLine {
    pub segments: Vec<IsoSegment>,
    pub iso_value: f32,
}

/// A single segment of an iso line.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct IsoSegment {
    pub a: [f32; 3],
    pub b: [f32; 3],
}

/// Get the scalar value at a vertex.
#[allow(dead_code)]
pub fn iso_value_at_vertex(scalars: &[f32], index: usize) -> f32 {
    scalars.get(index).copied().unwrap_or(0.0)
}

/// Linearly interpolate a position between two vertices where the scalar crosses iso_value.
fn interp_edge(p0: [f32;3], p1: [f32;3], s0: f32, s1: f32, iso: f32) -> [f32;3] {
    let denom = s1 - s0;
    let t = if denom.abs() < 1e-10 { 0.5 } else { (iso - s0) / denom };
    let t = t.clamp(0.0, 1.0);
    [p0[0]+(p1[0]-p0[0])*t, p0[1]+(p1[1]-p0[1])*t, p0[2]+(p1[2]-p0[2])*t]
}

/// March over triangles extracting iso-contour segments for a given isovalue.
#[allow(dead_code)]
pub fn march_triangles_for_iso(
    positions: &[[f32;3]],
    scalars: &[f32],
    indices: &[u32],
    iso: f32,
) -> Vec<IsoSegment> {
    let n = positions.len();
    let tris = indices.len() / 3;
    let mut segments = Vec::new();
    for t in 0..tris {
        let i0 = indices[t*3] as usize;
        let i1 = indices[t*3+1] as usize;
        let i2 = indices[t*3+2] as usize;
        if i0 >= n || i1 >= n || i2 >= n { continue; }
        let s0 = scalars.get(i0).copied().unwrap_or(0.0);
        let s1 = scalars.get(i1).copied().unwrap_or(0.0);
        let s2 = scalars.get(i2).copied().unwrap_or(0.0);
        let above0 = s0 >= iso; let above1 = s1 >= iso; let above2 = s2 >= iso;
        let crossings: Vec<[f32;3]> = {
            let mut c = Vec::new();
            if above0 != above1 { c.push(interp_edge(positions[i0], positions[i1], s0, s1, iso)); }
            if above1 != above2 { c.push(interp_edge(positions[i1], positions[i2], s1, s2, iso)); }
            if above2 != above0 { c.push(interp_edge(positions[i2], positions[i0], s2, s0, iso)); }
            c
        };
        if crossings.len() >= 2 {
            segments.push(IsoSegment { a: crossings[0], b: crossings[1] });
        }
    }
    segments
}

/// Extract an iso line from a mesh scalar field.
#[allow(dead_code)]
pub fn extract_iso_line(
    positions: &[[f32;3]],
    scalars: &[f32],
    indices: &[u32],
    iso: f32,
) -> IsoLine {
    let segments = march_triangles_for_iso(positions, scalars, indices, iso);
    IsoLine { segments, iso_value: iso }
}

/// Compute the total length of an iso line.
#[allow(dead_code)]
pub fn iso_line_length(iso: &IsoLine) -> f32 {
    iso.segments.iter().map(|s| {
        let dx = s.b[0]-s.a[0]; let dy = s.b[1]-s.a[1]; let dz = s.b[2]-s.a[2];
        (dx*dx+dy*dy+dz*dz).sqrt()
    }).sum()
}

/// Convert iso line segments to a polyline (ordered chain, best effort).
#[allow(dead_code)]
pub fn iso_line_to_polyline(iso: &IsoLine) -> Vec<[f32;3]> {
    if iso.segments.is_empty() { return Vec::new(); }
    let mut pts = Vec::with_capacity(iso.segments.len() + 1);
    pts.push(iso.segments[0].a);
    for s in &iso.segments { pts.push(s.b); }
    pts
}

/// Estimate enclosed area of the iso contour (shoelace, XY plane projection).
#[allow(dead_code)]
pub fn iso_contour_area(iso: &IsoLine) -> f32 {
    let pts = iso_line_to_polyline(iso);
    if pts.len() < 3 { return 0.0; }
    let mut area = 0.0f32;
    let n = pts.len();
    for i in 0..n {
        let j = (i+1) % n;
        area += pts[i][0] * pts[j][1];
        area -= pts[j][0] * pts[i][1];
    }
    (area / 2.0).abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_tri_mesh() -> (Vec<[f32;3]>, Vec<u32>, Vec<f32>) {
        let pos = vec![[0.0f32,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0]];
        let idx = vec![0u32,1,2];
        let scalars = vec![0.0f32, 1.0, 0.5];
        (pos, idx, scalars)
    }

    #[test]
    fn test_iso_value_at_vertex() {
        let s = vec![1.0f32, 2.0, 3.0];
        assert!((iso_value_at_vertex(&s, 1) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_iso_value_oob() {
        let s = vec![1.0f32];
        assert!((iso_value_at_vertex(&s, 10)).abs() < 1e-5);
    }

    #[test]
    fn test_march_triangles_produces_segment() {
        let (pos, idx, scalars) = simple_tri_mesh();
        let segs = march_triangles_for_iso(&pos, &scalars, &idx, 0.5);
        assert!(!segs.is_empty());
    }

    #[test]
    fn test_extract_iso_line_iso_value() {
        let (pos, idx, scalars) = simple_tri_mesh();
        let iso = extract_iso_line(&pos, &scalars, &idx, 0.5);
        assert!((iso.iso_value - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_iso_line_length_positive() {
        let (pos, idx, scalars) = simple_tri_mesh();
        let iso = extract_iso_line(&pos, &scalars, &idx, 0.5);
        if !iso.segments.is_empty() {
            assert!(iso_line_length(&iso) > 0.0);
        }
    }

    #[test]
    fn test_iso_line_to_polyline_count() {
        let segs = vec![IsoSegment { a:[0.0,0.0,0.0], b:[1.0,0.0,0.0] }];
        let iso = IsoLine { segments: segs, iso_value: 0.5 };
        let poly = iso_line_to_polyline(&iso);
        assert_eq!(poly.len(), 2);
    }

    #[test]
    fn test_iso_contour_area_empty() {
        let iso = IsoLine { segments: vec![], iso_value: 0.0 };
        assert!((iso_contour_area(&iso)).abs() < 1e-5);
    }

    #[test]
    fn test_no_iso_above_max() {
        let (pos, idx, scalars) = simple_tri_mesh();
        let segs = march_triangles_for_iso(&pos, &scalars, &idx, 2.0);
        assert!(segs.is_empty());
    }

    #[test]
    fn test_iso_segment_struct() {
        let s = IsoSegment { a: [0.0,0.0,0.0], b: [1.0,0.0,0.0] };
        let dx = s.b[0] - s.a[0];
        assert!((dx - 1.0).abs() < 1e-5);
    }
}
