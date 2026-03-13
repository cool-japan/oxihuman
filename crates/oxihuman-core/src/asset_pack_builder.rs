// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Alpha asset pack builder for OxiHuman.
//!
//! Provides high-level builder API for creating `.oxp` asset packs that
//! bundle morph presets, texture assets, and material definitions alongside
//! conventional target deltas.
//!
//! ## Quick start
//!
//! ```rust
//! use oxihuman_core::asset_pack_builder::{AssetPackBuilder, build_alpha_pack};
//!
//! // Generate the built-in alpha sample pack
//! let bytes = build_alpha_pack();
//! assert!(!bytes.is_empty());
//!
//! // Round-trip: load the bytes back into an index
//! use oxihuman_core::asset_pack_builder::load_pack_from_bytes;
//! let index = load_pack_from_bytes(&bytes).expect("load failed");
//! assert_eq!(index.presets.len(), 5);
//! ```

use std::collections::HashMap;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::pack_distribute::{PackBuilder, PackVerifier};

// ── Texture format ───────────────────────────────────────────────────────────

/// Supported texture encoding formats.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureFormat {
    Png,
    Jpeg,
    Exr,
}

impl TextureFormat {
    fn extension(&self) -> &'static str {
        match self {
            TextureFormat::Png => "png",
            TextureFormat::Jpeg => "jpg",
            TextureFormat::Exr => "exr",
        }
    }

    #[allow(dead_code)]
    fn mime_type(&self) -> &'static str {
        match self {
            TextureFormat::Png => "image/png",
            TextureFormat::Jpeg => "image/jpeg",
            TextureFormat::Exr => "image/x-exr",
        }
    }
}

// ── Asset types ──────────────────────────────────────────────────────────────

/// A texture asset stored inside an asset pack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureAsset {
    /// Logical name for this texture (e.g. `"skin_albedo"`).
    pub name: String,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Number of channels (1 = grey, 3 = RGB, 4 = RGBA).
    pub channels: u8,
    /// Raw pixel bytes, length must equal `width * height * channels`.
    pub data: Vec<u8>,
    /// Encoding format of the stored data.
    pub format: TextureFormat,
}

impl TextureAsset {
    /// Validate that the data length is consistent with the declared dimensions.
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            bail!("texture name must not be empty");
        }
        if self.width == 0 || self.height == 0 {
            bail!("texture dimensions must be non-zero");
        }
        if self.channels == 0 || self.channels > 4 {
            bail!("texture channels must be 1–4, got {}", self.channels);
        }
        let expected = self.width as usize * self.height as usize * self.channels as usize;
        if self.data.len() != expected {
            bail!(
                "texture '{}': expected {} bytes ({} x {} x {}), got {}",
                self.name,
                expected,
                self.width,
                self.height,
                self.channels,
                self.data.len()
            );
        }
        Ok(())
    }

    /// Canonical file path within the pack archive.
    fn pack_path(&self) -> String {
        format!("textures/{}.{}", self.name, self.format.extension())
    }
}

// ── MaterialDef ──────────────────────────────────────────────────────────────

/// A PBR material definition stored inside an asset pack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialDef {
    /// Logical name (e.g. `"skin"`).
    pub name: String,
    /// Base-colour / albedo RGBA in \[0, 1\].
    pub albedo_color: [f32; 4],
    /// Metallic factor \[0, 1\].
    pub metallic: f32,
    /// Roughness factor \[0, 1\].
    pub roughness: f32,
    /// Emissive RGB in \[0, 1\].
    pub emissive: [f32; 3],
    /// Optional reference to a texture asset name for albedo.
    pub albedo_texture: Option<String>,
    /// Optional reference to a texture asset name for the normal map.
    pub normal_texture: Option<String>,
}

impl MaterialDef {
    /// Validate that scalar factors are in range.
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            bail!("material name must not be empty");
        }
        if !(0.0..=1.0).contains(&self.metallic) {
            bail!(
                "material '{}': metallic {} is out of [0,1]",
                self.name,
                self.metallic
            );
        }
        if !(0.0..=1.0).contains(&self.roughness) {
            bail!(
                "material '{}': roughness {} is out of [0,1]",
                self.name,
                self.roughness
            );
        }
        Ok(())
    }

    /// Canonical file path within the pack archive.
    fn pack_path(&self) -> String {
        format!("materials/{}.json", self.name)
    }
}

// ── MorphPreset ──────────────────────────────────────────────────────────────

/// A named collection of morph parameter values forming a body preset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphPreset {
    /// Human-readable preset name (e.g. `"Athletic"`).
    pub name: String,
    /// Short description of this preset's characteristics.
    pub description: String,
    /// Parameter name → value mapping. Values are dimensionless scalars.
    pub params: HashMap<String, f64>,
    /// Categorical tags for filtering/search (e.g. `["body", "fitness"]`).
    pub tags: Vec<String>,
}

impl MorphPreset {
    /// Validate that the preset has at least a name.
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            bail!("morph preset name must not be empty");
        }
        Ok(())
    }

    /// Canonical file path within the pack archive.
    fn pack_path(&self) -> String {
        let slug: String = self
            .name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect();
        format!("presets/{}.json", slug)
    }
}

