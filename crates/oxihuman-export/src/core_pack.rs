// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! OHPK v1 — the OxiHuman core-pack container format.
//!
//! A single self-describing binary asset pack that bundles a base mesh plus a
//! set of sparse morph targets, so that the CLI can *write* one file and the
//! WASM runtime can *read* it with zero-copy, no-filesystem, `wasm32`-safe
//! parsing.  The design target is a base mesh plus ~40 morph targets in
//! `<= 2 MB` once DEFLATE-compressed.
//!
//! # Container layout
//!
//! ```text
//! ┌────────────────────────────── header (uncompressed) ──────────────────┐
//! │ magic  "OHPK"           : 4 bytes                                      │
//! │ version                 : u8   ( = 1 )                                 │
//! │ flags                   : u8   (bit0 = body is DEFLATE-compressed)     │
//! │ uncompressed body length: u32 LE                                       │
//! └───────────────────────────────────────────────────────────────────────┘
//! ┌────────────────────── body (DEFLATE-compressed) ─────────────────────┐
//! │ manifest_json_len       : u32 LE                                      │
//! │ manifest JSON           : manifest_json_len bytes                     │
//! │ base-mesh section       : see [`BaseMesh`] docs                       │
//! │ target_count            : u32 LE                                      │
//! │ per-target sections     : target_count × target section              │
//! └──────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! Quantisation ideas (sparse varint index deltas + max-abs i16 delta
//! quantisation) are ported from the WASM `compressed_target::LitePack`
//! encoder; nothing is imported across crates.
//!
//! # Model units
//!
//! The MakeHuman `base.obj` is authored in **decimetres** — one model unit is
//! one decimetre, i.e. `100 mm`.  See [`MODEL_UNIT_MM`].  All quantisation
//! error statistics are reported both in raw model units and in millimetres so
//! that downstream tools can reason about them physically.

use anyhow::{anyhow, ensure, Context, Result};
use serde::{Deserialize, Serialize};

// ───────────────────────────── format constants ────────────────────────────

/// Magic bytes at the very start of every OHPK file.
pub const OHPK_MAGIC: [u8; 4] = *b"OHPK";

/// Current OHPK container version.
pub const OHPK_VERSION: u8 = 1;

/// Flag bit: the body is compressed with `oxiarc-deflate` (raw DEFLATE).
pub const OHPK_FLAG_DEFLATE: u8 = 0x01;

/// DEFLATE compression level used when writing the body (0..=9).
pub const OHPK_DEFLATE_LEVEL: u8 = 9;

/// Millimetres per model unit.
///
/// MakeHuman's `base.obj` is authored in decimetres, so `1 unit = 1 dm =
/// 100 mm`.  Multiply any model-unit quantity by this constant to obtain
/// millimetres.
pub const MODEL_UNIT_MM: f32 = 100.0;

/// Number of representable steps across the signed 16-bit range
/// (`i16::MAX - i16::MIN`).
const I16_STEPS: f32 = 65_535.0;

/// Offset that maps the unsigned quantisation index `[0, 65535]` onto the
/// signed `i16` range `[-32768, 32767]`.
const I16_OFFSET: f32 = 32_768.0;

/// Symmetric half-range used for max-abs delta quantisation (`i16::MAX`).
const I16_HALF: f32 = 32_767.0;

/// Convert a length expressed in model units to millimetres.
#[inline]
pub fn model_units_to_mm(units: f32) -> f32 {
    units * MODEL_UNIT_MM
}

// ───────────────────────────────── manifest ────────────────────────────────

/// One provenance file entry (name + hex SHA-256 of the upstream source).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorePackFile {
    /// Upstream file name (repo-relative).
    pub name: String,
    /// Lower-case hex SHA-256 of the upstream file bytes.
    pub sha256: String,
}

/// Provenance block describing where the packed assets came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorePackProvenance {
    /// Upstream repository (e.g. a MakeHuman community data repo URL).
    pub upstream_repo: String,
    /// Exact upstream commit the assets were taken from.
    pub upstream_commit: String,
    /// Per-file integrity records.
    #[serde(default)]
    pub files: Vec<CorePackFile>,
}

/// The JSON manifest embedded at the head of every OHPK body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CorePackManifest {
    /// Human-readable pack name.
    pub name: String,
    /// Pack version string (independent of the container version).
    pub version: String,
    /// SPDX licence id — CC0 for the shipped MakeHuman-derived core pack.
    #[serde(default = "default_license")]
    pub license: String,
    /// Upstream provenance / integrity information.
    pub provenance: CorePackProvenance,
    /// Optional minimum modelled age in years (safety floor).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub age_floor_years: Option<f32>,
    /// Free-form category tags for the pack as a whole.
    #[serde(default)]
    pub categories: Vec<String>,
    /// Names of every morph target contained in the pack, in order.
    #[serde(default)]
    pub target_names: Vec<String>,
}

fn default_license() -> String {
    "CC0-1.0".to_string()
}

impl CorePackManifest {
    /// Construct a minimal manifest with the CC0 licence and empty provenance.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            license: default_license(),
            provenance: CorePackProvenance {
                upstream_repo: String::new(),
                upstream_commit: String::new(),
                files: Vec::new(),
            },
            age_floor_years: None,
            categories: Vec::new(),
            target_names: Vec::new(),
        }
    }
}

// ─────────────────────────── quantisation reports ──────────────────────────

/// Quantisation error for a single morph target.
#[derive(Debug, Clone, PartialEq)]
pub struct TargetErrorReport {
    /// Target name.
    pub name: String,
    /// Maximum absolute per-component error, in model units.
    pub max_error_units: f32,
    /// The same error expressed in millimetres.
    pub max_error_mm: f32,
}

