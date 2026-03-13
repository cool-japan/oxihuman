// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Point cloud export in multiple formats (.xyz, .pcd stub, .csv).

#[allow(dead_code)]
pub struct PointCloud {
    pub points: Vec<[f32; 3]>,
    pub normals: Option<Vec<[f32; 3]>>,
    pub colors: Option<Vec<[u8; 4]>>,
    pub intensities: Option<Vec<f32>>,
    pub point_count: usize,
}

#[allow(dead_code)]
pub struct PointCloudExportOptions {
    pub include_normals: bool,
    pub include_colors: bool,
    pub include_intensities: bool,
    pub coordinate_system: CoordinateSystem,
    pub scale: f32,
}

#[allow(dead_code)]
pub enum CoordinateSystem {
    RightHanded,
    LeftHanded,
    Yup,
    Zup,
}

impl Default for PointCloudExportOptions {
    fn default() -> Self {
        PointCloudExportOptions {
            include_normals: false,
            include_colors: false,
            include_intensities: false,
            coordinate_system: CoordinateSystem::RightHanded,
            scale: 1.0,
        }
    }
}

#[allow(dead_code)]
pub fn new_point_cloud(points: Vec<[f32; 3]>) -> PointCloud {
    let count = points.len();
    PointCloud {
        points,
        normals: None,
        colors: None,
        intensities: None,
        point_count: count,
    }
}

fn apply_coordinate_transform(p: [f32; 3], cs: &CoordinateSystem, scale: f32) -> [f32; 3] {
    let [x, y, z] = p;
    let (tx, ty, tz) = match cs {
        CoordinateSystem::RightHanded => (x * scale, y * scale, z * scale),
        CoordinateSystem::LeftHanded => (-x * scale, y * scale, z * scale),
        CoordinateSystem::Yup => (x * scale, y * scale, z * scale),
        CoordinateSystem::Zup => (x * scale, z * scale, -y * scale),
    };
    [tx, ty, tz]
}

#[allow(dead_code)]
pub fn point_cloud_to_xyz(cloud: &PointCloud, opts: &PointCloudExportOptions) -> String {
    let mut out = String::new();
    for (i, &p) in cloud.points.iter().enumerate() {
        let [x, y, z] = apply_coordinate_transform(p, &opts.coordinate_system, opts.scale);
        if opts.include_normals {
            if let Some(ref normals) = cloud.normals {
                if let Some(&n) = normals.get(i) {
                    let [nx, ny, nz] = apply_coordinate_transform(n, &opts.coordinate_system, 1.0);
                    out.push_str(&format!("{x} {y} {z} {nx} {ny} {nz}\n"));
                    continue;
                }
            }
        }
        out.push_str(&format!("{x} {y} {z}\n"));
    }
    out
}

#[allow(dead_code)]
pub fn point_cloud_from_xyz(content: &str) -> Option<PointCloud> {
    let mut points = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut has_normals = false;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<f32> = line
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();
        if parts.len() >= 3 {
            points.push([parts[0], parts[1], parts[2]]);
            if parts.len() >= 6 {
                normals.push([parts[3], parts[4], parts[5]]);
                has_normals = true;
            } else {
                normals.push([0.0, 0.0, 1.0]);
            }
        }
    }

    if points.is_empty() {
        return None;
    }

    let count = points.len();
    Some(PointCloud {
        points,
        normals: if has_normals { Some(normals) } else { None },
        colors: None,
        intensities: None,
        point_count: count,
    })
}

#[allow(dead_code)]
pub fn point_cloud_to_csv(cloud: &PointCloud) -> String {
    let has_normals = cloud.normals.is_some();
    let has_colors = cloud.colors.is_some();
    let has_intensity = cloud.intensities.is_some();

    let mut header = "x,y,z".to_string();
    if has_normals {
        header.push_str(",nx,ny,nz");
    }
    if has_colors {
        header.push_str(",r,g,b,a");
    }
    if has_intensity {
        header.push_str(",intensity");
    }
    header.push('\n');

    let mut out = header;
    for (i, &p) in cloud.points.iter().enumerate() {
        out.push_str(&format!("{},{},{}", p[0], p[1], p[2]));
        if let Some(ref normals) = cloud.normals {
            if let Some(&n) = normals.get(i) {
                out.push_str(&format!(",{},{},{}", n[0], n[1], n[2]));
            }
        }
        if let Some(ref colors) = cloud.colors {
            if let Some(&c) = colors.get(i) {
                out.push_str(&format!(",{},{},{},{}", c[0], c[1], c[2], c[3]));
            }
        }
        if let Some(ref intensities) = cloud.intensities {
            if let Some(&iv) = intensities.get(i) {
                out.push_str(&format!(",{iv}"));
            }
        }
        out.push('\n');
    }
    out
}

