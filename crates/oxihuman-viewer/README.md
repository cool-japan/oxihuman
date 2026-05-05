# oxihuman-viewer

Part of the [OxiHuman](../../README.md) workspace — privacy-first, client-side human body generator in pure Rust.

**Version:** 0.1.2 | **Status:** Stable | **Updated:** 2026-05-05

| Metric | Value |
|--------|-------|
| Passing tests | 4,974 |
| Public API items | 9,528 |
| Source files | ~880 `.rs` files |
| Stub modules | 0 |

---

## Overview

`oxihuman-viewer` provides the complete rendering, scene management, camera, lighting, post-processing, debug visualization, and UI composition stack for OxiHuman. Core camera, scene, and material systems are stable. Post-processing effects, path tracing, and advanced rendering pipelines are Alpha or partial stubs.

Hardware-accelerated GPU rendering via wgpu 22 is available under the optional `webgpu` feature flag. Without that feature the crate still compiles and provides the full CPU-side scene graph, material, and pipeline abstractions.

---

## Dependency

### Without GPU rendering (CPU-side only)

```toml
[dependencies]
oxihuman-viewer = "0.1"
```

### With WebGPU / wgpu hardware rendering

```toml
[dependencies]
oxihuman-viewer = { version = "0.1", features = ["webgpu"] }
```

### Workspace dependencies

```toml
[dependencies]
anyhow.workspace = true
oxihuman-core.workspace = true
oxihuman-mesh.workspace = true
oxihuman-physics.workspace = true

# only pulled in with the webgpu feature:
# wgpu = "22"
```

---

## Feature Flags

| Flag | Default | Description |
|------|---------|-------------|
| `webgpu` | off | Enables wgpu 22 integration for hardware-accelerated rendering on Vulkan, Metal, DX12, and WebGPU backends |

> The `webgpu` feature is **required** for any actual GPU rendering. Without it, pipeline and pass types are present but no GPU commands are issued.

---

## Module Reference

### Core Rendering — Stable

| Module | Description |
|--------|-------------|
| `material` | Material properties, PBR parameters, and material slots |
| `pipeline` | Render pipeline configuration and state management |
| `render_pass` | Render pass recording and attachment setup |
| `render_graph` | Declarative render graph with automatic dependency ordering |
| `scene` | Scene graph — nodes, transforms, visibility |
| `scene_compositor` | Multi-layer scene compositing |
| `scene_visibility` | Frustum and occlusion visibility queries |
| `shader_library` | Compiled shader cache and hot-reload |
| `gpu_buffer` | Typed GPU buffer management (vertex, index, uniform, storage) |
| `texture_cache` | Texture asset cache with mip generation |
| `environment` | Environment map / Image-Based Lighting (IBL) |

---

### Camera Systems — Stable

| Module | Description |
|--------|-------------|
| `camera_rig` | Hierarchical camera rig (parent–child constraint chain) |
| `camera_presets` | Named preset positions (orbit, fly, cinematic, etc.) |
| `camera_animation` | Keyframed and spline-driven camera animation |
| `camera_dolly` | Physical dolly track movement |
| `camera_smooth` | Exponential / critically-damped smooth follow |
| `camera_track` | Rail-based camera track spline |
| `camera_shake_v2` | Procedural camera shake (perlin + trauma model) |

---

### Lighting — Stable / Beta

| Module | Description |
|--------|-------------|
| `lighting` | Directional, point, spot, and area light types |
| `light_map` | Pre-baked lightmap UVs and texture upload |
| `light_probe` | Reflection capture and spherical harmonics probes |
| `light_volume_v2` | Volumetric light scattering volumes (v2) |
| `light_bake` | CPU-side lightmap baking pipeline |
| `light_scatter` | Single-scattering atmosphere integration |
| `shadow_cascade_view` | Cascaded Shadow Maps (CSM) view management |
| `shadow_caster` | Per-light shadow map rendering |
| `contact_shadow` | Screen-space contact shadow |

---

### Post-Processing Effects — 60+ modules (Alpha)

#### Depth of Field

| Module | Description |
|--------|-------------|
| `depth_of_field` | Bokeh depth-of-field (hex / circular aperture) |
| `focus_peaking_view` | Focus peaking highlight overlay |
| `rack_focus_view` | Animated rack-focus transition |

#### Color Grading

| Module | Description |
|--------|-------------|
| `color_correction` | Lift / gamma / gain color correction |
| `color_grade` | Full color grading pipeline |
| `color_temperature` | Kelvin white-balance adjustment |
| `exposure_control` | Auto and manual exposure |
| `lut_preview_view` | 3D LUT preview and application |

