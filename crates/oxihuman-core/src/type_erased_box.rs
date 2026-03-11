#![allow(dead_code)]

use std::any::Any;

#[allow(dead_code)]
pub struct TypeErasedBox {
    inner: Box<dyn Any>,
    type_name: String,
    size: usize,
}

#[allow(dead_code)]
pub fn new_type_erased<T: Any + 'static>(val: T) -> TypeErasedBox {
    let type_name = std::any::type_name::<T>().to_string();
    let size = std::mem::size_of::<T>();
    TypeErasedBox {
        inner: Box::new(val),
        type_name,
        size,
    }
}

#[allow(dead_code)]
pub fn type_erased_name(b: &TypeErasedBox) -> &str {
    &b.type_name
}

#[allow(dead_code)]
pub fn type_erased_size(b: &TypeErasedBox) -> usize {
    b.size
}

#[allow(dead_code)]
pub fn type_erased_is<T: Any + 'static>(b: &TypeErasedBox) -> bool {
    b.inner.is::<T>()
}

#[allow(dead_code)]
pub fn type_erased_downcast<T: Any + 'static>(b: &TypeErasedBox) -> Option<&T> {
    b.inner.downcast_ref::<T>()
}

#[allow(dead_code)]
pub fn type_erased_clone_stub(b: &TypeErasedBox) -> String {
    format!("clone_stub({})", b.type_name)
}

#[allow(dead_code)]
pub fn type_erased_to_json(b: &TypeErasedBox) -> String {
    format!("{{\"type\":\"{}\",\"size\":{}}}", b.type_name, b.size)
}

#[allow(dead_code)]
pub fn type_erased_drop(b: TypeErasedBox) {
    drop(b);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_i32() {
        let b = new_type_erased(42i32);
        assert!(type_erased_is::<i32>(&b));
    }

    #[test]
    fn test_downcast() {
        let b = new_type_erased(99i32);
        assert_eq!(type_erased_downcast::<i32>(&b), Some(&99));
    }

    #[test]
    fn test_wrong_type() {
        let b = new_type_erased(1.0f64);
        assert!(!type_erased_is::<i32>(&b));
        assert_eq!(type_erased_downcast::<i32>(&b), None);
    }

    #[test]
    fn test_name() {
        let b = new_type_erased(true);
        assert!(type_erased_name(&b).contains("bool"));
    }

    #[test]
    fn test_size() {
        let b = new_type_erased(0u64);
        assert_eq!(type_erased_size(&b), 8);
    }

    #[test]
    fn test_clone_stub() {
        let b = new_type_erased(0u8);
        let s = type_erased_clone_stub(&b);
        assert!(s.contains("clone_stub"));
    }

    #[test]
    fn test_to_json() {
        let b = new_type_erased(0u32);
        let j = type_erased_to_json(&b);
        assert!(j.contains("\"type\":"));
        assert!(j.contains("\"size\":"));
    }

    #[test]
    fn test_drop() {
        let b = new_type_erased(String::from("hello"));
        type_erased_drop(b);
    }

    #[test]
    fn test_string_type() {
        let b = new_type_erased(String::from("test"));
        assert!(type_erased_is::<String>(&b));
        assert_eq!(
            type_erased_downcast::<String>(&b).map(|s| s.as_str()),
            Some("test")
        );
    }

    #[test]
    fn test_zero_size() {
        let b = new_type_erased(());
        assert_eq!(type_erased_size(&b), 0);
    }
}
