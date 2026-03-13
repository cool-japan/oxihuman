// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
//! Utilities: locale/timezone/calendar, statistics, observability,
//! DDD patterns, math/geometry, memory allocators, file-system stubs,
//! serialization stubs, and security helpers.

#[path = "user_agent_parser.rs"]
pub mod user_agent_parser;
pub use user_agent_parser::{
    browser_name, os_name, parse_user_agent, BrowserFamily, OsFamily, UserAgent,
};

#[path = "locale_formatter.rs"]
pub mod locale_formatter;
pub use locale_formatter::{
    format_date_locale, format_number_locale, format_thousands, locale_currency_symbol,
    LocaleFormatter, LocaleId,
};

#[path = "timezone_offset.rs"]
pub mod timezone_offset;
pub use timezone_offset::{
    convert_utc_minutes, format_offset, known_offsets, offset_difference, parse_offset,
    TimezoneOffset,
};

#[path = "calendar_util.rs"]
pub mod calendar_util;
pub use calendar_util::{
    date_to_julian_day, day_of_week, days_between, days_in_month, is_leap_year, julian_day_to_date,
    CalDate,
};

#[path = "duration_parser.rs"]
pub mod duration_parser;
pub use duration_parser::{add_durations, duration_to_string, parse_duration, IsoDuration};

#[path = "cron_parser.rs"]
pub mod cron_parser;
pub use cron_parser::{
    cron_is_wildcard_all, cron_matches, describe_cron, parse_cron, CronExpr, CronField,
};

#[path = "holiday_calendar.rs"]
pub mod holiday_calendar;
pub use holiday_calendar::{
    is_holiday_in, jp_national_holidays, us_federal_holidays, Holiday, HolidayCalendar, HolidayDate,
};

#[path = "date_range.rs"]
pub mod date_range;
pub use date_range::{
    range_intersection, range_jdn_list, range_union, ranges_overlap, ymd_to_jdn, DateRange,
};

#[path = "fiscal_year.rs"]
pub mod fiscal_year;
pub use fiscal_year::{
    fiscal_quarter, fiscal_year_months, fiscal_year_of, fiscal_year_quarters, FiscalPeriod,
    FiscalYearConfig,
};

#[path = "work_calendar.rs"]
pub mod work_calendar;
pub use work_calendar::{add_business_days, count_bdays, is_business_day, next_bday, WorkCalendar};

#[path = "time_series_buffer.rs"]
pub mod time_series_buffer;
pub use time_series_buffer::{buffer_min_max, buffer_variance, TimeSeriesBuffer, TimeSeriesSample};

#[path = "moving_avg_calc.rs"]
pub mod moving_avg_calc;
pub use moving_avg_calc::{ema_batch, ma_crossover, sma_batch, EmaCalc, SimpleMaCalc};

#[path = "trend_detector.rs"]
pub mod trend_detector;
pub use trend_detector::{
    detect_trend, linear_regression, moving_slope, trend_direction_label, TrendResult,
};

#[path = "anomaly_scorer.rs"]
pub mod anomaly_scorer;
pub use anomaly_scorer::{anomaly_count, flag_anomalies, z_score_batch, AnomalyScorer};

#[path = "outlier_filter.rs"]
pub mod outlier_filter;
pub use outlier_filter::{
    filter_outliers, flag_outliers, iqr_bounds, outlier_count, percentile, winsorize, IqrFilter,
};

#[path = "bucket_histogram.rs"]
pub mod bucket_histogram;
pub use bucket_histogram::{histogram_mean, BucketHistogram};

#[path = "quantile_estimator.rs"]
pub mod quantile_estimator;
pub use quantile_estimator::{median_batch, quantile_batch, P2Quantile};

#[path = "feature_flag.rs"]
pub mod feature_flag;
pub use feature_flag::{
    feature_flag_count, is_feature_enabled, new_flag_registry, register_feature, toggle_feature,
    FeatureFlagEntry, FlagState,
};

#[path = "ab_test_config.rs"]
pub mod ab_test_config;
pub use ab_test_config::{
    ab_add_test, ab_select_variant, ab_total_weight, ab_variant_count, new_ab_test_config,
    AbTestConfig, AbVariant,
};

#[path = "experiment_tracker.rs"]
pub mod experiment_tracker;
pub use experiment_tracker::{
    new_experiment_tracker, tracker_assign, tracker_experiment_count, tracker_get_variant,
    tracker_participant_count, ExperimentAssignment, ExperimentTracker,
};

#[path = "metrics_counter.rs"]
pub mod metrics_counter;
pub use metrics_counter::{
    mc_get, mc_inc, mc_inc_by, mc_reset, mc_total, new_metrics_counter, MetricsCounter,
};

#[path = "metrics_gauge.rs"]
pub mod metrics_gauge;
pub use metrics_gauge::{
    gauge_count, gauge_current, gauge_max, gauge_min, gauge_set, new_metrics_gauge, GaugeEntry,
    MetricsGauge,
};

#[path = "metrics_histogram_sdk.rs"]
pub mod metrics_histogram_sdk;
pub use metrics_histogram_sdk::{
    hist_bucket, hist_count, hist_mean, hist_record, hist_sum, new_histogram_sdk,
    MetricsHistogramSdk,
};

