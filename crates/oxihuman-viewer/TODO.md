# oxihuman-viewer -- TODO

> Version: 0.2.2 | Updated: 2026-07-13

## Status: Stable

All core features implemented. 0 stubs. 4,988 passing tests. 879 `.rs` files across 161k SLoC
(359 files not wired into the module tree — see Future Work).

## Completed

- [x] wgpu/WebGPU rendering pipeline (RenderPipelineDescriptor, vertex layouts, blend states)
- [x] PBR material system (PbrMaterial, MaterialLibrary, color utilities)
- [x] Scene graph (Scene, SceneNode, Transform, Light, LightKind)
- [x] Camera system (CameraState, orbit/fly/dolly/path/smooth/shake controllers)
- [x] Camera animation (CameraAnimationPlayer, bookmarks, presets, frustum, jitter)
- [x] LOD manager v2 (LodManagerV2, LodConfig, LodTransition, build_lod_chain)
- [x] Morph updater (MorphUpdater, MorphSlider, MorphTargetDeltas)
- [x] Event loop and input handling (WindowState, InputState, OrbitCameraController)
- [x] Render stats v3 (FrameTimer, RenderStatsV3, RenderStatsSnapshot)
- [x] Screenshot capture (ScreenshotCapture, ImageBuffer)
- [x] GPU mesh upload (MeshUploadBuffer, buffer management)
- [x] Lighting presets system
- [x] Post-processing pipeline (bloom via `post_process`, depth of field, chromatic aberration, motion blur)
- [ ] Shadow mapping (cascade shadow, shadow atlas, shadow bias) — `shadow_map.rs`, `shadow_atlas.rs`,
      `cascade_shadow.rs`, `pcf_shadow.rs` exist but are not wired into the module tree; only shadow
      debug/catcher/volumetric/ray-traced views (`shadow_debug`, `shadow_catcher_view`,
      `volumetric_shadow_view`, `shadow_ray_view`) are actually compiled
- [x] Debug views (wireframe, normals, UVs, depth, AO, barycentric, bone influence)
- [x] Alpha blending modes (blend, coverage, discard, premult, sort, threshold)
- [x] Ambient occlusion (`ambient_occlusion_v2`, `ambient_occlusion_preview`, occlusion probes/volumes)
- [x] Texture management (cubemap, environment map, texture atlas)
- [x] Edge detection and outline rendering
- [x] Particle system rendering
- [x] Film/cinematic effects (grain, vignette, color grading, tone mapping)
- [x] Grid and gizmo overlays (axis, transform, bone visualizer)
- [x] Viewport management (split, stereo/XR)
- [x] Decal rendering system
- [x] Cluster-based rendering
- [x] Annotation rendering (world-space text/arrow annotations)
- [x] Atmosphere and sky rendering
- [x] GPU-based skinning and vertex paint

## Future Work

(No TODO/FIXME markers found in source.)

- [ ] Audit the 359 top-level `.rs` files with no `mod` declaration anywhere in the crate (verified via
      `grep` this session — files like `shadow_map.rs`, `shadow_atlas.rs`, `texture_streaming.rs`,
      `particle_system_renderer.rs`, `decal_renderer.rs` sit on disk but are not part of the compiled
      crate). Either wire them in or remove them; several categories (shadow mapping, texture streaming)
      have no compiled equivalent under any name, while others (particles, decals, edge/outline) are
      already covered by differently-named sibling modules that *are* wired.
