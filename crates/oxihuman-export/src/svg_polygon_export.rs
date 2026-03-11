// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! SVG polygon and polyline export.

/// An SVG polygon element.
#[allow(dead_code)]
pub struct SvgPolygon {
    pub points: Vec<[f32; 2]>,
    pub stroke: String,
    pub stroke_width: f32,
    pub fill: String,
    pub opacity: f32,
}

/// An SVG polyline element (open path).
#[allow(dead_code)]
pub struct SvgPolyline {
    pub points: Vec<[f32; 2]>,
    pub stroke: String,
    pub stroke_width: f32,
}

/// Create a new SVG polygon.
#[allow(dead_code)]
pub fn new_polygon(
    points: Vec<[f32; 2]>,
    stroke: &str,
    stroke_width: f32,
    fill: &str,
) -> SvgPolygon {
    SvgPolygon {
        points,
        stroke: stroke.to_string(),
        stroke_width,
        fill: fill.to_string(),
        opacity: 1.0,
    }
}

/// Create a new SVG polyline.
#[allow(dead_code)]
pub fn new_polyline(points: Vec<[f32; 2]>, stroke: &str, stroke_width: f32) -> SvgPolyline {
    SvgPolyline {
        points,
        stroke: stroke.to_string(),
        stroke_width,
    }
}

/// Serialize polygon points to the SVG `points` attribute format.
#[allow(dead_code)]
pub fn points_to_attr(points: &[[f32; 2]]) -> String {
    points
        .iter()
        .map(|&[x, y]| format!("{:.4},{:.4}", x, y))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Serialize a polygon to an SVG `<polygon>` tag.
#[allow(dead_code)]
pub fn polygon_to_tag(poly: &SvgPolygon) -> String {
    format!(
        "<polygon points=\"{}\" stroke=\"{}\" stroke-width=\"{}\" fill=\"{}\" opacity=\"{}\"/>",
        points_to_attr(&poly.points),
        poly.stroke,
        poly.stroke_width,
        poly.fill,
        poly.opacity
    )
}

/// Serialize a polyline to an SVG `<polyline>` tag.
#[allow(dead_code)]
pub fn polyline_to_tag(pl: &SvgPolyline) -> String {
    format!(
        "<polyline points=\"{}\" stroke=\"{}\" stroke-width=\"{}\" fill=\"none\"/>",
        points_to_attr(&pl.points),
        pl.stroke,
        pl.stroke_width
    )
}

/// A collection of SVG polygon and polyline elements.
#[allow(dead_code)]
pub struct SvgPolygonDoc {
    pub width: f32,
    pub height: f32,
    pub polygons: Vec<SvgPolygon>,
    pub polylines: Vec<SvgPolyline>,
}

/// Create a new SVG polygon document.
#[allow(dead_code)]
pub fn new_polygon_doc(width: f32, height: f32) -> SvgPolygonDoc {
    SvgPolygonDoc {
        width,
        height,
        polygons: Vec::new(),
        polylines: Vec::new(),
    }
}

/// Add a polygon.
#[allow(dead_code)]
pub fn add_polygon(doc: &mut SvgPolygonDoc, poly: SvgPolygon) {
    doc.polygons.push(poly);
}

/// Add a polyline.
#[allow(dead_code)]
pub fn add_polyline(doc: &mut SvgPolygonDoc, pl: SvgPolyline) {
    doc.polylines.push(pl);
}

/// Export document to an SVG string.
#[allow(dead_code)]
pub fn export_polygon_svg(doc: &SvgPolygonDoc) -> String {
    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\">",
        doc.width, doc.height
    );
    for poly in &doc.polygons {
        out.push_str(&polygon_to_tag(poly));
    }
    for pl in &doc.polylines {
        out.push_str(&polyline_to_tag(pl));
    }
    out.push_str("</svg>");
    out
}

/// Compute axis-aligned bounding box of a polygon.
#[allow(dead_code)]
pub fn polygon_aabb(poly: &SvgPolygon) -> ([f32; 2], [f32; 2]) {
    if poly.points.is_empty() {
        return ([0.0; 2], [0.0; 2]);
    }
    let mut mn = poly.points[0];
    let mut mx = poly.points[0];
    for &p in &poly.points {
        mn[0] = mn[0].min(p[0]);
        mn[1] = mn[1].min(p[1]);
        mx[0] = mx[0].max(p[0]);
        mx[1] = mx[1].max(p[1]);
    }
    (mn, mx)
}

/// Polygon vertex count.
#[allow(dead_code)]
pub fn polygon_vertex_count(poly: &SvgPolygon) -> usize {
    poly.points.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tri_poly() -> SvgPolygon {
        new_polygon(
            vec![[0.0, 0.0], [100.0, 0.0], [50.0, 100.0]],
            "black",
            1.0,
            "blue",
        )
    }

    #[test]
    fn polygon_tag_has_points() {
        let poly = tri_poly();
        let tag = polygon_to_tag(&poly);
        assert!(tag.contains("polygon"));
        assert!(tag.contains("points"));
    }

    #[test]
    fn polygon_vertex_count_correct() {
        let poly = tri_poly();
        assert_eq!(polygon_vertex_count(&poly), 3);
    }

    #[test]
    fn polyline_tag_has_points() {
        let pl = new_polyline(vec![[0.0, 0.0], [50.0, 50.0]], "red", 2.0);
        let tag = polyline_to_tag(&pl);
        assert!(tag.contains("polyline"));
    }

    #[test]
    fn polygon_aabb_correct() {
        let poly = tri_poly();
        let (mn, mx) = polygon_aabb(&poly);
        assert!((mn[0] - 0.0).abs() < 1e-5);
        assert!((mx[0] - 100.0).abs() < 1e-5);
    }

    #[test]
    fn doc_export_has_svg_tag() {
        let doc = new_polygon_doc(800.0, 600.0);
        let svg = export_polygon_svg(&doc);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn add_polygon_to_doc() {
        let mut doc = new_polygon_doc(800.0, 600.0);
        add_polygon(&mut doc, tri_poly());
        assert_eq!(doc.polygons.len(), 1);
    }

    #[test]
    fn add_polyline_to_doc() {
        let mut doc = new_polygon_doc(800.0, 600.0);
        let pl = new_polyline(vec![[0.0, 0.0], [100.0, 100.0]], "green", 1.0);
        add_polyline(&mut doc, pl);
        assert_eq!(doc.polylines.len(), 1);
    }

    #[test]
    fn points_to_attr_format() {
        let attr = points_to_attr(&[[1.0, 2.0], [3.0, 4.0]]);
        assert!(attr.contains(','));
        assert!(attr.contains(' '));
    }

    #[test]
    fn polygon_fill_in_tag() {
        let poly = tri_poly();
        let tag = polygon_to_tag(&poly);
        assert!(tag.contains("blue"));
    }

    #[test]
    fn empty_polygon_aabb() {
        let empty = new_polygon(vec![], "black", 1.0, "none");
        let (mn, mx) = polygon_aabb(&empty);
        assert_eq!(mn, [0.0, 0.0]);
        assert_eq!(mx, [0.0, 0.0]);
    }
}
