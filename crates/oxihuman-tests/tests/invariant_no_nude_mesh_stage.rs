// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! # Safety invariant: no unclothed human mesh ever leaves the exporter.
//!
//! OxiHuman never writes or returns a human mesh that has not had its bodysuit
//! layer applied. Every human-facing exporter entry point in `oxihuman-export`
//! calls `oxihuman_export::export_gate::ensure_export_allowed`, which refuses a
//! [`oxihuman_mesh::MeshBuffers`] whose `has_suit` flag is `false`.
//!
//! This test is the machine-checked enforcement of that invariant. It builds
//! ONE unsuited mesh (`has_suit = false`) and drives it through EVERY gated
//! entry point listed in the authoritative doc table at the top of
//! `crates/oxihuman-export/src/export_gate.rs`, asserting each one returns
//! `Err`. It then confirms the positive path: a suited mesh (`has_suit = true`)
//! is accepted by GLB / OBJ / STL / VRM.
//!
//! If you add a new human-mesh exporter, add it to the `export_gate.rs` table
//! AND to this test. If this test fails to compile because a signature moved,
//! the export surface changed — re-audit the gate before "fixing" the test.

use oxihuman_export as ox;
use oxihuman_mesh::MeshBuffers;

/// A minimal valid triangle mesh with the given suit state.
fn mesh(has_suit: bool) -> MeshBuffers {
    MeshBuffers {
        positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        normals: vec![[0.0, 0.0, 1.0]; 3],
        tangents: vec![[1.0, 0.0, 0.0, 1.0]; 3],
        uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
        indices: vec![0, 1, 2],
        colors: None,
        has_suit,
    }
}

/// A unique temp path so parallel test runs never collide.
fn tmp(tag: &str, ext: &str) -> std::path::PathBuf {
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("oxihuman_invariant_{tag}_{pid}_{nanos}.{ext}"))
}

fn skeleton() -> oxihuman_mesh::Skeleton {
    oxihuman_mesh::Skeleton {
        joints: vec![oxihuman_mesh::Joint {
            name: "root".to_string(),
            parent: None,
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        }],
    }
}

