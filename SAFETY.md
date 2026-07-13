# OxiHuman Safety Model

Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)

OxiHuman is an adult-body parametric human generator. This page states the
safety guarantees the codebase enforces and where each one is checked, so that
users, integrators, and auditors can verify them directly.

## 1. Bodysuit invariant

No unclothed human mesh ever leaves the exporter. Every human-facing exporter
entry point in `oxihuman-export` calls
`oxihuman_export::export_gate::ensure_export_allowed`, which refuses any
`oxihuman_mesh::MeshBuffers` whose `has_suit` flag is `false`.

- **Enforced by test:** `invariant_no_nude_mesh_stage`
- **Location:** `crates/oxihuman-tests/tests/invariant_no_nude_mesh_stage.rs`
- The test drives one unsuited mesh through every gated entry point and
  asserts each returns an error, then confirms a suited mesh is accepted by
  GLB / OBJ / STL / VRM.
- The authoritative list of gated entry points is the doc table at the top of
  `crates/oxihuman-export/src/export_gate.rs`. That file also documents the
  currently-tracked gaps (some geometry writers, e.g. PLY / FBX / X3D / 3MF,
  are pending gating in a follow-up wave).

## 2. Adult age floor

OxiHuman models adults only. The floor is enforced at three layers:

- **Data level:** the shipped core asset pack
  (`assets/packs/oxihuman-core-v1.ohpk`) contains no child or adolescent morph
  data. Its manifest declares `age_floor_years = 18.0`.
- **Load level:** the engine clamps any age parameter up to the pack's
  declared floor at load time (`commit_params` in the WASM engine; floor
  ~= age param 0.3542).
- **Core level:** `oxihuman_core::units::AGE_ADULT_FLOOR_YR` (`= 18.0`) is the
  authoritative constant, mirrored by the policy API
  (`Policy::param_floor` / `Policy::is_param_value_allowed`) so that native and
  third-party embedders can query and enforce the floor before building a mesh.

## 3. Content policy profiles

The `oxihuman_core::policy` module filters morph targets:

- **Standard (default):** blocks any target whose name or tags contain a
  blocklisted substring — `explicit`, `sexual`, `nudity`, `adult` — and allows
  everything else.
- **Strict:** additionally requires targets to appear in an explicit allowlist;
  suitable for children's apps and kiosk deployments.

Both profiles enforce the age floor of section 2 when asked via
`Policy::param_floor` / `Policy::is_param_value_allowed`.

## 4. Reporting a vulnerability

Report suspected safety or security issues through the repository's GitHub
**Security Advisories** ("Report a vulnerability") for the
`cool-japan/oxihuman` repository. If advisories are unavailable, open a
repository issue and mark it as a security concern. Please do not disclose
details publicly until a fix is available.

## 5. Why this design

OxiHuman is privacy-first: body generation, morphing, and measurement run
entirely on the user's own device, and measurements never leave it. The
safety guarantees above — the bodysuit invariant, the adult age floor, and the
content policy — are enforced locally in the same process that produces the
mesh, so a correct build cannot emit unclothed or under-age human geometry
regardless of how the library is embedded.

---

See also: [`docs/POLICY.md`](docs/POLICY.md) for the policy profile reference.
