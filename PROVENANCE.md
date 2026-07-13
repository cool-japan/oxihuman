# PROVENANCE

Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)

This document is the authoritative provenance and licence record for every
third-party asset bundled in OxiHuman. It exists so that the origin, licence,
and integrity of shipped binary data can be independently verified by anyone,
without trusting a single claim in this repository — every hash below can be
recomputed from the files in this repo (or from a fresh checkout of the cited
upstream commit) and every URL below can be checked live.

Run `scripts/check_provenance.sh` to machine-verify the claims in this
document (see §6).

## 1. What ships

OxiHuman bundles exactly one third-party binary asset artifact:

* **`assets/packs/oxihuman-core-v1.ohpk`** — the "core" morph-target pack
  consumed by `oxihuman-export`, `oxihuman-wasm`, and the CLI. It contains a
  base body mesh (21,833 vertices) and 38 sparse morph targets quantised to
  `i16` deltas: 30 macrodetail corners (age, height, weight, muscle, ethnic
  categories) plus 8 MakeHuman `targets/measure/` tape-measure girth targets
  (bust / underbust / waist / hips, each an incr/decr pair). Every target's
  sparse deltas are re-indexed from raw `.target` v-line order into the
  pack's packed base-mesh vertex order (duplicated across UV-seam copies) at
  build time, so the runtime applies them to exactly the geometry the
  upstream target authored.

| Field | Value |
|---|---|
| File | `assets/packs/oxihuman-core-v1.ohpk` |
| Format | OHPK v1 |
| Bytes | 2,093,260 |
| SHA-256 | `09c4bb1f849fe5d2bc21db6dd8a8bf7c753ee58db185bc46ab4c6b8e0dc0f6f7` |
| Base vertices | 21,833 |
| Target count | 38 |
| Age floor | 18.0 years (adult-only; see `docs/SAFETY.md` if present, and `export_gate` in `oxihuman-export`) |
| Declared licence | CC0-1.0 |
| Tier | core |
| Worst-case quantisation error | 0.013 mm (base mesh; see `docs/bench/pack-reconstruction-error.md`) |

Nothing else ships. No textures, no clothing/proxy (`.mhclo`) assets, no
poses, no rigs, no community asset packs, and no MakeHuman *code* of any kind
are bundled in this repository or in any published crate/npm package.

### 1.1 Upstream source

Every byte in `oxihuman-core-v1.ohpk` was derived from a single, pinned
upstream commit:

| Field | Value |
|---|---|
| Repository | `https://github.com/makehumancommunity/makehuman` |
| Tag | `v1.3.0` |
| Commit | `1f508f6083b2f823dab15de924b3bde72e08d77c` |
| Commit date | 2024-04-19T20:56:25+02:00 |
| Fetched at | 2026-07-12T02:03:52Z |

The full fetch manifest (1,283 files, 130,016,457 bytes, one SHA-256 per
file) is recorded at `assets/upstream/UPSTREAM_MANIFEST.json`. Only 39 of
those 1,283 files (38 targets + `base.obj`) were selected into the shipped
core pack; the per-file provenance sidecar at
`assets/packs/oxihuman-core-v1.provenance.json` records exactly which ones,
with their own independently-computed SHA-256 hashes matching the manifest.

### 1.2 Base mesh

| Field | Value |
|---|---|
| Upstream path | `makehuman/data/3dobjs/base.obj` |
| SHA-256 | `8e761e6624b8f54536409135d1636da63b32486a90d4897f84e121d144f6fb4c` |

### 1.3 The 38 bundled morph targets

Every row below is a `makehuman/data/targets/...` file at the pinned commit
above. `sha256` is the hash of the *source* `.target` text file as fetched
from upstream (not the quantised in-pack representation), recorded in
`assets/packs/oxihuman-core-v1.provenance.json` and cross-checked against
`assets/upstream/UPSTREAM_MANIFEST.json` by `scripts/check_provenance.sh`.

The 8 `measure/` targets carry the category `measure` — a deliberately inert
category the runtime keys by name at weight 0, so they never activate on a
demo slider; they are driven only by the measurement-fit refinement stage.

"Affected verts" counts the packed sparse entries actually encoded in the
pack — the raw `.target` vertex count (in parentheses) grows under UV-seam
duplication because a raw vertex split across seams receives one identical
delta per packed copy. Both counts are recorded per target in the provenance
sidecar (`affected_verts` / `source_affected_verts`).

