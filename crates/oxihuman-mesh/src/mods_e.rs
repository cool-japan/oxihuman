
pub mod mesh_align_vertices;
pub use mesh_align_vertices::*;

pub mod mesh_distribute_even;
pub use mesh_distribute_even::*;

pub mod mesh_loop_to_region;
pub use mesh_loop_to_region::*;

pub mod mesh_checker_deselect;
pub use mesh_checker_deselect::*;

pub mod mesh_anchor_point;
pub use mesh_anchor_point::*;

pub mod mesh_magnet_point;
pub use mesh_magnet_point::*;

pub mod mesh_follow_path;
pub use mesh_follow_path::*;

pub mod mesh_track_to;
pub use mesh_track_to::*;

pub mod mesh_copy_location;
pub use mesh_copy_location::*;

pub mod mesh_copy_rotation;
pub use mesh_copy_rotation::*;

pub mod mesh_copy_scale;
pub use mesh_copy_scale::*;

pub mod mesh_transform_constraint;
pub use mesh_transform_constraint::*;

pub mod mesh_floor_constraint;
pub use mesh_floor_constraint::*;

pub mod mesh_collision_bounds;
pub use mesh_collision_bounds::*;

pub mod mesh_rigid_body_shape;
pub use mesh_rigid_body_shape::*;

pub mod mesh_soft_body_goal;
pub use mesh_soft_body_goal::*;

pub mod mesh_cloth_collision;
pub use mesh_cloth_collision::*;

pub mod mesh_force_field_mesh;
pub use mesh_force_field_mesh::*;

pub mod mesh_meta_ball;
pub use mesh_meta_ball::*;

pub mod mesh_implicit_blob;
pub use mesh_implicit_blob::*;

pub mod mesh_boolean_union;
pub use mesh_boolean_union::*;

pub mod mesh_boolean_intersection;
pub use mesh_boolean_intersection::*;

pub mod mesh_boolean_difference;
pub use mesh_boolean_difference::*;

pub mod mesh_slice_plane;
pub use mesh_slice_plane::{
    classify_vertices as slice_classify_vertices, count_triangles_per_side,
    edge_plane_intersect as slice_edge_plane_intersect, slice_mesh_with_plane,
    SlicePlane as BoolSlicePlane, SliceResult as BoolSliceResult,
};

pub mod mesh_project_curve;
pub use mesh_project_curve::*;

pub mod mesh_intersect_ray;
pub use mesh_intersect_ray::{
    count_ray_intersections, intersect_ray_mesh, intersect_ray_mesh_all,
    ray_triangle_intersect as mti_ray_triangle_intersect, Ray as IntersectRay, RayHitResult,
};

pub mod mesh_closest_point_query;
pub use mesh_closest_point_query::{
    closest_point_on_triangle as query_closest_point_on_triangle, is_near_surface,
    query_closest_point, query_closest_within_radius, ClosestPointResult as MeshClosestPointResult,
};

pub mod mesh_signed_distance;
pub use mesh_signed_distance::*;

pub mod mesh_winding_query;
pub use mesh_winding_query::*;

pub mod mesh_volume_query;
pub use mesh_volume_query::*;

pub mod mesh_surface_area_query;
pub use mesh_surface_area_query::*;

pub mod mesh_bounding_box_query;
pub use mesh_bounding_box_query::*;

pub mod mesh_centroid_query;
pub use mesh_centroid_query::{
    center_mesh_at_centroid, centroid_deviation, surface_centroid,
    vertex_centroid as mesh_vertex_centroid, volume_centroid,
};

pub mod mesh_moment_of_inertia;
pub use mesh_moment_of_inertia::{
    add_inertia_tensors, mesh_inertia_tensor, parallel_axis_shift, scale_inertia_tensor,
    InertiaTensor as MeshInertiaTensor,
};

pub mod mesh_principal_axes;
pub use mesh_principal_axes::*;

pub mod mesh_topology_query;
pub use mesh_topology_query::*;

pub mod mesh_lod_chain;
pub use mesh_lod_chain::*;

pub mod mesh_progressive_mesh;
pub use mesh_progressive_mesh::*;

pub mod mesh_view_dependent_lod;
pub use mesh_view_dependent_lod::*;

