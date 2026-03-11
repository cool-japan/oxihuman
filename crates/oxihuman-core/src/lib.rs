// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Core utilities, data structures, and algorithms for the OxiHuman engine.
//!
//! This crate is the foundational layer of the OxiHuman workspace. It provides
//! everything that other crates depend on: parsers for `.obj` and `.target`
//! files, the [`Policy`] / [`PolicyProfile`] system for content filtering,
//! spatial indexing with an octree, asset hashing, event buses, undo/redo
//! stacks, plugin registries, and dozens of supporting subsystems.
//!
//! # Quick start
//!
//! ```rust
//! use oxihuman_core::policy::{Policy, PolicyProfile};
//! use oxihuman_core::parser::obj::parse_obj;
//!
//! let obj_src = "v 0 0 0\nv 1 0 0\nv 0 1 0\nvn 0 0 1\nvt 0 0\nf 1/1/1 2/1/1 3/1/1\n";
//! let mesh = parse_obj(obj_src).expect("OBJ parse failed");
//! assert_eq!(mesh.positions.len(), 3);
//!
//! let policy = Policy::new(PolicyProfile::Standard);
//! assert!(policy.is_target_allowed("height", &[]));
//! ```
//!
//! # Content policy
//!
//! All morph targets are filtered through a [`Policy`] before they can affect
//! a mesh. [`PolicyProfile::Standard`] blocks targets whose names or tags
//! contain explicit-content keywords. [`PolicyProfile::Strict`] additionally
//! requires targets to appear in an explicit allowlist.

pub mod category;
pub mod integrity;
pub mod manifest;
pub mod pack_verify;
pub mod parser;
pub mod policy;
pub mod report;
pub mod target_index;

pub use category::TargetCategory;
pub use manifest::AssetManifest;
pub use pack_verify::{
    scan_pack, verify_manifest_present, verify_pack, FileRecord, PackVerifyReport,
};
pub use policy::{Policy, PolicyProfile};
pub use report::{PipelineReport, ReportBuilder, ReportEvent, Severity};
pub use target_index::{TargetEntry, TargetIndex, TargetScanner};
pub mod pack_distribute;
pub use pack_distribute::{
    InstalledPack, PackBuilder, PackDependency, PackIntegrity, PackManifest, PackRegistry,
    PackTargetEntry, PackVerifier,
};

pub mod pack_sign;
pub use pack_sign::{
    double_hash_sign, pack_manifest_hash, read_signature_file, sign_pack_dir, signature_from_hex,
    signature_to_hex, verify_pack_signature, write_signature_file, PackSignature, SignedPack,
};

pub mod asset_pack_builder;
pub use asset_pack_builder::{
    build_alpha_pack, load_pack_from_bytes, AssetPackBuilder, AssetPackEntry, AssetPackIndex,
    AssetPackMeta, MaterialDef, MorphPreset, TargetDelta, TextureAsset, TextureFormat,
};

pub mod plugin_registry;
pub use plugin_registry::{
    default_builtin_plugins, parse_semver, semver_gte, PluginDescriptor, PluginKind, PluginRegistry,
};
pub mod event_bus;
pub use event_bus::{
    make_error_event, make_export_event, make_param_changed_event, Event, EventBus, EventKind,
};
pub mod asset_hash;
pub use asset_hash::{
    hash_bytes, hash_file_content, AssetHash, AssetHasher, AssetRecord, AssetRegistry,
};

pub mod workspace;
pub use workspace::{
    default_workspace_config, workspace_summary, AssetEntry, Workspace, WorkspaceConfig,
};
pub mod metrics;
pub use metrics::{Metric, MetricKind, MetricSample, MetricsRegistry};
pub mod spatial_index;
pub use spatial_index::{
    build_octree, insert_point, k_nearest_neighbors, nearest_neighbor, octree_depth,
    octree_leaf_count, octree_point_count, octree_stats, query_aabb, query_sphere, ray_query,
    Octree, OctreeNode,
};

pub mod command_bus;
pub use command_bus::{
    clear_history, command_descriptions, execute_command, new_command_bus, new_command_state,
    redo_count, redo_last, undo_count, undo_last, BatchCommand, Command, CommandBus, CommandResult,
    CommandState, SetFlagCommand, SetParamCommand,
};

pub mod task_graph;
pub use task_graph::{
    add_task, completed_count, critical_path_length, execute_sequential, failed_count,
    get_ready_tasks, graph_to_json, mark_complete, mark_failed, new_task_graph, pending_count,
    reset_graph, task_count, topological_order, Task, TaskGraph, TaskStatus,
};

pub mod config_schema;
pub use config_schema::{
    add_field, apply_defaults, config_value_get_bool, config_value_get_float, config_value_get_int,
    config_value_get_str, default_render_schema, merge_configs, new_config_schema, schema_to_json,
    validate_field_value, validate_value, ConfigSchema, ConfigValue, SchemaField, SchemaType,
};

pub mod undo_redo;
pub use undo_redo::{
    can_redo, can_undo, clear_undo_history, command_names, future_depth, history_depth,
    new_undo_stack, peek_redo, peek_undo, push_command, redo, truncate_history, undo, UndoCommand,
    UndoStack,
};

pub mod asset_cache;
pub use asset_cache::{
    cache_clear, cache_contains, cache_count, cache_get, cache_hit_rate, cache_insert,
    cache_remove, cache_size, cache_stats, evict_lru, evict_until_fits, most_accessed, new_cache,
    AssetCache, CacheEntry,
};

pub mod plugin_api;
pub use plugin_api::{
    activate_plugin, active_plugins, check_dependencies_met, deactivate_plugin, dependency_order,
    get_plugin, has_dependency, new_registry, plugin_count, plugin_version_string, register_plugin,
    set_plugin_error, unload_plugin, Plugin, PluginApiRegistry, PluginMetadata, PluginState,
};

pub mod serialization;
pub use serialization::{
    f32_array_to_json, json_finalize, json_key_f32, json_key_str, json_key_u32, new_bin_reader,
    new_bin_writer, new_json_builder, read_f32_le, read_str, read_u16_le, read_u32_le, read_u8,
    u32_array_to_json, write_bytes, write_f32_le, write_str, write_u16_le, write_u32_le, write_u8,
    BinReader, BinWriter, JsonBuilder,
};

pub mod event_log;
pub use event_log::{
    clear_log as clear_event_log, error_count, event_count, events_since, filter_by_category,
    filter_by_level, last_event, log_event, log_with_data, new_event_log, serialize_log_json,
    trim_log, warn_count, EventLog, LogEvent, LogLevel,
};

pub mod resource_manager;
pub use resource_manager::{
    fail_resource, failed_count as failed_resource_count, garbage_collect,
    get_by_key as get_resource_by_key, get_resource, load_resource,
    loaded_count as loaded_resource_count, new_resource_manager, register_resource,
    release_resource, retain_resource, total_memory, unload_resource, Resource, ResourceManager,
    ResourceState,
};

pub mod hot_reload;
pub use hot_reload::{
    change_count, changes_for_path, clear_changes as clear_reload_changes,
    default_hot_reload_config, disable_watcher, enable_watcher, extension_matches, is_watched,
    new_watcher, pending_changes, simulate_file_change, unwatch_path, watch_path,
    watched_path_count, ChangeKind, FileChange, HotReloadConfig, HotReloadWatcher,
};

pub mod debug_console;
pub use debug_console::{
    console_clear as clear_debug_console, console_entries_by_severity, console_entry_count,
    console_error_count, console_last_entry, console_log, console_to_string,
    default_debug_console_config, new_debug_console, severity_name, ConsoleEntry, ConsoleSeverity,
    DebugConsole, DebugConsoleConfig,
};

pub mod data_pipeline;
pub use data_pipeline::{
    add_stage as pipeline_add_stage, advance_stage, completed_stage_count, failed_stages,
    get_context_value, mark_stage_complete, mark_stage_failed, mark_stage_skipped, new_pipeline,
    pipeline_progress, pipeline_to_json, reset_pipeline, set_context_value, stage_count,
    DataPipeline, PipelineStage, StageStatus,
};

pub mod type_registry;
pub use type_registry::{
    add_property as registry_add_property, all_categories, get_type as registry_get_type, has_type,
    new_type_registry, property_count, register_type, serializable_types, type_count,
    type_registry_to_json, types_in_category, unregister_type, validate_type_meta, TypeMetadata,
    TypeRegistry,
};

pub mod localization;
pub use localization::{
    add_locale_string, add_locale_table, export_locale_json, has_key as locale_has_key,
    import_locale_strings, key_count as locale_key_count, locale_count, missing_keys,
    new_locale_table, new_localization, set_active_locale, translate, translate_with_context,
    LocaleString, LocaleTable, LocalizationSystem,
};

pub mod version_migration;
pub use version_migration::{
    has_migration_path, is_breaking_change, latest_version, migration_description,
    migration_step_count, new_migration_registry, new_semver, plan_has_breaking, plan_migration,
    register_migration, semver_compare, semver_parse, semver_to_string, MigrationPlan,
    MigrationRegistry, MigrationStep, SemVer,
};

pub mod dependency_resolver;
pub use dependency_resolver::{
    add_dep_node, all_dependents_transitive, dep_graph_to_json, dep_node_count, direct_dependents,
    get_dep_node, has_circular_dependency, missing_dependencies, new_dependency_graph,
    optional_dep_count, remove_dep_node, resolve_dependencies, Dependency, DependencyGraph,
    DependencyNode, ResolveError, ResolveResult,
};

pub mod scheduler;
pub use scheduler::{
    advance_time as scheduler_advance, cancel_task as scheduler_cancel, clear_completed_tasks,
    due_tasks, enabled_task_count, get_scheduled_task, new_scheduler, next_due_time, schedule_once,
    schedule_repeating, set_task_enabled, task_count as scheduler_task_count, tasks_by_priority,
    ScheduledTask, Scheduler, TaskPriority,
};

pub mod profiler;
pub use profiler::{
    average_frame_ns, begin_span, clear_profiler, disable_profiler, enable_profiler, end_frame,
    end_span, frame_count_profiler, hottest_span, last_frame as profiler_last_frame, new_profiler,
    profiler_to_json, span_by_name, span_duration_ns, total_frame_ns, ProfileFrame, ProfileSpan,
    Profiler,
};

pub mod feature_flags;
pub use feature_flags::{
    all_enabled_flags, default_bool_flag, default_int_flag, feature_registry_to_json, flag_count,
    flags_with_tag, get_flag, get_flag_bool, get_flag_int, is_enabled, new_feature_registry,
    register_flag, remove_flag, set_flag_value, FeatureFlag, FeatureFlagRegistry, FlagValue,
};

pub mod user_preferences;
pub use user_preferences::{
    get_bool as pref_get_bool, get_float as pref_get_float, get_int as pref_get_int, get_pref,
    get_string as pref_get_string, mark_clean, new_user_preferences, pref_count,
    preferences_from_pairs, preferences_to_json, prefs_in_category, remove_pref, reset_to_defaults,
    set_pref, PrefValue, Preference, UserPreferences,
};

pub mod notification_system;
pub use notification_system::{
    active_count as active_notification_count, active_notifications, advance_notifications,
    clear_all_notifications, dismiss_notification, has_errors, new_notification_system,
    notification_by_id, notification_count, notifications_by_severity,
    push_error as push_error_notification, push_info as push_info_notification, push_notification,
    Notification, NotificationSeverity, NotificationSystem,
};

pub mod command_queue;
pub use command_queue::{
    clear_queue, command_count, command_queue_to_json, commands_by_priority, dequeue, drain_all,
    enqueue, enqueue_batch, has_priority, is_queue_empty, max_queue_depth, new_command_queue,
    peek_next, total_enqueued, CommandPriority, CommandQueue, QueuedCommand,
};

pub mod memory_tracker;
pub use memory_tracker::{
    allocation_count, budget_remaining, current_usage, free_count, largest_category,
    memory_tracker_to_json, new_memory_tracker, over_budget, peak_usage, reset_tracker, set_budget,
    track_alloc, track_free, usage_by_category, AllocationRecord, MemoryCategory, MemoryTracker,
};

pub mod clipboard;
pub use clipboard::{
    clear_clipboard, clipboard_content_type, clipboard_has_content, clipboard_history_count,
    clipboard_to_json, copy_color, copy_parameters, copy_pose, copy_text, copy_to_clipboard,
    get_history_entry, new_clipboard, paste_from_clipboard, undo_paste, Clipboard,
    ClipboardContent, ClipboardEntry,
};

