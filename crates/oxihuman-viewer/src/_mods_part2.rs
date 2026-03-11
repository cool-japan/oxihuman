pub use occlusion_probe::{
    new_occlusion_probe, op_average_occlusion, op_set_enabled as op_set_probe_enabled,
    op_set_occlusion, ops_add, ops_clear, ops_enabled_count, ops_nearest_probe, ops_probe_count,
    ops_to_json, OcclusionProbe, OcclusionProbeSet,
};

pub mod render_pass_v2;
pub use render_pass_v2::{
    new_render_pass_v2, rpl2_add, rpl2_clear, rpl2_enabled_count, rpl2_pass_count, rpl2_to_json,
    rpl2_total_draws, rpv2_increment_draws, rpv2_pass_type_name, rpv2_set_enabled,
    rpv2_set_timestamp, PassTypeV2, RenderPassListV2, RenderPassV2,
};

pub mod alpha_discard;
pub use alpha_discard::{
    ad_coverage_estimate, ad_mode_name, ad_set_mode, ad_set_soft_range, ad_set_threshold,
    ad_should_keep, ad_to_json, default_alpha_discard_config, new_alpha_discard,
    AlphaDiscardConfig, AlphaDiscardMode,
};

pub mod camera_track;
pub use camera_track::{
    ct_add_key, ct_arc_length, ct_duration, ct_full_circle_rad, ct_key_count, ct_remove_key,
    ct_sample, ct_set_looping, ct_to_json as camera_track_to_json,
    new_camera_track as new_camera_path_track, CameraTrack as CameraPathTrack, CameraTrackInterp,
    CameraTrackKey, CameraTrackSample,
};

pub mod cluster_cull;
pub use cluster_cull::{
    cc_clear, cc_count, cc_depth_slice, cc_flat_index, cc_half_fov_ref, cc_mark_all_visible,
    cc_set_visible, cc_to_json as cluster_cull_to_json, cc_visible_count, new_cluster_cull,
    ClusterAabb, ClusterCull, ClusterEntry,
};

pub mod color_correction;
pub use color_correction::{
    cc_apply, cc_reset, cc_set_brightness, cc_set_contrast, cc_set_enabled, cc_set_saturation,
    cc_to_json, default_color_correction_config, new_color_correction_state, ColorCorrectionConfig,
    ColorCorrectionState,
};

pub mod decal_stamp;
pub use decal_stamp::{
    default_decal_stamp, ds_add_stamp, ds_clear, ds_count, ds_enabled_count, ds_project_point,
    ds_remove_stamp, ds_set_enabled, ds_set_opacity, ds_to_json as decal_stamp_to_json,
    new_decal_stamp_set, DecalStamp, DecalStampSet, StampBlend,
};

pub mod depth_resolve;
pub use depth_resolve::{
    default_depth_resolve_config, dr_linearize, dr_memory_saved, dr_method_name, dr_resolve,
    dr_set_method, dr_set_reversed_z, dr_to_json, new_depth_resolve, DepthResolveConfig,
    DepthResolveMethod,
};

pub mod render_queue;
pub use render_queue::{
    new_render_queue, rq_clear, rq_drain, rq_get, rq_is_empty, rq_len, rq_push, rq_sort_by_layer,
    rq_to_json, DrawCall, RenderQueue,
};

pub mod draw_sort;
pub use draw_sort::{
    is_sorted_back_to_front, is_sorted_front_to_back, opaque_key, sort_draws, transparent_key,
    DrawSortKey as DrawDepthSortKey, SortStrategy as DrawSortStrategy, SortedDraw,
};

pub mod edge_weld;
pub use edge_weld::{
    apply_remap_to_indices, count_unique, run_weld_pass, validate_weld_config, weld_positions,
    EdgeWeldConfig, WeldStats,
};

pub mod env_rotation_v2;
pub use env_rotation_v2::{
    advance_spin, clamp_elevation, ev_to_linear, normalise_azimuth, rotation_matrix, EnvRotMode,
    EnvRotation, EnvRotationV2Config,
};

pub mod film_curve;
pub use film_curve::{
    aces_approx, apply_exposure, film_curve_eval, film_curve_rgb, gamma_curve, log_clip, reinhard,
    reinhard_extended, FilmCurveParams,
};

pub mod gpu_readback;
pub use gpu_readback::{
    blank_result, bytes_per_pixel, r32_to_f32, rgba8_to_f32, staging_buffer_size, validate_result,
    ReadbackFormat, ReadbackRequest, ReadbackResult,
};

pub mod irr_probe;
pub use irr_probe::{
    blend_sh, nearest_probe as nearest_irr_probe, scale_sh, sh_l0, IrrProbe, IrrProbeSet, IrrSh9,
};

pub mod light_bake;
pub use light_bake::{
    average_irradiance, bake_coverage, lambertian_irradiance, new_bake_job, set_texel, BakeQuality,
    BakedTexel, LightBakeConfig, LightBakeJob,
};

