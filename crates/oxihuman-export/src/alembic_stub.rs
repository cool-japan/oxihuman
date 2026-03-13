// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Alembic-like export stub (text/binary, no real HDF5/Ogawa dependency).

#[allow(dead_code)]
/// Schema descriptor for an Alembic object.
#[derive(Debug, Clone)]
pub struct AlembicSchema {
    pub name: String,
    /// "GeomMesh" / "PolyMesh" / "XForm"
    pub kind: String,
}

#[allow(dead_code)]
/// One time sample of mesh data.
#[derive(Debug, Clone)]
pub struct AlembicSample {
    pub time: f32,
    pub positions: Vec<[f32; 3]>,
    pub face_counts: Vec<u32>,
    pub face_indices: Vec<u32>,
}

#[allow(dead_code)]
/// A named object with schema and samples.
#[derive(Debug, Clone)]
pub struct AlembicObject {
    pub path: String,
    pub schema: AlembicSchema,
    pub samples: Vec<AlembicSample>,
}

#[allow(dead_code)]
/// Root archive.
#[derive(Debug, Clone)]
pub struct AlembicArchive {
    pub start_time: f32,
    pub end_time: f32,
    pub fps: f32,
    pub objects: Vec<AlembicObject>,
}

// ---------------------------------------------------------------------------
// Builders
// ---------------------------------------------------------------------------

/// Build a single-sample static mesh archive.
pub fn build_single_mesh_archive(
    positions: &[[f32; 3]],
    tris: &[[u32; 3]],
    name: &str,
) -> AlembicArchive {
    let face_counts: Vec<u32> = vec![3; tris.len()];
    let face_indices: Vec<u32> = tris.iter().flat_map(|t| t.iter().copied()).collect();
    let sample = AlembicSample {
        time: 0.0,
        positions: positions.to_vec(),
        face_counts,
        face_indices,
    };
    let obj = AlembicObject {
        path: format!("/{}", name),
        schema: AlembicSchema {
            name: name.to_string(),
            kind: "PolyMesh".into(),
        },
        samples: vec![sample],
    };
    AlembicArchive {
        start_time: 0.0,
        end_time: 0.0,
        fps: 24.0,
        objects: vec![obj],
    }
}

/// Build an animated archive from per-frame position arrays.
pub fn build_animated_archive(
    frames: &[Vec<[f32; 3]>],
    tris: &[[u32; 3]],
    fps: f32,
) -> AlembicArchive {
    let face_counts: Vec<u32> = vec![3; tris.len()];
    let face_indices: Vec<u32> = tris.iter().flat_map(|t| t.iter().copied()).collect();
    let samples: Vec<AlembicSample> = frames
        .iter()
        .enumerate()
        .map(|(i, pos)| AlembicSample {
            time: i as f32 / fps.max(1.0),
            positions: pos.clone(),
            face_counts: face_counts.clone(),
            face_indices: face_indices.clone(),
        })
        .collect();
    let end_time = if frames.is_empty() {
        0.0
    } else {
        (frames.len() - 1) as f32 / fps.max(1.0)
    };
    let obj = AlembicObject {
        path: "/AnimMesh".into(),
        schema: AlembicSchema {
            name: "AnimMesh".into(),
            kind: "GeomMesh".into(),
        },
        samples,
    };
    AlembicArchive {
        start_time: 0.0,
        end_time,
        fps,
        objects: vec![obj],
    }
}

// ---------------------------------------------------------------------------
// Binary stub (magic + JSON body)
// ---------------------------------------------------------------------------

const OGAWA_MAGIC: &[u8] = b"OxiABC\x00\x01";

/// Serialise archive to minimal binary stub.
pub fn archive_to_ogawa_stub(archive: &AlembicArchive) -> Vec<u8> {
    let json = archive_to_json_inner(archive);
    let body = json.as_bytes();
    let mut out = Vec::with_capacity(OGAWA_MAGIC.len() + 4 + body.len());
    out.extend_from_slice(OGAWA_MAGIC);
    let len = body.len() as u32;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(body);
    out
}

/// Parse a stub produced by `archive_to_ogawa_stub`.
pub fn parse_ogawa_stub(data: &[u8]) -> Option<AlembicArchive> {
    if data.len() < OGAWA_MAGIC.len() + 4 {
        return None;
    }
    if &data[..OGAWA_MAGIC.len()] != OGAWA_MAGIC {
        return None;
    }
    let len_bytes: [u8; 4] = data[OGAWA_MAGIC.len()..OGAWA_MAGIC.len() + 4]
        .try_into()
        .ok()?;
    let body_len = u32::from_le_bytes(len_bytes) as usize;
    let body_start = OGAWA_MAGIC.len() + 4;
    if data.len() < body_start + body_len {
        return None;
    }
    let json_str = std::str::from_utf8(&data[body_start..body_start + body_len]).ok()?;
    archive_from_json_inner(json_str)
}