pub mod mesh_geomorph;
pub use mesh_geomorph::*;

pub mod mesh_impostor;
pub use mesh_impostor::*;

pub mod mesh_sprite_atlas;
pub use mesh_sprite_atlas::*;

pub mod mesh_decal_mesh;
pub use mesh_decal_mesh::*;

pub mod mesh_outline_mesh;
pub use mesh_outline_mesh::*;

pub mod mesh_shadow_mesh;
pub use mesh_shadow_mesh::*;

pub mod mesh_light_map_mesh;
pub use mesh_light_map_mesh::*;

pub mod mesh_ambient_occlusion_mesh;
pub use mesh_ambient_occlusion_mesh::*;

pub mod mesh_vertex_animation;
pub use mesh_vertex_animation::*;

pub mod mesh_morph_animation;
pub use mesh_morph_animation::*;

pub mod mesh_skinned_animation;
pub use mesh_skinned_animation::*;

pub mod mesh_blend_animation;
pub use mesh_blend_animation::*;

pub mod mesh_pose_snapshot;
pub use mesh_pose_snapshot::*;

pub mod mesh_lattice_deform;
pub use mesh_lattice_deform::{
    ffd_apply_to_point, ffd_lattice_point_count, ffd_lattice_size, ffd_reset,
    ffd_set_control_point, new_ffd_lattice, FfdLattice as FfdLatticeNew,
};

pub mod mesh_solidify;
pub use mesh_solidify::{
    new_solidify_params, solidify_face_count, solidify_flip_normal, solidify_vert_count,
    solidify_vertex as solidify_vertex_fn, SolidifyParams,
};

pub mod mesh_bridge_edge_loops;
pub use mesh_bridge_edge_loops::{
    bridge_face_count_bl, bridge_loops_quads, bridge_vertex_count_bl, loop_centroid_bl,
    loop_perimeter_bl, new_bridge_edge_loop, BridgeEdgeLoop,
};

pub mod mesh_screw_modifier;
pub use mesh_screw_modifier::{
    new_screw_params, screw_output_vertex_count, screw_total_angle_deg, screw_total_height,
    screw_transform_vertex, ScrewParams as ScrewModParams,
};

pub mod mesh_array_modifier;
pub use mesh_array_modifier::{
    array_face_count, array_instance_transform, array_total_size, array_vertex_count,
    new_array_params, ArrayParams,
};

pub mod mesh_bevel_modifier;
pub use mesh_bevel_modifier::{
    bevel_edge_offset, bevel_new_face_count, bevel_new_vertex_count, bevel_segment_point,
    new_bevel_params, BevelParams,
};

pub mod mesh_wireframe_modifier;
pub use mesh_wireframe_modifier::{
    new_wireframe_params, wireframe_edge_count as wireframe_mod_edge_count,
    wireframe_edge_tube_verts, wireframe_tube_length,
    wireframe_vertex_count as wireframe_mod_vertex_count, WireframeParams,
};

pub mod mesh_skin_modifier;
pub use mesh_skin_modifier::{
    new_skin_vertex, skin_cap_center, skin_segment_length, skin_segment_verts, skin_total_volume,
    SkinVertexNew,
};

pub mod mesh_mask_modifier;
pub use mesh_mask_modifier::{
    mask_apply, mask_count_visible, mask_invert, mask_keep_vertex, new_mask_params, MaskParams,
};

pub mod mesh_tube_deform;
pub use mesh_tube_deform::{
    new_tube_path, tube_deform_vertex, tube_path_at, tube_path_length, tube_point_count,
    tube_radius_at, TubePath,
};

pub mod mesh_curve_to_mesh;
pub use mesh_curve_to_mesh::{
    curve_length, curve_segment_ring, curve_to_mesh_face_count, curve_to_mesh_vertex_count,
    new_curve_mesh_params, CurveMeshParams,
};

pub mod mesh_spin;
pub use mesh_spin::{
    new_spin_params, spin_face_count, spin_is_closed, spin_vertex, spin_vertex_count, SpinParams,
};

pub mod mesh_decimate_modifier;
pub use mesh_decimate_modifier::{
    decimate_collapse_count, decimate_priority, decimate_ratio_clamp, decimate_target_face_count,
    new_decimate_params, DecimateParams,
};

