// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Binary STL with attribute bytes.

#[derive(Debug, Clone)]
pub struct BinaryStlTriangle {
    pub normal: [f32; 3],
    pub v0: [f32; 3],
    pub v1: [f32; 3],
    pub v2: [f32; 3],
    pub attribute: u16,
}

#[derive(Debug, Clone)]
pub struct BinaryStlMesh {
    pub header: [u8; 80],
    pub triangles: Vec<BinaryStlTriangle>,
}

pub fn new_binary_stl_mesh() -> BinaryStlMesh {
    let mut header = [0u8; 80];
    let msg = b"OxiHuman Binary STL";
    header[..msg.len()].copy_from_slice(msg);
    BinaryStlMesh {
        header,
        triangles: Vec::new(),
    }
}

pub fn add_binary_stl_triangle(mesh: &mut BinaryStlMesh, tri: BinaryStlTriangle) {
    mesh.triangles.push(tri);
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-10 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

fn write_f32(buf: &mut Vec<u8>, v: f32) {
    buf.extend_from_slice(&v.to_le_bytes());
}
fn write_u16(buf: &mut Vec<u8>, v: u16) {
    buf.extend_from_slice(&v.to_le_bytes());
}
fn write_u32(buf: &mut Vec<u8>, v: u32) {
    buf.extend_from_slice(&v.to_le_bytes());
}
fn write_vec3(buf: &mut Vec<u8>, v: [f32; 3]) {
    write_f32(buf, v[0]);
    write_f32(buf, v[1]);
    write_f32(buf, v[2]);
}

pub fn encode_binary_stl(mesh: &BinaryStlMesh) -> Vec<u8> {
    let mut buf = Vec::with_capacity(84 + mesh.triangles.len() * 50);
    buf.extend_from_slice(&mesh.header);
    write_u32(&mut buf, mesh.triangles.len() as u32);
    for tri in &mesh.triangles {
        write_vec3(&mut buf, tri.normal);
        write_vec3(&mut buf, tri.v0);
        write_vec3(&mut buf, tri.v1);
        write_vec3(&mut buf, tri.v2);
        write_u16(&mut buf, tri.attribute);
    }
    buf
}

pub fn mesh_to_binary_stl(positions: &[[f32; 3]], indices: &[u32]) -> BinaryStlMesh {
    let mut mesh = new_binary_stl_mesh();
    for tri in indices.chunks(3) {
        if tri.len() < 3 {
            continue;
        }
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if a >= positions.len() || b >= positions.len() || c >= positions.len() {
            continue;
        }
        let v0 = positions[a];
        let v1 = positions[b];
        let v2 = positions[c];
        let ab = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let ac = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let n = normalize3([
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ]);
        add_binary_stl_triangle(
            &mut mesh,
            BinaryStlTriangle {
                normal: n,
                v0,
                v1,
                v2,
                attribute: 0,
            },
        );
    }
    mesh
}

pub fn binary_stl_triangle_count(mesh: &BinaryStlMesh) -> usize {
    mesh.triangles.len()
}
pub fn binary_stl_size_bytes(mesh: &BinaryStlMesh) -> usize {
    84 + mesh.triangles.len() * 50
}
pub fn validate_binary_stl(mesh: &BinaryStlMesh) -> bool {
    !mesh.triangles.is_empty()
}

pub fn parse_binary_stl_header(data: &[u8]) -> Option<u32> {
    if data.len() < 84 {
        return None;
    }
    let count = u32::from_le_bytes([data[80], data[81], data[82], data[83]]);
    Some(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_binary_stl_mesh() {
        let m = new_binary_stl_mesh();
        assert_eq!(m.triangles.len(), 0);
        assert_eq!(m.header.len(), 80);
    }

    #[test]
    fn test_mesh_to_binary_stl() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let m = mesh_to_binary_stl(&pos, &idx);
        assert_eq!(binary_stl_triangle_count(&m), 1);
    }

    #[test]
    fn test_encode_binary_stl_size() {
        let m = new_binary_stl_mesh();
        let bytes = encode_binary_stl(&m);
        assert_eq!(bytes.len(), 84);
    }

    #[test]
    fn test_encode_binary_stl_with_tri() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let m = mesh_to_binary_stl(&pos, &idx);
        let bytes = encode_binary_stl(&m);
        assert_eq!(bytes.len(), 84 + 50);
    }

    #[test]
    fn test_binary_stl_size_bytes() {
        let m = new_binary_stl_mesh();
        assert_eq!(binary_stl_size_bytes(&m), 84);
    }

    #[test]
    fn test_validate_binary_stl_empty_fails() {
        let m = new_binary_stl_mesh();
        assert!(!validate_binary_stl(&m));
    }

    #[test]
    fn test_parse_binary_stl_header() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let m = mesh_to_binary_stl(&pos, &idx);
        let bytes = encode_binary_stl(&m);
        let count = parse_binary_stl_header(&bytes).expect("should succeed");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_header_contains_marker() {
        let m = new_binary_stl_mesh();
        let s = std::str::from_utf8(&m.header[..19]).expect("should succeed");
        assert!(s.contains("OxiHuman"));
    }
}
