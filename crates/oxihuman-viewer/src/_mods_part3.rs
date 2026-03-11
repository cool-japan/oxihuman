pub mod denoiser_view;
pub use denoiser_view::{
    dnv_is_before_side, dnv_mode_name, dnv_set_enabled, dnv_set_mode, dnv_set_split, dnv_to_json,
    new_denoiser_view, DenoiseSplitMode, DenoiserViewConfig,
};

pub mod temporal_aa_view;
pub use temporal_aa_view::{
    new_temporal_aa_view, tav_is_ghosting, tav_set_enabled, tav_set_ghosting_threshold,
    tav_set_jitter_scale, tav_to_json, tav_toggle_ghosting, tav_toggle_jitter,
    TemporalAaViewConfig,
};

pub mod lens_distortion_view;
pub use lens_distortion_view::{
    ldv_distort_uv, ldv_grid_vertex_count, ldv_set_divisions, ldv_set_enabled, ldv_set_k1,
    ldv_set_k2, ldv_to_json, ldv_toggle_grid, new_lens_distortion_view, LensDistortionViewConfig,
};

pub mod color_temperature_view;
pub use color_temperature_view::{
    ctv_is_daylight, ctv_kelvin_to_rgb, ctv_set_enabled, ctv_set_kelvin, ctv_set_tint, ctv_to_json,
    ctv_toggle_gamut, new_color_temperature_view, ColorTemperatureViewConfig,
};

pub mod exposure_meter_view;
pub use exposure_meter_view::{
    emv2_correction, emv2_luminance_to_ev, emv2_mode_name, emv2_set_enabled as meter_set_enabled,
    emv2_set_ev_target, emv2_set_mode, emv2_to_json as meter_to_json, emv2_toggle_histogram,
    new_exposure_meter_view, ExposureMeterViewConfig, MeteringMode,
};

pub mod histogram_tone_view;
pub use histogram_tone_view::{
    htv_accumulate, htv_clear, htv_peak, htv_to_json, htv_toggle_log, htv_total_samples,
    new_histogram_tone, HistogramTone, HistogramToneViewConfig,
};

pub mod lut_preview_view;
pub use lut_preview_view::{
    lpv2_data_size, lpv2_sample, lpv2_set_enabled, lpv2_set_intensity, lpv2_to_json,
    lpv2_toggle_before, new_lut_preview, LutPreview, LutPreviewViewConfig,
};

pub mod film_grain_view;
pub use film_grain_view::{
    fgv_apply, fgv_grain_value, fgv_set_enabled, fgv_set_grain_size, fgv_set_intensity,
    fgv_set_luma_sensitivity, fgv_to_json, fgv_toggle_animated, new_film_grain_view,
    FilmGrainViewConfig,
};

pub mod chromatic_shift_view;
pub use chromatic_shift_view::{
    csv_set_blue_offset, csv_set_enabled, csv_set_intensity, csv_set_red_offset,
    csv_shift_magnitude, csv_shifted_uv, csv_to_json, csv_toggle_channels,
    new_chromatic_shift_view, ChromaticShiftViewConfig,
};

pub mod virtual_camera_view;
pub use virtual_camera_view::{
    new_virtual_camera_view, vcv_hfov_deg, vcv_set_aperture, vcv_set_aspect, vcv_set_enabled,
    vcv_set_focal_length, vcv_to_json, AspectPreset, VirtualCameraView,
};

pub mod camera_shake_view;
pub use camera_shake_view::{
    csh_sample_offset, csh_set_amplitude, csh_set_enabled, csh_set_profile, csh_tick, csh_to_json,
    new_camera_shake_view, CameraShakeView, ShakeProfile,
};

pub mod dolly_zoom_view;
pub use dolly_zoom_view::{
    dzv_camera_distance, dzv_current_fov, dzv_set_enabled, dzv_set_progress, dzv_set_target_fov,
    dzv_to_json, new_dolly_zoom_view, DollyZoomView,
};

pub mod rack_focus_view;
pub use rack_focus_view::{
    new_rack_focus_view, rfv_current_focus_dist, rfv_set_blur, rfv_set_enabled,
    rfv_set_focus_progress, rfv_set_speed, rfv_to_json, RackFocusView,
};

pub mod anamorphic_view;
pub use anamorphic_view::{
    anv_set_enabled, anv_set_flare_intensity, anv_set_squeeze, anv_set_streak, anv_squeeze_factor,
    anv_to_json, new_anamorphic_view, AnamorphicView, SqueezeRatio,
};

