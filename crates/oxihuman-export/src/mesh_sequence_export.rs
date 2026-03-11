// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Mesh sequence (vertex animation cache) export.

/// A single frame of mesh positions.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshFrame {
    pub time: f32,
    pub positions: Vec<[f32; 3]>,
}

/// Mesh sequence export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshSequenceExport {
    pub frames: Vec<MeshFrame>,
    pub vertex_count: usize,
}

/// Create new sequence.
#[allow(dead_code)]
pub fn new_mesh_sequence(vertex_count: usize) -> MeshSequenceExport {
    MeshSequenceExport {
        frames: vec![],
        vertex_count,
    }
}

/// Add a frame.
#[allow(dead_code)]
pub fn add_frame(e: &mut MeshSequenceExport, time: f32, positions: &[[f32; 3]]) {
    if positions.len() == e.vertex_count {
        e.frames.push(MeshFrame {
            time,
            positions: positions.to_vec(),
        });
    }
}

/// Frame count.
#[allow(dead_code)]
pub fn ms_frame_count(e: &MeshSequenceExport) -> usize {
    e.frames.len()
}

/// Duration.
#[allow(dead_code)]
pub fn ms_duration(e: &MeshSequenceExport) -> f32 {
    if e.frames.is_empty() {
        return 0.0;
    }
    let min = e.frames.iter().map(|f| f.time).fold(f32::MAX, f32::min);
    let max = e.frames.iter().map(|f| f.time).fold(f32::MIN, f32::max);
    max - min
}

/// Estimated size in bytes.
#[allow(dead_code)]
pub fn ms_size_bytes(e: &MeshSequenceExport) -> usize {
    e.frames.len() * e.vertex_count * 12 // 3 floats * 4 bytes
}

/// Get positions at frame.
#[allow(dead_code)]
pub fn get_frame_positions(e: &MeshSequenceExport, idx: usize) -> Option<&[[f32; 3]]> {
    e.frames.get(idx).map(|f| f.positions.as_slice())
}

/// FPS estimate.
#[allow(dead_code)]
pub fn ms_fps(e: &MeshSequenceExport) -> f32 {
    let dur = ms_duration(e);
    if dur < 1e-12 || e.frames.len() < 2 {
        return 0.0;
    }
    (e.frames.len() - 1) as f32 / dur
}

/// Validate.
#[allow(dead_code)]
pub fn ms_validate(e: &MeshSequenceExport) -> bool {
    e.frames
        .iter()
        .all(|f| f.positions.len() == e.vertex_count && f.time >= 0.0)
}

/// Export to JSON.
#[allow(dead_code)]
pub fn mesh_sequence_to_json(e: &MeshSequenceExport) -> String {
    format!(
        "{{\"frames\":{},\"vertices\":{},\"duration\":{:.6}}}",
        ms_frame_count(e),
        e.vertex_count,
        ms_duration(e)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new() {
        let e = new_mesh_sequence(3);
        assert_eq!(ms_frame_count(&e), 0);
    }
    #[test]
    fn test_add_frame() {
        let mut e = new_mesh_sequence(2);
        add_frame(&mut e, 0.0, &[[0.0; 3]; 2]);
        assert_eq!(ms_frame_count(&e), 1);
    }
    #[test]
    fn test_wrong_count() {
        let mut e = new_mesh_sequence(2);
        add_frame(&mut e, 0.0, &[[0.0; 3]; 3]);
        assert_eq!(ms_frame_count(&e), 0);
    }
    #[test]
    fn test_duration() {
        let mut e = new_mesh_sequence(1);
        add_frame(&mut e, 0.0, &[[0.0; 3]]);
        add_frame(&mut e, 2.0, &[[1.0; 3]]);
        assert!((ms_duration(&e) - 2.0).abs() < 1e-6);
    }
    #[test]
    fn test_duration_empty() {
        let e = new_mesh_sequence(1);
        assert!((ms_duration(&e)).abs() < 1e-9);
    }
    #[test]
    fn test_size() {
        let mut e = new_mesh_sequence(10);
        add_frame(&mut e, 0.0, &[[0.0; 3]; 10]);
        assert_eq!(ms_size_bytes(&e), 120);
    }
    #[test]
    fn test_get_frame() {
        let mut e = new_mesh_sequence(1);
        add_frame(&mut e, 0.0, &[[5.0, 0.0, 0.0]]);
        let p = get_frame_positions(&e, 0).unwrap();
        assert!((p[0][0] - 5.0).abs() < 1e-6);
    }
    #[test]
    fn test_fps() {
        let mut e = new_mesh_sequence(1);
        for i in 0..25 {
            add_frame(&mut e, i as f32 / 24.0, &[[0.0; 3]]);
        }
        assert!((ms_fps(&e) - 24.0).abs() < 0.1);
    }
    #[test]
    fn test_validate() {
        let mut e = new_mesh_sequence(1);
        add_frame(&mut e, 0.0, &[[0.0; 3]]);
        assert!(ms_validate(&e));
    }
    #[test]
    fn test_to_json() {
        let e = new_mesh_sequence(5);
        assert!(mesh_sequence_to_json(&e).contains("\"vertices\":5"));
    }
}
