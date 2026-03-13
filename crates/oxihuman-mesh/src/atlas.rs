// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! UV atlas packing using the shelf algorithm.
//!
//! Pack UV islands from multiple meshes into a single atlas texture space
//! [0..1]² for baking multiple mesh textures into one atlas.

/// A rectangular UV island to be packed.
#[derive(Debug, Clone)]
pub struct UvIsland {
    pub id: usize,
    /// UV coordinates of this island (already in local [0..1] space).
    pub uvs: Vec<[f32; 2]>,
    /// Bounding box in UV space: (min_u, min_v, max_u, max_v).
    pub bbox: (f32, f32, f32, f32),
}

impl UvIsland {
    /// Create an island from UV coordinates, computing bbox automatically.
    pub fn new(id: usize, uvs: Vec<[f32; 2]>) -> Self {
        let bbox = if uvs.is_empty() {
            (0.0, 0.0, 0.0, 0.0)
        } else {
            let mut min_u = f32::INFINITY;
            let mut min_v = f32::INFINITY;
            let mut max_u = f32::NEG_INFINITY;
            let mut max_v = f32::NEG_INFINITY;
            for uv in &uvs {
                min_u = min_u.min(uv[0]);
                min_v = min_v.min(uv[1]);
                max_u = max_u.max(uv[0]);
                max_v = max_v.max(uv[1]);
            }
            (min_u, min_v, max_u, max_v)
        };
        Self { id, uvs, bbox }
    }

    /// Width of this island's bounding box.
    pub fn width(&self) -> f32 {
        self.bbox.2 - self.bbox.0
    }

    /// Height of this island's bounding box.
    pub fn height(&self) -> f32 {
        self.bbox.3 - self.bbox.1
    }
}

/// Configuration for atlas packing.
#[derive(Debug, Clone)]
pub struct AtlasConfig {
    /// Atlas resolution hint (e.g. 1024, 2048). Used for margin calculation.
    pub resolution: u32,
    /// Padding in UV space between islands (1 pixel = 1/resolution).
    pub padding: f32,
    /// Sort islands by area before packing (usually better utilization).
    pub sort_by_area: bool,
}

impl AtlasConfig {
    /// Create a config for the given resolution with sensible defaults.
    pub fn new(resolution: u32) -> Self {
        Self {
            resolution,
            padding: 2.0 / resolution as f32,
            sort_by_area: true,
        }
    }
}

impl Default for AtlasConfig {
    fn default() -> Self {
        Self::new(1024)
    }
}

/// Result of atlas packing: placement for each island.
#[derive(Debug, Clone)]
pub struct AtlasPlacement {
    pub island_id: usize,
    /// UV offset to apply: new_uv = old_uv + (offset_u, offset_v)
    pub offset_u: f32,
    pub offset_v: f32,
    /// Scale to apply (1.0 if no scaling — basic packer doesn't scale).
    pub scale: f32,
}

/// Result of the full packing operation.
pub struct AtlasResult {
    pub placements: Vec<AtlasPlacement>,
    /// Atlas utilization [0..1] — how much of the atlas is used.
    pub utilization: f32,
    /// Number of islands that fit in the atlas (may be less than input if atlas is too small).
    pub packed_count: usize,
}

impl AtlasResult {
    /// Apply placements to remap UVs. Returns remapped UV per (island_id, vertex_idx).
    pub fn remap_uvs(&self, islands: &[UvIsland]) -> Vec<Vec<[f32; 2]>> {
        // Build a lookup map from island_id to placement.
        let mut placement_map: std::collections::HashMap<usize, &AtlasPlacement> =
            std::collections::HashMap::new();
        for p in &self.placements {
            placement_map.insert(p.island_id, p);
        }

        islands
            .iter()
            .map(|island| {
                if let Some(p) = placement_map.get(&island.id) {
                    island
                        .uvs
                        .iter()
                        .map(|uv| {
                            let new_u = uv[0] + p.offset_u;
                            let new_v = uv[1] + p.offset_v;
                            [new_u, new_v]
                        })
                        .collect()
                } else {
                    // Island was not packed (atlas full); return original UVs.
                    island.uvs.clone()
                }
            })
            .collect()
    }
}

