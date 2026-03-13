
pub mod mesh_circular_edge;
pub use mesh_circular_edge::{
    circular_edge_to_json, circumference, detect_circular_edges,
    largest_loop as largest_circular_loop, loop_count as circular_loop_count,
    make_edge as make_circular_edge, open_chain_count, CircularEdgeResult, Edge as CircularEdge,
};

pub mod mesh_coplanar_face;
pub use mesh_coplanar_face::{
    are_coplanar, detect_coplanar_faces, dot3 as coplanar_dot3,
    face_normal_raw as coplanar_face_normal_raw, group_count as coplanar_group_count,
    largest_group as coplanar_largest_group, normalize3 as coplanar_normalize3,
    total_grouped_faces, CoplanarGroup, CoplanarResult,
};

pub mod mesh_corner_angle;
pub use mesh_corner_angle::{
    angle_face_count, avg_min_angle, compute_corner_angles, corner_angle, corner_angle_to_json,
    max_corner_angle, min_corner_angle, triangle_angles, validate_angle_sums, CornerAngleResult,
};

pub mod mesh_decimate_error;
pub use mesh_decimate_error::{
    add_quadrics, compute_edge_errors, error_edge_count, evaluate_quadric, max_error, min_error,
    quadric_from_plane, zero_quadric, DecimateErrorResult, QuadricError,
};

pub mod mesh_directed_edge;
pub use mesh_directed_edge::{
    boundary_edge_count as directed_boundary_edge_count, build_directed_edges, directed_edge_count,
    edges_for_face, find_twin, half_pi_ref, has_twin as directed_has_twin, interior_edge_count,
    DirectedEdge as DirectedEdgeDEM, DirectedEdgeMesh,
};

pub mod mesh_edge_flow;
pub use mesh_edge_flow::{
    compute_edge_flow, flow_curl, flow_direction, flow_divergence, flow_magnitude_ef, flow_to_json,
    flow_vertex_field, smooth_edge_flow, EdgeFlow,
};

pub mod mesh_edge_loop_select;
pub use mesh_edge_loop_select::{
    clear_loop_selection, grow_loop_selection, loop_edge_count, loop_is_closed_els, loop_to_json,
    loop_vertices_els, select_edge_loop, shrink_loop_selection, EdgeLoopSelect,
};

pub mod mesh_face_flip;
pub use mesh_face_flip::{
    consistent_winding, detect_flipped, flip_all_faces_ff, flip_count, flip_face_ff,
    flip_normals_with_faces, flip_selected_faces, FaceFlip,
};

pub mod mesh_face_pair;
pub use mesh_face_pair::{
    detect_face_pairs, edge_key as face_pair_edge_key, face_pair_count, face_pair_to_json,
    faces_are_paired, max_paired_face, pairing_ratio, FacePair, FacePairResult,
};

pub mod mesh_grid_deform;
pub use mesh_grid_deform::{
    apply_grid_deform, control_point_count, deform_vertex as grid_deform_vertex,
    grid_deform_to_json, grid_index, new_grid_deform, set_grid_delta, trilinear, GridDeform,
    GridDeformResult,
};

pub mod mesh_half_face;
pub use mesh_half_face::{
    boundary_half_face_count, build_half_face_mesh, half_face_count, half_face_mesh_to_json,
    half_face_vertices, is_closed_half_face_mesh, paired_half_face_count, HalfFace, HalfFaceMesh,
};

pub mod mesh_incircle;
pub use mesh_incircle::{
    compute_incircles, count_large_incircles, edge_length_ic, get_incircle,
    incircle_result_to_json, triangle_incircle, Incircle, IncircleResult,
};

pub mod mesh_k_ring;
pub use mesh_k_ring::{
    avg_ring_size_all, build_vertex_adjacency, k_ring, k_ring_to_json, one_ring, ring_contains,
    ring_size, KRingResult,
};

