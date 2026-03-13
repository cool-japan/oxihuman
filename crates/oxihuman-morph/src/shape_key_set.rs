#![allow(dead_code)]
//! A set of shape keys with weights for blend shape animation.

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ShapeKey {
    pub name: String,
    pub weight: f32,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ShapeKeySet {
    keys: Vec<ShapeKey>,
}

#[allow(dead_code)]
pub fn new_shape_key_set() -> ShapeKeySet {
    ShapeKeySet { keys: Vec::new() }
}

#[allow(dead_code)]
pub fn add_shape_key(set: &mut ShapeKeySet, name: &str, weight: f32) {
    set.keys.push(ShapeKey {
        name: name.to_string(),
        weight: weight.clamp(0.0, 1.0),
    });
}

#[allow(dead_code)]
pub fn get_shape_key<'a>(set: &'a ShapeKeySet, name: &str) -> Option<&'a ShapeKey> {
    set.keys.iter().find(|k| k.name == name)
}

#[allow(dead_code)]
pub fn shape_key_count(set: &ShapeKeySet) -> usize {
    set.keys.len()
}

#[allow(dead_code)]
pub fn shape_key_weight(set: &ShapeKeySet, name: &str) -> Option<f32> {
    set.keys.iter().find(|k| k.name == name).map(|k| k.weight)
}

#[allow(dead_code)]
pub fn set_shape_key_weight(set: &mut ShapeKeySet, name: &str, weight: f32) -> bool {
    if let Some(k) = set.keys.iter_mut().find(|k| k.name == name) {
        k.weight = weight.clamp(0.0, 1.0);
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn shape_keys_to_json(set: &ShapeKeySet) -> String {
    let entries: Vec<String> = set
        .keys
        .iter()
        .map(|k| format!("{{\"name\":\"{}\",\"weight\":{}}}", k.name, k.weight))
        .collect();
    format!("[{}]", entries.join(","))
}

#[allow(dead_code)]
pub fn reset_all_shape_keys(set: &mut ShapeKeySet) {
    for k in &mut set.keys {
        k.weight = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_shape_key_set() {
        let s = new_shape_key_set();
        assert_eq!(shape_key_count(&s), 0);
    }

    #[test]
    fn test_add_shape_key() {
        let mut s = new_shape_key_set();
        add_shape_key(&mut s, "smile", 0.5);
        assert_eq!(shape_key_count(&s), 1);
    }

    #[test]
    fn test_get_shape_key() {
        let mut s = new_shape_key_set();
        add_shape_key(&mut s, "blink", 0.3);
        assert!(get_shape_key(&s, "blink").is_some());
        assert!(get_shape_key(&s, "nope").is_none());
    }

    #[test]
    fn test_shape_key_weight() {
        let mut s = new_shape_key_set();
        add_shape_key(&mut s, "w", 0.7);
        assert!((shape_key_weight(&s, "w").expect("should succeed") - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_shape_key_weight() {
        let mut s = new_shape_key_set();
        add_shape_key(&mut s, "w", 0.1);
        assert!(set_shape_key_weight(&mut s, "w", 0.9));
        assert!((shape_key_weight(&s, "w").expect("should succeed") - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_set_missing_key_weight() {
        let mut s = new_shape_key_set();
        assert!(!set_shape_key_weight(&mut s, "nope", 0.5));
    }

    #[test]
    fn test_shape_keys_to_json() {
        let mut s = new_shape_key_set();
        add_shape_key(&mut s, "a", 0.2);
        let json = shape_keys_to_json(&s);
        assert!(json.contains("\"name\":\"a\""));
    }

    #[test]
    fn test_reset_all_shape_keys() {
        let mut s = new_shape_key_set();
        add_shape_key(&mut s, "a", 0.5);
        add_shape_key(&mut s, "b", 0.8);
        reset_all_shape_keys(&mut s);
        assert!((shape_key_weight(&s, "a").expect("should succeed")).abs() < 1e-6);
        assert!((shape_key_weight(&s, "b").expect("should succeed")).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_weight() {
        let mut s = new_shape_key_set();
        add_shape_key(&mut s, "x", 5.0);
        assert!((shape_key_weight(&s, "x").expect("should succeed") - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_multiple_keys() {
        let mut s = new_shape_key_set();
        for i in 0..8 {
            add_shape_key(&mut s, &format!("k{i}"), 0.1);
        }
        assert_eq!(shape_key_count(&s), 8);
    }
}