pub mod string_pool;
pub use string_pool::{
    clear_pool, contains as pool_contains, find_by_prefix, intern, intern_many, merge_pools,
    new_string_pool, pool_size, pool_stats_json, remove_unused, resolve, string_id_valid,
    total_bytes, StringId, StringPool,
};

pub mod logger;
pub use logger::{
    clear_log as clear_logger_log, entries_by_level, entry_count,
    filter_by_category as logger_filter_by_category, has_errors as logger_has_errors,
    last_n_entries as logger_last_n_entries, log_debug, log_error, log_info, log_message,
    log_trace, log_warn, logger_to_json, new_logger, set_min_level, LogEntry,
    LogLevel as LoggerLogLevel, Logger,
};

pub mod color_space;
pub use color_space::{
    clamp_color, color_distance_lab, color_temperature_to_rgb, hsl_to_rgb, hsv_to_rgb, lab_to_rgb,
    lerp_hsl, lerp_rgb, linear_to_srgb as cs_linear_to_srgb, luminance, rgb_to_hsl, rgb_to_hsv,
    rgb_to_lab, srgb_to_linear as cs_srgb_to_linear, ColorHsl, ColorHsv, ColorLab, ColorRgb,
};

pub mod topic_event_bus;
pub use topic_event_bus::{
    clear_event_bus, dispatch_pending as event_dispatch_pending, drain_topic, event_bus_to_json,
    event_count_total, has_subscribers, last_event_time, new_event_bus, publish as event_publish,
    publish_priority, subscribe as event_subscribe, topic_subscriber_count,
    unsubscribe as event_unsubscribe, EventBusTopic, EventPriority, EventRecord, PendingEvents,
    SubscriberId,
};

pub mod config_manager;
pub use config_manager::{
    active_profile, config_from_pairs, config_to_json, create_profile, delete_profile,
    get_profile_value, get_value_with_fallback, list_profiles, merge_profiles, new_config_manager,
    profile_count, reset_profile_to_defaults, set_profile_value, switch_profile, ConfigManager,
    ConfigProfile, ConfigValue as CfgValue,
};

pub mod arena_str;
pub use arena_str::ArenaStr;

pub mod bit_set;
pub use bit_set::BitSet;

pub mod bloom_counter;
pub use bloom_counter::BloomCounter;

pub mod byte_pool;
pub use byte_pool::BytePool;

pub mod cache_line;
pub use cache_line::CacheLine;

pub mod channel_pair;
pub use channel_pair::ChannelPair;

pub mod clock_source;
pub use clock_source::ClockSource;

pub mod compact_vec;
pub use compact_vec::CompactVec;

pub mod config_val;
pub use config_val::{ConfigStore, ConfigVal as TypedConfigVal};

pub mod counter_map;
pub use counter_map::CounterMap;

pub mod data_table;
pub use data_table::DataTable;

pub mod digest_hash;
pub use digest_hash::DigestHash;

pub mod double_list;
pub use double_list::DoubleList;

pub mod event_sink;
pub use event_sink::{EventRecord as SinkEventRecord, EventSink};

pub mod flag_register;
pub use flag_register::FlagRegister;

pub mod frame_counter;
pub use frame_counter::FrameCounter;

pub mod action_map;
pub use action_map::{ActionEntry, ActionMap};

pub mod async_queue;
pub use async_queue::{AsyncQueue, AsyncTask, TaskState};

pub mod batch_processor;
pub use batch_processor::BatchProcessor;

pub mod bloom_set;
pub use bloom_set::BloomSet;

pub mod buffer_slice;
pub use buffer_slice::BufferSlice;

pub mod cache_entry;
pub use cache_entry::{CacheEntryItem, CacheStore};

pub mod chain_map;
pub use chain_map::ChainMap;

pub mod checkpoint_store;
pub use checkpoint_store::{Checkpoint, CheckpointStore};

pub mod collection_ops;
pub use collection_ops::{
    chunk_vec, dedup_sorted, flatten_nested, interleave, max_f32, mean_f32, min_f32, partition_by,
    sliding_window_avg, sum_f32, unique_sorted, zip_with,
};

pub mod compact_hash;
pub use compact_hash::CompactHash;

pub mod config_reader;
pub use config_reader::ConfigReader;

pub mod cursor_writer;
pub use cursor_writer::CursorWriter;

pub mod decay_counter;
pub use decay_counter::DecayCounter;

pub mod diff_tracker;
pub use diff_tracker::{DiffEntry, DiffTracker};

pub mod dispatch_table;
pub use dispatch_table::{DispatchTable, HandlerEntry};

pub mod double_map;
pub use double_map::DoubleMap;

pub mod access_map;
pub use access_map::AccessMap;

pub mod array_stack;
pub use array_stack::ArrayStack;

pub mod batch_queue;
pub use batch_queue::BatchQueue;

pub mod bitmap_index;
pub use bitmap_index::BitmapIndex;

pub mod buffer_pool;
pub use buffer_pool::BufferPool;

pub mod cache_policy;
pub use cache_policy::{CachePolicy, PolicyEntry, PolicyKind};

pub mod chain_buffer;
pub use chain_buffer::ChainBuffer;

pub mod channel_router;
pub use channel_router::{ChannelRouter, RoutedMessage};

pub mod circular_buffer;
pub use circular_buffer::CircularBuffer;

pub mod color_util;
pub use color_util::{
    clamp01, hue_rotate, is_valid_component, lerp_rgba, linear_to_srgb as cu_linear_to_srgb,
    luminance_srgb, rgb_to_u32, srgb_to_linear as cu_srgb_to_linear, u32_to_rgb,
};

pub mod command_list;
pub use command_list::{CmdEntry, CmdPriority, CommandList};

pub mod compact_set;
pub use compact_set::CompactSet;

pub mod config_layer;
pub use config_layer::{ConfigLayer, LayeredConfig};

pub mod context_map;
pub use context_map::{ContextMap, CtxValue};

pub mod copy_buffer;
pub use copy_buffer::CopyBuffer;

pub mod crc_table;
pub use crc_table::{crc32, crc32_match, CrcTable};

pub mod error_log;
pub use error_log::{
    clear_error_log, count_by_severity, entries_for_category, error_entry_count, error_log_to_json,
    has_fatal, last_error, new_error_log, push_error, total_pushed, ErrorEntry, ErrorLog,
    ErrorSeverity,
};

pub mod event_dispatch;
pub use event_dispatch::{
    clear_all_handlers, clear_handlers, dispatch as dispatch_event,
    dispatch_count as dispatch_total_count, handler_count as dispatch_handler_count, handler_names,
    new_dispatcher, register_handler as dispatch_register_handler, registered_event_types,
    unregister_handler as dispatch_unregister_handler, DispatchRecord, EventDispatcher, HandlerId,
};

pub mod fixed_array;
pub use fixed_array::FixedArray;

pub mod free_slot;
pub use free_slot::{
    alloc as fs_alloc, free as fs_free, get as fs_get, get_mut as fs_get_mut, is_occupied,
    iter_occupied, new_free_slot, slot_capacity, slot_count, FreeSlot,
};

pub mod hash_bucket;
pub use hash_bucket::{
    hb_bucket_count, hb_clear, hb_contains, hb_count, hb_get, hb_insert, hb_keys, hb_load_factor,
    hb_remove, new_hash_bucket, BucketEntry, HashBucket,
};

pub mod id_pool;
pub use id_pool::{
    id_active_count, id_alloc, id_is_active, id_peek_next, id_recycled_count, id_release,
    id_release_all, id_total, new_id_pool, Id, IdPool,
};

pub mod index_list;
pub use index_list::{
    il_as_slice, il_clear, il_contains, il_get, il_is_empty, il_len, il_merge, il_push, il_remove,
    il_retain, new_index_list, IndexList,
};

pub mod interval_tree;
pub use interval_tree::{
    it_clear, it_contains_id, it_count, it_insert, it_query_point, it_query_range, it_remove,
    it_to_json, new_interval_tree, Interval, IntervalTree,
};

pub mod key_cache;
pub use key_cache::{
    kc_advance, kc_clear, kc_contains, kc_frame, kc_get, kc_hits, kc_insert, kc_len, kc_remove,
    new_key_cache, KeyCache, KeyCacheEntry,
};

pub mod lazy_map;
pub use lazy_map::{
    lm_clear, lm_compute_count, lm_declare, lm_get, lm_is_computed, lm_is_pending, lm_len,
    lm_pending_count, lm_pending_keys, lm_remove, lm_set, new_lazy_map, LazyMap,
};

pub mod linked_map;
pub use linked_map::{
    lmap_clear, lmap_contains, lmap_get, lmap_get_at, lmap_get_mut, lmap_insert, lmap_is_empty,
    lmap_keys, lmap_len, lmap_remove, lmap_values, new_linked_map, LinkedMap,
};

pub mod memo_table;
pub use memo_table::{
    memo_access_count, memo_clear, memo_contains, memo_get, memo_hit_rate, memo_hits,
    memo_invalidate, memo_len, memo_misses, memo_set, new_memo_table, MemoEntry, MemoTable,
};

pub mod message_log;
pub use message_log::{
    ml_by_priority, ml_by_tag, ml_clear, ml_get, ml_last, ml_len, ml_push, ml_remove_tag,
    ml_to_json, new_message_log, Message, MessageLog, MsgPriority,
};

pub mod metric_counter;
pub use metric_counter::{
    mc_count, mc_increment, mc_mean, mc_names, mc_record, mc_reset_all, mc_reset_one, mc_stats,
    mc_sum, mc_to_json, new_metric_counter, MetricCounter, MetricStats,
};

pub mod name_table;
pub use name_table::{
    new_name_table, nt_clear, nt_count, nt_has_id, nt_has_name, nt_id, nt_name, nt_names,
    nt_register, nt_rename, nt_unregister, NameTable,
};

pub mod node_pool;
pub use node_pool::{
    new_node_pool, np_alloc, np_capacity, np_count, np_free, np_get, np_get_mut, np_is_valid,
    NodeHandle, NodePool,
};

pub mod object_registry;
pub use object_registry::{
    new_object_registry, or_by_type, or_clear, or_contains, or_get, or_is_empty, or_len,
    or_register, or_remove, or_to_json, ObjectRegistry,
};

pub mod observer_list;
pub use observer_list::{
    new_observer_list, ol_clear, ol_count, ol_has_label, ol_is_empty, ol_notify, ol_notify_count,
    ol_subscribe, ol_unsubscribe, ObserverList,
};

pub mod option_cache;
pub use option_cache::{
    new_option_cache, oc_clear, oc_get, oc_has_key, oc_hit_rate, oc_is_empty, oc_len, oc_remove,
    oc_set_none, oc_set_some, OptionCache,
};

pub mod output_buffer;
pub use output_buffer::{
    new_output_buffer, ob_clear, ob_flush, ob_flush_count, ob_is_empty, ob_len, ob_peek,
    ob_write_bytes, ob_write_str, ob_write_u8, OutputBuffer,
};

pub mod page_allocator;
pub use page_allocator::{
    new_page_allocator, pa_alloc, pa_allocated_count, pa_free, pa_free_count, pa_page_count,
    pa_read, pa_reset, pa_total_bytes, pa_write, PageAllocator,
};

pub mod param_set;
pub use param_set::{
    new_param_set, ps_clear, ps_contains, ps_get_bool, ps_get_float, ps_get_int, ps_get_text,
    ps_is_empty, ps_len, ps_remove, ps_set_bool, ps_set_float, ps_set_int, ps_set_text, ParamSet,
    ParamValue,
};

pub mod patch_buffer;
pub use patch_buffer::{
    new_patch_buffer, pb_add, pb_applied_count, pb_apply, pb_clear, pb_count, pb_is_empty,
    pb_max_offset, pb_total_bytes, Patch, PatchBuffer,
};

pub mod path_cache;
pub use path_cache::{
    new_path_cache, pc_clear, pc_contains, pc_get, pc_hit_rate, pc_insert, pc_invalidate,
    pc_is_empty, pc_len, PathCache,
};

pub mod pattern_match;
pub use pattern_match::{
    count_occurrences, extract_between, glob_match, glob_match_ci, grep_lines, has_prefix,
    has_suffix, replace_all, tokenize, PatternMatcher,
};

