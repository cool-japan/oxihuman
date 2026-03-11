//! World space axes overlay (XYZ gizmo in viewport corner).

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct AxisLine {
    pub axis: u8,
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub color: [f32; 4],
    pub negative_color: [f32; 4],
}

#[allow(dead_code)]
pub struct WorldAxes {
    pub lines: Vec<AxisLine>,
    pub labels: [String; 3],
    pub size: f32,
    pub origin: [f32; 3],
    pub visible: bool,
    pub show_negative: bool,
}

#[allow(dead_code)]
pub fn new_world_axes(size: f32) -> WorldAxes {
    let mut axes = WorldAxes {
        lines: Vec::new(),
        labels: ["X".to_string(), "Y".to_string(), "Z".to_string()],
        size,
        origin: [0.0, 0.0, 0.0],
        visible: true,
        show_negative: true,
    };
    axes.lines = build_axis_lines(&axes);
    axes
}

#[allow(dead_code)]
pub fn build_axis_lines(axes: &WorldAxes) -> Vec<AxisLine> {
    let mut lines = Vec::new();
    let o = axes.origin;
    let s = axes.size;

    // Positive axes
    for axis in 0u8..3 {
        let mut end = o;
        end[axis as usize] += s;
        let color = axis_color(axis, true);
        let neg_color = axis_color(axis, false);
        lines.push(AxisLine {
            axis,
            start: o,
            end,
            color,
            negative_color: neg_color,
        });
    }

    // Negative axes
    if axes.show_negative {
        for axis in 0u8..3 {
            let mut end = o;
            end[axis as usize] -= s;
            let color = axis_color(axis, false);
            let neg_color = axis_color(axis, false);
            lines.push(AxisLine {
                axis,
                start: o,
                end,
                color,
                negative_color: neg_color,
            });
        }
    }

    lines
}

#[allow(dead_code)]
pub fn update_axes_from_view(
    axes: &mut WorldAxes,
    _view_matrix: &[[f32; 4]; 4],
    corner_pos: [f32; 3],
    size: f32,
) {
    axes.origin = corner_pos;
    axes.size = size;
    axes.lines = build_axis_lines(axes);
}

#[allow(dead_code)]
pub fn axis_color(axis: u8, positive: bool) -> [f32; 4] {
    let bright = if positive { 1.0f32 } else { 0.4f32 };
    match axis {
        0 => [bright, 0.0, 0.0, 1.0], // X = red
        1 => [0.0, bright, 0.0, 1.0], // Y = green
        2 => [0.0, 0.0, bright, 1.0], // Z = blue
        _ => [bright, bright, bright, 1.0],
    }
}

/// Project to 2D returning (start_2d, end_2d, color) for each axis line.
#[allow(dead_code)]
pub fn axes_to_screen_lines(
    axes: &WorldAxes,
    view_proj: &[[f32; 4]; 4],
) -> Vec<([f32; 2], [f32; 2], [f32; 4])> {
    axes.lines
        .iter()
        .map(|line| {
            let s2d = world_to_screen_axes(line.start, view_proj);
            let e2d = world_to_screen_axes(line.end, view_proj);
            (s2d, e2d, line.color)
        })
        .collect()
}

#[allow(dead_code)]
pub fn set_axes_visible(axes: &mut WorldAxes, visible: bool) {
    axes.visible = visible;
}

#[allow(dead_code)]
pub fn set_show_negative(axes: &mut WorldAxes, show: bool) {
    axes.show_negative = show;
    axes.lines = build_axis_lines(axes);
}

#[allow(dead_code)]
pub fn axes_line_count(axes: &WorldAxes) -> usize {
    axes.lines.len()
}

#[allow(dead_code)]
pub fn axis_label(axis: u8) -> &'static str {
    match axis {
        0 => "X",
        1 => "Y",
        2 => "Z",
        _ => "?",
    }
}

/// Rotate all axis line endpoints by a quaternion [x, y, z, w].
#[allow(dead_code)]
pub fn rotate_axes(axes: &mut WorldAxes, quat: [f32; 4]) {
    let [qx, qy, qz, qw] = quat;
    let rotate_vec = |v: [f32; 3]| -> [f32; 3] {
        let [vx, vy, vz] = v;
        // Quaternion rotation: v' = q * v * q^-1
        let tx = 2.0 * (qy * vz - qz * vy);
        let ty = 2.0 * (qz * vx - qx * vz);
        let tz = 2.0 * (qx * vy - qy * vx);
        [
            vx + qw * tx + qy * tz - qz * ty,
            vy + qw * ty + qz * tx - qx * tz,
            vz + qw * tz + qx * ty - qy * tx,
        ]
    };
    for line in &mut axes.lines {
        line.start = rotate_vec(line.start);
        line.end = rotate_vec(line.end);
    }
}

#[allow(dead_code)]
pub fn scale_axes(axes: &mut WorldAxes, factor: f32) {
    axes.size *= factor;
    for line in &mut axes.lines {
        line.end[0] = line.start[0] + (line.end[0] - line.start[0]) * factor;
        line.end[1] = line.start[1] + (line.end[1] - line.start[1]) * factor;
        line.end[2] = line.start[2] + (line.end[2] - line.start[2]) * factor;
    }
}

