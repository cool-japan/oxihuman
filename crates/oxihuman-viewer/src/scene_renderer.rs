#![allow(dead_code)]
//! Scene rendering coordinator with draw call tracking.

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct RenderConfig {
    pub width: u32,
    pub height: u32,
    pub msaa_samples: u32,
    pub vsync: bool,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct SceneRenderer {
    pub name: String,
    config: RenderConfig,
    draw_count: u32,
    vertex_count: u64,
}

#[allow(dead_code)]
pub fn new_scene_renderer(name: &str, config: RenderConfig) -> SceneRenderer {
    SceneRenderer {
        name: name.to_string(),
        config,
        draw_count: 0,
        vertex_count: 0,
    }
}

#[allow(dead_code)]
pub fn default_render_config() -> RenderConfig {
    RenderConfig {
        width: 1920,
        height: 1080,
        msaa_samples: 4,
        vsync: true,
    }
}

#[allow(dead_code)]
pub fn render_scene_stub(r: &mut SceneRenderer, draws: u32, verts: u64) {
    r.draw_count += draws;
    r.vertex_count += verts;
}

#[allow(dead_code)]
pub fn submitted_draw_count(r: &SceneRenderer) -> u32 {
    r.draw_count
}

#[allow(dead_code)]
pub fn rendered_vertex_count(r: &SceneRenderer) -> u64 {
    r.vertex_count
}

#[allow(dead_code)]
pub fn scene_renderer_stats(r: &SceneRenderer) -> String {
    format!(
        "draws={}, verts={}, {}x{}",
        r.draw_count, r.vertex_count, r.config.width, r.config.height
    )
}

#[allow(dead_code)]
pub fn reset_scene_renderer(r: &mut SceneRenderer) {
    r.draw_count = 0;
    r.vertex_count = 0;
}

#[allow(dead_code)]
pub fn renderer_name(r: &SceneRenderer) -> &str {
    &r.name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_scene_renderer() {
        let r = new_scene_renderer("main", default_render_config());
        assert_eq!(renderer_name(&r), "main");
    }

    #[test]
    fn test_default_render_config() {
        let c = default_render_config();
        assert_eq!(c.width, 1920);
        assert_eq!(c.height, 1080);
    }

    #[test]
    fn test_render_scene_stub() {
        let mut r = new_scene_renderer("r", default_render_config());
        render_scene_stub(&mut r, 10, 1000);
        assert_eq!(submitted_draw_count(&r), 10);
        assert_eq!(rendered_vertex_count(&r), 1000);
    }

    #[test]
    fn test_accumulate_draws() {
        let mut r = new_scene_renderer("r", default_render_config());
        render_scene_stub(&mut r, 5, 100);
        render_scene_stub(&mut r, 3, 200);
        assert_eq!(submitted_draw_count(&r), 8);
        assert_eq!(rendered_vertex_count(&r), 300);
    }

    #[test]
    fn test_scene_renderer_stats() {
        let r = new_scene_renderer("r", default_render_config());
        let s = scene_renderer_stats(&r);
        assert!(s.contains("1920x1080"));
    }

    #[test]
    fn test_reset_scene_renderer() {
        let mut r = new_scene_renderer("r", default_render_config());
        render_scene_stub(&mut r, 10, 500);
        reset_scene_renderer(&mut r);
        assert_eq!(submitted_draw_count(&r), 0);
    }

    #[test]
    fn test_renderer_name() {
        let r = new_scene_renderer("test", default_render_config());
        assert_eq!(renderer_name(&r), "test");
    }

    #[test]
    fn test_initial_counts() {
        let r = new_scene_renderer("r", default_render_config());
        assert_eq!(submitted_draw_count(&r), 0);
        assert_eq!(rendered_vertex_count(&r), 0);
    }

    #[test]
    fn test_config_msaa() {
        let c = default_render_config();
        assert_eq!(c.msaa_samples, 4);
    }

    #[test]
    fn test_config_vsync() {
        let c = default_render_config();
        assert!(c.vsync);
    }
}