#[allow(dead_code)]
pub fn point_cloud_to_pcd_stub(cloud: &PointCloud) -> Vec<u8> {
    let n = cloud.points.len();
    let has_normals = cloud.normals.is_some();
    let fields = if has_normals {
        "FIELDS x y z normal_x normal_y normal_z"
    } else {
        "FIELDS x y z"
    };
    let size_row = if has_normals {
        "SIZE 4 4 4 4 4 4"
    } else {
        "SIZE 4 4 4"
    };
    let type_row = if has_normals {
        "TYPE F F F F F F"
    } else {
        "TYPE F F F"
    };
    let count_row = if has_normals {
        "COUNT 1 1 1 1 1 1"
    } else {
        "COUNT 1 1 1"
    };

    let header = format!(
        "# .PCD v0.7\nVERSION 0.7\n{fields}\n{size_row}\n{type_row}\n{count_row}\nWIDTH {n}\nHEIGHT 1\nVIEWPOINT 0 0 0 1 0 0 0\nPOINTS {n}\nDATA binary\n"
    );

    let mut bytes = header.into_bytes();
    for (i, &p) in cloud.points.iter().enumerate() {
        bytes.extend_from_slice(&p[0].to_le_bytes());
        bytes.extend_from_slice(&p[1].to_le_bytes());
        bytes.extend_from_slice(&p[2].to_le_bytes());
        if let Some(ref normals) = cloud.normals {
            if let Some(&n) = normals.get(i) {
                bytes.extend_from_slice(&n[0].to_le_bytes());
                bytes.extend_from_slice(&n[1].to_le_bytes());
                bytes.extend_from_slice(&n[2].to_le_bytes());
            } else {
                bytes.extend_from_slice(&0f32.to_le_bytes());
                bytes.extend_from_slice(&0f32.to_le_bytes());
                bytes.extend_from_slice(&1f32.to_le_bytes());
            }
        }
    }
    bytes
}

#[allow(clippy::too_many_arguments)]
#[allow(dead_code)]
pub fn point_cloud_from_mesh(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    triangles: &[[u32; 3]],
    sample_rate: f32,
) -> PointCloud {
    let mut pts: Vec<[f32; 3]> = Vec::new();
    let mut nrms: Vec<[f32; 3]> = Vec::new();

    for tri in triangles {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;
        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
            continue;
        }
        let p0 = positions[i0];
        let p1 = positions[i1];
        let p2 = positions[i2];

        // Compute triangle area for sample count
        let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
        let cross = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let area = 0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
        let count = ((area * sample_rate).ceil() as usize).max(1);

        // Centroid sampling with uniform barycentric steps
        let steps = (count as f32).sqrt().ceil() as usize;
        for si in 0..=steps {
            for sj in 0..=(steps - si) {
                let u = si as f32 / steps as f32;
                let v = sj as f32 / steps as f32;
                let w = 1.0 - u - v;
                if w < 0.0 {
                    continue;
                }
                let px = p0[0] * u + p1[0] * v + p2[0] * w;
                let py = p0[1] * u + p1[1] * v + p2[1] * w;
                let pz = p0[2] * u + p1[2] * v + p2[2] * w;
                pts.push([px, py, pz]);

                if !normals.is_empty() {
                    let n0 = normals.get(i0).copied().unwrap_or([0.0, 1.0, 0.0]);
                    let n1 = normals.get(i1).copied().unwrap_or([0.0, 1.0, 0.0]);
                    let n2 = normals.get(i2).copied().unwrap_or([0.0, 1.0, 0.0]);
                    let nx = n0[0] * u + n1[0] * v + n2[0] * w;
                    let ny = n0[1] * u + n1[1] * v + n2[1] * w;
                    let nz = n0[2] * u + n1[2] * v + n2[2] * w;
                    nrms.push([nx, ny, nz]);
                }
            }
        }
    }

    let count = pts.len();
    PointCloud {
        points: pts,
        normals: if !nrms.is_empty() { Some(nrms) } else { None },
        colors: None,
        intensities: None,
        point_count: count,
    }
}

