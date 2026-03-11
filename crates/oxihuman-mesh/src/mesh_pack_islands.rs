// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! UV island packing.

/// A UV island defined by its bounding box.
#[derive(Debug, Clone)]
pub struct UvIslandBounds {
    pub id: usize,
    pub u_min: f32,
    pub v_min: f32,
    pub u_max: f32,
    pub v_max: f32,
}

impl UvIslandBounds {
    /// Create from min/max coordinates.
    pub fn new(id: usize, u_min: f32, v_min: f32, u_max: f32, v_max: f32) -> Self {
        Self {
            id,
            u_min,
            v_min,
            u_max,
            v_max,
        }
    }

    /// Width of the island.
    pub fn width(&self) -> f32 {
        (self.u_max - self.u_min).max(0.0)
    }

    /// Height of the island.
    pub fn height(&self) -> f32 {
        (self.v_max - self.v_min).max(0.0)
    }

    /// Area of the island.
    pub fn area(&self) -> f32 {
        self.width() * self.height()
    }
}

/// Result of packing a single island.
#[derive(Debug, Clone)]
pub struct PackedIsland {
    pub id: usize,
    pub u_offset: f32,
    pub v_offset: f32,
}

/// Simple shelf-packing algorithm for UV islands.
/// Islands are sorted by height (descending) then placed left-to-right.
pub fn pack_uv_islands(
    islands: &[UvIslandBounds],
    atlas_width: f32,
    margin: f32,
) -> Vec<PackedIsland> {
    let mut sorted: Vec<&UvIslandBounds> = islands.iter().collect();
    sorted.sort_by(|a, b| {
        b.height()
            .partial_cmp(&a.height())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut packed = Vec::new();
    let mut shelf_x = 0.0_f32;
    let mut shelf_y = 0.0_f32;
    let mut shelf_h = 0.0_f32;

    for island in sorted {
        let w = island.width() + margin;
        let h = island.height() + margin;
        if shelf_x + w > atlas_width {
            shelf_y += shelf_h;
            shelf_x = 0.0;
            shelf_h = 0.0;
        }
        packed.push(PackedIsland {
            id: island.id,
            u_offset: shelf_x,
            v_offset: shelf_y,
        });
        shelf_x += w;
        if h > shelf_h {
            shelf_h = h;
        }
    }
    packed
}

/// Total area of all islands.
pub fn total_island_area(islands: &[UvIslandBounds]) -> f32 {
    islands.iter().map(|i| i.area()).sum()
}

/// Atlas utilization ratio (total island area / atlas area).
pub fn atlas_utilization(islands: &[UvIslandBounds], atlas_area: f32) -> f32 {
    if atlas_area < 1e-8 {
        return 0.0;
    }
    total_island_area(islands) / atlas_area
}

/// Find the island with the largest area.
pub fn largest_island(islands: &[UvIslandBounds]) -> Option<usize> {
    islands
        .iter()
        .enumerate()
        .max_by(|a, b| {
            a.1.area()
                .partial_cmp(&b.1.area())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_islands() -> Vec<UvIslandBounds> {
        vec![
            UvIslandBounds::new(0, 0.0, 0.0, 0.3, 0.3),
            UvIslandBounds::new(1, 0.0, 0.0, 0.5, 0.2),
            UvIslandBounds::new(2, 0.0, 0.0, 0.1, 0.5),
        ]
    }

    #[test]
    fn test_island_width_height() {
        /* width and height computed from bounds */
        let island = UvIslandBounds::new(0, 0.1, 0.2, 0.6, 0.9);
        assert!((island.width() - 0.5).abs() < 1e-6);
        assert!((island.height() - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_island_area() {
        /* area is width * height */
        let island = UvIslandBounds::new(0, 0.0, 0.0, 2.0, 3.0);
        assert!((island.area() - 6.0).abs() < 1e-6);
    }

    #[test]
    fn test_pack_count() {
        /* packing produces one result per island */
        let islands = sample_islands();
        let packed = pack_uv_islands(&islands, 1.0, 0.01);
        assert_eq!(packed.len(), islands.len());
    }

    #[test]
    fn test_pack_offsets_non_negative() {
        /* packed offsets are non-negative */
        let islands = sample_islands();
        let packed = pack_uv_islands(&islands, 1.0, 0.01);
        for p in &packed {
            assert!(p.u_offset >= 0.0);
            assert!(p.v_offset >= 0.0);
        }
    }

    #[test]
    fn test_total_island_area() {
        /* total area sums correctly */
        let islands = sample_islands();
        let total = total_island_area(&islands);
        assert!(total > 0.0);
    }

    #[test]
    fn test_atlas_utilization_range() {
        /* utilization is in [0, 1] for reasonable atlas */
        let islands = sample_islands();
        let util = atlas_utilization(&islands, 1.0);
        assert!((0.0..=1.0).contains(&util));
    }

    #[test]
    fn test_largest_island() {
        /* largest island index is correct */
        let islands = sample_islands();
        let idx = largest_island(&islands).unwrap();
        /* island 2 has height 0.5 width 0.1 = 0.05; island 0 = 0.09; island 1 = 0.1 */
        /* island 1 (0.5*0.2=0.10) should be largest */
        assert_eq!(islands[idx].id, 1);
    }

    #[test]
    fn test_largest_island_empty() {
        /* largest island on empty list returns None */
        assert!(largest_island(&[]).is_none());
    }
}
