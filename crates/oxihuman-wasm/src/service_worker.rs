// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Offline asset caching strategy and service-worker code generation.
//!
//! This module is pure Rust — it generates JavaScript and JSON *as strings*.
//! No browser APIs or JavaScript glue are required to use it.
//!
//! # Overview
//!
//! 1. Build a [`ServiceWorkerConfig`] describing which assets to cache and
//!    which [`CacheStrategy`] to apply.
//! 2. Call [`generate_sw_js`] to obtain the text of a `service-worker.js`
//!    file ready to deploy to your web root.
//! 3. Optionally call [`generate_cache_manifest_json`] to produce a JSON
//!    manifest that the service worker can fetch to verify asset integrity.
//!
//! # Example
//! ```rust
//! use oxihuman_wasm::service_worker::{
//!     CacheEntry, CacheStrategy, ServiceWorkerConfig, generate_sw_js,
//!     generate_cache_manifest_json,
//! };
//!
//! let config = ServiceWorkerConfig {
//!     cache_name: "oxihuman-v1".to_string(),
//!     asset_urls: vec![
//!         "/oxihuman_wasm.js".to_string(),
//!         "/oxihuman_wasm_bg.wasm".to_string(),
//!         "/index.html".to_string(),
//!     ],
//!     max_cache_size_mb: 50.0,
//!     cache_strategy: CacheStrategy::CacheFirst,
//! };
//!
//! let sw_js  = generate_sw_js(&config);
//! let entries: Vec<CacheEntry> = vec![];
//! let manifest = generate_cache_manifest_json(&config, &entries);
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// -----------------------------------------------------------------------
// CacheStrategy
// -----------------------------------------------------------------------

/// The fetch-interception strategy used by the generated service worker.
///
/// | Strategy                  | Network cost  | Freshness guarantee |
/// |---------------------------|---------------|---------------------|
/// | [`CacheFirst`]            | Low           | Stale until evicted |
/// | [`NetworkFirst`]          | High          | Always up-to-date   |
/// | [`StaleWhileRevalidate`]  | Medium        | Slightly stale      |
///
/// [`CacheFirst`]:           CacheStrategy::CacheFirst
/// [`NetworkFirst`]:         CacheStrategy::NetworkFirst
/// [`StaleWhileRevalidate`]: CacheStrategy::StaleWhileRevalidate
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheStrategy {
    /// Serve from cache when available; only fetch from network on a cache miss.
    ///
    /// Best for: static assets (WASM, JS bundles, fonts) that change rarely.
    CacheFirst,

    /// Always attempt a network fetch; fall back to cache only on failure.
    ///
    /// Best for: dynamic content that must be fresh (API responses, JSON data).
    NetworkFirst,

    /// Serve from cache immediately, then fetch from the network in the
    /// background and update the cache entry for the next request.
    ///
    /// Best for: content where slight staleness is acceptable but performance
    /// matters (HTML shells, CSS).
    StaleWhileRevalidate,
}

impl CacheStrategy {
    /// Return `true` if the cached entry should be used for the current request.
    ///
    /// - `CacheFirst` / `StaleWhileRevalidate`: use cache if the entry has not
    ///   expired (`now < last_fetched + ttl`, or `ttl == 0` meaning no expiry).
    /// - `NetworkFirst`: never short-circuit to cache (always try network first).
    pub fn should_use_cache(&self, entry: &CacheEntry, now: u64) -> bool {
        match self {
            CacheStrategy::NetworkFirst => false,
            CacheStrategy::CacheFirst | CacheStrategy::StaleWhileRevalidate => {
                if entry.ttl_secs == 0 {
                    true // no expiry
                } else {
                    now < entry.last_fetched_unix.saturating_add(entry.ttl_secs)
                }
            }
        }
    }

    /// Return the JavaScript strategy name used in the generated SW script.
    fn js_name(&self) -> &'static str {
        match self {
            CacheStrategy::CacheFirst => "cache-first",
            CacheStrategy::NetworkFirst => "network-first",
            CacheStrategy::StaleWhileRevalidate => "stale-while-revalidate",
        }
    }
}

// -----------------------------------------------------------------------
// ServiceWorkerConfig
// -----------------------------------------------------------------------

