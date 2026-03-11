#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Generational entity pool with recycling.

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId {
    pub index: usize,
    pub generation: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EntityPool {
    generations: Vec<u32>,
    alive: Vec<bool>,
    free_list: Vec<usize>,
}

#[allow(dead_code)]
pub fn new_entity_pool() -> EntityPool {
    EntityPool {
        generations: Vec::new(),
        alive: Vec::new(),
        free_list: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn spawn_entity(pool: &mut EntityPool) -> EntityId {
    if let Some(idx) = pool.free_list.pop() {
        pool.generations[idx] += 1;
        pool.alive[idx] = true;
        EntityId {
            index: idx,
            generation: pool.generations[idx],
        }
    } else {
        let idx = pool.generations.len();
        pool.generations.push(0);
        pool.alive.push(true);
        EntityId {
            index: idx,
            generation: 0,
        }
    }
}

#[allow(dead_code)]
pub fn despawn_entity(pool: &mut EntityPool, id: EntityId) -> bool {
    if id.index < pool.alive.len()
        && pool.alive[id.index]
        && pool.generations[id.index] == id.generation
    {
        pool.alive[id.index] = false;
        pool.free_list.push(id.index);
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn entity_is_alive(pool: &EntityPool, id: EntityId) -> bool {
    id.index < pool.alive.len()
        && pool.alive[id.index]
        && pool.generations[id.index] == id.generation
}

#[allow(dead_code)]
pub fn entity_count_ep(pool: &EntityPool) -> usize {
    pool.alive.iter().filter(|&&a| a).count()
}

#[allow(dead_code)]
pub fn entity_generation(pool: &EntityPool, id: EntityId) -> Option<u32> {
    if id.index < pool.generations.len() {
        Some(pool.generations[id.index])
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn pool_compact(pool: &mut EntityPool) {
    pool.free_list.sort_unstable();
    pool.free_list.dedup();
}

#[allow(dead_code)]
pub fn pool_to_json(pool: &EntityPool) -> String {
    format!(
        r#"{{"alive":{},"total":{},"free":{}}}"#,
        entity_count_ep(pool),
        pool.generations.len(),
        pool.free_list.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pool() {
        let pool = new_entity_pool();
        assert_eq!(entity_count_ep(&pool), 0);
    }

    #[test]
    fn test_spawn() {
        let mut pool = new_entity_pool();
        let id = spawn_entity(&mut pool);
        assert_eq!(id.index, 0);
        assert!(entity_is_alive(&pool, id));
    }

    #[test]
    fn test_despawn() {
        let mut pool = new_entity_pool();
        let id = spawn_entity(&mut pool);
        assert!(despawn_entity(&mut pool, id));
        assert!(!entity_is_alive(&pool, id));
    }

    #[test]
    fn test_generation_increment() {
        let mut pool = new_entity_pool();
        let id1 = spawn_entity(&mut pool);
        despawn_entity(&mut pool, id1);
        let id2 = spawn_entity(&mut pool);
        assert_eq!(id2.index, id1.index);
        assert_eq!(id2.generation, 1);
    }

    #[test]
    fn test_stale_id() {
        let mut pool = new_entity_pool();
        let id1 = spawn_entity(&mut pool);
        despawn_entity(&mut pool, id1);
        let _id2 = spawn_entity(&mut pool);
        assert!(!entity_is_alive(&pool, id1));
    }

    #[test]
    fn test_entity_count() {
        let mut pool = new_entity_pool();
        spawn_entity(&mut pool);
        spawn_entity(&mut pool);
        assert_eq!(entity_count_ep(&pool), 2);
    }

    #[test]
    fn test_despawn_invalid() {
        let mut pool = new_entity_pool();
        let fake = EntityId { index: 99, generation: 0 };
        assert!(!despawn_entity(&mut pool, fake));
    }

    #[test]
    fn test_entity_generation() {
        let mut pool = new_entity_pool();
        let id = spawn_entity(&mut pool);
        assert_eq!(entity_generation(&pool, id), Some(0));
    }

    #[test]
    fn test_pool_compact() {
        let mut pool = new_entity_pool();
        let id = spawn_entity(&mut pool);
        despawn_entity(&mut pool, id);
        pool_compact(&mut pool);
        assert_eq!(pool.free_list.len(), 1);
    }

    #[test]
    fn test_pool_to_json() {
        let pool = new_entity_pool();
        let json = pool_to_json(&pool);
        assert!(json.contains("\"alive\":0"));
    }
}
