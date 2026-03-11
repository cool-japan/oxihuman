#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArrayPool {
    array_size: usize,
    available: Vec<Vec<f32>>,
    rented: usize,
}

#[allow(dead_code)]
pub fn new_array_pool(array_size: usize, initial_count: usize) -> ArrayPool {
    let available = (0..initial_count)
        .map(|_| vec![0.0f32; array_size])
        .collect();
    ArrayPool {
        array_size,
        available,
        rented: 0,
    }
}

#[allow(dead_code)]
pub fn rent_array(pool: &mut ArrayPool) -> Vec<f32> {
    pool.rented += 1;
    if let Some(arr) = pool.available.pop() {
        arr
    } else {
        vec![0.0f32; pool.array_size]
    }
}

#[allow(dead_code)]
pub fn return_array(pool: &mut ArrayPool, mut arr: Vec<f32>) {
    if arr.len() == pool.array_size {
        for v in arr.iter_mut() {
            *v = 0.0;
        }
        pool.available.push(arr);
        if pool.rented > 0 {
            pool.rented -= 1;
        }
    }
}

#[allow(dead_code)]
pub fn pool_rented_count(pool: &ArrayPool) -> usize {
    pool.rented
}

#[allow(dead_code)]
pub fn pool_available_count(pool: &ArrayPool) -> usize {
    pool.available.len()
}

#[allow(dead_code)]
pub fn pool_total_count(pool: &ArrayPool) -> usize {
    pool.rented + pool.available.len()
}

#[allow(dead_code)]
pub fn pool_array_size(pool: &ArrayPool) -> usize {
    pool.array_size
}

#[allow(dead_code)]
pub fn pool_clear_ap(pool: &mut ArrayPool) {
    pool.available.clear();
    pool.rented = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pool() {
        let pool = new_array_pool(16, 4);
        assert_eq!(pool_available_count(&pool), 4);
    }

    #[test]
    fn test_rent() {
        let mut pool = new_array_pool(8, 2);
        let arr = rent_array(&mut pool);
        assert_eq!(arr.len(), 8);
        assert_eq!(pool_rented_count(&pool), 1);
    }

    #[test]
    fn test_return() {
        let mut pool = new_array_pool(8, 1);
        let arr = rent_array(&mut pool);
        return_array(&mut pool, arr);
        assert_eq!(pool_rented_count(&pool), 0);
        assert_eq!(pool_available_count(&pool), 1);
    }

    #[test]
    fn test_rent_creates_new() {
        let mut pool = new_array_pool(4, 0);
        let arr = rent_array(&mut pool);
        assert_eq!(arr.len(), 4);
    }

    #[test]
    fn test_total_count() {
        let pool = new_array_pool(4, 3);
        assert_eq!(pool_total_count(&pool), 3);
    }

    #[test]
    fn test_array_size() {
        let pool = new_array_pool(32, 1);
        assert_eq!(pool_array_size(&pool), 32);
    }

    #[test]
    fn test_clear() {
        let mut pool = new_array_pool(4, 5);
        pool_clear_ap(&mut pool);
        assert_eq!(pool_available_count(&pool), 0);
        assert_eq!(pool_rented_count(&pool), 0);
    }

    #[test]
    fn test_wrong_size_return() {
        let mut pool = new_array_pool(4, 0);
        let wrong = vec![0.0f32; 8];
        return_array(&mut pool, wrong);
        assert_eq!(pool_available_count(&pool), 0);
    }

    #[test]
    fn test_rent_returns_zeroed() {
        let mut pool = new_array_pool(4, 0);
        let mut arr = rent_array(&mut pool);
        arr[0] = 99.0;
        return_array(&mut pool, arr);
        let arr2 = rent_array(&mut pool);
        assert_eq!(arr2[0], 0.0);
    }

    #[test]
    fn test_multiple_rent() {
        let mut pool = new_array_pool(2, 3);
        let _a = rent_array(&mut pool);
        let _b = rent_array(&mut pool);
        assert_eq!(pool_rented_count(&pool), 2);
        assert_eq!(pool_available_count(&pool), 1);
    }
}