// ── AssetPackEntry ────────────────────────────────────────────────────────────

/// A single entry that can be stored in an asset pack.
#[derive(Debug, Clone)]
pub enum AssetPackEntry {
    /// Raw target-delta data (binary blob).
    Target(TargetDelta),
    /// A texture image asset.
    Texture(TextureAsset),
    /// A PBR material definition.
    Material(MaterialDef),
    /// A morph-parameter preset.
    Preset(MorphPreset),
}

/// Raw morph target delta stored as binary data.
#[derive(Debug, Clone)]
pub struct TargetDelta {
    /// Logical name for this target.
    pub name: String,
    /// Binary data (e.g. OBJ target offsets).
    pub data: Vec<u8>,
}

// ── Pack metadata ─────────────────────────────────────────────────────────────

/// Metadata attached to the top-level pack manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPackMeta {
    /// Pack format version string.
    pub version: String,
    /// Author or organisation name.
    pub author: String,
    /// SPDX license identifier.
    pub license: String,
    /// Free-form description of the pack contents.
    pub description: String,
    /// Creation timestamp (Unix seconds). `0` if not set.
    pub created_at: u64,
}

impl Default for AssetPackMeta {
    fn default() -> Self {
        Self {
            version: "0.1.0".to_string(),
            author: String::new(),
            license: "Apache-2.0".to_string(),
            description: String::new(),
            created_at: 0,
        }
    }
}

// ── AssetPackBuilder ──────────────────────────────────────────────────────────

/// Builder for creating `.oxp` asset packs with typed entries.
///
/// # Example
/// ```rust
/// use oxihuman_core::asset_pack_builder::{AssetPackBuilder, MorphPreset};
/// use std::collections::HashMap;
///
/// let mut builder = AssetPackBuilder::new("my-pack");
/// let mut params = HashMap::new();
/// params.insert("height".to_string(), 1.8);
/// builder.add_preset(MorphPreset {
///     name: "Tall".to_string(),
///     description: "Above-average stature".to_string(),
///     params,
///     tags: vec!["height".to_string()],
/// });
/// let bytes = builder.build().expect("build failed");
/// assert!(!bytes.is_empty());
/// ```
pub struct AssetPackBuilder {
    name: String,
    meta: AssetPackMeta,
    entries: Vec<AssetPackEntry>,
}

impl AssetPackBuilder {
    /// Create a new builder for a pack with the given name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            meta: AssetPackMeta::default(),
            entries: Vec::new(),
        }
    }

    /// Set pack metadata (version, author, license, description).
    pub fn set_meta(&mut self, meta: AssetPackMeta) -> &mut Self {
        self.meta = meta;
        self
    }

    /// Convenience: set author string.
    pub fn set_author(&mut self, author: &str) -> &mut Self {
        self.meta.author = author.to_string();
        self
    }

    /// Convenience: set version string.
    pub fn set_version(&mut self, version: &str) -> &mut Self {
        self.meta.version = version.to_string();
        self
    }

    /// Convenience: set license identifier.
    pub fn set_license(&mut self, license: &str) -> &mut Self {
        self.meta.license = license.to_string();
        self
    }

    /// Convenience: set description.
    pub fn set_description(&mut self, desc: &str) -> &mut Self {
        self.meta.description = desc.to_string();
        self
    }

    /// Add a raw target delta entry.
    pub fn add_target(&mut self, delta: TargetDelta) -> &mut Self {
        self.entries.push(AssetPackEntry::Target(delta));
        self
    }

    /// Add a texture asset.  The texture is validated before insertion.
    pub fn add_texture(&mut self, tex: TextureAsset) -> Result<&mut Self> {
        tex.validate()?;
        self.entries.push(AssetPackEntry::Texture(tex));
        Ok(self)
    }

    /// Add a material definition.  The material is validated before insertion.
    pub fn add_material(&mut self, mat: MaterialDef) -> Result<&mut Self> {
        mat.validate()?;
        self.entries.push(AssetPackEntry::Material(mat));
        Ok(self)
    }

    /// Add a morph preset.  The preset is validated before insertion.
    pub fn add_preset(&mut self, preset: MorphPreset) -> Result<&mut Self> {
        preset.validate()?;
        self.entries.push(AssetPackEntry::Preset(preset));
        Ok(self)
    }

    /// Unchecked variant for internal use where validation has already run.
    fn add_preset_unchecked(&mut self, preset: MorphPreset) -> &mut Self {
        self.entries.push(AssetPackEntry::Preset(preset));
        self
    }

    /// Unchecked material variant.
    fn add_material_unchecked(&mut self, mat: MaterialDef) -> &mut Self {
        self.entries.push(AssetPackEntry::Material(mat));
        self
    }

    /// Serialize all entries into an OXP binary package.
    pub fn build(&self) -> Result<Vec<u8>> {
        let mut pack = PackBuilder::new(&self.name, &self.meta.version, &self.meta.author);
        pack.set_description(&self.meta.description);
        pack.set_license(&self.meta.license);
        pack.set_created_at(self.meta.created_at);

        for entry in &self.entries {
            match entry {
                AssetPackEntry::Target(delta) => {
                    pack.add_target_file(&delta.name, "targets", &delta.data)
                        .with_context(|| format!("failed to add target '{}'", delta.name))?;
                }
                AssetPackEntry::Texture(tex) => {
                    let json = serde_json::to_vec(tex)
                        .with_context(|| format!("failed to serialize texture '{}'", tex.name))?;
                    pack.add_target_file(&tex.pack_path(), "textures", &json)
                        .with_context(|| format!("failed to add texture '{}'", tex.name))?;
                }
                AssetPackEntry::Material(mat) => {
                    let json = serde_json::to_vec(mat)
                        .with_context(|| format!("failed to serialize material '{}'", mat.name))?;
                    pack.add_target_file(&mat.pack_path(), "materials", &json)
                        .with_context(|| format!("failed to add material '{}'", mat.name))?;
                }
                AssetPackEntry::Preset(preset) => {
                    let json = serde_json::to_vec(preset)
                        .with_context(|| format!("failed to serialize preset '{}'", preset.name))?;
                    pack.add_target_file(&preset.pack_path(), "presets", &json)
                        .with_context(|| format!("failed to add preset '{}'", preset.name))?;
                }
            }
        }

        pack.build()
    }
}

