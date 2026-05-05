// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};

#[derive(Debug, Clone, Default)]
pub struct ObjMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

pub fn parse_obj(src: &str) -> Result<ObjMesh> {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut raw_uvs: Vec<[f32; 2]> = Vec::new();
    let mut raw_normals: Vec<[f32; 3]> = Vec::new();

    // Final per-vertex arrays (indexed by vertex index used in faces)
    let mut out_positions: Vec<[f32; 3]> = Vec::new();
    let mut out_uvs: Vec<[f32; 2]> = Vec::new();
    let mut out_normals: Vec<[f32; 3]> = Vec::new();
    let mut out_indices: Vec<u32> = Vec::new();

    // Cache: (pos_idx, uv_idx, norm_idx) → output index
    use std::collections::HashMap;
    let mut cache: HashMap<(u32, u32, u32), u32> = HashMap::new();

    for line in src.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.splitn(2, char::is_whitespace);
        let kw = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        let rest = parts.next().unwrap_or("").trim();

        match kw {
            "v" => {
                let coords = parse_floats3(rest)?;
                positions.push(coords);
            }
            "vt" => {
                let coords = parse_floats2(rest)?;
                raw_uvs.push(coords);
            }
            "vn" => {
                let coords = parse_floats3(rest)?;
                raw_normals.push(coords);
            }
            "f" => {
                // face: triangulate quads (fan triangulation)
                let verts: Vec<_> = rest.split_whitespace().collect();
                if verts.len() < 3 {
                    continue;
                }
                let face_indices: Vec<u32> = verts
                    .iter()
                    .map(|v| {
                        resolve_vertex(
                            v,
                            &positions,
                            &raw_uvs,
                            &raw_normals,
                            &mut out_positions,
                            &mut out_uvs,
                            &mut out_normals,
                            &mut cache,
                        )
                    })
                    .collect::<Result<_>>()?;

                // Fan triangulation
                for i in 1..face_indices.len() - 1 {
                    out_indices.push(face_indices[0]);
                    out_indices.push(face_indices[i]);
                    out_indices.push(face_indices[i + 1]);
                }
            }
            _ => {}
        }
    }

    Ok(ObjMesh {
        positions: out_positions,
        normals: out_normals,
        uvs: out_uvs,
        indices: out_indices,
    })
}
#[allow(clippy::too_many_arguments)]
fn resolve_vertex(
    spec: &str,
    positions: &[[f32; 3]],
    raw_uvs: &[[f32; 2]],
    raw_normals: &[[f32; 3]],
    out_pos: &mut Vec<[f32; 3]>,
    out_uvs: &mut Vec<[f32; 2]>,
    out_normals: &mut Vec<[f32; 3]>,
    cache: &mut std::collections::HashMap<(u32, u32, u32), u32>,
) -> Result<u32> {
    let parts: Vec<&str> = spec.split('/').collect();
    let pi = parse_obj_index(parts[0])? as usize;
    let ui = if parts.len() > 1 && !parts[1].is_empty() {
        parse_obj_index(parts[1])? as usize
    } else {
        0
    };
    let ni = if parts.len() > 2 && !parts[2].is_empty() {
        parse_obj_index(parts[2])? as usize
    } else {
        0
    };

    let key = (pi as u32, ui as u32, ni as u32);
    if let Some(&idx) = cache.get(&key) {
        return Ok(idx);
    }

    if pi == 0 || pi > positions.len() {
        bail!("position index {} out of range ({})", pi, positions.len());
    }
    let idx = out_pos.len() as u32;
    out_pos.push(positions[pi - 1]);
    if ui > 0 && ui <= raw_uvs.len() {
        out_uvs.push(raw_uvs[ui - 1]);
    } else {
        out_uvs.push([0.0, 0.0]);
    }
    if ni > 0 && ni <= raw_normals.len() {
        out_normals.push(raw_normals[ni - 1]);
    } else {
        out_normals.push([0.0, 1.0, 0.0]);
    }
    cache.insert(key, idx);
    Ok(idx)
}

fn parse_obj_index(s: &str) -> Result<u32> {
    Ok(s.trim().parse::<u32>()?)
}

fn parse_floats3(s: &str) -> Result<[f32; 3]> {
    let v: Vec<f32> = s
        .split_whitespace()
        .take(3)
        .map(|x| {
            x.parse::<f32>()
                .map_err(|e| anyhow::anyhow!("{}: {}", x, e))
        })
        .collect::<Result<_>>()?;
    if v.len() < 3 {
        bail!("expected 3 floats, got {}", v.len());
    }
    Ok([v[0], v[1], v[2]])
}

fn parse_floats2(s: &str) -> Result<[f32; 2]> {
    let v: Vec<f32> = s
        .split_whitespace()
        .take(2)
        .map(|x| {
            x.parse::<f32>()
                .map_err(|e| anyhow::anyhow!("{}: {}", x, e))
        })
        .collect::<Result<_>>()?;
    if v.len() < 2 {
        bail!("expected 2 floats, got {}", v.len());
    }
    Ok([v[0], v[1]])
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_OBJ: &str = r#"
# simple quad
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 1.0 1.0 0.0
v 0.0 1.0 0.0
vt 0.0 0.0
vt 1.0 0.0
vt 1.0 1.0
vt 0.0 1.0
vn 0.0 0.0 1.0
f 1/1/1 2/2/1 3/3/1 4/4/1
"#;

    #[test]
    fn parse_simple_quad() {
        let mesh = parse_obj(SIMPLE_OBJ).expect("should succeed");
        assert_eq!(mesh.positions.len(), 4);
        assert_eq!(mesh.indices.len(), 6); // quad → 2 triangles
    }

    #[test]
    fn parse_base_obj() {
        let path = {
            std::env::var("MAKEHUMAN_DATA_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|_| std::path::PathBuf::from("/tmp/oxihuman_nonexistent_data"))
                .join("3dobjs/base.obj")
        };
        if let Ok(src) = std::fs::read_to_string(&path) {
            let mesh = parse_obj(&src).expect("should succeed");
            // MakeHuman base mesh has ~19,158 unique base positions
            assert!(
                mesh.positions.len() > 10_000,
                "expected many vertices, got {}",
                mesh.positions.len()
            );
            assert!(!mesh.indices.is_empty());
        }
    }
}
