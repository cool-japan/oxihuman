// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
//! Core infrastructure: policy, manifest, parser, asset pack, event bus,
//! workspace, metrics, command bus, task graph, caches, registries, and
//! small utility data structures (arena_str … transform_pipe).

#[path = "category.rs"]
pub mod category;
#[path = "integrity.rs"]
pub mod integrity;
#[path = "manifest.rs"]
pub mod manifest;
#[path = "pack_verify.rs"]
pub mod pack_verify;
#[path = "parser/mod.rs"]
pub mod parser;
#[path = "policy.rs"]
pub mod policy;
#[path = "report.rs"]
pub mod report;
#[path = "target_index.rs"]
pub mod target_index;

pub use category::TargetCategory;
pub use manifest::AssetManifest;
pub use pack_verify::{
    scan_pack, verify_manifest_present, verify_pack, FileRecord, PackVerifyReport,
};
pub use policy::{Policy, PolicyProfile};
pub use report::{PipelineReport, ReportBuilder, ReportEvent, Severity};
pub use target_index::{TargetEntry, TargetIndex, TargetScanner};
#[path = "pack_distribute.rs"]
pub mod pack_distribute;
pub use pack_distribute::{
    InstalledPack, PackBuilder, PackDependency, PackIntegrity, PackManifest, PackRegistry,
    PackTargetEntry, PackVerifier,
};

#[path = "pack_sign.rs"]
pub mod pack_sign;
pub use pack_sign::{
    double_hash_sign, pack_manifest_hash, read_signature_file, sign_pack_dir, signature_from_hex,
    signature_to_hex, verify_pack_signature, write_signature_file, PackSignature, SignedPack,
};

#[path = "asset_pack_builder.rs"]
pub mod asset_pack_builder;
pub use asset_pack_builder::{
    build_alpha_pack, load_pack_from_bytes, AssetPackBuilder, AssetPackEntry, AssetPackIndex,
    AssetPackMeta, MaterialDef, MorphPreset, TargetDelta, TextureAsset, TextureFormat,
};

#[path = "plugin_registry.rs"]
pub mod plugin_registry;
pub use plugin_registry::{
    default_builtin_plugins, parse_semver, semver_gte, PluginDescriptor, PluginKind, PluginRegistry,
};
#[path = "event_bus.rs"]
pub mod event_bus;
pub use event_bus::{
    make_error_event, make_export_event, make_param_changed_event, Event, EventBus, EventKind,
};
#[path = "asset_hash.rs"]
pub mod asset_hash;
pub use asset_hash::{
    hash_bytes, hash_file_content, AssetHash, AssetHasher, AssetRecord, AssetRegistry,
};

#[path = "workspace.rs"]
pub mod workspace;
pub use workspace::{
    default_workspace_config, workspace_summary, AssetEntry, Workspace, WorkspaceConfig,
};
#[path = "metrics.rs"]
pub mod metrics;
pub use metrics::{Metric, MetricKind, MetricSample, MetricsRegistry};
#[path = "spatial_index.rs"]
pub mod spatial_index;
pub use spatial_index::{
    build_octree, insert_point, k_nearest_neighbors, nearest_neighbor, octree_depth,
    octree_leaf_count, octree_point_count, octree_stats, query_aabb, query_sphere, ray_query,
    Octree, OctreeNode,
};

#[path = "command_bus.rs"]
pub mod command_bus;
pub use command_bus::{
    clear_history, command_descriptions, execute_command, new_command_bus, new_command_state,
    redo_count, redo_last, undo_count, undo_last, BatchCommand, Command, CommandBus, CommandResult,
    CommandState, SetFlagCommand, SetParamCommand,
};

#[path = "task_graph.rs"]
pub mod task_graph;
pub use task_graph::{
    add_task, completed_count, critical_path_length, execute_sequential, failed_count,
    get_ready_tasks, graph_to_json, mark_complete, mark_failed, new_task_graph, pending_count,
    reset_graph, task_count, topological_order, Task, TaskGraph, TaskStatus,
};

#[path = "config_schema.rs"]
pub mod config_schema;
pub use config_schema::{
    add_field, apply_defaults, config_value_get_bool, config_value_get_float, config_value_get_int,
    config_value_get_str, default_render_schema, merge_configs, new_config_schema, schema_to_json,
    validate_field_value, validate_value, ConfigSchema, ConfigValue, SchemaField, SchemaType,
};

#[path = "undo_redo.rs"]
pub mod undo_redo;
pub use undo_redo::{
    can_redo, can_undo, clear_undo_history, command_names, future_depth, history_depth,
    new_undo_stack, peek_redo, peek_undo, push_command, redo, truncate_history, undo, UndoCommand,
    UndoStack,
};

#[path = "asset_cache.rs"]
pub mod asset_cache;
pub use asset_cache::{
    cache_clear, cache_contains, cache_count, cache_get, cache_hit_rate, cache_insert,
    cache_remove, cache_size, cache_stats, evict_lru, evict_until_fits, most_accessed, new_cache,
    AssetCache, CacheEntry,
};