/// Aggregate quantisation-honesty report for a whole pack.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantizationReport {
    /// Maximum absolute base-position error (any axis), in model units.
    pub base_pos_max_error_units: f32,
    /// The base-position error expressed in millimetres.
    pub base_pos_max_error_mm: f32,
    /// Maximum absolute UV error, if the pack carries UVs (dimensionless).
    pub base_uv_max_error: Option<f32>,
    /// Per-target quantisation errors.
    pub targets: Vec<TargetErrorReport>,
}

// ─────────────────────────── low-level byte writer ─────────────────────────

#[inline]
fn push_u16(buf: &mut Vec<u8>, v: u16) {
    buf.extend_from_slice(&v.to_le_bytes());
}

#[inline]
fn push_u32(buf: &mut Vec<u8>, v: u32) {
    buf.extend_from_slice(&v.to_le_bytes());
}

#[inline]
fn push_i16(buf: &mut Vec<u8>, v: i16) {
    buf.extend_from_slice(&v.to_le_bytes());
}

#[inline]
fn push_f32(buf: &mut Vec<u8>, v: f32) {
    buf.extend_from_slice(&v.to_le_bytes());
}

/// Append an unsigned LEB128 varint.
fn push_uvarint(buf: &mut Vec<u8>, mut v: u64) {
    loop {
        let byte = (v & 0x7f) as u8;
        v >>= 7;
        if v == 0 {
            buf.push(byte);
            return;
        }
        buf.push(byte | 0x80);
    }
}

// ─────────────────────────── low-level byte reader ─────────────────────────

/// A bounds-checked, panic-free forward cursor over an in-memory byte slice.
struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    #[inline]
    fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self
            .pos
            .checked_add(n)
            .ok_or_else(|| anyhow!("OHPK reader length overflow"))?;
        ensure!(
            end <= self.data.len(),
            "OHPK truncated: need {n} bytes at offset {} but only {} remain",
            self.pos,
            self.remaining()
        );
        let slice = &self.data[self.pos..end];
        self.pos = end;
        Ok(slice)
    }

    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16> {
        let b: [u8; 2] = self
            .take(2)?
            .try_into()
            .map_err(|_| anyhow!("OHPK u16 slice conversion"))?;
        Ok(u16::from_le_bytes(b))
    }

    fn u32(&mut self) -> Result<u32> {
        let b: [u8; 4] = self
            .take(4)?
            .try_into()
            .map_err(|_| anyhow!("OHPK u32 slice conversion"))?;
        Ok(u32::from_le_bytes(b))
    }

    fn i16(&mut self) -> Result<i16> {
        let b: [u8; 2] = self
            .take(2)?
            .try_into()
            .map_err(|_| anyhow!("OHPK i16 slice conversion"))?;
        Ok(i16::from_le_bytes(b))
    }

    fn f32(&mut self) -> Result<f32> {
        let b: [u8; 4] = self
            .take(4)?
            .try_into()
            .map_err(|_| anyhow!("OHPK f32 slice conversion"))?;
        Ok(f32::from_le_bytes(b))
    }

    /// Read a `u32`-length-prefixed byte run.
    fn len_prefixed_u32(&mut self) -> Result<&'a [u8]> {
        let n = self.u32()? as usize;
        self.take(n)
    }

    /// Read a `u16`-length-prefixed UTF-8 string.
    fn string_u16(&mut self) -> Result<String> {
        let n = self.u16()? as usize;
        let bytes = self.take(n)?;
        String::from_utf8(bytes.to_vec()).context("OHPK invalid UTF-8 string")
    }

    /// Decode an unsigned LEB128 varint.
    fn uvarint(&mut self) -> Result<u64> {
        let mut result: u64 = 0;
        let mut shift: u32 = 0;
        loop {
            let byte = self.u8()?;
            ensure!(shift < 64, "OHPK varint overflow");
            result |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return Ok(result);
            }
            shift += 7;
        }
    }
}

// ─────────────────────────── quantisation helpers ──────────────────────────

/// Per-axis affine quantiser: `value ≈ q * scale + bias` where `q: i16`.
#[derive(Debug, Clone, Copy)]
struct AxisQuant {
    scale: f32,
    bias: f32,
}

impl AxisQuant {
    /// Fit a quantiser mapping `[lo, hi]` onto the full signed 16-bit range.
    fn fit(lo: f32, hi: f32) -> Self {
        let span = hi - lo;
        if span > 0.0 && span.is_finite() {
            let scale = span / I16_STEPS;
            // q = -32768 → lo, q = 32767 → hi.
            let bias = lo + I16_OFFSET * scale;
            Self { scale, bias }
        } else {
            // Degenerate (flat / non-finite) axis: everything maps to `lo`.
            Self {
                scale: 0.0,
                bias: lo,
            }
        }
    }

    #[inline]
    fn quantise(&self, v: f32) -> i16 {
        if self.scale > 0.0 {
            let t = ((v - self.bias) / self.scale).round();
            t.clamp(-I16_OFFSET, I16_HALF) as i16
        } else {
            0
        }
    }

    #[inline]
    fn dequantise(&self, q: i16) -> f32 {
        f32::from(q) * self.scale + self.bias
    }
}

