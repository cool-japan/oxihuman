// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! WGSL shader source strings for the OxiHuman wgpu render pipeline.
//!
//! All shaders are embedded as `pub const &str` and compiled at pipeline creation
//! time via `wgpu::Device::create_shader_module`.

// ── PBR Vertex Shader ─────────────────────────────────────────────────────────

/// PBR vertex shader.
///
/// Inputs per vertex: position (f32x3), normal (f32x3), uv (f32x2), tangent (f32x4).
/// Uniforms: camera bind group (group 0), model transform (group 1).
pub const VERTEX_SHADER_PBR: &str = r#"
// ── Camera uniform (group 0, binding 0) ──────────────────────────────────────
struct CameraUniform {
    view:       mat4x4<f32>,
    proj:       mat4x4<f32>,
    view_proj:  mat4x4<f32>,
    eye_pos:    vec4<f32>,  // w unused
    near_far:   vec2<f32>,  // near, far
    _pad:       vec2<f32>,
}
@group(0) @binding(0) var<uniform> camera: CameraUniform;

// ── Model transform (group 1, binding 0) ─────────────────────────────────────
struct ModelUniform {
    model:        mat4x4<f32>,
    model_inv_t:  mat4x4<f32>,  // inverse-transpose for normals
}
@group(1) @binding(0) var<uniform> model: ModelUniform;

// ── Vertex input ──────────────────────────────────────────────────────────────
struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) normal:   vec3<f32>,
    @location(2) uv:       vec2<f32>,
    @location(3) tangent:  vec4<f32>,  // xyz = tangent, w = bitangent sign
}

// ── Vertex output / fragment input ────────────────────────────────────────────
struct VertexOut {
    @builtin(position) clip_pos:    vec4<f32>,
    @location(0)       world_pos:   vec3<f32>,
    @location(1)       world_norm:  vec3<f32>,
    @location(2)       uv:          vec2<f32>,
    @location(3)       world_tan:   vec3<f32>,
    @location(4)       world_bitan: vec3<f32>,
}

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;

    let world_pos4  = model.model * vec4<f32>(in.position, 1.0);
    out.world_pos   = world_pos4.xyz;
    out.clip_pos    = camera.view_proj * world_pos4;

    // Normal, tangent, bitangent in world space
    out.world_norm  = normalize((model.model_inv_t * vec4<f32>(in.normal, 0.0)).xyz);
    let wtan        = normalize((model.model       * vec4<f32>(in.tangent.xyz, 0.0)).xyz);
    out.world_tan   = wtan;
    out.world_bitan = cross(out.world_norm, wtan) * in.tangent.w;

    out.uv = in.uv;
    return out;
}
"#;

// ── PBR Fragment Shader ───────────────────────────────────────────────────────

/// PBR fragment shader.
///
/// Features: Cook-Torrance BRDF, normal mapping, up to 8 point lights +
/// 1 directional light, albedo/metallic/roughness/normal/emissive textures.
pub const FRAGMENT_SHADER_PBR: &str = r#"
const PI: f32 = 3.14159265358979;
const MAX_POINT_LIGHTS: u32 = 8u;

// ── Camera uniform (group 0, binding 0) ──────────────────────────────────────
struct CameraUniform {
    view:      mat4x4<f32>,
    proj:      mat4x4<f32>,
    view_proj: mat4x4<f32>,
    eye_pos:   vec4<f32>,
    near_far:  vec2<f32>,
    _pad:      vec2<f32>,
}
@group(0) @binding(0) var<uniform> camera: CameraUniform;

// ── Material uniform (group 2, binding 0) ─────────────────────────────────────
struct MaterialUniform {
    albedo:    vec4<f32>,   // base color (linear), w = alpha
    metallic:  f32,
    roughness: f32,
    _pad:      vec2<f32>,
    emissive:  vec4<f32>,   // emissive color + intensity in w
}
@group(2) @binding(0) var<uniform> material: MaterialUniform;