pub mod mesh_arc_length;
pub use mesh_arc_length::{
    circle_polyline, compute_arc_length, sample_at_arc_length, sample_at_t,
    sample_count as arc_sample_count, segment_lengths, total_length as arc_total_length,
    uniform_resample, ArcLengthResult, ArcSample,
};

pub mod mesh_bvh_leaf;
pub use mesh_bvh_leaf::{
    avg_leaf_surface_area, build_leaf_aabbs, leaf_centroid, leaf_count, leaf_surface_area,
    leaf_volume, query_leaves_aabb, LeafAabb, LeafQueryResult,
};

pub mod mesh_cage_wrap;
pub use mesh_cage_wrap::{
    cage_wrap, cage_wrap_to_json, default_cage_wrap_config, wrapped_centroid, wrapped_vertex_count,
    CageWrapConfig, CageWrapResult,
};

pub mod mesh_catenary;
pub use mesh_catenary::{
    catenary_arc_length, catenary_sag, catenary_tube_vertex_count, catenary_y,
    default_catenary_params, euler_number, lowest_point, point_count as catenary_point_count,
    sample_catenary, to_positions as catenary_to_positions, CatenaryParams, CatenaryPoint,
};

pub mod mesh_collapse_group;
pub use mesh_collapse_group::{
    apply_collapse_groups, build_collapse_groups, collapse_group_to_json,
    group_count as collapse_group_count, max_group_size, total_collapsed, CollapseGroup,
    CollapseGroupResult,
};

pub mod mesh_cotan_laplace;
pub use mesh_cotan_laplace::{
    build_cotan_laplacian, cotan_smooth_step, cotan_weight, entry_count as cotan_entry_count,
    total_weight as cotan_total_weight, CotanEntry, CotanLaplacian,
};

pub mod mesh_curve_sample;
pub use mesh_curve_sample::{
    chord_length, circle_curve, extract_positions as curve_extract_positions, extract_tangents,
    line_curve, resample_at, sample_count as curve_sample_count, sample_uniform, CurveEvalFn,
    CurveSample,
};

pub mod mesh_cylinder_cap;
pub use mesh_cylinder_cap::{
    cap_to_json, cap_triangle_count, cap_vertex_count, dome_cap, flat_cap, is_flat_cap, CapType,
    CylinderCap,
};

pub mod mesh_deform_axis;
pub use mesh_deform_axis::{
    default_axis_deform_config, deform_vertex_count, stretch_deform, taper_deform, twist_deform,
    AxisDeformConfig, DeformAxis,
};

pub mod mesh_dual_contour;
pub use mesh_dual_contour::{
    dc_vertex_count, dual_contour, qef_add_plane, qef_error, qef_plane_count, qef_solve,
    DualContourResult, QefAccumulator,
};

pub mod mesh_edge_collapse_v2;
pub use mesh_edge_collapse_v2::{
    collapse_edge_v2, collect_edges as ecv2_collect_edges, ec_v2_face_count, ec_v2_vertex_count,
    edge_length_v2 as ecv2_edge_length, edge_midpoint_v2 as ecv2_edge_midpoint,
    find_cheapest_edge_v2, CollapseOpV2, EdgeCollapseV2Result, EdgeV2,
};

pub mod mesh_face_centroid_v2;
pub use mesh_face_centroid_v2::{
    area_weighted_centroid_v2, compute_face_centroids_v2, face_count_v2, max_area_face_v2,
    nearest_centroid_v2, total_area_v2, FaceCentroidSetV2, FaceCentroidV2,
};

pub mod mesh_face_label;
pub use mesh_face_label::{
    distinct_label_count, face_labels_to_json, faces_with_label, flood_fill_face_label,
    get_face_label, largest_label_group, new_face_labels, set_face_label, FaceLabelSet,
};

pub mod mesh_face_material;
pub use mesh_face_material::{
    add_slot as face_mat_add_slot, assign_face_material, distinct_material_count,
    face_material_to_json, faces_for_material, find_slot_by_id, get_face_material,
    new_face_material_set, new_material_slot, slot_count as face_mat_slot_count, FaceMaterialSet,
    MaterialSlot,
};

