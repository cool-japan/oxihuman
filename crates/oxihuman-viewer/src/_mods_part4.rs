pub mod voxel_grid_view;
pub use voxel_grid_view::{
    new_voxel_grid_view, vgv_normalize, vgv_set_enabled, vgv_set_show_empty, vgv_set_value_max,
    vgv_set_value_min, vgv_set_voxel_opacity, voxel_grid_view_to_json, VoxelGridView,
};
pub mod level_set_view;
pub use level_set_view::{
    level_set_view_to_json, lsv_set_enabled, lsv_set_iso_value, lsv_set_show_gradient,
    lsv_set_surface_opacity, lsv_set_wireframe, new_level_set_view, LevelSetView,
};
pub mod signed_distance_view;
pub use signed_distance_view::{
    new_signed_distance_view, sdv2_color_for_dist, sdv2_set_band_width, sdv2_set_enabled,
    sdv2_set_exterior_color, sdv2_set_interior_color, sdv2_set_show_zero_crossing,
    signed_distance_view_to_json, SignedDistanceView,
};
pub mod curl_noise_view;
pub use curl_noise_view::{
    cnv_angular_freq, cnv_set_arrow_density, cnv_set_arrow_scale, cnv_set_enabled,
    cnv_set_frequency, cnv_set_show_magnitude, curl_noise_view_to_json, new_curl_noise_view,
    CurlNoiseView,
};
pub mod turbulence_view;
pub use turbulence_view::{
    new_turbulence_view, turb_normalize_tke, turb_set_eddy_scale, turb_set_enabled,
    turb_set_show_eddies, turb_set_tke_max, turb_set_tke_min, turbulence_view_to_json,
    TurbulenceView,
};
pub mod pressure_field_view;
pub use pressure_field_view::{
    new_pressure_field_view, pfv_normalize, pfv_set_enabled, pfv_set_isobar_count,
    pfv_set_pressure_max, pfv_set_pressure_min, pfv_set_show_isobars, pressure_field_view_to_json,
    PressureFieldView,
};
pub mod velocity_field_view;
pub use velocity_field_view::{
    new_velocity_field_view, velocity_field_view_to_json, vfv_arrow_length, vfv_set_arrow_scale,
    vfv_set_clamp_arrows, vfv_set_enabled, vfv_set_grid_stride, vfv_set_max_speed,
    VelocityFieldView,
};
pub mod temperature_field_view;
pub use temperature_field_view::{
    new_temperature_field_view, temperature_field_view_to_json, tfv_normalize, tfv_set_enabled,
    tfv_set_isotherm_count, tfv_set_show_isotherms, tfv_set_temp_max, tfv_set_temp_min,
    TemperatureFieldView,
};
pub mod density_field_view;
pub use density_field_view::{
    density_field_view_to_json, dfv_normalize, dfv_set_density_max, dfv_set_density_min,
    dfv_set_enabled, dfv_set_log_scale, dfv_set_opacity, new_density_field_view, DensityFieldView,
};
pub mod vorticity_field_view;
pub use vorticity_field_view::{
    new_vorticity_field_view, vorticity_field_view_to_json, vvf_is_visible, vvf_set_enabled,
    vvf_set_magnitude_threshold, vvf_set_opacity, vvf_set_show_direction, vvf_set_vorticity_scale,
    VorticityFieldView,
};
pub mod streamline_view;
pub use streamline_view::{
    new_streamline_view, slv_set_enabled, slv_set_line_opacity, slv_set_max_steps,
    slv_set_seed_count, slv_set_step_size, slv_vertex_budget, streamline_view_to_json,
    StreamlineView,
};
pub mod lod_debug_view;
pub use lod_debug_view::{
    lddv_color_intensity, lddv_set_distance_threshold, lddv_set_enabled, lddv_set_lod_count,
    lddv_set_show_labels, lod_debug_view_to_json, new_lod_debug_view, LodDebugView,
};
pub mod frustum_cull_view;
pub use frustum_cull_view::{
    fcv_set_enabled as frustum_cull_set_enabled, fcv_set_highlight_culled,
    fcv_set_show_frustum_planes, fcv_update_cull_ratio, fcv_visible_count,
    frustum_cull_view_to_json, new_frustum_cull_view, FrustumCullView,
};
pub mod occlusion_cull_view;
pub use occlusion_cull_view::{
    new_occlusion_cull_view, occlusion_cull_view_to_json, occv_set_enabled,
    occv_set_highlight_occluded, occv_set_query_latency, occv_set_show_occluder_proxies,
    occv_update_occluded_ratio, OcclusionCullView,
};
pub mod draw_call_view;
pub use draw_call_view::{
    dcv_batch_efficiency, dcv_is_over_threshold, dcv_set_enabled, dcv_set_show_batch_boundaries,
    dcv_set_warn_threshold, dcv_update_counts, draw_call_view_to_json, new_draw_call_view,
    DrawCallView,
};
pub mod gpu_memory_view;
pub use gpu_memory_view::{
    gmv_set_budget_bytes, gmv_set_buffer_bytes, gmv_set_enabled as gpu_mem_set_enabled,
    gmv_set_texture_bytes, gmv_total_bytes, gmv_usage_fraction, gpu_memory_view_to_json,
    new_gpu_memory_view, GpuMemoryView,
};
pub mod shader_compile_view;
pub use shader_compile_view::{
    new_shader_compile_view, scv_all_ready, scv_progress, scv_set_enabled,
    scv_set_show_error_details, scv_update_counts, shader_compile_view_to_json, ShaderCompileView,
    ShaderStatus,
};
pub mod texture_atlas_view;
pub use texture_atlas_view::{
    new_texture_atlas_view, tav_set_dimensions, tav_set_enabled as tex_atlas_set_enabled,
    tav_set_region_count, tav_set_show_borders, tav_total_texels, texture_atlas_view_to_json,
    TextureAtlasView,
};
pub mod mipmap_chain_view;
pub use mipmap_chain_view::{
    mipmap_chain_view_to_json, mmv_level_dimensions, mmv_set_base_dimensions, mmv_set_enabled,
    mmv_set_show_level_borders, mmv_total_texels, new_mipmap_chain_view, MipmapChainView,
};
pub mod render_queue_view;
pub use render_queue_view::{
    new_render_queue_view, render_queue_view_to_json, rqv_normalize_priority, rqv_set_enabled,
    rqv_set_max_priority, rqv_set_queue_depth, rqv_set_show_priority_colors, RenderQueueView,
};
pub mod material_instance_view;
pub use material_instance_view::{
    material_instance_view_to_json, miv_avg_instances_per_base, miv_set_enabled,
    miv_set_highlight_overdraw, miv_set_show_overrides, miv_update_counts,
    new_material_instance_view, MaterialInstanceView,
};
pub mod vertex_buffer_view;
pub use vertex_buffer_view::{
    new_vertex_buffer_view, vbv_set_enabled, vbv_set_show_attribute_layout, vbv_set_stride,
    vbv_update_stats, vbv_vertex_count, vertex_buffer_view_to_json, VertexBufferView,
};
pub mod index_buffer_view;
pub use index_buffer_view::{
    ibv_set_enabled, ibv_set_index_format, ibv_set_show_primitive_count, ibv_total_bytes,
    ibv_triangle_count, ibv_update_stats, index_buffer_view_to_json, new_index_buffer_view,
    IndexBufferView,
};
pub mod uniform_buffer_view;
pub use uniform_buffer_view::{
    new_uniform_buffer_view, ubv_avg_bytes_per_ubo, ubv_set_enabled, ubv_set_show_binding_slots,
    ubv_update_stats, uniform_buffer_view_to_json, UniformBufferView,
};
pub mod pipeline_state_view;
pub use pipeline_state_view::{
    new_pipeline_state_view, pipeline_state_view_to_json, psv_cache_hit_ratio,
    psv_set_enabled as pipeline_set_enabled, psv_set_show_state_diff, psv_update_stats,
    PipelineStateView,
};
pub mod render_target_debug_view;
pub use render_target_debug_view::{
    new_render_target_debug_view, render_target_debug_view_to_json, rtdv_set_attachment_count,
    rtdv_set_dimensions, rtdv_set_enabled, rtdv_set_show_depth_attachment, rtdv_total_pixels,
    RenderTargetDebugView,
};
pub mod compute_dispatch_view;
pub use compute_dispatch_view::{
    cdv_set_dispatches, cdv_set_enabled as compute_dispatch_set_enabled, cdv_set_workgroup_size,
    cdv_threads_per_dispatch, cdv_total_threads, compute_dispatch_view_to_json,
    new_compute_dispatch_view, ComputeDispatchView,
};

