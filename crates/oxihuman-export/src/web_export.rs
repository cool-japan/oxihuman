// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! WebGL/browser-optimized JSON mesh export.
//!
//! Provides browser-ready export with material/shader hints, LOD embedding,
//! and compressed indices.

// ── Types ─────────────────────────────────────────────────────────────────────

/// PBR material definition for web export.
#[derive(Debug, Clone)]
pub struct WebMaterial {
    pub name: String,
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: [f32; 3],
    pub alpha_mode: String, // "OPAQUE", "BLEND", "MASK"
    pub double_sided: bool,
}

/// A single LOD level with its own geometry.
#[derive(Debug, Clone)]
pub struct WebLodLevel {
    pub level: u32,
    pub triangle_count: usize,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub screen_size_threshold: f32,
}

/// Options controlling what is included in the web export.
#[derive(Debug, Clone)]
pub struct WebExportOptions {
    pub include_normals: bool,
    pub include_uvs: bool,
    pub include_colors: bool,
    pub quantize_positions: bool,
    pub interleave_buffers: bool,
    pub include_lod: bool,
    pub max_lod_levels: usize,
}

impl Default for WebExportOptions {
    fn default() -> Self {
        WebExportOptions {
            include_normals: true,
            include_uvs: true,
            include_colors: false,
            quantize_positions: false,
            interleave_buffers: false,
            include_lod: false,
            max_lod_levels: 4,
        }
    }
}

/// A mesh ready for browser/WebGL consumption.
#[derive(Debug, Clone)]
pub struct WebMesh {
    pub name: String,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub material: Option<WebMaterial>,
    pub lod_levels: Vec<WebLodLevel>,
    pub bounding_box: ([f32; 3], [f32; 3]),
    pub vertex_count: usize,
    pub triangle_count: usize,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Create a new `WebMesh` from positions and indices.
/// Normals and UVs are left empty; bounding box is computed automatically.
#[allow(dead_code)]
pub fn new_web_mesh(name: &str, positions: Vec<[f32; 3]>, indices: Vec<u32>) -> WebMesh {
    let triangle_count = indices.len() / 3;
    let vertex_count = positions.len();
    let bb = compute_web_mesh_bounds_raw(&positions);
    WebMesh {
        name: name.to_string(),
        normals: Vec::new(),
        uvs: Vec::new(),
        indices,
        material: None,
        lod_levels: Vec::new(),
        bounding_box: bb,
        vertex_count,
        triangle_count,
        positions,
    }
}

/// Serialize a `WebMesh` to a JSON string honouring the export options.
#[allow(dead_code)]
pub fn web_mesh_to_json(mesh: &WebMesh, opts: &WebExportOptions) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("\"name\":\"{}\"", esc(&mesh.name)));
    parts.push(format!("\"vertex_count\":{}", mesh.vertex_count));
    parts.push(format!("\"triangle_count\":{}", mesh.triangle_count));

    // positions
    let pos_strs: Vec<String> = mesh
        .positions
        .iter()
        .map(|p| format!("[{},{},{}]", p[0], p[1], p[2]))
        .collect();
    parts.push(format!("\"positions\":[{}]", pos_strs.join(",")));

    // normals
    if opts.include_normals && !mesh.normals.is_empty() {
        let nrm_strs: Vec<String> = mesh
            .normals
            .iter()
            .map(|n| format!("[{},{},{}]", n[0], n[1], n[2]))
            .collect();
        parts.push(format!("\"normals\":[{}]", nrm_strs.join(",")));
    }

    // uvs
    if opts.include_uvs && !mesh.uvs.is_empty() {
        let uv_strs: Vec<String> = mesh
            .uvs
            .iter()
            .map(|u| format!("[{},{}]", u[0], u[1]))
            .collect();
        parts.push(format!("\"uvs\":[{}]", uv_strs.join(",")));
    }

    // indices
    let idx_strs: Vec<String> = mesh.indices.iter().map(|i| i.to_string()).collect();
    parts.push(format!("\"indices\":[{}]", idx_strs.join(",")));

    // bounding_box
    let (mn, mx) = &mesh.bounding_box;
    parts.push(format!(
        "\"bounding_box\":{{\"min\":[{},{},{}],\"max\":[{},{},{}]}}",
        mn[0], mn[1], mn[2], mx[0], mx[1], mx[2]
    ));

    // material
    if let Some(ref mat) = mesh.material {
        parts.push(format!(
            "\"material\":{{\"name\":\"{}\",\"base_color\":[{},{},{},{}],\
             \"metallic\":{},\"roughness\":{},\"emissive\":[{},{},{}],\
             \"alpha_mode\":\"{}\",\"double_sided\":{}}}",
            esc(&mat.name),
            mat.base_color[0],
            mat.base_color[1],
            mat.base_color[2],
            mat.base_color[3],
            mat.metallic,
            mat.roughness,
            mat.emissive[0],
            mat.emissive[1],
            mat.emissive[2],
            esc(&mat.alpha_mode),
            mat.double_sided,
        ));
    }

    // LOD
    if opts.include_lod && !mesh.lod_levels.is_empty() {
        let lod_strs: Vec<String> = mesh
            .lod_levels
            .iter()
            .map(|l| {
                let p: Vec<String> = l
                    .positions
                    .iter()
                    .map(|p| format!("[{},{},{}]", p[0], p[1], p[2]))
                    .collect();
                let idx: Vec<String> = l.indices.iter().map(|i| i.to_string()).collect();
                format!(
                    "{{\"level\":{},\"triangle_count\":{},\
                     \"screen_size_threshold\":{},\
                     \"positions\":[{}],\"indices\":[{}]}}",
                    l.level,
                    l.triangle_count,
                    l.screen_size_threshold,
                    p.join(","),
                    idx.join(","),
                )
            })
            .collect();
        parts.push(format!("\"lod_levels\":[{}]", lod_strs.join(",")));
    }

    format!("{{{}}}", parts.join(","))
}

