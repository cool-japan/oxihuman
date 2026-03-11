//! XR/VR viewport configuration and eye view layout.

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum XrEye {
    Left,
    Right,
    Center,
}

#[allow(dead_code)]
pub struct XrEyeView {
    pub eye: XrEye,
    pub view_matrix: [[f32; 4]; 4],
    pub projection_matrix: [[f32; 4]; 4],
    pub viewport: [u32; 4], // x, y, width, height
    pub ipd_offset: f32,    // inter-pupillary distance offset
}

#[allow(dead_code)]
pub struct XrViewport {
    pub eyes: Vec<XrEyeView>,
    pub render_width: u32,
    pub render_height: u32,
    pub ipd: f32, // inter-pupillary distance in meters
    pub near: f32,
    pub far: f32,
    pub fov_deg: f32,
    pub enabled: bool,
}

#[allow(dead_code)]
pub struct XrPose {
    pub position: [f32; 3],
    pub orientation: [f32; 4], // quaternion
}

fn identity_mat4() -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

#[allow(dead_code)]
pub fn new_xr_viewport(width: u32, height: u32, ipd: f32) -> XrViewport {
    XrViewport {
        eyes: Vec::new(),
        render_width: width,
        render_height: height,
        ipd,
        near: 0.01,
        far: 1000.0,
        fov_deg: 90.0,
        enabled: false,
    }
}

#[allow(dead_code)]
pub fn build_eye_views(xr: &mut XrViewport) {
    let aspect = if xr.render_height > 0 {
        (xr.render_width as f32 / 2.0) / xr.render_height as f32
    } else {
        1.0
    };
    let proj = eye_projection_matrix(xr.fov_deg, aspect, xr.near, xr.far);

    let eyes = [XrEye::Left, XrEye::Right];
    xr.eyes.clear();
    let half_w = xr.render_width / 2;

    for (i, &eye) in eyes.iter().enumerate() {
        let offset = xr_ipd_offset(eye, xr.ipd);
        let mut view = identity_mat4();
        view[3][0] = -offset; // translate by IPD offset
        let x = (i as u32) * half_w;
        xr.eyes.push(XrEyeView {
            eye,
            view_matrix: view,
            projection_matrix: proj,
            viewport: [x, 0, half_w, xr.render_height],
            ipd_offset: offset,
        });
    }
}

#[allow(dead_code)]
pub fn eye_view_matrix(pose: &XrPose, eye: XrEye, ipd: f32) -> [[f32; 4]; 4] {
    let offset = xr_ipd_offset(eye, ipd);
    // Build a simple translation matrix using pose position + IPD offset
    let mut m = identity_mat4();
    m[3][0] = -(pose.position[0] + offset);
    m[3][1] = -pose.position[1];
    m[3][2] = -pose.position[2];
    m
}

#[allow(dead_code)]
pub fn eye_projection_matrix(fov_deg: f32, aspect: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
    let fov_rad = fov_deg.to_radians();
    let f = 1.0 / (fov_rad / 2.0).tan();
    let nf = 1.0 / (near - far);
    [
        [f / aspect, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, (far + near) * nf, -1.0],
        [0.0, 0.0, 2.0 * far * near * nf, 0.0],
    ]
}

#[allow(dead_code)]
pub fn xr_viewport_for_eye(xr: &XrViewport, eye: XrEye) -> Option<&XrEyeView> {
    xr.eyes.iter().find(|e| e.eye == eye)
}

#[allow(dead_code)]
pub fn set_xr_pose(xr: &mut XrViewport, pose: XrPose) {
    let ipd = xr.ipd;
    for eye_view in &mut xr.eyes {
        eye_view.view_matrix = eye_view_matrix(&pose, eye_view.eye, ipd);
    }
}

#[allow(dead_code)]
pub fn xr_reprojection_matrix(prev_pose: &XrPose, curr_pose: &XrPose) -> [[f32; 4]; 4] {
    // Simplified: return a translation from prev to curr
    let mut m = identity_mat4();
    m[3][0] = curr_pose.position[0] - prev_pose.position[0];
    m[3][1] = curr_pose.position[1] - prev_pose.position[1];
    m[3][2] = curr_pose.position[2] - prev_pose.position[2];
    m
}

#[allow(dead_code)]
pub fn eye_count(xr: &XrViewport) -> usize {
    xr.eyes.len()
}

#[allow(dead_code)]
pub fn xr_render_resolution(xr: &XrViewport) -> (u32, u32) {
    (xr.render_width, xr.render_height)
}

#[allow(dead_code)]
pub fn enable_xr(xr: &mut XrViewport) {
    xr.enabled = true;
}

#[allow(dead_code)]
pub fn disable_xr(xr: &mut XrViewport) {
    xr.enabled = false;
}