pub mod payload_buffer;
pub use payload_buffer::{
    new_payload_buffer, pybuf_clear, pybuf_drain, pybuf_is_empty, pybuf_is_full, pybuf_len,
    pybuf_peek, pybuf_pop, pybuf_push, pybuf_total_bytes, PayloadBuffer, PayloadEntry,
};

pub mod peg_parser;
pub use peg_parser::{
    node_text, parse_choice, parse_depth, parse_ident, parse_integer, parse_list, parse_literal,
    parse_opt, skip_whitespace, ParseNode,
};

pub mod persistent_map;
pub use persistent_map::{
    new_persistent_map, pm_clear, pm_contains, pm_get, pm_insert, pm_is_empty, pm_len, pm_remove,
    pm_restore, pm_snapshot, pm_snapshot_count, pm_version, PersistentMap,
};

pub mod pipe_filter;
pub use pipe_filter::{
    new_pipe_filter, pf_add, pf_apply, pf_clear, pf_count, pf_is_empty, pf_passed, pf_passes,
    pf_rejected, FilterOp, PipeFilter,
};

pub mod pipeline_context;
pub use pipeline_context::{
    ctx_add_error, ctx_add_warning, ctx_advance, ctx_get, ctx_has, ctx_has_errors, ctx_is_done,
    ctx_mark_done, ctx_reset, ctx_set, ctx_stage, new_pipeline_context, PipelineContext,
};

pub mod placeholder_map;
pub use placeholder_map::{
    new_placeholder_map, plm_clear, plm_contains, plm_get, plm_is_empty, plm_len, plm_remove,
    plm_render, plm_set, plm_substituted, PlaceholderMap,
};

pub mod plan_executor;
pub use plan_executor::{
    new_plan_executor, pe_add_step, pe_complete, pe_done_count, pe_fail, pe_failed_count,
    pe_is_aborted, pe_is_complete, pe_reset, pe_skip, pe_step_count, pe_total_ms, PlanExecutor,
    PlanStep, StepState,
};

pub mod pool_allocator;
pub use pool_allocator::{new_pool, PoolAllocator, PoolHandle, PoolSlot};

pub mod priority_map;
pub use priority_map::{
    clear_priority_map, get_highest, has_key_pm, insert_priority, new_priority_map, priority_count,
    priority_to_vec, remove_highest, PriorityMap,
};

pub mod proc_context;
pub use proc_context::{new_proc_context, CtxVal as ProcCtxVal, ProcContext};

pub mod query_cache;
pub use query_cache::{new_query_cache, QueryCache, QueryEntry};

pub mod radix_sort;
pub use radix_sort::{
    count_distinct_u32, is_sorted_u32, is_sorted_u64, radix_sort_pairs_u32, radix_sort_u32,
    radix_sort_u64,
};

pub mod range_map;
pub use range_map::{new_range_map, RangeEntry, RangeMap};

pub mod ref_counted;
pub use ref_counted::{new_ref_counted, RefCounted, RefEntry};

pub mod registry_map;
pub use registry_map::{new_registry_map, RegistryItem, RegistryMap};

pub mod resource_pool;
pub use resource_pool::{
    new_resource_pool, ResourcePool, ResourceSlot, ResourceState as PoolResourceState,
};

pub mod result_stack;
pub use result_stack::{new_result_stack, ResultEntry, ResultKind, ResultStack};

pub mod retry_policy;
pub use retry_policy::{
    max_retries, new_retry_policy, reset_retry, retry_count, retry_delay_ms, retry_exhausted,
    retry_with_backoff, should_retry, RetryPolicy, RetryResult,
};

pub mod ring_log;
pub use ring_log::{new_ring_log, RingLog, RingLogEntry, RingLogLevel};

pub mod role_map;
pub use role_map::{new_role_map, RoleMap};

pub mod route_table;
pub use route_table::{new_route_table, RouteEntry as RouteTableEntry, RouteMatch, RouteTable};

pub mod rule_engine;
pub use rule_engine::{make_rule, new_rule_engine, Condition, Rule, RuleAction, RuleEngine};

pub mod schedule_queue;
pub use schedule_queue::{new_schedule_queue, ScheduleQueue, ScheduleTask};

pub mod search_index;
pub use search_index::{new_search_index, SearchDoc, SearchIndex};

pub mod segment_tree;
pub use segment_tree::{
    build_segment_tree, seg_get, seg_query, seg_total, seg_update, SegmentTree,
};

pub mod selector_map;
pub use selector_map::{new_selector_map, SelectorMap};

pub mod semaphore_pool;
pub use semaphore_pool::{new_semaphore_pool, Semaphore, SemaphorePool};

pub mod sequence_map;
pub use sequence_map::{new_sequence_map, SeqEntry, SequenceMap};

pub mod service_locator;
pub use service_locator::{new_service_locator, ServiceDescriptor, ServiceLocator};

pub mod session_store;
pub use session_store::{new_session_store, Session, SessionStore};

pub mod set_trie;
pub use set_trie::{new_set_trie, SetTrie};

pub mod signal_handler;
pub use signal_handler::{new_signal_handler, HandlerEntry as SignalHandlerEntry, SignalHandler};

pub mod simple_graph;
pub use simple_graph::{new_simple_graph, GraphEdge, SimpleGraph};

pub mod size_cache;
pub use size_cache::{new_size_cache, SizeCache, SizeCacheEntry};

pub mod skip_list;
pub use skip_list::{
    new_skip_list, skip_find, skip_insert, skip_len, skip_range, skip_remove, SkipEntry, SkipList,
};

pub mod sliding_window;
pub use sliding_window::{new_sliding_window, SlidingWindow};

pub mod sort_key;
pub use sort_key::{new_sort_key, SortCriterion, SortDir, SortKey};

pub mod source_map;
pub use source_map::{new_source_map, SourceMap, SourceMapping};

pub mod span_tracker;
pub use span_tracker::{new_span_tracker, SpanRecord, SpanTracker};

pub mod sparse_array;
pub use sparse_array::{
    new_sparse_array, sparse_clear, sparse_count, sparse_get, sparse_has, sparse_keys,
    sparse_remove, sparse_set_val, SparseArray,
};

pub mod state_bag;
pub use state_bag::{
    new_state_bag, sb_clear, sb_get, sb_len, sb_remove, sb_set, BagValue, StateBag,
};

pub mod state_machine_v2;
pub use state_machine_v2::{
    new_state_machine, sm_add_state, sm_add_transition, sm_current, sm_fire,
    GuardFn as StateMachineGuardFn, StateMachineV2, Transition as StateMachineTransition,
};

pub mod static_vec;
pub use static_vec::{new_static_vec, StaticVec};

pub mod storage_backend;
pub use storage_backend::{
    new_storage_backend, sb_contains as storage_contains, sb_get as storage_get, sb_put,
    sb_remove as storage_remove, Bucket, StorageBackend,
};

pub mod stream_parser;
pub use stream_parser::{
    new_stream_parser, sp_feed, sp_read_u32_le, sp_read_u8, ParseResult, StreamParser,
};

pub mod string_set;
pub use string_set::{
    new_string_set, ss_contains, ss_insert, ss_len, ss_remove, ss_to_vec, StringSet,
};

pub mod struct_map;
pub use struct_map::{new_struct_map, stm_contains, stm_get, stm_set, FieldVal, StructMap};

pub mod sub_task;
pub use sub_task::{
    new_sub_task_set, sts_add, sts_done, sts_failed, sts_overall, SubTask, SubTaskSet,
    SubTaskStatus,
};

pub mod symbol_table;
pub use symbol_table::{
    new_symbol_table, sym_find, sym_intern, sym_len, sym_lookup, SymbolId, SymbolTable,
};

pub mod sync_barrier;
pub use sync_barrier::{
    barrier_arrive, barrier_is_released, barrier_register, barrier_reset, new_sync_barrier,
    BarrierState, SyncBarrier,
};

pub mod tag_filter;
pub use tag_filter::{new_tag_filter, tf_exclude, tf_matches, tf_require, TagFilter};

pub mod text_buffer;
pub use text_buffer::{
    new_text_buffer, tb_append, tb_append_line, tb_as_str, tb_clear, tb_find, tb_line_count,
    TextBuffer,
};

pub mod thread_local_pool;
pub use thread_local_pool::{new_thread_local_pool, ThreadLocalPool};

pub mod time_source;
pub use time_source::{
    current_time_ms, elapsed_since, new_time_source, time_diff_ms, time_source_reset,
    timestamp_add_ms, timestamp_is_after, timestamp_to_string, TimeSource, Timestamp,
};

pub mod token_stream;
pub use token_stream::{
    new_token_stream, tks_drain, tks_is_empty, tks_next, tks_peek, tks_push, tks_remaining,
    tks_rewind, tks_skip_while, tks_total, Token, TokenKind, TokenStream,
};

pub mod topo_map;
pub use topo_map::{
    new_topo_map, tm_add_edge, tm_add_node, tm_clear, tm_has_node, tm_label, tm_node_count,
    tm_remove_node, tm_topo_sort, TopoMap, TopoNode,
};

pub mod trace_buffer;
pub use trace_buffer::{
    new_trace_buffer, tb_avg_duration_us, tb_by_tag, tb_clear as trace_clear, tb_get as trace_get,
    tb_is_empty as trace_is_empty, tb_len as trace_len, tb_max_event, tb_record as trace_record,
    tb_tick as trace_tick, TraceBuffer, TraceEvent,
};

pub mod transform_pipe;
pub use transform_pipe::{
    new_transform_pipe, tp_add, tp_apply, tp_clear, tp_get, tp_is_empty, tp_len, tp_pop,
    TransformKind, TransformPipe, TransformStage,
};

pub mod tree_index;
pub use tree_index::{
    new_tree_index, ti_add_child, ti_add_root, ti_children, ti_count, ti_depth, ti_descendants,
    ti_label, ti_parent, ti_roots, TreeIndex, TreeNode,
};

pub mod trie_map;
pub use trie_map::{
    new_trie_map, trm_contains, trm_get, trm_insert, trm_is_empty, trm_keys_with_prefix, trm_len,
    trm_remove, TrieMap,
};

pub mod type_alias_map;
pub use type_alias_map::{
    new_type_alias_map, tam_aliases_for, tam_all_aliases, tam_clear, tam_count, tam_is_alias,
    tam_register, tam_remove, tam_resolve, tam_resolve_chain, TypeAliasMap,
};

pub mod type_cache;
pub use type_cache::{
    new_type_cache, tc_clear, tc_contains, tc_get, tc_is_empty, tc_len, tc_remove, tc_store,
    tc_total_bytes, tc_version, TypeCache, TypeCacheEntry,
};

pub mod type_erased;
pub use type_erased::{
    new_type_erased, te_clear, te_contains, te_get, te_insert, te_is_empty, te_keys,
    te_keys_by_type, te_len, te_remove, te_type_tag, ErasedSlot, TypeErased,
};

pub mod uid_gen;
pub use uid_gen::{
    new_uid_gen, ug_alloc, ug_allocated_count, ug_local_id, ug_namespace, ug_peek_next, ug_recycle,
    ug_recycled_count, ug_reset, UidGen,
};

pub mod union_find_v2;
pub use union_find_v2::{
    new_union_find, uf_component_count, uf_component_size, uf_connected, uf_element_count, uf_find,
    uf_reset, uf_union, UnionFindV2,
};

pub mod update_queue;
pub use update_queue::{
    new_update_queue, uq_clear, uq_dequeue, uq_drain_all, uq_enqueue, uq_is_empty, uq_len, uq_peek,
    uq_total_enqueued, UpdateItem, UpdateQueue,
};

pub mod value_cache;
pub use value_cache::{
    new_value_cache, vc_clear, vc_dirty_count, vc_get, vc_invalidate, vc_invalidate_all,
    vc_is_valid, vc_len, vc_remove, vc_store, ValueCache, ValueEntry,
};

pub mod value_map;
pub use value_map::{
    new_value_map, vm_clear, vm_contains, vm_get, vm_get_bool, vm_get_float, vm_get_int, vm_len,
    vm_remove, vm_set_bool, vm_set_float, vm_set_int, vm_set_text, MapVal, ValueMap,
};

pub mod var_store;
pub use var_store::{
    new_var_store, vs_changed_names, vs_clear, vs_declare, vs_flush, vs_get, vs_is_changed, vs_len,
    vs_remove, vs_reset, vs_set, VarEntry, VarStore,
};

