// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! High-level [`AlembicWriter`] API: construction, mesh-buffer conversion,
//! animated-sequence export, and file I/O.
//!
//! All serialization primitives live in [`super::alembic_ogawa_core`].

use anyhow::{ensure, Result};
use oxihuman_mesh::MeshBuffers;

use super::alembic_ogawa_core::{
    build_archive_metadata, build_object_group, build_time_sampling, encode_u32,
    encode_string, serialize_ogawa, validate_object, AbcObject, AbcObjectKind, AbcPolyMesh,
    AlembicWriter, OgawaGroup, WriteContext,
};

// ── AlembicWriter implementation ────────────────────────────────────────────

impl AlembicWriter {
    /// Create a new empty writer.
    pub fn new() -> Self {
        Self {
            time_sampling: None,
            objects: Vec::new(),
        }
    }

    /// Add an object (and its subtree) to the archive.
    pub fn add_object(&mut self, obj: &AbcObject) -> Result<()> {
        validate_object(obj)?;
        self.objects.push(obj.clone());
        Ok(())
    }

    /// Configure time sampling for animated data.
    ///
    /// - `start`: start time in seconds
    /// - `dt`: time step between samples
    /// - `num_samples`: total number of time samples
    pub fn set_time_sampling(&mut self, start: f64, dt: f64, num_samples: usize) -> Result<()> {
        ensure!(dt > 0.0, "time step `dt` must be positive, got {}", dt);
        ensure!(
            num_samples > 0,
            "num_samples must be at least 1, got {}",
            num_samples
        );
        self.time_sampling = Some(super::alembic_ogawa_core::TimeSampling {
            start,
            dt,
            num_samples,
        });
        Ok(())
    }

    /// Return the number of animated frames currently recorded in this writer.
    ///
    /// For a static (single-mesh) writer this returns 0; for a sequence writer
    /// it returns the number of animated position samples attached to the first
    /// PolyMesh object.
    pub fn frame_count(&self) -> usize {
        for obj in &self.objects {
            if let AbcObjectKind::PolyMesh(ref mesh) = obj.kind {
                // 1 base frame + number of animated samples
                return 1 + mesh.animated_positions.len();
            }
        }
        0
    }

