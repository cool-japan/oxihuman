// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! GPX format for position tracks.

#[derive(Debug, Clone)]
pub struct GpxTrackPoint {
    pub lat: f64,
    pub lon: f64,
    pub ele: f64,
    pub time: String,
}

#[derive(Debug, Clone)]
pub struct GpxTrack {
    pub name: String,
    pub points: Vec<GpxTrackPoint>,
}

#[derive(Debug, Clone)]
pub struct GpxDocument {
    pub name: String,
    pub creator: String,
    pub tracks: Vec<GpxTrack>,
    pub waypoints: Vec<GpxTrackPoint>,
}

pub fn new_gpx_document(name: &str) -> GpxDocument {
    GpxDocument {
        name: name.to_string(),
        creator: "OxiHuman".to_string(),
        tracks: Vec::new(),
        waypoints: Vec::new(),
    }
}

pub fn add_gpx_track(doc: &mut GpxDocument, name: &str) {
    doc.tracks.push(GpxTrack {
        name: name.to_string(),
        points: Vec::new(),
    });
}

pub fn add_gpx_track_point(doc: &mut GpxDocument, lat: f64, lon: f64, ele: f64, time: &str) {
    if let Some(track) = doc.tracks.last_mut() {
        track.points.push(GpxTrackPoint {
            lat,
            lon,
            ele,
            time: time.to_string(),
        });
    }
}

pub fn add_gpx_waypoint(doc: &mut GpxDocument, lat: f64, lon: f64, ele: f64, time: &str) {
    doc.waypoints.push(GpxTrackPoint {
        lat,
        lon,
        ele,
        time: time.to_string(),
    });
}

fn render_trkpt(pt: &GpxTrackPoint, tag: &str) -> String {
    format!(
        "  <{} lat=\"{}\" lon=\"{}\">\n    <ele>{}</ele>\n    <time>{}</time>\n  </{}>\n",
        tag, pt.lat, pt.lon, pt.ele, pt.time, tag
    )
}

pub fn render_gpx(doc: &GpxDocument) -> String {
    let mut s = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <gpx version=\"1.1\" creator=\"{}\" xmlns=\"http://www.topografix.com/GPX/1/1\">\n\
           <metadata><name>{}</name></metadata>\n",
        doc.creator, doc.name
    );
    for wpt in &doc.waypoints {
        s.push_str(&render_trkpt(wpt, "wpt"));
    }
    for trk in &doc.tracks {
        s.push_str(&format!("  <trk><name>{}</name><trkseg>\n", trk.name));
        for pt in &trk.points {
            s.push_str(&render_trkpt(pt, "trkpt"));
        }
        s.push_str("  </trkseg></trk>\n");
    }
    s.push_str("</gpx>\n");
    s
}

pub fn export_gpx(doc: &GpxDocument) -> Vec<u8> {
    render_gpx(doc).into_bytes()
}
pub fn gpx_track_count(doc: &GpxDocument) -> usize {
    doc.tracks.len()
}
pub fn gpx_point_count(doc: &GpxDocument) -> usize {
    doc.tracks.iter().map(|t| t.points.len()).sum()
}
pub fn validate_gpx(doc: &GpxDocument) -> bool {
    !doc.name.is_empty()
}
pub fn gpx_size_bytes(doc: &GpxDocument) -> usize {
    render_gpx(doc).len()
}

pub fn body_scan_track(positions_2d: &[(f64, f64)]) -> GpxDocument {
    let mut doc = new_gpx_document("Body Scan Track");
    add_gpx_track(&mut doc, "scan");
    for &(lat, lon) in positions_2d {
        add_gpx_track_point(&mut doc, lat, lon, 0.0, "");
    }
    doc
}

pub fn gpx_waypoint_count(doc: &GpxDocument) -> usize {
    doc.waypoints.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_gpx_document() {
        let d = new_gpx_document("T");
        assert_eq!(d.name, "T");
    }

    #[test]
    fn test_add_gpx_track() {
        let mut d = new_gpx_document("T");
        add_gpx_track(&mut d, "track1");
        assert_eq!(gpx_track_count(&d), 1);
    }

    #[test]
    fn test_add_gpx_track_point() {
        let mut d = new_gpx_document("T");
        add_gpx_track(&mut d, "t");
        add_gpx_track_point(&mut d, 35.0, 139.0, 100.0, "2026-01-01T00:00:00Z");
        assert_eq!(gpx_point_count(&d), 1);
    }

    #[test]
    fn test_render_gpx_contains_gpx_tag() {
        let d = new_gpx_document("T");
        let s = render_gpx(&d);
        assert!(s.contains("<gpx"));
    }

    #[test]
    fn test_export_gpx_nonempty() {
        let d = new_gpx_document("T");
        assert!(!export_gpx(&d).is_empty());
    }

    #[test]
    fn test_validate_gpx() {
        let d = new_gpx_document("V");
        assert!(validate_gpx(&d));
    }

    #[test]
    fn test_body_scan_track() {
        let pts = vec![(0.0, 0.0), (1.0, 1.0)];
        let d = body_scan_track(&pts);
        assert_eq!(gpx_point_count(&d), 2);
    }

    #[test]
    fn test_gpx_waypoint_count() {
        let mut d = new_gpx_document("T");
        add_gpx_waypoint(&mut d, 35.0, 139.0, 0.0, "");
        assert_eq!(gpx_waypoint_count(&d), 1);
    }
}