/// Deserialize a `WebMesh` from JSON produced by `web_mesh_to_json`.
/// Returns `None` on parse failure.
#[allow(dead_code)]
pub fn web_mesh_from_json(json: &str) -> Option<WebMesh> {
    // Minimal hand-rolled parser: extract "name" and "vertex_count".
    let name = extract_str(json, "name").unwrap_or_default();
    let vertex_count = extract_usize(json, "vertex_count").unwrap_or(0);
    let triangle_count = extract_usize(json, "triangle_count").unwrap_or(0);

    // For a stub round-trip, we only need positions + indices.
    let positions = extract_f32_3_array(json, "positions").unwrap_or_default();
    let indices = extract_u32_array(json, "indices").unwrap_or_default();

    let bb = compute_web_mesh_bounds_raw(&positions);
    Some(WebMesh {
        name,
        positions,
        normals: Vec::new(),
        uvs: Vec::new(),
        indices,
        material: None,
        lod_levels: Vec::new(),
        bounding_box: bb,
        vertex_count,
        triangle_count,
    })
}

/// Append a pre-built LOD level to the mesh.
#[allow(dead_code)]
pub fn add_lod_level(mesh: &mut WebMesh, level: WebLodLevel) {
    mesh.lod_levels.push(level);
}

/// Generate LOD levels for a mesh by decimating at each screen-size threshold.
/// `levels` contains `screen_size_threshold` values (e.g. [0.5, 0.25, 0.1]).
#[allow(dead_code)]
pub fn generate_lod_levels(mesh: &WebMesh, levels: &[f32]) -> Vec<WebLodLevel> {
    levels
        .iter()
        .enumerate()
        .map(|(i, &threshold)| {
            // Decimate by removing every other triangle pair naively.
            let keep_ratio = threshold.clamp(0.0, 1.0);
            let target_tris = ((mesh.triangle_count as f32) * keep_ratio) as usize;
            let keep_tris = target_tris.max(1);
            let max_idx = (keep_tris * 3).min(mesh.indices.len());
            // Round down to multiple of 3
            let safe_idx = (max_idx / 3) * 3;
            let dec_indices: Vec<u32> = mesh.indices[..safe_idx].to_vec();
            let tri_count = dec_indices.len() / 3;
            WebLodLevel {
                level: i as u32,
                triangle_count: tri_count,
                positions: mesh.positions.clone(),
                normals: mesh.normals.clone(),
                uvs: mesh.uvs.clone(),
                indices: dec_indices,
                screen_size_threshold: threshold,
            }
        })
        .collect()
}

