// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
//! Data structures and algorithms: tree/trie/heap variants, graph algorithms,
//! spatial structures, codec/compression/encryption, networking stubs,
//! and text-processing utilities.

#[path = "tree_index.rs"]
pub mod tree_index;
pub use tree_index::{
    new_tree_index, ti_add_child, ti_add_root, ti_children, ti_count, ti_depth, ti_descendants,
    ti_label, ti_parent, ti_roots, TreeIndex, TreeNode,
};

#[path = "trie_map.rs"]
pub mod trie_map;
pub use trie_map::{
    new_trie_map, trm_contains, trm_get, trm_insert, trm_is_empty, trm_keys_with_prefix, trm_len,
    trm_remove, TrieMap,
};

#[path = "type_alias_map.rs"]
pub mod type_alias_map;
pub use type_alias_map::{
    new_type_alias_map, tam_aliases_for, tam_all_aliases, tam_clear, tam_count, tam_is_alias,
    tam_register, tam_remove, tam_resolve, tam_resolve_chain, TypeAliasMap,
};

#[path = "type_cache.rs"]
pub mod type_cache;
pub use type_cache::{
    new_type_cache, tc_clear, tc_contains, tc_get, tc_is_empty, tc_len, tc_remove, tc_store,
    tc_total_bytes, tc_version, TypeCache, TypeCacheEntry,
};

#[path = "type_erased.rs"]
pub mod type_erased;
pub use type_erased::{
    new_type_erased, te_clear, te_contains, te_get, te_insert, te_is_empty, te_keys,
    te_keys_by_type, te_len, te_remove, te_type_tag, ErasedSlot, TypeErased,
};

#[path = "uid_gen.rs"]
pub mod uid_gen;
pub use uid_gen::{
    new_uid_gen, ug_alloc, ug_allocated_count, ug_local_id, ug_namespace, ug_peek_next, ug_recycle,
    ug_recycled_count, ug_reset, UidGen,
};

#[path = "union_find_v2.rs"]
pub mod union_find_v2;
pub use union_find_v2::{
    new_union_find, uf_component_count, uf_component_size, uf_connected, uf_element_count, uf_find,
    uf_reset, uf_union, UnionFindV2,
};

#[path = "update_queue.rs"]
pub mod update_queue;
pub use update_queue::{
    new_update_queue, uq_clear, uq_dequeue, uq_drain_all, uq_enqueue, uq_is_empty, uq_len, uq_peek,
    uq_total_enqueued, UpdateItem, UpdateQueue,
};

#[path = "value_cache.rs"]
pub mod value_cache;
pub use value_cache::{
    new_value_cache, vc_clear, vc_dirty_count, vc_get, vc_invalidate, vc_invalidate_all,
    vc_is_valid, vc_len, vc_remove, vc_store, ValueCache, ValueEntry,
};

#[path = "value_map.rs"]
pub mod value_map;
pub use value_map::{
    new_value_map, vm_clear, vm_contains, vm_get, vm_get_bool, vm_get_float, vm_get_int, vm_len,
    vm_remove, vm_set_bool, vm_set_float, vm_set_int, vm_set_text, MapVal, ValueMap,
};

#[path = "var_store.rs"]
pub mod var_store;
pub use var_store::{
    new_var_store, vs_changed_names, vs_clear, vs_declare, vs_flush, vs_get, vs_is_changed, vs_len,
    vs_remove, vs_reset, vs_set, VarEntry, VarStore,
};

#[path = "quad_tree.rs"]
pub mod quad_tree;
pub use quad_tree::{
    new_quad_tree, qt_clear, qt_count, qt_insert, qt_is_empty, qt_query_circle, qt_query_rect,
    Aabb2, QtPoint, QuadTree,
};

#[path = "color_palette.rs"]
pub mod color_palette;
pub use color_palette::{
    add_color, blend_palette_colors, get_color, new_color_palette, palette_size, ColorEntry,
    ColorPalette,
};

#[path = "bitmask_ops.rs"]
pub mod bitmask_ops;
pub use bitmask_ops::{
    clear_bit, count_leading_zeros, count_trailing_zeros, extract_range, highest_bit, is_bit_set,
    lowest_bit, parity, popcount, range_mask, rotate_left, rotate_right, set_bit, set_bit_indices,
    toggle_bit,
};

#[path = "version_vector.rs"]
pub mod version_vector;
pub use version_vector::{
    new_version_vector, vv_compare, vv_concurrent, vv_get, vv_happens_before, vv_increment,
    vv_merge, vv_node_count, vv_nodes, vv_reset_node, VersionVector,
};

#[path = "text_tokenizer.rs"]
pub mod text_tokenizer;
pub use text_tokenizer::{
    is_numeric_token, token_count, token_numbers, token_words, tokenize as text_tokenize,
    Token as TextToken,
};

#[path = "rolling_stats.rs"]
pub mod rolling_stats;
pub use rolling_stats::{
    new_rolling_stats, rs_clear, rs_count, rs_is_full, rs_max, rs_mean, rs_median, rs_min, rs_push,
    rs_std, rs_sum, rs_variance, RollingStats,
};

