
pub mod mesh_wedge;
pub use mesh_wedge::{
    generate_wedge, wedge_base_angle, wedge_indices_valid, wedge_to_json, wedge_triangle_count,
    wedge_vertex_count, wedge_volume, WedgeMesh,
};

pub mod mesh_wire_mesh;
pub use mesh_wire_mesh::{
    extract_wire_edges, wire_avg_length, wire_edge_count, wire_indices_valid, wire_mesh_to_json,
    wire_total_length, WireEdge, WireMesh,
};

pub mod mesh_zup_convert;
pub use mesh_zup_convert::{
    bounds_zup, convert_up_axis, convert_yup_to_zup, convert_zup_to_yup, normals_yup_to_zup,
    round_trip_error, yup_to_zup, zup_to_yup, UpAxis,
};

pub mod mesh_adaptive_lod;
pub use mesh_adaptive_lod::{
    adaptive_lod_to_json, default_adaptive_lod_config, lod_level_count as adaptive_lod_level_count,
    lod_reduction_ratio as adaptive_lod_reduction_ratio, select_lod_level,
    triangle_count_at as adaptive_triangle_count_at, AdaptiveLodConfig, LodDescriptor,
};

pub mod mesh_batching;
pub use mesh_batching::{
    batch_indices_valid, batch_meshes, batch_to_json, batch_triangle_count, batch_vertex_count,
    submesh_count, BatchResult, BatchSource,
};

pub mod mesh_bvh_node;
pub use mesh_bvh_node::{
    aabb_surface_area, build_bvh_nodes, bvh_leaf_count, bvh_node_count, merge_node_aabb,
    ray_aabb_hit, triangle_aabb as bvh_triangle_aabb, BvhNode2, BvhNodeAabb,
};

pub mod mesh_cage_volume;
pub use mesh_cage_volume::{
    cage_volume_to_json, compute_cage_volume, equivalent_sphere_radius, is_outward_cage,
    scaled_cage_volume, signed_tet_volume, CageVolumeResult,
};

pub mod mesh_chart_pack;
pub use mesh_chart_pack::{
    chart_area, chart_bounding_box, chart_pack_to_json, chart_utilization, new_uv_chart,
    pack_charts, pack_charts_stub, packing_utilization, rects_overlap_cp, total_chart_area,
    ChartPackResult, ChartRect, PackedChart, PackingResult, UvChart,
};

pub mod mesh_clip_region;
pub use mesh_clip_region::{
    accepted_triangle_count as clip_region_accepted_count, clip_indices_valid, clip_region_to_json,
    clip_to_region, position_in_region, ClipRegion, ClipRegionResult,
};

pub mod mesh_coarse_grid;
pub use mesh_coarse_grid::{
    build_coarse_grid, coarse_grid_to_json, find_nearby_vertices, occupied_cell_count,
    position_to_grid_cell, total_indexed_vertices, vertices_in_cell_cg, CoarseGrid, GridCell,
};

pub mod mesh_compress_index;
pub use mesh_compress_index::{
    compress_index_to_json, decode_delta_indices, encode_delta_indices, estimate_delta_size_bytes,
    fits_u16, max_index_ci, pack_u16, unpack_u16, CompressedIndices,
};

pub mod mesh_vertex_split;
pub use mesh_vertex_split::{
    indices_valid as split_indices_valid, split_by_uvs, split_to_json as vertex_split_to_json,
    split_uvs_unique, split_vertex_count, SplitResult,
};

pub mod mesh_vertex_weight;
pub use mesh_vertex_weight::{
    average_weight as avg_vertex_weight, blend_weight_buffers,
    clamp_weights as clamp_vertex_weights, get_weight as get_vertex_weight, new_weight_buffer,
    normalize_weights as normalize_vertex_weights, set_weight as set_vertex_weight,
    weight_buffer_to_json, weight_vertex_count, weights_above_threshold, weights_valid,
    VertexWeightBuffer,
};

pub mod mesh_mirror_cut;
pub use mesh_mirror_cut::{
    axis_extent, count_positive_vertices, cut_negative_side, cut_positive_side, mirror_cut,
    mirror_cut_to_json, MirrorAxis as MirrorCutAxis, MirrorCutResult,
};

pub mod mesh_face_peel;
pub use mesh_face_peel::{
    boundary_faces, build_face_adjacency as peel_build_face_adjacency, peel_layers, peel_one_layer,
    peel_result_to_json, remaining_after_peels, FacePeelResult,
};

pub mod mesh_raycast;
pub use mesh_raycast::{
    hit_point, hit_to_json as raycast_hit_to_json, ray_direction as make_ray_direction,
    ray_triangle, raycast, raycast_all, Ray as RaycastRay, RayHit as RaycastHit,
};