// ── Textures (group 2, bindings 1-5) ─────────────────────────────────────────
@group(2) @binding(1) var t_albedo:            texture_2d<f32>;
@group(2) @binding(2) var s_albedo:            sampler;
@group(2) @binding(3) var t_normal:            texture_2d<f32>;
@group(2) @binding(4) var s_normal:            sampler;
@group(2) @binding(5) var t_metallic_roughness: texture_2d<f32>;
@group(2) @binding(6) var s_metallic_roughness: sampler;

// ── Light array (group 3, binding 0) ─────────────────────────────────────────
struct PointLight {
    position:  vec4<f32>,   // xyz position, w = range
    color:     vec4<f32>,   // rgb color, w = intensity
}
struct DirectionalLight {
    direction: vec4<f32>,   // xyz normalized direction, w unused
    color:     vec4<f32>,   // rgb, w = intensity
}
struct LightArray {
    point_lights: array<PointLight, 8>,
    dir_light:    DirectionalLight,
    num_points:   u32,
    _pad:         vec3<u32>,
}
@group(3) @binding(0) var<uniform> lights: LightArray;

// ── Fragment input ────────────────────────────────────────────────────────────
struct FragIn {
    @location(0) world_pos:   vec3<f32>,
    @location(1) world_norm:  vec3<f32>,
    @location(2) uv:          vec2<f32>,
    @location(3) world_tan:   vec3<f32>,
    @location(4) world_bitan: vec3<f32>,
}

// ── BRDF helpers ──────────────────────────────────────────────────────────────

fn distribution_ggx(n_dot_h: f32, roughness: f32) -> f32 {
    let a  = roughness * roughness;
    let a2 = a * a;
    let d  = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
    return a2 / (PI * d * d);
}

fn geometry_schlick_ggx(n_dot_v: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    return n_dot_v / (n_dot_v * (1.0 - k) + k);
}

fn geometry_smith(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
    return geometry_schlick_ggx(n_dot_v, roughness)
         * geometry_schlick_ggx(n_dot_l, roughness);
}

fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    let c = clamp(1.0 - cos_theta, 0.0, 1.0);
    let c5 = c * c * c * c * c;
    return f0 + (1.0 - f0) * c5;
}

fn pbr_direct(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>,
              albedo: vec3<f32>, metallic: f32, roughness: f32,
              radiance: vec3<f32>) -> vec3<f32> {
    let h = normalize(v + l);
    let n_dot_v = max(dot(n, v), 0.0001);
    let n_dot_l = max(dot(n, l), 0.0);
    let n_dot_h = max(dot(n, h), 0.0);
    let h_dot_v = max(dot(h, v), 0.0);

    let f0 = mix(vec3<f32>(0.04), albedo, metallic);

    let ndf = distribution_ggx(n_dot_h, roughness);
    let g   = geometry_smith(n_dot_v, n_dot_l, roughness);
    let f   = fresnel_schlick(h_dot_v, f0);

    let num   = ndf * g * f;
    let denom = 4.0 * n_dot_v * n_dot_l + 0.0001;
    let spec  = num / denom;

    let k_s = f;
    let k_d = (1.0 - k_s) * (1.0 - metallic);
    let diff = k_d * albedo / PI;

    return (diff + spec) * radiance * n_dot_l;
}