pub mod xr_viewport_v2;
pub use xr_viewport_v2::{
    new_xr_viewport_v2, xr_aspect_ratio_v2, xr_is_stereo_v2, xr_projection_matrix_v2,
    xr_stereo_offset_v2, XrViewportV2,
};

pub mod neural_render;
pub use neural_render::{
    neural_render_is_valid, neural_render_latent_size, neural_render_memory_mb,
    neural_render_param_count, new_neural_render_config, NeuralRenderConfig,
};

pub mod gaussian_splat_view;
pub use gaussian_splat_view::{
    new_gaussian_splat_config, splat_cull_count, splat_memory_mb, splat_param_count,
    splat_sh_coeff_count, GaussianSplatConfig,
};

pub mod nerf_view;
pub use nerf_view::{
    nerf_is_valid, nerf_memory_mb, nerf_ray_count, nerf_sample_count_per_frame, new_nerf_config,
    NerfConfig,
};

pub mod lidar_point_view;
pub use lidar_point_view::{
    lidar_bounding_box, lidar_filter_by_return, lidar_mean_intensity, lidar_point_count,
    lidar_push_point, new_lidar_point_cloud, LidarPointCloud,
};

pub mod thermal_camera_view;
pub use thermal_camera_view::{
    new_thermal_camera_config, thermal_hot_spot, thermal_mean_temp, thermal_pixel_to_temp,
    thermal_temp_to_color, ThermalCameraConfig,
};