pub mod quad_tree;
pub use quad_tree::{
    new_quad_tree, qt_clear, qt_count, qt_insert, qt_is_empty, qt_query_circle, qt_query_rect,
    Aabb2, QtPoint, QuadTree,
};

pub mod color_palette;
pub use color_palette::{
    add_color, blend_palette_colors, get_color, new_color_palette, palette_size, ColorEntry,
    ColorPalette,
};

pub mod bitmask_ops;
pub use bitmask_ops::{
    clear_bit, count_leading_zeros, count_trailing_zeros, extract_range, highest_bit, is_bit_set,
    lowest_bit, parity, popcount, range_mask, rotate_left, rotate_right, set_bit, set_bit_indices,
    toggle_bit,
};

pub mod version_vector;
pub use version_vector::{
    new_version_vector, vv_compare, vv_concurrent, vv_get, vv_happens_before, vv_increment,
    vv_merge, vv_node_count, vv_nodes, vv_reset_node, VersionVector,
};

pub mod text_tokenizer;
pub use text_tokenizer::{
    is_numeric_token, token_count, token_numbers, token_words, tokenize as text_tokenize,
    Token as TextToken,
};

pub mod rolling_stats;
pub use rolling_stats::{
    new_rolling_stats, rs_clear, rs_count, rs_is_full, rs_max, rs_mean, rs_median, rs_min, rs_push,
    rs_std, rs_sum, rs_variance, RollingStats,
};

pub mod bloom_filter;
pub use bloom_filter::{
    bloom_contains, bloom_fill_ratio, bloom_insert, bloom_reset, new_bloom_filter, BloomFilter,
};

pub mod escape_hatch;
pub use escape_hatch::{
    html_escape, html_unescape, json_string_escape, json_string_unescape, url_escape,
};

pub mod number_format;
pub use number_format::{
    format_bytes, format_float, format_float_sep, format_int_sep, format_percent, format_si,
    pad_left, pad_right,
};

pub mod delta_encoder_v2;
pub use delta_encoder_v2::{
    avg_delta, delta_decode, delta_decode_u32, delta_encode, delta_encode_u32, max_delta,
    zigzag_decode, zigzag_encode,
};

pub mod run_length;
pub use run_length::{
    rle_compression_ratio_v2, rle_decode, rle_decoded_len, rle_encode, rle_is_uniform, rle_merge,
    rle_most_frequent, rle_run_count, rle_verify_roundtrip, Run,
};

pub mod checksum_crc;
pub use checksum_crc::{
    additive_checksum, crc16, crc16_verify, crc8, crc8_verify, fletcher16, xor_checksum,
};

pub mod argument_parser;
pub use argument_parser::{
    arg_get, arg_get_f64, arg_get_i64, arg_get_or, arg_has_flag, arg_keys, arg_positional_count,
    new_parsed_args, parse_args, parse_args_str, ParsedArgs,
};

pub mod pipeline_stage;
pub use pipeline_stage::{
    execute_stage, new_pipeline_stage, stage_dependencies, stage_is_complete, stage_name_ps,
    stage_reset, stage_result, stage_to_json, PipelineStage as PsStage,
    StageResult as PsStageResult,
};

pub mod fsm_builder;
pub use fsm_builder::{
    add_state, add_transition, build_fsm, has_state, has_transition, new_fsm_builder, state_count,
    transition_count, BuiltFsm, FsmBuilder, FsmTransition,
};

pub mod decision_tree;
pub use decision_tree::{
    dt_add_branch, dt_add_leaf, dt_all_actions, dt_branch_count, dt_clear, dt_evaluate,
    dt_leaf_action, dt_leaf_count, dt_node_count, new_decision_tree, DecisionNode, DecisionTree,
};

pub mod graph_coloring;
pub use graph_coloring::{
    cg_add_edge, cg_degree, cg_edge_count, cg_max_degree, cg_vertex_count, coloring_is_valid,
    coloring_num_colors, greedy_color, new_color_graph, vertices_with_color, ColorGraph,
};

pub mod bezier_path;
pub use bezier_path::{
    bezier_eval, bezier_length, bezier_split, new_bezier_path, new_cubic_bezier, path_add_segment,
    path_clear, path_eval, path_get_segment, path_length, path_segment_count, BezierPath,
    CubicBezier,
};

pub mod poly_clip;
pub use poly_clip::{
    new_polygon, point_in_polygon, polygon_area, polygon_centroid, polygon_is_empty,
    polygon_signed_area, polygon_vertex_count, sutherland_hodgman, Polygon2D,
};

pub mod convex_hull_2d;
pub use convex_hull_2d::{
    convex_hull_2d, hull_area, hull_centroid, hull_contains_point, hull_diameter, hull_perimeter,
};

pub mod matrix2;
pub use matrix2::{
    mat2, mat2_add, mat2_approx_eq, mat2_det, mat2_identity, mat2_inverse, mat2_mul, mat2_mul_vec,
    mat2_rotation, mat2_scale, mat2_trace, mat2_transpose, mat2_zero, Mat2,
};

pub mod matrix3;
pub use matrix3::{
    mat3, mat3_add, mat3_approx_eq, mat3_det, mat3_identity, mat3_inverse, mat3_mul, mat3_mul_vec,
    mat3_outer, mat3_rot_z, mat3_scale, mat3_trace, mat3_transpose, mat3_zero, Mat3,
};

pub mod quaternion_ops;
pub use quaternion_ops::{
    quat, quat_approx_eq, quat_conjugate, quat_dot, quat_from_axis_angle, quat_identity,
    quat_inverse, quat_mul, quat_norm, quat_normalize, quat_rotate_vec, quat_slerp, Quat,
};

pub mod dual_quaternion;
pub use dual_quaternion::{
    dq_approx_eq, dq_conjugate, dq_dot, dq_from_rot_trans, dq_get_rotation, dq_get_translation,
    dq_identity, dq_mul, dq_normalize, dq_transform_point, dual_quat, DualQuat,
};

pub mod packed_vec3;
pub use packed_vec3::{
    decode_u16_to_f32, default_packed_config, encode_f32_to_u16, new_packed_buffer, pack_vec3,
    packed_config, pvbuf_bytes, pvbuf_clear, pvbuf_get, pvbuf_is_empty, pvbuf_len, pvbuf_push,
    unpack_vec3, PackedVec3, PackedVec3Buffer, PackedVec3Config,
};

pub mod packed_color;
pub use packed_color::{
    color_a, color_b, color_black, color_blend, color_g, color_lerp, color_premul_alpha, color_r,
    color_to_grayscale, color_transparent, color_white, decode_rgba_f32, decode_rgba_u8, rgba_f32,
    rgba_u8, PackedColor,
};

pub mod bsp_tree_2d;
pub use bsp_tree_2d::{
    bsp_build, bsp_collect_polygons, bsp_depth, bsp_get_root, bsp_is_leaf, bsp_line_side,
    bsp_polygon_count, bsp_set_root, bsp_split_polygon, new_bsp_tree, BspLine, BspNode, BspPolygon,
    BspTree2D,
};

pub mod hex_grid;
pub use hex_grid::{cube_round, hex_disk, hex_ring, pixel_to_hex_flat, HexCoord};

pub mod triangular_grid;
pub use triangular_grid::{TriGrid, Triangle2D};

pub mod noise_perlin;
pub use noise_perlin::{perlin2, perlin2_01, perlin3};

pub mod noise_simplex;
pub use noise_simplex::{simplex2, simplex2_01, simplex2_scaled};

pub mod noise_worley;
pub use noise_worley::{worley2, worley2_01, worley2_f1f2, worley2_ridged};

pub mod fractal_noise;
pub use fractal_noise::{
    fbm_max_amplitude, fbm_perlin2, fbm_simplex2, ridged_fbm_perlin2, turbulence_perlin2, FbmConfig,
};

pub mod easing_curves;
pub use easing_curves::{
    ease_by_name, ease_in_back, ease_in_bounce, ease_in_cubic, ease_in_elastic, ease_in_expo,
    ease_in_out_cubic, ease_linear, ease_out_back, ease_out_bounce, ease_out_cubic,
    ease_out_elastic, ease_out_expo,
};

pub mod animation_curve;
pub use animation_curve::{AnimCurve, InterpMode as CurveInterpMode, Keyframe};

pub mod spline_catmull;
pub use spline_catmull::{catmull_rom, catmull_rom2, catmull_rom3, CatmullRomSpline2D};

pub mod spline_hermite;
pub use spline_hermite::{
    hermite, hermite2, hermite3, hermite_basis, hermite_deriv, HermiteSpline2D,
};

pub mod color_convert;
pub use color_convert::{
    hsv_to_rgb as cc_hsv_to_rgb, linear_to_srgb as cc_linear_to_srgb, luma_bt709,
    rgb_to_hsv as cc_rgb_to_hsv, rgb_to_oklch_approx, srgb_to_linear as cc_srgb_to_linear,
};

pub mod color_gradient;
pub use color_gradient::{grayscale_gradient, rainbow_gradient, ColorGradient, ColorStop};

pub mod spatial_hash_2d;
pub use spatial_hash_2d::SpatialHash2D;

pub mod bit_matrix;
pub use bit_matrix::BitMatrix;

pub mod convex_hull_3d;
pub use convex_hull_3d::{convex_hull_3d, hull_face_count, hull_volume, ConvexHull3D, HullFace};

pub mod delaunay_2d;
pub use delaunay_2d::{
    delaunay_2d, delaunay_tri_count, delaunay_valid, DelaunayResult, DelaunayTri,
};

pub mod voronoi_2d;
pub use voronoi_2d::{
    build_voronoi, cell_centroid, voronoi_assign, Point2, Voronoi2D, VoronoiCell2D,
};

pub mod kd_tree_2d;
pub use kd_tree_2d::{kd2_build, kd2_nn_dist_sq, KdPoint2, KdTree2D};

pub mod kd_tree_3d;
pub use kd_tree_3d::{kd3_build, kd3_nearest_id, KdPoint3, KdTree3D};

pub mod r_tree;
pub use r_tree::{rtree_entry, RTree2D, RTreeEntry, Rect2};

pub mod aabb_tree_2d;
pub use aabb_tree_2d::{aabb2d_entry, Aabb2D, AabbEntry, AabbTree2D};

pub mod segment_tree_v2;
pub use segment_tree_v2::{seg2_max, seg2_min, seg2_sum, SegOp, SegTreeV2};

pub mod fenwick_tree;
pub use fenwick_tree::{fenwick_prefix, fenwick_range, FenwickTree};

pub mod suffix_array;
pub use suffix_array::{
    build_lcp_array, build_suffix_array, lcp_max, sa_contains, sa_find_all, sa_suffix_count,
};

pub mod rope_ds;
pub use rope_ds::{rope_concat, rope_from, Rope};

pub mod trie_v2;
pub use trie_v2::TrieV2;

pub mod skip_list_v2;
pub use skip_list_v2::SkipList2;

pub mod splay_tree;
pub use splay_tree::SplayTree;

pub mod avl_tree;
pub use avl_tree::AvlTree;

pub mod red_black_tree;
pub use red_black_tree::RedBlackTree;

pub mod b_tree;
pub use b_tree::BTree;

pub mod hash_map_open;
pub use hash_map_open::OpenHashMap;

pub mod bloom_filter_v3;
pub use bloom_filter_v3::BloomFilterV3;

pub mod count_min_v2;
pub use count_min_v2::CountMinSketchV2;

pub mod hyperloglog_v2;
pub use hyperloglog_v2::HyperLogLogV2;

pub mod cuckoo_filter;
pub use cuckoo_filter::CuckooFilter;

pub mod skip_list_v3;
pub use skip_list_v3::SkipListV3;

pub mod b_tree_v2;
pub use b_tree_v2::BTreeMapV2;

pub mod red_black_map;
pub use red_black_map::RedBlackMap;

pub mod avl_map;
pub use avl_map::AvlMap;

pub mod splay_map;
pub use splay_map::SplayMap;

pub mod treap_map;
pub use treap_map::TreapMap;

pub mod fibonacci_heap;
pub use fibonacci_heap::FibonacciHeap;

pub mod pairing_heap;
pub use pairing_heap::PairingHeap;

pub mod leftist_heap;
pub use leftist_heap::LeftistHeap;

pub mod binomial_heap;
pub use binomial_heap::BinomialHeap;

