// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Morphology engine for parametric human body generation.
//!
//! Provides target-based morphing, blendshape interpolation, age and body
//! composition models, FACS facial expressions, pose graphs, and GPU-ready
//! skin deformation — all in pure Rust.

pub mod age_model;
pub mod anim_retarget;
pub mod anthropometry;
pub mod apply;
pub mod blend_profile;
pub mod cache;
pub mod colors;
pub mod compress;
pub mod constraint;
pub mod curves;
pub mod delta_cache;
pub mod diff;
pub mod engine;
pub mod expression;
pub mod fitting;
pub mod history;
pub mod interpolate;
pub mod measurements;
pub mod params;
pub mod preset_io;
pub mod presets;
pub mod regions;
pub mod schema_migration;
pub mod search;
pub mod session;
pub mod shape_compare;
pub mod skin_color;
pub mod symmetry;
pub mod target_lib;
pub mod units;
pub mod weight_curves;

#[cfg(feature = "simd")]
pub mod simd_morph;

pub use age_model::{
    age_progression, aging_delta, estimate_bmi_category, param_age_to_years, years_to_param_age,
    AgeProfile, BmiCategory, LifeStage,
};
pub use anim_retarget::{
    concat_tracks, retarget_keyframe, retarget_track, reverse_track, scale_track_time, trim_track,
    AnimRetargetConfig, ParamRetarget,
};
pub use anthropometry::{AnthroDistribution, BodyRandomizer, Lcg};
pub use apply::apply_targets_parallel;
pub use blend_profile::{BlendEntry, BlendProfile, BlendProfileLibrary};
pub use cache::MeshCache;
pub use colors::{Color, ColorTheme, ThemePalette};
pub use compress::{
    compress_target, compression_ratio, decompress_target, max_reconstruction_error,
    read_compressed_cache, write_compressed_cache, CompressConfig, QuantizedDelta,
};
pub use curves::{CurveKind, EaseInCurve, EaseOutCurve, LinearCurve, SmoothStepCurve, WeightCurve};
pub use diff::{top_displaced_vertices, vertex_displacements, MeshDiffStats, ParamDiff};
pub use engine::HumanEngine;
pub use expression::{apply_expression_to_engine, ExpressionComponent, ExpressionPreset};
pub use fitting::{fit_params, quick_fit, FitResult, TargetMeasurements};
pub use history::{History, HistoryEntry};
pub use interpolate::{catmull_rom, lerp_params, Keyframe, MorphTrack};
pub use params::ParamState;
pub use preset_io::{
    load_preset_json, load_preset_library_json, preset_from_json_string, preset_to_json_string,
    save_preset_json, save_preset_library_json,
};
pub use presets::BodyPreset as EnumBodyPreset;
pub use regions::{BodyRegion, RegionParams, RegionTag};
pub use search::{fuzzy_score, is_subsequence, TargetEntry, TargetIndex};
pub use session::MorphSession;
pub use shape_compare::{
    average_shape, cluster_shapes, compare_shapes, cosine_similarity, euclidean_distance,
    interpolate_shapes, manhattan_distance, step_toward, ParamDifference, ShapeComparison,
};
pub use skin_color::{linear_to_srgb, srgb_to_linear, FitzpatrickType, SkinColor, SkinColorMap};
pub use symmetry::{
    is_left_side, is_right_side, mirror_target_deltas, mirror_target_name, symmetrize_positions,
    SymmetryMap,
};
pub use target_lib::LibraryStats;
pub use target_lib::TargetLibrary;
pub use weight_curves::{auto_weight_fn, auto_weight_fn_for_target, infer_category_from_name};
pub mod pose_blend;
pub use pose_blend::{
    angle_to_weight, make_elbow_corrective, make_shoulder_corrective, BlendInterpolation,
    JointRotation, PoseBlendLibrary, PoseCorrectiveShape,
};
pub mod timeline;
pub use timeline::Keyframe as TimelineKeyframe;
pub use timeline::{AnimTrack, Timeline, TrackInterp};
pub mod diversity;
pub use diversity::{
    default_body_params, generate_population, van_der_corput, DiversitySampler,
    Lcg as DiversityLcg, ParamSpec, SamplingStrategy,
};
pub mod incremental;
pub use incremental::{DirtyTracker, IncrementalMorphCache};
pub mod blend_tree;
pub use blend_tree::{
    blend_params, clamp_params, merge_params, scale_params, BlendMode, BlendNode, ParamMap,
};
pub mod param_constraint;
pub use param_constraint::{
    age_constraints, bmi_constraints, proportion_constraints, Constraint, ConstraintSolver,
    Params as ConstraintParams, SolveResult,
};
pub mod muscle_sim;
pub use muscle_sim::{
    bicep_muscle, calf_muscle, muscle_from_region, quadricep_muscle, BulgeDirection, Muscle,
    MuscleSimulator,
};
pub mod character_rig;
pub use character_rig::{
    minimal_human_rig, standard_human_rig, CharacterRig, MorphBinding, RigBodyRegion, RigJoint,
};
pub mod morph_layer;
pub use morph_layer::{blend_layer, LayerBlend, MorphLayer, MorphLayerStack};
pub mod pose_graph;
pub use pose_graph::{
    apply_easing, Easing, PoseGraph, PoseNode, PoseParams as GraphPoseParams, PoseTransition,
};
pub mod emotion_system;
pub use emotion_system::{
    default_emotion_system, lerp_emotion_blend, Emotion, EmotionBlend, EmotionExpression,
    EmotionSystem,
};
pub mod body_preset;
pub use body_preset::{
    preset_athletic, preset_average, preset_child, preset_elder, preset_heavy, preset_muscular,
    preset_petite, preset_slender, preset_tall, standard_preset_library, BodyCategory,
    BodyParams as PresetBodyParams, BodyPreset, PresetLibrary,
};
pub mod facs;
pub use facs::{
    default_facs_mapper, emotion_to_facs, parse_facs_string, ActionUnit, FacsIntensity, FacsMapper,
    FacsState,
};
pub mod gaze;
pub use gaze::{
    compute_gaze, eye_angles_to_point, gaze_to_rotation_matrix, iris_deform_weight,
    lid_follow_weight, EyeConfig, EyeGazeAngles, GazeResult, GazeTarget, SaccadeSequence,
};
pub mod speech_viseme;
pub use speech_viseme::{
    default_viseme_mapper, phoneme_to_viseme, LipSyncTrack, Phoneme, PhonemeEvent, Viseme,
    VisemeMapper, VisemeMorphWeights,
};
pub mod genetic;
pub use genetic::{
    average_params, blend_params as genetic_blend_params, clamp_params as genetic_clamp_params,
    crossover_blend, dominant_blend, inherit_random, lcg_f32, params_distance, GeneticParams,
    GeneticPopulation, GeneticProfile,
};
pub mod expression_library;
pub use expression_library::{
    expression_distance, ExpressionBlender, ExpressionLibrary,
    ExpressionPreset as ExpressionLibPreset,
};
pub mod body_scan_fit;
pub use body_scan_fit::{
    align_scan_to_mesh, estimate_measurements, fit_params_to_scan, measurements_to_params,
    quick_fit_from_bbox, scan_to_mesh_error, BodyMeasurementsEstimate, FitConfig,
    FitResult as ScanFitResult, IcpAligner, IcpResult, PhotoFitResult, PointCloud, ScanCloud,
    ScanFitConfig, ScanFitter,
};
pub mod mocap_bvh;
pub use mocap_bvh::{
    map_bvh_to_oxihuman, parse_bvh, retarget_scale, write_bvh, BvhChannel, BvhFile, BvhFrame,
    BvhJoint, BvhSkeleton,
};
pub mod body_landmark;
pub use body_landmark::{
    detect_landmarks, landmark_frame, nearest_vertex, remap_landmarks, transfer_landmarks,
    Landmark, LandmarkId, LandmarkSet, Side,
};
pub mod param_animation;
pub use param_animation::{
    blend_clip, breathing_clip, cubic_hermite, interpolate as param_interpolate, smoothstep_interp,
    InterpMode, Keyframe as ParamKeyframe, LoopMode, ParamClip, ParamTrack,
};
pub mod ethnic_variation;
pub use ethnic_variation::{
    age_to_param, bmi_to_params, height_m_to_param, lcg_normal, sample_heights, AnthroLibrary,
    AnthroProfile, AnthroSample,
};
pub mod crowd_generator;
pub use crowd_generator::{
    enforce_diversity, generate_crowd, generate_crowd_halton, halton, lcg_rand as crowd_lcg_rand,
    param_distance as crowd_param_distance, Crowd, CrowdCharacter, CrowdConfig, VariationClass,
};
pub mod muscle_control;
pub use muscle_control::{
    blend_rig_states, params_to_muscle_activation, rig_to_morphs, MuscleDefinition,
    MuscleGroup as MuscleControlGroup, MuscleRig, MuscleState, Side as MuscleSide,
};
pub mod emotion_space;
pub use emotion_space::{
    idw_weight, mix_expressions, pad_to_description, EmotionAnchor, EmotionSpace,
    EmotionTransition, PadPoint,
};
pub mod motion_graph;
pub use motion_graph::{
    blend_morph_maps as motion_blend_morphs, build_expression_graph, build_locomotion_graph,
    MotionController, MotionGraph, MotionState, MotionTransition, TransitionCondition,
};
pub mod influence_map;
pub use influence_map::{
    build_influence_map, influence_map_stats, target_vertex_coverage, top_influences_for_vertex,
    vertex_target_overlap, InfluenceMap, InfluenceMapStats, VertexInfluence,
};
pub mod expression_mixer;
pub use expression_mixer::{
    add_weight_maps, clamp_weight_map, corrective_layer, emotion_layer, lip_sync_layer,
    merge_weight_maps, micro_expression_layer, scale_weight_map, threshold_weight_map,
    top_n_weights, weight_map_magnitude, ExpressionMixer, MixLayer, MorphWeightMap,
};
pub mod expression_retarget;
pub use expression_retarget::{
    blend_retargeted, build_prefix_map, identity_map, makehuman_to_daz_map, retarget_stats,
    retarget_weights as retarget_expr_weights, scale_retarget_weights,
    MorphWeights as RetargetMorphWeights, RetargetConfig, RetargetMap as ExprRetargetMap,
    RetargetStats, UnmappedPolicy,
};
pub mod mutation_engine;
pub use mutation_engine::{
    default_human_specs, fitness_rank, tournament_select, MutationConfig, MutationEngine,
    MutationResult, ParamMap as MutationParamMap, ParamSpec as MutationParamSpec,
};
pub mod body_proportions;
pub use body_proportions::{
    golden_ratio_params, normalize_to_schema, params_to_ratios, proportion_score, standard_schemas,
    ProportionAnalysis, ProportionLibrary, ProportionSchema,
};
pub mod expression_sequence;
pub use expression_sequence::{
    blink_track, breathing_expr_track, ease_value, lerp_weights as seq_lerp_weights,
    EaseType as ExprEaseType, ExprKeyframe, ExprSequencer, ExprTrack, ExprWeights, SeqLoopMode,
};
pub mod skin_deform;
pub use skin_deform::{
    blend_skin_maps, bulge_weights, clamp_skin_map, default_skin_system, sag_weights,
    wrinkle_weights, MorphMap as SkinMorphMap, SkinDeformPattern, SkinDeformSystem,
};
pub mod character_dna;
pub use character_dna::{
    crossover_dna, decode_dna, dna_distance, dna_from_base64, dna_from_hex, dna_to_base64,
    dna_to_hex, dna_to_params_map, encode_dna, mutate_dna, CharacterDna, ExtendedDna,
};
pub mod pose_driver;
pub mod volume_morph;
pub mod weight_optimizer;
pub use pose_driver::{
    normalize_weights, pose_distance, rbf_gaussian, rbf_inverse_distance, rbf_thin_plate,
    PoseDriver, PoseDriverConfig, PoseDriverSample, RbfFalloff,
};
pub use volume_morph::{
    compute_mesh_volume, laplacian_smooth_deltas, mesh_volume_ratio, uniform_scale_correction,
    volume_error_percent, volume_preserving_delta, VolumeMorphConfig, VolumeMorphResult,
};
pub use weight_optimizer::{
    apply_weights, clamp_weights, gradient_wrt_weights, reconstruction_error, OptimizationResult,
    WeightOptimizer,
};
pub mod emotion_timeline;
pub mod micro_expression;
pub mod speech_baker;
pub use emotion_timeline::{
    apply_easing_fn, interpolate_emotions, normalize_emotion_time, EmotionKeyframe,
    EmotionTimeline, TimelineEasing, TimelineLoop,
};
pub use micro_expression::{
    inject_random_micros, merge_weights, micro_expr_weight_at, standard_micro_expressions,
    MicroExpression, MicroExpressionEvent, MicroExpressionLayer,
};
pub use speech_baker::{
    active_phonemes_at, bake_phoneme_sequence, blend_viseme_weights, build_default_viseme_map,
    BakedLipSync, BakerConfig, PhonemeEvent as BakerPhonemeEvent,
};
pub mod anthropometric_constraints;
pub mod body_symmetry;
pub mod corrective_shapes;
pub use anthropometric_constraints::{
    bmi_from_params, check_params_against_constraints, enforce_constraints, params_to_body_ratios,
    realism_score, standard_anthropometric_constraints, violation_severity, AnthroCheckResult,
    AnthroConstraint, AnthroConstraintSet, AnthroViolation,
};
pub use body_symmetry::{
    asymmetry_noise, enforce_symmetry, find_symmetry_pairs_x, inject_asymmetry, mirror_position,
    symmetrize_morph_deltas, symmetry_report, AsymmetryConfig, SymmetryAxis, SymmetryConfig,
    SymmetryReport,
};
pub use corrective_shapes::{
    apply_corrective_to_mesh, combine_corrective_deltas, corrective_distance, corrective_weight,
    standard_corrective_shapes, CorrectiveEvalResult, CorrectiveShape, CorrectiveShapeLibrary,
};
pub mod blend_shape_io;
pub mod expression_transfer;
pub mod retarget_mesh;
pub use blend_shape_io::{
    blend_shape_stats, export_blend_shape_obj_delta, export_blend_shapes_csv,
    export_blend_shapes_json, filter_zero_deltas, import_blend_shape_obj_delta,
    import_blend_shapes_csv, import_blend_shapes_json, merge_blend_shape_libraries,
    BlendShapeEntry, BlendShapeLibraryFile,
};
pub use expression_transfer::{
    barycentric_coords, delta_magnitude, interpolate_delta_barycentric, mesh_scale_ratio,
    transfer_expression, transfer_expression_batch, ExpressionTransferBatch,
    ExpressionTransferConfig, TransferInterp, TransferredExpression,
};
pub use retarget_mesh::{
    closest_vertex, retarget_error_stats, retarget_mesh_positions, smooth_transferred_positions,
    transfer_deltas, RetargetMeshConfig, RetargetMeshResult,
};
pub mod param_space_optimizer;
pub mod parametric_face;
pub mod target_authoring;
pub use param_space_optimizer::{
    analyze_param_space, build_correlation_matrix, find_redundant_params, normalize_param_samples,
    param_correlation, param_importance_score, param_variance, reduce_param_set,
    ParamSpaceAnalysis, ParamSpaceConfig,
};
pub use parametric_face::{
    apply_expression_preset, blend_face_params, expression_presets, standard_face_action_units,
    standard_face_params, FaceActionUnit, FaceModel, FaceParam,
};
pub use target_authoring::{
    authored_target_stats, create_target_from_delta_field, create_target_from_mesh_pair,
    invert_target, merge_targets, mirror_target_x, scale_target, smooth_target_deltas,
    target_delta_bounds, AuthoredTarget, AuthoringConfig,
};
pub mod mocap_retarget_adv;
pub use mocap_retarget_adv::{
    blend_poses, quat_inverse, quat_multiply, quat_slerp, retarget_pose_adv,
    standard_biped_retarget_map, Joint, RetargetMap, SkeletonPose,
};
pub mod expression_calibration;
pub use expression_calibration::{
    calibrate_expression_to_landmarks, landmark_delta, normalize_landmark_set,
    standard_68_landmarks, AuActivation, FacialLandmark, LandmarkSet as CalibLandmarkSet,
};
pub mod body_composition;
pub use body_composition::{
    bmi, body_fat_from_bmi_sex_age, classify_body_fat, interpolate_compositions,
    morph_params_from_composition, validate_composition, BodyComposition, CompositionProfile,
};
pub mod crowd_variation;
pub use crowd_variation::{
    cluster_crowd, crowd_diversity_score, crowd_to_json,
    generate_crowd as generate_crowd_variation, standard_crowd_axes, Crowd as CrowdVariation,
    CrowdMember, CrowdSpec, Distribution, VariationAxis,
};
pub mod age_progression_adv;
pub use age_progression_adv::{
    body_aging_params, compute_age_stage, default_aging_curve_female, default_aging_curve_male,
    face_aging_params, simulate_aging, skin_aging_params, AgeProfile as AgeProfileAdv, AgeStage,
    AgingCurve,
};
pub mod expression_composer;
pub use expression_composer::{
    add_layer, add_preset_layer, blend_layers, default_composer_config, evaluate_expression,
    expression_energy as composer_expression_energy,
    expression_to_json as composer_expression_to_json,
    get_layer_weight as composer_get_layer_weight, layer_count, new_composed_expression,
    new_expression_composer, normalize_expression as composer_normalize_expression, remove_layer,
    reset_expression, set_layer_weight, ComposedExpression, ExpressionComposer,
    ExpressionComposerConfig, ExpressionComposerLayer,
};

pub mod body_language;
pub use body_language::{
    apply_emotion_to_params, blend_body_emotions, classify_body_language, dominant_emotion,
    generate_pose_for_emotion, interpolate_pose_features, mirror_pose, normalize_pose_features,
    pose_similarity, pose_to_json, BodyEmotion, BodyLanguageProfile, PoseFeatures,
};

pub mod speech_prosody;
pub use speech_prosody::{
    blend_prosody_emotions, classify_prosody_emotion, dominant_prosody_emotion,
    estimate_arousal_valence, generate_prosody_for_emotion, interpolate_prosody, normalize_prosody,
    prosody_similarity, prosody_to_face_params, prosody_to_json, speech_rate_category,
    ProsodyEmotion, ProsodyFeatures, ProsodyProfile,
};

pub mod pose_symmetry;
pub use pose_symmetry::{
    apply_pose_offset, detect_symmetry_pairs, enforce_symmetry_pose, find_joint_by_name,
    interpolate_poses, mirror_joint_rotation, mirror_pose as mirror_skeleton_pose,
    pose_distance_sym, pose_symmetry_error, quat_slerp_pose, standard_biped_symmetry_pairs,
    JointPose, PoseSkeleton, SymmetryPair,
};

pub mod body_hair;
pub use body_hair::{
    add_region, blend_hair_profiles, curl_tip, default_hair_profile, generate_strands,
    hair_count_for_region, hair_profile_to_params, lod_density_factor, region_by_name,
    scale_density, total_strand_count, HairGenerationParams, HairProfile, HairRegion, HairStrand,
};

pub mod motion_warp;
pub use motion_warp::{
    apply_warp, blend_clips, clip_duration, concat_clips, identity_warp_curve, linear_warp_curve,
    loop_clip, pose_lerp, reverse_clip, sample_clip, speed_scale_clip, trim_clip, warp_time,
    MotionClip, MotionFrame, WarpCurve, WarpMode, WarpedClip,
};

pub mod facial_rig;
pub use facial_rig::{
    add_bone as add_facial_bone, add_corrective, apply_facial_pose, blend_facial_poses, bone_count,
    corrective_count, default_facial_rig, evaluate_correctives, facial_rig_to_json,
    get_bone as get_facial_bone, identity_pose as identity_facial_pose, new_facial_rig,
    quat_angle_between, set_bone_rotation, CorrectiveShape as FacialCorrectiveShape, FacialBone,
    FacialPose, FacialRig,
};

pub mod pose_interpolation;
pub use pose_interpolation::{
    add_pose_key, compute_cubic_tangents, cubic_hermite_interp, cubic_hermite_pose,
    curve_duration as interp_curve_duration, lerp_poses as lerp_interp_poses,
    normalize_quat as normalize_pose_quat, quat_dot, quat_multiply as pose_quat_multiply,
    quat_slerp_interp, sample_pose_curve, squad_intermediate, squad_quat, tcb_tangents,
    InterpMode as PoseInterpMode, PoseCurve, PoseKey, TcbParams,
};

pub mod expression_physics;
pub use expression_physics::{
    add_spring_joint, apply_impulse_to_joint, default_facial_physics, evaluate_expression_physics,
    joint_count as expr_joint_count, joint_displacement, joint_kinetic_energy,
    new_expression_physics, reset_to_rest, set_enabled as set_expr_physics_enabled,
    set_rest_position, step_expression_physics, ExpressionPhysics, PhysicsExpressionResult,
    SpringJoint,
};