pub mod tilt_shift_view;
pub use tilt_shift_view::{
    new_tilt_shift_view, tsh_blur_at_y, tsh_set_blur, tsh_set_enabled, tsh_set_focus_band,
    tsh_set_focus_center, tsh_to_json, TiltShiftView,
};

pub mod infrared_view;
pub use infrared_view::{
    irv_apply_pixel, irv_set_color_map, irv_set_enabled, irv_set_foliage_boost, irv_to_json,
    new_infrared_view, InfraredView, IrColorMap,
};

pub mod xray_view;
pub use xray_view::{
    new_xray_view, xrv_blend_pixel, xrv_set_edge_intensity, xrv_set_enabled, xrv_set_opacity,
    xrv_set_style, xrv_to_json, XrayStyle, XrayView,
};

pub mod thermal_view;
pub use thermal_view::{
    new_thermal_view, thv_map_temp, thv_normalize_temp, thv_set_enabled, thv_set_palette,
    thv_set_range, thv_to_json, ThermalPalette, ThermalView,
};

pub mod night_vision_view;
pub use night_vision_view::{
    new_night_vision_view, nvv_apply_pixel, nvv_set_enabled, nvv_set_gain, nvv_set_generation,
    nvv_to_json, NightVisionView, NvGeneration,
};

pub mod sonar_view;
pub use sonar_view::{
    new_sonar_view, snv_intensity_at, snv_set_enabled, snv_set_ring_count, snv_set_speed, snv_tick,
    snv_to_json, SonarView,
};

pub mod radar_view;
pub use radar_view::{
    new_radar_view, rdv_add_blip, rdv_blip_count, rdv_set_enabled, rdv_set_speed, rdv_tick,
    rdv_to_json, RadarBlip, RadarView,
};

pub mod oscilloscope_view;
pub use oscilloscope_view::{
    new_oscilloscope_view, osv_peak, osv_push_sample, osv_set_enabled, osv_set_time_div,
    osv_set_trigger, osv_to_json, OscilloscopeView, TriggerMode,
};

pub mod spectrum_analyzer_view;
pub use spectrum_analyzer_view::{
    new_spectrum_analyzer_view, sav_band_count, sav_feed, sav_set_enabled, sav_set_freq_scale,
    sav_tick_decay, sav_to_json, FreqScale, SpectrumAnalyzerView,
};

pub mod vectorscope_view;
pub use vectorscope_view::{
    new_vectorscope_view, vsv_add_pixel, vsv_clear, vsv_point_count, vsv_set_enabled, vsv_set_gain,
    vsv_set_mode, vsv_to_json, VectorscopeMode, VectorscopePoint, VectorscopeView,
};

pub mod waveform_monitor_view;
pub use waveform_monitor_view::{
    new_waveform_monitor_view, wmv_set_channel, wmv_set_enabled, wmv_set_ire_graticule,
    wmv_to_json, wmv_update_column, wmv_width, WaveformChannel, WaveformColumn,
    WaveformMonitorView,
};

pub mod pixel_art_view;
pub use pixel_art_view::{
    new_pixel_art_view, pav_set_enabled, pav_set_outline, pav_set_palette_colors,
    pav_set_pixel_size, pav_set_quantize_mode, pav_to_json, PixelArtView, QuantizeMode,
};

pub mod cel_shade_view;
pub use cel_shade_view::{
    clv_set_bands, clv_set_enabled, clv_set_outline_color, clv_set_outline_width, clv_set_specular,
    clv_to_json, new_cel_shade_view, CelShadeView, ShadeBands,
};

pub mod halftone_view;
pub use halftone_view::{
    hfv_set_angle, hfv_set_contrast, hfv_set_dot_shape, hfv_set_dot_size, hfv_set_enabled,
    hfv_to_json, new_halftone_view, DotShape, HalftoneView,
};

pub mod stipple_view;
pub use stipple_view::{
    new_stipple_view, stv_set_density, stv_set_dot_radius, stv_set_enabled, stv_set_jitter,
    stv_set_strategy, stv_to_json, StippleStrategy, StippleView,
};

pub mod oil_paint_view;
pub use oil_paint_view::{
    new_oil_paint_view, opv_set_brush_size, opv_set_brush_style, opv_set_enabled,
    opv_set_saturation_boost, opv_set_smoothing, opv_to_json, BrushStyle, OilPaintView,
};

pub mod watercolor_view;
pub use watercolor_view::{
    new_watercolor_view, wcv_set_bleed_amount, wcv_set_bleed_pattern, wcv_set_enabled,
    wcv_set_paper_texture, wcv_set_wetness, wcv_to_json, BleedPattern, WatercolorView,
};