/// Fit one [`AxisQuant`] per component of an interleaved point buffer.
///
/// `stride` is the number of `f32` components per element; only the first
/// `dims` of them are considered.
fn fit_axes(data: &[f32], stride: usize, dims: usize) -> Vec<AxisQuant> {
    let mut lo = vec![f32::INFINITY; dims];
    let mut hi = vec![f32::NEG_INFINITY; dims];
    for chunk in data.chunks_exact(stride) {
        for (axis, slot) in chunk.iter().take(dims).enumerate() {
            let v = *slot;
            if v < lo[axis] {
                lo[axis] = v;
            }
            if v > hi[axis] {
                hi[axis] = v;
            }
        }
    }
    (0..dims)
        .map(|axis| {
            let (l, h) = if lo[axis].is_finite() && hi[axis].is_finite() {
                (lo[axis], hi[axis])
            } else {
                (0.0, 0.0)
            };
            AxisQuant::fit(l, h)
        })
        .collect()
}

// ───────────────────────────── builder targets ─────────────────────────────

struct BuilderTarget {
    name: String,
    category: String,
    sparse: Vec<(u32, [f32; 3])>,
}

// ───────────────────────────────── builder ─────────────────────────────────

/// Incrementally assembles an OHPK v1 core pack in memory.
///
/// Nothing touches the filesystem; the whole pack is produced as a `Vec<u8>`
/// from [`build`](CorePackBuilder::build), making the builder usable on
/// `wasm32` as well.
pub struct CorePackBuilder {
    manifest: Option<CorePackManifest>,
    base_positions: Vec<f32>,
    base_indices: Vec<u32>,
    base_uvs: Option<Vec<f32>>,
    helper_meta: Option<Vec<u8>>,
    targets: Vec<BuilderTarget>,
}

impl Default for CorePackBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CorePackBuilder {
    /// Create an empty builder.
    pub fn new() -> Self {
        Self {
            manifest: None,
            base_positions: Vec::new(),
            base_indices: Vec::new(),
            base_uvs: None,
            helper_meta: None,
            targets: Vec::new(),
        }
    }

    /// Set the base mesh.
    ///
    /// * `positions` — flat `x, y, z` triples (length must be a multiple of 3).
    /// * `indices`   — triangle indices (stored raw as `u32`).
    /// * `uvs`       — optional flat `u, v` pairs; when present its length must
    ///   be exactly `2 * n_verts`.
    ///
    /// Length validation happens in [`build`](CorePackBuilder::build).
    pub fn set_base_mesh(
        &mut self,
        positions: &[f32],
        indices: &[u32],
        uvs: Option<&[f32]>,
    ) -> &mut Self {
        self.base_positions = positions.to_vec();
        self.base_indices = indices.to_vec();
        self.base_uvs = uvs.map(<[f32]>::to_vec);
        self
    }

    /// Attach opaque vertex-group / helper metadata for future use.
    pub fn set_helper_metadata(&mut self, bytes: &[u8]) -> &mut Self {
        self.helper_meta = Some(bytes.to_vec());
        self
    }

    /// Add a sparse morph target.
    ///
    /// `sparse` is a list of `(vertex_index, [dx, dy, dz])` in model units.
    /// Entries may be supplied in any order — they are sorted by index at
    /// encode time.
    pub fn add_target(
        &mut self,
        name: impl Into<String>,
        category: impl Into<String>,
        sparse: &[(u32, [f32; 3])],
    ) -> &mut Self {
        self.targets.push(BuilderTarget {
            name: name.into(),
            category: category.into(),
            sparse: sparse.to_vec(),
        });
        self
    }

    /// Replace the pack manifest.
    pub fn set_manifest(&mut self, manifest: CorePackManifest) -> &mut Self {
        self.manifest = Some(manifest);
        self
    }

    /// Serialise the pack to OHPK v1 bytes.
    ///
    /// Fails when no base mesh has been set, when buffer lengths are
    /// inconsistent, or when compression fails.
    pub fn build(&self) -> Result<Vec<u8>> {
        ensure!(
            !self.base_positions.is_empty(),
            "cannot build an empty core pack: no base mesh set"
        );
        ensure!(
            self.base_positions.len().is_multiple_of(3),
            "base positions length {} is not a multiple of 3",
            self.base_positions.len()
        );
        let n_verts = self.base_positions.len() / 3;
        ensure!(n_verts > 0, "cannot build a core pack with zero vertices");

        if let Some(uvs) = &self.base_uvs {
            ensure!(
                uvs.len() == n_verts * 2,
                "UV buffer length {} does not match 2 * n_verts ({})",
                uvs.len(),
                n_verts * 2
            );
        }

        let manifest = self.resolved_manifest();
        let manifest_json =
            serde_json::to_vec(&manifest).context("serialising OHPK manifest to JSON")?;

        // ---- body ---------------------------------------------------------
        let mut body = Vec::new();
        push_u32(&mut body, u32::try_from(manifest_json.len()).context("manifest too large")?);
        body.extend_from_slice(&manifest_json);

        self.encode_base_mesh(&mut body, n_verts)?;

        push_u32(
            &mut body,
            u32::try_from(self.targets.len()).context("too many targets")?,
        );
        for target in &self.targets {
            encode_target(&mut body, target);
        }

        // ---- header + compression ----------------------------------------
        let uncompressed_len = u32::try_from(body.len()).context("OHPK body exceeds 4 GiB")?;
        let compressed = oxiarc_deflate::deflate(&body, OHPK_DEFLATE_LEVEL)
            .map_err(|e| anyhow!("OHPK deflate failed: {e}"))?;

        let mut out = Vec::with_capacity(10 + compressed.len());
        out.extend_from_slice(&OHPK_MAGIC);
        out.push(OHPK_VERSION);
        out.push(OHPK_FLAG_DEFLATE);
        push_u32(&mut out, uncompressed_len);
        out.extend_from_slice(&compressed);
        Ok(out)
    }