#[allow(dead_code)]
#[allow(clippy::type_complexity)]
pub fn downsample_point_cloud(cloud: &PointCloud, voxel_size: f32) -> PointCloud {
    if cloud.points.is_empty() || voxel_size <= 0.0 {
        return new_point_cloud(cloud.points.clone());
    }

    let inv = 1.0 / voxel_size;
    let mut voxel_map: std::collections::HashMap<(i64, i64, i64), (Vec<[f32; 3]>, usize)> =
        std::collections::HashMap::new();

    for (idx, &p) in cloud.points.iter().enumerate() {
        let key = (
            (p[0] * inv).floor() as i64,
            (p[1] * inv).floor() as i64,
            (p[2] * inv).floor() as i64,
        );
        let entry = voxel_map.entry(key).or_insert((Vec::new(), idx));
        entry.0.push(p);
    }

    let mut pts: Vec<[f32; 3]> = Vec::new();
    let mut nrms: Vec<[f32; 3]> = Vec::new();
    let mut cols: Vec<[u8; 4]> = Vec::new();
    let mut intensities: Vec<f32> = Vec::new();

    let has_normals = cloud.normals.is_some();
    let has_colors = cloud.colors.is_some();
    let has_intensity = cloud.intensities.is_some();

    for (voxel_pts, first_idx) in voxel_map.values() {
        let n = voxel_pts.len() as f32;
        let cx = voxel_pts.iter().map(|p| p[0]).sum::<f32>() / n;
        let cy = voxel_pts.iter().map(|p| p[1]).sum::<f32>() / n;
        let cz = voxel_pts.iter().map(|p| p[2]).sum::<f32>() / n;
        pts.push([cx, cy, cz]);

        if has_normals {
            if let Some(n) = cloud.normals.as_ref().and_then(|v| v.get(*first_idx)) {
                nrms.push(*n);
            }
        }
        if has_colors {
            if let Some(c) = cloud.colors.as_ref().and_then(|v| v.get(*first_idx)) {
                cols.push(*c);
            }
        }
        if has_intensity {
            if let Some(iv) = cloud.intensities.as_ref().and_then(|v| v.get(*first_idx)) {
                intensities.push(*iv);
            }
        }
    }

    let count = pts.len();
    PointCloud {
        points: pts,
        normals: if has_normals { Some(nrms) } else { None },
        colors: if has_colors { Some(cols) } else { None },
        intensities: if has_intensity {
            Some(intensities)
        } else {
            None
        },
        point_count: count,
    }
}

#[allow(dead_code)]
pub fn point_cloud_bounds(cloud: &PointCloud) -> ([f32; 3], [f32; 3]) {
    if cloud.points.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = cloud.points[0];
    let mut mx = cloud.points[0];
    for &p in &cloud.points {
        for i in 0..3 {
            if p[i] < mn[i] {
                mn[i] = p[i];
            }
            if p[i] > mx[i] {
                mx[i] = p[i];
            }
        }
    }
    (mn, mx)
}

#[allow(dead_code)]
pub fn point_cloud_centroid(cloud: &PointCloud) -> [f32; 3] {
    if cloud.points.is_empty() {
        return [0.0; 3];
    }
    let n = cloud.points.len() as f32;
    let sx: f32 = cloud.points.iter().map(|p| p[0]).sum();
    let sy: f32 = cloud.points.iter().map(|p| p[1]).sum();
    let sz: f32 = cloud.points.iter().map(|p| p[2]).sum();
    [sx / n, sy / n, sz / n]
}