pub mod mesh_bvh;
pub use mesh_bvh::{
    build_bvh, bvh_to_json as bvh_tree_to_json, bvh_triangle_count, triangle_aabb_for,
    BvhAabb as BvhAabb2, BvhNode,
};

pub mod mesh_tube;
pub use mesh_tube::{
    generate_tube as gen_tube_along_spine, tube_index_count as gen_tube_index_count,
    tube_surface_area as gen_tube_surface_area, tube_to_json as gen_tube_to_json,
    tube_vertex_count as gen_tube_vertex_count, TubeMesh as TubeMeshSpine,
};

pub mod mesh_sphere;
pub use mesh_sphere::{
    default_uv_sphere_config, generate_uv_sphere, indices_valid as uv_sphere_indices_valid,
    normals_unit as uv_sphere_normals_unit, sphere_surface_area, sphere_volume, uv_sphere_to_json,
    uv_sphere_vertex_count, UvSphereConfig, UvSphereResult,
};

pub mod mesh_plane;
pub use mesh_plane::{
    default_plane_config, generate_plane, plane_area, plane_index_count, plane_indices_valid,
    plane_to_json, plane_uvs_in_range, plane_vertex_count, PlaneConfig, PlaneMesh,
};

pub mod mesh_cylinder_gen;
pub use mesh_cylinder_gen::{
    cylinder_gen_to_json, cylinder_indices_valid, cylinder_lateral_area, cylinder_side_index_count,
    cylinder_side_vertex_count, cylinder_total_area, cylinder_volume, default_cylinder_gen_config,
    generate_cylinder, CylinderGenConfig, CylinderGenResult,
};

pub mod mesh_cone_gen;
pub use mesh_cone_gen::{
    cone_gen_to_json, cone_index_count, cone_slant_height, cone_vertex_count, cone_volume,
    default_cone_gen_config, generate_cone, ConeGenConfig, ConeGenResult,
};

pub mod mesh_torus_gen;
pub use mesh_torus_gen::{
    default_torus_gen_config, generate_torus, torus_gen_to_json, torus_index_count,
    torus_surface_area, torus_vertex_count, torus_volume, TorusGenConfig, TorusGenResult,
};

pub mod mesh_icosphere;
pub use mesh_icosphere::{
    icosahedron_faces, icosahedron_verts, icosphere_vert_count, make_icosphere,
};

pub mod mesh_edge_split;
pub use mesh_edge_split::{
    edge_split_count, split_all_long_edges, split_creates_triangles, split_edge, split_edge_at_t,
    split_edge_midpoint, split_threshold as edge_split_threshold,
    validate_split as validate_edge_split, EdgeSplitResult,
};

pub mod mesh_capsule_gen;
pub use mesh_capsule_gen::{
    capsule_gen_to_json, capsule_index_count, capsule_total_height, capsule_vertex_count,
    capsule_volume, default_capsule_gen_config, generate_capsule, CapsuleGenConfig,
    CapsuleGenResult,
};

pub mod mesh_voxelize;
pub use mesh_voxelize::{
    default_voxelize_v2_params, total_voxel_count, voxelize_surface_v2, VoxelGridV2,
    VoxelizeV2Params,
};

pub mod mesh_convex_hull_v2;
pub use mesh_convex_hull_v2::{
    convex_hull_v2, hull_v2_centroid, hull_v2_face_count, hull_volume_v2, ConvexHullV2,
};

pub mod mesh_dual_laplacian;
pub use mesh_dual_laplacian::{
    cotangent_weight as dual_laplacian_cotangent_weight, dual_laplacian_smooth,
    laplacian_at_vertex, laplacian_energy_dual, DualLaplacianConfig,
};

pub mod mesh_sharp_feature;
pub use mesh_sharp_feature::{
    detect_sharp_features, is_sharp_vertex, max_sharp_dihedral, sharp_feature_edge_count,
    SharpFeatureEdge, SharpFeatureResult,
};

pub mod mesh_slice_stack;
pub use mesh_slice_stack::{
    contour_bounds, slice_at_z, slice_stack, total_contour_points, uniform_z_heights, SliceContour,
    SliceStackResult,
};

pub mod mesh_unfold;
pub use mesh_unfold::{
    place_triangle_2d, unfold_coverage, unfold_mesh, unfold_visited_count, UnfoldResult,
};

pub mod mesh_align_axes;
pub use mesh_align_axes::{
    align_to_principal_axes, compute_centroid as align_compute_centroid, covariance_3x3,
    variance_along_axis, AlignAxesResult,
};

pub mod mesh_moment_inertia;
pub use mesh_moment_inertia::{
    compute_inertia_tensor, principal_moments, tensor_frobenius_norm, tensor_is_symmetric,
    tensor_trace, InertiaTensor,
};