#[path = "bloom_filter.rs"]
pub mod bloom_filter;
pub use bloom_filter::{
    bloom_contains, bloom_fill_ratio, bloom_insert, bloom_reset, new_bloom_filter, BloomFilter,
};

#[path = "escape_hatch.rs"]
pub mod escape_hatch;
pub use escape_hatch::{
    html_escape, html_unescape, json_string_escape, json_string_unescape, url_escape,
};

#[path = "number_format.rs"]
pub mod number_format;
pub use number_format::{
    format_bytes, format_float, format_float_sep, format_int_sep, format_percent, format_si,
    pad_left, pad_right,
};

#[path = "delta_encoder_v2.rs"]
pub mod delta_encoder_v2;
pub use delta_encoder_v2::{
    avg_delta, delta_decode, delta_decode_u32, delta_encode, delta_encode_u32, max_delta,
    zigzag_decode, zigzag_encode,
};

#[path = "run_length.rs"]
pub mod run_length;
pub use run_length::{
    rle_compression_ratio_v2, rle_decode, rle_decoded_len, rle_encode, rle_is_uniform, rle_merge,
    rle_most_frequent, rle_run_count, rle_verify_roundtrip, Run,
};

#[path = "checksum_crc.rs"]
pub mod checksum_crc;
pub use checksum_crc::{
    additive_checksum, crc16, crc16_verify, crc8, crc8_verify, fletcher16, xor_checksum,
};

#[path = "argument_parser.rs"]
pub mod argument_parser;
pub use argument_parser::{
    arg_get, arg_get_f64, arg_get_i64, arg_get_or, arg_has_flag, arg_keys, arg_positional_count,
    new_parsed_args, parse_args, parse_args_str, ParsedArgs,
};

#[path = "pipeline_stage.rs"]
pub mod pipeline_stage;
pub use pipeline_stage::{
    execute_stage, new_pipeline_stage, stage_dependencies, stage_is_complete, stage_name_ps,
    stage_reset, stage_result, stage_to_json, PipelineStage as PsStage,
    StageResult as PsStageResult,
};

#[path = "fsm_builder.rs"]
pub mod fsm_builder;
pub use fsm_builder::{
    add_state, add_transition, build_fsm, has_state, has_transition, new_fsm_builder, state_count,
    transition_count, BuiltFsm, FsmBuilder, FsmTransition,
};

#[path = "decision_tree.rs"]
pub mod decision_tree;
pub use decision_tree::{
    dt_add_branch, dt_add_leaf, dt_all_actions, dt_branch_count, dt_clear, dt_evaluate,
    dt_leaf_action, dt_leaf_count, dt_node_count, new_decision_tree, DecisionNode, DecisionTree,
};

#[path = "graph_coloring.rs"]
pub mod graph_coloring;
pub use graph_coloring::{
    cg_add_edge, cg_degree, cg_edge_count, cg_max_degree, cg_vertex_count, coloring_is_valid,
    coloring_num_colors, greedy_color, new_color_graph, vertices_with_color, ColorGraph,
};

#[path = "bezier_path.rs"]
pub mod bezier_path;
pub use bezier_path::{
    bezier_eval, bezier_length, bezier_split, new_bezier_path, new_cubic_bezier, path_add_segment,
    path_clear, path_eval, path_get_segment, path_length, path_segment_count, BezierPath,
    CubicBezier,
};

#[path = "poly_clip.rs"]
pub mod poly_clip;
pub use poly_clip::{
    new_polygon, point_in_polygon, polygon_area, polygon_centroid, polygon_is_empty,
    polygon_signed_area, polygon_vertex_count, sutherland_hodgman, Polygon2D,
};

#[path = "convex_hull_2d.rs"]
pub mod convex_hull_2d;
pub use convex_hull_2d::{
    convex_hull_2d, hull_area, hull_centroid, hull_contains_point, hull_diameter, hull_perimeter,
};

#[path = "matrix2.rs"]
pub mod matrix2;
pub use matrix2::{
    mat2, mat2_add, mat2_approx_eq, mat2_det, mat2_identity, mat2_inverse, mat2_mul, mat2_mul_vec,
    mat2_rotation, mat2_scale, mat2_trace, mat2_transpose, mat2_zero, Mat2,
};

#[path = "matrix3.rs"]
pub mod matrix3;
pub use matrix3::{
    mat3, mat3_add, mat3_approx_eq, mat3_det, mat3_identity, mat3_inverse, mat3_mul, mat3_mul_vec,
    mat3_outer, mat3_rot_z, mat3_scale, mat3_trace, mat3_transpose, mat3_zero, Mat3,
};

#[path = "quaternion_ops.rs"]
pub mod quaternion_ops;
pub use quaternion_ops::{
    quat, quat_approx_eq, quat_conjugate, quat_dot, quat_from_axis_angle, quat_identity,
    quat_inverse, quat_mul, quat_norm, quat_normalize, quat_rotate_vec, quat_slerp, Quat,
};