pub mod motion_vector_v2;
pub use motion_vector_v2::{
    dilate_max, ndc_to_motion_vec, MotionVec2 as MotionVec2V2,
    MotionVectorBuffer as MotionVectorBufferV2,
};

pub mod occlusion_test;
pub use occlusion_test::{
    OcclusionAabb, OcclusionQuery, OcclusionResult as OcclusionTestResult, SoftOcclusionBuffer,
};

pub mod alpha_premult;
pub use alpha_premult::{
    alpha_composite, linear_to_srgb_u8, luminance as premult_luminance, premultiply,
    premultiply_buffer, srgb_u8_to_linear, unpremultiply, unpremultiply_buffer, RgbaF32,
};

pub mod camera_rig_v2;
pub use camera_rig_v2::{
    crv2_apply_impulse, crv2_dolly, crv2_eye_position, crv2_is_still, crv2_reset, crv2_set_mode,
    crv2_step, crv2_to_json, new_camera_rig_v2, CameraRigV2, RigModeV2,
};

pub mod cluster_tile;
pub use cluster_tile::{
    ctg_clear, ctg_flat_index, ctg_light_count, ctg_set_light_count, ctg_tile_count, ctg_tiles_x,
    ctg_tiles_y, ctg_to_json, ctg_total_light_assignments, new_cluster_tile_grid,
    ClusterTileConfig, ClusterTileEntry, ClusterTileGrid,
};

pub mod color_grade;
pub use color_grade::{
    cg_apply_channel, cg_apply_rgb, cg_blend, cg_is_identity, cg_reset, cg_set_exposure,
    cg_set_gain, cg_set_gamma, cg_set_lift, cg_set_saturation, cg_to_json, new_color_grade,
    ColorGradeParams,
};

pub mod decal_normal;
pub use decal_normal::{
    dn_add, dn_angle_rad, dn_apply_all, dn_blend_normal, dn_clear, dn_count, dn_enabled_count,
    dn_remove, dn_set_enabled, dn_to_json, new_decal_normal_manager, DecalNormal,
    DecalNormalManager, NormalBlendMode,
};

pub mod depth_of_field_v2;
pub use depth_of_field_v2::{
    dv2_blade_count, dv2_bokeh_area_mm2, dv2_coc_mm, dv2_coc_px, dv2_depth_range, dv2_reset,
    dv2_set_aperture, dv2_set_focal, dv2_set_focus, dv2_to_json, new_dof_v2, BokehShape,
    DofV2Config,
};

pub mod draw_material;
pub use draw_material::{
    dml_add, dml_clear, dml_count, dml_get, dml_remove, dml_set_roughness, dml_shading_name,
    dml_to_json, dml_uses_alpha_test, new_draw_material_lib, DrawMaterial, DrawMaterialLib,
    ShadingModel,
};

pub mod edge_outline;
pub use edge_outline::{
    eo_is_depth_edge, eo_is_edge, eo_is_normal_edge, eo_outline_alpha, eo_reset, eo_set_color,
    eo_set_depth_threshold, eo_set_enabled, eo_set_normal_threshold, eo_set_thickness, eo_to_json,
    new_edge_outline, EdgeOutlineConfig,
};

pub mod env_specular;
pub use env_specular::{
    es_fresnel, es_geometry_smith, es_ggx_d, es_memory_bytes, es_reset, es_roughness_to_mip,
    es_set_enabled, es_set_intensity, es_to_json, new_env_specular, EnvSpecularConfig,
};

pub mod film_saturation;
pub use film_saturation::{
    fs_apply, fs_blend, fs_is_identity, fs_pixel_saturation, fs_reset, fs_set_enabled,
    fs_set_hue_shift, fs_set_saturation, fs_set_vibrance, fs_to_json, new_film_saturation,
    FilmSaturationConfig,
};

pub mod gpu_stats;
pub use gpu_stats::{
    gs_average_draw_calls, gs_average_gpu_time_ns, gs_clear, gs_fps_from_ns, gs_frame_count,
    gs_last_frame, gs_peak_triangle_count, gs_push_frame, gs_to_json, gs_total_bandwidth_bytes,
    new_gpu_stats, GpuFrameStats, GpuStats,
};

pub mod instance_batch;
pub use instance_batch::{
    ib_add, ib_clear, ib_count, ib_memory_bytes, ib_remove, ib_set_tint, ib_set_visible,
    ib_to_json, ib_translation, ib_visible_count, ib_visible_transforms, new_instance_batch,
    BatchInstance, InstanceBatch, InstanceTransform,
};

pub mod light_probe_v2;
pub use light_probe_v2::{
    lp2_add, lp2_ambient_magnitude, lp2_blend_sh, lp2_clear, lp2_count, lp2_enabled_count,
    lp2_eval_sh, lp2_influence, lp2_nearest, lp2_remove, lp2_set_enabled, lp2_set_sh, lp2_to_json,
    new_light_probe_set_v2, LightProbeSetV2, LightProbeV2, Sh9, SH9_COUNT,
};

