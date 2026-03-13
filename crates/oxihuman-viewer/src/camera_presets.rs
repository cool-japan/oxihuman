// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Named camera preset positions for standard views.

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CameraPreset {
    Front,
    Back,
    Left,
    Right,
    Top,
    Bottom,
    FrontLeft45,
    FrontRight45,
    Isometric,
    Custom,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct CameraView {
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub up: [f32; 3],
    pub fov_deg: f32,
    pub near: f32,
    pub far: f32,
}

#[allow(dead_code)]
pub struct PresetLibrary {
    pub presets: Vec<(CameraPreset, CameraView)>,
}

#[allow(dead_code)]
pub fn preset_view(preset: CameraPreset, distance: f32) -> CameraView {
    let d = distance;
    let target = [0.0_f32, 0.0, 0.0];
    let fov = 45.0_f32;
    let near = 0.01_f32;
    let far = d * 10.0;

    let (position, up) = match preset {
        CameraPreset::Front => ([0.0, 0.0, -d], [0.0_f32, 1.0, 0.0]),
        CameraPreset::Back => ([0.0, 0.0, d], [0.0_f32, 1.0, 0.0]),
        CameraPreset::Left => ([-d, 0.0, 0.0], [0.0_f32, 1.0, 0.0]),
        CameraPreset::Right => ([d, 0.0, 0.0], [0.0_f32, 1.0, 0.0]),
        CameraPreset::Top => ([0.0, d, 0.0], [0.0_f32, 0.0, 1.0]),
        CameraPreset::Bottom => ([0.0, -d, 0.0], [0.0_f32, 0.0, -1.0]),
        CameraPreset::FrontLeft45 => {
            let p = d / std::f32::consts::SQRT_2;
            ([-p, 0.0, -p], [0.0_f32, 1.0, 0.0])
        }
        CameraPreset::FrontRight45 => {
            let p = d / std::f32::consts::SQRT_2;
            ([p, 0.0, -p], [0.0_f32, 1.0, 0.0])
        }
        CameraPreset::Isometric => {
            let q = d / 3.0_f32.sqrt();
            ([q, q, -q], [0.0_f32, 1.0, 0.0])
        }
        CameraPreset::Custom => ([0.0, 0.0, -d], [0.0_f32, 1.0, 0.0]),
    };

    CameraView {
        position,
        target,
        up,
        fov_deg: fov,
        near,
        far,
    }
}

#[allow(dead_code)]
pub fn default_preset_library(distance: f32) -> PresetLibrary {
    use CameraPreset::*;
    let standard = [
        Front,
        Back,
        Left,
        Right,
        Top,
        Bottom,
        FrontLeft45,
        FrontRight45,
        Isometric,
    ];
    let presets = standard
        .iter()
        .map(|&p| (p, preset_view(p, distance)))
        .collect();
    PresetLibrary { presets }
}

#[allow(dead_code)]
pub fn get_preset(lib: &PresetLibrary, preset: CameraPreset) -> Option<&CameraView> {
    lib.presets
        .iter()
        .find(|(p, _)| *p == preset)
        .map(|(_, v)| v)
}

#[allow(dead_code)]
pub fn add_custom_preset(lib: &mut PresetLibrary, view: CameraView) {
    lib.presets.push((CameraPreset::Custom, view));
}

/// Look-at view matrix (row-major, right-handed).
#[allow(dead_code)]
pub fn view_matrix(cam: &CameraView) -> [[f32; 4]; 4] {
    let f = normalize3(sub3(cam.target, cam.position));
    let r = normalize3(cross3(f, normalize3(cam.up)));
    let u = cross3(r, f);

    [
        [r[0], u[0], -f[0], 0.0],
        [r[1], u[1], -f[1], 0.0],
        [r[2], u[2], -f[2], 0.0],
        [
            -dot3(r, cam.position),
            -dot3(u, cam.position),
            dot3(f, cam.position),
            1.0,
        ],
    ]
}

/// Perspective projection matrix (column-major OpenGL convention).
#[allow(dead_code)]
pub fn projection_matrix(cam: &CameraView, aspect: f32) -> [[f32; 4]; 4] {
    let fov_rad = cam.fov_deg.to_radians();
    let f = 1.0 / (fov_rad / 2.0).tan();
    let nf = 1.0 / (cam.near - cam.far);

    [
        [f / aspect, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, (cam.far + cam.near) * nf, -1.0],
        [0.0, 0.0, 2.0 * cam.far * cam.near * nf, 0.0],
    ]
}

#[allow(dead_code)]
pub fn interpolate_views(a: &CameraView, b: &CameraView, t: f32) -> CameraView {
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    let lerp3 = |p: [f32; 3], q: [f32; 3]| -> [f32; 3] {
        [lerp(p[0], q[0]), lerp(p[1], q[1]), lerp(p[2], q[2])]
    };
    CameraView {
        position: lerp3(a.position, b.position),
        target: lerp3(a.target, b.target),
        up: normalize3(lerp3(a.up, b.up)),
        fov_deg: lerp(a.fov_deg, b.fov_deg),
        near: lerp(a.near, b.near),
        far: lerp(a.far, b.far),
    }
}

#[allow(dead_code)]
pub fn orbit_view(
    center: [f32; 3],
    azimuth_deg: f32,
    elevation_deg: f32,
    distance: f32,
) -> CameraView {
    let az = azimuth_deg.to_radians();
    let el = elevation_deg.to_radians();

    let x = center[0] + distance * el.cos() * az.sin();
    let y = center[1] + distance * el.sin();
    let z = center[2] + distance * el.cos() * az.cos();

    let up = if elevation_deg.abs() >= 89.9 {
        // Near poles: use a different up
        let az2 = (azimuth_deg + 90.0).to_radians();
        [az2.sin(), 0.0, az2.cos()]
    } else {
        [0.0, 1.0, 0.0]
    };

    CameraView {
        position: [x, y, z],
        target: center,
        up,
        fov_deg: 45.0,
        near: 0.01,
        far: distance * 10.0,
    }
}

#[allow(dead_code)]
pub fn camera_forward(cam: &CameraView) -> [f32; 3] {
    normalize3(sub3(cam.target, cam.position))
}

#[allow(dead_code)]
pub fn camera_right(cam: &CameraView) -> [f32; 3] {
    let fwd = camera_forward(cam);
    normalize3(cross3(fwd, normalize3(cam.up)))
}

#[allow(dead_code)]
pub fn zoom_view(cam: &CameraView, factor: f32) -> CameraView {
    let fwd = camera_forward(cam);
    let dist = len3(sub3(cam.target, cam.position));
    let new_dist = dist * factor;
    let new_pos = [
        cam.target[0] - fwd[0] * new_dist,
        cam.target[1] - fwd[1] * new_dist,
        cam.target[2] - fwd[2] * new_dist,
    ];
    let mut new_cam = cam.clone();
    new_cam.position = new_pos;
    new_cam
}

#[allow(dead_code)]
pub fn preset_name(preset: CameraPreset) -> &'static str {
    match preset {
        CameraPreset::Front => "Front",
        CameraPreset::Back => "Back",
        CameraPreset::Left => "Left",
        CameraPreset::Right => "Right",
        CameraPreset::Top => "Top",
        CameraPreset::Bottom => "Bottom",
        CameraPreset::FrontLeft45 => "FrontLeft45",
        CameraPreset::FrontRight45 => "FrontRight45",
        CameraPreset::Isometric => "Isometric",
        CameraPreset::Custom => "Custom",
    }
}