pub mod pencil_sketch_view;
pub use pencil_sketch_view::{
    new_pencil_sketch_view, psv_set_edge_threshold, psv_set_enabled, psv_set_hatching,
    psv_set_line_weight, psv_set_paper_grain, psv_set_style, psv_to_json, PencilSketchView,
    SketchStyle,
};

pub mod charcoal_view;
pub use charcoal_view::{
    chv_set_darkness, chv_set_enabled, chv_set_grain_scale, chv_set_smudge_amount,
    chv_set_smudge_dir, chv_to_json, new_charcoal_view, CharcoalView, SmudgeDir,
};

pub mod blueprint_view;
pub use blueprint_view::{
    bpv_set_enabled, bpv_set_grid_spacing, bpv_set_line_width, bpv_set_scheme,
    bpv_set_show_dimensions, bpv_to_json, new_blueprint_view, BlueprintScheme, BlueprintView,
};

pub mod anaglyph_view;
pub use anaglyph_view::{
    agv_set_convergence, agv_set_enabled, agv_set_eye_separation, agv_set_method, agv_to_json,
    new_anaglyph_view, AnaglyphMethod, AnaglyphView,
};

pub mod polarized_3d_view;
pub use polarized_3d_view::{
    new_polarized_3d_view, p3v_set_convergence, p3v_set_enabled, p3v_set_eye_separation,
    p3v_set_format, p3v_set_swap_eyes, p3v_to_json, Polarized3dView, StereoFormat,
};

pub mod lenticular_view;
pub use lenticular_view::{
    ltv_set_angle, ltv_set_axis, ltv_set_enabled, ltv_set_lpi, ltv_set_num_frames, ltv_to_json,
    new_lenticular_view, InterlaceAxis, LenticularView,
};

pub mod holographic_view;
pub use holographic_view::{
    hgv_set_brightness, hgv_set_color_mode, hgv_set_diffraction_scale, hgv_set_enabled,
    hgv_set_rotation_speed, hgv_to_json, new_holographic_view, HoloColorMode, HolographicView,
};

pub mod ascii_art_view;
pub use ascii_art_view::{
    aav_set_cell_size, aav_set_colored, aav_set_enabled, aav_set_invert, aav_set_palette,
    aav_to_json, new_ascii_art_view, AsciiArtView, AsciiPalette,
};

pub mod mosaic_view;
pub use mosaic_view::{
    msv_set_border_color, msv_set_border_width, msv_set_enabled, msv_set_tile_shape,
    msv_set_tile_size, msv_to_json, new_mosaic_view, MosaicView, TileShape,
};

pub mod glitch_view;
pub use glitch_view::{
    glv_set_enabled, glv_set_frequency, glv_set_intensity, glv_set_seed, glv_set_type, glv_to_json,
    new_glitch_view, GlitchType, GlitchView,
};

pub mod paint_over_view;
pub use paint_over_view::{
    new_paint_over_view, pov_add_stroke, pov_clear, pov_set_enabled, pov_set_opacity,
    pov_stroke_count, pov_to_json, PaintBrushTool, PaintOverView, PaintStroke,
};

pub mod reference_plane_view;
pub use reference_plane_view::{
    new_reference_plane_view, rfpv_set_axis, rfpv_set_enabled, rfpv_set_flip, rfpv_set_offset,
    rfpv_set_opacity, rfpv_to_json, PlaneAxis, ReferencePlaneView,
};

pub mod symmetry_plane_view;
pub use symmetry_plane_view::{
    new_symmetry_plane_view, spv_set_axis, spv_set_color, spv_set_enabled, spv_set_line_width,
    spv_show_mirror_mesh, spv_to_json, SymmetryAxis, SymmetryPlaneView,
};

pub mod cage_edit_view;
pub use cage_edit_view::{
    cev_clear_selection, cev_select_vertex, cev_selected_count, cev_set_cage_color,
    cev_set_enabled, cev_set_line_width, cev_set_selected_color, cev_to_json, new_cage_edit_view,
    CageEditView, CageVertexId,
};

pub mod weight_heat_map_view;
pub use weight_heat_map_view::{
    new_weight_heat_map_view, whm_set_bone_index, whm_set_enabled, whm_set_ramp,
    whm_set_thresholds, whm_to_json, HeatMapRamp, WeightHeatMapView,
};

pub mod bone_envelope_debug_view;
pub use bone_envelope_debug_view::{
    bedv_set_color, bedv_set_display_mode, bedv_set_enabled, bedv_set_opacity, bedv_show_head_tail,
    bedv_to_json, new_bone_envelope_debug_view, BoneEnvelopeDebugView, EnvelopeDisplayMode,
};

