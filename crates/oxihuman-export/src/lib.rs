// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export pipeline for OxiHuman — 50+ geometry, animation, and texture formats.
//!
//! This crate translates [`oxihuman_mesh::MeshBuffers`] into a wide array of
//! output formats. The primary entry point for most users is
//! [`export_auto`], which infers the format from the file extension. For batch
//! pipelines use [`batch_export`] or the async-friendly [`ExportJobQueue`].
//!
//! # Supported format families
//!
//! | Family | Key functions |
//! |---|---|
//! | glTF/GLB | [`export_glb`], [`export_gltf_sep`], [`export_glb_blend_shapes`] |
//! | OBJ / MTL | [`export_obj`], [`export_obj_mtl`] |
//! | COLLADA | [`export_collada`], [`export_collada_scene`] |
//! | STL | [`export_stl_binary`], [`export_stl_ascii`] |
//! | USD / USDZ | [`export_usda`], [`package_usdz`] |
//! | Alembic | [`AlembicWriter`] (Ogawa-compatible stub) |
//! | Point cache | [`export_pc2`], [`export_mdd`], [`export_point_cache`] |
//! | VRM | [`build_vrm_extensions_json`] |
//! | 3MF | [`export_3mf`] |
//! | Streaming | [`stream_mesh_positions`] |
//!
//! # Quick start
//!
//! ```rust,no_run
//! use oxihuman_export::export_auto;
//! use oxihuman_mesh::MeshBuffers;
//! use std::path::Path;
//!
//! fn export_human(mesh: &MeshBuffers) -> anyhow::Result<()> {
//!     export_auto(mesh, Path::new("/tmp/human.glb"))
//! }
//! ```

pub mod animation;
pub mod auto_export;
pub mod blend_shapes;
pub mod glb;
pub mod gltf_sep;
pub mod instancing;
pub mod json_mesh;
pub mod lod_export;
pub mod material;
pub mod metadata;
pub mod noise_tex;
pub mod obj;
pub mod pack;
pub mod params_json;
pub mod pipeline;
pub mod scene;
pub mod scene_graph;
pub mod stl;
pub mod tex_embed;
pub mod texture;
pub mod vertex_anim;

pub use animation::{export_animation_gltf, AnimClip, AnimKeyframe};
pub use auto_export::{
    batch_export, export_auto, export_with_options, is_format_supported, supported_extensions,
    ExportFormat, ExportOptions,
};
pub use blend_shapes::{export_glb_blend_shapes, BlendShape};
pub use glb::{export_glb, export_glb_with_meta, export_glb_with_skeleton};
pub use gltf_sep::{export_gltf_sep, verify_gltf_sep};
pub use instancing::{
    circle_instances, export_instanced_glb, grid_instances, row_instances, InstanceTransform,
};
pub use json_mesh::{export_json_mesh, export_json_mesh_to_file};
pub use lod_export::{
    default_lod_levels, export_default_lod_pack, export_lod_pack, export_lod_pack_with_stats,
    LodLevel, LodLevelStats, LodPackStats,
};
pub use metadata::{MeasurementsMeta, OxiHumanMeta, ParamsMeta};
pub use noise_tex::{
    fbm, generate_fbm_texture, generate_marble_texture, generate_noise_texture,
    generate_voronoi_texture, generate_wood_texture, smootherstep, smoothstep, value_noise,
};
pub use obj::export_obj;
pub use pack::{build_pack, PackBuilderConfig, PackManifest, PackStats, TargetEntry};
pub use params_json::{export_measurements, export_mesh_measurements, export_params};
pub use pipeline::run_pipeline;
pub use scene::{export_scene_glb, Scene, SceneMesh};
pub use scene_graph::{export_scene_graph_glb, SceneGraph, SceneNode, Transform};
pub use stl::{export_stl_ascii, export_stl_binary, mesh_to_stl_ascii};
pub use tex_embed::{export_glb_with_texture, EmbeddedTexture};
pub use texture::{
    generate_checker_texture, generate_flat_normal_map, generate_gradient_texture,
    generate_skin_texture, generate_uv_texture, PixelBuffer,
};
pub use vertex_anim::{export_morph_pair_glb, export_vertex_anim_glb, AnimFrame, VertexAnimation};
pub mod ply;
pub use ply::{export_mesh_as_point_cloud, export_ply, export_point_cloud_ply, PlyFormat};
pub mod job_queue;
pub use job_queue::{ExportJob, ExportJobQueue, JobStatus, QueueResult};
pub mod csv;
pub use csv::{
    export_faces_csv, export_map_csv, export_mesh_csv, export_normals_csv, export_stats_csv,
    export_uvs_csv, export_vertices_csv, faces_to_csv_string, vertices_to_csv_string,
    CsvExportReport,
};
pub mod svg;
pub use svg::{
    build_svg, build_uv_svg, export_svg, export_uv_svg, find_silhouette_edges, project_mesh,
    SvgExportOptions, SvgExportStats, SvgProjection,
};
pub mod point_cache;
pub use point_cache::{
    cache_frame_to_positions, export_point_cache, load_point_cache, mesh_sequence_to_cache,
    validate_point_cache_file, PointCache, PointCacheHeader, OPC_MAGIC,
};
pub mod report_html;
pub use report_html::{
    export_html_report, generate_html_report, html_escape, mesh_report_from_buffers,
    mesh_summary_html, MeshReportData, PipelineReportData,
};
pub mod asset_bundle;
pub use asset_bundle::{
    bundle_from_dir, export_bundle, extract_bundle, load_bundle, validate_bundle, AssetBundle,
    BundleEntry, MAX_ENTRY_NAME, OXB_MAGIC,
};
pub mod manifest_json;
pub use manifest_json::{
    detect_format, export_manifest, file_sha256, load_manifest, manifest_from_dir, ExportManifest,
    ManifestEntry,
};
pub mod usd;
pub use usd::{
    build_usda, export_usda, export_usda_scene, format_float2_array, format_float3_array,
    format_int_array, validate_usda, UsdExportOptions, UsdExportStats,
};
pub mod tga;
pub use tga::{
    export_float_rgb_tga, export_float_rgba_tga, export_tga_rgb, export_tga_rgba, read_tga_header,
    validate_tga, TgaImage,
};
pub mod geometry_cache;
pub use geometry_cache::{
    export_geo_cache, load_geo_cache, mesh_sequence_to_geo_cache, GeoCache, GeoCacheFrame,
    GeoCacheHeader, OXGC_MAGIC, OXGC_VERSION,
};

pub mod x3d;
pub use x3d::{
    build_x3d, build_x3d_scene, export_x3d, export_x3d_scene, format_coord_array,
    format_index_array, validate_x3d, X3dExportOptions, X3dExportStats,
};
pub mod collada;
pub use collada::{
    build_collada, build_collada_scene, export_collada, export_collada_scene, format_float_array,
    format_int_array_collada, validate_collada, ColladaExportOptions, ColladaExportStats,
};
pub mod obj_mtl;
pub use obj_mtl::{
    build_mtl, build_obj_with_mtl, export_mtl, export_obj_mtl, parse_mtl_names, validate_obj,
    MtlMaterial, ObjMtlOptions, ObjMtlStats,
};
pub mod gltf_ext;
pub use gltf_ext::{
    build_materials_json, extract_extensions_used, khr_materials_clearcoat,
    khr_materials_emissive_strength, khr_materials_ior, khr_materials_sheen,
    khr_materials_specular, khr_materials_transmission, khr_materials_unlit, khr_materials_volume,
    validate_material_json, AlphaMode, ClearcoatExt, GltfMaterialDef, SheenExt, SpecularExt,
    VolumeExt,
};
pub mod morph_delta_bin;
pub use morph_delta_bin::{
    from_target_files, merge_bins, morph_delta_stats, read_morph_delta_bin,
    validate_morph_delta_bin, write_morph_delta_bin, MorphDeltaBin, MorphDeltaBinStats,
    MorphDeltaEntry, MorphDeltaTarget, OXMD_MAGIC, OXMD_VERSION,
};
pub mod variant_pack;
pub use variant_pack::{
    build_manifest, filter_variants_by_tag, find_variant_by_id,
    load_manifest as load_variant_pack_manifest, validate_pack, variant_entry, write_variant_pack,
    VariantEntry, VariantPackManifest, VariantPackResult,
};
pub mod zip_pack;
pub use zip_pack::{
    crc32, pack_mesh_assets, read_zip_entry_names, validate_zip, write_zip, zip_bytes, ZipEntry,
    ZipPackResult,
};
pub mod mesh_quantize;
pub use mesh_quantize::{
    decode_normal_oct, dequantize_mesh, encode_normal_oct, quantize_mesh, quantize_stats,
    read_quantized_bin, write_quantized_bin, QuantizeRange, QuantizeStats, QuantizedMesh,
};
pub mod pc2;
pub use pc2::{
    export_pc2, mesh_sequence_to_pc2, pc2_stats, read_pc2, write_pc2, Pc2Cache, Pc2Header,
};
pub mod mdd;
pub use mdd::{export_mdd, mdd_duration, read_mdd, uniform_time_mdd, write_mdd, MddCache};
pub mod vrm;
pub use vrm::{
    avatar_permission_str, build_vrm_extensions_json, commercial_usage_str, default_vrm_meta,
    validate_vrm_options, vrm_humanoid_to_json, vrm_meta_to_json, AvatarPermission,
    CommercialUsage, VrmExportOptions, VrmHumanoid, VrmMeta,
};
pub mod batch_pipeline;
pub use batch_pipeline::{
    batch_result_summary, estimate_batch_size, generate_param_grid, run_batch,
    specs_from_param_grid, BatchCharacterSpec, BatchConfig, BatchOutputFormat, BatchResult,
};

pub mod fmt_3mf;
pub use fmt_3mf::{
    build_3mf_model_xml, build_content_types_xml, build_rels_xml, export_3mf, mesh_is_printable,
    unit_string, validate_3mf_zip, ThreeMfExportResult, ThreeMfOptions, ThreeMfUnit,
};

pub mod streaming_export;
pub use streaming_export::{
    decode_chunk_f16, decode_chunk_f32, encode_chunk_csv, encode_chunk_f16, encode_chunk_f32,
    reassemble_chunks, stream_mesh_positions, streaming_export_stats, StreamChunk, StreamFormat,
    StreamingExportConfig, StreamingExportResult,
};

pub mod gltf_anim;
pub use gltf_anim::{
    build_gltf_accessor_json, build_gltf_anim_json, build_morph_anim_channel, clip_duration,
    export_morph_animation, lerp_weights, resample_animation, validate_morph_weights, AnimPath,
    GltfAnimChannel, GltfAnimClip, GltfAnimExportResult, MorphWeightKeyframe,
};

pub mod animated_glb;
pub use animated_glb::{
    animated_glb_stats, build_animated_glb_json, build_skeleton_json, default_t_pose_skeleton,
    generate_idle_animation, AnimatedGlbOptions, AnimatedGlbResult, JointKeyframes, SkeletonJoint,
};
pub mod usd_anim;
pub use usd_anim::{
    build_usda_animated, build_usda_time_samples_block, export_usda_animated,
    format_usda_point_array, uniform_time_samples, usd_anim_stats, UsdAnimConfig, UsdTimeSample,
};

pub mod gltf_physics;
pub use gltf_physics::{
    biped_physics_scene, build_physics_extension_json, default_rigid_body, kinematic_body,
    validate_physics_scene, GltfPhysicsScene, PhysicsJointDescriptor, PhysicsShape,
    RigidBodyDescriptor,
};
pub mod openxr_scene;
pub use openxr_scene::{
    build_xr_scene_json, default_xr_scene, validate_xr_scene, xr_projection_layer, xr_quad_layer,
    XrCompositionLayer, XrReferenceSpace, XrScene, XrSwapchain,
};
pub mod alembic_ogawa_export;
pub use alembic_ogawa_export::{
    identity_matrix, read_data_at, read_group_at, read_root_offset, scale_matrix,
    translation_matrix, unit_cube_polymesh, validate_ogawa_magic, AbcCamera, AbcObject,
    AbcObjectKind, AbcPolyMesh, AbcSubD, AbcXform, AlembicWriter,
};
pub mod alembic_stub;
pub use alembic_stub::{
    archive_to_ogawa_stub, build_animated_archive, build_single_mesh_archive, parse_ogawa_stub,
    validate_archive, AlembicArchive, AlembicObject, AlembicSample, AlembicSchema,
};
pub mod realtime_stream;
pub use realtime_stream::{
    delta_decode_positions, delta_encode_positions, dequantize_positions_16bit,
    quantize_positions_16bit, StreamCompression, StreamConfig, StreamFrame, StreamSession,
};
pub mod web_export;
pub use web_export::{
    add_lod_level, compute_web_mesh_bounds, estimate_web_size_bytes, generate_lod_levels,
    new_web_mesh, quantize_web_mesh_positions, validate_web_mesh, web_export_batch,
    web_mesh_from_json, web_mesh_to_json, WebExportOptions, WebLodLevel, WebMaterial, WebMesh,
};
pub mod texture_atlas_export;
pub use texture_atlas_export::{
    add_region, atlas_region_for_id, atlas_to_png_stub, atlas_utilization, blit_to_atlas,
    find_free_space, new_texture_atlas, pack_textures, sample_atlas, split_atlas, AtlasInput,
    AtlasRegion, TextureAtlas,
};
pub mod point_cloud_export;
pub use point_cloud_export::*;

pub mod anim_retarget_export;
pub use anim_retarget_export::*;

pub mod bone_pose_export;
pub use bone_pose_export::{
    add_bone_transform, bone_count as bone_pose_count, export_pose_to_json, new_pose_export,
    BoneTransform as BonePoseTransform, PoseExport,
};

pub mod material_library;
pub use material_library::{
    add_material, blend_materials, count_textured, default_pbr_material, deserialize_library_json,
    export_material_ids, get_material, list_names, material_is_transparent,
    material_roughness_category, new_material_library, remove_material, serialize_library_json,
    MatAlphaMode, MatLibrary, PbrMaterialDef,
};

pub mod draco_compress;
pub use draco_compress::{
    compress_mesh, compression_ratio, decode_indices_delta, default_draco_config,
    dequantize_normals, dequantize_positions, dequantize_uvs, encode_indices_delta,
    estimate_compressed_size, quantize_mesh as draco_quantize_mesh, quantize_normals,
    quantize_positions, quantize_uvs, CompressedMesh, DracoConfig, DracoQuantizedMesh,
};
pub mod svg_export;
pub use svg_export::{
    add_path as add_svg_path, default_svg_config, edges_to_svg_paths,
    find_silhouette_edges as svg_find_silhouette_edges, mesh_silhouette_svg, new_svg_document,
    path_count, positions_to_svg_path, project_to_2d as svg_project_to_2d, scale_svg, svg_bounds,
    svg_document_to_string, SvgConfig, SvgDocument, SvgPath,
};

pub mod texture_packer;
pub use texture_packer::{
    atlas_pixel_count, atlas_utilization as texture_packer_atlas_utilization, blit_texture,
    default_pack_config, find_placement, generate_solid_color_texture, next_power_of_two,
    pack_config_max_size, pack_single, pack_textures as texture_packer_pack_textures,
    rects_overlap, uv_transform_for_rect, PackConfig, PackInput, PackResult, TextureRect,
};

pub mod usdz_export;
pub use usdz_export::{
    add_usd_material, add_usd_mesh, default_usd_material, material_to_usda, mesh_to_usda,
    new_usd_scene, package_usdz, scene_mesh_count, scene_to_usda, usdz_file_size_estimate,
    usdz_magic_bytes, validate_usd_scene, UsdMaterial, UsdMesh, UsdScene, UsdzPackage,
};

pub mod animation_curve_export;
pub use animation_curve_export::{
    add_curve_to_export, add_key as add_anim_key, anim_curve_duration, anim_curve_evaluate,
    anim_curve_key_count, anim_curve_push_key, anim_curve_to_json, auto_tangents,
    curve_duration as anim_curve_duration_legacy, curve_value_range, curves_to_csv,
    evaluate_curve as evaluate_anim_curve, export_to_json as export_curves_to_json,
    flatten_tangents, merge_curve_exports, new_anim_curve, new_anim_curve_data,
    new_anim_curve_export, resample_curve, AnimCurve, AnimCurveData, AnimCurveExport, AnimCurveKey,
    BezierKey, CurveInfinity,
};

pub mod fbx_stub;
pub use fbx_stub::{
    add_fbx_mesh, add_fbx_node, export_fbx_ascii, fbx_connections, fbx_export_size_estimate,
    fbx_header, fbx_identity_matrix, fbx_mesh_to_string, fbx_node_to_string, mesh_count_fbx,
    new_fbx_scene, node_count_fbx, validate_fbx_scene, FbxExport, FbxMesh, FbxNode, FbxScene,
};

