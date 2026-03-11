// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Winit event loop integration for the OxiHuman viewer.
//!
//! Provides [`ViewerEventLoop`], [`WindowState`], and [`InputState`] for
//! windowed rendering, orbit-camera mouse handling, and 60 fps frame timing.
//!
//! Gated behind the `winit` Cargo feature.

#[cfg(feature = "winit")]
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::EventLoop,
    keyboard::KeyCode,
    window::{Window, WindowAttributes, WindowId},
};

use std::time::{Duration, Instant};

use crate::camera::CameraState;

// ── Constants ─────────────────────────────────────────────────────────────────

/// Target frame period for 60 fps.
const TARGET_FRAME_DURATION: Duration = Duration::from_nanos(16_666_667);

/// Mouse sensitivity for orbit rotation (radians per pixel).
const ORBIT_ROTATE_SENSITIVITY: f32 = 0.005;

/// Mouse sensitivity for pan (world units per pixel).
const ORBIT_PAN_SENSITIVITY: f32 = 0.002;

/// Mouse scroll sensitivity for zoom (world units per tick).
const SCROLL_ZOOM_SENSITIVITY: f32 = 0.3;

// ── InputState ────────────────────────────────────────────────────────────────

/// Tracks keyboard and mouse input for a single frame.
#[derive(Debug, Clone, Default)]
pub struct InputState {
    /// Whether the left mouse button is currently held.
    pub left_button_down: bool,
    /// Whether the right mouse button is currently held.
    pub right_button_down: bool,
    /// Current cursor position in physical pixels, if known.
    pub cursor_position: Option<[f64; 2]>,
    /// Cursor position from the previous frame (for delta computation).
    pub prev_cursor_position: Option<[f64; 2]>,
    /// Accumulated scroll delta this frame (positive = zoom in).
    pub scroll_delta: f32,
    /// Whether the Shift modifier key is held.
    pub shift_held: bool,
    /// Whether the Ctrl modifier key is held.
    pub ctrl_held: bool,
    /// Whether the Alt modifier key is held.
    pub alt_held: bool,
    /// Set of logical key codes pressed this frame (winit feature only).
    #[cfg(feature = "winit")]
    pub keys_pressed: Vec<KeyCode>,
    /// Placeholder for non-winit builds (always empty).
    #[cfg(not(feature = "winit"))]
    pub keys_pressed: Vec<String>,
}

impl InputState {
    /// Compute the cursor delta since the previous frame.
    ///
    /// Returns `[dx, dy]` in physical pixels, or `[0.0, 0.0]` if unavailable.
    pub fn cursor_delta(&self) -> [f64; 2] {
        match (self.cursor_position, self.prev_cursor_position) {
            (Some(cur), Some(prev)) => [cur[0] - prev[0], cur[1] - prev[1]],
            _ => [0.0, 0.0],
        }
    }

    /// Advance frame — move current cursor to previous, reset per-frame fields.
    pub fn advance_frame(&mut self) {
        self.prev_cursor_position = self.cursor_position;
        self.scroll_delta = 0.0;
        self.keys_pressed.clear();
    }

    /// Returns `true` if any mouse button is held.
    pub fn any_button_down(&self) -> bool {
        self.left_button_down || self.right_button_down
    }
}

// ── WindowState ───────────────────────────────────────────────────────────────

/// CPU-side window state (surface, swapchain metadata).
///
/// The actual GPU surface/swapchain lives in the `webgpu` layer; this struct
/// tracks the associated dimensions and title so the rest of the loop can
/// query them without holding GPU handles.
#[derive(Debug, Clone)]
pub struct WindowState {
    /// Logical width in physical pixels.
    pub width: u32,
    /// Logical height in physical pixels.
    pub height: u32,
    /// Window title.
    pub title: String,
    /// Whether the window is currently focused.
    pub focused: bool,
    /// Whether the window was just resized this frame.
    pub resized: bool,
    /// Device-pixel ratio (HiDPI scale factor).
    pub scale_factor: f64,
}

impl WindowState {
    /// Construct a new [`WindowState`] with the given dimensions and title.
    pub fn new(width: u32, height: u32, title: &str) -> Self {
        WindowState {
            width,
            height,
            title: title.to_string(),
            focused: false,
            resized: false,
            scale_factor: 1.0,
        }
    }

