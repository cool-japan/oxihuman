// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

use std::collections::HashMap;

/// A parameter set (name → value mapping)
pub type ParamMap = HashMap<String, f32>;

/// Blend mode for combining two param sets
#[derive(Clone, Debug)]
pub enum BlendMode {
    /// Weighted average: result = a * (1-t) + b * t
    Lerp,
    /// Additive: result = a + b * t
    Additive,
    /// Override: result = a for t<0.5, else b
    Override,
    /// Multiply: result = a * b (t ignored)
    Multiply,
}

/// A node in the blend tree
#[derive(Clone, Debug)]
pub enum BlendNode {
    /// Leaf node: a named parameter set
    Params { name: String, params: ParamMap },
    /// Binary blend: mix two children
    Blend {
        mode: BlendMode,
        weight: f32,
        left: Box<BlendNode>,
        right: Box<BlendNode>,
    },
    /// Scale all values in child by a factor
    Scale { factor: f32, child: Box<BlendNode> },
    /// Clamp all values in child to [min, max]
    Clamp {
        min: f32,
        max: f32,
        child: Box<BlendNode>,
    },
    /// Select one of N children by index
    Select {
        index: usize,
        children: Vec<BlendNode>,
    },
}

impl BlendNode {
    /// Evaluate the node, returning a parameter map
    pub fn evaluate(&self) -> ParamMap {
        match self {
            BlendNode::Params { params, .. } => params.clone(),

            BlendNode::Blend {
                mode,
                weight,
                left,
                right,
            } => {
                let left_result = left.evaluate();
                let right_result = right.evaluate();
                blend_params(&left_result, &right_result, *weight, mode)
            }

            BlendNode::Scale { factor, child } => {
                let result = child.evaluate();
                scale_params(&result, *factor)
            }

            BlendNode::Clamp { min, max, child } => {
                let result = child.evaluate();
                clamp_params(&result, *min, *max)
            }

            BlendNode::Select { index, children } => {
                if children.is_empty() {
                    ParamMap::new()
                } else {
                    let i = index % children.len();
                    children[i].evaluate()
                }
            }
        }
    }

    /// Leaf constructor
    pub fn leaf(name: impl Into<String>, params: ParamMap) -> Self {
        BlendNode::Params {
            name: name.into(),
            params,
        }
    }

