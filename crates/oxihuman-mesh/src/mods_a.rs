pub mod groups;
pub use groups::{VertexGroup, VertexGroupMap};
pub mod bounds;
pub mod clothing;
pub mod integrity;
pub mod lod;
pub mod measurements;
pub mod mesh;
pub mod normals;
pub mod pose_library;
pub mod skeleton;
pub mod skinning;
pub mod smooth;
pub mod stats;
pub mod suit;
pub mod uvgen;
pub mod weld;
pub use uvgen::{
    flip_v, normalize_uvs, offset_uvs, project_uvs, rotate_uvs, tile_uvs, UvProjection,
};
pub mod decimate;
pub use decimate::{decimate, decimate_ratio, decimate_with_info, DecimateResult};
pub mod catmull_clark;
pub mod subdivide;
pub use catmull_clark::{
    catmull_clark_subdivide, catmull_clark_subdivide_n, catmull_clark_with_config,
    CatmullClarkConfig,
};
pub mod retarget;
pub use retarget::{retarget_pose, retarget_sequence, RetargetMap};
pub mod atlas;
pub mod shapes;
pub mod vpaint;
pub use shapes::{capsule, cone, cylinder, quad, sphere};
pub use vpaint::{build_adjacency, WeightMap};
pub mod convex_hull;
pub use convex_hull::{convex_hull, mesh_convex_hull, point_in_hull, ConvexHull};
pub mod octree;
pub use atlas::{mesh_to_island, pack_atlas, AtlasConfig, AtlasPlacement, AtlasResult, UvIsland};
pub use octree::{Octree, OctreeAabb};
pub use subdivide::{loop_subdivide, midpoint_subdivide};
pub mod repair;
pub use repair::{
    count_zero_length_edges, ensure_complete_triangles, fix_out_of_range_indices,
    flip_face_winding, flip_winding, has_degenerate_faces, has_valid_indices,
    remove_degenerate_faces, remove_duplicate_faces, repair_mesh, MeshRepairReport,
};

pub mod connectivity;
pub use connectivity::{
    connected_component_count, connectivity_stats, find_boundary_edges, find_boundary_loops,
    find_connected_components, find_non_manifold_edges, split_components, vertex_valence,
    ConnectivityStats,
};

pub mod edge_loops;
pub use edge_loops::{
    boundary_edges, edge_adjacency, edges_to_chains, edges_to_loops, extract_edge_loop,
    extract_edges, sharp_edges, uv_seam_edges, Edge,
};
pub mod ik;
pub use ik::{fabrik_solve, solve_2bone_ik, IkJoint, IkResult};
pub mod uv_quality;
pub use uv_quality::{
    compute_face_stretches, compute_uv_utilization, count_uv_overlaps, face_conformal_distortion,
    face_uv_stretch, is_degenerate_uv_face, uv_quality_report, uv_triangle_area,
    worst_stretch_faces, UvQualityReport,
};
pub mod geodesic;
pub use geodesic::{
    geodesic_all_pairs, geodesic_from_vertex, geodesic_from_vertices, geodesic_heat_map,
    mesh_diameter, GeodesicResult,
};
pub mod visibility;
pub use visibility::{
    backface_cull, classify_aabb, count_front_facing, is_front_facing, Frustum, Plane, Visibility,
};

pub use bounds::{compute_bounds, Aabb, BoundingSphere, BoundsResult, Obb};
pub use clothing::{apply_clothing, ClothingMesh};
pub use integrity::{check_index_bounds, check_integrity, check_positions_finite, IntegrityReport};
pub use measurements::{compute_aabb, compute_measurements, Aabb as MeasAabb, BodyMeasurements};
pub use mesh::MeshBuffers;
pub use normals::{compute_normals, compute_tangents};
pub use pose_library::{Pose, PoseLibrary, IDENTITY_QUAT};
pub use skeleton::{Joint, Skeleton};
pub use skinning::{apply_lbs, bind_pose_matrices, SkinWeights};
pub use smooth::{laplacian_smooth, smooth_normals, taubin_smooth, SmoothConfig};
pub use stats::{
    compute_stats, edge_lengths, face_areas, surface_area, volume_estimate, MeshStats,
};
pub use suit::ensure_suit_mesh;
pub use weld::{
    deduplicate_faces, remove_unused_vertices, weld_by_position, weld_by_position_and_uv,
    WeldResult,
};