pub mod d_ary_heap;
pub use d_ary_heap::DAryHeap;

pub mod interval_query;
pub use interval_query::{Interval as QueryInterval, IntervalQueryTree};

pub mod topological_sort;
pub use topological_sort::{
    new_topo_graph as new_topo_sort_graph, topo_add_edge, topo_add_node, topo_clear,
    topo_edge_count, topo_has_cycle, topo_has_cycle_dag, topo_layer_count, topo_node_count,
    topo_remove_node, topo_sort, topo_sort_dag, topo_sources, TopoGraph as TopoSortGraph,
    TopoResult,
};

pub mod strongly_connected;
pub use strongly_connected::{
    is_strongly_connected, largest_scc, new_scc_graph, scc_add_edge, scc_count, tarjan_scc,
    SccGraph,
};

pub mod shortest_path_bfs;
pub use shortest_path_bfs::{
    bfs_add_edge, bfs_add_undirected, bfs_distance, bfs_distances, bfs_reachable,
    bfs_shortest_path, new_bfs_graph, BfsGraph,
};

pub mod bellman_ford;
pub use bellman_ford::{
    bellman_ford, bf_add_edge, bf_distance, bf_edge_count, bf_has_negative_cycle, new_bf_graph,
    BfEdge, BfGraph, BfResult,
};

pub mod floyd_warshall;
pub use floyd_warshall::{
    floyd_warshall, fw_add_edge, fw_distance, fw_has_negative_cycle, fw_solve, new_fw_result,
    FwResult,
};

pub mod prim_mst;
pub use prim_mst::{
    new_prim_graph, prim_add_edge, prim_edge_count, prim_mst, prim_mst_weight, prim_node_count,
    MstEdge, PrimGraph,
};

pub mod kruskal_mst;
pub use kruskal_mst::{
    kruskal_edges_from, kruskal_is_spanning, kruskal_mst, kruskal_mst_weight,
    new_union_find as new_kruskal_union_find, KruskalEdge, UnionFind,
};

pub mod max_flow_ff;
pub use max_flow_ff::{
    fg_add_edge, fg_has_augmenting_path, fg_node_count as flow_node_count, fg_total_capacity_from,
    max_flow, new_flow_graph, FlowGraph,
};

pub mod bipartite_match;
pub use bipartite_match::{
    bip_add_edge, bip_edge_count, bipartite_matching, has_perfect_matching, max_matching_size,
    new_bipartite, BipartiteGraph,
};

pub mod string_hash;
pub use string_hash::{
    compute_string_hash, hash_combine_strings, hash_empty_string, hash_to_hex_sh, string_hash_seed,
    string_hash_u32, string_hash_u64, string_hashes_equal, StringHash,
};

pub mod aho_corasick;
pub use aho_corasick::{
    ac_add_pattern, ac_build, ac_contains, ac_pattern_count, ac_search, new_aho_corasick, AcMatch,
    AcNode, AhoCorasick,
};

pub mod suffix_array_v2;
pub use suffix_array_v2::{build_sa_v2, sa2_contains, sa2_is_sorted, sa2_len, sa2_search};

pub mod lcp_array;
pub use lcp_array::{build_lcp, distinct_substrings, lcp_avg, lcp_max_val, lcp_query, lcp_valid};

pub mod z_algorithm;
pub use z_algorithm::{z_contains, z_count, z_function, z_max, z_search, z_valid};

pub mod kmp_search;
pub use kmp_search::{
    kmp_contains, kmp_count, kmp_failure, kmp_failure_len, kmp_max_failure, kmp_search,
};

pub mod graph_articulation;
pub use graph_articulation::{
    artic_add_edge, artic_component_count, find_articulation_points, find_bridges, is_biconnected,
    new_artic_graph, ArticGraph,
};

pub mod edit_script;
pub use edit_script::{
    apply_edit_script, build_edit_script, edit_distance_from_script, script_to_diff_string, EditOp,
    EditScript,
};

pub mod patience_diff;
pub use patience_diff::{
    patience_diff, patience_diff_to_string, unique_common_lines, PatienceDiff, PatienceHunk,
};

pub mod histogram_diff;
pub use histogram_diff::{
    build_histogram, histogram_diff, histogram_diff_to_string, HistogramDiff, HistogramDiffConfig,
    HistogramHunk,
};

pub mod three_way_merge;
pub use three_way_merge::{
    clean_line_count, is_clean_merge, three_way_merge, MergeRegion, MergeResult,
};

pub mod conflict_marker;
pub use conflict_marker::{
    parse_conflict_markers, render_conflict_block, resolve_all, ConflictBlock, ParsedConflicts,
    MARKER_OURS, MARKER_SEP, MARKER_THEIRS,
};

pub mod patch_apply;
pub use patch_apply::{
    apply_patch, can_apply_cleanly, count_overlapping_hunks, parse_unified_diff, HunkLine,
    PatchError, UnifiedHunk, UnifiedPatch,
};

pub mod line_indexer;
pub use line_indexer::{build_line_indexer, line_to_offset, offset_to_position, LineIndexer};

pub mod syntax_highlighter;
pub use syntax_highlighter::{
    classify_token, count_kind as count_highlight_kind, highlight_tokens, to_ansi_string,
    HighlightKind, HighlightToken, HighlighterConfig, Language as HighlightLanguage,
};

pub mod indent_detector;
pub use indent_detector::{
    count_mixed_indent_lines, detect_indent, normalize_to_spaces, normalize_to_tabs, IndentResult,
    IndentStyle,
};

pub mod whitespace_normalizer;
pub use whitespace_normalizer::{
    collapse_blank_lines, detect_issues as detect_whitespace_issues,
    normalize as normalize_whitespace, strip_trailing, trailing_whitespace_count, LineEnding,
    NormalizerConfig, WhitespaceStats,
};

pub mod unicode_segmenter;
pub use unicode_segmenter::{
    grapheme_count, has_multibyte_graphemes, nth_grapheme, reverse_graphemes, segment_graphemes,
    truncate_graphemes, word_wrap_graphemes, Grapheme,
};

pub mod char_classifier;
pub use char_classifier::{
    classify_char, classify_str, default_classifier_config, is_alnum, is_alpha, is_digit,
    is_punctuation, is_whitespace as char_is_whitespace, to_ascii_lower, to_ascii_upper, CharClass,
    ClassifierConfig,
};

pub mod word_boundary;
pub use word_boundary::{
    extract_words, find_word_spans, is_boundary_at, is_word_char, word_boundary_positions,
    word_count, WordBoundaryConfig, WordSpan,
};

pub mod sentence_splitter;
pub use sentence_splitter::{
    avg_words_per_sentence, filter_short_sentences, longest_sentence, sentence_count,
    split_sentences, Sentence, SentenceSplitterConfig,
};

pub mod paragraph_detector;
pub use paragraph_detector::{
    detect_paragraphs, filter_by_min_words, kind_summary, longest_paragraph, paragraph_count,
    Paragraph, ParagraphConfig, ParagraphKind,
};

pub mod lexer_token_stream;
pub use lexer_token_stream::{
    count_tokens_of_kind, lex_string, LexToken, LexTokenKind, LexerStream,
};

pub mod compression_lz4;
pub use compression_lz4::{
    lz4_compress, lz4_compress_bound, lz4_decompress, lz4_is_compressed, lz4_roundtrip_ok,
    Lz4Compressor, Lz4Config,
};

pub mod compression_zstd;
pub use compression_zstd::{
    zstd_compress, zstd_decompress, zstd_frame_size_estimate, zstd_frame_valid, zstd_roundtrip_ok,
    ZstdCompressor, ZstdConfig,
};

pub mod compression_brotli;
pub use compression_brotli::{
    brotli_compress, brotli_decompress, brotli_max_compressed_size, brotli_quality_valid,
    brotli_roundtrip_ok, BrotliCompressor, BrotliConfig,
};

pub mod compression_snappy;
pub use compression_snappy::{
    snappy_compress, snappy_decompress, snappy_max_compressed_length, snappy_roundtrip_ok,
    snappy_validate_compressed_buffer, SnappyCompressor, SnappyConfig,
};

pub mod encryption_aes;
pub use encryption_aes::{
    aes_derive_key_stub, aes_gcm_decrypt, aes_gcm_encrypt, aes_key_len_valid, AesGcmCipher,
    AesGcmConfig, AesKeyLen,
};

pub mod encryption_chacha;
pub use encryption_chacha::{
    chacha_decrypt, chacha_encrypt, chacha_key_from_seed, chacha_nonce_len, chacha_roundtrip_ok,
    ChaChaCipher, ChaChaConfig,
};

pub mod hashing_sha256;
pub use hashing_sha256::{hmac_sha256_stub, sha256_eq, sha256_hash, Sha256Digest, Sha256Hasher};

pub mod hashing_blake3;
pub use hashing_blake3::{
    blake3_hash, blake3_keyed_hash, blake3_output_len, blake3_stable, Blake3Digest, Blake3Hasher,
};

pub mod hashing_xxhash;
pub use hashing_xxhash::{xxhash32, xxhash64, xxhash64_eq, xxhash64_hex, XxHasher};

pub mod base64_codec;
pub use base64_codec::{
    base64_decode, base64_decoded_len, base64_encode, base64_encode_str, base64_encoded_len,
    base64_is_valid, default_base64_config, Base64Config,
};

pub mod base58_codec;
pub use base58_codec::{
    base58_decode, base58_encode, base58_encoded_len_estimate, base58_is_valid, base58_roundtrip_ok,
};

pub mod hex_codec;
pub use hex_codec::{
    hex_decode, hex_encode, hex_encode_upper, hex_is_valid, hex_roundtrip_ok, hex_strip_prefix,
};

pub mod url_encode;
pub use url_encode::{
    url_decode, url_encode as url_encode_str, url_encode_query, url_is_safe, url_roundtrip_ok,
};

pub mod url_parser_stub;

pub mod html_escape;
pub use html_escape::{
    html_escape as html_entity_escape, html_escape_attr, html_needs_escape,
    html_roundtrip_ok as html_entity_roundtrip_ok, html_unescape as html_entity_unescape,
};

pub mod csv_parser;
pub use csv_parser::{
    csv_col_count, csv_field, csv_row_count, parse_csv, parse_csv_line, CsvRecord, CsvTable,
};

pub mod tsv_parser;
pub use tsv_parser::{
    parse_tsv, parse_tsv_line, tsv_col_count, tsv_field, tsv_row_count, tsv_to_string, TsvRecord,
    TsvTable,
};

pub mod json_pointer;
pub use json_pointer::{
    escape_token, pointer_leaf, pointer_parent, unescape_token, JsonPointer, JsonPointerError,
};

pub mod json_patch;
pub use json_patch::{
    count_ops, has_test_ops, parse_op_kind, validate_path, JsonPatch, PatchError as JsonPatchError,
    PatchOp,
};

pub mod json_schema_validator;
pub use json_schema_validator::{
    is_required, required_count, validate_number, validate_string, SchemaNode,
    SchemaType as JsonSchemaType, ValidationError as JsonSchemaValidationError,
};

pub mod toml_parser;
pub use toml_parser::{
    get_integer as toml_get_integer, get_string as toml_get_string, parse_line as parse_toml_line,
    parse_toml, TomlDocument, TomlParseError, TomlValue,
};

pub mod yaml_parser;
pub use yaml_parser::{
    get_int as yaml_get_int, parse_scalar, parse_scalar_line, parse_yaml, YamlDocument, YamlError,
    YamlScalar,
};

pub mod xml_tokenizer;
pub use xml_tokenizer::{
    collect_text, count_end_tags, count_start_tags, is_balanced, XmlError, XmlToken, XmlTokenizer,
};

pub mod protobuf_varint;
pub use protobuf_varint::{
    decode_varint, decode_zigzag, encode_varint, encode_zigzag, varint_roundtrip_ok, varint_size,
    VarintError,
};

pub mod message_pack_codec;
pub use message_pack_codec::{
    array_len, buffers_equal, encode as msgpack_encode, encoded_size as msgpack_encoded_size,
    is_nil, MsgError, MsgValue,
};

pub mod cbor_codec;
pub use cbor_codec::{
    cbor_array_len, cbor_encoded_len, cbor_is_null, encode_cbor, major_of, CborError, CborMajor,
    CborValue,
};