pub mod ik_chain_view;
pub use ik_chain_view::{
    ikv_set_chain_color, ikv_set_enabled, ikv_set_solver, ikv_set_target_color,
    ikv_show_iterations, ikv_show_pole_vector, ikv_to_json, new_ik_chain_view, IkChainView,
    IkSolverType,
};

pub mod constraint_arc_view;
pub use constraint_arc_view::{
    cav_set_arc_color, cav_set_arc_segments, cav_set_enabled, cav_set_limit_color, cav_set_type,
    cav_show_labels, cav_to_json, new_constraint_arc_view, ConstraintArcView, ConstraintType,
};

pub mod driver_graph_view;
pub use driver_graph_view::{
    dgv_add_node, dgv_clear, dgv_node_count, dgv_set_enabled, dgv_set_zoom, dgv_show_edge_labels,
    dgv_to_json, new_driver_graph_view, DriverGraphView, GraphNode, GraphNodeType,
};

pub mod nla_track_view;
pub use nla_track_view::{
    new_nla_track_view, ntv_add_strip, ntv_clear, ntv_set_enabled, ntv_set_track_height,
    ntv_show_influence, ntv_strip_count, ntv_to_json, NlaBlendMode, NlaStrip, NlaTrackView,
};

pub mod action_clip_view;
pub use action_clip_view::{
    acv_add_clip, acv_clear, acv_clip_count, acv_set_current_frame, acv_set_enabled, acv_set_zoom,
    acv_to_json, new_action_clip_view, ActionClip, ActionClipView, ClipPlayState,
};

pub mod keyframe_dot_view;
pub use keyframe_dot_view::{
    kdv_add_dot, kdv_clear, kdv_dot_count, kdv_set_dot_color, kdv_set_enabled,
    kdv_show_frame_numbers, kdv_to_json, new_keyframe_dot_view, KeyframeDot, KeyframeDotState,
    KeyframeDotView,
};

pub mod pose_library_view;
pub use pose_library_view::{
    new_pose_library_view, plv_add_pose, plv_clear, plv_pose_count, plv_set_columns,
    plv_set_enabled, plv_set_sort_order, plv_set_thumbnail_size, plv_to_json, PoseEntry,
    PoseLibraryView, PoseSortOrder,
};

pub mod retarget_map_view;
pub use retarget_map_view::{
    new_retarget_map_view, rmv_add_mapping, rmv_clear, rmv_highlight_errors, rmv_mapping_count,
    rmv_set_enabled, rmv_show_unmapped, rmv_to_json, rmv_unmapped_count, MappingQuality,
    RetargetBoneMapping, RetargetMapView,
};

pub mod blend_tree_graph_view;
pub use blend_tree_graph_view::{
    btgv_add_node, btgv_clear, btgv_node_count, btgv_set_enabled, btgv_set_pan, btgv_set_zoom,
    btgv_show_weights, btgv_to_json, new_blend_tree_graph_view, BlendTreeGraphView, BlendTreeNode,
    BlendTreeNodeType,
};

pub mod state_machine_view;
pub use state_machine_view::{
    new_state_machine_view, smv_add_state, smv_add_transition, smv_clear, smv_set_enabled,
    smv_set_zoom, smv_show_conditions, smv_state_count, smv_to_json, smv_transition_count, SmState,
    SmTransition, StateKind, StateMachineView,
};

pub mod usd_preview_view;
pub use usd_preview_view::{
    new_usd_preview_view, upv_has_stage, upv_set_frame, upv_set_render_mode, upv_set_stage,
    upv_to_json, UsdPreviewConfig, UsdPreviewView, UsdRenderMode,
};

pub mod material_graph_view;
pub use material_graph_view::{
    mgv_add_node, mgv_remove_node, mgv_set_pan, mgv_set_zoom, mgv_to_json, new_material_graph_view,
    MaterialGraphView, MaterialNode,
};

pub mod shader_graph_view;
pub use shader_graph_view::{
    new_shader_graph_view, sgv_add_node, sgv_clear, sgv_connect, sgv_set_zoom, sgv_to_json,
    ShaderEdge, ShaderGraphView, ShaderNode,
};

pub mod texture_node_view;
pub use texture_node_view::{
    new_texture_node_view, tnv_add_node, tnv_deselect, tnv_select, tnv_set_zoom, tnv_to_json,
    TextureNode, TextureNodeView,
};

pub mod geometry_node_view;
pub use geometry_node_view::{
    gnv_add_node, gnv_clear, gnv_count_type, gnv_set_zoom, gnv_to_json, new_geometry_node_view,
    GeoNode, GeoNodeType, GeometryNodeView,
};