/// Returns ±ipd/2 for left/right, 0 for center.
#[allow(dead_code)]
pub fn xr_ipd_offset(eye: XrEye, ipd: f32) -> f32 {
    match eye {
        XrEye::Left => -ipd / 2.0,
        XrEye::Right => ipd / 2.0,
        XrEye::Center => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_xr_viewport() {
        let xr = new_xr_viewport(2560, 1440, 0.064);
        assert_eq!(xr.render_width, 2560);
        assert_eq!(xr.render_height, 1440);
        assert!((xr.ipd - 0.064).abs() < 1e-6);
        assert!(!xr.enabled);
        assert!(xr.eyes.is_empty());
    }

    #[test]
    fn test_build_eye_views_has_two_eyes() {
        let mut xr = new_xr_viewport(2560, 1440, 0.064);
        build_eye_views(&mut xr);
        assert_eq!(eye_count(&xr), 2);
    }

    #[test]
    fn test_build_eye_views_left_and_right() {
        let mut xr = new_xr_viewport(2560, 1440, 0.064);
        build_eye_views(&mut xr);
        assert!(xr_viewport_for_eye(&xr, XrEye::Left).is_some());
        assert!(xr_viewport_for_eye(&xr, XrEye::Right).is_some());
        assert!(xr_viewport_for_eye(&xr, XrEye::Center).is_none());
    }

    #[test]
    fn test_ipd_offset_left_negative() {
        let offset = xr_ipd_offset(XrEye::Left, 0.064);
        assert!(offset < 0.0);
        assert!((offset - (-0.032)).abs() < 1e-6);
    }

    #[test]
    fn test_ipd_offset_right_positive() {
        let offset = xr_ipd_offset(XrEye::Right, 0.064);
        assert!(offset > 0.0);
        assert!((offset - 0.032).abs() < 1e-6);
    }

    #[test]
    fn test_ipd_offset_center_zero() {
        let offset = xr_ipd_offset(XrEye::Center, 0.064);
        assert!((offset).abs() < 1e-6);
    }

    #[test]
    fn test_enable_xr() {
        let mut xr = new_xr_viewport(1920, 1080, 0.064);
        enable_xr(&mut xr);
        assert!(xr.enabled);
    }

    #[test]
    fn test_disable_xr() {
        let mut xr = new_xr_viewport(1920, 1080, 0.064);
        enable_xr(&mut xr);
        disable_xr(&mut xr);
        assert!(!xr.enabled);
    }

    #[test]
    fn test_eye_count_after_build() {
        let mut xr = new_xr_viewport(3840, 2160, 0.064);
        assert_eq!(eye_count(&xr), 0);
        build_eye_views(&mut xr);
        assert_eq!(eye_count(&xr), 2);
    }

    #[test]
    fn test_render_resolution() {
        let xr = new_xr_viewport(1920, 1080, 0.064);
        assert_eq!(xr_render_resolution(&xr), (1920, 1080));
    }

    #[test]
    fn test_projection_matrix_non_identity() {
        let proj = eye_projection_matrix(90.0, 2.0, 0.01, 1000.0);
        // With aspect=2.0, proj[0][0] = f/aspect = 0.5, not 1.0
        assert!((proj[0][0] - 1.0).abs() > 0.01);
        // proj[2][3] should be -1.0 (perspective divide)
        assert!((proj[2][3] - (-1.0)).abs() < 1e-5);
    }

    #[test]
    fn test_set_xr_pose() {
        let mut xr = new_xr_viewport(2560, 1440, 0.064);
        build_eye_views(&mut xr);
        let pose = XrPose {
            position: [1.0, 2.0, 3.0],
            orientation: [0.0, 0.0, 0.0, 1.0],
        };
        set_xr_pose(&mut xr, pose);
        // Matrices should be updated (not identity translation)
        let left = xr_viewport_for_eye(&xr, XrEye::Left).unwrap();
        // The view matrix should reflect the pose
        assert!((left.view_matrix[3][1] - (-2.0)).abs() < 1e-5);
    }

    #[test]
    fn test_reprojection_matrix() {
        let prev = XrPose {
            position: [0.0, 0.0, 0.0],
            orientation: [0.0, 0.0, 0.0, 1.0],
        };
        let curr = XrPose {
            position: [1.0, 0.5, -0.5],
            orientation: [0.0, 0.0, 0.0, 1.0],
        };
        let m = xr_reprojection_matrix(&prev, &curr);
        assert!((m[3][0] - 1.0).abs() < 1e-5);
        assert!((m[3][1] - 0.5).abs() < 1e-5);
        assert!((m[3][2] - (-0.5)).abs() < 1e-5);
    }
}