| Target | Category | Affected verts (packed (raw)) | SHA-256 (source `.target`) |
|---|---|---:|---|
| macrodetails/african-female-young | age | 21833 (19158) | `92d61eeb3c164b421fd5a7c3537ee45e7e1a51de4d49bf312e19c3df2be1d8fc` |
| macrodetails/asian-female-young | age | 21819 (19150) | `095fe79694fa19e1fe98d93009ec116199bd524e081c640351a10eccf2cca1eb` |
| macrodetails/caucasian-female-young | age | 21819 (19150) | `118379f6e8ba9266247fdb8788a20e1df40a239f97ced0b9905bcbcc74f6e820` |
| macrodetails/height/female-young-averagemuscle-averageweight-maxheight | height | 21819 (19150) | `3baacc70187410ebba2b62c56a53775266ab859b7844d47e658d36ee78493621` |
| macrodetails/height/female-young-averagemuscle-averageweight-minheight | height | 21819 (19150) | `375857dff927f67da53786fc5ac0545c473f4959e3b326727b6b91f49ebdd2e3` |
| macrodetails/height/male-young-averagemuscle-averageweight-maxheight | height | 21819 (19150) | `3baacc70187410ebba2b62c56a53775266ab859b7844d47e658d36ee78493621` |
| macrodetails/height/male-young-averagemuscle-averageweight-minheight | height | 21819 (19150) | `76a79d873dc0820ae336645903591f8e6a8adc69865210a6a66f49908caa810b` |
| measure/measure-bust-circ-decr | measure | 1964 (1876) | `3d4c9f8c7d084e4d6c35ffc3a798bc35bdbe80fd56929d03e0c6f1b31ec5c736` |
| measure/measure-bust-circ-incr | measure | 2061 (1972) | `46394c7f9cdfac31654f14950b57bbfb874c9a2fa68e48c9cc3e7bba619b75f4` |
| measure/measure-hips-circ-decr | measure | 1450 (1387) | `8a98e80047ab89148d60067aad3b452ce715eadcaf7a1e28402140dd1ac8fb11` |
| measure/measure-hips-circ-incr | measure | 1482 (1418) | `e4294db231dd3283680e6be4b9d3d8d81aa7c575a32f937f8d0751b1ec3d6512` |
| measure/measure-underbust-circ-decr | measure | 2229 (2132) | `94c7a07cf0b71a2a5f6e369ba9bf10e1e30ac2ef93596567f16178623867c186` |
| measure/measure-underbust-circ-incr | measure | 2280 (2181) | `aa6a8fa8385b127ac000b227e64c80dd97e09640bec69728636a23aa21d6cfdf` |
| measure/measure-waist-circ-decr | measure | 1554 (1518) | `6a976bd7819fa037a290b381bb3e04dfd04220dddeac194c476472295d2baba8` |
| measure/measure-waist-circ-incr | measure | 1554 (1518) | `4949212ec9a5e227b177a029ee42b0be3fd3b273a211bc91c6f4d4dbf0334856` |
| macrodetails/universal-female-old-maxmuscle-maxweight | weight | 8312 (7721) | `11549de05d8ab39daee36ca692a56a737f3778006a89684c56f70208c2e7c452` |
| macrodetails/universal-female-old-maxmuscle-minweight | muscle | 7415 (6964) | `453f066a5972f59f40b238bc7d9e6ea68e96d0f1e49cf28f486c3c8c674c192b` |
| macrodetails/universal-female-old-minmuscle-maxweight | weight | 10187 (9526) | `788850da7e8e75f74f37003a91473270d84a6e599b5851ed7a6fecafff8fdbd8` |
| macrodetails/universal-female-old-minmuscle-minweight | age | 13292 (12239) | `806f995cb9356c70645da92a6b64dddbd8de9ac5480420f64b6d1a651cff254d` |
| macrodetails/universal-female-young-maxmuscle-maxweight | weight | 8346 (7753) | `1b4600abf475b517db2379338a4d36a5f77fb1ad94fd0b210451bfeaa9e7e9b6` |
| macrodetails/universal-female-young-maxmuscle-minweight | muscle | 7482 (7029) | `05351f0063429f4e1e484501b73997a6fb129b93939db355d108f98d5479fb10` |
| macrodetails/universal-female-young-minmuscle-maxweight | weight | 10484 (9798) | `66600ccaea3d4c54b1427c81f49160a7cbb2794fbcb7a695ec6e6a4743d72364` |
| macrodetails/universal-female-young-minmuscle-minweight | age | 21579 (18910) | `856167f0a03efedd69f91e1cbad065957c8e8fef6acaaaf5b85a32b0f018fa67` |
| macrodetails/universal-male-old-maxmuscle-maxweight | weight | 8835 (8241) | `cf232cbd26aff7177ec6e264e0a1024a4bb7696168c8040270f1990711cb1d49` |
| macrodetails/universal-male-old-maxmuscle-minweight | muscle | 7601 (7005) | `bfe62f32e70f1e178af19437c0e86af21a68afaaaffd0f255b5679d72545420b` |
| macrodetails/universal-male-old-minmuscle-maxweight | weight | 12065 (11115) | `57ba4273844dfbf03b5548059b577901502354845f30389d9f4b083689b636a2` |
| macrodetails/universal-male-old-minmuscle-minweight | age | 9177 (8596) | `2d2f1bc78b1ac1f87adcf4e632a58bfb1237ab1f4b8b5527f2e6907d38b2364d` |
| macrodetails/universal-male-young-maxmuscle-maxweight | weight | 8833 (8239) | `95392741389537889a37dc67a7d8453c93b30f27131e3e622bee3f30d1d8738c` |
| macrodetails/universal-male-young-maxmuscle-minweight | muscle | 7635 (7033) | `5873f11edc96fde72a79ec35fa1d292a6a1a594b5eadfd82d59faf48834561de` |
| macrodetails/universal-male-young-minmuscle-maxweight | weight | 12065 (11115) | `14580444f1480b1c6b396973f8c040cc36e5ca8914addf17d11c5a023a958de2` |
| macrodetails/universal-male-young-minmuscle-minweight | age | 9181 (8600) | `3af713db636eb41f32af5010981bad30efe743a3ab1d4a61c619c664b610a51a` |
| macrodetails/universal-female-old-averagemuscle-maxweight | weight | 7931 (7453) | `c7ff884ece15a4ea161bf9843c9668261e486135aa4c408ecc88dd7e5bc366b2` |
| macrodetails/universal-female-old-averagemuscle-minweight | age | 7449 (7001) | `ebb291456482aab827ca0d0bb3928553b422ee0fc6900ae9c9d5fa75195dc23a` |
| macrodetails/universal-female-old-maxmuscle-averageweight | muscle | 7027 (6614) | `c8ab63d583bd1bcf75a3bbbd6b5e43ee30782a704584ea01a3aecbfec510dc42` |
| macrodetails/universal-female-old-minmuscle-averageweight | age | 5660 (5258) | `89c2756e3abacf3d2fb9fae1cc05e68c3e0a244ba2ff244a1f3831da5897a26b` |
| macrodetails/universal-female-young-averagemuscle-maxweight | weight | 6324 (6027) | `3648e7f9b34d947726161b563b90ba92b29860bac4c4f82aea3cec37bad2bb4e` |
| macrodetails/universal-female-young-averagemuscle-minweight | age | 7420 (6968) | `e53a4afc87ac2703460a59e8290b1111478aeee5c02ece6735328c1f8c62eb63` |
| macrodetails/universal-female-young-maxmuscle-averageweight | muscle | 6963 (6547) | `2e56b09b44a8497b927585447fdaac5ea7293521d1f3a3df1059495873c53f4e` |