#[path = "plugin_api.rs"]
pub mod plugin_api;
pub use plugin_api::{
    activate_plugin, active_plugins, check_dependencies_met, deactivate_plugin, dependency_order,
    get_plugin, has_dependency, new_registry, plugin_count, plugin_version_string, register_plugin,
    set_plugin_error, unload_plugin, Plugin, PluginApiRegistry, PluginMetadata, PluginState,
};

#[path = "serialization.rs"]
pub mod serialization;
pub use serialization::{
    f32_array_to_json, json_finalize, json_key_f32, json_key_str, json_key_u32, new_bin_reader,
    new_bin_writer, new_json_builder, read_f32_le, read_str, read_u16_le, read_u32_le, read_u8,
    u32_array_to_json, write_bytes, write_f32_le, write_str, write_u16_le, write_u32_le, write_u8,
    BinReader, BinWriter, JsonBuilder,
};

#[path = "event_log.rs"]
pub mod event_log;
pub use event_log::{
    clear_log as clear_event_log, error_count, event_count, events_since, filter_by_category,
    filter_by_level, last_event, log_event, log_with_data, new_event_log, serialize_log_json,
    trim_log, warn_count, EventLog, LogEvent, LogLevel,
};

#[path = "resource_manager.rs"]
pub mod resource_manager;
pub use resource_manager::{
    fail_resource, failed_count as failed_resource_count, garbage_collect,
    get_by_key as get_resource_by_key, get_resource, load_resource,
    loaded_count as loaded_resource_count, new_resource_manager, register_resource,
    release_resource, retain_resource, total_memory, unload_resource, Resource, ResourceManager,
    ResourceState,
};

#[path = "hot_reload.rs"]
pub mod hot_reload;
pub use hot_reload::{
    change_count, changes_for_path, clear_changes as clear_reload_changes,
    default_hot_reload_config, disable_watcher, enable_watcher, extension_matches, is_watched,
    new_watcher, pending_changes, simulate_file_change, unwatch_path, watch_path,
    watched_path_count, ChangeKind, FileChange, HotReloadConfig, HotReloadWatcher,
};

#[path = "debug_console.rs"]
pub mod debug_console;
pub use debug_console::{
    console_clear as clear_debug_console, console_entries_by_severity, console_entry_count,
    console_error_count, console_last_entry, console_log, console_to_string,
    default_debug_console_config, new_debug_console, severity_name, ConsoleEntry, ConsoleSeverity,
    DebugConsole, DebugConsoleConfig,
};

#[path = "data_pipeline.rs"]
pub mod data_pipeline;
pub use data_pipeline::{
    add_stage as pipeline_add_stage, advance_stage, completed_stage_count, failed_stages,
    get_context_value, mark_stage_complete, mark_stage_failed, mark_stage_skipped, new_pipeline,
    pipeline_progress, pipeline_to_json, reset_pipeline, set_context_value, stage_count,
    DataPipeline, PipelineStage, StageStatus,
};

#[path = "type_registry.rs"]
pub mod type_registry;
pub use type_registry::{
    add_property as registry_add_property, all_categories, get_type as registry_get_type, has_type,
    new_type_registry, property_count, register_type, serializable_types, type_count,
    type_registry_to_json, types_in_category, unregister_type, validate_type_meta, TypeMetadata,
    TypeRegistry,
};

#[path = "localization.rs"]
pub mod localization;
pub use localization::{
    add_locale_string, add_locale_table, export_locale_json, has_key as locale_has_key,
    import_locale_strings, key_count as locale_key_count, locale_count, missing_keys,
    new_locale_table, new_localization, set_active_locale, translate, translate_with_context,
    LocaleString, LocaleTable, LocalizationSystem,
};

#[path = "version_migration.rs"]
pub mod version_migration;
pub use version_migration::{
    has_migration_path, is_breaking_change, latest_version, migration_description,
    migration_step_count, new_migration_registry, new_semver, plan_has_breaking, plan_migration,
    register_migration, semver_compare, semver_parse, semver_to_string, MigrationPlan,
    MigrationRegistry, MigrationStep, SemVer,
};

#[path = "dependency_resolver.rs"]
pub mod dependency_resolver;
pub use dependency_resolver::{
    add_dep_node, all_dependents_transitive, dep_graph_to_json, dep_node_count, direct_dependents,
    get_dep_node, has_circular_dependency, missing_dependencies, new_dependency_graph,
    optional_dep_count, remove_dep_node, resolve_dependencies, Dependency, DependencyGraph,
    DependencyNode, ResolveError, ResolveResult,
};

#[path = "scheduler.rs"]
pub mod scheduler;
pub use scheduler::{
    advance_time as scheduler_advance, cancel_task as scheduler_cancel, clear_completed_tasks,
    due_tasks, enabled_task_count, get_scheduled_task, new_scheduler, next_due_time, schedule_once,
    schedule_repeating, set_task_enabled, task_count as scheduler_task_count, tasks_by_priority,
    ScheduledTask, Scheduler, TaskPriority,
};

