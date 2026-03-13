pub mod mesh_bilaplacian;
pub use mesh_bilaplacian::{
    bilaplacian_energy, bilaplacian_fairing, bilaplacian_smooth, cotangent_weight,
    mesh_laplacian_energy,
};
pub mod mesh_featureline;
pub use mesh_featureline::{
    compute_all_gaussian_curvatures, compute_all_mean_curvatures, extract_feature_lines,
    extract_ridges, extract_valleys, feature_line_density, shape_index_at_vertex,
};

pub mod mesh_topology_repair;
pub use mesh_topology_repair::{
    count_topology_issues, detect_degenerate_faces, detect_duplicate_vertices,
    find_isolated_vertices, find_non_manifold_edges_advanced, new_topology_issue,
    remove_isolated_vertices, remove_non_manifold_faces, stitch_boundary_edges,
    topology_issue_name, topology_repair_report, vertex_face_count, TopologyIssue,
};
pub mod mesh_curvature_tensor;
pub use mesh_curvature_tensor::{
    classify_surface_point, compute_all_curvature_tensors, compute_vertex_curvature_tensor,
    curvedness_from_tensor, gaussian_curvature_from_tensor, mean_curvature_from_tensor,
    shape_index_from_tensor, CurvatureTensor,
};
pub mod mesh_geodesic_path;
pub use mesh_geodesic_path::{
    dijkstra_geodesic, dijkstra_geodesic_between, farthest_point_sampling, geodesic_centroid,
    geodesic_distance, geodesic_voronoi, mesh_diameter_geodesic, GeodesicPath,
};

pub mod mesh_texture_bake;
pub use mesh_texture_bake::{
    bake_ambient_occlusion, bake_curvature, bake_dispatch,
    bake_normal_map as texture_bake_normal_map, bake_target_to_u8, bake_thickness,
    load_bake_from_f32_slice, new_bake_target, sample_bake_at_uv, save_bake_ppm, BakeMode, BakeRay,
    BakeTarget,
};

pub mod mesh_segment;
pub use mesh_segment::{
    assign_face_colors, compute_segment_adjacency, largest_segment, merge_small_segments,
    segment_boundary_edges, segment_by_connectivity, segment_by_normal_deviation,
    segment_by_planarity, segment_dispatch, segment_stats, MeshSegment, SegmentCriteria,
};

pub mod mesh_seam_welder;
pub use mesh_seam_welder::{
    compute_average_normal, count_boundary_verts, detect_uv_islands, find_duplicate_positions,
    find_seam_edges, merge_vertex_groups, project_uv_along_edge, seam_boundary_length,
    seam_weld_report, weld_seams, MergeGroupResult, SeamEdge, WeldSeamResult,
};

pub mod mesh_projection;
pub use mesh_projection::{
    barycentric_to_point, compute_barycentric, interpolate_uv_at_barycentric, project_along_axis,
    project_mesh_onto_mesh, project_point_to_mesh, project_point_to_triangle, shrink_wrap_proj,
    snap_to_surface, transfer_attributes, ProjectionMode, ProjectionResult,
};

pub mod mesh_curve_deform;
pub use mesh_curve_deform::{
    bezier_arc_length, bezier_tangent, cross3 as curve_cross3, deform_mesh_along_curve,
    evaluate_bezier, normalize3 as curve_normalize3, project_to_curve_param, sample_curve_points,
    BezierCurve, CurveAxis, CurvePoint, SplineDeformResult,
};

pub mod mesh_align;
pub use mesh_align::{
    align_bounding_boxes, align_to_axes, apply_rotation, bounding_box as align_bounding_box,
    compute_centroid, covariance_matrix_3x3, find_nearest, icp_align, normalize_to_unit_box,
    scale_mesh as align_scale_mesh, translate_mesh as align_translate_mesh, AlignResult, IcpResult,
};

pub mod mesh_parametric;
pub use mesh_parametric::{
    make_capsule, make_cone, make_cylinder, make_plane, make_sphere, make_torus, merge_parametric,
    parametric_index_count, parametric_vertex_count, validate_parametric_mesh, ParametricMesh,
    ParametricShape,
};

