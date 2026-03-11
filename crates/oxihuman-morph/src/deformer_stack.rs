#![allow(dead_code)]

//! Ordered deformer stack with enable/disable.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DeformerEntry {
    pub name: String,
    pub deformer_type: String,
    pub enabled: bool,
    pub order: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DeformerStack {
    pub entries: Vec<DeformerEntry>,
}

#[allow(dead_code)]
pub fn new_deformer_stack() -> DeformerStack {
    DeformerStack {
        entries: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn ds_push(stack: &mut DeformerStack, name: &str, deformer_type: &str) {
    let order = stack.entries.len() as u32;
    stack.entries.push(DeformerEntry {
        name: name.to_string(),
        deformer_type: deformer_type.to_string(),
        enabled: true,
        order,
    });
}

#[allow(dead_code)]
pub fn ds_enable(stack: &mut DeformerStack, name: &str, enabled: bool) {
    if let Some(e) = stack.entries.iter_mut().find(|e| e.name == name) {
        e.enabled = enabled;
    }
}

#[allow(dead_code)]
pub fn ds_remove(stack: &mut DeformerStack, name: &str) {
    stack.entries.retain(|e| e.name != name);
}

#[allow(dead_code)]
pub fn ds_count(stack: &DeformerStack) -> usize {
    stack.entries.len()
}

#[allow(dead_code)]
pub fn ds_active_count(stack: &DeformerStack) -> usize {
    stack.entries.iter().filter(|e| e.enabled).count()
}

#[allow(dead_code)]
pub fn ds_move_up(stack: &mut DeformerStack, name: &str) -> bool {
    if let Some(idx) = stack.entries.iter().position(|e| e.name == name) {
        if idx > 0 {
            stack.entries.swap(idx, idx - 1);
            return true;
        }
    }
    false
}

#[allow(dead_code)]
pub fn ds_move_down(stack: &mut DeformerStack, name: &str) -> bool {
    if let Some(idx) = stack.entries.iter().position(|e| e.name == name) {
        if idx + 1 < stack.entries.len() {
            stack.entries.swap(idx, idx + 1);
            return true;
        }
    }
    false
}

#[allow(dead_code)]
pub fn ds_clear(stack: &mut DeformerStack) {
    stack.entries.clear();
}

#[allow(dead_code)]
pub fn ds_has(stack: &DeformerStack, name: &str) -> bool {
    stack.entries.iter().any(|e| e.name == name)
}

#[allow(dead_code)]
pub fn ds_to_json(stack: &DeformerStack) -> String {
    format!(
        "{{\"count\":{},\"active_count\":{}}}",
        stack.entries.len(),
        ds_active_count(stack)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stack() {
        let s = new_deformer_stack();
        assert_eq!(ds_count(&s), 0);
    }

    #[test]
    fn test_push() {
        let mut s = new_deformer_stack();
        ds_push(&mut s, "smooth", "laplacian");
        assert_eq!(ds_count(&s), 1);
    }

    #[test]
    fn test_enable_disable() {
        let mut s = new_deformer_stack();
        ds_push(&mut s, "smooth", "laplacian");
        ds_enable(&mut s, "smooth", false);
        assert_eq!(ds_active_count(&s), 0);
    }

    #[test]
    fn test_remove() {
        let mut s = new_deformer_stack();
        ds_push(&mut s, "twist", "twist");
        ds_remove(&mut s, "twist");
        assert!(!ds_has(&s, "twist"));
    }

    #[test]
    fn test_move_up() {
        let mut s = new_deformer_stack();
        ds_push(&mut s, "a", "t");
        ds_push(&mut s, "b", "t");
        ds_move_up(&mut s, "b");
        assert_eq!(s.entries[0].name, "b");
    }

    #[test]
    fn test_move_down() {
        let mut s = new_deformer_stack();
        ds_push(&mut s, "a", "t");
        ds_push(&mut s, "b", "t");
        ds_move_down(&mut s, "a");
        assert_eq!(s.entries[1].name, "a");
    }

    #[test]
    fn test_move_up_at_top() {
        let mut s = new_deformer_stack();
        ds_push(&mut s, "a", "t");
        let moved = ds_move_up(&mut s, "a");
        assert!(!moved);
    }

    #[test]
    fn test_clear() {
        let mut s = new_deformer_stack();
        ds_push(&mut s, "a", "t");
        ds_clear(&mut s);
        assert_eq!(ds_count(&s), 0);
    }

    #[test]
    fn test_to_json() {
        let s = new_deformer_stack();
        let json = ds_to_json(&s);
        assert!(json.contains("active_count"));
    }

    #[test]
    fn test_active_count_all_enabled() {
        let mut s = new_deformer_stack();
        ds_push(&mut s, "a", "t");
        ds_push(&mut s, "b", "t");
        assert_eq!(ds_active_count(&s), 2);
    }
}