pub mod mesh_mirror_modifier;
pub use mesh_mirror_modifier::{
    mirror_copy_count, mirror_flip_normal as mirror_mod_flip_normal, mirror_should_merge,
    mirror_vertex as mirror_mod_vertex, new_mirror_params, MirrorParams as MirrorModParams,
};

pub mod mesh_stroke_extrude;
pub use mesh_stroke_extrude::{
    new_stroke_point, stroke_area, stroke_extrude_quad,
    stroke_face_count as stroke_extrude_face_count, stroke_total_length,
    stroke_vertex_count as stroke_extrude_vertex_count, StrokePoint,
};

pub mod mesh_thicken_normals;
pub use mesh_thicken_normals::{
    thicken_average_normal, thicken_boundary_vertex, thicken_is_normalized, thicken_mesh_vertices,
    thicken_vertex,
};

pub mod mesh_bend_modifier;
pub use mesh_bend_modifier::{
    bend_angle_at, bend_curvature, bend_is_unlimited, bend_vertex, new_bend_params, BendParams,
};

pub mod mesh_taper_modifier;
pub use mesh_taper_modifier::{
    new_taper_params, taper_is_uniform, taper_scale_at, taper_vertex, taper_volume_ratio,
    TaperParams,
};

pub mod mesh_twist_modifier;
pub use mesh_twist_modifier::{
    new_twist_params, twist_angle_at, twist_is_zero, twist_vertex, TwistParams,
};

pub mod mesh_shear_modifier;
pub use mesh_shear_modifier::{
    new_shear_params, shear_area_ratio, shear_is_identity, shear_matrix_2x2, shear_vertex,
    ShearParams,
};

pub mod mesh_cast_modifier;
pub use mesh_cast_modifier::{
    cast_blend, cast_target_cylinder, cast_target_sphere, cast_vertex, new_cast_sphere, CastParams,
};

pub mod mesh_smooth_modifier;
pub use mesh_smooth_modifier::{
    smooth_cotangent_weight, smooth_mesh_pass, smooth_passes, smooth_vertex_laplacian,
};

pub mod mesh_displace_modifier;
pub use mesh_displace_modifier::{
    displace_along_axis, displace_along_normal, displace_midlevel_offset,
    displace_vertex_new as displace_vertex_params, new_displace_params, DisplaceParams,
};

pub mod mesh_warp_modifier;
pub use mesh_warp_modifier::{
    new_warp_params, warp_falloff_linear, warp_falloff_smooth, warp_influence,
    warp_vertex_new as warp_vertex_params, WarpParams,
};

pub mod mesh_transfer_weights;
pub use mesh_transfer_weights::{
    new_weight_map as new_mesh_weight_map, weight_blend, weight_get, weight_normalize, weight_set,
    weight_transfer_nearest, WeightMap as MeshWeightMap,
};

pub mod mesh_proximity_deform;
pub use mesh_proximity_deform::{
    new_proximity_deformer, proximity_count_influenced, proximity_deform_vertex,
    proximity_influence, proximity_nearest_distance, ProximityDeformer,
};

pub mod mesh_seam_flatten;
pub use mesh_seam_flatten::{
    new_uv_seam as new_mesh_uv_seam, seam_add_pair,
    seam_boundary_length as seam_flatten_boundary_length, seam_contains_vertex, seam_flatten_uv,
    seam_pair_count, UvSeam as MeshUvSeam,
};

pub mod mesh_hole_detect;
pub use mesh_hole_detect::{
    boundary_add_loop, boundary_hole_count, boundary_total_vertices, detect_boundary_edges,
    is_closed_mesh as mesh_hole_is_closed, new_mesh_boundary, MeshBoundary,
};

pub mod mesh_data_transfer;
pub use mesh_data_transfer::{
    new_vertex_data, transfer_nearest_normal, transfer_nearest_uv, vertex_data_count,
    vertex_nearest_index, VertexData,
};

pub mod mesh_fiber_orientation;
pub use mesh_fiber_orientation::{
    fiber_anisotropy_index, fiber_count, fiber_get, fiber_mean_coherence, fiber_set,
    new_fiber_field, FiberField,
};