#[path = "telemetry_span.rs"]
pub mod telemetry_span;
pub use telemetry_span::{
    new_telemetry_span, span_duration_us, span_end, span_set_attr, span_set_error, span_set_ok,
    SpanStatus, TelemetrySpan,
};

#[path = "distributed_trace.rs"]
pub mod distributed_trace;
pub use distributed_trace::{
    new_trace_context, trace_child, trace_from_header, trace_is_sampled, trace_to_header,
    TraceContext,
};

#[path = "log_aggregator.rs"]
pub mod log_aggregator;
pub use log_aggregator::{
    agg_count, agg_count_level, agg_push, agg_set_min, new_log_aggregator, AggLogEntry,
    AggLogLevel, LogAggregator,
};

#[path = "audit_log.rs"]
pub mod audit_log;
pub use audit_log::{
    audit_chain_hash, audit_count, audit_filter_actor, audit_record, new_audit_log, AuditEntry,
    AuditLog,
};

#[path = "change_log.rs"]
pub mod change_log;
pub use change_log::{
    cl_append, cl_count, cl_filter_kind, cl_since, new_change_log, ChangeEntry, ChangeLog,
};

#[path = "notification_queue.rs"]
pub mod notification_queue;
pub use notification_queue::{
    new_notification_queue, nq_is_empty, nq_len, nq_peek_priority, nq_pop, nq_push, NotifPriority,
    Notification as QueueNotification, NotificationQueue,
};

#[path = "task_scheduler.rs"]
pub mod task_scheduler;
pub use task_scheduler::{
    new_task_scheduler, ts_advance, ts_cancel, ts_run_count, ts_schedule, ts_task_count,
    RecurrenceRule, SchedulerTask, TaskScheduler,
};

#[path = "deadline_tracker.rs"]
pub mod deadline_tracker;
pub use deadline_tracker::{
    dt_add, dt_complete, dt_count, dt_met_count, dt_overdue_count, new_deadline_tracker,
    DeadlineEntry, DeadlineStatus, DeadlineTracker,
};

#[path = "quota_manager.rs"]
pub mod quota_manager;
pub use quota_manager::{
    new_quota_manager, qm_available, qm_consume, qm_release, qm_set, qm_utilization, QuotaEntry,
    QuotaManager,
};

#[path = "capacity_planner.rs"]
pub mod capacity_planner;
pub use capacity_planner::{
    cp_add, cp_most_utilized, cp_over_threshold, cp_spec_count, new_capacity_planner,
    CapacityPlanner, CapacitySpec,
};

#[path = "memory_pool_typed.rs"]
pub mod memory_pool_typed;
pub use memory_pool_typed::{new_memory_pool_typed, MemoryPoolTyped};

#[path = "object_arena.rs"]
pub mod object_arena;
pub use object_arena::{new_object_arena, ArenaHandle, Generation, ObjectArena};

#[path = "slab_allocator.rs"]
pub mod slab_allocator;
pub use slab_allocator::{SlabAllocator, SlabKey};

#[path = "buddy_allocator.rs"]
pub mod buddy_allocator;
pub use buddy_allocator::{new_buddy_allocator, BuddyAllocator};

#[path = "region_allocator.rs"]
pub mod region_allocator;
pub use region_allocator::{new_region_allocator, Region, RegionAllocator};

#[path = "gc_stub.rs"]
pub mod gc_stub;
pub use gc_stub::{new_gc_stub, GcId, GcObject, GcState, GcStub};

#[path = "reference_counted.rs"]
pub mod reference_counted;
pub use reference_counted::{
    new_ref_counted as new_manual_ref_counted, rc_count, rc_is_unique,
    RefCounted as ManualRefCounted,
};

#[path = "weak_reference.rs"]
pub mod weak_reference;
pub use weak_reference::{new_weak_pair, weak_is_alive, StrongOwner, WeakRef};

#[path = "copy_on_write.rs"]
pub mod copy_on_write;
pub use copy_on_write::{cow_read, cow_share, new_cow_handle, CowHandle};

#[path = "persistent_hash_map.rs"]
pub mod persistent_hash_map;
pub use persistent_hash_map::{new_persistent_hash_map, PersistentHashMap, PmapVersion};

#[path = "persistent_vector.rs"]
pub mod persistent_vector;
pub use persistent_vector::{new_persistent_vector, PersistentVector, PvecVersion};

#[path = "finger_tree.rs"]
pub mod finger_tree;
pub use finger_tree::{new_finger_tree, FingerTree};

#[path = "rope_string.rs"]
pub mod rope_string;
pub use rope_string::RopeString;

#[path = "gap_buffer.rs"]
pub mod gap_buffer;
pub use gap_buffer::{new_gap_buffer, GapBuffer};

#[path = "piece_table.rs"]
pub mod piece_table;
pub use piece_table::{new_piece_table, Piece, PieceSource, PieceTable};

#[path = "zipper_list.rs"]
pub mod zipper_list;
pub use zipper_list::{new_zipper_list, ZipperList};

#[path = "text_diff_myers.rs"]
pub mod text_diff_myers;
pub use text_diff_myers::{
    apply_diff as apply_myers_diff, diff_lines, diff_stats, edit_distance, is_same,
    EditOp as MyersEditOp, MyersDiff,
};