    /// Clone the manifest, filling `target_names` from the added targets when
    /// the caller left it empty.
    fn resolved_manifest(&self) -> CorePackManifest {
        let mut manifest = self
            .manifest
            .clone()
            .unwrap_or_else(|| CorePackManifest::new("oxihuman-core", "0"));
        if manifest.target_names.is_empty() {
            manifest.target_names = self.targets.iter().map(|t| t.name.clone()).collect();
        }
        manifest
    }

    fn encode_base_mesh(&self, body: &mut Vec<u8>, n_verts: usize) -> Result<()> {
        let pos_axes = fit_axes(&self.base_positions, 3, 3);
        let n_indices = self.base_indices.len();

        push_u32(body, u32::try_from(n_verts).context("n_verts too large")?);
        push_u32(body, u32::try_from(n_indices).context("n_indices too large")?);

        for a in &pos_axes {
            push_f32(body, a.scale);
        }
        for a in &pos_axes {
            push_f32(body, a.bias);
        }

        // Quantise positions and record the honest max per-component error.
        let mut pos_max_err = 0.0f32;
        let mut quantised: Vec<i16> = Vec::with_capacity(n_verts * 3);
        for chunk in self.base_positions.chunks_exact(3) {
            for (axis, value) in chunk.iter().enumerate() {
                let q = pos_axes[axis].quantise(*value);
                let err = (value - pos_axes[axis].dequantise(q)).abs();
                if err > pos_max_err {
                    pos_max_err = err;
                }
                quantised.push(q);
            }
        }
        push_f32(body, pos_max_err);
        for q in &quantised {
            push_i16(body, *q);
        }

        for idx in &self.base_indices {
            push_u32(body, *idx);
        }

        // Optional UV section.
        match &self.base_uvs {
            Some(uvs) => {
                body.push(1u8);
                let uv_axes = fit_axes(uvs, 2, 2);
                for a in &uv_axes {
                    push_f32(body, a.scale);
                }
                for a in &uv_axes {
                    push_f32(body, a.bias);
                }
                let mut uv_max_err = 0.0f32;
                let mut uv_q: Vec<i16> = Vec::with_capacity(n_verts * 2);
                for pair in uvs.chunks_exact(2) {
                    for (axis, value) in pair.iter().enumerate() {
                        let q = uv_axes[axis].quantise(*value);
                        let err = (value - uv_axes[axis].dequantise(q)).abs();
                        if err > uv_max_err {
                            uv_max_err = err;
                        }
                        uv_q.push(q);
                    }
                }
                push_f32(body, uv_max_err);
                for q in &uv_q {
                    push_i16(body, *q);
                }
            }
            None => body.push(0u8),
        }

        // Optional helper-metadata section (reserved for future use).
        match &self.helper_meta {
            Some(meta) => {
                body.push(1u8);
                push_u32(body, u32::try_from(meta.len()).context("helper meta too large")?);
                body.extend_from_slice(meta);
            }
            None => body.push(0u8),
        }

        Ok(())
    }
}

/// Encode a single sparse target section into `body`.
fn encode_target(body: &mut Vec<u8>, target: &BuilderTarget) {
    // Sort affected vertices ascending so index deltas stay non-negative.
    let mut sparse = target.sparse.clone();
    sparse.sort_by_key(|(idx, _)| *idx);

    // Max-abs component drives the symmetric i16 delta quantiser.
    let max_abs = sparse
        .iter()
        .flat_map(|(_, d)| d.iter())
        .fold(0.0f32, |acc, &c| acc.max(c.abs()));
    let scale = if max_abs > 0.0 && max_abs.is_finite() {
        max_abs / I16_HALF
    } else {
        0.0
    };

    // Quantise deltas and record the honest per-component max error.
    let mut quantised: Vec<[i16; 3]> = Vec::with_capacity(sparse.len());
    let mut max_err = 0.0f32;
    for (_, d) in &sparse {
        let mut q = [0i16; 3];
        for axis in 0..3 {
            let qi = if scale > 0.0 {
                (d[axis] / scale).round().clamp(-I16_HALF, I16_HALF) as i16
            } else {
                0
            };
            let deq = f32::from(qi) * scale;
            let err = (d[axis] - deq).abs();
            if err > max_err {
                max_err = err;
            }
            q[axis] = qi;
        }
        quantised.push(q);
    }

    let name = target.name.as_bytes();
    let category = target.category.as_bytes();
    push_u16(body, name.len() as u16);
    body.extend_from_slice(name);
    push_u16(body, category.len() as u16);
    body.extend_from_slice(category);

    push_f32(body, scale);
    push_f32(body, max_err);
    push_u32(body, sparse.len() as u32);

    // Delta-encoded vertex indices as unsigned varints.
    let mut prev = 0u32;
    for (idx, _) in &sparse {
        let delta = idx.wrapping_sub(prev);
        push_uvarint(body, u64::from(delta));
        prev = *idx;
    }

    // Quantised deltas, i16 × 3 each.
    for q in &quantised {
        push_i16(body, q[0]);
        push_i16(body, q[1]);
        push_i16(body, q[2]);
    }
}

// ─────────────────────────────── parsed types ──────────────────────────────

/// A parsed base mesh with its quantisers retained for de-quantisation.
struct BaseMesh {
    n_verts: usize,
    pos_axes: [AxisQuant; 3],
    pos_max_err: f32,
    positions_q: Vec<i16>,
    indices: Vec<u32>,
    uvs: Option<UvSection>,
    helper_meta: Option<Vec<u8>>,
}

struct UvSection {
    axes: [AxisQuant; 2],
    max_err: f32,
    data_q: Vec<i16>,
}