pub mod mesh_topology_flow;
pub use mesh_topology_flow::{
    adjacent_faces, adjacent_vertices, boundary_vertices, build_topology, face_vertex_count,
    find_edge_loop, find_edge_ring, find_poles, is_closed_mesh, is_manifold, topology_stats,
    vertex_valence as topo_vertex_valence, FlowDir, HalfEdge, Pole, TopoEdgeLoop, TopologyMesh,
};

pub mod mesh_extrude;
pub use mesh_extrude::{
    compute_face_normal as extrude_face_normal, default_extrude_config, extrude_along_curve,
    extrude_distance, extrude_edges, extrude_faces, extrude_vertex_count, extrude_vertices,
    inset_faces, solidify_mesh, ExtrudeConfig, ExtrudeMode, ExtrudeResult,
};

pub mod mesh_fillet;
pub use mesh_fillet::{
    arc_points, bevel_vertices, blend_edge_points, chamfer_amount_from_radius, chamfer_edge,
    chamfer_edges, default_fillet_config, edge_dihedral_angle, fillet_edge, find_sharp_edges,
    ChamferResult, FilletConfig, FilletResult,
};

pub mod mesh_heat_diffuse;
pub use mesh_heat_diffuse::{
    build_adjacency as heat_build_adjacency, default_heat_config, diffuse_heat, diffuse_step,
    geodesic_heat_distance, heat_field_max, heat_field_min, heat_gradient, heat_to_color,
    new_heat_field, normalize_field as heat_normalize_field, set_heat_source, threshold_field,
    HeatDiffuseConfig, HeatField, HeatSource,
};

pub mod mesh_uv_stitch;
pub use mesh_uv_stitch::{
    boundary_uv_edges, clamp_uvs, detect_uv_islands_stitched, find_uv_seams, island_area_s,
    mirror_uvs_horizontal, mirror_uvs_vertical, pack_islands_simple, stitch_all_seams, stitch_seam,
    stitch_seam_count, uv_seam_length, uv_stretch, StitchResult, UvIslandS, UvSeam,
};

pub mod mesh_voronoi;
pub use mesh_voronoi::{
    cell_count, centroidal_voronoi_step, compute_voronoi, default_voronoi_config, largest_cell,
    smallest_cell, vertex_cell_id, voronoi_balance_score, voronoi_boundary_edges,
    voronoi_cell_area, voronoi_cell_centroid, voronoi_from_seeds, voronoi_random_seeds,
    VoronoiCell, VoronoiConfig, VoronoiDiagram, VoronoiMetric,
};

pub mod mesh_wave_deform;
pub use mesh_wave_deform::{
    apply_multiple_ripples, apply_ripple, apply_wave_deform, default_wave_params,
    distance3 as wave_distance3, dot3 as wave_dot3, normalize3 as wave_normalize3, ripple_value,
    standing_wave, wave_envelope, wave_interference, wave_value, RippleSource, WaveParams,
    WaveShape,
};

pub mod mesh_dual;
pub use mesh_dual::{
    average_dual_edge_length, build_face_adjacency as dual_build_face_adjacency, compute_dual_mesh,
    dual_edge_count, dual_edge_lengths, dual_mesh_bounds, dual_to_graph_adjacency,
    dual_to_positions, dual_vertex_count, dual_vertex_degree, face_centroid, is_dual_connected,
    DualMesh,
};

pub mod mesh_sweep;
pub use mesh_sweep::{
    circle_profile, cross3_sweep, frame_at_path_point, helix_path, line_path, normalize3_sweep,
    path_arc_lengths, path_length as sweep_path_length, profile_perimeter, rectangle_profile,
    sweep_profile_along_path, sweep_result_face_count, sweep_result_vertex_count,
    transform_profile_point, SweepPath, SweepProfile, SweepResult,
};

pub mod mesh_medial_axis;
pub use mesh_medial_axis::{
    approximate_medial_axis, default_medial_config, medial_axis_bounds, medial_axis_connectivity,
    medial_axis_length, medial_axis_to_json, medial_edge_count, medial_point_count,
    medial_to_spheres, nearest_surface_distance, prune_short_branches, thickest_point, MedialAxis,
    MedialAxisConfig, MedialPoint,
};

pub mod mesh_vertex_cluster;
pub use mesh_vertex_cluster::{
    apply_vertex_remap, build_cluster_grid, cluster_centroid, cluster_face_count,
    cluster_reduction_ratio, cluster_vertex_count, cluster_vertices, default_cluster_config,
    merge_close_vertices, merge_duplicate_uvs, remove_degenerate_triangles, verify_cluster_remap,
    ClusterConfig, ClusterResult,
};

