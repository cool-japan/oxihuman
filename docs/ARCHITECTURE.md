# OxiHuman Architecture

Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)

## Crate Graph

```
oxihuman-core        ← parsers (.target/.obj/.mhclo), policy, integrity
     ↓
oxihuman-morph       ← HumanEngine, scatter-add morph apply, ParamState
     ↓
oxihuman-mesh        ← MeshBuffers, normal/tangent recompute, LOD, suit check
     ↓
oxihuman-export      ← GLB v2 binary writer, params JSON
     ↓
oxihuman-wasm        ← WASM bindings (Phase 2)
oxihuman-viewer      ← WebGPU renderer (Phase 2)
oxihuman-physics     ← collision proxies (Phase 3)
```

## Data Flow

1. **Load**: `parse_obj(base.obj)` → `ObjMesh` (19,158 vertices)
2. **Engine**: `HumanEngine::new(base, policy)` → stores SoA positions
3. **Targets**: `engine.load_target(parse_target(src), weight_fn)`
4. **Params**: `engine.set_params(ParamState { height, weight, ... })`
5. **Build**: `engine.build_mesh()` → scatter-add all targets → `MeshBuffers`
6. **Normals**: `compute_normals(&mut buf)` → face-averaged normals
7. **Suit**: `apply_suit_flag(&mut buf)` → sets `has_suit = true`
8. **Export**: `export_glb(&buf, path)` → GLB 2.0 binary file

## Morph Algorithm

Based on MakeHuman's `algos3d.py`:
```
for each active target t with weight w:
    for each delta in t.deltas:
        x[delta.vid] += delta.dx * w
        y[delta.vid] += delta.dy * w
        z[delta.vid] += delta.dz * w
```

Positions stored as SoA (x[], y[], z[]) for cache-friendly iteration.

## File Formats

- `.target` — sparse vertex deltas: `vid dx dy dz` per line
- `.obj` — base mesh: standard Wavefront OBJ
- `.mhclo` — clothing binding: barycentric weights on base mesh
- `.glb` — output: GLB 2.0 binary (GLTF + embedded BIN)
