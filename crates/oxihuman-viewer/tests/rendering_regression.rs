// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Software-rasterizer-based rendering regression tests.
//!
//! All tests run entirely on the CPU using `ScreenshotCapture::capture_software_render`.
//! No GPU or window system is required.

use oxihuman_viewer::lighting_presets::LightingPreset;
use oxihuman_viewer::{ImageBuffer, ScreenshotCapture};
use std::f64::consts::PI;

// ── Shared math / geometry helpers ────────────────────────────────────────────

/// Column-major 4x4 identity matrix.
fn identity() -> [f64; 16] {
    [
        1.0, 0.0, 0.0, 0.0, // col 0
        0.0, 1.0, 0.0, 0.0, // col 1
        0.0, 0.0, 1.0, 0.0, // col 2
        0.0, 0.0, 0.0, 1.0, // col 3
    ]
}

/// Simple orthographic projection: maps NDC x/y directly, depth z maps [-1,1]→[-1,1].
/// Column-major. Suitable for objects with positions roughly in [-1,1]^3.
fn ortho_proj() -> [f64; 16] {
    // Column-major perspective-divide-compatible matrix.
    // clip = proj * view * [x,y,z,1]^T
    // We want ndc_x = x, ndc_y = y, ndc_z = z for positions in [-1,1].
    // Use: w=1 output (no perspective divide effect).
    // Column-major [col][row*4+col], i.e. col0=rows 0-3, col1=rows 4-7, etc.
    [
        1.0, 0.0, 0.0, 0.0, // col 0
        0.0, 1.0, 0.0, 0.0, // col 1
        0.0, 0.0, 1.0, 0.0, // col 2
        0.0, 0.0, 0.0, 1.0, // col 3
    ]
}

/// Look-at view matrix (column-major f64).
/// Camera at `eye` looking at `target` with up=[0,1,0].
fn look_at(eye: [f64; 3], target: [f64; 3]) -> [f64; 16] {
    let up = [0.0_f64, 1.0, 0.0];
    let fwd = normalize3(sub3(target, eye));
    let right = normalize3(cross3(fwd, up));
    let up_r = cross3(right, fwd);

    let tx = -dot3(right, eye);
    let ty = -dot3(up_r, eye);
    let tz = dot3(fwd, eye);

    // Column-major
    [
        right[0], up_r[0], -fwd[0], 0.0, // col 0
        right[1], up_r[1], -fwd[1], 0.0, // col 1
        right[2], up_r[2], -fwd[2], 0.0, // col 2
        tx, ty, tz, 1.0, // col 3
    ]
}

/// Perspective projection matrix (column-major f64, OpenGL conventions).
fn perspective(fov_y_deg: f64, aspect: f64, near: f64, far: f64) -> [f64; 16] {
    let f = 1.0 / (fov_y_deg.to_radians() * 0.5).tan();
    let nf = 1.0 / (near - far);
    [
        f / aspect,
        0.0,
        0.0,
        0.0, // col 0
        0.0,
        f,
        0.0,
        0.0, // col 1
        0.0,
        0.0,
        (far + near) * nf,
        -1.0, // col 2
        0.0,
        0.0,
        2.0 * far * near * nf,
        0.0, // col 3
    ]
}

