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

/// [`ObjMesh`] plus the raw-`v`-line → packed-vertex mapping produced while
/// packing face corners (see [`parse_obj_with_mapping`]).
#[derive(Debug, Clone, Default)]
pub struct ObjMeshWithMapping {
    /// The packed mesh, identical to what [`parse_obj`] returns.
    pub mesh: ObjMesh,
    /// `raw_to_packed[i]` lists every packed vertex index that carries the
    /// position of raw `v`-line `i` (0-based). A raw vertex referenced with
    /// several distinct `vt`/`vn` combinations is split into one packed copy
    /// per combination (UV/normal seam splitting), so an entry holds `0..n`
    /// packed indices — `0` only for a raw vertex no face references.
    ///
    /// Every packed vertex appears in exactly one entry, exactly once, so
    /// this is the authoritative permutation for re-indexing external
    /// per-vertex data (e.g. MakeHuman `.target` sparse deltas, which address
    /// raw `v`-line order) onto the packed mesh.
    pub raw_to_packed: Vec<Vec<u32>>,
}

/// Parse OBJ source into a packed [`ObjMesh`] (face-first-occurrence vertex
/// order with UV/normal-seam splitting).
///
/// Equivalent to [`parse_obj_with_mapping`] with the raw→packed mapping
/// discarded; both share the same parser internals.
pub fn parse_obj(src: &str) -> Result<ObjMesh> {
    Ok(parse_obj_with_mapping(src)?.mesh)
}