pub mod mesh_orient;
pub use mesh_orient::{
    all_normals_outward, compute_mesh_normals as orient_compute_mesh_normals,
    consistent_winding_check, default_orient_config, face_normal as orient_face_normal,
    flip_all_faces, flip_face, mesh_centroid as orient_mesh_centroid, orient_mesh,
    orient_result_summary, triangle_normal as orient_triangle_normal, triangle_normal_normalized,
    OrientConfig, OrientResult,
};

pub mod mesh_simplicial;
pub use mesh_simplicial::{
    add_simplex1, add_simplex2, betti_0, boundary_edges_simplex, boundary_of_edge,
    boundary_of_triangle, dual_graph_simplex, euler_characteristic, from_mesh_indices,
    is_manifold_simplex, new_simplicial_complex, simplex1_count, simplex2_count, Simplex0,
    Simplex1, Simplex2, SimplicialComplex,
};

pub mod mesh_spectral;
pub use mesh_spectral::{
    build_laplacian, default_spectral_config, degree_centrality, graph_laplacian_energy,
    laplacian_operator, laplacian_smooth_signal, laplacian_vertex_count, mesh_diameter_spectral,
    normalize_signal as spectral_normalize_signal, power_iterate, spectral_embedding_1d,
    spectral_partition, GraphLaplacian, SpectralConfig,
};

pub mod mesh_polar_decomp;
pub use mesh_polar_decomp::{
    compute_deformation_gradient, deformation_field_divergence, mat3_det, mat3_identity, mat3_mul,
    mat3_scale, mat3_transpose, per_face_deformation, polar_decompose, rigid_body_deformation,
    rotation_error, stretch_ratio, DeformationGradient, PolarDecomp,
};

pub mod mesh_geodesic;
pub use mesh_geodesic::{
    build_edge_graph, default_geodesic_config, farthest_point, geodesic_diameter,
    geodesic_distances, geodesic_distances_multi, geodesic_heat, geodesic_path,
    geodesic_vertex_count, geodesic_voronoi as mesh_geodesic_voronoi, level_set_isolines,
    normalize_geodesic, EdgeGraph, GeodesicConfig, GeodesicResult as MeshGeodesicResult,
    VoronoiLabels,
};

pub mod mesh_mean_curvature;
pub use mesh_mean_curvature::{
    compute_gaussian_curvature as mc_gaussian_curvature,
    compute_mean_curvature as mc_mean_curvature, cotangent_weight as mean_curv_cotangent_weight,
    curvature_color_map, curvature_max, curvature_mean_value, curvature_min,
    default_curvature_config, laplacian_smooth_positions, mean_curvature_vector,
    principal_curvatures, vertex_area_mixed, CurvatureColor, CurvatureConfig, CurvaturePair,
    CurvatureResult,
};

pub mod mesh_convex_hull;
pub use mesh_convex_hull::{
    compute_convex_hull, convex_hull_surface_area, convex_hull_volume, default_hull_config,
    hull_bounding_box, hull_centroid, hull_edge_count, hull_face_count, hull_vertex_count,
    is_point_inside_hull, project_to_hull_surface, support_point, ConvexHullResult, HullConfig,
};

pub mod mesh_decimate;
pub use mesh_decimate::{
    collapse_edge as mesh_collapse_edge, compute_quadric,
    count_boundary_edges as mesh_count_boundary_edges, decimate_mesh, decimate_step,
    decimate_to_target_faces, decimation_ratio, default_decimate_config,
    edge_collapse_cost as mesh_edge_collapse_cost, find_cheapest_edge, mesh_complexity_score,
    validate_decimated_mesh, DecimateConfig, DecimateResult as MeshDecimateResult,
};

pub mod mesh_catmull_clark;
pub use mesh_catmull_clark::{
    compute_edge_points, compute_face_points, compute_vertex_points, default_subdiv_config,
    is_quad_mesh, smooth_boundary_vertices, subdivide_catmull_clark, subdivide_n_levels,
    subdivision_face_count_estimate, subdivision_level, subdivision_vertex_count_estimate,
    triangulate_quads, SubdivConfig, SubdivResult,
};