#[path = "merge_conflict_resolver.rs"]
pub mod merge_conflict_resolver;
pub use merge_conflict_resolver::{
    auto_resolve_ours, auto_resolve_theirs, count_conflicts, format_conflict,
    three_way_merge as three_way_merge_resolve, MergeConfig, MergeResult as ConflictMergeResult,
};

#[path = "patch_generator.rs"]
pub mod patch_generator;
pub use patch_generator::{
    apply_patch_stub, generate_patch, is_identity_patch, serialize_patch, total_changed_lines,
    DiffHunk, UnifiedPatch as GeneratedPatch,
};

#[path = "file_watcher_stub.rs"]
pub mod file_watcher_stub;
pub use file_watcher_stub::{
    drain_and_count, is_watched as path_is_watched, new_file_watcher, watch_paths, FileWatcherStub,
    FsEvent,
};

#[path = "directory_scanner.rs"]
pub mod directory_scanner;
pub use directory_scanner::{
    filter_by_ext, find_by_name, new_scanner, sort_entries, total_size as scanner_total_size,
    DirEntry, DirectoryScanner, ScanConfig,
};

#[path = "file_metadata.rs"]
pub mod file_metadata;
pub use file_metadata::{
    filter_large, largest_file, newest_file, read_metadata_stub, total_size as metadata_total_size,
    FileMetadata, MetadataStore,
};

#[path = "path_normalizer.rs"]
pub mod path_normalizer;
pub use path_normalizer::{
    file_extension, is_absolute, join_paths, make_absolute, normalize_path,
    strip_prefix as path_strip_prefix, NormalizedPath,
};

#[path = "symlink_resolver.rs"]
pub mod symlink_resolver;
pub use symlink_resolver::{
    detect_cycle, new_symlink_resolver, register_all, resolve_batch, Symlink, SymlinkResolver,
};

#[path = "file_lock_stub.rs"]
pub mod file_lock_stub;
pub use file_lock_stub::{
    new_lock_manager, release_all, try_exclusive, try_shared, FileLockManager, LockMode, LockRecord,
};

#[path = "temp_file_manager.rs"]
pub mod temp_file_manager;
pub use temp_file_manager::{
    alive_total_bytes, create_n, delete_by_path, new_temp_manager, TempFile, TempFileManager,
};

#[path = "archive_reader.rs"]
pub mod archive_reader;
pub use archive_reader::{
    list_matching, open_archive_stub, read_entry_bytes, read_entry_text, total_compressed,
    ArchiveEntry, ArchiveReader,
};

#[path = "archive_writer.rs"]
pub mod archive_writer;
pub use archive_writer::{
    build_archive, estimate_size, new_archive_writer, write_entries, ArchiveWriter, WriteEntry,
};

#[path = "compression_pipeline.rs"]
pub mod compression_pipeline;
pub use compression_pipeline::{
    compress_bytes, estimate_compressed_size, lz4_brotli_pipeline, zstd_pipeline, CompressAlgo,
    CompressResult, CompressionPipeline, PipelineStage as CompressPipelineStage,
};

#[path = "checksum_verifier.rs"]
pub mod checksum_verifier;
pub use checksum_verifier::{
    checksum_map, compute_checksum, crc32_bytes, sha256_stub, verify_checksum, verify_hex,
    Checksum, ChecksumAlgo,
};

#[path = "file_transfer_stub.rs"]
pub mod file_transfer_stub;
pub use file_transfer_stub::{
    cancel_job, new_transfer_manager, tick_all, TransferJob, TransferManager, TransferState,
};

#[path = "object_storage_stub.rs"]
pub mod object_storage_stub;
pub use object_storage_stub::{
    copy_object, download, new_object_storage, upload, ObjectMeta, ObjectStorage, StoredObject,
};

#[path = "error_aggregator.rs"]
pub mod error_aggregator;
pub use error_aggregator::{
    aggregator_clear, aggregator_count, aggregator_has_errors, aggregator_messages,
    aggregator_push, new_error_aggregator, ErrorAggregator,
};

#[path = "capability_flags.rs"]
pub mod capability_flags;
pub use capability_flags::{
    flags_all, flags_any, flags_clear, flags_count, flags_reset, flags_set, flags_test,
    new_capability_flags, CapabilityFlags,
};

#[path = "dep_graph_simple.rs"]
pub mod dep_graph_simple;
pub use dep_graph_simple::{
    dep_add_edge, dep_add_node, dep_has_cycle, dep_node_count as simple_dep_node_count,
    dep_topo_sort, new_dep_graph, DepGraph,
};

#[path = "resource_tracker.rs"]
pub mod resource_tracker;
pub use resource_tracker::{
    new_resource_tracker, tracker_allocate, tracker_available, tracker_free, tracker_is_full,
    tracker_peak, tracker_reset, tracker_utilization, ResourceTracker,
};

#[path = "lazy_eval.rs"]
pub mod lazy_eval;
pub use lazy_eval::{
    lazy_get_or_compute, lazy_invalidate, lazy_is_computed, lazy_set, new_lazy_f32, LazyValue,
};

#[path = "feature_gate.rs"]
pub mod feature_gate;
pub use feature_gate::{
    gate_clear, gate_disable, gate_enable, gate_feature_count, gate_is_enabled, gate_toggle,
    new_feature_gate, FeatureGate,
};

