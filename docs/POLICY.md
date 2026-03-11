# OxiHuman Safety Policy

Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)

## Overview

OxiHuman enforces a layered safety policy to ensure the generated human meshes
are appropriate for all audiences by default.

## Policy Profiles

### Standard (default)
- Blocks any morph target whose name or tags contain: `explicit`, `sexual`, `nudity`, `adult`
- Allows all other targets

### Strict
- Only allows targets explicitly listed in the manifest allowlist
- Suitable for children's apps and kiosk deployments

## Export Safety

The exporter (`oxihuman-export`) will **refuse** to write a GLB file unless
the mesh has the `has_suit` flag set to `true`. This ensures the body is always
covered before any data leaves the system.

## Asset Integrity

All asset bundles may include SHA-256 hashes. The `integrity` module verifies
these hashes at load time.