// ---------------------------------------------------------------------------
// JSON helpers (no external deps)
// ---------------------------------------------------------------------------

fn f32_vec3_to_json(v: &[[f32; 3]]) -> String {
    let parts: Vec<String> = v
        .iter()
        .map(|p| format!("[{:.6},{:.6},{:.6}]", p[0], p[1], p[2]))
        .collect();
    format!("[{}]", parts.join(","))
}

fn u32_vec_to_json(v: &[u32]) -> String {
    let parts: Vec<String> = v.iter().map(|x| x.to_string()).collect();
    format!("[{}]", parts.join(","))
}

fn archive_to_json_inner(archive: &AlembicArchive) -> String {
    let mut obj_jsons = Vec::new();
    for obj in &archive.objects {
        let mut sample_jsons = Vec::new();
        for s in &obj.samples {
            sample_jsons.push(format!(
                r#"{{"time":{:.6},"positions":{},"face_counts":{},"face_indices":{}}}"#,
                s.time,
                f32_vec3_to_json(&s.positions),
                u32_vec_to_json(&s.face_counts),
                u32_vec_to_json(&s.face_indices),
            ));
        }
        obj_jsons.push(format!(
            r#"{{"path":"{}","schema_name":"{}","schema_kind":"{}","samples":[{}]}}"#,
            obj.path,
            obj.schema.name,
            obj.schema.kind,
            sample_jsons.join(","),
        ));
    }
    format!(
        r#"{{"start_time":{:.6},"end_time":{:.6},"fps":{:.6},"objects":[{}]}}"#,
        archive.start_time,
        archive.end_time,
        archive.fps,
        obj_jsons.join(","),
    )
}

fn parse_f32(s: &str) -> f32 {
    s.trim().parse().unwrap_or(0.0)
}

fn parse_u32(s: &str) -> u32 {
    s.trim().parse().unwrap_or(0)
}

/// Very minimal JSON parser for round-trip of our own format.
fn archive_from_json_inner(json: &str) -> Option<AlembicArchive> {
    // Extract top-level scalar fields
    let start_time = extract_f32_field(json, "start_time").unwrap_or(0.0);
    let end_time = extract_f32_field(json, "end_time").unwrap_or(0.0);
    let fps = extract_f32_field(json, "fps").unwrap_or(24.0);

    // Extract objects array
    let objects_str = extract_array_str(json, "objects")?;
    let objects = parse_objects_array(objects_str);

    Some(AlembicArchive {
        start_time,
        end_time,
        fps,
        objects,
    })
}