pub mod mesh_ambient_occlusion;
pub use mesh_ambient_occlusion::bake_ambient_occlusion as bake_ao_full;
pub use mesh_ambient_occlusion::{
    ao_estimate,
    ao_params_is_valid,
    ao_sample_hemisphere,
    ao_to_color,
    new_ao_params,
    // Full AO baking API
    AoBvh,
    AoConfig,
    AoParams,
    Lcg as AoLcg,
    MeshBuffers as AoMeshBuffers,
};

pub mod mesh_thickness_map;
pub use mesh_thickness_map::{
    new_thickness_map, thickness_get, thickness_max, thickness_mean, thickness_set,
    thickness_to_color, ThicknessMap as MeshThicknessMap,
};

pub mod mesh_cavity_map;
pub use mesh_cavity_map::{
    cavity_get, cavity_is_cavity, cavity_is_convex, cavity_mean, cavity_set, cavity_to_color,
    new_cavity_map, CavityMap,
};

pub mod mesh_curvature_map;
pub use mesh_curvature_map::{
    curv_gaussian_to_color, curv_get_gaussian, curv_get_mean, curv_mean_to_color,
    curv_set_gaussian, curv_set_mean, new_curvature_map, CurvatureMap as MeshCurvatureMap,
};

pub mod mesh_bent_normal;
pub use mesh_bent_normal::{
    bent_normal_ao, bent_normal_count, bent_normal_get, bent_normal_set, bent_normal_to_color,
    new_bent_normal_map, BentNormalMap,
};

pub mod mesh_edge_flow_field;
pub use mesh_edge_flow_field::{
    edge_flow_field_curl_magnitude, edge_flow_field_divergence, edge_flow_field_get,
    edge_flow_field_mean_speed, edge_flow_field_set, new_edge_flow_field, EdgeFlowField,
};

pub mod mesh_retopo_guide;
pub use mesh_retopo_guide::{
    new_retopo_stroke, stroke_direction, stroke_length, stroke_point_count, stroke_resample,
    stroke_snap_to_surface, RetopoStroke,
};

pub mod mesh_symmetry_map;
pub use mesh_symmetry_map::{
    new_symmetry_map, sym_add_pair, sym_find_mirror, sym_is_symmetric_position,
    sym_mirror_position, sym_pair_count, SymmetryMap,
};

pub mod mesh_pose_space;
pub use mesh_pose_space::{
    new_pose_key, pose_apply, pose_best_key, pose_key_count, pose_weight, PoseKey,
};

pub mod mesh_corrective_shape;
pub use mesh_corrective_shape::{
    corrective_apply, corrective_is_active, corrective_peak_weight, corrective_weight,
    new_corrective_shape, CorrectiveShape,
};

pub mod mesh_wrinkle_map;
pub use mesh_wrinkle_map::{
    new_wrinkle_driver, wrinkle_factor, wrinkle_mean_weight, wrinkle_peak_count,
    wrinkle_weights_at, WrinkleDriver,
};

pub mod mesh_jiggle;
pub use mesh_jiggle::{
    jiggle_impulse, jiggle_is_settled, jiggle_offset, jiggle_set_target, jiggle_step,
    new_jiggle_vertex, JiggleVertex,
};

pub mod mesh_flow_map;
pub use mesh_flow_map::{
    flow_map_get, flow_map_mean_speed, flow_map_normalize_dir, flow_map_set, flow_map_to_color,
    new_flow_map, FlowMap,
};

pub mod mesh_stress_lines;
pub use mesh_stress_lines::{
    new_stress_line, stress_line_color, stress_line_is_critical, stress_line_length,
    stress_line_point_count, stress_line_push, StressLine,
};

pub mod mesh_curvature_lines;
pub use mesh_curvature_lines::{
    curvline_anisotropy, curvline_color, curvline_length, curvline_point_count, curvline_push,
    new_curvature_line, CurvatureLine,
};

pub mod mesh_poke_faces;
pub use mesh_poke_faces::{
    poke_center, poke_face, poke_face_area, poke_triangle_count as poke_face_triangle_count,
    poke_vertex_count as poke_face_vertex_count, PokeResult,
};

pub mod mesh_inset_faces;
pub use mesh_inset_faces::{
    default_inset_config, inset_centroid, inset_creates_valid_face, inset_face, inset_face_center,
    inset_face_thickness, inset_faces as inset_faces_config, inset_individual_faces,
    inset_result_face_count, inset_validate_config, InsetConfig, InsetResult,
};