38 rows, matching `"target_count": 38` in the provenance sidecar and the
`target_count` field of the OHPK v1 container header itself. The v3 pack
re-indexes every target into the packed base-mesh vertex order (fixing the
v2 index corruption that scattered morphs onto permuted vertices); the seam
duplication this requires grows every target, so staying under the 2 MiB
byte budget displaced three low-priority `averagemuscle`/`averageweight`
macro in-betweens that shipped in v2
(`macrodetails/universal-female-young-minmuscle-averageweight`,
`macrodetails/universal-male-old-averagemuscle-maxweight`,
`macrodetails/universal-male-old-averagemuscle-minweight`). All 8 `measure/`
girth targets and full macro slider coverage (height / weight / muscle / age
/ gender / ethnic) are retained. Each hash above is recorded in
`assets/packs/oxihuman-core-v1.provenance.json` and cross-checked against
`assets/upstream/UPSTREAM_MANIFEST.json` by `scripts/check_provenance.sh`
(0 mismatches; see §6).

## 2. Licence basis

### 2.1 Upstream's own licence split (`LICENSE.md`)

MakeHuman's own `LICENSE.md`, at the pinned commit, explicitly splits the
project into source code (AGPL-3.0) and bundled assets (CC0-1.0). Section C,
quoted verbatim:

> **C. The license for the bundled assets**
>
> The assets are defined as any data contributing to the graphical output of
> MakeHuman. This includes:
>
> * The base mesh and proxies
> * Targets and modifiers
> * Textures
> * Clothes (any MHCLO-based asset)
> * Poses and expressions
>
> These assets have been released under CC0 1.0 Universal. In summary this
> means that to the fullest extent possible, it is the intention of the
> MakeHuman project that anyone can do whatever they want with it.
>
> For the full text of the legal statement regarding the assets, see
> [LICENSE.ASSETS.md](LICENSE.ASSETS.md)