pub mod compositor_view;
pub use compositor_view::{
    cv_active_node_count, cv_add_node, cv_set_muted, cv_set_zoom, cv_to_json as compositor_to_json,
    new_compositor_view, CompositorNode, CompositorView,
};

pub mod sequence_editor_view;
pub use sequence_editor_view::{
    new_sequence_editor_view, sev_add_strip, sev_set_fps, sev_set_frame, sev_to_json,
    sev_total_frames, SeqStrip, SequenceEditorView,
};

pub mod dope_sheet_view;
pub use dope_sheet_view::{
    dsv_add_key, dsv_select_frame, dsv_selected_count, dsv_set_current_frame, dsv_to_json,
    new_dope_sheet_view, DopeKey, DopeSheetView,
};

pub mod curve_editor_view;
pub use curve_editor_view::{
    cev_add_curve, cev_add_key, cev_set_frame, cev_to_json as curve_editor_to_json, cev_total_keys,
    new_curve_editor_view, CurveEditorView, CurveInterp, CurveKey, FCurve,
};

pub mod driver_editor_view;
pub use driver_editor_view::{
    dev_active_count, dev_add_driver, dev_select, dev_set_expression, dev_to_json,
    new_driver_editor_view, DriverDef, DriverEditorView,
};

pub mod outliner_filter_view;
pub use outliner_filter_view::{
    new_outliner_filter_view, ofv_active_count, ofv_add_entry, ofv_set_mode, ofv_set_search,
    ofv_to_json, OutlinerFilterEntry, OutlinerFilterMode, OutlinerFilterView,
};

pub mod property_panel_view;
pub use property_panel_view::{
    new_property_panel_view, ppv_add_property, ppv_set_search, ppv_set_value, ppv_to_json,
    ppv_visible_count, PropertyEntry, PropertyPanelView,
};

pub mod tool_shelf_view;
pub use tool_shelf_view::{
    new_tool_shelf_view, tsv_active_count, tsv_add_item, tsv_select,
    tsv_to_json as tool_shelf_to_json, tsv_toggle_collapse, ShelfItem, ToolShelfView,
};

pub mod header_bar_view;
pub use header_bar_view::{
    hbv_add_menu, hbv_enabled_menu_count, hbv_set_editor_type, hbv_to_json,
    hbv_toggle_region_header, new_header_bar_view, HeaderBarView, HeaderMenu,
};

pub mod status_bar_view;
pub use status_bar_view::{
    new_status_bar_view, sbv_add_segment, sbv_display_string, sbv_set_message, sbv_to_json,
    sbv_update_segment, StatusBarView, StatusSegment,
};

pub mod pie_menu_view;
pub use pie_menu_view::{
    new_pie_menu_view, pmv_add_slice, pmv_hide, pmv_show, pmv_slice_at_angle, pmv_slice_position,
    pmv_to_json, PieMenuView, PieSlice,
};

pub mod eevee_settings_view;
pub use eevee_settings_view::{
    eevee_set_bloom, eevee_set_samples, eevee_set_shadow_cube, eevee_set_ssr,
    eevee_settings_to_json, new_eevee_settings_view, EeveeSettingsView,
};

pub mod cycles_settings_view;
pub use cycles_settings_view::{
    cycles_set_diffuse, cycles_set_glossy, cycles_set_max_bounces, cycles_set_samples,
    cycles_settings_to_json, new_cycles_settings_view, CyclesSettingsView,
};

pub mod light_linking_view;
pub use light_linking_view::{
    light_link_add, light_link_count, light_link_excluded_count, light_link_remove_light,
    light_linking_to_json, new_light_linking_view, LightLink, LightLinkingView,
};

pub mod shadow_catcher_view;
pub use shadow_catcher_view::{
    new_shadow_catcher_view, shadow_catcher_add_object, shadow_catcher_set_enabled,
    shadow_catcher_set_intensity, shadow_catcher_set_only_shadow, shadow_catcher_to_json,
    ShadowCatcherView,
};

pub mod volume_scatter_view;
pub use volume_scatter_view::{
    new_volume_scatter_debug_view, new_volume_scatter_view, volume_albedo_color,
    volume_density_color, volume_mean_free_path, volume_phase_function_hg,
    volume_scatter_optical_depth, volume_scatter_set_anisotropy, volume_scatter_set_density,
    volume_scatter_set_step_size, volume_scatter_show_bounds, volume_scatter_to_json,
    VolumeScatterDebugView, VolumeScatterView,
};

pub mod subsurface_profile_view;
pub use subsurface_profile_view::{
    new_subsurface_profile_view, sss_profile_set_ior, sss_profile_set_radius,
    sss_profile_set_scale, sss_profile_show, sss_profile_to_json, SubsurfaceProfileView,
};

