pub mod lighting;
pub mod post_process;
pub use lighting::{
    kelvin_to_rgb, normalize_dir, srgb_to_linear_color, AmbientLight, DirectionalLight,
    HdriDescriptor, LightingSetup, PointLight,
};
pub use post_process::{
    apply_tone_map, luminance, tone_map_aces_approx, tone_map_linear, tone_map_reinhard,
    BloomConfig, FxaaConfig, PostProcessPipeline, SsaoConfig, ToneMapMethod, ToneMappingConfig,
};

pub mod debug_draw;
pub mod render_pass;
pub use debug_draw::{
    debug_draw_to_json, draw_aabb, draw_mesh_normals, draw_physics_proxies_debug, draw_skeleton,
    DebugArrow, DebugDrawList, DebugLine, DebugSphere,
};
pub use render_pass::{
    attachment_format_bytes, depth_format_bytes, is_depth_only, render_pass_summary,
    total_attachment_count, AttachmentFormat, ColorAttachment, DepthAttachment, DepthFormat,
    LoadOp, RenderPassDescriptor, StoreOp,
};

pub mod texture_cache;
pub use texture_cache::{
    default_placeholder_texture, texture_format_bytes, texture_memory_bytes, TextureCache,
    TextureDescriptor, TextureEntry, TextureFilter, TextureFormat, TextureWrap,
};
pub mod environment;
pub use environment::{
    environment_to_json, fog_factor, sky_color_at_angle, sun_direction_from_angles,
    AtmosphereParams, EnvironmentDescriptor, SkyGradient, SkyModel,
};

pub mod camera_rig;
pub use camera_rig::{
    default_orbit_camera, look_at_matrix, perspective_matrix, CameraMode, CameraRig, FlyState,
    OrbitState,
};
pub mod selection;
pub use selection::{raycast_select, SelectionBuffer, SelectionItem, SelectionKind};

pub mod overlay_renderer;
pub use overlay_renderer::{
    crosshair_overlay, default_hud_overlay, fps_counter_element, text_element, OverlayAnchor,
    OverlayColor, OverlayElement, OverlayLayer, OverlayRenderer,
};
pub mod lod_manager;
pub use lod_manager::{
    lod_blend_factor, lod_stats_json, select_lod_level, standard_lod_levels, LodLevel, LodManager,
    LodState,
};
pub mod shader_library;
pub use shader_library::{
    compile_variant, default_pbr_shaders, get_shader, list_shaders, new_shader_library,
    register_include, register_shader, remove_shader, resolve_includes, shader_count, shader_hash,
    validate_shader_entry, ShaderEntry, ShaderLanguage, ShaderLibrary, ShaderStage, ShaderVariant,
};
pub mod scene_compositor;
pub use scene_compositor::*;

pub mod viewport_grid;
pub use viewport_grid::*;

pub mod gizmo;
pub use gizmo::{
    drag_gizmo, gizmo_handles, gizmo_world_matrix, new_gizmo, pick_gizmo_axis, reset_gizmo,
    rotate_gizmo, scale_gizmo, set_gizmo_mode, snap_rotate, snap_scale, snap_translate,
    translate_gizmo, GizmoAxis, GizmoDragResult, GizmoHandle, GizmoMode, GizmoState,
};

pub mod annotation;
pub use annotation::{
    add_arrow, add_highlight, add_label, add_measurement as add_measurement_annotation,
    annotation_count, clear_layer, get_annotation, measurement_length, measurement_midpoint,
    new_layer as new_annotation_layer, project_to_screen, remove_annotation, set_layer_visible,
    visible_annotations, Annotation, AnnotationLayer, AnnotationType, MeasurementAnnotation,
};
pub mod camera_presets;
pub use camera_presets::{
    add_custom_preset, camera_forward, camera_right, default_preset_library, get_preset,
    interpolate_views, orbit_view, preset_count, preset_name, preset_view,
    projection_matrix as camera_projection_matrix, view_matrix as camera_view_matrix, zoom_view,
    CameraPreset, CameraView, PresetLibrary,
};

pub mod render_debug;
pub use render_debug::{
    clear_debug_draw, draw_aabb as render_draw_aabb, draw_line, draw_normal_vectors, draw_point,
    draw_skeleton as render_draw_skeleton, draw_sphere_wireframe, draw_text, draw_wireframe,
    line_count, new_debug_draw, set_enabled as set_debug_enabled, total_primitive_count, DebugDraw,
    DebugLine as RenderDebugLine, DebugPoint, DebugText,
};

pub mod timeline_viewer;
pub use timeline_viewer::{
    add_key_to_track, add_track as add_timeline_track, frame_to_time, get_track, keys_in_range,
    new_timeline, pause, play, remove_key_from_track, remove_track, set_current_time, step_frame,
    time_to_frame, track_count, zoom_timeline, TimelineSelection, TimelineTrack, TimelineView,
};