pub mod mri_slice_view;
pub use mri_slice_view::{
    mri_get_pixel, mri_mean_value, mri_pixel_count, mri_set_pixel, mri_window_to_display,
    new_mri_slice, MriSlice,
};

pub mod ct_scan_view;
pub use ct_scan_view::{
    ct_density_above, ct_get_voxel, ct_set_voxel, ct_volume_mm3, ct_voxel_count, new_ct_volume,
    CtVolume,
};

pub mod photogrammetry_view;
pub use photogrammetry_view::{
    new_photogrammetry_model, photogram_is_calibrated, photogram_point_density,
    photogram_quality_score, photogram_real_scale_factor, PhotogrammetryModel,
};

pub mod holographic_display;
pub use holographic_display::{
    hologram_bandwidth_ghz, hologram_diffraction_limit_um, hologram_field_of_view,
    hologram_is_retinal, new_hologram_config, HologramConfig,
};

pub mod light_field_camera;
pub use light_field_camera::{
    lf_angular_resolution, lf_memory_mb, lf_spatial_resolution, lf_total_views,
    new_light_field_config, LightFieldConfig,
};

pub mod ultrasound_view;
pub use ultrasound_view::{
    new_ultrasound_image, us_axial_resolution_mm, us_get_pixel, us_mean_echogenicity,
    us_pixel_to_depth_mm, us_set_pixel, UltrasoundImage,
};

pub mod microscopy_view;
pub use microscopy_view::{
    micro_field_of_view_um, micro_get_pixel, micro_pixel_count, micro_resolution_um,
    micro_set_pixel, new_microscopy_image, MicroscopyImage,
};

pub mod histology_view;
pub use histology_view::{
    histo_get_pixel, histo_mean_eosin, histo_mean_hematoxylin, histo_pixel_count, histo_set_pixel,
    new_histology_slide, HistologySlide,
};

pub mod dermoscopy_view;
pub use dermoscopy_view::{
    dermo_asymmetry_score, dermo_get_pixel, dermo_mean_color, dermo_pixel_count, dermo_set_pixel,
    new_dermoscopy_image, DermoscopyImage,
};

pub mod endoscopy_view;
pub use endoscopy_view::{
    endo_duration_ms, endo_get_pixel, endo_mean_brightness, endo_pixel_count, endo_set_pixel,
    new_endoscopy_frame, EndoscopyFrame,
};

pub mod anisotropy_debug_view;
pub use anisotropy_debug_view::{
    aniso_debug_line, aniso_rotation_from_map, aniso_should_show, aniso_tangent_color,
    new_anisotropy_debug_view, AnisotropyDebugView,
};