fn sub3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot3(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalize3(v: [f64; 3]) -> [f64; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-15 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

/// Compute per-vertex normals (area-weighted average of incident triangle normals).
fn compute_normals(positions: &[[f64; 3]], triangles: &[[usize; 3]]) -> Vec<[f64; 3]> {
    let mut normals = vec![[0.0_f64; 3]; positions.len()];
    for tri in triangles {
        let p0 = positions[tri[0]];
        let p1 = positions[tri[1]];
        let p2 = positions[tri[2]];
        let e1 = sub3(p1, p0);
        let e2 = sub3(p2, p0);
        let n = cross3(e1, e2);
        for &vi in tri {
            normals[vi][0] += n[0];
            normals[vi][1] += n[1];
            normals[vi][2] += n[2];
        }
    }
    normals.iter().map(|&n| normalize3(n)).collect()
}

/// Build a unit-sphere mesh via UV subdivision (latitude × longitude).
fn build_sphere(lat_steps: usize, lon_steps: usize) -> (Vec<[f64; 3]>, Vec<[usize; 3]>) {
    let mut positions: Vec<[f64; 3]> = Vec::new();
    let mut triangles: Vec<[usize; 3]> = Vec::new();

    // Generate vertices: lat in [0, PI], lon in [0, 2*PI]
    for lat in 0..=lat_steps {
        let theta = PI * lat as f64 / lat_steps as f64;
        for lon in 0..=lon_steps {
            let phi = 2.0 * PI * lon as f64 / lon_steps as f64;
            positions.push([
                theta.sin() * phi.cos(),
                theta.cos(),
                theta.sin() * phi.sin(),
            ]);
        }
    }

    let stride = lon_steps + 1;
    for lat in 0..lat_steps {
        for lon in 0..lon_steps {
            let a = lat * stride + lon;
            let b = a + 1;
            let c = a + stride;
            let d = c + 1;
            triangles.push([a, c, b]);
            triangles.push([b, c, d]);
        }
    }

    (positions, triangles)
}

/// Build a unit cube centred at origin with outward normals.
#[allow(clippy::type_complexity)]
fn build_cube() -> (Vec<[f64; 3]>, Vec<[usize; 3]>, Vec<[f64; 3]>) {
    // 6 faces × 4 verts = 24 vertices (no shared verts so normals are crisp)
    #[rustfmt::skip]
    let face_data: &[([f64; 3], [[f64; 3]; 4])] = &[
        // (normal, [v0, v1, v2, v3])
        ([0.0, 0.0,  1.0], [[-0.5,-0.5, 0.5],[ 0.5,-0.5, 0.5],[ 0.5, 0.5, 0.5],[-0.5, 0.5, 0.5]]),
        ([0.0, 0.0, -1.0], [[ 0.5,-0.5,-0.5],[-0.5,-0.5,-0.5],[-0.5, 0.5,-0.5],[ 0.5, 0.5,-0.5]]),
        ([-1.0, 0.0, 0.0], [[-0.5,-0.5,-0.5],[-0.5,-0.5, 0.5],[-0.5, 0.5, 0.5],[-0.5, 0.5,-0.5]]),
        ([ 1.0, 0.0, 0.0], [[ 0.5,-0.5, 0.5],[ 0.5,-0.5,-0.5],[ 0.5, 0.5,-0.5],[ 0.5, 0.5, 0.5]]),
        ([0.0, -1.0, 0.0], [[-0.5,-0.5,-0.5],[ 0.5,-0.5,-0.5],[ 0.5,-0.5, 0.5],[-0.5,-0.5, 0.5]]),
        ([0.0,  1.0, 0.0], [[-0.5, 0.5, 0.5],[ 0.5, 0.5, 0.5],[ 0.5, 0.5,-0.5],[-0.5, 0.5,-0.5]]),
    ];

    let mut positions: Vec<[f64; 3]> = Vec::new();
    let mut normals: Vec<[f64; 3]> = Vec::new();
    let mut triangles: Vec<[usize; 3]> = Vec::new();

    for (normal, verts) in face_data {
        let base = positions.len();
        for &v in verts {
            positions.push(v);
            normals.push(*normal);
        }
        // Two triangles per quad (CCW winding when viewed from outside)
        triangles.push([base, base + 1, base + 2]);
        triangles.push([base, base + 2, base + 3]);
    }

    (positions, triangles, normals)
}

/// Pixel luminance (linear approximation).
fn luminance(px: [u8; 4]) -> f64 {
    0.2126 * px[0] as f64 / 255.0 + 0.7152 * px[1] as f64 / 255.0 + 0.0722 * px[2] as f64 / 255.0
}

/// Average luminance of an entire image.
fn average_luminance(buf: &ImageBuffer) -> f64 {
    let n = (buf.width * buf.height) as f64;
    if n == 0.0 {
        return 0.0;
    }
    let mut sum = 0.0;
    for y in 0..buf.height {
        for x in 0..buf.width {
            if let Some(px) = buf.pixel_at(x, y) {
                sum += luminance(px);
            }
        }
    }
    sum / n
}

/// Background colour hardcoded in `ScreenshotCapture` (R=38, G=38, B=46, A=255).
const BG: [u8; 4] = [38, 38, 46, 255];

// ── Tests ─────────────────────────────────────────────────────────────────────

/// Render an empty scene (no geometry) — all pixels must equal the background.
#[test]
fn test_render_empty_scene() {
    let cap = ScreenshotCapture::new(64, 64);
    let buf = cap
        .capture_software_render(
            &[],
            &[],
            &[],
            &identity(),
            &ortho_proj(),
            &LightingPreset::studio(),
        )
        .expect("should succeed");

    assert_eq!(buf.width, 64);
    assert_eq!(buf.height, 64);
    assert_eq!(buf.channels, 4);

    for y in 0..buf.height {
        for x in 0..buf.width {
            let px = buf.pixel_at(x, y).expect("should succeed");
            assert_eq!(
                px, BG,
                "pixel ({x},{y}) expected background {BG:?}, got {px:?}"
            );
        }
    }
}

/// Render a single white-lit triangle facing the camera from z=0.
/// The triangle centre pixel must be brighter (higher luminance) than the four corners.
#[test]
fn test_render_single_triangle() {
    // Large triangle that covers most of the image in NDC space.
    // Vertices ordered CCW when viewed from +z (camera looking along -z, i.e. eye at z=2):
    // With our look_at matrix, the forward direction flips y in screen space, so we must
    // ensure cross_z >= 0 in screen space. We use a triangle pointing *up* in NDC:
    //   top-centre, bottom-left, bottom-right → CCW in standard math orientation.
    // Screen-space y is flipped (sy = (1 - (ndc_y*0.5+0.5)) * h), so a CCW triangle in NDC
    // becomes CW in screen space, which would be culled. We therefore wind the triangle CW
    // in world/NDC space so it appears CCW in screen space.
    let positions: Vec<[f64; 3]> = vec![
        [0.0, 0.8, 0.0],   // top centre
        [0.8, -0.7, 0.0],  // bottom right  (CW winding in NDC → CCW in screen space)
        [-0.8, -0.7, 0.0], // bottom left
    ];
    let normals = vec![[0.0, 0.0, 1.0]; 3]; // facing +z toward camera
    let triangles = vec![[0, 1, 2]];

    let w = 64u32;
    let h = 64u32;
    let cap = ScreenshotCapture::new(w, h);

    // Camera at (0,0,2) looking at origin — view matrix puts geometry at z~0 in eye space.
    // Use identity view + ortho proj so NDC == world positions.
    let buf = cap
        .capture_software_render(
            &positions,
            &triangles,
            &normals,
            &identity(),
            &ortho_proj(),
            &LightingPreset::medical(), // flat, even illumination
        )
        .expect("should succeed");

    // Centre pixel should be inside the triangle and lit
    let cx = w / 2;
    let cy = h / 2;
    let center = buf.pixel_at(cx, cy).expect("should succeed");
    let center_lum = luminance(center);
    let bg_lum = luminance(BG);

    assert!(
        center_lum > bg_lum,
        "centre pixel should be brighter than background: centre lum={center_lum:.4}, bg lum={bg_lum:.4}"
    );

    // Corners (0,0), (w-1,0), (0,h-1), (w-1,h-1) should remain at background
    let corners = [(0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1)];
    for (cx, cy) in corners {
        let px = buf.pixel_at(cx, cy).expect("should succeed");
        assert_eq!(
            px, BG,
            "corner ({cx},{cy}) should be background, got {px:?}"
        );
    }
}

/// Render a unit cube and verify the bounding box of lit pixels is roughly square.
#[test]
fn test_render_cube_silhouette() {
    let (positions, triangles, normals) = build_cube();

    let w = 128u32;
    let h = 128u32;
    let cap = ScreenshotCapture::new(w, h);

    // Camera at (0,0,3) looking at origin with a perspective projection.
    // The cube sits in [-0.5,0.5]^3. At distance 3 with fov=60°, it should
    // project to a modest rectangular region near the centre of the image.
    let view = look_at([0.0, 0.0, 3.0], [0.0, 0.0, 0.0]);
    let proj = perspective(60.0, w as f64 / h as f64, 0.1, 100.0);

    let buf = cap
        .capture_software_render(
            &positions,
            &triangles,
            &normals,
            &view,
            &proj,
            &LightingPreset::studio(),
        )
        .expect("should succeed");

    // Collect the bounding box of all non-background pixels
    let mut min_x = w;
    let mut max_x = 0u32;
    let mut min_y = h;
    let mut max_y = 0u32;
    let mut lit_count = 0u32;

    for y in 0..h {
        for x in 0..w {
            let px = buf.pixel_at(x, y).expect("should succeed");
            if px != BG {
                lit_count += 1;
                if x < min_x {
                    min_x = x;
                }
                if x > max_x {
                    max_x = x;
                }
                if y < min_y {
                    min_y = y;
                }
                if y > max_y {
                    max_y = y;
                }
            }
        }
    }

    assert!(
        lit_count > 0,
        "cube render should produce at least one lit pixel"
    );

    let span_x = (max_x as i32 - min_x as i32).abs() + 1;
    let span_y = (max_y as i32 - min_y as i32).abs() + 1;

    // Bounding box should be roughly square: the spans should agree within 30%
    let ratio = span_x as f64 / span_y as f64;
    assert!(
        ratio > 0.7 && ratio < 1.3,
        "cube silhouette bounding box should be roughly square: span_x={span_x}, span_y={span_y}, ratio={ratio:.3}"
    );

    // Bounding box should be centred on the image (within ±20% of image size)
    let cx = (min_x + max_x) / 2;
    let cy = (min_y + max_y) / 2;
    let img_cx = w / 2;
    let img_cy = h / 2;
    let max_off = w / 5;

    assert!(
        (cx as i32 - img_cx as i32).unsigned_abs() <= max_off,
        "cube silhouette should be centred horizontally: cx={cx}, img_cx={img_cx}"
    );
    assert!(
        (cy as i32 - img_cy as i32).unsigned_abs() <= max_off,
        "cube silhouette should be centred vertically: cy={cy}, img_cy={img_cy}"
    );
}

/// Render the same sphere under all 6 lighting presets.
/// Each preset should produce a measurably different average brightness.
#[test]
fn test_render_lighting_presets() {
    let (positions, triangles) = build_sphere(12, 16);
    let normals = compute_normals(&positions, &triangles);

    let w = 64u32;
    let h = 64u32;
    let cap = ScreenshotCapture::new(w, h);

    let view = look_at([0.0, 0.0, 3.0], [0.0, 0.0, 0.0]);
    let proj = perspective(60.0, 1.0, 0.1, 100.0);

    let presets = LightingPreset::all_presets();
    assert_eq!(presets.len(), 6, "should have 6 presets");

    let avg_lums: Vec<f64> = presets
        .iter()
        .map(|preset| {
            let buf = cap
                .capture_software_render(&positions, &triangles, &normals, &view, &proj, preset)
                .expect("should succeed");
            average_luminance(&buf)
        })
        .collect();

    // Verify no two presets produce identical average luminance.
    // (Allow a tiny epsilon for floating-point rounding, but presets should
    //  differ by at least 0.001 in average luminance from at least one other.)
    let all_same = avg_lums.windows(2).all(|w| (w[0] - w[1]).abs() < 0.001);
    assert!(
        !all_same,
        "not all lighting presets should produce identical brightness: {avg_lums:?}"
    );

    // More specifically: the minimum and maximum average luminance must differ
    // by at least 0.01 across the 6 presets.
    let min_lum = avg_lums.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_lum = avg_lums.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    assert!(
        max_lum - min_lum > 0.01,
        "lighting presets should span at least 0.01 luminance range: min={min_lum:.4}, max={max_lum:.4}, values={avg_lums:?}"
    );
}

/// Render cube in solid (filled) mode vs a wireframe simulation.
/// Wireframe mode is simulated by rendering only edge-adjacent thin triangles,
/// producing a result with more dark (background) pixels in the interior.
///
/// Strategy: compare the rendered cube against the background-only image.
/// The solid render must have significantly more lit pixels than the wireframe render.
#[test]
fn test_render_wireframe_overlay() {
    let w = 128u32;
    let h = 128u32;
    let cap = ScreenshotCapture::new(w, h);

    let view = look_at([1.5, 1.2, 2.5], [0.0, 0.0, 0.0]);
    let proj = perspective(60.0, 1.0, 0.1, 100.0);

    // ── Solid render ──────────────────────────────────────────────────────────
    let (solid_pos, solid_tri, solid_nrm) = build_cube();
    let solid_buf = cap
        .capture_software_render(
            &solid_pos,
            &solid_tri,
            &solid_nrm,
            &view,
            &proj,
            &LightingPreset::studio(),
        )
        .expect("should succeed");

    // ── Wireframe simulation: render only thin edge triangles ─────────────────
    // For each edge of each triangle, build a very thin degenerate triangle
    // (edge + a point displaced slightly toward the edge midpoint). This produces
    // thin lit strips along the edges with the interior remaining background.
    let mut wire_pos: Vec<[f64; 3]> = Vec::new();
    let mut wire_tri: Vec<[usize; 3]> = Vec::new();
    let mut wire_nrm: Vec<[f64; 3]> = Vec::new();

    const WIRE_WIDTH: f64 = 0.018; // fractional edge width

    for tri in &solid_tri {
        let p = [solid_pos[tri[0]], solid_pos[tri[1]], solid_pos[tri[2]]];
        let n = [solid_nrm[tri[0]], solid_nrm[tri[1]], solid_nrm[tri[2]]];

        // Centroid of triangle
        let centroid = [
            (p[0][0] + p[1][0] + p[2][0]) / 3.0,
            (p[0][1] + p[1][1] + p[2][1]) / 3.0,
            (p[0][2] + p[1][2] + p[2][2]) / 3.0,
        ];

        // For each of the 3 edges, create a thin quad strip
        for edge in 0..3usize {
            let v0 = p[edge];
            let v1 = p[(edge + 1) % 3];
            let n0 = n[edge];
            let n1 = n[(edge + 1) % 3];

            // Inset versions: pull v0/v1 slightly toward centroid for a thin strip
            let inset_v0 = [
                v0[0] + WIRE_WIDTH * (centroid[0] - v0[0]),
                v0[1] + WIRE_WIDTH * (centroid[1] - v0[1]),
                v0[2] + WIRE_WIDTH * (centroid[2] - v0[2]),
            ];
            let inset_v1 = [
                v1[0] + WIRE_WIDTH * (centroid[0] - v1[0]),
                v1[1] + WIRE_WIDTH * (centroid[1] - v1[1]),
                v1[2] + WIRE_WIDTH * (centroid[2] - v1[2]),
            ];

            let base = wire_pos.len();
            wire_pos.push(v0);
            wire_pos.push(v1);
            wire_pos.push(inset_v0);
            wire_pos.push(inset_v1);
            wire_nrm.push(n0);
            wire_nrm.push(n1);
            wire_nrm.push(n0);
            wire_nrm.push(n1);
            wire_tri.push([base, base + 1, base + 2]);
            wire_tri.push([base + 1, base + 3, base + 2]);
        }
    }

    let wire_buf = cap
        .capture_software_render(
            &wire_pos,
            &wire_tri,
            &wire_nrm,
            &view,
            &proj,
            &LightingPreset::studio(),
        )
        .expect("should succeed");

    // Count lit pixels in each render
    let solid_lit = count_non_bg_pixels(&solid_buf);
    let wire_lit = count_non_bg_pixels(&wire_buf);

    assert!(
        solid_lit > 0,
        "solid render should have lit pixels, got {solid_lit}"
    );
    assert!(
        wire_lit > 0,
        "wireframe render should have some lit pixels, got {wire_lit}"
    );
    assert!(
        solid_lit > wire_lit,
        "solid render should have more lit pixels than wireframe: solid={solid_lit}, wire={wire_lit}"
    );
}

fn count_non_bg_pixels(buf: &ImageBuffer) -> u32 {
    let mut count = 0u32;
    for y in 0..buf.height {
        for x in 0..buf.width {
            if buf.pixel_at(x, y).expect("should succeed") != BG {
                count += 1;
            }
        }
    }
    count
}

/// Render a frame and export as PPM binary format.
/// Verify the P6 header, width/height in the header, and total byte count.
#[test]
fn test_screenshot_ppm_format() {
    use std::io::Write as _;

    let w = 32u32;
    let h = 24u32;

    let (positions, triangles) = build_sphere(8, 12);
    let normals = compute_normals(&positions, &triangles);

    let cap = ScreenshotCapture::new(w, h);
    let view = look_at([0.0, 0.0, 3.0], [0.0, 0.0, 0.0]);
    let proj = perspective(60.0, w as f64 / h as f64, 0.1, 100.0);

    let buf = cap
        .capture_software_render(
            &positions,
            &triangles,
            &normals,
            &view,
            &proj,
            &LightingPreset::studio(),
        )
        .expect("should succeed");

    let ppm_bytes = buf.to_ppm().expect("should succeed");

    // Verify P6 magic bytes
    assert!(
        ppm_bytes.starts_with(b"P6\n"),
        "PPM must start with 'P6\\n'"
    );

    // Parse the PPM header (ASCII-only portion before the third newline).
    // The pixel data that follows may contain arbitrary binary bytes, so we must
    // not attempt to decode the full buffer as UTF-8.
    let header_end = ppm_bytes
        .windows(1)
        .enumerate()
        .filter(|(_, b)| b[0] == b'\n')
        .nth(2)
        .map(|(i, _)| i + 1)
        .expect("PPM header must have 3 newlines");

    let header_str = std::str::from_utf8(&ppm_bytes[..header_end]).expect("should succeed");
    let mut lines = header_str.lines();
    let magic = lines.next().expect("should succeed");
    assert_eq!(magic, "P6");

    let dims_line = lines.next().expect("should succeed");
    let mut dims = dims_line.split_whitespace();
    let parsed_w: u32 = dims
        .next()
        .expect("should succeed")
        .parse()
        .expect("should succeed");
    let parsed_h: u32 = dims
        .next()
        .expect("should succeed")
        .parse()
        .expect("should succeed");
    assert_eq!(parsed_w, w, "PPM width mismatch");
    assert_eq!(parsed_h, h, "PPM height mismatch");

    let maxval_line = lines.next().expect("should succeed");
    assert_eq!(maxval_line, "255", "PPM maxval should be 255");

    // Total size: header bytes + width * height * 3 (RGB)
    let expected_pixel_bytes = w as usize * h as usize * 3;
    assert_eq!(
        ppm_bytes.len() - header_end,
        expected_pixel_bytes,
        "PPM pixel data size mismatch"
    );

    // Write to temp file and verify it can be read back
    let tmp = std::env::temp_dir().join("oxihuman_test_render.ppm");
    let mut f = std::fs::File::create(&tmp).expect("should succeed");
    f.write_all(&ppm_bytes).expect("should succeed");
    let read_back = std::fs::read(&tmp).expect("should succeed");
    assert_eq!(read_back, ppm_bytes, "PPM file round-trip failed");
    let _ = std::fs::remove_file(&tmp);
}

/// Render a frame and export as TGA format.
/// Verify the 18-byte header with correct magic values, dimensions and pixel depth.
#[test]
fn test_screenshot_tga_format() {
    use std::io::Write as _;

    let w = 40u32;
    let h = 30u32;

    let cap = ScreenshotCapture::new(w, h);
    // Use an empty scene for a simple, deterministic result
    let buf = cap
        .capture_software_render(
            &[],
            &[],
            &[],
            &identity(),
            &ortho_proj(),
            &LightingPreset::studio(),
        )
        .expect("should succeed");

    let tga_bytes = buf.to_tga().expect("should succeed");

    // TGA header is exactly 18 bytes
    assert!(
        tga_bytes.len() >= 18,
        "TGA must have at least 18-byte header"
    );

    // Byte 0: ID length (0 = no image ID)
    assert_eq!(tga_bytes[0], 0, "TGA id_length must be 0");
    // Byte 1: colour map type (0 = no colour map)
    assert_eq!(tga_bytes[1], 0, "TGA color_map_type must be 0");
    // Byte 2: image type (2 = uncompressed true-color)
    assert_eq!(
        tga_bytes[2], 2,
        "TGA image_type must be 2 (uncompressed true-color)"
    );

    // Bytes 12-13: width (little-endian u16)
    let tga_w = u16::from_le_bytes([tga_bytes[12], tga_bytes[13]]);
    assert_eq!(tga_w as u32, w, "TGA width mismatch");

    // Bytes 14-15: height (little-endian u16)
    let tga_h = u16::from_le_bytes([tga_bytes[14], tga_bytes[15]]);
    assert_eq!(tga_h as u32, h, "TGA height mismatch");

    // Byte 16: pixel depth (32 for RGBA)
    assert_eq!(tga_bytes[16], 32, "TGA pixel depth must be 32");

    // Byte 17: image descriptor (bit 5 = top-to-bottom, bits 0-3 = 8 alpha bits)
    assert_eq!(tga_bytes[17], 0x28, "TGA image descriptor must be 0x28");

    // Total size: 18-byte header + w*h*4 pixel bytes
    let expected_total = 18 + w as usize * h as usize * 4;
    assert_eq!(tga_bytes.len(), expected_total, "TGA total size mismatch");

    // Pixel data starts at offset 18. For an empty scene, all pixels are background.
    // TGA stores BGRA: B=46, G=38, R=38, A=255
    assert_eq!(tga_bytes[18], 46, "TGA first pixel B channel (background)");
    assert_eq!(tga_bytes[19], 38, "TGA first pixel G channel (background)");
    assert_eq!(tga_bytes[20], 38, "TGA first pixel R channel (background)");
    assert_eq!(tga_bytes[21], 255, "TGA first pixel A channel (background)");

    // Write to temp file
    let tmp = std::env::temp_dir().join("oxihuman_test_render.tga");
    let mut f = std::fs::File::create(&tmp).expect("should succeed");
    f.write_all(&tga_bytes).expect("should succeed");
    let read_back = std::fs::read(&tmp).expect("should succeed");
    assert_eq!(read_back, tga_bytes, "TGA file round-trip failed");
    let _ = std::fs::remove_file(&tmp);
}

/// Render the same scene twice with identical parameters.
/// The two results must be pixel-for-pixel identical (deterministic rendering).
#[test]
fn test_render_deterministic() {
    let (positions, triangles) = build_sphere(10, 14);
    let normals = compute_normals(&positions, &triangles);

    let w = 64u32;
    let h = 64u32;
    let view = look_at([2.0, 1.0, 3.0], [0.0, 0.0, 0.0]);
    let proj = perspective(55.0, 1.0, 0.1, 50.0);
    let preset = LightingPreset::outdoor();

    let cap = ScreenshotCapture::new(w, h);

    let buf_a = cap
        .capture_software_render(&positions, &triangles, &normals, &view, &proj, &preset)
        .expect("should succeed");

    let buf_b = cap
        .capture_software_render(&positions, &triangles, &normals, &view, &proj, &preset)
        .expect("should succeed");

    assert_eq!(
        buf_a.data, buf_b.data,
        "Two renders of the same scene must be pixel-identical (deterministic)"
    );
}

/// A mesh with normals pointing toward the camera must appear brighter than
/// one whose normals point away (the back face, backlit scenario).
///
/// Setup: camera at origin using identity view matrix. A quad at z=0 is
/// rendered twice — once with normals `[0,0,1]` (toward +z, toward camera)
/// and once with normals `[0,0,-1]` (away from camera).
///
/// A single strong directional light comes from the +z side: its `direction`
/// vector is `[0,0,1]` so `to_light = -direction = [0,0,-1]`… wait — we
/// actually want `to_light = [0,0,1]` so we point the light's travel direction
/// as `[0,0,-1]` (light *travels* toward -z, meaning it arrives from +z).
/// `to_light = -direction = [0,0,1]`.
/// Then `dot(n=[0,0,1], to_light=[0,0,1]) = 1` → maximum diffuse.
/// And `dot(n=[0,0,-1], to_light=[0,0,1]) = -1` → clamped to 0, only ambient.
#[test]
fn test_render_normal_map_shading() {
    use oxihuman_viewer::lighting_presets::{Light, LightKind};

    // Quad vertices; CW winding in world/NDC space → CCW in screen space → front-facing
    let positions: Vec<[f64; 3]> = vec![
        [-0.5, 0.5, 0.0],  // top-left
        [0.5, 0.5, 0.0],   // top-right
        [0.5, -0.5, 0.0],  // bottom-right
        [-0.5, -0.5, 0.0], // bottom-left
    ];
    let triangles: Vec<[usize; 3]> = vec![[0, 1, 2], [0, 2, 3]];

    // Normals pointing toward +z (same direction as camera in identity view)
    let normals_toward: Vec<[f64; 3]> = vec![[0.0, 0.0, 1.0]; 4];
    // Normals pointing toward -z (away from camera)
    let normals_away: Vec<[f64; 3]> = vec![[0.0, 0.0, -1.0]; 4];

    // A single strong directional light: direction=[0,0,-1] → to_light=[0,0,1].
    // This maximally illuminates normals = [0,0,1] and not [0,0,-1].
    let front_light = Light {
        kind: LightKind::Directional,
        position: [0.0; 3],
        direction: [0.0, 0.0, -1.0], // normalised by hand
        color: [1.0, 1.0, 1.0],
        intensity: 3.0,
        radius: f64::INFINITY,
    };
    let preset = LightingPreset::custom(vec![front_light], [0.05, 0.05, 0.05]);

    let w = 64u32;
    let h = 64u32;
    let cap = ScreenshotCapture::new(w, h);
    let view = identity();
    let proj = ortho_proj();

    let buf_toward = cap
        .capture_software_render(
            &positions,
            &triangles,
            &normals_toward,
            &view,
            &proj,
            &preset,
        )
        .expect("should succeed");

    let buf_away = cap
        .capture_software_render(&positions, &triangles, &normals_away, &view, &proj, &preset)
        .expect("should succeed");

    // Compute average luminance over a central patch where the quad is visible
    let cx = w / 2;
    let cy = h / 2;
    let sample_radius = 10u32;

    let avg_lum_in_patch = |buf: &ImageBuffer| -> f64 {
        let mut sum = 0.0;
        let mut count = 0u32;
        for dy in 0..sample_radius * 2 {
            for dx in 0..sample_radius * 2 {
                let px_x = cx.saturating_sub(sample_radius) + dx;
                let px_y = cy.saturating_sub(sample_radius) + dy;
                if px_x < w && px_y < h {
                    if let Some(px) = buf.pixel_at(px_x, px_y) {
                        sum += luminance(px);
                        count += 1;
                    }
                }
            }
        }
        if count > 0 {
            sum / count as f64
        } else {
            0.0
        }
    };

    let lum_toward = avg_lum_in_patch(&buf_toward);
    let lum_away = avg_lum_in_patch(&buf_away);

    assert!(
        lum_toward > lum_away,
        "normals pointing toward camera should produce higher brightness than normals pointing away: \
         toward={lum_toward:.4}, away={lum_away:.4}"
    );
}

/// Orbit the camera 90° around the scene and verify the rendered output changes.
/// The pixel-wise mean absolute difference must exceed a meaningful threshold.
#[test]
fn test_camera_orbit_changes_view() {
    let (positions, triangles) = build_sphere(12, 16);
    let normals = compute_normals(&positions, &triangles);

    let w = 64u32;
    let h = 64u32;
    let cap = ScreenshotCapture::new(w, h);
    let proj = perspective(60.0, 1.0, 0.1, 100.0);
    let preset = LightingPreset::studio();

    // Original camera at (0,0,3)
    let view_a = look_at([0.0, 0.0, 3.0], [0.0, 0.0, 0.0]);

    // Orbited camera 90° around Y axis: (3,0,0)
    let view_b = look_at([3.0, 0.0, 0.0], [0.0, 0.0, 0.0]);

    let buf_a = cap
        .capture_software_render(&positions, &triangles, &normals, &view_a, &proj, &preset)
        .expect("should succeed");

    let buf_b = cap
        .capture_software_render(&positions, &triangles, &normals, &view_b, &proj, &preset)
        .expect("should succeed");

    // Calculate mean absolute difference per channel across all pixels
    let pixel_count = (w * h) as f64;
    let total_diff: f64 = buf_a
        .data
        .iter()
        .zip(buf_b.data.iter())
        .map(|(&a, &b)| (a as f64 - b as f64).abs())
        .sum();
    let mean_diff = total_diff / (pixel_count * 4.0); // 4 channels

    // After a 90° orbit the lighting and visible normals change significantly.
    // We expect a mean per-channel difference of at least 1.0 out of 255.
    assert!(
        mean_diff > 1.0,
        "after 90° orbit the render should differ noticeably from the original: mean_diff={mean_diff:.3}"
    );
}
