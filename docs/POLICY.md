# OxiHuman Safety Policy

Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)

> For the full safety model — the bodysuit invariant, the adult age floor, the
> reporting channel, and the privacy rationale — see [`SAFETY.md`](../SAFETY.md)
> at the repository root. This page is the policy-profile reference.

## Overview

OxiHuman enforces a layered safety policy to ensure the generated human meshes
are appropriate for all audiences by default.

## Policy Profiles

### Standard (default)
- Blocks any morph target whose name or tags contain: `explicit`, `sexual`, `nudity`, `adult`
- Allows all other targets
- Enforces the adult age floor on age parameters (see below)

### Strict
- Only allows targets explicitly listed in the manifest allowlist
- Suitable for children's apps and kiosk deployments
- Enforces the adult age floor on age parameters (see below)

## Age Floor

OxiHuman models adults only. `oxihuman_core::units::AGE_ADULT_FLOOR_YR` (`18.0`)
is the authoritative constant. Both policy profiles enforce it through the
`Policy` API:

- `Policy::param_floor(name)` returns `Some(18.0)` for age parameters
  (`"age"`, `"age_years"`; matched case-insensitively) and `None` otherwise.
- `Policy::is_param_value_allowed(name, value)` rejects age values below the
  floor under every profile.

The shipped core pack additionally contains no child/adolescent morph data at
the data level (`age_floor_years = 18.0` in its manifest), and the engine
clamps age inputs at load time.

## Export Safety

The exporter (`oxihuman-export`) will **refuse** to write any human mesh unless
its `has_suit` flag is set to `true`. Every gated exporter entry point routes
through `oxihuman_export::export_gate::ensure_export_allowed`; the invariant is
machine-checked by the `invariant_no_nude_mesh_stage` test in
`crates/oxihuman-tests/tests/`. This ensures the body is always covered before
any data leaves the system.

## Asset Integrity

All asset bundles may include SHA-256 hashes. The `integrity` module verifies
these hashes at load time.