    /// Serialize all added objects to bytes (convenience alias for [`Self::export`]).
    ///
    /// Returns the Ogawa binary representation of the archive.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        self.export()
    }

    /// Write the archive to a file on disk.
    ///
    /// Creates (or truncates) the file at `path` and writes the Ogawa binary
    /// representation produced by [`Self::export`].
    pub fn write_to_file(&self, path: &std::path::Path) -> Result<()> {
        let data = self.export()?;
        std::fs::write(path, data)?;
        Ok(())
    }

    /// Create an [`AlembicWriter`] pre-populated from a single static mesh.
    ///
    /// The `MeshBuffers` triangle list is converted to an [`AbcPolyMesh`] with
    /// all face counts set to 3.  Positions are widened from `f32` to `f64`
    /// and normals / UVs are included when present.
    pub fn from_mesh_buffers(mesh: &MeshBuffers) -> Result<Self> {
        let positions: Vec<[f64; 3]> = mesh
            .positions
            .iter()
            .map(|p| [p[0] as f64, p[1] as f64, p[2] as f64])
            .collect();

        let face_counts: Vec<i32> = vec![3i32; mesh.indices.len() / 3];
        let face_indices: Vec<i32> = mesh.indices.iter().map(|&i| i as i32).collect();

        let normals: Option<Vec<[f64; 3]>> = if mesh.normals.len() == mesh.positions.len()
            && !mesh.normals.is_empty()
        {
            Some(
                mesh.normals
                    .iter()
                    .map(|n| [n[0] as f64, n[1] as f64, n[2] as f64])
                    .collect(),
            )
        } else {
            None
        };

        let uvs: Option<Vec<[f64; 2]>> = if mesh.uvs.len() == mesh.positions.len()
            && !mesh.uvs.is_empty()
        {
            Some(
                mesh.uvs
                    .iter()
                    .map(|uv| [uv[0] as f64, uv[1] as f64])
                    .collect(),
            )
        } else {
            None
        };

        let abc_mesh = AbcPolyMesh {
            positions,
            face_counts,
            face_indices,
            normals,
            uvs,
            animated_positions: Vec::new(),
        };

        let mut writer = Self::new();
        writer.add_object(&AbcObject {
            name: "mesh".into(),
            kind: AbcObjectKind::PolyMesh(abc_mesh),
            children: vec![],
        })?;
        Ok(writer)
    }

    /// Create an [`AlembicWriter`] for an animated mesh sequence.
    ///
    /// `frames` is a slice of per-frame [`MeshBuffers`]; `fps` is the playback
    /// rate in frames-per-second.  The first frame is used as the rest pose
    /// and all subsequent frames are stored as animated position samples.
    ///
    /// # Errors
    ///
    /// Returns an error when `frames` is empty or `fps` is not positive.
    pub fn from_mesh_sequence(frames: &[MeshBuffers], fps: f64) -> Result<Self> {
        ensure!(!frames.is_empty(), "frames must not be empty");
        ensure!(fps > 0.0, "fps must be positive, got {}", fps);

        let first = &frames[0];
        let positions: Vec<[f64; 3]> = first
            .positions
            .iter()
            .map(|p| [p[0] as f64, p[1] as f64, p[2] as f64])
            .collect();

        let face_counts: Vec<i32> = vec![3i32; first.indices.len() / 3];
        let face_indices: Vec<i32> = first.indices.iter().map(|&i| i as i32).collect();

        let dt = 1.0 / fps;
        let animated_positions: Vec<(f64, Vec<[f64; 3]>)> = frames
            .iter()
            .enumerate()
            .skip(1)
            .map(|(i, frame)| {
                let time = i as f64 * dt;
                let pos: Vec<[f64; 3]> = frame
                    .positions
                    .iter()
                    .map(|p| [p[0] as f64, p[1] as f64, p[2] as f64])
                    .collect();
                (time, pos)
            })
            .collect();

        let n_samples = frames.len();
        let abc_mesh = AbcPolyMesh {
            positions,
            face_counts,
            face_indices,
            normals: None,
            uvs: None,
            animated_positions,
        };

        let mut writer = Self::new();
        if n_samples > 1 {
            writer.set_time_sampling(0.0, dt, n_samples)?;
        }
        writer.add_object(&AbcObject {
            name: "anim_mesh".into(),
            kind: AbcObjectKind::PolyMesh(abc_mesh),
            children: vec![],
        })?;
        Ok(writer)
    }

    /// Serialize all added objects into a valid Ogawa binary `.abc` file.
    pub fn export(&self) -> Result<Vec<u8>> {
        use super::alembic_ogawa_core::ABC_CORE_VERSION;

        ensure!(!self.objects.is_empty(), "no objects added to export");

        let mut ctx = WriteContext { groups: Vec::new() };

        // Build the archive metadata group
        let meta_group_idx = build_archive_metadata(&mut ctx)?;

        // Build time sampling group
        let ts_group_idx = build_time_sampling(&mut ctx, &self.time_sampling)?;

        // Build object hierarchy starting from root
        let mut child_group_indices = Vec::new();
        for obj in &self.objects {
            let idx = build_object_group(&mut ctx, obj, &self.time_sampling)?;
            child_group_indices.push(idx);
        }

        // Build root object group (contains all top-level objects)
        let root_obj_idx = {
            let mut root_group = OgawaGroup::new();
            root_group.add_data(encode_string(""));
            root_group.add_data(encode_string("AbcObject_v1"));
            root_group.add_group(meta_group_idx);
            root_group.add_group(ts_group_idx);
            for &child_idx in &child_group_indices {
                root_group.add_group(child_idx);
            }
            ctx.alloc_group(root_group)
        };

        // Build the file-level root group
        let file_root_idx = {
            let mut file_root = OgawaGroup::new();
            let mut ver_data = Vec::with_capacity(4);
            ver_data.extend_from_slice(&encode_u32(ABC_CORE_VERSION));
            file_root.add_data(ver_data);
            file_root.add_group(root_obj_idx);
            ctx.alloc_group(file_root)
        };

        serialize_ogawa(&ctx.groups, file_root_idx)
    }
}

impl Default for AlembicWriter {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::alembic_ogawa_core::{
        identity_matrix, read_data_at, read_group_at, read_root_offset, scale_matrix,
        translation_matrix, unit_cube_polymesh, validate_ogawa_magic, AbcCamera, AbcSubD,
        AbcXform, ABC_CORE_VERSION,
    };

