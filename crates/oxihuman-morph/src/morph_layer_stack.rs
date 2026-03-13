#![allow(dead_code)]

/// A single entry in the morph layer stack.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphLayerEntry {
    pub name: String,
    pub weight: f32,
    pub values: Vec<f32>,
}

/// Stack-based morph layer system.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphLayerStack {
    layers: Vec<MorphLayerEntry>,
}

#[allow(dead_code)]
pub fn new_morph_layer_stack() -> MorphLayerStack {
    MorphLayerStack { layers: Vec::new() }
}

#[allow(dead_code)]
pub fn push_morph_layer(stack: &mut MorphLayerStack, name: &str, weight: f32, values: &[f32]) {
    stack.layers.push(MorphLayerEntry {
        name: name.to_string(),
        weight,
        values: values.to_vec(),
    });
}

#[allow(dead_code)]
pub fn pop_morph_layer(stack: &mut MorphLayerStack) -> Option<MorphLayerEntry> {
    stack.layers.pop()
}

#[allow(dead_code)]
pub fn evaluate_morph_stack(stack: &MorphLayerStack, out: &mut [f32]) {
    for v in out.iter_mut() {
        *v = 0.0;
    }
    for layer in &stack.layers {
        for (i, val) in layer.values.iter().enumerate() {
            if i < out.len() {
                out[i] += val * layer.weight;
            }
        }
    }
}

#[allow(dead_code)]
pub fn morph_stack_count(stack: &MorphLayerStack) -> usize {
    stack.layers.len()
}

#[allow(dead_code)]
pub fn morph_layer_at(stack: &MorphLayerStack, index: usize) -> Option<&MorphLayerEntry> {
    stack.layers.get(index)
}

#[allow(dead_code)]
pub fn morph_stack_to_json(stack: &MorphLayerStack) -> String {
    let entries: Vec<String> = stack.layers.iter().map(|l| {
        format!("{{\"name\":\"{}\",\"weight\":{:.4}}}", l.name, l.weight)
    }).collect();
    format!("{{\"layers\":[{}]}}", entries.join(","))
}

#[allow(dead_code)]
pub fn morph_stack_clear(stack: &mut MorphLayerStack) {
    stack.layers.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stack() {
        let s = new_morph_layer_stack();
        assert_eq!(morph_stack_count(&s), 0);
    }

    #[test]
    fn test_push_pop() {
        let mut s = new_morph_layer_stack();
        push_morph_layer(&mut s, "base", 1.0, &[1.0, 2.0]);
        assert_eq!(morph_stack_count(&s), 1);
        let popped = pop_morph_layer(&mut s);
        assert!(popped.is_some());
        assert_eq!(morph_stack_count(&s), 0);
    }

    #[test]
    fn test_evaluate() {
        let mut s = new_morph_layer_stack();
        push_morph_layer(&mut s, "a", 0.5, &[2.0, 4.0]);
        push_morph_layer(&mut s, "b", 1.0, &[1.0, 1.0]);
        let mut out = [0.0f32; 2];
        evaluate_morph_stack(&s, &mut out);
        assert!((out[0] - 2.0).abs() < 1e-6);
        assert!((out[1] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_layer_at() {
        let mut s = new_morph_layer_stack();
        push_morph_layer(&mut s, "layer0", 1.0, &[0.0]);
        let entry = morph_layer_at(&s, 0).expect("should succeed");
        assert_eq!(entry.name, "layer0");
    }

    #[test]
    fn test_layer_at_out_of_bounds() {
        let s = new_morph_layer_stack();
        assert!(morph_layer_at(&s, 0).is_none());
    }

    #[test]
    fn test_to_json() {
        let mut s = new_morph_layer_stack();
        push_morph_layer(&mut s, "x", 0.5, &[1.0]);
        let j = morph_stack_to_json(&s);
        assert!(j.contains("\"name\":\"x\""));
    }

    #[test]
    fn test_clear() {
        let mut s = new_morph_layer_stack();
        push_morph_layer(&mut s, "a", 1.0, &[1.0]);
        push_morph_layer(&mut s, "b", 1.0, &[2.0]);
        morph_stack_clear(&mut s);
        assert_eq!(morph_stack_count(&s), 0);
    }

    #[test]
    fn test_pop_empty() {
        let mut s = new_morph_layer_stack();
        assert!(pop_morph_layer(&mut s).is_none());
    }

    #[test]
    fn test_evaluate_empty() {
        let s = new_morph_layer_stack();
        let mut out = [5.0f32; 3];
        evaluate_morph_stack(&s, &mut out);
        assert!((out[0]).abs() < 1e-6);
    }

    #[test]
    fn test_multiple_layers_ordering() {
        let mut s = new_morph_layer_stack();
        push_morph_layer(&mut s, "first", 1.0, &[10.0]);
        push_morph_layer(&mut s, "second", 1.0, &[20.0]);
        assert_eq!(morph_layer_at(&s, 0).expect("should succeed").name, "first");
        assert_eq!(morph_layer_at(&s, 1).expect("should succeed").name, "second");
    }
}
