// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Body measurement calculator using cross-section slicing.
//!
//! Implements 24+ anthropometric tape measurements by slicing a triangle mesh
//! with horizontal or arbitrary cutting planes, then computing perimeters,
//! widths, depths, volumes, and surface areas.

use std::collections::HashMap;

/// A single body measurement result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Measurement {
    /// Human-readable measurement name.
    pub name: String,
    /// Measured value in centimetres (or cm² / cm³ for area/volume).
    pub value_cm: f64,
    /// What kind of measurement this is.
    pub kind: MeasurementType,
    /// Confidence in [0, 1]. 1.0 means exact geometric computation.
    pub confidence: f64,
}

/// Classification of measurement types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MeasurementType {
    Circumference,
    Length,
    Width,
    Depth,
    Height,
    Volume,
    SurfaceArea,
    Index,
}

/// Standard anthropometric measurement set (24 measurements).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnthropometricSet {
    /// Total height (stature) in cm.
    pub stature: f64,
    /// Neck circumference in cm.
    pub neck_circumference: f64,
    /// Chest circumference in cm.
    pub chest_circumference: f64,
    /// Under-bust circumference in cm.
    pub underbust_circumference: f64,
    /// Waist circumference in cm.
    pub waist_circumference: f64,
    /// Hip circumference in cm.
    pub hip_circumference: f64,
    /// Upper arm circumference in cm.
    pub upper_arm_circumference: f64,
    /// Forearm circumference in cm.
    pub forearm_circumference: f64,
    /// Wrist circumference in cm.
    pub wrist_circumference: f64,
    /// Thigh circumference in cm.
    pub thigh_circumference: f64,
    /// Knee circumference in cm.
    pub knee_circumference: f64,
    /// Calf circumference in cm.
    pub calf_circumference: f64,
    /// Ankle circumference in cm.
    pub ankle_circumference: f64,
    /// Head circumference in cm.
    pub head_circumference: f64,
    /// Shoulder breadth (bi-acromial) in cm.
    pub shoulder_breadth: f64,
    /// Arm length in cm.
    pub arm_length: f64,
    /// Inseam length in cm.
    pub inseam: f64,
    /// Torso length in cm.
    pub torso_length: f64,
    /// Sitting height in cm.
    pub sitting_height: f64,
    /// Foot length in cm.
    pub foot_length: f64,
    /// Hand length in cm.
    pub hand_length: f64,
    /// Estimated BMI.
    pub bmi_estimate: f64,
    /// Body surface area in cm².
    pub body_surface_area: f64,
    /// Body volume in cm³.
    pub body_volume: f64,
}

impl AnthropometricSet {
    /// Convert the set into a vector of individual `Measurement` records.
    pub fn to_measurements(&self) -> Vec<Measurement> {
        vec![
            Measurement {
                name: "stature".into(),
                value_cm: self.stature,
                kind: MeasurementType::Height,
                confidence: 1.0,
            },
            Measurement {
                name: "neck_circumference".into(),
                value_cm: self.neck_circumference,
                kind: MeasurementType::Circumference,
                confidence: 1.0,
            },
            Measurement {
                name: "chest_circumference".into(),
                value_cm: self.chest_circumference,
                kind: MeasurementType::Circumference,
                confidence: 1.0,
            },
            Measurement {
                name: "underbust_circumference".into(),
                value_cm: self.underbust_circumference,
                kind: MeasurementType::Circumference,
                confidence: 1.0,
            },
            Measurement {
                name: "waist_circumference".into(),
                value_cm: self.waist_circumference,
                kind: MeasurementType::Circumference,
                confidence: 1.0,
            },
            Measurement {
                name: "hip_circumference".into(),
                value_cm: self.hip_circumference,
                kind: MeasurementType::Circumference,
                confidence: 1.0,
            },
            Measurement {
                name: "upper_arm_circumference".into(),
                value_cm: self.upper_arm_circumference,
                kind: MeasurementType::Circumference,
                confidence: 1.0,
            },
            Measurement {
                name: "forearm_circumference".into(),
                value_cm: self.forearm_circumference,
                kind: MeasurementType::Circumference,
                confidence: 1.0,
            },
            Measurement {
                name: "wrist_circumference".into(),
                value_cm: self.wrist_circumference,
                kind: MeasurementType::Circumference,
                confidence: 1.0,
            },
            Measurement {
                name: "thigh_circumference".into(),
                value_cm: self.thigh_circumference,
                kind: MeasurementType::Circumference,
                confidence: 1.0,
            },
            Measurement {
                name: "knee_circumference".into(),
                value_cm: self.knee_circumference,
                kind: MeasurementType::Circumference,
                confidence: 1.0,
            },
            Measurement {
                name: "calf_circumference".into(),
                value_cm: self.calf_circumference,
                kind: MeasurementType::Circumference,
                confidence: 1.0,
            },
            Measurement {
                name: "ankle_circumference".into(),
                value_cm: self.ankle_circumference,
                kind: MeasurementType::Circumference,
                confidence: 1.0,
            },
            Measurement {
                name: "head_circumference".into(),
                value_cm: self.head_circumference,
                kind: MeasurementType::Circumference,
                confidence: 1.0,
            },
            Measurement {
                name: "shoulder_breadth".into(),
                value_cm: self.shoulder_breadth,
                kind: MeasurementType::Width,
                confidence: 1.0,
            },
            Measurement {
                name: "arm_length".into(),
                value_cm: self.arm_length,
                kind: MeasurementType::Length,
                confidence: 1.0,
            },
            Measurement {
                name: "inseam".into(),
                value_cm: self.inseam,
                kind: MeasurementType::Length,
                confidence: 1.0,
            },
            Measurement {
                name: "torso_length".into(),
                value_cm: self.torso_length,
                kind: MeasurementType::Length,
                confidence: 1.0,
            },
            Measurement {
                name: "sitting_height".into(),
                value_cm: self.sitting_height,
                kind: MeasurementType::Height,
                confidence: 1.0,
            },
            Measurement {
                name: "foot_length".into(),
                value_cm: self.foot_length,
                kind: MeasurementType::Length,
                confidence: 0.8,
            },
            Measurement {
                name: "hand_length".into(),
                value_cm: self.hand_length,
                kind: MeasurementType::Length,
                confidence: 0.8,
            },
            Measurement {
                name: "bmi_estimate".into(),
                value_cm: self.bmi_estimate,
                kind: MeasurementType::Index,
                confidence: 0.7,
            },
            Measurement {
                name: "body_surface_area".into(),
                value_cm: self.body_surface_area,
                kind: MeasurementType::SurfaceArea,
                confidence: 1.0,
            },
            Measurement {
                name: "body_volume".into(),
                value_cm: self.body_volume,
                kind: MeasurementType::Volume,
                confidence: 1.0,
            },
        ]
    }
}

