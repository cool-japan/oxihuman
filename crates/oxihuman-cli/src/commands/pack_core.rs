// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! `pack-core` — curate MakeHuman CC0 assets into an OHPK v1 core pack.
//!
//! This subcommand reads the upstream MakeHuman data tree (base mesh +
//! `.target` morph files fetched by `scripts/fetch_upstream_assets.sh`) and
//! bakes a single self-describing [`oxihuman_export::CorePack`] (OHPK v1)
//! binary that the WASM runtime can load with zero filesystem access.
//!
//! Two tiers are produced:
//!
//! * **core** — a small, curated, *adult-only* set (~40-64 targets) tuned to
//!   drive the BodyLab demo sliders (height / weight / muscle / age / gender /
//!   ethnic). The core pack must fit inside a hard byte budget (default 2 MiB)
//!   so it can be served as a single asset. When the freshly built pack is over
//!   budget, the lowest-priority targets are dropped (see `core_priority`)
//!   until it fits.
//! * **full** — every upstream target that survives the safety policy and the
//!   adult-only floor, written to `dist/` (git-ignored). No byte budget.
//!
//! # Vertex indexing
//!
//! `.target` files address the base mesh in raw `v`-line order, but the OHPK
//! base mesh stores vertices in the OBJ loader's face-first-occurrence order
//! with UV-seam splitting (~21.8k packed vertices vs ~19.2k raw v-lines for
//! the MakeHuman base). Every target's sparse deltas are therefore re-indexed
//! through the loader's raw→packed mapping — duplicating each delta across
//! all seam copies of its raw vertex — before encoding (see
//! `remap_targets_to_packed`). Packs built before this remapping existed
//! scatter-added their targets onto permuted vertices.
//!
//! # Target naming
//!
//! Every packed target keeps its *upstream-relative* name (relative to the
//! `.../data/targets/` root, without the `.target` extension), e.g.
//! `macrodetails/universal-female-young-maxmuscle-averageweight`. Its category
//! is inferred from the file stem with
//! [`oxihuman_morph::weight_curves::infer_category_from_name`], the exact same
//! mapping the runtime [`oxihuman_morph::HumanEngine`] uses, so the pack's
//! height/weight/muscle/age/gender/ethnic parameters keep working after the
//! pack is loaded.
//!
//! # Safety
//!
//! Every candidate is filtered through
//! [`oxihuman_core::policy::Policy::is_target_allowed`] (Standard profile) and,
//! independently of any policy profile, hard-excluded when its path contains an
//! explicit body-part token (see `EXPLICIT_TOKENS`). Baby/child macrodetail
//! files are excluded from *both* tiers: the shipped default is adult-only and
//! the manifest records `age_floor_years = 18.0`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, ensure, Context, Result};

use oxihuman_core::integrity::hash_bytes;
use oxihuman_core::parser::obj::parse_obj_with_mapping;
use oxihuman_core::parser::target::parse_target;
use oxihuman_core::policy::{Policy, PolicyProfile};
use oxihuman_export::{
    model_units_to_mm, CorePack, CorePackBuilder, CorePackFile, CorePackManifest,
    CorePackProvenance, MODEL_UNIT_MM,
};
use oxihuman_morph::weight_curves::infer_category_from_name;

// ── constants ───────────────────────────────────────────────────────────────

/// Upstream repository (canonical, without the `.git` suffix).
const UPSTREAM_REPO: &str = "https://github.com/makehumancommunity/makehuman";
/// Exact upstream commit the CC0 assets were taken from.
const UPSTREAM_COMMIT: &str = "1f508f6083b2f823dab15de924b3bde72e08d77c";
/// Upstream release tag matching [`UPSTREAM_COMMIT`].
const UPSTREAM_TAG: &str = "v1.3.0";

/// Modelled-age floor recorded in the manifest for both tiers (adult-only).
///
/// This is the *trigger* metadata written into the pack manifest; the runtime
/// enforces the floor using the authoritative policy constant of the same
/// value ([`oxihuman_core::policy::AGE_ADULT_FLOOR_YR`]), which we source here
/// so the two can never drift apart.
const AGE_FLOOR_YEARS: f32 = oxihuman_core::policy::AGE_ADULT_FLOOR_YR;

/// Default core-tier byte budget for the packed file on disk (2 MiB).
const DEFAULT_BUDGET_BYTES: usize = 2 * 1024 * 1024;

/// Substrings that hard-exclude a target regardless of the policy profile.
///
/// These cover explicit anatomy that must never ship in either tier. The check
/// is a case-insensitive substring match against the full upstream-relative
/// target name.
const EXPLICIT_TOKENS: &[&str] = &[
    "genital", "penis", "vagina", "breast", "nipple", "areola", "buttock", "cup", "firmness",
];

/// Age tokens that mark a macrodetail file as non-adult (excluded from both
/// tiers to honour the age floor).
const NON_ADULT_TOKENS: &[&str] = &["baby", "child"];

/// Upstream `targets/measure/` tape-measure targets curated into the core tier.
///
/// These are the MakeHuman *measure modifiers* authored precisely for
/// tape-measure-driven local girth adjustment (an `incr`/`decr` pair per
/// dimension). They are what makes the brief's `{chest 96, waist 82, hip 98}`
/// scenario reachable: the macro `universal-*` corners only span a narrow
/// body-girth envelope (chest ≲ 85 cm), whereas driving `measure-bust-circ-*`
/// pushes the chest cross-section out to a full bust.
///
/// Stored under category `"measure"` (see [`curated_category`]) so they stay
/// param-inert until the fit-refinement phase drives them *by name*; a demo
/// slider never touches them. Only the four circumference dimensions that feed
/// the readout are curated — the distance measures (`napetowaist`,
/// `waisttohip`, limb lengths) are ~10× larger and add no girth range, so they
/// are left to the full tier.
const MEASURE_GIRTH_TARGETS: &[&str] = &[
    "measure/measure-bust-circ-incr",
    "measure/measure-bust-circ-decr",
    "measure/measure-underbust-circ-incr",
    "measure/measure-underbust-circ-decr",
    "measure/measure-waist-circ-incr",
    "measure/measure-waist-circ-decr",
    "measure/measure-hips-circ-incr",
    "measure/measure-hips-circ-decr",
];