// ── AssetPackIndex ────────────────────────────────────────────────────────────

/// Summary index built by scanning a loaded OXP pack.
///
/// Returned by [`load_pack_from_bytes`].
#[derive(Debug, Clone)]
pub struct AssetPackIndex {
    /// Pack name from the manifest.
    pub name: String,
    /// Pack version from the manifest.
    pub version: String,
    /// Author from the manifest.
    pub author: String,
    /// License from the manifest.
    pub license: String,
    /// Description from the manifest.
    pub description: String,
    /// All texture assets decoded from the pack.
    pub textures: Vec<TextureAsset>,
    /// All material definitions decoded from the pack.
    pub materials: Vec<MaterialDef>,
    /// All morph presets decoded from the pack.
    pub presets: Vec<MorphPreset>,
    /// Names of raw target-delta entries.
    pub target_names: Vec<String>,
    /// Total byte size of the raw package.
    pub total_bytes: usize,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Load an OXP package from raw bytes and return a scanned index.
///
/// The integrity hash is verified before any deserialization occurs.
pub fn load_pack_from_bytes(bytes: &[u8]) -> Result<AssetPackIndex> {
    // Integrity check first
    let manifest =
        PackVerifier::verify_integrity(bytes).with_context(|| "OXP integrity check failed")?;

    let mut index = AssetPackIndex {
        name: manifest.name.clone(),
        version: manifest.version.clone(),
        author: manifest.author.clone(),
        license: manifest.license.clone(),
        description: manifest.description.clone(),
        textures: Vec::new(),
        materials: Vec::new(),
        presets: Vec::new(),
        target_names: Vec::new(),
        total_bytes: bytes.len(),
    };

    // Scan every target entry by category prefix
    for entry in &manifest.targets {
        match entry.category.as_str() {
            "textures" => {
                let data = PackVerifier::extract_file(bytes, &entry.file_path)
                    .with_context(|| format!("extracting texture '{}'", entry.file_path))?;
                let tex: TextureAsset = serde_json::from_slice(&data)
                    .with_context(|| format!("deserializing texture '{}'", entry.file_path))?;
                index.textures.push(tex);
            }
            "materials" => {
                let data = PackVerifier::extract_file(bytes, &entry.file_path)
                    .with_context(|| format!("extracting material '{}'", entry.file_path))?;
                let mat: MaterialDef = serde_json::from_slice(&data)
                    .with_context(|| format!("deserializing material '{}'", entry.file_path))?;
                index.materials.push(mat);
            }
            "presets" => {
                let data = PackVerifier::extract_file(bytes, &entry.file_path)
                    .with_context(|| format!("extracting preset '{}'", entry.file_path))?;
                let preset: MorphPreset = serde_json::from_slice(&data)
                    .with_context(|| format!("deserializing preset '{}'", entry.file_path))?;
                index.presets.push(preset);
            }
            "targets" => {
                index.target_names.push(entry.name.clone());
            }
            other => {
                // Unknown category: silently skip (forward-compatible)
                let _ = other;
            }
        }
    }

    Ok(index)
}

// ── Alpha pack factory ────────────────────────────────────────────────────────

/// Build the built-in alpha sample asset pack.
///
/// Contains:
/// - 5 body morph presets: Athletic, Slim, Heavy, Tall, Short
/// - 3 PBR material definitions: Skin, Cloth, Metal
/// - Pack manifest with version, author, and license info
///
/// Returns the serialized OXP bytes.  This function never fails.
pub fn build_alpha_pack() -> Vec<u8> {
    build_alpha_pack_inner().unwrap_or_else(|_| Vec::new())
}

fn build_alpha_pack_inner() -> Result<Vec<u8>> {
    let mut builder = AssetPackBuilder::new("oxihuman-alpha");
    builder
        .set_version("0.1.0-alpha")
        .set_author("COOLJAPAN OU (Team Kitasan)")
        .set_license("Apache-2.0")
        .set_description("OxiHuman alpha sample asset pack — body presets and PBR materials.");

    // ── 5 Morph Presets ─────────────────────────────────────────────────────

    builder.add_preset_unchecked(MorphPreset {
        name: "Athletic".to_string(),
        description: "Well-developed musculature, low body fat, balanced proportions.".to_string(),
        params: {
            let mut m = HashMap::new();
            m.insert("muscle_mass".to_string(), 0.75);
            m.insert("body_fat".to_string(), 0.12);
            m.insert("height_scale".to_string(), 1.0);
            m.insert("shoulder_width".to_string(), 0.6);
            m.insert("waist_width".to_string(), 0.38);
            m
        },
        tags: vec![
            "fitness".to_string(),
            "sport".to_string(),
            "body".to_string(),
        ],
    });

    builder.add_preset_unchecked(MorphPreset {
        name: "Slim".to_string(),
        description: "Slender frame with minimal muscle definition and low body fat.".to_string(),
        params: {
            let mut m = HashMap::new();
            m.insert("muscle_mass".to_string(), 0.30);
            m.insert("body_fat".to_string(), 0.10);
            m.insert("height_scale".to_string(), 1.0);
            m.insert("shoulder_width".to_string(), 0.42);
            m.insert("waist_width".to_string(), 0.32);
            m
        },
        tags: vec!["slim".to_string(), "body".to_string()],
    });

    builder.add_preset_unchecked(MorphPreset {
        name: "Heavy".to_string(),
        description: "Larger frame with higher body fat and increased mass.".to_string(),
        params: {
            let mut m = HashMap::new();
            m.insert("muscle_mass".to_string(), 0.45);
            m.insert("body_fat".to_string(), 0.38);
            m.insert("height_scale".to_string(), 1.0);
            m.insert("shoulder_width".to_string(), 0.68);
            m.insert("waist_width".to_string(), 0.62);
            m
        },
        tags: vec![
            "heavy".to_string(),
            "overweight".to_string(),
            "body".to_string(),
        ],
    });

    builder.add_preset_unchecked(MorphPreset {
        name: "Tall".to_string(),
        description: "Above-average height with proportionally elongated limbs.".to_string(),
        params: {
            let mut m = HashMap::new();
            m.insert("muscle_mass".to_string(), 0.50);
            m.insert("body_fat".to_string(), 0.18);
            m.insert("height_scale".to_string(), 1.20);
            m.insert("leg_length".to_string(), 0.65);
            m.insert("torso_length".to_string(), 0.60);
            m
        },
        tags: vec!["tall".to_string(), "height".to_string(), "body".to_string()],
    });

    builder.add_preset_unchecked(MorphPreset {
        name: "Short".to_string(),
        description: "Below-average height with proportionally compact build.".to_string(),
        params: {
            let mut m = HashMap::new();
            m.insert("muscle_mass".to_string(), 0.50);
            m.insert("body_fat".to_string(), 0.18);
            m.insert("height_scale".to_string(), 0.82);
            m.insert("leg_length".to_string(), 0.45);
            m.insert("torso_length".to_string(), 0.44);
            m
        },
        tags: vec![
            "short".to_string(),
            "height".to_string(),
            "body".to_string(),
        ],
    });

    // ── 3 Material Definitions ───────────────────────────────────────────────

    builder.add_material_unchecked(MaterialDef {
        name: "Skin".to_string(),
        albedo_color: [0.87, 0.72, 0.60, 1.0],
        metallic: 0.0,
        roughness: 0.70,
        emissive: [0.0, 0.0, 0.0],
        albedo_texture: Some("skin_albedo".to_string()),
        normal_texture: Some("skin_normal".to_string()),
    });

    builder.add_material_unchecked(MaterialDef {
        name: "Cloth".to_string(),
        albedo_color: [0.40, 0.40, 0.55, 1.0],
        metallic: 0.0,
        roughness: 0.90,
        emissive: [0.0, 0.0, 0.0],
        albedo_texture: Some("cloth_albedo".to_string()),
        normal_texture: None,
    });

    builder.add_material_unchecked(MaterialDef {
        name: "Metal".to_string(),
        albedo_color: [0.80, 0.80, 0.82, 1.0],
        metallic: 0.95,
        roughness: 0.20,
        emissive: [0.0, 0.0, 0.0],
        albedo_texture: None,
        normal_texture: None,
    });

    builder.build()
}

// ── Distribution manifest ─────────────────────────────────────────────────────

/// Recursively collect all file paths within a directory.
///
/// Returns a sorted list of `(relative_path_string, absolute_path)` pairs.
fn collect_files_recursive(
    base: &std::path::Path,
    dir: &std::path::Path,
    out: &mut Vec<(String, std::path::PathBuf)>,
) -> Result<()> {
    for entry in
        std::fs::read_dir(dir).with_context(|| format!("reading dir: {}", dir.display()))?
    {
        let entry = entry.with_context(|| format!("iterating dir: {}", dir.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(base, &path, out)?;
        } else if path.is_file() {
            let rel = path
                .strip_prefix(base)
                .map_err(|e| anyhow::anyhow!("strip prefix {}: {e}", path.display()))?
                .to_string_lossy()
                .into_owned();
            out.push((rel, path));
        }
    }
    Ok(())
}

/// Generate a distribution manifest JSON for an asset pack directory.
///
/// Walks `pack_dir` recursively and records the SHA-256 hash of every file.
/// Returns a pretty-printed JSON string suitable for saving as
/// `<pack-name>.dist-manifest.json`.
///
/// # Errors
///
/// Returns an error if any file cannot be read or if JSON serialization fails.
pub fn generate_distribution_manifest(pack_dir: &std::path::Path) -> Result<String> {
    use sha2::{Digest, Sha256};

    let mut pairs: Vec<(String, std::path::PathBuf)> = Vec::new();
    collect_files_recursive(pack_dir, pack_dir, &mut pairs)?;
    // Sort for deterministic output.
    pairs.sort_by(|a, b| a.0.cmp(&b.0));

    let mut entries: HashMap<String, String> = HashMap::new();
    for (rel, abs) in pairs {
        let data =
            std::fs::read(&abs).with_context(|| format!("reading file: {}", abs.display()))?;
        let hash = hex::encode(Sha256::digest(&data));
        entries.insert(rel, hash);
    }

    let manifest = serde_json::json!({
        "schema_version": "0.1.1",
        "files": entries,
    });
    serde_json::to_string_pretty(&manifest).context("serializing distribution manifest")
}

/// Verify a distribution manifest against the actual files in `pack_dir`.
///
/// Returns `true` if every file listed in the manifest exists on disk and
/// its SHA-256 hash matches the recorded value.  Returns `false` (not an
/// error) when a file is missing or its hash differs.
///
/// # Errors
///
/// Returns an error only for I/O failures or malformed manifest JSON.
pub fn verify_distribution_manifest(
    manifest_json: &str,
    pack_dir: &std::path::Path,
) -> Result<bool> {
    use sha2::{Digest, Sha256};

    let manifest: serde_json::Value =
        serde_json::from_str(manifest_json).context("parsing distribution manifest JSON")?;
    let files = manifest["files"]
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("manifest missing 'files' object"))?;

    for (rel_path, expected_value) in files {
        let expected = expected_value
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("hash not a string for '{}'", rel_path))?;
        let full_path = pack_dir.join(rel_path);
        if !full_path.exists() {
            return Ok(false);
        }
        let data = std::fs::read(&full_path)
            .with_context(|| format!("reading file for verification: {}", full_path.display()))?;
        let actual = hex::encode(Sha256::digest(&data));
        if actual != expected {
            return Ok(false);
        }
    }
    Ok(true)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helper ───────────────────────────────────────────────────────────────

