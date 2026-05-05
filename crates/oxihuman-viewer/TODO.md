# oxihuman-viewer -- TODO

> Version: 0.1.2 | Updated: 2026-05-05

## Status: Stable

All core features implemented. 0 stubs. 4,974 passing tests. ~880 modules across 161k SLoC.

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
- [x] Post-processing pipeline (bloom, depth of field, chromatic aberration, motion blur)
- [x] Shadow mapping (cascade shadow, shadow atlas, shadow bias)
- [x] Debug views (wireframe, normals, UVs, depth, AO, barycentric, bone influence)
- [x] Alpha blending modes (blend, coverage, discard, premult, sort, threshold)
- [x] Ambient occlusion (AO baker, AO renderer, SSAO, HBAO)
- [x] Texture management (cubemap, environment map, texture atlas, texture streaming)
- [x] Edge detection and outline rendering
- [x] Particle system rendering
- [x] Film/cinematic effects (grain, vignette, color grading, tone mapping)
- [x] Grid and gizmo overlays (axis, transform, bone visualizer)
- [x] Viewport management (split, picture-in-picture, stereo)
- [x] Decal rendering system
- [x] Cluster-based rendering
- [x] Billboard and annotation rendering
- [x] Atmosphere and sky rendering
- [x] GPU-based skinning and vertex paint

## Future Work

(No TODO/FIXME markers found in source)