#[path = "profiler.rs"]
pub mod profiler;
pub use profiler::{
    average_frame_ns, begin_span, clear_profiler, disable_profiler, enable_profiler, end_frame,
    end_span, frame_count_profiler, hottest_span, last_frame as profiler_last_frame, new_profiler,
    profiler_to_json, span_by_name, span_duration_ns, total_frame_ns, ProfileFrame, ProfileSpan,
    Profiler,
};

#[path = "feature_flags.rs"]
pub mod feature_flags;
pub use feature_flags::{
    all_enabled_flags, default_bool_flag, default_int_flag, feature_registry_to_json, flag_count,
    flags_with_tag, get_flag, get_flag_bool, get_flag_int, is_enabled, new_feature_registry,
    register_flag, remove_flag, set_flag_value, FeatureFlag, FeatureFlagRegistry, FlagValue,
};

#[path = "user_preferences.rs"]
pub mod user_preferences;
pub use user_preferences::{
    get_bool as pref_get_bool, get_float as pref_get_float, get_int as pref_get_int, get_pref,
    get_string as pref_get_string, mark_clean, new_user_preferences, pref_count,
    preferences_from_pairs, preferences_to_json, prefs_in_category, remove_pref, reset_to_defaults,
    set_pref, PrefValue, Preference, UserPreferences,
};

#[path = "notification_system.rs"]
pub mod notification_system;
pub use notification_system::{
    active_count as active_notification_count, active_notifications, advance_notifications,
    clear_all_notifications, dismiss_notification, has_errors, new_notification_system,
    notification_by_id, notification_count, notifications_by_severity,
    push_error as push_error_notification, push_info as push_info_notification, push_notification,
    Notification, NotificationSeverity, NotificationSystem,
};

#[path = "command_queue.rs"]
pub mod command_queue;
pub use command_queue::{
    clear_queue, command_count, command_queue_to_json, commands_by_priority, dequeue, drain_all,
    enqueue, enqueue_batch, has_priority, is_queue_empty, max_queue_depth, new_command_queue,
    peek_next, total_enqueued, CommandPriority, CommandQueue, QueuedCommand,
};

#[path = "memory_tracker.rs"]
pub mod memory_tracker;
pub use memory_tracker::{
    allocation_count, budget_remaining, current_usage, free_count, largest_category,
    memory_tracker_to_json, new_memory_tracker, over_budget, peak_usage, reset_tracker, set_budget,
    track_alloc, track_free, usage_by_category, AllocationRecord, MemoryCategory, MemoryTracker,
};

#[path = "clipboard.rs"]
pub mod clipboard;
pub use clipboard::{
    clear_clipboard, clipboard_content_type, clipboard_has_content, clipboard_history_count,
    clipboard_to_json, copy_color, copy_parameters, copy_pose, copy_text, copy_to_clipboard,
    get_history_entry, new_clipboard, paste_from_clipboard, undo_paste, Clipboard,
    ClipboardContent, ClipboardEntry,
};

#[path = "string_pool.rs"]
pub mod string_pool;
pub use string_pool::{
    clear_pool, contains as pool_contains, find_by_prefix, intern, intern_many, merge_pools,
    new_string_pool, pool_size, pool_stats_json, remove_unused, resolve, string_id_valid,
    total_bytes, StringId, StringPool,
};

#[path = "logger.rs"]
pub mod logger;
pub use logger::{
    clear_log as clear_logger_log, entries_by_level, entry_count,
    filter_by_category as logger_filter_by_category, has_errors as logger_has_errors,
    last_n_entries as logger_last_n_entries, log_debug, log_error, log_info, log_message,
    log_trace, log_warn, logger_to_json, new_logger, set_min_level, LogEntry,
    LogLevel as LoggerLogLevel, Logger,
};

#[path = "color_space.rs"]
pub mod color_space;
pub use color_space::{
    clamp_color, color_distance_lab, color_temperature_to_rgb, hsl_to_rgb, hsv_to_rgb, lab_to_rgb,
    lerp_hsl, lerp_rgb, linear_to_srgb as cs_linear_to_srgb, luminance, rgb_to_hsl, rgb_to_hsv,
    rgb_to_lab, srgb_to_linear as cs_srgb_to_linear, ColorHsl, ColorHsv, ColorLab, ColorRgb,
};

#[path = "topic_event_bus.rs"]
pub mod topic_event_bus;
pub use topic_event_bus::{
    clear_event_bus, dispatch_pending as event_dispatch_pending, drain_topic, event_bus_to_json,
    event_count_total, has_subscribers, last_event_time, new_event_bus, publish as event_publish,
    publish_priority, subscribe as event_subscribe, topic_subscriber_count,
    unsubscribe as event_unsubscribe, EventBusTopic, EventPriority, EventRecord, PendingEvents,
    SubscriberId,
};

#[path = "config_manager.rs"]
pub mod config_manager;
pub use config_manager::{
    active_profile, config_from_pairs, config_to_json, create_profile, delete_profile,
    get_profile_value, get_value_with_fallback, list_profiles, merge_profiles, new_config_manager,
    profile_count, reset_profile_to_defaults, set_profile_value, switch_profile, ConfigManager,
    ConfigProfile, ConfigValue as CfgValue,
};