/// THE invariant: every gated exporter refuses an unsuited human mesh, and
/// accepts a suited one.
#[test]
fn invariant_no_nude_mesh_stage() {
    let m = mesh(false); // the mesh that must never be exportable
    let s = mesh(true); // the suited mesh the positive path accepts

    // ── glb.rs ────────────────────────────────────────────────────────────
    assert!(ox::glb::build_glb_bytes(&m).is_err(), "build_glb_bytes");
    assert!(
        ox::glb::export_glb(&m, &tmp("glb", "glb")).is_err(),
        "export_glb"
    );
    assert!(
        ox::glb::export_glb_with_material(
            &m,
            &ox::material::PbrMaterial::default_material(),
            &tmp("glbmat", "glb"),
        )
        .is_err(),
        "export_glb_with_material"
    );
    let meta = ox::metadata::OxiHumanMeta::minimal();
    assert!(
        ox::glb::build_glb_with_meta_bytes(&m, &meta).is_err(),
        "build_glb_with_meta_bytes"
    );
    assert!(
        ox::glb::export_glb_with_meta(&m, &meta, &tmp("glbmeta", "glb")).is_err(),
        "export_glb_with_meta"
    );
    let skel = skeleton();
    assert!(
        ox::glb::build_glb_with_skeleton_bytes(&m, &skel).is_err(),
        "build_glb_with_skeleton_bytes"
    );
    assert!(
        ox::glb::export_glb_with_skeleton(&m, &skel, &tmp("glbskel", "glb")).is_err(),
        "export_glb_with_skeleton"
    );

    // ── gltf_sep.rs ───────────────────────────────────────────────────────
    assert!(
        ox::gltf_sep::export_gltf_sep(&m, &tmp("gltf", "gltf"), &tmp("gltf", "bin")).is_err(),
        "export_gltf_sep"
    );

    // ── obj.rs ────────────────────────────────────────────────────────────
    assert!(
        ox::obj::mesh_to_obj_string(&m).is_err(),
        "mesh_to_obj_string"
    );
    assert!(
        ox::obj::export_obj(&m, &tmp("obj", "obj")).is_err(),
        "export_obj"
    );

    // ── obj_mtl.rs ────────────────────────────────────────────────────────
    assert!(
        ox::obj_mtl::export_obj_mtl(
            &m,
            &tmp("objmtl", "obj"),
            &[ox::obj_mtl::MtlMaterial::default()],
            &ox::obj_mtl::ObjMtlOptions::default(),
        )
        .is_err(),
        "export_obj_mtl"
    );

    // ── stl.rs ────────────────────────────────────────────────────────────
    assert!(
        ox::stl::mesh_to_stl_ascii(&m, "body").is_err(),
        "mesh_to_stl_ascii"
    );
    assert!(
        ox::stl::export_stl_ascii(&m, &tmp("stla", "stl"), "body").is_err(),
        "export_stl_ascii"
    );
    assert!(ox::stl::encode_stl_binary(&m).is_err(), "encode_stl_binary");
    assert!(
        ox::stl::export_stl_binary(&m, &tmp("stlb", "stl")).is_err(),
        "export_stl_binary"
    );

    // ── ascii_stl_export.rs ───────────────────────────────────────────────
    assert!(
        ox::ascii_stl_export::export_ascii_stl_mesh(
            &m,
            &ox::ascii_stl_export::AsciiStlOptions::default(),
        )
        .is_err(),
        "export_ascii_stl_mesh"
    );

    // ── binary_stl_export.rs ──────────────────────────────────────────────
    assert!(
        ox::binary_stl_export::encode_binary_stl_mesh(&m).is_err(),
        "encode_binary_stl_mesh"
    );

    // ── vrm_export.rs ─────────────────────────────────────────────────────
    {
        let mut vrm = ox::vrm_export::VrmExporter::new();
        assert!(
            vrm.set_mesh_buffers(&m).is_err(),
            "VrmExporter::set_mesh_buffers"
        );
    }

    // ── collada.rs ────────────────────────────────────────────────────────
    let collada_opts = ox::collada::ColladaExportOptions::default();
    assert!(
        ox::collada::export_collada(&m, &tmp("dae", "dae"), &collada_opts).is_err(),
        "export_collada"
    );
    assert!(
        ox::collada::export_collada_scene(&[(&m, "body")], &tmp("daescene", "dae"), &collada_opts)
            .is_err(),
        "export_collada_scene"
    );

    // ── usd.rs ────────────────────────────────────────────────────────────
    let usd_opts = ox::usd::UsdExportOptions::default();
    assert!(
        ox::usd::export_usda(&m, &tmp("usda", "usda"), &usd_opts).is_err(),
        "export_usda"
    );
    assert!(
        ox::usd::export_usda_scene(&[(&m, "body")], &tmp("usdascene", "usda"), &usd_opts).is_err(),
        "export_usda_scene"
    );

    // ── usd_anim.rs ───────────────────────────────────────────────────────
    let anim_cfg = ox::usd_anim::UsdAnimConfig::default();
    let samples = ox::usd_anim::uniform_time_samples(&m.positions, &anim_cfg);
    assert!(
        ox::usd_anim::export_usda_animated(&m, &samples, &anim_cfg, &tmp("usdanim", "usda"))
            .is_err(),
        "export_usda_animated"
    );

    // ── auto_export.rs (single gate in route_export) ──────────────────────
    for ext in ["glb", "gltf", "obj", "stl", "json"] {
        assert!(
            ox::export_auto(&m, &tmp("auto", ext)).is_err(),
            "export_auto .{ext}"
        );
    }
    assert!(
        ox::export_with_options(
            &m,
            &tmp("autoopt", "glb"),
            &ox::ExportOptions::new(ox::ExportFormat::Glb),
        )
        .is_err(),
        "export_with_options"
    );

    // ── fmt_3mf.rs ────────────────────────────────────────────────────────
    assert!(
        ox::fmt_3mf::export_3mf(&m, &ox::fmt_3mf::ThreeMfOptions::default()).is_err(),
        "export_3mf"
    );

    // ── ply.rs ────────────────────────────────────────────────────────────
    assert!(
        ox::ply::export_ply(&m, &tmp("ply", "ply"), ox::ply::PlyFormat::Ascii).is_err(),
        "export_ply"
    );
    assert!(
        ox::ply::export_mesh_as_point_cloud(&m, &tmp("plypc", "ply"), ox::ply::PlyFormat::Ascii)
            .is_err(),
        "export_mesh_as_point_cloud"
    );

    // ── fbx_binary.rs ─────────────────────────────────────────────────────
    assert!(
        ox::fbx_binary::export_mesh_fbx_binary(&m).is_err(),
        "export_mesh_fbx_binary"
    );

    // ── x3d.rs ────────────────────────────────────────────────────────────
    let x3d_opts = ox::x3d::X3dExportOptions::default();
    assert!(
        ox::x3d::export_x3d(&m, &tmp("x3d", "x3d"), &x3d_opts).is_err(),
        "export_x3d"
    );
    assert!(
        ox::x3d::export_x3d_scene(&[(&m, "body")], &tmp("x3dscene", "x3d"), &x3d_opts).is_err(),
        "export_x3d_scene"
    );
    assert!(
        ox::x3d::build_x3d(&m, &x3d_opts).is_err(),
        "build_x3d must refuse an unsuited mesh even as the in-memory XML builder"
    );
    assert!(
        ox::x3d::build_x3d_scene(&[(&m, "body")], &x3d_opts).is_err(),
        "build_x3d_scene must refuse an unsuited mesh even as the in-memory XML builder"
    );

    // ── alembic_ogawa_io.rs ───────────────────────────────────────────────
    assert!(
        ox::AlembicWriter::from_mesh_buffers(&m).is_err(),
        "AlembicWriter::from_mesh_buffers"
    );
    assert!(
        ox::AlembicWriter::from_mesh_sequence(std::slice::from_ref(&m), 24.0).is_err(),
        "AlembicWriter::from_mesh_sequence"
    );

    // ── lod_export.rs ─────────────────────────────────────────────────────
    let lod_dir = tmp("loddir", "d");
    assert!(
        ox::lod_export::export_lod_pack(
            &m,
            "nude",
            &lod_dir,
            &ox::lod_export::default_lod_levels(),
        )
        .is_err(),
        "export_lod_pack"
    );
    assert!(
        ox::lod_export::export_lod_pack_with_stats(
            &m,
            "nude",
            &lod_dir,
            &ox::lod_export::default_lod_levels(),
        )
        .is_err(),
        "export_lod_pack_with_stats"
    );

    // ── positive path: a suited mesh is accepted ──────────────────────────
    assert!(ox::glb::build_glb_bytes(&s).is_ok(), "suited glb ok");
    assert!(ox::obj::mesh_to_obj_string(&s).is_ok(), "suited obj ok");
    assert!(ox::stl::encode_stl_binary(&s).is_ok(), "suited stl ok");
    {
        let mut vrm = ox::vrm_export::VrmExporter::new();
        assert!(vrm.set_mesh_buffers(&s).is_ok(), "suited vrm ok");
    }
    assert!(
        ox::fmt_3mf::export_3mf(&s, &ox::fmt_3mf::ThreeMfOptions::default()).is_ok(),
        "suited 3mf ok"
    );
    assert!(
        ox::fbx_binary::export_mesh_fbx_binary(&s).is_ok(),
        "suited fbx ok"
    );
    assert!(
        ox::AlembicWriter::from_mesh_buffers(&s).is_ok(),
        "suited alembic ok"
    );
    assert!(
        ox::x3d::build_x3d(&s, &x3d_opts).is_ok(),
        "suited build_x3d ok"
    );
    assert!(
        ox::x3d::build_x3d_scene(&[(&s, "body")], &x3d_opts).is_ok(),
        "suited build_x3d_scene ok"
    );
}