pub mod motion_history;
pub use motion_history::{
    mh_average_speed, mh_clear, mh_latest, mh_net_displacement, mh_oldest, mh_path_length,
    mh_peak_speed, mh_push, mh_sample_count, mh_to_json, new_motion_history, MotionHistory,
    MotionSample,
};

pub mod occlusion_volume;
pub use occlusion_volume::{
    new_occlusion_volume_set, ovs_add, ovs_clear, ovs_count, ovs_enabled_count, ovs_occlusion_at,
    ovs_remove, ovs_set_enabled, ovs_to_json, ovs_total_volume, OcclusionVolume,
    OcclusionVolumeSet,
};

pub mod alpha_sort;
pub use alpha_sort::{
    as_average_alpha, as_clear, as_count, as_depth_angle_rad, as_is_empty, as_max_depth, as_push,
    as_sort_back_to_front, as_sort_front_to_back, as_to_json, new_alpha_sort_buffer,
    AlphaSortBuffer, AlphaSortEntry,
};

pub mod camera_smooth;
pub use camera_smooth::{
    csm_distance, csm_is_at_origin, csm_reset, csm_set_instant, csm_step,
    csm_to_json as camera_smooth_to_json, csm_view_angle_rad, default_camera_smooth_config,
    new_camera_smooth_state, CameraSmoothConfig, CameraSmoothState,
};

pub mod cluster_cull_v2;
pub use cluster_cull_v2::{
    ccv2_clear, ccv2_count, ccv2_cull_by_volume, ccv2_depth_angle_rad, ccv2_mark_all_visible,
    ccv2_register, ccv2_set_visible, ccv2_to_json, ccv2_total_volume, ccv2_visible_count,
    new_cluster_cull_v2, Aabb2, ClusterCullV2, ClusterEntryV2,
};

pub mod color_matrix;
pub use color_matrix::{
    cm_apply, cm_blend, cm_determinant, cm_hue_rotate_angle_rad, cm_identity, cm_is_identity,
    cm_saturation, cm_set_enabled, cm_to_json, ColorMatrix,
};

pub mod decal_fade;
pub use decal_fade::{
    default_decal_fade_config, df_advance, df_alpha, df_is_done, df_phase_angle_rad, df_reset,
    df_to_json, df_total_duration, new_decal_fade_state, DecalFadeConfig, DecalFadePhase,
    DecalFadeState,
};

pub mod depth_sample;
pub use depth_sample::{
    default_depth_sample_config, ds_average_linear, ds_depth_range, ds_is_valid, ds_linearize,
    ds_ndc_from_linear, ds_perspective_angle_rad, ds_sample_buffer,
    ds_to_json as depth_sample_to_json, DepthSampleConfig,
};

pub mod draw_group;
pub use draw_group::{
    dg_add, dg_average_sort_key, dg_clear, dg_count, dg_count_by_group, dg_group_ids, dg_is_empty,
    dg_key_angle_rad, dg_sort_by_key, dg_to_json, new_draw_group_manager, DrawGroupEntry,
    DrawGroupManager,
};

pub mod edge_flow_view;
pub use edge_flow_view::{
    efv_add_edge, efv_average_angle_rad, efv_clear, efv_count, efv_count_by_loop,
    efv_flow_angle_rad, efv_is_empty, efv_loop_ids, efv_set_enabled, efv_to_json,
    new_edge_flow_view, EdgeFlowEntry, EdgeFlowView,
};

pub mod env_map_v2;
pub use env_map_v2::{
    default_env_map_v2_config, emv2_effective_intensity, emv2_rotation_rad, emv2_roughness_for_mip,
    emv2_set_enabled, emv2_set_intensity, emv2_set_mip_count, emv2_set_rotation_deg,
    emv2_solid_angle, emv2_source_name, emv2_to_json, EnvMapV2Config, EnvMapV2Source,
    MAX_MIP_LEVELS_V2,
};

pub mod film_vignette;
pub use film_vignette::{
    default_film_vignette_config, fvig_alpha, fvig_fade_angle_rad, fvig_reset, fvig_rim_alpha,
    fvig_set_enabled, fvig_set_radius, fvig_set_softness, fvig_set_strength, fvig_to_json,
    FilmVignetteConfig,
};

pub mod gpu_timestamp;
pub use gpu_timestamp::{
    gts_average_ns, gts_begin_frame, gts_clear, gts_count, gts_max_ns, gts_record,
    gts_slowest_pass, gts_time_angle_rad, gts_to_json, gts_total_ns, new_gpu_timestamp_manager,
    GpuTimestampEntry, GpuTimestampManager,
};