pub mod stats_overlay;
pub use stats_overlay::{
    average_draw_calls, average_fps, average_frame_time, average_triangle_count,
    clear_history as clear_stats_history, format_stats_text, fps_from_frame_time, history_len,
    last_frame, max_fps, min_fps, new_stats_overlay, peak_frame_time, push_frame_stats, FrameStats,
    StatsOverlay,
};

pub mod world_axes;
pub use world_axes::{
    axes_line_count, axes_to_screen_lines, axis_color, axis_label, build_axis_lines,
    new_world_axes, rotate_axes, scale_axes as scale_world_axes, set_axes_visible,
    set_show_negative, update_axes_from_view, world_to_screen_axes, AxisLine, WorldAxes,
};

pub mod wireframe_overlay;
pub use wireframe_overlay::{
    boundary_edges_wire, build_wireframe, classify_crease_edges, default_wireframe_config,
    edge_count_wire, extract_edges as wireframe_extract_edges, filter_crease_only,
    merge_wireframes, set_wireframe_color, visible_edge_count, wireframe_aabb,
    wireframe_line_segments, WireEdge, WireframeConfig, WireframeOverlay,
};

pub mod xr_viewport;
pub use xr_viewport::{
    build_eye_views, disable_xr, enable_xr, eye_count, eye_projection_matrix, eye_view_matrix,
    new_xr_viewport, set_xr_pose, xr_ipd_offset, xr_render_resolution, xr_reprojection_matrix,
    xr_viewport_for_eye, XrEye, XrEyeView, XrPose, XrViewport,
};

pub mod color_picker;
pub use color_picker::{
    color_distance, complementary_color, hex_to_rgb, hsv_to_rgb, lerp_color, linear_to_rgb,
    new_color_picker, rgb_to_hex, rgb_to_hsv, rgb_to_linear, set_alpha, set_hex as picker_set_hex,
    set_hsv as picker_set_hsv, set_rgb as picker_set_rgb, ColorPickerState, ColorSpace,
};

pub mod measurement_display;
pub use measurement_display::{
    add_measurement as add_body_measurement, convert_to_unit, display_value,
    get_measurement_by_name, measurement_count, measurement_line_segments, measurement_to_string,
    new_measurement_display, set_unit as set_measurement_unit, standard_body_measurements,
    total_display_range, unit_suffix, visible_measurements, BodyMeasurement, MeasurementDisplay,
    MeasurementUnit,
};

pub mod ambient_occlusion_preview;
pub use ambient_occlusion_preview::{
    ao_average, ao_hemisphere_sample, ao_max, ao_min, ao_to_grayscale, ao_to_vertex_color,
    apply_ao_power, combine_ao_buffers, compute_ao_cpu, default_ao_config, new_ao_buffer,
    ray_intersects_triangle as ao_ray_intersects, smooth_ao, AoBuffer, AoConfig,
};

pub mod focus_point;
pub use focus_point::{
    average_focus, disable_dof, dof_far_plane, dof_near_plane, enable_dof, focus_history_len,
    focus_in_range, new_focus_history, new_focus_point, push_focus, set_focus_mode,
    set_focus_position, smooth_focus_to, update_focus_distance, FocusHistory, FocusMode,
    FocusPoint,
};

pub mod light_probe;
pub use light_probe::{
    add_light_probe, blend_probes, default_sh_coefficients, get_light_probe,
    light_probe_set_to_json, nearest_probe, new_light_probe_set, probe_count,
    probe_influence_weight, remove_light_probe as remove_light_probe_entry, sample_probe_sh,
    set_probe_enabled, set_probe_sh, LightProbe, LightProbeSet, ProbeType,
};

pub mod particle_renderer;
pub use particle_renderer::{
    alive_particle_count, average_particle_age, billboard_corners, clear_particles, emit_particle,
    is_particle_alive, new_particle_system, particle_count, particle_normalized_age,
    particle_uv_frame, particles_as_quads, sort_particles_by_depth, update_particles,
    ParticleBlend, ParticleSystem, RenderParticle,
};

pub mod motion_trail;
pub use motion_trail::{
    add_trail_point, clear_trail, new_motion_trail, newest_trail_point, oldest_trail_point,
    trail_aabb, trail_color_at, trail_enabled, trail_length, trail_point_count, trail_segments,
    trail_width_at, update_trail, MotionTrail, TrailPoint, TrailSegment,
};

pub mod grid_overlay;
pub use grid_overlay::{
    build_grid_overlay, default_grid_overlay_config, grid_cell_at_2d, grid_line_count_2d,
    grid_lines_in_rect, grid_spacing_pixels, major_line_count_2d, nearest_grid_point,
    set_grid_color, set_grid_visible, snap_to_grid_2d, update_grid_overlay, GridLine2D,
    GridOverlay, GridOverlayConfig,
};