    #[test]
    fn test_ogawa_magic_valid() {
        let mut writer = AlembicWriter::new();
        writer
            .add_object(&AbcObject {
                name: "cube".into(),
                kind: AbcObjectKind::PolyMesh(unit_cube_polymesh()),
                children: vec![],
            })
            .expect("add_object failed");
        let data = writer.export().expect("export failed");
        assert!(validate_ogawa_magic(&data));
    }

    #[test]
    fn test_ogawa_magic_invalid() {
        assert!(!validate_ogawa_magic(b"NOT_ABC\x00"));
    }

    #[test]
    fn test_root_offset_present() {
        let mut writer = AlembicWriter::new();
        writer
            .add_object(&AbcObject {
                name: "test".into(),
                kind: AbcObjectKind::PolyMesh(unit_cube_polymesh()),
                children: vec![],
            })
            .expect("add_object failed");
        let data = writer.export().expect("export failed");
        let offset = read_root_offset(&data).expect("no root offset");
        assert!(offset >= 16, "root offset should be past the header");
        assert!((offset as usize) < data.len(), "root offset within file");
    }

    #[test]
    fn test_root_group_readable() {
        let mut writer = AlembicWriter::new();
        writer
            .add_object(&AbcObject {
                name: "mesh".into(),
                kind: AbcObjectKind::PolyMesh(unit_cube_polymesh()),
                children: vec![],
            })
            .expect("add_object failed");
        let data = writer.export().expect("export failed");
        let offset = read_root_offset(&data).expect("no root offset");
        let (count, _offsets) = read_group_at(&data, offset).expect("no root group");
        assert!(count >= 2, "root group child count: {}", count);
    }

    #[test]
    fn test_export_xform() {
        let mut writer = AlembicWriter::new();
        writer
            .add_object(&AbcObject {
                name: "xform_node".into(),
                kind: AbcObjectKind::Xform(AbcXform {
                    matrix: translation_matrix(1.0, 2.0, 3.0),
                    animated_matrices: vec![],
                }),
                children: vec![],
            })
            .expect("add_object failed");
        let data = writer.export().expect("export failed");
        assert!(validate_ogawa_magic(&data));
        assert!(data.len() > 16);
    }

    #[test]
    fn test_export_camera() {
        let mut writer = AlembicWriter::new();
        writer
            .add_object(&AbcObject {
                name: "cam".into(),
                kind: AbcObjectKind::Camera(AbcCamera {
                    focal_length: 50.0,
                    near_clip: 0.1,
                    far_clip: 1000.0,
                    horizontal_aperture: 3.6,
                    vertical_aperture: 2.4,
                }),
                children: vec![],
            })
            .expect("add_object failed");
        let data = writer.export().expect("export failed");
        assert!(validate_ogawa_magic(&data));
    }

    #[test]
    fn test_export_subd() {
        let mut writer = AlembicWriter::new();
        writer
            .add_object(&AbcObject {
                name: "subd_mesh".into(),
                kind: AbcObjectKind::SubD(AbcSubD {
                    positions: vec![
                        [-1.0, -1.0, 0.0],
                        [1.0, -1.0, 0.0],
                        [1.0, 1.0, 0.0],
                        [-1.0, 1.0, 0.0],
                    ],
                    face_counts: vec![4],
                    face_indices: vec![0, 1, 2, 3],
                    crease_indices: vec![0, 1],
                    crease_lengths: vec![2],
                    crease_sharpnesses: vec![3.0],
                }),
                children: vec![],
            })
            .expect("add_object failed");
        let data = writer.export().expect("export failed");
        assert!(validate_ogawa_magic(&data));
    }

    #[test]
    fn test_export_with_normals_and_uvs() {
        let mut mesh = unit_cube_polymesh();
        mesh.normals = Some(vec![[0.0, 0.0, 1.0]; 8]);
        mesh.uvs = Some(vec![[0.0, 0.0]; 8]);

        let mut writer = AlembicWriter::new();
        writer
            .add_object(&AbcObject {
                name: "textured".into(),
                kind: AbcObjectKind::PolyMesh(mesh),
                children: vec![],
            })
            .expect("add_object failed");
        let data = writer.export().expect("export failed");
        assert!(validate_ogawa_magic(&data));
        assert!(data.len() > 500);
    }

