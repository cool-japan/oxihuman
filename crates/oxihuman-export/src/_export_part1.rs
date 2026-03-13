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
pub mod alembic_ogawa_core;
pub mod alembic_ogawa_io;
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
pub mod usda_export;
pub use usda_export::{
    BlendShapeTimeSamples, UsdBlendShape, UsdSkinBinding, UsdSkeleton, UsdSubdivScheme, UsdaWriter,
    UsdMaterial as UsdaWriterMaterial, UsdMesh as UsdaWriterMesh,
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

pub mod fbx_binary;
pub use fbx_binary::{export_mesh_fbx_binary, FbxBinaryWriter, FbxNode as FbxBinaryNode, FbxProperty};

pub mod fbx_stub;
#[allow(deprecated)]
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