pub mod icon_renderer;
pub use icon_renderer::{
    all_icon_types, build_icon_atlas, get_icon_glyph, icon_bounds, icon_glyph_arrow,
    icon_glyph_check, icon_glyph_circle, icon_glyph_cross, icon_index_count, icon_type_name,
    icon_vertex_count, render_icon, transform_icon, IconAtlas, IconGlyph, IconType, RenderedIcon,
};

pub mod camera_animation;
pub use camera_animation::{
    add_keyframe, advance_camera_anim, blend_tracks, camera_at_end, camera_at_start,
    keyframe_count, nearest_keyframe, new_camera_track, remove_keyframe, reverse_track,
    sample_track, set_looping, track_duration, track_to_json, CameraAnimState, CameraKeyframe,
    CameraTrack,
};

pub mod background_renderer;
pub use background_renderer::{
    background_pixel_grid, background_to_json, background_type_name, checkerboard_color,
    default_background_config, gradient_color, is_dynamic_background, new_background_config,
    sample_background, set_background_type, set_bottom_color, set_checker_size, set_top_color,
    BackgroundConfig, BackgroundSample, BackgroundType,
};

pub mod split_view;
pub use split_view::{
    active_pane, default_split_view_config, layout_name, new_split_view, pane_at_position,
    pane_count, pane_rect, resize_split, set_active_pane, set_layout, split_ratio,
    split_view_to_json, toggle_maximize_pane, SplitLayout, SplitViewConfig, ViewportPane,
};

pub mod tooltip_renderer;
pub use tooltip_renderer::{
    advance_tooltip, clear_all_tooltips, default_tooltip_config, hide_tooltip, new_tooltip,
    set_tooltip_text, show_tooltip, tooltip_at_screen_pos, tooltip_background_color,
    tooltip_bounds, tooltip_fade_alpha, tooltip_text_color, tooltip_visible,
    update_tooltip_position, Tooltip, TooltipAnchor, TooltipConfig,
};

pub mod ruler_tool;
pub use ruler_tool::{
    add_ruler_point, clear_ruler, compute_angle_deg, compute_circumference, compute_distance,
    new_ruler_tool, remove_last_point, ruler_measurement_text, ruler_point_count, ruler_to_json,
    ruler_total_length, ruler_unit_label, set_ruler_mode, set_ruler_scale, RulerMeasurement,
    RulerMode, RulerPoint, RulerTool,
};

pub mod histogram_view;
pub use histogram_view::{
    build_histogram, cumulative_histogram, default_histogram_config, histogram_bin_count,
    histogram_entropy, histogram_max_count, histogram_mean, histogram_median_bin,
    histogram_percentile, histogram_to_ascii, histogram_to_json, merge_histograms,
    normalize_histogram, Histogram, HistogramBin, HistogramConfig,
};

pub mod depth_of_field;
pub use depth_of_field::{
    blur_radius_at_depth, compute_coc, default_dof_config, dof_far_plane as dof_far_plane_cfg,
    dof_near_plane as dof_near_plane_cfg, dof_sample_at_depth, dof_to_json, focus_region_extent,
    is_in_focus, new_dof_config, set_aperture, set_focal_distance, set_focal_length, DofConfig,
    DofFocusRegion, DofMode, DofSample,
};

pub mod lens_overlay;
pub use lens_overlay::{
    blend_lens_configs, chromatic_aberration_offset, default_lens_effect_config,
    flare_streak_positions, is_flare_visible, lens_flare_count, lens_flare_intensity,
    lens_overlay_to_json, new_lens_flare, set_flare_threshold, set_vignette_strength,
    vignette_alpha, vignette_mask_pixel, ChromaticOffsets, LensEffectConfig, LensFlare,
    StreakPositions, VignetteConfig,
};

pub mod alpha_blend;
pub use alpha_blend::{
    alpha_blend_mode_name, apply_blend, blend_additive, blend_multiply, blend_normal, blend_screen,
    new_rgba, premultiply_alpha, rgba_to_json, AlphaBlendMode, Rgba,
};

pub mod camera_dolly;
pub use camera_dolly::{
    add_dolly_point, dolly_distance, dolly_duration, dolly_evaluate, dolly_point_count,
    dolly_reset, dolly_to_json, new_camera_dolly, CameraDolly,
};

pub mod compute_pass;
pub use compute_pass::{
    compute_pass_to_json, default_workgroup_size, dispatch_groups_for_elements, is_power_of_two,
    new_compute_pass, set_dispatch, set_workgroup, total_invocations, validate_compute_pass,
    ComputePassDesc, WorkgroupSize,
};

pub mod cubemap_preview;
pub use cubemap_preview::{
    cross_layout_size, cubemap_preview_to_json, default_cubemap_preview_config,
    direction_to_spherical, face_cross_position, face_name, face_pixel_offset,
    roughness_to_mip as cubemap_roughness_to_mip, texel_solid_angle, uv_to_direction, CubeFace,
    CubemapPreviewConfig,
};