/// Category recorded for the tape-measure targets. Deliberately *not* a known
/// [`oxihuman_core::category::TargetCategory`] so the runtime keys them by name
/// at weight `0.0` (inert) instead of wiring them to a macro slider.
const MEASURE_CATEGORY: &str = "measure";

// ── tier selection ──────────────────────────────────────────────────────────

/// Which pack to build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tier {
    Core,
    Full,
}

impl Tier {
    fn parse(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "core" => Ok(Tier::Core),
            "full" => Ok(Tier::Full),
            other => bail!("--tier must be 'core' or 'full', got '{other}'"),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Tier::Core => "core",
            Tier::Full => "full",
        }
    }
}

// ── a candidate/selected target ─────────────────────────────────────────────

/// A target that has been selected and parsed, ready to feed the builder.
struct SelectedTarget {
    /// Upstream-relative name (relative to `.../data/targets/`, no extension),
    /// e.g. `macrodetails/universal-female-young-maxmuscle-averageweight`.
    name: String,
    /// Inferred morph category (drives the runtime weight function).
    category: String,
    /// Repo-relative path used for provenance/sha lookup, e.g.
    /// `makehuman/data/targets/macrodetails/universal-...target`.
    upstream_rel: String,
    /// SHA-256 (hex) of the source `.target` file bytes.
    sha256: String,
    /// Priority rank; lower is kept longer under the byte budget.
    priority: u32,
    /// Un-quantised sparse deltas in model units. Parsed in raw MakeHuman
    /// `v`-line order, then re-indexed (and seam-duplicated) into the packed
    /// base-mesh vertex order by [`remap_targets_to_packed`] before encoding.
    sparse: Vec<(u32, [f32; 3])>,
    /// Number of raw `v`-line vertices the source `.target` affected (before
    /// seam duplication) — recorded in the provenance sidecar.
    source_affected: usize,
}

// ── argument parsing ────────────────────────────────────────────────────────

struct Args {
    upstream: PathBuf,
    tier: Tier,
    out: PathBuf,
    manifest: PathBuf,
    budget_bytes: usize,
    report: PathBuf,
}

fn default_out_for(tier: Tier) -> PathBuf {
    match tier {
        Tier::Core => PathBuf::from("assets/packs/oxihuman-core-v1.ohpk"),
        Tier::Full => PathBuf::from("dist/oxihuman-full-v1.ohpk"),
    }
}

fn parse_args(args: &[String]) -> Result<Args> {
    let mut upstream = PathBuf::from("assets/upstream/makehuman");
    let mut tier = Tier::Core;
    let mut out: Option<PathBuf> = None;
    let mut manifest = PathBuf::from("assets/alpha_pack/oxihuman_assets.toml");
    let mut budget_bytes = DEFAULT_BUDGET_BYTES;
    let mut report = PathBuf::from("docs/bench/pack-reconstruction-error.md");

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--upstream" => {
                i += 1;
                upstream = PathBuf::from(arg_value(args, i, "--upstream")?);
            }
            "--tier" => {
                i += 1;
                tier = Tier::parse(arg_value(args, i, "--tier")?)?;
            }
            "--out" => {
                i += 1;
                out = Some(PathBuf::from(arg_value(args, i, "--out")?));
            }
            "--manifest" => {
                i += 1;
                manifest = PathBuf::from(arg_value(args, i, "--manifest")?);
            }
            "--budget-gzip-bytes" => {
                i += 1;
                budget_bytes = arg_value(args, i, "--budget-gzip-bytes")?
                    .parse()
                    .context("--budget-gzip-bytes must be a non-negative integer")?;
            }
            "--report" => {
                i += 1;
                report = PathBuf::from(arg_value(args, i, "--report")?);
            }
            other => bail!("pack-core: unknown option '{other}'"),
        }
        i += 1;
    }

    Ok(Args {
        upstream,
        tier,
        out: out.unwrap_or_else(|| default_out_for(tier)),
        manifest,
        budget_bytes,
        report,
    })
}

fn arg_value<'a>(args: &'a [String], i: usize, flag: &str) -> Result<&'a str> {
    args.get(i)
        .map(String::as_str)
        .ok_or_else(|| anyhow!("pack-core: {flag} requires a value"))
}

// ── entry point ─────────────────────────────────────────────────────────────