pub mod instance_lod;
pub use instance_lod::{
    default_instance_lod_config, ilod_average_lod, ilod_clear, ilod_count, ilod_count_at_level,
    ilod_lod_angle_rad, ilod_register, ilod_select_level, ilod_to_json, new_instance_lod_manager,
    InstanceLodConfig, InstanceLodEntry, InstanceLodManager, MAX_LOD_LEVELS,
};

pub mod light_volume_v2;
pub use light_volume_v2::{
    lv2_add, lv2_clear, lv2_count, lv2_enabled_count, lv2_remove, lv2_set_enabled,
    lv2_solid_angle_rad, lv2_sphere_volume, lv2_to_json, lv2_total_intensity,
    new_light_volume_set_v2, LightVolumeKind, LightVolumeSetV2, LightVolumeV2,
};

pub mod motion_sample;
pub use motion_sample::{
    msb_average_velocity, msb_clear, msb_count, msb_is_empty, msb_peak_speed, msb_push,
    msb_speed_angle_rad, msb_to_json, new_motion_sample_buffer, MotionSampleBuffer,
    MotionSampleEntry, MOTION_SAMPLE_CAPACITY,
};

pub mod occlusion_query_v2;
pub use occlusion_query_v2::{
    new_occlusion_query_v2, oqv2_angle_rad, oqv2_average_samples, oqv2_clear, oqv2_count,
    oqv2_occluded_count, oqv2_pending_count, oqv2_register, oqv2_resolve, oqv2_to_json,
    oqv2_visibility_ratio, oqv2_visible_count, OcclusionQueryV2Entry, OcclusionQueryV2Manager,
    OqState,
};

pub mod render_target_v2;
pub use render_target_v2::{
    default_render_target_v2_config, rtv2_aspect_ratio, rtv2_attachment_count, rtv2_fov_angle_rad,
    rtv2_is_valid, rtv2_memory_bytes, rtv2_set_size, rtv2_to_json, RenderTargetV2Config,
    RtFormatV2, MAX_COLOR_ATTACHMENTS,
};

pub mod render_scope;
pub use render_scope::{
    new_render_scope_registry, rs_add, rs_clear, rs_count, rs_enabled_count, rs_get,
    rs_over_budget_count, rs_record_time, rs_remove, rs_set_enabled, rs_to_json, rs_total_time_us,
    RenderScope, RenderScopeRegistry,
};

pub mod grid_fade;
pub use grid_fade::{
    gf_fade_range, gf_mid_opacity, gf_set_base_opacity, gf_set_fade_end, gf_set_fade_start,
    gf_set_min_opacity, gf_to_json, grid_fade_opacity, new_grid_fade_config, GridFadeConfig,
};

pub mod sky_gradient;
pub use sky_gradient::{
    default_sky_gradient_config, new_sky_gradient_state,
    sky_color_at_angle as sg_sky_color_at_angle, sky_is_day, sky_set_sun_direction,
    sky_set_time_of_day, sky_sun_direction, sky_to_json, SkyGradientConfig, SkyGradientState,
};

pub mod fog_volume;
pub use fog_volume::{
    fog_factor as fv_fog_factor, fv_blend_color, fv_set_color, fv_set_density, fv_set_kind,
    fv_to_json, new_fog_volume_config, FogKind, FogVolumeConfig,
};

pub mod rain_overlay;
pub use rain_overlay::{
    generate_rain_streaks, new_rain_overlay_config, ro_avg_streak_diag, ro_set_intensity,
    ro_set_opacity, ro_set_wind_angle_deg, ro_to_json, RainOverlayConfig, RainStreak,
};

pub mod snow_overlay;
pub use snow_overlay::{
    generate_snowflakes, new_snow_overlay_config, so_average_radius, so_set_count,
    so_set_fall_speed, so_set_opacity, so_step, so_to_json, SnowFlake, SnowOverlayConfig,
};

pub mod dust_overlay;
pub use dust_overlay::{
    do_average_alpha, do_set_opacity, do_set_tint, do_step, do_to_json, generate_dust_particles,
    new_dust_overlay_config, DustOverlayConfig, DustParticle,
};

pub mod lens_distortion;
pub use lens_distortion::{
    default_lens_distortion_config, ld_distort_uv, ld_is_barrel, ld_reset, ld_set_enabled,
    ld_set_strength, ld_to_json, new_lens_distortion_state, LensDistortionConfig,
    LensDistortionState,
};

pub mod film_grain;
pub use film_grain::{
    apply_color_grain, apply_film_grain, apply_mono_grain, estimate_noise_level, grain_value,
    hash_to_float, hash_u32, luminance_scale, FilmGrainConfig,
};

pub mod color_temperature;
pub use color_temperature::{
    ct_is_default, ct_reset, ct_set_kelvin, ct_set_tint, ct_to_json, ct_white_balance,
    kelvin_to_rgb as ct_kelvin_to_rgb, new_color_temperature_state, ColorTemperatureState,
};