/// Pack UV islands into a [0..1]² atlas using the shelf algorithm.
///
/// Islands are placed left-to-right on "shelves" (rows). When a shelf is full,
/// a new shelf starts above the current one.
pub fn pack_atlas(islands: &[UvIsland], config: &AtlasConfig) -> AtlasResult {
    // Clone and optionally sort by area descending.
    let mut indexed: Vec<(usize, &UvIsland)> = islands.iter().enumerate().collect();
    if config.sort_by_area {
        indexed.sort_by(|(_, a), (_, b)| {
            let area_a = a.width() * a.height();
            let area_b = b.width() * b.height();
            area_b
                .partial_cmp(&area_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    let mut placements: Vec<AtlasPlacement> = Vec::new();
    let mut cursor_u: f32 = 0.0;
    let mut cursor_v: f32 = 0.0;
    let mut shelf_height: f32 = 0.0;
    let mut packed_count: usize = 0;
    let mut total_island_area: f32 = 0.0;

    for (_, island) in &indexed {
        let w = island.width() + config.padding;
        let h = island.height() + config.padding;

        if cursor_u + w > 1.0 {
            // Start a new shelf.
            cursor_u = 0.0;
            cursor_v += shelf_height;
            shelf_height = 0.0;
        }

        if cursor_v + h > 1.0 {
            // Atlas full — stop.
            break;
        }

        // Compute offset: translate island so its bbox min aligns with cursor.
        let offset_u = cursor_u - island.bbox.0;
        let offset_v = cursor_v - island.bbox.1;

        placements.push(AtlasPlacement {
            island_id: island.id,
            offset_u,
            offset_v,
            scale: 1.0,
        });

        cursor_u += w;
        shelf_height = shelf_height.max(h);
        packed_count += 1;
        total_island_area += island.width() * island.height();
    }

    // Utilization: total island area / used atlas area.
    let used_v = cursor_v + shelf_height;
    let used_u = 1.0f32; // We always span the full width conceptually.
    let used_area = used_u * used_v;
    let utilization = if used_area > 0.0 {
        (total_island_area / used_area).min(1.0)
    } else {
        0.0
    };

    AtlasResult {
        placements,
        utilization,
        packed_count,
    }
}

/// Extract UV islands from a MeshBuffers (treats whole mesh as one island).
pub fn mesh_to_island(mesh: &crate::mesh::MeshBuffers, id: usize) -> UvIsland {
    UvIsland::new(id, mesh.uvs.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::MeshBuffers;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn make_mesh(uvs: Vec<[f32; 2]>) -> MeshBuffers {
        let n = uvs.len().max(3);
        MeshBuffers::from_morph(MB {
            positions: vec![[0.0, 0.0, 0.0]; n],
            normals: vec![[0.0, 0.0, 1.0]; n],
            uvs,
            indices: vec![0, 1, 2],
            has_suit: false,
        })
    }

    #[test]
    fn island_bbox_computed() {
        let island = UvIsland::new(0, vec![[0.1, 0.2], [0.5, 0.8]]);
        assert!((island.bbox.0 - 0.1).abs() < 1e-6, "min_u");
        assert!((island.bbox.1 - 0.2).abs() < 1e-6, "min_v");
        assert!((island.bbox.2 - 0.5).abs() < 1e-6, "max_u");
        assert!((island.bbox.3 - 0.8).abs() < 1e-6, "max_v");
    }

    #[test]
    fn island_width_height() {
        let island = UvIsland::new(0, vec![[0.1, 0.2], [0.5, 0.8]]);
        assert!((island.width() - 0.4).abs() < 1e-6, "width");
        assert!((island.height() - 0.6).abs() < 1e-6, "height");
    }

    #[test]
    fn pack_single_island_fits() {
        let island = UvIsland::new(0, vec![[0.0, 0.0], [0.2, 0.0], [0.2, 0.2], [0.0, 0.2]]);
        let config = AtlasConfig::new(1024);
        let result = pack_atlas(&[island], &config);
        assert_eq!(result.packed_count, 1);
    }

    #[test]
    fn pack_two_islands_both_fit() {
        let island_a = UvIsland::new(0, vec![[0.0, 0.0], [0.2, 0.0], [0.2, 0.2], [0.0, 0.2]]);
        let island_b = UvIsland::new(1, vec![[0.0, 0.0], [0.3, 0.0], [0.3, 0.3], [0.0, 0.3]]);
        let config = AtlasConfig::new(1024);
        let result = pack_atlas(&[island_a, island_b], &config);
        assert_eq!(result.packed_count, 2);
    }

    #[test]
    fn pack_utilization_positive() {
        let island = UvIsland::new(0, vec![[0.0, 0.0], [0.2, 0.0], [0.2, 0.2], [0.0, 0.2]]);
        let config = AtlasConfig::new(1024);
        let result = pack_atlas(&[island], &config);
        assert!(result.utilization > 0.0, "utilization should be > 0");
    }

    #[test]
    fn remap_uvs_count_matches() {
        let islands = vec![
            UvIsland::new(0, vec![[0.0, 0.0], [0.2, 0.0], [0.2, 0.2]]),
            UvIsland::new(1, vec![[0.0, 0.0], [0.3, 0.3]]),
        ];
        let config = AtlasConfig::new(1024);
        let result = pack_atlas(&islands, &config);
        let remapped = result.remap_uvs(&islands);
        assert_eq!(remapped.len(), islands.len());
    }

    #[test]
    fn remap_uvs_in_range() {
        // Use small islands near origin so remapped UVs stay in [0..1].
        let islands = vec![
            UvIsland::new(0, vec![[0.0, 0.0], [0.1, 0.0], [0.1, 0.1], [0.0, 0.1]]),
            UvIsland::new(1, vec![[0.0, 0.0], [0.1, 0.0], [0.1, 0.1], [0.0, 0.1]]),
        ];
        let config = AtlasConfig::new(1024);
        let result = pack_atlas(&islands, &config);
        let remapped = result.remap_uvs(&islands);
        for island_uvs in &remapped {
            for uv in island_uvs {
                assert!(
                    uv[0] >= -1e-5 && uv[0] <= 1.0 + 1e-5,
                    "u out of range: {}",
                    uv[0]
                );
                assert!(
                    uv[1] >= -1e-5 && uv[1] <= 1.0 + 1e-5,
                    "v out of range: {}",
                    uv[1]
                );
            }
        }
    }

    #[test]
    fn mesh_to_island_extracts_uvs() {
        let uvs = vec![[0.0f32, 0.0], [0.5, 0.0], [0.5, 0.5]];
        let mesh = make_mesh(uvs.clone());
        let island = mesh_to_island(&mesh, 0);
        assert_eq!(island.uvs.len(), mesh.uvs.len());
    }
}