/// Build a core or full OHPK v1 pack from the upstream MakeHuman data tree.
pub fn cmd_pack_core(args: &[String]) -> Result<()> {
    let cfg = parse_args(args)?;

    let targets_root = cfg.upstream.join("makehuman/data/targets");
    let base_obj_path = cfg.upstream.join("makehuman/data/3dobjs/base.obj");

    if !base_obj_path.exists() {
        bail!(
            "base mesh not found at {} (did you run scripts/fetch_upstream_assets.sh?)",
            base_obj_path.display()
        );
    }
    if !targets_root.exists() {
        bail!("targets root not found at {}", targets_root.display());
    }

    // Per-source SHA-256 map, keyed by repo-relative path, read from the
    // upstream manifest when present (fallback: compute from file bytes).
    let sha_map = load_upstream_sha_map(&cfg.upstream);

    // 1. Select + parse targets for the chosen tier.
    let mut selected = match cfg.tier {
        Tier::Core => select_core_targets(&cfg.upstream, &targets_root, &sha_map)?,
        Tier::Full => select_full_targets(&cfg.upstream, &targets_root, &sha_map)?,
    };
    if selected.is_empty() {
        bail!(
            "no targets selected for tier '{}' — nothing to pack",
            cfg.tier.as_str()
        );
    }
    // Keep a stable, priority-then-name order so budget trimming is deterministic.
    selected.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then_with(|| a.name.cmp(&b.name))
    });

    // 2. Parse the base mesh (reuse the shared OBJ loader — do not rewrite),
    //    keeping the raw-v-line → packed-vertex mapping the loader produced.
    let base_src = std::fs::read_to_string(&base_obj_path)
        .with_context(|| format!("reading base OBJ: {}", base_obj_path.display()))?;
    let parsed_base = parse_obj_with_mapping(&base_src).context("parsing base OBJ")?;
    let base = &parsed_base.mesh;
    let base_positions: Vec<f32> = base
        .positions
        .iter()
        .flat_map(|p| [p[0], p[1], p[2]])
        .collect();
    let base_uvs: Vec<f32> = base.uvs.iter().flat_map(|uv| [uv[0], uv[1]]).collect();
    let n_verts = base.positions.len();
    let base_sha = sha_map
        .get("makehuman/data/3dobjs/base.obj")
        .cloned()
        .unwrap_or_else(|| hash_bytes(base_src.as_bytes()));

    println!(
        "pack-core: tier={} base={} verts ({} raw v-lines), {} tris; {} candidate targets",
        cfg.tier.as_str(),
        n_verts,
        parsed_base.raw_to_packed.len(),
        base.indices.len() / 3,
        selected.len()
    );

    // 2b. Re-index every target's sparse deltas from raw MakeHuman v-line
    //     order into the packed vertex order of the base mesh (the order the
    //     WASM engine scatter-adds against), duplicating each delta across
    //     all UV-seam copies of its raw vertex.
    let (raw_entries, packed_entries) =
        remap_targets_to_packed(&mut selected, &parsed_base.raw_to_packed, n_verts)?;
    println!(
        "pack-core: remapped {raw_entries} raw sparse entries -> {packed_entries} packed entries (seam duplication)"
    );

    // 3. Build, enforcing the byte budget for the core tier by dropping the
    //    lowest-priority targets until the packed file fits.
    let bytes = build_within_budget(
        &base_positions,
        &base.indices,
        &base_uvs,
        &base_sha,
        &mut selected,
        cfg.tier,
        cfg.budget_bytes,
    )?;

    // 4. Write the pack file.
    if let Some(parent) = cfg.out.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating output dir {}", parent.display()))?;
    }
    std::fs::write(&cfg.out, &bytes)
        .with_context(|| format!("writing pack: {}", cfg.out.display()))?;
    let pack_sha = hash_bytes(&bytes);

    println!(
        "pack-core: wrote {} ({} bytes, {} targets) sha256={}",
        cfg.out.display(),
        bytes.len(),
        selected.len(),
        pack_sha
    );
    if cfg.tier == Tier::Core {
        let pct = (bytes.len() as f64 / cfg.budget_bytes as f64) * 100.0;
        println!(
            "pack-core: core budget {}/{} bytes ({:.1}% of budget)",
            bytes.len(),
            cfg.budget_bytes,
            pct
        );
    }

    // 5. Machine-readable provenance sidecar.
    let provenance_path = cfg.out.with_extension("provenance.json");
    write_provenance_json(
        &provenance_path,
        cfg.tier,
        &cfg.out,
        &pack_sha,
        bytes.len(),
        &base_sha,
        n_verts,
        &selected,
    )?;
    println!("pack-core: wrote provenance {}", provenance_path.display());

    // 6. Honest reconstruction-error report (core tier drives the published
    //    number; the doc path is shared so we only write it for core).
    if cfg.tier == Tier::Core {
        write_reconstruction_report(&cfg.report, &bytes, &selected, &cfg)?;
        println!(
            "pack-core: wrote reconstruction report {}",
            cfg.report.display()
        );
    }

    // Note the manifest reference used (updated separately in the asset toml).
    let _ = &cfg.manifest;

    Ok(())
}

// ── raw → packed index remapping ────────────────────────────────────────────

/// Re-index every selected target's sparse deltas from raw MakeHuman
/// `v`-line order into the packed vertex order of the base mesh.
///
/// `.target` files address the base mesh's raw `v`-line order (`0..n_raw`),
/// but the OHPK base mesh stores vertices in the OBJ loader's
/// face-first-occurrence order with UV-seam splitting — a permutation *and*
/// duplication of the raw order. Each raw delta is therefore duplicated
/// across **all** packed copies of its raw vertex (seam copies are the same
/// geometric point, so they must move together or the seam tears).
///
/// Returns `(total_raw_entries, total_packed_entries)`. Fails when a target
/// addresses a raw vertex the base mesh does not have, or when any produced
/// packed index would fall outside `0..n_packed_verts`.
fn remap_targets_to_packed(
    selected: &mut [SelectedTarget],
    raw_to_packed: &[Vec<u32>],
    n_packed_verts: usize,
) -> Result<(usize, usize)> {
    let mut total_raw = 0usize;
    let mut total_packed = 0usize;
    for t in selected.iter_mut() {
        let mut packed: Vec<(u32, [f32; 3])> =
            Vec::with_capacity(t.sparse.len() + t.sparse.len() / 4);
        let mut unreferenced = 0usize;
        for &(raw_vid, delta) in &t.sparse {
            let copies = raw_to_packed.get(raw_vid as usize).ok_or_else(|| {
                anyhow!(
                    "target '{}' addresses raw vertex {} but the base mesh has only {} v-lines",
                    t.name,
                    raw_vid,
                    raw_to_packed.len()
                )
            })?;
            if copies.is_empty() {
                // Raw vertex never referenced by any face: not part of the
                // packed mesh, so the delta has no geometry to land on.
                unreferenced += 1;
                continue;
            }
            for &p in copies {
                ensure!(
                    (p as usize) < n_packed_verts,
                    "target '{}': packed index {} out of range ({} packed verts)",
                    t.name,
                    p,
                    n_packed_verts
                );
                packed.push((p, delta));
            }
        }
        // Seam duplication only ever grows the entry count; every referenced
        // raw vertex must land on at least one packed vertex.
        ensure!(
            packed.len() >= t.sparse.len() - unreferenced,
            "target '{}': packed entries {} < referenced raw entries {}",
            t.name,
            packed.len(),
            t.sparse.len() - unreferenced
        );
        total_raw += t.sparse.len();
        total_packed += packed.len();
        t.sparse = packed;
    }
    Ok((total_raw, total_packed))
}