    /// Update dimensions after a resize event.
    pub fn handle_resize(&mut self, new_width: u32, new_height: u32) {
        self.width = new_width.max(1);
        self.height = new_height.max(1);
        self.resized = true;
    }

    /// Clear the per-frame `resized` flag at the start of a frame.
    pub fn clear_frame_flags(&mut self) {
        self.resized = false;
    }

    /// Aspect ratio (width / height).
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height.max(1) as f32
    }
}

impl Default for WindowState {
    fn default() -> Self {
        WindowState::new(1280, 720, "OxiHuman Viewer")
    }
}

// ── FrameTiming ───────────────────────────────────────────────────────────────

/// Tracks frame timing and computes `dt` each frame.
#[derive(Debug, Clone)]
pub struct FrameTiming {
    last_frame_start: Instant,
    /// Delta time for the most recently completed frame, in seconds.
    pub dt_seconds: f32,
    /// Total elapsed seconds since the loop started.
    pub elapsed_seconds: f64,
    loop_start: Instant,
}

impl FrameTiming {
    /// Create a new [`FrameTiming`] anchored to the current instant.
    pub fn new() -> Self {
        let now = Instant::now();
        FrameTiming {
            last_frame_start: now,
            dt_seconds: 0.0,
            elapsed_seconds: 0.0,
            loop_start: now,
        }
    }

    /// Record the start of a new frame and compute `dt`.
    pub fn begin_frame(&mut self) {
        let now = Instant::now();
        self.dt_seconds = now
            .duration_since(self.last_frame_start)
            .as_secs_f32()
            .clamp(0.0, 0.1); // clamp to avoid spiral of death after pauses
        self.elapsed_seconds = now.duration_since(self.loop_start).as_secs_f64();
        self.last_frame_start = now;
    }

    /// Returns the instant the current frame started.
    pub fn frame_start(&self) -> Instant {
        self.last_frame_start
    }

    /// Returns the remaining time until the next 60 fps deadline, if any.
    ///
    /// Can be used to sleep or yield to the OS.
    pub fn remaining_frame_budget(&self) -> Option<Duration> {
        let elapsed = self.last_frame_start.elapsed();
        TARGET_FRAME_DURATION.checked_sub(elapsed)
    }
}

impl Default for FrameTiming {
    fn default() -> Self {
        FrameTiming::new()
    }
}

// ── OrbitCameraController ─────────────────────────────────────────────────────

/// Applies orbit-camera updates from input deltas to a [`CameraState`].
///
/// - Left mouse drag  → rotate (yaw + pitch)
/// - Right mouse drag → pan (translate target in the view plane)
/// - Scroll wheel     → zoom (move along the look-at vector)
pub struct OrbitCameraController {
    /// Rotation sensitivity multiplier (radians per pixel).
    pub rotate_sensitivity: f32,
    /// Pan sensitivity multiplier (world units per pixel).
    pub pan_sensitivity: f32,
    /// Zoom sensitivity multiplier (world units per scroll tick).
    pub zoom_sensitivity: f32,
}

impl Default for OrbitCameraController {
    fn default() -> Self {
        OrbitCameraController {
            rotate_sensitivity: ORBIT_ROTATE_SENSITIVITY,
            pan_sensitivity: ORBIT_PAN_SENSITIVITY,
            zoom_sensitivity: SCROLL_ZOOM_SENSITIVITY,
        }
    }
}

impl OrbitCameraController {
    /// Apply mouse-drag and scroll deltas to `camera`.
    ///
    /// - `dx`, `dy` are cursor pixel deltas.
    /// - `scroll` is the scroll wheel delta (positive = zoom in).
    /// - `left_down` / `right_down` indicate which button is held.
    pub fn apply(
        &self,
        camera: &mut CameraState,
        dx: f64,
        dy: f64,
        scroll: f32,
        left_down: bool,
        right_down: bool,
    ) {
        // Left drag → orbit rotation
        if left_down && (dx.abs() > f64::EPSILON || dy.abs() > f64::EPSILON) {
            let yaw_deg = (dx as f32) * self.rotate_sensitivity.to_degrees();
            let pitch_deg = (dy as f32) * self.rotate_sensitivity.to_degrees();
            camera.orbit(yaw_deg, pitch_deg);
        }

        // Right drag → pan
        if right_down && (dx.abs() > f64::EPSILON || dy.abs() > f64::EPSILON) {
            let pan_x = -(dx as f32) * self.pan_sensitivity;
            let pan_y = (dy as f32) * self.pan_sensitivity;
            apply_pan(camera, pan_x, pan_y);
        }

        // Scroll → zoom
        if scroll.abs() > f32::EPSILON {
            camera.zoom(scroll * self.zoom_sensitivity);
        }
    }
}

