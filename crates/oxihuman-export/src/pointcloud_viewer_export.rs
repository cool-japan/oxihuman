//! Point cloud format export for external viewers (LAS stub, E57 stub).

#[allow(dead_code)]
pub struct LasHeader {
    pub version_major: u8,
    pub version_minor: u8,
    pub point_count: u32,
    pub x_scale: f64,
    pub y_scale: f64,
    pub z_scale: f64,
    pub x_offset: f64,
    pub y_offset: f64,
    pub z_offset: f64,
}

#[allow(dead_code)]
pub struct LasPoint {
    pub x: i32, // quantized
    pub y: i32,
    pub z: i32,
    pub intensity: u16,
    pub classification: u8,
}

#[allow(dead_code)]
pub struct LasFile {
    pub header: LasHeader,
    pub points: Vec<LasPoint>,
}

#[allow(dead_code)]
pub struct E57Stub {
    pub point_count: usize,
    pub has_color: bool,
    pub xml_header: String,
}

#[allow(dead_code)]
pub fn new_las_header(point_count: u32, scale: f64) -> LasHeader {
    LasHeader {
        version_major: 1,
        version_minor: 4,
        point_count,
        x_scale: scale,
        y_scale: scale,
        z_scale: scale,
        x_offset: 0.0,
        y_offset: 0.0,
        z_offset: 0.0,
    }
}

#[allow(dead_code)]
pub fn positions_to_las(positions: &[[f32; 3]], scale: f64) -> LasFile {
    let header = new_las_header(positions.len() as u32, scale);
    let points: Vec<LasPoint> = positions
        .iter()
        .map(|p| LasPoint {
            x: (p[0] as f64 / scale).round() as i32,
            y: (p[1] as f64 / scale).round() as i32,
            z: (p[2] as f64 / scale).round() as i32,
            intensity: 0,
            classification: 0,
        })
        .collect();
    LasFile { header, points }
}

#[allow(dead_code)]
pub fn las_point_to_world(point: &LasPoint, header: &LasHeader) -> [f64; 3] {
    [
        point.x as f64 * header.x_scale + header.x_offset,
        point.y as f64 * header.y_scale + header.y_offset,
        point.z as f64 * header.z_scale + header.z_offset,
    ]
}

#[allow(dead_code)]
pub fn las_file_size_estimate(las: &LasFile) -> usize {
    // LAS header ~375 bytes + 20 bytes per point (simplified)
    375 + las.points.len() * 20
}

#[allow(dead_code)]
pub fn export_las_binary_stub(las: &LasFile) -> Vec<u8> {
    // Simplified stub: file signature + point count + scale values
    let mut bytes = Vec::new();
    // "LASF" signature
    bytes.extend_from_slice(b"LASF");
    // version major / minor
    bytes.push(las.header.version_major);
    bytes.push(las.header.version_minor);
    // point count (little-endian u32)
    bytes.extend_from_slice(&las.header.point_count.to_le_bytes());
    // scale (x,y,z) as little-endian f64
    bytes.extend_from_slice(&las.header.x_scale.to_le_bytes());
    bytes.extend_from_slice(&las.header.y_scale.to_le_bytes());
    bytes.extend_from_slice(&las.header.z_scale.to_le_bytes());
    // simplified point data: x,y,z as i32
    for p in &las.points {
        bytes.extend_from_slice(&p.x.to_le_bytes());
        bytes.extend_from_slice(&p.y.to_le_bytes());
        bytes.extend_from_slice(&p.z.to_le_bytes());
        bytes.extend_from_slice(&p.intensity.to_le_bytes());
        bytes.push(p.classification);
    }
    bytes
}

#[allow(dead_code)]
pub fn new_e57_stub(point_count: usize, has_color: bool) -> E57Stub {
    let xml_header = format!(
        "<?xml version=\"1.0\"?><e57Root pointCount=\"{}\" hasColor=\"{}\"/>",
        point_count, has_color
    );
    E57Stub {
        point_count,
        has_color,
        xml_header,
    }
}

#[allow(dead_code)]
pub fn e57_xml_header(stub: &E57Stub) -> String {
    stub.xml_header.clone()
}

/// Returns (min, max) in quantized coordinates.
#[allow(dead_code)]
pub fn las_bounds(las: &LasFile) -> ([i32; 3], [i32; 3]) {
    if las.points.is_empty() {
        return ([0, 0, 0], [0, 0, 0]);
    }
    let mut min = [i32::MAX; 3];
    let mut max = [i32::MIN; 3];
    for p in &las.points {
        let coords = [p.x, p.y, p.z];
        for k in 0..3 {
            if coords[k] < min[k] {
                min[k] = coords[k];
            }
            if coords[k] > max[k] {
                max[k] = coords[k];
            }
        }
    }
    (min, max)
}

#[allow(dead_code)]
pub fn las_point_count(las: &LasFile) -> usize {
    las.points.len()
}

#[allow(dead_code)]
pub fn filter_las_by_classification(las: &LasFile, class: u8) -> Vec<&LasPoint> {
    las.points
        .iter()
        .filter(|p| p.classification == class)
        .collect()
}