pub mod voice_driven_anim;
pub use voice_driven_anim::{
    amplitude_to_jaw, audio_frames_to_jaw_curve, blend_jaw_curves, default_voice_anim_config,
    frames_to_viseme_weights, frequency_to_viseme_index, jaw_curve_duration, jaw_curve_max,
    sample_jaw_curve, silence_duration, smooth_jaw_curve, voice_anim_from_frames, voiced_segments,
    AudioFrame, JawCurve, VoiceAnimConfig, VoiceAnimResult,
};

pub mod morph_quantize;
pub use morph_quantize::{
    apply_quantized_morph, default_morph_quantize_config, dequantize_delta,
    dequantize_morph as dequantize_morph_target,
    filter_zero_deltas as filter_quantized_zero_deltas, max_quantization_error,
    merge_quantized_morphs, morph_compression_ratio, pack_quantized_morph, quantize_delta,
    quantize_morph as quantize_morph_target, quantized_delta_count, unpack_quantized_morph,
    MorphQuantizeConfig, QuantizedDelta as MorphQuantizedDelta, QuantizedMorph,
};

pub mod blend_shape_graph;
pub use blend_shape_graph::{
    add_blend_node, add_leaf_node, add_root, blend_graph_to_json,
    connect_nodes as graph_connect_nodes, evaluate_graph, get_node, leaf_nodes, new_blend_graph,
    node_count as graph_node_count, prune_zero_weight, set_node_weight, topological_sort_graph,
    BlendGraph, BlendNode as BlendGraphNode, BlendOp, EvalResult as BlendEvalResult,
};

pub mod wrinkle_map;
pub use wrinkle_map::{
    add_procedural_wrinkles, apply_wrinkle_weight, blend_wrinkle_maps,
    compute_wrinkle_from_deformation, default_wrinkle_config, new_wrinkle_map,
    normalize_wrinkle_map, smooth_wrinkle_map, threshold_wrinkle_map, wrinkle_map_max,
    wrinkle_map_min, wrinkle_region_average, wrinkle_to_normal_delta, WrinkleConfig, WrinkleMap,
    WrinkleRegion,
};

pub mod muscle_line;
pub use muscle_line::{
    add_muscle_to_group, apply_muscle_deformation, compute_muscle_deformation, contract_muscle,
    default_arm_muscles, muscle_direction, muscle_group_deformation, muscle_influence_weight,
    muscle_length, new_muscle_group, new_muscle_line,
    point_to_line_distance as muscle_point_to_line, relax_muscle, MuscleDeformation, MuscleGroup,
    MuscleLine,
};

pub mod secondary_motion;
pub use secondary_motion::{
    add_secondary_bone, blend_secondary_to_target, chain_kinetic_energy, default_secondary_config,
    new_secondary_bone, new_secondary_chain, reset_secondary_chain, secondary_bone_count,
    secondary_bone_lag, secondary_chain_positions, set_chain_wind, update_secondary_bone,
    update_secondary_chain, SecondaryBone, SecondaryChain, SecondaryMotionConfig,
};

pub mod breathing_sim;
pub use breathing_sim::{
    add_breath_region, advance_breath, apply_breathing, blend_breath_states, breath_region_count,
    breath_value_at, current_phase, default_breath_cycle, exhale_value, inhale_value,
    new_breathing_state, set_breath_amplitude, set_breath_rate, BreathCycle, BreathPhase,
    BreathRegion, BreathingState,
};

pub mod lip_sync_advanced;
pub use lip_sync_advanced::{
    add_phoneme_event, default_coarticulation, evaluate_lip_sync,
    event_count as lip_sync_event_count, lip_sync_to_viseme_weights, merge_lip_sync_tracks,
    new_lip_sync_track, phoneme_to_mouth_shape, phonemes_at_time, scale_lip_sync_timing,
    sort_phoneme_events, trim_lip_sync, CoarticulationParams, LipSyncFrame,
    LipSyncTrack as AdvLipSyncTrack, PhonemeEvent as AdvPhonemeEvent,
};

pub mod blink_control;
pub use blink_control::{
    blink_speed_for_emotion, blink_value, default_blink_params, disable_blink, enable_blink,
    force_close_eyes, force_open_eyes, is_blinking, lcg_next as blink_lcg_next, new_blink_state,
    set_blink_synchronized, trigger_manual_blink, update_blink, BlinkParams, BlinkPhase,
    BlinkState,
};

pub mod morph_delta_stream;
pub use morph_delta_stream::{
    apply_chunk as apply_delta_chunk, apply_stream, clear_stream, default_stream_config,
    filter_stream_threshold, merge_chunks, new_delta_stream, push_chunk, split_deltas_into_chunks,
    stream_chunk_count, stream_delta_count, stream_memory_bytes, stream_to_flat_deltas, DeltaChunk,
    DeltaStream, StreamConfig,
};

pub mod expression_recorder;
pub use expression_recorder::{
    advance_playback, new_recorder_state, new_recording as new_expression_recording, playback_at,
    record_snapshot, recording_duration, reverse_recording, scale_recording_time,
    snapshot_count as recording_snapshot_count, start_playback, start_recording, stop_playback,
    stop_recording, trim_recording, ExpressionRecording, ExpressionSnapshot, RecorderState,
};

pub mod muscle_action_unit;
pub use muscle_action_unit::{
    active_aus, add_action_unit, apply_au_frame, au_count, au_frame_from_set, au_to_emotion,
    blend_au_frames, evaluate_au_set, get_au, new_au_set, reset_all_aus, set_au_intensity,
    standard_facs_set, ActionUnit as MauActionUnit, AuFrame, AuSet,
};

pub mod pose_database;
pub use pose_database::{
    add_pose_entry, all_tags, get_pose, import_poses, nearest_pose, new_pose_database, pose_count,
    pose_database_to_json, pose_similarity as pose_db_similarity, remove_pose,
    search_by_name as pose_search_by_name, search_by_tag as pose_search_by_tag,
    sort_by_name as pose_sort_by_name, PoseDatabase, PoseEntry,
};

pub mod eye_control;
pub use eye_control::{
    auto_blink_tick, blink_factor, clamp_gaze, default_eye_config, eye_pitch_deg, eye_yaw_deg,
    gaze_blend, gaze_distance, is_blinking_eye, look_at_target, new_eye_state, saccade_towards,
    trigger_blink, update_eye_gaze, EyeControlConfig, EyeState, GazeTarget as EyeGazeTarget,
};

pub mod cloth_blend;
pub use cloth_blend::{
    apply_body_offset, blend_cloth_layers, cloth_blend_energy, cloth_collision_push,
    cloth_layer_count, cloth_to_rest, default_cloth_blend_config, get_layer_weight,
    new_cloth_layer, normalize_cloth_weights, set_layer_weight as set_cloth_layer_weight,
    smooth_cloth_blend, ClothBlendConfig, ClothBlendResult, ClothLayer,
};

pub mod jaw_control;
pub use jaw_control::{
    blend_jaw_states, build_default_phoneme_map, clamp_jaw_range, default_jaw_config,
    jaw_lateral_offset, jaw_open_amount, jaw_open_for_phoneme, jaw_to_morph_weights, jaw_velocity,
    new_jaw_state, reset_jaw, set_jaw_lateral, set_jaw_open, update_jaw, JawConfig, JawState,
    PhonemeJawMap,
};

pub mod skin_shader;
pub use skin_shader::{
    apply_age_effect, blend_skin_params, default_skin_params, new_skin_preset, preset_count,
    set_hemoglobin, set_melanin, set_roughness, set_sss_strength, set_zone_tint,
    skin_color_from_params, skin_preset_to_json, zone_params, SkinPreset, SkinShaderParams,
    SkinZone,
};

pub mod tongue_control;
pub use tongue_control::{
    blend_tongue_states, build_tongue_phoneme_map, default_tongue_config, new_tongue_state,
    reset_tongue, set_tongue_elevation, set_tongue_extension, set_tongue_lateral, set_tongue_shape,
    tongue_extension_amount, tongue_for_phoneme, tongue_shape_name, tongue_to_morph_weights,
    update_tongue, TongueConfig, TongueShape, TongueState,
};

pub mod hair_strand;
pub use hair_strand::{
    apply_gravity_to_strand, blend_strands, curl_amplitude, curl_frequency, default_strand_config,
    generate_strand_points, new_hair_strand, set_strand_profile, strand_bounding_box,
    strand_length, strand_point_count, strand_tangent_at, strand_to_vertices,
    HairProfile as StrandHairProfile, HairStrand as StrandHairStrand, HairStrandConfig,
};

pub mod cheek_control;
pub use cheek_control::{
    apply_smile_effect, blend_cheek_states, cheek_hollow_left, cheek_hollow_right, cheek_puff_left,
    cheek_puff_right, cheek_to_morph_weights, default_cheek_config, new_cheek_state, reset_cheeks,
    set_cheek_hollow, set_cheek_puff, set_cheek_raise, update_cheeks, CheekConfig,
    CheekMorphWeights, CheekSide, CheekState,
};

pub mod brow_control;
pub use brow_control::{
    blend_brow_states, brow_furrow_amount, brow_raise_left, brow_raise_right,
    brow_to_morph_weights, default_brow_config, emotion_to_brow, new_brow_state, reset_brows,
    set_brow_arch, set_brow_furrow, set_brow_lower, set_brow_raise, update_brows, BrowConfig,
    BrowMorphWeights, BrowSide, BrowState,
};

pub mod body_weight_control;
pub use body_weight_control::{
    bw_blend, bw_distribution_curve, bw_reset, bw_set_belly, bw_set_lower, bw_set_overall,
    bw_set_upper, bw_to_json, bw_to_weights, default_body_weight_config, new_body_weight_state,
    BodyWeightConfig, BodyWeightState,
};

pub mod brow_height_control;
pub use brow_height_control::{
    browh_blend, browh_effective_right, browh_reset, browh_set_both, browh_set_left,
    browh_set_right, browh_set_symmetry, browh_to_json, browh_to_weights,
    default_brow_height_config, new_brow_height_state, BrowHeightConfig, BrowHeightState,
};

pub mod cheek_depth_control;
pub use cheek_depth_control::{
    cd_blend, cd_effective_depth, cd_reset, cd_set_both, cd_set_hollow, cd_set_left, cd_set_right,
    cd_to_json, cd_to_weights, default_cheek_depth_config, new_cheek_depth_state, CheekDepthConfig,
    CheekDepthState,
};

pub mod chin_width_control;
pub use chin_width_control::{
    cw_blend, cw_effective_width, cw_reset, cw_set_cleft, cw_set_taper, cw_set_width, cw_to_json,
    cw_to_weights, default_chin_width_config, new_chin_width_state, ChinWidthConfig,
    ChinWidthState,
};

pub mod ear_tragus_control;
pub use ear_tragus_control::{
    default_ear_tragus_config, et_blend, et_reset, et_set_both, et_set_left, et_set_protrusion,
    et_set_right, et_to_json, et_to_weights, new_ear_tragus_state, EarTragusConfig, EarTragusState,
};

pub mod eye_fold_control;
pub use eye_fold_control::{
    default_eye_fold_config, ef_blend, ef_reset, ef_set_both, ef_set_crease, ef_set_left,
    ef_set_right, ef_to_json, ef_to_weights, new_eye_fold_state, EyeFoldConfig, EyeFoldState,
};

pub mod face_length_control;
pub use face_length_control::{
    default_face_length_config, facel_clamp, facel_effective_scale, facel_reset,
    facel_set_lower_contrib, facel_set_scale, facel_set_upper_contrib, facel_to_json,
    facel_to_weights, new_face_length_state, FaceLengthConfig, FaceLengthState,
};

pub mod finger_joint_control;
pub use finger_joint_control::{
    fj_blend, fj_curl_angle, fj_finger_index, fj_reset, fj_set_all_curl, fj_set_curl,
    fj_set_spread, fj_set_stiffness, fj_to_json, new_finger_joint_state, Finger, FingerJointState,
};

pub mod forehead_width_control;
pub use forehead_width_control::{
    default_forehead_width_config, fw_blend, fw_effective_width, fw_reset, fw_set_bossing,
    fw_set_temple, fw_set_width, fw_to_json, fw_to_weights, new_forehead_width_state,
    ForeheadWidthConfig, ForeheadWidthState,
};

pub mod glabella_control;
pub use glabella_control::{
    apply_glabella_control, default_glabella_config, default_glabella_control, glabella_blend,
    glabella_clamp, glabella_reset, glabella_set_depth, glabella_set_height, glabella_set_width,
    glabella_to_json, glabella_to_weights, new_glabella_state, GlabellaConfig, GlabellaControl,
    GlabellaState,
};

pub mod hand_thickness_control;
pub use hand_thickness_control::{
    default_hand_thickness_config, ht_blend, ht_reset, ht_set_finger_girth, ht_set_left,
    ht_set_palm_width, ht_set_right, ht_to_json, ht_to_weights, new_hand_thickness_state,
    HandThicknessConfig, HandThicknessState,
};

pub mod intercanthal_control;
pub use intercanthal_control::{
    default_intercanthal_config, ic_blend, ic_effective_distance, ic_reset, ic_set_bridge_width,
    ic_set_distance, ic_set_tilt, ic_to_json, ic_to_weights, new_intercanthal_state,
    IntercanthalConfig, IntercanthalState,
};

pub mod jaw_depth_control;
pub use jaw_depth_control::{
    default_jaw_depth_config, jd_blend, jd_effective_depth, jd_reset, jd_set_angle, jd_set_depth,
    jd_set_ramus, jd_to_json, jd_to_weights, new_jaw_depth_state, JawDepthConfig, JawDepthState,
};

pub mod lip_philtrum_control;
pub use lip_philtrum_control::{
    default_lip_philtrum_config, lp_blend, lp_prominence, lp_reset, lp_set_depth, lp_set_length,
    lp_set_width, lp_to_json, lp_to_weights, new_lip_philtrum_state, LipPhiltrumConfig,
    LipPhiltrumState,
};

pub mod nasal_alar_control;
pub use nasal_alar_control::{
    default_nasal_alar_config, na_blend, na_overall_width, na_reset, na_set_flare,
    na_set_thickness, na_set_width, na_to_json, na_to_weights, new_nasal_alar_state,
    NasalAlarConfig, NasalAlarState,
};

pub mod neck_width_control;
pub use neck_width_control::{
    default_neck_width_config, new_neck_width_state, nw_blend, nw_circumference, nw_reset,
    nw_set_front_depth, nw_set_trapezius, nw_set_width, nw_to_json, nw_to_weights, NeckWidthConfig,
    NeckWidthState,
};

pub mod body_taper_control;
pub use body_taper_control::{
    bt_blend, bt_reset, bt_set_hip, bt_set_shoulder, bt_set_waist, bt_silhouette_area,
    bt_taper_ratio, bt_to_json, bt_to_weights, default_body_taper_config, new_body_taper_state,
    BodyTaperConfig, BodyTaperState,
};

pub mod brow_spacing_control;
pub use brow_spacing_control::{
    bs_arch_angle, bs_blend, bs_effective_left, bs_effective_right, bs_reset, bs_set_arch_offset,
    bs_set_spacing, bs_set_symmetry, bs_to_json, bs_to_weights, default_brow_spacing_config,
    new_brow_spacing_state, BrowSpacingConfig, BrowSpacingState,
};

pub mod cheek_fullness_control;
pub use cheek_fullness_control::{
    cf_blend, cf_effective_left, cf_effective_right, cf_reset, cf_set_fullness, cf_set_projection,
    cf_set_symmetry, cf_to_json, cf_to_weights, cf_volume_estimate, default_cheek_fullness_config,
    new_cheek_fullness_state, CheekFullnessConfig, CheekFullnessState,
};

pub mod chin_shape_control;
pub use chin_shape_control::{
    cs_blend, cs_profile_angle, cs_reset, cs_set_cleft, cs_set_projection, cs_set_vertical,
    cs_set_width, cs_to_json, cs_to_weights, default_chin_shape_config, new_chin_shape_state,
    ChinShapeConfig, ChinShapeState,
};

pub mod ear_concha_control;
pub use ear_concha_control::{
    default_ear_concha_config, ec_blend, ec_cavity_volume, ec_effective_left, ec_effective_right,
    ec_reset, ec_set_depth, ec_set_symmetry, ec_set_width, ec_to_json, ec_to_weights,
    new_ear_concha_state, EarConchaConfig, EarConchaState,
};

pub mod eye_lid_crease;
pub use eye_lid_crease::{
    default_eye_lid_crease_config, elc_blend, elc_crease_angle, elc_reset, elc_set_depth,
    elc_set_fold, elc_set_height, elc_set_symmetry, elc_to_json, elc_to_weights,
    new_eye_lid_crease_state, EyeLidCreaseConfig, EyeLidCreaseState,
};

pub mod face_roundness_control;
pub use face_roundness_control::{
    default_face_roundness_config, fr_blend, fr_overall_softness, fr_perimeter_estimate, fr_reset,
    fr_set_forehead_curve, fr_set_jaw_softness, fr_set_roundness, fr_to_json, fr_to_weights,
    new_face_roundness_state, FaceRoundnessConfig, FaceRoundnessState,
};

pub mod finger_tip_control;
pub use finger_tip_control::{
    default_finger_tip_config, ft_blend, ft_cross_section_area, ft_reset, ft_set_nail_length,
    ft_set_taper, ft_set_width, ft_to_json, ft_to_weights, new_finger_tip_state, FingerTipConfig,
    FingerTipState,
};

pub mod forehead_protrusion_control;
pub use forehead_protrusion_control::{
    default_forehead_protrusion_config, fp_blend, fp_reset, fp_set_bossing, fp_set_protrusion,
    fp_set_slope, fp_slope_angle, fp_to_json, fp_to_weights, new_forehead_protrusion_state,
    ForeheadProtrusionConfig, ForeheadProtrusionState,
};

pub mod hand_palm_control;
pub use hand_palm_control::{
    default_hand_palm_config, hp_blend, hp_cross_section, hp_reset, hp_set_arch, hp_set_thickness,
    hp_set_width, hp_to_json, hp_to_weights, new_hand_palm_state, HandPalmConfig, HandPalmState,
};

pub mod jaw_line_control;
pub use jaw_line_control::{
    default_jaw_line_config, jl_blend, jl_gonial_angle, jl_reset, jl_set_angle, jl_set_definition,
    jl_set_width, jl_sharpness, jl_to_json, jl_to_weights, new_jaw_line_state, JawLineConfig,
    JawLineState,
};

pub mod lip_thickness_control;
pub use lip_thickness_control::{
    apply_lip_thickness, default_lip_thickness, lip_thickness_blend, total_lip_volume, LipThickness,
};

pub mod nasal_septum_control;
pub use nasal_septum_control::{
    default_nasal_septum_config, new_nasal_septum_state, ns_angular_deviation_rad, ns_blend,
    ns_is_neutral, ns_reset, ns_set_depth, ns_set_deviation, ns_set_width, ns_to_json,
    ns_to_weights, NasalSeptumConfig, NasalSeptumState, NasalSeptumWeights,
};

pub mod neck_tendon_control;
pub use neck_tendon_control::{
    default_neck_tendon_config, new_neck_tendon_state, nt_asymmetry, nt_blend, nt_is_neutral,
    nt_reset, nt_set_atlas, nt_set_platysma, nt_set_scm_both, nt_set_scm_left, nt_set_scm_right,
    nt_to_json, nt_to_weights, NeckTendonConfig, NeckTendonState, NeckTendonWeights,
};

pub mod orbital_rim_control;
pub use orbital_rim_control::{
    apply_orbital_rim, default_orbital_rim, default_orbital_rim_config, new_orbital_rim_state,
    orb_mirror, orb_reset, orb_set_depth, orb_set_tilt, orb_to_json, orb_to_weights,
    orbital_rim_blend, OrbitalRim, OrbitalRimConfig, OrbitalRimState,
};

pub mod rib_cage_control;
pub use rib_cage_control::{
    default_rib_cage_config, new_rib_cage_state, rc_blend, rc_flare_angle_rad, rc_is_neutral,
    rc_reset, rc_set_barrel, rc_set_depth, rc_set_flare, rc_set_width, rc_to_json, rc_to_weights,
    RibCageConfig, RibCageState, RibCageWeights,
};

pub mod body_segment_control;
pub use body_segment_control::{
    blend_segment_states, default_body_segment_config, get_segment_scale, limb_length_m,
    new_body_segment_state, reset_all_segments, reset_segment, rhythm_scale,
    segment_angle_contribution, segment_name, set_segment_scale,
    state_to_json as segment_state_to_json, total_limb_scale, BodySegment, BodySegmentConfig,
    BodySegmentState, SegmentScale,
};

pub mod brow_tail_control;
pub use brow_tail_control::{
    bt_blend as brow_tail_blend, bt_is_neutral, bt_reset as brow_tail_reset, bt_set_angle,
    bt_set_raise, bt_symmetry, bt_to_json as brow_tail_to_json, bt_to_morph_weights,
    default_brow_tail_config, new_brow_tail_state, BrowTailConfig, BrowTailSide, BrowTailState,
};

