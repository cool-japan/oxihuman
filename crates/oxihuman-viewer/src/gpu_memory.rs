#![allow(dead_code)]

/// A memory block in GPU memory.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MemoryBlock { pub offset: usize, pub size: usize, pub label: String }

/// Simulated GPU memory allocator.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GpuMemory { total: usize, blocks: Vec<MemoryBlock> }

#[allow(dead_code)]
pub fn new_gpu_memory(total_bytes: usize) -> GpuMemory { GpuMemory { total: total_bytes, blocks: Vec::new() } }

#[allow(dead_code)]
pub fn allocate_block(mem: &mut GpuMemory, size: usize, label: &str) -> Option<usize> {
    let used = allocated_bytes(mem);
    if used + size > mem.total { return None; }
    let offset = used;
    mem.blocks.push(MemoryBlock { offset, size, label: label.to_string() });
    Some(offset)
}

#[allow(dead_code)]
pub fn free_block(mem: &mut GpuMemory, offset: usize) -> bool {
    if let Some(pos) = mem.blocks.iter().position(|b| b.offset == offset) {
        mem.blocks.remove(pos); true
    } else { false }
}

#[allow(dead_code)]
pub fn allocated_bytes(mem: &GpuMemory) -> usize { mem.blocks.iter().map(|b| b.size).sum() }

#[allow(dead_code)]
pub fn free_bytes(mem: &GpuMemory) -> usize { mem.total - allocated_bytes(mem) }

#[allow(dead_code)]
pub fn total_bytes(mem: &GpuMemory) -> usize { mem.total }

#[allow(dead_code)]
pub fn memory_fragmentation(mem: &GpuMemory) -> f32 {
    if mem.blocks.len() <= 1 { return 0.0; }
    let gaps = mem.blocks.len().saturating_sub(1);
    gaps as f32 / mem.blocks.len() as f32
}

#[allow(dead_code)]
pub fn memory_to_json(mem: &GpuMemory) -> String {
    format!("{{\"total\":{},\"allocated\":{},\"blocks\":{}}}", mem.total, allocated_bytes(mem), mem.blocks.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { assert_eq!(total_bytes(&new_gpu_memory(1024)), 1024); }
    #[test] fn test_allocate() {
        let mut m = new_gpu_memory(1024);
        assert!(allocate_block(&mut m, 256, "verts").is_some());
        assert_eq!(allocated_bytes(&m), 256);
    }
    #[test] fn test_allocate_full() {
        let mut m = new_gpu_memory(100);
        allocate_block(&mut m, 100, "a");
        assert!(allocate_block(&mut m, 1, "b").is_none());
    }
    #[test] fn test_free() {
        let mut m = new_gpu_memory(1024);
        let off = allocate_block(&mut m, 256, "a").unwrap();
        assert!(free_block(&mut m, off));
        assert_eq!(allocated_bytes(&m), 0);
    }
    #[test] fn test_free_missing() { assert!(!free_block(&mut new_gpu_memory(100), 999)); }
    #[test] fn test_free_bytes() {
        let mut m = new_gpu_memory(1000);
        allocate_block(&mut m, 400, "a");
        assert_eq!(free_bytes(&m), 600);
    }
    #[test] fn test_fragmentation_empty() { assert!((memory_fragmentation(&new_gpu_memory(100))).abs() < 1e-6); }
    #[test] fn test_fragmentation_multiple() {
        let mut m = new_gpu_memory(1000);
        allocate_block(&mut m, 100, "a"); allocate_block(&mut m, 100, "b");
        assert!(memory_fragmentation(&m) > 0.0);
    }
    #[test] fn test_to_json() { assert!(memory_to_json(&new_gpu_memory(100)).contains("total")); }
    #[test] fn test_total() { assert_eq!(total_bytes(&new_gpu_memory(2048)), 2048); }
}