pub mod hair_render_view;
pub use hair_render_view::{
    hair_render_estimated_tris, hair_render_set_as_mesh, hair_render_set_strand_count,
    hair_render_set_width, hair_render_show_guides, hair_render_to_json, new_hair_render_view,
    HairRenderView,
};

pub mod particle_render_view;
pub use particle_render_view::{
    new_particle_render_view, particle_render_displayed_count, particle_render_set_mode,
    particle_render_set_percentage, particle_render_set_velocity_scale,
    particle_render_show_velocity, particle_render_to_json, ParticleDisplayMode,
    ParticleRenderView,
};

pub mod object_motion_path_view;
pub use object_motion_path_view::{
    motion_path_frame_count, motion_path_set_enabled, motion_path_set_line_width,
    motion_path_set_range, motion_path_show_frames, new_object_motion_path_view,
    object_motion_path_to_json, ObjectMotionPathView,
};

pub mod constraint_indicator_view;
pub use constraint_indicator_view::{
    constraint_ind_active_count, constraint_ind_add, constraint_ind_mute,
    constraint_indicator_to_json, new_constraint_indicator_view, ConstraintIndicator,
    ConstraintIndicatorView,
};

pub mod custom_property_view;
pub use custom_property_view::{
    custom_prop_add, custom_prop_get, custom_prop_remove, custom_prop_set, custom_property_to_json,
    new_custom_property_view, CustomProperty, CustomPropertyView,
};

pub mod modifier_stack_view;
pub use modifier_stack_view::{
    add_modifier_slot, modifier_count, move_modifier_up, new_modifier_stack_view, toggle_modifier,
    ModifierSlot, ModifierStackView,
};

pub mod particle_system_view;
pub use particle_system_view::{
    new_particle_system_view, particle_count as particle_system_view_count, set_frame_range,
    toggle_visibility, ParticleSystemView,
};

pub mod hair_system_view;
pub use hair_system_view::{
    hair_system_control_points, hair_system_set_length, hair_system_set_segments,
    hair_system_set_strand_count, hair_system_set_visible, hair_system_to_json,
    new_hair_system_view, HairSystemType, HairSystemView,
};

pub mod rigid_body_props_view;
pub use rigid_body_props_view::{
    new_rigid_body_props_view, rbpv_kinetic_energy_proxy, rbpv_set_friction,
    rbpv_set_linear_damping, rbpv_set_mass, rbpv_set_restitution, rigid_body_props_to_json,
    RigidBodyPropsView, RigidBodyTypeView,
};

pub mod cloth_props_view;
pub use cloth_props_view::{
    cloth_props_to_json, cloth_pv_set_bending, cloth_pv_set_damping, cloth_pv_set_mass,
    cloth_pv_set_quality, cloth_pv_set_tension, new_cloth_props_view, ClothPropsView,
};

pub mod render_region_view;
pub use render_region_view::{
    new_render_region_view, render_region_view_to_json, rrv_region_area, rrv_set_border_color,
    rrv_set_bounds, rrv_set_enabled as rrv_region_set_enabled, RenderRegionView,
};

pub mod safe_area_view;
pub use safe_area_view::{
    new_safe_area_view, safe_area_view_to_json, sav_action_safe_area, sav_set_action_margin,
    sav_set_title_margin, sav_show_action, sav_show_title, SafeAreaPreset, SafeAreaView,
};

pub mod aspect_ratio_view;
pub use aspect_ratio_view::{
    arv_aspect_value, arv_set_enabled, arv_set_mask_alpha, arv_set_ratio,
    aspect_ratio_view_to_json, new_aspect_ratio_view, AspectRatioView,
};

pub mod frame_guides_view;
pub use frame_guides_view::{
    fgv_grid_divisions, fgv_set_color, fgv_set_enabled as frame_guides_set_enabled,
    fgv_set_guide_type, fgv_set_line_width, frame_guides_view_to_json, new_frame_guides_view,
    FrameGuidesView, GuideType,
};

pub mod metadata_overlay_view;
pub use metadata_overlay_view::{
    metadata_overlay_view_to_json, mov_exposure_value, mov_set_aperture, mov_set_focal_length,
    mov_set_iso, mov_set_shutter, new_metadata_overlay_view, MetadataOverlayView,
};

pub mod timecode_overlay_view;
pub use timecode_overlay_view::{
    new_timecode_overlay_view, tcv_format_string, tcv_set_enabled, tcv_set_format,
    tcv_set_position, tcv_set_timecode, tcv_total_frames, timecode_overlay_view_to_json,
    TimecodeFormat, TimecodeOverlayView,
};