#[path = "arena_str.rs"]
pub mod arena_str;
pub use arena_str::ArenaStr;

#[path = "bit_set.rs"]
pub mod bit_set;
pub use bit_set::BitSet;

#[path = "bloom_counter.rs"]
pub mod bloom_counter;
pub use bloom_counter::BloomCounter;

#[path = "byte_pool.rs"]
pub mod byte_pool;
pub use byte_pool::BytePool;

#[path = "cache_line.rs"]
pub mod cache_line;
pub use cache_line::CacheLine;

#[path = "channel_pair.rs"]
pub mod channel_pair;
pub use channel_pair::ChannelPair;

#[path = "clock_source.rs"]
pub mod clock_source;
pub use clock_source::ClockSource;

#[path = "compact_vec.rs"]
pub mod compact_vec;
pub use compact_vec::CompactVec;

#[path = "config_val.rs"]
pub mod config_val;
pub use config_val::{ConfigStore, ConfigVal as TypedConfigVal};

#[path = "counter_map.rs"]
pub mod counter_map;
pub use counter_map::CounterMap;

#[path = "data_table.rs"]
pub mod data_table;
pub use data_table::DataTable;

#[path = "digest_hash.rs"]
pub mod digest_hash;
pub use digest_hash::DigestHash;

#[path = "double_list.rs"]
pub mod double_list;
pub use double_list::DoubleList;

#[path = "event_sink.rs"]
pub mod event_sink;
pub use event_sink::{EventRecord as SinkEventRecord, EventSink};

#[path = "flag_register.rs"]
pub mod flag_register;
pub use flag_register::FlagRegister;

#[path = "frame_counter.rs"]
pub mod frame_counter;
pub use frame_counter::FrameCounter;

#[path = "action_map.rs"]
pub mod action_map;
pub use action_map::{ActionEntry, ActionMap};

#[path = "async_queue.rs"]
pub mod async_queue;
pub use async_queue::{AsyncQueue, AsyncTask, TaskState};

#[path = "batch_processor.rs"]
pub mod batch_processor;
pub use batch_processor::BatchProcessor;

#[path = "bloom_set.rs"]
pub mod bloom_set;
pub use bloom_set::BloomSet;

#[path = "buffer_slice.rs"]
pub mod buffer_slice;
pub use buffer_slice::BufferSlice;

#[path = "cache_entry.rs"]
pub mod cache_entry;
pub use cache_entry::{CacheEntryItem, CacheStore};

#[path = "chain_map.rs"]
pub mod chain_map;
pub use chain_map::ChainMap;

#[path = "checkpoint_store.rs"]
pub mod checkpoint_store;
pub use checkpoint_store::{Checkpoint, CheckpointStore};

#[path = "collection_ops.rs"]
pub mod collection_ops;
pub use collection_ops::{
    chunk_vec, dedup_sorted, flatten_nested, interleave, max_f32, mean_f32, min_f32, partition_by,
    sliding_window_avg, sum_f32, unique_sorted, zip_with,
};

#[path = "compact_hash.rs"]
pub mod compact_hash;
pub use compact_hash::CompactHash;

#[path = "config_reader.rs"]
pub mod config_reader;
pub use config_reader::ConfigReader;

#[path = "cursor_writer.rs"]
pub mod cursor_writer;
pub use cursor_writer::CursorWriter;

#[path = "decay_counter.rs"]
pub mod decay_counter;
pub use decay_counter::DecayCounter;

#[path = "diff_tracker.rs"]
pub mod diff_tracker;
pub use diff_tracker::{DiffEntry, DiffTracker};

#[path = "dispatch_table.rs"]
pub mod dispatch_table;
pub use dispatch_table::{DispatchTable, HandlerEntry};

#[path = "double_map.rs"]
pub mod double_map;
pub use double_map::DoubleMap;

#[path = "access_map.rs"]
pub mod access_map;
pub use access_map::AccessMap;

#[path = "array_stack.rs"]
pub mod array_stack;
pub use array_stack::ArrayStack;

#[path = "batch_queue.rs"]
pub mod batch_queue;
pub use batch_queue::BatchQueue;

#[path = "bitmap_index.rs"]
pub mod bitmap_index;
pub use bitmap_index::BitmapIndex;

#[path = "buffer_pool.rs"]
pub mod buffer_pool;
pub use buffer_pool::BufferPool;

#[path = "cache_policy.rs"]
pub mod cache_policy;
pub use cache_policy::{CachePolicy, PolicyEntry, PolicyKind};

#[path = "chain_buffer.rs"]
pub mod chain_buffer;
pub use chain_buffer::ChainBuffer;

#[path = "channel_router.rs"]
pub mod channel_router;
pub use channel_router::{ChannelRouter, RoutedMessage};

#[path = "circular_buffer.rs"]
pub mod circular_buffer;
pub use circular_buffer::CircularBuffer;