pub mod exposure_control;
pub use exposure_control::{
    auto_expose_average, compute_ev, default_exposure_settings, ev_to_multiplier,
    evaluate_exposure, exposure_to_json, set_aperture as ec_set_aperture, set_ev_bias, set_iso,
    ExposureMode, ExposureResult, ExposureSettings,
};

pub mod gamma_correction;
pub use gamma_correction::{
    gamma_decode, gamma_encode, gc_apply, gc_is_identity, gc_reset, gc_set_gain, gc_set_lift,
    gc_to_json, new_gamma_correction_state, GammaCorrectionState, GammaMode,
};

pub mod halftone_effect;
pub use halftone_effect::{
    ht_coverage, ht_dot_radius, ht_set_angle_deg, ht_set_cell_size, ht_set_fg, ht_set_intensity,
    ht_to_json, new_halftone_effect_config, HalftoneDotShape, HalftoneEffectConfig,
};

pub mod pixel_sort_effect;
pub use pixel_sort_effect::{
    new_pixel_sort_effect_config, ps_set_intensity, ps_set_threshold_high, ps_set_threshold_low,
    ps_should_sort, ps_sort_range, ps_sort_row, ps_to_json, PixelSortEffectConfig, SortDirection,
    SortKey,
};

pub mod datamosh_effect;
pub use datamosh_effect::{
    dm_accumulate, dm_artifact_strength, dm_block_count, dm_is_iframe, dm_set_block_size,
    dm_set_feedback, dm_set_iframe_interval, dm_set_intensity, dm_to_json,
    new_datamosh_effect_config, DatamoshEffectConfig,
};

pub mod scan_line_effect;
pub use scan_line_effect::{
    new_scan_line_effect_config, sl_darkening_at, sl_line_count, sl_midpoint_darkening,
    sl_set_darkness, sl_set_line_spacing, sl_set_opacity, sl_to_json, ScanLineEffectConfig,
};

pub mod crt_warp_effect;
pub use crt_warp_effect::{
    cw_corner_distortion, cw_distort_uv, cw_is_visible, cw_set_curvature, cw_set_k1, cw_set_k2,
    cw_set_vignette, cw_to_json, cw_vignette_at, new_crt_warp_effect_config, CrtWarpEffectConfig,
};

pub mod cubemap_debug;
pub use cubemap_debug::{
    cd_highlight, cd_set_exposure, cd_toggle_labels, cross_layout_pixel_count,
    cubemap_debug_to_json, face_label, face_uv, new_cubemap_debug_config,
    normalize_dir as cubemap_normalize_dir, CubeFace as DebugCubeFace, CubemapDebugConfig,
};

pub mod light_probe_debug;
pub use light_probe_debug::{
    evaluate_sh_irradiance, light_probe_debug_to_json, lpd_set_exposure,
    new_light_probe_debug_config, new_sh_probe, sh_l0_scale, sh_probe_energy, sh_probe_reset,
    LightProbeDebugConfig, ShProbe,
};

pub mod particle_trail;
pub use particle_trail::{
    default_trail_config, new_particle_trail, trail_add_point, trail_clear, trail_get_point,
    trail_length as particle_trail_length, trail_point_count as particle_trail_point_count,
    trail_to_json, trail_update, ParticleTrail, TrailConfig, TrailPoint as ParticleTrailPoint,
};

pub mod flow_field;
pub use flow_field::{
    ff_set_arrow_scale, ff_set_max_magnitude, flow_divergence, flow_field_to_json, flow_magnitude,
    magnitude_to_color, new_flow_field_config, normalize_velocity, vortex_flow_field,
    FlowFieldConfig, FlowVector,
};

pub mod vector_field;
pub use vector_field::{
    curl_vector_field, new_vector_field_config, vec3_magnitude, vec3_normalize,
    vector_field_to_json, vf_glyph_count, vf_magnitude_color, vf_set_glyph_scale,
    vf_set_glyph_style, GlyphStyle, VectorAt, VectorFieldConfig,
};

pub mod scalar_field;
pub use scalar_field::{
    new_scalar_field_config, scalar_field_to_json, sf_gradient_1d, sf_interpolate, sf_is_above_iso,
    sf_isovalues, sf_normalize_color, sf_set_isovalue, sf_set_opacity, ScalarFieldConfig,
    ScalarSample,
};

pub mod contour_line;
pub use contour_line::{
    cl_generate_levels, cl_interpolate_crossing, cl_is_on_contour, cl_level_spacing, cl_set_levels,
    cl_set_line_width, cl_set_range, contour_line_to_json, new_contour_line_config, ContourLevel,
    ContourLineConfig,
};

pub mod mesh_patch_view;
pub use mesh_patch_view::{
    generate_patch_records, mesh_patch_view_to_json, mpv_active_patch_count,
    mpv_set_boundary_width, mpv_set_opacity, mpv_toggle_boundaries, new_mesh_patch_view_config,
    patch_color, MeshPatchViewConfig, PatchRecord,
};