    #[test]
    fn test_export_animated_mesh() {
        let base = unit_cube_polymesh();
        let animated_positions = vec![
            (0.0, base.positions.clone()),
            (
                1.0 / 24.0,
                base.positions
                    .iter()
                    .map(|p| [p[0] + 0.1, p[1], p[2]])
                    .collect(),
            ),
        ];
        let mesh = AbcPolyMesh {
            animated_positions,
            ..base
        };

        let mut writer = AlembicWriter::new();
        writer.set_time_sampling(0.0, 1.0 / 24.0, 2).expect("set_time_sampling failed");
        writer
            .add_object(&AbcObject {
                name: "anim_cube".into(),
                kind: AbcObjectKind::PolyMesh(mesh),
                children: vec![],
            })
            .expect("add_object failed");
        let data = writer.export().expect("export failed");
        assert!(validate_ogawa_magic(&data));
    }

    #[test]
    fn test_export_animated_xform() {
        let mut writer = AlembicWriter::new();
        writer.set_time_sampling(0.0, 1.0 / 24.0, 3).expect("set_time_sampling failed");
        writer
            .add_object(&AbcObject {
                name: "moving".into(),
                kind: AbcObjectKind::Xform(AbcXform {
                    matrix: identity_matrix(),
                    animated_matrices: vec![
                        (0.0, translation_matrix(0.0, 0.0, 0.0)),
                        (1.0 / 24.0, translation_matrix(1.0, 0.0, 0.0)),
                        (2.0 / 24.0, translation_matrix(2.0, 0.0, 0.0)),
                    ],
                }),
                children: vec![],
            })
            .expect("add_object failed");
        let data = writer.export().expect("export failed");
        assert!(validate_ogawa_magic(&data));
    }

    #[test]
    fn test_hierarchy() {
        let mesh = AbcObject {
            name: "body".into(),
            kind: AbcObjectKind::PolyMesh(unit_cube_polymesh()),
            children: vec![],
        };
        let xform = AbcObject {
            name: "root_xform".into(),
            kind: AbcObjectKind::Xform(AbcXform {
                matrix: identity_matrix(),
                animated_matrices: vec![],
            }),
            children: vec![mesh],
        };

        let mut writer = AlembicWriter::new();
        writer.add_object(&xform).expect("add_object failed");
        let data = writer.export().expect("export failed");
        assert!(validate_ogawa_magic(&data));
        assert!(data.len() > 200);
    }

    #[test]
    fn test_multiple_objects() {
        let mut writer = AlembicWriter::new();
        for i in 0..5 {
            writer
                .add_object(&AbcObject {
                    name: format!("mesh_{}", i),
                    kind: AbcObjectKind::PolyMesh(unit_cube_polymesh()),
                    children: vec![],
                })
                .expect("add_object failed");
        }
        let data = writer.export().expect("export failed");
        assert!(validate_ogawa_magic(&data));
    }

    #[test]
    fn test_empty_writer_fails() {
        let writer = AlembicWriter::new();
        assert!(writer.export().is_err());
    }