#[path = "dual_quaternion.rs"]
pub mod dual_quaternion;
pub use dual_quaternion::{
    dq_approx_eq, dq_conjugate, dq_dot, dq_from_rot_trans, dq_get_rotation, dq_get_translation,
    dq_identity, dq_mul, dq_normalize, dq_transform_point, dual_quat, DualQuat,
};

#[path = "packed_vec3.rs"]
pub mod packed_vec3;
pub use packed_vec3::{
    decode_u16_to_f32, default_packed_config, encode_f32_to_u16, new_packed_buffer, pack_vec3,
    packed_config, pvbuf_bytes, pvbuf_clear, pvbuf_get, pvbuf_is_empty, pvbuf_len, pvbuf_push,
    unpack_vec3, PackedVec3, PackedVec3Buffer, PackedVec3Config,
};

#[path = "packed_color.rs"]
pub mod packed_color;
pub use packed_color::{
    color_a, color_b, color_black, color_blend, color_g, color_lerp, color_premul_alpha, color_r,
    color_to_grayscale, color_transparent, color_white, decode_rgba_f32, decode_rgba_u8, rgba_f32,
    rgba_u8, PackedColor,
};

#[path = "bsp_tree_2d.rs"]
pub mod bsp_tree_2d;
pub use bsp_tree_2d::{
    bsp_build, bsp_collect_polygons, bsp_depth, bsp_get_root, bsp_is_leaf, bsp_line_side,
    bsp_polygon_count, bsp_set_root, bsp_split_polygon, new_bsp_tree, BspLine, BspNode, BspPolygon,
    BspTree2D,
};

#[path = "hex_grid.rs"]
pub mod hex_grid;
pub use hex_grid::{cube_round, hex_disk, hex_ring, pixel_to_hex_flat, HexCoord};

#[path = "triangular_grid.rs"]
pub mod triangular_grid;
pub use triangular_grid::{TriGrid, Triangle2D};

#[path = "noise_perlin.rs"]
pub mod noise_perlin;
pub use noise_perlin::{perlin2, perlin2_01, perlin3};

#[path = "noise_simplex.rs"]
pub mod noise_simplex;
pub use noise_simplex::{simplex2, simplex2_01, simplex2_scaled};

#[path = "noise_worley.rs"]
pub mod noise_worley;
pub use noise_worley::{worley2, worley2_01, worley2_f1f2, worley2_ridged};

#[path = "fractal_noise.rs"]
pub mod fractal_noise;
pub use fractal_noise::{
    fbm_max_amplitude, fbm_perlin2, fbm_simplex2, ridged_fbm_perlin2, turbulence_perlin2, FbmConfig,
};

#[path = "easing_curves.rs"]
pub mod easing_curves;
pub use easing_curves::{
    ease_by_name, ease_in_back, ease_in_bounce, ease_in_cubic, ease_in_elastic, ease_in_expo,
    ease_in_out_cubic, ease_linear, ease_out_back, ease_out_bounce, ease_out_cubic,
    ease_out_elastic, ease_out_expo,
};

#[path = "animation_curve.rs"]
pub mod animation_curve;
pub use animation_curve::{AnimCurve, InterpMode as CurveInterpMode, Keyframe};

#[path = "spline_catmull.rs"]
pub mod spline_catmull;
pub use spline_catmull::{catmull_rom, catmull_rom2, catmull_rom3, CatmullRomSpline2D};

#[path = "spline_hermite.rs"]
pub mod spline_hermite;
pub use spline_hermite::{
    hermite, hermite2, hermite3, hermite_basis, hermite_deriv, HermiteSpline2D,
};

#[path = "color_convert.rs"]
pub mod color_convert;
pub use color_convert::{
    hsv_to_rgb as cc_hsv_to_rgb, linear_to_srgb as cc_linear_to_srgb, luma_bt709,
    rgb_to_hsv as cc_rgb_to_hsv, rgb_to_oklch_approx, srgb_to_linear as cc_srgb_to_linear,
};

#[path = "color_gradient.rs"]
pub mod color_gradient;
pub use color_gradient::{grayscale_gradient, rainbow_gradient, ColorGradient, ColorStop};

#[path = "spatial_hash_2d.rs"]
pub mod spatial_hash_2d;
pub use spatial_hash_2d::SpatialHash2D;

#[path = "bit_matrix.rs"]
pub mod bit_matrix;
pub use bit_matrix::BitMatrix;

#[path = "convex_hull_3d.rs"]
pub mod convex_hull_3d;
pub use convex_hull_3d::{convex_hull_3d, hull_face_count, hull_volume, ConvexHull3D, HullFace};

#[path = "delaunay_2d.rs"]
pub mod delaunay_2d;
pub use delaunay_2d::{
    delaunay_2d, delaunay_tri_count, delaunay_valid, DelaunayResult, DelaunayTri,
};

#[path = "voronoi_2d.rs"]
pub mod voronoi_2d;
pub use voronoi_2d::{
    build_voronoi, cell_centroid, voronoi_assign, Point2, Voronoi2D, VoronoiCell2D,
};