/// Configuration controlling the generated service worker and cache manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceWorkerConfig {
    /// Name of the Cache Storage bucket (e.g. `"oxihuman-v1"`).
    ///
    /// The activate handler deletes all caches whose names do **not** match
    /// this value, implementing a rolling-cache upgrade pattern.
    pub cache_name: String,

    /// List of asset URLs to pre-cache during the service-worker `install`
    /// event.  Relative URLs are resolved against the service-worker scope.
    pub asset_urls: Vec<String>,

    /// Soft upper bound on the total cache size in megabytes.
    ///
    /// The generated service worker does not enforce this limit automatically;
    /// it is embedded in the script as a comment / constant so that custom
    /// trimming logic can consume it.
    pub max_cache_size_mb: f64,

    /// Fetch-interception strategy applied to all requests within the scope.
    pub cache_strategy: CacheStrategy,
}

// -----------------------------------------------------------------------
// CacheEntry
// -----------------------------------------------------------------------

/// A single entry in a [`CacheManifest`].
///
/// Entries are keyed by URL and carry enough metadata for a service worker to
/// validate freshness and integrity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Absolute or relative URL of the cached asset.
    pub url: String,

    /// SHA-256 hex digest of the response body at fetch time.
    ///
    /// An empty string means no integrity check is available.
    pub sha256: String,

    /// Response body size in bytes.
    pub size_bytes: u64,

    /// Unix timestamp (seconds since epoch) when the entry was cached.
    pub last_fetched_unix: u64,

    /// Time-to-live in seconds.  `0` means the entry never expires.
    pub ttl_secs: u64,
}

// -----------------------------------------------------------------------
// CacheManifest
// -----------------------------------------------------------------------

/// A snapshot of the cache at a point in time.
///
/// Can be serialised to JSON via [`generate_cache_manifest_json`] and fetched
/// by the service worker to implement integrity checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheManifest {
    /// Cache entries indexed by URL.
    pub entries: HashMap<String, CacheEntry>,

    /// Unix timestamp (seconds) when this manifest was generated.
    pub generated_at: u64,

    /// Manifest schema version (currently `1`).
    pub version: u32,
}

impl CacheManifest {
    /// Create an empty manifest with the current timestamp placeholder `0`.
    pub fn new() -> Self {
        CacheManifest {
            entries: HashMap::new(),
            generated_at: 0,
            version: 1,
        }
    }

    /// Insert or replace an entry.
    pub fn insert(&mut self, entry: CacheEntry) {
        self.entries.insert(entry.url.clone(), entry);
    }
}

impl Default for CacheManifest {
    fn default() -> Self {
        Self::new()
    }
}

// -----------------------------------------------------------------------
// generate_sw_js
// -----------------------------------------------------------------------

/// Generate the text content of a `service-worker.js` file.
///
/// The generated script:
/// - Caches all [`ServiceWorkerConfig::asset_urls`] during `install`.
/// - Deletes stale caches (any cache not named
///   [`ServiceWorkerConfig::cache_name`]) during `activate`.
/// - Intercepts `fetch` events and applies the chosen
///   [`ServiceWorkerConfig::cache_strategy`].
///
/// Write the returned string to `service-worker.js` in your web root and
/// register it from your HTML:
/// ```html
/// <script>
///   navigator.serviceWorker.register('/service-worker.js');
/// </script>
/// ```
pub fn generate_sw_js(config: &ServiceWorkerConfig) -> String {
    let asset_urls_js = build_js_string_array(&config.asset_urls);
    let cache_name = js_string_literal(&config.cache_name);
    let strategy = config.cache_strategy.js_name();
    let max_mb = config.max_cache_size_mb;
    let fetch_handler = build_fetch_handler(&config.cache_strategy, &config.cache_name);

    format!(
        r#"// OxiHuman Service Worker — generated by oxihuman-wasm
// Cache strategy : {strategy}
// Max cache size : {max_mb} MB
// DO NOT EDIT — regenerate via oxihuman-wasm ServiceWorkerConfig

'use strict';

const CACHE_NAME   = {cache_name};
const ASSET_URLS   = {asset_urls_js};
const MAX_CACHE_MB = {max_mb};

// ── Install ──────────────────────────────────────────────────────────────────
self.addEventListener('install', (event) => {{
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => {{
      return cache.addAll(ASSET_URLS);
    }}).then(() => {{
      return self.skipWaiting();
    }})
  );
}});

// ── Activate ─────────────────────────────────────────────────────────────────
self.addEventListener('activate', (event) => {{
  event.waitUntil(
    caches.keys().then((cacheNames) => {{
      return Promise.all(
        cacheNames
          .filter((name) => name !== CACHE_NAME)
          .map((name) => caches.delete(name))
      );
    }}).then(() => {{
      return self.clients.claim();
    }})
  );
}});