    #[test]
    fn test_empty_name_rejected() {
        let mut writer = AlembicWriter::new();
        let result = writer.add_object(&AbcObject {
            name: "".into(),
            kind: AbcObjectKind::PolyMesh(unit_cube_polymesh()),
            children: vec![],
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_face_indices_rejected() {
        let mut writer = AlembicWriter::new();
        let result = writer.add_object(&AbcObject {
            name: "bad".into(),
            kind: AbcObjectKind::PolyMesh(AbcPolyMesh {
                positions: vec![[0.0, 0.0, 0.0]; 3],
                face_counts: vec![3],
                face_indices: vec![0, 1],
                normals: None,
                uvs: None,
                animated_positions: vec![],
            }),
            children: vec![],
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_camera_rejected() {
        let mut writer = AlembicWriter::new();
        let result = writer.add_object(&AbcObject {
            name: "bad_cam".into(),
            kind: AbcObjectKind::Camera(AbcCamera {
                focal_length: -10.0,
                near_clip: 0.1,
                far_clip: 100.0,
                horizontal_aperture: 3.6,
                vertical_aperture: 2.4,
            }),
            children: vec![],
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_time_sampling_rejected() {
        let mut writer = AlembicWriter::new();
        assert!(writer.set_time_sampling(0.0, 0.0, 10).is_err());
        assert!(writer.set_time_sampling(0.0, 1.0, 0).is_err());
    }

    #[test]
    fn test_data_node_read() {
        let mut writer = AlembicWriter::new();
        writer
            .add_object(&AbcObject {
                name: "obj".into(),
                kind: AbcObjectKind::PolyMesh(unit_cube_polymesh()),
                children: vec![],
            })
            .expect("add_object failed");
        let data = writer.export().expect("export failed");

        let root_off = read_root_offset(&data).expect("no root offset");
        let (count, offsets) = read_group_at(&data, root_off).expect("no root group");
        assert!(count > 0);

        // First child of root should be version data
        let version_data = read_data_at(&data, offsets[0]);
        assert!(version_data.is_some());
        let vd = version_data.expect("version data missing");
        assert_eq!(vd.len(), 4);
        let ver = u32::from_le_bytes([vd[0], vd[1], vd[2], vd[3]]);
        assert_eq!(ver, ABC_CORE_VERSION);
    }

    #[test]
    fn test_identity_matrix_values() {
        let m = identity_matrix();
        assert!((m[0] - 1.0).abs() < f64::EPSILON);
        assert!((m[5] - 1.0).abs() < f64::EPSILON);
        assert!((m[10] - 1.0).abs() < f64::EPSILON);
        assert!((m[15] - 1.0).abs() < f64::EPSILON);
        assert!((m[1]).abs() < f64::EPSILON);
        assert!((m[4]).abs() < f64::EPSILON);
    }

    #[test]
    fn test_translation_matrix_values() {
        let m = translation_matrix(5.0, 10.0, 15.0);
        assert!((m[12] - 5.0).abs() < f64::EPSILON);
        assert!((m[13] - 10.0).abs() < f64::EPSILON);
        assert!((m[14] - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_scale_matrix_values() {
        let m = scale_matrix(2.5);
        assert!((m[0] - 2.5).abs() < f64::EPSILON);
        assert!((m[5] - 2.5).abs() < f64::EPSILON);
        assert!((m[10] - 2.5).abs() < f64::EPSILON);
        assert!((m[15] - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_mismatched_animated_positions_rejected() {
        let mut mesh = unit_cube_polymesh();
        mesh.animated_positions = vec![(0.0, mesh.positions.clone()), (1.0, vec![[0.0; 3]; 5])];
        let mut writer = AlembicWriter::new();
        let result = writer.add_object(&AbcObject {
            name: "bad_anim".into(),
            kind: AbcObjectKind::PolyMesh(mesh),
            children: vec![],
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_mismatched_normals_rejected() {
        let mut mesh = unit_cube_polymesh();
        mesh.normals = Some(vec![[0.0, 0.0, 1.0]; 3]);
        let mut writer = AlembicWriter::new();
        assert!(writer
            .add_object(&AbcObject {
                name: "bad_n".into(),
                kind: AbcObjectKind::PolyMesh(mesh),
                children: vec![],
            })
            .is_err());
    }

    #[test]
    fn test_file_to_disk_round_trip() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_alembic_ogawa_io.abc");

        let mut writer = AlembicWriter::new();
        writer
            .add_object(&AbcObject {
                name: "cube".into(),
                kind: AbcObjectKind::PolyMesh(unit_cube_polymesh()),
                children: vec![],
            })
            .expect("add_object failed");
        let data = writer.export().expect("export failed");

        std::fs::write(&path, &data).expect("write failed");
        let read_back = std::fs::read(&path).expect("read failed");
        assert_eq!(data, read_back);
        assert!(validate_ogawa_magic(&read_back));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_nested_hierarchy_deep() {
        let leaf = AbcObject {
            name: "leaf_mesh".into(),
            kind: AbcObjectKind::PolyMesh(unit_cube_polymesh()),
            children: vec![],
        };
        let level2 = AbcObject {
            name: "level2".into(),
            kind: AbcObjectKind::Xform(AbcXform {
                matrix: translation_matrix(0.0, 1.0, 0.0),
                animated_matrices: vec![],
            }),
            children: vec![leaf],
        };
        let level1 = AbcObject {
            name: "level1".into(),
            kind: AbcObjectKind::Xform(AbcXform {
                matrix: scale_matrix(2.0),
                animated_matrices: vec![],
            }),
            children: vec![level2],
        };
        let root = AbcObject {
            name: "scene_root".into(),
            kind: AbcObjectKind::Xform(AbcXform {
                matrix: identity_matrix(),
                animated_matrices: vec![],
            }),
            children: vec![level1],
        };

        let mut writer = AlembicWriter::new();
        writer.add_object(&root).expect("add_object failed");
        let data = writer.export().expect("export failed");
        assert!(validate_ogawa_magic(&data));
        assert!(data.len() > 300);
    }

    #[test]
    fn test_unit_cube_topology() {
        let mesh = unit_cube_polymesh();
        assert_eq!(mesh.positions.len(), 8);
        assert_eq!(mesh.face_counts.len(), 6);
        assert!(mesh.face_counts.iter().all(|&c| c == 4));
        assert_eq!(mesh.face_indices.len(), 24);
    }
}

#[cfg(test)]
mod convenience_api_tests {
    use super::*;
    use super::super::alembic_ogawa_core::validate_ogawa_magic;
    use oxihuman_morph::engine::MeshBuffers as MorphMeshBuffers;

    /// Build a minimal triangle [`MeshBuffers`] suitable for export tests.
    fn make_triangle_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MorphMeshBuffers {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            has_suit: false,
        })
    }

    #[test]
    fn test_alembic_magic_bytes() {
        let mesh = make_triangle_mesh();
        let writer = AlembicWriter::from_mesh_buffers(&mesh)
            .expect("from_mesh_buffers failed");
        let data = writer.to_bytes().expect("to_bytes failed");
        // Ogawa magic: 0xFF 0x00 0x00 0x00 0x00 0x01 0x00 0x00 at offset 0
        assert!(
            data.len() > 8,
            "output should be larger than the magic header"
        );
        assert_eq!(
            &data[..8],
            &[0xFF, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00],
            "first 8 bytes must be Ogawa magic"
        );
    }

    #[test]
    fn test_single_mesh_round_trip() {
        let mesh = make_triangle_mesh();
        let writer = AlembicWriter::from_mesh_buffers(&mesh)
            .expect("from_mesh_buffers failed");
        let path = std::env::temp_dir().join("test_alembic_single_convenience.abc");
        writer.write_to_file(&path).expect("write_to_file failed");
        assert!(path.exists(), "output file must exist after write_to_file");
        let data = std::fs::read(&path).expect("failed to re-read written file");
        // Root group offset is stored at bytes 8..16 (u64 LE)
        assert!(data.len() >= 16, "file must be at least 16 bytes");
        let root_off = u64::from_le_bytes(
            data[8..16]
                .try_into()
                .expect("slice length is exactly 8 bytes"),
        );
        assert!(root_off > 0, "root group offset must be non-zero");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_animated_sequence_frame_count() {
        let frames: Vec<MeshBuffers> = (0..10).map(|_| make_triangle_mesh()).collect();
        let writer =
            AlembicWriter::from_mesh_sequence(&frames, 24.0).expect("from_mesh_sequence failed");
        assert_eq!(
            writer.frame_count(),
            10,
            "writer should record all 10 frames"
        );
    }

    #[test]
    fn test_from_mesh_buffers_with_normals_uvs() {
        let mesh = make_triangle_mesh();
        let writer = AlembicWriter::from_mesh_buffers(&mesh).expect("from_mesh_buffers failed");
        let data = writer.export().expect("export failed");
        assert!(validate_ogawa_magic(&data));
        // Should have normals and UVs embedded
        assert!(data.len() > 100);
    }

    #[test]
    fn test_from_mesh_sequence_single_frame() {
        let frames = vec![make_triangle_mesh()];
        let writer =
            AlembicWriter::from_mesh_sequence(&frames, 24.0).expect("from_mesh_sequence failed");
        // Single frame: no time sampling
        assert_eq!(writer.frame_count(), 1);
        let data = writer.export().expect("export failed");
        assert!(validate_ogawa_magic(&data));
    }

    #[test]
    fn test_from_mesh_sequence_empty_fails() {
        let result = AlembicWriter::from_mesh_sequence(&[], 24.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_mesh_sequence_zero_fps_fails() {
        let frames = vec![make_triangle_mesh()];
        let result = AlembicWriter::from_mesh_sequence(&frames, 0.0);
        assert!(result.is_err());
    }
}
