#![allow(dead_code)]

/// Collects render statistics across frames.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderStatsCollector {
    draw_calls: u64,
    vertices: u64,
    triangles: u64,
}

#[allow(dead_code)]
pub fn new_stats_collector() -> RenderStatsCollector {
    RenderStatsCollector { draw_calls: 0, vertices: 0, triangles: 0 }
}

#[allow(dead_code)]
pub fn record_draw_call(c: &mut RenderStatsCollector) { c.draw_calls += 1; }

#[allow(dead_code)]
pub fn record_vertices(c: &mut RenderStatsCollector, count: u64) { c.vertices += count; }

#[allow(dead_code)]
pub fn record_triangles(c: &mut RenderStatsCollector, count: u64) { c.triangles += count; }

#[allow(dead_code)]
pub fn draw_calls_total(c: &RenderStatsCollector) -> u64 { c.draw_calls }

#[allow(dead_code)]
pub fn vertices_total(c: &RenderStatsCollector) -> u64 { c.vertices }

#[allow(dead_code)]
pub fn triangles_total(c: &RenderStatsCollector) -> u64 { c.triangles }

#[allow(dead_code)]
pub fn stats_to_json_rsc(c: &RenderStatsCollector) -> String {
    format!("{{\"draw_calls\":{},\"vertices\":{},\"triangles\":{}}}", c.draw_calls, c.vertices, c.triangles)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let c = new_stats_collector(); assert_eq!(draw_calls_total(&c), 0); }
    #[test] fn test_draw_call() { let mut c = new_stats_collector(); record_draw_call(&mut c); assert_eq!(draw_calls_total(&c), 1); }
    #[test] fn test_vertices() { let mut c = new_stats_collector(); record_vertices(&mut c, 100); assert_eq!(vertices_total(&c), 100); }
    #[test] fn test_triangles() { let mut c = new_stats_collector(); record_triangles(&mut c, 50); assert_eq!(triangles_total(&c), 50); }
    #[test] fn test_accumulate() {
        let mut c = new_stats_collector();
        record_draw_call(&mut c); record_draw_call(&mut c);
        assert_eq!(draw_calls_total(&c), 2);
    }
    #[test] fn test_vertices_accumulate() {
        let mut c = new_stats_collector();
        record_vertices(&mut c, 100); record_vertices(&mut c, 200);
        assert_eq!(vertices_total(&c), 300);
    }
    #[test] fn test_to_json() {
        let mut c = new_stats_collector();
        record_draw_call(&mut c);
        assert!(stats_to_json_rsc(&c).contains("draw_calls"));
    }
    #[test] fn test_all_zero() {
        let c = new_stats_collector();
        assert_eq!(vertices_total(&c), 0);
        assert_eq!(triangles_total(&c), 0);
    }
    #[test] fn test_triangles_accumulate() {
        let mut c = new_stats_collector();
        record_triangles(&mut c, 10); record_triangles(&mut c, 20);
        assert_eq!(triangles_total(&c), 30);
    }
    #[test] fn test_combined() {
        let mut c = new_stats_collector();
        record_draw_call(&mut c); record_vertices(&mut c, 3); record_triangles(&mut c, 1);
        assert_eq!(draw_calls_total(&c), 1);
        assert_eq!(vertices_total(&c), 3);
        assert_eq!(triangles_total(&c), 1);
    }
}