pub mod mesh_heat_diffuse_v2;
pub use mesh_heat_diffuse_v2::{
    build_heat_adjacency_v2, diffuse_heat_v2, heat_gradient_v2, new_heat_field_v2,
    normalize_heat_field_v2, set_heat_source_v2, HeatDiffuseV2Config, HeatFieldV2,
};

pub mod mesh_tangent_frames;
pub use mesh_tangent_frames::{
    average_tangent_v2, compute_tangent_frames as compute_tangent_frames_v2,
    degenerate_frame_count, frames_orthonormal, world_to_tangent_space, TangentFrameV2,
};

pub mod mesh_normal_map_bake;
pub use mesh_normal_map_bake::{
    average_pixel_normal, bake_normal_map_v2, normal_map_v2_size_bytes, normal_to_rgb_v2,
    rgb_to_normal_v2, NormalMapBakeV2Config, NormalMapV2,
};

pub mod mesh_decimate_v2;
pub use mesh_decimate_v2::{
    decimate_v2, decimation_ratio_v2, quadric_error, DecimateV2Config, DecimateV2Result, Quadric4,
};

pub mod mesh_ambient_occ;
pub use mesh_ambient_occ::{
    apply_ao_to_colors, average_ao, bent_normal_at, compute_vertex_ao, hemisphere_samples_v2,
    AmbientOccV2Config,
};

pub mod mesh_quad_to_tri;
pub use mesh_quad_to_tri::{
    quad_buffer_to_triangles, quads_to_triangles, result_triangle_count, total_surface_area_q2t,
    QuadFaceV2, QuadToTriResult,
};

pub mod mesh_tri_to_quad;
pub use mesh_tri_to_quad::{
    quad_count_t2q, quadification_ratio, quads_to_flat_buffer, remaining_tri_count,
    triangles_to_quads, QuadPair, TriToQuadResult,
};

pub mod mesh_wire_frame;
pub use mesh_wire_frame::{
    generate_wireframe, total_wireframe_length, wireframe_edge_count, wireframe_indices_valid,
    wireframe_to_line_buffer, WireEdge2, WireframeMesh,
};

pub mod mesh_parametric_surf;
pub use mesh_parametric_surf::{
    compute_smooth_normals_ps, parametric_surf_triangle_count, parametric_surf_vertex_count,
    sphere_fn, tessellate_parametric, torus_fn, ParametricSurface,
};

pub mod mesh_revolve;
pub use mesh_revolve::{
    cone_profile, cylinder_profile, revolve_profile, revolve_triangle_count, RevolveSurface,
};

pub mod mesh_thicken;
pub use mesh_thicken::{
    compute_vertex_normals as thicken_compute_vertex_normals, thicken_mesh, thicken_triangle_count,
    thicken_vertex_count, ThickenResult,
};

pub mod mesh_trim;
pub use mesh_trim::{count_above_plane, trim_mesh, trim_triangle_count, TrimPlane, TrimResult};

pub mod mesh_inflate;
pub use mesh_inflate::{
    all_moved_outward, avg_displacement as inflate_avg_displacement, compute_avg_normals,
    inflate_mesh, max_displacement as inflate_max_displacement,
};

pub mod mesh_erode;
pub use mesh_erode::{all_moved_inward, erode_iterative, erode_mesh, erode_stats, ErodeStats};

pub mod mesh_fractal_displace;
pub use mesh_fractal_displace::{
    fbm as fractal_fbm, fractal_displace, max_fractal_displacement,
    value_noise_3d as fractal_value_noise_3d, FractalDispParams,
};

pub mod mesh_scatter;
pub use mesh_scatter::{
    scatter_points, total_area as scatter_total_area, triangle_area as scatter_triangle_area,
    LcgRng as ScatterLcgRng, ScatteredPoint,
};

pub mod mesh_closest_point;
pub use mesh_closest_point::{
    closest_point_on_mesh, closest_point_on_triangle as cp_on_triangle, dist3 as closest_dist3,
    ClosestPointResult,
};

pub mod mesh_smooth_color;
pub use mesh_smooth_color::{
    average_color as smooth_average_color, build_color_adjacency,
    clamp_colors as smooth_clamp_colors, colors_in_range, smooth_vertex_colors,
};

pub mod mesh_attr_transfer;
pub use mesh_attr_transfer::{
    avg_transfer_error, max_transfer_error, nearest_vertex as attr_nearest_vertex,
    transfer_rgba_attr, transfer_scalar_attr, transfer_vec3_attr,
};

pub mod mesh_debug_vis;
pub use mesh_debug_vis::{
    generate_bitangent_lines, generate_normal_lines, generate_tangent_lines,
    line_length as debug_line_length, lines_to_color_buffer, lines_to_position_buffer, DebugLine,
};

pub mod mesh_boolean_csg;
pub use mesh_boolean_csg::{
    csg_combine, csg_inside_count, sample_csg_grid, CsgOp, SdfBox, SdfSphere,
};