@fragment
fn fs_main(in: FragIn) -> @location(0) vec4<f32> {
    // Sample textures
    let albedo_samp = textureSample(t_albedo, s_albedo, in.uv);
    let mr_samp     = textureSample(t_metallic_roughness, s_metallic_roughness, in.uv);
    let norm_samp   = textureSample(t_normal, s_normal, in.uv).xyz * 2.0 - 1.0;

    let albedo    = albedo_samp.rgb * material.albedo.rgb;
    let alpha     = albedo_samp.a  * material.albedo.a;
    let metallic  = mr_samp.b * material.metallic;
    let roughness = clamp(mr_samp.g * material.roughness, 0.04, 1.0);

    // TBN normal mapping
    let tbn = mat3x3<f32>(
        normalize(in.world_tan),
        normalize(in.world_bitan),
        normalize(in.world_norm),
    );
    let n = normalize(tbn * norm_samp);
    let v = normalize(camera.eye_pos.xyz - in.world_pos);

    var lo = vec3<f32>(0.0);

    // Directional light
    let dl   = normalize(-lights.dir_light.direction.xyz);
    let drad = lights.dir_light.color.rgb * lights.dir_light.color.a;
    lo += pbr_direct(n, v, dl, albedo, metallic, roughness, drad);

    // Point lights
    for (var i = 0u; i < min(lights.num_points, MAX_POINT_LIGHTS); i++) {
        let pl       = lights.point_lights[i];
        let l_vec    = pl.position.xyz - in.world_pos;
        let dist     = length(l_vec);
        let range    = max(pl.position.w, 0.0001);
        let atten    = clamp(1.0 - (dist / range) * (dist / range), 0.0, 1.0);
        let l        = l_vec / dist;
        let radiance = pl.color.rgb * pl.color.a * atten;
        lo += pbr_direct(n, v, l, albedo, metallic, roughness, radiance);
    }

    // Ambient (simple IBL placeholder)
    let ambient = vec3<f32>(0.03) * albedo;
    let emissive = material.emissive.rgb * material.emissive.a;

    let color = ambient + lo + emissive;
    return vec4<f32>(color, alpha);
}
"#;

// ── Wireframe Vertex Shader ───────────────────────────────────────────────────

/// Wireframe vertex shader.
///
/// Minimal: only reads position; transforms via camera + model uniforms.
pub const VERTEX_SHADER_WIREFRAME: &str = r#"
struct CameraUniform {
    view:      mat4x4<f32>,
    proj:      mat4x4<f32>,
    view_proj: mat4x4<f32>,
    eye_pos:   vec4<f32>,
    near_far:  vec2<f32>,
    _pad:      vec2<f32>,
}
@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct ModelUniform {
    model:       mat4x4<f32>,
    model_inv_t: mat4x4<f32>,
}
@group(1) @binding(0) var<uniform> model: ModelUniform;

struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) normal:   vec3<f32>,
    @location(2) uv:       vec2<f32>,
    @location(3) tangent:  vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_pos: vec4<f32>,
}

@vertex
fn vs_wireframe(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    let world_pos = model.model * vec4<f32>(in.position, 1.0);
    out.clip_pos  = camera.view_proj * world_pos;
    return out;
}
"#;

// ── Wireframe Fragment Shader ─────────────────────────────────────────────────

/// Wireframe fragment shader.
///
/// Outputs a configurable solid color (default: lime green).
pub const FRAGMENT_SHADER_WIREFRAME: &str = r#"
struct WireframeParams {
    color: vec4<f32>,
}
@group(2) @binding(0) var<uniform> wf_params: WireframeParams;

@fragment
fn fs_wireframe() -> @location(0) vec4<f32> {
    return wf_params.color;
}
"#;

// ── Morph Compute Shader ─────────────────────────────────────────────────────

/// GPU morph target application compute shader.
///
/// Applies `num_targets` morph deltas weighted by `weights[i]` to produce
/// the final displaced position buffer.
///
/// Workgroup size: 64 threads.  Dispatch with ceil(num_vertices / 64) groups.
pub const COMPUTE_SHADER_MORPH: &str = r#"
struct MorphParams {
    num_vertices: u32,
    num_targets:  u32,
    _pad:         vec2<u32>,
    weights:      array<f32, 64>,   // up to 64 morph targets
}

@group(0) @binding(0) var<storage, read>       in_positions:  array<vec4<f32>>;
@group(0) @binding(1) var<storage, read>       morph_deltas:  array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> out_positions: array<vec4<f32>>;
@group(0) @binding(3) var<uniform>             params:        MorphParams;

@compute @workgroup_size(64)
fn cs_morph(@builtin(global_invocation_id) gid: vec3<u32>) {
    let vid = gid.x;
    if (vid >= params.num_vertices) {
        return;
    }
    var pos = in_positions[vid].xyz;
    for (var t = 0u; t < params.num_targets; t++) {
        let delta_idx = t * params.num_vertices + vid;
        let delta     = morph_deltas[delta_idx].xyz;
        pos += delta * params.weights[t];
    }
    out_positions[vid] = vec4<f32>(pos, 1.0);
}
"#;