// ── build with budget ───────────────────────────────────────────────────────

/// Build the pack, trimming lowest-priority targets until it fits the budget
/// (core tier only). `selected` is truncated in place to reflect what shipped.
#[allow(clippy::too_many_arguments)]
fn build_within_budget(
    base_positions: &[f32],
    base_indices: &[u32],
    base_uvs: &[f32],
    base_sha: &str,
    selected: &mut Vec<SelectedTarget>,
    tier: Tier,
    budget_bytes: usize,
) -> Result<Vec<u8>> {
    loop {
        let manifest = build_manifest(tier, base_sha, selected);
        let bytes = build_pack_bytes(base_positions, base_indices, base_uvs, selected, manifest)?;

        if tier == Tier::Full || bytes.len() <= budget_bytes {
            return Ok(bytes);
        }

        // Over budget: drop the single lowest-priority (last) target and retry.
        // `selected` is sorted priority-ascending, so the tail is lowest.
        match selected.pop() {
            Some(dropped) => {
                println!(
                    "pack-core: over budget ({} > {}), dropping '{}' (priority {})",
                    bytes.len(),
                    budget_bytes,
                    dropped.name,
                    dropped.priority
                );
            }
            None => bail!(
                "pack-core: base mesh alone ({} bytes) exceeds the {} byte budget",
                bytes.len(),
                budget_bytes
            ),
        }
        if selected.is_empty() {
            bail!("pack-core: could not fit any targets within the byte budget");
        }
    }
}

fn build_pack_bytes(
    base_positions: &[f32],
    base_indices: &[u32],
    base_uvs: &[f32],
    selected: &[SelectedTarget],
    manifest: CorePackManifest,
) -> Result<Vec<u8>> {
    let mut builder = CorePackBuilder::new();
    let uvs = if base_uvs.len() == base_positions.len() / 3 * 2 && !base_uvs.is_empty() {
        Some(base_uvs)
    } else {
        None
    };
    builder.set_base_mesh(base_positions, base_indices, uvs);
    builder.set_manifest(manifest);
    for t in selected {
        builder.add_target(t.name.clone(), t.category.clone(), &t.sparse);
    }
    builder.build().context("serialising OHPK core pack")
}

fn build_manifest(tier: Tier, base_sha: &str, selected: &[SelectedTarget]) -> CorePackManifest {
    let mut files = Vec::with_capacity(selected.len() + 1);
    files.push(CorePackFile {
        name: "makehuman/data/3dobjs/base.obj".to_string(),
        sha256: base_sha.to_string(),
    });
    for t in selected {
        files.push(CorePackFile {
            name: t.upstream_rel.clone(),
            sha256: t.sha256.clone(),
        });
    }

    let mut categories: Vec<String> = selected.iter().map(|t| t.category.clone()).collect();
    categories.sort();
    categories.dedup();

    let name = match tier {
        Tier::Core => "OxiHuman Core Pack",
        Tier::Full => "OxiHuman Full Pack",
    };

    CorePackManifest {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        license: "CC0-1.0".to_string(),
        provenance: CorePackProvenance {
            upstream_repo: UPSTREAM_REPO.to_string(),
            upstream_commit: UPSTREAM_COMMIT.to_string(),
            files,
        },
        age_floor_years: Some(AGE_FLOOR_YEARS),
        categories,
        target_names: selected.iter().map(|t| t.name.clone()).collect(),
    }
}

// ── core-tier curation ──────────────────────────────────────────────────────

/// Priority rank for a core target (lower is kept longer under the budget).
///
/// The whole-body MakeHuman *height* and *ethnic* macro targets each touch all
/// ~19k vertices, so each costs roughly twice a `universal` muscle/weight
/// target (which touches only the ~8-11k torso/limb vertices). Only a bounded
/// number of macro targets fit inside the 2 MiB budget, so the ordering below
/// front-loads a *minimal essential set* that makes every demo slider dimension
/// work, then fills the remaining budget with the cheaper `universal` corners
/// to maximise coverage of the muscle/weight/gender/age space. Highest priority
/// (kept longest) first:
///
///   0. **Essential coverage** — the young min/max height corners (height
///      slider, 4) plus one female blend per ethnicity (ethnic selector, 3),
///      *and* the eight `measure/` girth targets (4). Together these light up
///      every demo slider **and** open the tape-measure girth envelope the
///      brief scenario needs; the girth pair is the headline feature of the
///      v2 pack, so it ranks with the essentials and is never trimmed.
///   1. `universal-*` **extremes** — muscle ∈ {min,max} *and* weight ∈
///      {min,max}: the 16 corners spanning muscle/weight for both genders and
///      both adult ages. Cheap (~half a macro target) and high value.
///   2. `universal-*` remaining — the `average` muscle/weight in-betweens (20);
///      fill as budget allows.
///   3. Remaining young ethnic blends (the male trio, 3).
///   4. Old height corners (4).
///   5. Old ethnic blends (6).
///   6. `stomach|hip|torso` adult-neutral waist/hip polish (8) — dropped
///      first; the `measure/` targets supersede them for girth shaping.
fn core_priority(rel_name: &str) -> u32 {
    if rel_name.starts_with("measure/") {
        // Tape-measure girth targets: the v2 headline feature, kept longest.
        0
    } else if rel_name.starts_with("macrodetails/height/") {
        // Young corners are the essential height slider; old corners are extra.
        if rel_name.contains("-young-") {
            0
        } else {
            4
        }
    } else if rel_name.starts_with("macrodetails/universal-") {
        let is_extreme = !rel_name.contains("averagemuscle") && !rel_name.contains("averageweight");
        if is_extreme {
            1
        } else {
            2
        }
    } else if rel_name.starts_with("macrodetails/") {
        // Ethnic adult blend: `{ethnic}-{gender}-{young|old}`.
        if rel_name.ends_with("-old") {
            5
        } else if rel_name.contains("-female-") {
            0 // one female blend per ethnicity = the essential ethnic set
        } else {
            3 // remaining young (male) ethnic blends
        }
    } else {
        6
    }
}