pub mod sampling;
pub use sampling::{
    face_area, sample_one_per_face, sample_poisson_disk, sample_surface, total_surface_area, Lcg,
    SurfacePoint,
};
pub mod curvature;
pub use curvature::{
    compute_curvature, compute_gaussian_curvature, compute_mean_curvature, curvature_stats,
    find_curvature_peaks, find_feature_vertices, find_saddle_points, CurvatureStats,
    VertexCurvature,
};
pub mod ao_bake;
pub use ao_bake::{
    ao_to_vertex_colors, bake_vertex_ao, fast_vertex_ao, hemisphere_samples, ray_hits_mesh,
    ray_triangle_intersect, tangent_to_world, AoBakeConfig,
};
pub mod bvh;
pub use bvh::{Bvh, BvhAabb, RayHit};
pub mod dqs;
pub use dqs::{apply_dqs, joint_delta_dq, matrix_to_dual_quat, DualQuat, Quat};
pub mod seam_cut;
pub use seam_cut::{
    count_uv_islands, cut_uv_seams, face_uv_bounds, find_uv_seam_edges_detailed, has_uv_seams,
    split_uv_islands, SeamCutResult,
};
pub mod marching_cubes;
pub use marching_cubes::{marching_cubes, marching_cubes_welded, ScalarField};
pub mod winding;
pub use winding::{
    classify_points, is_inside, mesh_surface_area, triangle_solid_angle, winding_number,
    winding_numbers_batch, winding_sign, WINDING_THRESHOLD,
};
pub mod remesh;
pub use remesh::{
    collapse_short_edges, compute_mean_edge_length, compute_target_edge_length,
    flip_edges_for_valence, remesh, smooth_vertices, split_long_edges, RemeshParams, RemeshResult,
};

pub mod voxelize;
pub use voxelize::{
    mesh_bounds, voxel_to_mesh, voxelize, voxelize_solid, voxelize_surface, VoxelGrid,
    VoxelizeParams,
};

pub mod heat_map;
pub use heat_map::{
    color3_to_u8, lerp_color, sample_ramp, scalars_to_colors, scalars_to_colors_range, Color3,
    Color4, ColorRamp, HeatMap,
};

pub mod ffd;
pub use ffd::{apply_ffd, bernstein, binomial, make_bend_lattice, make_twist_lattice, FfdLattice};

pub mod mesh_diff;
pub use mesh_diff::{
    blend_meshes, compute_displacement, displacement_to_heat_mesh, interpolate_mesh_sequence,
    mesh_diff_stats, meshes_approx_equal, DisplacementField, MeshDiffStats,
};

pub mod normal_map_bake;
pub use normal_map_bake::{
    bake_normal_map, closest_surface_point, normal_to_rgb, rasterize_uv_triangle, rgb_to_normal,
    NormalMapBakeParams, NormalMapTexture,
};

pub mod terrain;
pub use terrain::{
    compute_slope, generate_dome_terrain, generate_grid, generate_sine_terrain,
    mesh_to_heightfield, smooth_heightfield, terrain_from_heightfield, HeightField, TerrainParams,
};

pub mod spring_deform;
pub use spring_deform::{
    build_edge_springs, find_boundary_vertices, jiggle_deform, SpringParams, SpringSystem,
};

pub mod mesh_merge;
pub use mesh_merge::{
    append_mesh, extract_face_range, filter_faces, merge_many, merge_two, merge_with_params,
    rotate_mesh, scale_mesh, split_by_connectivity, translate_mesh, MergeParams, MergeResult,
};

pub mod microdisp;
pub use microdisp::{
    apply_micro_displacement, fbm_noise_3d, sample_displacement, skin_displacement, value_noise_3d,
    voronoi_3d, wrinkle_displacement, DisplacementPattern, MicroDispParams, MicroDispResult,
};

pub mod thickness;
pub use thickness::{
    compute_thickness, cone_samples, ray_mesh_hits, ray_triangle_hit, sample_thickness_at,
    ThicknessMap, ThicknessParams,
};

pub mod proxy_gen;
pub use proxy_gen::{
    capsule_mesh, fit_aabb, fit_capsule, fit_obb, fit_sphere, mesh_proxy, proxy_to_mesh,
    region_proxy, sphere_mesh, ProxyFitResult, ProxyShape, ProxyShapeType,
};

pub mod ray_pick;
pub use ray_pick::{
    box_select_vertices, closest_ray_point, pick_all_faces, pick_face, pick_vertex,
    point_to_ray_distance, project_onto_ray, sphere_select_vertices, PickParams, PickResult, Ray,
};

pub mod hair_cards;
pub use hair_cards::{
    curled_hair_card, guides_from_mesh, hair_card_from_guide, hair_cards_from_guides,
    straight_hair_card, CardNormalMode, HairCardParams, HairGuide,
};