pub mod mesh_swept_solid;
pub use mesh_swept_solid::{
    sweep_profile, swept_triangle_count, swept_vertex_count, Profile2D as SweptProfile2D, SweptMesh,
};

pub mod mesh_pipe_network;
pub use mesh_pipe_network::{
    build_pipe_network, pipe_triangle_count, tube_segment, PipeEdge, PipeNetworkMesh, PipeNode,
};

pub mod mesh_lattice_frame;
pub use mesh_lattice_frame::{
    build_lattice_mesh, cubic_lattice_beams, cubic_lattice_nodes, lattice_triangle_count, Beam,
    LatticeMesh,
};

pub mod mesh_voronoi_cell;
pub use mesh_voronoi_cell::{
    build_voronoi_cells, nearest_seed, total_assigned_points, Aabb3, VoronoiCell as VoronoiGridCell,
};

pub mod mesh_spring_net;
pub use mesh_spring_net::{
    average_rest_length, build_spring_net, spring_edge_count, spring_stretch, SpringEdge, SpringNet,
};

pub mod mesh_helix;
pub use mesh_helix::{
    build_helix_mesh, helix_path as helix_tube_path, helix_triangle_count, helix_vertex_count,
    HelixMesh, HelixParams,
};

pub mod mesh_mobius;
pub use mesh_mobius::{
    build_mobius_mesh, mobius_point, mobius_triangle_count, mobius_vertex_count, MobiusMesh,
    MobiusParams,
};

pub mod mesh_klein_bottle;
pub use mesh_klein_bottle::{
    build_klein_mesh, klein_point, klein_triangle_count, klein_vertex_count, KleinMesh, KleinParams,
};

pub mod mesh_trefoil;
pub use mesh_trefoil::{
    build_trefoil_mesh, trefoil_path, trefoil_point, trefoil_triangle_count, trefoil_vertex_count,
    TrefoilMesh, TrefoilParams,
};

pub mod mesh_superellipsoid;
pub use mesh_superellipsoid::{
    build_superellipsoid_mesh, spow, superellipsoid_point, superellipsoid_triangle_count,
    superellipsoid_vertex_count, SuperellipsoidMesh, SuperellipsoidParams,
};

pub mod mesh_dupin_cyclide;
pub use mesh_dupin_cyclide::{
    build_cyclide_mesh, cyclide_point, cyclide_triangle_count, cyclide_vertex_count, CyclideMesh,
    CyclideParams,
};

pub mod mesh_minimal_surface;
pub use mesh_minimal_surface::{
    build_minimal_surface_mesh, enneper_point, minimal_surface_triangle_count,
    minimal_surface_vertex_count, scherk_point, MinimalSurfaceMesh, MinimalSurfaceParams,
    MinimalSurfaceType,
};

pub mod mesh_fused_deposition;
pub use mesh_fused_deposition::{
    add_rect_path, add_zigzag_infill, layer_count as fdm_layer_count, layer_path_length,
    slice_bbox_into_layers, total_path_points, FdmJob, FdmLayer,
};

pub mod mesh_isosurface;
pub use mesh_isosurface::{
    extract_isosurface, fill_sphere_sdf, isosurface_triangle_count, isosurface_vertex_count,
    IsosurfaceMesh, ScalarGrid,
};

pub mod mesh_multiresolution;
pub use mesh_multiresolution::{
    apply_displacement as mr_apply_displacement, pop_level, push_level,
    reset_displacements as mr_reset_displacements,
    total_displacement_magnitude as mr_total_displacement_magnitude, MrLevel, MultiresolutionMesh,
};

pub mod mesh_ptex;
pub use mesh_ptex::{PtexFaceData, PtexFaceRes, PtexTexture};

pub mod mesh_paint_mask;
pub use mesh_paint_mask::{
    average_weight as paint_average_weight, count_above as paint_count_above,
    from_bytes as paint_from_bytes, to_bytes as paint_to_bytes, PaintMask,
};

pub mod mesh_face_map;
pub use mesh_face_map::{
    face_in_any_group, merge_face_maps, rename_group,
    total_face_count as face_map_total_face_count, FaceMap,
};

pub mod mesh_vertex_group_weight;
pub use mesh_vertex_group_weight::{
    dominant_group, normalize_across_groups, VertexWeight, VertexWeightGroup, VertexWeightGroupSet,
};

pub mod mesh_shape_key_mix;
pub use mesh_shape_key_mix::{
    dominant_key, mix_shape_keys, reset_all_weights, set_key_weight, total_influence, ShapeKey,
};

pub mod mesh_cloth_pins;
pub use mesh_cloth_pins::{
    average_strength, fully_pinned_count, pinned_vertex_indices, scale_strengths, ClothPinEntry,
    ClothPinGroup,
};