/// Enumerate the curated relative names (without `.target`) for the core tier.
fn core_relative_names() -> Vec<String> {
    let genders = ["female", "male"];
    let ages = ["young", "old"];
    let muscles = ["minmuscle", "averagemuscle", "maxmuscle"];
    let weights = ["minweight", "averageweight", "maxweight"];
    let ethnics = ["african", "asian", "caucasian"];
    let heights = ["minheight", "maxheight"];

    let mut names = Vec::new();

    // Universal muscle/weight corners (adult ages only): 2*2*3*3 = 36.
    for g in genders {
        for a in ages {
            for m in muscles {
                for w in weights {
                    names.push(format!("macrodetails/universal-{g}-{a}-{m}-{w}"));
                }
            }
        }
    }

    // Ethnic adult blends (young + old): 3*2*2 = 12.
    for e in ethnics {
        for g in genders {
            for a in ages {
                names.push(format!("macrodetails/{e}-{g}-{a}"));
            }
        }
    }

    // Height min/max corners at neutral muscle/weight: 2*2*2 = 8.
    for g in genders {
        for a in ages {
            for h in heights {
                names.push(format!(
                    "macrodetails/height/{g}-{a}-averagemuscle-averageweight-{h}"
                ));
            }
        }
    }

    // Tape-measure girth targets (essential, priority 0): the bust / underbust
    // / waist / hips incr+decr pairs that open the girth envelope. 8.
    for n in MEASURE_GIRTH_TARGETS {
        names.push((*n).to_string());
    }

    // Adult-neutral waist / hip shaping polish (dropped first under budget): 8.
    // Superseded by the `measure/` girth targets above; retained only so the
    // full tier / older curations stay comparable.
    for n in [
        "stomach/stomach-tone-decr",
        "stomach/stomach-tone-incr",
        "stomach/stomach-navel-in",
        "stomach/stomach-navel-out",
        "hip/hip-scale-horiz-decr",
        "hip/hip-scale-horiz-incr",
        "torso/torso-scale-horiz-decr",
        "torso/torso-scale-horiz-incr",
    ] {
        names.push(n.to_string());
    }

    names
}

/// Category to record for a curated target. Tape-measure targets under
/// `measure/` are forced to [`MEASURE_CATEGORY`] so the runtime keeps them
/// param-inert (driven by name during fit refinement); everything else uses
/// the same name-inference the runtime engine applies.
fn curated_category(rel_name: &str, stem: &str) -> String {
    if rel_name.starts_with("measure/") {
        MEASURE_CATEGORY.to_string()
    } else {
        infer_category_from_name(stem).as_str().to_string()
    }
}

fn select_core_targets(
    upstream: &Path,
    targets_root: &Path,
    sha_map: &BTreeMap<String, String>,
) -> Result<Vec<SelectedTarget>> {
    let policy = Policy::new(PolicyProfile::Standard);
    let mut out = Vec::new();
    let mut missing = 0usize;

    for rel_name in core_relative_names() {
        let path = targets_root.join(format!("{rel_name}.target"));
        if !path.exists() {
            missing += 1;
            eprintln!("pack-core: warning: curated target not found, skipping: {rel_name}");
            continue;
        }
        if let Some(sel) = try_select(
            upstream,
            &path,
            &rel_name,
            &policy,
            core_priority(&rel_name),
            sha_map,
        )? {
            out.push(sel);
        }
    }

    if missing > 0 {
        eprintln!("pack-core: {missing} curated core target(s) were missing upstream");
    }
    Ok(out)
}

// ── full-tier curation ──────────────────────────────────────────────────────

fn select_full_targets(
    upstream: &Path,
    targets_root: &Path,
    sha_map: &BTreeMap<String, String>,
) -> Result<Vec<SelectedTarget>> {
    let policy = Policy::new(PolicyProfile::Standard);
    let mut paths = Vec::new();
    collect_target_files(targets_root, &mut paths);
    paths.sort();

    let mut out = Vec::new();
    for path in &paths {
        let rel_name = match path.strip_prefix(targets_root) {
            Ok(p) => rel_without_ext(p),
            Err(_) => continue,
        };
        if let Some(sel) = try_select(upstream, path, &rel_name, &policy, 0, sha_map)? {
            out.push(sel);
        }
    }
    Ok(out)
}

fn collect_target_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_target_files(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("target") {
                out.push(path);
            }
        }
    }
}

// ── shared selection helper ─────────────────────────────────────────────────

/// Apply the safety filters, parse, and build a [`SelectedTarget`]; returns
/// `Ok(None)` when the candidate is filtered out or carries no deltas.
fn try_select(
    upstream: &Path,
    path: &Path,
    rel_name: &str,
    policy: &Policy,
    priority: u32,
    sha_map: &BTreeMap<String, String>,
) -> Result<Option<SelectedTarget>> {
    let lower = rel_name.to_lowercase();

    // Hard safety exclusions (independent of policy profile).
    if EXPLICIT_TOKENS.iter().any(|tok| lower.contains(tok)) {
        return Ok(None);
    }
    // Adult-only floor: no baby/child files in either tier.
    if NON_ADULT_TOKENS.iter().any(|tok| lower.contains(tok)) {
        return Ok(None);
    }
    // Policy allowlist / blocked-tag enforcement.
    if !policy.is_target_allowed(rel_name, &[]) {
        return Ok(None);
    }

    let stem = Path::new(rel_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(rel_name);
    let category = curated_category(rel_name, stem);

    let src = std::fs::read_to_string(path)
        .with_context(|| format!("reading target {}", path.display()))?;
    let parsed =
        parse_target(stem, &src).with_context(|| format!("parsing target {}", path.display()))?;
    if parsed.deltas.is_empty() {
        return Ok(None);
    }
    let sparse: Vec<(u32, [f32; 3])> = parsed
        .deltas
        .iter()
        .map(|d| (d.vid, [d.dx, d.dy, d.dz]))
        .collect();

    let upstream_rel = path
        .strip_prefix(upstream)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| rel_name.to_string());
    let sha256 = sha_map
        .get(&upstream_rel)
        .cloned()
        .unwrap_or_else(|| hash_bytes(src.as_bytes()));

    let source_affected = sparse.len();
    Ok(Some(SelectedTarget {
        name: rel_name.to_string(),
        category,
        upstream_rel,
        sha256,
        priority,
        sparse,
        source_affected,
    }))
}