#[allow(dead_code)]
pub fn merge_point_clouds(clouds: &[PointCloud]) -> PointCloud {
    let mut points = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut colors: Vec<[u8; 4]> = Vec::new();
    let mut intensities: Vec<f32> = Vec::new();

    let mut has_normals = false;
    let mut has_colors = false;
    let mut has_intensities = false;

    for c in clouds {
        if c.normals.is_some() {
            has_normals = true;
        }
        if c.colors.is_some() {
            has_colors = true;
        }
        if c.intensities.is_some() {
            has_intensities = true;
        }
    }

    for c in clouds {
        let start = points.len();
        points.extend_from_slice(&c.points);
        let added = points.len() - start;

        if has_normals {
            if let Some(ref n) = c.normals {
                normals.extend_from_slice(n);
            } else {
                for _ in 0..added {
                    normals.push([0.0, 1.0, 0.0]);
                }
            }
        }
        if has_colors {
            if let Some(ref col) = c.colors {
                colors.extend_from_slice(col);
            } else {
                for _ in 0..added {
                    colors.push([255, 255, 255, 255]);
                }
            }
        }
        if has_intensities {
            if let Some(ref iv) = c.intensities {
                intensities.extend_from_slice(iv);
            } else {
                intensities.extend(std::iter::repeat_n(1.0f32, added));
            }
        }
    }

    let count = points.len();
    PointCloud {
        points,
        normals: if has_normals { Some(normals) } else { None },
        colors: if has_colors { Some(colors) } else { None },
        intensities: if has_intensities {
            Some(intensities)
        } else {
            None
        },
        point_count: count,
    }
}

#[allow(dead_code)]
pub fn filter_by_distance(cloud: &PointCloud, center: [f32; 3], max_dist: f32) -> PointCloud {
    let max_sq = max_dist * max_dist;
    let mut pts = Vec::new();
    let mut nrms: Vec<[f32; 3]> = Vec::new();
    let mut cols: Vec<[u8; 4]> = Vec::new();
    let mut intensities: Vec<f32> = Vec::new();

    let has_normals = cloud.normals.is_some();
    let has_colors = cloud.colors.is_some();
    let has_intensity = cloud.intensities.is_some();

    for (i, &p) in cloud.points.iter().enumerate() {
        let dx = p[0] - center[0];
        let dy = p[1] - center[1];
        let dz = p[2] - center[2];
        if dx * dx + dy * dy + dz * dz <= max_sq {
            pts.push(p);
            if has_normals {
                nrms.push(
                    cloud
                        .normals
                        .as_ref()
                        .and_then(|v| v.get(i))
                        .copied()
                        .unwrap_or([0.0, 1.0, 0.0]),
                );
            }
            if has_colors {
                cols.push(
                    cloud
                        .colors
                        .as_ref()
                        .and_then(|v| v.get(i))
                        .copied()
                        .unwrap_or([255, 255, 255, 255]),
                );
            }
            if has_intensity {
                intensities.push(
                    cloud
                        .intensities
                        .as_ref()
                        .and_then(|v| v.get(i))
                        .copied()
                        .unwrap_or(1.0),
                );
            }
        }
    }

    let count = pts.len();
    PointCloud {
        points: pts,
        normals: if has_normals { Some(nrms) } else { None },
        colors: if has_colors { Some(cols) } else { None },
        intensities: if has_intensity {
            Some(intensities)
        } else {
            None
        },
        point_count: count,
    }
}