fn extract_f32_field(json: &str, key: &str) -> Option<f32> {
    let pattern = format!(r#""{}":"#, key);
    let start = json.find(&pattern)? + pattern.len();
    let rest = json[start..].trim_start();
    let end = rest
        .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
        .unwrap_or(rest.len());
    Some(parse_f32(&rest[..end]))
}

fn extract_str_field(json: &str, key: &str) -> Option<String> {
    let p = format!(r#""{}":""#, key);
    let start = json.find(&p)? + p.len();
    let end = json[start..].find('"')?;
    Some(json[start..start + end].to_string())
}

fn extract_array_str<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let pattern = format!(r#""{}":["#, key);
    let start = json.find(&pattern)? + pattern.len() - 1; // include '['
                                                          // find matching ']'
    let sub = &json[start..];
    let mut depth = 0i32;
    let mut end = 0;
    for (i, c) in sub.char_indices() {
        match c {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    end = i + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    if end == 0 {
        return None;
    }
    Some(&sub[..end])
}

fn parse_f32_array(json: &str) -> Vec<f32> {
    // json like "[1.0,2.0,3.0]"
    let inner = json.trim().trim_start_matches('[').trim_end_matches(']');
    if inner.trim().is_empty() {
        return vec![];
    }
    inner.split(',').map(parse_f32).collect()
}

fn parse_u32_array(json: &str) -> Vec<u32> {
    let inner = json.trim().trim_start_matches('[').trim_end_matches(']');
    if inner.trim().is_empty() {
        return vec![];
    }
    inner.split(',').map(parse_u32).collect()
}

fn parse_vec3_array(json: &str) -> Vec<[f32; 3]> {
    // json like "[[x,y,z],[x,y,z],...]"
    let trimmed = json.trim();
    // Strip exactly one outer '[' and ']'
    let inner = trimmed
        .strip_prefix('[')
        .unwrap_or(trimmed)
        .strip_suffix(']')
        .unwrap_or(trimmed.strip_prefix('[').unwrap_or(trimmed));
    if inner.trim().is_empty() {
        return vec![];
    }
    let mut result = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    let bytes = inner.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'[' => {
                if depth == 0 {
                    start = i;
                }
                depth += 1;
            }
            b']' => {
                depth -= 1;
                if depth == 0 {
                    let part = &inner[start..=i];
                    let vals = parse_f32_array(part);
                    if vals.len() >= 3 {
                        result.push([vals[0], vals[1], vals[2]]);
                    }
                }
            }
            _ => {}
        }
    }
    result
}

fn parse_sample(json: &str) -> AlembicSample {
    let time = extract_f32_field(json, "time").unwrap_or(0.0);
    let positions = if let Some(s) = extract_array_str(json, "positions") {
        parse_vec3_array(s)
    } else {
        vec![]
    };
    let face_counts = if let Some(s) = extract_array_str(json, "face_counts") {
        parse_u32_array(s)
    } else {
        vec![]
    };
    let face_indices = if let Some(s) = extract_array_str(json, "face_indices") {
        parse_u32_array(s)
    } else {
        vec![]
    };
    AlembicSample {
        time,
        positions,
        face_counts,
        face_indices,
    }
}

fn parse_objects_array(json: &str) -> Vec<AlembicObject> {
    // json is the full "[{...},{...}]" string
    let inner = json.trim().trim_start_matches('[').trim_end_matches(']');
    if inner.trim().is_empty() {
        return vec![];
    }
    let mut objects = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    let bytes = inner.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'{' => {
                if depth == 0 {
                    start = i;
                }
                depth += 1;
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    let obj_str = &inner[start..=i];
                    let path = extract_str_field(obj_str, "path").unwrap_or_default();
                    let schema_name = extract_str_field(obj_str, "schema_name").unwrap_or_default();
                    let schema_kind = extract_str_field(obj_str, "schema_kind").unwrap_or_default();
                    let samples_arr = extract_array_str(obj_str, "samples").unwrap_or("[]");
                    let samples = parse_samples_array(samples_arr);
                    objects.push(AlembicObject {
                        path,
                        schema: AlembicSchema {
                            name: schema_name,
                            kind: schema_kind,
                        },
                        samples,
                    });
                }
            }
            _ => {}
        }
    }
    objects
}

fn parse_samples_array(json: &str) -> Vec<AlembicSample> {
    let inner = json.trim().trim_start_matches('[').trim_end_matches(']');
    if inner.trim().is_empty() {
        return vec![];
    }
    let mut samples = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    let bytes = inner.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'{' => {
                if depth == 0 {
                    start = i;
                }
                depth += 1;
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    let s_str = &inner[start..=i];
                    samples.push(parse_sample(s_str));
                }
            }
            _ => {}
        }
    }
    samples
}

// ---------------------------------------------------------------------------
// Archive stats / utilities
// ---------------------------------------------------------------------------

/// Number of frames (samples in first object).
pub fn archive_frame_count(archive: &AlembicArchive) -> usize {
    archive
        .objects
        .first()
        .map(|o| o.samples.len())
        .unwrap_or(0)
}

/// Vertex count from the first sample of the first object.
pub fn archive_vertex_count(archive: &AlembicArchive) -> usize {
    archive
        .objects
        .first()
        .and_then(|o| o.samples.first())
        .map(|s| s.positions.len())
        .unwrap_or(0)
}

/// Validate the archive and return a list of errors.
pub fn validate_archive(archive: &AlembicArchive) -> Vec<String> {
    let mut errors = Vec::new();
    if archive.fps <= 0.0 {
        errors.push("fps must be positive".into());
    }
    if archive.end_time < archive.start_time {
        errors.push("end_time < start_time".into());
    }
    if archive.objects.is_empty() {
        errors.push("archive has no objects".into());
    }
    for obj in &archive.objects {
        if obj.path.is_empty() {
            errors.push("object has empty path".into());
        }
        for (si, s) in obj.samples.iter().enumerate() {
            let expected_idx: usize = s.face_counts.iter().map(|&c| c as usize).sum();
            if !s.face_indices.is_empty() && expected_idx != s.face_indices.len() {
                errors.push(format!(
                    "object '{}' sample {}: face_indices length mismatch",
                    obj.path, si
                ));
            }
        }
    }
    errors
}

/// Compute AABB over all samples in all objects.
pub fn archive_bounding_box(archive: &AlembicArchive) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for obj in &archive.objects {
        for s in &obj.samples {
            for p in &s.positions {
                for d in 0..3 {
                    if p[d] < min[d] {
                        min[d] = p[d];
                    }
                    if p[d] > max[d] {
                        max[d] = p[d];
                    }
                }
            }
        }
    }
    if min[0].is_infinite() {
        ([0.0; 3], [0.0; 3])
    } else {
        (min, max)
    }
}

