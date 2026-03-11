// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! GeoTIFF raster export stub.

/// GeoTIFF pixel data type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeoTiffPixelType {
    Uint8,
    Uint16,
    Float32,
}

/// GeoTIFF export stub.
#[derive(Debug, Clone)]
pub struct GeoTiffExport {
    pub width: u32,
    pub height: u32,
    pub pixel_type: GeoTiffPixelType,
    pub srs_wkt: String,
    /// Geo-transform coefficients [x_origin, pixel_width, 0, y_origin, 0, -pixel_height]
    pub geo_transform: [f64; 6],
    pub data: Vec<f32>,
}

/// Create a new GeoTIFF export.
pub fn new_geotiff_export(
    width: u32,
    height: u32,
    pixel_type: GeoTiffPixelType,
    srs_wkt: &str,
    geo_transform: [f64; 6],
) -> GeoTiffExport {
    let data = vec![0.0f32; (width * height) as usize];
    GeoTiffExport {
        width,
        height,
        pixel_type,
        srs_wkt: srs_wkt.to_string(),
        geo_transform,
        data,
    }
}

/// Set a pixel value (row, col) → value.
pub fn geotiff_set_pixel(export: &mut GeoTiffExport, row: u32, col: u32, value: f32) {
    if row < export.height && col < export.width {
        let idx = (row * export.width + col) as usize;
        export.data[idx] = value;
    }
}

/// Get a pixel value.
pub fn geotiff_get_pixel(export: &GeoTiffExport, row: u32, col: u32) -> f32 {
    if row < export.height && col < export.width {
        export.data[(row * export.width + col) as usize]
    } else {
        f32::NAN
    }
}

/// Return the pixel count.
pub fn geotiff_pixel_count(export: &GeoTiffExport) -> usize {
    export.data.len()
}

/// Convert pixel (row, col) to geographic coordinates.
pub fn geotiff_pixel_to_geo(export: &GeoTiffExport, row: u32, col: u32) -> (f64, f64) {
    let gt = &export.geo_transform;
    let x = gt[0] + col as f64 * gt[1] + row as f64 * gt[2];
    let y = gt[3] + col as f64 * gt[4] + row as f64 * gt[5];
    (x, y)
}

/// Compute min and max pixel values.
pub fn geotiff_min_max(export: &GeoTiffExport) -> (f32, f32) {
    if export.data.is_empty() {
        return (0.0, 0.0);
    }
    let mut mn = export.data[0];
    let mut mx = export.data[0];
    for &v in &export.data {
        mn = mn.min(v);
        mx = mx.max(v);
    }
    (mn, mx)
}

/// Validate that data length matches width × height.
pub fn validate_geotiff(export: &GeoTiffExport) -> bool {
    export.data.len() == (export.width * export.height) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_gt() -> [f64; 6] {
        [0.0, 1.0, 0.0, 100.0, 0.0, -1.0]
    }

    #[test]
    fn test_new_export_pixel_count() {
        let exp = new_geotiff_export(4, 3, GeoTiffPixelType::Float32, "", default_gt());
        assert_eq!(geotiff_pixel_count(&exp), 12);
    }

    #[test]
    fn test_set_get_pixel() {
        let mut exp = new_geotiff_export(4, 3, GeoTiffPixelType::Float32, "", default_gt());
        geotiff_set_pixel(&mut exp, 1, 2, 42.0);
        assert!((geotiff_get_pixel(&exp, 1, 2) - 42.0).abs() < 1e-6);
    }

    #[test]
    fn test_get_out_of_bounds() {
        let exp = new_geotiff_export(4, 3, GeoTiffPixelType::Float32, "", default_gt());
        assert!(geotiff_get_pixel(&exp, 10, 0).is_nan());
    }

    #[test]
    fn test_pixel_to_geo() {
        let exp = new_geotiff_export(
            4,
            4,
            GeoTiffPixelType::Float32,
            "",
            [0.0, 2.0, 0.0, 100.0, 0.0, -2.0],
        );
        let (x, y) = geotiff_pixel_to_geo(&exp, 0, 0);
        assert!((x).abs() < 1e-9);
        assert!((y - 100.0).abs() < 1e-9);
    }

    #[test]
    fn test_validate() {
        let exp = new_geotiff_export(5, 5, GeoTiffPixelType::Uint16, "", default_gt());
        assert!(validate_geotiff(&exp));
    }

    #[test]
    fn test_min_max_all_zero() {
        let exp = new_geotiff_export(4, 4, GeoTiffPixelType::Float32, "", default_gt());
        let (mn, mx) = geotiff_min_max(&exp);
        assert!(mn.abs() < 1e-6);
        assert!(mx.abs() < 1e-6);
    }

    #[test]
    fn test_min_max_after_set() {
        let mut exp = new_geotiff_export(4, 4, GeoTiffPixelType::Float32, "", default_gt());
        geotiff_set_pixel(&mut exp, 0, 0, -5.0);
        geotiff_set_pixel(&mut exp, 1, 1, 10.0);
        let (mn, mx) = geotiff_min_max(&exp);
        assert!((mn - (-5.0)).abs() < 1e-6);
        assert!((mx - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_srs_stored() {
        let exp = new_geotiff_export(1, 1, GeoTiffPixelType::Uint8, "EPSG:4326", default_gt());
        assert_eq!(exp.srs_wkt, "EPSG:4326");
    }

    #[test]
    fn test_dimensions_stored() {
        let exp = new_geotiff_export(7, 5, GeoTiffPixelType::Uint8, "", default_gt());
        assert_eq!(exp.width, 7);
        assert_eq!(exp.height, 5);
    }
}