pub mod mesh_remesh;
pub use mesh_remesh::{
    adaptive_target_length, collapse_short_edges as mesh_remesh_collapse_short_edges,
    compute_target_edge_length as mesh_remesh_target_edge_length, count_irregular_vertices,
    default_remesh_config, edge_lengths_stats, equalize_valence,
    isotropic_remesh as mesh_isotropic_remesh, remesh_iterations, remesh_quality_score,
    split_long_edges as mesh_remesh_split_long_edges, tangential_smooth, RemeshConfig,
    RemeshResult as MeshRemeshResult, RemeshStats,
};

pub mod mesh_bezier_patch;
pub use mesh_bezier_patch::{
    default_patch_config, evaluate_patch, new_bezier_patch, patch_bounding_box, patch_midpoint,
    patch_normal, patch_tangent_u, patch_tangent_v, patch_triangle_count, patch_vertex_count,
    subdivide_patch, tessellate_patch, BezierPatch, PatchConfig, PatchSample, PatchTessellation,
    SubPatches,
};

pub mod mesh_octree;
pub use mesh_octree::{
    build_octree, default_octree_config, octree_bounds, octree_depth, octree_leaf_count,
    octree_node_count, octree_stats, query_aabb, query_nearest_point, query_sphere,
    ray_intersect_octree, refit_octree, AabbBounds, OctreeConfig, OctreeNode, OctreeQuery,
    OctreeStats, RayHit as OctreeRayHit,
};

pub mod mesh_apex_vertex;
pub use mesh_apex_vertex::{
    apex_angle, apex_count, apex_frequency, apex_result_to_json, dist_sq, find_apex_vertices,
    is_apex_vertex, triangle_apex, vertex_angle, ApexResult,
};

pub mod mesh_border_edge;
pub use mesh_border_edge::{
    border_analysis, border_edge_count as border_edge_count_be, border_result_to_json,
    border_total_length, border_vertex_count, build_edge_count_map, detect_border_edges,
    is_border_vertex, BorderResult, DirectedEdge,
};

pub mod mesh_centroid_decompose;
pub use mesh_centroid_decompose::{
    all_face_centroids, centroid_decompose, decomp_region_count, decomp_result_to_json,
    decomp_total_faces, default_centroid_decomp_config, face_centroid as decomp_face_centroid,
    CentroidDecompConfig, CentroidDecompResult, DecompRegion,
};

pub mod mesh_collapse_edge;
pub use mesh_collapse_edge::{
    collapse_n_edges, collapse_result_to_json_v2, collapse_single_edge, edge_length_v2,
    edge_midpoint_v2, find_shortest_edge, unique_edge_count, CollapseEdgeV2, CollapseResultV2,
};

pub mod mesh_cusp_detect;
pub use mesh_cusp_detect::{
    angle_between, compute_face_normals as cusp_compute_face_normals, cusp_count,
    cusp_result_to_json, default_cusp_threshold, detect_cusps, face_normal_raw, is_cusp,
    normalize3 as cusp_normalize3, CuspDetectResult, CuspVertex,
};

pub mod mesh_dihedral_angle;
pub use mesh_dihedral_angle::{
    compute_dihedral_angles, dihedral_angle_from_normals, dihedral_edge_count,
    dihedral_result_to_json, face_normal as dihedral_face_normal,
    sharp_edges_by_angle as dihedral_sharp_edges, DihedralEdge, DihedralResult,
};

pub mod mesh_disk_map;
pub use mesh_disk_map::{
    default_disk_map_config, disk_map, disk_map_to_json, disk_map_vertex_count, is_in_unit_disk,
    map_boundary_to_circle, uv_triangle_area as disk_uv_triangle_area, DiskMapConfig,
    DiskMapResult,
};

pub mod mesh_edge_hash;
pub use mesh_edge_hash::{
    boundary_edges as hash_boundary_edges, build_edge_hash_map, edge_count as edge_hash_count,
    edge_exists, edge_hash_to_json, faces_for_edge, make_edge_key, non_manifold_edges,
    unique_vertices, EdgeHashMap, EdgeKey as HashEdgeKey,
};