pub mod avro_codec;
pub use avro_codec::{
    decode_long, encode_bytes as avro_encode_bytes, encode_long, is_union, record_field_count,
    type_name as avro_type_name, AvroError, AvroField, AvroType, AvroValue,
};

pub mod flatbuffer_stub;
pub use flatbuffer_stub::{padded_size, read_u32 as flatbuf_read_u32, FlatBuilder, FlatError};

pub mod capnproto_stub;
pub use capnproto_stub::{
    message_is_empty, serialize_message, traversal_limit_words, CapnMessage, CapnSegment,
};

pub mod thrift_codec;
pub use thrift_codec::{
    decode_i32, encode_field_header, encode_i32, encode_string as thrift_encode_string,
    is_struct as thrift_is_struct, struct_field_count, type_of as thrift_type_of, ThriftError,
    ThriftField, ThriftType, ThriftValue,
};

pub mod grpc_codec;
pub use grpc_codec::{
    decode_frame, decode_frame_header, encode_frame as grpc_encode_frame, framed_length,
    is_complete_frame, split_frames, GrpcError, GrpcFrameHeader,
};

pub mod websocket_frame;
pub use websocket_frame::{apply_mask, is_control_frame, text_frame, WsError, WsFrame, WsOpcode};

pub mod http_parser;
pub use http_parser::{
    content_length, find_header, is_http11, parse_request, parse_response, HttpError, HttpHeader,
    HttpMethod, HttpRequest, HttpResponse,
};

pub mod oauth2_stub;
pub use oauth2_stub::{
    build_authorization_url, exchange_code_for_token, generate_pkce_challenge,
    refresh_token as oauth2_refresh_token, OAuth2Client, OAuth2Config, OAuth2Token, PkceChallenge,
};

pub mod jwt_codec;
pub use jwt_codec::{
    algorithm_name, base64url_encode, jwt_decode, jwt_encode, jwt_is_structurally_valid,
    DecodedJwt, JwtAlgorithm, JwtClaims, JwtHeader,
};

pub mod session_token;
pub use session_token::{
    create_session, generate_token as generate_session_token, purge_expired, revoke_session,
    validate_session, Session as HttpSession, SessionConfig, SessionStore as HttpSessionStore,
};

pub mod rate_limiter_sliding;
pub use rate_limiter_sliding::{
    check_and_record, evict_old, new_rate_limiter, remaining_budget, requests_in_window,
    reset_limiter, SlidingRateLimiter, SlidingRateLimiterConfig,
};

pub mod circuit_breaker;
pub use circuit_breaker::{
    current_state, is_request_allowed, new_circuit_breaker, record_failure, record_success,
    CircuitBreaker, CircuitBreakerConfig, CircuitState,
};

pub mod http_retry_policy;
pub use http_retry_policy::{
    can_retry, delay_for_attempt, new_retry_state, next_delay_ms, remaining_attempts,
    reset_retry as reset_http_retry, BackoffStrategy, HttpRetryPolicyConfig, RetryState,
};

pub mod health_check;
pub use health_check::{
    add_result, aggregate_health, all_healthy, count_by_status, new_aggregator, HealthAggregator,
    HealthCheckConfig, HealthCheckResult, HealthReport, HealthStatus,
};

pub mod service_registry;
pub use service_registry::{
    deregister_instance, new_registry as new_service_registry, register_instance, resolve_service,
    set_service_health, total_instance_count, ServiceInstance, ServiceRegistry,
    ServiceRegistryConfig,
};

pub mod message_router;
pub use message_router::{
    add_route, all_handler_ids, new_router, remove_routes_for, route_message, set_handler_enabled,
    MessageRouter, MessageRouterConfig, RoutableMessage, RouteEntry,
};

pub mod publish_subscribe;
pub use publish_subscribe::{
    clear_topic, messages_for_topic, new_pubsub_bus, publish, subscribe, subscriber_count,
    unsubscribe, PubSubBus, PubSubConfig, Subscription, TopicMessage,
};

pub mod request_pipeline;
pub use request_pipeline::{
    enabled_stage_count, new_pipeline as new_request_pipeline, register_stage, remove_stage,
    run_pipeline, stage_names, MiddlewareResult, MiddlewareStage, PipelineConfig, RequestContext,
    RequestPipeline,
};

pub mod response_cache;
pub use response_cache::{
    cache_get as response_cache_get, cache_invalidate, cache_size as response_cache_size,
    cache_store, new_response_cache, purge_expired_responses, CachedResponse, ResponseCache,
    ResponseCacheConfig,
};

pub mod content_negotiation;
pub use content_negotiation::{
    default_quality, is_text_type, mime_to_extension, negotiate, parse_accept_header, MediaType,
    NegotiationResult,
};

pub mod multipart_parser;
pub use multipart_parser::{
    extract_boundary, find_part_by_name, parse_multipart, total_body_bytes, MultipartBody,
    MultipartError, MultipartPart,
};

pub mod cookie_jar;
pub use cookie_jar::{
    delete_cookie, get_cookie, jar_size, new_cookie_jar, purge_expired_cookies,
    serialize_set_cookie, set_cookie, Cookie, CookieJar,
};

pub mod user_agent_parser;
pub use user_agent_parser::{
    browser_name, os_name, parse_user_agent, BrowserFamily, OsFamily, UserAgent,
};

pub mod locale_formatter;
pub use locale_formatter::{
    format_date_locale, format_number_locale, format_thousands, locale_currency_symbol,
    LocaleFormatter, LocaleId,
};

pub mod timezone_offset;
pub use timezone_offset::{
    convert_utc_minutes, format_offset, known_offsets, offset_difference, parse_offset,
    TimezoneOffset,
};

pub mod calendar_util;
pub use calendar_util::{
    date_to_julian_day, day_of_week, days_between, days_in_month, is_leap_year, julian_day_to_date,
    CalDate,
};

pub mod duration_parser;
pub use duration_parser::{add_durations, duration_to_string, parse_duration, IsoDuration};

pub mod cron_parser;
pub use cron_parser::{
    cron_is_wildcard_all, cron_matches, describe_cron, parse_cron, CronExpr, CronField,
};

pub mod holiday_calendar;
pub use holiday_calendar::{
    is_holiday_in, jp_national_holidays, us_federal_holidays, Holiday, HolidayCalendar, HolidayDate,
};

pub mod date_range;
pub use date_range::{
    range_intersection, range_jdn_list, range_union, ranges_overlap, ymd_to_jdn, DateRange,
};

pub mod fiscal_year;
pub use fiscal_year::{
    fiscal_quarter, fiscal_year_months, fiscal_year_of, fiscal_year_quarters, FiscalPeriod,
    FiscalYearConfig,
};

pub mod work_calendar;
pub use work_calendar::{add_business_days, count_bdays, is_business_day, next_bday, WorkCalendar};

pub mod time_series_buffer;
pub use time_series_buffer::{buffer_min_max, buffer_variance, TimeSeriesBuffer, TimeSeriesSample};

pub mod moving_avg_calc;
pub use moving_avg_calc::{ema_batch, ma_crossover, sma_batch, EmaCalc, SimpleMaCalc};

pub mod trend_detector;
pub use trend_detector::{
    detect_trend, linear_regression, moving_slope, trend_direction_label, TrendResult,
};

pub mod anomaly_scorer;
pub use anomaly_scorer::{anomaly_count, flag_anomalies, z_score_batch, AnomalyScorer};

pub mod outlier_filter;
pub use outlier_filter::{
    filter_outliers, flag_outliers, iqr_bounds, outlier_count, percentile, winsorize, IqrFilter,
};

pub mod bucket_histogram;
pub use bucket_histogram::{histogram_mean, BucketHistogram};

pub mod quantile_estimator;
pub use quantile_estimator::{median_batch, quantile_batch, P2Quantile};

pub mod feature_flag;
pub use feature_flag::{
    feature_flag_count, is_feature_enabled, new_flag_registry, register_feature, toggle_feature,
    FeatureFlagEntry, FlagState,
};

pub mod ab_test_config;
pub use ab_test_config::{
    ab_add_test, ab_select_variant, ab_total_weight, ab_variant_count, new_ab_test_config,
    AbTestConfig, AbVariant,
};

pub mod experiment_tracker;
pub use experiment_tracker::{
    new_experiment_tracker, tracker_assign, tracker_experiment_count, tracker_get_variant,
    tracker_participant_count, ExperimentAssignment, ExperimentTracker,
};

pub mod metrics_counter;
pub use metrics_counter::{
    mc_get, mc_inc, mc_inc_by, mc_reset, mc_total, new_metrics_counter, MetricsCounter,
};

pub mod metrics_gauge;
pub use metrics_gauge::{
    gauge_count, gauge_current, gauge_max, gauge_min, gauge_set, new_metrics_gauge, GaugeEntry,
    MetricsGauge,
};

pub mod metrics_histogram_sdk;
pub use metrics_histogram_sdk::{
    hist_bucket, hist_count, hist_mean, hist_record, hist_sum, new_histogram_sdk,
    MetricsHistogramSdk,
};

pub mod telemetry_span;
pub use telemetry_span::{
    new_telemetry_span, span_duration_us, span_end, span_set_attr, span_set_error, span_set_ok,
    SpanStatus, TelemetrySpan,
};

pub mod distributed_trace;
pub use distributed_trace::{
    new_trace_context, trace_child, trace_from_header, trace_is_sampled, trace_to_header,
    TraceContext,
};

pub mod log_aggregator;
pub use log_aggregator::{
    agg_count, agg_count_level, agg_push, agg_set_min, new_log_aggregator, AggLogEntry,
    AggLogLevel, LogAggregator,
};

pub mod audit_log;
pub use audit_log::{
    audit_chain_hash, audit_count, audit_filter_actor, audit_record, new_audit_log, AuditEntry,
    AuditLog,
};

pub mod change_log;
pub use change_log::{
    cl_append, cl_count, cl_filter_kind, cl_since, new_change_log, ChangeEntry, ChangeLog,
};

pub mod notification_queue;
pub use notification_queue::{
    new_notification_queue, nq_is_empty, nq_len, nq_peek_priority, nq_pop, nq_push, NotifPriority,
    Notification as QueueNotification, NotificationQueue,
};

pub mod task_scheduler;
pub use task_scheduler::{
    new_task_scheduler, ts_advance, ts_cancel, ts_run_count, ts_schedule, ts_task_count,
    RecurrenceRule, SchedulerTask, TaskScheduler,
};

pub mod deadline_tracker;
pub use deadline_tracker::{
    dt_add, dt_complete, dt_count, dt_met_count, dt_overdue_count, new_deadline_tracker,
    DeadlineEntry, DeadlineStatus, DeadlineTracker,
};

pub mod quota_manager;
pub use quota_manager::{
    new_quota_manager, qm_available, qm_consume, qm_release, qm_set, qm_utilization, QuotaEntry,
    QuotaManager,
};

pub mod capacity_planner;
pub use capacity_planner::{
    cp_add, cp_most_utilized, cp_over_threshold, cp_spec_count, new_capacity_planner,
    CapacityPlanner, CapacitySpec,
};

pub mod memory_pool_typed;
pub use memory_pool_typed::{new_memory_pool_typed, MemoryPoolTyped};

pub mod object_arena;
pub use object_arena::{new_object_arena, ArenaHandle, Generation, ObjectArena};

pub mod slab_allocator;
pub use slab_allocator::{SlabAllocator, SlabKey};

pub mod buddy_allocator;
pub use buddy_allocator::{new_buddy_allocator, BuddyAllocator};

pub mod region_allocator;
pub use region_allocator::{new_region_allocator, Region, RegionAllocator};

pub mod gc_stub;
pub use gc_stub::{new_gc_stub, GcId, GcObject, GcState, GcStub};

pub mod reference_counted;
pub use reference_counted::{
    new_ref_counted as new_manual_ref_counted, rc_count, rc_is_unique,
    RefCounted as ManualRefCounted,
};

pub mod weak_reference;
pub use weak_reference::{new_weak_pair, weak_is_alive, StrongOwner, WeakRef};

pub mod copy_on_write;
pub use copy_on_write::{cow_read, cow_share, new_cow_handle, CowHandle};

pub mod persistent_hash_map;
pub use persistent_hash_map::{new_persistent_hash_map, PersistentHashMap, PmapVersion};

pub mod persistent_vector;
pub use persistent_vector::{new_persistent_vector, PersistentVector, PvecVersion};

