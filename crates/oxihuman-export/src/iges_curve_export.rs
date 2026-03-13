// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! IGES curve entity export stub.

/// IGES entity type for a parametric curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IgesCurveType {
    /// Line (type 110)
    Line,
    /// Circular arc (type 100)
    CircularArc,
    /// B-Spline curve (type 126)
    BSpline,
}

/// A single IGES curve entity stub.
#[derive(Debug, Clone)]
pub struct IgesCurveEntity {
    pub curve_type: IgesCurveType,
    pub control_points: Vec<[f64; 3]>,
    pub entity_id: u32,
}

/// IGES curve export container.
#[derive(Debug, Clone, Default)]
pub struct IgesCurveExport {
    pub entities: Vec<IgesCurveEntity>,
    pub author: String,
    pub version: String,
}

/// Create a new IGES curve export with the given author string.
pub fn new_iges_curve_export(author: &str) -> IgesCurveExport {
    IgesCurveExport {
        entities: Vec::new(),
        author: author.to_string(),
        version: "5.3".to_string(),
    }
}

/// Add a curve entity to the export.
pub fn add_iges_curve(
    export: &mut IgesCurveExport,
    curve_type: IgesCurveType,
    control_points: Vec<[f64; 3]>,
) -> u32 {
    let id = export.entities.len() as u32 + 1;
    export.entities.push(IgesCurveEntity {
        curve_type,
        control_points,
        entity_id: id,
    });
    id
}

/// Return the number of curve entities.
pub fn iges_curve_count(export: &IgesCurveExport) -> usize {
    export.entities.len()
}

/// Render a stub IGES global section header.
pub fn iges_global_section(export: &IgesCurveExport) -> String {
    format!(
        "1H,,1H;,{},IGES-CURVE-STUB,{},,,6,,,,,,;",
        export.author, export.version
    )
}

/// Render a stub IGES entity line for the given entity index.
pub fn iges_entity_line(export: &IgesCurveExport, idx: usize) -> Option<String> {
    let e = export.entities.get(idx)?;
    let type_code = match e.curve_type {
        IgesCurveType::Line => 110,
        IgesCurveType::CircularArc => 100,
        IgesCurveType::BSpline => 126,
    };
    Some(format!("{:7},{:7}D{:7}", type_code, e.entity_id, idx + 1))
}

/// Validate that all entities have at least one control point.
pub fn validate_iges_curves(export: &IgesCurveExport) -> bool {
    export.entities.iter().all(|e| !e.control_points.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_export_empty() {
        let exp = new_iges_curve_export("Test");
        assert_eq!(iges_curve_count(&exp), 0);
    }

    #[test]
    fn test_add_line_entity() {
        let mut exp = new_iges_curve_export("Test");
        let id = add_iges_curve(
            &mut exp,
            IgesCurveType::Line,
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
        );
        assert_eq!(id, 1);
        assert_eq!(iges_curve_count(&exp), 1);
    }

    #[test]
    fn test_add_multiple_entities() {
        let mut exp = new_iges_curve_export("Test");
        add_iges_curve(&mut exp, IgesCurveType::Line, vec![[0.0, 0.0, 0.0]]);
        add_iges_curve(&mut exp, IgesCurveType::BSpline, vec![[1.0, 0.0, 0.0]]);
        assert_eq!(iges_curve_count(&exp), 2);
    }

    #[test]
    fn test_global_section_contains_author() {
        let exp = new_iges_curve_export("JohnDoe");
        assert!(iges_global_section(&exp).contains("JohnDoe"));
    }

    #[test]
    fn test_entity_line_line_type() {
        let mut exp = new_iges_curve_export("X");
        add_iges_curve(&mut exp, IgesCurveType::Line, vec![[0.0, 0.0, 0.0]]);
        let line = iges_entity_line(&exp, 0).expect("should succeed");
        assert!(line.contains("110"));
    }

    #[test]
    fn test_entity_line_bspline_type() {
        let mut exp = new_iges_curve_export("X");
        add_iges_curve(&mut exp, IgesCurveType::BSpline, vec![[0.0, 0.0, 0.0]]);
        let line = iges_entity_line(&exp, 0).expect("should succeed");
        assert!(line.contains("126"));
    }

    #[test]
    fn test_entity_line_out_of_bounds() {
        let exp = new_iges_curve_export("X");
        assert!(iges_entity_line(&exp, 0).is_none());
    }

    #[test]
    fn test_validate_valid() {
        let mut exp = new_iges_curve_export("X");
        add_iges_curve(&mut exp, IgesCurveType::Line, vec![[0.0, 0.0, 0.0]]);
        assert!(validate_iges_curves(&exp));
    }

    #[test]
    fn test_version_stored() {
        let exp = new_iges_curve_export("A");
        assert_eq!(exp.version, "5.3");
    }

    #[test]
    fn test_circular_arc_type_code() {
        let mut exp = new_iges_curve_export("X");
        add_iges_curve(&mut exp, IgesCurveType::CircularArc, vec![[0.0, 0.0, 0.0]]);
        let line = iges_entity_line(&exp, 0).expect("should succeed");
        assert!(line.contains("100"));
    }
}
