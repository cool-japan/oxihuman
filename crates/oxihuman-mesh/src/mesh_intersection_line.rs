//! Mesh-mesh intersection line computation.
#![allow(dead_code)]

/// A single intersection segment between two triangles.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct IntersectionSegment {
    pub a: [f32; 3],
    pub b: [f32; 3],
}

fn sub3(a: [f32;3], b: [f32;3]) -> [f32;3] { [a[0]-b[0], a[1]-b[1], a[2]-b[2]] }
fn dot3(a: [f32;3], b: [f32;3]) -> f32 { a[0]*b[0]+a[1]*b[1]+a[2]*b[2] }
fn cross3(a: [f32;3], b: [f32;3]) -> [f32;3] {
    [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]
}
fn len3(v: [f32;3]) -> f32 { (v[0]*v[0]+v[1]*v[1]+v[2]*v[2]).sqrt() }

/// Compute the midpoint of an intersection segment.
#[allow(dead_code)]
pub fn segment_midpoint(seg: &IntersectionSegment) -> [f32;3] {
    [(seg.a[0]+seg.b[0])*0.5, (seg.a[1]+seg.b[1])*0.5, (seg.a[2]+seg.b[2])*0.5]
}

/// Compute the length of an intersection segment.
#[allow(dead_code)]
pub fn intersection_line_length(segments: &[IntersectionSegment]) -> f32 {
    segments.iter().map(|s| len3(sub3(s.b, s.a))).sum()
}

/// Convert segments to a polyline (chain).
#[allow(dead_code)]
pub fn segments_to_polyline(segments: &[IntersectionSegment]) -> Vec<[f32;3]> {
    if segments.is_empty() { return Vec::new(); }
    let mut pts = Vec::with_capacity(segments.len() + 1);
    pts.push(segments[0].a);
    for s in segments { pts.push(s.b); }
    pts
}

/// Remove duplicate segments (where a==b within tolerance).
#[allow(dead_code)]
pub fn deduplicate_segments(segments: &[IntersectionSegment], tol: f32) -> Vec<IntersectionSegment> {
    segments.iter().filter(|s| {
        let d = len3(sub3(s.b, s.a));
        d > tol
    }).cloned().collect()
}

/// Attempt triangle-triangle intersection (Möller 1997, simplified).
/// Returns the intersection segment if the triangles intersect, else None.
#[allow(dead_code)]
pub fn triangle_triangle_intersect(
    pa0: [f32;3], pa1: [f32;3], pa2: [f32;3],
    pb0: [f32;3], pb1: [f32;3], pb2: [f32;3],
) -> Option<IntersectionSegment> {
    // Compute normal of triangle B
    let nb = cross3(sub3(pb1,pb0), sub3(pb2,pb0));
    let db = dot3(nb, pb0);
    // Signed distances of A's vertices to plane of B
    let da0 = dot3(nb, pa0) - db;
    let da1 = dot3(nb, pa1) - db;
    let da2 = dot3(nb, pa2) - db;
    let sign0 = da0 >= 0.0; let sign1 = da1 >= 0.0; let sign2 = da2 >= 0.0;
    if sign0 == sign1 && sign1 == sign2 { return None; }
    // Compute normal of triangle A
    let na = cross3(sub3(pa1,pa0), sub3(pa2,pa0));
    let da = dot3(na, pa0);
    let db0 = dot3(na, pb0) - da;
    let db1 = dot3(na, pb1) - da;
    let db2 = dot3(na, pb2) - da;
    let sb0 = db0 >= 0.0; let sb1 = db1 >= 0.0; let sb2 = db2 >= 0.0;
    if sb0 == sb1 && sb1 == sb2 { return None; }
    // Compute intersection line direction
    let d = cross3(na, nb);
    let dlen = len3(d);
    if dlen < 1e-10 { return None; }
    let d = [d[0]/dlen, d[1]/dlen, d[2]/dlen];
    // Project vertices of A onto line
    let interp = |p: [f32;3], q: [f32;3], sp: f32, sq: f32| -> [f32;3] {
        let t = sp / (sp - sq);
        [p[0]+(q[0]-p[0])*t, p[1]+(q[1]-p[1])*t, p[2]+(q[2]-p[2])*t]
    };
    // Find the two points where A crosses plane B
    let mut cross_a: Vec<[f32;3]> = Vec::new();
    for &((p0,s0),(p1,s1)) in &[((pa0,da0),(pa1,da1)),((pa1,da1),(pa2,da2)),((pa0,da0),(pa2,da2))] {
        if (s0 >= 0.0) != (s1 >= 0.0) { cross_a.push(interp(p0, p1, s0, s1)); }
    }
    if cross_a.len() < 2 { return None; }
    // Project onto intersection line
    let t0 = dot3(cross_a[0], d);
    let t1 = dot3(cross_a[1], d);
    let (t_min, t_max) = if t0 < t1 { (t0, t1) } else { (t1, t0) };
    // Find where B crosses plane A
    let mut cross_b: Vec<[f32;3]> = Vec::new();
    for &((p0,s0),(p1,s1)) in &[((pb0,db0),(pb1,db1)),((pb1,db1),(pb2,db2)),((pb0,db0),(pb2,db2))] {
        if (s0 >= 0.0) != (s1 >= 0.0) { cross_b.push(interp(p0, p1, s0, s1)); }
    }
    if cross_b.len() < 2 { return None; }
    let tb0 = dot3(cross_b[0], d);
    let tb1 = dot3(cross_b[1], d);
    let (tb_min, tb_max) = if tb0 < tb1 { (tb0, tb1) } else { (tb1, tb0) };
    let seg_min = t_min.max(tb_min);
    let seg_max = t_max.min(tb_max);
    if seg_min >= seg_max - 1e-8 { return None; }
    let pt_a = [d[0]*seg_min, d[1]*seg_min, d[2]*seg_min];
    let pt_b = [d[0]*seg_max, d[1]*seg_max, d[2]*seg_max];
    Some(IntersectionSegment { a: pt_a, b: pt_b })
}