#[path = "color_util.rs"]
pub mod color_util;
pub use color_util::{
    clamp01, hue_rotate, is_valid_component, lerp_rgba, linear_to_srgb as cu_linear_to_srgb,
    luminance_srgb, rgb_to_u32, srgb_to_linear as cu_srgb_to_linear, u32_to_rgb,
};

#[path = "command_list.rs"]
pub mod command_list;
pub use command_list::{CmdEntry, CmdPriority, CommandList};

#[path = "compact_set.rs"]
pub mod compact_set;
pub use compact_set::CompactSet;

#[path = "config_layer.rs"]
pub mod config_layer;
pub use config_layer::{ConfigLayer, LayeredConfig};

#[path = "context_map.rs"]
pub mod context_map;
pub use context_map::{ContextMap, CtxValue};

#[path = "copy_buffer.rs"]
pub mod copy_buffer;
pub use copy_buffer::CopyBuffer;

#[path = "crc_table.rs"]
pub mod crc_table;
pub use crc_table::{crc32, crc32_match, CrcTable};

#[path = "error_log.rs"]
pub mod error_log;
pub use error_log::{
    clear_error_log, count_by_severity, entries_for_category, error_entry_count, error_log_to_json,
    has_fatal, last_error, new_error_log, push_error, total_pushed, ErrorEntry, ErrorLog,
    ErrorSeverity,
};

#[path = "event_dispatch.rs"]
pub mod event_dispatch;
pub use event_dispatch::{
    clear_all_handlers, clear_handlers, dispatch as dispatch_event,
    dispatch_count as dispatch_total_count, handler_count as dispatch_handler_count, handler_names,
    new_dispatcher, register_handler as dispatch_register_handler, registered_event_types,
    unregister_handler as dispatch_unregister_handler, DispatchRecord, EventDispatcher, HandlerId,
};

#[path = "fixed_array.rs"]
pub mod fixed_array;
pub use fixed_array::FixedArray;

#[path = "free_slot.rs"]
pub mod free_slot;
pub use free_slot::{
    alloc as fs_alloc, free as fs_free, get as fs_get, get_mut as fs_get_mut, is_occupied,
    iter_occupied, new_free_slot, slot_capacity, slot_count, FreeSlot,
};

#[path = "hash_bucket.rs"]
pub mod hash_bucket;
pub use hash_bucket::{
    hb_bucket_count, hb_clear, hb_contains, hb_count, hb_get, hb_insert, hb_keys, hb_load_factor,
    hb_remove, new_hash_bucket, BucketEntry, HashBucket,
};

#[path = "id_pool.rs"]
pub mod id_pool;
pub use id_pool::{
    id_active_count, id_alloc, id_is_active, id_peek_next, id_recycled_count, id_release,
    id_release_all, id_total, new_id_pool, Id, IdPool,
};

#[path = "index_list.rs"]
pub mod index_list;
pub use index_list::{
    il_as_slice, il_clear, il_contains, il_get, il_is_empty, il_len, il_merge, il_push, il_remove,
    il_retain, new_index_list, IndexList,
};

#[path = "interval_tree.rs"]
pub mod interval_tree;
pub use interval_tree::{
    it_clear, it_contains_id, it_count, it_insert, it_query_point, it_query_range, it_remove,
    it_to_json, new_interval_tree, Interval, IntervalTree,
};

#[path = "key_cache.rs"]
pub mod key_cache;
pub use key_cache::{
    kc_advance, kc_clear, kc_contains, kc_frame, kc_get, kc_hits, kc_insert, kc_len, kc_remove,
    new_key_cache, KeyCache, KeyCacheEntry,
};

#[path = "lazy_map.rs"]
pub mod lazy_map;
pub use lazy_map::{
    lm_clear, lm_compute_count, lm_declare, lm_get, lm_is_computed, lm_is_pending, lm_len,
    lm_pending_count, lm_pending_keys, lm_remove, lm_set, new_lazy_map, LazyMap,
};

#[path = "linked_map.rs"]
pub mod linked_map;
pub use linked_map::{
    lmap_clear, lmap_contains, lmap_get, lmap_get_at, lmap_get_mut, lmap_insert, lmap_is_empty,
    lmap_keys, lmap_len, lmap_remove, lmap_values, new_linked_map, LinkedMap,
};

#[path = "memo_table.rs"]
pub mod memo_table;
pub use memo_table::{
    memo_access_count, memo_clear, memo_contains, memo_get, memo_hit_rate, memo_hits,
    memo_invalidate, memo_len, memo_misses, memo_set, new_memo_table, MemoEntry, MemoTable,
};

#[path = "message_log.rs"]
pub mod message_log;
pub use message_log::{
    ml_by_priority, ml_by_tag, ml_clear, ml_get, ml_last, ml_len, ml_push, ml_remove_tag,
    ml_to_json, new_message_log, Message, MessageLog, MsgPriority,
};

#[path = "metric_counter.rs"]
pub mod metric_counter;
pub use metric_counter::{
    mc_count, mc_increment, mc_mean, mc_names, mc_record, mc_reset_all, mc_reset_one, mc_stats,
    mc_sum, mc_to_json, new_metric_counter, MetricCounter, MetricStats,
};

