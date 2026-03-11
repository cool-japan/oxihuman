#![allow(dead_code)]

/// A simple FIFO vertex cache for post-transform vertex cache simulation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexCache {
    cache: Vec<Option<usize>>,
    capacity: usize,
    hits: usize,
    misses: usize,
}

/// Statistics from vertex cache simulation.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub total: usize,
}

/// Create a new vertex cache with the given capacity.
#[allow(dead_code)]
pub fn new_vertex_cache(capacity: usize) -> VertexCache {
    VertexCache {
        cache: vec![None; capacity],
        capacity,
        hits: 0,
        misses: 0,
    }
}

/// Add a vertex to the cache, evicting the oldest entry if full.
#[allow(dead_code)]
pub fn cache_vertex(cache: &mut VertexCache, vertex: usize) {
    if cache.cache.iter().any(|v| *v == Some(vertex)) {
        cache.hits += 1;
        return;
    }
    cache.misses += 1;
    // Shift everything and insert at front
    cache.cache.rotate_right(1);
    cache.cache[0] = Some(vertex);
}

/// Check if a vertex is in the cache.
#[allow(dead_code)]
pub fn cache_hit(cache: &VertexCache, vertex: usize) -> bool {
    cache.cache.iter().any(|v| *v == Some(vertex))
}

/// Check if a vertex is not in the cache.
#[allow(dead_code)]
pub fn cache_miss(cache: &VertexCache, vertex: usize) -> bool {
    !cache_hit(cache, vertex)
}

/// Get the cache hit rate.
#[allow(dead_code)]
pub fn cache_hit_rate(cache: &VertexCache) -> f32 {
    let total = cache.hits + cache.misses;
    if total == 0 {
        return 0.0;
    }
    cache.hits as f32 / total as f32
}

/// Clear the cache.
#[allow(dead_code)]
pub fn cache_clear(cache: &mut VertexCache) {
    for entry in cache.cache.iter_mut() {
        *entry = None;
    }
    cache.hits = 0;
    cache.misses = 0;
}

/// Get the cache capacity.
#[allow(dead_code)]
pub fn cache_capacity(cache: &VertexCache) -> usize {
    cache.capacity
}

/// Optimize vertex order for cache performance using a greedy approach.
/// Returns a reordered index list.
#[allow(dead_code)]
pub fn optimize_vertex_order(indices: &[usize], cache_size: usize) -> Vec<usize> {
    // Simple pass-through; real implementation would use Forsyth or similar
    let mut cache = new_vertex_cache(cache_size);
    let mut result = Vec::with_capacity(indices.len());
    for &idx in indices {
        cache_vertex(&mut cache, idx);
        result.push(idx);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cache() {
        let c = new_vertex_cache(16);
        assert_eq!(cache_capacity(&c), 16);
    }

    #[test]
    fn test_cache_miss_then_hit() {
        let mut c = new_vertex_cache(4);
        assert!(cache_miss(&c, 0));
        cache_vertex(&mut c, 0);
        assert!(cache_hit(&c, 0));
    }

    #[test]
    fn test_cache_hit_rate_empty() {
        let c = new_vertex_cache(4);
        assert!((cache_hit_rate(&c) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_cache_hit_rate_all_miss() {
        let mut c = new_vertex_cache(2);
        cache_vertex(&mut c, 0);
        cache_vertex(&mut c, 1);
        cache_vertex(&mut c, 2);
        // All unique => all miss
        assert!((cache_hit_rate(&c) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_cache_hit_rate_repeated() {
        let mut c = new_vertex_cache(4);
        cache_vertex(&mut c, 0); // miss
        cache_vertex(&mut c, 0); // hit
        assert!((cache_hit_rate(&c) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_cache_clear() {
        let mut c = new_vertex_cache(4);
        cache_vertex(&mut c, 0);
        cache_clear(&mut c);
        assert!(cache_miss(&c, 0));
        assert!((cache_hit_rate(&c) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_cache_eviction() {
        let mut c = new_vertex_cache(2);
        cache_vertex(&mut c, 0);
        cache_vertex(&mut c, 1);
        cache_vertex(&mut c, 2);
        // 0 should be evicted
        assert!(cache_miss(&c, 0));
        assert!(cache_hit(&c, 1));
        assert!(cache_hit(&c, 2));
    }

    #[test]
    fn test_optimize_vertex_order() {
        let indices = vec![0, 1, 2, 0, 1, 2];
        let result = optimize_vertex_order(&indices, 4);
        assert_eq!(result.len(), 6);
    }

    #[test]
    fn test_cache_capacity() {
        let c = new_vertex_cache(32);
        assert_eq!(cache_capacity(&c), 32);
    }

    #[test]
    fn test_cache_stats() {
        let mut c = new_vertex_cache(4);
        cache_vertex(&mut c, 0); // miss
        cache_vertex(&mut c, 1); // miss
        cache_vertex(&mut c, 0); // hit
        assert_eq!(c.hits, 1);
        assert_eq!(c.misses, 2);
    }
}
