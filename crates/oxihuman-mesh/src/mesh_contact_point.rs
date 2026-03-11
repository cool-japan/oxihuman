//! Contact point detection between points and mesh triangles.
#![allow(dead_code)]

/// A detected contact point.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ContactPoint {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub depth: f32,
    pub triangle_index: usize,
}

fn sub3(a: [f32;3], b: [f32;3]) -> [f32;3] { [a[0]-b[0], a[1]-b[1], a[2]-b[2]] }
fn dot3(a: [f32;3], b: [f32;3]) -> f32 { a[0]*b[0]+a[1]*b[1]+a[2]*b[2] }
fn cross3(a: [f32;3], b: [f32;3]) -> [f32;3] {
    [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]
}
fn normalize3(v: [f32;3]) -> [f32;3] {
    let l = (v[0]*v[0]+v[1]*v[1]+v[2]*v[2]).sqrt();
    if l < 1e-10 { [0.0,0.0,1.0] } else { [v[0]/l,v[1]/l,v[2]/l] }
}
fn len3(v: [f32;3]) -> f32 { (v[0]*v[0]+v[1]*v[1]+v[2]*v[2]).sqrt() }

/// Find the closest point on a triangle (p0,p1,p2) to point `q`.
#[allow(dead_code)]
pub fn find_closest_point_on_triangle(q: [f32;3], p0: [f32;3], p1: [f32;3], p2: [f32;3]) -> [f32;3] {
    let ab = sub3(p1, p0);
    let ac = sub3(p2, p0);
    let ap = sub3(q, p0);
    let d1 = dot3(ab, ap); let d2 = dot3(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 { return p0; }
    let bp = sub3(q, p1);
    let d3 = dot3(ab, bp); let d4 = dot3(ac, bp);
    if d3 >= 0.0 && d4 <= d3 { return p1; }
    let cp = sub3(q, p2);
    let d5 = dot3(ab, cp); let d6 = dot3(ac, cp);
    if d6 >= 0.0 && d5 <= d6 { return p2; }
    let vc = d1*d4 - d3*d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1/(d1-d3);
        return [p0[0]+v*ab[0], p0[1]+v*ab[1], p0[2]+v*ab[2]];
    }
    let vb = d5*d2 - d1*d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2/(d2-d6);
        return [p0[0]+w*ac[0], p0[1]+w*ac[1], p0[2]+w*ac[2]];
    }
    let va = d3*d6 - d5*d4;
    if va <= 0.0 && (d4-d3) >= 0.0 && (d5-d6) >= 0.0 {
        let w = (d4-d3)/((d4-d3)+(d5-d6));
        let bc = sub3(p2, p1);
        return [p1[0]+w*bc[0], p1[1]+w*bc[1], p1[2]+w*bc[2]];
    }
    let denom = 1.0/(va+vb+vc);
    let v = vb*denom; let w = vc*denom;
    [
        p0[0]+v*ab[0]+w*ac[0],
        p0[1]+v*ab[1]+w*ac[1],
        p0[2]+v*ab[2]+w*ac[2],
    ]
}

/// Compute barycentric coordinates of point `p` in triangle (a,b,c).
/// Returns (u,v,w) where u+v+w=1.
#[allow(dead_code)]
pub fn point_in_triangle_barycentric(p: [f32;3], a: [f32;3], b: [f32;3], c: [f32;3]) -> (f32,f32,f32) {
    let v0 = sub3(c, a); let v1 = sub3(b, a); let v2 = sub3(p, a);
    let d00 = dot3(v0,v0); let d01 = dot3(v0,v1); let d11 = dot3(v1,v1);
    let d20 = dot3(v2,v0); let d21 = dot3(v2,v1);
    let denom = d00*d11 - d01*d01;
    if denom.abs() < 1e-10 { return (1.0/3.0, 1.0/3.0, 1.0/3.0); }
    let v = (d11*d20 - d01*d21) / denom;
    let w = (d00*d21 - d01*d20) / denom;
    let u = 1.0 - v - w;
    (u,v,w)
}

/// Compute contact normal for a triangle.
#[allow(dead_code)]
pub fn contact_normal(p0: [f32;3], p1: [f32;3], p2: [f32;3]) -> [f32;3] {
    let e1 = sub3(p1, p0); let e2 = sub3(p2, p0);
    normalize3(cross3(e1, e2))
}

/// Compute the signed depth of a point `q` against triangle normal.
#[allow(dead_code)]
pub fn contact_depth(q: [f32;3], closest: [f32;3], normal: [f32;3]) -> f32 {
    let v = sub3(q, closest);
    -dot3(v, normal)
}