pub mod geometry_nodes_export;
pub use geometry_nodes_export::{
    add_geo_link, add_geo_node, default_output_node, export_geo_graph_json,
    export_geo_graph_python, find_output_node, geo_link_count, geo_node_count, get_geo_node,
    new_geo_graph, nodes_of_type, remove_geo_node, validate_geo_graph, GeoNode, GeoNodeGraph,
    GeoNodeLink, GeoNodeSocket, GeoNodeType,
};

pub mod haptic_export;
pub use haptic_export::{
    add_haptic_sample, add_haptic_track, average_intensity as haptic_average_intensity,
    clamp_haptic_intensities, evaluate_haptic_at, export_haptic_csv, export_haptic_json,
    haptic_export_duration, haptic_sample_count, haptic_track_count, new_haptic_export,
    peak_intensity as haptic_peak_intensity, resample_haptic_track, HapticActuator, HapticExport,
    HapticSample, HapticTrack,
};

pub mod pointcloud_viewer_export;
pub use pointcloud_viewer_export::{
    decimate_las, e57_xml_header, export_las_binary_stub, filter_las_by_classification, las_bounds,
    las_file_size_estimate, las_point_count, las_point_to_world, las_to_positions, new_e57_stub,
    new_las_header, positions_to_las, E57Stub, LasFile, LasHeader, LasPoint,
};

pub mod mesh_report;
pub use mesh_report::{
    compute_mesh_stats, count_boundary_mesh_edges, find_degenerate_faces, generate_mesh_report,
    is_watertight, mesh_health_score, mesh_surface_area, mesh_volume_signed, report_to_html,
    report_to_json, report_warnings, triangle_area, MeshReport, MeshStats,
};

pub mod screenshot_export;
pub use screenshot_export::{
    apply_gamma_correction, blend_overlay as screenshot_blend_overlay, clear_screenshot,
    crop_screenshot, default_screenshot_config, encode_ppm, encode_raw, encode_tga, flip_vertical,
    get_pixel, new_screenshot_buffer, screenshot_size_bytes, set_pixel, ScreenshotBuffer,
    ScreenshotConfig, ScreenshotFormat,
};

pub mod curve_export;
pub use curve_export::{
    add_bezier_curve, add_nurbs_curve, bezier_arc_length, bezier_curve_count,
    curve_collection_to_json, curve_collection_to_svg_paths, evaluate_bezier as curve_eval_bezier,
    evaluate_nurbs, linear_bezier, new_curve_collection, nurbs_curve_count, nurbs_default_knots,
    sample_bezier, sample_nurbs, BezierCurve, CurveCollection, NurbsCurve,
};

pub mod skeleton_export;
pub use skeleton_export::{
    add_export_bone, add_skeleton_frame, bind_pose_snapshot, bone_count_export,
    bone_world_matrix as export_bone_matrix, child_bones, frame_count as skeleton_frame_count,
    get_export_bone, new_skeleton_export, root_bones, skeleton_duration, skeleton_to_bvh_stub,
    skeleton_to_json, ExportBone, SkeletonExport,
};

pub mod export_preset;
pub use export_preset::{
    add_custom_option as preset_add_custom_option, add_preset, clone_preset, default_glb_preset,
    default_obj_preset, get_preset_by_name, new_preset_library,
    preset_count as export_preset_count, preset_library_to_json, preset_to_json,
    presets_for_target, remove_preset as remove_export_preset, set_default_preset,
    target_extension, ExportPreset, ExportTarget, PresetLibraryExport,
};

pub mod format_detect;
pub use format_detect::{
    all_3d_formats, detect_from_bytes, detect_from_extension, detect_from_path,
    extension_to_format, format_info, format_name, glb_magic, is_3d_format, is_image_format,
    is_text_format, mime_type as format_mime_type, png_magic, DetectedFormat, FormatInfo,
};

pub mod normal_map_export;
pub use normal_map_export::{
    blend_normal_maps, compute_object_space_normals, default_normal_map_config,
    encode_normal_map_ppm, flat_normal_map, get_normal_pixel, new_normal_map_buffer,
    normal_map_from_vertex_normals, normal_map_pixel_count, normal_map_size_bytes, normal_to_rgb,
    rgb_to_normal, set_normal_pixel, NormalMapBuffer, NormalMapConfig, NormalMapSpace,
};

pub mod occlusion_export;
pub use occlusion_export::{
    apply_occlusion_power, bake_ao_to_buffer, blur_occlusion_buffer, composite_ao_with_albedo,
    default_occlusion_config, encode_occlusion_ppm, fill_occlusion_buffer, get_occlusion_pixel,
    new_occlusion_buffer, occlusion_buffer_average, occlusion_map_to_rgba, occlusion_pixel_count,
    set_occlusion_pixel, OcclusionMapBuffer, OcclusionMapConfig,
};

pub mod vertex_color_export;
pub use vertex_color_export::{
    apply_gamma_correction as apply_vertex_gamma_correction, blend_vertex_colors,
    compute_ao_vertex_colors, decode_from_bytes, default_vertex_color_config, encode_to_bytes,
    fill_uniform_color, float_to_vertex_color, get_vertex_color, new_vertex_color_buffer,
    set_vertex_color, to_csv_string, vertex_color_count, vertex_color_to_float, VertexColorBuffer,
    VertexColorExportConfig, VertexColorFormat,
};

pub mod morph_export;
pub use morph_export::{
    default_morph_export_config, filter_morph_by_threshold, morph_bundle_to_json,
    morph_delta_magnitude, morph_delta_normals, morph_delta_positions, morph_export_size_bytes,
    morph_target_count, morph_target_name, morph_weight_range, new_morph_target_export,
    normalize_morph_deltas, pack_morph_bundle, MorphExportBundle, MorphExportConfig,
    MorphTargetExport,
};

pub mod weight_map_export;
pub use weight_map_export::{
    blend_weight_maps, clear_weight_map, default_weight_map_config, encode_weight_map_ppm,
    get_weight_pixel, new_weight_map_buffer, normalize_weights, set_weight_pixel, top_n_weights,
    weight_map_from_vertices, weight_map_pixel_count, weight_map_stats, weight_map_to_csv,
    BoneWeight, WeightMapBuffer, WeightMapConfig, WeightMapStats,
};

pub mod bump_map_export;
pub use bump_map_export::{
    blur_bump_map, bump_from_positions, bump_map_pixel_count, bump_map_range, bump_to_normal_map,
    clamp_bump_values, default_bump_map_config, encode_bump_map_ppm, get_bump_value,
    invert_bump_map, new_bump_map_buffer, scale_bump_values, set_bump_value, BumpMapBuffer,
    BumpMapConfig, BumpMapMode, BumpMapRange,
};

pub mod displacement_export;
pub use displacement_export::{
    compute_displacement_from_meshes, default_displacement_config, displacement_magnitude_map,
    displacement_pixel_count, displacement_stats, displacement_to_csv, encode_displacement_ppm,
    get_displacement, invert_displacement, new_displacement_buffer, remap_displacement_range,
    set_displacement, smooth_displacement, DisplacementBuffer, DisplacementConfig,
    DisplacementMode,
};

pub mod material_export;
pub use material_export::{
    add_material_to_bundle, default_material_export_config,
    default_pbr_material as default_pbr_export_material, get_property,
    material_count as material_export_count, material_property_count,
    material_to_gltf_json as material_export_to_gltf_json,
    material_to_json as material_export_to_json, new_export_material, set_property_color,
    set_property_float, set_property_texture_path, validate_material as validate_export_material,
    ExportMaterial, MaterialExportBundle, MaterialExportConfig, MaterialProperty,
};

pub mod rig_export;
pub use rig_export::{
    add_bone, bone_chain, bone_count, default_rig_export_config, find_bone_by_name, new_export_rig,
    remove_bone, rig_depth, rig_root_bones as rig_export_root_bones, rig_to_csv, rig_to_json,
    set_bone_bind_pose, total_bone_length, validate_rig, ExportRig, RigExportBone, RigExportConfig,
    RigValidationResult,
};

pub mod pose_export;
pub use pose_export::{
    add_frame, clip_to_csv as pose_clip_to_csv, clip_to_json as pose_clip_to_json,
    default_pose_export_config, frame_count, merge_clips, new_pose_clip, pose_clip_duration,
    pose_clip_fps, reverse_clip, sample_clip_at, scale_clip_timing, set_clip_fps, trim_clip,
    ExportPoseClip, ExportPoseFrame, FramePair, PoseExportConfig,
};

pub mod ao_vertex_export;
pub use ao_vertex_export::{
    ao_average as ao_vertex_average, ao_clamp, ao_invert, ao_validate as ao_vertex_validate,
    ao_value_at, ao_vertex_count, ao_vertex_to_csv, ao_vertex_to_json, new_ao_vertex_export,
    AoVertexExport,
};

pub mod bend_deform_export;
pub use bend_deform_export::{
    bend_angle_deg, bend_axis_length, bend_deform_to_json, bend_validate, default_bend_deform,
    set_bend_angle_deg, set_bend_axis, set_bend_limits, BendDeformExport,
};

pub mod blend_mask_export;
pub use blend_mask_export::{
    blend_mask_to_json, mask_active_count, mask_average_weight, mask_clamp, mask_get_weight,
    mask_invert, mask_set_weight, mask_vertex_count, new_blend_mask, BlendMaskExport,
};

pub mod camera_rig_export;
pub use camera_rig_export::{
    camera_rig_to_json, new_camera_rig_export, rig_add_keyframe, rig_clear, rig_duration,
    rig_keyframe_at, rig_keyframe_count, rig_validate, CameraRigExport, CameraRigKeyframe,
};

pub mod color_ramp_export;
pub use color_ramp_export::{
    color_ramp_to_json, new_color_ramp, ramp_add_stop, ramp_clear, ramp_evaluate, ramp_stop_count,
    ramp_validate, ColorRampExport, ColorStop,
};

pub mod constraint_target_export;
pub use constraint_target_export::{
    constraint_target_to_json, ct_influence, ct_offset_magnitude, ct_set_influence, ct_set_offset,
    ct_target_name, ct_validate, new_constraint_target, ConstraintTargetExport,
};

pub mod curve_profile_export;
pub use curve_profile_export::{
    cp_add_point, cp_arc_length, cp_bounding_box, cp_clear, cp_point_at, cp_point_count,
    cp_reverse, curve_profile_to_json, new_curve_profile, CurveProfileExport, ProfilePoint,
};

pub mod face_weight_export;
pub use face_weight_export::{
    fw_average, fw_face_count, fw_get, fw_max, fw_min, fw_normalize, fw_set, fw_to_json,
    fw_validate, new_face_weight_export, FaceWeightExport,
};

pub mod gradient_export;
pub use gradient_export::{
    grad_add_stop, grad_clear, grad_sample, grad_stop_count, grad_type_name, grad_validate,
    gradient_to_json, new_gradient, GradientExport, GradientStop, GradientType,
};

pub mod ik_chain_export;
pub use ik_chain_export::{
    ik_add_joint, ik_bone_length, ik_chain_length, ik_chain_push,
    ik_chain_spec_to_json as ik_chain_spec_json, ik_chain_to_json, ik_clear, ik_has_pole,
    ik_joint_count, ik_set_pole, ik_set_target, ik_total_length, ik_validate, new_ik_bone,
    new_ik_chain, new_ik_chain_export, IkBone, IkChain, IkChainExport, IkJointExport,
};

pub mod lattice_deform_export;
pub use lattice_deform_export::{
    lattice_get_point, lattice_point_count, lattice_resolution, lattice_set_origin,
    lattice_set_point, lattice_set_size, lattice_to_json, lattice_validate, new_lattice_deform,
    LatticeDeformExport,
};

pub mod mesh_proxy_export;
pub use mesh_proxy_export::{
    mesh_proxy2_to_json, new_mesh_proxy2, proxy2_bounds_fn, proxy2_center_fn,
    proxy2_reduction_ratio, proxy2_to_obj, proxy2_triangle_count, proxy2_validate,
    proxy2_vertex_count_fn, MeshProxyExport2,
};

pub mod morph_channel_export;
pub use morph_channel_export::{
    mc_add_channel, mc_channel_count, mc_clear, mc_find_by_name, mc_get_channel, mc_set_weight,
    mc_total_vertices, mc_validate, morph_channel_to_json, new_morph_channel_export, MorphChannel,
    MorphChannelExport,
};

pub mod uv_coord_export;
pub use uv_coord_export::{
    new_uv_coord_export, uv_coord_to_json, uvc_add, uvc_bounds, uvc_count, uvc_flip_v, uvc_get,
    uvc_normalize, uvc_validate, UvCoordExport,
};

pub mod skin_cluster_export;
pub use skin_cluster_export::{
    new_skin_cluster_export, sc_add_joint, sc_add_vertex, sc_get_influences, sc_joint_count,
    sc_max_influence_count, sc_normalize_weights, sc_to_csv, sc_validate, sc_vertex_count,
    skin_cluster_to_json, JointInfluence as ScJointInfluence, SkinClusterExport,
};

pub mod pivot_point_export;
pub use pivot_point_export::{
    new_pivot_point_export, pivot_point_to_json, pp_add, pp_add_with_orientation, pp_centroid,
    pp_clear, pp_count, pp_distance, pp_find_by_name, pp_get, pp_set_position, pp_validate,
    PivotPoint, PivotPointExport,
};

pub mod action_export;
pub use action_export::{
    action_duration, action_duration_frames, action_fcurve_count, action_push_fcurve,
    action_spec_to_json, action_to_json, add_keyframe as action_add_keyframe, clear_keyframes,
    keyframe_count as action_keyframe_count, new_action_export, new_action_export_spec,
    sample_action, validate_action, ActionExport, ActionExportSpec, ActionKeyframe,
};

pub mod blend_target_export;
pub use blend_target_export::{
    blend_target_to_json, bt_validate, bt_vertex_count, get_delta as bt_get_delta,
    max_delta_magnitude, new_blend_target, nonzero_delta_count, set_delta as bt_set_delta,
    set_weight as bt_set_weight, BlendTargetExport,
};

pub mod bone_constraint_export;
pub use bone_constraint_export::{
    bc_validate, bone_constraint_to_json, constraint_type_name, deg_to_rad, new_bone_constraint,
    rad_to_deg, set_influence as bc_set_influence, BoneConstraint, ConstraintType,
};

pub mod bone_roll_export;
pub use bone_roll_export::{
    add_bone_roll, avg_roll, bone_roll_to_json, br_bone_count, br_validate, get_roll,
    get_roll_by_name, new_bone_roll_export, normalize_roll, BoneRollExport,
};

pub mod camera_fov_export;
pub use camera_fov_export::{
    add_fov_keyframe, camera_fov_to_json, fov_duration, fov_keyframe_count, fov_to_focal_length,
    fov_to_radians, fov_validate, new_camera_fov_export, CameraFovExport, FovKeyframe,
};

pub mod cloth_weight_export;
pub use cloth_weight_export::{
    cloth_weight_to_json, cw_average, cw_count, cw_get, cw_invert, cw_pinned_count, cw_set,
    cw_validate, new_cloth_weight_export, ClothWeightExport,
};

pub mod corrective_shape_export;
pub use corrective_shape_export::{
    corrective_shape_to_json, cs_nonzero_count, cs_set_delta, cs_validate, cs_vertex_count,
    evaluate_driver, new_corrective_shape, set_driver_axis, set_driver_range,
    CorrectiveShapeExport,
};

pub mod diffuse_color_export;
pub use diffuse_color_export::{
    add_diffuse_color, dc_count, dc_validate, diffuse_bundle_to_json, get_diffuse_color,
    linear_to_srgb, new_diffuse_bundle, srgb_to_linear, DiffuseColorBundle, DiffuseColorExport,
};

pub mod edge_loop_export;
pub use edge_loop_export::{
    add_edge_loop, edge_loop_bundle_to_json, el_loop_count, el_total_vertices, el_validate,
    get_edge_loop, is_closed_loop, largest_loop_size, new_edge_loop_bundle, EdgeLoopBundle,
    EdgeLoopExport,
};

pub mod envelope_export;
pub use envelope_export::{
    add_envelope, avg_radius, env_count, env_validate, envelope_bundle_to_json, envelope_volume,
    get_envelope, new_envelope_bundle, EnvelopeBundle, EnvelopeExport,
};

pub mod face_island_export;
pub use face_island_export::{
    add_island, face_island_to_json, fi_island_count, fi_largest, fi_smallest, fi_total_faces,
    fi_validate, get_island, new_face_island_export, FaceIsland, FaceIslandExport,
};

pub mod geometry_instancing_export;
pub use geometry_instancing_export::{
    add_instance as geo_add_instance, add_instance_full, geo_instancing_to_json, gi_count,
    gi_validate, instances_of_mesh, new_geo_instancing_export, unique_mesh_count, GeoInstance,
    GeoInstancingExport,
};

pub mod hair_guide_export;
pub use hair_guide_export::{
    add_guide, avg_guide_length, guide_length, hair_guide_to_json, hg_count, hg_total_points,
    hg_validate, new_hair_guide_bundle, HairGuideBundle, HairGuideExport,
};

