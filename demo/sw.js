/**
 * OxiHuman Demo — Service Worker (sw.js)
 * Copyright (C) 2026 COOLJAPAN OU (Team Kitasan)
 * SPDX-License-Identifier: Apache-2.0
 *
 * Cache-first strategy for WASM and static assets.
 * Pattern mirrors the output produced by oxihuman_wasm::service_worker::generate_sw_js.
 *
 * Registration: from index.html or app.js call
 *   navigator.serviceWorker.register('./sw.js');
 */

'use strict';

// ── Configuration ─────────────────────────────────────────────────────────────

const CACHE_NAME    = 'oxihuman-demo-v1';
const MAX_CACHE_MB  = 50;
const MAX_CACHE_BYTES = MAX_CACHE_MB * 1024 * 1024;

/**
 * Assets to precache during the install phase.
 * Adjust pkg/ paths to match your wasm-pack output directory.
 */
const PRECACHE_ASSETS = [
  './index.html',
  './app.js',
  './sw.js',
  './pkg/oxihuman_wasm.js',
  './pkg/oxihuman_wasm_bg.wasm',
];

/**
 * URL prefixes that should use a network-first strategy
 * (e.g. live API calls, if any are added in future).
 * Everything else uses cache-first.
 */
const NETWORK_FIRST_PREFIXES = [
  '/api/',
];

// ── Install ───────────────────────────────────────────────────────────────────

self.addEventListener('install', event => {
  event.waitUntil(
    caches.open(CACHE_NAME).then(cache => {
      // Add assets individually so a single 404 does not abort the whole install.
      const tasks = PRECACHE_ASSETS.map(url =>
        cache.add(url).catch(err => {
          console.warn('[OxiHuman SW] Could not precache:', url, err.message);
        })
      );
      return Promise.all(tasks);
    }).then(() => self.skipWaiting())
  );
});

// ── Activate ──────────────────────────────────────────────────────────────────

self.addEventListener('activate', event => {
  event.waitUntil(
    caches.keys().then(keys => {
      const deletions = keys
        .filter(k => k !== CACHE_NAME)
        .map(k => {
          console.log('[OxiHuman SW] Deleting stale cache:', k);
          return caches.delete(k);
        });
      return Promise.all(deletions);
    }).then(() => self.clients.claim())
  );
});

// ── Fetch ─────────────────────────────────────────────────────────────────────

self.addEventListener('fetch', event => {
  const { request } = event;

  // Only intercept GET requests.
  if (request.method !== 'GET') return;

  // Skip cross-origin requests.
  if (!request.url.startsWith(self.location.origin)) return;

  const url = new URL(request.url);

  // Network-first for API and versioned cache-busted URLs.
  if (NETWORK_FIRST_PREFIXES.some(prefix => url.pathname.startsWith(prefix))) {
    event.respondWith(networkFirst(request));
    return;
  }

  // Cache-first for everything else (WASM, JS, HTML).
  event.respondWith(cacheFirst(request));
});

// ── Strategy helpers ──────────────────────────────────────────────────────────

/**
 * Cache-first: serve from cache; fall back to network and update cache.
 * @param {Request} request
 * @returns {Promise<Response>}
 */
async function cacheFirst(request) {
  const cache    = await caches.open(CACHE_NAME);
  const cached   = await cache.match(request);
  if (cached) return cached;

  const response = await fetchAndCache(cache, request);
  return response;
}

/**
 * Network-first: try network; fall back to cache on failure.
 * @param {Request} request
 * @returns {Promise<Response>}
 */
async function networkFirst(request) {
  const cache = await caches.open(CACHE_NAME);
  try {
    const response = await fetchAndCache(cache, request);
    return response;
  } catch (_) {
    const cached = await cache.match(request);
    if (cached) return cached;
    return new Response('Offline and not cached.', {
      status: 503,
      headers: { 'Content-Type': 'text/plain' },
    });
  }
}

/**
 * Fetch a request, store a clone in the cache, and return the response.
 * Respects MAX_CACHE_BYTES by evicting LRU-like entries when over budget.
 *
 * @param {Cache}   cache
 * @param {Request} request
 * @returns {Promise<Response>}
 */
async function fetchAndCache(cache, request) {
  const response = await fetch(request);
  if (response.ok && response.type !== 'opaque') {
    // Clone before consuming the body.
    cache.put(request, response.clone()).then(() => {
      enforceCacheBudget(cache).catch(() => {});
    });
  }
  return response;
}

/**
 * Evict entries from the cache if total estimated size exceeds the budget.
 * We use Content-Length headers as a rough size estimate.
 *
 * @param {Cache} cache
 */
async function enforceCacheBudget(cache) {
  const keys = await cache.keys();
  let totalBytes = 0;
  /** @type {Array<{ request: Request, size: number }>} */
  const entries = [];

  for (const req of keys) {
    const resp = await cache.match(req);
    if (!resp) continue;
    const cl   = resp.headers.get('Content-Length');
    const size = cl ? parseInt(cl, 10) : 0;
    totalBytes += size;
    entries.push({ request: req, size });
  }

  if (totalBytes <= MAX_CACHE_BYTES) return;

  // Evict smallest non-critical entries first to free space.
  entries.sort((a, b) => a.size - b.size);
  for (const entry of entries) {
    if (totalBytes <= MAX_CACHE_BYTES) break;
    const urlStr = entry.request.url;
    const isCritical = urlStr.endsWith('.wasm') || urlStr.endsWith('index.html');
    if (isCritical) continue;
    await cache.delete(entry.request);
    totalBytes -= entry.size;
    console.log('[OxiHuman SW] Evicted from cache:', urlStr);
  }
}

// ── Message handler ───────────────────────────────────────────────────────────

/**
 * Allow the main thread to send control messages to the service worker.
 * Supported messages:
 *   { type: 'SKIP_WAITING' }  — force activation of a waiting worker
 *   { type: 'CACHE_CLEAR'  }  — purge all caches
 */
self.addEventListener('message', event => {
  const { type } = event.data || {};

  if (type === 'SKIP_WAITING') {
    self.skipWaiting();
    return;
  }

  if (type === 'CACHE_CLEAR') {
    caches.keys().then(keys =>
      Promise.all(keys.map(k => caches.delete(k)))
    ).then(() => {
      if (event.source) {
        event.source.postMessage({ type: 'CACHE_CLEARED' });
      }
    });
    return;
  }
});
