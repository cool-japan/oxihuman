// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Constraint batch / island management.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConstraintIsland {
    pub particle_ids: Vec<u32>,
    pub constraint_ids: Vec<u32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConstraintBatch {
    pub islands: Vec<ConstraintIsland>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub max_island_size: usize,
}

#[allow(dead_code)]
pub fn default_batch_config() -> BatchConfig {
    BatchConfig { max_island_size: 128 }
}

#[allow(dead_code)]
pub fn new_constraint_batch() -> ConstraintBatch {
    ConstraintBatch { islands: Vec::new() }
}

#[allow(dead_code)]
pub fn batch_add_island(batch: &mut ConstraintBatch, island: ConstraintIsland) {
    batch.islands.push(island);
}

#[allow(dead_code)]
pub fn batch_island_count(batch: &ConstraintBatch) -> usize {
    batch.islands.len()
}

#[allow(dead_code)]
pub fn batch_get_island(batch: &ConstraintBatch, i: usize) -> Option<&ConstraintIsland> {
    batch.islands.get(i)
}

#[allow(dead_code)]
pub fn batch_clear(batch: &mut ConstraintBatch) {
    batch.islands.clear();
}

/// Merge island j into island i, removing j.
#[allow(dead_code)]
pub fn batch_merge_islands(batch: &mut ConstraintBatch, i: usize, j: usize) {
    if i >= batch.islands.len() || j >= batch.islands.len() || i == j {
        return;
    }
    let (lo, hi) = if i < j { (i, j) } else { (j, i) };
    let j_island = batch.islands.remove(hi);
    let target_idx = lo;
    batch.islands[target_idx].particle_ids.extend(j_island.particle_ids);
    batch.islands[target_idx].constraint_ids.extend(j_island.constraint_ids);
}

#[allow(dead_code)]
pub fn batch_total_constraints(batch: &ConstraintBatch) -> usize {
    batch.islands.iter().map(|is| is.constraint_ids.len()).sum()
}

#[allow(dead_code)]
pub fn batch_total_particles(batch: &ConstraintBatch) -> usize {
    batch.islands.iter().map(|is| is.particle_ids.len()).sum()
}

#[allow(dead_code)]
pub fn batch_to_json(batch: &ConstraintBatch) -> String {
    format!(
        "{{\"island_count\":{},\"total_particles\":{},\"total_constraints\":{}}}",
        batch_island_count(batch),
        batch_total_particles(batch),
        batch_total_constraints(batch)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_island(pids: &[u32], cids: &[u32]) -> ConstraintIsland {
        ConstraintIsland {
            particle_ids: pids.to_vec(),
            constraint_ids: cids.to_vec(),
        }
    }

    #[test]
    fn test_default_config() {
        let cfg = default_batch_config();
        assert_eq!(cfg.max_island_size, 128);
    }

    #[test]
    fn test_new_batch_empty() {
        let b = new_constraint_batch();
        assert_eq!(batch_island_count(&b), 0);
    }

    #[test]
    fn test_add_island() {
        let mut b = new_constraint_batch();
        batch_add_island(&mut b, make_island(&[1, 2], &[0]));
        assert_eq!(batch_island_count(&b), 1);
    }

    #[test]
    fn test_get_island() {
        let mut b = new_constraint_batch();
        batch_add_island(&mut b, make_island(&[5], &[10]));
        let is = batch_get_island(&b, 0).unwrap();
        assert_eq!(is.particle_ids, &[5]);
    }

    #[test]
    fn test_batch_clear() {
        let mut b = new_constraint_batch();
        batch_add_island(&mut b, make_island(&[1], &[0]));
        batch_clear(&mut b);
        assert_eq!(batch_island_count(&b), 0);
    }

    #[test]
    fn test_merge_islands() {
        let mut b = new_constraint_batch();
        batch_add_island(&mut b, make_island(&[1, 2], &[0]));
        batch_add_island(&mut b, make_island(&[3, 4], &[1, 2]));
        batch_merge_islands(&mut b, 0, 1);
        assert_eq!(batch_island_count(&b), 1);
        assert_eq!(batch_total_particles(&b), 4);
    }

    #[test]
    fn test_total_constraints() {
        let mut b = new_constraint_batch();
        batch_add_island(&mut b, make_island(&[1], &[0, 1]));
        batch_add_island(&mut b, make_island(&[2], &[2]));
        assert_eq!(batch_total_constraints(&b), 3);
    }

    #[test]
    fn test_total_particles() {
        let mut b = new_constraint_batch();
        batch_add_island(&mut b, make_island(&[1, 2, 3], &[]));
        assert_eq!(batch_total_particles(&b), 3);
    }

    #[test]
    fn test_to_json() {
        let b = new_constraint_batch();
        let j = batch_to_json(&b);
        assert!(j.contains("island_count"));
    }

    #[test]
    fn test_get_island_out_of_bounds() {
        let b = new_constraint_batch();
        assert!(batch_get_island(&b, 5).is_none());
    }
}