pub mod finger_tree;
pub use finger_tree::{new_finger_tree, FingerTree};

pub mod rope_string;
pub use rope_string::RopeString;

pub mod gap_buffer;
pub use gap_buffer::{new_gap_buffer, GapBuffer};

pub mod piece_table;
pub use piece_table::{new_piece_table, Piece, PieceSource, PieceTable};

pub mod zipper_list;
pub use zipper_list::{new_zipper_list, ZipperList};

pub mod text_diff_myers;
pub use text_diff_myers::{
    apply_diff as apply_myers_diff, diff_lines, diff_stats, edit_distance, is_same,
    EditOp as MyersEditOp, MyersDiff,
};

pub mod merge_conflict_resolver;
pub use merge_conflict_resolver::{
    auto_resolve_ours, auto_resolve_theirs, count_conflicts, format_conflict,
    three_way_merge as three_way_merge_resolve, MergeConfig, MergeResult as ConflictMergeResult,
};

pub mod patch_generator;
pub use patch_generator::{
    apply_patch_stub, generate_patch, is_identity_patch, serialize_patch, total_changed_lines,
    DiffHunk, UnifiedPatch as GeneratedPatch,
};

pub mod file_watcher_stub;
pub use file_watcher_stub::{
    drain_and_count, is_watched as path_is_watched, new_file_watcher, watch_paths, FileWatcherStub,
    FsEvent,
};

pub mod directory_scanner;
pub use directory_scanner::{
    filter_by_ext, find_by_name, new_scanner, sort_entries, total_size as scanner_total_size,
    DirEntry, DirectoryScanner, ScanConfig,
};

pub mod file_metadata;
pub use file_metadata::{
    filter_large, largest_file, newest_file, read_metadata_stub, total_size as metadata_total_size,
    FileMetadata, MetadataStore,
};

pub mod path_normalizer;
pub use path_normalizer::{
    file_extension, is_absolute, join_paths, make_absolute, normalize_path,
    strip_prefix as path_strip_prefix, NormalizedPath,
};

pub mod symlink_resolver;
pub use symlink_resolver::{
    detect_cycle, new_symlink_resolver, register_all, resolve_batch, Symlink, SymlinkResolver,
};

pub mod file_lock_stub;
pub use file_lock_stub::{
    new_lock_manager, release_all, try_exclusive, try_shared, FileLockManager, LockMode, LockRecord,
};

pub mod temp_file_manager;
pub use temp_file_manager::{
    alive_total_bytes, create_n, delete_by_path, new_temp_manager, TempFile, TempFileManager,
};

pub mod archive_reader;
pub use archive_reader::{
    list_matching, open_archive_stub, read_entry_bytes, read_entry_text, total_compressed,
    ArchiveEntry, ArchiveReader,
};

pub mod archive_writer;
pub use archive_writer::{
    build_archive, estimate_size, new_archive_writer, write_entries, ArchiveWriter, WriteEntry,
};

pub mod compression_pipeline;
pub use compression_pipeline::{
    compress_bytes, estimate_compressed_size, lz4_brotli_pipeline, zstd_pipeline, CompressAlgo,
    CompressResult, CompressionPipeline, PipelineStage as CompressPipelineStage,
};

pub mod checksum_verifier;
pub use checksum_verifier::{
    checksum_map, compute_checksum, crc32_bytes, sha256_stub, verify_checksum, verify_hex,
    Checksum, ChecksumAlgo,
};

pub mod file_transfer_stub;
pub use file_transfer_stub::{
    cancel_job, new_transfer_manager, tick_all, TransferJob, TransferManager, TransferState,
};

pub mod object_storage_stub;
pub use object_storage_stub::{
    copy_object, download, new_object_storage, upload, ObjectMeta, ObjectStorage, StoredObject,
};

pub mod error_aggregator;
pub use error_aggregator::{
    aggregator_clear, aggregator_count, aggregator_has_errors, aggregator_messages,
    aggregator_push, new_error_aggregator, ErrorAggregator,
};

pub mod capability_flags;
pub use capability_flags::{
    flags_all, flags_any, flags_clear, flags_count, flags_reset, flags_set, flags_test,
    new_capability_flags, CapabilityFlags,
};

pub mod dep_graph_simple;
pub use dep_graph_simple::{
    dep_add_edge, dep_add_node, dep_has_cycle, dep_node_count as simple_dep_node_count,
    dep_topo_sort, new_dep_graph, DepGraph,
};

pub mod resource_tracker;
pub use resource_tracker::{
    new_resource_tracker, tracker_allocate, tracker_available, tracker_free, tracker_is_full,
    tracker_peak, tracker_reset, tracker_utilization, ResourceTracker,
};

pub mod lazy_eval;
pub use lazy_eval::{
    lazy_get_or_compute, lazy_invalidate, lazy_is_computed, lazy_set, new_lazy_f32, LazyValue,
};

pub mod feature_gate;
pub use feature_gate::{
    gate_clear, gate_disable, gate_enable, gate_feature_count, gate_is_enabled, gate_toggle,
    new_feature_gate, FeatureGate,
};

pub mod snapshot_manager;
pub use snapshot_manager::{
    new_snapshot_manager, snapshot_clear, snapshot_count, snapshot_get, snapshot_is_full,
    snapshot_latest, snapshot_save, SnapshotManager,
};

pub mod health_monitor;
pub use health_monitor::{
    health_all_ok, health_count, health_failing_count, health_register, health_summary,
    health_update, new_health_monitor, HealthMonitor,
};

pub mod error_audit_log;
pub use error_audit_log::{
    audit_event_by_actor, audit_event_clear, audit_event_count, audit_event_last,
    audit_event_record, audit_event_since, new_audit_event_log, AuditEventEntry, AuditEventLog,
};

pub mod clock_version_vector;
pub use clock_version_vector::{
    cvv_concurrent, cvv_dominates, cvv_get, cvv_increment, cvv_merge, cvv_node_count,
    new_clock_version_vector, ClockVersionVector,
};

pub mod compression_stub;
pub use compression_stub::{compress_rle, compression_ratio, decompress_rle, is_compressible};

pub mod token_bucket_limiter;
pub use token_bucket_limiter::{
    limiter_available, limiter_consume, limiter_is_full, limiter_refill, limiter_reset,
    new_rate_limiter_tb, TokenBucketLimiter,
};

pub mod simple_message_queue;
pub use simple_message_queue::{
    new_message_queue_simple, smq_clear, smq_is_empty, smq_is_full, smq_len, smq_peek, smq_pop,
    smq_push, SimpleMessageQueue,
};

pub mod config_diff;
pub use config_diff::{
    config_added_keys, config_changed_keys, config_diff_count, config_is_identical,
    config_removed_keys,
};

pub mod resolve_path_utils;
pub use resolve_path_utils::{
    path_basename, path_dirname, path_ext, path_is_abs, path_join_parts, resolve_path,
    resolve_path_normalize,
};

pub mod disjoint_set;
pub use disjoint_set::{
    ds_component_count, ds_connected, ds_find, ds_same, ds_size, ds_union, new_disjoint_set,
    DisjointSet,
};

pub mod bloom_filter_prob;
pub use bloom_filter_prob::{
    bloom_prob_bit_count, bloom_prob_estimated_fp_rate, bloom_prob_hash_count, bloom_prob_insert,
    bloom_prob_may_contain, bloom_prob_reset, new_bloom_filter_prob, BloomFilterProb,
};

pub mod fenwick_tree_v2;
pub use fenwick_tree_v2::{
    ft2_add, ft2_len, ft2_point_query, ft2_prefix_sum, ft2_range_sum, new_fenwick_tree_v2,
    FenwickTreeV2,
};

pub mod segment_tree_range;
pub use segment_tree_range::{
    new_segment_tree_range, seg_range_query_max, seg_range_query_min, seg_range_size,
    seg_range_update, SegmentTreeRange,
};

pub mod interval_tree_simple;
pub use interval_tree_simple::{
    itree_simple_contains_point, itree_simple_count_overlaps, itree_simple_insert,
    itree_simple_query_overlaps, itree_simple_size, new_interval_tree_simple, IntervalSimple,
    IntervalTreeSimple,
};

pub mod hash_ring;
pub use hash_ring::{
    add_ring_node, get_ring_node, new_hash_ring, remove_ring_node, ring_distribution,
    ring_node_count, ring_rebalance, ring_to_json, HashRing,
};

pub mod hash_ring_new;
pub use hash_ring_new::{
    new_hash_ring_new, ring_new_add_node, ring_new_get_node, ring_new_is_empty,
    ring_new_node_count, ring_new_remove_node, HashRingNew,
};

pub mod rolling_hash;
pub use rolling_hash::{
    find_pattern, new_rolling_hash, rolling_hash_push, rolling_hash_value,
    simple_hash as rh_simple_hash, RollingHash,
};

pub mod rolling_hash_new;
pub use rolling_hash_new::{
    new_rolling_hash_new, rh_new_hash, rh_new_push, rh_new_reset, rh_new_window_full,
    rh_new_window_size, RollingHashNew,
};

pub mod edit_distance;
pub use edit_distance::{
    common_prefix_len, hamming_distance, is_within_distance, jaro_similarity, levenshtein,
};

pub mod edit_distance_lev;
pub use edit_distance_lev::{
    edit_distance_bounded_lev, edit_distance_lev, edit_is_close_lev, edit_similarity_lev,
    longest_common_subsequence_lev,
};

pub mod fuzzy_matcher;
pub use fuzzy_matcher::{
    fuzzy_best_match, fuzzy_match_score, fuzzy_matches, fuzzy_rank_candidates,
};

pub mod tokenizer_simple;
pub use tokenizer_simple::{
    token_count as word_token_count, token_frequency, tokenize_sentences, tokenize_words,
    unique_tokens,
};

pub mod prefix_sum_2d;
pub use prefix_sum_2d::{new_prefix_sum_2d, ps2d_cols, ps2d_query, ps2d_rows, PrefixSum2d};

pub mod skip_list_simple;
pub use skip_list_simple::{
    new_skip_list_simple, skip_simple_contains, skip_simple_insert, skip_simple_len,
    skip_simple_range, skip_simple_remove, SkipListSimple,
};

pub mod regex_stub;
pub use regex_stub::{regex_count_matches, regex_first_match, regex_match, regex_match_all};

pub mod aho_corasick_stub;
pub use aho_corasick_stub::{
    ac_stub_any_match, ac_stub_count_matches, ac_stub_first_match, ac_stub_search,
};

pub mod suffix_array_stub;
pub use suffix_array_stub::{
    build_suffix_array_stub, lcp_array_stub, sa_stub_count_occurrences, sa_stub_find_all,
    sa_stub_longest_repeated_substring, sa_stub_search,
};

pub mod b_tree_simple;
pub use b_tree_simple::{
    btree_simple_get, btree_simple_insert, btree_simple_len, btree_simple_range,
    btree_simple_remove, new_btree_simple, BTreeSimple,
};

pub mod merkle_tree;
pub use merkle_tree::{
    merkle_combine_hashes, merkle_leaf_count, merkle_root, merkle_verify_leaf, new_merkle_tree,
    MerkleTree,
};

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

pub mod hex_codec_new;
pub use hex_codec_new::{
    hex_byte_count_new, hex_decode_new, hex_encode_new, hex_encode_upper_new, hex_is_valid_new,
};

pub mod uuid_generator;
pub use uuid_generator::{
    uuid_from_bytes, uuid_from_u128, uuid_is_valid_string, uuid_nil, uuid_to_string, uuid_version,
    Uuid,
};

pub mod bitset_fixed;
pub use bitset_fixed::{
    bitset_and, bitset_clear, bitset_count_ones, bitset_count_zeros, bitset_flip, bitset_get,
    bitset_or, bitset_set, new_bitset_fixed, BitsetFixed,
};

pub mod varint_u64_codec;
pub use varint_u64_codec::{
    varint_decode_i64, varint_decode_u64, varint_encode_i64, varint_encode_u64,
    varint_encoded_size_u64,
};

pub mod endian_utils;
pub use endian_utils::{
    f32_from_le_bytes, f32_to_le_bytes, is_little_endian, u16_from_le, u16_to_le, u32_from_be,
    u32_from_le, u32_to_be, u32_to_le,
};

pub mod crc_simple;
pub use crc_simple::{
    crc16 as crc16_simple, crc16_check as crc16_simple_check, crc16_update, crc8 as crc8_simple,
    crc8_check as crc8_simple_check, crc8_update,
};