fn rel_without_ext(p: &Path) -> String {
    let s = p.to_string_lossy().replace('\\', "/");
    s.strip_suffix(".target").map(str::to_string).unwrap_or(s)
}

// ── upstream sha map ────────────────────────────────────────────────────────

/// Load the per-file SHA-256 map from `assets/upstream/UPSTREAM_MANIFEST.json`
/// (the sibling of the upstream data dir). Missing/malformed → empty map, and
/// callers fall back to hashing the file bytes directly.
fn load_upstream_sha_map(upstream: &Path) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let manifest_path = match upstream.parent() {
        Some(parent) => parent.join("UPSTREAM_MANIFEST.json"),
        None => return map,
    };
    let Ok(text) = std::fs::read_to_string(&manifest_path) else {
        return map;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return map;
    };
    if let Some(files) = value.get("files").and_then(|f| f.as_array()) {
        for f in files {
            if let (Some(path), Some(sha)) = (
                f.get("path").and_then(|p| p.as_str()),
                f.get("sha256").and_then(|s| s.as_str()),
            ) {
                map.insert(path.to_string(), sha.to_string());
            }
        }
    }
    map
}

// ── provenance sidecar ──────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn write_provenance_json(
    path: &Path,
    tier: Tier,
    pack_path: &Path,
    pack_sha: &str,
    pack_bytes: usize,
    base_sha: &str,
    base_verts: usize,
    selected: &[SelectedTarget],
) -> Result<()> {
    let targets: Vec<serde_json::Value> = selected
        .iter()
        .map(|t| {
            serde_json::json!({
                "name": t.name,
                "category": t.category,
                "source": t.upstream_rel,
                "sha256": t.sha256,
                // Packed entries actually encoded (after seam duplication)…
                "affected_verts": t.sparse.len(),
                // …and the raw v-line vertices the source `.target` affected.
                "source_affected_verts": t.source_affected,
            })
        })
        .collect();

    let doc = serde_json::json!({
        "pack": {
            "name": pack_path.file_name().map(|n| n.to_string_lossy().to_string()),
            "tier": tier.as_str(),
            "format": "OHPK v1",
            "sha256": pack_sha,
            "bytes": pack_bytes,
            "base_vertex_count": base_verts,
            "target_count": selected.len(),
            "age_floor_years": AGE_FLOOR_YEARS,
            "license": "CC0-1.0",
        },
        "upstream": {
            "repo": UPSTREAM_REPO,
            "commit": UPSTREAM_COMMIT,
            "tag": UPSTREAM_TAG,
            "base_mesh": {
                "path": "makehuman/data/3dobjs/base.obj",
                "sha256": base_sha,
            },
        },
        "targets": targets,
    });

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating provenance dir {}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(&doc).context("serialising provenance JSON")?;
    std::fs::write(path, text).with_context(|| format!("writing provenance {}", path.display()))?;
    Ok(())
}

// ── reconstruction-error report ─────────────────────────────────────────────

/// Per-target reconstruction error, recomputed by comparing the original sparse
/// deltas against the pack's de-quantised deltas.
struct ErrRow {
    name: String,
    affected: usize,
    max_err_mm: f32,
    rms_err_mm: f32,
}