pub mod mesh_face_strip;
pub use mesh_face_strip::{
    build_triangle_strip, longest_strip, strip_efficiency, strip_length, strip_restart_count,
    strip_to_indices, strip_to_json, strips_from_mesh, FaceStrip,
};

pub mod mesh_lofted_surface;
pub use mesh_lofted_surface::{
    circle_profile_at, default_loft_config, interpolate_profiles, loft_bounds, loft_centroid,
    loft_face_count, loft_surface, loft_to_json, loft_vertex_count, LoftConfig, LoftedSurface,
};

pub mod mesh_manifold_check;
pub use mesh_manifold_check::{
    boundary_edge_count_mc, check_manifold, count_boundary_loops, is_closed_manifold,
    manifold_report_to_json, non_manifold_edge_count as manifold_non_manifold_edge_count,
    ManifoldReport,
};

pub mod mesh_mean_value;
pub use mesh_mean_value::{
    dominant_vertex as mean_value_dominant_vertex, interpolate_scalar as mean_value_interpolate,
    max_weight as mean_value_max_weight, mean_value_coords_2d, mean_value_to_json,
    regular_polygon as mean_value_regular_polygon, weights_count as mean_value_weights_count,
    weights_sum as mean_value_weights_sum, MeanValueWeights,
};

pub mod mesh_mirror_stitch;
pub use mesh_mirror_stitch::{
    mirror_stitch, stitch_bounds, stitch_result_face_count, stitch_result_to_json,
    stitch_result_vertex_count, validate_stitch_result, MirrorAxis as StitchMirrorAxis,
    MirrorStitchResult,
};

pub mod mesh_mls_deform;
pub use mesh_mls_deform::{
    default_mls_config, handles_valid, mls_avg_displacement, mls_deform,
    mls_displacement_magnitude, mls_result_to_json, MlsConfig, MlsHandle,
};

pub mod mesh_morse_theory;
pub use mesh_morse_theory::{
    compute_morse_critical_points, critical_point_count, field_range, find_global_maximum,
    find_global_minimum, morse_euler_characteristic, morse_result_to_json, CriticalPoint,
    CriticalPointType, MorseResult,
};

pub mod mesh_multi_res;
pub use mesh_multi_res::{
    build_multi_res, coarsest_level, default_multi_res_config, finest_level, get_level,
    level_count, level_face_counts, midpoint_upsample, multi_res_to_json, MeshLevel,
    MultiResConfig, MultiResMesh,
};

pub mod mesh_needle_triangle;
pub use mesh_needle_triangle::{
    avg_aspect_ratio as needle_avg_aspect_ratio, detect_needle_triangles, is_needle_free,
    max_aspect_ratio as needle_max_aspect_ratio, needle_count, needle_ratio, needle_result_to_json,
    triangle_aspect_ratio_needle, NeedleDetectResult,
};

pub mod mesh_normal_transfer_v2;
pub use mesh_normal_transfer_v2::{
    default_normal_transfer_v2_config, normals_are_unit as normals_are_unit_v2,
    transfer_normals_v2, transfer_v2_result_to_json, transfer_v2_success_rate,
    NormalTransferV2Config, NormalTransferV2Result,
};

pub mod mesh_oct_encode;
pub use mesh_oct_encode::{
    oct_compression_ratio_f32, oct_decode, oct_decode_batch, oct_decode_u8, oct_encode,
    oct_encode_batch, oct_encode_decode_error, oct_encode_to_json, oct_encode_u8,
};

pub mod mesh_orthographic_proj;
pub use mesh_orthographic_proj::{
    normalize_projected, project_orthographic, project_to_image_space, projected_area,
    projected_centroid, projected_to_json, OrthoProjectionAxis, OrthoProjectionResult,
};

pub mod mesh_param_chart;
pub use mesh_param_chart::{
    build_param_charts, chart_count, chart_set_to_json, largest_chart as largest_param_chart,
    total_chart_faces, uv_utilization as param_chart_uv_utilization, ParamChart, ParamChartSet,
};