/// Parse OBJ source, additionally returning the mapping from raw `v`-line
/// indices to the packed vertex indices the parser assigned them.
///
/// The packed vertex order is *not* the `v`-line order: vertices are emitted
/// in face-first-occurrence order and split per distinct `v/vt/vn`
/// combination. Data indexed by raw `v`-line order (MakeHuman `.target`
/// sparse deltas) must be re-indexed through
/// [`ObjMeshWithMapping::raw_to_packed`] — duplicating each raw entry across
/// all seam copies — before it can address the packed mesh.
pub fn parse_obj_with_mapping(src: &str) -> Result<ObjMeshWithMapping> {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut raw_uvs: Vec<[f32; 2]> = Vec::new();
    let mut raw_normals: Vec<[f32; 3]> = Vec::new();

    // Final per-vertex arrays (indexed by vertex index used in faces)
    let mut out_positions: Vec<[f32; 3]> = Vec::new();
    let mut out_uvs: Vec<[f32; 2]> = Vec::new();
    let mut out_normals: Vec<[f32; 3]> = Vec::new();
    let mut out_indices: Vec<u32> = Vec::new();

    // raw v-line index → all packed copies produced for it.
    let mut raw_to_packed: Vec<Vec<u32>> = Vec::new();

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
                raw_to_packed.push(Vec::new());
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
                            &mut raw_to_packed,
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

    Ok(ObjMeshWithMapping {
        mesh: ObjMesh {
            positions: out_positions,
            normals: out_normals,
            uvs: out_uvs,
            indices: out_indices,
        },
        raw_to_packed,
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
    raw_to_packed: &mut [Vec<u32>],
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
    if let Some(copies) = raw_to_packed.get_mut(pi - 1) {
        copies.push(idx);
    }
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
        let path = oxihuman_test_utils::base_obj();
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

    /// A quad + a triangle sharing raw vertices 2 and 3, but with *different*
    /// UV indices on the triangle — a genuine UV seam, so those raw vertices
    /// must be split into two packed copies each.
    const SEAM_OBJ: &str = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 1.0 1.0 0.0
v 0.0 1.0 0.0
v 0.5 2.0 0.0
vt 0.0 0.0
vt 1.0 0.0
vt 1.0 1.0
vt 0.0 1.0
vt 0.25 0.75
vt 0.75 0.75
vt 0.5 1.0
f 1/1 2/2 3/3 4/4
f 4/5 3/6 5/7
"#;

    /// Verify the raw→packed mapping on the synthetic seam mesh: seam
    /// vertices are duplicated, mapping positions match the raw positions,
    /// and every packed vertex appears exactly once across the mapping.
    #[test]
    fn mapping_splits_uv_seams() {
        let parsed = parse_obj_with_mapping(SEAM_OBJ).expect("should succeed");
        let raw_positions: [[f32; 3]; 5] = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.5, 2.0, 0.0],
        ];
        assert_eq!(parsed.raw_to_packed.len(), 5, "one entry per v line");
        // 4 quad corners + 2 seam re-splits + 1 apex = 7 packed vertices.
        assert_eq!(parsed.mesh.positions.len(), 7);
        assert_eq!(parsed.raw_to_packed[0].len(), 1);
        assert_eq!(parsed.raw_to_packed[1].len(), 1);
        assert_eq!(parsed.raw_to_packed[2].len(), 2, "v3 sits on the seam");
        assert_eq!(parsed.raw_to_packed[3].len(), 2, "v4 sits on the seam");
        assert_eq!(parsed.raw_to_packed[4].len(), 1);
        for (raw, copies) in parsed.raw_to_packed.iter().enumerate() {
            for &packed in copies {
                assert_eq!(
                    parsed.mesh.positions[packed as usize], raw_positions[raw],
                    "packed copy {packed} of raw {raw} must carry the raw position"
                );
            }
        }
    }

    /// Exhaustive mapping invariants shared by every parse: (a) each packed
    /// vertex appears exactly once across the whole mapping, (b) every packed
    /// copy carries exactly the raw `v`-line position, and (c) `parse_obj`
    /// and `parse_obj_with_mapping` produce identical meshes.
    fn assert_mapping_invariants(src: &str) {
        let parsed = parse_obj_with_mapping(src).expect("parse with mapping");
        let plain = parse_obj(src).expect("plain parse");
        assert_eq!(plain.positions, parsed.mesh.positions);
        assert_eq!(plain.indices, parsed.mesh.indices);
        assert_eq!(plain.uvs, parsed.mesh.uvs);

        // Re-derive the raw v-line positions directly from the source text so
        // the check does not trust the parser under test.
        let raw_positions: Vec<[f32; 3]> = src
            .lines()
            .map(str::trim)
            .filter_map(|l| l.strip_prefix("v "))
            .map(|rest| parse_floats3(rest).expect("raw v line parses"))
            .collect();
        assert_eq!(parsed.raw_to_packed.len(), raw_positions.len());

        let n_packed = parsed.mesh.positions.len();
        let mut seen = vec![0u32; n_packed];
        for (raw, copies) in parsed.raw_to_packed.iter().enumerate() {
            for &packed in copies {
                let p = packed as usize;
                assert!(p < n_packed, "packed index {packed} out of range");
                seen[p] += 1;
                assert_eq!(
                    parsed.mesh.positions[p], raw_positions[raw],
                    "packed {packed} != raw {raw} position"
                );
            }
        }
        for (packed, count) in seen.iter().enumerate() {
            assert_eq!(
                *count, 1,
                "packed vertex {packed} must appear exactly once across the mapping, saw {count}"
            );
        }
    }

    #[test]
    fn mapping_invariants_synthetic() {
        assert_mapping_invariants(SIMPLE_OBJ);
        assert_mapping_invariants(SEAM_OBJ);
    }

    /// The real MakeHuman base mesh (skipped on a lean checkout): the packed
    /// mesh has more vertices than v-lines (UV-seam splits) and the mapping
    /// is a complete, position-preserving partition of the packed vertices.
    #[test]
    fn mapping_invariants_base_obj() {
        let path = oxihuman_test_utils::base_obj();
        let Ok(src) = std::fs::read_to_string(&path) else {
            eprintln!("upstream base.obj absent — skipping mapping invariant test");
            return;
        };
        let parsed = parse_obj_with_mapping(&src).expect("should succeed");
        assert!(
            parsed.mesh.positions.len() > parsed.raw_to_packed.len(),
            "expected seam splitting: {} packed vs {} raw",
            parsed.mesh.positions.len(),
            parsed.raw_to_packed.len()
        );
        assert_mapping_invariants(&src);
    }
}