pub mod bone_envelope_view;
pub use bone_envelope_view::{
    bev_set_opacity, bev_toggle_wireframe, bone_envelope_to_json, envelope_approx_volume,
    envelope_length, new_bone_envelope_config, point_in_envelope, BoneEnvelope, BoneEnvelopeConfig,
};

pub mod joint_angle_overlay;
pub use joint_angle_overlay::{
    arc_points, deg_to_rad, filter_annotations, format_angle, jao_set_arc_radius,
    jao_set_font_size, jao_toggle_units, joint_angle_overlay_to_json,
    new_joint_angle_overlay_config, rad_to_deg, JointAngleAnnotation, JointAngleOverlayConfig,
};

pub mod blend_shape_preview;
pub use blend_shape_preview::{
    blend_shape_preview_to_json, bsp_apply_blend, bsp_normalized_weight, bsp_reset, bsp_set_weight,
    bsp_update, displacement_heat_color, new_blend_shape_preview_config,
    new_blend_shape_preview_state, BlendShapePreviewConfig, BlendShapePreviewState,
};

pub mod texture_channel_view;
pub use texture_channel_view::{
    apply_channel_view, channel_label, extract_channel, new_texture_channel_view_config,
    tcv_set_channel, tcv_set_exposure, tcv_toggle_grayscale, tcv_toggle_invert,
    texture_channel_view_to_json, TextureChannel, TextureChannelViewConfig,
};

pub mod albedo_view;
pub use albedo_view::{
    albedo_view_to_json, apply_albedo_view, av_disable, av_enable, av_reset, av_set_exposure,
    av_set_gamma, luminance as albedo_luminance, new_albedo_view_config, AlbedoViewConfig,
};

pub mod roughness_view;
pub use roughness_view::{
    apply_roughness_view, is_rough, new_roughness_view_config, roughness_to_smoothness,
    roughness_to_specular_power, roughness_view_to_json, rv_disable, rv_enable, rv_reset,
    rv_set_exposure, rv_toggle_invert, RoughnessViewConfig,
};

pub mod metallic_view;
pub use metallic_view::{
    apply_metallic_view, is_metallic, metallic_view_to_json, mv_disable, mv_enable, mv_reset,
    mv_set_threshold, mv_toggle_binary, new_metallic_view_config, MetallicViewConfig,
};

pub mod cavity_map_view;
pub use cavity_map_view::{
    apply_cavity_map, cavity_map_view_to_json, cmv_disable, cmv_enable, cmv_reset,
    cmv_set_intensity, cmv_set_ridge_strength, cmv_set_valley_strength, curvature_to_cavity,
    new_cavity_map_view_config, CavityMapViewConfig,
};

pub mod gbuffer_view;
pub use gbuffer_view::{
    default_gbuffer_view_config, gbuffer_view_to_json, linearize_depth as gbuffer_linearize_depth,
    new_gbuffer_pixel, visualize_channel, GBufferChannel, GBufferPixel, GBufferViewConfig,
};

pub mod grid_3d;
pub use grid_3d::{
    generate_grid_lines_xz, grid_3d_to_json, grid_extent, grid_line_count, grid_set_cell_count,
    grid_set_cell_size, grid_set_opacity, new_grid_3d_config, Grid3DConfig, GridLine3D,
};

pub mod axis_label;
pub use axis_label::{
    al_label_text, al_set_font_size, al_toggle_axis, al_visible_count, axis_label_to_json,
    generate_axis_labels, new_axis_label_config, AxisLabelConfig, AxisLabelEntry,
};

pub mod scale_ruler_3d;
pub use scale_ruler_3d::{
    generate_tick_marks, new_scale_ruler_3d_config, scale_ruler_3d_to_json, sr_major_tick_count,
    sr_set_length, sr_set_major_spacing, ScaleRuler3DConfig, TickMark3D,
};

pub mod compass_rose;
pub use compass_rose::{
    compass_rose_to_json, cr_cardinal_count, cr_heading_rad, cr_set_heading, cr_set_size,
    generate_compass_points, new_compass_rose_config, CompassPoint, CompassRoseConfig,
};

pub mod sun_position;
pub use sun_position::{
    new_sun_position_config, sp_is_above_horizon, sp_set_azimuth, sp_set_elevation,
    sp_sky_luminance, sun_direction_vector, sun_position_to_json, SunPositionConfig,
};

pub mod shadow_debug;
pub use shadow_debug::{
    new_shadow_debug_config, sd_active_cascade_color, sd_cascade_split, sd_set_active_cascade,
    sd_set_opacity, sd_toggle_splits, shadow_debug_to_json, ShadowDebugConfig,
};

pub mod depth_buffer_view;
pub use depth_buffer_view::{
    dbv_disable, dbv_enable, dbv_set_contrast, dbv_set_mode, depth_buffer_view_to_json,
    depth_to_color, new_depth_buffer_view_config, remap_depth, DepthBufferViewConfig,
    DepthRemapMode,
};