#[path = "snapshot_manager.rs"]
pub mod snapshot_manager;
pub use snapshot_manager::{
    new_snapshot_manager, snapshot_clear, snapshot_count, snapshot_get, snapshot_is_full,
    snapshot_latest, snapshot_save, SnapshotManager,
};

#[path = "health_monitor.rs"]
pub mod health_monitor;
pub use health_monitor::{
    health_all_ok, health_count, health_failing_count, health_register, health_summary,
    health_update, new_health_monitor, HealthMonitor,
};

#[path = "error_audit_log.rs"]
pub mod error_audit_log;
pub use error_audit_log::{
    audit_event_by_actor, audit_event_clear, audit_event_count, audit_event_last,
    audit_event_record, audit_event_since, new_audit_event_log, AuditEventEntry, AuditEventLog,
};

#[path = "clock_version_vector.rs"]
pub mod clock_version_vector;
pub use clock_version_vector::{
    cvv_concurrent, cvv_dominates, cvv_get, cvv_increment, cvv_merge, cvv_node_count,
    new_clock_version_vector, ClockVersionVector,
};

#[path = "compression_stub.rs"]
pub mod compression_stub;
pub use compression_stub::{compress_rle, compression_ratio, decompress_rle, is_compressible};

#[path = "token_bucket_limiter.rs"]
pub mod token_bucket_limiter;
pub use token_bucket_limiter::{
    limiter_available, limiter_consume, limiter_is_full, limiter_refill, limiter_reset,
    new_rate_limiter_tb, TokenBucketLimiter,
};

#[path = "simple_message_queue.rs"]
pub mod simple_message_queue;
pub use simple_message_queue::{
    new_message_queue_simple, smq_clear, smq_is_empty, smq_is_full, smq_len, smq_peek, smq_pop,
    smq_push, SimpleMessageQueue,
};

#[path = "config_diff.rs"]
pub mod config_diff;
pub use config_diff::{
    config_added_keys, config_changed_keys, config_diff_count, config_is_identical,
    config_removed_keys,
};

#[path = "resolve_path_utils.rs"]
pub mod resolve_path_utils;
pub use resolve_path_utils::{
    path_basename, path_dirname, path_ext, path_is_abs, path_join_parts, resolve_path,
    resolve_path_normalize,
};

#[path = "disjoint_set.rs"]
pub mod disjoint_set;
pub use disjoint_set::{
    ds_component_count, ds_connected, ds_find, ds_same, ds_size, ds_union, new_disjoint_set,
    DisjointSet,
};

#[path = "bloom_filter_prob.rs"]
pub mod bloom_filter_prob;
pub use bloom_filter_prob::{
    bloom_prob_bit_count, bloom_prob_estimated_fp_rate, bloom_prob_hash_count, bloom_prob_insert,
    bloom_prob_may_contain, bloom_prob_reset, new_bloom_filter_prob, BloomFilterProb,
};

#[path = "fenwick_tree_v2.rs"]
pub mod fenwick_tree_v2;
pub use fenwick_tree_v2::{
    ft2_add, ft2_len, ft2_point_query, ft2_prefix_sum, ft2_range_sum, new_fenwick_tree_v2,
    FenwickTreeV2,
};

#[path = "segment_tree_range.rs"]
pub mod segment_tree_range;
pub use segment_tree_range::{
    new_segment_tree_range, seg_range_query_max, seg_range_query_min, seg_range_size,
    seg_range_update, SegmentTreeRange,
};

#[path = "interval_tree_simple.rs"]
pub mod interval_tree_simple;
pub use interval_tree_simple::{
    itree_simple_contains_point, itree_simple_count_overlaps, itree_simple_insert,
    itree_simple_query_overlaps, itree_simple_size, new_interval_tree_simple, IntervalSimple,
    IntervalTreeSimple,
};

#[path = "hash_ring.rs"]
pub mod hash_ring;
pub use hash_ring::{
    add_ring_node, get_ring_node, new_hash_ring, remove_ring_node, ring_distribution,
    ring_node_count, ring_rebalance, ring_to_json, HashRing,
};

#[path = "hash_ring_new.rs"]
pub mod hash_ring_new;
pub use hash_ring_new::{
    new_hash_ring_new, ring_new_add_node, ring_new_get_node, ring_new_is_empty,
    ring_new_node_count, ring_new_remove_node, HashRingNew,
};

#[path = "rolling_hash.rs"]
pub mod rolling_hash;
pub use rolling_hash::{
    find_pattern, new_rolling_hash, rolling_hash_push, rolling_hash_value,
    simple_hash as rh_simple_hash, RollingHash,
};

#[path = "rolling_hash_new.rs"]
pub mod rolling_hash_new;
pub use rolling_hash_new::{
    new_rolling_hash_new, rh_new_hash, rh_new_push, rh_new_reset, rh_new_window_full,
    rh_new_window_size, RollingHashNew,
};

#[path = "edit_distance.rs"]
pub mod edit_distance;
pub use edit_distance::{
    common_prefix_len, hamming_distance, is_within_distance, jaro_similarity, levenshtein,
};

