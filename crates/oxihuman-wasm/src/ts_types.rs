// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! TypeScript type declarations and documentation for the OxiHuman WASM API.
//!
//! This module does **not** contain runnable Rust logic.  Its purpose is to
//! inject supplemental TypeScript `interface` definitions into the `.d.ts`
//! file emitted by `wasm-bindgen`, and to provide rich doc-comments that
//! surface in IDE tooling.
//!
//! The two main mechanisms used here are:
//!
//! 1. `#[wasm_bindgen(typescript_custom_section)]` — injects raw TypeScript
//!    text verbatim into the generated `.d.ts` file.
//! 2. `#[wasm_bindgen(typescript_type = "...")]` on `extern "C"` blocks —
//!    allows Rust functions to accept or return opaque TS types.
//!
//! # Binary Mesh Format (`MeshBytes`)
//!
//! The raw bytes returned by `OxiHumanEngine.build_mesh_bytes()` follow a
//! fixed binary layout called **OXIM v1**:
//!
//! | Offset | Type     | Field          | Description                      |
//! |--------|----------|----------------|----------------------------------|
//! | 0      | u8\[4\]  | magic          | ASCII `"OXIM"` (0x4F 0x58 0x49 0x4D) |
//! | 4      | u32 LE   | version        | Format version (currently `1`)   |
//! | 8      | u32 LE   | vertex_count N | Number of vertices               |
//! | 12     | u32 LE   | index_count  M | Number of indices (= faces × 3)  |
//! | 16     | f32\[N×3\] | positions    | XYZ positions, interleaved       |
//! | …      | f32\[N×3\] | normals      | XYZ normals, interleaved         |
//! | …      | f32\[N×2\] | uvs          | UV coordinates, interleaved      |
//! | …      | u32\[M\]   | indices      | Triangle index list              |
//!
//! > **Note:** The actual bytes returned by `build_mesh_bytes()` use the
//! > legacy header format (`version` at offset 0, **not** `"OXIM"` magic).
//! > The magic-prefixed format is documented here as the canonical target
//! > for future format version 2.  Consumers should check
//! > `BUFFER_FORMAT_VERSION` at runtime.

#[cfg(feature = "bindgen")]
mod ts_impl {
    use wasm_bindgen::prelude::*;

    // -----------------------------------------------------------------------
    // Custom TypeScript section: OxiHumanParams interface
    // -----------------------------------------------------------------------

    /// Custom TypeScript section injected into the generated `.d.ts` file.
    ///
    /// Defines the `OxiHumanParams` interface describing the serialised form
    /// of [`crate::engine::WasmEngine`]'s parameter state as produced by
    /// `OxiHumanEngine.export_params_json()`.
    #[wasm_bindgen(typescript_custom_section)]
    const TS_OXIHUMAN_PARAMS: &'static str = r#"
/**
 * Serialised form of the engine's morphing parameter state.
 *
 * All numeric fields are normalised to `[0.0, 1.0]` unless noted otherwise.
 *
 * Produced by `OxiHumanEngine.export_params_json()` and consumed by
 * `OxiHumanEngine.import_params_json()`.
 */
export interface OxiHumanParams {
  /** Body height (0 = minimum, 1 = maximum). */
  height: number;
  /** Body mass / adipose level (0 = lean, 1 = heavy). */
  weight: number;
  /** Muscle definition (0 = no muscle tone, 1 = very muscular). */
  muscle: number;
  /** Apparent age (0 = youth, 1 = elderly). */
  age: number;
  /**
   * Arbitrary extra morph parameters keyed by morph-target name.
   *
   * Each value drives the blend weight for the morph target of the same name,
   * in `[0.0, 1.0]`.
   */
  extra: Record<string, number>;
}
"#;

    // -----------------------------------------------------------------------
    // Custom TypeScript section: MeshBytes binary format interface
    // -----------------------------------------------------------------------

