// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! GeoJSON format for spatial data.

#[derive(Debug, Clone)]
pub enum GeoJsonGeometry {
    Point([f64; 2]),
    LineString(Vec<[f64; 2]>),
    Polygon(Vec<Vec<[f64; 2]>>),
}

#[derive(Debug, Clone)]
pub struct GeoJsonFeature {
    pub geometry: GeoJsonGeometry,
    pub properties: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct GeoJsonCollection {
    pub features: Vec<GeoJsonFeature>,
}

pub fn new_geojson_collection() -> GeoJsonCollection {
    GeoJsonCollection {
        features: Vec::new(),
    }
}

pub fn add_point_feature(
    coll: &mut GeoJsonCollection,
    lon: f64,
    lat: f64,
    props: Vec<(&str, &str)>,
) {
    coll.features.push(GeoJsonFeature {
        geometry: GeoJsonGeometry::Point([lon, lat]),
        properties: props
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    });
}

pub fn add_linestring_feature(
    coll: &mut GeoJsonCollection,
    coords: Vec<[f64; 2]>,
    props: Vec<(&str, &str)>,
) {
    coll.features.push(GeoJsonFeature {
        geometry: GeoJsonGeometry::LineString(coords),
        properties: props
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    });
}

fn render_coords(c: &[f64; 2]) -> String {
    format!("[{}, {}]", c[0], c[1])
}

fn render_geometry(g: &GeoJsonGeometry) -> String {
    match g {
        GeoJsonGeometry::Point(c) => format!(
            "{{\"type\":\"Point\",\"coordinates\":{}}}",
            render_coords(c)
        ),
        GeoJsonGeometry::LineString(pts) => {
            let cs: Vec<String> = pts.iter().map(render_coords).collect();
            format!(
                "{{\"type\":\"LineString\",\"coordinates\":[{}]}}",
                cs.join(",")
            )
        }
        GeoJsonGeometry::Polygon(rings) => {
            let rs: Vec<String> = rings
                .iter()
                .map(|ring| {
                    let cs: Vec<String> = ring.iter().map(render_coords).collect();
                    format!("[{}]", cs.join(","))
                })
                .collect();
            format!(
                "{{\"type\":\"Polygon\",\"coordinates\":[{}]}}",
                rs.join(",")
            )
        }
    }
}

fn render_props(props: &[(String, String)]) -> String {
    let pairs: Vec<String> = props
        .iter()
        .map(|(k, v)| format!("\"{}\":\"{}\"", k, v))
        .collect();
    format!("{{{}}}", pairs.join(","))
}

pub fn render_geojson(coll: &GeoJsonCollection) -> String {
    let features: Vec<String> = coll
        .features
        .iter()
        .map(|f| {
            format!(
                "{{\"type\":\"Feature\",\"geometry\":{},\"properties\":{}}}",
                render_geometry(&f.geometry),
                render_props(&f.properties)
            )
        })
        .collect();
    format!(
        "{{\"type\":\"FeatureCollection\",\"features\":[{}]}}",
        features.join(",")
    )
}

pub fn export_geojson(coll: &GeoJsonCollection) -> Vec<u8> {
    render_geojson(coll).into_bytes()
}
pub fn geojson_feature_count(coll: &GeoJsonCollection) -> usize {
    coll.features.len()
}
pub fn validate_geojson(coll: &GeoJsonCollection) -> bool {
    !coll.features.is_empty()
}
pub fn geojson_size_bytes(coll: &GeoJsonCollection) -> usize {
    render_geojson(coll).len()
}

pub fn body_landmarks_to_geojson(landmarks: &[(&str, f64, f64)]) -> GeoJsonCollection {
    let mut coll = new_geojson_collection();
    for &(name, lon, lat) in landmarks {
        add_point_feature(&mut coll, lon, lat, vec![("name", name)]);
    }
    coll
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_geojson_collection() {
        let c = new_geojson_collection();
        assert_eq!(geojson_feature_count(&c), 0);
    }

    #[test]
    fn test_add_point_feature() {
        let mut c = new_geojson_collection();
        add_point_feature(&mut c, 139.0, 35.0, vec![("city", "Tokyo")]);
        assert_eq!(geojson_feature_count(&c), 1);
    }

    #[test]
    fn test_render_geojson_contains_type() {
        let c = new_geojson_collection();
        let s = render_geojson(&c);
        assert!(s.contains("FeatureCollection"));
    }

    #[test]
    fn test_export_geojson_nonempty() {
        let c = new_geojson_collection();
        assert!(!export_geojson(&c).is_empty());
    }

    #[test]
    fn test_validate_geojson() {
        let mut c = new_geojson_collection();
        add_point_feature(&mut c, 0.0, 0.0, vec![]);
        assert!(validate_geojson(&c));
    }

    #[test]
    fn test_add_linestring() {
        let mut c = new_geojson_collection();
        add_linestring_feature(&mut c, vec![[0.0, 0.0], [1.0, 1.0]], vec![]);
        let s = render_geojson(&c);
        assert!(s.contains("LineString"));
    }

    #[test]
    fn test_body_landmarks_to_geojson() {
        let lm = vec![("head", 0.0, 0.0), ("foot", 0.1, -0.1)];
        let c = body_landmarks_to_geojson(&lm);
        assert_eq!(geojson_feature_count(&c), 2);
    }

    #[test]
    fn test_geojson_size_bytes() {
        let c = new_geojson_collection();
        assert!(geojson_size_bytes(&c) > 0);
    }
}