#[path = "edit_distance_lev.rs"]
pub mod edit_distance_lev;
pub use edit_distance_lev::{
    edit_distance_bounded_lev, edit_distance_lev, edit_is_close_lev, edit_similarity_lev,
    longest_common_subsequence_lev,
};

#[path = "fuzzy_matcher.rs"]
pub mod fuzzy_matcher;
pub use fuzzy_matcher::{
    fuzzy_best_match, fuzzy_match_score, fuzzy_matches, fuzzy_rank_candidates,
};

#[path = "tokenizer_simple.rs"]
pub mod tokenizer_simple;
pub use tokenizer_simple::{
    token_count as word_token_count, token_frequency, tokenize_sentences, tokenize_words,
    unique_tokens,
};

#[path = "prefix_sum_2d.rs"]
pub mod prefix_sum_2d;
pub use prefix_sum_2d::{new_prefix_sum_2d, ps2d_cols, ps2d_query, ps2d_rows, PrefixSum2d};

#[path = "skip_list_simple.rs"]
pub mod skip_list_simple;
pub use skip_list_simple::{
    new_skip_list_simple, skip_simple_contains, skip_simple_insert, skip_simple_len,
    skip_simple_range, skip_simple_remove, SkipListSimple,
};

#[path = "regex_stub.rs"]
pub mod regex_stub;
pub use regex_stub::{regex_count_matches, regex_first_match, regex_match, regex_match_all};

#[path = "aho_corasick_stub.rs"]
pub mod aho_corasick_stub;
pub use aho_corasick_stub::{
    ac_stub_any_match, ac_stub_count_matches, ac_stub_first_match, ac_stub_search,
};

#[path = "suffix_array_stub.rs"]
pub mod suffix_array_stub;
pub use suffix_array_stub::{
    build_suffix_array_stub, lcp_array_stub, sa_stub_count_occurrences, sa_stub_find_all,
    sa_stub_longest_repeated_substring, sa_stub_search,
};

#[path = "b_tree_simple.rs"]
pub mod b_tree_simple;
pub use b_tree_simple::{
    btree_simple_get, btree_simple_insert, btree_simple_len, btree_simple_range,
    btree_simple_remove, new_btree_simple, BTreeSimple,
};

#[path = "merkle_tree.rs"]
pub mod merkle_tree;
pub use merkle_tree::{
    merkle_combine_hashes, merkle_leaf_count, merkle_root, merkle_verify_leaf, new_merkle_tree,
    MerkleTree,
};

#[path = "base64_stub.rs"]
pub mod base64_stub;
pub use base64_stub::{
    base64_decode as b64s_decode, base64_decode_config as b64s_decode_config,
    base64_decode_url_safe as b64s_decode_url_safe, base64_decoded_len as b64s_decoded_len,
    base64_encode as b64s_encode, base64_encode_config as b64s_encode_config,
    base64_encode_mime as b64s_encode_mime, base64_encode_url_safe as b64s_encode_url_safe,
    base64_encoded_len as b64s_encoded_len, base64_is_valid as b64s_is_valid,
    base64_is_valid_config as b64s_is_valid_config,
    base64_is_valid_url_safe as b64s_is_valid_url_safe, Base64Config as B64sConfig,
    Base64DecodeError as B64sDecodeError, Base64Decoder as B64sDecoder,
    Base64Encoder as B64sEncoder, Base64Padding as B64sPadding, Base64Variant as B64sVariant,
};

#[path = "hex_codec_new.rs"]
pub mod hex_codec_new;
pub use hex_codec_new::{
    hex_byte_count_new, hex_decode_new, hex_encode_new, hex_encode_upper_new, hex_is_valid_new,
};

#[path = "uuid_generator.rs"]
pub mod uuid_generator;
pub use uuid_generator::{
    uuid_from_bytes, uuid_from_u128, uuid_is_valid_string, uuid_nil, uuid_to_string, uuid_version,
    Uuid,
};

#[path = "bitset_fixed.rs"]
pub mod bitset_fixed;
pub use bitset_fixed::{
    bitset_and, bitset_clear, bitset_count_ones, bitset_count_zeros, bitset_flip, bitset_get,
    bitset_or, bitset_set, new_bitset_fixed, BitsetFixed,
};

#[path = "varint_u64_codec.rs"]
pub mod varint_u64_codec;
pub use varint_u64_codec::{
    varint_decode_i64, varint_decode_u64, varint_encode_i64, varint_encode_u64,
    varint_encoded_size_u64,
};

#[path = "endian_utils.rs"]
pub mod endian_utils;
pub use endian_utils::{
    f32_from_le_bytes, f32_to_le_bytes, is_little_endian, u16_from_le, u16_to_le, u32_from_be,
    u32_from_le, u32_to_be, u32_to_le,
};

#[path = "crc_simple.rs"]
pub mod crc_simple;
pub use crc_simple::{
    crc16 as crc16_simple, crc16_check as crc16_simple_check, crc16_update, crc8 as crc8_simple,
    crc8_check as crc8_simple_check, crc8_update,
};

#[path = "fletcher_checksum.rs"]
pub mod fletcher_checksum;
pub use fletcher_checksum::{
    fletcher16 as fletcher16_cs, fletcher16_check as fletcher16_cs_check, fletcher16_combine,
    fletcher32, fletcher32_check,
};