/// A parsed morph target inside a [`CorePack`].
pub struct CorePackTarget {
    name: String,
    category: String,
    scale: f32,
    max_abs_error_units: f32,
    indices: Vec<u32>,
    deltas_q: Vec<[i16; 3]>,
}

impl CorePackTarget {
    /// Target name.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Target category.
    #[inline]
    pub fn category(&self) -> &str {
        &self.category
    }

    /// Number of affected vertices.
    #[inline]
    pub fn len(&self) -> usize {
        self.indices.len()
    }

    /// Whether the target affects no vertices.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// Honest maximum absolute per-component quantisation error (model units).
    #[inline]
    pub fn max_abs_error_units(&self) -> f32 {
        self.max_abs_error_units
    }

    /// The affected vertex indices (sorted ascending).
    #[inline]
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    /// De-quantised sparse deltas as `(vertex_index, [dx, dy, dz])`.
    pub fn sparse(&self) -> Vec<(u32, [f32; 3])> {
        self.indices
            .iter()
            .zip(&self.deltas_q)
            .map(|(&idx, q)| {
                (
                    idx,
                    [
                        f32::from(q[0]) * self.scale,
                        f32::from(q[1]) * self.scale,
                        f32::from(q[2]) * self.scale,
                    ],
                )
            })
            .collect()
    }
}

/// A fully parsed OHPK v1 core pack held in memory.
pub struct CorePack {
    manifest: CorePackManifest,
    base: BaseMesh,
    targets: Vec<CorePackTarget>,
}

impl CorePack {
    /// Parse an OHPK v1 pack from bytes.
    ///
    /// Every malformed input — bad magic, unsupported version, truncated
    /// stream, an oversized declared body length, or a corrupt DEFLATE
    /// payload — yields an `Err` and never panics.
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        ensure!(bytes.len() >= 10, "OHPK too short for a header");
        ensure!(bytes[0..4] == OHPK_MAGIC, "bad OHPK magic bytes");
        let version = bytes[4];
        ensure!(
            version == OHPK_VERSION,
            "unsupported OHPK version {version} (expected {OHPK_VERSION})"
        );
        let flags = bytes[5];
        let declared_len = u32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]) as usize;
        let payload = &bytes[10..];

        let body = if flags & OHPK_FLAG_DEFLATE != 0 {
            let inflated =
                oxiarc_deflate::inflate(payload).map_err(|e| anyhow!("OHPK inflate failed: {e}"))?;
            ensure!(
                inflated.len() == declared_len,
                "OHPK body length mismatch: header says {declared_len}, inflated {}",
                inflated.len()
            );
            inflated
        } else {
            ensure!(
                payload.len() == declared_len,
                "OHPK body length mismatch: header says {declared_len}, payload {}",
                payload.len()
            );
            payload.to_vec()
        };

        Self::parse_body(&body)
    }

    fn parse_body(body: &[u8]) -> Result<Self> {
        let mut r = Reader::new(body);

        let manifest_bytes = r.len_prefixed_u32().context("reading manifest JSON")?;
        let manifest: CorePackManifest =
            serde_json::from_slice(manifest_bytes).context("parsing OHPK manifest JSON")?;

        let base = parse_base_mesh(&mut r)?;

        let target_count = r.u32().context("reading target count")? as usize;
        let mut targets = Vec::with_capacity(target_count.min(1024));
        for i in 0..target_count {
            let t = parse_target(&mut r).with_context(|| format!("parsing target #{i}"))?;
            targets.push(t);
        }

        Ok(Self {
            manifest,
            base,
            targets,
        })
    }

    /// The embedded manifest.
    #[inline]
    pub fn manifest(&self) -> &CorePackManifest {
        &self.manifest
    }

    /// Number of vertices in the base mesh.
    #[inline]
    pub fn vertex_count(&self) -> usize {
        self.base.n_verts
    }

    /// De-quantised base positions as flat `x, y, z` triples.
    pub fn base_positions(&self) -> Vec<f32> {
        let mut out = Vec::with_capacity(self.base.positions_q.len());
        for chunk in self.base.positions_q.chunks_exact(3) {
            for (axis, &q) in chunk.iter().enumerate() {
                out.push(self.base.pos_axes[axis].dequantise(q));
            }
        }
        out
    }

    /// The raw base-mesh triangle indices.
    #[inline]
    pub fn base_indices(&self) -> &[u32] {
        &self.base.indices
    }

    /// De-quantised base UVs as flat `u, v` pairs, if present.
    pub fn base_uvs(&self) -> Option<Vec<f32>> {
        self.base.uvs.as_ref().map(|uv| {
            let mut out = Vec::with_capacity(uv.data_q.len());
            for pair in uv.data_q.chunks_exact(2) {
                for (axis, &q) in pair.iter().enumerate() {
                    out.push(uv.axes[axis].dequantise(q));
                }
            }
            out
        })
    }

    /// Opaque helper / vertex-group metadata, if the pack carries any.
    #[inline]
    pub fn helper_metadata(&self) -> Option<&[u8]> {
        self.base.helper_meta.as_deref()
    }

    /// The parsed morph targets.
    #[inline]
    pub fn targets(&self) -> &[CorePackTarget] {
        &self.targets
    }

    /// Build the honest quantisation-error report for the whole pack.
    pub fn quantization_report(&self) -> QuantizationReport {
        let targets = self
            .targets
            .iter()
            .map(|t| TargetErrorReport {
                name: t.name.clone(),
                max_error_units: t.max_abs_error_units,
                max_error_mm: model_units_to_mm(t.max_abs_error_units),
            })
            .collect();
        QuantizationReport {
            base_pos_max_error_units: self.base.pos_max_err,
            base_pos_max_error_mm: model_units_to_mm(self.base.pos_max_err),
            base_uv_max_error: self.base.uvs.as_ref().map(|uv| uv.max_err),
            targets,
        }
    }
}

