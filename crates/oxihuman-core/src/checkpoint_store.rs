// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Stores named checkpoints of serialised state for save/restore workflows.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Checkpoint {
    pub name: String,
    pub data: Vec<u8>,
    pub timestamp: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheckpointStore {
    checkpoints: Vec<Checkpoint>,
    max_checkpoints: usize,
    clock: u64,
}

#[allow(dead_code)]
impl CheckpointStore {
    pub fn new(max_checkpoints: usize) -> Self {
        Self {
            checkpoints: Vec::new(),
            max_checkpoints: max_checkpoints.max(1),
            clock: 0,
        }
    }

    pub fn advance_clock(&mut self, dt: u64) {
        self.clock += dt;
    }

    pub fn save(&mut self, name: &str, data: Vec<u8>) {
        if let Some(cp) = self.checkpoints.iter_mut().find(|c| c.name == name) {
            cp.data = data;
            cp.timestamp = self.clock;
            return;
        }
        if self.checkpoints.len() >= self.max_checkpoints {
            self.checkpoints.remove(0);
        }
        self.checkpoints.push(Checkpoint {
            name: name.to_string(),
            data,
            timestamp: self.clock,
        });
    }

    pub fn restore(&self, name: &str) -> Option<&[u8]> {
        self.checkpoints
            .iter()
            .find(|c| c.name == name)
            .map(|c| c.data.as_slice())
    }

    pub fn remove(&mut self, name: &str) -> bool {
        let before = self.checkpoints.len();
        self.checkpoints.retain(|c| c.name != name);
        self.checkpoints.len() < before
    }

    pub fn count(&self) -> usize {
        self.checkpoints.len()
    }

    pub fn names(&self) -> Vec<&str> {
        self.checkpoints.iter().map(|c| c.name.as_str()).collect()
    }

    pub fn latest(&self) -> Option<&Checkpoint> {
        self.checkpoints.iter().max_by_key(|c| c.timestamp)
    }

    pub fn oldest(&self) -> Option<&Checkpoint> {
        self.checkpoints.iter().min_by_key(|c| c.timestamp)
    }

    pub fn total_bytes(&self) -> usize {
        self.checkpoints.iter().map(|c| c.data.len()).sum()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.checkpoints.iter().any(|c| c.name == name)
    }

    pub fn clear(&mut self) {
        self.checkpoints.clear();
    }

    pub fn max_checkpoints(&self) -> usize {
        self.max_checkpoints
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let cs = CheckpointStore::new(5);
        assert_eq!(cs.count(), 0);
        assert_eq!(cs.max_checkpoints(), 5);
    }

    #[test]
    fn test_save_and_restore() {
        let mut cs = CheckpointStore::new(5);
        cs.save("cp1", vec![1, 2, 3]);
        assert_eq!(cs.restore("cp1"), Some([1u8, 2, 3].as_slice()));
    }

    #[test]
    fn test_overwrite() {
        let mut cs = CheckpointStore::new(5);
        cs.save("cp1", vec![1]);
        cs.save("cp1", vec![2]);
        assert_eq!(cs.restore("cp1"), Some([2u8].as_slice()));
        assert_eq!(cs.count(), 1);
    }

    #[test]
    fn test_max_eviction() {
        let mut cs = CheckpointStore::new(2);
        cs.save("a", vec![1]);
        cs.save("b", vec![2]);
        cs.save("c", vec![3]);
        assert_eq!(cs.count(), 2);
        assert!(!cs.contains("a"));
        assert!(cs.contains("c"));
    }

    #[test]
    fn test_remove() {
        let mut cs = CheckpointStore::new(5);
        cs.save("a", vec![1]);
        assert!(cs.remove("a"));
        assert!(!cs.remove("a"));
    }

    #[test]
    fn test_latest() {
        let mut cs = CheckpointStore::new(5);
        cs.save("a", vec![1]);
        cs.advance_clock(10);
        cs.save("b", vec![2]);
        assert_eq!(cs.latest().expect("should succeed").name, "b");
    }

    #[test]
    fn test_oldest() {
        let mut cs = CheckpointStore::new(5);
        cs.save("a", vec![1]);
        cs.advance_clock(10);
        cs.save("b", vec![2]);
        assert_eq!(cs.oldest().expect("should succeed").name, "a");
    }

    #[test]
    fn test_total_bytes() {
        let mut cs = CheckpointStore::new(5);
        cs.save("a", vec![1, 2]);
        cs.save("b", vec![3, 4, 5]);
        assert_eq!(cs.total_bytes(), 5);
    }

    #[test]
    fn test_names() {
        let mut cs = CheckpointStore::new(5);
        cs.save("x", vec![1]);
        cs.save("y", vec![2]);
        assert_eq!(cs.names().len(), 2);
    }

    #[test]
    fn test_clear() {
        let mut cs = CheckpointStore::new(5);
        cs.save("a", vec![1]);
        cs.clear();
        assert_eq!(cs.count(), 0);
    }
}