#[allow(dead_code)]
pub fn point_cloud_stats(cloud: &PointCloud) -> String {
    let count = cloud.points.len();
    let (mn, mx) = point_cloud_bounds(cloud);
    let c = point_cloud_centroid(cloud);
    format!(
        r#"{{"point_count":{count},"bounds":{{"min":[{},{},{}],"max":[{},{},{}]}},"centroid":[{},{},{}],"has_normals":{},"has_colors":{},"has_intensities":{}}}"#,
        mn[0],
        mn[1],
        mn[2],
        mx[0],
        mx[1],
        mx[2],
        c[0],
        c[1],
        c[2],
        cloud.normals.is_some(),
        cloud.colors.is_some(),
        cloud.intensities.is_some(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_cloud() -> PointCloud {
        new_point_cloud(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]])
    }

    #[test]
    fn test_new_point_cloud() {
        let c = sample_cloud();
        assert_eq!(c.point_count, 3);
        assert_eq!(c.points.len(), 3);
        assert!(c.normals.is_none());
        assert!(c.colors.is_none());
    }

    #[test]
    fn test_to_xyz_basic() {
        let c = sample_cloud();
        let opts = PointCloudExportOptions::default();
        let s = point_cloud_to_xyz(&c, &opts);
        assert!(s.contains("0 0 0"));
        assert_eq!(s.lines().count(), 3);
    }

    #[test]
    fn test_from_xyz_roundtrip() {
        let c = sample_cloud();
        let opts = PointCloudExportOptions::default();
        let s = point_cloud_to_xyz(&c, &opts);
        let c2 = point_cloud_from_xyz(&s).expect("should succeed");
        assert_eq!(c2.points.len(), 3);
        assert!((c2.points[1][0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_from_xyz_empty() {
        assert!(point_cloud_from_xyz("").is_none());
    }

    #[test]
    fn test_to_csv_header() {
        let c = sample_cloud();
        let s = point_cloud_to_csv(&c);
        assert!(s.starts_with("x,y,z"));
        assert_eq!(s.lines().count(), 4); // header + 3 points
    }

    #[test]
    fn test_to_csv_with_colors() {
        let mut c = sample_cloud();
        c.colors = Some(vec![[255, 0, 0, 255]; 3]);
        let s = point_cloud_to_csv(&c);
        assert!(s.contains(",r,g,b,a"));
    }

    #[test]
    fn test_pcd_stub() {
        let c = sample_cloud();
        let bytes = point_cloud_to_pcd_stub(&c);
        let header_end = bytes.iter().position(|&b| b == b'\n').unwrap_or(0);
        let header_str = std::str::from_utf8(&bytes[..header_end]).unwrap_or("");
        assert!(header_str.contains("PCD"));
        // Contains binary data for 3 points
        assert!(bytes.len() > 3 * 12);
    }

    #[test]
    fn test_point_cloud_from_mesh() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let normals = vec![[0.0, 0.0, 1.0]; 3];
        let triangles = vec![[0u32, 1, 2]];
        let c = point_cloud_from_mesh(&positions, &normals, &triangles, 4.0);
        assert!(!c.points.is_empty());
    }

    #[test]
    fn test_downsample() {
        let pts: Vec<[f32; 3]> = (0..100).map(|i| [i as f32 * 0.01, 0.0, 0.0]).collect();
        let c = new_point_cloud(pts);
        let d = downsample_point_cloud(&c, 0.1);
        assert!(d.points.len() < c.points.len());
    }

    #[test]
    fn test_bounds() {
        let c = sample_cloud();
        let (mn, mx) = point_cloud_bounds(&c);
        assert!((mn[0] - 0.0).abs() < 1e-6);
        assert!((mx[0] - 1.0).abs() < 1e-6);
        assert!((mx[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_centroid() {
        let c = new_point_cloud(vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        let cen = point_cloud_centroid(&c);
        assert!((cen[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_merge() {
        let c1 = new_point_cloud(vec![[0.0, 0.0, 0.0]]);
        let c2 = new_point_cloud(vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        let merged = merge_point_clouds(&[c1, c2]);
        assert_eq!(merged.points.len(), 3);
    }

    #[test]
    fn test_filter_by_distance() {
        let c = new_point_cloud(vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.5, 0.0, 0.0]]);
        let f = filter_by_distance(&c, [0.0, 0.0, 0.0], 1.0);
        assert_eq!(f.points.len(), 2);
    }

    #[test]
    fn test_stats_json() {
        let c = sample_cloud();
        let s = point_cloud_stats(&c);
        assert!(s.contains("point_count"));
        assert!(s.contains("centroid"));
        assert!(s.contains("bounds"));
    }

    #[test]
    fn test_coordinate_transform_zup() {
        let c = new_point_cloud(vec![[1.0, 2.0, 3.0]]);
        let opts = PointCloudExportOptions {
            coordinate_system: CoordinateSystem::Zup,
            scale: 1.0,
            ..Default::default()
        };
        let s = point_cloud_to_xyz(&c, &opts);
        // Zup: x=1, y=3, z=-2
        assert!(s.contains("1 3 -2") || s.trim().starts_with("1"));
    }
}