pub mod iridescence_view;
pub use iridescence_view::{
    iridescence_color_at, iridescence_factor_color, iridescence_is_active,
    iridescence_thickness_range, new_iridescence_view, IridescenceView,
};

pub mod clearcoat_view;
pub use clearcoat_view::{
    clearcoat_factor_color, clearcoat_fresnel, clearcoat_is_visible, clearcoat_roughness_color,
    new_clearcoat_view, ClearcoatView,
};

pub mod sheen_view;
pub use sheen_view::{
    new_sheen_view, sheen_color_debug, sheen_directional_albedo, sheen_is_visible,
    sheen_roughness_color, SheenView,
};

pub mod transmission_view;
pub use transmission_view::{
    new_transmission_view, transmission_factor_color, transmission_fresnel_at_normal,
    transmission_ior_color, transmission_is_refractive, TransmissionView,
};

pub mod displacement_preview;
pub use displacement_preview::{
    displacement_is_elevated, displacement_magnitude_color, displacement_preview_color,
    displacement_preview_vector, new_displacement_preview, DisplacementPreview,
};

pub mod parallax_debug_view;
pub use parallax_debug_view::{
    new_parallax_debug_view, parallax_depth_color, parallax_iteration_color, parallax_offset,
    parallax_relief_steps, ParallaxDebugView,
};

pub mod ray_march_debug;
pub use ray_march_debug::{
    new_ray_march_debug, ray_march_distance_color, ray_march_efficiency, ray_march_hit_color,
    ray_march_step_color, RayMarchDebug,
};

pub mod sdf_debug_view;
pub use sdf_debug_view::{
    new_sdf_debug_view, sdf_color, sdf_contour_intensity, sdf_gradient_approx, sdf_is_surface,
    SdfDebugView,
};

pub mod gi_debug_view;
pub use gi_debug_view::{
    gi_irradiance_color, gi_is_converged, gi_probe_color, gi_radiance_to_ldr, new_gi_debug_view,
    GiDebugView,
};

pub mod irradiance_cache_view;
pub use irradiance_cache_view::{
    ic_sample_color, ic_sample_radius_color, ic_validity_color, ic_weight_color,
    new_irradiance_cache_view, IrradianceCacheView,
};

pub mod photon_map_view;
pub use photon_map_view::{
    new_photon_map_view, photon_density_color, photon_direction_color,
    photon_is_caustic_contributor, photon_power_color, PhotonMapView,
};

pub mod caustic_debug_view;
pub use caustic_debug_view::{
    caustic_concentration_factor, caustic_irradiance_color, caustic_is_bright,
    caustic_photon_hit_color, new_caustic_debug_view, CausticDebugView,
};

pub mod bidirectional_path_view;
pub use bidirectional_path_view::{
    bdpt_connection_weight, bdpt_depth_color, bdpt_path_color, bdpt_strategy_count,
    new_bdpt_debug_view, BdptDebugView,
};

pub mod metropolis_debug_view;
pub use metropolis_debug_view::{
    mlt_acceptance_color, mlt_energy_color, mlt_is_large_step, mlt_mutation_type_color,
    new_metropolis_debug_view, MetropolisDebugView,
};

pub mod spectral_render_view;
pub use spectral_render_view::{
    new_spectral_render_view, spectral_energy_at, spectral_is_visible, spectral_to_xyz,
    spectral_wavelength_to_rgb, SpectralRenderView,
};

pub mod fluorescence_view;
pub use fluorescence_view::{
    fluor_emission_color, fluor_energy_ratio, fluor_is_uv_excited, fluor_stokes_shift_nm,
    new_fluorescence_view, FluorescenceView,
};

pub mod polarization_view;
pub use polarization_view::{
    new_polarization_view, polar_degree_of_polarization, polar_is_circularly_polarized,
    polar_linear_angle_deg, polar_to_color, PolarizationView,
};

pub mod bssrdf_view;
pub use bssrdf_view::{
    bssrdf_dipole_profile, bssrdf_is_within_radius, bssrdf_mean_free_path, bssrdf_radius_color,
    new_bssrdf_view, BssrdfView,
};

pub mod hair_shader_view;
pub use hair_shader_view::{
    hair_azimuthal_distribution, hair_r_lobe_color, hair_trt_lobe_color, hair_tt_lobe_color,
    new_hair_shader_view, HairShaderView,
};