pub mod decal_projector;
pub use decal_projector::{
    decal_to_json, decal_volume, new_decal_projector, point_in_decal_box, project_to_decal_uv,
    reset_decal, set_decal_enabled, set_decal_extents, set_decal_opacity, DecalProjector,
};

pub mod depth_linearize;
pub use depth_linearize::{
    default_depth_params, depth_params_to_json, depth_precision, depth_range, depth_to_ndc,
    is_reversed_z, linearize_buffer, linearize_depth, DepthLinearizeParams,
};

pub mod draw_state;
pub use draw_state::{
    default_draw_state, draw_state_to_json, reset_draw_state, set_blend_enabled, set_cull_face,
    set_depth_func, set_depth_test, set_depth_write, state_change_count, CullFace,
    DepthFunc as DrawDepthFunc, DrawState,
};

pub mod edge_detect_post;
pub use edge_detect_post::{
    default_edge_detect_config, diagonal_weight, edge_detect_to_json, edge_method_name, is_edge,
    laplacian_magnitude, roberts_magnitude, set_edge_threshold, sobel_magnitude, EdgeDetectConfig,
    EdgeMethod,
};

pub mod env_cubemap;
pub use env_cubemap::{
    env_cubemap_to_json, env_rotation_rad, estimated_memory_bytes, new_env_cubemap,
    reset_env_cubemap, set_env_enabled, set_env_intensity, set_env_rotation,
    total_texel_count as env_total_texel_count, EnvCubemap,
};

pub mod film_strip;
pub use film_strip::{
    capture_frame, clear_film_strip, film_strip_to_json, frame_at_time,
    frame_count as film_frame_count, new_film_strip, set_fps, time_per_frame, total_duration,
    FilmFrame, FilmStrip,
};

pub mod gpu_buffer;
pub use gpu_buffer::{
    buffer_clear, buffer_is_mapped, buffer_read, buffer_size_gpu, buffer_to_json, buffer_usage,
    buffer_write, new_gpu_buffer, BufferUsage, GpuBuffer,
};

pub mod indirect_specular;
pub use indirect_specular::{
    default_indirect_specular_config, fresnel_schlick, fresnel_schlick_roughness, ggx_distribution,
    indirect_specular_to_json, roughness_to_mip_level, set_indirect_intensity,
    set_roughness_offset, IndirectSpecularConfig,
};

pub mod light_attenuation;
pub use light_attenuation::{
    attenuation_model_name, attenuation_to_json, compute_attenuation, default_light_attenuation,
    luminous_intensity, set_radius as set_attenuation_radius, spot_attenuation, AttenuationModel,
    LightAttenuation,
};

pub mod object_id_pass;
pub use object_id_pass::{
    color_to_id, id_to_color, lookup_by_name, lookup_object, new_object_id_pass,
    object_count as id_object_count, object_id_pass_to_json, register_object, remove_object,
    set_object_visible, ObjectIdEntry, ObjectIdPass,
};

pub mod pixel_grid;
pub use pixel_grid::{
    default_pixel_grid_config, grid_line_alpha, grid_line_count as pixel_grid_line_count,
    pixel_grid_to_json, set_grid_line_color, set_grid_line_width, set_zoom_threshold,
    should_show_grid, snap_to_pixel, PixelGridConfig,
};

pub mod stencil_mask;
pub use stencil_mask::{
    apply_stencil_op, default_stencil_mask, evaluate_stencil, stencil_mask_to_json,
    stencil_test_equal, stencil_test_not_equal, stencil_write_mask, StencilFunc, StencilMask,
    StencilOp,
};

pub mod bent_normal_debug;
pub use bent_normal_debug::{
    add_bent_normal, bent_normal_angle, bent_normal_count, bent_normal_debug_to_json,
    bent_normal_to_color, clear_bent_normals, default_bent_normal_config, filtered_entries,
    new_bent_normal_debug, normalize_bn, set_bent_normal_enabled, BentNormalConfig,
    BentNormalDebug, BentNormalEntry, BentNormalMode,
};

pub mod cluster_light;
pub use cluster_light::{
    add_cluster_light, cluster_depth_slice, cluster_light_count, cluster_light_to_json,
    enabled_light_count as cluster_enabled_light_count, light_influence, new_cluster_light_manager,
    remove_cluster_light, total_tile_count, ClusterConfig, ClusterLight, ClusterLightManager,
};

pub mod cubemap_filter;
pub use cubemap_filter::{
    build_mip_chain, cubemap_filter_to_json, diffuse_irradiance_approx, estimated_filter_memory,
    ggx_d, mip_count, mip_to_roughness, new_cubemap_filter, roughness_to_mip, solid_angle_estimate,
    CubeFaceFilter, CubemapFilter, CubemapFilterConfig, MipEntry,
};

