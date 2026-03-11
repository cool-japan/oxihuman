#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GenerationId {
    index: u32,
    generation: u32,
}

#[allow(dead_code)]
pub fn new_generation_id(index: u32, generation: u32) -> GenerationId {
    GenerationId { index, generation }
}

#[allow(dead_code)]
pub fn generation(id: &GenerationId) -> u32 {
    id.generation
}

#[allow(dead_code)]
pub fn index_of(id: &GenerationId) -> u32 {
    id.index
}

#[allow(dead_code)]
pub fn increment_generation(id: &mut GenerationId) {
    id.generation = id.generation.wrapping_add(1);
}

#[allow(dead_code)]
pub fn generation_is_valid(id: &GenerationId, expected_gen: u32) -> bool {
    id.generation == expected_gen
}

#[allow(dead_code)]
pub fn generation_to_u64(id: &GenerationId) -> u64 {
    ((id.generation as u64) << 32) | (id.index as u64)
}

#[allow(dead_code)]
pub fn generation_from_parts(index: u32, gen: u32) -> GenerationId {
    GenerationId {
        index,
        generation: gen,
    }
}

#[allow(dead_code)]
pub fn generation_ids_equal(a: &GenerationId, b: &GenerationId) -> bool {
    a.index == b.index && a.generation == b.generation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let id = new_generation_id(0, 1);
        assert_eq!(index_of(&id), 0);
        assert_eq!(generation(&id), 1);
    }

    #[test]
    fn test_increment() {
        let mut id = new_generation_id(5, 0);
        increment_generation(&mut id);
        assert_eq!(generation(&id), 1);
    }

    #[test]
    fn test_is_valid() {
        let id = new_generation_id(0, 3);
        assert!(generation_is_valid(&id, 3));
        assert!(!generation_is_valid(&id, 2));
    }

    #[test]
    fn test_to_u64() {
        let id = new_generation_id(1, 2);
        let v = generation_to_u64(&id);
        assert_eq!(v, (2u64 << 32) | 1);
    }

    #[test]
    fn test_from_parts() {
        let id = generation_from_parts(10, 20);
        assert_eq!(index_of(&id), 10);
        assert_eq!(generation(&id), 20);
    }

    #[test]
    fn test_equal() {
        let a = new_generation_id(1, 1);
        let b = new_generation_id(1, 1);
        assert!(generation_ids_equal(&a, &b));
    }

    #[test]
    fn test_not_equal_gen() {
        let a = new_generation_id(1, 1);
        let b = new_generation_id(1, 2);
        assert!(!generation_ids_equal(&a, &b));
    }

    #[test]
    fn test_not_equal_index() {
        let a = new_generation_id(1, 1);
        let b = new_generation_id(2, 1);
        assert!(!generation_ids_equal(&a, &b));
    }

    #[test]
    fn test_wrapping_increment() {
        let mut id = new_generation_id(0, u32::MAX);
        increment_generation(&mut id);
        assert_eq!(generation(&id), 0);
    }

    #[test]
    fn test_zero_id() {
        let id = new_generation_id(0, 0);
        assert_eq!(generation_to_u64(&id), 0);
    }
}