    /// Custom TypeScript section describing the binary mesh format returned by
    /// `OxiHumanEngine.build_mesh_bytes()`.
    ///
    /// This is injected as documentation-only into the `.d.ts` file so that
    /// TypeScript consumers know how to parse the `Uint8Array`.
    #[wasm_bindgen(typescript_custom_section)]
    const TS_MESH_BYTES: &'static str = r#"
/**
 * Parsed representation of the binary mesh buffer returned by
 * `OxiHumanEngine.build_mesh_bytes()`.
 *
 * ## Wire format (OXIM v1 / legacy v1)
 *
 * The raw `Uint8Array` is laid out as follows (all multi-byte integers are
 * **little-endian**):
 *
 * | Offset (bytes) | Type        | Field          |
 * |---------------|-------------|----------------|
 * | 0             | u32         | version        |
 * | 4             | u32         | vertex_count N |
 * | 8             | u32         | index_count  M |
 * | 12            | f32[N × 3]  | positions XYZ  |
 * | 12 + N×12     | f32[N × 3]  | normals XYZ    |
 * | 12 + N×24     | f32[N × 2]  | uvs UV         |
 * | 12 + N×32     | u32[M]      | indices        |
 *
 * ## Parsing example
 * ```typescript
 * function parseMeshBytes(buf: Uint8Array): MeshBytes {
 *   const view = new DataView(buf.buffer, buf.byteOffset, buf.byteLength);
 *   let off = 0;
 *   const version      = view.getUint32(off, true); off += 4;
 *   const vertexCount  = view.getUint32(off, true); off += 4;
 *   const indexCount   = view.getUint32(off, true); off += 4;
 *
 *   const positions = new Float32Array(buf.buffer, buf.byteOffset + off, vertexCount * 3);
 *   off += vertexCount * 3 * 4;
 *   const normals   = new Float32Array(buf.buffer, buf.byteOffset + off, vertexCount * 3);
 *   off += vertexCount * 3 * 4;
 *   const uvs       = new Float32Array(buf.buffer, buf.byteOffset + off, vertexCount * 2);
 *   off += vertexCount * 2 * 4;
 *   const indices   = new Uint32Array(buf.buffer, buf.byteOffset + off, indexCount);
 *
 *   return { version, vertexCount, indexCount, positions, normals, uvs, indices };
 * }
 * ```
 */
export interface MeshBytes {
  /** Format version (currently `1`). */
  version: number;
  /** Number of vertices N. */
  vertexCount: number;
  /** Number of indices M (= triangles × 3). */
  indexCount: number;
  /** Flattened XYZ positions: length = N × 3. */
  positions: Float32Array;
  /** Flattened XYZ normals: length = N × 3. */
  normals: Float32Array;
  /** Flattened UV coordinates: length = N × 2. */
  uvs: Float32Array;
  /** Triangle indices: length = M. */
  indices: Uint32Array;
}
"#;

    // -----------------------------------------------------------------------
    // Custom TypeScript section: ServiceWorkerConfig interface
    // -----------------------------------------------------------------------

    /// Custom TypeScript section describing the service-worker configuration
    /// object that can be passed to the generated `generateSwJs` helper.
    #[wasm_bindgen(typescript_custom_section)]
    const TS_SW_CONFIG: &'static str = r#"
/**
 * Configuration for the OxiHuman offline service worker.
 *
 * Pass to `generateSwJs()` or `generateCacheManifestJson()` to produce the
 * service-worker JavaScript and cache-manifest JSON respectively.
 */
export interface OxiHumanSwConfig {
  /** Name of the Cache Storage bucket, e.g. `"oxihuman-v1"`. */
  cacheName: string;
  /** List of asset URLs to pre-cache on install. */
  assetUrls: string[];
  /** Maximum total cache size in megabytes (soft limit). */
  maxCacheSizeMb: number;
  /**
   * Caching strategy:
   * - `"CacheFirst"` — serve from cache; fall back to network.
   * - `"NetworkFirst"` — try network first; fall back to cache.
   * - `"StaleWhileRevalidate"` — serve from cache immediately, then refresh.
   */
  cacheStrategy: "CacheFirst" | "NetworkFirst" | "StaleWhileRevalidate";
}

/**
 * A single entry in the cache manifest produced by
 * `generateCacheManifestJson()`.
 */
export interface OxiHumanCacheEntry {
  /** Full URL of the cached asset. */
  url: string;
  /** SHA-256 hex digest of the asset content at cache time. */
  sha256: string;
  /** Asset size in bytes. */
  sizeBytes: number;
  /** Unix timestamp (seconds) when the entry was last fetched. */
  lastFetchedUnix: number;
  /** Time-to-live in seconds; `0` means no expiry. */
  ttlSecs: number;
}

/**
 * Generate the text of a service-worker JavaScript file from the given
 * configuration.  Write the returned string to `service-worker.js` in your
 * web root.
 *
 * The generated script implements:
 * - An `install` event handler that pre-caches all `assetUrls`.
 * - An `activate` event handler that deletes stale caches.
 * - A `fetch` event handler implementing the chosen `cacheStrategy`.
 */
export function generateSwJs(config: OxiHumanSwConfig): string;

/**
 * Generate a JSON cache-manifest string for the given entries.
 *
 * The manifest can be fetched by the service worker at runtime to verify
 * asset integrity via the embedded `sha256` digests.
 */
export function generateCacheManifestJson(
  config: OxiHumanSwConfig,
  entries: OxiHumanCacheEntry[],
): string;
"#;