pub mod skin_shader_view;
pub use skin_shader_view::{
    new_skin_shader_view, skin_diffuse_color, skin_layer_weight, skin_specular_color,
    skin_sss_radius_color, SkinShaderView,
};

pub mod velvet_shader_view;
pub use velvet_shader_view::{
    new_velvet_view, velvet_retroreflection, velvet_rim_highlight, velvet_sheen_color,
    velvet_silhouette_boost, VelvetView,
};

pub mod toon_shader_view;
pub use toon_shader_view::{
    new_toon_view, toon_is_outline, toon_outline_factor, toon_quantize, toon_shade, ToonView,
};

pub mod water_surface_view;
pub use water_surface_view::{
    new_water_surface_view, water_foam_factor, water_fresnel, water_gerstner_height,
    water_normal_from_height, WaterSurfaceView,
};

pub mod fire_sim_view;
pub use fire_sim_view::{
    fire_blackbody_color, fire_density_color, fire_is_hot, fire_temperature_color, new_fire_view,
    FireView,
};

pub mod smoke_sim_view;
pub use smoke_sim_view::{
    new_smoke_view, smoke_density_color, smoke_opacity, smoke_temperature_color,
    smoke_velocity_color, SmokeView,
};

pub mod sand_sim_view;
pub use sand_sim_view::{
    new_sand_view, sand_flow_color, sand_is_flowing, sand_packing_fraction, sand_stress_color,
    SandView,
};

pub mod ice_shader_view;
pub use ice_shader_view::{
    ice_absorption_color, ice_caustic_strength, ice_fresnel, ice_refraction_offset,
    new_ice_shader_view, IceShaderView,
};

pub mod glass_shader_view;
pub use glass_shader_view::{
    glass_critical_angle_deg, glass_fresnel_schlick, glass_refraction_dir,
    glass_transmission_color, new_glass_shader_view, GlassShaderView,
};

pub mod metal_shader_view;
pub use metal_shader_view::{
    metal_fresnel_conductor, metal_fresnel_rgb, metal_is_mirror, metal_reflectance_at_normal,
    new_metal_shader_view, MetalShaderView,
};

pub mod snow_render_view;
pub use snow_render_view::{
    new_snow_render_view, snow_ao_color, snow_is_fresh, snow_sparkle_intensity, snow_sss_color,
    SnowRenderView,
};

pub mod depth_peeling_view;
pub use depth_peeling_view::{
    dp_blend, dp_coverage, dp_is_single_pass, dp_set_layer_count, new_depth_peeling_view,
    DepthPeelingView,
};

pub mod weighted_oit_view;
pub use weighted_oit_view::{
    new_woit_view, woit_blend, woit_is_high_power, woit_set_weight_power, woit_weight, WoitView,
};

pub mod taa_view;
pub use taa_view::{
    new_taa_view, taa_blend, taa_halton_jitter, taa_is_aggressive, taa_set_blend_factor, TaaView,
};

pub mod blue_noise_view;
pub use blue_noise_view::{
    bn_blend, bn_interleaved_gradient, bn_is_animated, bn_set_intensity, new_blue_noise_view,
    BlueNoiseView,
};

pub mod ordered_dither_view;
pub use ordered_dither_view::{
    new_ordered_dither_view, od_apply, od_bayer4_threshold, od_blend, od_is_fine_grain,
    od_set_matrix_size, OrderedDitherView,
};

pub mod color_management_view;
pub use color_management_view::{
    cm_blend as cm_view_blend, cm_encode, cm_is_linear_workflow, cm_linearize, cm_set_exposure,
    new_color_management_view, ColorManagementView,
};

pub mod halton_sequence_view;
pub use halton_sequence_view::{
    halton_is_coprime_bases, halton_jitter, halton_point, halton_sample, new_halton_view,
    HaltonView,
};

pub mod reconstruction_filter_view;
pub use reconstruction_filter_view::{
    box_filter, filter_is_separable, gaussian_filter, lanczos, mitchell_netravali,
};

pub mod voxel_cone_tracing_view;
pub use voxel_cone_tracing_view::{
    new_vct_view, vct_blend, vct_cone_half_angle_rad, vct_is_high_resolution, vct_set_resolution,
    VctView,
};