/// Project a 3D world position to 2D screen coordinates using view-projection matrix.
#[allow(dead_code)]
pub fn world_to_screen_axes(world_pos: [f32; 3], view_proj: &[[f32; 4]; 4]) -> [f32; 2] {
    let [x, y, z] = world_pos;
    let clip_x = view_proj[0][0] * x + view_proj[0][1] * y + view_proj[0][2] * z + view_proj[0][3];
    let clip_y = view_proj[1][0] * x + view_proj[1][1] * y + view_proj[1][2] * z + view_proj[1][3];
    let clip_w = view_proj[3][0] * x + view_proj[3][1] * y + view_proj[3][2] * z + view_proj[3][3];
    if clip_w.abs() < 1e-8 {
        return [0.0, 0.0];
    }
    [(clip_x / clip_w + 1.0) * 0.5, (clip_y / clip_w + 1.0) * 0.5]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity_vp() -> [[f32; 4]; 4] {
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }

    #[test]
    fn test_new_world_axes() {
        let axes = new_world_axes(1.0);
        assert!((axes.size - 1.0).abs() < 1e-6);
        assert!(axes.visible);
        assert!(!axes.lines.is_empty());
    }

    #[test]
    fn test_axis_color_x_red() {
        let color = axis_color(0, true);
        assert!(color[0] > 0.5, "X axis should be red (high R)");
        assert!(color[1] < 0.1, "X axis red: G should be low");
        assert!(color[2] < 0.1, "X axis red: B should be low");
    }

    #[test]
    fn test_axis_color_y_green() {
        let color = axis_color(1, true);
        assert!(color[0] < 0.1);
        assert!(color[1] > 0.5);
        assert!(color[2] < 0.1);
    }

    #[test]
    fn test_axis_color_z_blue() {
        let color = axis_color(2, true);
        assert!(color[0] < 0.1);
        assert!(color[1] < 0.1);
        assert!(color[2] > 0.5);
    }

    #[test]
    fn test_axis_color_negative_darker() {
        let pos_color = axis_color(0, true);
        let neg_color = axis_color(0, false);
        assert!(pos_color[0] > neg_color[0], "Negative should be darker");
    }

    #[test]
    fn test_build_axis_lines_count_with_negative() {
        let axes = new_world_axes(1.0);
        // 3 positive + 3 negative
        assert_eq!(axes.lines.len(), 6);
    }

    #[test]
    fn test_build_axis_lines_count_no_negative() {
        let mut axes = new_world_axes(1.0);
        set_show_negative(&mut axes, false);
        // Only 3 positive
        assert_eq!(axes.lines.len(), 3);
    }

    #[test]
    fn test_axis_label() {
        assert_eq!(axis_label(0), "X");
        assert_eq!(axis_label(1), "Y");
        assert_eq!(axis_label(2), "Z");
    }

    #[test]
    fn test_set_axes_visible() {
        let mut axes = new_world_axes(1.0);
        assert!(axes.visible);
        set_axes_visible(&mut axes, false);
        assert!(!axes.visible);
    }

    #[test]
    fn test_set_show_negative_false() {
        let mut axes = new_world_axes(1.0);
        set_show_negative(&mut axes, false);
        assert!(!axes.show_negative);
        assert_eq!(axes_line_count(&axes), 3);
    }

    #[test]
    fn test_set_show_negative_true() {
        let mut axes = new_world_axes(1.0);
        set_show_negative(&mut axes, false);
        set_show_negative(&mut axes, true);
        assert_eq!(axes_line_count(&axes), 6);
    }

    #[test]
    fn test_scale_axes() {
        let mut axes = new_world_axes(1.0);
        scale_axes(&mut axes, 2.0);
        assert!((axes.size - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_world_to_screen_axes_identity() {
        let vp = identity_vp();
        let screen = world_to_screen_axes([0.0, 0.0, 0.0], &vp);
        assert!((screen[0] - 0.5).abs() < 1e-6, "x center should be 0.5");
        assert!((screen[1] - 0.5).abs() < 1e-6, "y center should be 0.5");
    }

    #[test]
    fn test_axes_to_screen_lines() {
        let axes = new_world_axes(1.0);
        let vp = identity_vp();
        let screen_lines = axes_to_screen_lines(&axes, &vp);
        assert_eq!(screen_lines.len(), axes.lines.len());
    }

    #[test]
    fn test_rotate_axes() {
        let mut axes = new_world_axes(1.0);
        // Identity quaternion [0,0,0,1] should not change positions
        let original_end = axes.lines[0].end;
        rotate_axes(&mut axes, [0.0, 0.0, 0.0, 1.0]);
        let new_end = axes.lines[0].end;
        for i in 0..3 {
            assert!(
                (original_end[i] - new_end[i]).abs() < 1e-5,
                "Identity quat rotation changed axis: orig={original_end:?} new={new_end:?}"
            );
        }
    }
}
