#![allow(dead_code)]

/// Chain of expression drivers that feed into each other.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionDriverChain {
    drivers: Vec<(String, f32)>,
    active: bool,
}

#[allow(dead_code)]
pub fn new_driver_chain() -> ExpressionDriverChain {
    ExpressionDriverChain { drivers: Vec::new(), active: true }
}

#[allow(dead_code)]
pub fn add_chain_driver(chain: &mut ExpressionDriverChain, name: &str, scale: f32) {
    chain.drivers.push((name.to_string(), scale));
}

#[allow(dead_code)]
pub fn evaluate_driver_chain(chain: &ExpressionDriverChain, input: f32) -> f32 {
    if !chain.active { return 0.0; }
    let mut val = input;
    for (_, scale) in &chain.drivers { val *= scale; }
    val
}

#[allow(dead_code)]
pub fn driver_chain_count(chain: &ExpressionDriverChain) -> usize { chain.drivers.len() }

#[allow(dead_code)]
pub fn driver_output_dc(chain: &ExpressionDriverChain, idx: usize) -> f32 {
    chain.drivers.get(idx).map(|(_, s)| *s).unwrap_or(0.0)
}

#[allow(dead_code)]
pub fn chain_to_json_dc(chain: &ExpressionDriverChain) -> String {
    let e: Vec<String> = chain.drivers.iter()
        .map(|(n, s)| format!("{{\"name\":\"{}\",\"scale\":{:.4}}}", n, s)).collect();
    format!("{{\"active\":{},\"drivers\":[{}]}}", chain.active, e.join(","))
}

#[allow(dead_code)]
pub fn chain_clear_dc(chain: &mut ExpressionDriverChain) { chain.drivers.clear(); }

#[allow(dead_code)]
pub fn chain_is_active(chain: &ExpressionDriverChain) -> bool { chain.active }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { assert_eq!(driver_chain_count(&new_driver_chain()), 0); }
    #[test] fn test_add() {
        let mut c = new_driver_chain();
        add_chain_driver(&mut c, "d1", 2.0);
        assert_eq!(driver_chain_count(&c), 1);
    }
    #[test] fn test_evaluate() {
        let mut c = new_driver_chain();
        add_chain_driver(&mut c, "d1", 2.0);
        add_chain_driver(&mut c, "d2", 0.5);
        assert!((evaluate_driver_chain(&c, 1.0) - 1.0).abs() < 1e-6);
    }
    #[test] fn test_evaluate_empty() {
        let c = new_driver_chain();
        assert!((evaluate_driver_chain(&c, 5.0) - 5.0).abs() < 1e-6);
    }
    #[test] fn test_inactive() {
        let mut c = new_driver_chain();
        c.active = false;
        assert!((evaluate_driver_chain(&c, 5.0)).abs() < 1e-6);
    }
    #[test] fn test_output() {
        let mut c = new_driver_chain();
        add_chain_driver(&mut c, "x", 3.0);
        assert!((driver_output_dc(&c, 0) - 3.0).abs() < 1e-6);
    }
    #[test] fn test_output_oob() { assert!((driver_output_dc(&new_driver_chain(), 0)).abs() < 1e-6); }
    #[test] fn test_to_json() {
        let mut c = new_driver_chain();
        add_chain_driver(&mut c, "a", 1.0);
        assert!(chain_to_json_dc(&c).contains("active"));
    }
    #[test] fn test_clear() {
        let mut c = new_driver_chain();
        add_chain_driver(&mut c, "x", 1.0);
        chain_clear_dc(&mut c);
        assert_eq!(driver_chain_count(&c), 0);
    }
    #[test] fn test_is_active() { assert!(chain_is_active(&new_driver_chain())); }
    #[test] fn test_chain_multiply() {
        let mut c = new_driver_chain();
        add_chain_driver(&mut c, "a", 2.0);
        add_chain_driver(&mut c, "b", 3.0);
        assert!((evaluate_driver_chain(&c, 1.0) - 6.0).abs() < 1e-6);
    }
}