pub mod displacement_map;
pub use displacement_map::{
    apply_displacement_map, apply_displacement_masked, mesh_to_displacement_map,
    DisplacedMeshResult, DisplacementApplyParams, DisplacementMap2D,
};

pub mod cloth_panel;
pub use cloth_panel::{
    circular_panel, join_panels, layout_panels_flat, rectangular_panel, sleeve_panel,
    total_panel_area, trapezoid_panel, triangle_panel, tshirt_panels, ClothPanel, PanelParams,
};

pub mod cloth_sim;
pub use cloth_sim::{
    apply_sim_to_mesh, build_cloth_grid, simulate_n_steps, ClothParticle, ClothSim, ClothSimParams,
    ClothSimResult, DistanceConstraint,
};

pub mod mesh_paint;
pub use mesh_paint::{
    apply_brush, brush_falloff_weight, brush_flatten, brush_grab, brush_inflate, brush_pinch,
    brush_smooth, build_adjacency as mesh_paint_build_adjacency, BrushParams, BrushStroke,
    BrushStrokeResult, BrushType,
};

pub mod mesh_slice;
pub use mesh_slice::{
    circumference_at_height, edge_plane_intersect, horizontal_slice, ray_plane_intersect,
    slice_mesh, split_mesh, width_profile, CrossSection, SlicePlane, SliceResult,
};

pub mod mesh_hollow;
pub use mesh_hollow::{
    area_weighted_normals, boundary_loops, hollow_mesh, offset_mesh, shell_thickness,
    stitch_boundary_loops, HollowParams, HollowResult,
};

pub mod mesh_warp;
pub use mesh_warp::{
    displacement_field, dist3, simple_warp, warp_mesh, RbfKernel, RbfWarp, RbfWarpConfig,
    WarpHandle,
};

pub mod mesh_label;
pub use mesh_label::{
    body_seed_vertices, flood_fill_label, label_by_height, propagate_labels, region_boundary_edges,
    BodyRegion, MeshLabels,
};

pub mod mesh_patch;
pub use mesh_patch::{
    ear_clip, fan_patch, fill_hole, fill_holes, find_holes, hole_count, is_ear, is_watertight,
    polygon_signed_area_2d, project_polygon_2d, MeshHole, PatchResult, PatchStrategy,
};

pub mod mesh_mirror;
pub use mesh_mirror::{
    extract_half, find_symmetry_pairs, flip_normals_axis, flip_positions, mirror_copy, mirror_mesh,
    reverse_winding, symmetrize_mesh, symmetry_error, MirrorAxis, MirrorConfig, MirrorResult,
};

pub mod mesh_feature;
pub use mesh_feature::{
    build_edge_face_map, chain_edges, dihedral_angle, extract_all_features,
    extract_boundary_edges_fl, extract_sharp_edges, extract_silhouette,
    face_normal as feature_face_normal, FeatureEdge, FeatureLines, FeatureType,
};

pub mod mesh_normal_delta;
pub use mesh_normal_delta::{
    apply_morph_normals, compute_batch_normal_deltas, compute_normal_deltas,
    compute_vertex_normals as compute_vertex_normals_v, normals_approx_equal, safe_normalize,
    to_tangent_space_delta, MorphNormalDeltas, NormalDelta,
};

pub mod mesh_boolean;
pub use mesh_boolean::{
    boolean_op, classify_vertices, combine_meshes as boolean_combine,
    filter_faces_by_classification, flip_winding as boolean_flip_winding, BooleanOp, BooleanResult,
};

pub mod mesh_crease;
pub use mesh_crease::{
    apply_crease_to_subdivision_config, auto_crease_by_angle, crease_stats, mark_boundary_edges,
    merge_crease_maps, CreaseConfig, CreaseEdge, CreaseMap, CreaseStats, CreaseSubdivData, EdgeKey,
};

pub mod mesh_decal;
pub use mesh_decal::{
    affected_faces as decal_affected_faces, apply_decal_colors, decal_falloff_weight, decal_stats,
    project_decal, standard_decal, DecalBounds, DecalConfig, DecalFalloff, DecalResult, DecalStats,
    DecalVertex,
};

pub mod mesh_bridge;
pub use mesh_bridge::{
    align_loops, bridge_loops, loop_centroid, loop_from_boundary, open_cylinder, BridgeConfig,
    BridgeInterpolation, BridgeResult, EdgeLoop,
};

pub mod mesh_offset;
pub use mesh_offset::{
    closest_point_on_triangle, grow_mesh, offset_mesh as normal_offset_mesh, offset_mesh_variable,
    shell_offset, shrink_mesh, shrink_wrap, OffsetParams, OffsetResult,
};

