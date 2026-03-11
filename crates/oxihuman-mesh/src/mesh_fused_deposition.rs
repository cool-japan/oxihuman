// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! FDM (Fused Deposition Modeling) layer-by-layer mesh slice visualization.

/// A single FDM print layer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FdmLayer {
    /// Z height of this layer.
    pub z: f32,
    /// Layer thickness.
    pub thickness: f32,
    /// Extrusion paths as sequences of 2D points (XY).
    pub paths: Vec<Vec<[f32; 2]>>,
}

/// An FDM print job (collection of layers).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FdmJob {
    pub layers: Vec<FdmLayer>,
    pub layer_height: f32,
}

/// Slice a bounding box into uniform layers.
#[allow(dead_code)]
pub fn slice_bbox_into_layers(min_z: f32, max_z: f32, layer_height: f32) -> FdmJob {
    let lh = layer_height.max(1e-4);
    let n = ((max_z - min_z) / lh).ceil() as usize;
    let layers = (0..n)
        .map(|i| {
            let z = min_z + i as f32 * lh + lh * 0.5;
            FdmLayer {
                z,
                thickness: lh,
                paths: Vec::new(),
            }
        })
        .collect();
    FdmJob {
        layers,
        layer_height: lh,
    }
}

/// Add a rectangular extrusion path to a layer.
#[allow(dead_code)]
pub fn add_rect_path(layer: &mut FdmLayer, min: [f32; 2], max: [f32; 2]) {
    let path = vec![
        [min[0], min[1]],
        [max[0], min[1]],
        [max[0], max[1]],
        [min[0], max[1]],
        [min[0], min[1]],
    ];
    layer.paths.push(path);
}

/// Add a zigzag infill path to a layer.
#[allow(dead_code)]
pub fn add_zigzag_infill(layer: &mut FdmLayer, min: [f32; 2], max: [f32; 2], spacing: f32) {
    let sp = spacing.max(1e-4);
    let mut path = Vec::new();
    let mut x = min[0];
    let mut going_up = true;
    while x <= max[0] + 1e-5 {
        if going_up {
            path.push([x, min[1]]);
            path.push([x, max[1]]);
        } else {
            path.push([x, max[1]]);
            path.push([x, min[1]]);
        }
        x += sp;
        going_up = !going_up;
    }
    if !path.is_empty() {
        layer.paths.push(path);
    }
}

/// Total path length of a layer (sum of segment lengths).
#[allow(dead_code)]
pub fn layer_path_length(layer: &FdmLayer) -> f32 {
    layer
        .paths
        .iter()
        .flat_map(|path| {
            path.windows(2).map(|w| {
                let dx = w[1][0] - w[0][0];
                let dy = w[1][1] - w[0][1];
                (dx * dx + dy * dy).sqrt()
            })
        })
        .sum()
}

/// Total number of path points across all layers.
#[allow(dead_code)]
pub fn total_path_points(job: &FdmJob) -> usize {
    job.layers
        .iter()
        .flat_map(|l| l.paths.iter())
        .map(|p| p.len())
        .sum()
}

/// Layer count.
#[allow(dead_code)]
pub fn layer_count(job: &FdmJob) -> usize {
    job.layers.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_count_correct() {
        let job = slice_bbox_into_layers(0.0, 5.0, 0.2);
        assert_eq!(layer_count(&job), 25);
    }

    #[test]
    fn layer_z_increases() {
        let job = slice_bbox_into_layers(0.0, 2.0, 0.2);
        let zs: Vec<f32> = job.layers.iter().map(|l| l.z).collect();
        for w in zs.windows(2) {
            assert!(w[1] > w[0]);
        }
    }

    #[test]
    fn empty_job_no_layers() {
        let job = slice_bbox_into_layers(0.0, 0.0, 0.2);
        assert_eq!(layer_count(&job), 0);
    }

    #[test]
    fn rect_path_adds_5_points() {
        let job = slice_bbox_into_layers(0.0, 1.0, 0.5);
        let mut job = job;
        add_rect_path(&mut job.layers[0], [0.0, 0.0], [1.0, 1.0]);
        assert_eq!(job.layers[0].paths[0].len(), 5);
    }

    #[test]
    fn rect_path_length_positive() {
        let job = slice_bbox_into_layers(0.0, 1.0, 0.5);
        let mut job = job;
        add_rect_path(&mut job.layers[0], [0.0, 0.0], [1.0, 1.0]);
        assert!(layer_path_length(&job.layers[0]) > 0.0);
    }

    #[test]
    fn zigzag_adds_path() {
        let job = slice_bbox_into_layers(0.0, 1.0, 0.5);
        let mut job = job;
        add_zigzag_infill(&mut job.layers[0], [0.0, 0.0], [2.0, 2.0], 0.5);
        assert!(!job.layers[0].paths.is_empty());
    }

    #[test]
    fn total_path_points_zero_initially() {
        let job = slice_bbox_into_layers(0.0, 2.0, 0.5);
        assert_eq!(total_path_points(&job), 0);
    }

    #[test]
    fn layer_height_stored() {
        let job = slice_bbox_into_layers(0.0, 1.0, 0.25);
        assert!((job.layer_height - 0.25).abs() < 1e-5);
    }

    #[test]
    fn layer_thickness_stored() {
        let job = slice_bbox_into_layers(0.0, 1.0, 0.2);
        for l in &job.layers {
            assert!((l.thickness - 0.2).abs() < 1e-5);
        }
    }

    #[test]
    fn zigzag_path_length_positive() {
        let job = slice_bbox_into_layers(0.0, 1.0, 0.5);
        let mut job = job;
        add_zigzag_infill(&mut job.layers[0], [0.0, 0.0], [2.0, 2.0], 0.5);
        assert!(layer_path_length(&job.layers[0]) > 0.0);
    }
}
