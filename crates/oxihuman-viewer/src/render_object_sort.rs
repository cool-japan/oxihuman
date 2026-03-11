#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct SortObject {
    id: u32,
    distance: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderObjectSort {
    objects: Vec<SortObject>,
}

#[allow(dead_code)]
pub fn new_object_sort() -> RenderObjectSort {
    RenderObjectSort { objects: Vec::new() }
}

#[allow(dead_code)]
pub fn add_object_ros(s: &mut RenderObjectSort, id: u32, distance: f32) {
    s.objects.push(SortObject { id, distance });
}

#[allow(dead_code)]
pub fn sort_objects(s: &mut RenderObjectSort) {
    s.objects.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(std::cmp::Ordering::Equal));
}

#[allow(dead_code)]
pub fn sorted_count(s: &RenderObjectSort) -> usize { s.objects.len() }

#[allow(dead_code)]
pub fn sorted_at(s: &RenderObjectSort, idx: usize) -> u32 {
    if idx < s.objects.len() { s.objects[idx].id } else { 0 }
}

#[allow(dead_code)]
pub fn sort_by_distance(s: &mut RenderObjectSort, reverse: bool) {
    if reverse {
        s.objects.sort_by(|a, b| b.distance.partial_cmp(&a.distance).unwrap_or(std::cmp::Ordering::Equal));
    } else {
        sort_objects(s);
    }
}

#[allow(dead_code)]
pub fn sort_to_json(s: &RenderObjectSort) -> String {
    format!("{{\"count\":{}}}", s.objects.len())
}

#[allow(dead_code)]
pub fn clear_object_sort(s: &mut RenderObjectSort) { s.objects.clear(); }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let s = new_object_sort(); assert_eq!(sorted_count(&s), 0); }
    #[test] fn test_add() { let mut s = new_object_sort(); add_object_ros(&mut s, 1, 5.0); assert_eq!(sorted_count(&s), 1); }
    #[test] fn test_sort() { let mut s = new_object_sort(); add_object_ros(&mut s, 1, 10.0); add_object_ros(&mut s, 2, 5.0); sort_objects(&mut s); assert_eq!(sorted_at(&s, 0), 2); }
    #[test] fn test_at_oob() { let s = new_object_sort(); assert_eq!(sorted_at(&s, 0), 0); }
    #[test] fn test_sort_reverse() { let mut s = new_object_sort(); add_object_ros(&mut s, 1, 5.0); add_object_ros(&mut s, 2, 10.0); sort_by_distance(&mut s, true); assert_eq!(sorted_at(&s, 0), 2); }
    #[test] fn test_json() { let s = new_object_sort(); assert!(sort_to_json(&s).contains("count")); }
    #[test] fn test_clear() { let mut s = new_object_sort(); add_object_ros(&mut s, 1, 1.0); clear_object_sort(&mut s); assert_eq!(sorted_count(&s), 0); }
    #[test] fn test_multiple() { let mut s = new_object_sort(); for i in 0..5 { add_object_ros(&mut s, i, i as f32); } assert_eq!(sorted_count(&s), 5); }
    #[test] fn test_sort_stable() { let mut s = new_object_sort(); add_object_ros(&mut s, 1, 1.0); sort_objects(&mut s); assert_eq!(sorted_at(&s, 0), 1); }
    #[test] fn test_already_sorted() { let mut s = new_object_sort(); add_object_ros(&mut s, 1, 1.0); add_object_ros(&mut s, 2, 2.0); sort_objects(&mut s); assert_eq!(sorted_at(&s, 0), 1); }
}