pub mod fletcher_checksum;
pub use fletcher_checksum::{
    fletcher16 as fletcher16_cs, fletcher16_check as fletcher16_cs_check, fletcher16_combine,
    fletcher32, fletcher32_check,
};

pub mod gray_code;
pub use gray_code::{from_gray, gray_bits, gray_distance, gray_next, gray_prev, to_gray};

pub mod morton_code;
pub use morton_code::{
    morton_decode_2d, morton_decode_3d, morton_encode_2d, morton_encode_3d, morton_neighbor,
};

pub mod hilbert_curve;
pub use hilbert_curve::{
    hilbert_d_to_xy, hilbert_max_index, hilbert_order_for_size, hilbert_xy_to_d,
};

pub mod packed_array;
pub use packed_array::{
    new_packed_array, packed_bits_per_elem, packed_get, packed_len, packed_set,
    packed_storage_bytes, PackedArray,
};

pub mod hamming_code;
pub use hamming_code::{
    hamming_decode_nibble, hamming_encode_nibble, hamming_introduce_error, hamming_is_valid,
    hamming_syndrome,
};

pub mod bitmask_flags;
pub use bitmask_flags::{
    bmf_clear, bmf_count_set, bmf_raw, bmf_set, bmf_set_by_index, bmf_test, new_bitmask_flags,
    BitmaskFlags,
};

pub mod atomic_counter_stub;
pub use atomic_counter_stub::{
    counter_add, counter_compare_and_swap, counter_decrement, counter_get, counter_increment,
    counter_reset, new_atomic_counter, AtomicCounter,
};

pub mod string_utils;
pub use string_utils::{
    str_camel_to_snake, str_capitalize, str_count_char, str_pad_left, str_pad_right, str_repeat,
    str_reverse, str_snake_to_camel, str_truncate,
};

pub mod iterator_utils;
pub use iterator_utils::{
    chunks_of, drop_while_vec, flat_map_vec, intersperse, partition_vec, take_while_vec,
    zip_with_vecs,
};

pub mod result_utils;
pub use result_utils::{
    collect_results, first_ok, map_err_str, ok_or_else_str, transpose_option_result,
    unwrap_or_default_str,
};

pub mod observer_pattern;
pub use observer_pattern::{
    bus_clear, bus_emit, bus_event_count, bus_events_of_type, bus_has_event_type,
    new_event_bus_observer, ObserverEventBus,
};

pub mod pipeline_pattern;
pub use pipeline_pattern::{
    context_advance, context_get, context_set, context_stage, new_pipeline_context_struct,
    new_pipeline_struct, pipeline_stage_count, PipelineContextStruct, PipelineStruct,
};

pub mod specification_pattern;
pub use specification_pattern::{
    new_spec, spec_and, spec_name, spec_not, spec_or, spec_range, spec_satisfies_range, Spec,
};

pub mod entity_id;
pub use entity_id::{
    entity_id_is_same_kind, entity_id_kind, entity_id_nil, entity_id_parse, entity_id_to_string,
    entity_id_value, new_entity_id, EntityId,
};

pub mod value_object;
pub use value_object::{
    money_add, money_to_string, new_money, new_percentage, percentage_clamp,
    percentage_from_fraction, percentage_to_fraction, Money, Percentage,
};

pub mod aggregate_root;
pub use aggregate_root::{
    aggregate_apply_event, aggregate_clear_events, aggregate_id, aggregate_increment_version,
    aggregate_pending_events, aggregate_version, new_aggregate_root, AggregateRoot,
};

pub mod domain_event;
pub use domain_event::{
    event_is_type, event_to_json, events_after, events_by_aggregate, events_of_type,
    new_domain_event, DomainEvent,
};

pub mod command_bus_ddd;
pub use command_bus_ddd::{
    log_clear, log_command_count, log_commands_by_name, log_dispatch, log_last_command,
    new_command_log, new_ddd_command, CommandLog, DddCommand,
};

pub mod query_bus;
pub use query_bus::{
    new_query, new_query_result, query_get_param, query_result_data, query_result_is_success,
    query_set_param, Query, QueryResult,
};

pub mod repository_stub;
pub use repository_stub::{
    new_string_repo, repo_all_ids, repo_count, repo_delete, repo_exists, repo_find, repo_save,
    StringRepo,
};

pub mod event_sourcing;
pub use event_sourcing::{
    es_append, es_events_for, es_latest_version, es_replay, es_total_events, new_event_store,
    EventStore,
};

pub mod strategy_pattern;
pub use strategy_pattern::{
    new_sort_strategy, sort_apply_f32, sort_apply_str, sort_is_ascending, sort_reverse,
    sort_strategy_name, SortStrategy,
};

pub mod option_ext;
pub use option_ext::{
    option_count, option_filter_positive, option_map_or_zero, option_or_default_f32, option_sum,
    option_to_result, option_zip_with,
};

// ── Wave 151A: Core Math Modules ────────────────────────────────────────────

pub mod vector_math;
pub use vector_math::{
    vec3_add, vec3_cross, vec3_dot, vec3_len, vec3_lerp, vec3_norm, vec3_scale, vec3_sub,
};

pub mod color_math;
pub use color_math::{
    color_lerp as cm_color_lerp, hsl_to_rgb as cm_hsl_to_rgb, hsv_to_rgb as cm_hsv_to_rgb,
    linear_to_srgb_cm, luminance_cm, rgb_to_hsl as cm_rgb_to_hsl, rgb_to_hsv as cm_rgb_to_hsv,
    srgb_to_linear_cm,
};

pub mod noise_functions;
pub use noise_functions::{
    checkerboard, fractal_noise_2d, gradient_noise_1d, ridged_noise_2d, turbulence_2d,
    value_noise_2d, white_noise,
};

pub mod geometry_2d;
pub use geometry_2d::{
    circle_area, dist_2d, point_in_circle, point_in_rect, point_in_triangle_2d,
    polygon_perimeter_2d, segment_intersect, triangle_area_2d,
};

pub mod geometry_3d;
pub use geometry_3d::{
    barycentric_2d, box_volume, closest_point_on_segment, dist_3d, ray_plane_intersect,
    ray_sphere_intersect, sphere_volume, triangle_normal,
};

pub mod matrix2x2;
pub use matrix2x2::{
    mat2_det as mat2x2_det, mat2_identity as mat2x2_identity, mat2_inv, mat2_mul as mat2x2_mul,
    mat2_transform as mat2x2_transform, mat2_transpose as mat2x2_transpose, Mat2 as Mat2x2,
};

pub mod matrix3x3;
pub use matrix3x3::{
    mat3_det as mat3x3_det, mat3_from_rotation_z, mat3_from_scale,
    mat3_identity as mat3x3_identity, mat3_mul as mat3x3_mul, mat3_transform as mat3x3_transform,
    mat3_transpose as mat3x3_transpose, Mat3 as Mat3x3,
};

pub mod easing_functions;
pub use easing_functions::{
    ease_bounce_out, ease_elastic_out, ease_in_cubic as ease_in_cubic_fn, ease_in_out_quad,
    ease_in_out_sine, ease_in_quad, ease_linear as ease_linear_fn,
    ease_out_cubic as ease_out_cubic_fn, ease_out_quad,
};

pub mod transform3d;
pub use transform3d::{
    transform_apply, transform_combine, transform_identity, transform_inverse_translation,
    transform_lerp, transform_rotate, transform_scale_uniform, transform_to_mat4,
    transform_translate, Transform3d,
};

pub mod quaternion_math;
pub use quaternion_math::{
    quat_conjugate as qm_quat_conjugate, quat_from_axis_angle as quat_from_axis_angle_math,
    quat_identity as quat_identity_math, quat_mul as quat_mul_math, quat_norm_sq,
    quat_normalize as quat_normalize_math, quat_rotate_vec3 as quat_rotate_vec3_math,
    quat_slerp as quat_slerp_math, quat_to_euler, QuatMath,
};

pub mod random_utils;
pub use random_utils::{lcg_choose, lcg_normal, lcg_sample_uniform, lcg_shuffle, Lcg};

pub mod statistics_utils;
pub use statistics_utils::{
    max_val, mean, median, min_val, pearson_r, percentile as su_percentile, std_dev, variance,
};

pub mod bezier_curve;
pub use bezier_curve::{
    bezier_cubic, bezier_cubic_2d, bezier_cubic_sample, bezier_cubic_tangent, bezier_quadratic,
    bezier_quadratic_2d,
};

pub mod spline_curve;
pub use spline_curve::{
    catmull_rom_2d as catmull_rom_2d_spline, catmull_rom_3d, catmull_rom_chain_2d, catmull_rom_f32,
    catmull_rom_tangent_f32,
};

pub mod color_palette_gen;
pub use color_palette_gen::{
    palette_analogous, palette_complementary, palette_cosine, palette_gradient,
    palette_monochromatic, palette_rainbow, palette_triadic,
};

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

pub mod frequency_analyzer;
pub use frequency_analyzer::{
    dc_component, magnitude_spectrum, new_frequency_analyzer, peak_bin_index, FrequencyAnalyzer,
    FrequencyBin,
};

pub mod histogram_builder;
pub use histogram_builder::{new_histogram, HistBin, HistogramBuilder};

pub mod graph_search;
pub use graph_search::{new_adj_graph, AdjGraph};

pub mod priority_queue_ext;
pub use priority_queue_ext::{new_priority_queue_ext, PqEntry, PriorityQueueExt};

pub mod matrix_solver;
pub use matrix_solver::{determinant, gaussian_solve, identity_matrix, mat_vec_mul, residual_norm};

pub mod polynomial_eval;
pub use polynomial_eval::{horner_eval, new_polynomial, poly_deriv, poly_eval, Polynomial};

pub mod running_statistics;
pub use running_statistics::{new_running_stats, RunningStatistics};

pub mod string_search;
pub use string_search::{new_string_searcher, StringSearcher};

pub mod median_filter;
pub use median_filter::{median_filter_1d, new_median_filter, slice_median, MedianFilter};

pub mod moving_average;
pub use moving_average::{
    apply_ema, apply_sma, new_ema, new_sma, new_wma, ExponentialMovingAverage, SimpleMovingAverage,
    WeightedMovingAverage,
};

pub mod color_quantizer;
pub use color_quantizer::{
    color_dist_sq, new_color_quantizer, quantize_pixels, ColorQuantizer, RgbColor,
};

pub mod hyperloglog;
pub use hyperloglog::HyperLogLog;

pub mod count_min_sketch;
pub use count_min_sketch::CountMinSketch;

pub mod t_digest;
pub use t_digest::{Centroid, TDigest};

pub mod bloom_filter_counting;
pub use bloom_filter_counting::{
    cbf_contains_str, cbf_insert_str, cbf_remove_str, new_counting_bloom_filter,
    CountingBloomFilter,
};

pub mod reservoir_sample;
pub use reservoir_sample::{
    feed_weighted, new_reservoir_sampler, sample_indices, sample_slice, ReservoirSampler,
};

pub mod hash_grid;
pub use hash_grid::{hg_insert_2d, hg_query_2d, new_hash_grid, HashGrid, HgPoint3};

pub mod kd_tree;
pub use kd_tree::{new_kd_tree, new_kd_tree_2d, KdPoint3 as KdPoint3Simple, KdTree3};

pub mod octree_simple;
pub use octree_simple::{new_simple_octree, OctAabb, SimpleOctree3};

pub mod bvh_simple;
pub use bvh_simple::{new_bvh, BvhAabb, BvhPrimitive, SimpleBvh};

pub mod grid_index;
pub use grid_index::{new_grid_index, new_grid_index_region, GridIndex};

pub mod ear_clip_triangulate;
pub use ear_clip_triangulate::{
    ear_clip, ear_clip_flat, is_convex, polygon_bbox,
    polygon_signed_area as ec_polygon_signed_area, EcPoint,
};

pub mod huffman_stub;
pub mod lz77_stub;
pub use huffman_stub::{
    build_frequency_table, decode_symbol, encode_symbol, huffman_decode, huffman_encode,
    table_size, BitReader, BitWriter, HuffNode, HuffmanCodeTable, HuffmanError, HuffmanSymbol,
    HuffmanTable, HuffmanTree,
};

pub mod security;
pub use security::{
    checked_stride_offset, is_safe_content_type, sanitize_path, validate_file_size, SecurityError,
};
