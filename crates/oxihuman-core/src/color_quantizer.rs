// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Median-cut color quantization for RGB palettes.

/// An RGB color (0-255 per channel).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        RgbColor { r, g, b }
    }
}

/// Median-cut color quantizer.
pub struct ColorQuantizer {
    colors: Vec<RgbColor>,
}

/// Construct a new ColorQuantizer from a list of colors.
pub fn new_color_quantizer(colors: Vec<RgbColor>) -> ColorQuantizer {
    ColorQuantizer { colors }
}

impl ColorQuantizer {
    /// Quantize to `num_colors` palette entries using median-cut.
    pub fn quantize(&self, num_colors: usize) -> Vec<RgbColor> {
        if self.colors.is_empty() || num_colors == 0 {
            return Vec::new();
        }
        let buckets = median_cut(self.colors.clone(), num_colors);
        buckets.into_iter().map(|b| average_color(&b)).collect()
    }

    /// Find the closest palette color to `query`.
    pub fn closest(palette: &[RgbColor], query: RgbColor) -> Option<RgbColor> {
        palette
            .iter()
            .min_by_key(|&&c| color_dist_sq(c, query))
            .copied()
    }

    /// Number of input colors.
    pub fn color_count(&self) -> usize {
        self.colors.len()
    }
}

/// Compute squared Euclidean distance between two RGB colors.
pub fn color_dist_sq(a: RgbColor, b: RgbColor) -> u32 {
    let dr = a.r as i32 - b.r as i32;
    let dg = a.g as i32 - b.g as i32;
    let db = a.b as i32 - b.b as i32;
    (dr * dr + dg * dg + db * db) as u32
}

fn average_color(colors: &[RgbColor]) -> RgbColor {
    if colors.is_empty() {
        return RgbColor::new(0, 0, 0);
    }
    let n = colors.len() as u32;
    let r = (colors.iter().map(|c| c.r as u32).sum::<u32>() / n) as u8;
    let g = (colors.iter().map(|c| c.g as u32).sum::<u32>() / n) as u8;
    let b = (colors.iter().map(|c| c.b as u32).sum::<u32>() / n) as u8;
    RgbColor::new(r, g, b)
}

fn channel_range(colors: &[RgbColor]) -> (u8, u8, u8) {
    let (mut rmin, mut rmax) = (255u8, 0u8);
    let (mut gmin, mut gmax) = (255u8, 0u8);
    let (mut bmin, mut bmax) = (255u8, 0u8);
    for &c in colors {
        if c.r < rmin {
            rmin = c.r;
        }
        if c.r > rmax {
            rmax = c.r;
        }
        if c.g < gmin {
            gmin = c.g;
        }
        if c.g > gmax {
            gmax = c.g;
        }
        if c.b < bmin {
            bmin = c.b;
        }
        if c.b > bmax {
            bmax = c.b;
        }
    }
    let dr = rmax.saturating_sub(rmin);
    let dg = gmax.saturating_sub(gmin);
    let db = bmax.saturating_sub(bmin);
    (dr, dg, db)
}

fn median_cut(colors: Vec<RgbColor>, num_colors: usize) -> Vec<Vec<RgbColor>> {
    if num_colors <= 1 || colors.is_empty() {
        return vec![colors];
    }
    let mut buckets: Vec<Vec<RgbColor>> = vec![colors];
    while buckets.len() < num_colors {
        /* find the bucket with the largest range */
        let idx = buckets
            .iter()
            .enumerate()
            .max_by_key(|(_, b)| {
                let (dr, dg, db) = channel_range(b);
                dr.max(dg).max(db)
            })
            .map(|(i, _)| i);
        let Some(idx) = idx else { break };
        let bucket = buckets.remove(idx);
        let (dr, dg, db) = channel_range(&bucket);
        let mut sorted = bucket;
        if dr >= dg && dr >= db {
            sorted.sort_by_key(|c| c.r);
        } else if dg >= dr && dg >= db {
            sorted.sort_by_key(|c| c.g);
        } else {
            sorted.sort_by_key(|c| c.b);
        }
        let mid = sorted.len() / 2;
        let (lo, hi) = sorted.split_at(mid);
        buckets.push(lo.to_vec());
        buckets.push(hi.to_vec());
    }
    buckets.retain(|b| !b.is_empty());
    buckets
}

/// Quantize a slice of pixel data (r, g, b triples) to a palette.
pub fn quantize_pixels(pixels: &[(u8, u8, u8)], num_colors: usize) -> Vec<RgbColor> {
    let colors: Vec<RgbColor> = pixels
        .iter()
        .map(|&(r, g, b)| RgbColor::new(r, g, b))
        .collect();
    let q = new_color_quantizer(colors);
    q.quantize(num_colors)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_palette() -> Vec<RgbColor> {
        vec![
            RgbColor::new(255, 0, 0),
            RgbColor::new(0, 255, 0),
            RgbColor::new(0, 0, 255),
            RgbColor::new(128, 128, 128),
            RgbColor::new(255, 255, 0),
            RgbColor::new(0, 255, 255),
            RgbColor::new(255, 0, 255),
            RgbColor::new(0, 0, 0),
        ]
    }

    #[test]
    fn test_quantize_reduces_colors() {
        /* quantize to 4 returns at most 4 entries */
        let q = new_color_quantizer(sample_palette());
        let palette = q.quantize(4);
        assert!(palette.len() <= 4 && !palette.is_empty());
    }

    #[test]
    fn test_quantize_empty() {
        /* empty input returns empty palette */
        let q = new_color_quantizer(vec![]);
        assert!(q.quantize(4).is_empty());
    }

    #[test]
    fn test_color_dist_sq_same() {
        /* same color has zero distance */
        let c = RgbColor::new(100, 150, 200);
        assert_eq!(color_dist_sq(c, c), 0);
    }

    #[test]
    fn test_color_dist_sq_known() {
        /* distance (0,0,0) to (1,0,0) = 1 */
        let a = RgbColor::new(0, 0, 0);
        let b = RgbColor::new(1, 0, 0);
        assert_eq!(color_dist_sq(a, b), 1);
    }

    #[test]
    fn test_closest() {
        /* closest finds nearest palette color */
        let palette = vec![RgbColor::new(0, 0, 0), RgbColor::new(255, 0, 0)];
        let q = RgbColor::new(200, 0, 0);
        let closest = ColorQuantizer::closest(&palette, q).expect("should succeed");
        assert_eq!(closest.r, 255);
    }

    #[test]
    fn test_color_count() {
        /* color_count returns number of input colors */
        let q = new_color_quantizer(sample_palette());
        assert_eq!(q.color_count(), 8);
    }

    #[test]
    fn test_quantize_pixels() {
        /* quantize_pixels returns a palette */
        let pixels = vec![(255u8, 0, 0), (0, 255, 0), (0, 0, 255)];
        let pal = quantize_pixels(&pixels, 3);
        assert!(!pal.is_empty());
    }

    #[test]
    fn test_quantize_single_color() {
        /* single distinct color quantized to 1 returns it */
        let q = new_color_quantizer(vec![RgbColor::new(42, 42, 42); 5]);
        let palette = q.quantize(1);
        assert_eq!(palette.len(), 1);
        assert_eq!(palette[0].r, 42);
    }

    #[test]
    fn test_quantize_more_than_input() {
        /* requesting more colors than input doesn't panic */
        let q = new_color_quantizer(vec![RgbColor::new(1, 2, 3), RgbColor::new(4, 5, 6)]);
        let palette = q.quantize(100);
        assert!(!palette.is_empty());
    }
}