pub mod joint_limit_export;
pub use joint_limit_export::{
    clamp_x, is_symmetric, jl_validate, joint_limit_to_json, new_joint_limit, set_x_limits,
    set_y_limits, set_z_limits, total_range, JointLimitExport,
};

pub mod material_override_export;
pub use material_override_export::{
    add_color_override, add_float_override, add_text_override, material_override_to_json, mo_count,
    mo_validate, new_material_override_export, overrides_for_material, MaterialOverride,
    MaterialOverrideExport, OverrideValue,
};

pub mod mesh_sequence_export;
pub use mesh_sequence_export::{
    add_frame as ms_add_frame, get_frame_positions, mesh_sequence_to_json, ms_duration, ms_fps,
    ms_frame_count, ms_size_bytes, ms_validate, new_mesh_sequence, MeshFrame, MeshSequenceExport,
};

pub mod alpha_map_export;
pub use alpha_map_export::{
    alpha_map_to_json, alpha_pixel_count, average_alpha, encode_pgm as alpha_encode_pgm,
    fill_alpha, get_alpha, invert_alpha, new_alpha_map, set_alpha, validate_alpha_map,
    AlphaMapExport,
};

pub mod blend_shape_channel_export;
pub use blend_shape_channel_export::{
    channel_count_bsce, channel_export_size, channel_name_bsce, channel_to_bytes, channel_to_json,
    channel_weight, export_blend_channels, validate_channels, BlendChannel,
    BlendShapeChannelExport,
};

pub mod bone_hierarchy_export;
pub use bone_hierarchy_export::{
    add_hierarchy_bone, bone_hierarchy_count, bone_hierarchy_to_json,
    bone_length as hierarchy_bone_length, children_of, hierarchy_depth, new_bone_hierarchy,
    root_hierarchy_bones, validate_bone_hierarchy, BoneHierarchyExport, HierarchyBone,
};

pub mod camera_clip_export;
pub use camera_clip_export::{
    add_clip_keyframe, camera_clip_to_json, clip_animation_duration, clip_keyframe_count,
    clip_range, default_camera_clip, new_clip_animation, sample_clip_at as sample_camera_clip_at,
    validate_clip as validate_camera_clip, CameraClipAnimation, CameraClipExport, ClipKeyframe,
};

pub mod collision_shape_export;
pub use collision_shape_export::{
    add_collision_shape, box_collision, collision_bundle_to_json, collision_shape_count,
    new_collision_bundle, shape_volume, sphere_collision, validate_collision_bundle,
    CollisionShapeBundle, CollisionShapeExport, CollisionShapeType,
};

pub mod curve_key_export;
pub use curve_key_export::{
    add_bezier_key as add_bezier_key_ck, add_linear_key, curve_key_count, curve_key_duration,
    curve_key_to_json, curve_value_range_ck, evaluate_curve_key, new_curve_key_export, CurveKey,
    CurveKeyExport, KeyInterpolation,
};

pub mod deform_cage_export;
pub use deform_cage_export::{
    add_cage_face, add_cage_point, cage_centroid, cage_face_count, cage_point_count,
    cage_weight_sum, deform_cage_to_json, new_deform_cage, normalize_cage_weights,
    validate_deform_cage, CageControlPoint, DeformCageExport,
};

pub mod distance_field_export;
pub use distance_field_export::{
    clamp_df, count_interior_voxels, df_index, df_max_finite_value, df_min_value, df_voxel_count,
    distance_field_to_json, get_df_value, new_distance_field, set_df_value, DistanceFieldExport,
};

pub mod edge_weight_export;
pub use edge_weight_export::{
    add_weighted_edge, avg_edge_weight, count_heavy_edges, edge_weight_count, edge_weight_to_json,
    from_mesh_edges, max_edge_weight, min_edge_weight, new_edge_weight_export,
    normalize_edge_weights, EdgeWeightExport, WeightedEdge,
};

pub mod face_normal_export;
pub use face_normal_export::{
    avg_face_normal, compute_face_normal_fn, export_face_normals, face_normal_to_json,
    face_normals_to_csv, fn_face_count, get_face_normal, validate_face_normals, FaceNormalExport,
};

pub mod geometry_cache_v2_export;
pub use geometry_cache_v2_export::{
    add_geo_v2_frame, geo_cache_v2_to_json, geo_v2_duration, geo_v2_frame_count,
    geo_v2_header_bytes, geo_v2_size_bytes, new_geo_cache_v2, validate_geo_cache_v2,
    GeoCacheV2Export, GeoCacheV2Frame, GCV2_MAGIC, GCV2_VERSION,
};

pub mod hair_length_export;
pub use hair_length_export::{
    avg_hair_length, count_long_strands, hair_length_to_csv, hair_length_to_json,
    hair_strand_count, max_hair_length, min_hair_length, new_hair_length_export,
    scale_hair_lengths, validate_hair_lengths, HairLengthExport,
};

pub mod joint_weight_export;
pub use joint_weight_export::{
    add_joint_influence, joint_weight_to_json, jw_max_influences, jw_vertex_count,
    new_joint_weight_export, normalize_joint_weights, skinned_vertex_count, to_flat_arrays,
    validate_joint_weights, JointInfluence as JwJointInfluence, JointWeightExport,
};

pub mod material_texture_export;
pub use material_texture_export::{
    export_material_textures, slot_name, slot_texture_path, slot_to_json, slot_uv_channel,
    texture_export_size_mt, texture_slot_count, validate_material_textures, MaterialTextureExport,
    TextureSlot,
};

pub mod mesh_topology_export;
pub use mesh_topology_export::{
    export_mesh_topology, topology_edge_count, topology_export_size, topology_face_count,
    topology_is_manifold, topology_to_json, topology_vertex_count, validate_topology,
    MeshTopologyExport,
};

pub mod morph_target_export;
pub use morph_target_export::{
    export_morph_targets, morph_target_count_export, morph_target_delta_count,
    morph_target_export_size, morph_target_name as mt_morph_target_name, morph_target_to_bytes,
    morph_target_to_json, validate_morph_target_export, MorphTarget as MtMorphTarget,
    MorphTargetExport as MtMorphTargetExport,
};

pub mod animation_layer_export;
pub use animation_layer_export::{
    add_anim_layer, anim_layer_count, anim_layer_to_json, blend_mode_name, enabled_layer_count,
    find_layer_by_name, new_anim_layer_export, total_enabled_weight, validate_anim_layers,
    AnimLayer, AnimLayerExport, LayerBlendMode,
};

pub mod blend_tree_node_export;
pub use blend_tree_node_export::{
    add_blend_node, blend_node_count, blend_tree_to_json, find_blend_node, new_blend_tree_export,
    nodes_of_type as btn_nodes_of_type, set_blend_root, total_connections, validate_blend_tree,
    BlendNodeType, BlendTreeExport, BlendTreeNode,
};

pub mod bone_bind_pose_export;
pub use bone_bind_pose_export::{
    add_bind_pose_bone, all_inverse_binds_set, bind_bone_count, bind_pose_to_flat,
    bind_pose_to_json, find_bind_bone, new_bind_pose_export, set_local_matrix, validate_bind_pose,
    BindPoseExport, BoneBindPose, Mat4,
};

pub mod camera_track_export;
pub use camera_track_export::{
    add_camera_keyframe, camera_keyframe_count, camera_track_duration, camera_track_to_json,
    new_camera_track, sample_camera_position, validate_camera_track, CameraKeyframe,
    CameraTrackExport,
};

pub mod cloth_mesh_export;
pub use cloth_mesh_export::{
    cloth_mesh_to_json, cloth_vertex_count as cm_vertex_count, free_count as cloth_free_count,
    new_cloth_mesh_export, pin_vertex, pinned_count, set_uniform_drag, total_mass, unpin_vertex,
    validate_cloth_mesh, ClothMeshExport, ClothVertex,
};

pub mod collision_margin_export;
pub use collision_margin_export::{
    add_margin_entry, avg_margin, collision_margin_to_json, find_margin_entry, margin_entry_count,
    max_margin as collision_max_margin, new_collision_margin_export, shape_type_name,
    validate_margins, CollisionMarginEntry, CollisionMarginExport, MarginShapeType,
};

pub mod custom_attr_export;
pub use custom_attr_export::{
    add_attr, attr_count, custom_attr_to_json, find_attr, get_bool as custom_get_bool,
    get_float as custom_get_float, get_int as custom_get_int, new_custom_attr_export, remove_attr,
    validate_custom_attrs, AttrValue, CustomAttr, CustomAttrExport,
};

pub mod edge_crease_export_v2;
pub use edge_crease_export_v2::{
    add_crease_v2, crease_count_v2, edge_crease_v2_to_json, get_sharpness_v2, make_crease_key,
    max_sharpness_v2, new_edge_crease_export_v2, validate_creases_v2, CreaseEdgeKey, CreaseEntryV2,
    EdgeCreaseExportV2,
};

pub mod face_smooth_export;
pub use face_smooth_export::{
    count_in_group, distinct_group_count as face_smooth_distinct_groups, face_smooth_face_count,
    face_smooth_to_json, flat_face_count, get_smooth_group, new_face_smooth_export, set_flat,
    set_smooth_group, smooth_face_count, FaceSmoothExport,
};

pub mod fluid_velocity_export;
pub use fluid_velocity_export::{
    add_fluid_particle, avg_density as fluid_avg_density, avg_speed, export_positions_flat,
    export_velocities_flat, fluid_particle_count, fluid_velocity_to_json, max_speed,
    new_fluid_velocity_export, FluidParticle, FluidVelocityExport,
};

pub mod geometry_delta_export;
pub use geometry_delta_export::{
    add_normal_deltas, avg_delta_magnitude, compute_geometry_delta, delta_sparsity,
    delta_vertex_count as geo_delta_vertex_count, geometry_delta_to_json,
    max_delta_magnitude as geo_max_delta_magnitude, scale_delta, GeometryDelta,
};

pub mod hair_sim_export;
pub use hair_sim_export::{
    add_hair_strand, avg_stiffness, avg_strand_length_sim, hair_sim_to_json, hair_strand_count_sim,
    new_hair_sim_export, total_sim_points, validate_hair_sim, HairSimExport, HairSimStrand,
};

pub mod ik_pole_export;
pub use ik_pole_export::{
    add_ik_pole, avg_pole_influence, find_ik_pole, ik_pole_count, ik_pole_to_json,
    local_space_pole_count, new_ik_pole_export, validate_ik_poles, IkPoleExport, IkPoleTarget,
};

pub mod joint_name_export;
pub use joint_name_export::{
    add_joint_name, children_of_joint, find_joint_by_index, find_joint_by_name, joint_name_count,
    joint_name_to_json, new_joint_name_export, root_joints, validate_joint_names, JointNameEntry,
    JointNameExport,
};

pub mod lattice_point_export;
pub use lattice_point_export::{
    displace_lattice_point, find_lattice_point, lattice_point_count_lp, lattice_point_to_json,
    max_lattice_displacement, new_lattice_point_export, validate_lattice_points, LatticePoint,
    LatticePointExport,
};

pub mod mesh_delta_export;
pub use mesh_delta_export::{
    add_vertex_delta, apply_mesh_delta, delta_entry_count, mesh_delta_max_magnitude,
    mesh_delta_sparsity, mesh_delta_to_json, new_mesh_delta_export, validate_mesh_delta,
    MeshDeltaExport, VertexDeltaEntry,
};

pub mod blend_shape_driver_export;
pub use blend_shape_driver_export::{
    add_driver as bsd_add_driver, axis_name as bsd_axis_name, blend_shape_driver_export_to_json,
    driver_count as bsd_driver_count, driver_to_json, evaluate_driver as bsd_evaluate_driver,
    find_driver_by_name, new_blend_shape_driver_export, total_driven_shapes,
    validate_driver as bsd_validate_driver, BlendShapeDriver, BlendShapeDriverExport, DrivenShape,
    DriverAxis,
};

pub mod bone_axis_export;
pub use bone_axis_export::{
    add_bone_axis, axes_are_orthonormal, bone_axis_count, bone_axis_export_to_json,
    bone_axis_to_json, bone_length as bae_bone_length, find_bone_axis, new_bone_axis_export,
    total_bone_length as bae_total_length, validate_bone_axes, BoneAxis, BoneAxisExport,
};

pub mod camera_dof_export;
pub use camera_dof_export::{
    add_dof_keyframe, camera_dof_to_json, circle_of_confusion, default_camera_dof,
    dof_animation_duration, dof_keyframe_count, dof_range, new_dof_animation, sample_dof_at,
    validate_dof, CameraDofAnimation, CameraDofExport, DofKeyframe,
};

pub mod capsule_export;
pub use capsule_export::{
    add_capsule as add_capsule_export, capsule_bundle_to_json,
    capsule_count as capsule_export_count, capsule_surface_area, capsule_to_json,
    capsule_total_length, capsule_volume, default_capsule, find_capsule_by_name,
    new_capsule_bundle, total_capsule_volume, validate_capsule, CapsuleBundle, CapsuleExport,
};

pub mod collision_box_export;
pub use collision_box_export::{
    add_collision_box, box_surface_area as collision_box_surface_area,
    box_volume as collision_box_volume, collision_box_bundle_to_json, collision_box_count,
    collision_box_to_json, default_collision_box, find_box_by_name, new_collision_box_bundle,
    point_in_box, total_box_volume, validate_collision_box, CollisionBox, CollisionBoxBundle,
};

pub mod curve_tangent_export;
pub use curve_tangent_export::{
    add_key as curve_tangent_add_key, auto_tangents as curve_auto_tangents,
    curve_duration as curve_tangent_duration, curve_tangent_to_json, evaluate_curve_tangent,
    flatten_all_tangents, new_curve_tangent_export, value_range as curve_tangent_value_range,
    CurveTangentExport, CurveTangentKey,
};

pub mod deform_region_export;
pub use deform_region_export::{
    add_region as deform_add_region, deform_region_export_to_json, falloff_name,
    find_region as find_deform_region, new_deform_region_export, normalize_region_weights,
    region_avg_weight, region_count as deform_region_count, region_max_weight, region_to_json,
    total_vertex_count as deform_total_vertex_count, validate_region, DeformRegion,
    DeformRegionExport, FalloffType,
};

pub mod dynamic_bone_export;
pub use dynamic_bone_export::{
    add_dynamic_bone, bones_with_colliders, default_dynamic_bone, dynamic_bone_count,
    dynamic_bone_export_to_json, dynamic_bone_to_json, find_dynamic_bone, new_dynamic_bone_export,
    simulate_gravity_offset, total_colliders as dynamic_total_colliders, validate_dynamic_bone,
    DynamicBone, DynamicBoneExport,
};

pub mod edge_select_export;
pub use edge_select_export::{
    add_selected_edge, average_crease_value, build_edge_index as edge_select_build_index,
    crease_count as edge_crease_count, edge_select_to_json, has_edge, new_edge_select_export,
    seam_count as edge_seam_count, selected_edge_count, sharp_count as edge_sharp_count,
    validate_edge_export, EdgeFlags, EdgeSelectExport, SelectedEdge,
};

pub mod face_corner_export;
pub use face_corner_export::{
    add_face_corner, avg_uv as face_corner_avg_uv, build_triangle_export, corner_count,
    corners_for_face, face_corner_to_json, new_face_corner_export,
    normals_are_unit as face_corner_normals_unit, uv_bounds as face_corner_uv_bounds,
    validate_face_corners, FaceCorner, FaceCornerExport,
};

pub mod geometry_scatter_export;
pub use geometry_scatter_export::{
    add_instance_name, add_scatter_point, avg_scale as scatter_avg_scale, grid_scatter,
    instance_type_count, new_geometry_scatter_export, points_of_instance, scatter_bounds,
    scatter_point_count, scatter_to_json, validate_scatter, GeometryScatterExport, ScatterPoint,
};

pub mod hair_root_export;
pub use hair_root_export::{
    add_group as hair_add_group, add_hair_root, avg_hair_length as hair_avg_length,
    group_count as hair_group_count, hair_root_bounds, hair_root_to_csv, hair_root_to_json,
    max_hair_length as hair_max_length, min_hair_length as hair_min_length, new_hair_root_export,
    root_count, roots_in_group, validate_hair_roots, HairRoot, HairRootExport,
};

pub mod ik_solver_export;
pub use ik_solver_export::{
    add_solver, default_ik_solver, find_solver, ik_bundle_to_json, ik_solver_to_json,
    new_ik_solver_bundle, solver_count, solver_type_name, solvers_with_pole, total_ik_joints,
    validate_ik_solver, IkJointEntry, IkSolverBundle, IkSolverExport, IkSolverType,
};

pub mod joint_orient_v2_export;
pub use joint_orient_v2_export::{
    add_joint_orient, find_joint_orient, identity_joint_orient, joint_orient_count,
    joint_orient_export_to_json, joint_orient_to_json, new_joint_orient_v2_export,
    quaternion_is_unit as joint_quat_is_unit, rot_order_name, validate_joint_orients,
    JointOrientV2, JointOrientV2Export, RotOrder,
};