#[path = "kd_tree_2d.rs"]
pub mod kd_tree_2d;
pub use kd_tree_2d::{kd2_build, kd2_nn_dist_sq, KdPoint2, KdTree2D};

#[path = "kd_tree_3d.rs"]
pub mod kd_tree_3d;
pub use kd_tree_3d::{kd3_build, kd3_nearest_id, KdPoint3, KdTree3D};

#[path = "r_tree.rs"]
pub mod r_tree;
pub use r_tree::{rtree_entry, RTree2D, RTreeEntry, Rect2};

#[path = "aabb_tree_2d.rs"]
pub mod aabb_tree_2d;
pub use aabb_tree_2d::{aabb2d_entry, Aabb2D, AabbEntry, AabbTree2D};

#[path = "segment_tree_v2.rs"]
pub mod segment_tree_v2;
pub use segment_tree_v2::{seg2_max, seg2_min, seg2_sum, SegOp, SegTreeV2};

#[path = "fenwick_tree.rs"]
pub mod fenwick_tree;
pub use fenwick_tree::{fenwick_prefix, fenwick_range, FenwickTree};

#[path = "suffix_array.rs"]
pub mod suffix_array;
pub use suffix_array::{
    build_lcp_array, build_suffix_array, lcp_max, sa_contains, sa_find_all, sa_suffix_count,
};

#[path = "rope_ds.rs"]
pub mod rope_ds;
pub use rope_ds::{rope_concat, rope_from, Rope};

#[path = "trie_v2.rs"]
pub mod trie_v2;
pub use trie_v2::TrieV2;

#[path = "skip_list_v2.rs"]
pub mod skip_list_v2;
pub use skip_list_v2::SkipList2;

#[path = "splay_tree.rs"]
pub mod splay_tree;
pub use splay_tree::SplayTree;

#[path = "avl_tree.rs"]
pub mod avl_tree;
pub use avl_tree::AvlTree;

#[path = "red_black_tree.rs"]
pub mod red_black_tree;
pub use red_black_tree::RedBlackTree;

#[path = "b_tree.rs"]
pub mod b_tree;
pub use b_tree::BTree;

#[path = "hash_map_open.rs"]
pub mod hash_map_open;
pub use hash_map_open::OpenHashMap;

#[path = "bloom_filter_v3.rs"]
pub mod bloom_filter_v3;
pub use bloom_filter_v3::BloomFilterV3;

#[path = "count_min_v2.rs"]
pub mod count_min_v2;
pub use count_min_v2::CountMinSketchV2;

#[path = "hyperloglog_v2.rs"]
pub mod hyperloglog_v2;
pub use hyperloglog_v2::HyperLogLogV2;

#[path = "cuckoo_filter.rs"]
pub mod cuckoo_filter;
pub use cuckoo_filter::CuckooFilter;

#[path = "skip_list_v3.rs"]
pub mod skip_list_v3;
pub use skip_list_v3::SkipListV3;

#[path = "b_tree_v2.rs"]
pub mod b_tree_v2;
pub use b_tree_v2::BTreeMapV2;

#[path = "red_black_map.rs"]
pub mod red_black_map;
pub use red_black_map::RedBlackMap;

#[path = "avl_map.rs"]
pub mod avl_map;
pub use avl_map::AvlMap;

#[path = "splay_map.rs"]
pub mod splay_map;
pub use splay_map::SplayMap;

#[path = "treap_map.rs"]
pub mod treap_map;
pub use treap_map::TreapMap;

#[path = "fibonacci_heap.rs"]
pub mod fibonacci_heap;
pub use fibonacci_heap::FibonacciHeap;

#[path = "pairing_heap.rs"]
pub mod pairing_heap;
pub use pairing_heap::PairingHeap;

#[path = "leftist_heap.rs"]
pub mod leftist_heap;
pub use leftist_heap::LeftistHeap;

#[path = "binomial_heap.rs"]
pub mod binomial_heap;
pub use binomial_heap::BinomialHeap;

#[path = "d_ary_heap.rs"]
pub mod d_ary_heap;
pub use d_ary_heap::DAryHeap;

#[path = "interval_query.rs"]
pub mod interval_query;
pub use interval_query::{Interval as QueryInterval, IntervalQueryTree};

#[path = "topological_sort.rs"]
pub mod topological_sort;
pub use topological_sort::{
    new_topo_graph as new_topo_sort_graph, topo_add_edge, topo_add_node, topo_clear,
    topo_edge_count, topo_has_cycle, topo_has_cycle_dag, topo_layer_count, topo_node_count,
    topo_remove_node, topo_sort, topo_sort_dag, topo_sources, TopoGraph as TopoSortGraph,
    TopoResult,
};

#[path = "strongly_connected.rs"]
pub mod strongly_connected;
pub use strongly_connected::{
    is_strongly_connected, largest_scc, new_scc_graph, scc_add_edge, scc_count, tarjan_scc,
    SccGraph,
};