pub mod decal_batch;
pub use decal_batch::{
    add_decal_instance, clear_decal_batch, decal_batch_memory_bytes, decal_batch_to_json,
    decal_instance_count, enabled_decal_count, new_decal_batch, remove_decal_instance,
    sort_decals_by_layer, sub_batch_count, DecalBatch, DecalBatchConfig, DecalInstance,
};

pub mod depth_range;
pub use depth_range::{
    compute_precision_stats, default_depth_range, depth_range_ratio, depth_range_to_json,
    dr_set_planes, dr_set_reversed, is_valid_depth, linearize_depth_value, ndc_to_view_depth,
    view_to_ndc_depth, DepthPrecisionStats, DepthRange,
};

pub mod draw_command;
pub use draw_command::{
    clear_commands, command_count, draw_command_to_json, group_by_pipeline, is_recorder_full,
    new_draw_recorder, record_draw, sort_commands_ascending, sort_commands_descending,
    total_index_count as draw_total_index_count, total_instance_count, DrawCommand,
    DrawCommandRecorder, DrawPrimitive,
};

pub mod edge_fade;
pub use edge_fade::{
    default_edge_fade_config, edge_fade_alpha, edge_fade_to_json, is_in_fade_zone, rim_fade,
    screen_edge_fade, EdgeFadeConfig, EdgeFadeMode,
};

pub mod env_diffuse;
pub use env_diffuse::{
    add_probe as add_env_probe, ambient_color, blend_probes_env, env_diffuse_to_json,
    hemisphere_pdf, new_env_diffuse, probe_count_env, sample_sh_irradiance, EnvDiffuse,
    EnvDiffuseConfig, EnvDiffuseProbe, SH_COEFF_COUNT,
};

pub mod film_tone;
pub use film_tone::{
    apply_film_tone, apply_gamma, default_film_tone_config, film_tone_to_json, tone_aces_filmic,
    tone_hable, tone_linear, tone_logarithmic, tone_reinhard, FilmToneConfig, FilmToneOp,
};

pub mod gpu_timeline;
pub use gpu_timeline::{
    average_duration_ms, begin_frame, clear_timeline, frame_total_ns, gpu_timeline_to_json,
    has_spike, new_gpu_timeline, record_sample as record_gpu_sample,
    sample_count as gpu_sample_count, slowest_pass, GpuTimeSample, GpuTimeline, GpuTimelineConfig,
};

pub mod instanced_mesh;
pub use instanced_mesh::{
    add_instance, clear_instances, enabled_instance_count, instance_count, instance_memory_bytes,
    instanced_mesh_to_json, new_instanced_mesh, remove_instance, rotation_y, set_all_lod,
    translation_matrix, InstancedMesh, MeshInstance,
};

pub mod light_cookie;
pub use light_cookie::{
    add_cookie, clear_cookies, cone_half_angle_rad, cookie_count, cookie_to_json,
    enabled_cookie_count as cookie_enabled_count, new_cookie_manager, remove_cookie,
    sample_cookie_uv, spot_uv_scale, CookieProjection, LightCookie, LightCookieManager,
};

pub mod motion_blur_tile;
pub use motion_blur_tile::{
    blur_tile_count, build_tile_grid, motion_blur_tile_to_json, new_motion_blur_manager,
    sample_count_for_velocity, set_tile_velocity, tile_count, BlurTile, MotionBlurTileConfig,
    MotionBlurTileManager, TileVelocity,
};

pub mod object_mask;
pub use object_mask::{
    add_layer as add_object_layer, camera_visible, new_object_mask_registry, object_count_mask,
    object_mask_to_json, register_object_mask, remove_layer as remove_object_layer,
    remove_object_mask, set_object_mask, visible_object_count, MaskBits, ObjectMaskEntry,
    ObjectMaskRegistry,
};

pub mod render_feature;
pub use render_feature::{
    clear_features, disable_feature, enable_feature, feature_count_rf, feature_name,
    features_to_json, is_feature_enabled, new_render_feature, FeatureFlag, RenderFeature,
};

pub mod ambient_occlusion_v2;
pub use ambient_occlusion_v2::{
    ao_v2_blend_configs, ao_v2_estimated_cost, ao_v2_hemisphere_dir, ao_v2_set_intensity,
    ao_v2_set_radius, ao_v2_set_samples, ao_v2_set_technique, ao_v2_technique_name, ao_v2_to_json,
    new_ao_v2_config, AoTechnique, AoV2Config,
};

pub mod camera_lens;
pub use camera_lens::{
    cl_blend, cl_distortion_factor, cl_dof_range, cl_fov_vertical_rad, cl_from_preset,
    cl_reference_angle, cl_set_aperture, cl_set_focal, cl_set_focus, cl_to_json, new_camera_lens,
    CameraLens, LensPreset,
};