pub mod mesh_bisect;
pub use mesh_bisect::{
    bisect_classify_vertex, bisect_count_above, bisect_count_below, bisect_edge_intersection,
    bisect_lerp as bisect_lerp3, new_bisect_plane, BisectPlane,
};

pub mod mesh_intersect_line;
pub use mesh_intersect_line::{
    line_segment_triangle_intersect, ray_mesh_intersect_count, ray_point_at,
    ray_triangle_intersect as ray_tri_intersect,
};

pub mod mesh_select_by_angle;
pub use mesh_select_by_angle::{
    angle_between_normals_deg, face_normal_3, select_faces_by_angle, select_flat_faces,
};

pub mod mesh_select_linked;
pub use mesh_select_linked::{
    build_face_adjacency as linked_build_adjacency, count_islands, face_shares_edge,
    select_linked_faces, select_linked_vertices,
};

pub mod mesh_path_cut;
pub use mesh_path_cut::{
    new_path_cut, path_cut_edges, path_cut_is_closed, path_cut_length, path_cut_vertex_count,
    path_shortest, PathCut,
};

pub mod mesh_edge_slide;
pub use mesh_edge_slide::{
    edge_direction, edge_length as edge_slide_length, edge_slide_edge, edge_slide_midpoint,
    edge_slide_vert,
};

pub mod mesh_rip_vertices;
pub use mesh_rip_vertices::{
    rip_face_count_new, rip_is_valid_gap, rip_seam_length, rip_vertex, rip_vertex_count_new,
    RipResult,
};

pub mod mesh_merge_by_distance;
pub use mesh_merge_by_distance::{
    merge_apply_to_faces, merge_by_distance, merge_count_unique, merge_remove_degenerate,
    merge_weld_threshold,
};

pub mod mesh_polygon_fill;
pub use mesh_polygon_fill::{
    polygon_area_3d, polygon_is_planar, polygon_perimeter,
    polygon_vertex_count as polygon_fill_vertex_count, triangulate_polygon_3d,
};

pub mod mesh_knife_cut;
pub use mesh_knife_cut::{
    cut_edge_count, cut_vertex_count, knife_add_point, knife_cut,
    knife_cut_length as knife_cut_polyline_length, knife_cut_modified, knife_is_closed,
    knife_point_count, knife_project_to_face, new_knife_cut, KnifeCut, KnifeCutResult, KnifeLine,
};

pub mod mesh_vertex_slide;
pub use mesh_vertex_slide::{
    nearest_neighbor as vertex_nearest_neighbor, slide_to_json, slide_vertex, slide_vertices,
    vertex_distance, vertex_edge_direction, vertex_slide_clamp, vertex_slide_offset,
    vertex_slide_position, vertex_slide_toward_nearest, VertexSlideResult,
};

// --- Wave 151B additions ---
// Extra re-exports for updated mesh_catenary (new API additions)
pub use mesh_catenary::{catenary_to_polyline, new_catenary};

// Extra re-exports for updated mesh_helix (new API additions)
pub use mesh_helix::{helix_length, helix_point, helix_point_count, helix_to_polyline, new_helix};

// Extra re-exports for updated mesh_cylinder_gen (new API additions)
pub use mesh_cylinder_gen::{
    cylinder_face_count as cylinder_tri_face_count, cylinder_is_cone, cylinder_vertex,
    new_cylinder, CylinderParams,
};

// Extra re-exports for updated mesh_capsule_gen (new API additions)
pub use mesh_capsule_gen::{
    capsule_face_count as capsule_tri_face_count, capsule_surface_area, new_capsule, CapsuleParams,
};

pub mod mesh_torus;
pub use mesh_torus::{
    new_torus, torus_face_count as torus_new_face_count,
    torus_surface_area as torus_gen_surface_area, torus_vertex,
    torus_vertex_count as torus_new_vertex_count, torus_volume as torus_gen_volume, TorusParams,
};

pub mod mesh_sphere_gen;
pub use mesh_sphere_gen::{
    icosphere_vertex_count as icosphere_vert_count_gen, new_sphere_params,
    sphere_surface_area as sphere_gen_surface_area, uv_sphere_face_count, uv_sphere_vertex,
    uv_sphere_vertex_count as uv_sphere_gen_vertex_count, SphereParams,
};