pub mod keyframe_ease_export;
pub use keyframe_ease_export::{
    add_eased_key, apply_ease, ease_type_name, eased_key_count, evaluate_eased_curve,
    keyframe_ease_to_json, new_keyframe_ease_export, validate_ease_export, EaseType, EasedKeyframe,
    KeyframeEaseExport,
};

pub mod light_probe_export;
pub use light_probe_export::{
    add_probe, default_irradiance_probe, eval_sh_l1, find_probe, light_probe_export_to_json,
    new_light_probe_export, probe_count, probe_grid, probe_to_json, total_coverage_area,
    validate_probe, LightProbe, LightProbeExport, ProbeType,
};

pub mod anim_clip_blend_export;
pub use anim_clip_blend_export::{
    add_blend_clip, anim_clip_blend_to_json, blend_clip_count, find_blend_clip, max_clip_duration,
    new_anim_clip_blend_export, normalize_blend_weights, total_blend_weight, validate_blend_clips,
    AnimClipBlendExport, BlendClipEntry,
};

pub mod blend_shape_inbetween_export;
pub use blend_shape_inbetween_export::{
    add_inbetween_key, bsi_to_json, find_inbetween, inbetween_key_count, interpolate_inbetween,
    max_delta_magnitude_bsi, new_bsi_export, total_inbetween_deltas, BlendShapeInbetweenExport,
    InbetweenKey,
};

pub mod bone_custom_prop_export;
pub use bone_custom_prop_export::{
    add_bool_prop, add_float_prop, add_text_prop, bone_custom_prop_to_json, bone_names_with_props,
    find_prop as bone_find_prop, new_bone_custom_prop_export, prop_count, props_for_bone,
    BoneCustomProp, BoneCustomPropExport, BonePropValue,
};

pub mod camera_ortho_export;
pub use camera_ortho_export::{
    camera_ortho_to_json, default_ortho_camera, ortho_aspect_ratio, ortho_pixel_count,
    ortho_project_point, ortho_projection_matrix, ortho_resize, point_in_ortho_frustum,
    validate_ortho_camera, CameraOrthoExport,
};

pub mod cloth_sim_state_export;
pub use cloth_sim_state_export::{
    add_cloth_particle_state, avg_speed_css, cloth_sim_state_to_json, flat_positions_css,
    max_speed_css, new_cloth_sim_state, particle_count_css, pinned_count_css,
    validate_cloth_sim_state, ClothParticleState, ClothSimStateExport,
};

pub mod collision_capsule_export;
pub use collision_capsule_export::{
    add_collision_capsule, avg_radius_cc, capsule_count_cc, capsule_height, capsule_volume_cc,
    collision_capsule_to_json, find_capsule_cc, new_collision_capsule_export,
    total_capsule_volume_cc, validate_capsules, CollisionCapsule, CollisionCapsuleExport,
};

pub mod curve_nurbs_export;
pub use curve_nurbs_export::{
    add_nurbs_control_point, avg_weight as nurbs_avg_weight, control_point_count_nurbs,
    generate_uniform_knots, linear_nurbs_curve, new_nurbs_curve_export, nurbs_centroid,
    nurbs_to_json, validate_nurbs, NurbsCurveExport,
};

pub mod deform_stack_export;
pub use deform_stack_export::{
    add_deformer, deform_stack_to_json, deformer_count, deformers_of_type, enabled_deformer_count,
    find_deformer as find_deform_stack_entry, new_deform_stack, toggle_deformer, DeformStackExport,
    DeformerEntry, DeformerType,
};

pub mod edge_bevel_export;
pub use edge_bevel_export::{
    add_bevel_edge, avg_bevel_width, bevel_edge_count, edge_bevel_to_json, find_bevel_edge,
    max_bevel_width, new_edge_bevel_export, total_bevel_faces, validate_bevel_export, BevelEdge,
    EdgeBevelExport,
};

pub mod face_vertex_export;
pub use face_vertex_export::{
    face_vertex_to_json, fv_add_face, fv_add_vertex, fv_avg_face_size, fv_face_count,
    fv_indices_valid, fv_normals_unit, fv_to_triangles, fv_vertex_count, FaceVertexExport,
};

pub mod geo_modifier_export;
pub use geo_modifier_export::{
    add_geo_modifier, find_geo_modifier, geo_mod_count, geo_mod_enabled_count,
    geo_modifier_to_json, mods_of_type, new_geo_modifier_export, realtime_mod_count, GeoModEntry,
    GeoModType, GeoModifierExport,
};

pub mod hair_clump_export;
pub use hair_clump_export::{
    add_hair_clump, avg_clump_factor, clump_roots_bounds, hair_clump_count, hair_clump_to_json,
    largest_clump, new_hair_clump_export, total_strand_count_hc, validate_clump_factors, HairClump,
    HairClumpExport,
};

pub mod ik_target_export;
pub use ik_target_export::{
    add_ik_target, avg_chain_length as ik_avg_chain_length, find_ik_target, ik_target_count,
    ik_target_to_json, new_ik_target_bundle, set_pole_target, targets_with_pole,
    validate_ik_targets, IkTargetBundle, IkTargetExport,
};

pub mod joint_twist_export;
pub use joint_twist_export::{
    add_joint_twist, avg_twist_deg, find_joint_twist, joint_twist_count, joint_twist_to_json,
    max_twist_deg, new_joint_twist_export, twist_rad, validate_twist_axes, JointTwistEntry,
    JointTwistExport,
};

pub mod keyshape_export;
pub use keyshape_export::{
    add_keyshape, blend_keyshapes, find_keyshape, keyshape_count, keyshape_to_json,
    max_keyshape_delta, new_keyshape_export, total_keyshape_deltas, validate_keyshape_weights,
    KeyShape, KeyShapeExport,
};

pub mod lod_bias_export;
pub use lod_bias_export::{
    add_lod_bias, avg_lod_bias, find_lod_bias, high_bias_entries, lod_bias_count, lod_bias_to_json,
    max_lod_level, new_lod_bias_export, validate_lod_bias, LodBiasEntry, LodBiasExport,
};

pub mod anim_event_export;
pub use anim_event_export::{
    add_event as add_anim_event, event_count, events_from, has_event_named, new_event_track,
    serialize_events, sort_events, track_duration, trim_before, AnimEvent, AnimEventTrack,
};

pub mod blend_pose_export;
pub use blend_pose_export::{
    active_pose_count, blend_poses, identity_pose, normalise_quat, serialise_blended_pose,
    weights_normalised as blend_weights_normalised, BlendedPose, Pose, WeightedPose,
};

pub mod bone_envelope_export;
pub use bone_envelope_export::{
    add_envelope as add_bone_envelope, envelope_length, find_envelope, new_envelope_set,
    point_in_envelope, serialise_envelopes, BoneEnvelope, EnvelopeSet,
};

pub mod camera_shake_export;
pub use camera_shake_export::{
    add_shake_keyframe, channel_keyframe_count, evaluate_shake, is_decayed, new_camera_shake,
    peak_amplitude, serialise_shake, CameraShakeExport, ShakeChannel, ShakeKeyframe,
};

pub mod cloth_pressure_export;
pub use cloth_pressure_export::{
    compute_pressure_forces, compute_signed_volume, is_outward_pressure, max_force_magnitude,
    scale_forces, serialise_pressure_config, ClothPressureConfig, PressureForceCache,
};

pub mod collision_compound_export;
pub use collision_compound_export::{
    aabb_of_centers, add_box as compound_add_box, add_capsule as compound_add_capsule,
    add_sphere as compound_add_sphere, compound_volume, new_compound, primitive_count,
    serialise_compound, CollisionPrimitive, CompoundCollision,
};

pub mod curve_control_export;
pub use curve_control_export::{
    add_control_point, control_point_aabb, control_polygon_length, has_enough_points,
    new_curve_export, reverse_curve, serialise_curve as serialise_curve_export, ControlPoint,
    CurveControlExport, CurveType,
};

pub mod deform_weights_export;
pub use deform_weights_export::{
    add_weight_map as add_deform_weight_map, clamp_all_weights, find_map as find_deform_map,
    new_deform_weights_export, normalise_across_deformers, serialise_deform_weights, vertex_weight,
    DeformWeightsExport, DeformerWeightMap,
};

pub mod edge_smooth_export;
pub use edge_smooth_export::{
    add_edge as add_smooth_edge, edge_flag, from_triangles_all_smooth, hard_count, mark_all_hard,
    new_edge_smooth_export, serialise_edge_smooth, smooth_count, smooth_fraction, EdgeSmooth,
    EdgeSmoothExport,
};

pub mod face_color_export;
pub use face_color_export::{
    average_color, color_to_u8, fill_all as fill_all_face_colors, get_face_color,
    new_face_color_export, opaque_face_count, serialise_face_colors, set_face_color, FaceColor,
    FaceColorExport,
};

pub mod geo_warp_export;
pub use geo_warp_export::{
    add_warp_keyframe, interpolate_warp, max_displacement as warp_max_displacement, new_geo_warp,
    serialise_keyframe as serialise_warp_keyframe, warp_duration, zero_displacement, GeoWarpExport,
    WarpKeyframe,
};

pub mod hair_style_export;
pub use hair_style_export::{
    add_style, all_lengths_positive, average_length as hair_average_length, find_style,
    is_straight, new_hair_style_library, scale_lengths, serialise_style, HairStyle,
    HairStyleLibrary,
};

pub mod ik_fk_blend_export;
pub use ik_fk_blend_export::{
    add_record as add_ik_fk_record, all_weights_valid as ik_fk_all_weights_valid,
    average_ik_weight, find_record as find_ik_fk_record, ik_dominant_count, ik_weight,
    new_ik_fk_blend_export, serialise_ik_fk, set_blend, BlendMode, IkFkBlendExport,
    IkFkBlendRecord,
};

pub mod joint_parent_export;
pub use joint_parent_export::{
    add_joint, joint_count, joint_depth, new_joint_parent_export, serialise_parents,
    world_position, JointParentExport, JointRecord,
};

pub mod key_driver_export;
pub use key_driver_export::{
    add_driver, driver_count, drivers_for_shape, evaluate_curve as evaluate_driver_curve,
    evaluate_driver as evaluate_key_driver, names_unique, new_key_driver_export,
    serialise_curve as serialise_driver_curve, DriverCurve, KeyDriver, KeyDriverExport,
};

pub mod lod_group_export;
pub use lod_group_export::{
    add_lod_level as lg_add_lod_level, export_lod_group_to_json, level_count, new_lod_group_export,
    LodGroupExport, LodLevel as LgLodLevel,
};

pub mod anim_notify_export;
pub use anim_notify_export::{
    add_notify, clear_notifies, find_notify, new_notify_track, notifies_in_range, notify_count,
    notify_track_to_json, sort_notifies, track_duration as notify_track_duration, AnimNotify,
    AnimNotifyTrack,
};

pub mod blend_weight_export_v2;
pub use blend_weight_export_v2::{
    active_weight_count, add_blend_weight_v2, blend_weight_v2_to_json, find_weight_v2,
    new_blend_weight_export_v2, normalize_v2_weights, set_weight_v2, total_weight_sum,
    weight_count_v2, weights_all_valid, BlendWeightExportV2, BlendWeightV2,
};

pub mod bone_length_export;
pub use bone_length_export::{
    add_bone_length, bone_count_ble, bone_length, bone_length_to_json, bone_lengths_to_csv,
    find_bone_ble, max_bone_length, min_bone_length, new_bone_length_export, total_bone_length_ble,
    BoneLengthEntry, BoneLengthExport,
};

pub mod camera_stereo_export;
pub use camera_stereo_export::{
    camera_stereo_to_json, default_stereo_camera, parallax_angle_deg, stereo_fov_radians,
    stereo_left_offset, stereo_mode_name, stereo_right_offset, validate_stereo, CameraStereoExport,
    StereoMode,
};

pub mod cloth_pin_export;
pub use cloth_pin_export::{
    add_pin, add_pin_at, avg_pin_strength, cloth_pin_to_json, is_pinned, new_cloth_pin_export,
    pin_count, pins_with_position, remove_pin, validate_pins, ClothPin, ClothPinExport,
};

pub mod collision_sphere_export;
pub use collision_sphere_export::{
    add_sphere as add_collision_sphere, avg_sphere_radius, collision_sphere_to_json,
    find_sphere as find_collision_sphere, new_collision_sphere_export, point_in_sphere,
    sphere_count, sphere_surface_area, sphere_volume, total_sphere_volume, validate_spheres,
    CollisionSphere, CollisionSphereExport,
};

pub mod curve_modifier_export;
pub use curve_modifier_export::{
    add_curve_mod, axis_name as curve_mod_axis_name, clear_curve_mods, curve_mod_length,
    curve_modifier_to_json, mod_count, new_curve_modifier_export, total_curve_length,
    validate_curve_mod, CurveModAxis, CurveModEntry, CurveModifierExport,
};

pub mod deform_bind_export;
pub use deform_bind_export::{
    add_binding, binding_avg_weight, binding_count, binding_is_valid, deform_bind_to_json,
    find_binding, new_deform_bind_export, normalize_binding_weights, total_bound_vertices,
    DeformBindEntry, DeformBindExport,
};

pub mod edge_mark_export;
pub use edge_mark_export::{
    add_marked_edge, avg_crease_value as edge_mark_avg_crease, crease_edge_count_em,
    edge_mark_to_json, marked_edge_count, new_edge_mark_export, seam_edge_count_em,
    set_crease_value as set_edge_crease_value, sharp_edge_count_em, EdgeFlags as MarkEdgeFlags,
    EdgeMarkExport, MarkedEdge,
};

pub mod face_corner_uv_export;
pub use face_corner_uv_export::{
    add_corner_uv, avg_uv as face_corner_avg_uv_export, corner_count as fcuv_corner_count,
    corners_for_face as face_corners_for_face, face_corner_uv_to_json, face_count_fcuv,
    new_face_corner_uv_export, uv_bounds as face_corner_uv_bounds_export,
    uvs_in_unit_range as face_corner_uvs_in_range, FaceCornerUv, FaceCornerUvExport,
};

pub mod geo_instance_export;
pub use geo_instance_export::{
    add_instance_entry, clear_instances, geo_instance_to_json, instance_bounds, instance_count,
    instances_of_mesh as instances_of_mesh_export, new_geo_instance_set, unique_mesh_names,
    validate_instances, GeoInstanceEntry, GeoInstanceSetExport,
};

pub mod hair_width_export;
pub use hair_width_export::{
    add_strand_widths, avg_width, hair_width_to_json, max_width as hair_max_width,
    min_width as hair_min_width, new_hair_width_export, scale_widths, strand_count_hw,
    total_width_points, widths_positive, HairWidthExport, HairWidthStrand,
};

pub mod ik_weight_export;
pub use ik_weight_export::{
    add_ik_weight, avg_ik_weight, entry_count_ikw, find_ik_entry, fully_ik_joints,
    ik_weight_to_json, new_ik_weight_export, normalize_ik_fk, set_ik_fk_blend, weights_sum_to_one,
    IkWeightEntry, IkWeightExport,
};

pub mod joint_scale_export;
pub use joint_scale_export::{
    add_joint_scale, avg_scale_magnitude, find_joint_scale, is_uniform_scale, joint_scale_count,
    joint_scale_to_json, new_joint_scale_export, scales_positive, set_scale as set_joint_scale,
    uniform_scale_count, JointScaleEntry, JointScaleExport,
};

pub mod keyframe_blend_export;
pub use keyframe_blend_export::{
    add_blend_key, blend_mode_name as keyframe_blend_mode_name, channel_duration, key_count_kbe,
    keyframe_blend_to_json, keys_of_mode, new_keyframe_blend_export,
    sample_linear as sample_keyframe_linear, sort_keys_by_time, KeyBlendMode, KeyframeBlendEntry,
    KeyframeBlendExport,
};

pub mod lod_mesh_export;
pub use lod_mesh_export::{
    add_lod_mesh_level, find_lod_level, lod_level_count, lod_levels_sorted, lod_mesh_to_json,
    new_lod_mesh_export, reduction_ratio, sort_lod_levels, total_triangle_count, LodMeshExport,
    LodMeshLevel,
};

pub mod blend_corrective_export;
pub use blend_corrective_export::{
    add_corrective_blend, corrective_blend_count, corrective_blend_to_json, find_corrective_blend,
    max_corrective_delta, new_corrective_blend_bundle, shapes_for_bone, validate_corrective_bundle,
    CorrectiveBlend, CorrectiveBlendBundle,
};

pub mod camera_path_export_v2;
pub use camera_path_export_v2::{
    add_cam_path_key_v2, cam_path_v2_duration, cam_path_v2_key_count, cam_path_v2_position_at,
    cam_path_v2_to_json, new_camera_path_v2, validate_cam_path_v2, CamPathKeyV2, CameraPathV2,
};