pub mod mesh_particle_hair;
pub use mesh_particle_hair::{
    average_strand_length, max_strand_length, HairStrand, ParticleHairMesh,
};

pub mod mesh_fur_cards;
pub use mesh_fur_cards::{average_card_length, generate_fur_cards, FurCard, FurCardMesh};

pub mod mesh_feather;
pub use mesh_feather::{generate_feather, total_segment_count, Barb, Feather, FeatherParams};

pub mod mesh_scale_elements;
pub use mesh_scale_elements::{
    face_centroid as scale_face_centroid, scale_edges, scale_faces,
    selected_face_count as scale_selected_face_count,
};

pub mod mesh_push_pull;
pub use mesh_push_pull::{
    clamp_positions as push_pull_clamp_positions, push_all, push_pull, push_pull_magnitude,
    selected_vertex_count as push_pull_selected_vertex_count,
};

pub mod mesh_smooth_vertex;
pub use mesh_smooth_vertex::{
    build_adjacency as smooth_build_adjacency, max_displacement as smooth_max_displacement,
    smooth_n, smooth_step,
};

pub mod mesh_relax_uv;
pub use mesh_relax_uv::{
    build_uv_adjacency, max_uv_displacement, relax_step as uv_relax_step, relax_uvs,
    uvs_in_range as relax_uvs_in_range,
};

pub mod mesh_pack_islands;
pub use mesh_pack_islands::{
    atlas_utilization as pack_atlas_utilization, largest_island, pack_uv_islands,
    total_island_area, PackedIsland, UvIslandBounds,
};

pub mod mesh_seam_mark_v2;
pub use mesh_seam_mark_v2::{mark_boundary_seams2, sorted_seams2, SeamEdge2, SeamSet2};

pub mod mesh_normal_edit;
pub use mesh_normal_edit::{
    apply_normal_edits, edit_count as normal_edit_count, get_custom_normal, new_normal_edit_layer,
    new_normal_edit_params, normal_blend, normal_edit_apply, normal_flip, normal_is_valid,
    normal_normalize, set_custom_normal, NormalEdit, NormalEditLayer, NormalEditParams,
};

pub mod mesh_auto_smooth;
pub use mesh_auto_smooth::{
    auto_smooth, config_from_degrees as auto_smooth_config_degrees, count_sharp_vertices,
    dihedral_angle as auto_smooth_dihedral_angle, is_smooth_angle, AutoSmoothConfig,
    AutoSmoothResult,
};

pub mod mesh_face_orient;
pub use mesh_face_orient::{
    face_orientation, flip_orientation, is_consistently_oriented, orient_count,
    orient_faces_consistently, orient_from_normals, orient_to_json, orientation_errors, FaceOrient,
};

pub mod mesh_limited_dissolve;
pub use mesh_limited_dissolve::{
    config_from_degrees as dissolve_config_degrees, limited_dissolve, multi_face_groups,
    total_dissolved, within_dissolve_angle, DissolveResult, LimitedDissolveConfig,
};

pub mod mesh_planar_decimate;
pub use mesh_planar_decimate::{
    are_coplanar as planar_are_coplanar, config_degrees as planar_config_degrees,
    decimate_ratio as planar_decimate_ratio, planar_decimate, survivor_count, PlanarDecimateConfig,
    PlanarDecimateResult,
};

pub mod mesh_unsubdivide;
pub use mesh_unsubdivide::{
    find_loop_midpoints, had_effect as unsubdivide_had_effect, surviving_tri_count, unsubdivide,
    vertex_reduction_ratio, UnsubdivideConfig, UnsubdivideResult,
};

pub mod mesh_tris_to_quads;
pub use mesh_tris_to_quads::{
    merge_ratio as tris_to_quads_merge_ratio, quad_count as tris_quad_count, quads_to_tri_indices,
    total_faces as tris_to_quads_total_faces, tri_count as tris_remaining_tri_count, tris_to_quads,
    try_merge_to_quad, validate_quads, TrisToQuadsResult,
};

pub mod mesh_quads_to_tris;
pub use mesh_quads_to_tris::{
    default_split_mode, mixed_to_tris, quads_to_tris, split_quad, tri_index_count,
    triangle_count_from_quads, validate_quad_input, QuadSplitMode, QuadsToTrisResult,
};

pub mod mesh_ngon_fill;
pub use mesh_ngon_fill::{
    all_valid as ngon_all_valid, ear_clip_fill_2d, expected_tri_count as ngon_expected_tri_count,
    fan_fill, ngon_fill, triangle_count as ngon_tri_count, NgonFillMethod, NgonFillResult,
};

pub mod mesh_grid_fill;
pub use mesh_grid_fill::{
    dimensions_match, grid_bounds, grid_fill, grid_fill_face_count, grid_fill_is_valid,
    grid_fill_vertex, grid_fill_vertex_count, grid_quads_to_tris, new_grid_fill,
    quad_count as grid_quad_count, vertex_count as grid_vertex_count, GridFillConfig,
    GridFillParams, GridFillResult,
};