pub mod cheek_jowl_control;
pub use cheek_jowl_control::{
    cj_blend, cj_is_neutral, cj_reset, cj_set_sag, cj_set_volume, cj_symmetry, cj_to_json,
    cj_to_weights, cj_total_volume, default_cheek_jowl_config, new_cheek_jowl_state,
    CheekJowlConfig, CheekJowlState, JowlSide,
};

pub mod chin_dimple_control;
pub use chin_dimple_control::{
    cd_area, cd_blend as chin_dimple_blend, cd_effective_depth as chin_dimple_effective_depth,
    cd_is_neutral, cd_reset as chin_dimple_reset, cd_set_depth, cd_set_vertical_offset,
    cd_set_width, cd_to_json as chin_dimple_to_json, cd_to_weights as chin_dimple_to_weights,
    default_chin_dimple_config, new_chin_dimple_state, ChinDimpleConfig, ChinDimpleState,
};

pub mod ear_cup_control;
pub use ear_cup_control::{
    default_ear_cup_config, ec_average_cup, ec_blend as ear_cup_blend, ec_is_neutral,
    ec_reset as ear_cup_reset, ec_set_bias, ec_set_cup, ec_symmetry, ec_to_json as ear_cup_to_json,
    ec_to_weights as ear_cup_to_weights, new_ear_cup_state, EarCupConfig, EarCupSide, EarCupState,
};

pub mod eye_tilt_control;
pub use eye_tilt_control::{
    default_eye_tilt_config, et_asymmetry, et_blend as eye_tilt_blend, et_is_neutral,
    et_reference_angle_rad, et_reset as eye_tilt_reset, et_set_tilt, et_tilt_rad_left,
    et_tilt_rad_right, et_to_json as eye_tilt_to_json, et_to_weights as eye_tilt_to_weights,
    new_eye_tilt_state, EyeTiltConfig, EyeTiltSide, EyeTiltState,
};

pub mod face_symmetry_control;
pub use face_symmetry_control::{
    default_face_symmetry_config, fs_blend, fs_circular_noise, fs_get_deviation, fs_is_symmetric,
    fs_reset, fs_set_deviation, fs_set_enforce, fs_to_json, fs_total_deviation,
    new_face_symmetry_state, AsymmetryAxis, AsymmetryEntry, FaceSymmetryConfig, FaceSymmetryState,
};

pub mod foot_width_control;
pub use foot_width_control::{
    default_foot_width_config, fw_blend as foot_width_blend, fw_reset as foot_width_reset,
    fw_set_arch, fw_set_forefoot, fw_set_heel, fw_symmetry, fw_to_json as foot_width_to_json,
    fw_to_weights as foot_width_to_weights, new_foot_width_state, FootSide, FootWidthConfig,
    FootWidthState,
};

pub mod forehead_vein_control;
pub use forehead_vein_control::{
    default_forehead_vein_config, fv_blend, fv_is_neutral, fv_reset, fv_set_central, fv_set_temple,
    fv_symmetry, fv_to_json, fv_to_weights, fv_total_prominence, new_forehead_vein_state,
    ForeheadVeinConfig, ForeheadVeinState,
};

pub mod hand_knuckle_control;
pub use hand_knuckle_control::{
    default_knuckle_config, kk_average_prominence, kk_blend, kk_is_neutral, kk_reset,
    kk_set_all_prominence, kk_set_definition, kk_set_prominence, kk_to_json, kk_to_weights,
    new_knuckle_state, KnuckleConfig, KnuckleState, FINGER_COUNT,
};

pub mod jaw_protrusion_control;
pub use jaw_protrusion_control::{
    default_jaw_protrusion_config, jp_blend, jp_horizontal_offset, jp_is_neutral, jp_reset,
    jp_set_lateral, jp_set_plane, jp_set_protrusion, jp_to_json, jp_to_weights,
    new_jaw_protrusion_state, JawProtrusionConfig, JawProtrusionState,
};

pub mod lip_bow_control;
pub use lip_bow_control::{
    default_lip_bow_config, lb_blend, lb_bow_width, lb_is_flat, lb_reset, lb_set_arch, lb_set_dip,
    lb_set_spread, lb_to_json, lb_to_weights, new_lip_bow_state, LipBowConfig, LipBowState,
};

pub mod nasal_flare_control;
pub use nasal_flare_control::{
    default_nasal_flare_config, new_nasal_flare_state, nf_average_flare, nf_blend, nf_is_neutral,
    nf_reset, nf_set_base_elevation, nf_set_flare, nf_symmetry, nf_to_json, nf_to_weights,
    NasalFlareConfig, NasalFlareSide, NasalFlareState,
};

pub mod neck_crease_control;
pub use neck_crease_control::{
    default_neck_crease_config, nc_average_depth, nc_blend, nc_is_neutral, nc_reset, nc_set_depth,
    nc_set_spread, nc_to_json, nc_to_weights, new_neck_crease_state, CreaseTier, NeckCreaseConfig,
    NeckCreaseState,
};

pub mod shoulder_slope_control;
pub use shoulder_slope_control::{
    default_shoulder_slope_config, new_shoulder_slope_state, ss_blend, ss_is_neutral, ss_reset,
    ss_set_height, ss_set_slope, ss_slope_rad, ss_symmetry, ss_to_json, ss_to_weights,
    ShoulderSide, ShoulderSlopeConfig, ShoulderSlopeState,
};

pub mod thigh_control;
pub use thigh_control::{
    default_thigh_config, new_thigh_state, th_average_girth, th_blend, th_is_neutral, th_reset,
    th_set_girth, th_set_inner, th_set_outer, th_symmetry, th_to_json, th_to_weights, ThighConfig,
    ThighSide, ThighState,
};

pub mod body_lean_control;
pub use body_lean_control::{
    bl_blend, bl_is_neutral, bl_reset, bl_sagittal_angle_rad, bl_set_backward, bl_set_forward,
    bl_set_lateral, bl_to_json, bl_to_weights, default_body_lean_config, new_body_lean_state,
    BodyLeanConfig, BodyLeanState,
};

pub mod brow_wrinkle_control;
pub use brow_wrinkle_control::{
    bw_blend as brow_wrinkle_blend, bw_intensity, bw_is_neutral, bw_reset as brow_wrinkle_reset,
    bw_set_arch, bw_set_horizontal, bw_set_vertical, bw_symmetry,
    bw_to_json as brow_wrinkle_to_json, bw_to_weights as brow_wrinkle_to_weights,
    default_brow_wrinkle_config, new_brow_wrinkle_state, BrowWrinkleConfig, BrowWrinkleState,
};

pub mod cheek_puff_depth;
pub use cheek_puff_depth::{
    cpd_average_depth, cpd_blend, cpd_is_neutral, cpd_reset, cpd_set_bias, cpd_set_both,
    cpd_set_left, cpd_set_right, cpd_symmetry, cpd_to_json, cpd_to_weights,
    default_cheek_puff_depth_config, new_cheek_puff_depth_state, CheekPuffDepthConfig,
    CheekPuffDepthState,
};

pub mod chin_recess_control;
pub use chin_recess_control::{
    cr_blend, cr_is_neutral, cr_net_offset, cr_reset, cr_set_protrusion, cr_set_recess,
    cr_set_vertical, cr_to_json, cr_to_weights, default_chin_recess_config, new_chin_recess_state,
    ChinRecessConfig, ChinRecessState,
};

pub mod ear_rim_control;
pub use ear_rim_control::{
    default_ear_rim_config, er_average_roll, er_blend, er_is_neutral, er_reset, er_set_both_roll,
    er_set_roll, er_set_sharpness, er_symmetry, er_to_json, er_to_weights, new_ear_rim_state,
    EarRimConfig, EarRimSide, EarRimState,
};

pub mod eye_inner_corner;
pub use eye_inner_corner::{
    default_eye_inner_corner_config, eic_average_depth, eic_blend, eic_is_neutral, eic_reset,
    eic_set_both_depth, eic_set_depth_left, eic_set_depth_right, eic_set_tilt, eic_symmetry,
    eic_to_json, eic_to_weights, new_eye_inner_corner_state, EyeInnerCornerConfig,
    EyeInnerCornerState,
};

pub mod face_contour_control;
pub use face_contour_control::{
    default_face_contour_config, fc_blend, fc_contour_intensity, fc_is_neutral, fc_reset,
    fc_set_mandible, fc_set_taper, fc_set_temporal, fc_set_zygomatic, fc_to_json, fc_to_weights,
    new_face_contour_state, FaceContourConfig, FaceContourState,
};

pub mod foot_toe_spread;
pub use foot_toe_spread::{
    default_foot_toe_spread_config, fts_average_spread, fts_blend, fts_is_neutral, fts_reset,
    fts_set_curl, fts_set_left_all, fts_set_right_all, fts_set_toe, fts_to_json, fts_to_weights,
    new_foot_toe_spread_state, FootToeSpreadConfig, FootToeSpreadState, TOE_COUNT,
};

pub mod forehead_raise_control;
pub use forehead_raise_control::{
    default_forehead_raise_config, fhr_average_raise, fhr_blend, fhr_is_neutral, fhr_reset,
    fhr_set_all, fhr_set_center, fhr_set_sides, fhr_set_tension, fhr_symmetry, fhr_to_json,
    fhr_to_weights, new_forehead_raise_state, ForeheadRaiseConfig, ForeheadRaiseState,
};

pub mod hand_vein_control;
pub use hand_vein_control::{
    default_hand_vein_config, hv_average_prominence, hv_blend, hv_is_neutral, hv_reset,
    hv_set_both, hv_set_branching, hv_set_left, hv_set_right, hv_symmetry, hv_to_json,
    hv_to_weights, new_hand_vein_state, HandVeinConfig, HandVeinState,
};

pub mod jaw_shift_control;
pub use jaw_shift_control::{
    default_jaw_shift_config, js_blend, js_displacement_magnitude, js_is_neutral, js_net_ap,
    js_reset, js_set_anterior, js_set_lateral, js_set_posterior, js_set_torsion, js_to_json,
    js_to_weights, new_jaw_shift_state, JawShiftConfig, JawShiftState,
};

pub mod nasal_root_control;
pub use nasal_root_control::{
    default_nasal_root_config, new_nasal_root_state, nr_blend, nr_bridge_prominence, nr_is_neutral,
    nr_reset, nr_set_depth, nr_set_height, nr_set_squish, nr_set_width, nr_to_json, nr_to_weights,
    NasalRootConfig, NasalRootState,
};

pub mod neck_flexion_control;
pub use neck_flexion_control::{
    default_neck_flexion_config, new_neck_flexion_state, nf_blend as neck_flex_blend,
    nf_is_neutral as neck_flex_is_neutral, nf_lateral_angle_rad, nf_reset as neck_flex_reset,
    nf_sagittal_angle_rad, nf_set_backward, nf_set_forward, nf_set_lateral,
    nf_to_json as neck_flex_to_json, nf_to_weights as neck_flex_to_weights, NeckFlexionConfig,
    NeckFlexionState,
};

pub mod scapula_control;
pub use scapula_control::{
    default_scapula_config, new_scapula_state, sc_average_wing, sc_blend, sc_is_neutral, sc_reset,
    sc_set_both_wing, sc_set_elevation, sc_set_wing, sc_symmetry, sc_to_json, sc_to_weights,
    ScapulaConfig, ScapulaSide, ScapulaState,
};

pub mod shin_control;
pub use shin_control::{
    default_shin_config, new_shin_state, shn_average_girth, shn_blend, shn_is_neutral, shn_reset,
    shn_set_both_girth, shn_set_curvature, shn_set_girth, shn_symmetry, shn_to_json,
    shn_to_weights, ShinConfig, ShinSide, ShinState,
};

pub mod body_twist_control;
pub use body_twist_control::{
    btwist_blend, btwist_is_neutral, btwist_reset, btwist_set_lower, btwist_set_upper,
    btwist_to_json, btwist_to_weights, btwist_total_angle_rad, default_body_twist_config,
    new_body_twist_state, BodyTwistConfig, BodyTwistState,
};

pub mod brow_furrowing_control;
pub use brow_furrowing_control::{
    bfw_blend, bfw_intensity, bfw_is_neutral, bfw_reset, bfw_set_inner, bfw_set_outer,
    bfw_set_vertical, bfw_to_json, bfw_to_weights, default_brow_furrow_config,
    new_brow_furrow_state, BrowFurrowConfig, BrowFurrowState,
};

pub mod cheek_rise_control;
pub use cheek_rise_control::{
    cr_average as cheek_rise_average, cr_blend as cheek_rise_blend,
    cr_is_neutral as cheek_rise_is_neutral, cr_reset as cheek_rise_reset,
    cr_set_both as cheek_rise_set_both, cr_set_rise, cr_symmetry as cheek_rise_symmetry,
    cr_to_json as cheek_rise_to_json, cr_to_weights as cheek_rise_to_weights,
    default_cheek_rise_config, new_cheek_rise_state, CheekRiseConfig, CheekRiseSide,
    CheekRiseState,
};

pub mod chin_recession_control;
pub use chin_recession_control::{
    chin_rec_blend, chin_rec_is_neutral, chin_rec_net_offset, chin_rec_reset,
    chin_rec_set_recession, chin_rec_set_tilt, chin_rec_set_vertical, chin_rec_to_json,
    chin_rec_to_weights, default_chin_recession_config, new_chin_recession_state,
    ChinRecessionConfig, ChinRecessionState,
};

pub mod ear_helix_fold;
pub use ear_helix_fold::{
    default_ear_helix_config, ehf_average_fold, ehf_blend, ehf_is_neutral, ehf_reset, ehf_set_both,
    ehf_set_definition, ehf_set_fold, ehf_symmetry, ehf_to_json, ehf_to_weights,
    new_ear_helix_state, EarHelixConfig, EarHelixSide, EarHelixState,
};

pub mod eye_outer_corner;
pub use eye_outer_corner::{
    default_eye_outer_corner_config, eoc_average_depth, eoc_blend, eoc_is_neutral, eoc_reset,
    eoc_set_both_depth, eoc_set_depth, eoc_set_tilt, eoc_symmetry, eoc_to_json, eoc_to_weights,
    new_eye_outer_corner_state, EyeOuterCornerConfig, EyeOuterCornerState, EyeOuterSide,
};

pub mod face_vertical_control;
pub use face_vertical_control::{
    default_face_vertical_config, fv_blend as fvc_blend, fv_is_neutral as fvc_is_neutral,
    fv_reset as fvc_reset, fv_set_all, fv_set_lower, fv_set_middle, fv_set_upper,
    fv_to_json as fvc_to_json, fv_to_weights as fvc_to_weights, fv_total_scale,
    new_face_vertical_state, FaceVerticalConfig, FaceVerticalState,
};

pub mod foot_ball_control;
pub use foot_ball_control::{
    default_foot_ball_config, fb_average_width, fb_blend, fb_is_neutral, fb_reset, fb_set_both,
    fb_set_padding, fb_set_width, fb_symmetry, fb_to_json, fb_to_weights, new_foot_ball_state,
    FootBallConfig, FootBallSide, FootBallState,
};

pub mod forehead_tension_control;
pub use forehead_tension_control::{
    default_forehead_tension_config, ften_average, ften_blend, ften_is_neutral, ften_reset,
    ften_set_all, ften_set_central, ften_set_compression, ften_set_lateral, ften_to_json,
    ften_to_weights, new_forehead_tension_state, ForeheadTensionConfig, ForeheadTensionState,
};

pub mod hand_grip_control;
pub use hand_grip_control::{
    default_hand_grip_config, hg_average_curl, hg_blend, hg_is_neutral, hg_reset, hg_set_both,
    hg_set_curl, hg_set_palm_compression, hg_symmetry, hg_to_json, hg_to_weights,
    new_hand_grip_state, HandGripConfig, HandGripSide, HandGripState,
};

pub mod jaw_twist_control;
pub use jaw_twist_control::{
    default_jaw_twist_config, jtwist_blend, jtwist_is_neutral, jtwist_reset, jtwist_set_lateral,
    jtwist_set_twist, jtwist_to_json, jtwist_to_weights, jtwist_total_displacement,
    new_jaw_twist_state, JawTwistConfig, JawTwistState,
};

pub mod lip_purse_control;
pub use lip_purse_control::{
    default_lip_purse_config, lpur_blend, lpur_intensity, lpur_is_neutral, lpur_reset,
    lpur_set_both, lpur_set_lower, lpur_set_protrusion, lpur_set_upper, lpur_to_json,
    lpur_to_weights, new_lip_purse_state, LipPurseConfig, LipPurseState,
};

pub mod nasal_spine_control;
pub use nasal_spine_control::{
    default_nasal_spine_config, new_nasal_spine_state, ns_blend as nsp_blend,
    ns_is_neutral as nsp_is_neutral, ns_prominence, ns_reset as nsp_reset, ns_set_angle,
    ns_set_projection, ns_set_width as nsp_set_width, ns_to_json as nsp_to_json,
    ns_to_weights as nsp_to_weights, NasalSpineConfig, NasalSpineState,
};

pub mod neck_tilt_control;
pub use neck_tilt_control::{
    default_neck_tilt_config, new_neck_tilt_state, ntilt_blend, ntilt_is_neutral, ntilt_reset,
    ntilt_set_lateral, ntilt_set_sagittal, ntilt_to_json, ntilt_to_weights, ntilt_total_angle_rad,
    NeckTiltConfig, NeckTiltState,
};

pub mod shoulder_pad_control;
pub use shoulder_pad_control::{
    default_shoulder_pad_config, new_shoulder_pad_state, spad_average_bulk, spad_blend,
    spad_is_neutral, spad_reset, spad_set_acromion, spad_set_both, spad_set_bulk, spad_symmetry,
    spad_to_json, spad_to_weights, ShoulderPadConfig, ShoulderPadSide, ShoulderPadState,
};

pub mod temple_control;
pub use temple_control::{
    apply_temple_control, default_temple_config, default_temple_control, new_temple_state,
    temple_blend, temple_mirror, temple_reset, temple_set_hollow, temple_set_prominence,
    temple_to_json, temple_to_weights, TempleConfig, TempleControl, TempleState,
};

pub mod body_volume_control;
pub use body_volume_control::{
    bvc_blend, bvc_estimated_volume, bvc_is_neutral, bvc_reset, bvc_set_abdomen, bvc_set_chest,
    bvc_set_volume, bvc_to_json, bvc_to_weights, default_body_volume_config, new_body_volume_state,
    BodyVolumeConfig, BodyVolumeState, BodyVolumeWeights,
};

pub mod brow_peak_control;
pub use brow_peak_control::{
    bp_arch_at, bp_asymmetry, bp_average, bp_blend, bp_is_neutral, bp_reset, bp_set_peak,
    bp_set_position, bp_to_json, bp_to_weights, default_brow_peak_config, new_brow_peak_state,
    BrowPeakConfig, BrowPeakSide, BrowPeakState,
};

pub mod cheek_tighten_control;
pub use cheek_tighten_control::{
    ct_asymmetry, ct_average, ct_blend, ct_is_neutral, ct_reset, ct_set_both, ct_set_left,
    ct_set_right, ct_set_vertical_bias, ct_to_json, ct_to_weights, default_cheek_tighten_config,
    new_cheek_tighten_state, CheekTightenConfig, CheekTightenState,
};

pub mod chin_pad_control;
pub use chin_pad_control::{
    cpd_blend as cpd_pad_blend, cpd_is_neutral as cpd_pad_is_neutral, cpd_pad_size,
    cpd_reset as cpd_pad_reset, cpd_set_projection, cpd_set_spread, cpd_set_volume,
    cpd_to_json as cpd_pad_to_json, cpd_to_weights as cpd_pad_to_weights, default_chin_pad_config,
    new_chin_pad_state, ChinPadConfig, ChinPadState,
};

pub mod ear_fold_control;
pub use ear_fold_control::{
    default_ear_fold_config, ef2_average_fold, ef2_blend, ef2_is_neutral, ef2_reset, ef2_set_both,
    ef2_set_definition, ef2_set_fold, ef2_symmetry, ef2_to_json, ef2_to_weights,
    new_ear_fold_state, EarFoldConfig, EarFoldSide, EarFoldState,
};

pub mod eye_squint_control;
pub use eye_squint_control::{
    default_eye_squint_config, esq_asymmetry, esq_average, esq_blend, esq_compression_angle,
    esq_is_neutral, esq_reset, esq_set_both, esq_set_inner, esq_set_left, esq_set_right,
    esq_to_json, esq_to_weights, new_eye_squint_state, EyeSquintConfig, EyeSquintState,
};