pub mod mesh_plane_gen;
pub use mesh_plane_gen::{
    new_plane, plane_area as plane_gen_area, plane_face_count as plane_tri_face_count, plane_uv,
    plane_vertex, plane_vertex_count as plane_gen_vertex_count, PlaneParams,
};

pub mod mesh_icosphere_gen;
pub use mesh_icosphere_gen::{
    icosahedron_faces as icosahedron_faces_gen, icosahedron_vertices as icosahedron_verts_gen,
    icosphere_build, icosphere_subdivide, icosphere_vertex_count as icosphere_vertex_count_gen,
};

pub mod mesh_arrow_gen;
pub use mesh_arrow_gen::{
    arrow_face_count as arrow_gen_face_count, arrow_tip_position, arrow_total_length,
    arrow_vertex_count as arrow_gen_vertex_count, new_arrow, ArrowParams,
};

pub mod mesh_grid_gen;
pub use mesh_grid_gen::{
    grid_bounds as grid_3d_bounds, grid_cell_count, grid_edge_count, grid_vertex,
    grid_vertex_count as grid_3d_vertex_count, new_grid_params_3d, GridParams3d,
};

pub mod mesh_annulus_gen;
pub use mesh_annulus_gen::{
    annulus_area, annulus_face_count as annulus_gen_face_count, annulus_vertex,
    annulus_vertex_count as annulus_gen_vertex_count, new_annulus, AnnulusParams,
};

pub mod mesh_tetrahedron_gen;
pub use mesh_tetrahedron_gen::{
    tetrahedron_circumradius, tetrahedron_edge_length, tetrahedron_faces, tetrahedron_surface_area,
    tetrahedron_vertices, tetrahedron_volume,
};

pub mod mesh_box_gen;
pub use mesh_box_gen::{
    box_diagonal, box_face_count as box_gen_face_count, box_surface_area,
    box_vertex_count as box_gen_vertex_count, box_volume, new_box_mesh, BoxParams,
};

pub mod mesh_fibonacci_sphere;
pub use mesh_fibonacci_sphere::{
    fibonacci_coverage_estimate, fibonacci_min_angle, fibonacci_sphere, fibonacci_sphere_point,
};

pub mod mesh_terrain_gen;
pub use mesh_terrain_gen::{
    new_terrain, terrain_face_count as terrain_gen_face_count, terrain_height_fbm, terrain_vertex,
    terrain_vertex_count as terrain_gen_vertex_count, TerrainParams as TerrainGenParams,
};

pub mod mesh_fractal_gen;
pub use mesh_fractal_gen::{
    fractal_dimension_estimate, koch_point_count, koch_snowflake_points, mandelbrot_escape_time,
    sierpinski_triangle_centroids,
};

pub mod mesh_dual_contouring;
pub use mesh_dual_contouring::{
    dc_triangle_count_dc, dc_vertex_count_dc, dual_contour_dc, qef_dc_add_plane, qef_dc_error,
    qef_dc_solve, DualContourDcResult, QefDc,
};

pub mod mesh_mean_curvature_flow;
pub use mesh_mean_curvature_flow::{
    mcf_step, mcf_vertex_count, mean_curvature_flow, McfConfig, McfResult,
};

pub mod mesh_adaptive_subdivision;
pub use mesh_adaptive_subdivision::{
    adapt_subdiv_face_count, adapt_subdiv_vertex_count, adapt_subdivide_step, adaptive_subdivision,
    AdaptSubdivConfig, AdaptSubdivResult,
};

pub mod mesh_sharp_feature_preserve;
pub use mesh_sharp_feature_preserve::{
    detect_sharp_features as detect_sharp_features_pres, is_sharp_corner,
    sharp_feature_corner_count, sharp_feature_edge_count as sharp_feature_edge_count_pres,
    sharp_feature_max_angle, SharpFeatureCorner, SharpFeatureEdge as SharpFeatureEdgePres,
    SharpFeatureResult as SharpFeatureResultPres,
};