pub mod cluster_visibility;
pub use cluster_visibility::{
    cv_clear, cv_fill_all_visible, cv_register, cv_set_light_count, cv_set_visible, cv_to_json,
    cv_total_clusters, cv_total_lights, cv_visible_count, new_cluster_visibility, ClusterIdx,
    ClusterVisEntry, ClusterVisibility,
};

pub mod cubemap_mip;
pub use cubemap_mip::{
    cm_build_mip_chain, cm_chain_to_json, cm_level_count, cm_level_to_roughness, cm_max_levels,
    cm_memory_bytes, cm_roughness_to_level, cm_texel_solid_angle, new_mip_chain, CubemapMipChain,
    CubemapMipLevel,
};

pub mod decal_layer;
pub use decal_layer::{
    dl_add_layer, dl_blend_mode_name, dl_layer_count, dl_remove_layer, dl_set_opacity,
    dl_set_visible, dl_sort_layers, dl_to_json, dl_visible_count, new_decal_layer_stack,
    DecalBlendMode, DecalLayerEntry, DecalLayerStack,
};

pub mod depth_stencil;
pub use depth_stencil::{
    ds_cmp_name, ds_enable_stencil, ds_format_bits, ds_format_name, ds_has_stencil, ds_reset,
    ds_set_depth_cmp, ds_to_json, new_depth_stencil, DepthCmp, DepthStencilFormat,
    DepthStencilState2, StencilOp2,
};

pub mod draw_indirect_v2;
pub use draw_indirect_v2::{
    di_clear, di_cull_zero_instance, di_indexed_byte_size, di_indexed_count, di_push_indexed,
    di_push_unindexed, di_to_json, di_total_index_count, di_total_instances,
    new_draw_indirect_buffer, DrawIndexedIndirect, DrawIndirect, DrawIndirectBuffer,
};

pub mod edge_normal;
pub use edge_normal::{
    en_add, en_average_dihedral, en_clear, en_count, en_crease_edges, en_find_by_vertices,
    en_max_dihedral, en_normalize, en_to_json, new_edge_normal_map, EdgeNormalEntry, EdgeNormalMap,
    MeshEdge,
};

pub mod env_light;
pub use env_light::{
    el_blend, el_effective_intensity, el_set_enabled, el_set_intensity, el_set_rotation_deg,
    el_set_source, el_set_tint, el_source_name, el_tinted_intensity, el_to_json, new_env_light,
    EnvLight, EnvLightSource,
};

pub mod film_grain_v2;
pub use film_grain_v2::{
    fg2_blend, fg2_grain_at_luma, fg2_is_disabled, fg2_pattern_name, fg2_set_intensity,
    fg2_set_luma_sensitivity, fg2_set_size, fg2_to_json, new_film_grain_v2, FilmGrainV2Config,
    GrainPattern,
};

pub mod gpu_profiler;
pub use gpu_profiler::{
    gp_average_ns, gp_begin_frame, gp_clear, gp_marker_count, gp_markers_in_frame, gp_max_ns,
    gp_record, gp_slowest_pass, gp_to_json, gp_total_ns, new_gpu_profiler, GpuMarker, GpuProfiler,
};

pub mod instance_cull;
pub use instance_cull::{
    ic_aabb_volume, ic_clear, ic_count, ic_cull_by_distance, ic_register, ic_remove,
    ic_set_visible, ic_to_json, ic_visible_count, new_instance_cull, Aabb, CullEntry, InstanceCull,
};

pub mod light_map;
pub use light_map::{
    lm_add_entry, lm_clear, lm_encoding_name, lm_entry_count, lm_get_entry, lm_memory_bytes,
    lm_remove_entry, lm_to_json, new_lightmap_atlas, LightMapAtlas, LightMapEncoding,
    LightMapEntry,
};

pub mod motion_vector;
pub use motion_vector::{
    mv_average_magnitude, mv_clear, mv_get, mv_max_magnitude, mv_pixel_count, mv_scale, mv_set,
    mv_to_json, MotionVector, MotionVectorBuffer,
};

pub mod occlusion_cull;
pub use occlusion_cull::{
    new_occlusion_cull, oc_clear, oc_count, oc_depth_cull, oc_mark_all_visible, oc_occluded_count,
    oc_register, oc_set_result, oc_to_json, oc_visible_count, OcclusionCull, OcclusionEntry,
    OcclusionResult,
};

pub mod render_graph;
pub use render_graph::{
    new_render_graph, rg_add_dep, rg_add_node, rg_clear, rg_dep_count, rg_enabled_count,
    rg_node_count, rg_pass_name, rg_set_enabled, rg_to_json, rg_topo_sort, PassKind, RenderGraph,
    RgNode,
};

pub mod alpha_coverage;
pub use alpha_coverage::{
    ac_average_pixels_per_draw, ac_clear_stats, ac_is_enabled, ac_record_draw, ac_sample_mask,
    ac_set_mode, ac_set_threshold, ac_to_json, default_alpha_coverage_config, AlphaCoverageConfig,
    AlphaCoverageMode, AlphaCoverageStats,
};