#[path = "gray_code.rs"]
pub mod gray_code;
pub use gray_code::{from_gray, gray_bits, gray_distance, gray_next, gray_prev, to_gray};

#[path = "morton_code.rs"]
pub mod morton_code;
pub use morton_code::{
    morton_decode_2d, morton_decode_3d, morton_encode_2d, morton_encode_3d, morton_neighbor,
};

#[path = "hilbert_curve.rs"]
pub mod hilbert_curve;
pub use hilbert_curve::{
    hilbert_d_to_xy, hilbert_max_index, hilbert_order_for_size, hilbert_xy_to_d,
};

#[path = "packed_array.rs"]
pub mod packed_array;
pub use packed_array::{
    new_packed_array, packed_bits_per_elem, packed_get, packed_len, packed_set,
    packed_storage_bytes, PackedArray,
};

#[path = "hamming_code.rs"]
pub mod hamming_code;
pub use hamming_code::{
    hamming_decode_nibble, hamming_encode_nibble, hamming_introduce_error, hamming_is_valid,
    hamming_syndrome,
};

#[path = "bitmask_flags.rs"]
pub mod bitmask_flags;
pub use bitmask_flags::{
    bmf_clear, bmf_count_set, bmf_raw, bmf_set, bmf_set_by_index, bmf_test, new_bitmask_flags,
    BitmaskFlags,
};

#[path = "atomic_counter_stub.rs"]
pub mod atomic_counter_stub;
pub use atomic_counter_stub::{
    counter_add, counter_compare_and_swap, counter_decrement, counter_get, counter_increment,
    counter_reset, new_atomic_counter, AtomicCounter,
};

#[path = "string_utils.rs"]
pub mod string_utils;
pub use string_utils::{
    str_camel_to_snake, str_capitalize, str_count_char, str_pad_left, str_pad_right, str_repeat,
    str_reverse, str_snake_to_camel, str_truncate,
};

#[path = "iterator_utils.rs"]
pub mod iterator_utils;
pub use iterator_utils::{
    chunks_of, drop_while_vec, flat_map_vec, intersperse, partition_vec, take_while_vec,
    zip_with_vecs,
};

#[path = "result_utils.rs"]
pub mod result_utils;
pub use result_utils::{
    collect_results, first_ok, map_err_str, ok_or_else_str, transpose_option_result,
    unwrap_or_default_str,
};

#[path = "observer_pattern.rs"]
pub mod observer_pattern;
pub use observer_pattern::{
    bus_clear, bus_emit, bus_event_count, bus_events_of_type, bus_has_event_type,
    new_event_bus_observer, ObserverEventBus,
};

#[path = "pipeline_pattern.rs"]
pub mod pipeline_pattern;
pub use pipeline_pattern::{
    context_advance, context_get, context_set, context_stage, new_pipeline_context_struct,
    new_pipeline_struct, pipeline_stage_count, PipelineContextStruct, PipelineStruct,
};

#[path = "specification_pattern.rs"]
pub mod specification_pattern;
pub use specification_pattern::{
    new_spec, spec_and, spec_name, spec_not, spec_or, spec_range, spec_satisfies_range, Spec,
};

#[path = "entity_id.rs"]
pub mod entity_id;
pub use entity_id::{
    entity_id_is_same_kind, entity_id_kind, entity_id_nil, entity_id_parse, entity_id_to_string,
    entity_id_value, new_entity_id, EntityId,
};

#[path = "value_object.rs"]
pub mod value_object;
pub use value_object::{
    money_add, money_to_string, new_money, new_percentage, percentage_clamp,
    percentage_from_fraction, percentage_to_fraction, Money, Percentage,
};

#[path = "aggregate_root.rs"]
pub mod aggregate_root;
pub use aggregate_root::{
    aggregate_apply_event, aggregate_clear_events, aggregate_id, aggregate_increment_version,
    aggregate_pending_events, aggregate_version, new_aggregate_root, AggregateRoot,
};

#[path = "domain_event.rs"]
pub mod domain_event;
pub use domain_event::{
    event_is_type, event_to_json, events_after, events_by_aggregate, events_of_type,
    new_domain_event, DomainEvent,
};

#[path = "command_bus_ddd.rs"]
pub mod command_bus_ddd;
pub use command_bus_ddd::{
    log_clear, log_command_count, log_commands_by_name, log_dispatch, log_last_command,
    new_command_log, new_ddd_command, CommandLog, DddCommand,
};

#[path = "query_bus.rs"]
pub mod query_bus;
pub use query_bus::{
    new_query, new_query_result, query_get_param, query_result_data, query_result_is_success,
    query_set_param, Query, QueryResult,
};

#[path = "repository_stub.rs"]
pub mod repository_stub;
pub use repository_stub::{
    new_string_repo, repo_all_ids, repo_count, repo_delete, repo_exists, repo_find, repo_save,
    StringRepo,
};

#[path = "event_sourcing.rs"]
pub mod event_sourcing;
pub use event_sourcing::{
    es_append, es_events_for, es_latest_version, es_replay, es_total_events, new_event_store,
    EventStore,
};