// ── Fetch ─────────────────────────────────────────────────────────────────────
{fetch_handler}
"#
    )
}

/// Build the JavaScript `fetch` event listener body for the chosen strategy.
fn build_fetch_handler(strategy: &CacheStrategy, cache_name: &str) -> String {
    let cache_name_js = js_string_literal(cache_name);
    match strategy {
        CacheStrategy::CacheFirst => format!(
            r#"// Strategy: Cache First
self.addEventListener('fetch', (event) => {{
  event.respondWith(
    caches.match(event.request).then((cached) => {{
      if (cached) {{
        return cached;
      }}
      return fetch(event.request).then((response) => {{
        if (!response || response.status !== 200 || response.type === 'opaque') {{
          return response;
        }}
        const toCache = response.clone();
        caches.open({cache_name_js}).then((cache) => {{
          cache.put(event.request, toCache);
        }});
        return response;
      }}).catch(() => {{
        // Offline fallback: return whatever we have in cache.
        return caches.match('/index.html');
      }});
    }})
  );
}});
"#
        ),

        CacheStrategy::NetworkFirst => format!(
            r#"// Strategy: Network First
self.addEventListener('fetch', (event) => {{
  event.respondWith(
    fetch(event.request).then((response) => {{
      if (!response || response.status !== 200 || response.type === 'opaque') {{
        return response;
      }}
      const toCache = response.clone();
      caches.open({cache_name_js}).then((cache) => {{
        cache.put(event.request, toCache);
      }});
      return response;
    }}).catch(() => {{
      return caches.match(event.request).then((cached) => {{
        return cached || caches.match('/index.html');
      }});
    }})
  );
}});
"#
        ),

        CacheStrategy::StaleWhileRevalidate => format!(
            r#"// Strategy: Stale While Revalidate
self.addEventListener('fetch', (event) => {{
  event.respondWith(
    caches.open({cache_name_js}).then((cache) => {{
      return cache.match(event.request).then((cached) => {{
        const networkFetch = fetch(event.request).then((response) => {{
          if (response && response.status === 200 && response.type !== 'opaque') {{
            cache.put(event.request, response.clone());
          }}
          return response;
        }});
        // Serve stale immediately; background-refresh for next time.
        return cached || networkFetch;
      }});
    }})
  );
}});
"#
        ),
    }
}

// -----------------------------------------------------------------------
// generate_cache_manifest_json
// -----------------------------------------------------------------------