pub mod slate_view;
pub use slate_view::{
    new_slate_view, slate_view_to_json, slv_next_take, slv_set_opacity, slv_set_production,
    slv_set_scene, slv_set_take, SlateView,
};

pub mod clapper_view;
pub use clapper_view::{
    clapper_view_to_json, clv_animate_tick, clv_set_animation_speed, clv_set_scene, clv_set_state,
    clv_set_take, new_clapper_view, ClapperState, ClapperView,
};

pub mod countdown_leader_view;
pub use countdown_leader_view::{
    clv_advance_frame, clv_remaining_seconds, clv_reset, clv_set_frame_rate, clv_set_start_count,
    countdown_leader_view_to_json, new_countdown_leader_view, CountdownLeaderView,
};

pub mod test_pattern_view;
pub use test_pattern_view::{
    new_test_pattern_view, test_pattern_view_to_json, tpv_bar_count, tpv_set_enabled,
    tpv_set_grid_size, tpv_set_pattern, tpv_set_solid_color, TestPatternType, TestPatternView,
};

pub mod focus_peaking_view;
pub use focus_peaking_view::{
    focus_peaking_view_to_json, fpv_is_in_focus, fpv_set_enabled, fpv_set_highlight_color,
    fpv_set_sensitivity, fpv_set_threshold, new_focus_peaking_view, FocusPeakingView,
};

pub mod zebra_stripes_view;
pub use zebra_stripes_view::{
    new_zebra_stripes_view, zebra_stripes_view_to_json, zsv_is_overexposed, zsv_set_enabled,
    zsv_set_lower_threshold, zsv_set_stripe_frequency, zsv_set_upper_threshold, ZebraStripesView,
};

pub mod false_color_view;
pub use false_color_view::{
    false_color_view_to_json, fcv_map_luminance, fcv_set_enabled, fcv_set_legend_visible,
    fcv_set_luminance_range, fcv_set_palette, new_false_color_view, FalseColorPalette,
    FalseColorView,
};

pub mod luma_key_view;
pub use luma_key_view::{
    lkv_evaluate, lkv_set_clip_levels, lkv_set_enabled, lkv_set_invert, lkv_set_key_range,
    luma_key_view_to_json, new_luma_key_view, LumaKeyView,
};

pub mod chroma_key_view;
pub use chroma_key_view::{
    chroma_key_view_to_json, ckv_chroma_distance, ckv_evaluate, ckv_set_invert, ckv_set_key_color,
    ckv_set_softness, ckv_set_spill_suppression, ckv_set_tolerance, new_chroma_key_view,
    ChromaKeyView,
};

pub mod garbage_matte_view;
pub use garbage_matte_view::{
    garbage_matte_view_to_json, gmv_add_point, gmv_bounding_area, gmv_clear_points,
    gmv_set_enabled, gmv_set_feather, gmv_set_invert, new_garbage_matte_view, GarbageMatteView,
    MattePoint,
};

pub mod physics_debug_view;
pub use physics_debug_view::{
    new_physics_debug_view, pdv_set_enabled, pdv_set_opacity, pdv_set_show_colliders,
    pdv_set_show_forces, physics_debug_view_to_json, PhysicsDebugView,
};

pub mod collision_shape_view;
pub use collision_shape_view::{
    collision_shape_view_to_json, csv_clear_filter, csv_set_color,
    csv_set_enabled as collision_shape_set_enabled, csv_set_filter, csv_set_line_width,
    new_collision_shape_view, CollisionShapeKind, CollisionShapeView,
};

pub mod joint_pivot_view;
pub use joint_pivot_view::{
    joint_pivot_view_to_json, jpv_set_axis_length, jpv_set_color_x, jpv_set_color_y,
    jpv_set_enabled, jpv_set_pivot_radius, new_joint_pivot_view, JointPivotView,
};

pub mod force_vector_view;
pub use force_vector_view::{
    force_vector_view_to_json, fvv_display_length, fvv_set_enabled, fvv_set_min_magnitude,
    fvv_set_scale, fvv_set_show_torque, new_force_vector_view, ForceVectorView,
};

pub mod velocity_arrow_view;
pub use velocity_arrow_view::{
    new_velocity_arrow_view, vav_arrow_length, vav_color_at_speed, vav_set_enabled,
    vav_set_max_speed, vav_set_scale, velocity_arrow_view_to_json, VelocityArrowView,
};

pub mod acceleration_view;
pub use acceleration_view::{
    acceleration_view_to_json, acv_display_length, acv_set_color,
    acv_set_enabled as acceleration_set_enabled, acv_set_min_magnitude, acv_set_scale,
    new_acceleration_view, AccelerationView,
};

