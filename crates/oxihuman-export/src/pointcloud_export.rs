// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Point cloud export in XYZ, PCD, and PLY formats.

// ── Enums ─────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum PointCloudFormat {
    Xyz,
    Pcd,
    Ply,
}

// ── Structs ───────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PointCloudConfig {
    pub format: PointCloudFormat,
    pub include_normals: bool,
    pub include_colors: bool,
    pub precision: u8,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CloudPoint {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [u8; 4],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PointCloudData {
    pub points: Vec<CloudPoint>,
    pub config: PointCloudConfig,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PointCloudExportResult {
    pub data_string: String,
    pub point_count: usize,
    pub byte_size: usize,
}

// ── Functions ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn default_pointcloud_config() -> PointCloudConfig {
    PointCloudConfig {
        format: PointCloudFormat::Xyz,
        include_normals: false,
        include_colors: false,
        precision: 6,
    }
}

#[allow(dead_code)]
pub fn new_point_cloud_data(cfg: PointCloudConfig) -> PointCloudData {
    PointCloudData {
        points: Vec::new(),
        config: cfg,
    }
}

#[allow(dead_code)]
pub fn add_point(cloud: &mut PointCloudData, pt: CloudPoint) {
    cloud.points.push(pt);
}

#[allow(dead_code)]
pub fn new_cloud_point(pos: [f32; 3]) -> CloudPoint {
    CloudPoint {
        position: pos,
        normal: [0.0, 1.0, 0.0],
        color: [255, 255, 255, 255],
    }
}

#[allow(dead_code)]
pub fn export_point_cloud(cloud: &PointCloudData) -> PointCloudExportResult {
    let data_string = match cloud.config.format {
        PointCloudFormat::Xyz => export_to_xyz(cloud),
        PointCloudFormat::Pcd => {
            export_to_pcd_header(cloud)
        }
        PointCloudFormat::Ply => {
            let n = cloud.points.len();
            format!("ply\nformat ascii 1.0\nelement vertex {n}\nend_header\n")
        }
    };
    let byte_size = data_string.len();
    let point_count = point_count_cloud(cloud);
    PointCloudExportResult {
        data_string,
        point_count,
        byte_size,
    }
}

#[allow(dead_code)]
pub fn export_to_xyz(cloud: &PointCloudData) -> String {
    let prec = cloud.config.precision as usize;
    let mut out = String::new();
    for pt in &cloud.points {
        let [x, y, z] = pt.position;
        out.push_str(&format!("{x:.prec$} {y:.prec$} {z:.prec$}"));
        if cloud.config.include_normals {
            let [nx, ny, nz] = pt.normal;
            out.push_str(&format!(" {nx:.prec$} {ny:.prec$} {nz:.prec$}"));
        }
        if cloud.config.include_colors {
            let [r, g, b, a] = pt.color;
            out.push_str(&format!(" {r} {g} {b} {a}"));
        }
        out.push('\n');
    }
    out
}

#[allow(dead_code)]
pub fn export_to_pcd_header(cloud: &PointCloudData) -> String {
    let n = cloud.points.len();
    let has_normals = cloud.config.include_normals;
    let fields = if has_normals {
        "FIELDS x y z normal_x normal_y normal_z"
    } else {
        "FIELDS x y z"
    };
    let sizes = if has_normals {
        "SIZE 4 4 4 4 4 4"
    } else {
        "SIZE 4 4 4"
    };
    let types = if has_normals {
        "TYPE F F F F F F"
    } else {
        "TYPE F F F"
    };
    let counts = if has_normals {
        "COUNT 1 1 1 1 1 1"
    } else {
        "COUNT 1 1 1"
    };
    format!(
        "# .PCD v0.7\nVERSION 0.7\n{fields}\n{sizes}\n{types}\n{counts}\nWIDTH {n}\nHEIGHT 1\nVIEWPOINT 0 0 0 1 0 0 0\nPOINTS {n}\nDATA ascii\n"
    )
}

#[allow(dead_code)]
pub fn point_count_cloud(cloud: &PointCloudData) -> usize {
    cloud.points.len()
}

#[allow(dead_code)]
pub fn cloud_bounding_box(cloud: &PointCloudData) -> [[f32; 3]; 2] {
    if cloud.points.is_empty() {
        return [[0.0; 3], [0.0; 3]];
    }
    let first = cloud.points[0].position;
    let mut mn = first;
    let mut mx = first;
    for pt in &cloud.points {
        let p = pt.position;
        for i in 0..3 {
            if p[i] < mn[i] {
                mn[i] = p[i];
            }
            if p[i] > mx[i] {
                mx[i] = p[i];
            }
        }
    }
    [mn, mx]
}

#[allow(dead_code)]
pub fn pointcloud_to_json(cloud: &PointCloudData) -> String {
    let n = cloud.points.len();
    let [mn, mx] = cloud_bounding_box(cloud);
    format!(
        r#"{{"point_count":{n},"format":"{}","bounds":{{"min":[{},{},{}],"max":[{},{},{}]}}}}"#,
        format_name_cloud(cloud),
        mn[0],
        mn[1],
        mn[2],
        mx[0],
        mx[1],
        mx[2]
    )
}

#[allow(dead_code)]
pub fn format_name_cloud(cloud: &PointCloudData) -> &'static str {
    match cloud.config.format {
        PointCloudFormat::Xyz => "xyz",
        PointCloudFormat::Pcd => "pcd",
        PointCloudFormat::Ply => "ply",
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cloud() -> PointCloudData {
        let cfg = default_pointcloud_config();
        let mut cloud = new_point_cloud_data(cfg);
        add_point(&mut cloud, new_cloud_point([0.0, 0.0, 0.0]));
        add_point(&mut cloud, new_cloud_point([1.0, 0.0, 0.0]));
        add_point(&mut cloud, new_cloud_point([0.0, 1.0, 0.0]));
        cloud
    }

    #[test]
    fn test_default_config() {
        let cfg = default_pointcloud_config();
        assert_eq!(cfg.format, PointCloudFormat::Xyz);
        assert!(!cfg.include_normals);
        assert!(!cfg.include_colors);
        assert_eq!(cfg.precision, 6);
    }

    #[test]
    fn test_new_cloud_point() {
        let pt = new_cloud_point([1.0, 2.0, 3.0]);
        assert_eq!(pt.position, [1.0, 2.0, 3.0]);
        assert_eq!(pt.color, [255, 255, 255, 255]);
    }

    #[test]
    fn test_add_point_count() {
        let cloud = make_cloud();
        assert_eq!(point_count_cloud(&cloud), 3);
    }

    #[test]
    fn test_export_xyz_line_count() {
        let cloud = make_cloud();
        let xyz = export_to_xyz(&cloud);
        assert_eq!(xyz.lines().count(), 3);
    }

    #[test]
    fn test_export_pcd_header_contains_fields() {
        let cloud = make_cloud();
        let hdr = export_to_pcd_header(&cloud);
        assert!(hdr.contains("FIELDS x y z"));
        assert!(hdr.contains("POINTS 3"));
    }

    #[test]
    fn test_bounding_box() {
        let cloud = make_cloud();
        let [mn, mx] = cloud_bounding_box(&cloud);
        assert!((mn[0] - 0.0).abs() < 1e-6);
        assert!((mx[0] - 1.0).abs() < 1e-6);
        assert!((mx[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_bounding_box_empty() {
        let cfg = default_pointcloud_config();
        let cloud = new_point_cloud_data(cfg);
        let [mn, mx] = cloud_bounding_box(&cloud);
        assert_eq!(mn, [0.0, 0.0, 0.0]);
        assert_eq!(mx, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_format_name_xyz() {
        let cloud = make_cloud();
        assert_eq!(format_name_cloud(&cloud), "xyz");
    }

    #[test]
    fn test_format_name_pcd() {
        let cfg = PointCloudConfig {
            format: PointCloudFormat::Pcd,
            include_normals: false,
            include_colors: false,
            precision: 4,
        };
        let cloud = new_point_cloud_data(cfg);
        assert_eq!(format_name_cloud(&cloud), "pcd");
    }

    #[test]
    fn test_pointcloud_to_json() {
        let cloud = make_cloud();
        let json = pointcloud_to_json(&cloud);
        assert!(json.contains("point_count"));
        assert!(json.contains("xyz"));
        assert!(json.contains("bounds"));
    }

    #[test]
    fn test_export_point_cloud_xyz() {
        let cloud = make_cloud();
        let result = export_point_cloud(&cloud);
        assert_eq!(result.point_count, 3);
        assert!(!result.data_string.is_empty());
        assert!(result.byte_size > 0);
    }

    #[test]
    fn test_export_to_xyz_with_normals() {
        let cfg = PointCloudConfig {
            format: PointCloudFormat::Xyz,
            include_normals: true,
            include_colors: false,
            precision: 2,
        };
        let mut cloud = new_point_cloud_data(cfg);
        let mut pt = new_cloud_point([1.0, 0.0, 0.0]);
        pt.normal = [0.0, 1.0, 0.0];
        add_point(&mut cloud, pt);
        let xyz = export_to_xyz(&cloud);
        // Should have 6 numbers per line (x y z nx ny nz)
        let parts: Vec<&str> = xyz.lines().next().expect("should succeed").split_whitespace().collect();
        assert_eq!(parts.len(), 6);
    }
}