/// Translate the camera target in the view plane by `(pan_x, pan_y)`.
fn apply_pan(camera: &mut CameraState, pan_x: f32, pan_y: f32) {
    use crate::camera::{add3, cross3, normalize3, scale3, sub3};

    let fwd = normalize3(sub3(camera.target, camera.position));
    let right = normalize3(cross3(fwd, camera.up));
    let up = cross3(right, fwd);

    let delta = add3(scale3(right, pan_x), scale3(up, pan_y));
    camera.position = add3(camera.position, delta);
    camera.target = add3(camera.target, delta);
}

// ── ViewerEventLoop ───────────────────────────────────────────────────────────

/// High-level winit event loop integration.
///
/// Wraps a winit [`EventLoop`] and drives frame updates, input handling, and
/// camera orbit.
///
/// Enabled by the `winit` Cargo feature.
#[cfg(feature = "winit")]
pub struct ViewerEventLoop {
    /// The underlying winit event loop.
    pub event_loop: EventLoop<()>,
    /// The winit window.
    pub window: Window,
}

#[cfg(feature = "winit")]
impl ViewerEventLoop {
    /// Create a new [`ViewerEventLoop`] with the given title and dimensions.
    ///
    /// # Errors
    ///
    /// Returns an [`anyhow::Error`] if the event loop or window cannot be
    /// created (e.g., no display server available in headless environments).
    pub fn new(title: &str, width: u32, height: u32) -> anyhow::Result<Self> {
        let event_loop = EventLoop::new().map_err(|e| anyhow::anyhow!("EventLoop: {e}"))?;
        let attrs = WindowAttributes::default()
            .with_title(title)
            .with_inner_size(winit::dpi::LogicalSize::new(width, height));
        #[allow(deprecated)]
        let window = event_loop
            .create_window(attrs)
            .map_err(|e| anyhow::anyhow!("Window: {e}"))?;
        Ok(ViewerEventLoop { event_loop, window })
    }
}

/// Process a single [`WindowEvent`] and update [`WindowState`] + [`InputState`].
///
/// Returns `true` if the window should close.
///
/// This is a free function so it can be called from the `winit`-gated loop
/// without tying the entire module to the `winit` feature.
#[cfg(feature = "winit")]
pub fn process_window_event(
    event: &WindowEvent,
    win: &mut WindowState,
    input: &mut InputState,
) -> bool {
    match event {
        WindowEvent::CloseRequested => return true,

        WindowEvent::Resized(size) => {
            win.handle_resize(size.width, size.height);
        }

        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
            win.scale_factor = *scale_factor;
        }

        WindowEvent::Focused(focused) => {
            win.focused = *focused;
        }

        WindowEvent::CursorMoved { position, .. } => {
            input.cursor_position = Some([position.x, position.y]);
        }

        WindowEvent::MouseWheel { delta, .. } => {
            let lines = match delta {
                MouseScrollDelta::LineDelta(_, y) => *y,
                MouseScrollDelta::PixelDelta(PhysicalPosition { y, .. }) => *y as f32 / 30.0,
            };
            input.scroll_delta += lines;
        }

        WindowEvent::MouseInput { state, button, .. } => match button {
            MouseButton::Left => {
                input.left_button_down = *state == ElementState::Pressed;
            }
            MouseButton::Right => {
                input.right_button_down = *state == ElementState::Pressed;
            }
            _ => {}
        },

        WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    physical_key: winit::keyboard::PhysicalKey::Code(code),
                    state: ElementState::Pressed,
                    ..
                },
            ..
        } => {
            input.keys_pressed.push(*code);
        }

        WindowEvent::ModifiersChanged(mods) => {
            let state = mods.state();
            input.shift_held = state.shift_key();
            input.ctrl_held = state.control_key();
            input.alt_held = state.alt_key();
        }

        _ => {}
    }
    false
}

