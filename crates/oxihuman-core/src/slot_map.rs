//! Generational slot map with stable handles that detect stale references.
//!
//! Each slot stores a `f32` value together with a generation counter. A
//! [`SlotKey`] bundles the slot index with the generation at insertion time.
//! When the slot is removed and later reused, the old key becomes stale and
//! lookups return `None`.

/// A stable handle into the slot map.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotKey {
    /// Slot index.
    pub index: usize,
    /// Generation at the time this handle was created.
    pub generation: u32,
}

#[derive(Debug, Clone)]
struct Slot {
    value: f32,
    generation: u32,
    occupied: bool,
}

/// A generational slot map storing `f32` values.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SlotMap {
    slots: Vec<Slot>,
    free_list: Vec<usize>,
    count: usize,
}

/// Create a new, empty [`SlotMap`].
#[allow(dead_code)]
pub fn new_slot_map() -> SlotMap {
    SlotMap {
        slots: Vec::new(),
        free_list: Vec::new(),
        count: 0,
    }
}

/// Insert a value and return a [`SlotKey`] to it.
#[allow(dead_code)]
pub fn slot_insert(map: &mut SlotMap, value: f32) -> SlotKey {
    if let Some(idx) = map.free_list.pop() {
        let slot = &mut map.slots[idx];
        slot.value = value;
        slot.occupied = true;
        // generation was already incremented on removal
        map.count += 1;
        SlotKey {
            index: idx,
            generation: slot.generation,
        }
    } else {
        let idx = map.slots.len();
        map.slots.push(Slot {
            value,
            generation: 0,
            occupied: true,
        });
        map.count += 1;
        SlotKey {
            index: idx,
            generation: 0,
        }
    }
}

/// Remove the value identified by `key`.  Returns the removed value or `None`
/// if the key is stale.
#[allow(dead_code)]
pub fn slot_remove(map: &mut SlotMap, key: SlotKey) -> Option<f32> {
    let slot = map.slots.get_mut(key.index)?;
    if !slot.occupied || slot.generation != key.generation {
        return None;
    }
    let val = slot.value;
    slot.occupied = false;
    slot.generation = slot.generation.wrapping_add(1);
    map.free_list.push(key.index);
    map.count -= 1;
    Some(val)
}

/// Get a reference to the value identified by `key`, or `None` if stale.
#[allow(dead_code)]
pub fn slot_get(map: &SlotMap, key: SlotKey) -> Option<f32> {
    let slot = map.slots.get(key.index)?;
    if slot.occupied && slot.generation == key.generation {
        Some(slot.value)
    } else {
        None
    }
}

/// Return `true` if `key` refers to a live slot.
#[allow(dead_code)]
pub fn slot_contains(map: &SlotMap, key: SlotKey) -> bool {
    slot_get(map, key).is_some()
}

/// Return the number of live entries.
#[allow(dead_code)]
pub fn slot_len(map: &SlotMap) -> usize {
    map.count
}

/// Return `true` if the slot map has no live entries.
#[allow(dead_code)]
pub fn slot_is_empty(map: &SlotMap) -> bool {
    map.count == 0
}

/// Serialize the slot map to a compact JSON string.
#[allow(dead_code)]
pub fn slot_map_to_json(map: &SlotMap) -> String {
    format!(
        r#"{{"len":{},"slot_capacity":{}}}"#,
        map.count,
        map.slots.len()
    )
}

/// Remove all entries, resetting generation counters.
#[allow(dead_code)]
pub fn slot_clear(map: &mut SlotMap) {
    for slot in &mut map.slots {
        if slot.occupied {
            slot.occupied = false;
            slot.generation = slot.generation.wrapping_add(1);
        }
    }
    map.free_list.clear();
    // Re-populate free-list with all indices
    for i in 0..map.slots.len() {
        map.free_list.push(i);
    }
    map.count = 0;
}

/// Return the current generation of the slot at `index`, or 0 if out of range.
#[allow(dead_code)]
pub fn slot_generation(map: &SlotMap, index: usize) -> u32 {
    map.slots.get(index).map(|s| s.generation).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut m = new_slot_map();
        let k = slot_insert(&mut m, std::f32::consts::PI);
        assert!((slot_get(&m, k).unwrap() - std::f32::consts::PI).abs() < 1e-5);
    }

    #[test]
    fn test_len_increases_on_insert() {
        let mut m = new_slot_map();
        assert!(slot_is_empty(&m));
        slot_insert(&mut m, 1.0);
        assert_eq!(slot_len(&m), 1);
    }

    #[test]
    fn test_remove_returns_value() {
        let mut m = new_slot_map();
        let k = slot_insert(&mut m, 7.0);
        let v = slot_remove(&mut m, k).unwrap();
        assert!((v - 7.0).abs() < 1e-5);
        assert_eq!(slot_len(&m), 0);
    }

    #[test]
    fn test_stale_key_after_remove() {
        let mut m = new_slot_map();
        let k = slot_insert(&mut m, 1.0);
        slot_remove(&mut m, k);
        assert!(!slot_contains(&m, k));
        assert!(slot_get(&m, k).is_none());
    }

    #[test]
    fn test_old_key_stale_after_reinsert() {
        let mut m = new_slot_map();
        let k1 = slot_insert(&mut m, 1.0);
        slot_remove(&mut m, k1);
        let k2 = slot_insert(&mut m, 2.0);
        // k1 is stale; k2 uses the same slot with a higher generation
        assert!(!slot_contains(&m, k1));
        assert!(slot_contains(&m, k2));
        assert_eq!(k1.index, k2.index);
        assert!(k2.generation > k1.generation);
    }

    #[test]
    fn test_clear_invalidates_all() {
        let mut m = new_slot_map();
        let k1 = slot_insert(&mut m, 1.0);
        let k2 = slot_insert(&mut m, 2.0);
        slot_clear(&mut m);
        assert!(!slot_contains(&m, k1));
        assert!(!slot_contains(&m, k2));
        assert!(slot_is_empty(&m));
    }

    #[test]
    fn test_to_json() {
        let mut m = new_slot_map();
        slot_insert(&mut m, 0.5);
        let json = slot_map_to_json(&m);
        assert!(json.contains("len"));
        assert!(json.contains("slot_capacity"));
    }

    #[test]
    fn test_generation_increments() {
        let mut m = new_slot_map();
        let k = slot_insert(&mut m, 1.0);
        let gen0 = slot_generation(&m, k.index);
        slot_remove(&mut m, k);
        let gen1 = slot_generation(&m, k.index);
        assert!(gen1 > gen0);
    }

    #[test]
    fn test_multiple_inserts_and_removes() {
        let mut m = new_slot_map();
        let keys: Vec<SlotKey> = (0..10).map(|i| slot_insert(&mut m, i as f32)).collect();
        assert_eq!(slot_len(&m), 10);
        for k in &keys {
            slot_remove(&mut m, *k);
        }
        assert!(slot_is_empty(&m));
    }
}