#[path = "name_table.rs"]
pub mod name_table;
pub use name_table::{
    new_name_table, nt_clear, nt_count, nt_has_id, nt_has_name, nt_id, nt_name, nt_names,
    nt_register, nt_rename, nt_unregister, NameTable,
};

#[path = "node_pool.rs"]
pub mod node_pool;
pub use node_pool::{
    new_node_pool, np_alloc, np_capacity, np_count, np_free, np_get, np_get_mut, np_is_valid,
    NodeHandle, NodePool,
};

#[path = "object_registry.rs"]
pub mod object_registry;
pub use object_registry::{
    new_object_registry, or_by_type, or_clear, or_contains, or_get, or_is_empty, or_len,
    or_register, or_remove, or_to_json, ObjectRegistry,
};

#[path = "observer_list.rs"]
pub mod observer_list;
pub use observer_list::{
    new_observer_list, ol_clear, ol_count, ol_has_label, ol_is_empty, ol_notify, ol_notify_count,
    ol_subscribe, ol_unsubscribe, ObserverList,
};

#[path = "option_cache.rs"]
pub mod option_cache;
pub use option_cache::{
    new_option_cache, oc_clear, oc_get, oc_has_key, oc_hit_rate, oc_is_empty, oc_len, oc_remove,
    oc_set_none, oc_set_some, OptionCache,
};

#[path = "output_buffer.rs"]
pub mod output_buffer;
pub use output_buffer::{
    new_output_buffer, ob_clear, ob_flush, ob_flush_count, ob_is_empty, ob_len, ob_peek,
    ob_write_bytes, ob_write_str, ob_write_u8, OutputBuffer,
};

#[path = "page_allocator.rs"]
pub mod page_allocator;
pub use page_allocator::{
    new_page_allocator, pa_alloc, pa_allocated_count, pa_free, pa_free_count, pa_page_count,
    pa_read, pa_reset, pa_total_bytes, pa_write, PageAllocator,
};

#[path = "param_set.rs"]
pub mod param_set;
pub use param_set::{
    new_param_set, ps_clear, ps_contains, ps_get_bool, ps_get_float, ps_get_int, ps_get_text,
    ps_is_empty, ps_len, ps_remove, ps_set_bool, ps_set_float, ps_set_int, ps_set_text, ParamSet,
    ParamValue,
};

#[path = "patch_buffer.rs"]
pub mod patch_buffer;
pub use patch_buffer::{
    new_patch_buffer, pb_add, pb_applied_count, pb_apply, pb_clear, pb_count, pb_is_empty,
    pb_max_offset, pb_total_bytes, Patch, PatchBuffer,
};

#[path = "path_cache.rs"]
pub mod path_cache;
pub use path_cache::{
    new_path_cache, pc_clear, pc_contains, pc_get, pc_hit_rate, pc_insert, pc_invalidate,
    pc_is_empty, pc_len, PathCache,
};

#[path = "pattern_match.rs"]
pub mod pattern_match;
pub use pattern_match::{
    count_occurrences, extract_between, glob_match, glob_match_ci, grep_lines, has_prefix,
    has_suffix, replace_all, tokenize, PatternMatcher,
};

#[path = "payload_buffer.rs"]
pub mod payload_buffer;
pub use payload_buffer::{
    new_payload_buffer, pybuf_clear, pybuf_drain, pybuf_is_empty, pybuf_is_full, pybuf_len,
    pybuf_peek, pybuf_pop, pybuf_push, pybuf_total_bytes, PayloadBuffer, PayloadEntry,
};

#[path = "peg_parser.rs"]
pub mod peg_parser;
pub use peg_parser::{
    node_text, parse_choice, parse_depth, parse_ident, parse_integer, parse_list, parse_literal,
    parse_opt, skip_whitespace, ParseNode,
};

#[path = "persistent_map.rs"]
pub mod persistent_map;
pub use persistent_map::{
    new_persistent_map, pm_clear, pm_contains, pm_get, pm_insert, pm_is_empty, pm_len, pm_remove,
    pm_restore, pm_snapshot, pm_snapshot_count, pm_version, PersistentMap,
};

#[path = "pipe_filter.rs"]
pub mod pipe_filter;
pub use pipe_filter::{
    new_pipe_filter, pf_add, pf_apply, pf_clear, pf_count, pf_is_empty, pf_passed, pf_passes,
    pf_rejected, FilterOp, PipeFilter,
};

#[path = "pipeline_context.rs"]
pub mod pipeline_context;
pub use pipeline_context::{
    ctx_add_error, ctx_add_warning, ctx_advance, ctx_get, ctx_has, ctx_has_errors, ctx_is_done,
    ctx_mark_done, ctx_reset, ctx_set, ctx_stage, new_pipeline_context, PipelineContext,
};

#[path = "placeholder_map.rs"]
pub mod placeholder_map;
pub use placeholder_map::{
    new_placeholder_map, plm_clear, plm_contains, plm_get, plm_is_empty, plm_len, plm_remove,
    plm_render, plm_set, plm_substituted, PlaceholderMap,
};