pub mod face_width_v2_control;
pub use face_width_v2_control::{
    default_face_width_v2_config, fw2_average_width, fw2_blend, fw2_is_neutral, fw2_reset,
    fw2_set_bigonial, fw2_set_bizygomatic, fw2_set_temporal, fw2_to_json, fw2_to_weights,
    new_face_width_v2_state, FaceWidthV2Config, FaceWidthV2State,
};

pub mod foot_heel_control;
pub use foot_heel_control::{
    default_foot_heel_config, fhc_is_neutral, fhc_pad, fhc_pad_asymmetry, fhc_reset,
    fhc_set_both_pad, fhc_set_calcaneus, fhc_set_pad, fhc_to_json, fhc_to_weights,
    new_foot_heel_state, FootHeelConfig, FootHeelEntry, FootHeelState, FootSide as HeelFootSide,
};

pub mod glabella_depth_control;
pub use glabella_depth_control::{
    default_glabella_depth_config, gd_blend, gd_is_neutral, gd_reset, gd_set_depth, gd_set_v_shift,
    gd_set_width, gd_slope_angle_rad, gd_to_json, gd_to_weights, new_glabella_depth_state,
    GlabellaDepthConfig, GlabellaDepthState,
};

pub mod hand_width_control;
pub use hand_width_control::{
    default_hand_width_config, hwc_asymmetry, hwc_blend, hwc_effective_width, hwc_is_neutral,
    hwc_reset, hwc_set_both, hwc_set_finger_spread, hwc_set_left, hwc_set_right, hwc_to_json,
    hwc_to_weights, new_hand_width_state, HandWidthConfig, HandWidthState,
};

pub mod jaw_rest_control;
pub use jaw_rest_control::{
    default_jaw_rest_config, jr_blend, jr_gap_rad, jr_is_neutral, jr_reset, jr_set_gap,
    jr_set_lateral, jr_set_relaxation, jr_to_json, jr_to_weights, new_jaw_rest_state,
    JawRestConfig, JawRestState,
};

pub mod lip_retract_control;
pub use lip_retract_control::{
    default_lip_retract_config, lrc_angle_rad, lrc_average, lrc_blend, lrc_is_neutral, lrc_reset,
    lrc_set_both, lrc_set_corners, lrc_set_retract, lrc_to_json, lrc_to_weights,
    new_lip_retract_state, LipRetractConfig, LipRetractState, LipSide,
};

pub mod nasal_width_control;
pub use nasal_width_control::{
    default_nasal_width_config, new_nasal_width_state, nwc_blend, nwc_effective_width,
    nwc_is_neutral, nwc_reset, nwc_set_alar_flare, nwc_set_bridge, nwc_set_width, nwc_to_json,
    nwc_to_weights, NasalWidthConfig, NasalWidthState,
};

pub mod neck_forward_control;
pub use neck_forward_control::{
    default_neck_forward_config, new_neck_forward_state, nfc_angle_rad, nfc_blend, nfc_is_neutral,
    nfc_reset, nfc_set_forward, nfc_set_lateral, nfc_set_protrusion, nfc_to_json, nfc_to_weights,
    NeckForwardConfig, NeckForwardState,
};

pub mod sternum_control;
pub use sternum_control::{
    default_sternum_config, new_sternum_state, stc_blend, stc_is_neutral, stc_reset,
    stc_set_length, stc_set_manubrium, stc_set_xiphoid_angle, stc_to_json, stc_to_weights,
    stc_xiphoid_angle_rad, SternumConfig, SternumState,
};

pub mod body_asymmetry_v2;
pub use body_asymmetry_v2::{
    bav2_angular_spread_rad, bav2_average_deviation, bav2_blend, bav2_is_neutral, bav2_reset,
    bav2_set_offset, bav2_to_json, bav2_to_weights, bav2_total_deviation,
    default_body_asymmetry_v2_config, new_body_asymmetry_v2_state, BodyAsymmetryV2Config,
    BodyAsymmetryV2State, REGION_COUNT as ASYM_REGION_COUNT,
};

pub mod brow_arch_height;
pub use brow_arch_height::{
    bah_arch_angle_rad, bah_asymmetry, bah_average, bah_blend, bah_is_neutral, bah_reset, bah_set,
    bah_set_both, bah_to_json, bah_to_weights, default_brow_arch_height_config,
    new_brow_arch_height_state, BrowArchHeightConfig, BrowArchHeightState,
    BrowSide as BrowArchSide,
};

pub mod cheek_nasal_fold;
pub use cheek_nasal_fold::{
    cnf_asymmetry, cnf_average, cnf_blend, cnf_fold_angle_rad, cnf_is_neutral, cnf_reset, cnf_set,
    cnf_set_both, cnf_to_json, cnf_to_weights, default_cheek_nasal_fold_config,
    new_cheek_nasal_fold_state, CheekNasalFoldConfig, CheekNasalFoldState, FoldSide,
};

pub mod chin_groove_control;
pub use chin_groove_control::{
    cg_angle_rad, cg_blend, cg_groove_area, cg_is_neutral, cg_reset, cg_set_depth, cg_set_width,
    cg_to_json, cg_to_weights, default_chin_groove_config, new_chin_groove_state, ChinGrooveConfig,
    ChinGrooveState,
};

pub mod ear_antihelix_control;
pub use ear_antihelix_control::{
    default_ear_antihelix_config, eah_average, eah_blend, eah_is_neutral, eah_reset,
    eah_ridge_angle_rad, eah_set, eah_set_both, eah_symmetry, eah_to_json, eah_to_weights,
    new_ear_antihelix_state, EarAntihelixConfig, EarAntihelixState, EarSide as AntihelixEarSide,
};

pub mod eye_fissure_control;
pub use eye_fissure_control::{
    default_eye_fissure_config, ef_asymmetry as fissure_ef_asymmetry,
    ef_average as fissure_ef_average, ef_blend as fissure_ef_blend,
    ef_is_neutral as fissure_ef_is_neutral, ef_opening_angle_rad, ef_reset as fissure_ef_reset,
    ef_set as fissure_ef_set, ef_set_both as fissure_ef_set_both, ef_to_json as fissure_ef_to_json,
    ef_to_weights as fissure_ef_to_weights, new_eye_fissure_state, EyeFissureConfig,
    EyeFissureState, EyeSide as FissureEyeSide,
};

pub mod face_depth_control;
pub use face_depth_control::{
    default_face_depth_config, fdc_average, fdc_blend, fdc_is_neutral, fdc_profile_angle_rad,
    fdc_range, fdc_reset, fdc_set_all, fdc_set_lower, fdc_set_middle, fdc_set_upper, fdc_to_json,
    fdc_to_weights, new_face_depth_state, FaceDepthConfig, FaceDepthState,
};

pub mod foot_toe_shape;
pub use foot_toe_shape::{
    default_foot_toe_shape_config, fts_average_length, fts_curl_angle_rad,
    fts_is_neutral as toe_shape_is_neutral, fts_reset as toe_shape_reset,
    fts_set_all as toe_shape_set_all, fts_set_curl as toe_shape_set_curl,
    fts_set_toe as toe_shape_set_toe, fts_to_json as toe_shape_to_json,
    fts_to_weights as toe_shape_to_weights, new_foot_toe_shape_state, FootSide as ToeShapeFootSide,
    FootToeShapeConfig, FootToeShapeState, TOE_COUNT as FOOT_TOE_COUNT,
};

pub mod forehead_globe_control;
pub use forehead_globe_control::{
    default_forehead_globe_config, fgl_average, fgl_blend, fgl_curvature_angle_rad, fgl_is_neutral,
    fgl_reset, fgl_set_both, fgl_set_central, fgl_set_lateral, fgl_to_json, fgl_to_weights,
    new_forehead_globe_state, ForeheadGlobeConfig, ForeheadGlobeState,
};

pub mod hand_metacarpal_control;
pub use hand_metacarpal_control::{
    default_hand_metacarpal_config, hmc_average, hmc_is_neutral, hmc_reset, hmc_set, hmc_set_all,
    hmc_span_angle_rad, hmc_to_json, hmc_to_weights, new_hand_metacarpal_state,
    HandMetacarpalConfig, HandMetacarpalState, HandSide as MetacarpalHandSide, MC_COUNT,
};

pub mod jaw_ramus_control;
pub use jaw_ramus_control::{
    default_jaw_ramus_config, jram_blend, jram_flare_angle_rad, jram_is_neutral, jram_ramus_area,
    jram_reset, jram_set_flare, jram_set_height, jram_to_json, jram_to_weights,
    new_jaw_ramus_state, JawRamusConfig, JawRamusState,
};

pub mod lip_cupid_control;
pub use lip_cupid_control::{
    default_lip_cupid_config, lc_blend, lc_bow_acuity, lc_is_neutral, lc_peak_angle_rad, lc_reset,
    lc_set_depth, lc_set_peak, lc_to_json, lc_to_weights, new_lip_cupid_state, LipCupidConfig,
    LipCupidState,
};

pub mod nasal_ala_crease;
pub use nasal_ala_crease::{
    default_nasal_ala_crease_config, nac_average, nac_blend, nac_crease_angle_rad, nac_is_neutral,
    nac_reset, nac_set, nac_set_both, nac_symmetry, nac_to_json, nac_to_weights,
    new_nasal_ala_crease_state, NasalAlaCreaseConfig, NasalAlaCreaseState, NasalSide,
};

pub mod neck_wattle_control;
pub use neck_wattle_control::{
    default_neck_wattle_config, new_neck_wattle_state, nwat_blend, nwat_is_neutral, nwat_reset,
    nwat_sag_angle_rad, nwat_set_sag, nwat_set_spread, nwat_to_json, nwat_to_weights,
    nwat_volume_estimate, NeckWattleConfig, NeckWattleState,
};

pub mod shoulder_acromion;
pub use shoulder_acromion::{
    default_shoulder_acromion_config, new_shoulder_acromion_state, sac_asymmetry, sac_average,
    sac_blend, sac_is_neutral, sac_prominence_angle_rad, sac_reset, sac_set, sac_set_both,
    sac_to_json, sac_to_weights, ShoulderAcromionConfig, ShoulderAcromionState,
    ShoulderSide as AcromionShoulderSide,
};

pub mod toe_control;
pub use toe_control::{
    default_toe_control_config, new_toe_control_state, tc_average_length, tc_is_neutral, tc_reset,
    tc_set_length, tc_set_splay, tc_splay_angle_rad, tc_to_json, tc_to_weights, ToeControlConfig,
    ToeControlState, ToeFootSide, TOE_COUNT as TOE_CTRL_COUNT,
};

pub mod trapezius_control;
pub use trapezius_control::{
    blend_trap_params, evaluate_trapezius, lower_trap_profile, middle_trap_profile,
    neck_slope_offset, shrug_corrective, upper_trap_profile, TrapeziusParams, TrapeziusResult,
};

pub mod body_center_control;
pub use body_center_control::{
    bcc_blend, bcc_displacement, bcc_is_neutral, bcc_lean_angle_rad, bcc_reset, bcc_set_ap,
    bcc_set_lateral, bcc_to_json, bcc_to_weights, default_body_center_config,
    new_body_center_state, BodyCenterConfig, BodyCenterState,
};

pub mod brow_lateral_control;
pub use brow_lateral_control::{
    blat_asymmetry, blat_average, blat_blend, blat_is_neutral, blat_reset, blat_rotation_rad,
    blat_set, blat_set_both, blat_to_json, blat_to_weights, default_brow_lateral_config,
    new_brow_lateral_state, BrowLateralConfig, BrowLateralState, BrowSide as BrowLateralSide,
};

pub mod cheek_sag_control;
pub use cheek_sag_control::{
    csag_asymmetry, csag_average, csag_blend, csag_is_neutral, csag_reset, csag_set, csag_set_both,
    csag_to_json, csag_to_weights, default_cheek_sag_config, new_cheek_sag_state, CheekSagConfig,
    CheekSagState, SagSide,
};

pub mod chin_flat_control;
pub use chin_flat_control::{
    cf_angle_rad, cf_blend as chin_flat_blend, cf_is_neutral as chin_flat_is_neutral,
    cf_reset as chin_flat_reset, cf_set_flatten, cf_set_v_bias, cf_to_json as chin_flat_to_json,
    cf_to_weights as chin_flat_to_weights, default_chin_flat_config, new_chin_flat_state,
    ChinFlatConfig, ChinFlatState,
};

pub mod ear_lobe_size;
pub use ear_lobe_size::{
    default_ear_lobe_size_config, els_blend, els_is_neutral, els_reset, els_set_both_size,
    els_set_droop, els_set_size, els_symmetry, els_to_json, els_to_weights,
    new_ear_lobe_size_state, EarLobeSizeConfig, EarLobeSizeState, EarSide,
};

pub mod eye_droop_control;
pub use eye_droop_control::{
    default_eye_droop_config, edr_asymmetry, edr_blend, edr_is_neutral, edr_lid_angle_rad,
    edr_reset, edr_set, edr_set_both, edr_to_json, edr_to_weights, new_eye_droop_state,
    EyeDroopConfig, EyeDroopState, EyeSide,
};

pub mod face_flatness_control;
pub use face_flatness_control::{
    default_face_flatness_config, ffl_blend, ffl_depth_scale, ffl_is_neutral, ffl_reset,
    ffl_set_flatness, ffl_set_mid, ffl_to_json, ffl_to_weights, new_face_flatness_state,
    FaceFlatnessConfig, FaceFlatnessState,
};

pub mod foot_instep_control;
pub use foot_instep_control::{
    default_foot_instep_config, fi_arch_angle_rad, fi_average_arch, fi_blend, fi_is_neutral,
    fi_reset, fi_set_arch, fi_set_both, fi_to_json, fi_to_weights, new_foot_instep_state,
    FootInstepConfig, FootInstepSide, FootInstepState,
};

pub mod forehead_crease_control;
pub use forehead_crease_control::{
    default_forehead_crease_config, fhc_blend, fhc_effective_depth, fhc_intensity,
    fhc_is_neutral as forehead_crease_is_neutral, fhc_reset as forehead_crease_reset,
    fhc_set_depth, fhc_set_lines, fhc_set_spread, fhc_to_json as forehead_crease_to_json,
    new_forehead_crease_state, ForeheadCreaseConfig, ForeheadCreaseState,
};

pub mod hand_finger_splay;
pub use hand_finger_splay::{
    default_finger_splay_config, hfs_average_angle_rad, hfs_blend, hfs_finger_angle_rad,
    hfs_is_neutral, hfs_reset, hfs_set_all, hfs_set_finger, hfs_to_json, new_finger_splay_state,
    FingerSplayConfig, FingerSplayState, HandSide, FINGER_COUNT as FINGER_SPLAY_COUNT,
};

pub mod jaw_clench_control;
pub use jaw_clench_control::{
    default_jaw_clench_config, jcl_asymmetry, jcl_bite_force, jcl_blend, jcl_is_neutral, jcl_reset,
    jcl_set, jcl_set_both, jcl_temporal_angle_rad, jcl_to_json, jcl_to_weights,
    new_jaw_clench_state, ClenchSide, JawClenchConfig, JawClenchState,
};

pub mod lip_line_control;
pub use lip_line_control::{
    default_lip_line_config, ll_blend, ll_bow_angle_rad, ll_is_neutral, ll_reset, ll_set_lower_def,
    ll_set_upper_bow, ll_set_width_scale, ll_to_json, ll_to_weights, new_lip_line_state,
    LipLineConfig, LipLineState,
};

pub mod nasal_saddle_control;
pub use nasal_saddle_control::{
    default_nasal_saddle_config, new_nasal_saddle_state, nsd_blend, nsd_is_neutral, nsd_reset,
    nsd_root_angle_rad, nsd_set_depth, nsd_set_v_shift, nsd_set_width, nsd_to_json, nsd_to_weights,
    NasalSaddleConfig, NasalSaddleState,
};

pub mod neck_sterno_control;
pub use neck_sterno_control::{
    default_neck_sterno_config, new_neck_sterno_state, nst_asymmetry, nst_blend, nst_is_neutral,
    nst_pull_angle_rad, nst_reset, nst_set, nst_set_both, nst_set_head_rotation, nst_to_json,
    nst_to_weights, NeckSternoConfig, NeckSternoState, ScmSide,
};

pub mod shoulder_roll_control;
pub use shoulder_roll_control::{
    default_shoulder_roll_config, new_shoulder_roll_state, shr_angle_rad, shr_asymmetry, shr_blend,
    shr_is_neutral, shr_reset, shr_set, shr_set_both, shr_to_json, shr_to_weights, RollSide,
    ShoulderRollConfig, ShoulderRollState,
};

pub mod thigh_girth_control;
pub use thigh_girth_control::{
    default_thigh_girth_config, new_thigh_girth_state, tg_blend, tg_circumference, tg_is_neutral,
    tg_reset, tg_set_both, tg_set_girth, tg_set_medial, tg_symmetry, tg_to_json, tg_to_weights,
    ThighGirthConfig, ThighGirthSide, ThighGirthState,
};

pub mod finger_spread_control;
pub use finger_spread_control::{
    blend_finger_spread, clamp_spread, effective_spread, evaluate_finger_spread, finger_from_index,
    max_spread, preset_relaxed, preset_wide, web_stretch_weight, Finger as FingerSpread,
    FingerSpreadParams, FingerSpreadResult,
};

pub mod spine_curve_control;
pub use spine_curve_control::{
    default_spine_curve_config, new_spine_curve_state, scc_blend, scc_is_neutral,
    scc_kyphosis_angle_rad, scc_lordosis_angle_rad, scc_reset, scc_scoliosis_angle_rad,
    scc_set_kyphosis, scc_set_lordosis, scc_set_scoliosis, scc_to_json, scc_to_weights,
    scc_total_curvature, SpineCurveConfig, SpineCurveState,
};

pub mod pelvis_tilt_control;
pub use pelvis_tilt_control::{
    default_pelvis_tilt_config, new_pelvis_tilt_state, pt_blend, pt_frontal_angle_rad,
    pt_is_neutral, pt_magnitude, pt_reset, pt_sagittal_angle_rad, pt_set_frontal, pt_set_sagittal,
    pt_to_json, pt_to_weights, PelvisTiltConfig, PelvisTiltState,
};

pub mod clavicle_control;
pub use clavicle_control::{
    blend_clavicle_states, clavicle_to_json, compute_clavicle_weights, default_clavicle_config,
    new_clavicle_state, set_clavicle_angle, set_clavicle_length, set_clavicle_offset,
    set_clavicle_prominence, ClavicleConfig, ClavicleMorphWeights, ClavicleState,
};

pub mod wrist_control;
pub use wrist_control::{
    blend_wrist_params, deviation_weight, evaluate_wrist, flexion_weight, styloid_bump,
    tendon_ridge, WristParams, WristResult,
};

pub mod ankle_control;
pub use ankle_control::{
    achilles_groove, blend_ankle_params, corrective_weight as ankle_corrective_weight,
    dorsiflexion_corrective, evaluate_ankle_morph, malleolus_bump, plantarflexion_corrective,
    thickness_scale, AnkleCorrective, AnkleParams,
};

pub mod expression_randomizer;
pub use expression_randomizer::{
    blend_sampled, dominant_channel, expression_energy as randomizer_expression_energy,
    expression_to_json as randomizer_expression_to_json,
    normalize_expression as randomizer_normalize_expression, sample_expression,
    sample_sparse_expression, ExpressionRandomizerConfig,
};

pub mod muscle_group_driver;
pub use muscle_group_driver::{
    mgd_active_count, mgd_blend, mgd_bulge_weight, mgd_get, mgd_reset, mgd_set, mgd_to_json,
    mgd_total_activation, new_muscle_group_driver, MuscleActivation,
    MuscleGroup as MuscleBodyGroup, MuscleGroupDriver,
};

pub mod skin_fold_control;
pub use skin_fold_control::{
    default_skin_fold_config, new_skin_fold_state, sf_active_count, sf_blend, sf_depth_m, sf_get,
    sf_is_neutral, sf_reset, sf_set, sf_to_json, sf_width_m, FoldSite, SkinFoldConfig,
    SkinFoldState,
};

pub mod brow_asymmetry;
pub use brow_asymmetry::{
    ba_blend, ba_furrow_depth, ba_is_neutral, ba_lift_angle_rad, ba_lift_asymmetry, ba_reset,
    ba_set_furrow, ba_set_lift, ba_to_json, ba_to_weights, default_brow_asymmetry_config,
    new_brow_asymmetry_state, BrowAsymmetryConfig, BrowAsymmetryState, BrowSide as BrowAsymSide,
};

pub mod cheek_hollow_control;
pub use cheek_hollow_control::{
    blend_cheek_hollow, cheek_hollow_intensity, default_cheek_hollow, evaluate_cheek_hollow,
    is_valid_cheek_hollow, reset_cheek_hollow, set_cheek_hollow_depth, CheekHollowParams,
    CheekHollowResult,
};