    /// Lerp blend constructor
    pub fn lerp(weight: f32, left: BlendNode, right: BlendNode) -> Self {
        BlendNode::Blend {
            mode: BlendMode::Lerp,
            weight,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// Additive blend constructor
    pub fn additive(weight: f32, base: BlendNode, addon: BlendNode) -> Self {
        BlendNode::Blend {
            mode: BlendMode::Additive,
            weight,
            left: Box::new(base),
            right: Box::new(addon),
        }
    }

    /// Scale constructor
    pub fn scale(factor: f32, child: BlendNode) -> Self {
        BlendNode::Scale {
            factor,
            child: Box::new(child),
        }
    }

    /// Clamp constructor
    pub fn clamp(min: f32, max: f32, child: BlendNode) -> Self {
        BlendNode::Clamp {
            min,
            max,
            child: Box::new(child),
        }
    }

    /// Select constructor
    pub fn select(index: usize, children: Vec<BlendNode>) -> Self {
        BlendNode::Select { index, children }
    }

    /// Depth of this subtree
    pub fn depth(&self) -> usize {
        match self {
            BlendNode::Params { .. } => 1,
            BlendNode::Blend { left, right, .. } => 1 + left.depth().max(right.depth()),
            BlendNode::Scale { child, .. } => 1 + child.depth(),
            BlendNode::Clamp { child, .. } => 1 + child.depth(),
            BlendNode::Select { children, .. } => {
                let max_child = children.iter().map(|c| c.depth()).max().unwrap_or(0);
                1 + max_child
            }
        }
    }

    /// Count of leaf nodes
    pub fn leaf_count(&self) -> usize {
        match self {
            BlendNode::Params { .. } => 1,
            BlendNode::Blend { left, right, .. } => left.leaf_count() + right.leaf_count(),
            BlendNode::Scale { child, .. } => child.leaf_count(),
            BlendNode::Clamp { child, .. } => child.leaf_count(),
            BlendNode::Select { children, .. } => children.iter().map(|c| c.leaf_count()).sum(),
        }
    }
}

/// Blend two param maps
pub fn blend_params(a: &ParamMap, b: &ParamMap, weight: f32, mode: &BlendMode) -> ParamMap {
    match mode {
        BlendMode::Lerp => {
            let all_keys: std::collections::HashSet<&String> = a.keys().chain(b.keys()).collect();
            all_keys
                .into_iter()
                .map(|k| {
                    let a_val = *a.get(k).unwrap_or(&0.0);
                    let b_val = *b.get(k).unwrap_or(&0.0);
                    let val = a_val * (1.0 - weight) + b_val * weight;
                    (k.clone(), val)
                })
                .collect()
        }

        BlendMode::Additive => {
            let all_keys: std::collections::HashSet<&String> = a.keys().chain(b.keys()).collect();
            all_keys
                .into_iter()
                .map(|k| {
                    let a_val = *a.get(k).unwrap_or(&0.0);
                    let b_val = *b.get(k).unwrap_or(&0.0);
                    let val = a_val + b_val * weight;
                    (k.clone(), val)
                })
                .collect()
        }

        BlendMode::Override => {
            if weight < 0.5 {
                a.clone()
            } else {
                b.clone()
            }
        }

        BlendMode::Multiply => {
            let all_keys: std::collections::HashSet<&String> = a.keys().chain(b.keys()).collect();
            all_keys
                .into_iter()
                .map(|k| {
                    let a_val = *a.get(k).unwrap_or(&0.0);
                    let b_val = *b.get(k).unwrap_or(&0.0);
                    let val = a_val * b_val;
                    (k.clone(), val)
                })
                .collect()
        }
    }
}

/// Merge all keys from both maps (union), using value from a for keys only in a, b for b-only.
/// When a key exists in both, a wins.
pub fn merge_params(a: &ParamMap, b: &ParamMap) -> ParamMap {
    let mut result = b.clone();
    for (k, v) in a {
        result.insert(k.clone(), *v);
    }
    result
}

/// Scale all values in a param map
pub fn scale_params(params: &ParamMap, factor: f32) -> ParamMap {
    params
        .iter()
        .map(|(k, v)| (k.clone(), v * factor))
        .collect()
}

/// Clamp all values in a param map
pub fn clamp_params(params: &ParamMap, min: f32, max: f32) -> ParamMap {
    params
        .iter()
        .map(|(k, v)| (k.clone(), v.clamp(min, max)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_params(pairs: &[(&str, f32)]) -> ParamMap {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn test_leaf_evaluate() {
        let params = make_params(&[("height", 1.8), ("weight", 75.0)]);
        let node = BlendNode::leaf("base", params.clone());
        let result = node.evaluate();
        assert_eq!(result.get("height"), Some(&1.8));
        assert_eq!(result.get("weight"), Some(&75.0));
    }

    #[test]
    fn test_lerp_blend_zero() {
        let a = make_params(&[("x", 0.0)]);
        let b = make_params(&[("x", 10.0)]);
        let node = BlendNode::lerp(0.0, BlendNode::leaf("a", a), BlendNode::leaf("b", b));
        let result = node.evaluate();
        let x = result["x"];
        assert!((x - 0.0).abs() < 1e-6, "Expected 0.0, got {x}");
    }

    #[test]
    fn test_lerp_blend_one() {
        let a = make_params(&[("x", 0.0)]);
        let b = make_params(&[("x", 10.0)]);
        let node = BlendNode::lerp(1.0, BlendNode::leaf("a", a), BlendNode::leaf("b", b));
        let result = node.evaluate();
        let x = result["x"];
        assert!((x - 10.0).abs() < 1e-6, "Expected 10.0, got {x}");
    }

    #[test]
    fn test_lerp_blend_half() {
        let a = make_params(&[("x", 0.0)]);
        let b = make_params(&[("x", 10.0)]);
        let node = BlendNode::lerp(0.5, BlendNode::leaf("a", a), BlendNode::leaf("b", b));
        let result = node.evaluate();
        let x = result["x"];
        assert!((x - 5.0).abs() < 1e-6, "Expected 5.0, got {x}");
    }

    #[test]
    fn test_additive_blend() {
        let base = make_params(&[("x", 3.0)]);
        let addon = make_params(&[("x", 2.0)]);
        // weight=0.5: result = 3 + 2*0.5 = 4
        let node = BlendNode::additive(
            0.5,
            BlendNode::leaf("base", base),
            BlendNode::leaf("addon", addon),
        );
        let result = node.evaluate();
        let x = result["x"];
        assert!((x - 4.0).abs() < 1e-6, "Expected 4.0, got {x}");
    }

    #[test]
    fn test_override_blend() {
        let a = make_params(&[("x", 1.0)]);
        let b = make_params(&[("x", 9.0)]);
        // weight < 0.5 → return a
        let node_a = BlendNode::lerp(
            0.3,
            BlendNode::leaf("a", a.clone()),
            BlendNode::leaf("b", b.clone()),
        );
        // Use Override mode directly via Blend variant
        let node_over_a = BlendNode::Blend {
            mode: BlendMode::Override,
            weight: 0.3,
            left: Box::new(BlendNode::leaf("a", a.clone())),
            right: Box::new(BlendNode::leaf("b", b.clone())),
        };
        let node_over_b = BlendNode::Blend {
            mode: BlendMode::Override,
            weight: 0.7,
            left: Box::new(BlendNode::leaf("a", a.clone())),
            right: Box::new(BlendNode::leaf("b", b.clone())),
        };
        // lerp node just to suppress warning
        let _ = node_a.evaluate();
        let result_a = node_over_a.evaluate();
        let result_b = node_over_b.evaluate();
        assert!((result_a["x"] - 1.0).abs() < 1e-6);
        assert!((result_b["x"] - 9.0).abs() < 1e-6);
    }

    #[test]
    fn test_multiply_blend() {
        let a = make_params(&[("x", 3.0)]);
        let b = make_params(&[("x", 4.0)]);
        let node = BlendNode::Blend {
            mode: BlendMode::Multiply,
            weight: 0.5, // ignored
            left: Box::new(BlendNode::leaf("a", a)),
            right: Box::new(BlendNode::leaf("b", b)),
        };
        let result = node.evaluate();
        let x = result["x"];
        assert!((x - 12.0).abs() < 1e-6, "Expected 12.0, got {x}");
    }

    #[test]
    fn test_scale_node() {
        let params = make_params(&[("x", 5.0), ("y", 2.0)]);
        let node = BlendNode::scale(3.0, BlendNode::leaf("base", params));
        let result = node.evaluate();
        assert!((result["x"] - 15.0).abs() < 1e-6);
        assert!((result["y"] - 6.0).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_node() {
        let params = make_params(&[("x", -5.0), ("y", 15.0), ("z", 0.5)]);
        let node = BlendNode::clamp(0.0, 1.0, BlendNode::leaf("base", params));
        let result = node.evaluate();
        assert!((result["x"] - 0.0).abs() < 1e-6);
        assert!((result["y"] - 1.0).abs() < 1e-6);
        assert!((result["z"] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_select_node() {
        let c0 = BlendNode::leaf("c0", make_params(&[("v", 1.0)]));
        let c1 = BlendNode::leaf("c1", make_params(&[("v", 2.0)]));
        let c2 = BlendNode::leaf("c2", make_params(&[("v", 3.0)]));

        let node = BlendNode::select(1, vec![c0, c1, c2]);
        let result = node.evaluate();
        assert!((result["v"] - 2.0).abs() < 1e-6);

        // Test wrapping: index 4 % 3 = 1
        let c0b = BlendNode::leaf("c0", make_params(&[("v", 1.0)]));
        let c1b = BlendNode::leaf("c1", make_params(&[("v", 2.0)]));
        let c2b = BlendNode::leaf("c2", make_params(&[("v", 3.0)]));
        let node2 = BlendNode::select(4, vec![c0b, c1b, c2b]);
        let result2 = node2.evaluate();
        assert!((result2["v"] - 2.0).abs() < 1e-6);

        // Empty children → empty map
        let node3 = BlendNode::select(0, vec![]);
        let result3 = node3.evaluate();
        assert!(result3.is_empty());
    }

    #[test]
    fn test_blend_params_missing_key() {
        let a = make_params(&[("x", 4.0)]);
        let b = make_params(&[("y", 6.0)]);
        let result = blend_params(&a, &b, 0.5, &BlendMode::Lerp);
        // x: 4*0.5 + 0*0.5 = 2.0
        // y: 0*0.5 + 6*0.5 = 3.0
        assert!((result["x"] - 2.0).abs() < 1e-6);
        assert!((result["y"] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_merge_params() {
        let a = make_params(&[("x", 1.0), ("shared", 10.0)]);
        let b = make_params(&[("y", 2.0), ("shared", 99.0)]);
        let result = merge_params(&a, &b);
        // a wins on shared key
        assert!((result["x"] - 1.0).abs() < 1e-6);
        assert!((result["y"] - 2.0).abs() < 1e-6);
        assert!((result["shared"] - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_depth() {
        let leaf = BlendNode::leaf("l", make_params(&[("x", 1.0)]));
        assert_eq!(leaf.depth(), 1);

        let leaf2 = BlendNode::leaf("l2", make_params(&[("x", 2.0)]));
        let blend = BlendNode::lerp(0.5, leaf, leaf2);
        assert_eq!(blend.depth(), 2);

        let leaf3 = BlendNode::leaf("l3", make_params(&[("x", 3.0)]));
        let scaled = BlendNode::scale(1.0, leaf3);
        assert_eq!(scaled.depth(), 2);

        // deeper: blend of (blend of leaves) and leaf → depth 3
        let la = BlendNode::leaf("a", make_params(&[("x", 0.0)]));
        let lb = BlendNode::leaf("b", make_params(&[("x", 1.0)]));
        let lc = BlendNode::leaf("c", make_params(&[("x", 2.0)]));
        let inner = BlendNode::lerp(0.5, la, lb);
        let outer = BlendNode::lerp(0.5, inner, lc);
        assert_eq!(outer.depth(), 3);
    }

    #[test]
    fn test_leaf_count() {
        let leaf = BlendNode::leaf("l", make_params(&[("x", 1.0)]));
        assert_eq!(leaf.leaf_count(), 1);

        let la = BlendNode::leaf("a", make_params(&[("x", 0.0)]));
        let lb = BlendNode::leaf("b", make_params(&[("x", 1.0)]));
        let blend = BlendNode::lerp(0.5, la, lb);
        assert_eq!(blend.leaf_count(), 2);

        let c0 = BlendNode::leaf("c0", make_params(&[("v", 1.0)]));
        let c1 = BlendNode::leaf("c1", make_params(&[("v", 2.0)]));
        let c2 = BlendNode::leaf("c2", make_params(&[("v", 3.0)]));
        let sel = BlendNode::select(0, vec![c0, c1, c2]);
        assert_eq!(sel.leaf_count(), 3);
    }

    #[test]
    fn test_nested_blend() {
        // Build a tree: clamp(scale(lerp(leaf_a, leaf_b, 0.5), 2.0), 0.0, 5.0)
        // leaf_a: x=1, leaf_b: x=3 → lerp(0.5) → x=2 → scale(2) → x=4 → clamp(0,5) → x=4
        let a = BlendNode::leaf("a", make_params(&[("x", 1.0)]));
        let b = BlendNode::leaf("b", make_params(&[("x", 3.0)]));
        let blended = BlendNode::lerp(0.5, a, b);
        let scaled = BlendNode::scale(2.0, blended);
        let clamped = BlendNode::clamp(0.0, 5.0, scaled);
        let result = clamped.evaluate();
        assert!(
            (result["x"] - 4.0).abs() < 1e-6,
            "Expected 4.0, got {}",
            result["x"]
        );

        // Also verify leaf count and depth
        assert_eq!(clamped.depth(), 4);
        assert_eq!(clamped.leaf_count(), 2);
    }
}