    // -----------------------------------------------------------------------
    // Custom TypeScript section: AnimFrame interface
    // -----------------------------------------------------------------------

    /// Custom TypeScript section for the animation frame format produced by
    /// `OxiHumanAnimPlayer.export_anim_json()`.
    #[wasm_bindgen(typescript_custom_section)]
    const TS_ANIM_FRAME: &'static str = r#"
/**
 * A single keyframe snapshot as produced by
 * `OxiHumanAnimPlayer.export_anim_json()`.
 *
 * Keys are param names (`"height"`, `"weight"`, `"muscle"`, `"age"`, and any
 * extra morph-target names); values are normalised floats in `[0.0, 1.0]`.
 */
export type OxiHumanAnimFrame = Record<string, number>;
"#;

    // -----------------------------------------------------------------------
    // Wasm-bindgen JS/TS shim stubs for generateSwJs / generateCacheManifestJson
    // -----------------------------------------------------------------------
    // These two free functions are the Rust implementations re-exported to JS
    // via wasm-bindgen.  The `#[wasm_bindgen]` attributes here provide JS
    // entry points; the actual logic lives in `service_worker.rs`.

    /// Generate service-worker JavaScript from a JSON configuration object.
    ///
    /// `config_json` must be a JSON string matching the `OxiHumanSwConfig`
    /// TypeScript interface defined in this module.
    ///
    /// Returns the service-worker JavaScript as a string, ready to be written
    /// to `service-worker.js` in your web root.
    ///
    /// Throws a JavaScript `Error` if `config_json` is malformed.
    #[wasm_bindgen(js_name = "generateSwJs")]
    pub fn generate_sw_js_from_json(config_json: &str) -> Result<String, JsError> {
        let config = parse_sw_config_json(config_json).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(crate::service_worker::generate_sw_js(&config))
    }

    /// Generate a cache-manifest JSON string.
    ///
    /// `config_json` must match `OxiHumanSwConfig`; `entries_json` must be a
    /// JSON array of `OxiHumanCacheEntry` objects.
    ///
    /// Throws a JavaScript `Error` if either argument is malformed.
    #[wasm_bindgen(js_name = "generateCacheManifestJson")]
    pub fn generate_cache_manifest_json_from_json(
        config_json: &str,
        entries_json: &str,
    ) -> Result<String, JsError> {
        use crate::service_worker::CacheEntry;

        let config = parse_sw_config_json(config_json).map_err(|e| JsError::new(&e.to_string()))?;
        let entries: Vec<CacheEntry> =
            serde_json::from_str(entries_json).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(crate::service_worker::generate_cache_manifest_json(
            &config, &entries,
        ))
    }

    // -----------------------------------------------------------------------
    // Internal helper: deserialise OxiHumanSwConfig from JSON
    // -----------------------------------------------------------------------

    fn parse_sw_config_json(
        json: &str,
    ) -> Result<crate::service_worker::ServiceWorkerConfig, anyhow::Error> {
        use crate::service_worker::{CacheStrategy, ServiceWorkerConfig};

        let v: serde_json::Value = serde_json::from_str(json)?;

        let cache_name = v["cacheName"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("missing cacheName"))?
            .to_string();

        let asset_urls: Vec<String> = v["assetUrls"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("missing assetUrls"))?
            .iter()
            .filter_map(|u| u.as_str().map(|s| s.to_string()))
            .collect();

        let max_cache_size_mb = v["maxCacheSizeMb"].as_f64().unwrap_or(50.0);

        let strategy_str = v["cacheStrategy"].as_str().unwrap_or("CacheFirst");
        let cache_strategy = match strategy_str {
            "NetworkFirst" => CacheStrategy::NetworkFirst,
            "StaleWhileRevalidate" => CacheStrategy::StaleWhileRevalidate,
            _ => CacheStrategy::CacheFirst,
        };

        Ok(ServiceWorkerConfig {
            cache_name,
            asset_urls,
            max_cache_size_mb,
            cache_strategy,
        })
    }
}

#[cfg(feature = "bindgen")]
pub use ts_impl::*;