pub mod mesh_qem_simplify;
pub use mesh_qem_simplify::{
    build_vertex_quadrics, edge_collapse_cost_qem, qem_face_count, qem_simplify, qem_vertex_count,
    QemConfig, QemResult, Quadric as QemQuadric,
};

pub mod mesh_loop_detection;
pub use mesh_loop_detection::{
    classify_loop, closed_loop_count, detect_boundary_loops_ld, loop_count_ld,
    total_loop_perimeter, BoundaryLoop, LoopClass, LoopDetectResult,
};

pub mod mesh_patch_sew;
pub use mesh_patch_sew::{
    patch_sew_indices_valid, patch_sew_seam_pairs, patch_sew_triangle_count,
    patch_sew_vertex_count, sew_patches, PatchSewConfig, PatchSewResult,
};

pub mod mesh_genus_compute;
pub use mesh_genus_compute::{
    compute_genus, count_boundary_loops_genus, count_unique_edges, euler_char, genus_value,
    GenusResult,
};

pub mod mesh_sphere_map;
pub use mesh_sphere_map::{
    project_to_sphere_uv, sphere_map, sphere_map_avg_u, sphere_map_uv_count,
    sphere_map_uvs_in_range, SphereMapResult,
};

pub mod mesh_cotangent_weights;
pub use mesh_cotangent_weights::{
    build_cotangent_weights, cot_angle_at, cot_avg_vertex_area, cot_total_weight, cot_weight_count,
    cot_weights_nonneg, CotWeight, CotWeightResult,
};

pub mod mesh_principal_curvature;
pub use mesh_principal_curvature::{
    avg_gaussian_curvature, avg_mean_curvature, compute_principal_curvatures, max_k1,
    PrincipalCurvature,
};

pub mod mesh_anisotropic_smooth;
pub use mesh_anisotropic_smooth::{
    aniso_avg_displacement, aniso_smooth_step, aniso_vertex_count, anisotropic_smooth,
    AnisoSmoothConfig, AnisoSmoothResult,
};

pub mod mesh_self_intersect;
pub use mesh_self_intersect::{
    detect_self_intersections, intersecting_faces, intersection_face_fraction,
    intersection_pair_count, is_self_intersection_free, IntersectingPair, SelfIntersectResult,
};

pub mod mesh_harmonic_map;
pub use mesh_harmonic_map::{
    build_adjacency as harmonic_build_adjacency, harmonic_map, harmonic_map_vertex_count,
    map_boundary_to_circle as harmonic_map_boundary_to_circle, uv_area_ratio, HarmonicMapResult,
};

pub mod mesh_displacement_map;
pub use mesh_displacement_map::{
    dm_apply_to_vertex, dm_avg_displacement, dm_get, dm_max_displacement, dm_set,
    new_displacement_map, DisplacementMap,
};

pub mod mesh_cage_lattice;
pub use mesh_cage_lattice::{
    apply_lattice_deform, bend_lattice, bernstein as lattice_bernstein,
    control_point_count as cage_control_point_count, evaluate_ffd, lattice_deformed_bounds,
    lattice_volume, move_control_point, new_lattice_cage, reset_lattice, twist_lattice,
    validate_lattice, world_to_lattice_params, LatticeCage, LatticeCageResult,
};

pub mod mesh_tweak_tool;
pub use mesh_tweak_tool::{
    apply_tweak, average_displacement as tweak_average_displacement, clamp_tweak_result,
    compute_soft_weights, count_selected as tweak_count_selected, default_tweak_params,
    scale_tweak_delta, soft_weight, undo_tweak, SoftFalloff, TweakParams, TweakResult,
};

pub mod mesh_relax_tool;
pub use mesh_relax_tool::{
    build_adjacency as relax_build_adjacency, displacement_magnitudes,
    laplacian_step as relax_tool_laplacian_step, mean_displacement as relax_mean_displacement,
    no_pins, pin_isolated, relax_in_sphere, relax_mesh as relax_tool_mesh, taubin_relax,
    RelaxResult as RelaxToolResult,
};

pub mod mesh_inflate_tool;
pub use mesh_inflate_tool::{
    average_normal as inflate_average_normal, clamp_inflate_amount,
    compute_vertex_normals as inflate_compute_vertex_normals, deflate_mesh,
    inflate_mesh as inflate_tool_mesh, inflate_to_target_offset, inflate_uniform,
    inflate_with_weight_map, InflateResult,
};

