// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! CityGML urban model export stub.

/// CityGML LOD level (0–4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CityLod(pub u8);

/// A CityGML building feature stub.
#[derive(Debug, Clone)]
pub struct CityBuilding {
    pub id: String,
    pub lod: CityLod,
    pub footprint: Vec<[f64; 2]>,
    pub height: f64,
    pub attributes: std::collections::HashMap<String, String>,
}

/// CityGML export container.
#[derive(Debug, Clone, Default)]
pub struct CityGmlExport {
    pub srs_name: String,
    pub buildings: Vec<CityBuilding>,
}

/// Create a new CityGML export.
pub fn new_citygml_export(srs_name: &str) -> CityGmlExport {
    CityGmlExport {
        srs_name: srs_name.to_string(),
        buildings: Vec::new(),
    }
}

/// Add a building to the export.
pub fn add_city_building(
    export: &mut CityGmlExport,
    id: &str,
    lod: u8,
    footprint: Vec<[f64; 2]>,
    height: f64,
) {
    export.buildings.push(CityBuilding {
        id: id.to_string(),
        lod: CityLod(lod),
        footprint,
        height,
        attributes: Default::default(),
    });
}

/// Return building count.
pub fn citygml_building_count(export: &CityGmlExport) -> usize {
    export.buildings.len()
}

/// Return the maximum LOD across all buildings.
pub fn citygml_max_lod(export: &CityGmlExport) -> Option<u8> {
    export.buildings.iter().map(|b| b.lod.0).max()
}

/// Render a stub CityGML XML header.
pub fn citygml_xml_header(export: &CityGmlExport) -> String {
    format!(
        "<CityModel xmlns:gml=\"http://www.opengis.net/gml\" srsName=\"{}\">",
        export.srs_name
    )
}

/// Validate that all buildings have non-empty footprints.
pub fn validate_citygml(export: &CityGmlExport) -> bool {
    export.buildings.iter().all(|b| b.footprint.len() >= 3)
}

/// Compute total building volume (footprint area × height, stub approximation).
pub fn citygml_total_volume(export: &CityGmlExport) -> f64 {
    export
        .buildings
        .iter()
        .map(|b| {
            /* shoelace formula for footprint area */
            let n = b.footprint.len();
            if n < 3 {
                return 0.0;
            }
            let area: f64 = (0..n)
                .map(|i| {
                    let j = (i + 1) % n;
                    b.footprint[i][0] * b.footprint[j][1] - b.footprint[j][0] * b.footprint[i][1]
                })
                .sum::<f64>()
                .abs()
                * 0.5;
            area * b.height
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sq_footprint() -> Vec<[f64; 2]> {
        vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
    }

    #[test]
    fn test_new_export_empty() {
        let exp = new_citygml_export("EPSG:4326");
        assert_eq!(citygml_building_count(&exp), 0);
    }

    #[test]
    fn test_add_building() {
        let mut exp = new_citygml_export("EPSG:4326");
        add_city_building(&mut exp, "B1", 2, sq_footprint(), 10.0);
        assert_eq!(citygml_building_count(&exp), 1);
    }

    #[test]
    fn test_max_lod() {
        let mut exp = new_citygml_export("EPSG:4326");
        add_city_building(&mut exp, "B1", 1, sq_footprint(), 5.0);
        add_city_building(&mut exp, "B2", 3, sq_footprint(), 8.0);
        assert_eq!(citygml_max_lod(&exp), Some(3));
    }

    #[test]
    fn test_xml_header_contains_srs() {
        let exp = new_citygml_export("EPSG:4326");
        assert!(citygml_xml_header(&exp).contains("EPSG:4326"));
    }

    #[test]
    fn test_validate_valid() {
        let mut exp = new_citygml_export("EPSG:4326");
        add_city_building(&mut exp, "B1", 2, sq_footprint(), 5.0);
        assert!(validate_citygml(&exp));
    }

    #[test]
    fn test_validate_too_few_points() {
        let mut exp = new_citygml_export("EPSG:4326");
        add_city_building(&mut exp, "B1", 2, vec![[0.0, 0.0], [1.0, 0.0]], 5.0);
        assert!(!validate_citygml(&exp));
    }

    #[test]
    fn test_total_volume() {
        let mut exp = new_citygml_export("EPSG:4326");
        /* 10×10 footprint, height 5 → volume = 500 */
        add_city_building(&mut exp, "B1", 2, sq_footprint(), 5.0);
        assert!((citygml_total_volume(&exp) - 500.0).abs() < 1e-6);
    }

    #[test]
    fn test_max_lod_empty() {
        let exp = new_citygml_export("EPSG:4326");
        assert!(citygml_max_lod(&exp).is_none());
    }

    #[test]
    fn test_srs_name_stored() {
        let exp = new_citygml_export("EPSG:25832");
        assert_eq!(exp.srs_name, "EPSG:25832");
    }
}