#[allow(dead_code)]
pub fn preset_count(lib: &PresetLibrary) -> usize {
    lib.presets.len()
}

// ── Math helpers ─────────────────────────────────────────────────────────────

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_view_nonzero_position() {
        let view = preset_view(CameraPreset::Front, 5.0);
        let len =
            (view.position[0].powi(2) + view.position[1].powi(2) + view.position[2].powi(2)).sqrt();
        assert!(len > 0.1);
    }

    #[test]
    fn test_view_matrix_is_4x4() {
        let view = preset_view(CameraPreset::Front, 5.0);
        let m = view_matrix(&view);
        assert_eq!(m.len(), 4);
        assert_eq!(m[0].len(), 4);
    }

    #[test]
    fn test_view_matrix_no_nan() {
        let view = preset_view(CameraPreset::Front, 5.0);
        let m = view_matrix(&view);
        for row in &m {
            for v in row {
                assert!(!v.is_nan(), "NaN in view matrix");
            }
        }
    }

    #[test]
    fn test_projection_matrix_is_4x4() {
        let view = preset_view(CameraPreset::Front, 5.0);
        let m = projection_matrix(&view, 16.0 / 9.0);
        assert_eq!(m.len(), 4);
        assert_eq!(m[0].len(), 4);
    }

    #[test]
    fn test_projection_matrix_no_nan() {
        let view = preset_view(CameraPreset::Front, 5.0);
        let m = projection_matrix(&view, 1.0);
        for row in &m {
            for v in row {
                assert!(!v.is_nan(), "NaN in projection matrix");
            }
        }
    }

    #[test]
    fn test_orbit_at_elevation_90_y_dominant() {
        // At elevation 90 degrees, position Y should be much larger than X/Z
        let view = orbit_view([0.0, 0.0, 0.0], 0.0, 90.0, 10.0);
        assert!(
            view.position[1] > 9.0,
            "Y should be ~distance at 90 deg elevation"
        );
    }

    #[test]
    fn test_interpolate_views_midpoint() {
        let a = preset_view(CameraPreset::Front, 5.0);
        let b = preset_view(CameraPreset::Back, 5.0);
        let mid = interpolate_views(&a, &b, 0.5);
        // At t=0.5 the z position should be between a and b
        assert!(mid.position[2] > a.position[2].min(b.position[2]));
        assert!(mid.position[2] < a.position[2].max(b.position[2]));
    }

    #[test]
    fn test_interpolate_views_at_zero_equals_a() {
        let a = preset_view(CameraPreset::Front, 5.0);
        let b = preset_view(CameraPreset::Back, 5.0);
        let v = interpolate_views(&a, &b, 0.0);
        assert!((v.position[2] - a.position[2]).abs() < 1e-4);
    }

    #[test]
    fn test_camera_forward_unit_length() {
        let view = preset_view(CameraPreset::Front, 5.0);
        let fwd = camera_forward(&view);
        let len = (fwd[0].powi(2) + fwd[1].powi(2) + fwd[2].powi(2)).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_camera_right_unit_length() {
        let view = preset_view(CameraPreset::Front, 5.0);
        let r = camera_right(&view);
        let len = (r[0].powi(2) + r[1].powi(2) + r[2].powi(2)).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_preset_name_nonempty() {
        use CameraPreset::*;
        let presets = [
            Front,
            Back,
            Left,
            Right,
            Top,
            Bottom,
            FrontLeft45,
            FrontRight45,
            Isometric,
            Custom,
        ];
        for p in &presets {
            assert!(!preset_name(*p).is_empty());
        }
    }

    #[test]
    fn test_preset_count() {
        let lib = default_preset_library(5.0);
        assert_eq!(preset_count(&lib), 9);
    }

    #[test]
    fn test_get_preset_found() {
        let lib = default_preset_library(5.0);
        assert!(get_preset(&lib, CameraPreset::Front).is_some());
    }

    #[test]
    fn test_add_custom_preset_increases_count() {
        let mut lib = default_preset_library(5.0);
        let before = preset_count(&lib);
        add_custom_preset(&mut lib, preset_view(CameraPreset::Custom, 3.0));
        assert_eq!(preset_count(&lib), before + 1);
    }

    #[test]
    fn test_zoom_view_changes_distance() {
        let view = preset_view(CameraPreset::Front, 5.0);
        let zoomed = zoom_view(&view, 0.5);
        let d_orig = len3(sub3(view.target, view.position));
        let d_zoom = len3(sub3(zoomed.target, zoomed.position));
        assert!((d_zoom - d_orig * 0.5).abs() < 1e-3);
    }
}