// ── Shadow Map Vertex Shader ──────────────────────────────────────────────────

/// Shadow map vertex shader.
///
/// Transforms geometry into the light's clip space for depth rendering.
/// Uses a single `light_view_proj` uniform (group 0, binding 0).
pub const VERTEX_SHADER_SHADOW: &str = r#"
struct ShadowUniform {
    light_view_proj: mat4x4<f32>,
}
@group(0) @binding(0) var<uniform> shadow: ShadowUniform;

struct ModelUniform {
    model:       mat4x4<f32>,
    model_inv_t: mat4x4<f32>,
}
@group(1) @binding(0) var<uniform> model: ModelUniform;

struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) normal:   vec3<f32>,
    @location(2) uv:       vec2<f32>,
    @location(3) tangent:  vec4<f32>,
}

@vertex
fn vs_shadow(in: VertexIn) -> @builtin(position) vec4<f32> {
    let world_pos = model.model * vec4<f32>(in.position, 1.0);
    return shadow.light_view_proj * world_pos;
}
"#;

// ── Shadow Map Fragment Shader ────────────────────────────────────────────────

/// Shadow map fragment shader (depth-only pass).
///
/// This shader is intentionally empty — the GPU writes depth automatically.
/// It is kept as a distinct source string so that a `FragmentState` can be
/// provided when required by the backend.
pub const FRAGMENT_SHADER_SHADOW: &str = r#"
// Depth-only fragment shader — no colour outputs.
// The rasteriser writes gl_FragDepth automatically.
@fragment
fn fs_shadow() {}
"#;

// ── Fullscreen Quad Vertex Shader ─────────────────────────────────────────────

/// Fullscreen quad vertex shader for post-processing passes.
///
/// Generates a clip-space triangle that covers the entire screen without
/// a vertex buffer (vertex index → UV and position).
pub const VERTEX_SHADER_FULLSCREEN: &str = r#"
struct FullscreenOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0)       uv:       vec2<f32>,
}

@vertex
fn vs_fullscreen(@builtin(vertex_index) vi: u32) -> FullscreenOut {
    // Emit a giant triangle that covers the NDC square [-1,1]^2.
    let x = f32((vi & 1u) * 4u) - 1.0;
    let y = f32((vi & 2u) * 2u) - 1.0;
    var out: FullscreenOut;
    out.clip_pos = vec4<f32>(x, y, 0.0, 1.0);
    out.uv       = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}
"#;

// ── Tonemapping Fragment Shader ───────────────────────────────────────────────

/// ACES tonemapping + gamma correction fragment shader.
///
/// Reads an HDR colour from `t_hdr` and outputs a gamma-corrected sRGB value.
/// Tonemapping curve: ACES fitted approximation (Stephen Hill).
pub const FRAGMENT_SHADER_TONEMAP: &str = r#"
@group(0) @binding(0) var t_hdr: texture_2d<f32>;
@group(0) @binding(1) var s_hdr: sampler;

struct TonemapParams {
    exposure:       f32,
    gamma:          f32,
    _pad:           vec2<f32>,
}
@group(0) @binding(2) var<uniform> tonemap: TonemapParams;

// ACES fitted curve (Stephen Hill approximation)
fn aces_film(x: vec3<f32>) -> vec3<f32> {
    let a: f32 = 2.51;
    let b: f32 = 0.03;
    let c: f32 = 2.43;
    let d: f32 = 0.59;
    let e: f32 = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

@fragment
fn fs_tonemap(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let hdr    = textureSample(t_hdr, s_hdr, uv).rgb;
    let mapped = aces_film(hdr * tonemap.exposure);
    // Gamma correction (linear -> sRGB approximation)
    let gamma_inv = 1.0 / tonemap.gamma;
    let srgb = pow(mapped, vec3<f32>(gamma_inv));
    return vec4<f32>(srgb, 1.0);
}
"#;