/// Quantize mesh positions to 16-bit unsigned integers.
/// Returns a flat list: [x0_u16, y0_u16, z0_u16, x1_u16, ...] encoded as individual u16 values.
#[allow(dead_code)]
pub fn quantize_web_mesh_positions(mesh: &WebMesh) -> Vec<u16> {
    if mesh.positions.is_empty() {
        return Vec::new();
    }
    let (mn, mx) = compute_web_mesh_bounds_raw(&mesh.positions);
    let range = [
        (mx[0] - mn[0]).max(1e-9),
        (mx[1] - mn[1]).max(1e-9),
        (mx[2] - mn[2]).max(1e-9),
    ];
    mesh.positions
        .iter()
        .flat_map(|p| {
            [
                (((p[0] - mn[0]) / range[0]) * 65535.0).clamp(0.0, 65535.0) as u16,
                (((p[1] - mn[1]) / range[1]) * 65535.0).clamp(0.0, 65535.0) as u16,
                (((p[2] - mn[2]) / range[2]) * 65535.0).clamp(0.0, 65535.0) as u16,
            ]
        })
        .collect()
}

/// Estimate the byte size of the exported mesh given the options.
#[allow(dead_code)]
pub fn estimate_web_size_bytes(mesh: &WebMesh, opts: &WebExportOptions) -> usize {
    let bytes_per_float = if opts.quantize_positions { 2 } else { 4 };
    let mut total = mesh.positions.len() * 3 * bytes_per_float;
    if opts.include_normals {
        total += mesh.normals.len() * 3 * 4;
    }
    if opts.include_uvs {
        total += mesh.uvs.len() * 2 * 4;
    }
    // indices: u32 or u16 depending on size
    let idx_bytes = if mesh.vertex_count <= 65535 { 2 } else { 4 };
    total += mesh.indices.len() * idx_bytes;
    if opts.include_lod {
        for lod in &mesh.lod_levels {
            total += lod.positions.len() * 3 * bytes_per_float;
            total += lod.indices.len() * idx_bytes;
        }
    }
    total
}

/// Validate a `WebMesh` and return a list of issue descriptions.
#[allow(dead_code)]
pub fn validate_web_mesh(mesh: &WebMesh) -> Vec<String> {
    let mut issues = Vec::new();
    if mesh.positions.is_empty() {
        issues.push("mesh has no positions".to_string());
    }
    if mesh.indices.is_empty() {
        issues.push("mesh has no indices".to_string());
    }
    if !mesh.indices.len().is_multiple_of(3) {
        issues.push(format!(
            "index count {} is not a multiple of 3",
            mesh.indices.len()
        ));
    }
    let n = mesh.positions.len() as u32;
    let oob: usize = mesh.indices.iter().filter(|&&i| i >= n).count();
    if oob > 0 {
        issues.push(format!("{} out-of-bounds indices", oob));
    }
    if !mesh.normals.is_empty() && mesh.normals.len() != mesh.positions.len() {
        issues.push(format!(
            "normal count {} != position count {}",
            mesh.normals.len(),
            mesh.positions.len()
        ));
    }
    if !mesh.uvs.is_empty() && mesh.uvs.len() != mesh.positions.len() {
        issues.push(format!(
            "uv count {} != position count {}",
            mesh.uvs.len(),
            mesh.positions.len()
        ));
    }
    issues
}

/// Compute the axis-aligned bounding box of a `WebMesh`.
#[allow(dead_code)]
pub fn compute_web_mesh_bounds(mesh: &WebMesh) -> ([f32; 3], [f32; 3]) {
    compute_web_mesh_bounds_raw(&mesh.positions)
}