pub mod cloth_stiffness_export;
pub use cloth_stiffness_export::{
    add_cloth_stiffness, avg_stiffness_value, clamp_stiffness, cloth_stiffness_count,
    cloth_stiffness_to_json, count_stiffness_type, new_cloth_stiffness_export, validate_stiffness,
    ClothStiffnessEntry, ClothStiffnessExport, StiffnessType,
};

pub mod collision_triangle_export;
pub use collision_triangle_export::{
    collision_tri_count, collision_triangle_to_json, from_mesh as collision_from_mesh,
    total_collision_area, triangle_area_ct, triangles_for_material, CollisionTriangle,
    CollisionTriangleExport,
};

pub mod curve_bezier_export;
pub use curve_bezier_export::{
    add_bezier_cp, bezier_arc_length_approx, bezier_cp_count, bezier_curve_to_json, deg_to_rad_bc,
    eval_bezier_segment, new_bezier_curve_export, sample_bezier_curve, BezierControlPoint,
    BezierCurveExport,
};

pub mod deform_lattice_export;
pub use deform_lattice_export::{
    avg_lattice_displacement_v2, deform_lattice_to_json, lattice_control_point_count,
    lattice_displacement_at, new_deform_lattice, set_lattice_deformed, validate_deform_lattice,
    DeformLatticeExport, LatticeResV2,
};

pub mod edge_normal_export;
pub use edge_normal_export::{
    add_edge_normal, avg_edge_normal, compute_from_mesh as compute_edge_normals_from_mesh,
    edge_normal_count, edge_normal_to_json, find_edge_normal, new_edge_normal_export, normalize_en,
    normals_unit_en, EdgeNormalEntry, EdgeNormalExport,
};

pub mod face_tangent_export;
pub use face_tangent_export::{
    compute_face_tangents, cross_ft, face_tangent_count, face_tangent_to_json, normalize_ft,
    tangents_unit, FaceTangent, FaceTangentExport,
};

pub mod geo_point_export;
pub use geo_point_export::{
    add_geo_point, avg_geo_point_scale, geo_point_bounds, geo_point_count, geo_point_to_json,
    new_geo_point_export, points_with_label_gp, validate_geo_points, GeoPoint, GeoPointExport,
};

pub mod hair_density_export;
pub use hair_density_export::{
    add_hair_density, avg_hair_density, density_for_face, hair_density_count, hair_density_to_json,
    new_hair_density_export, scale_hair_densities, validate_hair_densities, HairDensityEntry,
    HairDensityExport,
};

pub mod ik_constraint_export;
pub use ik_constraint_export::{
    add_ik_constraint, avg_chain_length_ik, clamp_ik_weights, count_constraint_type,
    find_constraint_by_bone, ik_constraint_count, ik_constraint_to_json, new_ik_constraint_export,
    validate_ik_weights, IkConstraintEntry, IkConstraintExport, IkConstraintType,
};

pub mod joint_space_export;
pub use joint_space_export::{
    add_joint_transform, avg_translation_magnitude, find_joint_transform, identity_joint_transform,
    joint_space_to_json, joint_transform_count, new_joint_space_export, quaternions_unit,
    scales_positive_js, JointSpaceExport, JointSpaceTransform,
};

pub mod keyframe_set_export;
pub use keyframe_set_export::{
    add_channel, channel_count_ks, find_channel_ks, keyframe_set_duration, keyframe_set_to_json,
    new_keyframe_set_export, sample_channel_ks, total_keyframe_count, KeyframeChannel,
    KeyframeSetExport, KeyframeValue,
};

pub mod lod_switch_export;
pub use lod_switch_export::{
    active_lod_for_distance, add_lod_switch, lod_switch_count, lod_switch_reduction_ratio,
    lod_switch_to_json, new_lod_switch_export, total_triangle_count_ls, validate_lod_switch,
    LodSwitchEntry, LodSwitchExport,
};

pub mod toml_export;
pub use toml_export::{
    add_bool as add_toml_bool, add_float as add_toml_float, add_integer as add_toml_integer,
    add_string as add_toml_string, export_mesh_stats_toml, find_record as find_toml_record,
    new_toml_export, record_count, to_toml_string, TomlExport, TomlRecord,
};

pub mod yaml_export;
pub use yaml_export::{
    export_yaml, render_yaml, scene_to_yaml, validate_yaml_value, yaml_bool,
    yaml_float as yaml_export_float, yaml_int, yaml_list, yaml_list_len, yaml_map, yaml_map_get,
    yaml_null, yaml_size_bytes, yaml_str, YamlValue,
};

pub mod markdown_export;
pub use markdown_export::{
    add_md_row, column_count as md_column_count, export_mesh_list_md, export_mesh_stats_md,
    new_md_table, row_count as md_row_count, to_markdown_string, MdRow, MdTable,
};

pub mod html_export;
pub use html_export::{
    add_html_row, add_html_section, export_mesh_stats_html, html_escape as html_tag_escape,
    new_html_export, section_count as html_section_count, to_html_string,
    total_row_count as html_total_row_count, HtmlExport, HtmlSection,
};

pub mod latex_export;
pub use latex_export::{
    add_latex_bone, bone_count_latex, default_biped_latex_doc, export_latex, latex_set_scale,
    latex_size_bytes, new_latex_doc, render_latex, validate_latex_doc, LatexBone, LatexSkeletonDoc,
};

pub mod ascii_art_export;
pub use ascii_art_export::{
    aabb_summary, compute_aabb as compute_ascii_aabb, render_aabb_ascii, render_side_view_ascii,
    AsciiAabb,
};

pub mod dot_export;
pub use dot_export::{
    add_dot_edge, add_dot_node, dot_edge_count, dot_node_count, export_skeleton_dot, find_dot_node,
    new_dot_export, to_dot_string, DotEdge, DotExport, DotNode,
};

pub mod graphml_export;
pub use graphml_export::{
    add_graphml_edge, add_graphml_node, export_bones_graphml, find_graphml_node,
    graphml_edge_count, graphml_node_count, new_graphml_export, to_graphml_string, GraphMlEdge,
    GraphMlExport, GraphMlNode,
};

pub mod mermaid_export;
pub use mermaid_export::{
    add_mermaid_edge, add_mermaid_node, export_skeleton_mermaid, find_mermaid_node,
    mermaid_edge_count, mermaid_node_count, new_mermaid_export, set_mermaid_direction,
    to_mermaid_string, MermaidEdge, MermaidExport, MermaidNode,
};

pub mod plantuml_export;
pub use plantuml_export::{
    add_final_state, add_plant_state, add_plant_transition, find_plant_state, new_plantuml_export,
    plant_state_count, plant_transition_count, set_initial_state, to_plantuml_string, PlantState,
    PlantTransition, PlantUmlExport,
};

pub mod proto_text_export;
pub use proto_text_export::{
    add_proto_bool, add_proto_float, add_proto_int, add_proto_nested, add_proto_string,
    export_mesh_stats_proto, find_proto_field, new_proto_message, proto_field_count, to_proto_text,
    ProtoField, ProtoMessage,
};

pub mod capnp_stub_export;
pub use capnp_stub_export::{
    add_capnp_field, add_capnp_struct, capnp_last_struct_field_count, capnp_struct_count,
    export_mesh_capnp_schema, find_capnp_struct, new_capnp_export, to_capnp_schema, CapnpExport,
    CapnpField, CapnpStruct,
};

pub mod flatbuf_stub_export;
pub use flatbuf_stub_export::{
    add_fbs_field, add_fbs_table, export_mesh_fbs_schema, fbs_last_table_field_count,
    fbs_table_count, find_fbs_table, new_flatbuf_export, set_fbs_root_type, to_fbs_schema,
    FbsField, FbsTable, FlatbufExport,
};

pub mod avro_export;
pub use avro_export::{
    add_avro_field, add_avro_record, avro_field_count, avro_record_count, export_mesh_avro,
    new_avro_export, new_avro_schema, record_to_json as avro_record_to_json,
    records_to_json as avro_records_to_json, schema_to_json as avro_schema_to_json, AvroExport,
    AvroField, AvroRecord, AvroSchema,
};

pub mod parquet_stub_export;
pub use parquet_stub_export::{
    add_parquet_column, add_row_group, export_mesh_parquet_meta, new_parquet_export,
    parquet_row_group_count, parquet_total_rows, to_parquet_metadata_json, ParquetColumn,
    ParquetExport, ParquetRowGroup, ParquetType,
};

pub mod arrow_export;
pub use arrow_export::{
    add_arrow_batch, add_arrow_field, arrow_field_count, arrow_schema_to_json, arrow_total_rows,
    export_positions_arrow, new_arrow_export, ArrowBatch, ArrowExport, ArrowField, ArrowSchema,
    ArrowType,
};

pub mod wav_export;
pub use wav_export::{
    build_wav_header, encode_samples_i16, export_wav, silent_wav, sine_wav, wav_duration,
    wav_peak_amplitude, WavConfig, WavFile,
};

pub mod midi_export;
pub use midi_export::{
    add_note, build_midi_header, build_midi_track, encode_vlq, export_midi, midi_duration_secs,
    midi_duration_ticks, MidiExport, MidiNote,
};

pub mod osc_export;
pub use osc_export::{
    new_osc_message, osc_add_float, osc_add_int, osc_add_string, serialize_osc_bundle,
    serialize_osc_message, OscArg, OscBundle, OscMessage,
};

pub mod dmx_export;
pub use dmx_export::{
    active_channel_count, blend_frames, fill_frame, get_channel, serialize_dmx_frame, set_channel,
    set_rgb, DmxFrame,
};

pub mod artnet_export;
pub use artnet_export::{
    artnet_get_channel, artnet_set_channel, build_artnet_dmx_packet, universe_to_subnet_address,
    ArtDmxPacket, ARTNET_OP_DMX, ARTNET_PORT,
};

pub mod sacn_export;
pub use sacn_export::{
    build_sacn_packet, sacn_next_sequence, sacn_set_channel, sacn_validate, SacnConfig, SacnPacket,
    SACN_DEFAULT_PRIORITY,
};

pub mod ros_bag_export;
pub use ros_bag_export::{
    add_connection, build_ros_bag_header_bytes, find_connection_by_topic, has_topic,
    record_message, RosBag, RosBagConnection, RosBagHeader,
};

pub mod sensor_log_export;
pub use sensor_log_export::{
    add_reading, average_sensor_value, export_sensor_log_csv, filter_by_sensor, log_duration_ms,
    peak_sensor_value, reading_count, sort_by_timestamp as sort_sensor_log_by_timestamp,
    unique_sensor_ids, SensorLog, SensorReading,
};

pub mod telemetry_export;
pub use telemetry_export::{
    add_telemetry_channel, channel_average, channel_count_tl, export_telemetry_csv,
    export_telemetry_meta, find_channel_by_name, frame_count_tl, record_frame, session_duration_us,
    TelemetryChannel, TelemetryExport, TelemetryFrame,
};

pub mod event_log_export;
pub use event_log_export::{
    count_by_severity, event_count as event_log_count, export_event_log_csv,
    export_event_log_ndjson, filter_by_type as filter_events_by_type, has_errors, log_event,
    sort_events_by_time, EventEntry, EventLog, EventSeverity,
};

pub mod diff_export;
pub use diff_export::{
    addition_count, changed_line_count, compute_diff, export_diff_unified, is_identical,
    removal_count, DiffEntry, DiffExport, DiffOp,
};

pub mod changelog_export;
pub use changelog_export::{
    add_addition, add_changelog_entry, add_fix, entry_count, export_changelog_md,
    find_entry_by_version, latest_version, total_changes, Changelog, ChangelogEntry,
};

pub mod license_export;
pub use license_export::{
    add_license, export_license_json, export_license_txt, export_spdx_expression, find_by_spdx_id,
    has_license, license_count, osi_approved_count, remove_license, LicenseEntry, LicenseManifest,
};

pub mod manifest_export;
pub use manifest_export::{
    add_author, add_dependency, author_count, dependency_count, export_manifest_json,
    export_manifest_toml, find_dependency, optional_dependency_count,
    set_description as set_manifest_description, set_license as set_manifest_license, Dependency,
    SoftwareManifest,
};

pub mod inventory_export;
pub use inventory_export::{
    add_asset, asset_count, count_by_type as count_assets_by_type, export_inventory_csv,
    export_inventory_json, find_asset_by_id, find_asset_by_name, total_file_size,
    total_vertex_count as inventory_total_vertex_count, AssetEntry, AssetInventory, AssetStats,
    AssetType,
};

pub mod audit_log_export;
pub use audit_log_export::{
    audit_entry_count, count_by_actor, count_by_outcome, export_audit_csv, export_audit_ndjson,
    filter_by_action, has_denials, latest_timestamp, log_audit,
    sort_by_timestamp as sort_audit_by_timestamp, AuditEntry, AuditLog, AuditOutcome,
};

pub mod kml_export;
pub use kml_export::{
    add_kml_placemark, body_scan_to_kml, export_kml, kml_placemark_count, kml_size_bytes,
    kml_to_geojson_stub, new_kml_document, render_kml, validate_kml, KmlDocument, KmlPlacemark,
};

pub mod geojson_export;
pub use geojson_export::{
    add_linestring_feature, add_point_feature, body_landmarks_to_geojson, export_geojson,
    geojson_feature_count, geojson_size_bytes, new_geojson_collection, render_geojson,
    validate_geojson, GeoJsonCollection, GeoJsonFeature, GeoJsonGeometry,
};

pub mod gpx_export;
pub use gpx_export::{
    add_gpx_track, add_gpx_track_point, add_gpx_waypoint, body_scan_track, export_gpx,
    gpx_point_count, gpx_size_bytes, gpx_track_count, gpx_waypoint_count, new_gpx_document,
    render_gpx, validate_gpx, GpxDocument, GpxTrack, GpxTrackPoint,
};

pub mod nmea_export;
pub use nmea_export::{
    build_gga, build_rmc, decimal_lon_to_nmea, decimal_to_nmea, export_positions_as_gga,
    nmea_checksum, sentence_count as nmea_sentence_count, sentence_has_crlf,
};

pub mod svg_path_export;
pub use svg_path_export::{
    close_path as svg_close_path, command_count as svg_command_count, commands_to_d, cubic_to,
    line_to as svg_line_to, move_to as svg_move_to, new_svg_path, path_to_svg_tag,
    polyline_to_path as svg_polyline_to_path, starts_with_move, wrap_svg, SvgPathCmd,
    SvgPathElement,
};

pub mod svg_polygon_export;
pub use svg_polygon_export::{
    add_polygon as svg_add_polygon, add_polyline as svg_add_polyline, export_polygon_svg,
    new_polygon as new_svg_polygon, new_polygon_doc, new_polyline as new_svg_polyline,
    points_to_attr, polygon_aabb, polygon_to_tag, polygon_vertex_count, polyline_to_tag,
    SvgPolygon, SvgPolygonDoc, SvgPolyline,
};

pub mod eps_export;
pub use eps_export::{
    add_eps_path, edges_to_eps_paths, eps_bounding_box, eps_path_count, export_eps,
    new_eps_document, EpsDocument, EpsOptions, EpsPath,
};

pub mod pdf_stub_export;
pub use pdf_stub_export::{
    add_content_stream, add_text_stream, export_pdf_stub, is_valid_pdf_header, new_pdf_stub,
    pdf_estimated_size, pdf_object_count, PdfObject, PdfStub,
};

pub mod cbor_export;
pub use cbor_export::{
    cbor_header, encode_array_header, encode_bool, encode_bytes, encode_f32, encode_map_header,
    encode_null, encode_text, encode_uint, uint_byte_len, CborMajor,
};

pub mod bson_export;
pub use bson_export::{
    bson_byte_len, element_count, serialize_bson, BsonDocument, BsonElement, BsonType,
};

pub mod smile_export;
pub use smile_export::{
    export_smile, is_smile_magic, SmileDocument, SmileToken, SMILE_MAGIC, SMILE_VERSION,
};

pub mod ion_export;
pub use ion_export::{export_ion, ion_value_count, IonDocument, IonValue};

pub mod thrift_export;
pub use thrift_export::{ThriftEncoder, ThriftType};

pub mod protobuf_export;
pub use protobuf_export::{encode_tag, encode_varint, zigzag32, zigzag64, ProtoEncoder, WireType};

pub mod grpc_stub_export;
pub use grpc_stub_export::{
    add_metadata, build_grpc_request, build_grpc_response, is_ok as grpc_is_ok, GrpcCompression,
    GrpcFrame, GrpcRequest, GrpcResponse,
};

pub mod jsonrpc_export;
pub use jsonrpc_export::{
    is_success as jsonrpc_is_success, new_jsonrpc_error, new_jsonrpc_request, new_jsonrpc_result,
    serialize_request as serialize_jsonrpc_request,
    serialize_response as serialize_jsonrpc_response, JsonRpcError, JsonRpcRequest,
    JsonRpcResponse,
};

pub mod graphql_export;
pub use graphql_export::{
    add_variable, new_mutation, new_query, serialize_gql, var_count, GqlOpType, GqlOperation,
    GqlVar,
};

