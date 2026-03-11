//! Mesh stitching at boundary loops.
#![allow(dead_code)]

/// A pair of boundary loops to stitch.
#[allow(dead_code)]
pub struct StitchPair {
    pub loop_a: Vec<usize>,
    pub loop_b: Vec<usize>,
}

/// Find boundary vertices (appear in exactly one triangle edge).
#[allow(dead_code)]
pub fn find_boundary_verts2(positions: &[[f32;3]], indices: &[u32]) -> Vec<usize> {
    let n = positions.len();
    let mut edge_count = std::collections::HashMap::new();
    let tris = indices.len() / 3;
    for t in 0..tris {
        let i0 = indices[t*3] as usize;
        let i1 = indices[t*3+1] as usize;
        let i2 = indices[t*3+2] as usize;
        for &(a,b) in &[(i0.min(i1), i0.max(i1)), (i1.min(i2), i1.max(i2)), (i0.min(i2), i0.max(i2))] {
            *edge_count.entry((a,b)).or_insert(0usize) += 1;
        }
    }
    let mut boundary = std::collections::BTreeSet::new();
    for ((a,b), c) in &edge_count {
        if *c == 1 { boundary.insert(*a); boundary.insert(*b); }
    }
    boundary.into_iter().filter(|&v| v < n).collect()
}

/// Match boundary vertices between two loops by nearest position.
#[allow(dead_code)]
pub fn match_boundary_verts(
    loop_a: &[usize],
    loop_b: &[usize],
    positions: &[[f32;3]],
) -> Vec<(usize, usize)> {
    loop_a.iter().map(|&a| {
        let pa = if a < positions.len() { positions[a] } else { [0.0;3] };
        let best_b = loop_b.iter().copied().min_by(|&x, &y| {
            let px = if x < positions.len() { positions[x] } else { [0.0;3] };
            let py = if y < positions.len() { positions[y] } else { [0.0;3] };
            let dx = pa[0]-px[0]; let dy = pa[1]-px[1]; let dz = pa[2]-px[2];
            let ex = pa[0]-py[0]; let ey = pa[1]-py[1]; let ez = pa[2]-py[2];
            let da = dx*dx+dy*dy+dz*dz;
            let db = ex*ex+ey*ey+ez*ez;
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        }).unwrap_or(0);
        (a, best_b)
    }).collect()
}

/// Stitch two boundary loops together, generating new triangle indices.
#[allow(dead_code)]
pub fn stitch_loop_pair(loop_a: &[usize], loop_b: &[usize]) -> Vec<u32> {
    let na = loop_a.len(); let nb = loop_b.len();
    if na == 0 || nb == 0 { return Vec::new(); }
    let mut indices = Vec::new();
    let n = na.min(nb);
    for i in 0..n {
        let j = (i + 1) % n;
        let a0 = loop_a[i] as u32; let a1 = loop_a[j] as u32;
        let b0 = loop_b[i % nb] as u32; let b1 = loop_b[j % nb] as u32;
        indices.extend_from_slice(&[a0, b0, a1]);
        indices.extend_from_slice(&[b0, b1, a1]);
    }
    indices
}

/// Stitch two loops with a tolerance (merge vertices closer than tol).
#[allow(dead_code)]
pub fn stitch_with_tolerance(
    loop_a: &[usize],
    loop_b: &[usize],
    positions: &[[f32;3]],
    tol: f32,
) -> Vec<u32> {
    let pairs = match_boundary_verts(loop_a, loop_b, positions);
    let na = loop_a.len();
    if na == 0 { return Vec::new(); }
    let mut indices = Vec::new();
    for i in 0..pairs.len() {
        let j = (i + 1) % pairs.len();
        let (a0, b0) = pairs[i]; let (a1, b1) = pairs[j];
        // check distance
        if a0 < positions.len() && b0 < positions.len() {
            let pa = positions[a0]; let pb = positions[b0];
            let dx = pa[0]-pb[0]; let dy = pa[1]-pb[1]; let dz = pa[2]-pb[2];
            let d = (dx*dx+dy*dy+dz*dz).sqrt();
            if d <= tol * 10.0 {
                indices.extend_from_slice(&[a0 as u32, b0 as u32, a1 as u32]);
                indices.extend_from_slice(&[b0 as u32, b1 as u32, a1 as u32]);
            }
        }
    }
    indices
}

