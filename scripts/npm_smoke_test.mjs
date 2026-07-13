#!/usr/bin/env node
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
//
// Pre-publish smoke test for the @cooljapan/oxihuman npm package.
//
// Run against the `nodejs`-target wasm-pack output *before* `npm publish`
// (see .github/workflows/npm-publish.yml) so a broken WASM build fails the
// release job instead of silently shipping. Every assertion below is a
// hard failure -- nothing here is a soft warning.
//
// Usage:
//   node scripts/npm_smoke_test.mjs <path-to-pkg-node-dir> [path-to-ohpk-pack]
//
// The core-pack argument is optional and, when the referenced file does
// not yet exist on disk, that portion of the smoke test is skipped (the
// core pack and its loader API land in a later release wave). Once the
// file exists, the pack-loading assertions become mandatory and any
// failure to load it (or a missing loader API) fails the step.

import { createRequire } from "node:module";
import { existsSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const require = createRequire(import.meta.url);
const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, "..");

function fail(message) {
  console.error(`[npm_smoke_test] FAIL: ${message}`);
  process.exit(1);
}

function ok(message) {
  console.log(`[npm_smoke_test] OK: ${message}`);
}

function skip(message) {
  console.log(`[npm_smoke_test] SKIP: ${message}`);
}

const pkgDirArg = process.argv[2];
const packFileArg = process.argv[3];

if (!pkgDirArg) {
  fail(
    "usage: node npm_smoke_test.mjs <path-to-pkg-node-dir> [path-to-ohpk-pack]",
  );
}

const pkgDir = path.resolve(pkgDirArg);
const pkgJsonPath = path.join(pkgDir, "package.json");

if (!existsSync(pkgJsonPath)) {
  fail(`no package.json found in ${pkgDir} -- did the nodejs target build?`);
}

const pkgMeta = JSON.parse(readFileSync(pkgJsonPath, "utf8"));
const mainEntry = pkgMeta.main;
if (!mainEntry) {
  fail(`package.json in ${pkgDir} has no "main" entry`);
}

const mainPath = path.join(pkgDir, mainEntry);
if (!existsSync(mainPath)) {
  fail(`main entry ${mainPath} does not exist`);
}

// Expected version comes from the workspace Cargo.toml -- the single
// source of truth for the string returned by env!("CARGO_PKG_VERSION")
// inside oxihuman-wasm's get_version() binding.
const cargoTomlPath = path.join(repoRoot, "Cargo.toml");
if (!existsSync(cargoTomlPath)) {
  fail(`workspace Cargo.toml not found at ${cargoTomlPath}`);
}
const cargoToml = readFileSync(cargoTomlPath, "utf8");
const versionMatch = cargoToml.match(
  /\[workspace\.package\][\s\S]*?^version\s*=\s*"([^"]+)"/m,
);
if (!versionMatch) {
  fail("could not find [workspace.package] version in Cargo.toml");
}
const expectedVersion = versionMatch[1];

let mod;
try {
  mod = require(mainPath);
} catch (err) {
  fail(`failed to require() ${mainPath}: ${err.stack || err}`);
}

if (typeof mod.OxiHumanEngine !== "function") {
  fail("OxiHumanEngine class is not exported by the nodejs-target build");
}

let engine;
try {
  engine = new mod.OxiHumanEngine();
} catch (err) {
  fail(`new OxiHumanEngine() threw: ${err.stack || err}`);
}
if (!engine || typeof engine !== "object") {
  fail("new OxiHumanEngine() did not return an object");
}
ok("new OxiHumanEngine() constructs");

if (typeof mod.get_version !== "function") {
  fail("get_version() is not exported by the nodejs-target build");
}
const reportedVersion = mod.get_version();
if (reportedVersion !== expectedVersion) {
  fail(
    `get_version() returned "${reportedVersion}", expected workspace ` +
      `version "${expectedVersion}"`,
  );
}
ok(`get_version() returns the workspace version (${expectedVersion})`);

// Guarded: only exercised once the release core-pack asset exists on disk.
// The loader API may land as a static factory (OxiHumanEngine.from_core_pack_bytes,
// mirroring the existing from_obj_bytes) or as an instance method
// (engine.load_core_pack_bytes, mirroring the existing load_zip_pack_bytes)
// -- accept whichever shape is present so this script does not need to be
// re-synced the moment the API lands.
if (!packFileArg) {
  skip("no core-pack path argument supplied -- skipping core-pack assertion.");
} else {
  const packPath = path.resolve(packFileArg);
  if (!existsSync(packPath)) {
    skip(
      `core-pack asset ${packPath} not present yet -- skipping ` +
        "from_core_pack_bytes/load_core_pack_bytes assertion (expected pre-release).",
    );
  } else {
    const packBytes = readFileSync(packPath);
    let packedEngine;
    let loaderDescription;

    if (typeof mod.OxiHumanEngine.from_core_pack_bytes === "function") {
      loaderDescription = "OxiHumanEngine.from_core_pack_bytes() [static factory]";
      try {
        packedEngine = mod.OxiHumanEngine.from_core_pack_bytes(packBytes);
      } catch (err) {
        fail(`${loaderDescription} threw on ${packPath}: ${err.stack || err}`);
      }
    } else if (typeof engine.load_core_pack_bytes === "function") {
      loaderDescription = "engine.load_core_pack_bytes() [instance method]";
      try {
        engine.load_core_pack_bytes(packBytes);
        packedEngine = engine;
      } catch (err) {
        fail(`${loaderDescription} threw on ${packPath}: ${err.stack || err}`);
      }
    } else {
      fail(
        `core-pack asset ${packPath} exists but neither ` +
          "OxiHumanEngine.from_core_pack_bytes() nor engine.load_core_pack_bytes() " +
          "is exported -- incomplete package, refusing to pass",
      );
    }

    if (typeof packedEngine.vertex_count !== "function") {
      fail("vertex_count() is not exported by the nodejs-target build");
    }
    const vertexCount = packedEngine.vertex_count();
    if (!(vertexCount > 10000)) {
      fail(
        `vertex_count() returned ${vertexCount} after loading the core pack ` +
          `via ${loaderDescription}, expected > 10000`,
      );
    }
    ok(
      `${loaderDescription} loads the core pack, vertex_count=${vertexCount} (> 10000)`,
    );
  }
}

console.log("[npm_smoke_test] All smoke assertions passed.");