    fn make_1x1_texture(name: &str) -> TextureAsset {
        TextureAsset {
            name: name.to_string(),
            width: 1,
            height: 1,
            channels: 3,
            data: vec![255, 0, 0],
            format: TextureFormat::Png,
        }
    }

    fn make_simple_preset(name: &str) -> MorphPreset {
        let mut params = HashMap::new();
        params.insert("height_scale".to_string(), 1.0);
        MorphPreset {
            name: name.to_string(),
            description: "Test preset".to_string(),
            params,
            tags: vec!["test".to_string()],
        }
    }

    fn make_simple_material(name: &str) -> MaterialDef {
        MaterialDef {
            name: name.to_string(),
            albedo_color: [0.8, 0.6, 0.4, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            emissive: [0.0, 0.0, 0.0],
            albedo_texture: None,
            normal_texture: None,
        }
    }

    // 1. Empty builder produces valid OXP
    #[test]
    fn empty_builder_produces_valid_oxp() {
        let builder = AssetPackBuilder::new("empty-pack");
        let bytes = builder.build().expect("should succeed");
        assert!(!bytes.is_empty());
        // Integrity must pass
        let index = load_pack_from_bytes(&bytes).expect("should succeed");
        assert_eq!(index.name, "empty-pack");
    }

    // 2. Build with one preset round-trips
    #[test]
    fn single_preset_round_trip() {
        let mut builder = AssetPackBuilder::new("preset-pack");
        builder
            .add_preset(make_simple_preset("Alpha"))
            .expect("should succeed");
        let bytes = builder.build().expect("should succeed");
        let index = load_pack_from_bytes(&bytes).expect("should succeed");
        assert_eq!(index.presets.len(), 1);
        assert_eq!(index.presets[0].name, "Alpha");
    }

    // 3. Multiple presets preserved in order
    #[test]
    fn multiple_presets_round_trip() {
        let mut builder = AssetPackBuilder::new("multi-preset");
        for name in &["A", "B", "C"] {
            builder
                .add_preset(make_simple_preset(name))
                .expect("should succeed");
        }
        let bytes = builder.build().expect("should succeed");
        let index = load_pack_from_bytes(&bytes).expect("should succeed");
        assert_eq!(index.presets.len(), 3);
        let names: Vec<&str> = index.presets.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"A"));
        assert!(names.contains(&"B"));
        assert!(names.contains(&"C"));
    }

    // 4. Texture round-trip
    #[test]
    fn texture_round_trip() {
        let mut builder = AssetPackBuilder::new("tex-pack");
        builder
            .add_texture(make_1x1_texture("red"))
            .expect("should succeed");
        let bytes = builder.build().expect("should succeed");
        let index = load_pack_from_bytes(&bytes).expect("should succeed");
        assert_eq!(index.textures.len(), 1);
        assert_eq!(index.textures[0].name, "red");
        assert_eq!(index.textures[0].width, 1);
        assert_eq!(index.textures[0].channels, 3);
    }

    // 5. Material round-trip
    #[test]
    fn material_round_trip() {
        let mut builder = AssetPackBuilder::new("mat-pack");
        builder
            .add_material(make_simple_material("chrome"))
            .expect("should succeed");
        let bytes = builder.build().expect("should succeed");
        let index = load_pack_from_bytes(&bytes).expect("should succeed");
        assert_eq!(index.materials.len(), 1);
        assert_eq!(index.materials[0].name, "chrome");
        assert!((index.materials[0].roughness - 0.5).abs() < 1e-6);
    }

    // 6. Mixed entries all decoded
    #[test]
    fn mixed_entries_round_trip() {
        let mut builder = AssetPackBuilder::new("mixed-pack");
        builder
            .add_preset(make_simple_preset("P1"))
            .expect("should succeed");
        builder
            .add_texture(make_1x1_texture("T1"))
            .expect("should succeed");
        builder
            .add_material(make_simple_material("M1"))
            .expect("should succeed");
        let bytes = builder.build().expect("should succeed");
        let index = load_pack_from_bytes(&bytes).expect("should succeed");
        assert_eq!(index.presets.len(), 1);
        assert_eq!(index.textures.len(), 1);
        assert_eq!(index.materials.len(), 1);
    }

    // 7. Target delta preserved
    #[test]
    fn target_delta_round_trip() {
        let mut builder = AssetPackBuilder::new("delta-pack");
        builder.add_target(TargetDelta {
            name: "head_big.target".to_string(),
            data: vec![1, 2, 3, 4, 5],
        });
        let bytes = builder.build().expect("should succeed");
        let index = load_pack_from_bytes(&bytes).expect("should succeed");
        assert_eq!(index.target_names.len(), 1);
        assert_eq!(index.target_names[0], "head_big.target");
    }

    // 8. Alpha pack generation succeeds and is non-empty
    #[test]
    fn alpha_pack_not_empty() {
        let bytes = build_alpha_pack();
        assert!(!bytes.is_empty(), "alpha pack must produce bytes");
    }

    // 9. Alpha pack has exactly 5 presets
    #[test]
    fn alpha_pack_has_five_presets() {
        let bytes = build_alpha_pack();
        let index = load_pack_from_bytes(&bytes).expect("should succeed");
        assert_eq!(index.presets.len(), 5);
    }

    // 10. Alpha pack has exactly 3 materials
    #[test]
    fn alpha_pack_has_three_materials() {
        let bytes = build_alpha_pack();
        let index = load_pack_from_bytes(&bytes).expect("should succeed");
        assert_eq!(index.materials.len(), 3);
    }

    // 11. Alpha pack preset names are correct
    #[test]
    fn alpha_pack_preset_names() {
        let bytes = build_alpha_pack();
        let index = load_pack_from_bytes(&bytes).expect("should succeed");
        let names: Vec<&str> = index.presets.iter().map(|p| p.name.as_str()).collect();
        for expected in &["Athletic", "Slim", "Heavy", "Tall", "Short"] {
            assert!(names.contains(expected), "missing preset: {}", expected);
        }
    }

    // 12. Alpha pack material names are correct
    #[test]
    fn alpha_pack_material_names() {
        let bytes = build_alpha_pack();
        let index = load_pack_from_bytes(&bytes).expect("should succeed");
        let names: Vec<&str> = index.materials.iter().map(|m| m.name.as_str()).collect();
        for expected in &["Skin", "Cloth", "Metal"] {
            assert!(names.contains(expected), "missing material: {}", expected);
        }
    }

    // 13. Alpha pack manifest has correct author
    #[test]
    fn alpha_pack_manifest_author() {
        let bytes = build_alpha_pack();
        let index = load_pack_from_bytes(&bytes).expect("should succeed");
        assert!(
            index.author.contains("COOLJAPAN"),
            "unexpected author: {}",
            index.author
        );
    }

    // 14. Alpha pack manifest has license info
    #[test]
    fn alpha_pack_manifest_license() {
        let bytes = build_alpha_pack();
        let index = load_pack_from_bytes(&bytes).expect("should succeed");
        assert!(!index.license.is_empty());
    }

    // 15. Alpha pack integrity verified
    #[test]
    fn alpha_pack_integrity_ok() {
        let bytes = build_alpha_pack();
        // load_pack_from_bytes runs verify_integrity internally
        assert!(load_pack_from_bytes(&bytes).is_ok());
    }

    // 16. Texture validation rejects wrong data length
    #[test]
    fn texture_validation_wrong_length() {
        let tex = TextureAsset {
            name: "bad".to_string(),
            width: 4,
            height: 4,
            channels: 3,
            data: vec![0u8; 10], // too short
            format: TextureFormat::Png,
        };
        assert!(tex.validate().is_err());
    }

    // 17. Texture validation rejects zero dimensions
    #[test]
    fn texture_validation_zero_dimension() {
        let tex = TextureAsset {
            name: "bad".to_string(),
            width: 0,
            height: 1,
            channels: 3,
            data: vec![],
            format: TextureFormat::Jpeg,
        };
        assert!(tex.validate().is_err());
    }

    // 18. Texture validation rejects empty name
    #[test]
    fn texture_validation_empty_name() {
        let tex = TextureAsset {
            name: String::new(),
            width: 1,
            height: 1,
            channels: 3,
            data: vec![0, 0, 0],
            format: TextureFormat::Png,
        };
        assert!(tex.validate().is_err());
    }

    // 19. Material validation rejects metallic out of range
    #[test]
    fn material_validation_metallic_out_of_range() {
        let mat = MaterialDef {
            name: "bad".to_string(),
            albedo_color: [1.0, 0.0, 0.0, 1.0],
            metallic: 1.5,
            roughness: 0.5,
            emissive: [0.0; 3],
            albedo_texture: None,
            normal_texture: None,
        };
        assert!(mat.validate().is_err());
    }

    // 20. Material validation rejects roughness out of range
    #[test]
    fn material_validation_roughness_out_of_range() {
        let mat = MaterialDef {
            name: "bad".to_string(),
            albedo_color: [1.0, 0.0, 0.0, 1.0],
            metallic: 0.5,
            roughness: -0.1,
            emissive: [0.0; 3],
            albedo_texture: None,
            normal_texture: None,
        };
        assert!(mat.validate().is_err());
    }

    // 21. Material validation rejects empty name
    #[test]
    fn material_validation_empty_name() {
        let mat = MaterialDef {
            name: String::new(),
            albedo_color: [1.0, 0.0, 0.0, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            emissive: [0.0; 3],
            albedo_texture: None,
            normal_texture: None,
        };
        assert!(mat.validate().is_err());
    }

    // 22. Preset validation rejects empty name
    #[test]
    fn preset_validation_empty_name() {
        let preset = MorphPreset {
            name: String::new(),
            description: "desc".to_string(),
            params: HashMap::new(),
            tags: vec![],
        };
        assert!(preset.validate().is_err());
    }

    // 23. Index total_bytes reflects actual pack size
    #[test]
    fn index_total_bytes() {
        let mut builder = AssetPackBuilder::new("size-pack");
        builder
            .add_preset(make_simple_preset("X"))
            .expect("should succeed");
        let bytes = builder.build().expect("should succeed");
        let expected_len = bytes.len();
        let index = load_pack_from_bytes(&bytes).expect("should succeed");
        assert_eq!(index.total_bytes, expected_len);
    }

    // 24. Builder metadata propagates into index
    #[test]
    fn builder_metadata_in_index() {
        let mut builder = AssetPackBuilder::new("meta-pack");
        builder
            .set_author("Alice")
            .set_version("2.0.0")
            .set_license("MIT")
            .set_description("A test pack");
        let bytes = builder.build().expect("should succeed");
        let index = load_pack_from_bytes(&bytes).expect("should succeed");
        assert_eq!(index.author, "Alice");
        assert_eq!(index.version, "2.0.0");
        assert_eq!(index.license, "MIT");
        assert_eq!(index.description, "A test pack");
    }

    // 25. Corrupted bytes rejected by load_pack_from_bytes
    #[test]
    fn corrupted_bytes_rejected() {
        let mut builder = AssetPackBuilder::new("pack");
        builder
            .add_preset(make_simple_preset("P"))
            .expect("should succeed");
        let mut bytes = builder.build().expect("should succeed");
        // Flip a byte near the middle to corrupt the pack
        let mid = bytes.len() / 2;
        bytes[mid] ^= 0xFF;
        assert!(load_pack_from_bytes(&bytes).is_err());
    }

    // 26. TextureFormat extension and mime_type helpers
    #[test]
    fn texture_format_helpers() {
        assert_eq!(TextureFormat::Png.extension(), "png");
        assert_eq!(TextureFormat::Jpeg.extension(), "jpg");
        assert_eq!(TextureFormat::Exr.extension(), "exr");
        assert_eq!(TextureFormat::Png.mime_type(), "image/png");
    }

    // 27. Preset pack_path is unique per name
    #[test]
    fn preset_pack_paths_unique() {
        let p1 = make_simple_preset("Athletic Body");
        let p2 = make_simple_preset("Slim Body");
        assert_ne!(p1.pack_path(), p2.pack_path());
    }

    // 28. Alpha pack Athletic preset has expected params
    #[test]
    fn alpha_pack_athletic_params() {
        let bytes = build_alpha_pack();
        let index = load_pack_from_bytes(&bytes).expect("should succeed");
        let athletic = index
            .presets
            .iter()
            .find(|p| p.name == "Athletic")
            .expect("Athletic preset not found");
        assert!(
            athletic.params.contains_key("muscle_mass"),
            "Athletic preset missing muscle_mass param"
        );
        let mm = athletic.params["muscle_mass"];
        assert!(mm > 0.5, "Athletic muscle_mass should be > 0.5, got {}", mm);
    }

    // 29. Alpha pack Metal material is highly metallic
    #[test]
    fn alpha_pack_metal_material_metallic() {
        let bytes = build_alpha_pack();
        let index = load_pack_from_bytes(&bytes).expect("should succeed");
        let metal = index
            .materials
            .iter()
            .find(|m| m.name == "Metal")
            .expect("Metal material not found");
        assert!(
            metal.metallic > 0.9,
            "Metal material should have metallic > 0.9, got {}",
            metal.metallic
        );
    }

    // 30. AssetPackMeta default values
    #[test]
    fn asset_pack_meta_defaults() {
        let meta = AssetPackMeta::default();
        assert_eq!(meta.version, "0.1.0");
        assert_eq!(meta.license, "Apache-2.0");
        assert_eq!(meta.created_at, 0);
    }

    // 31–35. Distribution manifest tests

    #[test]
    fn test_generate_and_verify_manifest() {
        let dir = std::env::temp_dir().join("oxihuman_dist_test_basic");
        std::fs::create_dir_all(&dir).expect("create dir");
        std::fs::write(dir.join("test.bin"), b"hello world").expect("write test.bin");

        let manifest = generate_distribution_manifest(&dir).expect("generate manifest");
        assert!(
            manifest.contains("test.bin"),
            "manifest should reference test.bin"
        );
        assert!(
            manifest.contains("schema_version"),
            "manifest should have schema_version"
        );

        let ok = verify_distribution_manifest(&manifest, &dir).expect("verify manifest");
        assert!(ok, "fresh manifest must verify successfully");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_manifest_detects_tampered_file() {
        let dir = std::env::temp_dir().join("oxihuman_dist_test_tamper");
        std::fs::create_dir_all(&dir).expect("create dir");
        std::fs::write(dir.join("data.bin"), b"original").expect("write data.bin");

        let manifest = generate_distribution_manifest(&dir).expect("generate manifest");

        // Tamper with the file contents
        std::fs::write(dir.join("data.bin"), b"tampered!").expect("overwrite data.bin");

        let ok = verify_distribution_manifest(&manifest, &dir).expect("verify call should not err");
        assert!(!ok, "tampered file must cause verification failure");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_manifest_detects_missing_file() {
        let dir = std::env::temp_dir().join("oxihuman_dist_test_missing");
        std::fs::create_dir_all(&dir).expect("create dir");
        std::fs::write(dir.join("present.bin"), b"data").expect("write present.bin");

        let manifest = generate_distribution_manifest(&dir).expect("generate manifest");

        // Delete the file to simulate a missing-file scenario
        std::fs::remove_file(dir.join("present.bin")).ok();

        let ok = verify_distribution_manifest(&manifest, &dir).expect("verify call should not err");
        assert!(!ok, "missing file must cause verification failure");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_manifest_schema_version() {
        let dir = std::env::temp_dir().join("oxihuman_dist_test_schema");
        std::fs::create_dir_all(&dir).expect("create dir");
        std::fs::write(dir.join("x.bin"), b"x").expect("write x.bin");

        let manifest = generate_distribution_manifest(&dir).expect("generate manifest");
        let v: serde_json::Value =
            serde_json::from_str(&manifest).expect("manifest must be valid JSON");
        assert_eq!(
            v["schema_version"]
                .as_str()
                .expect("schema_version must be string"),
            "0.1.1"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_manifest_multiple_files() {
        let dir = std::env::temp_dir().join("oxihuman_dist_test_multi");
        std::fs::create_dir_all(&dir).expect("create dir");
        std::fs::write(dir.join("a.bin"), b"alpha").expect("write a.bin");
        std::fs::write(dir.join("b.bin"), b"beta").expect("write b.bin");
        std::fs::write(dir.join("c.bin"), b"gamma").expect("write c.bin");

        let manifest = generate_distribution_manifest(&dir).expect("generate manifest");
        let v: serde_json::Value =
            serde_json::from_str(&manifest).expect("manifest must be valid JSON");
        let files = v["files"].as_object().expect("files must be object");
        assert_eq!(files.len(), 3, "should have exactly 3 entries");

        let ok = verify_distribution_manifest(&manifest, &dir).expect("verify");
        assert!(ok);

        std::fs::remove_dir_all(&dir).ok();
    }
}