pub mod philtrum_control;
pub use philtrum_control::{
    apply_philtrum_control, default_philtrum_config, default_philtrum_control, new_philtrum_state,
    philtrum_blend, philtrum_clamp, philtrum_reset, philtrum_set_depth, philtrum_set_length,
    philtrum_set_width, philtrum_to_json, philtrum_to_weights, PhiltrumConfig, PhiltrumControl,
    PhiltrumState,
};

pub mod nasolabial_fold_control;
pub use nasolabial_fold_control::{
    default_nasolabial_fold_config, new_nasolabial_fold_state, nlf_asymmetry, nlf_blend, nlf_depth,
    nlf_is_neutral, nlf_reset, nlf_set_both, nlf_set_depth, nlf_set_length, nlf_to_json,
    nlf_to_weights, NasolabialFoldConfig, NasolabialFoldState, NlSide,
};

pub mod brow_ridge_control;
pub use brow_ridge_control::{
    blend_brow_ridge, clamp_morph, default_brow_ridge, evaluate_brow_ridge, is_valid_ridge,
    ridge_intensity, set_prominence, set_ridge_width, BrowRidgeParams, BrowRidgeResult,
};

pub mod cheekbone_v2;
pub use cheekbone_v2::{
    blend_cheekbone_v2, cheekbone_v2_to_json, default_cheekbone_v2, evaluate_cheekbone_v2,
    is_valid_cheekbone_v2, reset_cheekbone_v2, set_arch_width, set_zygomatic_prominence,
    CheekboneV2Params, CheekboneV2Weights,
};

pub mod alar_base_control;
pub use alar_base_control::{
    alar_base_to_json, blend_alar_base, default_alar_base, evaluate_alar_base, is_valid_alar_base,
    reset_alar_base, set_alar_flare, set_alar_width, AlarBaseParams, AlarBaseWeights,
};

pub mod columella_control;
pub use columella_control::{
    blend_columella, columella_to_json, default_columella, evaluate_columella, is_valid_columella,
    reset_columella, set_columella_angle, set_columella_length, set_columella_width,
    ColumellaParams, ColumellaWeights,
};

pub mod neck_thickness_control;
pub use neck_thickness_control::{
    default_neck_thickness_config, neck_clamp, neck_compute_volume, neck_reset, neck_set_depth,
    neck_set_length, neck_set_width, neck_to_json, neck_to_weights, new_neck_thickness_state,
    NeckThicknessConfig, NeckThicknessState,
};

pub mod breast_shape_control;
pub use breast_shape_control::{
    blend_breast_shape, breast_shape_to_json, default_breast_shape, evaluate_breast_shape,
    is_valid_breast_shape, reset_breast_shape, set_breast_ptosis, set_breast_volume,
    BreastShapeParams, BreastShapeWeights,
};

pub mod waist_control;
pub use waist_control::{
    apply_waist_control, default_waist_config, default_waist_control, new_waist_control,
    new_waist_state, set_waist_depth, set_waist_width, waist_circumference_approx, waist_clamp,
    waist_compute_circumference, waist_control_blend, waist_from_param, waist_ratio_to_hip,
    waist_reset, waist_set_depth, waist_set_width, waist_to_json, waist_to_param, waist_to_weights,
    WaistConfig, WaistControl, WaistState,
};

pub mod hip_width_control;
pub use hip_width_control::{
    apply_hip_width, default_hip_width, hip_circumference_approx, hip_ratio, hip_width_blend,
    HipWidth,
};

pub mod belly_shape_control;
pub use belly_shape_control::{
    belly_shape_to_json, belly_volume_factor, blend_belly_shape, default_belly_shape,
    evaluate_belly_shape, is_valid_belly_shape, reset_belly_shape, set_belly_bloat,
    set_belly_convexity, BellyShapeParams, BellyShapeWeights,
};

pub mod gluteal_control;
pub use gluteal_control::{
    blend_gluteal, default_gluteal, evaluate_gluteal, gluteal_area_index, gluteal_to_json,
    is_valid_gluteal, reset_gluteal, set_gluteal_lift, set_gluteal_projection, set_gluteal_volume,
    GlutealParams, GlutealWeights,
};

pub mod calf_control;
pub use calf_control::{
    apply_calf_control, calf_control_blend, calf_mirror, calf_reset, calf_set_definition,
    calf_set_muscle, calf_to_json, calf_to_weights, default_calf_config, default_calf_control,
    new_calf_state, CalfConfig, CalfControl, CalfState,
};

pub mod bicep_control;
pub use bicep_control::{
    belly_profile, blend_bicep_params, evaluate_bicep, flexion_corrective, head_separation_groove,
    mirror_bicep, peak_profile, vein_displacement, BicepParams, BicepResult,
};

pub mod forearm_control;
pub use forearm_control::{
    blend_forearm_states, compute_forearm_weights, default_forearm_config, forearm_to_json,
    new_forearm_state, set_forearm_muscle, set_forearm_pronation, set_forearm_taper,
    set_vein_visibility, ForearmConfig, ForearmMorphWeights, ForearmState,
};

pub mod finger_length_control;
pub use finger_length_control::{
    blend_finger_lengths, compute_finger_length_weights, default_finger_length_config,
    finger_length_to_json, new_finger_length_state, set_finger_knuckle_size,
    set_finger_overall_length, set_finger_taper, set_finger_thickness, FingerLengthConfig,
    FingerLengthState, FingerLengthWeights,
};

pub mod knee_control;
pub use knee_control::{
    blend_knee_params, evaluate_knee, fat_pad_profile,
    flexion_corrective as knee_flexion_corrective, patella_profile, popliteal_profile,
    skin_fold_weight, KneeParams, KneeResult,
};

pub mod foot_arch_control;
pub use foot_arch_control::{
    blend_foot_arches, compute_foot_arch_weights, default_foot_arch_config, foot_arch_to_json,
    new_foot_arch_state, set_foot_arch_height, set_foot_arch_length, set_foot_arch_stiffness,
    set_foot_pronation, FootArchConfig, FootArchState, FootArchWeights,
};

// toe_control already registered above.

pub mod eye_depth_control;
pub use eye_depth_control::{
    default_eye_depth_config, ed_clamp, ed_mirror, ed_reset, ed_set_depth_both, ed_set_depth_left,
    ed_set_depth_right, ed_to_json, ed_to_weights, new_eye_depth_state, EyeDepthConfig,
    EyeDepthState,
};

pub mod eye_spacing_control;
pub use eye_spacing_control::{
    blend_eye_spacings, compute_eye_spacing_weights, default_eye_spacing_config,
    eye_spacing_to_json, new_eye_spacing_state, set_eye_convergence, set_eye_depth,
    set_eye_distance, set_eye_vertical_offset, EyeSpacingConfig, EyeSpacingState,
    EyeSpacingWeights,
};

// eye_tilt_control already registered above.

pub mod gum_line_control;
pub use gum_line_control::{
    blend_gum_line_controls, compute_gum_line_control_weights, default_gum_line_control_config,
    gum_line_control_to_json, new_gum_line_control_state, set_gum_line_control_curvature,
    set_gum_line_control_exposure, set_gum_line_control_recession, set_gum_line_control_width,
    GumLineControlConfig, GumLineControlState, GumLineControlWeights,
};

pub mod pupil_size_control;
pub use pupil_size_control::{
    blend_pupil_size, default_pupil_size_params, pupil_area_fraction, pupil_asymmetry,
    pupil_circumference_ratio, pupil_from_light, pupil_radius_fraction, pupil_size_to_json,
    reset_pupil_size, set_pupil_dilation_both, set_pupil_dilation_left, set_pupil_dilation_right,
    PupilSizeParams,
};

pub mod iris_color_blend;
pub use iris_color_blend::{
    apply_preset, blend_iris_color, iris_color_blend_to_json, iris_luminance, preset_rgb,
    reset_iris_color, set_heterochromia, set_limbal_ring, IrisColorBlendParams, IrisColorPreset,
};

pub mod sclera_tone_control;
pub use sclera_tone_control::{
    apply_sclera_tone, blend_sclera_tone, default_sclera_tone_params, is_sclera_healthy,
    reset_sclera_tone, sclera_tone_to_json, set_sclera_brightness, set_sclera_redness,
    set_sclera_vein_visibility, set_sclera_yellowing, ScleraToneParams,
};

pub mod eyelash_density;
pub use eyelash_density::{
    blend_eyelash_density, curl_tip_offset, default_eyelash_density_params,
    eyelash_density_to_json, reset_eyelash_density, set_lash_curl, set_lash_darkness,
    set_lash_length, set_lash_thickness, set_lower_lash_count, set_upper_lash_count,
    total_lash_count, EyelashDensityParams,
};

pub mod eyebrow_shape_library;
pub use eyebrow_shape_library::{
    all_presets, blend_brow_shape, brow_height_at, brow_shape_to_json, preset_params,
    reset_brow_shape, BrowShapeParams, BrowShapePreset,
};

pub mod lip_color_zone;
pub use lip_color_zone::{
    apply_lip_zone_color, blend_lip_color_zone, default_lip_color_zone_params,
    lip_color_zone_to_json, reset_lip_color_zone, set_lip_desaturation, set_lip_gloss,
    set_lip_zone, LipColorZoneParams, LipZone,
};

pub mod tooth_shape_control;
pub use tooth_shape_control::{
    blend_tooth_shape, default_tooth_shape_params, reset_tooth_shape, set_tooth_crowding,
    set_tooth_height, set_tooth_overbite, set_tooth_rounding, set_tooth_whiteness, set_tooth_width,
    tooth_color_rgb, tooth_shape_to_json, ToothShapeParams,
};

pub mod tongue_shape_v2;
pub use tongue_shape_v2::{
    blend_tongue_shape_v2, default_tongue_shape_v2_params, reset_tongue_shape_v2, set_dorsum_arch,
    set_tip_curl, set_tip_pointedness, set_tongue_protrusion, tongue_height_profile,
    tongue_shape_v2_to_json, tongue_width_profile, TongueShapeV2Params,
};

pub mod body_symmetry_v2;
pub use body_symmetry_v2::{
    apply_symmetry, are_mirror_pair, body_symmetry_v2_to_json, default_body_symmetry_v2_params,
    mirror_position as body_mirror_position, reset_body_symmetry_v2, set_symmetry_weight,
    symmetrize_position, BodySymmetryV2Params, SymmetryAxis as BodySymAxis,
};

pub mod thigh_v2;
pub use thigh_v2::{
    default_thigh_v2_config, new_thigh_v2_state, tv2_average_girth, tv2_blend, tv2_is_neutral,
    tv2_reset, tv2_set_inner, tv2_set_outer, tv2_set_symmetry, tv2_to_json, tv2_to_weights,
    ThighV2Config, ThighV2State, ThighV2Weights,
};

pub mod hand_v2;
pub use hand_v2::{
    default_hand_v2_params, hv2_blend, hv2_is_neutral, hv2_reset, hv2_set_dorsum, hv2_set_knuckle,
    hv2_set_tendon, hv2_set_vein, hv2_surface_detail_estimate, hv2_to_json, HandV2Params,
};

pub mod palm_control;
pub use palm_control::{
    default_palm_params, palm_blend, palm_is_neutral, palm_reset, palm_set_arch,
    palm_set_thickness, palm_set_width, palm_surface_area_estimate, palm_to_json, palm_to_weights,
    PalmParams, PalmWeights,
};

pub mod thumb_control;
pub use thumb_control::{
    default_thumb_params, thumb_blend, thumb_is_neutral, thumb_opposition_angle_deg, thumb_reset,
    thumb_set_curvature, thumb_set_girth, thumb_set_length_scale, thumb_set_opposition,
    thumb_to_json, ThumbParams,
};

pub mod nail_shape_control;
pub use nail_shape_control::{
    default_nail_shape_params, nail_blend, nail_is_neutral, nail_reset, nail_set_curvature,
    nail_set_length, nail_set_shape, nail_set_width_scale, nail_sharpness_index, nail_to_json,
    NailShape, NailShapeParams,
};

pub mod skin_roughness;
pub use skin_roughness::{
    default_skin_roughness_params, sr_blend, sr_effective_roughness, sr_is_specular_dominant,
    sr_reset, sr_set_anisotropy, sr_set_micro_roughness, sr_set_roughness, sr_set_scale,
    sr_to_json, SkinRoughnessParams,
};

pub mod skin_pore_control;
pub use skin_pore_control::{
    default_skin_pore_params, sp_blend, sp_is_neutral, sp_pore_visibility, sp_reset,
    sp_set_density, sp_set_depth, sp_set_size, sp_set_variation, sp_to_json, SkinPoreParams,
};

pub mod wrinkle_depth_control;
pub use wrinkle_depth_control::{
    wd_blend, wd_is_neutral, wd_reset, wd_set_density, wd_set_depth, wd_set_softness, wd_to_json,
    wd_visibility, WrinkleDepthParams, WrinkleZone,
};

pub mod scar_morph;
pub use scar_morph::{
    default_scar_morph_params, scar_blend, scar_is_neutral, scar_reset, scar_set_length,
    scar_set_prominence, scar_set_roughness, scar_set_width, scar_to_json, scar_visibility,
    ScarMorphParams, ScarType,
};

pub mod tattoo_map_control;
pub use tattoo_map_control::{
    default_tattoo_params, tattoo_blend, tattoo_disable, tattoo_effective_opacity, tattoo_enable,
    tattoo_reset, tattoo_set_offset, tattoo_set_opacity, tattoo_set_rotation, tattoo_set_scale,
    tattoo_to_json, TattooParams, TattooRegion,
};

pub mod hair_thickness_control;
pub use hair_thickness_control::{
    default_hair_thickness_params, ht_blend as hair_ht_blend, ht_effective_diameter_um,
    ht_is_neutral, ht_reset as hair_ht_reset, ht_set_medulla, ht_set_root_taper, ht_set_shaft,
    ht_set_tip_taper, ht_to_json as hair_ht_to_json, HairThicknessParams,
};

pub mod hair_curl_control;
pub use hair_curl_control::{
    default_hair_curl_params, hc_blend, hc_effective_curl, hc_is_straight, hc_reset,
    hc_set_frequency, hc_set_pattern, hc_set_strength, hc_to_json, CurlPattern, HairCurlParams,
};

pub mod beard_density_control;
pub use beard_density_control::{
    bd_average_density, bd_is_clean_shaven, bd_reset, bd_set_global, bd_set_zone_density,
    bd_set_zone_length, bd_to_json, default_beard_density_state, BeardDensityState, BeardZone,
    BeardZoneEntry,
};

pub mod eyebrow_thickness_control;
pub use eyebrow_thickness_control::{
    default_eyebrow_thickness_params, ebt_asymmetry, ebt_average_thickness, ebt_blend,
    ebt_is_neutral, ebt_reset, ebt_set_density, ebt_set_fullness, ebt_set_thickness, ebt_to_json,
    EyebrowSide, EyebrowThicknessParams,
};

pub mod body_hair_control;
pub use body_hair_control::{
    bh_average_density, bh_is_smooth, bh_reset, bh_set_global, bh_set_region_density,
    bh_set_region_length, bh_to_json, default_body_hair_state, BodyHairEntry, BodyHairRegion,
    BodyHairState,
};

pub mod freckle_map_control;
pub use freckle_map_control::{
    default_freckle_params, fm_blend, fm_effective_density, fm_is_neutral, fm_reset,
    fm_set_body_coverage, fm_set_darkness, fm_set_density, fm_set_distribution,
    fm_set_face_coverage, fm_set_size, fm_set_sun_exposure, fm_to_json, FreckleDistribution,
    FreckleParams,
};

pub mod corrective_pose_driver;
pub use corrective_pose_driver::{
    evaluate_pose_driver, new_corrective_pose_driver, pose_driver_to_json, reset_pose_driver,
    set_pose_driver_threshold, CorrectivePoseDriver, CorrectivePoseDriverConfig,
};

pub mod sdk_driven_shape;
pub use sdk_driven_shape::{
    new_sdk_driven_shape, sdk_add_point, sdk_evaluate, sdk_point_count, sdk_reset, sdk_to_json,
    SdkCurvePoint, SdkDrivenShape,
};

pub mod proximity_wrap;
pub use proximity_wrap::{
    bake_proximity_weights, new_proximity_wrap, proximity_average_weight, proximity_influence,
    proximity_vertex_count, proximity_wrap_to_json, ProximityWrap, ProximityWrapConfig,
};

pub mod lattice_morph;
pub use lattice_morph::{
    lattice_get_point, lattice_point_count, lattice_set_point, lattice_set_weight, lattice_to_json,
    new_lattice_morph, LatticeDims, LatticeMorph,
};

pub mod cage_morph;
pub use cage_morph::{
    cage_set_vertex, cage_set_weight, cage_to_json, cage_total_delta_magnitude, cage_vertex_count,
    new_cage_morph, CageMorph, CageVertex,
};

pub mod delta_mush;
pub use delta_mush::{
    delta_mush_reset, delta_mush_set_smoothing, delta_mush_smooth, delta_mush_to_json,
    delta_mush_vertex_count, new_delta_mush, DeltaMush, DeltaMushConfig,
};

pub mod proximity_pin;
pub use proximity_pin::{
    new_pin_set, pin_add, pin_count, pin_enabled_count, pin_remove, pin_set_enabled,
    pin_set_influence, pin_set_to_json, ProximityPin, ProximityPinSet,
};

pub mod surface_deform;
pub use surface_deform::{
    new_surface_deform, surface_deform_bary_sum, surface_deform_bind, surface_deform_set_strength,
    surface_deform_to_json, surface_deform_unbind, surface_deform_vertex_count, SurfaceDeform,
    SurfaceDeformBinding,
};

pub mod mesh_deform_morph;
pub use mesh_deform_morph::{
    mdm_bind, mdm_set_weight, mdm_to_json, mdm_unbind, mdm_validate_weights, mdm_vertex_count,
    new_mesh_deform_morph, MeshDeformBinding, MeshDeformMorph,
};

pub mod inbetween_shape;
pub use inbetween_shape::{
    inbetween_evaluate, inbetween_reset, inbetween_set_delta, inbetween_to_json,
    inbetween_vertex_count, new_inbetween_shape, InbetweenShape,
};

pub mod pose_space_deform;
pub use pose_space_deform::{
    new_psd, psd_add_example, psd_evaluate, psd_example_count, psd_reset, psd_to_json,
    psd_vertex_count, PoseSpaceDeform, PsdExample,
};

pub mod rbf_deformer;
pub use rbf_deformer::{
    new_rbf_deformer, rbf_add_control_point, rbf_evaluate, rbf_kernel_value, rbf_point_count,
    rbf_to_json, RbfControlPoint, RbfDeformer, RbfKernel,
};

pub mod linear_blend_skin;
pub use linear_blend_skin::{
    lbs_add_influence, lbs_influence_count, lbs_is_normalized, lbs_normalize, lbs_to_json,
    lbs_vertex_count, new_lbs, LbsVertex, LinearBlendSkin, SkinInfluence,
};

pub mod dual_quaternion_skin;
pub use dual_quaternion_skin::{
    dqs_bone_count, dqs_normalize, dqs_set_bone, dqs_set_vertex, dqs_to_json, dqs_vertex_count,
    new_dqs, DqsVertex, DualQuat, DualQuaternionSkin,
};

pub mod omega_skin;
pub use omega_skin::{
    new_omega_skin, omega_effective_weight, omega_set_blend, omega_set_mode, omega_to_json,
    omega_vertex_count, OmegaMode, OmegaSkin, OmegaVertex,
};

pub mod fast_lbs;
pub use fast_lbs::{
    fast_lbs_is_valid, fast_lbs_normalize, fast_lbs_set, fast_lbs_to_json, fast_lbs_transform,
    fast_lbs_vertex_count, new_fast_lbs, FastLbs, FastLbsRecord,
};

pub mod neural_blend_shape;
pub use neural_blend_shape::{
    nbs_forward, nbs_load_weights, nbs_set_activation, nbs_set_enabled, nbs_to_json,
    new_neural_blend_shape, NbsActivation, NeuralBlendShape,
};

pub mod learned_corrective;
pub use learned_corrective::{
    lc_add_entry, lc_entry_count, lc_evaluate, lc_set_enabled,
    lc_to_json as learned_corrective_to_json, new_learned_corrective, CorrectiveEntry,
    LearnedCorrective,
};

pub mod data_driven_rig;
pub use data_driven_rig::{
    ddr_add_sample, ddr_clear_samples, ddr_evaluate, ddr_sample_count, ddr_set_enabled,
    ddr_to_json, new_data_driven_rig, DataDrivenRig, RigSample,
};

pub mod example_based_morph;
pub use example_based_morph::{
    ebm_add_example, ebm_evaluate, ebm_example_count, ebm_set_enabled, ebm_to_json,
    ebm_vertex_count, new_example_based_morph, ExampleBasedMorph, ExamplePose,
};

pub mod compressed_shape_key;
pub use compressed_shape_key::{
    csk_byte_size, csk_decode_delta, csk_set_enabled, csk_set_quant_bits, csk_set_scale,
    csk_to_json, new_compressed_shape_key, CompressedShapeKey, QuantBits,
};