#[path = "strategy_pattern.rs"]
pub mod strategy_pattern;
pub use strategy_pattern::{
    new_sort_strategy, sort_apply_f32, sort_apply_str, sort_is_ascending, sort_reverse,
    sort_strategy_name, SortStrategy,
};

#[path = "option_ext.rs"]
pub mod option_ext;
pub use option_ext::{
    option_count, option_filter_positive, option_map_or_zero, option_or_default_f32, option_sum,
    option_to_result, option_zip_with,
};

// ── Wave 151A: Core Math Modules ────────────────────────────────────────────

#[path = "vector_math.rs"]
pub mod vector_math;
pub use vector_math::{
    vec3_add, vec3_cross, vec3_dot, vec3_len, vec3_lerp, vec3_norm, vec3_scale, vec3_sub,
};

#[path = "color_math.rs"]
pub mod color_math;
pub use color_math::{
    color_lerp as cm_color_lerp, hsl_to_rgb as cm_hsl_to_rgb, hsv_to_rgb as cm_hsv_to_rgb,
    linear_to_srgb_cm, luminance_cm, rgb_to_hsl as cm_rgb_to_hsl, rgb_to_hsv as cm_rgb_to_hsv,
    srgb_to_linear_cm,
};

#[path = "noise_functions.rs"]
pub mod noise_functions;
pub use noise_functions::{
    checkerboard, fractal_noise_2d, gradient_noise_1d, ridged_noise_2d, turbulence_2d,
    value_noise_2d, white_noise,
};

#[path = "geometry_2d.rs"]
pub mod geometry_2d;
pub use geometry_2d::{
    circle_area, dist_2d, point_in_circle, point_in_rect, point_in_triangle_2d,
    polygon_perimeter_2d, segment_intersect, triangle_area_2d,
};

#[path = "geometry_3d.rs"]
pub mod geometry_3d;
pub use geometry_3d::{
    barycentric_2d, box_volume, closest_point_on_segment, dist_3d, ray_plane_intersect,
    ray_sphere_intersect, sphere_volume, triangle_normal,
};

#[path = "matrix2x2.rs"]
pub mod matrix2x2;
pub use matrix2x2::{
    mat2_det as mat2x2_det, mat2_identity as mat2x2_identity, mat2_inv, mat2_mul as mat2x2_mul,
    mat2_transform as mat2x2_transform, mat2_transpose as mat2x2_transpose, Mat2 as Mat2x2,
};

#[path = "matrix3x3.rs"]
pub mod matrix3x3;
pub use matrix3x3::{
    mat3_det as mat3x3_det, mat3_from_rotation_z, mat3_from_scale,
    mat3_identity as mat3x3_identity, mat3_mul as mat3x3_mul, mat3_transform as mat3x3_transform,
    mat3_transpose as mat3x3_transpose, Mat3 as Mat3x3,
};

#[path = "easing_functions.rs"]
pub mod easing_functions;
pub use easing_functions::{
    ease_bounce_out, ease_elastic_out, ease_in_cubic as ease_in_cubic_fn, ease_in_out_quad,
    ease_in_out_sine, ease_in_quad, ease_linear as ease_linear_fn,
    ease_out_cubic as ease_out_cubic_fn, ease_out_quad,
};

#[path = "transform3d.rs"]
pub mod transform3d;
pub use transform3d::{
    transform_apply, transform_combine, transform_identity, transform_inverse_translation,
    transform_lerp, transform_rotate, transform_scale_uniform, transform_to_mat4,
    transform_translate, Transform3d,
};

#[path = "quaternion_math.rs"]
pub mod quaternion_math;
pub use quaternion_math::{
    quat_conjugate as qm_quat_conjugate, quat_from_axis_angle as quat_from_axis_angle_math,
    quat_identity as quat_identity_math, quat_mul as quat_mul_math, quat_norm_sq,
    quat_normalize as quat_normalize_math, quat_rotate_vec3 as quat_rotate_vec3_math,
    quat_slerp as quat_slerp_math, quat_to_euler, QuatMath,
};

#[path = "random_utils.rs"]
pub mod random_utils;
pub use random_utils::{lcg_choose, lcg_normal, lcg_sample_uniform, lcg_shuffle, Lcg};

#[path = "statistics_utils.rs"]
pub mod statistics_utils;
pub use statistics_utils::{
    max_val, mean, median, min_val, pearson_r, percentile as su_percentile, std_dev, variance,
};

#[path = "bezier_curve.rs"]
pub mod bezier_curve;
pub use bezier_curve::{
    bezier_cubic, bezier_cubic_2d, bezier_cubic_sample, bezier_cubic_tangent, bezier_quadratic,
    bezier_quadratic_2d,
};

#[path = "spline_curve.rs"]
pub mod spline_curve;
pub use spline_curve::{
    catmull_rom_2d as catmull_rom_2d_spline, catmull_rom_3d, catmull_rom_chain_2d, catmull_rom_f32,
    catmull_rom_tangent_f32,
};

#[path = "color_palette_gen.rs"]
pub mod color_palette_gen;
pub use color_palette_gen::{
    palette_analogous, palette_complementary, palette_cosine, palette_gradient,
    palette_monochromatic, palette_rainbow, palette_triadic,
};

