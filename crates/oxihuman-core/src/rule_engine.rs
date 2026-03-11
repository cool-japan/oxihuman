// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Simple string-based rule engine: evaluate rules against a fact map.

use std::collections::HashMap;

/// Condition type for a rule.
#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    Equals(String, String),
    NotEquals(String, String),
    Contains(String, String),
    Absent(String),
    Present(String),
}

/// Action produced when a rule fires.
#[derive(Debug, Clone)]
pub struct RuleAction {
    pub name: String,
    pub payload: String,
}

/// A rule with conditions and actions.
#[derive(Debug, Clone)]
pub struct Rule {
    pub name: String,
    pub conditions: Vec<Condition>,
    pub actions: Vec<RuleAction>,
    pub priority: i32,
    pub enabled: bool,
}

/// Rule engine evaluating rules against a fact map.
pub struct RuleEngine {
    rules: Vec<Rule>,
    fire_count: u64,
}

fn eval_condition(cond: &Condition, facts: &HashMap<String, String>) -> bool {
    match cond {
        Condition::Equals(k, v) => facts.get(k).is_some_and(|fv| fv == v),
        Condition::NotEquals(k, v) => facts.get(k).is_some_and(|fv| fv != v),
        Condition::Contains(k, sub) => facts.get(k).is_some_and(|fv| fv.contains(sub.as_str())),
        Condition::Absent(k) => !facts.contains_key(k),
        Condition::Present(k) => facts.contains_key(k),
    }
}

#[allow(dead_code)]
impl RuleEngine {
    pub fn new() -> Self {
        RuleEngine {
            rules: Vec::new(),
            fire_count: 0,
        }
    }

    pub fn add_rule(&mut self, rule: Rule) {
        let pos = self.rules.partition_point(|r| r.priority > rule.priority);
        self.rules.insert(pos, rule);
    }

    pub fn evaluate(&mut self, facts: &HashMap<String, String>) -> Vec<RuleAction> {
        let mut actions = Vec::new();
        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }
            if rule.conditions.iter().all(|c| eval_condition(c, facts)) {
                self.fire_count += 1;
                actions.extend(rule.actions.clone());
            }
        }
        actions
    }

    pub fn set_enabled(&mut self, name: &str, enabled: bool) -> bool {
        if let Some(r) = self.rules.iter_mut().find(|r| r.name == name) {
            r.enabled = enabled;
            true
        } else {
            false
        }
    }

    pub fn remove_rule(&mut self, name: &str) -> bool {
        let before = self.rules.len();
        self.rules.retain(|r| r.name != name);
        self.rules.len() < before
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    pub fn enabled_count(&self) -> usize {
        self.rules.iter().filter(|r| r.enabled).count()
    }

    pub fn fire_count(&self) -> u64 {
        self.fire_count
    }

    pub fn clear(&mut self) {
        self.rules.clear();
    }

    pub fn rule_names(&self) -> Vec<&str> {
        self.rules.iter().map(|r| r.name.as_str()).collect()
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_rule_engine() -> RuleEngine {
    RuleEngine::new()
}

pub fn make_rule(
    name: &str,
    conds: Vec<Condition>,
    actions: Vec<RuleAction>,
    priority: i32,
) -> Rule {
    Rule {
        name: name.to_string(),
        conditions: conds,
        actions,
        priority,
        enabled: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn equals_condition_fires() {
        let mut engine = new_rule_engine();
        let rule = make_rule(
            "r1",
            vec![Condition::Equals(
                "status".to_string(),
                "active".to_string(),
            )],
            vec![RuleAction {
                name: "activate".to_string(),
                payload: String::new(),
            }],
            0,
        );
        engine.add_rule(rule);
        let f = facts(&[("status", "active")]);
        let actions = engine.evaluate(&f);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].name, "activate");
    }

    #[test]
    fn not_equals_fires() {
        let mut engine = new_rule_engine();
        let rule = make_rule(
            "r1",
            vec![Condition::NotEquals("x".to_string(), "bad".to_string())],
            vec![RuleAction {
                name: "ok".to_string(),
                payload: String::new(),
            }],
            0,
        );
        engine.add_rule(rule);
        let f = facts(&[("x", "good")]);
        assert_eq!(engine.evaluate(&f).len(), 1);
    }

    #[test]
    fn present_condition() {
        let mut engine = new_rule_engine();
        engine.add_rule(make_rule(
            "r1",
            vec![Condition::Present("key".to_string())],
            vec![RuleAction {
                name: "found".to_string(),
                payload: String::new(),
            }],
            0,
        ));
        assert_eq!(engine.evaluate(&facts(&[("key", "val")])).len(), 1);
        assert_eq!(engine.evaluate(&facts(&[])).len(), 0);
    }

    #[test]
    fn absent_condition() {
        let mut engine = new_rule_engine();
        engine.add_rule(make_rule(
            "r1",
            vec![Condition::Absent("ghost".to_string())],
            vec![RuleAction {
                name: "ok".to_string(),
                payload: String::new(),
            }],
            0,
        ));
        assert_eq!(engine.evaluate(&facts(&[])).len(), 1);
    }

    #[test]
    fn disabled_rule_skipped() {
        let mut engine = new_rule_engine();
        engine.add_rule(make_rule(
            "r1",
            vec![],
            vec![RuleAction {
                name: "fire".to_string(),
                payload: String::new(),
            }],
            0,
        ));
        engine.set_enabled("r1", false);
        assert_eq!(engine.evaluate(&facts(&[])).len(), 0);
    }

    #[test]
    fn fire_count_tracked() {
        let mut engine = new_rule_engine();
        engine.add_rule(make_rule(
            "r1",
            vec![],
            vec![RuleAction {
                name: "a".to_string(),
                payload: String::new(),
            }],
            0,
        ));
        engine.evaluate(&facts(&[]));
        engine.evaluate(&facts(&[]));
        assert_eq!(engine.fire_count(), 2);
    }

    #[test]
    fn remove_rule() {
        let mut engine = new_rule_engine();
        engine.add_rule(make_rule("r1", vec![], vec![], 0));
        assert!(engine.remove_rule("r1"));
        assert_eq!(engine.rule_count(), 0);
    }

    #[test]
    fn priority_ordering() {
        let mut engine = new_rule_engine();
        engine.add_rule(make_rule(
            "low",
            vec![],
            vec![RuleAction {
                name: "low".to_string(),
                payload: String::new(),
            }],
            1,
        ));
        engine.add_rule(make_rule(
            "high",
            vec![],
            vec![RuleAction {
                name: "high".to_string(),
                payload: String::new(),
            }],
            10,
        ));
        let names = engine.rule_names();
        assert_eq!(names[0], "high");
    }

    #[test]
    fn contains_condition() {
        let mut engine = new_rule_engine();
        engine.add_rule(make_rule(
            "r1",
            vec![Condition::Contains("msg".to_string(), "err".to_string())],
            vec![RuleAction {
                name: "alert".to_string(),
                payload: String::new(),
            }],
            0,
        ));
        assert_eq!(engine.evaluate(&facts(&[("msg", "some error")])).len(), 1);
        assert_eq!(engine.evaluate(&facts(&[("msg", "ok")])).len(), 0);
    }
}