pub mod sparse_blend_shape;
pub use sparse_blend_shape::{
    new_sparse_blend_shape, sbs_add_delta, sbs_apply, sbs_delta_count, sbs_set_enabled,
    sbs_set_weight, sbs_to_json, SparseBlendShape, SparseDelta,
};

pub mod gpu_morph_target;
pub use gpu_morph_target::{
    gmt_mark_dirty, gmt_set_enabled, gmt_set_weight, gmt_to_json, gmt_upload, new_gpu_morph_target,
    GpuMorphTarget, GpuUploadState,
};

pub mod streaming_morph;
pub use streaming_morph::{
    new_streaming_morph, sm_enqueue, sm_queue_len, sm_set_enabled, sm_tick, sm_to_json,
    StreamEntry, StreamState, StreamingMorph,
};

pub mod morph_lod_selector;
pub use morph_lod_selector::{
    lod_current_level, lod_level_count, lod_morph_count_at, lod_threshold, lod_to_json,
    new_morph_lod_selector, select_lod, set_lod_threshold, MorphLodSelector,
};

pub mod expression_retarget_ml;
pub use expression_retarget_ml::{
    erml_add_mapping, erml_mapping_count, erml_retarget, erml_set_enabled, erml_to_json,
    new_expression_retarget_ml, ExpressionRetargetMl, RetargetMapping,
};

pub mod voice_driven_morph;
pub use voice_driven_morph::{
    new_voice_driven_morph, vdm_add_mapping, vdm_mapping_count, vdm_process, vdm_set_enabled,
    vdm_set_smoothing, vdm_to_json, AudioBandMapping, VoiceDrivenMorph,
};

pub mod gaze_driven_shape;
pub use gaze_driven_shape::{
    gds_evaluate, gds_set_direction, gds_set_enabled, gds_set_gains, gds_to_json,
    new_gaze_driven_shape, GazeDirection, GazeDrivenShape,
};

pub mod emotion_blend_tree;
pub use emotion_blend_tree::{
    ebt_add_node, ebt_evaluate, ebt_node_count, ebt_set_enabled, ebt_set_root,
    ebt_to_json as emotion_blend_tree_to_json, new_emotion_blend_tree, BlendOp as EmotionBlendOp,
    EmotionBlendTree, EmotionNode,
};

pub mod procedural_wrinkle;
pub use procedural_wrinkle::{
    new_procedural_wrinkle, pw_add_region, pw_evaluate, pw_region_count, pw_set_enabled,
    pw_set_global_scale, pw_to_json, ProceduralWrinkle, WrinklePattern,
    WrinkleRegion as ProceduralWrinkleRegion,
};

pub mod age_progression_morph;
pub use age_progression_morph::{
    apm_add_stage, apm_evaluate, apm_set_age, apm_set_enabled, apm_stage_count, apm_to_json,
    new_age_progression_morph, AgeProgressionMorph, AgeStage as AgeProgressionStage,
};

pub mod ethnic_blend_morph;
pub use ethnic_blend_morph::{
    ebmrph_add_feature_set, ebmrph_evaluate, ebmrph_feature_count, ebmrph_set_blend_weight,
    ebmrph_set_enabled, ebmrph_to_json, new_ethnic_blend_morph, EthnicBlendMorph, EthnicFeatureSet,
};

pub mod body_mass_index_morph;
pub use body_mass_index_morph::{
    bmi_evaluate, bmi_set_enabled, bmi_set_influence, bmi_set_value, bmi_to_json, new_bmi_morph,
    BmiCategory as BmiBodyCategory, BodyMassIndexMorph,
};

pub mod muscle_tone_morph;
pub use muscle_tone_morph::{
    mtm_evaluate, mtm_override_count, mtm_set_definition, mtm_set_enabled, mtm_set_group_override,
    mtm_set_tone, mtm_to_json, new_muscle_tone_morph, MuscleGroup as ToneMuscleGroup,
    MuscleToneMorph,
};

pub mod body_hair_density;
pub use body_hair_density::{
    bhd_region_count, bhd_set_coarseness, bhd_set_density, bhd_set_enabled, bhd_set_region,
    bhd_to_json, new_body_hair_density, BodyHairDensity,
};

pub mod skin_thickness_morph;
pub use skin_thickness_morph::{
    new_skin_thickness_morph, stm_evaluate, stm_region_count, stm_set_enabled, stm_set_region,
    stm_set_thickness, stm_to_json, SkinRegion, SkinThicknessMorph,
};

pub mod subcutaneous_fat_morph;
pub use subcutaneous_fat_morph::{
    new_subcutaneous_fat_morph, sfm_evaluate, sfm_set_enabled, sfm_set_fat, sfm_set_pattern,
    sfm_set_visceral_ratio, sfm_to_json, FatPattern, SubcutaneousFatMorph,
};

pub mod vascular_visibility;
pub use vascular_visibility::{
    new_vascular_visibility, vv_evaluate, vv_region_count, vv_set_dilation, vv_set_enabled,
    vv_set_region, vv_set_visibility, vv_to_json, VascularVisibility, VeinRegion,
};

pub mod scar_tissue_morph;
pub use scar_tissue_morph::{
    new_scar_tissue_morph, scm_add_scar, scm_clear, scm_evaluate, scm_scar_count, scm_set_enabled,
    scm_set_intensity, scm_to_json, ScarRegion, ScarTissueMorph, ScarType as TissueScarType,
};

pub mod tattoo_morph;
pub use tattoo_morph::{
    new_tattoo_morph, tm_add_tattoo, tm_evaluate, tm_remove_tattoo, tm_set_enabled,
    tm_set_stretch_influence, tm_tattoo_count, tm_to_json, TattooEntry, TattooMorph,
};

pub mod piercing_deform;
pub use piercing_deform::{
    new_piercing_deform, pd_add_piercing, pd_evaluate, pd_piercing_count, pd_remove_piercing,
    pd_set_enabled, pd_to_json, PiercingDeform, PiercingEntry, PiercingLocation,
};

pub mod dental_morph;
pub use dental_morph::{
    dm_evaluate, dm_set_alignment, dm_set_enabled, dm_set_gum_exposure, dm_set_tooth_size,
    dm_set_whitening, dm_to_json, new_dental_morph, DentalAlignment, DentalMorph,
};

pub mod nail_morph;
pub use nail_morph::{
    new_nail_morph, nm_evaluate, nm_set_curvature, nm_set_enabled, nm_set_length, nm_set_shape,
    nm_set_thickness, nm_to_json, NailMorph, NailShape as NailMorphShape,
};

pub mod hair_follicle_density;
pub use hair_follicle_density::{
    hfd_evaluate, hfd_region_count, hfd_set_density, hfd_set_enabled, hfd_set_miniaturization,
    hfd_set_region, hfd_to_json, new_hair_follicle_density, HairFollicleDensity, ScalpRegion,
};

pub mod pore_size_morph;
pub use pore_size_morph::{
    new_pore_size_morph, psm_evaluate, psm_set_depth, psm_set_enabled, psm_set_size, psm_set_zone,
    psm_to_json, psm_zone_count, PoreSizeMorph, PoreZone,
};

pub mod crease_depth_morph;
pub use crease_depth_morph::{
    cdm_add_crease, cdm_clear, cdm_crease_count, cdm_evaluate, cdm_set_enabled,
    cdm_set_global_scale, cdm_to_json, new_crease_depth_morph, CreaseDepthMorph, CreaseEntry,
    CreaseRegion,
};

pub mod lip_volume_morph;
pub use lip_volume_morph::{
    lvm_evaluate, lvm_set_area, lvm_set_definition, lvm_set_enabled, lvm_set_projection,
    lvm_set_volume, lvm_to_json, new_lip_volume_morph, LipArea, LipVolumeMorph,
};

pub mod eye_size_morph;
pub use eye_size_morph::{
    esm_evaluate, esm_set_aperture, esm_set_enabled, esm_set_height, esm_set_side, esm_set_tilt,
    esm_set_width, esm_to_json, new_eye_size_morph, EyeSide as EyeSizeSide, EyeSizeMorph,
};

pub mod hydration_morph;
pub use hydration_morph::{
    hym_evaluate, hym_set_enabled, hym_set_intensity, hym_set_level, hym_to_json,
    new_hydration_morph, HydrationLevel, HydrationMorph,
};

pub mod sun_damage_morph;
pub use sun_damage_morph::{
    new_sun_damage_morph, sdm_evaluate, sdm_set_enabled, sdm_set_exposure_years, sdm_set_intensity,
    sdm_set_severity, sdm_to_json, SunDamageMorph, SunDamageSeverity,
};

pub mod stretch_mark_morph;
pub use stretch_mark_morph::{
    new_stretch_mark_morph, smm_add_entry, smm_clear, smm_entry_count, smm_evaluate,
    smm_set_enabled, smm_set_intensity, smm_to_json, StretchMarkEntry, StretchMarkMorph,
    StretchMarkRegion,
};

pub mod cellulite_morph;
pub use cellulite_morph::{
    clm_evaluate, clm_set_coverage, clm_set_depth, clm_set_enabled, clm_set_grade, clm_to_json,
    new_cellulite_morph, CelluliteGrade, CelluliteMorph,
};

pub mod edema_morph;
pub use edema_morph::{
    edm_add_region, edm_clear, edm_evaluate, edm_region_count, edm_set_enabled, edm_set_intensity,
    edm_to_json, new_edema_morph, EdemaMorph, EdemaRegion, EdemaType,
};

pub mod bruise_morph;
pub use bruise_morph::{
    brm_add_bruise, brm_bruise_count, brm_clear, brm_evaluate, brm_set_enabled, brm_set_intensity,
    brm_to_json, new_bruise_morph, BruiseEntry, BruiseMorph, BruiseStage,
};

pub mod flush_morph;
pub use flush_morph::{
    flm_evaluate, flm_set_cause, flm_set_enabled, flm_set_intensity, flm_set_spread, flm_to_json,
    new_flush_morph, FlushCause, FlushMorph,
};

pub mod pallor_morph;
pub use pallor_morph::{
    new_pallor_morph, plm_evaluate, plm_set_cause, plm_set_enabled, plm_set_intensity, plm_to_json,
    PallorCause, PallorMorph,
};

pub mod cyanosis_morph;
pub use cyanosis_morph::{
    cym_evaluate, cym_set_enabled, cym_set_intensity, cym_set_oxygen_saturation, cym_set_type,
    cym_to_json, new_cyanosis_morph, CyanosisMorph, CyanosisType,
};

pub mod jaundice_morph;
pub use jaundice_morph::{
    jdm_evaluate, jdm_set_bilirubin, jdm_set_enabled, jdm_set_intensity, jdm_set_severity,
    jdm_to_json, new_jaundice_morph, JaundiceMorph, JaundiceSeverity,
};

pub mod erythema_morph;
pub use erythema_morph::{
    erm_evaluate, erm_set_affected_area, erm_set_enabled, erm_set_intensity, erm_set_pattern,
    erm_to_json, new_erythema_morph, ErythemaMorph, ErythemaPattern,
};

pub mod freckle_morph;
pub use freckle_morph::{
    fkm_evaluate, fkm_set_density, fkm_set_enabled, fkm_set_pattern, fkm_set_size, fkm_to_json,
    new_freckle_morph, FreckileMorph, FrecklePattern,
};

pub mod mole_morph;
pub use mole_morph::{
    mom_add_mole, mom_clear, mom_evaluate, mom_mole_count, mom_set_enabled, mom_set_opacity,
    mom_to_json, new_mole_morph, MoleEntry, MoleMorph, MoleType,
};

pub mod acne_morph;
pub use acne_morph::{
    acm_add_lesion, acm_clear, acm_evaluate, acm_lesion_count, acm_set_enabled, acm_set_severity,
    acm_to_json, new_acne_morph, AcneLesion, AcneLesionType, AcneMorph,
};

pub mod rosacea_morph;
pub use rosacea_morph::{
    new_rosacea_morph, rsm_evaluate, rsm_set_enabled, rsm_set_redness, rsm_set_subtype,
    rsm_set_telangiectasia, rsm_to_json, RosaceaMorph, RosaceaSubtype,
};

pub mod vitiligo_morph;
pub use vitiligo_morph::{
    new_vitiligo_morph, vim_add_patch, vim_clear, vim_evaluate, vim_patch_count, vim_set_enabled,
    vim_set_extent, vim_set_pattern, vim_to_json, VitiligoMorph, VitiligoPaattern, VitilligoPatch,
};

pub mod pregnancy_morph;
pub use pregnancy_morph::{
    new_pregnancy_morph, pm_belly_weight, pm_breast_delta, pm_set_weeks, pm_to_json, pm_trimester,
    PregnancyMorph, PregnancyMorphConfig, Trimester,
};

pub mod infant_morph;
pub use infant_morph::{
    im_head_scale, im_limb_scale, im_set_age, im_to_json, im_weight, new_infant_morph, InfantMorph,
    InfantMorphConfig,
};

pub mod child_morph;
pub use child_morph::{
    cm_height_scale, cm_limb_scale, cm_progress, cm_set_age, cm_to_json, new_child_morph,
    ChildMorph, ChildMorphConfig,
};

pub mod adolescent_morph;
pub use adolescent_morph::{
    adol_hip_delta, adol_progress, adol_set_age, adol_shoulder_delta, adol_to_json,
    new_adolescent_morph, AdolSex, AdolescentMorph, AdolescentMorphConfig,
};

pub mod elderly_morph;
pub use elderly_morph::{
    em_height_loss, em_kyphosis, em_progress, em_set_age, em_skin_sag, em_to_json,
    new_elderly_morph, ElderlyMorph, ElderlyMorphConfig,
};

pub mod athletic_build_morph;
pub use athletic_build_morph::{
    ab_muscle_weight, ab_set_intensity, ab_shoulder_delta, ab_to_json, ab_waist_taper,
    new_athletic_build_morph, AthleticBuildConfig, AthleticBuildMorph,
};

pub mod ectomorph_morph;
pub use ectomorph_morph::{
    ect_hip_weight, ect_limb_weight, ect_set_intensity, ect_shoulder_weight, ect_to_json,
    new_ectomorph_morph, EctomorphConfig, EctomorphMorph,
};

pub mod mesomorph_morph;
pub use mesomorph_morph::{
    meso_chest_delta, meso_muscle_tone, meso_set_intensity, meso_to_json, meso_waist_ratio,
    new_mesomorph_morph, MesomorphConfig, MesomorphMorph,
};

pub mod endomorph_morph;
pub use endomorph_morph::{
    endo_belly_weight, endo_face_roundness, endo_limb_girth, endo_set_intensity, endo_to_json,
    new_endomorph_morph, EndomorphConfig, EndomorphMorph,
};

pub mod android_proportion;
pub use android_proportion::{
    andr_belly, andr_hip_narrow, andr_set_intensity, andr_to_json, andr_upper_mass,
    new_android_proportion, AndroidProportion, AndroidProportionConfig,
};

pub mod gynoid_proportion;
pub use gynoid_proportion::{
    gyn_hip_fullness, gyn_set_intensity, gyn_thigh_girth, gyn_to_json, gyn_upper_slim,
    new_gynoid_proportion, GynoidProportion, GynoidProportionConfig,
};

pub mod hourglass_proportion;
pub use hourglass_proportion::{
    hg_bust, hg_hips, hg_set_intensity, hg_to_json as hourglass_to_json, hg_waist, hg_whr,
    new_hourglass_proportion, HourglassConfig, HourglassProportion,
};

pub mod inverted_triangle_morph;
pub use inverted_triangle_morph::{
    inv_chest_width, inv_hip_narrow, inv_set_intensity, inv_shoulder_broad, inv_shoulder_hip_ratio,
    inv_to_json, new_inverted_triangle_morph, InvertedTriangleConfig, InvertedTriangleMorph,
};

pub mod rectangle_body_morph;
pub use rectangle_body_morph::{
    new_rectangle_body_morph, rect_set_intensity, rect_shoulder_hip_balance, rect_straightness,
    rect_to_json, rect_waist_fullness, RectangleBodyConfig, RectangleBodyMorph,
};

pub mod pear_shape_morph;
pub use pear_shape_morph::{
    new_pear_shape_morph, pear_hip_shoulder_ratio, pear_hip_width, pear_set_intensity,
    pear_shoulder_slim, pear_thigh_fullness, pear_to_json, PearShapeConfig, PearShapeMorph,
};

pub mod apple_shape_morph;
pub use apple_shape_morph::{
    apple_abdomen, apple_chest, apple_limb_slim, apple_set_intensity, apple_to_json,
    apple_waist_scale, new_apple_shape_morph, AppleShapeConfig, AppleShapeMorph,
};

pub mod posture_morph;
pub use posture_morph::{
    new_posture_morph, posture_apply_weights, posture_set_forward_lean, posture_set_lateral_lean,
    posture_set_sway, posture_to_json, PostureMorph, PostureMorphConfig,
};

pub mod slouch_morph;
pub use slouch_morph::{
    new_slouch_morph, slouch_apply, slouch_set_degree, slouch_set_head_forward,
    slouch_set_shoulder_round, slouch_to_json, SlouchMorph, SlouchMorphConfig,
};

pub mod kyphosis_morph;
pub use kyphosis_morph::{
    kyphosis_evaluate, kyphosis_set_apex, kyphosis_set_curve, kyphosis_set_spread,
    kyphosis_to_json, new_kyphosis_morph, KyphosisMorph, KyphosisMorphConfig,
};

pub mod lordosis_morph;
pub use lordosis_morph::{
    lordosis_evaluate, lordosis_set_anterior_tilt, lordosis_set_curve, lordosis_set_lumbar_apex,
    lordosis_to_json, new_lordosis_morph, LordosisMorph, LordosisMorphConfig,
};

pub mod scoliosis_morph;
pub use scoliosis_morph::{
    new_scoliosis_morph, scoliosis_displacement, scoliosis_set_direction, scoliosis_set_lateral,
    scoliosis_set_rotation, scoliosis_to_json, ScoliosisMorph, ScoliosisMorphConfig,
};

pub mod flat_foot_morph;
pub use flat_foot_morph::{
    flat_foot_arch_height, flat_foot_set_arch_collapse, flat_foot_set_pronation,
    flat_foot_set_toe_splay, flat_foot_to_json, new_flat_foot_morph, FlatFootMorph,
    FlatFootMorphConfig,
};

pub mod high_arch_morph;
pub use high_arch_morph::{
    high_arch_height, high_arch_set_claw_toe, high_arch_set_rise, high_arch_set_supination,
    high_arch_to_json, new_high_arch_morph, HighArchMorph, HighArchMorphConfig,
};

pub mod knock_knee_morph;
pub use knock_knee_morph::{
    knock_knee_separation, knock_knee_set_eversion, knock_knee_set_torsion, knock_knee_set_valgus,
    knock_knee_to_json, new_knock_knee_morph, KnockKneeMorph, KnockKneeMorphConfig,
};

pub mod bow_leg_morph;
pub use bow_leg_morph::{
    bow_leg_bow_out, bow_leg_set_inversion, bow_leg_set_torsion, bow_leg_set_varus,
    bow_leg_to_json, new_bow_leg_morph, BowLegMorph, BowLegMorphConfig,
};

pub mod pigeon_toe_morph;
pub use pigeon_toe_morph::{
    new_pigeon_toe_morph, pigeon_toe_set_femoral, pigeon_toe_set_intoeing,
    pigeon_toe_set_metatarsus, pigeon_toe_to_json, pigeon_toe_total_rotation, PigeonToeMorph,
    PigeonToeMorphConfig,
};

pub mod limb_length_morph;
pub use limb_length_morph::{
    limb_leg_discrepancy, limb_length_to_json, limb_set_left_arm, limb_set_left_leg,
    limb_set_right_arm, limb_set_right_leg, new_limb_length_morph, LimbLengthMorph,
    LimbLengthMorphConfig,
};

pub mod shoulder_height_morph;
pub use shoulder_height_morph::{
    new_shoulder_height_morph, shoulder_height_asymmetry, shoulder_height_set_left,
    shoulder_height_set_right, shoulder_height_set_tilt, shoulder_height_to_json,
    ShoulderHeightMorph, ShoulderHeightMorphConfig,
};

pub mod hip_tilt_morph;
pub use hip_tilt_morph::{
    hip_tilt_magnitude, hip_tilt_set_anterior, hip_tilt_set_lateral, hip_tilt_set_rotation,
    hip_tilt_to_json, new_hip_tilt_morph, HipTiltMorph, HipTiltMorphConfig,
};

pub mod head_tilt_morph;
pub use head_tilt_morph::{
    head_tilt_magnitude, head_tilt_set_axial, head_tilt_set_forward, head_tilt_set_lateral,
    head_tilt_to_json, new_head_tilt_morph, HeadTiltMorph, HeadTiltMorphConfig,
};

