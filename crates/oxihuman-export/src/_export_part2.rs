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

