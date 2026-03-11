#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DoubleBuffer<T> {
    front: Vec<T>,
    back: Vec<T>,
}

#[allow(dead_code)]
pub fn new_double_buffer<T>() -> DoubleBuffer<T> {
    DoubleBuffer {
        front: Vec::new(),
        back: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn write_buffer<T>(db: &mut DoubleBuffer<T>, val: T) {
    db.back.push(val);
}

#[allow(dead_code)]
pub fn read_buffer<T>(db: &DoubleBuffer<T>) -> &[T] {
    &db.front
}

#[allow(dead_code)]
pub fn swap_buffers<T>(db: &mut DoubleBuffer<T>) {
    std::mem::swap(&mut db.front, &mut db.back);
    db.back.clear();
}

#[allow(dead_code)]
pub fn buffer_len<T>(db: &DoubleBuffer<T>) -> usize {
    db.front.len()
}

#[allow(dead_code)]
pub fn buffer_is_empty<T>(db: &DoubleBuffer<T>) -> bool {
    db.front.is_empty()
}

#[allow(dead_code)]
pub fn front_buffer<T>(db: &DoubleBuffer<T>) -> &[T] {
    &db.front
}

#[allow(dead_code)]
pub fn back_buffer<T>(db: &DoubleBuffer<T>) -> &[T] {
    &db.back
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let db: DoubleBuffer<i32> = new_double_buffer();
        assert!(buffer_is_empty(&db));
    }

    #[test]
    fn test_write_and_swap() {
        let mut db = new_double_buffer();
        write_buffer(&mut db, 1);
        write_buffer(&mut db, 2);
        swap_buffers(&mut db);
        assert_eq!(read_buffer(&db), &[1, 2]);
    }

    #[test]
    fn test_back_clears_on_swap() {
        let mut db = new_double_buffer();
        write_buffer(&mut db, 10);
        swap_buffers(&mut db);
        assert!(back_buffer(&db).is_empty());
    }

    #[test]
    fn test_buffer_len() {
        let mut db = new_double_buffer();
        write_buffer(&mut db, 'a');
        write_buffer(&mut db, 'b');
        swap_buffers(&mut db);
        assert_eq!(buffer_len(&db), 2);
    }

    #[test]
    fn test_front_buffer() {
        let mut db = new_double_buffer();
        write_buffer(&mut db, 5);
        swap_buffers(&mut db);
        assert_eq!(front_buffer(&db), &[5]);
    }

    #[test]
    fn test_back_buffer_before_swap() {
        let mut db = new_double_buffer();
        write_buffer(&mut db, 7);
        assert_eq!(back_buffer(&db), &[7]);
    }

    #[test]
    fn test_double_swap() {
        let mut db = new_double_buffer();
        write_buffer(&mut db, 1);
        swap_buffers(&mut db);
        write_buffer(&mut db, 2);
        swap_buffers(&mut db);
        assert_eq!(read_buffer(&db), &[2]);
    }

    #[test]
    fn test_empty_after_creation() {
        let db: DoubleBuffer<f32> = new_double_buffer();
        assert_eq!(buffer_len(&db), 0);
    }

    #[test]
    fn test_read_before_swap() {
        let db: DoubleBuffer<i32> = new_double_buffer();
        assert!(read_buffer(&db).is_empty());
    }

    #[test]
    fn test_multiple_writes() {
        let mut db = new_double_buffer();
        for i in 0..5 {
            write_buffer(&mut db, i);
        }
        swap_buffers(&mut db);
        assert_eq!(buffer_len(&db), 5);
    }
}