#[allow(dead_code)]
pub fn las_to_positions(las: &LasFile) -> Vec<[f32; 3]> {
    las.points
        .iter()
        .map(|p| {
            let w = las_point_to_world(p, &las.header);
            [w[0] as f32, w[1] as f32, w[2] as f32]
        })
        .collect()
}

#[allow(dead_code)]
pub fn decimate_las(las: &LasFile, keep_every: usize) -> LasFile {
    let keep = keep_every.max(1);
    let points: Vec<LasPoint> = las
        .points
        .iter()
        .enumerate()
        .filter_map(|(i, p)| {
            if i % keep == 0 {
                Some(LasPoint {
                    x: p.x,
                    y: p.y,
                    z: p.z,
                    intensity: p.intensity,
                    classification: p.classification,
                })
            } else {
                None
            }
        })
        .collect();
    let new_count = points.len() as u32;
    let header = LasHeader {
        version_major: las.header.version_major,
        version_minor: las.header.version_minor,
        point_count: new_count,
        x_scale: las.header.x_scale,
        y_scale: las.header.y_scale,
        z_scale: las.header.z_scale,
        x_offset: las.header.x_offset,
        y_offset: las.header.y_offset,
        z_offset: las.header.z_offset,
    };
    LasFile { header, points }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_positions() -> Vec<[f32; 3]> {
        vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]
    }

    #[test]
    fn test_new_las_header() {
        let h = new_las_header(100, 0.001);
        assert_eq!(h.version_major, 1);
        assert_eq!(h.version_minor, 4);
        assert_eq!(h.point_count, 100);
        assert!((h.x_scale - 0.001).abs() < 1e-9);
    }

    #[test]
    fn test_positions_to_las() {
        let positions = sample_positions();
        let las = positions_to_las(&positions, 0.001);
        assert_eq!(las.points.len(), 3);
        assert_eq!(las.header.point_count, 3);
    }

    #[test]
    fn test_las_point_to_world_round_trip() {
        let positions = vec![[1.5_f32, 2.5, 3.5]];
        let scale = 0.001;
        let las = positions_to_las(&positions, scale);
        let world = las_point_to_world(&las.points[0], &las.header);
        assert!((world[0] - 1.5).abs() < 0.01);
        assert!((world[1] - 2.5).abs() < 0.01);
        assert!((world[2] - 3.5).abs() < 0.01);
    }

    #[test]
    fn test_las_point_count() {
        let las = positions_to_las(&sample_positions(), 0.001);
        assert_eq!(las_point_count(&las), 3);
    }

    #[test]
    fn test_las_bounds() {
        let las = positions_to_las(&sample_positions(), 1.0);
        let (mn, mx) = las_bounds(&las);
        assert!(mn[0] <= mx[0]);
        assert!(mn[1] <= mx[1]);
        assert!(mn[2] <= mx[2]);
    }

    #[test]
    fn test_las_bounds_empty() {
        let header = new_las_header(0, 0.001);
        let las = LasFile {
            header,
            points: vec![],
        };
        let (mn, mx) = las_bounds(&las);
        assert_eq!(mn, [0, 0, 0]);
        assert_eq!(mx, [0, 0, 0]);
    }

    #[test]
    fn test_filter_by_classification() {
        let mut las = positions_to_las(&sample_positions(), 0.001);
        las.points[0].classification = 1;
        las.points[1].classification = 2;
        las.points[2].classification = 1;
        let filtered = filter_las_by_classification(&las, 1);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_las_to_positions() {
        let positions = sample_positions();
        let las = positions_to_las(&positions, 0.001);
        let back = las_to_positions(&las);
        assert_eq!(back.len(), 3);
        assert!((back[0][0] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_decimate_las() {
        let positions: Vec<[f32; 3]> = (0..10).map(|i| [i as f32, 0.0, 0.0]).collect();
        let las = positions_to_las(&positions, 0.001);
        let dec = decimate_las(&las, 2);
        assert_eq!(dec.points.len(), 5);
    }

    #[test]
    fn test_decimate_las_keep_all() {
        let positions = sample_positions();
        let las = positions_to_las(&positions, 0.001);
        let dec = decimate_las(&las, 1);
        assert_eq!(dec.points.len(), 3);
    }

    #[test]
    fn test_e57_stub() {
        let stub = new_e57_stub(500, true);
        assert_eq!(stub.point_count, 500);
        assert!(stub.has_color);
        let xml = e57_xml_header(&stub);
        assert!(xml.contains("500"));
    }

    #[test]
    fn test_binary_stub_non_empty() {
        let las = positions_to_las(&sample_positions(), 0.001);
        let bytes = export_las_binary_stub(&las);
        assert!(!bytes.is_empty());
        assert!(bytes.starts_with(b"LASF"));
    }

    #[test]
    fn test_las_file_size_estimate() {
        let las = positions_to_las(&sample_positions(), 0.001);
        let size = las_file_size_estimate(&las);
        assert!(size > 375);
    }

    #[test]
    fn test_positions_to_las_empty() {
        let las = positions_to_las(&[], 0.001);
        assert_eq!(las.points.len(), 0);
        assert_eq!(las.header.point_count, 0);
    }
}
