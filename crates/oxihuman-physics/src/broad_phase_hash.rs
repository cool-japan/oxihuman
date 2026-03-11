#![allow(dead_code)]

use std::collections::HashMap;

/// Spatial hash broad phase collision detection.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HashBroadPhase {
    cell_size: f32,
    cells: HashMap<(i32, i32, i32), Vec<u32>>,
    body_count: usize,
}

fn hash_cell(pos: [f32; 3], cell_size: f32) -> (i32, i32, i32) {
    (
        (pos[0] / cell_size).floor() as i32,
        (pos[1] / cell_size).floor() as i32,
        (pos[2] / cell_size).floor() as i32,
    )
}

/// Creates a new hash broad phase with the given cell size.
#[allow(dead_code)]
pub fn new_hash_broad_phase(cell_size: f32) -> HashBroadPhase {
    HashBroadPhase {
        cell_size,
        cells: HashMap::new(),
        body_count: 0,
    }
}

/// Inserts an AABB into the hash.
#[allow(dead_code)]
pub fn insert_aabb_hash(bp: &mut HashBroadPhase, id: u32, min: [f32; 3], max: [f32; 3]) {
    let min_cell = hash_cell(min, bp.cell_size);
    let max_cell = hash_cell(max, bp.cell_size);
    for x in min_cell.0..=max_cell.0 {
        for y in min_cell.1..=max_cell.1 {
            for z in min_cell.2..=max_cell.2 {
                bp.cells.entry((x, y, z)).or_default().push(id);
            }
        }
    }
    bp.body_count += 1;
}

/// Removes an id from all cells.
#[allow(dead_code)]
pub fn remove_aabb_hash(bp: &mut HashBroadPhase, id: u32) {
    for cell in bp.cells.values_mut() {
        cell.retain(|&i| i != id);
    }
    bp.body_count = bp.body_count.saturating_sub(1);
}

/// Queries for potential overlaps with an AABB.
#[allow(dead_code)]
pub fn query_aabb_hash(bp: &HashBroadPhase, min: [f32; 3], max: [f32; 3]) -> Vec<u32> {
    let mut result = Vec::new();
    let min_cell = hash_cell(min, bp.cell_size);
    let max_cell = hash_cell(max, bp.cell_size);
    for x in min_cell.0..=max_cell.0 {
        for y in min_cell.1..=max_cell.1 {
            for z in min_cell.2..=max_cell.2 {
                if let Some(ids) = bp.cells.get(&(x, y, z)) {
                    for &id in ids {
                        if !result.contains(&id) {
                            result.push(id);
                        }
                    }
                }
            }
        }
    }
    result
}

/// Returns the number of potential pairs.
#[allow(dead_code)]
pub fn pair_count_hash(bp: &HashBroadPhase) -> usize {
    let mut count = 0;
    for cell in bp.cells.values() {
        let n = cell.len();
        count += n * n.saturating_sub(1) / 2;
    }
    count
}

/// Clears all cells.
#[allow(dead_code)]
pub fn clear_hash_broad(bp: &mut HashBroadPhase) {
    bp.cells.clear();
    bp.body_count = 0;
}

/// Returns the cell size.
#[allow(dead_code)]
pub fn cell_size(bp: &HashBroadPhase) -> f32 {
    bp.cell_size
}

/// Returns statistics about the broad phase.
#[allow(dead_code)]
pub fn hash_broad_stats(bp: &HashBroadPhase) -> String {
    format!(
        "{{\"cell_count\":{},\"body_count\":{}}}",
        bp.cells.len(),
        bp.body_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let bp = new_hash_broad_phase(1.0);
        assert!((cell_size(&bp) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_insert_query() {
        let mut bp = new_hash_broad_phase(1.0);
        insert_aabb_hash(&mut bp, 1, [0.0; 3], [0.5; 3]);
        let r = query_aabb_hash(&bp, [0.0; 3], [0.5; 3]);
        assert!(r.contains(&1));
    }

    #[test]
    fn test_remove() {
        let mut bp = new_hash_broad_phase(1.0);
        insert_aabb_hash(&mut bp, 1, [0.0; 3], [0.5; 3]);
        remove_aabb_hash(&mut bp, 1);
        let r = query_aabb_hash(&bp, [0.0; 3], [0.5; 3]);
        assert!(!r.contains(&1));
    }

    #[test]
    fn test_pair_count() {
        let mut bp = new_hash_broad_phase(10.0);
        insert_aabb_hash(&mut bp, 1, [0.0; 3], [1.0; 3]);
        insert_aabb_hash(&mut bp, 2, [0.0; 3], [1.0; 3]);
        assert!(pair_count_hash(&bp) > 0);
    }

    #[test]
    fn test_clear() {
        let mut bp = new_hash_broad_phase(1.0);
        insert_aabb_hash(&mut bp, 1, [0.0; 3], [1.0; 3]);
        clear_hash_broad(&mut bp);
        assert_eq!(bp.body_count, 0);
    }

    #[test]
    fn test_stats() {
        let bp = new_hash_broad_phase(1.0);
        let s = hash_broad_stats(&bp);
        assert!(s.contains("cell_count"));
    }

    #[test]
    fn test_no_overlap() {
        let mut bp = new_hash_broad_phase(1.0);
        insert_aabb_hash(&mut bp, 1, [0.0; 3], [0.5; 3]);
        let r = query_aabb_hash(&bp, [100.0; 3], [101.0; 3]);
        assert!(!r.contains(&1));
    }

    #[test]
    fn test_multiple_bodies() {
        let mut bp = new_hash_broad_phase(1.0);
        insert_aabb_hash(&mut bp, 1, [0.0; 3], [0.5; 3]);
        insert_aabb_hash(&mut bp, 2, [0.0; 3], [0.5; 3]);
        let r = query_aabb_hash(&bp, [0.0; 3], [0.5; 3]);
        assert!(r.contains(&1));
        assert!(r.contains(&2));
    }

    #[test]
    fn test_cell_size_value() {
        let bp = new_hash_broad_phase(2.5);
        assert!((cell_size(&bp) - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_empty_query() {
        let bp = new_hash_broad_phase(1.0);
        let r = query_aabb_hash(&bp, [0.0; 3], [1.0; 3]);
        assert!(r.is_empty());
    }
}