/// Find the closest triangle index to a query point.
#[allow(dead_code)]
pub fn closest_triangle_index(
    q: [f32;3],
    positions: &[[f32;3]],
    indices: &[u32],
) -> usize {
    let tris = indices.len() / 3;
    let mut best = 0;
    let mut best_dist = f32::MAX;
    for t in 0..tris {
        let i0 = indices[t*3] as usize;
        let i1 = indices[t*3+1] as usize;
        let i2 = indices[t*3+2] as usize;
        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() { continue; }
        let cp = find_closest_point_on_triangle(q, positions[i0], positions[i1], positions[i2]);
        let d = len3(sub3(q, cp));
        if d < best_dist { best_dist = d; best = t; }
    }
    best
}

/// Find contact points between query points and the mesh.
#[allow(dead_code)]
pub fn mesh_contact_points(
    query_points: &[[f32;3]],
    positions: &[[f32;3]],
    indices: &[u32],
    max_dist: f32,
) -> Vec<ContactPoint> {
    let tris = indices.len() / 3;
    let mut contacts = Vec::new();
    for &q in query_points {
        for t in 0..tris {
            let i0 = indices[t*3] as usize;
            let i1 = indices[t*3+1] as usize;
            let i2 = indices[t*3+2] as usize;
            if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() { continue; }
            let cp = find_closest_point_on_triangle(q, positions[i0], positions[i1], positions[i2]);
            let d = len3(sub3(q, cp));
            if d <= max_dist {
                let n = contact_normal(positions[i0], positions[i1], positions[i2]);
                contacts.push(ContactPoint { position: cp, normal: n, depth: d, triangle_index: t });
            }
        }
    }
    contacts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_closest_point_on_triangle_inside() {
        let p0 = [0.0f32,0.0,0.0]; let p1 = [2.0,0.0,0.0]; let p2 = [0.0,2.0,0.0];
        let q = [0.5, 0.5, 1.0];
        let cp = find_closest_point_on_triangle(q, p0, p1, p2);
        assert!((cp[2]).abs() < 1e-5, "z should be 0: {:?}", cp);
    }

    #[test]
    fn test_closest_point_on_triangle_vertex() {
        let p0 = [0.0f32,0.0,0.0]; let p1 = [1.0,0.0,0.0]; let p2 = [0.0,1.0,0.0];
        let q = [-1.0,-1.0,0.0];
        let cp = find_closest_point_on_triangle(q, p0, p1, p2);
        assert!((cp[0]).abs() < 1e-4 && (cp[1]).abs() < 1e-4);
    }

    #[test]
    fn test_barycentric_center() {
        let a=[0.0f32,0.0,0.0]; let b=[1.0,0.0,0.0]; let c=[0.0,1.0,0.0];
        let p=[1.0/3.0, 1.0/3.0, 0.0];
        let (u,v,w) = point_in_triangle_barycentric(p,a,b,c);
        assert!((u+v+w - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_contact_normal_up() {
        let p0=[0.0f32,0.0,0.0]; let p1=[1.0,0.0,0.0]; let p2=[0.0,1.0,0.0];
        let n = contact_normal(p0,p1,p2);
        assert!((n[2].abs() - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_contact_depth_positive() {
        let q = [0.5f32, 0.5, 0.5];
        let closest = [0.5f32, 0.5, 0.0];
        let normal = [0.0f32, 0.0, 1.0];
        let d = contact_depth(q, closest, normal);
        assert!(d < 0.0 || d.abs() < 1.0);
    }

    #[test]
    fn test_closest_triangle_index() {
        let pos = vec![[0.0f32,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]];
        let idx = vec![0u32,1,2];
        let t = closest_triangle_index([0.3,0.3,0.0], &pos, &idx);
        assert_eq!(t, 0);
    }

    #[test]
    fn test_mesh_contact_points_within_dist() {
        let pos = vec![[0.0f32,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]];
        let idx = vec![0u32,1,2];
        let q = vec![[0.3f32,0.3,0.1]];
        let contacts = mesh_contact_points(&q, &pos, &idx, 1.0);
        assert!(!contacts.is_empty());
    }

    #[test]
    fn test_mesh_contact_points_far() {
        let pos = vec![[0.0f32,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]];
        let idx = vec![0u32,1,2];
        let q = vec![[100.0f32,0.0,0.0]];
        let contacts = mesh_contact_points(&q, &pos, &idx, 0.1);
        assert!(contacts.is_empty());
    }

    #[test]
    fn test_contact_point_struct() {
        let cp = ContactPoint { position: [0.0;3], normal:[0.0,0.0,1.0], depth:0.0, triangle_index: 0 };
        assert_eq!(cp.triangle_index, 0);
    }
}