pub mod mesh_fill_holes;
pub use mesh_fill_holes::{
    boundary_vert_count, detect_holes, fill_holes as fill_holes_from_boundary, fill_triangle_count,
    has_holes, merge_filled as merge_filled_holes, FillHolesConfig, FillHolesResult,
    MeshHoleDetected,
};

pub mod mesh_convex_hull_2d;
pub use mesh_convex_hull_2d::{
    convex_hull_2d, cross_2d, hull_area, hull_perimeter, point_in_hull as point_in_hull_2d,
};

pub mod mesh_ear_clip;
pub use mesh_ear_clip::{
    all_valid as ear_clip_all_valid, ear_clip as ear_clip_polygon, ear_clip_count,
    ear_clip_triangulate, ear_is_ear, expected_count as ear_clip_expected_count, is_ccw,
    polygon_area_2d as ear_clip_polygon_area, polygon_is_convex_vertex, triangle_area_2d,
    triangle_count as ear_clip_tri_count, EarClipResult,
};

pub mod mesh_fan_triangulate;
pub use mesh_fan_triangulate::{
    fan_area as fan_polygon_area, fan_centroid, fan_triangle_count as fan_tri_count,
    fan_triangulate, fan_triangulate_3d, fan_triangulate_all, fan_triangulate_ngon,
    is_convex_polygon, polygon_area_2d as fan_polygon_area_2d,
};

pub mod mesh_cloth_collider;
pub use mesh_cloth_collider::*;

pub mod mesh_fluid_surface;
pub use mesh_fluid_surface::*;

pub mod mesh_hair_guide_gen;
pub use mesh_hair_guide_gen::*;

pub mod mesh_strand_mesh;
pub use mesh_strand_mesh::*;

pub mod mesh_tube_curve;
pub use mesh_tube_curve::{
    expected_triangle_count as tube_curve_expected_triangle_count,
    expected_vertex_count as tube_curve_expected_vertex_count, tube_along_curve,
    validate_tube_params, TubeCurveParams, TubeMesh as TubeCurveMesh,
};

pub mod mesh_cable_gen;
pub use mesh_cable_gen::{
    build_cable, cable_lateral_area as cable_gen_lateral_area, cable_mass,
    expected_triangle_count as cable_expected_triangle_count,
    expected_vertex_count as cable_expected_vertex_count, validate_cable_params, CableMesh,
    CableParams,
};

pub mod mesh_chain_link;
pub use mesh_chain_link::{
    build_chain, build_link, chain_triangle_count, triangles_per_link, validate_link_params,
    ChainLinkParams, LinkMesh,
};

pub mod mesh_spring_helix;
pub use mesh_spring_helix::{
    build_spring_helix, spring_arc_length, validate_spring_params, SpringHelixMesh,
    SpringHelixParams,
};

pub mod mesh_gear_tooth;
pub use mesh_gear_tooth::{
    addendum_radius, base_radius, build_gear, dedendum_radius, involute_point, pitch_radius,
    validate_gear_params, GearMesh, GearParams,
};

pub mod mesh_screw_helix;
pub use mesh_screw_helix::{
    build_screw, estimated_vertex_count as screw_estimated_vertex_count, screw_arc_length,
    thread_depth, validate_screw_params, ScrewMesh, ScrewParams,
};

pub mod mesh_torus_knot;
pub use mesh_torus_knot::{
    build_torus_knot, expected_vertex_count as torus_knot_expected_vertex_count, torus_knot_point,
    validate_torus_knot_params, TorusKnotMesh, TorusKnotParams,
};

pub mod mesh_nonorientable;
pub use mesh_nonorientable::{
    build_mobius_strip, expected_vertex_count as mobius_expected_vertex_count,
    mobius_point as nonorientable_mobius_point, validate_mobius_params,
    MobiusMesh as MobiusStripMesh, MobiusStripParams,
};

pub mod mesh_projective_plane;
pub use mesh_projective_plane::{
    build_klein_bottle, expected_triangle_count as klein_expected_triangle_count,
    expected_vertex_count as klein_expected_vertex_count, klein_point as projective_klein_point,
    validate_klein_params, KleinBottleMesh, KleinBottleParams,
};

pub mod mesh_torus_ring;
pub use mesh_torus_ring::{
    build_torus_ring, expected_vertex_count as torus_ring_expected_vertex_count, torus_point,
    torus_surface_area as torus_ring_surface_area, torus_volume as torus_ring_volume,
    validate_torus_params, TorusRingMesh, TorusRingParams,
};

