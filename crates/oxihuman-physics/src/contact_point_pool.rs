#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct ContactPoint {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub depth: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ContactPointPool {
    points: Vec<ContactPoint>,
    capacity: usize,
}

#[allow(dead_code)]
pub fn new_contact_pool(capacity: usize) -> ContactPointPool {
    ContactPointPool {
        points: Vec::with_capacity(capacity),
        capacity,
    }
}

#[allow(dead_code)]
pub fn pool_add_contact(pool: &mut ContactPointPool, position: [f32; 3], normal: [f32; 3], depth: f32) -> bool {
    if pool.points.len() >= pool.capacity {
        return false;
    }
    pool.points.push(ContactPoint {
        position,
        normal,
        depth,
    });
    true
}

#[allow(dead_code)]
pub fn pool_get_contact(pool: &ContactPointPool, index: usize) -> Option<&ContactPoint> {
    pool.points.get(index)
}

#[allow(dead_code)]
pub fn pool_count_cp(pool: &ContactPointPool) -> usize {
    pool.points.len()
}

#[allow(dead_code)]
pub fn pool_clear_cp(pool: &mut ContactPointPool) {
    pool.points.clear();
}

#[allow(dead_code)]
pub fn pool_capacity_cp(pool: &ContactPointPool) -> usize {
    pool.capacity
}

#[allow(dead_code)]
pub fn pool_is_full_cp(pool: &ContactPointPool) -> bool {
    pool.points.len() >= pool.capacity
}

#[allow(dead_code)]
pub fn pool_drain(pool: &mut ContactPointPool) -> Vec<ContactPoint> {
    pool.points.drain(..).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let p = new_contact_pool(10);
        assert_eq!(pool_count_cp(&p), 0);
    }

    #[test]
    fn test_add() {
        let mut p = new_contact_pool(10);
        assert!(pool_add_contact(&mut p, [0.0; 3], [0.0, 1.0, 0.0], 0.1));
        assert_eq!(pool_count_cp(&p), 1);
    }

    #[test]
    fn test_get() {
        let mut p = new_contact_pool(10);
        pool_add_contact(&mut p, [1.0, 2.0, 3.0], [0.0, 1.0, 0.0], 0.5);
        let c = pool_get_contact(&p, 0).unwrap();
        assert_eq!(c.position, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_get_out_of_bounds() {
        let p = new_contact_pool(10);
        assert!(pool_get_contact(&p, 0).is_none());
    }

    #[test]
    fn test_clear() {
        let mut p = new_contact_pool(10);
        pool_add_contact(&mut p, [0.0; 3], [0.0; 3], 0.0);
        pool_clear_cp(&mut p);
        assert_eq!(pool_count_cp(&p), 0);
    }

    #[test]
    fn test_capacity() {
        let p = new_contact_pool(5);
        assert_eq!(pool_capacity_cp(&p), 5);
    }

    #[test]
    fn test_full() {
        let mut p = new_contact_pool(2);
        pool_add_contact(&mut p, [0.0; 3], [0.0; 3], 0.0);
        pool_add_contact(&mut p, [0.0; 3], [0.0; 3], 0.0);
        assert!(pool_is_full_cp(&p));
    }

    #[test]
    fn test_add_when_full() {
        let mut p = new_contact_pool(1);
        pool_add_contact(&mut p, [0.0; 3], [0.0; 3], 0.0);
        assert!(!pool_add_contact(&mut p, [0.0; 3], [0.0; 3], 0.0));
    }

    #[test]
    fn test_drain() {
        let mut p = new_contact_pool(10);
        pool_add_contact(&mut p, [0.0; 3], [0.0; 3], 1.0);
        pool_add_contact(&mut p, [0.0; 3], [0.0; 3], 2.0);
        let drained = pool_drain(&mut p);
        assert_eq!(drained.len(), 2);
        assert_eq!(pool_count_cp(&p), 0);
    }

    #[test]
    fn test_not_full() {
        let p = new_contact_pool(5);
        assert!(!pool_is_full_cp(&p));
    }
}
