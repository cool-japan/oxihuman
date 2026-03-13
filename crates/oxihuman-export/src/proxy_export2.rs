//! Proxy mesh export (distinct from proxy_export).
#![allow(dead_code)]

/// A simplified proxy mesh.
#[allow(dead_code)]
pub struct ProxyMesh2 {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub name: String,
}

/// Export proxy mesh to OBJ string.
#[allow(dead_code)]
pub fn export_proxy_mesh2(proxy: &ProxyMesh2) -> String {
    let mut s = format!("# Proxy mesh: {}\n", proxy.name);
    for &[x,y,z] in &proxy.positions { s.push_str(&format!("v {} {} {}\n", x, y, z)); }
    let tris = proxy.indices.len() / 3;
    for t in 0..tris {
        let i0 = proxy.indices[t*3]+1; let i1 = proxy.indices[t*3+1]+1; let i2 = proxy.indices[t*3+2]+1;
        s.push_str(&format!("f {} {} {}\n", i0, i1, i2));
    }
    s
}

/// Build a proxy from existing positions/indices.
#[allow(dead_code)]
pub fn proxy2_from_mesh(positions: Vec<[f32;3]>, indices: Vec<u32>, name: &str) -> ProxyMesh2 {
    ProxyMesh2 { positions, indices, name: name.to_string() }
}

/// Get vertex count.
#[allow(dead_code)]
pub fn proxy2_vertex_count(proxy: &ProxyMesh2) -> usize { proxy.positions.len() }

/// Get face count.
#[allow(dead_code)]
pub fn proxy2_face_count(proxy: &ProxyMesh2) -> usize { proxy.indices.len() / 3 }

/// Export proxy to OBJ string.
#[allow(dead_code)]
pub fn proxy2_to_obj_string(proxy: &ProxyMesh2) -> String { export_proxy_mesh2(proxy) }

/// Get bounding box [min, max].
#[allow(dead_code)]
pub fn proxy2_bounds(proxy: &ProxyMesh2) -> Option<[[f32;3];2]> {
    if proxy.positions.is_empty() { return None; }
    let mut mn = proxy.positions[0]; let mut mx = proxy.positions[0];
    for &p in &proxy.positions {
        for k in 0..3 {
            if p[k] < mn[k] { mn[k] = p[k]; }
            if p[k] > mx[k] { mx[k] = p[k]; }
        }
    }
    Some([mn, mx])
}

/// Get proxy center.
#[allow(dead_code)]
pub fn proxy2_center(proxy: &ProxyMesh2) -> [f32;3] {
    let n = proxy.positions.len();
    if n == 0 { return [0.0;3]; }
    let sum = proxy.positions.iter().fold([0.0f32;3], |a, p| [a[0]+p[0], a[1]+p[1], a[2]+p[2]]);
    [sum[0]/n as f32, sum[1]/n as f32, sum[2]/n as f32]
}

/// Get proxy scale (largest extent dimension).
#[allow(dead_code)]
pub fn proxy2_scale(proxy: &ProxyMesh2) -> f32 {
    if let Some([mn, mx]) = proxy2_bounds(proxy) {
        let dx = mx[0]-mn[0]; let dy = mx[1]-mn[1]; let dz = mx[2]-mn[2];
        dx.max(dy).max(dz)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_proxy() -> ProxyMesh2 {
        let pos = vec![[0.0f32,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]];
        let idx = vec![0u32,1,2];
        proxy2_from_mesh(pos, idx, "test")
    }

    #[test]
    fn test_proxy_vertex_count() {
        let p = sample_proxy();
        assert_eq!(proxy2_vertex_count(&p), 3);
    }

    #[test]
    fn test_proxy_face_count() {
        let p = sample_proxy();
        assert_eq!(proxy2_face_count(&p), 1);
    }

    #[test]
    fn test_proxy_to_obj_string() {
        let p = sample_proxy();
        let s = proxy2_to_obj_string(&p);
        assert!(s.contains("v "));
    }

    #[test]
    fn test_proxy_bounds() {
        let p = sample_proxy();
        let b = proxy2_bounds(&p).expect("should succeed");
        assert!((b[0][0]).abs() < 1e-5);
    }

    #[test]
    fn test_proxy_center() {
        let p = sample_proxy();
        let c = proxy2_center(&p);
        assert!((c[0] - 1.0/3.0).abs() < 1e-4);
    }

    #[test]
    fn test_proxy_scale_positive() {
        let p = sample_proxy();
        assert!(proxy2_scale(&p) > 0.0);
    }

    #[test]
    fn test_proxy_bounds_empty() {
        let p = ProxyMesh2 { positions: vec![], indices: vec![], name: "e".to_string() };
        assert!(proxy2_bounds(&p).is_none());
    }

    #[test]
    fn test_proxy_center_empty() {
        let p = ProxyMesh2 { positions: vec![], indices: vec![], name: "e".to_string() };
        let c = proxy2_center(&p);
        assert!((c[0]).abs() < 1e-5);
    }

    #[test]
    fn test_export_proxy_mesh_name() {
        let p = sample_proxy();
        let s = export_proxy_mesh2(&p);
        assert!(s.contains("test"));
    }
}