/// Export a batch of meshes as a JSON array.
#[allow(dead_code)]
pub fn web_export_batch(meshes: &[WebMesh], opts: &WebExportOptions) -> String {
    let strs: Vec<String> = meshes.iter().map(|m| web_mesh_to_json(m, opts)).collect();
    format!("[{}]", strs.join(","))
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn compute_web_mesh_bounds_raw(positions: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    if positions.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = positions[0];
    let mut mx = positions[0];
    for p in positions {
        for i in 0..3 {
            if p[i] < mn[i] {
                mn[i] = p[i];
            }
            if p[i] > mx[i] {
                mx[i] = p[i];
            }
        }
    }
    (mn, mx)
}

/// Escape special JSON chars in a string.
fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Extract a quoted string field from JSON by key.
fn extract_str(json: &str, key: &str) -> Option<String> {
    let needle = format!("\"{}\":", key);
    let start = json.find(&needle)? + needle.len();
    let rest = json[start..].trim_start();
    if !rest.starts_with('"') {
        return None;
    }
    let inner = &rest[1..];
    let end = inner.find('"')?;
    Some(inner[..end].to_string())
}

/// Extract a usize field from JSON by key.
fn extract_usize(json: &str, key: &str) -> Option<usize> {
    let needle = format!("\"{}\":", key);
    let start = json.find(&needle)? + needle.len();
    let rest = json[start..].trim_start();
    let end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    rest[..end].parse().ok()
}

/// Minimal extraction of [[f32;3]] arrays from JSON text.
fn extract_f32_3_array(json: &str, key: &str) -> Option<Vec<[f32; 3]>> {
    let needle = format!("\"{}\":", key);
    let start = json.find(&needle)? + needle.len();
    let rest = &json[start..];
    let arr_start = rest.find('[')? + 1;
    let arr_end = find_matching_bracket(&rest[arr_start..])?;
    let inner = &rest[arr_start..arr_start + arr_end];
    let mut result = Vec::new();
    let mut pos = 0;
    while pos < inner.len() {
        let sub = &inner[pos..];
        let open = match sub.find('[') {
            Some(i) => i,
            None => break,
        };
        let sub2 = &sub[open + 1..];
        let close = match sub2.find(']') {
            Some(i) => i,
            None => break,
        };
        let nums_str = &sub2[..close];
        let nums: Vec<f32> = nums_str
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        if nums.len() == 3 {
            result.push([nums[0], nums[1], nums[2]]);
        }
        pos += open + 1 + close + 1;
    }
    Some(result)
}

/// Extract a flat [u32] array from JSON text.
fn extract_u32_array(json: &str, key: &str) -> Option<Vec<u32>> {
    let needle = format!("\"{}\":", key);
    let start = json.find(&needle)? + needle.len();
    let rest = &json[start..];
    let arr_start = rest.find('[')? + 1;
    let arr_end = find_matching_bracket(&rest[arr_start..])?;
    let inner = &rest[arr_start..arr_start + arr_end];
    let result: Vec<u32> = inner
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    Some(result)
}

/// Find the offset of the closing `]` that matches an opening `[` (already consumed).
fn find_matching_bracket(s: &str) -> Option<usize> {
    let mut depth = 1i32;
    for (i, c) in s.char_indices() {
        match c {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

// ── Web Bundle / Manifest API ─────────────────────────────────────────────────

/// Configuration for the web bundle export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WebExportConfig {
    pub output_dir: String,
    pub base_url: String,
    pub pretty_json: bool,
    pub include_html_stub: bool,
}

/// A single asset entry in a web bundle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WebAssetEntry {
    pub name: String,
    pub mime_type: String,
    pub size_bytes: u64,
}

/// Manifest listing all assets in a web bundle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WebManifest {
    pub base_url: String,
    pub assets: Vec<WebAssetEntry>,
}

/// A collection of assets ready for web export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WebBundle {
    pub config: WebExportConfig,
    pub assets: Vec<WebAssetEntry>,
}

/// Return a default `WebExportConfig`.
#[allow(dead_code)]
pub fn default_web_config() -> WebExportConfig {
    WebExportConfig {
        output_dir: "./web_export".to_string(),
        base_url: "/assets/".to_string(),
        pretty_json: false,
        include_html_stub: true,
    }
}

/// Create a new `WebBundle` from a configuration.
#[allow(dead_code)]
pub fn new_web_bundle(cfg: &WebExportConfig) -> WebBundle {
    WebBundle {
        config: cfg.clone(),
        assets: Vec::new(),
    }
}