fn parse_base_mesh(r: &mut Reader<'_>) -> Result<BaseMesh> {
    let n_verts = r.u32().context("reading n_verts")? as usize;
    let n_indices = r.u32().context("reading n_indices")? as usize;

    let pos_axes = [
        AxisQuant {
            scale: r.f32()?,
            bias: 0.0,
        },
        AxisQuant {
            scale: r.f32()?,
            bias: 0.0,
        },
        AxisQuant {
            scale: r.f32()?,
            bias: 0.0,
        },
    ];
    // Biases are stored after the three scales.
    let biases = [r.f32()?, r.f32()?, r.f32()?];
    let pos_axes = [
        AxisQuant {
            scale: pos_axes[0].scale,
            bias: biases[0],
        },
        AxisQuant {
            scale: pos_axes[1].scale,
            bias: biases[1],
        },
        AxisQuant {
            scale: pos_axes[2].scale,
            bias: biases[2],
        },
    ];

    let pos_max_err = r.f32().context("reading base position error")?;

    let mut positions_q = Vec::with_capacity(n_verts.min(1 << 20) * 3);
    for _ in 0..n_verts * 3 {
        positions_q.push(r.i16()?);
    }

    let mut indices = Vec::with_capacity(n_indices.min(1 << 21));
    for _ in 0..n_indices {
        indices.push(r.u32()?);
    }

    // Optional UV section.
    let has_uvs = r.u8().context("reading has_uvs flag")?;
    let uvs = if has_uvs != 0 {
        let axes = [
            AxisQuant {
                scale: r.f32()?,
                bias: 0.0,
            },
            AxisQuant {
                scale: r.f32()?,
                bias: 0.0,
            },
        ];
        let uv_bias = [r.f32()?, r.f32()?];
        let axes = [
            AxisQuant {
                scale: axes[0].scale,
                bias: uv_bias[0],
            },
            AxisQuant {
                scale: axes[1].scale,
                bias: uv_bias[1],
            },
        ];
        let max_err = r.f32()?;
        let mut data_q = Vec::with_capacity(n_verts.min(1 << 20) * 2);
        for _ in 0..n_verts * 2 {
            data_q.push(r.i16()?);
        }
        Some(UvSection {
            axes,
            max_err,
            data_q,
        })
    } else {
        None
    };

    // Optional helper-metadata section.
    let has_helper = r.u8().context("reading has_helper flag")?;
    let helper_meta = if has_helper != 0 {
        Some(r.len_prefixed_u32().context("reading helper meta")?.to_vec())
    } else {
        None
    };

    Ok(BaseMesh {
        n_verts,
        pos_axes,
        pos_max_err,
        positions_q,
        indices,
        uvs,
        helper_meta,
    })
}

fn parse_target(r: &mut Reader<'_>) -> Result<CorePackTarget> {
    let name = r.string_u16().context("reading target name")?;
    let category = r.string_u16().context("reading target category")?;
    let scale = r.f32().context("reading target scale")?;
    let max_abs_error_units = r.f32().context("reading target error")?;
    let n = r.u32().context("reading target affected count")? as usize;

    let mut indices = Vec::with_capacity(n.min(1 << 20));
    let mut prev = 0u32;
    for _ in 0..n {
        let delta = u32::try_from(r.uvarint()?).context("target index delta overflow")?;
        let idx = prev.wrapping_add(delta);
        indices.push(idx);
        prev = idx;
    }

    let mut deltas_q = Vec::with_capacity(n.min(1 << 20));
    for _ in 0..n {
        deltas_q.push([r.i16()?, r.i16()?, r.i16()?]);
    }

    Ok(CorePackTarget {
        name,
        category,
        scale,
        max_abs_error_units,
        indices,
        deltas_q,
    })
}