pub mod mesh_patch_blend;
pub use mesh_patch_blend::{
    all_weights_valid as patch_all_weights_valid, blend_patches, blend_patches_uniform,
    clamp_blend_weights, patch_blend_avg_weight, patch_blend_to_json, patch_blend_vertex_count,
    patch_distance, smooth_weight, PatchBlendResult,
};

pub mod mesh_planar_proj;
pub use mesh_planar_proj::{
    default_planar_proj_config, normalize_planar_uvs, planar_proj_tilted, planar_proj_to_json,
    planar_proj_vertex_count, planar_uv_bounds, project_planar, PlanarProjConfig, PlanarProjResult,
};

pub mod mesh_point_sample;
pub use mesh_point_sample::{
    default_point_sample_config, golden_angle_sphere_samples, poisson_disk_thin,
    sample_bary_sum_one, sample_centroid as point_sample_centroid,
    sample_count as point_sample_count, sample_mesh_surface, samples_all_normals_unit,
    samples_to_json, PointSampleConfig, SamplePoint,
};

pub mod mesh_polar_mesh;
pub use mesh_polar_mesh::{
    generate_polar_mesh, polar_bounding_radius, polar_face_count, polar_index_count,
    polar_is_valid, polar_scale, polar_to_json, polar_vertex_count, PolarMesh,
};

pub mod mesh_profile_extrude;
pub use mesh_profile_extrude::{
    extrude_centroid, extrude_indices_valid, extrude_profile, extrude_side_face_count,
    extrude_to_json, extrude_vertex_count as profile_extrude_vertex_count, square_profile,
    ExtrudeProfileResult, Profile2D,
};

pub mod mesh_quad_dominant;
pub use mesh_quad_dominant::{
    build_qd_grid, qd_indices_valid, qd_to_json, quad_count as qd_quad_count, quad_ratio,
    tri_count, triangulate_quads_qd, QdFace, QuadDominantMesh,
};

pub mod mesh_quad_mesh;
pub use mesh_quad_mesh::{
    build_quad_grid, quad_mesh_centroid, quad_mesh_face_count, quad_mesh_flip_winding,
    quad_mesh_indices_valid, quad_mesh_scale, quad_mesh_to_json, quad_mesh_to_tris,
    quad_mesh_vertex_count, QuadMesh,
};

pub mod mesh_quad_strip;
pub use mesh_quad_strip::{
    new_quad_strip, quad_count as qs_quad_count, quad_strip_area, quad_strip_from_path,
    quad_strip_normals, quad_strip_to_triangles, quad_strip_uvs, quad_strip_vertex_count,
    QuadStrip,
};

pub mod mesh_ray_cast;
pub use mesh_ray_cast::{
    ray_at, ray_cast_mesh, ray_cast_normalize, ray_hit_count, ray_triangle_intersect_rc, RayCast,
    RayCastHit,
};

pub mod mesh_relaxation;
pub use mesh_relaxation::{
    build_adjacency_relax, relax_avg_displacement, relax_mesh, relax_step, relax_to_json,
    RelaxConfig, RelaxResult,
};

pub mod mesh_remesh_adaptive;
pub use mesh_remesh_adaptive::{
    adaptive_remesh, adaptive_remesh_to_json,
    adaptive_target_length as adaptive_remesh_target_length, avg_edge_length_ar, collect_edges_ar,
    count_long_edges_ar, count_short_edges_ar, edge_length_ar, AdaptiveRemeshConfig,
    AdaptiveRemeshResult,
};

pub mod mesh_reverse_normal;
pub use mesh_reverse_normal::{
    compute_face_normals_rn, count_upward_normals, face_normal_rn, flip_all_winding_rn,
    normals_are_unit_rn, reverse_normal_to_json, reverse_normals, reverse_normals_selected,
};