#[path = "shortest_path_bfs.rs"]
pub mod shortest_path_bfs;
pub use shortest_path_bfs::{
    bfs_add_edge, bfs_add_undirected, bfs_distance, bfs_distances, bfs_reachable,
    bfs_shortest_path, new_bfs_graph, BfsGraph,
};

#[path = "bellman_ford.rs"]
pub mod bellman_ford;
pub use bellman_ford::{
    bellman_ford, bf_add_edge, bf_distance, bf_edge_count, bf_has_negative_cycle, new_bf_graph,
    BfEdge, BfGraph, BfResult,
};

#[path = "floyd_warshall.rs"]
pub mod floyd_warshall;
pub use floyd_warshall::{
    floyd_warshall, fw_add_edge, fw_distance, fw_has_negative_cycle, fw_solve, new_fw_result,
    FwResult,
};

#[path = "prim_mst.rs"]
pub mod prim_mst;
pub use prim_mst::{
    new_prim_graph, prim_add_edge, prim_edge_count, prim_mst, prim_mst_weight, prim_node_count,
    MstEdge, PrimGraph,
};

#[path = "kruskal_mst.rs"]
pub mod kruskal_mst;
pub use kruskal_mst::{
    kruskal_edges_from, kruskal_is_spanning, kruskal_mst, kruskal_mst_weight,
    new_union_find as new_kruskal_union_find, KruskalEdge, UnionFind,
};

#[path = "max_flow_ff.rs"]
pub mod max_flow_ff;
pub use max_flow_ff::{
    fg_add_edge, fg_has_augmenting_path, fg_node_count as flow_node_count, fg_total_capacity_from,
    max_flow, new_flow_graph, FlowGraph,
};

#[path = "bipartite_match.rs"]
pub mod bipartite_match;
pub use bipartite_match::{
    bip_add_edge, bip_edge_count, bipartite_matching, has_perfect_matching, max_matching_size,
    new_bipartite, BipartiteGraph,
};

#[path = "string_hash.rs"]
pub mod string_hash;
pub use string_hash::{
    compute_string_hash, hash_combine_strings, hash_empty_string, hash_to_hex_sh, string_hash_seed,
    string_hash_u32, string_hash_u64, string_hashes_equal, StringHash,
};

#[path = "aho_corasick.rs"]
pub mod aho_corasick;
pub use aho_corasick::{
    ac_add_pattern, ac_build, ac_contains, ac_pattern_count, ac_search, new_aho_corasick, AcMatch,
    AcNode, AhoCorasick,
};

#[path = "suffix_array_v2.rs"]
pub mod suffix_array_v2;
pub use suffix_array_v2::{build_sa_v2, sa2_contains, sa2_is_sorted, sa2_len, sa2_search};

#[path = "lcp_array.rs"]
pub mod lcp_array;
pub use lcp_array::{build_lcp, distinct_substrings, lcp_avg, lcp_max_val, lcp_query, lcp_valid};

#[path = "z_algorithm.rs"]
pub mod z_algorithm;
pub use z_algorithm::{z_contains, z_count, z_function, z_max, z_search, z_valid};

#[path = "kmp_search.rs"]
pub mod kmp_search;
pub use kmp_search::{
    kmp_contains, kmp_count, kmp_failure, kmp_failure_len, kmp_max_failure, kmp_search,
};

#[path = "graph_articulation.rs"]
pub mod graph_articulation;
pub use graph_articulation::{
    artic_add_edge, artic_component_count, find_articulation_points, find_bridges, is_biconnected,
    new_artic_graph, ArticGraph,
};

#[path = "edit_script.rs"]
pub mod edit_script;
pub use edit_script::{
    apply_edit_script, build_edit_script, edit_distance_from_script, script_to_diff_string, EditOp,
    EditScript,
};

#[path = "patience_diff.rs"]
pub mod patience_diff;
pub use patience_diff::{
    patience_diff, patience_diff_to_string, unique_common_lines, PatienceDiff, PatienceHunk,
};

#[path = "histogram_diff.rs"]
pub mod histogram_diff;
pub use histogram_diff::{
    build_histogram, histogram_diff, histogram_diff_to_string, HistogramDiff, HistogramDiffConfig,
    HistogramHunk,
};

#[path = "three_way_merge.rs"]
pub mod three_way_merge;
pub use three_way_merge::{
    clean_line_count, is_clean_merge, three_way_merge, MergeRegion, MergeResult,
};

#[path = "conflict_marker.rs"]
pub mod conflict_marker;
pub use conflict_marker::{
    parse_conflict_markers, render_conflict_block, resolve_all, ConflictBlock, ParsedConflicts,
    MARKER_OURS, MARKER_SEP, MARKER_THEIRS,
};

#[path = "patch_apply.rs"]
pub mod patch_apply;
pub use patch_apply::{
    apply_patch, can_apply_cleanly, count_overlapping_hunks, parse_unified_diff, HunkLine,
    PatchError, UnifiedHunk, UnifiedPatch,
};

#[path = "line_indexer.rs"]
pub mod line_indexer;
pub use line_indexer::{build_line_indexer, line_to_offset, offset_to_position, LineIndexer};