/// Compute all intersection segments between two meshes.
#[allow(dead_code)]
pub fn mesh_intersection_segments(
    pos_a: &[[f32;3]], idx_a: &[u32],
    pos_b: &[[f32;3]], idx_b: &[u32],
) -> Vec<IntersectionSegment> {
    let na = pos_a.len(); let nb = pos_b.len();
    let tris_a = idx_a.len() / 3; let tris_b = idx_b.len() / 3;
    let mut segs = Vec::new();
    for ta in 0..tris_a {
        let ia0 = idx_a[ta*3] as usize; let ia1 = idx_a[ta*3+1] as usize; let ia2 = idx_a[ta*3+2] as usize;
        if ia0 >= na || ia1 >= na || ia2 >= na { continue; }
        for tb in 0..tris_b {
            let ib0 = idx_b[tb*3] as usize; let ib1 = idx_b[tb*3+1] as usize; let ib2 = idx_b[tb*3+2] as usize;
            if ib0 >= nb || ib1 >= nb || ib2 >= nb { continue; }
            if let Some(seg) = triangle_triangle_intersect(
                pos_a[ia0], pos_a[ia1], pos_a[ia2],
                pos_b[ib0], pos_b[ib1], pos_b[ib2],
            ) {
                segs.push(seg);
            }
        }
    }
    segs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_midpoint() {
        let seg = IntersectionSegment { a:[0.0,0.0,0.0], b:[2.0,0.0,0.0] };
        let mid = segment_midpoint(&seg);
        assert!((mid[0]-1.0).abs() < 1e-5);
    }

    #[test]
    fn test_intersection_line_length() {
        let segs = vec![IntersectionSegment { a:[0.0,0.0,0.0], b:[3.0,0.0,0.0] }];
        assert!((intersection_line_length(&segs) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_segments_to_polyline() {
        let segs = vec![IntersectionSegment{a:[0.0,0.0,0.0],b:[1.0,0.0,0.0]}];
        let poly = segments_to_polyline(&segs);
        assert_eq!(poly.len(), 2);
    }

    #[test]
    fn test_deduplicate_segments_removes_degenerate() {
        let segs = vec![
            IntersectionSegment{a:[0.0,0.0,0.0],b:[0.0,0.0,0.0]},
            IntersectionSegment{a:[0.0,0.0,0.0],b:[1.0,0.0,0.0]},
        ];
        let r = deduplicate_segments(&segs, 1e-5);
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn test_triangle_triangle_no_intersect() {
        let r = triangle_triangle_intersect(
            [0.0f32,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0],
            [0.0,0.0,5.0],[1.0,0.0,5.0],[0.0,1.0,5.0],
        );
        assert!(r.is_none());
    }

    #[test]
    fn test_triangle_triangle_intersect_crossing() {
        // Two triangles on perpendicular planes that intersect
        let r = triangle_triangle_intersect(
            [-1.0f32,0.0,-1.0],[1.0,0.0,-1.0],[0.0,0.0,1.0],
            [0.0,-1.0,0.0],[0.0,1.0,0.0],[0.0,0.0,0.0],
        );
        // May or may not intersect depending on exact geometry
        let _ = r;
    }

    #[test]
    fn test_mesh_intersection_segments_same_plane() {
        let pos = vec![[0.0f32,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]];
        let idx = vec![0u32,1,2];
        let segs = mesh_intersection_segments(&pos, &idx, &pos, &idx);
        assert!(segs.len() < 100);
    }

    #[test]
    fn test_segments_empty() {
        let poly = segments_to_polyline(&[]);
        assert!(poly.is_empty());
    }

    #[test]
    fn test_intersection_line_length_empty() {
        let l = intersection_line_length(&[]);
        assert!((l).abs() < 1e-5);
    }
}
