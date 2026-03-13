#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

/// A step in a task chain.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChainStep {
    name: String,
    completed: bool,
}

/// A chain of sequential tasks.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TaskChain {
    name: String,
    steps: Vec<ChainStep>,
    current: usize,
}

#[allow(dead_code)]
pub fn new_task_chain(name: &str) -> TaskChain {
    TaskChain {
        name: name.to_string(),
        steps: Vec::new(),
        current: 0,
    }
}

#[allow(dead_code)]
pub fn add_step(chain: &mut TaskChain, step_name: &str) {
    chain.steps.push(ChainStep {
        name: step_name.to_string(),
        completed: false,
    });
}

#[allow(dead_code)]
pub fn execute_chain(chain: &mut TaskChain) -> bool {
    if chain.current < chain.steps.len() {
        chain.steps[chain.current].completed = true;
        chain.current += 1;
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn chain_step_count(chain: &TaskChain) -> usize {
    chain.steps.len()
}

#[allow(dead_code)]
pub fn chain_is_complete(chain: &TaskChain) -> bool {
    !chain.steps.is_empty() && chain.current >= chain.steps.len()
}

#[allow(dead_code)]
pub fn chain_current_step(chain: &TaskChain) -> Option<&str> {
    if chain.current < chain.steps.len() {
        Some(&chain.steps[chain.current].name)
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn chain_reset(chain: &mut TaskChain) {
    chain.current = 0;
    for step in &mut chain.steps {
        step.completed = false;
    }
}

#[allow(dead_code)]
pub fn chain_to_json(chain: &TaskChain) -> String {
    let steps: Vec<String> = chain
        .steps
        .iter()
        .map(|s| format!("{{\"name\":\"{}\",\"completed\":{}}}", s.name, s.completed))
        .collect();
    format!(
        "{{\"name\":\"{}\",\"current\":{},\"steps\":[{}]}}",
        chain.name,
        chain.current,
        steps.join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_task_chain() {
        let c = new_task_chain("build");
        assert_eq!(chain_step_count(&c), 0);
    }

    #[test]
    fn test_add_step() {
        let mut c = new_task_chain("build");
        add_step(&mut c, "compile");
        assert_eq!(chain_step_count(&c), 1);
    }

    #[test]
    fn test_execute_chain() {
        let mut c = new_task_chain("build");
        add_step(&mut c, "compile");
        assert!(execute_chain(&mut c));
        assert!(!execute_chain(&mut c));
    }

    #[test]
    fn test_chain_is_complete() {
        let mut c = new_task_chain("build");
        add_step(&mut c, "compile");
        assert!(!chain_is_complete(&c));
        execute_chain(&mut c);
        assert!(chain_is_complete(&c));
    }

    #[test]
    fn test_chain_current_step() {
        let mut c = new_task_chain("build");
        add_step(&mut c, "compile");
        add_step(&mut c, "link");
        assert_eq!(chain_current_step(&c), Some("compile"));
        execute_chain(&mut c);
        assert_eq!(chain_current_step(&c), Some("link"));
    }

    #[test]
    fn test_chain_reset() {
        let mut c = new_task_chain("build");
        add_step(&mut c, "compile");
        execute_chain(&mut c);
        chain_reset(&mut c);
        assert!(!chain_is_complete(&c));
    }

    #[test]
    fn test_chain_to_json() {
        let mut c = new_task_chain("build");
        add_step(&mut c, "compile");
        let json = chain_to_json(&c);
        assert!(json.contains("\"name\":\"build\""));
    }

    #[test]
    fn test_empty_chain_not_complete() {
        let c = new_task_chain("empty");
        assert!(!chain_is_complete(&c));
    }

    #[test]
    fn test_chain_current_step_done() {
        let mut c = new_task_chain("build");
        add_step(&mut c, "compile");
        execute_chain(&mut c);
        assert!(chain_current_step(&c).is_none());
    }

    #[test]
    fn test_chain_multiple_steps() {
        let mut c = new_task_chain("pipeline");
        add_step(&mut c, "a");
        add_step(&mut c, "b");
        add_step(&mut c, "c");
        execute_chain(&mut c);
        execute_chain(&mut c);
        execute_chain(&mut c);
        assert!(chain_is_complete(&c));
    }
}