pub mod contact_point_view;
pub use contact_point_view::{
    contact_point_view_to_json, cpv_set_color_normal, cpv_set_color_point, cpv_set_enabled,
    cpv_set_normal_length, cpv_set_point_size, new_contact_point_view, ContactPointView,
};

pub mod friction_cone_view;
pub use friction_cone_view::{
    fcv_cone_radius, fcv_set_color, fcv_set_cone_height,
    fcv_set_enabled as friction_cone_set_enabled, fcv_set_friction_coefficient,
    friction_cone_view_to_json, new_friction_cone_view, FrictionConeView,
};

pub mod bounding_volume_view;
pub use bounding_volume_view::{
    bounding_volume_view_to_json, bvv_set_color_inner, bvv_set_color_leaf, bvv_set_enabled,
    bvv_set_max_depth, bvv_set_show_inner, new_bounding_volume_view, BoundingVolumeView,
};

pub mod broad_phase_view;
pub use broad_phase_view::{
    bpv_set_color_aabb, bpv_set_color_overlap, bpv_set_enabled as broad_phase_set_enabled,
    bpv_set_show_pairs, broad_phase_view_to_json, new_broad_phase_view, BroadPhaseView,
};

pub mod narrow_phase_view;
pub use narrow_phase_view::{
    narrow_phase_view_to_json, new_narrow_phase_view, npv_depth_display_length,
    npv_set_depth_scale, npv_set_enabled, npv_set_show_depth, npv_set_show_witnesses,
    NarrowPhaseView,
};

pub mod island_debug_view;
pub use island_debug_view::{
    idv_island_color, idv_set_alpha, idv_set_enabled, idv_set_max_islands, idv_set_show_id,
    island_debug_view_to_json, new_island_debug_view, IslandDebugView,
};

pub mod sleeping_body_view;
pub use sleeping_body_view::{
    new_sleeping_body_view, sbv_set_awake_tint, sbv_set_enabled, sbv_set_show_timer,
    sbv_set_sleeping_tint, sbv_tint_for_state, sleeping_body_view_to_json, SleepingBodyView,
};

pub mod constraint_debug_view;
pub use constraint_debug_view::{
    cdv_color_for_state, cdv_set_enabled, cdv_set_force_scale, cdv_set_show_forces,
    cdv_set_show_limits, constraint_debug_view_to_json, new_constraint_debug_view,
    ConstraintDebugView,
};

pub mod ragdoll_debug_view;
pub use ragdoll_debug_view::{
    new_ragdoll_debug_view, ragdoll_debug_view_to_json, rdv_set_body_color,
    rdv_set_enabled as ragdoll_debug_set_enabled, rdv_set_joint_radius, rdv_set_show_bodies,
    rdv_set_show_joints, RagdollDebugView,
};

pub mod cloth_debug_view;
pub use cloth_debug_view::{
    cldv_set_enabled, cldv_set_particle_size, cldv_set_show_constraints, cldv_set_show_particles,
    cldv_set_show_velocities, cldv_set_velocity_scale, cloth_debug_view_to_json,
    new_cloth_debug_view, ClothDebugView,
};
pub mod fluid_debug_view;
pub use fluid_debug_view::{
    fdv_set_enabled, fdv_set_opacity, fdv_set_show_pressure, fdv_set_show_velocity,
    fdv_set_velocity_scale, fluid_debug_view_to_json, new_fluid_debug_view, FluidDebugView,
};
pub mod smoke_debug_view;
pub use smoke_debug_view::{
    new_smoke_debug_view, sdv_set_color_scale, sdv_set_density_threshold, sdv_set_enabled,
    sdv_set_slice_axis, sdv_set_slice_position, smoke_debug_view_to_json, SmokeDebugView,
};
pub mod fire_debug_view;
pub use fire_debug_view::{
    fire_debug_view_to_json, frdv_normalize_temp, frdv_set_enabled, frdv_set_show_fuel,
    frdv_set_show_temperature, frdv_set_temp_max, frdv_set_temp_min, new_fire_debug_view,
    FireDebugView,
};
pub mod sph_particle_view;
pub use sph_particle_view::{
    new_sph_particle_view, sph_normalize_speed, sph_particle_view_to_json, sph_set_color_by_speed,
    sph_set_enabled, sph_set_max_speed, sph_set_opacity, sph_set_particle_radius, SphParticleView,
};
pub mod mpm_grid_view;
pub use mpm_grid_view::{
    mpm_grid_view_to_json, mpm_set_enabled, mpm_set_mass_threshold, mpm_set_node_size,
    mpm_set_show_mass, mpm_set_show_momentum, new_mpm_grid_view, MpmGridView,
};