#### Artistic Effects

| Module | Description |
|--------|-------------|
| `cel_shade_view` | Toon / cel shading with outline |
| `oil_paint_view` | Kuwahara oil-paint stylization |
| `pencil_sketch_view` | Pencil sketch edge rendering |
| `watercolor_view` | Watercolor paper texture and bleed |
| `pixel_art_view` | Pixel-art downscale and palette quantization |

#### Visual Effects

| Module | Description |
|--------|-------------|
| `bloom` | Physically-based bloom (dual Kawase) |
| `lens_distortion` | Barrel / pincushion lens distortion |
| `chromatic_shift_view` | Chromatic aberration |
| `film_grain` | Film grain noise overlay |
| `motion_blur` | Per-object and camera motion blur |

#### Advanced Rendering — Partial Stubs

| Module | Description |
|--------|-------------|
| `path_tracer_view` | Wavefront path tracer (stub — GPU kernel integration pending) |
| `temporal_aa_view` | Temporal Anti-Aliasing (TAA) accumulation |
| `denoiser_view` | Post-process denoiser (stub — spatial filter fallback) |
| `neural_render` | Neural radiance / super-resolution placeholder |

---

### Debug Visualization — 100+ views (Beta / Alpha)

#### Physics Debug

| Module | Description |
|--------|-------------|
| `physics_debug_view` | Physics world overlay (broad-phase cells, AABBs) |
| `collision_shape_view` | Collision shape wireframe overlay |
| `joint_pivot_view` | Joint axes and pivot point gizmos |
| `force_vector_view` | Force and impulse vector arrows |
| `ragdoll_debug_view` | Ragdoll constraint chain visualization |

#### Geometry Analysis

| Module | Description |
|--------|-------------|
| `curvature_view` | Mean and Gaussian curvature heat-map |
| `normal_channel_view` | Object-space and tangent-space normal visualization |
| `tangent_space_view` | Tangent frame axes visualization |
| `seam_view` | UV seam edge highlight |
| `topology_view` | Pole, n-gon, and non-manifold edge highlight |

#### Specialized Scientific Views

| Module | Description |
|--------|-------------|
| `microscopy_view` | Simulated optical microscopy rendering |
| `thermal_view` | False-color thermal / infrared heat-map |
| `xray_view` | X-ray density projection |
| `infrared_view` | Near-infrared band visualization |
| `sonar_view` | Sonar ping sweep visualization |
| `lidar_point_view` | LiDAR point cloud colorized by distance |

#### Buffer Inspection

| Module | Description |
|--------|-------------|
| `gbuffer_view` | G-buffer channel inspection (albedo, normals, roughness, metallic) |
| `depth_buffer_view` | Linearized depth buffer visualization |
| `vertex_buffer_view` | Per-vertex attribute heatmap |

---

### UI & Composition — Stable / Beta

| Module | Description |
|--------|-------------|
| `selection` | Object selection with highlight pass |
| `gizmo` | Transform gizmo (translate, rotate, scale) |
| `ruler_tool` | 3D ruler / distance measurement tool |
| `measurement_display` | On-screen measurement label overlay |
| `annotation` | World-space text and arrow annotations |
| `overlay_renderer` | General 2D overlay compositing pass |
| `stats_overlay` | Real-time performance stats HUD |
| `world_axes` | World-space orientation axes widget |
| `viewport_grid` | Infinite grid plane with fading |
| `split_view` | Multi-pane viewport split layout |
| `xr_viewport` | XR / stereoscopic viewport adapter |

---

## Stability Breakdown

| Category | Stability | Notes |
|----------|-----------|-------|
| Material / PBR | Stable | Production-ready |
| Render pipeline / graph | Stable | Production-ready |
| Scene graph | Stable | Production-ready |
| GPU buffer / texture cache | Stable | Production-ready |
| Camera systems | Stable | Production-ready |
| Lighting (all types) | Stable / Beta | Shadow cascade is Beta |
| IBL / environment | Stable | |
| Post-processing — color | Alpha | API may change |
| Post-processing — artistic | Alpha | Stub implementations |
| Post-processing — VFX | Alpha | |
| Path tracer | Partial stub | GPU kernel not yet integrated |
| TAA / denoiser | Alpha / Partial | Spatial fallback active |
| Debug views (100+) | Beta / Alpha | Many are display-only stubs |
| UI gizmos / overlay | Beta | |
| XR viewport | Alpha | |

> **Note:** The `webgpu` feature is required for actual GPU rendering. Without it, all pipeline types compile but no GPU commands are dispatched.

---

## License

Apache-2.0 — Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