pub mod openapi_export;
pub use openapi_export::{
    add_param, add_path as add_openapi_path, export_openapi_json, new_openapi_doc,
    path_count as openapi_path_count, OpenApiDoc, OpenApiInfo, OpenApiParam, OpenApiPath,
};

pub mod swagger_export;
pub use swagger_export::{
    add_operation, add_tag, export_swagger_json, new_swagger_doc, operation_count, SwaggerDoc,
    SwaggerInfo, SwaggerOperation,
};

pub mod raml_export;
pub use raml_export::{
    add_method as add_raml_method, add_resource, export_raml, new_raml_doc, resource_count,
    RamlDoc, RamlMethod, RamlResource,
};

pub mod hal_export;
pub use hal_export::{link_count, property_count, serialize_hal, HalLink, HalResource};

pub mod jsonld_export;
pub use jsonld_export::{
    export_jsonld, node_count, serialize_node, LdContextEntry, LdDocument, LdNode,
};

pub mod turtle_export;
pub use turtle_export::{
    contains_triple, export_turtle, prefix_count, triple_count, RdfTriple, TurtleDoc,
};

pub mod rdf_xml_export;
pub use rdf_xml_export::{
    description_count, export_rdf_xml, has_xml_declaration, RdfDescription, RdfNs, RdfXmlDoc,
};

pub mod spine_export;
pub use spine_export::{
    export_spine_json, find_bone as spine_find_bone, total_bone_length as spine_total_bone_length,
    SpineBone, SpineExport, SpineSlot,
};

pub mod dragonbones_export;
pub use dragonbones_export::{
    avg_frame_rate as db_avg_frame_rate, export_dragonbones_json,
    find_armature as db_find_armature, total_bone_count_db, DbArmature, DragonBonesExport,
};

pub mod cocos_export;
pub use cocos_export::{
    export_cocos_json, find_node as cocos_find_node, scene_depth, CocosNode, CocosScene,
};

pub mod lottie_export;
pub use lottie_export::{
    active_frame_range, export_lottie_json, find_lottie_layer, validate_lottie, LottieExport,
    LottieLayer,
};

pub mod gif_export;
pub use gif_export::{estimate_gif_size, gif_metadata_json, validate_gif, GifExport, GifFrame};

pub mod apng_export;
pub use apng_export::{
    apng_metadata_json, estimate_raw_bytes as apng_estimate_raw_bytes, validate_apng, ApngExport,
    ApngFrame,
};

pub mod webp_export;
pub use webp_export::{
    average_luminance as webp_average_luminance, estimate_webp_bytes, validate_webp,
    webp_metadata_json, WebpExport, WebpOptions,
};

pub mod avif_export;
pub use avif_export::{
    all_opaque, avif_metadata_json, estimate_avif_bytes, validate_avif, AvifExport, AvifOptions,
    AvifPreset,
};

pub mod jpeg_xl_export;
pub use jpeg_xl_export::{
    estimate_jxl_bytes, jxl_metadata_json, peak_pixel_value, validate_jxl, JxlExport, JxlMode,
    JxlOptions,
};

pub mod openexr_export;
pub use openexr_export::{
    estimate_exr_bytes, exr_metadata_json, find_channel as exr_find_channel, ExrChannel,
    ExrChannelType, ExrExport,
};

pub mod tiff_export;
pub use tiff_export::{
    estimate_tiff_bytes, tiff_metadata_json, validate_tiff, TiffCompression, TiffExport,
    TiffOptions,
};

pub mod bmp_export;
pub use bmp_export::{
    average_brightness, build_bmp_header, estimate_bmp_bytes, validate_bmp, BmpBitDepth, BmpExport,
};

pub mod ico_export;
pub use ico_export::{
    estimate_ico_bytes, find_ico_entry, ico_metadata_json, validate_ico, IcoEntry, IcoExport,
};

pub mod psd_export;
pub use psd_export::{
    estimate_psd_bytes, find_psd_layer, psd_metadata_json, validate_psd, PsdBlendMode, PsdExport,
    PsdLayer,
};

pub mod pdf_export;
pub use pdf_export::{
    estimate_pdf_bytes, pdf_header_bytes, pdf_metadata_json, validate_pdf, PdfExport, PdfPage,
    PdfPageSize,
};

pub mod epub_export;
pub use epub_export::{
    epub_metadata_json, opf_manifest_stub, validate_epub, EpubChapter, EpubExport, EpubMeta,
};

pub mod abc_pointcloud_export;
pub use abc_pointcloud_export::{
    abc_pointcloud_config_to_json, add_abc_frame, estimate_abc_size_bytes,
    frame_count as abc_frame_count, frame_point_count, new_abc_pointcloud,
    total_point_count as abc_total_point_count, validate_abc_pointcloud, AbcPointcloudConfig,
    AbcPointcloudExport, AbcPointcloudFrame,
};

pub mod vdb_export;
pub use vdb_export::{
    default_vdb_config, new_vdb_grid, vdb_active_count, vdb_clear, vdb_export_to_bytes,
    vdb_get_voxel, vdb_set_voxel, vdb_stats, vdb_to_json, VdbConfig, VdbExportResult, VdbGrid,
};

pub mod bgeo_export;
pub use bgeo_export::{
    bgeo_add_attr, bgeo_add_point, bgeo_attr_count, bgeo_bounds, bgeo_find_attr, bgeo_header_bytes,
    bgeo_set_prim_count, bgeo_size_estimate, new_bgeo_export, validate_bgeo, BgeoAttr,
    BgeoAttrType, BgeoExport, BGEO_VERSION,
};

pub mod hip_export;
pub use hip_export::{
    hip_add_node, hip_extension, hip_find_node, hip_node_count, hip_set_parm, hip_size_estimate,
    hip_to_string, new_hip_export, validate_hip, HipExport, HipFormat, HipNode,
};

pub mod nuke_export;
pub use nuke_export::{
    new_nuke_export, nuke_add_node, nuke_count_by_class, nuke_find_node, nuke_node_count,
    nuke_set_knob, nuke_set_position, nuke_size_estimate, nuke_to_string, validate_nuke, NukeNode,
    NukeScriptExport,
};

pub mod hiero_export;
pub use hiero_export::{
    clips_on_track, hiero_add_clip, hiero_clip_count, hiero_find_clip, hiero_script_size,
    hiero_to_python, new_hiero_timeline, timeline_duration_frames, validate_hiero_timeline,
    HieroClip, HieroTimeline,
};

pub mod resolve_export;
pub use resolve_export::{
    new_resolve_timeline, resolve_add_clip, resolve_clip_count, resolve_clips_for_reel,
    resolve_duration_frames, resolve_script_size, resolve_to_python, timeline_type_name,
    validate_resolve_timeline, ResolveClip, ResolveTimeline, ResolveTimelineType,
};

pub mod edl_export;
pub use edl_export::{
    edl_add_event, edl_event_count, edl_size_bytes, edl_to_string, events_for_reel,
    frames_to_timecode, new_edl_export, validate_edl, EdlEvent, EdlExport,
};

pub mod aaf_export;
pub use aaf_export::{
    aaf_add_component, aaf_add_track, aaf_component_count, aaf_duration_frames, aaf_find_track,
    aaf_size_estimate, aaf_to_xml_stub, aaf_track_count, new_aaf_export, validate_aaf,
    AafComponent, AafEssenceKind, AafExport, AafTrack,
};

pub mod mxf_export;
pub use mxf_export::{
    mxf_add_track, mxf_duration_frames, mxf_find_track_by_type, mxf_header_bytes, mxf_op_name,
    mxf_size_estimate, mxf_track_count, new_mxf_export, validate_mxf, MxfExport, MxfOpPattern,
    MxfTrack,
};

pub mod r3d_export;
pub use r3d_export::{
    new_r3d_export, r3d_add_frame, r3d_average_iso, r3d_duration_seconds, r3d_frame_count,
    r3d_metadata_string, r3d_resolution, r3d_size_estimate, validate_r3d, R3dCodec, R3dExport,
    R3dFrame,
};

pub mod arriraw_export;
pub use arriraw_export::{
    arriraw_add_frame, arriraw_average_iso, arriraw_duration, arriraw_frame_count,
    arriraw_metadata_string, arriraw_resolution, arriraw_size_estimate, new_arriraw_export,
    validate_arriraw, ArriFrame, ArriModel, ArriRawExport,
};

pub mod cineraw_export;
pub use cineraw_export::{
    cineraw_add_frame, cineraw_avg_shutter_angle, cineraw_duration, cineraw_frame_count,
    cineraw_metadata_string, cineraw_resolution, cineraw_size_estimate, new_cineraw_export,
    validate_cineraw, CineDngBitDepth, CineDngFrame, CineRawExport,
};

pub mod cdl_export;
pub use cdl_export::{
    apply_cdl, cdl_add_identity, cdl_add_node, cdl_find_node, cdl_node_count, cdl_size_bytes,
    cdl_to_xml, new_cdl_export, validate_cdl, CdlExport, CdlNode,
};

pub mod cube_lut_export;
pub use cube_lut_export::{
    cube_apply_gain, cube_entry_count, cube_sample, cube_size_bytes, cube_to_string,
    expected_entry_count, new_cube_lut, validate_cube_lut, CubeLut,
};

pub mod csp_lut_export;
pub use csp_lut_export::{
    csp_add_shaper, csp_average_brightness, csp_entry_count, csp_sample, csp_shaper_count,
    csp_size_bytes, csp_to_string, new_csp_lut, validate_csp_lut, CspLut, CspShaper,
};

pub mod srt_export;
pub use srt_export::{
    ms_to_srt_time, render_srt, total_duration_ms as srt_total_duration_ms, validate_srt,
    SrtDocument, SrtEntry,
};

pub mod vtt_export;
pub use vtt_export::{
    max_cue_length, ms_to_vtt_time, render_vtt, total_duration_ms as vtt_total_duration_ms,
    validate_vtt, VttCue, VttDocument,
};

pub mod ass_export;
pub use ass_export::{
    cs_to_ass_time, default_style as default_ass_style, render_ass, render_dialogues,
    render_script_info, total_duration_cs, validate_ass, AssDialogue, AssDocument, AssStyle,
};

pub mod ttml_export;
pub use ttml_export::{
    ms_to_ttml_time, render_ttml, total_duration_ms as ttml_total_duration_ms, validate_ttml,
    xml_escape as ttml_xml_escape, TtmlDocument, TtmlParagraph, TtmlSpan,
};

pub mod smil_export;
pub use smil_export::{
    add_fullscreen_region, ms_to_smil_clock, render_smil, validate_smil, SmilDocument, SmilMedia,
    SmilRegion,
};

pub mod bibtex_export;
pub use bibtex_export::{
    render_bibtex, render_entry as render_bibtex_entry, validate_entry as validate_bibtex_entry,
    BibtexBibliography, BibtexEntry, BibtexEntryType,
};

pub mod ris_export;
pub use ris_export::{
    count_by_type as ris_count_by_type, render_record as render_ris_record, render_ris,
    validate_record as validate_ris_record, RisDatabase, RisField, RisRecord, RisType,
};

pub mod endnote_export;
pub use endnote_export::{
    count_by_type as endnote_count_by_type, render_endnote_xml, render_ref_xml,
    validate_ref as validate_endnote_ref, EndnoteLibrary, EndnoteRef, EndnoteRefType,
};

pub mod zotero_export;
pub use zotero_export::{
    count_by_type as zotero_count_by_type, item_to_json as zotero_item_to_json,
    library_to_json as zotero_library_to_json, validate_item as validate_zotero_item, CslCreator,
    CslItem, ZoteroLibrary,
};

pub mod json_ld_export;
pub use json_ld_export::{
    add_schema_context, node_to_json as json_ld_node_to_json, render_json_ld,
    validate_document as validate_json_ld, JsonLdDocument, JsonLdNode,
};

pub mod rdf_export;
pub use rdf_export::{
    count_by_predicate as rdf_count_by_predicate, render_triple_turtle, render_turtle,
    subjects_with_object, validate_graph as validate_rdf_graph, RdfGraph,
    RdfTriple as RdfExportTriple,
};

pub mod owl_export;
pub use owl_export::{
    all_superclass_iris, render_owl_turtle, root_class_count, validate_ontology, OwlClass,
    OwlObjectProperty, OwlOntology,
};

pub mod sparql_export;
pub use sparql_export::{
    add_rdf_prefix as sparql_add_rdf_prefix, add_schema_prefix as sparql_add_schema_prefix,
    render_sparql, validate_query as validate_sparql_query, SparqlPrefix, SparqlQuery,
    SparqlQueryType,
};

pub mod graphql_schema_export;
pub use graphql_schema_export::{
    render_schema_sdl, render_type_sdl, validate_schema as validate_graphql_schema, GqlField,
    GqlFieldType, GqlObjectType, GqlSchema,
};

pub mod openapi_schema_export;
pub use openapi_schema_export::{
    render_openapi_json, total_operation_count, validate_spec as validate_openapi_spec, ApiInfo,
    ApiOperation, ApiPath, HttpMethod, OpenApiSpec,
};

pub mod asyncapi_export;
pub use asyncapi_export::{
    add_server as asyncapi_add_server, publish_channel_count, render_asyncapi_json,
    subscribe_channel_count, validate_spec as validate_asyncapi_spec, AsyncApiSpec, AsyncChannel,
    AsyncMessage, AsyncProtocol,
};

pub mod iges_curve_export;
pub use iges_curve_export::{
    add_iges_curve, iges_curve_count, iges_entity_line, iges_global_section, new_iges_curve_export,
    validate_iges_curves, IgesCurveEntity, IgesCurveExport, IgesCurveType,
};

pub mod step_solid_export;
pub use step_solid_export::{
    add_step_entity, new_step_solid_export, step_entity_count, step_entity_line, step_file_header,
    validate_step_export, StepEntity, StepEntityKind, StepSolidExport,
};

pub mod brep_export;
pub use brep_export::{
    add_brep_edge, add_brep_face, add_brep_vertex, euler_characteristic, new_brep_export,
    validate_brep, BRepEdge, BRepExport, BRepFace, BRepVertex,
};

pub mod sat_export;
pub use sat_export::{
    add_sat_entity, find_sat_entity, new_sat_export, sat_entity_count, sat_entity_line, sat_header,
    validate_sat, SatEntity, SatExport,
};

pub mod parasolid_export;
pub use parasolid_export::{
    add_ps_entity, new_parasolid_export, ps_count_by_tag, ps_entity_count, ps_xt_header,
    validate_parasolid, ParasolidExport, PsEntity, PsEntityTag,
};

pub mod jt_export;
pub use jt_export::{
    add_jt_lod, jt_file_header, jt_high_lod, jt_lod_count, jt_total_tri_count,
    jt_total_vertex_count, new_jt_export, validate_jt_export, JtExport, JtLod, JtLodLevel,
};

pub mod threedxml_export;
pub use threedxml_export::{
    add_threedxml_occurrence, add_threedxml_rep, new_threedxml_export, threedxml_occurrence_count,
    threedxml_rep_count, threedxml_xml_header, validate_threedxml, ThreeDXmlExport,
    ThreeDXmlOccurrence, ThreeDXmlRep,
};

pub mod ifc_export;
pub use ifc_export::{
    add_ifc_entity, ifc_count_class, ifc_entity_count, ifc_entity_line, ifc_header, new_ifc_export,
    validate_ifc, IfcClass, IfcEntity, IfcExport,
};

pub mod citygml_export;
pub use citygml_export::{
    add_city_building, citygml_building_count, citygml_max_lod, citygml_total_volume,
    citygml_xml_header, new_citygml_export, validate_citygml, CityBuilding, CityGmlExport, CityLod,
};

pub mod landxml_export;
pub use landxml_export::{
    add_landxml_alignment, add_landxml_surface, landxml_alignment_count, landxml_surface_count,
    landxml_total_tris, landxml_xml_header, new_landxml_export, validate_landxml, LandXmlAlignment,
    LandXmlExport, LandXmlSurface,
};

pub mod geotiff_export;
pub use geotiff_export::{
    geotiff_get_pixel, geotiff_min_max, geotiff_pixel_count, geotiff_pixel_to_geo,
    geotiff_set_pixel, new_geotiff_export, validate_geotiff, GeoTiffExport, GeoTiffPixelType,
};

pub mod las_export;
pub use las_export::{
    add_las_point, build_las_header_bytes, las_file_size_estimate_v2, las_from_positions,
    las_point_count_v2, las_world_x, new_las_export, validate_las, LasExport, LasHeaderV2,
    LasPointV2, LAS_MAGIC,
};

pub mod e57_export;
pub use e57_export::{
    add_e57_point, e57_bbox, e57_from_positions, e57_point_count, e57_size_estimate,
    e57_xml_header_v2, export_e57_stub, new_e57_export, validate_e57, E57Export, E57Point,
    E57_MAGIC,
};

pub mod pts_pointcloud_export;
pub use pts_pointcloud_export::{
    add_pts_point, export_pts_text, new_pts_export, pts_bbox, pts_centroid, pts_point_count,
    PtsExport, PtsPoint,
};