pub mod mesh_geosphere;
pub use mesh_geosphere::{
    build_geosphere, expected_triangle_count as geosphere_expected_triangle_count,
    sphere_surface_area as geosphere_sphere_surface_area, validate_geosphere_params, GeosphereMesh,
    GeosphereParams,
};

pub mod mesh_end_cap;
pub use mesh_end_cap::{
    build_capped_cylinder, lateral_area as end_cap_lateral_area,
    total_surface_area as end_cap_total_surface_area,
    validate_params as validate_capped_cylinder_params, CappedCylinderMesh, CappedCylinderParams,
};

pub mod mesh_loft;
pub use mesh_loft::{
    loft_profiles, loft_triangle_count, loft_vertex_count as mesh_loft_vertex_count,
    profile_centroid, LoftResult,
};

pub mod mesh_ruled_surface;
pub use mesh_ruled_surface::{
    build_ruled_surface, lerp3 as ruled_lerp3, ruled_tri_count, ruled_vertex_count,
    validate_ruled_surface, RuledSurface,
};

pub mod mesh_coons_patch;
pub use mesh_coons_patch::{
    build_coons_patch, coons_eval, coons_tri_count, coons_vertex_count, validate_coons_patch,
    CoonsPatch,
};

pub mod mesh_nurbs_surface;
pub use mesh_nurbs_surface::{
    new_nurbs_surface, nurbs_control_bbox, nurbs_control_point_count, tessellate_nurbs,
    uniform_knots, validate_nurbs, NurbsSurface,
};

pub mod mesh_bezier_surface;
pub use mesh_bezier_surface::{
    bernstein3, bezier_surface_tri_count, bezier_surface_vertex_count, eval_bezier_surface,
    tessellate_bezier_surface, validate_bezier_surface, BezierControlGrid, BezierSurface,
};

pub mod mesh_gordon_surface;
pub use mesh_gordon_surface::{
    gordon_total_u_points, gordon_u_curve_count, gordon_v_curve_count, new_gordon_surface,
    tessellate_gordon, validate_gordon, GordonSurface,
};

pub mod mesh_skinned_surface;
pub use mesh_skinned_surface::{
    build_skinned_surface, chord_length_params, skinned_tri_count, skinned_vertex_count,
    validate_skinned_surface, SkinnedSurface,
};

pub mod mesh_revolution_surface;
pub use mesh_revolution_surface::{
    build_revolution_surface, revolution_surface_area, revolution_tri_count,
    revolution_vertex_count, validate_revolution_surface, RevolutionSurface,
};

pub mod mesh_extruded_polygon;
pub use mesh_extruded_polygon::{
    build_extruded_polygon, extruded_tri_count, extruded_vertex_count,
    lateral_area as extruded_lateral_area, validate_extruded_polygon, ExtrudedPolygon,
};

pub mod mesh_prism_frustum;
pub use mesh_prism_frustum::{
    build_prism_frustum, frustum_lateral_area, frustum_tri_count, frustum_vertex_count,
    frustum_volume, validate_prism_frustum, PrismFrustum,
};

pub mod mesh_platonic_solid;
pub use mesh_platonic_solid::{
    build_platonic_solid, is_unit_sphere as platonic_is_unit_sphere, platonic_tri_count,
    platonic_vertex_count, validate_platonic, PlatonicKind, PlatonicSolid,
};

pub mod mesh_archimedean_solid;
pub use mesh_archimedean_solid::{
    archimedean_tri_count, archimedean_vertex_count, build_archimedean_solid,
    is_unit_sphere as archimedean_is_unit_sphere, validate_archimedean, ArchimedeanKind,
    ArchimedeanSolid,
};

pub mod mesh_antiprism;
pub use mesh_antiprism::{
    antiprism_expected_tris, antiprism_tri_count, antiprism_vertex_count, build_antiprism,
    has_alternating_strip, validate_antiprism, Antiprism,
};

pub mod mesh_bipyramid;
pub use mesh_bipyramid::{
    bipyramid_expected_tris, bipyramid_surface_area, bipyramid_tri_count, bipyramid_vertex_count,
    build_bipyramid, validate_bipyramid, Bipyramid,
};

pub mod mesh_trapezohedron;
pub use mesh_trapezohedron::{
    build_trapezohedron, trapezohedron_expected_tris, trapezohedron_surface_area,
    trapezohedron_tri_count, trapezohedron_vertex_count, validate_trapezohedron, Trapezohedron,
};

pub mod mesh_stellated_solid;
pub use mesh_stellated_solid::{
    build_stellated_solid, stellated_tri_count, stellated_vertex_count, stellations_protrude,
    validate_stellated, StellatedSolid, StellationBase,
};

pub mod mesh_crease_set;
pub use mesh_crease_set::{
    add_crease_set, add_edge_to_crease_set, find_crease_set, merge_crease_set_collections,
    new_crease_set_collection, total_crease_edges, validate_crease_sets, CreaseSet,
    CreaseSetCollection,
};