pub mod screen_space_gi_view;
pub use screen_space_gi_view::{
    new_ssgi_view, ssgi_blend, ssgi_irradiance_weight, ssgi_is_high_quality, ssgi_set_sample_count,
    SsgiView,
};

pub mod dlss_view;
pub use dlss_view::{
    dlss_is_enabled, dlss_render_resolution, dlss_scale_factor, dlss_set_quality, new_dlss_view,
    DlssQuality, DlssView,
};

pub mod fsr_view;
pub use fsr_view::{
    fsr_is_enabled, fsr_render_resolution, fsr_scale_factor, fsr_set_mode, new_fsr_view, FsrMode,
    FsrView,
};

pub mod volumetric_shadow_view;
pub use volumetric_shadow_view::{
    new_vol_shadow_view, vol_shadow_blend, vol_shadow_is_dense, vol_shadow_set_extinction,
    vol_shadow_transmittance, VolShadowView,
};

pub mod deep_shadow_view;
pub use deep_shadow_view::{
    ds_blend, ds_is_high_depth, ds_set_layer_count, ds_transmittance, new_deep_shadow_view,
    DeepShadowView,
};

pub mod stochastic_transparency_view;
pub use stochastic_transparency_view::{
    new_stochastic_transparency_view, st_alpha_test, st_blend, st_effective_alpha,
    st_is_high_sample, st_set_sample_count, StochasticTransparencyView,
};

pub mod error_diffusion_view;
pub use error_diffusion_view::{
    ed_blend, ed_floyd_steinberg_weights, ed_is_high_fidelity, ed_quantize, ed_set_color_levels,
    new_error_diffusion_view, ErrorDiffusionView,
};

pub mod chromatic_split_view;
pub use chromatic_split_view::{
    csv_blend, csv_disable, csv_enable, csv_is_enabled, csv_set_offset,
    csv_to_json as csplit_to_json, new_chromatic_split_view, ChromaticSplitView,
};

pub mod heatmap_depth_view;
pub use heatmap_depth_view::{
    hdv_depth_to_color, hdv_enable, hdv_is_enabled, hdv_set_range, hdv_to_json,
    new_heatmap_depth_view, HeatmapDepthView,
};

pub mod tangent_frame_view;
pub use tangent_frame_view::{
    new_tangent_frame_view, tfv_active_channel_count, tfv_enable, tfv_set_scale, tfv_to_json,
    tfv_toggle_normal, tfv_toggle_tangent, TangentFrameView,
};

pub mod mesh_density_view;
pub use mesh_density_view::{
    mdv_density_color, mdv_enable, mdv_is_enabled, mdv_set_thresholds, mdv_to_json,
    new_mesh_density_view, MeshDensityView,
};

pub mod mip_level_view;
pub use mip_level_view::{
    mlv_enable, mlv_is_enabled, mlv_level_count, mlv_mip_color, mlv_set_mip, mlv_to_json,
    new_mip_level_view, MipLevelView,
};

pub mod uv_stretch_view;
pub use uv_stretch_view::{
    new_uv_stretch_view, usv_enable, usv_is_enabled, usv_mode_name, usv_set_thresholds,
    usv_stretch_color, usv_to_json, UvStretchView,
};

pub mod vertex_color_view;
pub use vertex_color_view::{
    new_vertex_color_view, vcv_channel_name, vcv_enable, vcv_is_enabled, vcv_set_channel,
    vcv_to_json as vcolor_to_json, VertexColorView,
};

pub mod morph_delta_view;
pub use morph_delta_view::{
    mdlv_delta_color, mdlv_enable, mdlv_is_enabled, mdlv_set_scale, mdlv_to_json,
    new_morph_delta_view, MorphDeltaView,
};

pub mod bone_influence_view;
pub use bone_influence_view::{
    biv_enable, biv_influence_color, biv_is_enabled, biv_is_valid_bone, biv_set_bone, biv_to_json,
    new_bone_influence_view, BoneInfluenceView,
};

pub mod lightmap_density_view;
pub use lightmap_density_view::{
    ldv_density_color, ldv_enable, ldv_is_enabled, ldv_set_density, ldv_to_json as lmdv_to_json,
    ldv_toggle_grid as lmdv_toggle_grid, new_lightmap_density_view, LightmapDensityView,
};