pub mod xyz_pointcloud_export;
pub use xyz_pointcloud_export::{
    add_xyz_point, add_xyz_point_normal, export_xyz_text, new_xyz_export, validate_xyz, xyz_bbox,
    xyz_centroid, xyz_point_count, XyzExport,
};

pub mod pcd_export;
pub use pcd_export::{
    add_pcd_point, build_pcd_header, export_pcd_ascii, export_pcd_binary, new_pcd_export,
    pcd_centroid, pcd_from_positions, pcd_point_count, validate_pcd, PcdDataType, PcdExport,
};

pub mod ptx_export;
pub use ptx_export::{
    add_ptx_point, build_ptx_header_string, export_ptx_string, new_ptx_export,
    ptx_file_size_estimate, ptx_from_positions, ptx_point_count, validate_ptx, PtxExport,
    PtxHeader, PtxPoint,
};

pub mod svg_animation_export;
pub use svg_animation_export::{
    add_smil_element, anim_element_count, new_svg_anim_document, render_svg_anim,
    total_anim_duration_ms, validate_svg_anim, SmilAnimElement, SvgAnimDocument,
};

pub mod css_animation_export;
pub use css_animation_export::{
    add_css_keyframe, css_keyframe_count, new_css_animation, render_css_animation_rule,
    render_css_keyframes, validate_css_animation, CssAnimation, CssKeyframe,
};

pub mod web_animation_api_export;
pub use web_animation_api_export::{
    add_web_anim_keyframe, new_web_anim_export, render_web_anim_json, validate_web_anim,
    web_anim_keyframe_count, WebAnimExport, WebAnimKeyframe, WebAnimOptions,
};

pub mod canvas_2d_export;
pub use canvas_2d_export::{
    command_count as canvas_command_count, draw_line, new_canvas_2d_export, push_cmd,
    render_canvas_js, validate_canvas_export, Canvas2dCmd, Canvas2dExport,
};

pub mod webgl_export;
pub use webgl_export::{
    add_webgl_f32_buffer, add_webgl_index_buffer, find_webgl_buffer, new_webgl_export,
    validate_webgl_export, webgl_buffer_count, webgl_total_bytes, WebGlBuffer, WebGlBufferType,
    WebGlExport,
};

pub mod shader_toy_export;
pub use shader_toy_export::{
    add_shader_toy_channel, new_shader_toy_export, render_shader_toy_stub, set_common_shader,
    set_image_shader, shader_contains, shader_toy_channel_count, validate_shader_toy,
    ShaderToyChannel, ShaderToyExport,
};

pub mod glsl_export;
pub use glsl_export::{
    add_glsl_define, add_glsl_shader, find_glsl_shader, glsl_shader_count, new_glsl_export,
    render_glsl_shader, validate_glsl_export, GlslExport, GlslShader, GlslStage,
};

pub mod hlsl_export;
pub use hlsl_export::{
    add_hlsl_define, add_hlsl_shader, find_hlsl_shader, hlsl_shader_count, new_hlsl_export,
    render_hlsl_shader, validate_hlsl_export, HlslExport, HlslProfile, HlslShader,
};

pub mod msl_export;
pub use msl_export::{
    add_msl_function, add_msl_include, find_msl_function, msl_function_count, new_msl_export,
    render_msl_source, validate_msl_export, MslExport, MslFunction, MslFunctionType,
};

pub mod wgsl_export;
pub use wgsl_export::{
    add_wgsl_entry_point, add_wgsl_global, add_wgsl_struct, find_wgsl_entry, new_wgsl_export,
    render_wgsl_source, validate_wgsl_export, wgsl_entry_point_count, WgslEntryPoint, WgslExport,
    WgslStage,
};

pub mod spir_v_export;
pub use spir_v_export::{
    add_spirv_entry_point, new_spirv_export, spirv_byte_size, spirv_entry_point_count,
    spirv_has_valid_header, spirv_to_bytes, spirv_word_count, validate_spirv_magic, SpirVExport,
    SPIRV_MAGIC, SPIRV_VERSION_1_5,
};

pub mod cuda_ptx_export;
pub use cuda_ptx_export::{
    add_cuda_ptx_kernel, cuda_ptx_kernel_count, cuda_ptx_size_estimate, find_cuda_ptx_kernel,
    new_cuda_ptx_export, render_cuda_ptx, validate_cuda_ptx, CudaPtxExport, CudaPtxKernel,
    PTX_ISA_VERSION, PTX_TARGET_SM80,
};

pub mod opencl_export;
pub use opencl_export::{
    add_cl_kernel, add_cl_kernel_arg, cl_kernel_count, find_cl_kernel, new_opencl_export,
    render_opencl_source, validate_opencl_export, ClKernel, ClKernelArg, OpenClExport,
};

pub mod compute_shader_export;
pub use compute_shader_export::{
    add_compute_binding, compute_binding_count, compute_group_count, new_compute_shader_export,
    render_compute_summary, set_compute_source, validate_compute_shader, ComputeApi,
    ComputeShaderExport, DispatchConfig,
};

pub mod ray_gen_shader_export;
pub use ray_gen_shader_export::{
    add_ray_shader, find_ray_shader, new_ray_gen_shader_export, ray_shader_count,
    render_ray_gen_summary, validate_ray_gen_export, RayGenShaderExport, RayShader, RayShaderType,
};

pub mod mesh_shader_export;
pub use mesh_shader_export::{
    add_mesh_shader_program, find_mesh_shader_program, mesh_shader_program_count,
    new_mesh_shader_export, render_mesh_shader_summary, validate_mesh_shader_export,
    MeshShaderExport, MeshShaderProgram, MeshShaderStage,
};

pub mod onnx_export;
pub use onnx_export::*;

pub mod tflite_export;
pub use tflite_export::*;

pub mod torch_script_export;
pub use torch_script_export::*;

pub mod coreml_export;
pub use coreml_export::*;

pub mod ncnn_export;
pub use ncnn_export::*;

pub mod openvino_export;
pub use openvino_export::*;

pub mod tensorrt_export;
pub use tensorrt_export::*;

pub mod rknn_export;
pub use rknn_export::*;

pub mod snpe_export;
pub use snpe_export::*;

pub mod deepsparse_export;
pub use deepsparse_export::*;

pub mod gguf_export;
pub use gguf_export::*;

pub mod safetensors_export;
pub use safetensors_export::*;

pub mod npz_export;
pub use npz_export::*;

pub mod pickle_export;
pub use pickle_export::*;

pub mod hdf5_weights_export;
pub use hdf5_weights_export::*;

pub mod checkpoint_export;
pub use checkpoint_export::*;

pub mod ros2_export;
pub use ros2_export::*;

pub mod mqtt_export;
pub use mqtt_export::*;

pub mod amqp_export;
pub use amqp_export::*;

pub mod kafka_export;
pub use kafka_export::*;

pub mod nats_export;
pub use nats_export::*;

pub mod grpc_service_export;
pub use grpc_service_export::*;

pub mod thrift_service_export;
pub use thrift_service_export::*;

pub mod zeromq_export;
pub use zeromq_export::*;

pub mod websocket_msg_export;
pub use websocket_msg_export::*;

pub mod sse_export;
pub use sse_export::*;

pub mod long_poll_export;
pub use long_poll_export::*;

pub mod rest_schema_export;
pub use rest_schema_export::*;

pub mod graphql_query_export;
pub use graphql_query_export::*;

pub mod odata_export;
pub use odata_export::*;

pub mod hateoas_export;
pub use hateoas_export::*;

pub mod wav_pcm_export;
pub use wav_pcm_export::*;

pub mod midi_clip_export;
pub use midi_clip_export::*;

pub mod osc_bundle_export;
pub use osc_bundle_export::*;

pub mod faust_export;
pub use faust_export::*;

pub mod supercollider_export;
pub use supercollider_export::*;

pub mod max_msp_export;
pub use max_msp_export::*;

pub mod pure_data_export;
pub use pure_data_export::*;

pub mod csound_export;
pub use csound_export::*;

pub mod chuck_export;
pub use chuck_export::*;

pub mod sonic_pi_export;
pub use sonic_pi_export::*;

pub mod lilypond_export;
pub use lilypond_export::*;

pub mod musicxml_export;
pub use musicxml_export::*;

pub mod abc_notation_export;
pub use abc_notation_export::*;

pub mod mxl_export;
pub use mxl_export::*;

pub mod guitar_pro_export;
pub use guitar_pro_export::*;

pub mod tablature_export;
pub use tablature_export::*;

pub mod opencolorio_export;
pub use opencolorio_export::*;

pub mod aces_export;
pub use aces_export::*;

pub mod icc_profile_export;
pub use icc_profile_export::*;

pub mod colormatch_export;
pub use colormatch_export::*;

pub mod spectral_export;
pub use spectral_export::*;

pub mod cri_export;
pub use cri_export::*;

pub mod munsell_export;
pub use munsell_export::*;

pub mod pantone_export;
pub use pantone_export::*;

pub mod ral_export;
pub use ral_export::*;

pub mod iec_61966_export;
pub use iec_61966_export::*;

pub mod dci_p3_export;
pub use dci_p3_export::*;

pub mod bt2020_export;
pub use bt2020_export::*;

pub mod hlg_export;
pub use hlg_export::*;

pub mod pq_export;
pub use pq_export::*;

pub mod display_p3_export;
pub use display_p3_export::*;

pub mod pro_photo_export;
pub use pro_photo_export::*;

pub mod haptic_frame_export;
pub use haptic_frame_export::{
    haptic_frame_count, haptic_frame_duration, haptic_frame_to_bytes, haptic_max_force,
    haptic_sequence_to_bytes, new_haptic_frame, HapticFrame,
};

pub mod biometric_export;
pub use biometric_export::{
    biometric_average_hr, biometric_min_spo2, biometric_sequence_to_csv, biometric_to_csv_line,
    new_biometric_sample, BiometricSample,
};

pub mod depth_map_export;
pub use depth_map_export::{
    depth_map_get, depth_map_max, depth_map_min, depth_map_normalize, depth_map_set,
    depth_map_to_u16, new_depth_map, DepthMap,
};

pub mod thermal_map_export;
pub use thermal_map_export::{
    new_thermal_map, thermal_get, thermal_mean_temp, thermal_set, thermal_to_bytes,
    thermal_to_false_color, ThermalMap,
};

pub mod flow_field_export;
pub use flow_field_export::{
    flow_divergence_at, flow_get, flow_max_speed, flow_set, flow_to_bytes, new_flow_field,
    FlowField,
};

pub mod stress_field_export;
pub use stress_field_export::{
    new_stress_field, stress_get, stress_max_principal_approx, stress_set, stress_to_bytes,
    stress_von_mises, StressField,
};

pub mod pressure_map_export;
pub use pressure_map_export::{
    new_pressure_map, pressure_center_of_pressure, pressure_get, pressure_max, pressure_set,
    pressure_to_bytes, pressure_total_force, PressureMap,
};

pub mod contact_area_export;
pub use contact_area_export::{
    contact_area, contact_count, contact_get, contact_set, contact_to_bytes, new_contact_map,
    ContactMap,
};

pub mod deformation_export;
pub use deformation_export::{
    deform_get, deform_max_displacement, deform_rms_displacement, deform_set, deform_to_bytes,
    new_deformation_field, DeformationField,
};

pub mod trajectory_export;
pub use trajectory_export::{
    new_trajectory_point, trajectory_duration, trajectory_max_speed, trajectory_sequence_to_csv,
    trajectory_to_csv_line, trajectory_total_distance, TrajectoryPoint,
};

pub mod landmark_export;
pub use landmark_export::{
    landmark_centroid, landmark_distance, landmark_to_json_line, landmarks_bounding_box,
    landmarks_to_json, new_landmark, Landmark,
};

pub mod emg_export;
pub use emg_export::{
    emg_duration_s, emg_peak, emg_push_sample, emg_rms, emg_to_bytes, emg_to_csv, new_emg_channel,
    EmgChannel,
};

pub mod galvanic_export;
pub use galvanic_export::{
    gsr_detect_responses, gsr_mean_conductance, gsr_peak_conductance, gsr_to_bytes, gsr_to_csv,
    new_gsr_sample, GsrSample,
};

pub mod strain_field_export;
pub use strain_field_export::{
    new_strain_field, strain_exceeds_threshold, strain_get, strain_max, strain_mean, strain_set,
    strain_to_bytes, StrainField,
};

pub mod medical_dicom_export;
pub use medical_dicom_export::{
    dicom_get_pixel, dicom_hu_to_display, dicom_pixel_count, dicom_set_pixel, dicom_to_bytes,
    new_dicom_slice, DicomSlice,
};

pub mod brain_signal_export;
pub use brain_signal_export::{
    eeg_band_power, eeg_duration_s, eeg_push_sample, eeg_rms, eeg_to_bytes, eeg_to_csv,
    new_eeg_channel, EegChannel,
};

pub mod muscle_activation_export;
pub use muscle_activation_export::{
    activation_duration_s, activation_mean, activation_peak, activation_push, activation_to_bytes,
    activation_to_csv, new_muscle_activation, MuscleActivation,
};

pub mod joint_torque_export;
pub use joint_torque_export::{
    new_joint_torque, torque_duration_s, torque_mean_magnitude, torque_peak, torque_push,
    torque_to_csv, JointTorque,
};

pub mod ground_reaction_export;
pub use ground_reaction_export::{
    grf_duration_s, grf_impulse, grf_peak_vertical, grf_push, grf_to_csv, new_ground_reaction,
    GroundReactionForce,
};

pub mod center_of_mass_export;
pub use center_of_mass_export::{
    com_duration_s, com_mean_height, com_push, com_to_csv, com_total_distance, new_center_of_mass,
    CenterOfMass,
};

pub mod inertia_tensor_export;
pub use inertia_tensor_export::{
    inertia_is_symmetric, inertia_principal_moments, inertia_set_diagonal, inertia_to_bytes,
    inertia_to_json, new_inertia_tensor, InertiaTensor,
};

pub mod skin_deformation_export;
pub use skin_deformation_export::{
    new_skin_deform_map, skin_deform_get, skin_deform_max_stretch, skin_deform_rms,
    skin_deform_set, skin_deform_to_bytes, SkinDeformMap,
};

pub mod ao_map_export;
pub use ao_map_export::{
    ao_get, ao_mean, ao_set, ao_to_bytes_req as ao_map_to_bytes, ao_to_u8, new_ao_map_req, AoMap,
};

pub mod roughness_map_export;
pub use roughness_map_export::{
    new_roughness_metalness_map, rm_get, rm_mean_roughness, rm_set, rm_to_bytes,
    RoughnessMetalnessMap,
};

pub mod displacement_map_export;
pub use displacement_map_export::{
    disp_get, disp_max_height, disp_set, disp_to_bytes, disp_to_u16, new_displacement_map,
    DisplacementMap,
};

pub mod opacity_map_export;
pub use opacity_map_export::{
    new_opacity_map, opacity_get, opacity_mean, opacity_set, opacity_threshold_mask, opacity_to_u8,
    OpacityMap,
};

pub mod emission_map_export;
pub use emission_map_export::{
    emission_get, emission_mean, emission_set, emission_to_bytes, emission_total_power,
    new_emission_map, EmissionMap,
};

pub mod subsurface_map_export;
pub use subsurface_map_export::{
    new_subsurface_map, sss_get_color, sss_get_radius, sss_set, sss_to_bytes, SubsurfaceMap,
};

pub mod wrinkle_map_export_data;
pub use wrinkle_map_export_data::{
    new_wrinkle_map_data, wrinkle_active_count, wrinkle_get, wrinkle_max_weight, wrinkle_set,
    wrinkle_to_bytes, WrinkleMapData,
};

pub mod ior_map_export;
pub use ior_map_export::{
    ior_get, ior_is_valid, ior_mean, ior_set, ior_to_bytes, new_ior_map, IorMap,
};

pub mod transmission_map_export;
pub use transmission_map_export::{
    new_transmission_map, trans_get, trans_mean, trans_set, trans_threshold_mask, trans_to_bytes,
    TransmissionMap,
};

pub mod scatter_coefficient_export;
pub use scatter_coefficient_export::{
    new_scatter_coefficients, scatter_albedo, scatter_count, scatter_extinction, scatter_push,
    scatter_to_csv, ScatterCoefficients,
};

pub mod skin_color_export;
pub use skin_color_export::{
    new_skin_color_spectrum, spectrum_count, spectrum_mean_reflectance, spectrum_push,
    spectrum_to_csv, spectrum_to_rgb, SkinColorSpectrum,
};

pub mod melanin_map_export;
pub use melanin_map_export::{
    melanin_map_get, melanin_map_mean_eu, melanin_map_set, melanin_map_to_bytes, melanin_map_total,
    new_melanin_map, MelaninMap,
};

pub mod hemoglobin_map_export;
pub use hemoglobin_map_export::{
    hemo_map_get, hemo_map_oxygen_saturation, hemo_map_set, hemo_map_to_bytes, new_hemoglobin_map,
    HemoglobinMap,
};

