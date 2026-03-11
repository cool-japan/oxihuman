// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Fixed-size page allocator backed by a Vec of pages.
#[allow(dead_code)]
pub struct PageAllocator {
    page_size: usize,
    pages: Vec<Vec<u8>>,
    free_pages: Vec<usize>,
    allocated: usize,
}

#[allow(dead_code)]
impl PageAllocator {
    pub fn new(page_size: usize) -> Self {
        Self {
            page_size,
            pages: Vec::new(),
            free_pages: Vec::new(),
            allocated: 0,
        }
    }
    pub fn alloc_page(&mut self) -> usize {
        if let Some(idx) = self.free_pages.pop() {
            self.allocated += 1;
            idx
        } else {
            let idx = self.pages.len();
            self.pages.push(vec![0u8; self.page_size]);
            self.allocated += 1;
            idx
        }
    }
    pub fn free_page(&mut self, idx: usize) -> bool {
        if idx < self.pages.len() && !self.free_pages.contains(&idx) {
            self.free_pages.push(idx);
            self.allocated = self.allocated.saturating_sub(1);
            true
        } else {
            false
        }
    }
    pub fn write_page(&mut self, idx: usize, data: &[u8]) -> bool {
        if idx >= self.pages.len() {
            return false;
        }
        let len = data.len().min(self.page_size);
        self.pages[idx][..len].copy_from_slice(&data[..len]);
        true
    }
    pub fn read_page(&self, idx: usize) -> Option<&[u8]> {
        self.pages.get(idx).map(|p| p.as_slice())
    }
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }
    pub fn free_count(&self) -> usize {
        self.free_pages.len()
    }
    pub fn allocated_count(&self) -> usize {
        self.allocated
    }
    pub fn page_size(&self) -> usize {
        self.page_size
    }
    pub fn is_free(&self, idx: usize) -> bool {
        self.free_pages.contains(&idx)
    }
    pub fn total_bytes(&self) -> usize {
        self.pages.len() * self.page_size
    }
    pub fn reset(&mut self) {
        self.pages.clear();
        self.free_pages.clear();
        self.allocated = 0;
    }
}

#[allow(dead_code)]
pub fn new_page_allocator(page_size: usize) -> PageAllocator {
    PageAllocator::new(page_size)
}
#[allow(dead_code)]
pub fn pa_alloc(a: &mut PageAllocator) -> usize {
    a.alloc_page()
}
#[allow(dead_code)]
pub fn pa_free(a: &mut PageAllocator, idx: usize) -> bool {
    a.free_page(idx)
}
#[allow(dead_code)]
pub fn pa_write(a: &mut PageAllocator, idx: usize, data: &[u8]) -> bool {
    a.write_page(idx, data)
}
#[allow(dead_code)]
pub fn pa_read(a: &PageAllocator, idx: usize) -> Option<&[u8]> {
    a.read_page(idx)
}
#[allow(dead_code)]
pub fn pa_page_count(a: &PageAllocator) -> usize {
    a.page_count()
}
#[allow(dead_code)]
pub fn pa_free_count(a: &PageAllocator) -> usize {
    a.free_count()
}
#[allow(dead_code)]
pub fn pa_allocated_count(a: &PageAllocator) -> usize {
    a.allocated_count()
}
#[allow(dead_code)]
pub fn pa_total_bytes(a: &PageAllocator) -> usize {
    a.total_bytes()
}
#[allow(dead_code)]
pub fn pa_reset(a: &mut PageAllocator) {
    a.reset();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_alloc_read() {
        let mut a = new_page_allocator(64);
        let idx = pa_alloc(&mut a);
        assert!(pa_read(&a, idx).is_some());
    }
    #[test]
    fn test_write_read() {
        let mut a = new_page_allocator(8);
        let idx = pa_alloc(&mut a);
        pa_write(&mut a, idx, &[1, 2, 3, 4]);
        let data = pa_read(&a, idx).unwrap();
        assert_eq!(&data[..4], &[1, 2, 3, 4]);
    }
    #[test]
    fn test_free_recycles() {
        let mut a = new_page_allocator(16);
        let idx = pa_alloc(&mut a);
        pa_free(&mut a, idx);
        let idx2 = pa_alloc(&mut a);
        assert_eq!(idx, idx2);
    }
    #[test]
    fn test_page_count() {
        let mut a = new_page_allocator(16);
        pa_alloc(&mut a);
        pa_alloc(&mut a);
        assert_eq!(pa_page_count(&a), 2);
    }
    #[test]
    fn test_free_count() {
        let mut a = new_page_allocator(16);
        let idx = pa_alloc(&mut a);
        pa_free(&mut a, idx);
        assert_eq!(pa_free_count(&a), 1);
    }
    #[test]
    fn test_allocated_count() {
        let mut a = new_page_allocator(16);
        pa_alloc(&mut a);
        assert_eq!(pa_allocated_count(&a), 1);
    }
    #[test]
    fn test_total_bytes() {
        let mut a = new_page_allocator(32);
        pa_alloc(&mut a);
        pa_alloc(&mut a);
        assert_eq!(pa_total_bytes(&a), 64);
    }
    #[test]
    fn test_reset() {
        let mut a = new_page_allocator(16);
        pa_alloc(&mut a);
        pa_reset(&mut a);
        assert_eq!(pa_page_count(&a), 0);
    }
    #[test]
    fn test_is_free() {
        let mut a = new_page_allocator(16);
        let idx = pa_alloc(&mut a);
        pa_free(&mut a, idx);
        assert!(a.is_free(idx));
    }
    #[test]
    fn test_read_invalid() {
        let a = new_page_allocator(16);
        assert!(pa_read(&a, 99).is_none());
    }
}
