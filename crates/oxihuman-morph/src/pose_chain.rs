#![allow(dead_code)]
//! Pose chain: a sequence of linked pose transforms evaluated in order.

/// A single link in a pose chain.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChainLink {
    pub name: String,
    pub weight: f32,
    pub offset: [f32; 3],
}

/// A chain of pose links.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PoseChain {
    links: Vec<ChainLink>,
}

/// Create a new empty pose chain.
#[allow(dead_code)]
pub fn new_pose_chain() -> PoseChain {
    PoseChain { links: Vec::new() }
}

/// Add a link to the chain.
#[allow(dead_code)]
pub fn add_chain_link(chain: &mut PoseChain, name: &str, weight: f32, offset: [f32; 3]) {
    chain.links.push(ChainLink {
        name: name.to_string(),
        weight,
        offset,
    });
}

/// Evaluate the chain, returning accumulated offset weighted by each link.
#[allow(dead_code)]
pub fn evaluate_chain(chain: &PoseChain) -> [f32; 3] {
    let mut result = [0.0f32; 3];
    for link in &chain.links {
        result[0] += link.offset[0] * link.weight;
        result[1] += link.offset[1] * link.weight;
        result[2] += link.offset[2] * link.weight;
    }
    result
}

/// Return the number of links.
#[allow(dead_code)]
pub fn chain_length(chain: &PoseChain) -> usize {
    chain.links.len()
}

/// Return a reference to the link at `index`.
#[allow(dead_code)]
pub fn chain_link_at(chain: &PoseChain, index: usize) -> Option<&ChainLink> {
    chain.links.get(index)
}

/// Serialize to JSON-like string.
#[allow(dead_code)]
pub fn chain_to_json(chain: &PoseChain) -> String {
    let entries: Vec<String> = chain
        .links
        .iter()
        .map(|l| {
            format!(
                "{{\"name\":\"{}\",\"weight\":{},\"offset\":[{},{},{}]}}",
                l.name, l.weight, l.offset[0], l.offset[1], l.offset[2]
            )
        })
        .collect();
    format!("{{\"links\":[{}]}}", entries.join(","))
}

/// Remove all links.
#[allow(dead_code)]
pub fn chain_clear(chain: &mut PoseChain) {
    chain.links.clear();
}

/// Reverse the order of links.
#[allow(dead_code)]
pub fn chain_reverse(chain: &mut PoseChain) {
    chain.links.reverse();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_chain() {
        let c = new_pose_chain();
        assert_eq!(chain_length(&c), 0);
    }

    #[test]
    fn test_add_link() {
        let mut c = new_pose_chain();
        add_chain_link(&mut c, "hip", 1.0, [0.0, 1.0, 0.0]);
        assert_eq!(chain_length(&c), 1);
    }

    #[test]
    fn test_evaluate_empty() {
        let c = new_pose_chain();
        let r = evaluate_chain(&c);
        assert!((r[0]).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_single() {
        let mut c = new_pose_chain();
        add_chain_link(&mut c, "a", 0.5, [2.0, 0.0, 0.0]);
        let r = evaluate_chain(&c);
        assert!((r[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_multiple() {
        let mut c = new_pose_chain();
        add_chain_link(&mut c, "a", 1.0, [1.0, 0.0, 0.0]);
        add_chain_link(&mut c, "b", 1.0, [0.0, 1.0, 0.0]);
        let r = evaluate_chain(&c);
        assert!((r[0] - 1.0).abs() < 1e-6);
        assert!((r[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_chain_link_at() {
        let mut c = new_pose_chain();
        add_chain_link(&mut c, "knee", 1.0, [0.0, 0.0, 0.0]);
        assert_eq!(chain_link_at(&c, 0).expect("should succeed").name, "knee");
        assert!(chain_link_at(&c, 99).is_none());
    }

    #[test]
    fn test_to_json() {
        let c = new_pose_chain();
        let json = chain_to_json(&c);
        assert!(json.contains("\"links\":[]"));
    }

    #[test]
    fn test_clear() {
        let mut c = new_pose_chain();
        add_chain_link(&mut c, "a", 1.0, [0.0, 0.0, 0.0]);
        chain_clear(&mut c);
        assert_eq!(chain_length(&c), 0);
    }

    #[test]
    fn test_reverse() {
        let mut c = new_pose_chain();
        add_chain_link(&mut c, "first", 1.0, [0.0, 0.0, 0.0]);
        add_chain_link(&mut c, "second", 1.0, [0.0, 0.0, 0.0]);
        chain_reverse(&mut c);
        assert_eq!(chain_link_at(&c, 0).expect("should succeed").name, "second");
    }

    #[test]
    fn test_weighted_offset() {
        let mut c = new_pose_chain();
        add_chain_link(&mut c, "a", 0.0, [100.0, 100.0, 100.0]);
        let r = evaluate_chain(&c);
        assert!((r[0]).abs() < 1e-6);
    }
}