pub mod stencil_buffer_view;
pub use stencil_buffer_view::{
    new_stencil_buffer_view_config, sbv_disable, sbv_enable, sbv_palette_size, sbv_set_opacity,
    sbv_set_palette_entry, stencil_buffer_view_to_json, stencil_value_to_color,
    StencilBufferViewConfig,
};

pub mod overdraw_view;
pub use overdraw_view::{
    new_overdraw_view, odv_enable, odv_is_enabled, odv_overdraw_color, odv_reset, odv_set_max,
    odv_to_json, OverdrawView,
};

pub mod triangle_count_overlay;
pub use triangle_count_overlay::{
    build_triangle_entry, format_triangle_count, new_triangle_count_overlay_config, tco_disable,
    tco_enable, tco_set_threshold, total_triangle_count, triangle_count_overlay_to_json,
    TriangleCountEntry, TriangleCountOverlayConfig,
};

pub mod draw_call_heatmap;
pub use draw_call_heatmap::{
    draw_call_heatmap_to_json, draw_call_to_color, hm_cell_count, hm_disable, hm_enable,
    hm_find_hottest, hm_set_grid_resolution, hm_set_max_calls, hm_set_opacity,
    new_draw_call_heatmap_config, DrawCallHeatmapConfig,
};

pub mod texture_mip_debug;
pub use texture_mip_debug::{
    compute_mip_level, mip_level_color, new_texture_mip_debug_config, texture_mip_debug_to_json,
    tmd_color_count, tmd_disable, tmd_enable, tmd_set_opacity, TextureMipDebugConfig,
};

pub mod clip_space_debug;
pub use clip_space_debug::{
    classify_clip_vertex, clip_space_debug_to_json, clip_status_color, count_inside_vertices,
    csd_disable, csd_enable, csd_set_point_size, new_clip_space_debug_config, ClipSpaceDebugConfig,
    ClipStatus,
};

pub mod light_count_overlay;
pub use light_count_overlay::{
    lco_average_lights, lco_disable, lco_enable, lco_set_max_lights, lco_set_opacity,
    lco_set_tile_size, lco_tile_grid, light_count_overlay_to_json, light_count_to_color,
    new_light_count_overlay_config, LightCountOverlayConfig,
};

pub mod vertex_id_overlay;
pub use vertex_id_overlay::{
    count_displayable, format_vertex_id, new_vertex_id_overlay_config, should_display_vertex,
    vertex_id_overlay_to_json, vid_disable, vid_enable, vid_set_max_id, vid_set_stride,
    VertexIdOverlayConfig,
};

pub mod noise_overlay;
pub use noise_overlay::{
    default_noise_overlay_config, grain_sample, noise_disable, noise_enable, noise_overlay_to_json,
    noise_set_intensity, noise_set_pattern, noise_set_scale, noise_value_at, perlin_approx,
    NoiseOverlayConfig, NoisePattern,
};

pub mod checkerboard_pattern;
pub use checkerboard_pattern::{
    cb_disable, cb_enable, cb_generate_row, cb_is_color_a, cb_sample, cb_set_cell_size,
    cb_set_color_a, cb_set_color_b, cb_to_json, default_checkerboard_config, CheckerboardConfig,
};

pub mod uv_checker_view;
pub use uv_checker_view::{
    default_uv_checker_config, uvc_disable, uvc_enable, uvc_gradient_sample, uvc_grid_sample,
    uvc_set_grid_divisions, uvc_set_mode, uvc_set_opacity, uvc_to_json, uvc_zone_sample,
    UvCheckerConfig, UvCheckerMode,
};

pub mod tangent_space_view;
pub use tangent_space_view::{
    default_tangent_space_config, tsv_disable, tsv_enable, tsv_handedness, tsv_is_orthonormal,
    tsv_set_component, tsv_set_line_length, tsv_to_json, tsv_vector_to_color, TangentSpaceConfig,
    TbnComponent,
};

pub mod barycentric_view;
pub use barycentric_view::{
    bary_coords, bary_disable, bary_enable, bary_interpolate_color, bary_is_inside,
    bary_set_edge_width, bary_to_json, default_barycentric_config, BarycentricConfig,
};

pub mod face_normal_view;
pub use face_normal_view::{
    compute_face_normal, default_face_normal_config, face_centroid, fn_disable, fn_enable,
    fn_set_color, fn_set_line_length, fn_to_json, is_backface, FaceNormalConfig,
};

pub mod vertex_normal_view;
pub use vertex_normal_view::{
    default_vertex_normal_config, vn_build_lines, vn_disable, vn_enable, vn_normal_endpoint,
    vn_normal_to_color, vn_normalize, vn_set_line_length, vn_set_max_verts, vn_to_json,
    VertexNormalConfig,
};