pub mod mesh_root_find;
pub use mesh_root_find::{
    bisect, bracket_check, newton_raphson, regula_falsi, residual_at, root_find_to_json,
    RootFindResult,
};

pub mod mesh_scalar_field;
pub use mesh_scalar_field::{
    build_scalar_field, count_above as scalar_count_above, count_below as scalar_count_below,
    field_avg, field_max as scalar_field_max, field_min as scalar_field_min, normalize_field,
    scalar_field_to_json, sdf_sphere_field, sine_field, ScalarFieldV,
};

pub mod mesh_seam_mark;
pub use mesh_seam_mark::{
    clear_seams, is_seam, mark_boundary_seams, mark_seam, seam_count, seam_edges_sorted,
    seam_mark_to_json, unmark_seam, SeamMarkSet,
};

pub mod mesh_sharp_edge;
pub use mesh_sharp_edge::{
    detect_sharp_edges, has_sharp_edges, max_dihedral_angle as sharp_max_dihedral_angle,
    sharp_edge_count, sharp_edge_to_json, sharp_edge_vertices, SharpEdge, SharpEdgeResult,
};

pub mod mesh_shell_deform;
pub use mesh_shell_deform::{
    avg_shell_displacement, shell_deform, shell_deform_to_json, shell_deform_valid,
    vertex_normals_shell, ShellDeformParams, ShellDeformResult,
};

pub mod mesh_silhouette;
pub use mesh_silhouette::{
    extract_silhouette as extract_silhouette_mesh, has_silhouette, silhouette_edge_count,
    silhouette_to_json, silhouette_vertices, view_dir_from_camera, SilhouetteEdge,
    SilhouetteResult,
};

pub mod mesh_skeleton_deform;
pub use mesh_skeleton_deform::{
    deform_avg_displacement, normalize_skeleton_weights, skeleton_deform, skeleton_deform_to_json,
    skinned_vertex_count_sd, BoneSd, BoneTransformSd, SkeletonWeight,
};

pub mod mesh_smooth_weight;
pub use mesh_smooth_weight::{
    average_weight, build_adjacency_sw, clamp_weights as clamp_mesh_weights,
    count_above_threshold as weight_count_above_threshold, detect_boundary_sw,
    normalize_smooth_weights, smooth_weights as smooth_vertex_weights, SmoothWeightConfig,
    SmoothWeightResult,
};

pub mod mesh_sort_vertex;
pub use mesh_sort_vertex::{
    centroid as sort_vertex_centroid, is_valid_permutation, nearest_to_origin, remap_indices,
    sort_vertices, sorted_axis_range, SortAxis, SortVertexResult,
};

pub mod mesh_span_tree;
pub use mesh_span_tree::{
    component_count as span_component_count, edges_from_mesh as span_edges_from_mesh,
    max_span_edge, min_span_edge, minimum_spanning_tree, prune_long_edges, SpanEdge,
    SpanTreeResult,
};

pub mod mesh_sphere_proj;
pub use mesh_sphere_proj::{
    all_on_sphere, cart_to_spherical, mean_radius, project_mesh_to_sphere, project_to_sphere,
    sphere_uvs, spherical_to_cart, spherical_uv,
};

pub mod mesh_spring_mesh;
pub use mesh_spring_mesh::{
    max_velocity as spring_max_velocity, simulate_spring_mesh, spring_energy, spring_step,
    springs_from_mesh, Spring, SpringMeshParams,
};

pub mod mesh_stitch_seam;
pub use mesh_stitch_seam::{
    find_boundary_loop, is_closed as stitch_is_closed, loop_avg_edge_length,
    loop_centroid as stitch_loop_centroid, stitch_loops, stitched_face_count, SeamEdgePair,
    StitchSeamResult,
};

pub mod mesh_stroke_path;
pub use mesh_stroke_path::{
    build_stroke_path, scale_stroke_widths, straight_stroke, stroke_arc_length, stroke_face_count,
    uvs_in_range as stroke_uvs_in_range, StrokePathResult, StrokeSample,
};