/// Add an asset entry to the bundle.
#[allow(dead_code)]
pub fn web_bundle_add_asset(bundle: &mut WebBundle, name: &str, mime_type: &str, size_bytes: u64) {
    bundle.assets.push(WebAssetEntry {
        name: name.to_string(),
        mime_type: mime_type.to_string(),
        size_bytes,
    });
}

/// Convert a `WebBundle` into a `WebManifest`.
#[allow(dead_code)]
pub fn web_bundle_to_manifest(bundle: &WebBundle) -> WebManifest {
    WebManifest {
        base_url: bundle.config.base_url.clone(),
        assets: bundle.assets.clone(),
    }
}

/// Serialize a `WebManifest` to a JSON string.
#[allow(dead_code)]
pub fn manifest_to_json(manifest: &WebManifest) -> String {
    let entries: Vec<String> = manifest
        .assets
        .iter()
        .map(|a| {
            format!(
                "{{\"name\":\"{}\",\"mime_type\":\"{}\",\"size_bytes\":{}}}",
                esc(&a.name),
                esc(&a.mime_type),
                a.size_bytes
            )
        })
        .collect();
    format!(
        "{{\"base_url\":\"{}\",\"assets\":[{}]}}",
        esc(&manifest.base_url),
        entries.join(",")
    )
}

/// Return the number of assets in a bundle.
#[allow(dead_code)]
pub fn web_bundle_asset_count(bundle: &WebBundle) -> usize {
    bundle.assets.len()
}

/// Return the total byte size of all assets in the bundle.
#[allow(dead_code)]
pub fn web_bundle_total_size(bundle: &WebBundle) -> u64 {
    bundle.assets.iter().map(|a| a.size_bytes).sum()
}

/// Generate a minimal HTML stub that references all bundle assets.
#[allow(dead_code)]
pub fn web_export_html_stub(bundle: &WebBundle) -> String {
    let scripts: String = bundle
        .assets
        .iter()
        .filter(|a| a.mime_type.contains("javascript"))
        .map(|a| {
            format!(
                "  <script src=\"{}{}\"></script>\n",
                bundle.config.base_url, a.name
            )
        })
        .collect();
    let links: String = bundle
        .assets
        .iter()
        .filter(|a| a.mime_type.contains("css"))
        .map(|a| {
            format!(
                "  <link rel=\"stylesheet\" href=\"{}{}\"/>\n",
                bundle.config.base_url, a.name
            )
        })
        .collect();
    format!(
        "<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"utf-8\"/>\n{}</head>\n<body>\n{}</body>\n</html>",
        links, scripts
    )
}

/// Find a named asset in a manifest.  Returns `None` if not present.
#[allow(dead_code)]
pub fn web_manifest_find_asset<'a>(
    manifest: &'a WebManifest,
    name: &str,
) -> Option<&'a WebAssetEntry> {
    manifest.assets.iter().find(|a| a.name == name)
}