#[path = "syntax_highlighter.rs"]
pub mod syntax_highlighter;
pub use syntax_highlighter::{
    classify_token, count_kind as count_highlight_kind, highlight_tokens, to_ansi_string,
    HighlightKind, HighlightToken, HighlighterConfig, Language as HighlightLanguage,
};

#[path = "indent_detector.rs"]
pub mod indent_detector;
pub use indent_detector::{
    count_mixed_indent_lines, detect_indent, normalize_to_spaces, normalize_to_tabs, IndentResult,
    IndentStyle,
};

#[path = "whitespace_normalizer.rs"]
pub mod whitespace_normalizer;
pub use whitespace_normalizer::{
    collapse_blank_lines, detect_issues as detect_whitespace_issues,
    normalize as normalize_whitespace, strip_trailing, trailing_whitespace_count, LineEnding,
    NormalizerConfig, WhitespaceStats,
};

#[path = "unicode_segmenter.rs"]
pub mod unicode_segmenter;
pub use unicode_segmenter::{
    grapheme_count, has_multibyte_graphemes, nth_grapheme, reverse_graphemes, segment_graphemes,
    truncate_graphemes, word_wrap_graphemes, Grapheme,
};

#[path = "char_classifier.rs"]
pub mod char_classifier;
pub use char_classifier::{
    classify_char, classify_str, default_classifier_config, is_alnum, is_alpha, is_digit,
    is_punctuation, is_whitespace as char_is_whitespace, to_ascii_lower, to_ascii_upper, CharClass,
    ClassifierConfig,
};

#[path = "word_boundary.rs"]
pub mod word_boundary;
pub use word_boundary::{
    extract_words, find_word_spans, is_boundary_at, is_word_char, word_boundary_positions,
    word_count, WordBoundaryConfig, WordSpan,
};

#[path = "sentence_splitter.rs"]
pub mod sentence_splitter;
pub use sentence_splitter::{
    avg_words_per_sentence, filter_short_sentences, longest_sentence, sentence_count,
    split_sentences, Sentence, SentenceSplitterConfig,
};

#[path = "paragraph_detector.rs"]
pub mod paragraph_detector;
pub use paragraph_detector::{
    detect_paragraphs, filter_by_min_words, kind_summary, longest_paragraph, paragraph_count,
    Paragraph, ParagraphConfig, ParagraphKind,
};

#[path = "lexer_token_stream.rs"]
pub mod lexer_token_stream;
pub use lexer_token_stream::{
    count_tokens_of_kind, lex_string, LexToken, LexTokenKind, LexerStream,
};

#[path = "compression_lz4.rs"]
pub mod compression_lz4;
pub use compression_lz4::{
    lz4_compress, lz4_compress_bound, lz4_decompress, lz4_is_compressed, lz4_roundtrip_ok,
    Lz4Compressor, Lz4Config,
};

#[path = "compression_zstd.rs"]
pub mod compression_zstd;
pub use compression_zstd::{
    zstd_compress, zstd_decompress, zstd_frame_size_estimate, zstd_frame_valid, zstd_roundtrip_ok,
    ZstdCompressor, ZstdConfig,
};

#[path = "compression_brotli.rs"]
pub mod compression_brotli;
pub use compression_brotli::{
    brotli_compress, brotli_decompress, brotli_max_compressed_size, brotli_quality_valid,
    brotli_roundtrip_ok, BrotliCompressor, BrotliConfig,
};

#[path = "compression_snappy.rs"]
pub mod compression_snappy;
pub use compression_snappy::{
    snappy_compress, snappy_decompress, snappy_max_compressed_length, snappy_roundtrip_ok,
    snappy_validate_compressed_buffer, SnappyCompressor, SnappyConfig,
};

#[path = "encryption_aes.rs"]
pub mod encryption_aes;
pub use encryption_aes::{
    aes_derive_key_stub, aes_gcm_decrypt, aes_gcm_encrypt, aes_key_len_valid, AesGcmCipher,
    AesGcmConfig, AesKeyLen,
};

#[path = "encryption_chacha.rs"]
pub mod encryption_chacha;
pub use encryption_chacha::{
    chacha_decrypt, chacha_encrypt, chacha_key_from_seed, chacha_nonce_len, chacha_roundtrip_ok,
    ChaChaCipher, ChaChaConfig,
};

#[path = "hashing_sha256.rs"]
pub mod hashing_sha256;
pub use hashing_sha256::{hmac_sha256_stub, sha256_eq, sha256_hash, Sha256Digest, Sha256Hasher};

#[path = "hashing_blake3.rs"]
pub mod hashing_blake3;
pub use hashing_blake3::{
    blake3_hash, blake3_keyed_hash, blake3_output_len, blake3_stable, Blake3Digest, Blake3Hasher,
};

#[path = "hashing_xxhash.rs"]
pub mod hashing_xxhash;
pub use hashing_xxhash::{xxhash32, xxhash64, xxhash64_eq, xxhash64_hex, XxHasher};