pub mod mesh_subdiv_loop;
pub use mesh_subdiv_loop::{
    avg_edge_length_ls, bounding_box_ls, expected_face_count, expected_vertex_count_approx,
    indices_valid as loop_indices_valid, loop_subdiv_step, loop_subdivide_mesh, LoopSubdivResult,
};

pub mod mesh_surface_flow;
pub use mesh_surface_flow::{
    advect_point_on_surface, flow_at_vertex, flow_magnitude_at, new_flow_field, set_flow_vector,
    smooth_flow_field, FlowField, FlowVector,
};

pub mod mesh_surface_normal;
pub use mesh_surface_normal::{
    average_normal as surface_average_normal, compute_flat_normals, compute_smooth_normals,
    count_back_facing, degenerate_normal_count, flip_normals, normal_angle, normals_are_unit,
};

pub mod mesh_sweep_solid;
pub use mesh_sweep_solid::{
    regular_polygon_profile, sweep_bounds, sweep_indices_valid, sweep_solid, sweep_triangle_count,
    PathPointSolid, Profile2DSolid, SweepSolidResult,
};

pub mod mesh_tangent_frame;
pub use mesh_tangent_frame::{
    average_tangent, compute_tangent_frames, count_valid_frames, frame_to_matrix, is_orthonormal,
    tangent_angle, tangent_to_world as tangent_to_world_frame, TangentFrame,
};

pub mod mesh_texture_proj;
pub use mesh_texture_proj::{
    box_project, cylindrical_project, planar_project, project_all,
    spherical_project as texture_spherical_project, tile_uvs_proj,
    uvs_in_unit_range as texture_uvs_in_unit_range, TextureProjMode,
};

pub mod mesh_thin_shell;
pub use mesh_thin_shell::{
    compute_vertex_normals as thin_shell_vertex_normals, half_turn_radians, offset_by_normal,
    shell_face_count, shell_volume, thin_shell, ThinShellConfig, ThinShellResult,
};

pub mod mesh_topo_sort;
pub use mesh_topo_sort::{
    assign_levels, build_dag, compute_in_degree, order_is_complete, reverse_topo,
    sort_faces_by_edge_order, topo_sort, TopoSortResult,
};

pub mod mesh_transfer_attr;
pub use mesh_transfer_attr::{
    attrs_same_name, barycentric, nearest_vertex as transfer_nearest_vertex, scale_scalar_attr,
    transfer_scalar_nn, transfer_vec3_nn, ScalarAttr, TransferMethod, TransferResult, Vec3Attr,
};

pub mod mesh_tri_strip;
pub use mesh_tri_strip::{
    decode_tri_strip, encode_tri_strip, new_tri_strip, strip_efficiency_ratio, strip_half_angle,
    strip_index_count, strip_restart_count as tri_strip_restart_count,
    strip_to_json as tri_strip_to_json, strip_triangle_count, TriStrip,
};

pub mod mesh_tube_mesh;
pub use mesh_tube_mesh::{
    generate_tube, tube_circumference, tube_face_count, tube_indices_valid, tube_normals_unit,
    tube_surface_area, tube_to_json, tube_vertex_count, TubeMesh,
};

pub mod mesh_uv_atlas;
pub use mesh_uv_atlas::{
    add_island as uv_atlas_add_island, atlas_to_json, atlas_utilization as uv_atlas_utilization,
    find_island_by_id, island_count, islands_overlap, largest_island_area, new_uv_atlas,
    total_island_faces, uv_in_atlas_bounds, UvAtlas, UvAtlasIsland,
};

pub mod mesh_uv_seam;
pub use mesh_uv_seam::{
    add_seam_edge, clear_seams as uv_seam_clear, detect_boundary_seams, is_seam_edge,
    new_uv_seam_set, remove_seam_edge, seam_edge_count, seam_set_to_json, seam_vertices,
    UvSeamEdge, UvSeamSet,
};