#[path = "interpolation.rs"]
pub mod interpolation;
pub use interpolation::{
    bilinear_interp, cubic_interp, hermite_interp, inverse_lerp_f32, lerp_f32 as lerp_f32_interp,
    smootherstep, smoothstep_f32,
};

pub use matrix2x2::{
    mat2_from_angle, mat2_inverse as mat2x2_inverse, mat2_mul_vec2, mat2_scale as mat2_scale_fn,
};

pub use matrix3x3::{
    mat3_from_axis_angle, mat3_inverse as mat3x3_inverse, mat3_mul_vec3, mat3_scale as mat3x3_scale,
};

pub use easing_functions::{
    ease_in_elastic as ef_ease_in_elastic, ease_out_bounce as ef_ease_out_bounce,
};

#[path = "frequency_analyzer.rs"]
pub mod frequency_analyzer;
pub use frequency_analyzer::{
    dc_component, magnitude_spectrum, new_frequency_analyzer, peak_bin_index, FrequencyAnalyzer,
    FrequencyBin,
};

#[path = "histogram_builder.rs"]
pub mod histogram_builder;
pub use histogram_builder::{new_histogram, HistBin, HistogramBuilder};

#[path = "graph_search.rs"]
pub mod graph_search;
pub use graph_search::{new_adj_graph, AdjGraph};

#[path = "priority_queue_ext.rs"]
pub mod priority_queue_ext;
pub use priority_queue_ext::{new_priority_queue_ext, PqEntry, PriorityQueueExt};

#[path = "matrix_solver.rs"]
pub mod matrix_solver;
pub use matrix_solver::{determinant, gaussian_solve, identity_matrix, mat_vec_mul, residual_norm};

#[path = "polynomial_eval.rs"]
pub mod polynomial_eval;
pub use polynomial_eval::{horner_eval, new_polynomial, poly_deriv, poly_eval, Polynomial};

#[path = "running_statistics.rs"]
pub mod running_statistics;
pub use running_statistics::{new_running_stats, RunningStatistics};

#[path = "string_search.rs"]
pub mod string_search;
pub use string_search::{new_string_searcher, StringSearcher};

#[path = "median_filter.rs"]
pub mod median_filter;
pub use median_filter::{median_filter_1d, new_median_filter, slice_median, MedianFilter};

#[path = "moving_average.rs"]
pub mod moving_average;
pub use moving_average::{
    apply_ema, apply_sma, new_ema, new_sma, new_wma, ExponentialMovingAverage, SimpleMovingAverage,
    WeightedMovingAverage,
};

#[path = "color_quantizer.rs"]
pub mod color_quantizer;
pub use color_quantizer::{
    color_dist_sq, new_color_quantizer, quantize_pixels, ColorQuantizer, RgbColor,
};

#[path = "hyperloglog.rs"]
pub mod hyperloglog;
pub use hyperloglog::HyperLogLog;

#[path = "count_min_sketch.rs"]
pub mod count_min_sketch;
pub use count_min_sketch::CountMinSketch;

#[path = "t_digest.rs"]
pub mod t_digest;
pub use t_digest::{Centroid, TDigest};

#[path = "bloom_filter_counting.rs"]
pub mod bloom_filter_counting;
pub use bloom_filter_counting::{
    cbf_contains_str, cbf_insert_str, cbf_remove_str, new_counting_bloom_filter,
    CountingBloomFilter,
};

#[path = "reservoir_sample.rs"]
pub mod reservoir_sample;
pub use reservoir_sample::{
    feed_weighted, new_reservoir_sampler, sample_indices, sample_slice, ReservoirSampler,
};

#[path = "hash_grid.rs"]
pub mod hash_grid;
pub use hash_grid::{hg_insert_2d, hg_query_2d, new_hash_grid, HashGrid, HgPoint3};

#[path = "kd_tree.rs"]
pub mod kd_tree;
pub use kd_tree::{new_kd_tree, new_kd_tree_2d, KdPoint3 as KdPoint3Simple, KdTree3};

#[path = "octree_simple.rs"]
pub mod octree_simple;
pub use octree_simple::{new_simple_octree, OctAabb, SimpleOctree3};

#[path = "bvh_simple.rs"]
pub mod bvh_simple;
pub use bvh_simple::{new_bvh, BvhAabb, BvhPrimitive, SimpleBvh};

#[path = "grid_index.rs"]
pub mod grid_index;
pub use grid_index::{new_grid_index, new_grid_index_region, GridIndex};

#[path = "ear_clip_triangulate.rs"]
pub mod ear_clip_triangulate;
pub use ear_clip_triangulate::{
    ear_clip, ear_clip_flat, is_convex, polygon_bbox,
    polygon_signed_area as ec_polygon_signed_area, EcPoint,
};

#[path = "huffman_stub.rs"]
pub mod huffman_stub;
#[path = "lz77_stub.rs"]
pub mod lz77_stub;
pub use huffman_stub::{
    build_frequency_table, decode_symbol, encode_symbol, huffman_decode, huffman_encode,
    table_size, BitReader, BitWriter, HuffNode, HuffmanCodeTable, HuffmanError, HuffmanSymbol,
    HuffmanTable, HuffmanTree,
};

#[path = "security.rs"]
pub mod security;
pub use security::{
    checked_stride_offset, is_safe_content_type, sanitize_path, validate_file_size, SecurityError,
};