/// Merge two archives by combining their objects.
pub fn merge_archives(mut a: AlembicArchive, b: AlembicArchive) -> AlembicArchive {
    a.objects.extend(b.objects);
    a.start_time = a.start_time.min(b.start_time);
    a.end_time = a.end_time.max(b.end_time);
    a
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn tri_cube_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ]
    }

    fn tri_indices() -> Vec<[u32; 3]> {
        vec![[0, 1, 2], [0, 2, 3], [4, 5, 6], [4, 6, 7]]
    }

    #[test]
    fn test_build_single_mesh_has_one_object() {
        let pos = tri_cube_positions();
        let tris = tri_indices();
        let archive = build_single_mesh_archive(&pos, &tris, "cube");
        assert_eq!(archive.objects.len(), 1);
    }

    #[test]
    fn test_build_single_mesh_vertex_count() {
        let pos = tri_cube_positions();
        let tris = tri_indices();
        let archive = build_single_mesh_archive(&pos, &tris, "cube");
        assert_eq!(archive_vertex_count(&archive), 8);
    }

    #[test]
    fn test_build_animated_frame_count() {
        let pos = tri_cube_positions();
        let tris = tri_indices();
        let frames: Vec<Vec<[f32; 3]>> = (0..10).map(|_| pos.clone()).collect();
        let archive = build_animated_archive(&frames, &tris, 24.0);
        assert_eq!(archive_frame_count(&archive), 10);
    }

    #[test]
    fn test_build_animated_single_frame() {
        let pos = tri_cube_positions();
        let tris = tri_indices();
        let archive = build_animated_archive(&[pos], &tris, 24.0);
        assert_eq!(archive_frame_count(&archive), 1);
    }

    #[test]
    fn test_ogawa_stub_round_trip() {
        let pos = tri_cube_positions();
        let tris = tri_indices();
        let archive = build_single_mesh_archive(&pos, &tris, "test");
        let bytes = archive_to_ogawa_stub(&archive);
        let recovered = parse_ogawa_stub(&bytes).expect("round-trip failed");
        assert_eq!(recovered.objects.len(), 1);
        assert!((recovered.fps - 24.0).abs() < 0.1);
    }

    #[test]
    fn test_ogawa_stub_vertex_count_round_trip() {
        let pos = tri_cube_positions();
        let tris = tri_indices();
        let archive = build_single_mesh_archive(&pos, &tris, "cube");
        let bytes = archive_to_ogawa_stub(&archive);
        let recovered = parse_ogawa_stub(&bytes).expect("should succeed");
        assert_eq!(archive_vertex_count(&recovered), 8);
    }

    #[test]
    fn test_parse_bad_magic_returns_none() {
        let bad = b"NOTMAGIC\x00\x00\x00\x00";
        assert!(parse_ogawa_stub(bad).is_none());
    }

    #[test]
    fn test_parse_too_short_returns_none() {
        assert!(parse_ogawa_stub(b"Ox").is_none());
    }

    #[test]
    fn test_validate_empty_archive_has_error() {
        let archive = AlembicArchive {
            start_time: 0.0,
            end_time: 0.0,
            fps: 24.0,
            objects: vec![],
        };
        let errors = validate_archive(&archive);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_validate_valid_archive_no_errors() {
        let pos = tri_cube_positions();
        let tris = tri_indices();
        let archive = build_single_mesh_archive(&pos, &tris, "ok");
        let errors = validate_archive(&archive);
        assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
    }

    #[test]
    fn test_bounding_box_covers_all_points() {
        let pos = vec![[0.0f32, 0.0, 0.0], [5.0, 3.0, 1.0], [-1.0, 2.0, 4.0]];
        let tris = vec![[0u32, 1, 2]];
        let archive = build_single_mesh_archive(&pos, &tris, "bb");
        let (mn, mx) = archive_bounding_box(&archive);
        assert!(mn[0] <= -1.0);
        assert!(mx[0] >= 5.0);
        assert!(mx[1] >= 3.0);
    }

    #[test]
    fn test_bounding_box_empty_archive() {
        let archive = AlembicArchive {
            start_time: 0.0,
            end_time: 0.0,
            fps: 24.0,
            objects: vec![],
        };
        let (mn, mx) = archive_bounding_box(&archive);
        assert_eq!(mn, [0.0; 3]);
        assert_eq!(mx, [0.0; 3]);
    }

    #[test]
    fn test_merge_archives_combines_objects() {
        let pos = tri_cube_positions();
        let tris = tri_indices();
        let a = build_single_mesh_archive(&pos, &tris, "a");
        let b = build_single_mesh_archive(&pos, &tris, "b");
        let merged = merge_archives(a, b);
        assert_eq!(merged.objects.len(), 2);
    }
}