/// Stitch boundary loops from two meshes at their shared boundary.
#[allow(dead_code)]
pub fn stitch_boundary_loops2(
    pos_a: &[[f32;3]],
    idx_a: &[u32],
    pos_b: &[[f32;3]],
    idx_b: &[u32],
    offset: usize,
) -> Vec<u32> {
    let bv_a = find_boundary_verts2(pos_a, idx_a);
    let _bv_b: Vec<usize> = find_boundary_verts2(pos_b, idx_b).iter().map(|&v| v + offset).collect();
    let bv_b_local = find_boundary_verts2(pos_b, idx_b);
    stitch_loop_pair(&bv_a, &bv_b_local.iter().map(|&v| v + offset).collect::<Vec<_>>())
}

/// Compute total arc length of a boundary loop.
#[allow(dead_code)]
pub fn boundary_loop_length2(loop_verts: &[usize], positions: &[[f32;3]]) -> f32 {
    let n = loop_verts.len();
    if n < 2 { return 0.0; }
    (0..n).map(|i| {
        let j = (i + 1) % n;
        let a = loop_verts[i]; let b = loop_verts[j];
        if a >= positions.len() || b >= positions.len() { return 0.0; }
        let pa = positions[a]; let pb = positions[b];
        let dx = pb[0]-pa[0]; let dy = pb[1]-pa[1]; let dz = pb[2]-pa[2];
        (dx*dx+dy*dy+dz*dz).sqrt()
    }).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ring_mesh() -> (Vec<[f32;3]>, Vec<u32>) {
        let pos = vec![[0.0f32,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0],[1.0,1.0,0.0]];
        let idx = vec![0u32,1,2, 1,3,2];
        (pos, idx)
    }

    #[test]
    fn test_find_boundary_verts2() {
        let (pos, idx) = ring_mesh();
        let bv = find_boundary_verts2(&pos, &idx);
        assert!(!bv.is_empty());
    }

    #[test]
    fn test_match_boundary_verts() {
        let pos = vec![[0.0f32,0.0,0.0],[1.0,0.0,0.0],[0.5,0.5,0.0]];
        let a = vec![0usize, 1];
        let b = vec![0usize, 2];
        let pairs = match_boundary_verts(&a, &b, &pos);
        assert_eq!(pairs.len(), 2);
    }

    #[test]
    fn test_stitch_loop_pair_count() {
        let la = vec![0usize,1,2];
        let lb = vec![3usize,4,5];
        let idx = stitch_loop_pair(&la, &lb);
        assert!(!idx.is_empty());
    }

    #[test]
    fn test_stitch_with_tolerance_empty_loops() {
        let idx = stitch_with_tolerance(&[], &[], &[], 0.01);
        assert!(idx.is_empty());
    }

    #[test]
    fn test_boundary_loop_length2() {
        let pos = vec![[0.0f32,0.0,0.0],[1.0,0.0,0.0],[1.0,1.0,0.0],[0.0,1.0,0.0]];
        let loop_v = vec![0usize,1,2,3];
        let l = boundary_loop_length2(&loop_v, &pos);
        assert!((l - 4.0).abs() < 1e-4);
    }

    #[test]
    fn test_stitch_pair_struct() {
        let sp = StitchPair { loop_a: vec![0,1,2], loop_b: vec![3,4,5] };
        assert_eq!(sp.loop_a.len(), 3);
    }

    #[test]
    fn test_stitch_loop_pair_empty() {
        let idx = stitch_loop_pair(&[], &[1,2,3]);
        assert!(idx.is_empty());
    }

    #[test]
    fn test_boundary_loop_length2_single() {
        let pos = vec![[0.0f32,0.0,0.0]];
        let l = boundary_loop_length2(&[0usize], &pos);
        assert!((l).abs() < 1e-5);
    }

    #[test]
    fn test_stitch_boundary_loops2() {
        let (pos_a, idx_a) = ring_mesh();
        let (pos_b, idx_b) = ring_mesh();
        let result = stitch_boundary_loops2(&pos_a, &idx_a, &pos_b, &idx_b, 4);
        assert!(result.len().is_multiple_of(3));
    }
}
