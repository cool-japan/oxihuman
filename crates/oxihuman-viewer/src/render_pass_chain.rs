#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct ChainPass {
    name: String,
    deps: Vec<usize>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderPassChain {
    passes: Vec<ChainPass>,
}

#[allow(dead_code)]
pub fn new_pass_chain() -> RenderPassChain {
    RenderPassChain { passes: Vec::new() }
}

#[allow(dead_code)]
pub fn add_pass_to_chain(chain: &mut RenderPassChain, name: &str, deps: &[usize]) -> usize {
    let idx = chain.passes.len();
    chain.passes.push(ChainPass { name: name.to_string(), deps: deps.to_vec() });
    idx
}

#[allow(dead_code)]
pub fn chain_pass_count(chain: &RenderPassChain) -> usize { chain.passes.len() }

#[allow(dead_code)]
pub fn execute_chain_rpc(chain: &RenderPassChain) -> Vec<String> {
    chain.passes.iter().map(|p| p.name.clone()).collect()
}

#[allow(dead_code)]
pub fn chain_to_json(chain: &RenderPassChain) -> String {
    format!("{{\"pass_count\":{}}}", chain.passes.len())
}

#[allow(dead_code)]
pub fn chain_clear_rpc(chain: &mut RenderPassChain) { chain.passes.clear(); }

#[allow(dead_code)]
pub fn chain_validate(chain: &RenderPassChain) -> bool {
    for (i, pass) in chain.passes.iter().enumerate() {
        for &d in &pass.deps {
            if d >= i { return false; }
        }
    }
    true
}

#[allow(dead_code)]
pub fn chain_dependencies(chain: &RenderPassChain, idx: usize) -> Vec<usize> {
    if idx < chain.passes.len() { chain.passes[idx].deps.clone() } else { Vec::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let c = new_pass_chain(); assert_eq!(chain_pass_count(&c), 0); }
    #[test] fn test_add() { let mut c = new_pass_chain(); add_pass_to_chain(&mut c, "depth", &[]); assert_eq!(chain_pass_count(&c), 1); }
    #[test] fn test_execute() { let mut c = new_pass_chain(); add_pass_to_chain(&mut c, "a", &[]); let r = execute_chain_rpc(&c); assert_eq!(r[0], "a"); }
    #[test] fn test_json() { let c = new_pass_chain(); assert!(chain_to_json(&c).contains("pass_count")); }
    #[test] fn test_clear() { let mut c = new_pass_chain(); add_pass_to_chain(&mut c, "x", &[]); chain_clear_rpc(&mut c); assert_eq!(chain_pass_count(&c), 0); }
    #[test] fn test_validate_empty() { let c = new_pass_chain(); assert!(chain_validate(&c)); }
    #[test] fn test_validate_ok() { let mut c = new_pass_chain(); add_pass_to_chain(&mut c, "a", &[]); add_pass_to_chain(&mut c, "b", &[0]); assert!(chain_validate(&c)); }
    #[test] fn test_validate_bad() { let mut c = new_pass_chain(); add_pass_to_chain(&mut c, "a", &[1]); assert!(!chain_validate(&c)); }
    #[test] fn test_deps() { let mut c = new_pass_chain(); add_pass_to_chain(&mut c, "a", &[]); add_pass_to_chain(&mut c, "b", &[0]); assert_eq!(chain_dependencies(&c, 1), vec![0]); }
    #[test] fn test_deps_oob() { let c = new_pass_chain(); assert!(chain_dependencies(&c, 5).is_empty()); }
}
