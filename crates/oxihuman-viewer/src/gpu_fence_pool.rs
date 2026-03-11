#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct GpuFence { id: u64, signaled: bool }

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GpuFencePool {
    fences: Vec<GpuFence>,
    next_id: u64,
    capacity: usize,
}

#[allow(dead_code)]
pub fn new_fence_pool(capacity: usize) -> GpuFencePool {
    GpuFencePool { fences: Vec::new(), next_id: 0, capacity }
}

#[allow(dead_code)]
pub fn acquire_fence(pool: &mut GpuFencePool) -> u64 {
    if pool.fences.len() >= pool.capacity { return u64::MAX; }
    let id = pool.next_id;
    pool.next_id += 1;
    pool.fences.push(GpuFence { id, signaled: false });
    id
}

#[allow(dead_code)]
pub fn release_fence(pool: &mut GpuFencePool, id: u64) {
    pool.fences.retain(|f| f.id != id);
}

#[allow(dead_code)]
pub fn fence_count_gfp(pool: &GpuFencePool) -> usize { pool.fences.len() }

#[allow(dead_code)]
pub fn fence_is_signaled_gfp(pool: &GpuFencePool, id: u64) -> bool {
    pool.fences.iter().find(|f| f.id == id).map(|f| f.signaled).unwrap_or(false)
}

#[allow(dead_code)]
pub fn fence_pool_to_json(pool: &GpuFencePool) -> String {
    format!("{{\"count\":{},\"capacity\":{}}}", pool.fences.len(), pool.capacity)
}

#[allow(dead_code)]
pub fn clear_fence_pool(pool: &mut GpuFencePool) { pool.fences.clear(); }

#[allow(dead_code)]
pub fn fence_pool_capacity(pool: &GpuFencePool) -> usize { pool.capacity }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let p = new_fence_pool(8); assert_eq!(fence_count_gfp(&p), 0); }
    #[test] fn test_acquire() { let mut p = new_fence_pool(8); let id = acquire_fence(&mut p); assert_eq!(id, 0); }
    #[test] fn test_count() { let mut p = new_fence_pool(8); acquire_fence(&mut p); assert_eq!(fence_count_gfp(&p), 1); }
    #[test] fn test_release() { let mut p = new_fence_pool(8); let id = acquire_fence(&mut p); release_fence(&mut p, id); assert_eq!(fence_count_gfp(&p), 0); }
    #[test] fn test_signaled() { let mut p = new_fence_pool(8); let id = acquire_fence(&mut p); assert!(!fence_is_signaled_gfp(&p, id)); }
    #[test] fn test_json() { let p = new_fence_pool(4); assert!(fence_pool_to_json(&p).contains("capacity")); }
    #[test] fn test_clear() { let mut p = new_fence_pool(8); acquire_fence(&mut p); clear_fence_pool(&mut p); assert_eq!(fence_count_gfp(&p), 0); }
    #[test] fn test_capacity() { let p = new_fence_pool(16); assert_eq!(fence_pool_capacity(&p), 16); }
    #[test] fn test_full() { let mut p = new_fence_pool(1); acquire_fence(&mut p); let id = acquire_fence(&mut p); assert_eq!(id, u64::MAX); }
    #[test] fn test_signaled_missing() { let p = new_fence_pool(8); assert!(!fence_is_signaled_gfp(&p, 999)); }
}