pub mod mesh_pinch_tool;
pub use mesh_pinch_tool::{
    affected_centroid, apply_expand, apply_pinch, count_in_radius, default_pinch_strength,
    max_pinch_displacement, pinch_steps, pinch_to_line, PinchResult,
};

pub mod mesh_crease_tool;
pub use mesh_crease_tool::{
    auto_crease_by_dihedral, avg_sharpness, clear_creases, crease_count as crease_tool_count,
    get_crease, list_creases, max_sharpness as crease_max_sharpness, new_crease_tool,
    remove_crease, scale_creases, set_crease, CreaseTool, EdgeKey as CreaseToolEdgeKey,
};

pub mod mesh_bridge_tool;
pub use mesh_bridge_tool::{
    align_loops_twist, bridge_loops as bridge_two_loops, bridge_new_vertex_count,
    bridge_quad_estimate, bridge_total_length, loop_centroid as bridge_loop_centroid,
    loops_compatible, make_bridge_loop, BridgeLoop, BridgeToolConfig, BridgeToolResult,
};

pub mod mesh_fillet_edge;
pub use mesh_fillet_edge::{
    apply_edge_fillet, arc_points as fillet_arc_points, default_fillet_tool_config,
    fillet_arc_length, fillet_edge_simple, fillet_radius_from_chamfer, fillet_vertex_estimate,
    validate_fillet_config, FilletToolConfig, FilletToolResult,
};

pub mod mesh_chamfer_edge;
pub use mesh_chamfer_edge::{
    chamfer_edges as chamfer_mesh_edges, chamfer_from_bevel_width, chamfer_offset_points,
    chamfer_strip, chamfer_vertex_estimate, default_chamfer_config, scale_chamfer_result,
    validate_chamfer_config, ChamferToolConfig, ChamferToolResult,
};

pub mod mesh_bevel_vertex;
pub use mesh_bevel_vertex::{
    bevel_at_factor, bevel_centroid as bevel_vert_centroid, bevel_polygon_area_2d,
    bevel_vertex_estimate, bevel_vertex_positions, bevel_vertices as apply_bevel_vertices,
    default_bevel_amount, validate_bevel_amount, VertexBevelResult,
};

pub mod mesh_inset_face;
pub use mesh_inset_face::{
    inset_amount_from_depth, inset_area_ratio, inset_faces as apply_face_inset, inset_polygon,
    inset_triangle, inset_vertex_estimate, validate_inset_amount, InsetResult as InsetFaceResult,
};

pub mod mesh_poke_face;
pub use mesh_poke_face::{
    count_poke_result_triangles, poke_all_faces, poke_faces as apply_poke_faces, poke_polygon,
    poke_triangle as poke_single_triangle, poke_triangle_estimate, poke_with_offset,
    validate_poke_input, PokeResult as PokeFaceResult,
};

pub mod mesh_flip_normals;
pub use mesh_flip_normals::{
    count_inconsistent_faces, flip_all_windings, flip_normals as flip_mesh_normals,
    flip_normals_selected, flip_selected_windings, negate_normals, negate_selected_normals,
    tri_normal as flip_tri_normal, windings_consistent, FlipNormalsResult,
};

pub mod mesh_recalc_normals;
pub use mesh_recalc_normals::{
    average_face_normal, compute_angle_weighted_normals,
    compute_face_normals as recalc_face_normals, compute_vertex_normals_recalc, normal_deviation,
    recalc_normals, smooth_normals_recalc, RecalcNormalsResult,
};

pub mod mesh_power_crust;
pub use mesh_power_crust::{
    classify_pole, generate_power_poles, medial_ball_radius, nearest_power_site,
    power_crust_has_geometry, power_crust_stub, power_crust_triangle_count, power_distance,
    PowerCrustBuilder, PowerCrustConfig, PowerCrustResult, PowerSite,
};

pub mod mesh_poisson_recon;
pub use mesh_poisson_recon::{
    density_estimate, estimate_output_faces, point_cloud_bbox, point_cloud_centroid,
    point_to_voxel, poisson_reconstruct_stub, required_octree_depth, PoissonConfig,
    PoissonReconConfig, PoissonReconResult, PoissonReconstructor,
};