pub mod mesh_uv_pack;
pub use mesh_uv_pack::{
    pack_from_mesh, pack_stats, pack_uv_rects, transform_island_uvs, uv_rect_bounds, PackConfig,
    PackResult, PackSort, UvRect,
};

pub mod mesh_sdf;
pub use mesh_sdf::{
    box_sdf, compute_sdf, sample_sdf, sdf_intersection, sdf_smooth_union, sdf_stats,
    sdf_subtraction, sdf_to_mesh, sdf_union, sphere_sdf, SdfGrid, SdfParams, SdfStats,
};

pub mod mesh_clip;
pub use mesh_clip::{
    clip_above_y, clip_below_y, clip_mesh, clip_to_box, ClipConfig, ClipPlane, ClipResult,
};

pub mod adaptive_subdivide;
pub mod feature_decimation;
pub mod mesh_repair_advanced;
pub use adaptive_subdivide::{
    adaptive_subdivide, build_face_adjacency, dihedral_angle as adaptive_dihedral_angle,
    face_max_dihedral_angle, face_normal as adaptive_face_normal, loop_subdivide_marked,
    AdaptiveSubdivideConfig, AdaptiveSubdivideResult,
};
pub use feature_decimation::{
    collapse_edge, compute_edge_dihedral, count_valid_triangles, edge_collapse_error,
    feature_decimate, mark_feature_edges, FeatureDecimateConfig, FeatureDecimateResult,
};
pub use mesh_repair_advanced::{
    fill_hole_fan, fill_hole_smooth, find_boundary_holes, find_t_junctions, is_manifold_mesh,
    remove_t_junction, repair_mesh_advanced, AdvancedRepairConfig, AdvancedRepairReport,
};

pub mod mesh_arap;
pub mod mesh_cage;
pub mod mesh_flow;
pub use mesh_arap::{
    arap_deform, arap_energy, arap_global_step, arap_local_step, build_arap_laplacian,
    mat3_mul_vec, nearest_rotation_3x3, ArapConfig, ArapHandle, ArapResult, ArapWeight,
};
pub use mesh_cage::{
    apply_cage_weights, cage_encloses_point, mean_value_coordinates, validate_cage_weights,
    CageDeformResult, CageDeformer,
};
pub use mesh_flow::{
    build_adjacency_list, flow_mesh, laplacian_step, mean_curvature_step,
    mesh_volume_from_positions, rescale_to_volume, taubin_step, vertex_laplacian, FlowConfig,
    FlowMethod, FlowResult,
};

pub mod mesh_lscm;
pub mod mesh_skin_weights;
pub mod mesh_tet;
pub use mesh_lscm::{
    compute_local_frame, lscm_parameterize, normalize_uvs_lscm, project_to_uv,
    triangle_conformal_energy, uv_area, uv_stretch_metric, LscmConfig, LscmResult,
};
pub use mesh_skin_weights::{
    bone_heat, compute_auto_skin_weights, diffuse_weights, dominant_bone,
    max_influences_per_vertex, normalize_skin_weights, prune_small_weights, skin_weights_to_json,
    AutoSkinResult, BoneEndpoint, SkinWeightConfig, WeightFalloff,
};
pub use mesh_tet::{
    tet_centroid, tet_dihedral_angles, tet_mesh_volume, tet_volume, tetrahedralize,
    validate_tet_mesh, TetGenConfig, TetGenResult, TetMesh,
};

pub mod mesh_noise_gen;
pub mod mesh_pca;
pub mod mesh_progressive;
pub use mesh_noise_gen::{
    apply_noise_texture, displace_mesh_noise, fbm_noise, generate_organic_surface,
    generate_sphere_bumps, lcg_value_noise, noise_magnitude_stats, NoiseDisplaceResult,
    NoiseGenConfig,
};
pub use mesh_pca::{
    compute_shape_pca, explained_variance_ratio, flat_to_shape, mean_shape,
    pca_reconstruction_error, project_shape, reconstruct_shape, shape_to_flat, PcaConfig,
    PcaResult, ShapePca,
};
pub use mesh_progressive::{
    build_progressive_mesh, collapse_error_sequence, extract_lod_level, extract_lod_ratio,
    progressive_lod_levels, refine_lod, CollapseRecord, ProgressiveMesh, ProgressiveMeshConfig,
};

pub mod mesh_remesh_isotropic;
pub use mesh_remesh_isotropic::{
    average_edge_length, collapse_short_edges as isotropic_collapse_short_edges, edge_length_stats,
    flip_edges_for_valence as isotropic_flip_edges_for_valence, isotropic_remesh,
    split_long_edges as isotropic_split_long_edges, tangential_relaxation,
};