pub mod camera_pivot;
pub use camera_pivot::{
    cp_advance_transition, cp_distance_to, cp_is_at_origin, cp_move_by, cp_orbit_position,
    cp_reset, cp_set_position, cp_start_transition, cp_to_json, cp_transition_done,
    default_camera_pivot, CameraPivot, PivotTransition,
};

pub mod cluster_probe;
pub use cluster_probe::{
    cpr_add, cpr_clear, cpr_contains_point, cpr_count, cpr_enabled_count, cpr_find_for_point,
    cpr_remove, cpr_set_enabled, cpr_set_intensity, cpr_to_json, cpr_volume, new_cluster_probe,
    ClusterProbe, ClusterProbeSet,
};

pub mod color_lut;
pub use color_lut::{
    identity_color_lut, lut_apply_with_intensity, lut_data_len, lut_is_identity, lut_sample,
    lut_set_intensity, lut_to_json, ColorLut, LUT_DIM, LUT_SIZE,
};

pub mod decal_project;
pub use decal_project::{
    dp_add, dp_area, dp_clear, dp_count, dp_count_by_blend, dp_project_point, dp_remove,
    dp_set_blend, dp_set_opacity, dp_to_json, new_decal_instance, DecalBlend,
    DecalInstance as DpDecalInstance, DecalProjector as DpDecalProjector,
};

pub mod depth_bias;
pub use depth_bias::{
    db_estimated_bias_at_slope, db_is_neutral, db_reset, db_set_clamp, db_set_constant,
    db_set_enabled, db_set_slope_scale, db_to_json, default_depth_bias, shadow_map_depth_bias,
    DepthBiasConfig,
};

pub mod draw_call_sort;
pub use draw_call_sort::{
    dc_batch_count, dc_count_by_material, dc_sort, dc_sort_key, dc_split_opaque_transparent,
    dc_to_json_summary, new_draw_entry, DrawEntry, DrawSortKey, SortStrategy,
};

pub mod edge_silhouette;
pub use edge_silhouette::{
    default_silhouette_config, sil_count_silhouettes, sil_detect_edges, sil_is_silhouette_edge,
    sil_reset, sil_set_color, sil_set_enabled, sil_set_thickness, sil_to_json, SilhouetteConfig,
    SilhouetteEdge,
};

pub mod env_prefilter;
pub use env_prefilter::{
    default_env_prefilter_config, ep_ggx_ndf, ep_mark_ready, ep_memory_bytes, ep_resolution_at_mip,
    ep_roughness_for_mip, ep_set_sample_count, ep_to_json, new_prefilter_map, EnvPrefilterConfig,
    PrefilterMap, MAX_MIP_LEVELS,
};

pub mod film_response;
pub use film_response::{
    default_film_response, fr_aces, fr_apply_exposure, fr_ev_stops, fr_gamma_correct, fr_hejl,
    fr_process, fr_reinhard, fr_set_exposure, fr_set_gamma, fr_set_operator, fr_to_json,
    fr_tonemap, FilmResponseConfig, FilmicOperator,
};

pub mod gpu_marker;
pub use gpu_marker::{
    gm_clear, gm_current_label, gm_depth, gm_is_empty, gm_make_color, gm_path, gm_pop, gm_push,
    gm_to_json, GpuDebugMarker, GpuMarkerStack,
};

pub mod irradiance_cache;
pub use irradiance_cache::{
    ic_add_sample, ic_average_irradiance, ic_clear as irr_clear, ic_count as irr_count, ic_nearest,
    ic_tick, ic_to_json as irr_to_json, ic_valid_count, new_irradiance_cache, IrradianceCache,
    IrradianceSample,
};

pub mod light_falloff;
pub use light_falloff::{
    default_light_falloff, lfo_at_range_is_zero, lfo_attenuation, lfo_reset, lfo_set_model,
    lfo_set_range, lfo_spot_attenuation, lfo_to_json, FalloffModel, LightFalloffConfig,
};

pub mod motion_field;
pub use motion_field::{
    mf_average_magnitude, mf_clear, mf_get, mf_max_magnitude, mf_pixel_count, mf_scale, mf_set,
    mf_to_json, new_motion_field, MotionField, MotionVec2,
};

pub mod occlusion_map;
pub use occlusion_map::{
    default_occlusion_map_config, new_occlusion_buffer, om_apply_intensity, om_average, om_clear,
    om_get_pixel, om_set_intensity, om_set_pixel, om_set_radius, om_to_json, AoAlgorithm,
    OcclusionBuffer, OcclusionMapConfig,
};

pub mod render_layer;
pub use render_layer::{
    layer_clear, layer_draw_count, layer_is_visible, layer_name_rl, layer_priority, layer_to_json,
    new_render_layer_rl, set_layer_visible as set_render_layer_visible, LayerPriority, RenderLayer,
};