/// Generate a JSON cache-manifest string from a config and a slice of entries.
///
/// The resulting JSON object has the shape:
/// ```json
/// {
///   "version": 1,
///   "cache_name": "oxihuman-v1",
///   "generated_at": 0,
///   "entries": {
///     "/index.html": { "url": "/index.html", "sha256": "...", ... }
///   }
/// }
/// ```
///
/// The `generated_at` field is always `0` when generated from Rust without
/// access to a real clock.  The service worker should substitute the actual
/// timestamp at runtime if needed.
pub fn generate_cache_manifest_json(
    config: &ServiceWorkerConfig,
    entries: &[CacheEntry],
) -> String {
    let mut manifest = CacheManifest::new();
    for entry in entries {
        manifest.insert(entry.clone());
    }

    // Serialize to a JSON Value so we can add cache_name at the top level.
    let mut map = serde_json::Map::new();
    map.insert("version".to_string(), serde_json::json!(manifest.version));
    map.insert(
        "cache_name".to_string(),
        serde_json::json!(config.cache_name),
    );
    map.insert(
        "generated_at".to_string(),
        serde_json::json!(manifest.generated_at),
    );
    map.insert(
        "entries".to_string(),
        serde_json::to_value(&manifest.entries)
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
    );

    serde_json::to_string_pretty(&serde_json::Value::Object(map))
        .unwrap_or_else(|_| r#"{"error":"serialization failed"}"#.to_string())
}

// -----------------------------------------------------------------------
// Private helpers
// -----------------------------------------------------------------------

/// Format a `Vec<String>` as a JavaScript array literal.
fn build_js_string_array(urls: &[String]) -> String {
    let items: Vec<String> = urls.iter().map(|u| js_string_literal(u)).collect();
    format!("[\n  {}\n]", items.join(",\n  "))
}

/// Wrap a string in JavaScript single-quoted string literal, escaping `'` and `\`.
fn js_string_literal(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('\'', "\\'");
    format!("'{escaped}'")
}

// -----------------------------------------------------------------------
// Unit tests
// -----------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config(strategy: CacheStrategy) -> ServiceWorkerConfig {
        ServiceWorkerConfig {
            cache_name: "test-cache-v1".to_string(),
            asset_urls: vec![
                "/index.html".to_string(),
                "/app.wasm".to_string(),
                "/app.js".to_string(),
            ],
            max_cache_size_mb: 50.0,
            cache_strategy: strategy,
        }
    }

    fn sample_entry(url: &str) -> CacheEntry {
        CacheEntry {
            url: url.to_string(),
            sha256: "abc123".to_string(),
            size_bytes: 1024,
            last_fetched_unix: 1_700_000_000,
            ttl_secs: 3600,
        }
    }

    // -- generate_sw_js --

    #[test]
    fn sw_js_contains_cache_name() {
        let cfg = sample_config(CacheStrategy::CacheFirst);
        let js = generate_sw_js(&cfg);
        assert!(js.contains("test-cache-v1"), "SW JS missing cache name");
    }

    #[test]
    fn sw_js_contains_all_asset_urls() {
        let cfg = sample_config(CacheStrategy::CacheFirst);
        let js = generate_sw_js(&cfg);
        assert!(js.contains("/index.html"), "missing index.html");
        assert!(js.contains("/app.wasm"), "missing app.wasm");
        assert!(js.contains("/app.js"), "missing app.js");
    }

    #[test]
    fn sw_js_install_handler_present() {
        let cfg = sample_config(CacheStrategy::CacheFirst);
        let js = generate_sw_js(&cfg);
        assert!(
            js.contains("addEventListener('install'"),
            "missing install handler"
        );
    }

    #[test]
    fn sw_js_activate_handler_present() {
        let cfg = sample_config(CacheStrategy::NetworkFirst);
        let js = generate_sw_js(&cfg);
        assert!(
            js.contains("addEventListener('activate'"),
            "missing activate handler"
        );
    }

    #[test]
    fn sw_js_fetch_handler_present() {
        let cfg = sample_config(CacheStrategy::StaleWhileRevalidate);
        let js = generate_sw_js(&cfg);
        assert!(
            js.contains("addEventListener('fetch'"),
            "missing fetch handler"
        );
    }

    #[test]
    fn sw_js_cache_first_strategy_comment() {
        let cfg = sample_config(CacheStrategy::CacheFirst);
        let js = generate_sw_js(&cfg);
        assert!(js.contains("Cache First"), "missing Cache First comment");
    }

    #[test]
    fn sw_js_network_first_strategy_comment() {
        let cfg = sample_config(CacheStrategy::NetworkFirst);
        let js = generate_sw_js(&cfg);
        assert!(
            js.contains("Network First"),
            "missing Network First comment"
        );
    }

    #[test]
    fn sw_js_swr_strategy_comment() {
        let cfg = sample_config(CacheStrategy::StaleWhileRevalidate);
        let js = generate_sw_js(&cfg);
        assert!(js.contains("Stale While Revalidate"), "missing SWR comment");
    }

    #[test]
    fn sw_js_contains_skip_waiting() {
        let cfg = sample_config(CacheStrategy::CacheFirst);
        let js = generate_sw_js(&cfg);
        assert!(js.contains("skipWaiting"), "missing skipWaiting call");
    }

    #[test]
    fn sw_js_contains_clients_claim() {
        let cfg = sample_config(CacheStrategy::CacheFirst);
        let js = generate_sw_js(&cfg);
        assert!(js.contains("clients.claim"), "missing clients.claim call");
    }

    // -- generate_cache_manifest_json --

    #[test]
    fn manifest_json_is_valid_json() {
        let cfg = sample_config(CacheStrategy::CacheFirst);
        let entries = vec![sample_entry("/index.html"), sample_entry("/app.wasm")];
        let json = generate_cache_manifest_json(&cfg, &entries);
        let v: serde_json::Value =
            serde_json::from_str(&json).expect("manifest must be valid JSON");
        assert!(v.is_object(), "manifest must be a JSON object");
    }

    #[test]
    fn manifest_json_contains_version() {
        let cfg = sample_config(CacheStrategy::CacheFirst);
        let json = generate_cache_manifest_json(&cfg, &[]);
        assert!(json.contains("\"version\""), "manifest missing version");
    }

    #[test]
    fn manifest_json_contains_cache_name() {
        let cfg = sample_config(CacheStrategy::NetworkFirst);
        let json = generate_cache_manifest_json(&cfg, &[]);
        assert!(
            json.contains("test-cache-v1"),
            "manifest missing cache name"
        );
    }

    #[test]
    fn manifest_json_entries_contain_sha256() {
        let cfg = sample_config(CacheStrategy::CacheFirst);
        let entries = vec![sample_entry("/index.html")];
        let json = generate_cache_manifest_json(&cfg, &entries);
        assert!(json.contains("abc123"), "manifest missing sha256");
    }

    #[test]
    fn manifest_json_entries_contain_url() {
        let cfg = sample_config(CacheStrategy::CacheFirst);
        let entries = vec![sample_entry("/app.wasm")];
        let json = generate_cache_manifest_json(&cfg, &entries);
        assert!(json.contains("/app.wasm"), "manifest missing url");
    }

    #[test]
    fn manifest_json_empty_entries() {
        let cfg = sample_config(CacheStrategy::CacheFirst);
        let json = generate_cache_manifest_json(&cfg, &[]);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let entries = v["entries"].as_object().unwrap();
        assert!(entries.is_empty(), "expected empty entries");
    }

    // -- CacheStrategy::should_use_cache --

    #[test]
    fn cache_first_uses_cache_when_not_expired() {
        let entry = CacheEntry {
            url: "/x".to_string(),
            sha256: "".to_string(),
            size_bytes: 0,
            last_fetched_unix: 1_000,
            ttl_secs: 3_600,
        };
        assert!(CacheStrategy::CacheFirst.should_use_cache(&entry, 1_000));
        assert!(CacheStrategy::CacheFirst.should_use_cache(&entry, 4_599));
    }

    #[test]
    fn cache_first_does_not_use_cache_when_expired() {
        let entry = CacheEntry {
            url: "/x".to_string(),
            sha256: "".to_string(),
            size_bytes: 0,
            last_fetched_unix: 1_000,
            ttl_secs: 3_600,
        };
        assert!(!CacheStrategy::CacheFirst.should_use_cache(&entry, 4_601));
    }

    #[test]
    fn cache_first_uses_cache_when_ttl_zero() {
        let entry = CacheEntry {
            url: "/x".to_string(),
            sha256: "".to_string(),
            size_bytes: 0,
            last_fetched_unix: 0,
            ttl_secs: 0,
        };
        assert!(CacheStrategy::CacheFirst.should_use_cache(&entry, u64::MAX));
    }

    #[test]
    fn network_first_never_uses_cache() {
        let entry = CacheEntry {
            url: "/x".to_string(),
            sha256: "".to_string(),
            size_bytes: 0,
            last_fetched_unix: 0,
            ttl_secs: 0,
        };
        assert!(!CacheStrategy::NetworkFirst.should_use_cache(&entry, 0));
        assert!(!CacheStrategy::NetworkFirst.should_use_cache(&entry, u64::MAX));
    }

    #[test]
    fn swr_uses_cache_when_not_expired() {
        let entry = CacheEntry {
            url: "/x".to_string(),
            sha256: "".to_string(),
            size_bytes: 0,
            last_fetched_unix: 1_000,
            ttl_secs: 3_600,
        };
        assert!(CacheStrategy::StaleWhileRevalidate.should_use_cache(&entry, 1_000));
    }

    // -- js_string_literal --

    #[test]
    fn js_string_literal_wraps_in_single_quotes() {
        let s = js_string_literal("hello");
        assert_eq!(s, "'hello'");
    }

    #[test]
    fn js_string_literal_escapes_single_quote() {
        let s = js_string_literal("it's");
        assert_eq!(s, "'it\\'s'");
    }

    #[test]
    fn js_string_literal_escapes_backslash() {
        let s = js_string_literal("a\\b");
        assert_eq!(s, "'a\\\\b'");
    }

    // -- build_js_string_array --

    #[test]
    fn js_array_contains_brackets() {
        let arr = build_js_string_array(&["/a".to_string(), "/b".to_string()]);
        assert!(arr.starts_with('['), "should start with [");
        assert!(arr.ends_with(']'), "should end with ]");
    }

    #[test]
    fn js_array_contains_all_urls() {
        let arr = build_js_string_array(&["/index.html".to_string(), "/app.js".to_string()]);
        assert!(arr.contains("/index.html"));
        assert!(arr.contains("/app.js"));
    }
}