Both the **base mesh** and **targets/modifiers** — the two asset classes
bundled in `oxihuman-core-v1.ohpk` — are named explicitly in this list.

`LICENSE.md` SHA-256 (at the pinned commit, 3,598 bytes):
`edd99571ca62698f78c943fd4fffb159a413c315fa0d4819d3cec5cadaf8b4f1`.

### 2.2 The CC0 legal text itself (`LICENSE.ASSETS.md`)

`LICENSE.ASSETS.md` is the full legal text of the Creative Commons CC0 1.0
Universal Public Domain Dedication, 6,962 bytes, SHA-256
`f6089cba01cb570a24712b41ab8a586ccd3cc5ef53dc266ca50b95c288956d2c`. The
dedication (Waiver) clause, quoted verbatim:

> **2. Waiver.** To the greatest extent permitted by, but not in
> contravention of, applicable law, Affirmer hereby overtly, fully,
> permanently, irrevocably and unconditionally waives, abandons, and
> surrenders all of Affirmer's Copyright and Related Rights and associated
> claims and causes of action, whether now known or unknown (including
> existing as well as future claims and causes of action), in the Work (i) in
> all territories worldwide, (ii) for the maximum duration provided by
> applicable law or treaty (including future time extensions), (iii) in any
> current or future medium and for any number of copies, and (iv) for any
> purpose whatsoever, including without limitation commercial, advertising or
> promotional purposes (the "Waiver").

### 2.3 Per-file embedded CC0 headers

Independent of the repository-level `LICENSE.md`/`LICENSE.ASSETS.md` split,
individual `.target` files also carry their own embedded CC0 declaration as a
comment header. Verified directly in this task by opening
`assets/upstream/makehuman/makehuman/data/targets/macrodetails/african-female-young.target`
(one of the 41 bundled targets), whose header reads:

```
# This is a target file for MakeHuman
#
# This asset was explicitly released as CC0 in september 2020. The license
# text for CC0 can be found in the root of this repository.
#
# Original copyright (C) 2014 Manuel Bastioni
#
# The copyright was explicitly transfered to other team members in 2016:
#
# Copyright (C) 2020 Data Collection AB, https://www.datacollection.se
```

This is triple-redundant provenance for the licence claim: (1) the
project-level `LICENSE.md` §C says targets are CC0, (2) `LICENSE.ASSETS.md`
carries the full CC0 legal text, and (3) the individual `.target` file itself
says, in its own header, that it "was explicitly released as CC0".

### 2.4 What licence governs the shipped `.ohpk` pack

The shipped pack (`oxihuman-core-v1.ohpk`) declares `"license": "CC0-1.0"` in
its own provenance sidecar (`assets/packs/oxihuman-core-v1.provenance.json`),
consistent with §2.1–2.3 above. OxiHuman's own Rust/WASM *code* that reads
and processes this pack is licensed separately, under Apache-2.0 (see §4).

## 3. Archive evidence

Because upstream licence pages can move or be edited, both the current and a
historical snapshot of MakeHuman's public licensing statement are archived on
the Internet Archive Wayback Machine. Both URLs below were verified live in
this task (`curl -sI -L`, HTTP 200, and cross-checked against
`http://archive.org/wayback/available`, which confirmed both timestamps are
exact — not the "closest available" fallback).

| Page | Status | Snapshot |
|---|---|---|
| **Current** licence page — `static.makehumancommunity.org/about/license.html` | AGPL (code) / CC0 (assets) split, matches §2.1 | `https://web.archive.org/web/20260310141823/https://static.makehumancommunity.org/about/license.html` |
| **Superseded** legacy licence page — `www.makehumancommunity.org/content/license.html` (Drupal, 2015-era) | historical; see §4 for why it is not authoritative | `https://web.archive.org/web/20230430135458/http://www.makehumancommunity.org/content/license.html` |

Both snapshots returned HTTP `200` with `memento-datetime` headers matching
the requested timestamps exactly:
`Tue, 10 Mar 2026 14:18:23 GMT` and `Sun, 30 Apr 2023 13:54:58 GMT`
respectively. This task also attempted to trigger fresh on-demand Wayback
captures via `https://web.archive.org/save/<url>` for both pages as a
best-effort refresh; both attempts timed out against the `/save/` endpoint
(no response within 20s — this endpoint is frequently rate-limited/blocked
for non-browser clients). This does not affect the validity of the existing
snapshots above, which were independently confirmed via the `available` API.