pub mod alpha_threshold;
pub use alpha_threshold::{
    at_blend, at_coverage_estimate, at_is_opaque, at_set_enabled, at_set_soft_range,
    at_set_threshold, at_soft_alpha, at_to_json, default_alpha_threshold_config,
    AlphaThresholdConfig,
};

pub mod camera_shake_v2;
pub use camera_shake_v2::{
    cs2_add_trauma, cs2_is_active, cs2_position_offset, cs2_reset, cs2_rotation_offset,
    cs2_to_json, cs2_update, default_camera_shake_v2_config, new_camera_shake_v2_state,
    CameraShakeV2Config, CameraShakeV2State,
};

pub mod cluster_shadow;
pub use cluster_shadow::{
    csm_add_entry, csm_clear, csm_enabled_count, csm_entry_count, csm_remove_entry,
    csm_set_enabled, csm_to_json, csm_total_clusters, default_cluster_shadow_config,
    new_cluster_shadow_manager, ClusterShadowConfig, ClusterShadowEntry, ClusterShadowManager,
};

pub mod color_space_convert;
pub use color_space_convert::{
    aces_cg_to_linear, color_space_name, convert_color_space, csc_luminance, linear_to_aces_cg,
    linear_to_srgb_channel, linear_to_srgb_rgb, srgb_to_linear_channel, srgb_to_linear_rgb,
    ColorSpace as CscColorSpace,
};

pub mod decal_uv;
pub use decal_uv::{
    default_decal_uv_config, default_decal_uv_transform, duv_is_in_bounds, duv_project_point,
    duv_set_rotation, duv_tile_uv_offset, duv_tile_uv_scale, duv_tiles_per_row, duv_to_json,
    DecalUvConfig, DecalUvTransform,
};

pub mod depth_clip;
pub use depth_clip::{
    dc_blend as depth_clip_blend, dc_depth_range, dc_is_in_range, dc_linearize, dc_precision_bits,
    dc_ratio, dc_set_planes as depth_clip_set_planes, dc_set_reversed,
    dc_to_json as depth_clip_to_json, default_depth_clip_config, DepthClipConfig,
};

pub mod draw_item;
pub use draw_item::{
    batch_add, batch_clear, batch_count as draw_batch_count, batch_count_by_kind,
    batch_sort_by_key, batch_to_json, batch_visible_count, di_kind_name, di_set_sort_key,
    di_set_visible, new_draw_item, DrawItem, DrawItemBatch, DrawItemKind,
};

pub mod edge_curvature;
pub use edge_curvature::{
    default_edge_curvature_config, ec_build_samples, ec_estimate_curvature, ec_is_crease,
    ec_scale_curvature, ec_set_enabled as ec_set_curvature_enabled, ec_set_radius,
    ec_set_threshold as ec_set_curvature_threshold, ec_to_json as ec_curvature_to_json,
    CurvatureSample, EdgeCurvatureConfig,
};

pub mod env_sky;
pub use env_sky::{
    default_env_sky_config, esky_model_name, esky_set_azimuth, esky_set_elevation,
    esky_set_intensity, esky_set_turbidity, esky_sky_color_at_angle, esky_sun_direction,
    esky_to_json, EnvSkyConfig, SkyModelType,
};

pub mod film_halation;
pub use film_halation::{
    default_film_halation_config, fhal_apply, fhal_blend, fhal_luminance, fhal_set_enabled,
    fhal_set_intensity, fhal_set_radius, fhal_set_threshold, fhal_set_tint, fhal_to_json,
    FilmHalationConfig,
};

pub mod gpu_perf;
pub use gpu_perf::{
    gperf_average_ns, gperf_begin_frame, gperf_clear, gperf_entry_count, gperf_frame_total_ns,
    gperf_max_ns, gperf_record, gperf_slowest_pass, gperf_to_json, gperf_total_ns, new_gpu_perf,
    GpuPerf, GpuPerfEntry,
};

pub mod irradiance_volume;
pub use irradiance_volume::{
    add_probe as add_irradiance_probe, irradiance_at_world_pos, new_irradiance_volume, probe_at,
    probe_count as irr_probe_count, probe_spacing, sample_irradiance, volume_bounds,
    IrradianceProbe, IrradianceVolume,
};

pub mod light_scatter;
pub use light_scatter::{
    default_light_scatter_config, lsc_blend, lsc_mie_phase, lsc_radial_weight, lsc_set_density,
    lsc_set_enabled, lsc_set_exposure, lsc_set_samples, lsc_to_json, LightScatterConfig,
};

pub mod motion_stretch;
pub use motion_stretch::{
    default_motion_stretch_config, ms_build_instance, ms_compute_stretch, ms_set_enabled,
    ms_set_max_stretch, ms_stretch_direction, ms_to_json, ms_velocity_magnitude,
    MotionStretchConfig, StretchInstance,
};

pub mod occlusion_probe;
