// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Main viewer struct and render loop.

use crate::camera::CameraState;
use crate::gpu::MeshUploadBuffer;
use crate::scene_state::{ViewerConfig, ViewerStats};

// ── Viewer ────────────────────────────────────────────────────────────────────

/// Stub viewer -- will be replaced by wgpu surface in Phase 2.
pub struct Viewer {
    config: ViewerConfig,
    pub camera: CameraState,
    pub current_mesh: Option<MeshUploadBuffer>,
    frame_count: u64,
}

impl Viewer {
    pub fn new(config: ViewerConfig) -> Self {
        Viewer {
            config,
            camera: CameraState::default(),
            current_mesh: None,
            frame_count: 0,
        }
    }

    /// Upload a mesh buffer to the viewer (CPU-side store; GPU upload in Phase 2).
    pub fn upload_mesh(&mut self, buf: MeshUploadBuffer) {
        self.current_mesh = Some(buf);
    }

    /// Simulate a render tick.  Returns [`ViewerStats`] for the current frame.
    pub fn render_frame(&mut self) -> ViewerStats {
        self.frame_count += 1;
        let (verts, tris) = self
            .current_mesh
            .as_ref()
            .map(|m| (m.positions.len(), m.indices.len() / 3))
            .unwrap_or((0, 0));
        ViewerStats {
            frame_count: self.frame_count,
            vertex_count: verts,
            triangle_count: tris,
        }
    }

    /// Return a reference to the current camera state.
    pub fn get_camera(&self) -> &CameraState {
        &self.camera
    }

    /// Orbit the camera around its target.
    pub fn orbit_camera(&mut self, yaw: f32, pitch: f32) {
        self.camera.orbit(yaw, pitch);
    }

    /// Return a reference to the viewer configuration.
    pub fn config(&self) -> &ViewerConfig {
        &self.config
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewer_render_frame_returns_stats() {
        let mut v = Viewer::new(ViewerConfig::default());
        let stats = v.render_frame();
        assert_eq!(stats.frame_count, 1);
        assert_eq!(stats.vertex_count, 0);
        assert_eq!(stats.triangle_count, 0);
    }

    #[test]
    fn viewer_upload_mesh_reflected_in_stats() {
        let buf = MeshUploadBuffer {
            positions: vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0; 2]; 3],
            indices: vec![0, 1, 2],
            timestamp: 0,
        };
        let mut v = Viewer::new(ViewerConfig::default());
        v.upload_mesh(buf);
        let stats = v.render_frame();
        assert_eq!(stats.vertex_count, 3);
        assert_eq!(stats.triangle_count, 1);
    }

    #[test]
    fn viewer_orbit_camera_delegates() {
        let mut v = Viewer::new(ViewerConfig::default());
        let before = v.get_camera().position;
        v.orbit_camera(30.0, 0.0);
        assert_ne!(v.get_camera().position, before);
    }

    #[test]
    fn viewer_frame_count_increments() {
        let mut v = Viewer::new(ViewerConfig::default());
        v.render_frame();
        v.render_frame();
        let s = v.render_frame();
        assert_eq!(s.frame_count, 3);
    }
}