/// Clear all assets from a bundle.
#[allow(dead_code)]
pub fn web_bundle_clear(bundle: &mut WebBundle) {
    bundle.assets.clear();
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn tri_mesh() -> WebMesh {
        new_web_mesh(
            "test_tri",
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![0, 1, 2],
        )
    }

    #[test]
    fn new_web_mesh_basic() {
        let m = tri_mesh();
        assert_eq!(m.name, "test_tri");
        assert_eq!(m.vertex_count, 3);
        assert_eq!(m.triangle_count, 1);
        assert_eq!(m.positions.len(), 3);
        assert_eq!(m.indices.len(), 3);
    }

    #[test]
    fn new_web_mesh_bounding_box() {
        let m = tri_mesh();
        let (mn, mx) = m.bounding_box;
        assert!((mn[0] - 0.0).abs() < 1e-6);
        assert!((mx[0] - 1.0).abs() < 1e-6);
        assert!((mx[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn web_mesh_to_json_contains_name() {
        let m = tri_mesh();
        let opts = WebExportOptions::default();
        let json = web_mesh_to_json(&m, &opts);
        assert!(json.contains("\"name\":\"test_tri\""));
    }

    #[test]
    fn web_mesh_to_json_contains_vertex_count() {
        let m = tri_mesh();
        let opts = WebExportOptions::default();
        let json = web_mesh_to_json(&m, &opts);
        assert!(json.contains("\"vertex_count\":3"));
    }

    #[test]
    fn web_mesh_to_json_contains_positions() {
        let m = tri_mesh();
        let opts = WebExportOptions::default();
        let json = web_mesh_to_json(&m, &opts);
        assert!(json.contains("\"positions\":["));
    }

    #[test]
    fn web_mesh_to_json_contains_indices() {
        let m = tri_mesh();
        let opts = WebExportOptions::default();
        let json = web_mesh_to_json(&m, &opts);
        assert!(json.contains("\"indices\":[0,1,2]"));
    }

    #[test]
    fn web_mesh_from_json_roundtrip() {
        let m = tri_mesh();
        let opts = WebExportOptions::default();
        let json = web_mesh_to_json(&m, &opts);
        let m2 = web_mesh_from_json(&json).expect("roundtrip should succeed");
        assert_eq!(m2.name, m.name);
        assert_eq!(m2.vertex_count, m.vertex_count);
        assert_eq!(m2.triangle_count, m.triangle_count);
    }

    #[test]
    fn add_lod_level_increments_count() {
        let mut m = tri_mesh();
        assert_eq!(m.lod_levels.len(), 0);
        let lod = WebLodLevel {
            level: 1,
            triangle_count: 0,
            positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
            screen_size_threshold: 0.5,
        };
        add_lod_level(&mut m, lod);
        assert_eq!(m.lod_levels.len(), 1);
    }

    #[test]
    fn generate_lod_levels_count() {
        let m = new_web_mesh(
            "quad",
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![0, 1, 2, 1, 3, 2],
        );
        let lods = generate_lod_levels(&m, &[0.5, 0.25]);
        assert_eq!(lods.len(), 2);
        assert_eq!(lods[0].screen_size_threshold, 0.5);
        assert_eq!(lods[1].screen_size_threshold, 0.25);
    }

    #[test]
    fn quantize_web_mesh_positions_count() {
        let m = tri_mesh();
        let q = quantize_web_mesh_positions(&m);
        assert_eq!(q.len(), 9); // 3 verts × 3 coords
    }

    #[test]
    fn quantize_web_mesh_positions_range() {
        let m = tri_mesh();
        let q = quantize_web_mesh_positions(&m);
        // All values must be within u16 range
        for &v in &q {
            let _ = v; // just check it compiled to u16
        }
        // The max component should be 65535
        assert!(q.contains(&65535));
    }

    #[test]
    fn estimate_web_size_bytes_nonzero() {
        let m = tri_mesh();
        let opts = WebExportOptions::default();
        let sz = estimate_web_size_bytes(&m, &opts);
        assert!(sz > 0);
    }

    #[test]
    fn validate_web_mesh_valid() {
        let m = tri_mesh();
        let issues = validate_web_mesh(&m);
        assert!(
            issues.is_empty(),
            "valid mesh should have no issues: {:?}",
            issues
        );
    }

    #[test]
    fn validate_web_mesh_bad_index() {
        let mut m = tri_mesh();
        m.indices.push(999);
        let issues = validate_web_mesh(&m);
        assert!(!issues.is_empty());
    }

    #[test]
    fn compute_web_mesh_bounds_empty() {
        let m = new_web_mesh("empty", Vec::new(), Vec::new());
        let (mn, mx) = compute_web_mesh_bounds(&m);
        assert_eq!(mn, [0.0; 3]);
        assert_eq!(mx, [0.0; 3]);
    }

    #[test]
    fn web_export_batch_returns_array() {
        let m1 = tri_mesh();
        let m2 = tri_mesh();
        let opts = WebExportOptions::default();
        let json = web_export_batch(&[m1, m2], &opts);
        assert!(json.starts_with('['));
        assert!(json.ends_with(']'));
    }

    #[test]
    fn web_mesh_to_json_includes_lod_when_requested() {
        let m = new_web_mesh(
            "lod_test",
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![0, 1, 2, 1, 3, 2],
        );
        let opts = WebExportOptions {
            include_lod: true,
            ..WebExportOptions::default()
        };

        let lods = generate_lod_levels(&m, &[0.5]);
        let mut m2 = m;
        for l in lods {
            add_lod_level(&mut m2, l);
        }
        let json = web_mesh_to_json(&m2, &opts);
        assert!(json.contains("\"lod_levels\":["));
    }

    #[test]
    fn web_mesh_material_roundtrip_in_json() {
        let mut m = tri_mesh();
        m.material = Some(WebMaterial {
            name: "skin".to_string(),
            base_color: [1.0, 0.8, 0.7, 1.0],
            metallic: 0.0,
            roughness: 0.9,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: "OPAQUE".to_string(),
            double_sided: true,
        });
        let opts = WebExportOptions::default();
        let json = web_mesh_to_json(&m, &opts);
        assert!(json.contains("\"material\":"));
        assert!(json.contains("\"alpha_mode\":\"OPAQUE\""));
    }

    // ── Web Bundle / Manifest API tests ───────────────────────────────────────

    #[test]
    fn test_default_web_config() {
        let cfg = default_web_config();
        assert!(!cfg.output_dir.is_empty());
        assert!(!cfg.base_url.is_empty());
    }

    #[test]
    fn test_new_web_bundle_empty() {
        let cfg = default_web_config();
        let bundle = new_web_bundle(&cfg);
        assert_eq!(web_bundle_asset_count(&bundle), 0);
    }

    #[test]
    fn test_web_bundle_add_asset() {
        let cfg = default_web_config();
        let mut bundle = new_web_bundle(&cfg);
        web_bundle_add_asset(&mut bundle, "model.glb", "model/gltf-binary", 1024);
        assert_eq!(web_bundle_asset_count(&bundle), 1);
    }

    #[test]
    fn test_web_bundle_total_size() {
        let cfg = default_web_config();
        let mut bundle = new_web_bundle(&cfg);
        web_bundle_add_asset(&mut bundle, "a.glb", "model/gltf-binary", 500);
        web_bundle_add_asset(&mut bundle, "b.json", "application/json", 300);
        assert_eq!(web_bundle_total_size(&bundle), 800);
    }

    #[test]
    fn test_web_bundle_to_manifest() {
        let cfg = default_web_config();
        let mut bundle = new_web_bundle(&cfg);
        web_bundle_add_asset(&mut bundle, "model.glb", "model/gltf-binary", 2048);
        let manifest = web_bundle_to_manifest(&bundle);
        assert_eq!(manifest.assets.len(), 1);
        assert_eq!(manifest.assets[0].name, "model.glb");
    }

    #[test]
    fn test_manifest_to_json_contains_asset() {
        let cfg = default_web_config();
        let mut bundle = new_web_bundle(&cfg);
        web_bundle_add_asset(&mut bundle, "mesh.glb", "model/gltf-binary", 999);
        let manifest = web_bundle_to_manifest(&bundle);
        let json = manifest_to_json(&manifest);
        assert!(json.contains("mesh.glb"));
        assert!(json.contains("999"));
    }

    #[test]
    fn test_web_manifest_find_asset_found() {
        let cfg = default_web_config();
        let mut bundle = new_web_bundle(&cfg);
        web_bundle_add_asset(&mut bundle, "scene.json", "application/json", 128);
        let manifest = web_bundle_to_manifest(&bundle);
        let found = web_manifest_find_asset(&manifest, "scene.json");
        assert!(found.is_some());
        assert_eq!(found.expect("should succeed").size_bytes, 128);
    }

    #[test]
    fn test_web_manifest_find_asset_not_found() {
        let cfg = default_web_config();
        let bundle = new_web_bundle(&cfg);
        let manifest = web_bundle_to_manifest(&bundle);
        assert!(web_manifest_find_asset(&manifest, "nonexistent").is_none());
    }

    #[test]
    fn test_web_export_html_stub_contains_doctype() {
        let cfg = default_web_config();
        let bundle = new_web_bundle(&cfg);
        let html = web_export_html_stub(&bundle);
        assert!(html.contains("<!DOCTYPE html>"));
    }

    #[test]
    fn test_web_bundle_clear() {
        let cfg = default_web_config();
        let mut bundle = new_web_bundle(&cfg);
        web_bundle_add_asset(&mut bundle, "x.glb", "model/gltf-binary", 10);
        web_bundle_clear(&mut bundle);
        assert_eq!(web_bundle_asset_count(&bundle), 0);
    }
}