fn write_reconstruction_report(
    path: &Path,
    bytes: &[u8],
    selected: &[SelectedTarget],
    cfg: &Args,
) -> Result<()> {
    let pack = CorePack::parse(bytes).context("re-parsing built pack for error report")?;

    // Map original deltas by target name for comparison.
    let mut orig: BTreeMap<&str, &Vec<(u32, [f32; 3])>> = BTreeMap::new();
    for t in selected {
        orig.insert(t.name.as_str(), &t.sparse);
    }

    let mut rows: Vec<ErrRow> = Vec::with_capacity(pack.targets().len());
    for t in pack.targets() {
        let Some(original) = orig.get(t.name()) else {
            continue;
        };
        // Original deltas indexed by vid (parse_target sorts ascending; the
        // pack also sorts ascending, so a position zip is safe, but we index by
        // vid to be robust).
        let mut orig_by_vid: BTreeMap<u32, [f32; 3]> = BTreeMap::new();
        for (vid, d) in original.iter() {
            orig_by_vid.insert(*vid, *d);
        }

        let dequant = t.sparse();
        let mut max_err = 0.0f32;
        let mut sq_sum = 0.0f64;
        let mut comps = 0usize;
        for (vid, got) in &dequant {
            let want = orig_by_vid.get(vid).copied().unwrap_or([0.0; 3]);
            for k in 0..3 {
                let e = (got[k] - want[k]).abs();
                if e > max_err {
                    max_err = e;
                }
                sq_sum += (e as f64) * (e as f64);
                comps += 1;
            }
        }
        let rms_units = if comps > 0 {
            (sq_sum / comps as f64).sqrt() as f32
        } else {
            0.0
        };
        rows.push(ErrRow {
            name: t.name().to_string(),
            affected: t.len(),
            max_err_mm: model_units_to_mm(max_err),
            rms_err_mm: model_units_to_mm(rms_units),
        });
    }

    // Worst case by max error.
    rows.sort_by(|a, b| {
        b.max_err_mm
            .partial_cmp(&a.max_err_mm)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let worst = rows.first();

    let report = pack.quantization_report();

    let mut md = String::new();
    md.push_str("# OxiHuman Core Pack — Reconstruction Error\n\n");
    md.push_str(
        "Honest quantisation error for the shipped **core** OHPK v1 pack. Each\n\
         morph target's sparse deltas are stored as `i16` max-abs quantised\n\
         values; the numbers below are recomputed by de-quantising the packed\n\
         pack and comparing against the original `.target` deltas.\n\n",
    );
    md.push_str(&format!(
        "* Pack: `{}`\n* Targets: {}\n* Base vertices: {}\n* Model units: decimetres (1 unit = {} mm)\n* Base-mesh max position error: {:.6} mm\n",
        cfg.out.display(),
        rows.len(),
        pack.vertex_count(),
        MODEL_UNIT_MM,
        report.base_pos_max_error_mm,
    ));
    if let Some(uv) = report.base_uv_max_error {
        md.push_str(&format!(
            "* Base-mesh max UV error: {uv:.6} (dimensionless)\n"
        ));
    }
    md.push('\n');

    if let Some(w) = worst {
        md.push_str(&format!(
            "**Worst-case target:** `{}` — max {:.4} mm, RMS {:.4} mm over {} affected vertices.\n\n",
            w.name, w.max_err_mm, w.rms_err_mm, w.affected
        ));
    }

    md.push_str("| Target | Affected verts | Max err (mm) | RMS err (mm) |\n");
    md.push_str("|---|---:|---:|---:|\n");
    for (i, r) in rows.iter().enumerate() {
        let marker = if i == 0 { " ⚠️" } else { "" };
        md.push_str(&format!(
            "| `{}`{} | {} | {:.4} | {:.4} |\n",
            r.name, marker, r.affected, r.max_err_mm, r.rms_err_mm
        ));
    }

    md.push_str("\n## Reproduce\n\n```sh\n");
    md.push_str(&format!(
        "oxihuman pack-core --tier core --upstream {} --out {} --report {}\n",
        cfg.upstream.display(),
        cfg.out.display(),
        path.display()
    ));
    md.push_str("```\n");

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating report dir {}", parent.display()))?;
    }
    std::fs::write(path, md).with_context(|| format!("writing report {}", path.display()))?;
    Ok(())
}

// ── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_export::CorePack;

    fn upstream_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/upstream/makehuman")
    }

    fn shipped_pack_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs/oxihuman-core-v1.ohpk")
    }

    fn make_target(name: &str, sparse: Vec<(u32, [f32; 3])>) -> SelectedTarget {
        let source_affected = sparse.len();
        SelectedTarget {
            name: name.to_string(),
            category: "measure".to_string(),
            upstream_rel: format!("makehuman/data/targets/{name}.target"),
            sha256: "0".repeat(64),
            priority: 0,
            sparse,
            source_affected,
        }
    }

    /// Raw `v`-line positions parsed straight from OBJ text (independent of
    /// the packed loader under test).
    fn raw_v_positions(src: &str) -> Vec<[f32; 3]> {
        src.lines()
            .map(str::trim)
            .filter_map(|l| l.strip_prefix("v "))
            .map(|rest| {
                let mut it = rest.split_whitespace();
                let mut p = [0.0f32; 3];
                for c in p.iter_mut() {
                    *c = it.next().and_then(|s| s.parse().ok()).unwrap_or(f32::NAN);
                }
                p
            })
            .collect()
    }

    #[test]
    fn remap_duplicates_across_seam_copies() {
        // raw 0 → packed 0; raw 1 → packed {1, 2} (seam); raw 2 → unreferenced;
        // raw 3 → packed 3.
        let mapping: Vec<Vec<u32>> = vec![vec![0], vec![1, 2], vec![], vec![3]];
        let d0 = [0.1f32, 0.0, 0.0];
        let d1 = [0.0f32, 0.2, 0.0];
        let d2 = [0.0f32, 0.0, 0.3];
        let d3 = [0.4f32, 0.5, 0.6];
        let mut selected = vec![make_target("t", vec![(0, d0), (1, d1), (2, d2), (3, d3)])];
        let (raw, packed) =
            remap_targets_to_packed(&mut selected, &mapping, 4).expect("remap succeeds");
        assert_eq!(raw, 4);
        assert_eq!(packed, 4, "1 + 2 seam copies + 0 (unreferenced) + 1");
        assert_eq!(
            selected[0].sparse,
            vec![(0, d0), (1, d1), (2, d1), (3, d3)],
            "seam copies must receive the same delta; unreferenced raw verts drop"
        );
        assert_eq!(selected[0].source_affected, 4);

        // A raw index beyond the mapping is a hard error, not a silent skip.
        let mut bad = vec![make_target("bad", vec![(9, d0)])];
        assert!(remap_targets_to_packed(&mut bad, &mapping, 4).is_err());
    }

    /// Build-path invariant on real upstream data (skipped on a lean
    /// checkout): for every target and every packed sparse entry, the pack's
    /// base position at that packed index equals the raw source vertex's
    /// position (the delta lands on geometry identical to what the raw
    /// `.target` addressed), and seam duplication never loses entries.
    #[test]
    fn built_pack_deltas_land_on_source_geometry() {
        let upstream = upstream_root();
        let base_path = upstream.join("makehuman/data/3dobjs/base.obj");
        let Ok(base_src) = std::fs::read_to_string(&base_path) else {
            eprintln!("upstream base.obj absent — skipping pack-build invariant test");
            return;
        };
        let parsed = parse_obj_with_mapping(&base_src).expect("parse base");
        let raw_pos = raw_v_positions(&base_src);
        let n_verts = parsed.mesh.positions.len();

        // Reverse map: packed vertex → raw v-line source.
        let mut packed_to_raw = vec![usize::MAX; n_verts];
        for (raw, copies) in parsed.raw_to_packed.iter().enumerate() {
            for &p in copies {
                packed_to_raw[p as usize] = raw;
            }
        }

        // Select two upstream targets through the production path: one
        // sharply localised measure target + one whole-body macro corner.
        let targets_root = upstream.join("makehuman/data/targets");
        let policy = Policy::new(PolicyProfile::Standard);
        let sha_map = BTreeMap::new();
        let mut selected = Vec::new();
        for rel in [
            "measure/measure-waist-circ-incr",
            "macrodetails/universal-female-young-maxmuscle-averageweight",
        ] {
            let path = targets_root.join(format!("{rel}.target"));
            if !path.exists() {
                eprintln!("{rel}: upstream target absent — skipping invariant test");
                return;
            }
            let sel = try_select(&upstream, &path, rel, &policy, 0, &sha_map)
                .expect("select target")
                .expect("target not filtered");
            selected.push(sel);
        }
        let raw_affected: Vec<usize> = selected.iter().map(|t| t.source_affected).collect();
        let raw_deltas: Vec<BTreeMap<u32, [f32; 3]>> = selected
            .iter()
            .map(|t| t.sparse.iter().copied().collect())
            .collect();

        remap_targets_to_packed(&mut selected, &parsed.raw_to_packed, n_verts)
            .expect("remap succeeds");

        // Build + re-parse the pack exactly as `cmd_pack_core` does.
        let base_positions: Vec<f32> = parsed
            .mesh
            .positions
            .iter()
            .flat_map(|p| [p[0], p[1], p[2]])
            .collect();
        let base_uvs: Vec<f32> = parsed
            .mesh
            .uvs
            .iter()
            .flat_map(|uv| [uv[0], uv[1]])
            .collect();
        let manifest = build_manifest(Tier::Core, &"0".repeat(64), &selected);
        let bytes = build_pack_bytes(
            &base_positions,
            &parsed.mesh.indices,
            &base_uvs,
            &selected,
            manifest,
        )
        .expect("build pack");
        let pack = CorePack::parse(&bytes).expect("re-parse pack");

        let report = pack.quantization_report();
        let base_tol = report.base_pos_max_error_units + 1e-6;
        let pack_base = pack.base_positions();

        for (ti, t) in pack.targets().iter().enumerate() {
            let target_tol = t.max_abs_error_units() + 1e-6;
            let sparse = t.sparse();
            assert!(
                sparse.len() >= raw_affected[ti],
                "{}: packed affected {} < raw affected {}",
                t.name(),
                sparse.len(),
                raw_affected[ti]
            );
            for (packed_idx, delta) in &sparse {
                let p = *packed_idx as usize;
                assert!(p < n_verts, "{}: packed index {p} out of range", t.name());
                let raw = packed_to_raw[p];
                assert_ne!(
                    raw,
                    usize::MAX,
                    "{}: packed {p} has no raw source",
                    t.name()
                );
                // (a) The geometry at the packed index is the raw vertex.
                for k in 0..3 {
                    let got = pack_base[p * 3 + k];
                    let want = raw_pos[raw][k];
                    assert!(
                        (got - want).abs() <= base_tol,
                        "{}: base pos mismatch at packed {p} (raw {raw}) axis {k}: {got} vs {want}",
                        t.name()
                    );
                }
                // (b) The delta is the raw target's delta for that vertex.
                let want_delta = raw_deltas[ti]
                    .get(&(raw as u32))
                    .unwrap_or_else(|| panic!("{}: raw {raw} not in source target", t.name()));
                for k in 0..3 {
                    assert!(
                        (delta[k] - want_delta[k]).abs() <= target_tol,
                        "{}: delta mismatch at packed {p} (raw {raw}) axis {k}: {} vs {}",
                        t.name(),
                        delta[k],
                        want_delta[k]
                    );
                }
            }
        }
    }

    /// The shipped core pack must carry correctly remapped indices: every
    /// target's packed entries, pulled back through the loader mapping, must
    /// reproduce the upstream `.target` deltas exactly (within quantisation).
    /// Skipped when upstream assets or the shipped pack are absent.
    #[test]
    fn shipped_pack_indices_match_upstream_targets() {
        let upstream = upstream_root();
        let base_path = upstream.join("makehuman/data/3dobjs/base.obj");
        let pack_path = shipped_pack_path();
        let (Ok(base_src), Ok(pack_bytes)) = (
            std::fs::read_to_string(&base_path),
            std::fs::read(&pack_path),
        ) else {
            eprintln!("upstream base.obj or shipped pack absent — skipping shipped-pack test");
            return;
        };
        let parsed = parse_obj_with_mapping(&base_src).expect("parse base");
        let pack = CorePack::parse(&pack_bytes).expect("parse shipped pack");
        assert_eq!(pack.vertex_count(), parsed.mesh.positions.len());

        let targets_root = upstream.join("makehuman/data/targets");
        for t in pack.targets() {
            let src_path = targets_root.join(format!("{}.target", t.name()));
            let Ok(src) = std::fs::read_to_string(&src_path) else {
                panic!("{}: upstream source .target missing", t.name());
            };
            let raw = parse_target(t.name(), &src).expect("parse upstream target");
            // Expected packed sparse set: every raw delta duplicated across
            // its seam copies.
            let mut expected: BTreeMap<u32, [f32; 3]> = BTreeMap::new();
            for d in &raw.deltas {
                if let Some(copies) = parsed.raw_to_packed.get(d.vid as usize) {
                    for &p in copies {
                        expected.insert(p, [d.dx, d.dy, d.dz]);
                    }
                }
            }
            let got: BTreeMap<u32, [f32; 3]> = t.sparse().into_iter().collect();
            assert_eq!(
                got.len(),
                expected.len(),
                "{}: packed entry count mismatch (shipped pack predates index remapping?)",
                t.name()
            );
            let tol = t.max_abs_error_units() + 1e-6;
            for (idx, want) in &expected {
                let Some(gd) = got.get(idx) else {
                    panic!("{}: expected packed index {idx} missing", t.name());
                };
                for k in 0..3 {
                    assert!(
                        (gd[k] - want[k]).abs() <= tol,
                        "{}: delta mismatch at packed {idx} axis {k}: {} vs {}",
                        t.name(),
                        gd[k],
                        want[k]
                    );
                }
            }
        }
    }
}
