// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Mesh processing, topology, and geometry algorithms for OxiHuman.
//!
//! This crate sits between the morph engine and the export pipeline. It takes
//! raw [`oxihuman_morph::engine::MeshBuffers`] output and enriches it with
//! recomputed normals, tangents, optional vertex colors, UV atlasing, LOD
//! decimation, Catmull-Clark subdivision, cloth/skeleton skinning, geodesic
//! distances, and a suite of topology repair routines.
//!
//! # Key types
//!
//! - [`MeshBuffers`] — the canonical mesh representation used by exporters.
//! - [`Skeleton`] / [`Joint`] — rig hierarchy for linear blend skinning.
//! - [`SkinWeights`] — per-vertex bone weights.
//! - [`BodyMeasurements`] — computed anthropometric measurements from a mesh.
//!
//! # Example: build and repair a mesh
//!
//! ```rust
//! use oxihuman_mesh::mesh::MeshBuffers;
//! use oxihuman_mesh::repair::repair_mesh;
//! use oxihuman_morph::engine::MeshBuffers as MorphBuffers;
//!
//! let morph_out = MorphBuffers {
//!     positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
//!     normals:   vec![[0.0, 0.0, 1.0]; 3],
//!     uvs:       vec![[0.0, 0.0]; 3],
//!     indices:   vec![0, 1, 2],
//!     has_suit:  false,
//! };
//! let mut mesh = MeshBuffers::from_morph(morph_out);
//! let report = repair_mesh(&mut mesh);
//! assert!(report.degenerate_faces_removed == 0 || report.duplicate_faces_removed == 0);
//! ```

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

/// Set a uniform RGBA color on every vertex of the mesh.
///
/// After calling this, `mesh.colors` will be `Some(vec![rgba; n_verts])`.
/// Existing color data (if any) is replaced.
pub fn set_uniform_color(mesh: &mut MeshBuffers, rgba: [f32; 4]) {
    let n = mesh.positions.len();
    mesh.colors = Some(vec![rgba; n]);
}

#[cfg(test)]
mod colors_tests {
    use super::*;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn three_vert_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            has_suit: false,
        })
    }

    #[test]
    fn set_uniform_color_sets_all_verts() {
        let mut mesh = three_vert_mesh();
        set_uniform_color(&mut mesh, [1.0, 0.5, 0.0, 1.0]);
        let colors = mesh.colors.as_ref().unwrap();
        assert_eq!(colors.len(), mesh.positions.len());
    }

    #[test]
    fn color_rgba_values_correct() {
        let mut mesh = three_vert_mesh();
        let rgba = [0.2f32, 0.4, 0.6, 0.8];
        set_uniform_color(&mut mesh, rgba);
        let colors = mesh.colors.as_ref().unwrap();
        for c in colors {
            assert!((c[0] - 0.2).abs() < 1e-6, "R mismatch");
            assert!((c[1] - 0.4).abs() < 1e-6, "G mismatch");
            assert!((c[2] - 0.6).abs() < 1e-6, "B mismatch");
            assert!((c[3] - 0.8).abs() < 1e-6, "A mismatch");
        }
    }

    #[test]
    fn no_color_mesh_colors_none() {
        let mesh = three_vert_mesh();
        assert!(
            mesh.colors.is_none(),
            "from_morph should produce colors=None"
        );
    }
}

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