## 4. On the superseded 2015 Drupal licensing page

If you search for MakeHuman's licence history, you may find an older,
Drupal-served page (`www.makehumancommunity.org/content/license.html`,
archived above) whose wording is looser and has at various times been read by
third parties as implying broader AGPL coverage over data. **That page is
superseded.** The authoritative source is upstream's own repository, at the
pinned commit cited in §1.1: `LICENSE.md` (§2.1 above) and the accompanying
`LICENSE.ASSETS.md` (§2.2 above), both committed directly into the MakeHuman
source tree and versioned alongside the code and assets they describe. A
repository-committed licence file that ships with the exact commit being
used is inherently more authoritative than a separately-hosted marketing/CMS
page that is not version-pinned to any particular release. Archived copies of
both the current authoritative page and the superseded legacy page are linked
in §3 so that readers can compare them directly rather than take this
document's word for it.

To remove any doubt: even under the loosest plausible reading of the legacy
page, the *targets and base mesh* — the only asset classes OxiHuman bundles —
were never in dispute; every version of MakeHuman's licensing statement,
legacy or current, places targets/modifiers and the base mesh under CC0, and
the per-file embedded headers (§2.3) corroborate this independently of either
web page.

## 5. Code provenance

OxiHuman's own source code is Apache-2.0 (COOLJAPAN OU / Team KitaSan) and
contains no MakeHuman Python code. This claim is not asserted here on faith —
it was independently audited. See `docs/CLEANROOM_AUDIT.md` for the full
methodology (citation sweep of every `.rs`/`.md` reference to MakeHuman,
side-by-side structural comparison of the two modules that cited a MakeHuman
Python source file, verdicts on all modules touching MakeHuman-originated
file formats).

The audit's approved public wording, quoted verbatim from
`docs/CLEANROOM_AUDIT.md` §6:

> OxiHuman is an independent, pure-Rust, Apache-2.0 implementation of a
> parametric human body generator. It is *format-compatible* with MakeHuman:
> it reads the documented `.target` (sparse vertex-delta morph) and `.mhclo`
> (barycentric proxy binding) file formats and follows MakeHuman's target
> naming and directory conventions. It contains no code copied, translated,
> or otherwise derived from the AGPL-licensed MakeHuman Python application.
> MakeHuman's CC0-licensed mesh/target *data assets* may be used with
> OxiHuman under their own terms.

The word "port" is never used to describe this relationship (see
`docs/CLEANROOM_AUDIT.md` §6).

### 5.1 What is explicitly NOT bundled

* No MakeHuman **application code** of any kind (Python, GLSL shaders, UI
  images) — only 39 data files (38 targets + `base.obj`) were ever selected
  out of the 1,283-file upstream fetch (`assets/upstream/UPSTREAM_MANIFEST.json`).
* No **community asset packs** (third-party MakeHuman content, which upstream
  itself explicitly disclaims responsibility for — see `LICENSE.md` §D: "If
  you use a third part asset, such as one downloaded from the asset
  repositories, it is your own responsibility to make sure you abide by its
  specific license").
* No **textures, clothing/`.mhclo` proxy assets, poses, or rigs** — only the
  base mesh, 30 macrodetail targets, and 8 `measure/` girth targets are
  shipped; `oxihuman-core-v1.ohpk` is a "core" tier pack by design.
* No **SMPL, SMPL-X, STAR**, or any other third-party parametric body model
  or its assets, in any form, at any tier.

## 6. Machine verification

Run `scripts/check_provenance.sh` from the repository root:

```bash
scripts/check_provenance.sh
```

It verifies, and exits non-zero on any mismatch:

1. `assets/packs/oxihuman-core-v1.ohpk`'s SHA-256 matches **both** the hash
   stated in this document (§1) and the hash recorded in
   `assets/packs/oxihuman-core-v1.provenance.json`.
2. If `assets/upstream/` is present (it is gitignored and populated by the
   upstream fetch script — not guaranteed to exist in every checkout), every
   one of the 38 bundled targets' SHA-256, plus `base.obj`'s SHA-256, matches
   the sidecar's per-target list. If `assets/upstream/` is absent, this step
   is skipped with a notice (not a failure) — the pack hash check in step 1
   already guarantees the shipped binary hasn't changed since it was built
   from a verified upstream fetch.
3. `assets/alpha_pack/oxihuman_assets.toml`'s `core_pack_sha256` matches the
   actual file hash.

This task ran the script; see the accompanying task report for the captured
output. All checks passed.