#[path = "base64_codec.rs"]
pub mod base64_codec;
pub use base64_codec::{
    base64_decode, base64_decoded_len, base64_encode, base64_encode_str, base64_encoded_len,
    base64_is_valid, default_base64_config, Base64Config,
};

#[path = "base58_codec.rs"]
pub mod base58_codec;
pub use base58_codec::{
    base58_decode, base58_encode, base58_encoded_len_estimate, base58_is_valid, base58_roundtrip_ok,
};

#[path = "hex_codec.rs"]
pub mod hex_codec;
pub use hex_codec::{
    hex_decode, hex_encode, hex_encode_upper, hex_is_valid, hex_roundtrip_ok, hex_strip_prefix,
};

#[path = "url_encode.rs"]
pub mod url_encode;
pub use url_encode::{
    url_decode, url_encode as url_encode_str, url_encode_query, url_is_safe, url_roundtrip_ok,
};

#[path = "url_parser_stub.rs"]
pub mod url_parser_stub;

#[path = "html_escape.rs"]
pub mod html_escape;
pub use html_escape::{
    html_escape as html_entity_escape, html_escape_attr, html_needs_escape,
    html_roundtrip_ok as html_entity_roundtrip_ok, html_unescape as html_entity_unescape,
};

#[path = "csv_parser.rs"]
pub mod csv_parser;
pub use csv_parser::{
    csv_col_count, csv_field, csv_row_count, parse_csv, parse_csv_line, CsvRecord, CsvTable,
};

#[path = "tsv_parser.rs"]
pub mod tsv_parser;
pub use tsv_parser::{
    parse_tsv, parse_tsv_line, tsv_col_count, tsv_field, tsv_row_count, tsv_to_string, TsvRecord,
    TsvTable,
};

#[path = "json_pointer.rs"]
pub mod json_pointer;
pub use json_pointer::{
    escape_token, pointer_leaf, pointer_parent, unescape_token, JsonPointer, JsonPointerError,
};

#[path = "json_patch.rs"]
pub mod json_patch;
pub use json_patch::{
    count_ops, has_test_ops, parse_op_kind, validate_path, JsonPatch, PatchError as JsonPatchError,
    PatchOp,
};

#[path = "json_schema_validator.rs"]
pub mod json_schema_validator;
pub use json_schema_validator::{
    is_required, required_count, validate_number, validate_string, SchemaNode,
    SchemaType as JsonSchemaType, ValidationError as JsonSchemaValidationError,
};

#[path = "toml_parser.rs"]
pub mod toml_parser;
pub use toml_parser::{
    get_integer as toml_get_integer, get_string as toml_get_string, parse_line as parse_toml_line,
    parse_toml, TomlDocument, TomlParseError, TomlValue,
};

#[path = "yaml_parser.rs"]
pub mod yaml_parser;
pub use yaml_parser::{
    get_int as yaml_get_int, parse_scalar, parse_scalar_line, parse_yaml, YamlDocument, YamlError,
    YamlScalar,
};

#[path = "xml_tokenizer.rs"]
pub mod xml_tokenizer;
pub use xml_tokenizer::{
    collect_text, count_end_tags, count_start_tags, is_balanced, XmlError, XmlToken, XmlTokenizer,
};

#[path = "protobuf_varint.rs"]
pub mod protobuf_varint;
pub use protobuf_varint::{
    decode_varint, decode_zigzag, encode_varint, encode_zigzag, varint_roundtrip_ok, varint_size,
    VarintError,
};

#[path = "message_pack_codec.rs"]
pub mod message_pack_codec;
pub use message_pack_codec::{
    array_len, buffers_equal, encode as msgpack_encode, encoded_size as msgpack_encoded_size,
    is_nil, MsgError, MsgValue,
};

#[path = "cbor_codec.rs"]
pub mod cbor_codec;
pub use cbor_codec::{
    cbor_array_len, cbor_encoded_len, cbor_is_null, encode_cbor, major_of, CborError, CborMajor,
    CborValue,
};

#[path = "avro_codec.rs"]
pub mod avro_codec;
pub use avro_codec::{
    decode_long, encode_bytes as avro_encode_bytes, encode_long, is_union, record_field_count,
    type_name as avro_type_name, AvroError, AvroField, AvroType, AvroValue,
};

#[path = "flatbuffer_stub.rs"]
pub mod flatbuffer_stub;
pub use flatbuffer_stub::{padded_size, read_u32 as flatbuf_read_u32, FlatBuilder, FlatError};

#[path = "capnproto_stub.rs"]
pub mod capnproto_stub;
pub use capnproto_stub::{
    message_is_empty, serialize_message, traversal_limit_words, CapnMessage, CapnSegment,
};

#[path = "thrift_codec.rs"]
pub mod thrift_codec;
pub use thrift_codec::{
    decode_i32, encode_field_header, encode_i32, encode_string as thrift_encode_string,
    is_struct as thrift_is_struct, struct_field_count, type_of as thrift_type_of, ThriftError,
    ThriftField, ThriftType, ThriftValue,
};

