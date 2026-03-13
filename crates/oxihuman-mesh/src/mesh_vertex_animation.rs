// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Vertex animation frame storage — stores per-frame vertex position arrays.

/// A single frame of vertex animation data.
#[derive(Debug, Clone)]
pub struct VertexAnimFrame {
    pub frame_index: u32,
    pub time: f32,
    pub positions: Vec<[f32; 3]>,
}

/// A sequence of vertex animation frames.
#[derive(Debug, Default, Clone)]
pub struct VertexAnimSequence {
    pub frames: Vec<VertexAnimFrame>,
    pub frame_rate: f32,
    pub vertex_count: usize,
}

impl VertexAnimSequence {
    /// Creates a new sequence with the given frame rate and vertex count.
    pub fn new(frame_rate: f32, vertex_count: usize) -> Self {
        Self {
            frame_rate,
            vertex_count,
            frames: Vec::new(),
        }
    }

    /// Adds a frame to the sequence.
    pub fn push_frame(&mut self, frame: VertexAnimFrame) {
        self.frames.push(frame);
    }

    /// Returns the total duration in seconds.
    pub fn duration(&self) -> f32 {
        self.frames.last().map(|f| f.time).unwrap_or(0.0)
    }

    /// Returns the frame count.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Looks up the frame nearest to the given time.
    pub fn frame_at_time(&self, time: f32) -> Option<&VertexAnimFrame> {
        self.frames.iter().min_by(|a, b| {
            (a.time - time)
                .abs()
                .partial_cmp(&(b.time - time).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

/// Interpolates between two vertex position arrays at factor `t`.
pub fn lerp_frames(a: &[[f32; 3]], b: &[[f32; 3]], t: f32) -> Vec<[f32; 3]> {
    let n = a.len().min(b.len());
    let t = t.clamp(0.0, 1.0);
    (0..n)
        .map(|i| {
            [
                a[i][0] + (b[i][0] - a[i][0]) * t,
                a[i][1] + (b[i][1] - a[i][1]) * t,
                a[i][2] + (b[i][2] - a[i][2]) * t,
            ]
        })
        .collect()
}

/// Computes the maximum displacement between two frames.
pub fn max_frame_displacement(a: &[[f32; 3]], b: &[[f32; 3]]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(pa, pb)| {
            let dx = pa[0] - pb[0];
            let dy = pa[1] - pb[1];
            let dz = pa[2] - pb[2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .fold(0.0f32, f32::max)
}

/// Validates that all frames have the expected vertex count.
pub fn validate_frame_vertex_counts(seq: &VertexAnimSequence) -> bool {
    seq.frames
        .iter()
        .all(|f| f.positions.len() == seq.vertex_count)
}

/// Computes the average position across all vertices in a frame.
pub fn frame_centroid(frame: &VertexAnimFrame) -> [f32; 3] {
    if frame.positions.is_empty() {
        return [0.0; 3];
    }
    let n = frame.positions.len() as f32;
    let sum = frame.positions.iter().fold([0.0f32; 3], |acc, p| {
        [acc[0] + p[0], acc[1] + p[1], acc[2] + p[2]]
    });
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_frame(idx: u32, t: f32, n: usize) -> VertexAnimFrame {
        VertexAnimFrame {
            frame_index: idx,
            time: t,
            positions: vec![[0.0; 3]; n],
        }
    }

    #[test]
    fn test_new_sequence_empty() {
        /* New sequence should have zero frames */
        assert_eq!(VertexAnimSequence::new(24.0, 100).frame_count(), 0);
    }

    #[test]
    fn test_push_frame() {
        /* Pushing a frame should increase count */
        let mut seq = VertexAnimSequence::new(24.0, 10);
        seq.push_frame(make_frame(0, 0.0, 10));
        assert_eq!(seq.frame_count(), 1);
    }

    #[test]
    fn test_duration_empty() {
        /* Empty sequence should have zero duration */
        assert_eq!(VertexAnimSequence::new(24.0, 10).duration(), 0.0);
    }

    #[test]
    fn test_duration_last_frame() {
        /* Duration should equal last frame's time */
        let mut seq = VertexAnimSequence::new(24.0, 10);
        seq.push_frame(make_frame(0, 0.0, 10));
        seq.push_frame(make_frame(1, 1.0, 10));
        assert!((seq.duration() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_frame_at_time() {
        /* frame_at_time should return nearest frame */
        let mut seq = VertexAnimSequence::new(24.0, 5);
        seq.push_frame(make_frame(0, 0.0, 5));
        seq.push_frame(make_frame(1, 1.0, 5));
        let f = seq.frame_at_time(0.4).expect("should succeed");
        assert_eq!(f.frame_index, 0);
    }

    #[test]
    fn test_lerp_frames_at_zero() {
        /* t=0 should return array a */
        let a = vec![[1.0f32, 0.0, 0.0]];
        let b = vec![[3.0f32, 0.0, 0.0]];
        let r = lerp_frames(&a, &b, 0.0);
        assert!((r[0][0] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_lerp_frames_at_one() {
        /* t=1 should return array b */
        let a = vec![[1.0f32, 0.0, 0.0]];
        let b = vec![[3.0f32, 0.0, 0.0]];
        let r = lerp_frames(&a, &b, 1.0);
        assert!((r[0][0] - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_max_frame_displacement_same() {
        /* Same frames → zero displacement */
        let pos = vec![[1.0f32, 2.0, 3.0]];
        assert_eq!(max_frame_displacement(&pos, &pos), 0.0);
    }

    #[test]
    fn test_validate_frame_vertex_counts_valid() {
        /* Frames with correct vertex count should validate */
        let mut seq = VertexAnimSequence::new(24.0, 3);
        seq.push_frame(make_frame(0, 0.0, 3));
        assert!(validate_frame_vertex_counts(&seq));
    }

    #[test]
    fn test_frame_centroid_origin() {
        /* Frame at origin should have centroid at origin */
        let frame = make_frame(0, 0.0, 4);
        let c = frame_centroid(&frame);
        assert_eq!(c, [0.0; 3]);
    }
}