pub mod mesh_vertex_attr;
pub use mesh_vertex_attr::{
    add_attr_layer, attr_average, attr_set_to_json, find_layer_by_name, get_attr, layer_count,
    new_vertex_attr_set, set_attr as set_vertex_attr_val, VertexAttrLayer, VertexAttrSet,
};

pub mod mesh_vertex_color;
pub use mesh_vertex_color::{
    blend_vertex_colors, clear_vertex_colors, color_to_rgba,
    get_vertex_color as get_vertex_color_map, new_vertex_color_map,
    set_vertex_color as set_vertex_color_map, vertex_color_count as vertex_color_map_count,
    vertex_colors_to_bytes, VertexColor, VertexColorMap,
};

pub mod mesh_vertex_deform;
pub use mesh_vertex_deform::{
    apply_deform, clear_deform, deform_magnitude, deform_to_json, max_deform_magnitude,
    new_vertex_deform, nonzero_delta_count, scale_deform, set_delta as set_deform_delta,
    sine_deform_y, VertexDeform,
};

pub mod mesh_vertex_group;
pub use mesh_vertex_group::{
    add_group as vg_add_group, add_to_group, find_group_by_name as find_vertex_group,
    group_average_weight, group_count, group_set_to_json, group_vertex_count, new_vertex_group_set,
    remove_from_group, weight_in_group, VertexGroupEntry, VertexGroupSetV2, VertexGroupV2,
};

pub mod mesh_vertex_merge;
pub use mesh_vertex_merge::{
    merge_at_indices, merge_count, merge_threshold, merge_vertices_by_distance,
    MergeResult as VertexMergeResult,
};

pub mod mesh_vertex_order_v2;
pub use mesh_vertex_order_v2::{
    apply_vertex_order, invert_permutation, is_valid_permutation as is_valid_vertex_perm,
    order_by_axis, order_cache_linear, order_result_to_json, remap_indices as remap_vertex_indices,
    OrderStrategy, VertexOrderResult,
};

pub mod mesh_vertex_project;
pub use mesh_vertex_project::{
    project_to_cylinder, project_to_plane, project_to_sphere as project_vertices_to_sphere,
    project_to_surface, VertexProject,
};

pub mod mesh_vertex_smooth_v2;
pub use mesh_vertex_smooth_v2::{
    build_adjacency_v2, default_smooth_v2_config, laplacian_step_v2, smooth_displacement,
    smooth_v2_to_json, taubin_smooth_v2, SmoothV2Config,
};

pub mod mesh_vertex_snap;
pub use mesh_vertex_snap::{
    snap_count, snap_threshold_vs, snap_to_grid, snap_vertex_to_nearest, VertexSnap,
};

pub mod mesh_vertex_weld;
pub use mesh_vertex_weld::{
    unweld_vertex, weld_by_normal, weld_by_position as weld_by_position_vw, weld_count, weld_map,
    weld_statistics, weld_threshold_value, weld_vertices, WeldResult2,
};

pub mod mesh_visibility;
pub use mesh_visibility::{
    classify_visibility, dot3_vis, face_normal_vis, front_facing_ratio, grazing_angle_deg,
    visibility_to_json, FaceVisibility, VisibilityResult,
};

pub mod mesh_voxel_oct;
pub use mesh_voxel_oct::{
    build_vox_octree, compute_aabb_vo, node_count_vo, occupied_leaf_count, point_in_aabb,
    split_aabb, vox_octree_to_json, VoxAabb, VoxOctNode, VoxOctree,
};

pub mod mesh_warp_deform;
pub use mesh_warp_deform::{
    active_handle_count, apply_warp_deform, avg_warp_displacement, dist3_wd, rbf_weight,
    warp_config_to_json, WarpDeformConfig, WarpHandle2,
};

pub mod mesh_wave_mesh;
pub use mesh_wave_mesh::{
    generate_wave_mesh, wave_height, wave_params_to_json, wave_triangle_count, wave_vertex_count,
    wave_y_range, WaveMeshParams, WaveMeshResult,
};