/// Body measurement calculator using cross-section slicing of a triangle mesh.
///
/// The mesh is assumed to be in a coordinate system where Y is up (height),
/// X is left-right, and Z is front-back.  Units are centimetres.
pub struct BodyMeasurements {
    vertex_positions: Vec<[f64; 3]>,
    triangles: Vec<[usize; 3]>,
    y_min: f64,
    y_max: f64,
    #[allow(dead_code)]
    x_min: f64,
    #[allow(dead_code)]
    x_max: f64,
    #[allow(dead_code)]
    z_min: f64,
    #[allow(dead_code)]
    z_max: f64,
}

// ---------- helpers ----------

fn dot(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn sub(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: &[f64; 3], s: f64) -> [f64; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn length(a: &[f64; 3]) -> f64 {
    dot(a, a).sqrt()
}

fn normalize(a: &[f64; 3]) -> Option<[f64; 3]> {
    let l = length(a);
    if l < 1e-15 {
        None
    } else {
        Some(scale(a, 1.0 / l))
    }
}

fn lerp_point(a: &[f64; 3], b: &[f64; 3], t: f64) -> [f64; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Signed distance of a point from a plane defined by (point, normal).
fn signed_dist(pt: &[f64; 3], plane_pt: &[f64; 3], plane_n: &[f64; 3]) -> f64 {
    dot(&sub(pt, plane_pt), plane_n)
}

/// Squared distance between two points.
fn dist_sq(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let d = sub(a, b);
    dot(&d, &d)
}

impl BodyMeasurements {
    /// Create a new measurement calculator from mesh vertices and triangle indices.
    pub fn new(vertices: Vec<[f64; 3]>, triangles: Vec<[usize; 3]>) -> Self {
        let (mut y_min, mut y_max) = (f64::MAX, f64::MIN);
        let (mut x_min, mut x_max) = (f64::MAX, f64::MIN);
        let (mut z_min, mut z_max) = (f64::MAX, f64::MIN);
        for v in &vertices {
            if v[1] < y_min {
                y_min = v[1];
            }
            if v[1] > y_max {
                y_max = v[1];
            }
            if v[0] < x_min {
                x_min = v[0];
            }
            if v[0] > x_max {
                x_max = v[0];
            }
            if v[2] < z_min {
                z_min = v[2];
            }
            if v[2] > z_max {
                z_max = v[2];
            }
        }
        Self {
            vertex_positions: vertices,
            triangles,
            y_min,
            y_max,
            x_min,
            x_max,
            z_min,
            z_max,
        }
    }

    /// Total body height (stature) as the Y extent of the bounding box.
    pub fn stature(&self) -> f64 {
        self.y_max - self.y_min
    }

    // ---- cross-section slicing ----

    /// Slice the mesh with an arbitrary plane, returning ordered intersection
    /// polygon(s).  Each inner `Vec` is one closed loop.
    fn slice_mesh(
        &self,
        plane_point: &[f64; 3],
        plane_normal: &[f64; 3],
    ) -> anyhow::Result<Vec<Vec<[f64; 3]>>> {
        let normal =
            normalize(plane_normal).ok_or_else(|| anyhow::anyhow!("zero-length plane normal"))?;

        // Phase 1: collect intersection segments
        let mut segments: Vec<([f64; 3], [f64; 3])> = Vec::new();

        for tri in &self.triangles {
            let v0 = &self.vertex_positions[tri[0]];
            let v1 = &self.vertex_positions[tri[1]];
            let v2 = &self.vertex_positions[tri[2]];

            let d0 = signed_dist(v0, plane_point, &normal);
            let d1 = signed_dist(v1, plane_point, &normal);
            let d2 = signed_dist(v2, plane_point, &normal);

            let mut pts = Vec::new();
            // Check each edge for intersection
            Self::edge_plane_intersect(v0, v1, d0, d1, &mut pts);
            Self::edge_plane_intersect(v1, v2, d1, d2, &mut pts);
            Self::edge_plane_intersect(v2, v0, d2, d0, &mut pts);

            // Deduplicate very close points
            pts.dedup_by(|a, b| dist_sq(a, b) < 1e-20);

            if pts.len() >= 2 {
                segments.push((pts[0], pts[1]));
            }
        }

        if segments.is_empty() {
            return Err(anyhow::anyhow!("no intersection found for the given plane"));
        }

        // Phase 2: chain segments into ordered loops
        let loops = Self::chain_segments(&segments)?;

        Ok(loops)
    }

    /// Test if an edge (p0->p1) with signed distances d0, d1 crosses the
    /// plane. If so, push the intersection point.
    fn edge_plane_intersect(
        p0: &[f64; 3],
        p1: &[f64; 3],
        d0: f64,
        d1: f64,
        pts: &mut Vec<[f64; 3]>,
    ) {
        const EPS: f64 = 1e-12;
        if d0.abs() < EPS {
            pts.push(*p0);
        } else if (d0 > 0.0 && d1 < 0.0) || (d0 < 0.0 && d1 > 0.0) {
            let t = d0 / (d0 - d1);
            pts.push(lerp_point(p0, p1, t));
        }
    }

    /// Chain unordered line segments into closed loops.
    fn chain_segments(segments: &[([f64; 3], [f64; 3])]) -> anyhow::Result<Vec<Vec<[f64; 3]>>> {
        if segments.is_empty() {
            return Ok(Vec::new());
        }

        // Build an adjacency structure using quantized points as keys
        let quantize = |p: &[f64; 3]| -> (i64, i64, i64) {
            let scale = 1e6;
            (
                (p[0] * scale).round() as i64,
                (p[1] * scale).round() as i64,
                (p[2] * scale).round() as i64,
            )
        };

        // Build adjacency map: quantized point -> list of (other_end, segment_index)
        type AdjMap = HashMap<(i64, i64, i64), Vec<(usize, [f64; 3])>>;
        let mut adjacency: AdjMap = HashMap::new();

        for (i, (a, b)) in segments.iter().enumerate() {
            let ka = quantize(a);
            let kb = quantize(b);
            adjacency.entry(ka).or_default().push((i, *b));
            adjacency.entry(kb).or_default().push((i, *a));
        }

        let mut used = vec![false; segments.len()];
        let mut loops = Vec::new();

        for start_idx in 0..segments.len() {
            if used[start_idx] {
                continue;
            }

            let mut chain = Vec::new();
            let (first_pt, _) = segments[start_idx];
            chain.push(first_pt);

            let mut current = segments[start_idx].1;
            chain.push(current);
            used[start_idx] = true;

            // Walk the chain
            let max_iters = segments.len() + 1;
            for _ in 0..max_iters {
                let key = quantize(&current);
                let mut found = false;
                if let Some(neighbors) = adjacency.get(&key) {
                    for &(seg_idx, other_end) in neighbors {
                        if !used[seg_idx] {
                            used[seg_idx] = true;
                            current = other_end;
                            chain.push(current);
                            found = true;
                            break;
                        }
                    }
                }
                if !found {
                    break;
                }
            }

            if chain.len() >= 3 {
                loops.push(chain);
            }
        }

        if loops.is_empty() {
            return Err(anyhow::anyhow!(
                "could not form any closed loops from intersection segments"
            ));
        }

        Ok(loops)
    }

    /// Order polygon points by angle around their centroid, projected onto
    /// the cutting plane.
    fn order_polygon_points(points: &[[f64; 3]], normal: &[f64; 3]) -> Vec<[f64; 3]> {
        if points.len() <= 2 {
            return points.to_vec();
        }

        // Compute centroid
        let n = points.len() as f64;
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        for p in points {
            cx += p[0];
            cy += p[1];
            cz += p[2];
        }
        let centroid = [cx / n, cy / n, cz / n];

        // Build local 2D frame on the plane
        // Pick a vector not parallel to normal
        let trial = if normal[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };

        let u = match normalize(&cross(normal, &trial)) {
            Some(v) => v,
            None => return points.to_vec(),
        };
        let v = cross(normal, &u);

        // Project each point to 2D and compute angle
        let mut indexed: Vec<(f64, usize)> = points
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let d = sub(p, &centroid);
                let px = dot(&d, &u);
                let py = dot(&d, &v);
                (py.atan2(px), i)
            })
            .collect();

        indexed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        indexed.iter().map(|&(_, i)| points[i]).collect()
    }

    /// Compute the perimeter of an ordered polygon.
    fn polygon_perimeter(points: &[[f64; 3]]) -> f64 {
        if points.len() < 2 {
            return 0.0;
        }
        let mut perim = 0.0;
        for i in 0..points.len() {
            let j = (i + 1) % points.len();
            perim += length(&sub(&points[j], &points[i]));
        }
        perim
    }

    /// Compute circumference at a given height (Y coordinate) by slicing the
    /// mesh with a horizontal plane.
    pub fn circumference_at_height(&self, y: f64) -> anyhow::Result<f64> {
        let plane_pt = [0.0, y, 0.0];
        let plane_n = [0.0, 1.0, 0.0];
        let loops = self.slice_mesh(&plane_pt, &plane_n)?;

        // Sum perimeters of all loops (handles multi-component cross-sections
        // like left+right legs).
        let mut total = 0.0;
        for loop_pts in &loops {
            total += Self::polygon_perimeter(loop_pts);
        }
        Ok(total)
    }

    /// Compute circumference along an arbitrary cutting plane.
    pub fn circumference_at_plane(
        &self,
        plane_point: &[f64; 3],
        plane_normal: &[f64; 3],
    ) -> anyhow::Result<f64> {
        let loops = self.slice_mesh(plane_point, plane_normal)?;
        let mut total = 0.0;
        for loop_pts in &loops {
            let ordered = Self::order_polygon_points(loop_pts, plane_normal);
            total += Self::polygon_perimeter(&ordered);
        }
        Ok(total)
    }

    /// Compute width (X extent) at a given height.
    pub fn width_at_height(&self, y: f64) -> anyhow::Result<f64> {
        let loops = self.slice_mesh(&[0.0, y, 0.0], &[0.0, 1.0, 0.0])?;
        let mut x_min = f64::MAX;
        let mut x_max = f64::MIN;
        for loop_pts in &loops {
            for p in loop_pts {
                if p[0] < x_min {
                    x_min = p[0];
                }
                if p[0] > x_max {
                    x_max = p[0];
                }
            }
        }
        if x_min > x_max {
            return Err(anyhow::anyhow!("no points found at height {y}"));
        }
        Ok(x_max - x_min)
    }

    /// Compute depth (Z extent) at a given height.
    pub fn depth_at_height(&self, y: f64) -> anyhow::Result<f64> {
        let loops = self.slice_mesh(&[0.0, y, 0.0], &[0.0, 1.0, 0.0])?;
        let mut z_min = f64::MAX;
        let mut z_max = f64::MIN;
        for loop_pts in &loops {
            for p in loop_pts {
                if p[2] < z_min {
                    z_min = p[2];
                }
                if p[2] > z_max {
                    z_max = p[2];
                }
            }
        }
        if z_min > z_max {
            return Err(anyhow::anyhow!("no points found at height {y}"));
        }
        Ok(z_max - z_min)
    }

    /// Compute total body volume using the divergence theorem.
    ///
    /// V = (1/6) * |sum over triangles of v0 . (v1 x v2)|
    pub fn body_volume(&self) -> anyhow::Result<f64> {
        let mut vol = 0.0;
        for tri in &self.triangles {
            if tri[0] >= self.vertex_positions.len()
                || tri[1] >= self.vertex_positions.len()
                || tri[2] >= self.vertex_positions.len()
            {
                return Err(anyhow::anyhow!("triangle index out of bounds"));
            }
            let v0 = &self.vertex_positions[tri[0]];
            let v1 = &self.vertex_positions[tri[1]];
            let v2 = &self.vertex_positions[tri[2]];
            vol += dot(v0, &cross(v1, v2));
        }
        Ok((vol / 6.0).abs())
    }

    /// Compute total surface area by summing triangle areas.
    pub fn surface_area(&self) -> anyhow::Result<f64> {
        let mut area = 0.0;
        for tri in &self.triangles {
            if tri[0] >= self.vertex_positions.len()
                || tri[1] >= self.vertex_positions.len()
                || tri[2] >= self.vertex_positions.len()
            {
                return Err(anyhow::anyhow!("triangle index out of bounds"));
            }
            let v0 = &self.vertex_positions[tri[0]];
            let v1 = &self.vertex_positions[tri[1]];
            let v2 = &self.vertex_positions[tri[2]];
            let e1 = sub(v1, v0);
            let e2 = sub(v2, v0);
            area += length(&cross(&e1, &e2)) * 0.5;
        }
        Ok(area)
    }

    /// Body surface area using the Du Bois formula.
    ///
    /// BSA (m²) = 0.007184 * height_cm^0.725 * weight_kg^0.425
    pub fn bsa_dubois(height_cm: f64, weight_kg: f64) -> f64 {
        0.007184 * height_cm.powf(0.725) * weight_kg.powf(0.425)
    }

    /// Body surface area using the Mosteller formula.
    ///
    /// BSA (m²) = sqrt(height_cm * weight_kg / 3600)
    pub fn bsa_mosteller(height_cm: f64, weight_kg: f64) -> f64 {
        ((height_cm * weight_kg) / 3600.0).sqrt()
    }

    /// Estimate BMI from body volume and height.
    ///
    /// Uses the relationship: weight ~ volume * density (density ~ 1.01 g/cm³ for human body).
    pub fn estimate_bmi(&self) -> anyhow::Result<f64> {
        let vol = self.body_volume()?;
        let height_cm = self.stature();
        if height_cm < 1.0 {
            return Err(anyhow::anyhow!("stature too small to compute BMI"));
        }
        let density_g_per_cm3 = 1.01;
        let weight_kg = vol * density_g_per_cm3 / 1000.0;
        let height_m = height_cm / 100.0;
        Ok(weight_kg / (height_m * height_m))
    }

    /// Estimate weight (kg) from body volume assuming average human density.
    pub fn estimate_weight_kg(&self) -> anyhow::Result<f64> {
        let vol = self.body_volume()?;
        let density_g_per_cm3 = 1.01;
        Ok(vol * density_g_per_cm3 / 1000.0)
    }

    // ---- landmark height ratios for a standard human ----
    // These are fractions of total height.  They represent typical
    // anthropometric landmark positions.

    fn height_frac(&self, frac: f64) -> f64 {
        self.y_min + self.stature() * frac
    }

    /// Maximum circumference search: scan a range of heights and return the
    /// maximum circumference found.
    fn max_circumference_in_range(
        &self,
        y_lo: f64,
        y_hi: f64,
        steps: usize,
    ) -> anyhow::Result<f64> {
        if steps == 0 {
            return Err(anyhow::anyhow!("steps must be > 0"));
        }
        let dy = (y_hi - y_lo) / steps as f64;
        let mut max_circ = 0.0_f64;
        for i in 0..=steps {
            let y = y_lo + dy * i as f64;
            match self.circumference_at_height(y) {
                Ok(c) => {
                    if c > max_circ {
                        max_circ = c;
                    }
                }
                Err(_) => continue,
            }
        }
        if max_circ <= 0.0 {
            return Err(anyhow::anyhow!(
                "no valid circumference found in range [{y_lo}, {y_hi}]"
            ));
        }
        Ok(max_circ)
    }

    /// Minimum circumference search in a height range.
    fn min_circumference_in_range(
        &self,
        y_lo: f64,
        y_hi: f64,
        steps: usize,
    ) -> anyhow::Result<f64> {
        if steps == 0 {
            return Err(anyhow::anyhow!("steps must be > 0"));
        }
        let dy = (y_hi - y_lo) / steps as f64;
        let mut min_circ = f64::MAX;
        for i in 0..=steps {
            let y = y_lo + dy * i as f64;
            match self.circumference_at_height(y) {
                Ok(c) => {
                    if c < min_circ && c > 0.0 {
                        min_circ = c;
                    }
                }
                Err(_) => continue,
            }
        }
        if min_circ >= f64::MAX {
            return Err(anyhow::anyhow!(
                "no valid circumference found in range [{y_lo}, {y_hi}]"
            ));
        }
        Ok(min_circ)
    }

    /// Maximum width (X extent) search in a height range.
    fn max_width_in_range(&self, y_lo: f64, y_hi: f64, steps: usize) -> anyhow::Result<f64> {
        if steps == 0 {
            return Err(anyhow::anyhow!("steps must be > 0"));
        }
        let dy = (y_hi - y_lo) / steps as f64;
        let mut max_w = 0.0_f64;
        for i in 0..=steps {
            let y = y_lo + dy * i as f64;
            match self.width_at_height(y) {
                Ok(w) => {
                    if w > max_w {
                        max_w = w;
                    }
                }
                Err(_) => continue,
            }
        }
        if max_w <= 0.0 {
            return Err(anyhow::anyhow!(
                "no valid width found in range [{y_lo}, {y_hi}]"
            ));
        }
        Ok(max_w)
    }

    /// Compute all standard anthropometric measurements.
    ///
    /// Uses typical landmark height ratios to locate measurement planes.
    /// The mesh should be in a standing T-pose with Y up.
    pub fn compute_all(&self) -> anyhow::Result<AnthropometricSet> {
        let h = self.stature();
        if h < 1.0 {
            return Err(anyhow::anyhow!(
                "mesh has negligible height ({h} cm), cannot compute measurements"
            ));
        }

        let scan_steps = 10;

        // Head: top 12.5% of height
        let _head_top = self.y_max;
        let _head_bottom = self.height_frac(0.875);
        let head_circ = self
            .max_circumference_in_range(self.height_frac(0.91), self.height_frac(0.96), scan_steps)
            .unwrap_or(h * 0.34);

        // Neck: ~82-87% of height
        let neck_circ = self
            .min_circumference_in_range(self.height_frac(0.82), self.height_frac(0.87), scan_steps)
            .unwrap_or(h * 0.22);

        // Shoulder breadth: max width in ~78-84% height range
        let shoulder_breadth = self
            .max_width_in_range(self.height_frac(0.78), self.height_frac(0.84), scan_steps)
            .unwrap_or(h * 0.26);

        // Chest: max circumference in ~72-78% height
        let chest_circ = self
            .max_circumference_in_range(self.height_frac(0.72), self.height_frac(0.78), scan_steps)
            .unwrap_or(h * 0.56);

        // Underbust: ~68-72% of height
        let underbust_circ = self
            .min_circumference_in_range(self.height_frac(0.68), self.height_frac(0.72), scan_steps)
            .unwrap_or(h * 0.48);

        // Waist: minimum circumference in ~58-68% height
        let waist_circ = self
            .min_circumference_in_range(self.height_frac(0.58), self.height_frac(0.68), scan_steps)
            .unwrap_or(h * 0.44);

        // Hip: max circumference in ~48-55% height
        let hip_circ = self
            .max_circumference_in_range(self.height_frac(0.48), self.height_frac(0.55), scan_steps)
            .unwrap_or(h * 0.58);

        // Upper arm: max circumference ~70-76% height
        // (approximation -- arms in T-pose may extend beyond torso)
        let upper_arm_circ = self
            .circumference_at_height(self.height_frac(0.73))
            .unwrap_or(h * 0.18);

        // Forearm: ~65-70% height
        let forearm_circ = self
            .circumference_at_height(self.height_frac(0.67))
            .unwrap_or(h * 0.15);

        // Wrist: ~48-50% height (wrist level, approximate)
        let wrist_circ = self
            .min_circumference_in_range(
                self.height_frac(0.48),
                self.height_frac(0.52),
                scan_steps / 2,
            )
            .unwrap_or(h * 0.10);

        // Thigh: max circumference ~42-48% height
        let thigh_circ = self
            .max_circumference_in_range(self.height_frac(0.42), self.height_frac(0.48), scan_steps)
            .unwrap_or(h * 0.35);

        // Knee: ~28-32% height
        let knee_circ = self
            .circumference_at_height(self.height_frac(0.30))
            .unwrap_or(h * 0.22);

        // Calf: max circumference ~20-28% height
        let calf_circ = self
            .max_circumference_in_range(self.height_frac(0.20), self.height_frac(0.28), scan_steps)
            .unwrap_or(h * 0.22);

        // Ankle: min circumference ~5-8% height
        let ankle_circ = self
            .min_circumference_in_range(
                self.height_frac(0.05),
                self.height_frac(0.08),
                scan_steps / 2,
            )
            .unwrap_or(h * 0.14);

        // Arm length: from shoulder height to wrist height (approximate)
        let arm_length = (self.height_frac(0.82) - self.height_frac(0.49)).abs();

        // Inseam: from crotch (~47% height) to floor
        let inseam = (self.height_frac(0.47) - self.y_min).abs();

        // Torso length: from shoulder to crotch
        let torso_length = (self.height_frac(0.82) - self.height_frac(0.47)).abs();

        // Sitting height: from top of head to seat (~52% height)
        let sitting_height = (self.y_max - self.height_frac(0.52)).abs();

        // Foot length: approximate from bounding box at ankle level
        let foot_length = self
            .depth_at_height(self.height_frac(0.02))
            .unwrap_or(h * 0.15);

        // Hand length: approximate (hand ~10.5% of stature)
        let hand_length = h * 0.105;

        // Volume and surface area
        let body_vol = self.body_volume().unwrap_or(0.0);
        let body_sa = self.surface_area().unwrap_or(0.0);

        // BMI estimate
        let bmi = self.estimate_bmi().unwrap_or(0.0);

        Ok(AnthropometricSet {
            stature: h,
            neck_circumference: neck_circ,
            chest_circumference: chest_circ,
            underbust_circumference: underbust_circ,
            waist_circumference: waist_circ,
            hip_circumference: hip_circ,
            upper_arm_circumference: upper_arm_circ,
            forearm_circumference: forearm_circ,
            wrist_circumference: wrist_circ,
            thigh_circumference: thigh_circ,
            knee_circumference: knee_circ,
            calf_circumference: calf_circ,
            ankle_circumference: ankle_circ,
            head_circumference: head_circ,
            shoulder_breadth,
            arm_length,
            inseam,
            torso_length,
            sitting_height,
            foot_length,
            hand_length,
            bmi_estimate: bmi,
            body_surface_area: body_sa,
            body_volume: body_vol,
        })
    }

    /// Compute a single named measurement.
    pub fn measure_by_name(&self, name: &str) -> anyhow::Result<Measurement> {
        let set = self.compute_all()?;
        let (value, kind) = match name {
            "stature" => (set.stature, MeasurementType::Height),
            "neck_circumference" => (set.neck_circumference, MeasurementType::Circumference),
            "chest_circumference" => (set.chest_circumference, MeasurementType::Circumference),
            "underbust_circumference" => {
                (set.underbust_circumference, MeasurementType::Circumference)
            }
            "waist_circumference" => (set.waist_circumference, MeasurementType::Circumference),
            "hip_circumference" => (set.hip_circumference, MeasurementType::Circumference),
            "upper_arm_circumference" => {
                (set.upper_arm_circumference, MeasurementType::Circumference)
            }
            "forearm_circumference" => (set.forearm_circumference, MeasurementType::Circumference),
            "wrist_circumference" => (set.wrist_circumference, MeasurementType::Circumference),
            "thigh_circumference" => (set.thigh_circumference, MeasurementType::Circumference),
            "knee_circumference" => (set.knee_circumference, MeasurementType::Circumference),
            "calf_circumference" => (set.calf_circumference, MeasurementType::Circumference),
            "ankle_circumference" => (set.ankle_circumference, MeasurementType::Circumference),
            "head_circumference" => (set.head_circumference, MeasurementType::Circumference),
            "shoulder_breadth" => (set.shoulder_breadth, MeasurementType::Width),
            "arm_length" => (set.arm_length, MeasurementType::Length),
            "inseam" => (set.inseam, MeasurementType::Length),
            "torso_length" => (set.torso_length, MeasurementType::Length),
            "sitting_height" => (set.sitting_height, MeasurementType::Height),
            "foot_length" => (set.foot_length, MeasurementType::Length),
            "hand_length" => (set.hand_length, MeasurementType::Length),
            "bmi_estimate" => (set.bmi_estimate, MeasurementType::Index),
            "body_surface_area" => (set.body_surface_area, MeasurementType::SurfaceArea),
            "body_volume" => (set.body_volume, MeasurementType::Volume),
            _ => return Err(anyhow::anyhow!("unknown measurement name: {name}")),
        };
        Ok(Measurement {
            name: name.to_string(),
            value_cm: value,
            kind,
            confidence: 1.0,
        })
    }

    /// Return the list of all supported measurement names.
    pub fn supported_measurements() -> &'static [&'static str] {
        &[
            "stature",
            "neck_circumference",
            "chest_circumference",
            "underbust_circumference",
            "waist_circumference",
            "hip_circumference",
            "upper_arm_circumference",
            "forearm_circumference",
            "wrist_circumference",
            "thigh_circumference",
            "knee_circumference",
            "calf_circumference",
            "ankle_circumference",
            "head_circumference",
            "shoulder_breadth",
            "arm_length",
            "inseam",
            "torso_length",
            "sitting_height",
            "foot_length",
            "hand_length",
            "bmi_estimate",
            "body_surface_area",
            "body_volume",
        ]
    }

    /// Compute the cross-sectional area at a given height.
    ///
    /// Slices the mesh and computes the area of the resulting polygon using
    /// the shoelace formula projected onto the cutting plane.
    pub fn cross_section_area_at_height(&self, y: f64) -> anyhow::Result<f64> {
        let normal = [0.0, 1.0, 0.0];
        let loops = self.slice_mesh(&[0.0, y, 0.0], &normal)?;

        let mut total_area = 0.0;
        for loop_pts in &loops {
            let ordered = Self::order_polygon_points(loop_pts, &normal);
            total_area += Self::polygon_area_2d(&ordered, &normal);
        }
        Ok(total_area)
    }

    /// Compute area of a planar polygon using the shoelace formula, projected
    /// onto the coordinate plane most aligned with `normal`.
    fn polygon_area_2d(points: &[[f64; 3]], normal: &[f64; 3]) -> f64 {
        if points.len() < 3 {
            return 0.0;
        }

        // Choose projection axes: drop the axis most aligned with normal
        let abs_n = [normal[0].abs(), normal[1].abs(), normal[2].abs()];
        let (ax_u, ax_v) = if abs_n[0] >= abs_n[1] && abs_n[0] >= abs_n[2] {
            (1, 2) // drop X
        } else if abs_n[1] >= abs_n[0] && abs_n[1] >= abs_n[2] {
            (0, 2) // drop Y
        } else {
            (0, 1) // drop Z
        };

        let mut area = 0.0;
        let n = points.len();
        for i in 0..n {
            let j = (i + 1) % n;
            area += points[i][ax_u] * points[j][ax_v];
            area -= points[j][ax_u] * points[i][ax_v];
        }
        (area / 2.0).abs()
    }

    /// Waist-to-hip ratio.
    pub fn waist_hip_ratio(&self) -> anyhow::Result<f64> {
        let set = self.compute_all()?;
        if set.hip_circumference < 1e-9 {
            return Err(anyhow::anyhow!("hip circumference is zero"));
        }
        Ok(set.waist_circumference / set.hip_circumference)
    }

    /// Waist-to-height ratio.
    pub fn waist_height_ratio(&self) -> anyhow::Result<f64> {
        let set = self.compute_all()?;
        if set.stature < 1e-9 {
            return Err(anyhow::anyhow!("stature is zero"));
        }
        Ok(set.waist_circumference / set.stature)
    }

    /// Sitting height ratio (sitting height / stature).
    pub fn sitting_height_ratio(&self) -> anyhow::Result<f64> {
        let set = self.compute_all()?;
        if set.stature < 1e-9 {
            return Err(anyhow::anyhow!("stature is zero"));
        }
        Ok(set.sitting_height / set.stature)
    }

    /// Ponderal index: height / cube_root(weight).
    pub fn ponderal_index(&self) -> anyhow::Result<f64> {
        let height_cm = self.stature();
        let weight_kg = self.estimate_weight_kg()?;
        if weight_kg < 1e-9 {
            return Err(anyhow::anyhow!("weight estimate is zero"));
        }
        let height_m = height_cm / 100.0;
        Ok(height_m / weight_kg.cbrt())
    }

    /// Cormic index: sitting_height / stature * 100.
    pub fn cormic_index(&self) -> anyhow::Result<f64> {
        Ok(self.sitting_height_ratio()? * 100.0)
    }

    /// Compute a height profile of circumferences for visualization.
    ///
    /// Returns `(height, circumference)` pairs from bottom to top.
    pub fn circumference_profile(&self, steps: usize) -> Vec<(f64, f64)> {
        let steps = steps.max(2);
        let dy = self.stature() / (steps - 1) as f64;
        let mut profile = Vec::with_capacity(steps);
        for i in 0..steps {
            let y = self.y_min + dy * i as f64;
            let circ = self.circumference_at_height(y).unwrap_or(0.0);
            profile.push((y, circ));
        }
        profile
    }

    /// Compute a height profile of cross-section areas.
    pub fn area_profile(&self, steps: usize) -> Vec<(f64, f64)> {
        let steps = steps.max(2);
        let dy = self.stature() / (steps - 1) as f64;
        let mut profile = Vec::with_capacity(steps);
        for i in 0..steps {
            let y = self.y_min + dy * i as f64;
            let area = self.cross_section_area_at_height(y).unwrap_or(0.0);
            profile.push((y, area));
        }
        profile
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a simple unit cube mesh: 8 vertices, 12 triangles.
    /// Cube from (0,0,0) to (1,1,1).
    fn unit_cube() -> (Vec<[f64; 3]>, Vec<[usize; 3]>) {
        let verts = vec![
            [0.0, 0.0, 0.0], // 0
            [1.0, 0.0, 0.0], // 1
            [1.0, 1.0, 0.0], // 2
            [0.0, 1.0, 0.0], // 3
            [0.0, 0.0, 1.0], // 4
            [1.0, 0.0, 1.0], // 5
            [1.0, 1.0, 1.0], // 6
            [0.0, 1.0, 1.0], // 7
        ];
        // 6 faces, 2 triangles each = 12 triangles
        // Winding order for outward-facing normals
        let tris = vec![
            // front (z=0)
            [0, 2, 1],
            [0, 3, 2],
            // back (z=1)
            [4, 5, 6],
            [4, 6, 7],
            // bottom (y=0)
            [0, 1, 5],
            [0, 5, 4],
            // top (y=1)
            [3, 6, 2],
            [3, 7, 6],
            // left (x=0)
            [0, 4, 7],
            [0, 7, 3],
            // right (x=1)
            [1, 2, 6],
            [1, 6, 5],
        ];
        (verts, tris)
    }

    /// Scale cube to given size.
    fn scaled_cube(sx: f64, sy: f64, sz: f64) -> (Vec<[f64; 3]>, Vec<[usize; 3]>) {
        let (mut verts, tris) = unit_cube();
        for v in &mut verts {
            v[0] *= sx;
            v[1] *= sy;
            v[2] *= sz;
        }
        (verts, tris)
    }

    #[test]
    fn test_stature() {
        let (verts, tris) = scaled_cube(30.0, 170.0, 20.0);
        let bm = BodyMeasurements::new(verts, tris);
        assert!((bm.stature() - 170.0).abs() < 1e-9);
    }

    #[test]
    fn test_body_volume_cube() {
        let (verts, tris) = unit_cube();
        let bm = BodyMeasurements::new(verts, tris);
        let vol = bm.body_volume().expect("volume computation failed");
        // Unit cube volume = 1.0
        assert!((vol - 1.0).abs() < 1e-6, "expected 1.0, got {vol}");
    }

    #[test]
    fn test_surface_area_cube() {
        let (verts, tris) = unit_cube();
        let bm = BodyMeasurements::new(verts, tris);
        let sa = bm.surface_area().expect("surface area failed");
        // Unit cube SA = 6.0
        assert!((sa - 6.0).abs() < 1e-6, "expected 6.0, got {sa}");
    }

    #[test]
    fn test_circumference_at_height_cube() {
        let (verts, tris) = unit_cube();
        let bm = BodyMeasurements::new(verts, tris);
        // Slice at y=0.5 should give a square cross-section with perimeter 4.0
        let circ = bm.circumference_at_height(0.5).expect("circ failed");
        assert!((circ - 4.0).abs() < 0.5, "expected ~4.0, got {circ}");
    }

    #[test]
    fn test_width_at_height_cube() {
        let (verts, tris) = unit_cube();
        let bm = BodyMeasurements::new(verts, tris);
        let w = bm.width_at_height(0.5).expect("width failed");
        assert!((w - 1.0).abs() < 0.1, "expected ~1.0, got {w}");
    }

    #[test]
    fn test_depth_at_height_cube() {
        let (verts, tris) = unit_cube();
        let bm = BodyMeasurements::new(verts, tris);
        let d = bm.depth_at_height(0.5).expect("depth failed");
        assert!((d - 1.0).abs() < 0.1, "expected ~1.0, got {d}");
    }

    #[test]
    fn test_bsa_dubois() {
        // For a 170cm, 70kg person: BSA ~ 1.81 m²
        let bsa = BodyMeasurements::bsa_dubois(170.0, 70.0);
        assert!(bsa > 1.5 && bsa < 2.2, "BSA {bsa} out of expected range");
    }

    #[test]
    fn test_bsa_mosteller() {
        let bsa = BodyMeasurements::bsa_mosteller(170.0, 70.0);
        assert!(bsa > 1.5 && bsa < 2.2, "BSA {bsa} out of expected range");
    }

    #[test]
    fn test_supported_measurements_count() {
        let names = BodyMeasurements::supported_measurements();
        assert!(
            names.len() >= 24,
            "expected 24+ measurements, got {}",
            names.len()
        );
    }

    #[test]
    fn test_scaled_volume() {
        let (verts, tris) = scaled_cube(10.0, 10.0, 10.0);
        let bm = BodyMeasurements::new(verts, tris);
        let vol = bm.body_volume().expect("volume failed");
        assert!((vol - 1000.0).abs() < 1e-3, "expected 1000, got {vol}");
    }

    #[test]
    fn test_scaled_surface_area() {
        let (verts, tris) = scaled_cube(2.0, 3.0, 4.0);
        let bm = BodyMeasurements::new(verts, tris);
        let sa = bm.surface_area().expect("sa failed");
        // SA = 2*(2*3 + 3*4 + 2*4) = 2*(6+12+8) = 52
        assert!((sa - 52.0).abs() < 1e-3, "expected 52, got {sa}");
    }

    #[test]
    fn test_compute_all_on_box() {
        // A rough "person-shaped" box: 30cm wide, 170cm tall, 20cm deep
        let (verts, tris) = scaled_cube(30.0, 170.0, 20.0);
        let bm = BodyMeasurements::new(verts, tris);
        let set = bm.compute_all().expect("compute_all failed");
        assert!((set.stature - 170.0).abs() < 1e-6);
        assert!(set.body_volume > 0.0);
        assert!(set.body_surface_area > 0.0);
    }

    #[test]
    fn test_measure_by_name() {
        let (verts, tris) = scaled_cube(30.0, 170.0, 20.0);
        let bm = BodyMeasurements::new(verts, tris);
        let m = bm
            .measure_by_name("stature")
            .expect("measure_by_name failed");
        assert_eq!(m.name, "stature");
        assert!((m.value_cm - 170.0).abs() < 1e-6);
    }

    #[test]
    fn test_measure_by_name_unknown() {
        let (verts, tris) = unit_cube();
        let bm = BodyMeasurements::new(verts, tris);
        assert!(bm.measure_by_name("nonexistent").is_err());
    }

    #[test]
    fn test_anthropometric_set_to_measurements() {
        let set = AnthropometricSet {
            stature: 170.0,
            neck_circumference: 37.0,
            chest_circumference: 95.0,
            underbust_circumference: 80.0,
            waist_circumference: 75.0,
            hip_circumference: 98.0,
            upper_arm_circumference: 30.0,
            forearm_circumference: 25.0,
            wrist_circumference: 17.0,
            thigh_circumference: 55.0,
            knee_circumference: 38.0,
            calf_circumference: 37.0,
            ankle_circumference: 23.0,
            head_circumference: 56.0,
            shoulder_breadth: 44.0,
            arm_length: 56.0,
            inseam: 80.0,
            torso_length: 60.0,
            sitting_height: 82.0,
            foot_length: 26.0,
            hand_length: 18.0,
            bmi_estimate: 24.0,
            body_surface_area: 18000.0,
            body_volume: 70000.0,
        };
        let ms = set.to_measurements();
        assert_eq!(ms.len(), 24);
    }

    #[test]
    fn test_circumference_profile() {
        let (verts, tris) = unit_cube();
        let bm = BodyMeasurements::new(verts, tris);
        let profile = bm.circumference_profile(5);
        assert_eq!(profile.len(), 5);
    }

    #[test]
    fn test_area_profile() {
        let (verts, tris) = unit_cube();
        let bm = BodyMeasurements::new(verts, tris);
        let profile = bm.area_profile(5);
        assert_eq!(profile.len(), 5);
    }

    #[test]
    fn test_cross_section_area_cube() {
        let (verts, tris) = unit_cube();
        let bm = BodyMeasurements::new(verts, tris);
        let area = bm.cross_section_area_at_height(0.5).expect("area failed");
        // Cross-section of unit cube at y=0.5 should be ~1.0
        assert!((area - 1.0).abs() < 0.5, "expected ~1.0, got {area}");
    }

    #[test]
    fn test_waist_hip_ratio() {
        let (verts, tris) = scaled_cube(30.0, 170.0, 20.0);
        let bm = BodyMeasurements::new(verts, tris);
        // For a box, waist and hip should be similar
        let ratio = bm.waist_hip_ratio().expect("whr failed");
        assert!(ratio > 0.0 && ratio < 3.0, "ratio {ratio} out of range");
    }

    #[test]
    fn test_empty_mesh() {
        let bm = BodyMeasurements::new(vec![], vec![]);
        assert!(bm.body_volume().expect("empty volume should be 0") < 1e-9);
        assert!(bm.surface_area().expect("empty sa should be 0") < 1e-9);
    }

    #[test]
    fn test_polygon_perimeter() {
        // Square with side 1
        let pts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ];
        let perim = BodyMeasurements::polygon_perimeter(&pts);
        assert!((perim - 4.0).abs() < 1e-9);
    }

    #[test]
    fn test_polygon_area_2d() {
        // Unit square in the XZ plane (normal = Y)
        let pts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ];
        let area = BodyMeasurements::polygon_area_2d(&pts, &[0.0, 1.0, 0.0]);
        assert!((area - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_order_polygon_points() {
        let normal = [0.0, 1.0, 0.0];
        // Shuffled square
        let pts = vec![
            [1.0, 0.0, 1.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 0.0],
        ];
        let ordered = BodyMeasurements::order_polygon_points(&pts, &normal);
        assert_eq!(ordered.len(), 4);
        // Perimeter of ordered should be 4.0 (unit square)
        let perim = BodyMeasurements::polygon_perimeter(&ordered);
        assert!((perim - 4.0).abs() < 1e-6, "perimeter {perim}");
    }

    #[test]
    fn test_estimate_weight() {
        let (verts, tris) = scaled_cube(30.0, 170.0, 20.0);
        let bm = BodyMeasurements::new(verts, tris);
        let w = bm.estimate_weight_kg().expect("weight failed");
        // 30*170*20 = 102000 cm³, * 1.01 / 1000 ~ 103 kg
        assert!(w > 50.0 && w < 200.0, "weight {w} out of range");
    }

    #[test]
    fn test_ponderal_index() {
        let (verts, tris) = scaled_cube(30.0, 170.0, 20.0);
        let bm = BodyMeasurements::new(verts, tris);
        let pi = bm.ponderal_index().expect("PI failed");
        assert!(pi > 0.0, "ponderal index should be positive");
    }
}