pub mod facial_asymmetry_morph;
pub use facial_asymmetry_morph::{
    facial_asym_score, facial_asym_set_horizontal_shift, facial_asym_set_left_scale,
    facial_asym_set_right_scale, facial_asym_set_vertical_offset, facial_asymmetry_to_json,
    new_facial_asymmetry_morph, FacialAsymmetryMorph, FacialAsymmetryMorphConfig,
};

pub mod jaw_asymmetry_morph;
pub use jaw_asymmetry_morph::{
    jaw_asym_deviation_magnitude, jaw_asym_set_chin_shift, jaw_asym_set_lateral,
    jaw_asym_set_ramus, jaw_asymmetry_to_json, new_jaw_asymmetry_morph, JawAsymmetryMorph,
    JawAsymmetryMorphConfig,
};

pub mod eyelash_morph;
pub use eyelash_morph::{
    eyelash_morph_to_json, eyelash_set_curl, eyelash_set_density, eyelash_set_length,
    new_eyelash_morph, EyelashMorph,
};

pub mod eyebrow_density_morph;
pub use eyebrow_density_morph::{
    ebrow_density_set_density, ebrow_density_set_fullness, ebrow_density_set_gap_fill,
    eyebrow_density_morph_to_json, new_eyebrow_density_morph, EyebrowDensityMorph,
};

pub mod beard_density_morph;
pub use beard_density_morph::{
    beard_density_morph_to_json, beard_density_set_coarseness, beard_density_set_coverage,
    beard_density_set_density, beard_density_visual_thickness, new_beard_density_morph,
    BeardDensityMorph,
};

pub mod mustache_morph;
pub use mustache_morph::{
    mustache_morph_to_json, mustache_set_density, mustache_set_droop, mustache_set_width,
    mustache_style_score, new_mustache_morph, MustacheMorph,
};

pub mod sideburn_morph;
pub use sideburn_morph::{
    new_sideburn_morph, sideburn_area_estimate, sideburn_morph_to_json, sideburn_set_length,
    sideburn_set_taper, sideburn_set_width, SideburnMorph,
};

pub mod arm_hair_morph;
pub use arm_hair_morph::{
    arm_hair_morph_to_json, arm_hair_set_darkness, arm_hair_set_density, arm_hair_set_length,
    arm_hair_visibility, new_arm_hair_morph, ArmHairMorph,
};

pub mod scalp_hairline_morph;
pub use scalp_hairline_morph::{
    hairline_blend, hairline_is_receded, hairline_overall_weight, hairline_recession_symmetric,
    hairline_set_position, new_hairline_morph, HairlineMorph,
};

pub mod hair_part_morph;
pub use hair_part_morph::{
    hair_part_infer_style, hair_part_morph_to_json, hair_part_set_depth, hair_part_set_offset,
    hair_part_set_style, new_hair_part_morph, HairPartMorph, HairPartStyle,
};

pub mod hair_volume_morph;
pub use hair_volume_morph::{
    hair_volume_fullness, hair_volume_morph_to_json, hair_volume_set_crown_lift,
    hair_volume_set_side_puff, hair_volume_set_volume, new_hair_volume_morph, HairVolumeMorph,
};

pub mod hair_wave_morph;
pub use hair_wave_morph::{
    hair_wave_displacement_at, hair_wave_morph_to_json, hair_wave_set_amplitude,
    hair_wave_set_frequency, hair_wave_set_tightness, new_hair_wave_morph,
    CurlPattern as HairWaveCurlPattern, HairWaveMorph,
};

pub mod skin_texture_scale_morph;
pub use skin_texture_scale_morph::{
    new_skin_texture_scale_morph, skin_tex_scale_aspect_ratio, skin_tex_scale_set_u,
    skin_tex_scale_set_uniform, skin_tex_scale_set_v, skin_texture_scale_morph_to_json,
    SkinTextureScaleMorph,
};

pub mod skin_gloss_morph;
pub use skin_gloss_morph::{
    new_skin_gloss_morph, skin_gloss_effective, skin_gloss_morph_to_json, skin_gloss_set_oiliness,
    skin_gloss_set_roughness, skin_gloss_set_specularity, SkinGlossMorph,
};

pub mod skin_subsurface_morph;
pub use skin_subsurface_morph::{
    new_skin_subsurface_morph, skin_sss_mean_depth, skin_sss_set_depth, skin_sss_set_red_depth,
    skin_sss_set_rgb_depths, skin_subsurface_morph_to_json, SkinSubsurfaceMorph,
};

pub mod skin_translucency_morph;
pub use skin_translucency_morph::{
    new_skin_translucency_morph, skin_trans_light_bleed, skin_trans_set_thin_skin,
    skin_trans_set_translucency, skin_trans_set_vein_visibility, skin_translucency_morph_to_json,
    SkinTranslucencyMorph,
};

pub mod iris_size_morph;
pub use iris_size_morph::{
    iris_size_mean, iris_size_morph_to_json, iris_size_set_diameter, iris_size_set_left,
    iris_size_set_right, new_iris_size_morph, IrisSizeMorph,
};

pub mod pupil_dilation_morph;
pub use pupil_dilation_morph::{
    new_pupil_dilation_morph, pupil_apply_light_response, pupil_dilation_morph_to_json,
    pupil_set_dilation, pupil_set_left, pupil_set_right, PupilDilationMorph,
};

pub mod tongue_tip_morph;
pub use tongue_tip_morph::{
    new_tongue_tip_morph, tongue_tip_morph_to_json, tongue_tip_set_curl,
    tongue_tip_set_lateral_spread, tongue_tip_set_protrusion, tongue_tip_set_sharpness,
    TongueTipMorph,
};

pub mod tongue_dorsum_morph;
pub use tongue_dorsum_morph::{
    new_tongue_dorsum_morph, tongue_dorsum_morph_to_json, tongue_dorsum_set_arch,
    tongue_dorsum_set_groove, tongue_dorsum_set_posterior_raise, tongue_dorsum_set_width,
    TongueDorsumMorph,
};

pub mod uvula_morph;
pub use uvula_morph::{
    new_uvula_morph, uvula_morph_to_json, uvula_set_elevation, uvula_set_length,
    uvula_set_tip_bulge, uvula_set_width, uvula_surface_area, UvulaMorph,
};

pub mod soft_palate_morph;
pub use soft_palate_morph::{
    new_soft_palate_morph, soft_palate_is_sealed, soft_palate_morph_to_json,
    soft_palate_set_curvature, soft_palate_set_raise, soft_palate_set_tension,
    soft_palate_set_width, SoftPalateMorph,
};

pub mod pharynx_morph;
pub use pharynx_morph::{
    new_pharynx_morph, pharynx_cross_section, pharynx_morph_to_json, pharynx_set_constriction,
    pharynx_set_epiglottis_tilt, pharynx_set_length, pharynx_set_wall_tension, PharynxMorph,
};

pub mod vocal_tract_morph;
pub use vocal_tract_morph::{
    new_vocal_tract_morph, vocal_tract_mean_constriction, vocal_tract_morph_to_json,
    vocal_tract_set_back_constriction, vocal_tract_set_length, vocal_tract_set_lip_rounding,
    vocal_tract_set_mid_constriction, VocalTractMorph,
};

pub mod larynx_position_morph;
pub use larynx_position_morph::{
    larynx_position_morph_to_json, larynx_set_anterior_posterior, larynx_set_height,
    larynx_set_tilt, larynx_tract_lengthening, new_larynx_position_morph, LarynxPositionMorph,
};

pub mod thyroid_cartilage_morph;
pub use thyroid_cartilage_morph::{
    new_thyroid_cartilage_morph, thyroid_cartilage_morph_to_json, thyroid_set_angle,
    thyroid_set_height, thyroid_set_prominence, thyroid_set_width, ThyroidCartilageMorph,
};

pub mod cricoid_cartilage_morph;
pub use cricoid_cartilage_morph::{
    cricoid_cartilage_morph_to_json, cricoid_ring_circumference, cricoid_set_arch_height,
    cricoid_set_arch_width, cricoid_set_posterior_plate_height, cricoid_set_ring_radius,
    new_cricoid_cartilage_morph, CricoidCartilageMorph,
};

pub mod arytenoid_morph;
pub use arytenoid_morph::{
    arytenoid_is_phonating, arytenoid_morph_to_json, arytenoid_set_adduction,
    arytenoid_set_asymmetry, arytenoid_set_rotation, arytenoid_set_tilt, new_arytenoid_morph,
    ArytenoidMorph,
};

pub mod glottis_morph;
pub use glottis_morph::{
    glottis_is_closed, glottis_morph_to_json, glottis_set_mucosal_wave, glottis_set_opening,
    glottis_set_posterior_gap, glottis_set_tension, new_glottis_morph, GlottisMorph,
};

pub mod trachea_morph;
pub use trachea_morph::{
    new_trachea_morph, trachea_morph_to_json, trachea_set_calibre, trachea_set_curvature,
    trachea_set_length, trachea_set_wall_thickness, trachea_volume, TracheaMorph,
};

pub mod diaphragm_morph;
pub use diaphragm_morph::{
    diaphragm_effective_dome, diaphragm_morph_to_json, diaphragm_set_contraction,
    diaphragm_set_descent, diaphragm_set_dome_height, diaphragm_set_excursion_range,
    new_diaphragm_morph, DiaphragmMorph,
};

pub mod rib_cage_morph;
pub use rib_cage_morph::{
    new_rib_cage_morph, rib_cage_mean_expansion, rib_cage_morph_to_json, rib_cage_set_expansion,
    rib_cage_set_lateral_flare, rib_cage_set_lower_chest, rib_cage_set_upper_chest, RibCageMorph,
};

pub mod abdomen_expand_morph;
pub use abdomen_expand_morph::{
    abdomen_expand_mean, abdomen_expand_morph_to_json, abdomen_expand_set_expansion,
    abdomen_expand_set_lateral_bulge, abdomen_expand_set_lower, abdomen_expand_set_upper,
    new_abdomen_expand_morph, AbdomenExpandMorph,
};

pub mod pelvic_floor_morph;
pub use pelvic_floor_morph::{
    new_pelvic_floor_morph, pelvic_floor_elevation, pelvic_floor_morph_to_json,
    pelvic_floor_set_contraction, pelvic_floor_set_descent, pelvic_floor_set_levator_tension,
    pelvic_floor_set_perineal_body, PelvicFloorMorph,
};
pub mod spine_curve_morph;
pub use spine_curve_morph::{
    new_spine_curve_morph, scm_set_cervical_lordosis, scm_set_lumbar_lordosis,
    scm_set_overall_flex, scm_set_thoracic_kyphosis, scm_total_angle_rad,
    spine_curve_morph_to_json, SpineCurveMorph,
};
pub mod intervertebral_morph;
pub use intervertebral_morph::{
    intervertebral_morph_to_json, ivm_effective_height, ivm_set_cervical_height,
    ivm_set_degeneration, ivm_set_lumbar_height, ivm_set_thoracic_height, new_intervertebral_morph,
    IntervertebralMorph,
};
pub mod sacrum_morph;
pub use sacrum_morph::{
    new_sacrum_morph, sac_pelvic_inlet, sac_set_curvature, sac_set_promontory_depth, sac_set_tilt,
    sac_set_width, sacrum_morph_to_json, SacrumMorph,
};
pub mod coccyx_morph;
pub use coccyx_morph::{
    coc_set_deviation, coc_set_flexion, coc_set_length, coc_set_prominence, coc_tip_displacement,
    coccyx_morph_to_json, new_coccyx_morph, CoccyxMorph,
};
pub mod sternum_morph;
pub use sternum_morph::{
    new_sternum_morph, sternum_blend, sternum_is_protruding, sternum_overall_weight,
    sternum_set_protrusion, SternumMorph,
};
pub mod clavicle_morph;
pub use clavicle_morph::{
    clavicle_blend, clavicle_is_prominent, clavicle_overall_weight, clavicle_set_prominence,
    new_clavicle_morph, ClavicleMorph,
};
pub mod scapula_morph;
pub use scapula_morph::{
    new_scapula_morph, scap_set_rotation, scap_set_size, scap_set_spine_prominence,
    scap_set_winging, scap_visibility, scapula_morph_to_json, ScapulaMorph,
};
pub mod humerus_morph;
pub use humerus_morph::{
    hum_elbow_breadth, hum_set_epicondyle_width, hum_set_head_size, hum_set_length,
    hum_set_shaft_curvature, humerus_morph_to_json, new_humerus_morph, HumerusMorph,
};
pub mod radius_ulna_morph;
pub use radius_ulna_morph::{
    new_radius_ulna_morph, radius_ulna_morph_to_json, ru_set_bowing, ru_set_length,
    ru_set_radius_ratio, ru_set_styloid_prominence, ru_wrist_width, RadiusUlnaMorph,
};
pub mod carpals_morph;
pub use carpals_morph::{
    carp_set_height, carp_set_spacing, carp_set_tunnel_depth, carp_set_width, carp_tunnel_area,
    carpals_morph_to_json, new_carpals_morph, CarpalsMorph,
};
pub mod femur_morph;
pub use femur_morph::{
    fem_q_angle, fem_set_anteversion, fem_set_condyle_width, fem_set_length, fem_set_neck_angle,
    femur_morph_to_json, new_femur_morph, FemurMorph,
};
pub mod tibia_fibula_morph;
pub use tibia_fibula_morph::{
    new_tibia_fibula_morph, tf_foot_progression, tf_set_fibula_offset, tf_set_length,
    tf_set_malleolus_width, tf_set_tibial_torsion, tibia_fibula_morph_to_json, TibiaFibulaMorph,
};
pub mod tarsals_morph;
pub use tarsals_morph::{
    new_tarsals_morph, tars_contact_area, tars_set_arch_height, tars_set_calcaneus_length,
    tars_set_midfoot_width, tars_set_talus_tilt, tarsals_morph_to_json, TarsalsMorph,
};
pub mod skull_morph;
pub use skull_morph::{
    new_skull_morph, skl_cranial_index, skl_set_cranial_height, skl_set_cranial_length,
    skl_set_cranial_width, skl_set_frontal_slope, skl_set_occipital_projection,
    skull_morph_to_json, SkullMorph,
};
pub mod mandible_morph;
pub use mandible_morph::{
    mand_set_body_width, mand_set_gonial_angle, mand_set_ramus_height, mand_set_symphysis_height,
    mand_squareness, mandible_morph_to_json, new_mandible_morph, MandibleMorph,
};
pub mod orbital_morph;
pub use orbital_morph::{
    new_orbital_morph, orb_aperture_area, orb_set_depth as orbital_set_depth, orb_set_height,
    orb_set_rim_prominence, orb_set_tilt as orbital_set_tilt, orb_set_width, orbital_morph_to_json,
    OrbitalMorph,
};
pub mod nose_ala_morph;
pub use nose_ala_morph::{
    nala_set_curvature, nala_set_flare, nala_set_thickness, nala_set_width, nala_surface_area,
    new_nose_ala_morph, nose_ala_morph_to_json, NoseAlaMorph,
};
pub mod nose_root_morph;
pub use nose_root_morph::{
    new_nose_root_morph, nose_root_morph_to_json, nroot_aspect_ratio, nroot_set_bridge_continuity,
    nroot_set_depth, nroot_set_height, nroot_set_width, NoseRootMorph,
};
pub mod nasal_septum_morph;
pub use nasal_septum_morph::{
    nasal_septum_morph_to_json, new_nasal_septum_morph, nsept_deviation_magnitude,
    nsept_set_caudal_angle, nsept_set_deviation, nsept_set_dorsal_height, nsept_set_thickness,
    NasalSeptumMorph,
};
pub mod nasal_spine_morph;
pub use nasal_spine_morph::{
    nasal_spine_morph_to_json, new_nasal_spine_morph, nspine_set_angulation, nspine_set_base_width,
    nspine_set_projection, nspine_set_prominence, nspine_volume_estimate, NasalSpineMorph,
};
pub mod columella_morph;
pub use columella_morph::{
    columella_blend, columella_is_hanging, columella_overall_weight, columella_set_inclination,
    columella_set_width, new_columella_morph, ColumellaMorph,
};
pub mod philtrum_morph;
pub use philtrum_morph::{
    new_philtrum_morph, philtrum_blend as philtrum_morph_blend, philtrum_is_deep,
    philtrum_overall_weight, philtrum_set_depth as philtrum_morph_set_depth,
    philtrum_set_width as philtrum_morph_set_width, PhiltrumMorph,
};
pub mod upper_lip_body_morph;
pub use upper_lip_body_morph::{
    new_upper_lip_body_morph, ulb_set_fullness, ulb_set_projection, ulb_set_roll,
    ulb_set_vermilion_height, ulb_volume_estimate, upper_lip_body_morph_to_json, UpperLipBodyMorph,
};
pub mod lower_lip_body_morph;
pub use lower_lip_body_morph::{
    llb_set_fullness, llb_set_labiomental_groove, llb_set_projection, llb_set_vermilion_height,
    llb_volume_estimate, lower_lip_body_morph_to_json, new_lower_lip_body_morph, LowerLipBodyMorph,
};
pub mod lip_commissure_morph;
pub use lip_commissure_morph::{
    lcom_effective_angle, lcom_set_angle, lcom_set_depth, lcom_set_downturn, lcom_set_width,
    lip_commissure_morph_to_json, new_lip_commissure_morph, LipCommissureMorph,
};
pub mod mentolabial_morph;
pub use mentolabial_morph::{
    mentolabial_blend, mentolabial_is_deep, mentolabial_overall_weight, mentolabial_set_depth,
    new_mentolabial_morph, MentolabialMorph,
};
pub mod malar_eminence_morph;
pub use malar_eminence_morph::{
    mal_blend, mal_is_prominent, mal_overall_weight, mal_set_left, mal_set_projection,
    mal_set_right, new_malar_eminence_morph, MalarEminenceMorph,
};
pub mod zygomatic_arch_morph;
pub use zygomatic_arch_morph::{
    new_zygomatic_arch_morph, zygomatic_blend, zygomatic_is_prominent, zygomatic_overall_weight,
    zygomatic_set_prominence, zygomatic_set_width, ZygomaticArchMorph,
};
pub mod temporal_region_morph;
pub use temporal_region_morph::{
    new_temporal_region_morph, temp_effective_volume, temp_set_hollowing, temp_set_muscle_fullness,
    temp_set_superior_extent, temp_set_width, temporal_region_morph_to_json, TemporalRegionMorph,
};
pub mod mastoid_morph;
pub use mastoid_morph::{
    mastoid_blend, mastoid_is_prominent, mastoid_overall_weight, mastoid_set_size,
    new_mastoid_morph, MastoidMorph,
};
pub mod gonion_morph;
pub use gonion_morph::{
    gon_angle_degrees, gon_set_flare, gon_set_gonial_angle, gon_set_prominence, gon_set_rounding,
    gonion_morph_to_json, new_gonion_morph, GonionMorph,
};
pub mod pogonion_morph;
pub use pogonion_morph::{
    new_pogonion_morph, pogonion_blend, pogonion_has_cleft, pogonion_overall_weight,
    pogonion_set_protrusion, PogonionMorph,
};

pub mod temple_width_morph;
pub use temple_width_morph::{
    new_temple_width_morph, temple_blend as twm_blend, temple_is_wide, temple_overall_weight,
    temple_set_prominence as twm_set_prominence, temple_set_width, TempleWidthMorph,
};

pub mod cranium_height_morph;
pub use cranium_height_morph::{
    cranium_blend, cranium_cephalic_index, cranium_is_dolichocephalic, cranium_set_brachycephaly,
    cranium_set_vault_height, new_cranium_height_morph, CraniumHeightMorph,
};

pub mod occiput_morph;
pub use occiput_morph::{
    new_occiput_morph, occiput_blend, occiput_overall_weight, occiput_set_flatness,
    occiput_set_protrusion, OcciputMorph,
};

pub mod infraorbital_rim_morph;
pub use infraorbital_rim_morph::{
    infraorbital_blend, infraorbital_overall_weight, infraorbital_set_depth,
    infraorbital_set_width, new_infraorbital_rim_morph, InfraorbitalRimMorph,
};

pub mod glabella_morph;
pub use glabella_morph::{
    glabella_blend as glabella_morph_blend, glabella_is_pronounced, glabella_overall_weight,
    glabella_set_prominence, new_glabella_morph, GlabellaMorph,
};

pub mod supraorbital_morph;
pub use supraorbital_morph::{
    new_supraorbital_morph, supraorbital_blend, supraorbital_is_heavy, supraorbital_overall_weight,
    supraorbital_set_ridge, supraorbital_set_slope, SupraorbitalMorph,
};

pub mod nasolabial_morph;
pub use nasolabial_morph::{
    nasolabial_blend, nasolabial_is_deep, nasolabial_overall_weight, nasolabial_set_depth,
    new_nasolabial_morph, NasolabialMorph,
};

pub mod marionette_line_morph;
pub use marionette_line_morph::{
    marionette_blend, marionette_is_pronounced, marionette_overall_weight, marionette_set_depth,
    new_marionette_morph, MarionetteMorph,
};