pub mod mesh_sharp_vertex;
pub use mesh_sharp_vertex::{
    get_vertex_sharpness, is_sharp_vertex as is_sharp_corner_vertex, mark_sharp_vertex,
    max_sharpness, new_sharp_vertex_set, sharp_vertex_count, unmark_sharp_vertex, SharpVertexSet,
};

pub mod mesh_custom_split_normals;
pub use mesh_custom_split_normals::{
    clear_split_normals, get_split_normal, new_split_normal_layer, set_split_normal,
    split_normal_count, validate_split_normals, SplitNormal, SplitNormalLayer,
};

pub mod mesh_normal_override;
pub use mesh_normal_override::{
    get_override_normal, new_normal_override_layer, override_count, remove_override,
    set_override_normal, validate_overrides, NormalOverrideLayer,
};

pub mod mesh_uv_pin;
pub use mesh_uv_pin::{
    clear_pins, get_pin_uv, is_pinned, new_uv_pin_set, pin_count, pin_vertex, unpin_vertex,
    UvPinSet,
};

pub mod mesh_uv_minimize_stretch;
pub use mesh_uv_minimize_stretch::{
    average_stretch, minimize_stretch, triangle_uv_stretch, StretchMinConfig, StretchMinResult,
};

pub mod mesh_uv_align_axis;
pub use mesh_uv_align_axis::{
    align_to_axis, dominant_angle_deg, rotate_uv, uv_bounding_box, uv_centroid, AlignAxis,
    UvAlignResult,
};

pub mod mesh_uv_rotate_align;
pub use mesh_uv_rotate_align::{
    edge_is_horizontal, edge_uv_angle_deg, rotate_align_edge, rotate_island, UvRotateAlignResult,
};

pub mod mesh_uv_scale_to_bounds;
pub use mesh_uv_scale_to_bounds::{
    scale_uvs_to_bounds, scale_uvs_uniform, uv_extents, UvTargetBounds,
};

pub mod mesh_uv_copy_paste;
pub use mesh_uv_copy_paste::{
    clear_clipboard, clipboard_fits, clipboard_size, copy_uvs, new_uv_clipboard, paste_exact,
    paste_uvs, UvClipboard,
};

pub mod mesh_vertex_color_layer;
pub use mesh_vertex_color_layer::{
    add_vertex_color_layer, get_layer_mut, layer_average_color,
    layer_count as vertex_color_layer_count, new_vertex_color_layer_set, remove_vertex_color_layer,
    set_active_layer, VertexColorLayer, VertexColorLayerSet,
};

pub mod mesh_attribute_layer;
pub use mesh_attribute_layer::{
    add_attribute_layer, attribute_average, attribute_layer_count, get_attribute_layer,
    get_attribute_layer_mut, new_attribute_layer_set, remove_attribute_layer,
    validate_attribute_layers, AttrDomain, AttributeLayer, AttributeLayerSet,
};

pub mod mesh_custom_data;
pub use mesh_custom_data::{
    clear_custom_data, custom_entry_count, get_custom_entry, list_custom_keys,
    new_custom_data_block, remove_custom_entry, set_custom_entry, CustomDataBlock, CustomDataEntry,
    CustomDataValue,
};

pub mod mesh_property_layer;
pub use mesh_property_layer::{
    add_property_layer, get_property, new_property_layer_set, property_layer_count,
    property_min_max, remove_property_layer, reset_property_layer, set_property, PropertyLayer,
    PropertyLayerSet,
};

pub mod mesh_flag_layer;
pub use mesh_flag_layer::{
    add_flag_layer, clear_flags, flag_layer_count, flag_set_count, get_flag, invert_flags,
    new_flag_layer_set, set_flag, FlagLayer, FlagLayerSet,
};

pub mod mesh_topo_repair_ext;
pub use mesh_topo_repair_ext::*;

pub mod mesh_duplicate_face;
pub use mesh_duplicate_face::*;

pub mod mesh_non_manifold_fix;
pub use mesh_non_manifold_fix::*;

pub mod mesh_isolated_vertex_remove;
pub use mesh_isolated_vertex_remove::*;

pub mod mesh_face_split_edge;
pub use mesh_face_split_edge::*;

pub mod mesh_loop_slide;
pub use mesh_loop_slide::*;

pub mod mesh_rip_vertex;
pub use mesh_rip_vertex::*;

pub mod mesh_rip_fill;
pub use mesh_rip_fill::*;

pub mod mesh_dissolve_face;
pub use mesh_dissolve_face::*;

pub mod mesh_dissolve_edge;
pub use mesh_dissolve_edge::*;

pub mod mesh_dissolve_vertex;
pub use mesh_dissolve_vertex::*;

pub mod mesh_flatten_face;
pub use mesh_flatten_face::*;