// ──────────────────────────────────── tests ────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest() -> CorePackManifest {
        CorePackManifest {
            name: "oxihuman-core".to_string(),
            version: "1.2.3".to_string(),
            license: "CC0-1.0".to_string(),
            provenance: CorePackProvenance {
                upstream_repo: "https://example.invalid/makehuman-data".to_string(),
                upstream_commit: "0123456789abcdef0123456789abcdef01234567".to_string(),
                files: vec![
                    CorePackFile {
                        name: "base.obj".to_string(),
                        sha256: "a".repeat(64),
                    },
                    CorePackFile {
                        name: "targets/smile.target".to_string(),
                        sha256: "b".repeat(64),
                    },
                ],
            },
            age_floor_years: Some(18.0),
            categories: vec!["body".to_string(), "face".to_string()],
            target_names: vec!["smile".to_string(), "blink".to_string()],
        }
    }

    /// A tiny quad mesh (4 verts, 2 triangles) spanning a known cube-ish box.
    fn sample_base() -> (Vec<f32>, Vec<u32>, Vec<f32>) {
        let positions = vec![
            -1.0, -2.0, 0.5, // v0
            1.0, -2.0, 0.5, // v1
            1.0, 2.0, -0.5, // v2
            -1.0, 2.0, -0.5, // v3
        ];
        let indices = vec![0u32, 1, 2, 0, 2, 3];
        let uvs = vec![0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0];
        (positions, indices, uvs)
    }

    fn build_sample() -> Vec<u8> {
        let (positions, indices, uvs) = sample_base();
        let mut b = CorePackBuilder::new();
        b.set_manifest(sample_manifest());
        b.set_base_mesh(&positions, &indices, Some(&uvs));
        // Two sparse targets.
        b.add_target(
            "smile",
            "face",
            &[(1u32, [0.01, -0.02, 0.03]), (3u32, [-0.005, 0.004, 0.0])],
        );
        b.add_target("blink", "face", &[(2u32, [0.0, 0.0, 0.001])]);
        b.build().expect("build sample pack")
    }

    #[test]
    fn manifest_exact_round_trip() {
        let bytes = build_sample();
        let pack = CorePack::parse(&bytes).expect("parse pack");
        assert_eq!(pack.manifest(), &sample_manifest());
    }

    #[test]
    fn manifest_defaults_license_cc0() {
        let m = CorePackManifest::new("x", "1");
        assert_eq!(m.license, "CC0-1.0");
    }

    #[test]
    fn base_positions_round_trip_within_bound() {
        let (positions, _indices, _uvs) = sample_base();
        let bytes = build_sample();
        let pack = CorePack::parse(&bytes).expect("parse pack");

        let decoded = pack.base_positions();
        assert_eq!(decoded.len(), positions.len());

        // Recompute the actual max per-component error and compare it to the
        // honestly reported one.
        let mut actual_max = 0.0f32;
        for (o, d) in positions.iter().zip(&decoded) {
            actual_max = actual_max.max((o - d).abs());
        }
        let report = pack.quantization_report();
        // Reported error must be honest: never smaller than reality …
        assert!(
            report.base_pos_max_error_units + 1e-9 >= actual_max,
            "reported {} < actual {}",
            report.base_pos_max_error_units,
            actual_max
        );
        // … and it is in fact exact (same de-quantisation formula).
        assert!(
            (report.base_pos_max_error_units - actual_max).abs() < 1e-6,
            "reported {} vs actual {}",
            report.base_pos_max_error_units,
            actual_max
        );
        // mm conversion follows the decimetre convention.
        assert!(
            (report.base_pos_max_error_mm - actual_max * MODEL_UNIT_MM).abs() < 1e-4,
            "mm conversion mismatch"
        );

        // The error must respect the per-axis quantisation step (span/65535/2).
        // The x-span is 2.0, y-span 4.0, z-span 1.0 → worst half-step = 4/65535/2.
        let worst_half_step = 4.0f32 / I16_STEPS / 2.0;
        assert!(
            actual_max <= worst_half_step + 1e-6,
            "actual {} exceeds half-step {}",
            actual_max,
            worst_half_step
        );
    }

    #[test]
    fn base_uvs_round_trip() {
        let (_p, _i, uvs) = sample_base();
        let bytes = build_sample();
        let pack = CorePack::parse(&bytes).expect("parse pack");
        let decoded = pack.base_uvs().expect("uvs present");
        assert_eq!(decoded.len(), uvs.len());
        for (o, d) in uvs.iter().zip(&decoded) {
            assert!((o - d).abs() < 1e-3, "uv {o} vs {d}");
        }
    }

    #[test]
    fn base_indices_round_trip() {
        let (_p, indices, _u) = sample_base();
        let bytes = build_sample();
        let pack = CorePack::parse(&bytes).expect("parse pack");
        assert_eq!(pack.base_indices(), indices.as_slice());
    }

    #[test]
    fn sparse_target_round_trip() {
        let bytes = build_sample();
        let pack = CorePack::parse(&bytes).expect("parse pack");
        assert_eq!(pack.targets().len(), 2);

        let smile = &pack.targets()[0];
        assert_eq!(smile.name(), "smile");
        assert_eq!(smile.category(), "face");
        assert_eq!(smile.len(), 2);

        let sparse = smile.sparse();
        // Sorted by index ascending: 1 then 3.
        assert_eq!(sparse[0].0, 1);
        assert_eq!(sparse[1].0, 3);

        // Values recovered within the reported target error bound.
        let report = pack.quantization_report();
        let smile_err = report
            .targets
            .iter()
            .find(|t| t.name == "smile")
            .map(|t| t.max_error_units)
            .expect("smile in report");

        let expected = [
            (1u32, [0.01f32, -0.02, 0.03]),
            (3u32, [-0.005, 0.004, 0.0]),
        ];
        for ((idx, got), (eidx, want)) in sparse.iter().zip(expected.iter()) {
            assert_eq!(idx, eidx);
            for k in 0..3 {
                assert!(
                    (got[k] - want[k]).abs() <= smile_err + 1e-6,
                    "target comp {} got {} want {} err bound {}",
                    k,
                    got[k],
                    want[k],
                    smile_err
                );
            }
        }
        // The report exposes mm as well.
        assert!(smile_err >= 0.0);
        let smile_mm = report
            .targets
            .iter()
            .find(|t| t.name == "smile")
            .map(|t| t.max_error_mm)
            .expect("smile mm");
        assert!((smile_mm - smile_err * MODEL_UNIT_MM).abs() < 1e-4);
    }

    #[test]
    fn target_error_is_honest() {
        let bytes = build_sample();
        let pack = CorePack::parse(&bytes).expect("parse pack");
        let expected: std::collections::HashMap<u32, [f32; 3]> = [
            (1u32, [0.01f32, -0.02, 0.03]),
            (3u32, [-0.005, 0.004, 0.0]),
        ]
        .into_iter()
        .collect();

        let smile = &pack.targets()[0];
        let mut actual_max = 0.0f32;
        for (idx, got) in smile.sparse() {
            let want = expected[&idx];
            for k in 0..3 {
                actual_max = actual_max.max((got[k] - want[k]).abs());
            }
        }
        // Reported error must be >= actual and exact to float epsilon.
        assert!(smile.max_abs_error_units() + 1e-9 >= actual_max);
        assert!((smile.max_abs_error_units() - actual_max).abs() < 1e-6);
    }

    #[test]
    fn deflate_round_trip_through_oxiarc() {
        // A larger, mostly-flat target to exercise real DEFLATE on the body.
        let positions: Vec<f32> = (0..300).map(|i| (i as f32) * 0.01 - 1.5).collect();
        let indices: Vec<u32> = (0..99).collect();
        let mut sparse: Vec<(u32, [f32; 3])> = Vec::new();
        for i in (0..100).step_by(7) {
            sparse.push((i as u32, [0.002, -0.001, 0.0005]));
        }
        let mut b = CorePackBuilder::new();
        b.set_manifest(CorePackManifest::new("deflate-test", "1"));
        b.set_base_mesh(&positions, &indices, None);
        b.add_target("wave", "body", &sparse);
        let bytes = b.build().expect("build");

        // Header must advertise DEFLATE.
        assert_eq!(&bytes[0..4], &OHPK_MAGIC);
        assert_eq!(bytes[4], OHPK_VERSION);
        assert_ne!(bytes[5] & OHPK_FLAG_DEFLATE, 0);

        let pack = CorePack::parse(&bytes).expect("parse");
        assert_eq!(pack.vertex_count(), 100);
        assert_eq!(pack.base_indices().len(), 99);
        assert!(pack.base_uvs().is_none());
        assert_eq!(pack.targets().len(), 1);
        assert_eq!(pack.targets()[0].len(), sparse.len());
    }

    #[test]
    fn empty_pack_build_errors() {
        let b = CorePackBuilder::new();
        assert!(b.build().is_err());
    }

    #[test]
    fn odd_positions_length_errors() {
        let mut b = CorePackBuilder::new();
        b.set_base_mesh(&[0.0, 1.0], &[], None); // length 2, not a multiple of 3
        assert!(b.build().is_err());
    }

    #[test]
    fn mismatched_uv_length_errors() {
        let mut b = CorePackBuilder::new();
        b.set_base_mesh(&[0.0, 0.0, 0.0], &[0], Some(&[0.0])); // needs 2 uvs, got 1
        assert!(b.build().is_err());
    }

    #[test]
    fn bad_magic_rejected() {
        let mut bytes = build_sample();
        bytes[0] = b'X';
        assert!(CorePack::parse(&bytes).is_err());
    }

    #[test]
    fn bad_version_rejected() {
        let mut bytes = build_sample();
        bytes[4] = 99;
        assert!(CorePack::parse(&bytes).is_err());
    }

    #[test]
    fn truncated_header_rejected() {
        let bytes = build_sample();
        assert!(CorePack::parse(&bytes[0..6]).is_err());
    }

    #[test]
    fn truncated_body_rejected() {
        let bytes = build_sample();
        // Cut the compressed body in half — inflate must fail, not panic.
        let cut = 10 + (bytes.len() - 10) / 2;
        assert!(CorePack::parse(&bytes[0..cut]).is_err());
    }

    #[test]
    fn oversized_declared_length_rejected() {
        let mut bytes = build_sample();
        // Force the declared uncompressed length to a huge value.
        bytes[6] = 0xff;
        bytes[7] = 0xff;
        bytes[8] = 0xff;
        bytes[9] = 0x7f;
        let res = CorePack::parse(&bytes);
        assert!(res.is_err());
    }

    #[test]
    fn empty_input_rejected() {
        assert!(CorePack::parse(&[]).is_err());
    }

    #[test]
    fn helper_metadata_round_trip() {
        let (positions, indices, _uvs) = sample_base();
        let mut b = CorePackBuilder::new();
        b.set_manifest(CorePackManifest::new("helper", "1"));
        b.set_base_mesh(&positions, &indices, None);
        b.set_helper_metadata(&[1, 2, 3, 4, 5]);
        let bytes = b.build().expect("build");
        let pack = CorePack::parse(&bytes).expect("parse");
        assert_eq!(pack.helper_metadata(), Some(&[1u8, 2, 3, 4, 5][..]));
    }

    #[test]
    fn target_names_auto_filled_when_empty() {
        let (positions, indices, _uvs) = sample_base();
        let mut manifest = CorePackManifest::new("auto", "1");
        manifest.target_names.clear();
        let mut b = CorePackBuilder::new();
        b.set_manifest(manifest);
        b.set_base_mesh(&positions, &indices, None);
        b.add_target("a", "cat", &[(0u32, [0.1, 0.0, 0.0])]);
        b.add_target("b", "cat", &[(1u32, [0.0, 0.1, 0.0])]);
        let bytes = b.build().expect("build");
        let pack = CorePack::parse(&bytes).expect("parse");
        assert_eq!(pack.manifest().target_names, vec!["a", "b"]);
    }

    #[test]
    fn size_target_reasonable() {
        // Sanity: a moderate mesh + targets stays comfortably compact.
        let n = 2000usize;
        let positions: Vec<f32> = (0..n * 3).map(|i| ((i % 97) as f32) * 0.01).collect();
        let indices: Vec<u32> = (0..(n as u32 - 1) * 3).map(|i| i % n as u32).collect();
        let mut b = CorePackBuilder::new();
        b.set_manifest(CorePackManifest::new("size", "1"));
        b.set_base_mesh(&positions, &indices, None);
        for t in 0..40 {
            let sparse: Vec<(u32, [f32; 3])> = (0..50)
                .map(|k| ((t * 50 + k) as u32 % n as u32, [0.001, 0.001, 0.001]))
                .collect();
            b.add_target(format!("t{t}"), "body", &sparse);
        }
        let bytes = b.build().expect("build");
        assert!(bytes.len() < 2 * 1024 * 1024, "pack size {}", bytes.len());
    }
}