pub mod roughness_debug_view;
pub use roughness_debug_view::{
    new_roughness_debug_view, rdv_enable, rdv_is_enabled, rdv_roughness_color, rdv_set_range,
    rdv_to_json as rough_to_json, RoughnessDebugView,
};

pub mod metalness_debug_view;
pub use metalness_debug_view::{
    mnv_enable, mnv_is_enabled, mnv_metalness_color, mnv_set_threshold, mnv_to_json,
    new_metalness_debug_view, MetalnessDebugView,
};

pub mod ao_debug_view;
pub use ao_debug_view::{
    adv_ao_color, adv_enable, adv_is_enabled, adv_set_power, adv_to_json, new_ao_debug_view,
    AoDebugView,
};

pub mod normal_channel_view;
pub use normal_channel_view::{
    ncv_active_channels, ncv_enable, ncv_is_enabled, ncv_set_mask, ncv_to_json,
    new_normal_channel_view, NormalChannelView,
};

pub mod emission_debug_view;
pub use emission_debug_view::{
    edv_emission_color, edv_enable, edv_is_enabled, edv_set_exposure, edv_to_json,
    new_emission_debug_view, EmissionDebugView,
};

pub mod cluster_view;
pub use cluster_view::{
    clv_cluster_color, clv_enable, clv_is_enabled, clv_set_cluster_count,
    clv_to_json as cluster_to_json, clv_toggle_centroids, new_cluster_view, ClusterView,
};

pub mod geodesic_heat_view;
pub use geodesic_heat_view::{
    ghv_distance_to_color, ghv_enable, ghv_is_enabled, ghv_set_max_distance, ghv_set_source,
    ghv_to_json, new_geodesic_heat_view, GeodesicHeatView,
};

pub mod curvature_map_view;
pub use curvature_map_view::{
    cmv_curvature_color, cmv_enable as curvmap_enable, cmv_is_enabled, cmv_mode_name, cmv_set_mode,
    cmv_set_scale, cmv_to_json, new_curvature_map_view, CurvatureMapView, CurvatureMode,
};

/* seam_view already registered above */

pub mod stretch_map_view;
pub use stretch_map_view::{
    new_stretch_map_view, smv_enable, smv_is_enabled, smv_set_thresholds, smv_stretch_color,
    smv_to_json as stretchmap_to_json, StretchMapView,
};

pub mod density_view;
pub use density_view::{
    dv_density_color, dv_enable, dv_is_enabled, dv_set_thresholds, dv_to_json, new_density_view,
    DensityView,
};

pub mod boundary_view;
pub use boundary_view::{
    bov_boundary_color, bov_enable, bov_is_enabled, bov_set_color, bov_to_json, bov_toggle_pulse,
    new_boundary_view, BoundaryView,
};

pub mod selection_mask_view;
pub use selection_mask_view::{
    new_selection_mask_view, smkv_enable, smkv_is_enabled, smkv_overlay_color, smkv_set_alpha,
    smkv_set_mode, smkv_to_json, SelectionMaskMode, SelectionMaskView,
};

pub mod weight_heat_view;
pub use weight_heat_view::{
    new_weight_heat_view, whv_enable, whv_is_enabled, whv_set_bone, whv_to_json,
    whv_toggle_all_bones, whv_weight_color, WeightHeatView,
};

pub mod morph_magnitude_view;
pub use morph_magnitude_view::{
    mmv_delta_color, mmv_enable, mmv_is_enabled, mmv_set_max_delta, mmv_to_json, mmv_toggle_arrows,
    new_morph_magnitude_view, MorphMagnitudeView,
};

/* velocity_field_view already registered above */

/* pressure_field_view already registered above */

pub mod stress_tensor_view;
pub use stress_tensor_view::{
    new_stress_tensor_view, stv_enable, stv_is_enabled, stv_principal_color, stv_set_glyph_scale,
    stv_to_json as stresstensor_to_json, stv_toggle_compression, stv_toggle_tension,
    StressTensorView,
};

pub mod contact_force_view;
pub use contact_force_view::{
    cfv_arrow_length, cfv_enable, cfv_force_color, cfv_is_enabled, cfv_set_force_scale,
    cfv_set_max_force, cfv_to_json, new_contact_force_view, ContactForceView,
};

pub mod particle_trail_view;
pub use particle_trail_view::{
    new_particle_trail_view, ptv_enable, ptv_is_enabled, ptv_set_trail_length,
    ptv_to_json as particletrail_to_json, ptv_toggle_fade, ptv_trail_alpha, ParticleTrailView,
};