pub mod neck_length_morph;
pub use neck_length_morph::{
    neck_blend, neck_is_long, neck_overall_weight, neck_set_length as nlm_set_length,
    neck_set_width as nlm_set_width, new_neck_length_morph, NeckLengthMorph,
};

pub mod orbital_depth_morph;
pub use orbital_depth_morph::{
    new_orbital_depth_morph, orbital_blend, orbital_is_deep, orbital_overall_weight,
    orbital_set_depth as orbital_depth_set_depth, OrbitalDepthMorph,
};

pub mod malar_fat_morph;
pub use malar_fat_morph::{
    malar_fat_blend, malar_fat_is_full, malar_fat_overall_weight, malar_fat_set_volume,
    new_malar_fat_morph, MalarFatMorph,
};

pub mod buccal_fat_morph;
pub use buccal_fat_morph::{
    buccal_blend, buccal_is_prominent, buccal_overall_weight, buccal_set_volume,
    new_buccal_fat_morph, BuccalFatMorph,
};

pub mod temporal_hollow_morph;
pub use temporal_hollow_morph::{
    new_temporal_hollow_morph, temporal_hollow_blend, temporal_hollow_is_sunken,
    temporal_hollow_overall_weight, temporal_hollow_set_depth, TemporalHollowMorph,
};

pub mod submental_morph;
pub use submental_morph::{
    new_submental_morph, submental_blend, submental_has_double_chin, submental_overall_weight,
    submental_set_fat, SubmentalMorph,
};

pub mod jowl_morph;
pub use jowl_morph::{
    jowl_blend, jowl_is_prominent, jowl_overall_weight, jowl_set_volume, new_jowl_morph, JowlMorph,
};

pub mod masseter_morph;
pub use masseter_morph::{
    masseter_blend, masseter_is_hypertrophied, masseter_overall_weight, masseter_set_hypertrophy,
    new_masseter_morph, MasseterMorph,
};

pub mod frontalis_morph;
pub use frontalis_morph::{
    frontalis_blend, frontalis_overall_weight, frontalis_set_contraction, frontalis_shows_lines,
    new_frontalis_morph, FrontalisMorph,
};

pub mod corrugator_morph;
pub use corrugator_morph::{
    corrugator_blend, corrugator_is_contracted, corrugator_overall_weight,
    corrugator_set_contraction, new_corrugator_morph, CorrugatorMorph,
};

pub mod orbicularis_oculi_morph;
pub use orbicularis_oculi_morph::{
    new_orbicularis_oculi_morph, orbicularis_blend, orbicularis_overall_weight,
    orbicularis_set_contraction, orbicularis_shows_crow_feet, OrbicularisOculiMorph,
};

pub mod zygomaticus_morph;
pub use zygomaticus_morph::{
    new_zygomaticus_morph, zygomaticus_blend, zygomaticus_is_smiling, zygomaticus_overall_weight,
    zygomaticus_set_major, ZygomaticusMorph,
};

pub mod depressor_anguli_morph;
pub use depressor_anguli_morph::{
    depressor_blend, depressor_is_active, depressor_overall_weight, depressor_set_contraction,
    new_depressor_anguli_morph, DepressorAnguliMorph,
};

pub mod platysma_morph;
pub use platysma_morph::{
    new_platysmae_morph, platysmae_blend, platysmae_overall_weight, platysmae_set_band_prominence,
    platysmae_shows_bands, PlatysmaeMorph,
};

pub mod sternocleidomastoid_morph;
pub use sternocleidomastoid_morph::{
    new_scm_morph, scm_blend, scm_is_defined, scm_overall_weight, scm_set_definition, ScmMorph,
};

pub mod trapezius_morph;
pub use trapezius_morph::{
    new_trapezius_morph, trapezius_blend, trapezius_is_muscular, trapezius_overall_weight,
    trapezius_set_size, TrapeziusMorph,
};

pub mod parotid_morph;
pub use parotid_morph::{
    new_parotid_morph, parotid_blend, parotid_is_prominent, parotid_overall_weight,
    parotid_set_size, ParotidMorph,
};

pub mod tooth_morph;
pub use tooth_morph::{
    new_tooth_morph, tooth_blend, tooth_is_prominent, tooth_overall_weight, tooth_set_size,
    ToothMorph,
};

pub mod gum_morph;
pub use gum_morph::{
    gum_blend, gum_is_gummy_smile, gum_overall_weight, gum_set_exposure, new_gum_morph, GumMorph,
};

pub mod tongue_shape_morph;
pub use tongue_shape_morph::{
    new_tongue_shape_morph, tongue_blend, tongue_is_wide, tongue_overall_weight, tongue_set_width,
    TongueShapeMorph,
};

pub mod lip_thickness_morph;
pub use lip_thickness_morph::{
    lip_thickness_blend as lip_thickness_morph_blend, lip_thickness_is_full,
    lip_thickness_overall_weight, lip_thickness_set_upper, new_lip_thickness_morph,
    LipThicknessMorph,
};

pub mod lip_cupids_bow_morph;
pub use lip_cupids_bow_morph::{
    cupids_blend, cupids_is_defined, cupids_overall_weight, cupids_set_peak, new_cupids_bow_morph,
    CupidsBowMorph,
};

pub mod vermillion_border_morph;
pub use vermillion_border_morph::{
    new_vermillion_border_morph, vermillion_blend, vermillion_is_defined,
    vermillion_overall_weight, vermillion_set_sharpness, VermillionBorderMorph,
};

pub mod eye_spacing_morph;
pub use eye_spacing_morph::{
    eye_spacing_blend, eye_spacing_is_wide, eye_spacing_overall_weight, eye_spacing_set_distance,
    new_eye_spacing_morph, EyeSpacingMorph,
};

pub mod canthal_tilt_morph;
pub use canthal_tilt_morph::{
    canthal_blend, canthal_is_upswept, canthal_overall_weight, canthal_set_outer_tilt,
    new_canthal_tilt_morph, CanthalTiltMorph,
};

pub mod sclera_show_morph;
pub use sclera_show_morph::{
    new_sclera_show_morph, sclera_blend, sclera_has_sanpaku, sclera_overall_weight,
    sclera_set_inferior, ScleraShowMorph,
};

pub mod pupil_size_morph;
pub use pupil_size_morph::{
    new_pupil_size_morph, pupil_blend, pupil_is_dilated, pupil_overall_weight,
    pupil_set_dilation as pupil_size_set_dilation, PupilSizeMorph,
};

pub mod eyelid_crease_morph;
pub use eyelid_crease_morph::{
    eyelid_blend, eyelid_has_crease, eyelid_overall_weight, eyelid_set_crease,
    new_eyelid_crease_morph, EyelidCreaseMorph,
};

pub mod epicanthal_fold_morph;
pub use epicanthal_fold_morph::{
    epicanthal_blend, epicanthal_is_present, epicanthal_overall_weight, epicanthal_set_coverage,
    new_epicanthal_fold_morph, EpicanthalFoldMorph,
};

pub mod lateral_canthus_morph;
pub use lateral_canthus_morph::{
    lateral_canthus_blend, lateral_canthus_is_upturned, lateral_canthus_overall_weight,
    lateral_canthus_set_tilt, new_lateral_canthus_morph, LateralCanthusMorph,
};

pub mod philtrum_depth_morph;
pub use philtrum_depth_morph::{
    new_philtrum_depth_morph, philtrum_depth_blend, philtrum_depth_is_deep,
    philtrum_depth_overall_weight, philtrum_depth_set_depth, PhiltrumDepthMorph,
};

pub mod bmi_body_shape_morph;
pub use bmi_body_shape_morph::{
    bmi_blend, bmi_blend_weight, bmi_category, bmi_is_healthy,
    new_bmi_morph as new_bmi_body_shape_morph, BmiMorph,
};

pub mod muscle_definition_morph;
pub use muscle_definition_morph::{
    muscle_def_blend, muscle_def_is_athletic, muscle_def_overall_weight, muscle_def_set_tone,
    new_muscle_definition_morph, MuscleDefinitionMorph,
};

pub mod visceral_fat_morph;
pub use visceral_fat_morph::{
    new_visceral_fat_morph, visceral_blend, visceral_is_high, visceral_overall_weight,
    visceral_set_level, VisceralFatMorph,
};

pub mod subcut_fat_morph;
pub use subcut_fat_morph::{
    new_subcut_fat_morph, subcut_blend, subcut_is_uniform, subcut_overall_weight, subcut_set_torso,
    SubcutaneousFatMorphNew,
};

pub mod breast_shape_morph_new;
pub use breast_shape_morph_new::{
    breast_blend_new, breast_bra_size_category, breast_overall_weight_new, breast_set_volume_new,
    new_breast_shape_morph_new, BreastShapeMorphNew,
};

pub mod hip_shape_morph;
pub use hip_shape_morph::{
    hip_blend, hip_is_wide, hip_overall_weight, hip_set_width, new_hip_shape_morph, HipShapeMorph,
};

pub mod waist_morph_new;
pub use waist_morph_new::{
    new_waist_morph, waist_blend, waist_is_hourglass, waist_overall_weight, waist_set_narrowing,
    WaistMorph,
};

pub mod abdomen_morph;
pub use abdomen_morph::{
    abdomen_blend, abdomen_overall_weight, abdomen_set_protrusion, abdomen_shows_abs,
    new_abdomen_morph, AbdomenMorph,
};

pub mod back_muscle_morph;
pub use back_muscle_morph::{
    back_blend, back_is_muscular, back_overall_weight, back_set_latissimus, new_back_muscle_morph,
    BackMuscleMorph,
};

pub mod chest_muscle_morph;
pub use chest_muscle_morph::{
    chest_blend, chest_is_muscular, chest_overall_weight, chest_set_pec, new_chest_muscle_morph,
    ChestMuscleMorph,
};

pub mod arm_muscle_morph;
pub use arm_muscle_morph::{
    arm_blend, arm_is_muscular, arm_overall_weight, arm_set_bicep, new_arm_muscle_morph,
    ArmMuscleMorph,
};

pub mod leg_muscle_morph;
pub use leg_muscle_morph::{
    leg_blend, leg_is_muscular, leg_overall_weight, leg_set_quad, new_leg_muscle_morph,
    LegMuscleMorph,
};

pub mod glute_morph;
pub use glute_morph::{
    glute_blend, glute_is_prominent, glute_overall_weight, glute_set_volume, new_glute_morph,
    GluteMorph,
};

pub mod knee_shape_morph;
pub use knee_shape_morph::{
    knee_blend, knee_is_valgus, knee_overall_weight, knee_set_prominence, new_knee_shape_morph,
    KneeShapeMorph,
};

pub mod ankle_shape_morph;
pub use ankle_shape_morph::{
    ankle_blend, ankle_is_slender, ankle_overall_weight, ankle_set_width, new_ankle_morph,
    AnkleMorph,
};

pub mod body_water_morph;
pub use body_water_morph::{
    body_water_blend, body_water_is_dehydrated, body_water_overall_weight,
    body_water_set_hydration, new_body_water_morph, BodyWaterMorph,
};

pub mod skin_pore_morph;
pub use skin_pore_morph::{
    new_skin_pore_morph, pore_blend, pore_is_visible, pore_overall_weight, pore_set_density,
    pore_set_size, SkinPoreMorph,
};

pub mod alar_base_morph;
pub use alar_base_morph::{
    alar_blend, alar_is_wide, alar_overall_weight, alar_set_flare, alar_set_width,
    new_alar_base_morph, AlarBaseMorph,
};

pub mod nasal_tip_projection_morph;
pub use nasal_tip_projection_morph::{
    new_nasal_tip_projection_morph, ntp_blend, ntp_is_projected, ntp_overall_weight,
    ntp_set_projection, ntp_set_rotation, NasalTipProjectionMorph,
};

pub mod lid_fullness_morph;
pub use lid_fullness_morph::{
    lid_blend, lid_is_puffy, lid_overall_weight, lid_set_lower, lid_set_upper,
    new_lid_fullness_morph, LidFullnessMorph,
};

pub mod brow_bone_bossing_morph;
pub use brow_bone_bossing_morph::{
    bbb_blend, bbb_is_prominent, bbb_overall_weight, bbb_set_central, bbb_set_lateral,
    new_brow_bone_bossing_morph, BrowBoneBossingMorph,
};

pub mod mandible_angle_morph;
pub use mandible_angle_morph::{
    mand_blend, mand_is_square, mand_overall_weight,
    mand_set_gonial_angle as mand_set_gonial_angle_v2, mand_set_masseter, new_mandible_angle_morph,
    MandibleAngleMorph,
};

pub mod genial_tubercle_morph;
pub use genial_tubercle_morph::{
    gt_blend, gt_is_prominent, gt_overall_weight, gt_set_prominence, gt_set_width,
    new_genial_tubercle_morph, GenialTubercleMorph,
};

pub mod temple_fossa_morph;
pub use temple_fossa_morph::{
    new_temple_fossa_morph, tf_blend, tf_is_hollow, tf_overall_weight, tf_set_depth, tf_set_width,
    TempleFossaMorph,
};

pub mod infraorbital_morph;
pub use infraorbital_morph::{
    io_blend, io_is_hollow, io_overall_weight, io_set_hollow, io_set_puffiness,
    new_infraorbital_morph, InfraorbitalMorph,
};

pub mod upper_lip_roll_morph;
pub use upper_lip_roll_morph::{
    new_upper_lip_roll_morph, ulr_blend, ulr_is_everted, ulr_overall_weight, ulr_set_eversion,
    ulr_set_tubercle, UpperLipRollMorph,
};

pub mod lower_lip_roll_morph;
pub use lower_lip_roll_morph::{
    llr_blend, llr_is_everted, llr_overall_weight, llr_set_eversion, llr_set_fullness,
    new_lower_lip_roll_morph, LowerLipRollMorph,
};

pub mod vermillion_width_morph;
pub use vermillion_width_morph::{
    new_vermillion_width_morph, vw_blend, vw_is_wide, vw_overall_weight, vw_set_lower,
    vw_set_upper, VermillionWidthMorph,
};

pub mod nasal_dorsum_morph;
pub use nasal_dorsum_morph::{
    nd_blend, nd_is_humped, nd_overall_weight, nd_set_height, nd_set_hump, nd_set_width,
    new_nasal_dorsum_morph, NasalDorsumMorph,
};

pub mod scalp_morph;
pub use scalp_morph::{
    new_scalp_morph, scalp_blend, scalp_is_receding, scalp_overall_weight, scalp_set_crown_width,
    scalp_set_hairline, scalp_set_recession, ScalpMorph,
};

pub mod orbital_rim_morph;
pub use orbital_rim_morph::{
    new_orbital_rim_morph, or_blend, or_is_deep, or_overall_weight, or_set_depth, or_set_height,
    or_set_roundness, OrbitalRimMorph,
};

pub mod nasal_root_morph;
pub use nasal_root_morph::{
    new_nasal_root_morph, nr_blend as nrm_blend, nr_is_deep, nr_overall_weight,
    nr_set_depth as nrm_set_depth, nr_set_width as nrm_set_width, NasalRootMorph,
};

pub mod philtrum_ridge_morph;
pub use philtrum_ridge_morph::{
    new_philtrum_ridge_morph, pr_blend, pr_is_defined, pr_overall_weight, pr_set_definition,
    pr_set_length, pr_set_width, PhiltrumRidgeMorph,
};

pub mod cupid_bow_morph;
pub use cupid_bow_morph::{
    cb_blend, cb_is_pronounced, cb_overall_weight, cb_set_peak_height, cb_set_valley_depth,
    cb_set_width, new_cupid_bow_morph, CupidBowMorph,
};

pub mod oral_commissure_morph;
pub use oral_commissure_morph::{
    new_oral_commissure_morph, oc_blend, oc_is_downturned, oc_overall_weight, oc_set_angle,
    oc_set_depth, OralCommissureMorph,
};

pub mod labiomental_morph;
pub use labiomental_morph::{
    lm_blend, lm_is_deep, lm_overall_weight, lm_set_fold_depth, lm_set_width,
    new_labiomental_morph, LabiomentalMorph,
};

pub mod gnathion_morph;
pub use gnathion_morph::{
    gn_blend, gn_is_elongated, gn_overall_weight, gn_set_roundness, gn_set_vertical_drop,
    new_gnathion_morph, GnathionMorph,
};

pub mod ramus_morph;
pub use ramus_morph::{
    new_ramus_morph, rm_blend, rm_is_tall, rm_overall_weight, rm_set_angle, rm_set_height,
    rm_set_width, RamusMorph,
};

pub mod condyle_morph;
pub use condyle_morph::{
    cy_blend, cy_is_displaced, cy_overall_weight, cy_set_offset_x, cy_set_offset_y, cy_set_size,
    new_condyle_morph, CondyleMorph,
};

pub mod symphysis_morph;
pub use symphysis_morph::{
    new_symphysis_morph, sy_blend, sy_is_wide, sy_overall_weight, sy_set_curvature, sy_set_height,
    sy_set_width, SymphysisMorph,
};

pub mod coronoid_morph;
pub use coronoid_morph::{
    cor_blend, cor_is_neutral, cor_overall_weight, cor_set_apex_curve, cor_set_breadth,
    cor_set_height, cor_to_json, new_coronoid_morph, CoronoidMorph,
};

pub mod styloid_morph;
pub use styloid_morph::{
    new_styloid_morph, sty_blend, sty_is_neutral, sty_overall_weight, sty_set_angle_offset,
    sty_set_length, sty_set_tip_sharpness, sty_to_json, StyloidMorph,
};

pub mod zygomatic_body_morph;
pub use zygomatic_body_morph::{
    new_zygomatic_body_morph, zyg_blend, zyg_is_neutral, zyg_overall_weight, zyg_set_breadth,
    zyg_set_height, zyg_set_projection, zyg_to_json, ZygomaticBodyMorph,
};

pub mod frontal_sinus_morph;
pub use frontal_sinus_morph::{
    fsin_blend, fsin_is_neutral, fsin_overall_weight, fsin_set_bossing, fsin_set_lateral_extent,
    fsin_set_slope, fsin_to_json, new_frontal_sinus_morph, FrontalSinusMorph,
};

pub mod parietal_morph;
pub use parietal_morph::{
    new_parietal_morph, par_blend, par_is_neutral, par_overall_weight, par_set_boss,
    par_set_coronal_curve, par_set_sagittal_curve, par_to_json, ParietalMorph,
};

pub mod occipital_morph;
pub use occipital_morph::{
    new_occipital_morph, occ_blend, occ_is_neutral, occ_overall_weight, occ_set_nuchal_width,
    occ_set_protuberance, occ_set_squama_curve, occ_to_json, OccipitalMorph,
};

pub mod cervical_morph;
pub use cervical_morph::{
    cerv_blend, cerv_is_neutral, cerv_overall_weight, cerv_set_forward_head, cerv_set_lateral_list,
    cerv_set_lordosis, cerv_to_json, new_cervical_morph, CervicalMorph,
};

pub mod iliac_crest_morph;
pub use iliac_crest_morph::{
    iliac_blend, iliac_is_neutral, iliac_overall_weight, iliac_set_asis_prominence,
    iliac_set_flare, iliac_set_height, iliac_to_json, new_iliac_crest_morph, IliacCrestMorph,
};

pub mod pubic_arch_morph;
pub use pubic_arch_morph::{
    new_pubic_arch_morph, pub_angle_deg, pub_angle_rad, pub_blend, pub_is_neutral,
    pub_overall_weight, pub_set_angle_factor, pub_set_concavity, pub_set_symphysis_height,
    pub_to_json, PubicArchMorph,
};

pub mod delta_painter;
pub use delta_painter::{BrushFalloff, DeltaPainter, MirrorAxis, MorphTargetData, PaintBrush};

pub mod target_tools;
pub use target_tools::{
    add_targets, clamp_target, merge_targets as merge_target_deltas, mirror_target,
    scale_target as scale_target_deltas, sparsify_target, subtract_targets,
    SymmetryReport as TargetSymmetryReport, TargetInfo, TargetInspector, TargetValidator,
    ValidationWarning, WarningKind,
};

// ── Advanced morph features ─────────────────────────────────────────────────

pub mod expression_blend;
pub use expression_blend::{
    default_expression_defs, ExpressionBlender as ExprBlender, ExpressionDef,
};

pub mod neural_blend;
pub use neural_blend::{
    softmax as neural_softmax, NeuralBlendNet, NeuralBlendTrainer, BODY_TARGET_NAMES,
    HIDDEN_SIZE as NEURAL_HIDDEN_SIZE, INPUT_SIZE as NEURAL_INPUT_SIZE,
};

pub mod pose_retarget;
pub use pose_retarget::{
    JointPoseData, PoseRetargeter, PoseSnapshot, RetargetConfig as PoseRetargetConfig,
    RetargetMapping as PoseRetargetMapping, ScaleMode,
};