#[path = "plan_executor.rs"]
pub mod plan_executor;
pub use plan_executor::{
    new_plan_executor, pe_add_step, pe_complete, pe_done_count, pe_fail, pe_failed_count,
    pe_is_aborted, pe_is_complete, pe_reset, pe_skip, pe_step_count, pe_total_ms, PlanExecutor,
    PlanStep, StepState,
};

#[path = "pool_allocator.rs"]
pub mod pool_allocator;
pub use pool_allocator::{new_pool, PoolAllocator, PoolHandle, PoolSlot};

#[path = "priority_map.rs"]
pub mod priority_map;
pub use priority_map::{
    clear_priority_map, get_highest, has_key_pm, insert_priority, new_priority_map, priority_count,
    priority_to_vec, remove_highest, PriorityMap,
};

#[path = "proc_context.rs"]
pub mod proc_context;
pub use proc_context::{new_proc_context, CtxVal as ProcCtxVal, ProcContext};

#[path = "query_cache.rs"]
pub mod query_cache;
pub use query_cache::{new_query_cache, QueryCache, QueryEntry};

#[path = "radix_sort.rs"]
pub mod radix_sort;
pub use radix_sort::{
    count_distinct_u32, is_sorted_u32, is_sorted_u64, radix_sort_pairs_u32, radix_sort_u32,
    radix_sort_u64,
};

#[path = "range_map.rs"]
pub mod range_map;
pub use range_map::{new_range_map, RangeEntry, RangeMap};

#[path = "ref_counted.rs"]
pub mod ref_counted;
pub use ref_counted::{new_ref_counted, RefCounted, RefEntry};

#[path = "registry_map.rs"]
pub mod registry_map;
pub use registry_map::{new_registry_map, RegistryItem, RegistryMap};

#[path = "resource_pool.rs"]
pub mod resource_pool;
pub use resource_pool::{
    new_resource_pool, ResourcePool, ResourceSlot, ResourceState as PoolResourceState,
};

#[path = "result_stack.rs"]
pub mod result_stack;
pub use result_stack::{new_result_stack, ResultEntry, ResultKind, ResultStack};

#[path = "retry_policy.rs"]
pub mod retry_policy;
pub use retry_policy::{
    max_retries, new_retry_policy, reset_retry, retry_count, retry_delay_ms, retry_exhausted,
    retry_with_backoff, should_retry, RetryPolicy, RetryResult,
};

#[path = "ring_log.rs"]
pub mod ring_log;
pub use ring_log::{new_ring_log, RingLog, RingLogEntry, RingLogLevel};

#[path = "role_map.rs"]
pub mod role_map;
pub use role_map::{new_role_map, RoleMap};

#[path = "route_table.rs"]
pub mod route_table;
pub use route_table::{new_route_table, RouteEntry as RouteTableEntry, RouteMatch, RouteTable};

#[path = "rule_engine.rs"]
pub mod rule_engine;
pub use rule_engine::{make_rule, new_rule_engine, Condition, Rule, RuleAction, RuleEngine};

#[path = "schedule_queue.rs"]
pub mod schedule_queue;
pub use schedule_queue::{new_schedule_queue, ScheduleQueue, ScheduleTask};

#[path = "search_index.rs"]
pub mod search_index;
pub use search_index::{new_search_index, SearchDoc, SearchIndex};

#[path = "segment_tree.rs"]
pub mod segment_tree;
pub use segment_tree::{
    build_segment_tree, seg_get, seg_query, seg_total, seg_update, SegmentTree,
};

#[path = "selector_map.rs"]
pub mod selector_map;
pub use selector_map::{new_selector_map, SelectorMap};

#[path = "semaphore_pool.rs"]
pub mod semaphore_pool;
pub use semaphore_pool::{new_semaphore_pool, Semaphore, SemaphorePool};

#[path = "sequence_map.rs"]
pub mod sequence_map;
pub use sequence_map::{new_sequence_map, SeqEntry, SequenceMap};

#[path = "service_locator.rs"]
pub mod service_locator;
pub use service_locator::{new_service_locator, ServiceDescriptor, ServiceLocator};

#[path = "session_store.rs"]
pub mod session_store;
pub use session_store::{new_session_store, Session, SessionStore};

#[path = "set_trie.rs"]
pub mod set_trie;
pub use set_trie::{new_set_trie, SetTrie};

#[path = "signal_handler.rs"]
pub mod signal_handler;
pub use signal_handler::{new_signal_handler, HandlerEntry as SignalHandlerEntry, SignalHandler};

#[path = "simple_graph.rs"]
pub mod simple_graph;
pub use simple_graph::{new_simple_graph, GraphEdge, SimpleGraph};

#[path = "size_cache.rs"]
pub mod size_cache;
pub use size_cache::{new_size_cache, SizeCache, SizeCacheEntry};

#[path = "skip_list.rs"]
pub mod skip_list;
pub use skip_list::{
    new_skip_list, skip_find, skip_insert, skip_len, skip_range, skip_remove, SkipEntry, SkipList,
};

#[path = "sliding_window.rs"]
pub mod sliding_window;
pub use sliding_window::{new_sliding_window, SlidingWindow};