pub mod sebum_map_export;
pub use sebum_map_export::{
    new_sebum_map, sebum_get, sebum_mean, sebum_set, sebum_to_bytes, sebum_zones, SebumMap,
};

pub mod pore_map_export;
pub use pore_map_export::{
    new_pore_map, pore_count, pore_density, pore_get, pore_mean_size, pore_set, PoreMap,
};

pub mod skin_hair_root_export;
pub use skin_hair_root_export::{
    new_skin_hair_root, skin_hair_root_count, skin_hair_root_density_per_cm2,
    skin_hair_root_mean_diameter, skin_hair_root_to_csv_line, skin_hair_roots_to_csv, SkinHairRoot,
};

pub mod eyelash_export;
pub use eyelash_export::{
    eyelash_count, eyelash_length, eyelash_mean_length, eyelash_to_csv_line, eyelashes_to_csv,
    new_eyelash, Eyelash,
};

pub mod eyebrow_export;
pub use eyebrow_export::{
    eyebrow_density, eyebrow_hair_to_csv_line, eyebrow_hairs_to_csv, eyebrow_mean_length,
    new_eyebrow_hair, EyebrowHair,
};

pub mod beard_export;
pub use beard_export::{
    beard_count, beard_coverage_density, beard_mean_length, beard_strand_to_csv_line,
    beard_strands_to_csv, new_beard_strand, BeardStrand,
};

pub mod tattoo_map_export;
pub use tattoo_map_export::{
    new_tattoo_map, tattoo_coverage, tattoo_get_pixel, tattoo_set_pixel, tattoo_to_bytes, TattooMap,
};

pub mod scar_map_export;
pub use scar_map_export::{
    new_scar_map, scar_coverage, scar_get, scar_mean_elevation, scar_set, scar_to_bytes, ScarMap,
};

pub mod vein_map_export;
pub use vein_map_export::{
    new_vein_map, vein_add_path, vein_mean_depth, vein_path_count, vein_to_json, vein_total_length,
    VeinMap,
};

pub mod age_spot_export;
pub use age_spot_export::{
    new_age_spot, spot_area_mm2, spot_to_csv_line, spots_count, spots_mean_darkness, spots_to_csv,
    AgeSpot,
};

pub mod tooth_export;
pub use tooth_export::{
    new_tooth, teeth_to_json, tooth_count, tooth_is_molar, tooth_to_json, tooth_total_length, Tooth,
};

pub mod nail_export;
pub use nail_export::{
    nail_area_mm2, nail_count, nail_is_long, nail_to_json, nails_to_json, new_nail, Nail,
};

pub mod microstructure_export;
pub use microstructure_export::{
    micro_age_index, micro_is_smooth, micro_roughness_index, micro_skin_type_estimate,
    micro_to_json, new_skin_microstructure, SkinMicrostructure,
};

pub mod nla_strip_export;
pub use nla_strip_export::{
    new_nla_strip, strip_duration, strip_overlaps, strip_to_json, strips_to_json, NlaStrip,
};

pub mod light_export;
pub use light_export::{
    default_point_light_export, light_is_directional, light_lux_at_distance, light_to_json,
    lights_to_json, new_light_data, LightData, LightExport, LIGHT_DIRECTIONAL, LIGHT_POINT,
    LIGHT_SPOT,
};

pub mod camera_export;
pub use camera_export::{
    camera_is_orthographic, camera_projection_matrix, camera_to_json, camera_view_distance,
    default_camera_export, new_camera_data, CameraData, CameraExport,
};

pub mod world_export;
pub use world_export::{
    new_world_data, world_ambient_energy, world_fog_visibility, world_has_hdri, world_to_json,
    WorldData,
};

pub mod freestyle_export;
pub use freestyle_export::{
    freestyle_push_point, freestyle_stroke_count, freestyle_stroke_length,
    freestyle_strokes_to_svg, new_freestyle_stroke, FreestyleStroke,
};

pub mod cryptomatte_export;
pub use cryptomatte_export::{
    cryptomatte_coverage_sum, cryptomatte_entries_to_json, cryptomatte_name_to_hash,
    cryptomatte_to_json, new_cryptomatte_entry, CryptomatteEntry,
};

pub mod deep_image_export;
pub use deep_image_export::{
    deep_image_sample_count, deep_pixel_flatten, deep_pixel_push, new_deep_image, new_deep_pixel,
    DeepImage, DeepPixel,
};

pub mod constraint_export;
pub use constraint_export::{
    constraint_count, constraint_is_active, constraint_is_active_spec as constraint_is_active_data,
    constraint_kind_name, constraint_set_influence, constraint_to_json, constraint_validate,
    constraints_to_json, new_constraint_data, new_constraint_export, ConstraintData,
    ConstraintExport, ConstraintKind,
};

pub mod driver_export;
pub use driver_export::{
    driver_add_coefficient, driver_coefficient_count, driver_evaluate, driver_has_expression,
    driver_push_variable, driver_to_json as driver_data_to_json, driver_type_name, driver_validate,
    driver_variable_count, new_driver_data, new_driver_export, DriverData, DriverExport,
    DriverType,
};

pub mod render_pass_export;
pub use render_pass_export::{
    add_render_pass, new_render_pass, new_render_pass_export, pass_get_pixel, pass_mean,
    pass_set_pixel, pass_to_bytes, RenderPass, RenderPassEntry, RenderPassExport,
};

pub mod grease_pencil_export;
pub use grease_pencil_export::{
    gp_point_count, gp_push_point, gp_stroke_length, gp_stroke_to_json, gp_strokes_to_json,
    new_gp_stroke, new_grease_pencil_export, GpLayer, GpStroke, GreasePencilExport,
};

pub mod collection_export;
pub use collection_export::{
    collection_child_count, collection_object_count, collection_push_child, collection_push_object,
    collection_to_json, new_collection_export, new_collection_node, CollectionExport,
    CollectionNode, CollectionObject,
};

pub mod compositor_export;
pub use compositor_export::{
    comp_node_is_output, comp_node_to_json, comp_nodes_to_json, comp_push_input, comp_push_output,
    default_compositor_export, export_compositor_to_json, new_compositor_node, CompositorExport,
    CompositorNode,
};

// --- Wave 151B additions ---
pub mod gltf2_export;
pub use gltf2_export::{
    asset_to_json, gltf2_scene_json, new_gltf2_asset, new_gltf2_node, node_to_json, nodes_to_json,
    Gltf2Asset, Gltf2Node,
};

pub mod vrm_export;
pub use vrm_export::{
    VrmBoneName, VrmCommercialUsage as VrmCommercialUsage10, VrmCreditNotation, VrmExporter,
    VrmHumanBone, VrmHumanoid as VrmHumanoid10, VrmMeta as VrmMeta10, VrmModification,
};

pub mod openxr_export;
pub use openxr_export::{
    new_xr_skeleton, xr_is_hand_skeleton, xr_joint_count, xr_push_joint, xr_to_json, XrSkeleton,
};

pub mod mixamo_export;
pub use mixamo_export::{
    mixamo_bone_to_json, mixamo_is_standard_bone, mixamo_rig_to_json, mixamo_standard_bones,
    new_mixamo_bone, MixamoBone,
};

pub mod smpl_export;
pub use smpl_export::{
    new_smpl_params, smpl_gender_name, smpl_param_count, smpl_set_beta, smpl_set_pose,
    smpl_to_json, SmplParams,
};

pub mod mediapipe_export;
pub use mediapipe_export::{
    new_mediapipe_landmark, new_mediapipe_pose, pose_is_complete, pose_landmark_name,
    pose_push_landmark, pose_to_json, MediapipeLandmark, MediapipePose,
};

pub mod openpose_export;
pub use openpose_export::{
    body_is_valid_coco, body_push_keypoint, body_to_json, keypoint_name_coco, new_openpose_body,
    new_openpose_keypoint, OpenPoseBody, OpenPoseKeypoint,
};

pub mod daz3d_export;
pub use daz3d_export::{
    daz_is_genesis8, daz_morph_count, daz_push_morph, daz_to_json, new_daz_figure, DazFigureExport,
};

pub mod makehuman_export;
pub use makehuman_export::{
    mh_find_param, mh_param_count, mh_push_param, mh_to_mhm_string, new_mh_export, MhExport,
    MhMorphParam,
};

pub mod cmumotion_export;
pub use cmumotion_export::{
    cmu_duration_s, cmu_frame_count, cmu_push_frame, cmu_to_csv, new_cmu_motion, CmuFrame,
    CmuMotion,
};

pub mod h36m_export;
pub use h36m_export::{
    h36m_joint_count, h36m_joint_name, h36m_push_joint, h36m_to_csv_line, new_h36m_skeleton,
    H36mSkeleton,
};

pub mod panoptic_export;
pub use panoptic_export::{
    new_panoptic_body, panoptic_is_body25, panoptic_keypoint_count, panoptic_push_keypoint,
    panoptic_to_json, PanopticBody,
};

pub mod smplx_export;
pub use smplx_export::{
    new_smplx_params, smplx_num_betas, smplx_num_expression, smplx_set_expression, smplx_to_json,
    SmplxParams,
};

pub mod flame_export;
pub use flame_export::{
    flame_expression_count, flame_set_shape, flame_shape_count, flame_to_json, new_flame_params,
    FlameParams,
};

pub mod tdmm_export;
pub use tdmm_export::{
    new_tdmm_params, tdmm_exp_count, tdmm_id_count, tdmm_reconstruct_stub, tdmm_to_json, TdmmParams,
};

pub mod dense_pose_export;
pub use dense_pose_export::{
    dense_pose_coverage, dense_pose_get, dense_pose_set, dense_pose_to_bytes,
    new_dense_pose_result, DensePoseResult,
};

pub mod three_js_export;
pub use three_js_export::{
    add_geometry, add_object, export_threejs_to_json, new_three_js_scene, ThreeJsGeometry,
    ThreeJsObject, ThreeJsScene,
};

pub mod babylon_export;
pub use babylon_export::{
    babylon_material_count, babylon_mesh_count, new_babylon_export, to_babylon_json, BabylonExport,
    BabylonMaterial, BabylonMesh,
};

pub mod a_frame_export;
pub use a_frame_export::{
    aframe_add_box, aframe_add_sphere, aframe_entity_count, aframe_push_entity,
    export_mesh_as_aframe, new_aframe_scene, render_aframe_html, validate_aframe, AFrameEntity,
    AFrameScene,
};

pub mod openscad_export;
pub use openscad_export::{
    export_mesh_as_openscad, new_openscad_export, openscad_node_count, openscad_push_node,
    render_openscad, validate_openscad, OpenScadExport, ScadNode, ScadPrim,
};

pub mod scad_export;
pub use scad_export::{
    new_scad_export, render_scad, scad_add_box, scad_add_cylinder, scad_add_metadata,
    scad_add_sphere, scad_bounding_box, scad_prim_count, validate_scad, ScadExport, ScadPrimType,
    ScadPrimitive,
};

pub mod dotobj_export;
pub use dotobj_export::{
    dotobj_add_material, dotobj_face_count, dotobj_material_count, dotobj_set_mesh,
    new_dotobj_export, render_dotmtl, render_dotobj, validate_dotobj, DotObjExport, DotObjMaterial,
};

pub mod ply_binary_export;
pub use ply_binary_export::{
    export_ply_binary, new_ply_binary_export, ply_binary_face_bytes, ply_binary_face_count,
    ply_binary_header, ply_binary_set_mesh, ply_binary_size_estimate, ply_binary_vertex_bytes,
    ply_binary_vertex_count, validate_ply_binary, PlyBinaryExport,
};

pub mod wrl_export;
pub use wrl_export::{
    export_mesh_as_wrl, new_wrl_document, render_wrl, validate_wrl, wrl_add_shape, wrl_shape_count,
    wrl_size_estimate, wrl_total_vertex_count, WrlDocument, WrlShape,
};

pub mod x_export;
pub use x_export::{
    new_x_document, render_x_document, validate_x_document, x_add_mesh, x_file_header,
    x_mesh_count, x_mesh_from_geometry, x_size_estimate, x_total_face_count, x_total_vertex_count,
    XDocument, XMesh,
};

pub mod ac3d_export;
pub use ac3d_export::{
    ac3d_add_material, ac3d_add_mesh, ac3d_material_count, ac3d_object_count, ac3d_size_estimate,
    new_ac3d_export, render_ac3d, validate_ac3d, Ac3dExport, Ac3dMaterial, Ac3dObject, Ac3dSurface,
};

pub mod lwo_export;
pub use lwo_export::{
    lwo_add_layer, lwo_add_surface, lwo_form_header, lwo_layer_count, lwo_size_estimate,
    lwo_surface_count, lwo_total_polygon_count, lwo_total_vertex_count, new_lwo_export,
    render_lwo_summary, validate_lwo, LwoExport, LwoLayer, LwoSurface,
};

pub mod nff_export;
pub use nff_export::{
    new_nff_document, nff_add_light, nff_add_mesh, nff_add_polygon, nff_add_sphere,
    nff_light_count, nff_polygon_count, nff_primitive_count, nff_set_background, nff_size_estimate,
    nff_sphere_count, render_nff, validate_nff, NffDocument, NffLight, NffPolygon, NffSphere,
    NffSurface,
};

pub mod markdown_report_export;
pub use markdown_report_export::{
    add_md_row as report_add_md_row, add_md_section, default_body_report as default_body_md_report,
    export_markdown_report, markdown_byte_count, md_row_count as report_md_row_count,
    md_section_count, new_markdown_report, render_markdown, validate_markdown_report,
    MarkdownReport, MarkdownSection,
};

pub mod html_report_export;
pub use html_report_export::{
    add_measurement as html_add_measurement, add_note as html_add_note, default_html_body_report,
    export_html_body_report, html_report_size_bytes, measurement_count as html_measurement_count,
    new_html_body_report, note_count as html_note_count, render_html_report, validate_html_report,
    HtmlBodyReport,
};

pub mod dot_graph_export;
pub use dot_graph_export::{
    dot_size_bytes, export_dot, new_dot_graph, render_dot, scene_to_dot, validate_dot_graph,
    DotGraph,
};

pub mod svg_skeleton_export;
pub use svg_skeleton_export::{
    add_svg_bone, bone_length_px, default_biped_svg_skeleton, export_svg_skeleton,
    new_svg_skeleton_doc, render_svg_skeleton, svg_bone_count, svg_skeleton_size_bytes,
    validate_svg_skeleton_doc, SvgBone, SvgSkeletonDoc,
};

pub mod pdf_metadata_export;
pub use pdf_metadata_export::{
    default_pdf_metadata_for_body_report, new_pdf_metadata, pdf_add_keyword, pdf_keyword_count,
    pdf_metadata_to_info_dict, pdf_metadata_to_xmp, pdf_set_page_count, pdf_set_subject,
    validate_pdf_metadata, PdfMetadata,
};

pub mod toml_config_export;
pub use toml_config_export::{
    default_export_config_toml, export_toml, new_toml_table, render_toml_table, toml_add_sub_table,
    toml_entry_count, toml_set_bool, toml_set_float, toml_set_int, toml_set_string,
    toml_size_bytes, validate_toml_table, TomlTable, TomlValue,
};

pub mod ron_export;
pub use ron_export::{
    export_ron, render_ron, ron_bool, ron_float, ron_int, ron_list, ron_list_len, ron_map,
    ron_map_get, ron_none, ron_size_bytes, ron_some, ron_str, ron_struct, scene_to_ron,
    validate_ron_value, RonValue,
};

pub mod binary_stl_export;
pub use binary_stl_export::{
    add_binary_stl_triangle, binary_stl_size_bytes, binary_stl_triangle_count, encode_binary_stl,
    mesh_to_binary_stl, new_binary_stl_mesh, parse_binary_stl_header, validate_binary_stl,
    BinaryStlMesh, BinaryStlTriangle,
};

pub mod ascii_stl_export;
pub use ascii_stl_export::{
    ascii_stl_size_bytes, count_ascii_stl_triangles, default_ascii_stl_options, export_ascii_stl,
    parse_ascii_stl_vertices, render_ascii_stl, validate_ascii_stl, AsciiStlOptions,
};

pub mod opensim_export;
pub use opensim_export::{
    add_opensim_body, add_opensim_joint, add_opensim_muscle, default_biped_opensim_model,
    export_opensim, new_opensim_model, opensim_body_count, opensim_muscle_count,
    opensim_size_bytes, render_opensim_xml, validate_opensim_model, OpenSimBody, OpenSimJoint,
    OpenSimModel, OpenSimMuscle,
};

pub mod opensim_ik_export;
pub use opensim_ik_export::{
    add_ik_coordinate_task, add_ik_marker_task, default_ik_setup, export_opensim_ik,
    ik_coordinate_task_count, ik_marker_task_count, ik_set_time_range, new_opensim_ik_setup,
    opensim_ik_size_bytes, render_opensim_ik_xml, validate_opensim_ik, IkCoordinateTask, IkMarker,
    OpenSimIkSetup,
};