pub mod fluid_vorticity_view;
pub use fluid_vorticity_view::{
    fvv_enable, fvv_is_enabled, fvv_set_max_vorticity, fvv_to_json, fvv_toggle_direction,
    fvv_vorticity_color, new_fluid_vorticity_view, FluidVorticityView,
};

pub mod topology_view;
pub use topology_view::{
    new_topology_view, tv_disable, tv_enable, tv_is_pole, tv_set_show_irregular, tv_set_show_poles,
    tv_to_json, tv_valence_color, TopologyView,
};

pub mod valence_map_view;
pub use valence_map_view::{
    new_valence_map_view, vmv_enable, vmv_is_regular, vmv_set_opacity, vmv_set_target_valence,
    vmv_to_json, vmv_valence_color, ValenceMapView,
};

pub mod ngon_highlight_view;
pub use ngon_highlight_view::{
    new_ngon_highlight_view, nhv_enable, nhv_is_ngon, nhv_set_color, nhv_set_min_verts,
    nhv_set_opacity, nhv_to_json, NgonHighlightView,
};

pub mod tri_highlight_view;
pub use tri_highlight_view::{
    new_tri_highlight_view, thv_enable, thv_is_tri, thv_set_color, thv_set_opacity,
    thv_set_show_count, thv_to_json as tri_highlight_to_json, thv_tri_ratio, TriHighlightView,
};

pub mod pole_visualizer;
pub use pole_visualizer::{
    new_pole_visualizer, pv_color_for_valence, pv_enable as pole_vis_enable, pv_is_pole,
    pv_set_e_pole_color, pv_set_glyph_radius, pv_set_n_pole_color, pv_to_json as pole_vis_to_json,
    PoleVisualizer,
};

pub mod angle_distortion_view;
pub use angle_distortion_view::{
    adv_angle_color, adv_distortion_score, adv_enable as angle_dist_enable, adv_set_max_deg,
    adv_set_opacity, adv_to_json as angle_dist_to_json, new_angle_distortion_view,
    AngleDistortionView,
};

pub mod shading_normal_view;
pub use shading_normal_view::{
    new_shading_normal_view, snv_enable, snv_normal_endpoint, snv_normal_facing, snv_set_scale,
    snv_set_show_face_normals, snv_to_json as shading_normal_to_json, ShadingNormalView,
};

pub mod tangent_basis_view;
pub use tangent_basis_view::{
    new_tangent_basis_view, tbv_active_channel_count, tbv_bitangent_color, tbv_enable,
    tbv_normal_color, tbv_set_scale, tbv_set_show_bitangent, tbv_set_show_tangent,
    tbv_tangent_color, tbv_to_json, TangentBasisView,
};

pub mod mikkt_view;
pub use mikkt_view::{
    mkv_bitangent, mkv_enable, mkv_handedness_color, mkv_set_scale, mkv_set_show_handedness,
    mkv_to_json, new_mikkt_view, MikktView,
};

pub mod lod_coverage_view;
pub use lod_coverage_view::{
    lcv_active_lod, lcv_enable, lcv_lod_color, lcv_set_threshold, lcv_to_json,
    new_lod_coverage_view, LodCoverageView,
};

pub mod material_id_view;
pub use material_id_view::{
    midv_color_for_id, midv_enable, midv_set_opacity, midv_set_palette_size, midv_to_json,
    new_material_id_view, MaterialIdView,
};

pub mod object_id_view;
pub use object_id_view::{
    new_object_id_view, oidv_color_for_id, oidv_enable, oidv_set_opacity, oidv_set_palette_size,
    oidv_to_json, ObjectIdView,
};

pub mod instance_id_view;
pub use instance_id_view::{
    iidv_color_for_id, iidv_enable, iidv_set_opacity, iidv_set_palette_size, iidv_set_tint_mode,
    iidv_to_json, new_instance_id_view, InstanceIdView,
};

pub mod depth_peel_view;
pub use depth_peel_view::{
    dpv_enable, dpv_layer_alpha, dpv_layer_color, dpv_set_alpha, dpv_set_layer_count, dpv_to_json,
    new_depth_peel_view, DepthPeelView,
};

pub mod vertex_paint_state;