#[path = "sort_key.rs"]
pub mod sort_key;
pub use sort_key::{new_sort_key, SortCriterion, SortDir, SortKey};

#[path = "source_map.rs"]
pub mod source_map;
pub use source_map::{new_source_map, SourceMap, SourceMapping};

#[path = "span_tracker.rs"]
pub mod span_tracker;
pub use span_tracker::{new_span_tracker, SpanRecord, SpanTracker};

#[path = "sparse_array.rs"]
pub mod sparse_array;
pub use sparse_array::{
    new_sparse_array, sparse_clear, sparse_count, sparse_get, sparse_has, sparse_keys,
    sparse_remove, sparse_set_val, SparseArray,
};

#[path = "state_bag.rs"]
pub mod state_bag;
pub use state_bag::{
    new_state_bag, sb_clear, sb_get, sb_len, sb_remove, sb_set, BagValue, StateBag,
};

#[path = "state_machine_v2.rs"]
pub mod state_machine_v2;
pub use state_machine_v2::{
    new_state_machine, sm_add_state, sm_add_transition, sm_current, sm_fire,
    GuardFn as StateMachineGuardFn, StateMachineV2, Transition as StateMachineTransition,
};

#[path = "static_vec.rs"]
pub mod static_vec;
pub use static_vec::{new_static_vec, StaticVec};

#[path = "storage_backend.rs"]
pub mod storage_backend;
pub use storage_backend::{
    new_storage_backend, sb_contains as storage_contains, sb_get as storage_get, sb_put,
    sb_remove as storage_remove, Bucket, StorageBackend,
};

#[path = "stream_parser.rs"]
pub mod stream_parser;
pub use stream_parser::{
    new_stream_parser, sp_feed, sp_read_u32_le, sp_read_u8, ParseResult, StreamParser,
};

#[path = "string_set.rs"]
pub mod string_set;
pub use string_set::{
    new_string_set, ss_contains, ss_insert, ss_len, ss_remove, ss_to_vec, StringSet,
};

#[path = "struct_map.rs"]
pub mod struct_map;
pub use struct_map::{new_struct_map, stm_contains, stm_get, stm_set, FieldVal, StructMap};

#[path = "sub_task.rs"]
pub mod sub_task;
pub use sub_task::{
    new_sub_task_set, sts_add, sts_done, sts_failed, sts_overall, SubTask, SubTaskSet,
    SubTaskStatus,
};

#[path = "symbol_table.rs"]
pub mod symbol_table;
pub use symbol_table::{
    new_symbol_table, sym_find, sym_intern, sym_len, sym_lookup, SymbolId, SymbolTable,
};

#[path = "sync_barrier.rs"]
pub mod sync_barrier;
pub use sync_barrier::{
    barrier_arrive, barrier_is_released, barrier_register, barrier_reset, new_sync_barrier,
    BarrierState, SyncBarrier,
};

#[path = "tag_filter.rs"]
pub mod tag_filter;
pub use tag_filter::{new_tag_filter, tf_exclude, tf_matches, tf_require, TagFilter};

#[path = "text_buffer.rs"]
pub mod text_buffer;
pub use text_buffer::{
    new_text_buffer, tb_append, tb_append_line, tb_as_str, tb_clear, tb_find, tb_line_count,
    TextBuffer,
};

#[path = "thread_local_pool.rs"]
pub mod thread_local_pool;
pub use thread_local_pool::{new_thread_local_pool, ThreadLocalPool};

#[path = "time_source.rs"]
pub mod time_source;
pub use time_source::{
    current_time_ms, elapsed_since, new_time_source, time_diff_ms, time_source_reset,
    timestamp_add_ms, timestamp_is_after, timestamp_to_string, TimeSource, Timestamp,
};

#[path = "token_stream.rs"]
pub mod token_stream;
pub use token_stream::{
    new_token_stream, tks_drain, tks_is_empty, tks_next, tks_peek, tks_push, tks_remaining,
    tks_rewind, tks_skip_while, tks_total, Token, TokenKind, TokenStream,
};

#[path = "topo_map.rs"]
pub mod topo_map;
pub use topo_map::{
    new_topo_map, tm_add_edge, tm_add_node, tm_clear, tm_has_node, tm_label, tm_node_count,
    tm_remove_node, tm_topo_sort, TopoMap, TopoNode,
};

#[path = "trace_buffer.rs"]
pub mod trace_buffer;
pub use trace_buffer::{
    new_trace_buffer, tb_avg_duration_us, tb_by_tag, tb_clear as trace_clear, tb_get as trace_get,
    tb_is_empty as trace_is_empty, tb_len as trace_len, tb_max_event, tb_record as trace_record,
    tb_tick as trace_tick, TraceBuffer, TraceEvent,
};

#[path = "transform_pipe.rs"]
pub mod transform_pipe;
pub use transform_pipe::{
    new_transform_pipe, tp_add, tp_apply, tp_clear, tp_get, tp_is_empty, tp_len, tp_pop,
    TransformKind, TransformPipe, TransformStage,
};