pub mod mesh_face_centroid;
pub use mesh_face_centroid::{
    centroid_bounds as fc_centroid_bounds, centroid_distance, compute_all_face_centroids,
    face_centroid_count, face_centroid_to_json, get_face_centroid, mean_centroid,
    nearest_face_centroid, triangle_centroid, FaceCentroidData,
};

pub mod mesh_edge_valence;
pub use mesh_edge_valence::{
    avg_valence as valence_avg, boundary_edge_count as valence_boundary_edge_count,
    compute_edge_valences, edge_valence, edge_valence_to_json, manifold_edge_count,
    max_valence as valence_max, non_manifold_edge_count, total_edge_count, EdgeValenceResult,
};

pub mod mesh_face_ring;
pub use mesh_face_ring::{
    all_face_rings, avg_ring_size, face_ring_for_vertex, face_ring_to_json, max_ring_size,
    ring_contains_face, ring_face_count, vertices_with_ring_size, FaceRing,
};

pub mod mesh_fan_mesh;
pub use mesh_fan_mesh::{
    fan_area, fan_from_boundary, fan_mesh_to_json, fan_triangle_count, fan_vertex_count,
    generate_fan, FanMesh,
};

pub mod mesh_geodesic_voronoi;
pub use mesh_geodesic_voronoi::{
    build_adjacency as voronoi_build_adjacency, compute_geodesic_voronoi, largest_cell_size,
    vertex_label, voronoi_cell_count, voronoi_result_to_json, GeoVoronoiCell, GeoVoronoiResult,
};

pub mod mesh_hex_mesh;
pub use mesh_hex_mesh::{
    generate_hex_mesh, hex_cell_count, hex_diagonal_factor, hex_mesh_area, hex_mesh_to_json,
    hex_triangle_count, hex_vertex_count, HexMesh,
};

pub mod mesh_index_remap;
pub use mesh_index_remap::{
    apply_remap, compact_remap, indices_in_bounds, max_index, remap_result_to_json,
    reverse_winding as remap_reverse_winding, unused_index_count, used_index_count, RemapResult,
};

pub mod mesh_vertex_degree;
pub use mesh_vertex_degree::{
    avg_degree, compute_vertex_degrees, degree_result_to_json,
    irregular_vertex_count as degree_irregular_count, max_degree, min_nonzero_degree,
    vertex_degree, vertices_with_degree, VertexDegreeResult,
};

pub mod mesh_angle_bisector;
pub use mesh_angle_bisector::{
    angle_bisector, bisector_count, bisector_result_to_json, compute_bisectors,
    dot3 as bisector_dot3, is_valid_bisector, normalize3 as bisector_normalize3, vec_length,
    vertex_angle as bisector_vertex_angle, AngleBisectorResult,
};

pub mod mesh_aspect_ratio;
pub use mesh_aspect_ratio::{
    aspect_ratio_to_json, compute_aspect_ratios, count_poor_triangles,
    edge_length as aspect_edge_length, min_aspect_ratio, ratio_at,
    triangle_area as aspect_triangle_area, triangle_aspect_ratio, AspectRatioResult,
};

pub mod mesh_bary_coord;
pub use mesh_bary_coord::{
    bary_distance_to_center, bary_interpolate, bary_interpolate3, bary_to_json, clamp_bary,
    compute_bary, dot3 as bary_dot3, is_inside_triangle, BaryCoord,
};

pub mod mesh_bbox_tree;
pub use mesh_bbox_tree::{
    bbox_overlap, bbox_tree_to_json, bbox_volume, build_bbox_tree, merge_bbox,
    node_count as bbox_node_count, point_in_bbox, triangle_bbox, BboxNode, BboxTree,
};

pub mod mesh_bilinear_patch;
pub use mesh_bilinear_patch::{
    evaluate_patch as evaluate_bilinear_patch, new_bilinear_patch, patch_area_approx,
    patch_center as bilinear_patch_center, patch_du, patch_dv,
    patch_to_json as bilinear_patch_to_json, tessellate_patch as tessellate_bilinear,
    BilinearPatch,
};

pub mod mesh_boundary_vertex;
pub use mesh_boundary_vertex::{
    boundary_count, boundary_fraction, boundary_vertex_to_json,
    build_edge_map as boundary_build_edge_map, detect_boundary_vertices,
    find_boundary_edges as boundary_find_edges, is_boundary_vertex, BoundaryVertexResult,
};

