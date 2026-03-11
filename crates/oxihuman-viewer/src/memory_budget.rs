#![allow(dead_code)]

//! GPU memory budget tracking and allocation hints.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryHeapType {
    DeviceLocal,
    HostVisible,
    HostCached,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MemoryHeapBudget {
    pub heap_type: MemoryHeapType,
    pub budget_bytes: u64,
    pub used_bytes: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GpuMemoryBudget {
    pub heaps: Vec<MemoryHeapBudget>,
    pub warn_threshold: f32,
}

#[allow(dead_code)]
pub fn new_gpu_memory_budget(warn_threshold: f32) -> GpuMemoryBudget {
    GpuMemoryBudget {
        heaps: Vec::new(),
        warn_threshold: warn_threshold.clamp(0.0, 1.0),
    }
}

#[allow(dead_code)]
pub fn gmb_add_heap(budget: &mut GpuMemoryBudget, heap_type: MemoryHeapType, budget_bytes: u64) {
    budget.heaps.push(MemoryHeapBudget {
        heap_type,
        budget_bytes,
        used_bytes: 0,
    });
}

#[allow(dead_code)]
pub fn gmb_allocate(budget: &mut GpuMemoryBudget, heap_idx: usize, bytes: u64) -> bool {
    if let Some(heap) = budget.heaps.get_mut(heap_idx) {
        if heap.used_bytes + bytes <= heap.budget_bytes {
            heap.used_bytes += bytes;
            return true;
        }
    }
    false
}

#[allow(dead_code)]
pub fn gmb_free(budget: &mut GpuMemoryBudget, heap_idx: usize, bytes: u64) {
    if let Some(heap) = budget.heaps.get_mut(heap_idx) {
        heap.used_bytes = heap.used_bytes.saturating_sub(bytes);
    }
}

#[allow(dead_code)]
pub fn gmb_usage_ratio(budget: &GpuMemoryBudget, heap_idx: usize) -> f32 {
    if let Some(heap) = budget.heaps.get(heap_idx) {
        if heap.budget_bytes == 0 {
            return 0.0;
        }
        heap.used_bytes as f32 / heap.budget_bytes as f32
    } else {
        0.0
    }
}

#[allow(dead_code)]
pub fn gmb_is_over_threshold(budget: &GpuMemoryBudget, heap_idx: usize) -> bool {
    gmb_usage_ratio(budget, heap_idx) >= budget.warn_threshold
}

#[allow(dead_code)]
pub fn gmb_total_budget(budget: &GpuMemoryBudget) -> u64 {
    budget.heaps.iter().map(|h| h.budget_bytes).sum()
}

#[allow(dead_code)]
pub fn gmb_total_used(budget: &GpuMemoryBudget) -> u64 {
    budget.heaps.iter().map(|h| h.used_bytes).sum()
}

#[allow(dead_code)]
pub fn gmb_heap_count(budget: &GpuMemoryBudget) -> usize {
    budget.heaps.len()
}

#[allow(dead_code)]
pub fn gmb_to_json(budget: &GpuMemoryBudget) -> String {
    format!(
        "{{\"heap_count\":{},\"total_budget_mb\":{:.1},\"total_used_mb\":{:.1}}}",
        budget.heaps.len(),
        gmb_total_budget(budget) as f32 / (1024.0 * 1024.0),
        gmb_total_used(budget) as f32 / (1024.0 * 1024.0)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_budget() {
        let b = new_gpu_memory_budget(0.9);
        assert_eq!(gmb_heap_count(&b), 0);
    }

    #[test]
    fn test_add_heap() {
        let mut b = new_gpu_memory_budget(0.9);
        gmb_add_heap(&mut b, MemoryHeapType::DeviceLocal, 1024 * 1024 * 1024);
        assert_eq!(gmb_heap_count(&b), 1);
    }

    #[test]
    fn test_allocate() {
        let mut b = new_gpu_memory_budget(0.9);
        gmb_add_heap(&mut b, MemoryHeapType::DeviceLocal, 1024);
        let ok = gmb_allocate(&mut b, 0, 512);
        assert!(ok);
    }

    #[test]
    fn test_allocate_over_budget() {
        let mut b = new_gpu_memory_budget(0.9);
        gmb_add_heap(&mut b, MemoryHeapType::DeviceLocal, 512);
        gmb_allocate(&mut b, 0, 400);
        let ok = gmb_allocate(&mut b, 0, 200);
        assert!(!ok);
    }

    #[test]
    fn test_free() {
        let mut b = new_gpu_memory_budget(0.9);
        gmb_add_heap(&mut b, MemoryHeapType::DeviceLocal, 1024);
        gmb_allocate(&mut b, 0, 512);
        gmb_free(&mut b, 0, 256);
        assert!((gmb_usage_ratio(&b, 0) - 0.25).abs() < 1e-5);
    }

    #[test]
    fn test_usage_ratio() {
        let mut b = new_gpu_memory_budget(0.9);
        gmb_add_heap(&mut b, MemoryHeapType::DeviceLocal, 1000);
        gmb_allocate(&mut b, 0, 500);
        assert!((gmb_usage_ratio(&b, 0) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_over_threshold() {
        let mut b = new_gpu_memory_budget(0.8);
        gmb_add_heap(&mut b, MemoryHeapType::DeviceLocal, 100);
        gmb_allocate(&mut b, 0, 90);
        assert!(gmb_is_over_threshold(&b, 0));
    }

    #[test]
    fn test_total_budget() {
        let mut b = new_gpu_memory_budget(0.9);
        gmb_add_heap(&mut b, MemoryHeapType::DeviceLocal, 1000);
        gmb_add_heap(&mut b, MemoryHeapType::HostVisible, 500);
        assert_eq!(gmb_total_budget(&b), 1500);
    }

    #[test]
    fn test_total_used() {
        let mut b = new_gpu_memory_budget(0.9);
        gmb_add_heap(&mut b, MemoryHeapType::DeviceLocal, 1000);
        gmb_allocate(&mut b, 0, 300);
        assert_eq!(gmb_total_used(&b), 300);
    }

    #[test]
    fn test_to_json() {
        let b = new_gpu_memory_budget(0.9);
        let json = gmb_to_json(&b);
        assert!(json.contains("heap_count"));
    }
}
