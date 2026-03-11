#![allow(dead_code)]

/// An object in the render pool.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct RenderObject { id: usize, name: String, visible: bool }

/// Pool of renderable objects.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderObjectPool { objects: Vec<RenderObject>, next_id: usize }

#[allow(dead_code)]
pub fn new_render_object_pool() -> RenderObjectPool { RenderObjectPool { objects: Vec::new(), next_id: 0 } }

#[allow(dead_code)]
pub fn pool_add_object(pool: &mut RenderObjectPool, name: &str) -> usize {
    let id = pool.next_id; pool.next_id += 1;
    pool.objects.push(RenderObject { id, name: name.to_string(), visible: true });
    id
}

#[allow(dead_code)]
pub fn pool_remove_object(pool: &mut RenderObjectPool, id: usize) -> bool {
    if let Some(pos) = pool.objects.iter().position(|o| o.id == id) { pool.objects.remove(pos); true } else { false }
}

#[allow(dead_code)]
pub fn pool_object_count(pool: &RenderObjectPool) -> usize { pool.objects.len() }

#[allow(dead_code)]
pub fn pool_get_object(pool: &RenderObjectPool, id: usize) -> Option<&str> {
    pool.objects.iter().find(|o| o.id == id).map(|o| o.name.as_str())
}

#[allow(dead_code)]
pub fn pool_visible_objects(pool: &RenderObjectPool) -> Vec<usize> {
    pool.objects.iter().filter(|o| o.visible).map(|o| o.id).collect()
}

#[allow(dead_code)]
pub fn pool_to_json(pool: &RenderObjectPool) -> String {
    format!("{{\"count\":{},\"visible\":{}}}", pool.objects.len(), pool_visible_objects(pool).len())
}

#[allow(dead_code)]
pub fn pool_clear_rop(pool: &mut RenderObjectPool) { pool.objects.clear(); }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { assert_eq!(pool_object_count(&new_render_object_pool()), 0); }
    #[test] fn test_add() {
        let mut p = new_render_object_pool();
        pool_add_object(&mut p, "mesh1");
        assert_eq!(pool_object_count(&p), 1);
    }
    #[test] fn test_remove() {
        let mut p = new_render_object_pool();
        let id = pool_add_object(&mut p, "a");
        assert!(pool_remove_object(&mut p, id));
        assert_eq!(pool_object_count(&p), 0);
    }
    #[test] fn test_remove_missing() { assert!(!pool_remove_object(&mut new_render_object_pool(), 99)); }
    #[test] fn test_get() {
        let mut p = new_render_object_pool();
        let id = pool_add_object(&mut p, "cube");
        assert_eq!(pool_get_object(&p, id), Some("cube"));
    }
    #[test] fn test_get_missing() { assert!(pool_get_object(&new_render_object_pool(), 0).is_none()); }
    #[test] fn test_visible() {
        let mut p = new_render_object_pool();
        pool_add_object(&mut p, "a"); pool_add_object(&mut p, "b");
        assert_eq!(pool_visible_objects(&p).len(), 2);
    }
    #[test] fn test_to_json() {
        let mut p = new_render_object_pool();
        pool_add_object(&mut p, "x");
        assert!(pool_to_json(&p).contains("count"));
    }
    #[test] fn test_clear() {
        let mut p = new_render_object_pool();
        pool_add_object(&mut p, "x");
        pool_clear_rop(&mut p);
        assert_eq!(pool_object_count(&p), 0);
    }
    #[test] fn test_unique_ids() {
        let mut p = new_render_object_pool();
        let a = pool_add_object(&mut p, "a");
        let b = pool_add_object(&mut p, "b");
        assert_ne!(a, b);
    }
}