pub mod mesh_catmull_clark_weight;
pub use mesh_catmull_clark_weight::{
    boundary_edge_weight, centroid as cc_centroid, face_point_weight, is_regular_valence,
    lerp3 as cc_lerp3, loop_beta, smoothing_factor, vertex_weight as cc_vertex_weight,
    warren_weight, weights_to_json as cc_weights_to_json,
};

pub mod mesh_circumcenter;
pub use mesh_circumcenter::{
    avg_circumradius, circumcenter_to_json, compute_circumcenters, is_acute_triangle,
    max_circumradius, triangle_circumcenter, Circumcenter,
};

pub mod mesh_color_attr;
pub use mesh_color_attr::{
    apply_gamma, average_color, clamp_colors, color_attr_to_json, color_vertex_count,
    get_color as attr_get_color, lerp_color as attr_lerp_color, new_color_attr,
    set_color as attr_set_color, ColorAttr,
};

pub mod mesh_conformal_map;
pub use mesh_conformal_map::{
    average_distortion, conformal_energy, conformal_map_to_json,
    cotangent_weight as conformal_cotangent_weight, is_in_unit_square,
    map_boundary_to_circle as conformal_map_boundary, normalize_uvs as conformal_normalize_uvs,
    ConformalMapResult,
};

pub mod mesh_convex_face;
pub use mesh_convex_face::{
    analyze_triangle_convexity, convex_count, convex_face_to_json, cross3 as convex_cross3,
    dot3 as convex_dot3, face_area as convex_face_area, face_normal as convex_face_normal,
    is_quad_convex, is_triangle_convex, ConvexFaceResult,
};

pub mod mesh_curvature_discrete;
pub use mesh_curvature_discrete::{
    avg_curvature as discrete_avg_curvature, compute_discrete_curvature,
    curvature_to_json as discrete_curvature_to_json,
    gaussian_curvature as discrete_gaussian_curvature, max_abs_curvature, mean_curvature_simple,
    vertex_angle as discrete_vertex_angle, DiscreteCurvature,
};

pub mod mesh_dart_graph;
pub use mesh_dart_graph::{
    boundary_dart_count, build_dart_graph, dart_count, dart_graph_to_json,
    face_count as dart_face_count, get_dart, has_twin, Dart, DartGraph,
};

pub mod mesh_edge_bisect;
pub use mesh_edge_bisect::{
    bisect_all_edges, bisect_to_json, bisect_triangle_count, bisect_vertex_count,
    midpoint as bisect_midpoint, new_vertex_count as bisect_new_vertex_count, EdgeBisectResult,
};

pub mod mesh_edge_contract;
pub use mesh_edge_contract::{
    contract_edge, contract_n_edges, contract_to_json, edge_length as contract_edge_length,
    find_shortest_edge as contract_find_shortest, triangle_count as contract_triangle_count,
    EdgeContractResult,
};

pub mod mesh_face_dual;
pub use mesh_face_dual::{
    avg_dual_edge_length as face_dual_avg_edge_length, build_face_dual, dual_degree,
    dual_edge_count as face_dual_edge_count, dual_vertex_count as face_dual_vertex_count,
    face_dual_to_json, triangle_centroid as dual_triangle_centroid, FaceDual,
};

pub mod mesh_area_gradient;
pub use mesh_area_gradient::{
    area_gradient_to_json, avg_face_area, compute_area_gradients, cross3 as area_grad_cross3,
    get_gradient, gradient_face_count, max_face_area, min_face_area,
    safe_normalize as area_grad_safe_normalize, total_area as area_grad_total_area,
    triangle_area as area_grad_triangle_area, AreaGradientResult,
};

pub mod mesh_bend_deform;
pub use mesh_bend_deform::{
    apply_bend_deform, bend_deform_to_json, bend_param, bend_vertex as bend_deform_vertex,
    bend_vertex_count, bend_within_tolerance, default_bend_config, deg_to_rad as bend_deg_to_rad,
    rad_to_deg as bend_rad_to_deg, BendDeformConfig, BendDeformResult,
};

pub mod mesh_cell_partition;
pub use mesh_cell_partition::{
    avg_occupancy, build_cell_partition, cell_partition_to_json, max_cell_occupancy,
    occupied_cells, position_to_cell, total_cells, vertices_in_cell, CellPartition,
};