#[path = "grpc_codec.rs"]
pub mod grpc_codec;
pub use grpc_codec::{
    decode_frame, decode_frame_header, encode_frame as grpc_encode_frame, framed_length,
    is_complete_frame, split_frames, GrpcError, GrpcFrameHeader,
};

#[path = "websocket_frame.rs"]
pub mod websocket_frame;
pub use websocket_frame::{apply_mask, is_control_frame, text_frame, WsError, WsFrame, WsOpcode};

#[path = "http_parser.rs"]
pub mod http_parser;
pub use http_parser::{
    content_length, find_header, is_http11, parse_request, parse_response, HttpError, HttpHeader,
    HttpMethod, HttpRequest, HttpResponse,
};

#[path = "oauth2_stub.rs"]
pub mod oauth2_stub;
pub use oauth2_stub::{
    build_authorization_url, exchange_code_for_token, generate_pkce_challenge,
    refresh_token as oauth2_refresh_token, OAuth2Client, OAuth2Config, OAuth2Token, PkceChallenge,
};

#[path = "jwt_codec.rs"]
pub mod jwt_codec;
pub use jwt_codec::{
    algorithm_name, base64url_encode, jwt_decode, jwt_encode, jwt_is_structurally_valid,
    DecodedJwt, JwtAlgorithm, JwtClaims, JwtHeader,
};

#[path = "session_token.rs"]
pub mod session_token;
pub use session_token::{
    create_session, generate_token as generate_session_token, purge_expired, revoke_session,
    validate_session, Session as HttpSession, SessionConfig, SessionStore as HttpSessionStore,
};

#[path = "rate_limiter_sliding.rs"]
pub mod rate_limiter_sliding;
pub use rate_limiter_sliding::{
    check_and_record, evict_old, new_rate_limiter, remaining_budget, requests_in_window,
    reset_limiter, SlidingRateLimiter, SlidingRateLimiterConfig,
};

#[path = "circuit_breaker.rs"]
pub mod circuit_breaker;
pub use circuit_breaker::{
    current_state, is_request_allowed, new_circuit_breaker, record_failure, record_success,
    CircuitBreaker, CircuitBreakerConfig, CircuitState,
};

#[path = "http_retry_policy.rs"]
pub mod http_retry_policy;
pub use http_retry_policy::{
    can_retry, delay_for_attempt, new_retry_state, next_delay_ms, remaining_attempts,
    reset_retry as reset_http_retry, BackoffStrategy, HttpRetryPolicyConfig, RetryState,
};

#[path = "health_check.rs"]
pub mod health_check;
pub use health_check::{
    add_result, aggregate_health, all_healthy, count_by_status, new_aggregator, HealthAggregator,
    HealthCheckConfig, HealthCheckResult, HealthReport, HealthStatus,
};

#[path = "service_registry.rs"]
pub mod service_registry;
pub use service_registry::{
    deregister_instance, new_registry as new_service_registry, register_instance, resolve_service,
    set_service_health, total_instance_count, ServiceInstance, ServiceRegistry,
    ServiceRegistryConfig,
};

#[path = "message_router.rs"]
pub mod message_router;
pub use message_router::{
    add_route, all_handler_ids, new_router, remove_routes_for, route_message, set_handler_enabled,
    MessageRouter, MessageRouterConfig, RoutableMessage, RouteEntry,
};

#[path = "publish_subscribe.rs"]
pub mod publish_subscribe;
pub use publish_subscribe::{
    clear_topic, messages_for_topic, new_pubsub_bus, publish, subscribe, subscriber_count,
    unsubscribe, PubSubBus, PubSubConfig, Subscription, TopicMessage,
};

#[path = "request_pipeline.rs"]
pub mod request_pipeline;
pub use request_pipeline::{
    enabled_stage_count, new_pipeline as new_request_pipeline, register_stage, remove_stage,
    run_pipeline, stage_names, MiddlewareResult, MiddlewareStage, PipelineConfig, RequestContext,
    RequestPipeline,
};

#[path = "response_cache.rs"]
pub mod response_cache;
pub use response_cache::{
    cache_get as response_cache_get, cache_invalidate, cache_size as response_cache_size,
    cache_store, new_response_cache, purge_expired_responses, CachedResponse, ResponseCache,
    ResponseCacheConfig,
};

#[path = "content_negotiation.rs"]
pub mod content_negotiation;
pub use content_negotiation::{
    default_quality, is_text_type, mime_to_extension, negotiate, parse_accept_header, MediaType,
    NegotiationResult,
};

#[path = "multipart_parser.rs"]
pub mod multipart_parser;
pub use multipart_parser::{
    extract_boundary, find_part_by_name, parse_multipart, total_body_bytes, MultipartBody,
    MultipartError, MultipartPart,
};

#[path = "cookie_jar.rs"]
pub mod cookie_jar;
pub use cookie_jar::{
    delete_cookie, get_cookie, jar_size, new_cookie_jar, purge_expired_cookies,
    serialize_set_cookie, set_cookie, Cookie, CookieJar,
};