pub mod edge_length_view;
pub use edge_length_view::{
    default_edge_length_config, el_disable, el_edge_length, el_enable, el_length_to_color,
    el_set_range, el_stats, el_to_json as edge_length_to_json, EdgeLengthConfig,
};

pub mod face_area_view;
pub use face_area_view::{
    default_face_area_config, fa_area_to_color, fa_average_area, fa_disable, fa_enable,
    fa_set_range, fa_to_json, fa_total_area, fa_triangle_area, FaceAreaConfig,
};

pub mod curvature_view;
pub use curvature_view::{
    cv_curvature_to_color, cv_disable, cv_enable, cv_gaussian_curvature, cv_mean_curvature,
    cv_set_scale, cv_set_type, cv_to_json as curvature_to_json, default_curvature_config,
    CurvatureConfig, CurvatureType,
};

pub mod stretch_view;
pub use stretch_view::{
    default_stretch_config, sv_area_ratio, sv_disable, sv_enable, sv_max_stretch, sv_set_metric,
    sv_set_threshold, sv_stretch_to_color, sv_to_json, StretchConfig, StretchMetric,
};

pub mod seam_view;
pub use seam_view::{
    default_seam_view_config, svm_count_seams, svm_disable, svm_edge_color, svm_enable,
    svm_is_seam_edge, svm_set_line_width, svm_set_seam_color, svm_to_json, SeamViewConfig,
    UvEdgePair,
};

pub mod hard_edge_view;
pub use hard_edge_view::{
    default_hard_edge_config, he_count_hard, he_dihedral_angle_deg, he_disable, he_edge_color,
    he_enable, he_is_hard_edge, he_set_line_width, he_set_threshold, he_to_json, HardEdgeConfig,
};

pub mod pole_view;
pub use pole_view::{
    default_pole_view_config, pv_classify_pole, pv_count_poles, pv_disable, pv_enable,
    pv_pole_color, pv_set_high_threshold, pv_set_point_size, pv_to_json, PoleKind, PoleViewConfig,
};

pub mod ngon_view;
pub use ngon_view::{
    default_ngon_view_config, ngv_classify, ngv_count_ngons, ngv_disable, ngv_enable,
    ngv_face_color, ngv_face_stats, ngv_set_highlight_ngons_only, ngv_to_json, FacePolyType,
    NgonViewConfig,
};

pub mod non_manifold_view;
pub use non_manifold_view::{
    default_non_manifold_config, nm_count_boundary_edges, nm_count_non_manifold_edges, nm_disable,
    nm_enable, nm_is_boundary_edge, nm_is_non_manifold_edge, nm_kind_color, nm_set_line_width,
    nm_set_point_size, nm_to_json, nm_topology_stats, NonManifoldConfig, NonManifoldKind,
};

pub mod render_pass_view;
pub use render_pass_view::{
    new_render_pass_view, rpv_add_pass, rpv_enabled_count, rpv_pass_count, rpv_select,
    rpv_selected_name, rpv_to_json, rpv_toggle, RenderPassEntry, RenderPassView,
};

pub mod g_buffer_view;
pub use g_buffer_view::{
    gbv2_apply_exposure, gbv2_channel_name, gbv2_set_channel, gbv2_set_enabled, gbv2_set_exposure,
    gbv2_to_json, new_gbuffer_view2, GBufferViewConfig2, GbvChannel,
};

pub mod light_probe_view;
pub use light_probe_view::{
    lpv_sample_direction, lpv_set_enabled, lpv_set_exposure, lpv_set_radius, lpv_sphere_area,
    lpv_to_json, lpv_toggle_sh, new_light_probe_view, LightProbeViewConfig,
};

pub mod shadow_ray_view;
pub use shadow_ray_view::{
    new_shadow_ray_view, srv_coverage_factor, srv_pixel_color, srv_set_enabled, srv_set_max_rays,
    srv_set_shadow_tint, srv_to_json, ShadowRayViewConfig,
};

pub mod reflection_ray_view;
pub use reflection_ray_view::{
    new_reflection_ray_view, rrv_miss_color, rrv_reflect, rrv_set_enabled, rrv_set_intensity,
    rrv_set_max_bounces, rrv_to_json, rrv_toggle_miss, ReflectionRayViewConfig,
};

pub mod gi_irradiance_view;
pub use gi_irradiance_view::{
    giv_irradiance_to_color, giv_set_enabled, giv_set_exposure, giv_to_json, giv_toggle_direct,
    giv_toggle_probes, giv_tonemap, new_gi_irradiance_view, GiIrradianceViewConfig,
};

pub mod path_tracer_view;
pub use path_tracer_view::{
    new_path_tracer_view, ptv_noise_level, ptv_sample_density_color, ptv_set_depth,
    ptv_set_enabled, ptv_set_exposure, ptv_set_spp, ptv_to_json, PathTracerViewConfig,
};