// ── ApplicationHandler impl for the viewer ─────────────────────────────────

/// Internal app state that implements the winit 0.30 [`ApplicationHandler`].
#[cfg(feature = "winit")]
struct ViewerApp<F>
where
    F: FnMut(&CameraState, &WindowState, &FrameTiming),
{
    window: Option<Window>,
    win_state: WindowState,
    input: InputState,
    timing: FrameTiming,
    camera: CameraState,
    orbit: OrbitCameraController,
    frame_callback: F,
    init_title: String,
    init_width: u32,
    init_height: u32,
}

#[cfg(feature = "winit")]
impl<F> winit::application::ApplicationHandler for ViewerApp<F>
where
    F: FnMut(&CameraState, &WindowState, &FrameTiming),
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = WindowAttributes::default()
                .with_title(self.init_title.clone())
                .with_inner_size(winit::dpi::LogicalSize::new(
                    self.init_width,
                    self.init_height,
                ));
            match event_loop.create_window(attrs) {
                Ok(w) => {
                    let sz = w.inner_size();
                    self.win_state = WindowState::new(sz.width, sz.height, &self.init_title);
                    self.window = Some(w);
                }
                Err(e) => {
                    eprintln!("OxiHuman Viewer: failed to create window: {e}");
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let should_close = process_window_event(&event, &mut self.win_state, &mut self.input);
        if should_close {
            event_loop.exit();
            return;
        }

        if let WindowEvent::RedrawRequested = event {
            self.timing.begin_frame();
            self.win_state.clear_frame_flags();

            let [dx, dy] = self.input.cursor_delta();
            self.orbit.apply(
                &mut self.camera,
                dx,
                dy,
                self.input.scroll_delta,
                self.input.left_button_down,
                self.input.right_button_down,
            );

            (self.frame_callback)(&self.camera, &self.win_state, &self.timing);

            self.input.advance_frame();
            if let Some(w) = &self.window {
                w.request_redraw();
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}

/// Run the viewer main loop using the winit 0.30 `ApplicationHandler` API.
///
/// This function does **not** return.  A [`CameraState`] and a mutable frame
/// callback are provided so callers can inject rendering logic without coupling
/// to the concrete GPU backend.
///
/// This is gated behind `#[cfg(feature = "winit")]`.
#[cfg(feature = "winit")]
pub fn run<F>(viewer_loop: ViewerEventLoop, camera: CameraState, frame_callback: F) -> !
where
    F: FnMut(&CameraState, &WindowState, &FrameTiming) + 'static,
{
    let ViewerEventLoop {
        event_loop, window, ..
    } = viewer_loop;

    let inner = window.inner_size();
    let title = window.title();
    let win_state = WindowState::new(inner.width, inner.height, &title);

    let mut app = ViewerApp {
        window: Some(window),
        win_state,
        input: InputState::default(),
        timing: FrameTiming::new(),
        camera,
        orbit: OrbitCameraController::default(),
        frame_callback,
        init_title: title,
        init_width: inner.width,
        init_height: inner.height,
    };

    match event_loop.run_app(&mut app) {
        Ok(()) => std::process::exit(0),
        Err(e) => panic!("Event loop exited with error: {e}"),
    }
}

// ── non-winit stubs ───────────────────────────────────────────────────────────

/// Headless stub: create a default [`WindowState`] without an OS window.
///
/// Available on all platforms regardless of the `winit` feature.
pub fn headless_window_state(width: u32, height: u32) -> WindowState {
    WindowState::new(width, height, "OxiHuman Headless")
}

/// Simulate one headless frame, updating `timing` and applying orbit to `camera`.
///
/// Useful in tests and CI where no display server is available.
pub fn tick_headless(
    camera: &mut CameraState,
    win: &mut WindowState,
    input: &mut InputState,
    timing: &mut FrameTiming,
) {
    timing.begin_frame();
    win.clear_frame_flags();

    let orbit = OrbitCameraController::default();
    let [dx, dy] = input.cursor_delta();
    orbit.apply(
        camera,
        dx,
        dy,
        input.scroll_delta,
        input.left_button_down,
        input.right_button_down,
    );

    input.advance_frame();
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_state_resize_clamps_to_one() {
        let mut ws = WindowState::default();
        ws.handle_resize(0, 0);
        assert_eq!(ws.width, 1);
        assert_eq!(ws.height, 1);
        assert!(ws.resized);
    }

    #[test]
    fn window_state_clear_flags() {
        let mut ws = WindowState::default();
        ws.handle_resize(100, 100);
        assert!(ws.resized);
        ws.clear_frame_flags();
        assert!(!ws.resized);
    }

    #[test]
    fn window_state_aspect_ratio() {
        let ws = WindowState::new(1280, 720, "test");
        let ar = ws.aspect_ratio();
        assert!((ar - 16.0 / 9.0).abs() < 1e-4, "expected 16:9, got {ar}");
    }

    #[test]
    fn input_state_cursor_delta_none_when_no_prev() {
        let input = InputState::default();
        assert_eq!(input.cursor_delta(), [0.0, 0.0]);
    }

    #[test]
    #[allow(clippy::field_reassign_with_default)]
    fn input_state_cursor_delta_computed() {
        let mut input = InputState::default();
        input.prev_cursor_position = Some([100.0, 200.0]);
        input.cursor_position = Some([110.0, 190.0]);
        let [dx, dy] = input.cursor_delta();
        assert!((dx - 10.0).abs() < 1e-6);
        assert!((dy + 10.0).abs() < 1e-6);
    }

    #[test]
    #[allow(clippy::field_reassign_with_default)]
    fn input_state_advance_frame_resets_scroll() {
        let mut input = InputState::default();
        input.scroll_delta = 3.0;
        input.advance_frame();
        assert_eq!(input.scroll_delta, 0.0);
    }

    #[test]
    fn input_state_advance_frame_clears_keys() {
        let mut input = InputState::default();
        #[cfg(feature = "winit")]
        input.keys_pressed.push(winit::keyboard::KeyCode::KeyA);
        #[cfg(not(feature = "winit"))]
        input.keys_pressed.push("KeyA".to_string());
        input.advance_frame();
        assert!(input.keys_pressed.is_empty());
    }

    #[test]
    fn frame_timing_dt_non_negative() {
        let mut timing = FrameTiming::new();
        timing.begin_frame();
        assert!(timing.dt_seconds >= 0.0);
    }

    #[test]
    fn orbit_controller_left_drag_changes_camera() {
        let mut cam = CameraState::default();
        let before = cam.position;
        let ctrl = OrbitCameraController::default();
        ctrl.apply(&mut cam, 50.0, 0.0, 0.0, true, false);
        assert_ne!(cam.position, before, "left drag should orbit camera");
    }

    #[test]
    fn orbit_controller_scroll_zooms() {
        let mut cam = CameraState::default();
        use crate::camera::{len3, sub3};
        let before_dist = len3(sub3(cam.position, cam.target));
        let ctrl = OrbitCameraController::default();
        ctrl.apply(&mut cam, 0.0, 0.0, 2.0, false, false);
        let after_dist = len3(sub3(cam.position, cam.target));
        assert!(
            after_dist < before_dist,
            "positive scroll should zoom in (decrease distance)"
        );
    }

    #[test]
    fn orbit_controller_right_drag_pans() {
        let mut cam = CameraState::default();
        let before_target = cam.target;
        let ctrl = OrbitCameraController::default();
        ctrl.apply(&mut cam, 100.0, 0.0, 0.0, false, true);
        assert_ne!(cam.target, before_target, "right drag should pan target");
    }

    #[test]
    fn headless_window_state_dimensions() {
        let ws = headless_window_state(800, 600);
        assert_eq!(ws.width, 800);
        assert_eq!(ws.height, 600);
    }

    #[test]
    fn tick_headless_updates_timing() {
        let mut cam = CameraState::default();
        let mut win = WindowState::default();
        let mut input = InputState::default();
        let mut timing = FrameTiming::new();
        tick_headless(&mut cam, &mut win, &mut input, &mut timing);
        assert!(timing.dt_seconds >= 0.0);
    }
}
