#![allow(dead_code)]
//! Rule-based constraints for body proportions.

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ProportionRule {
    pub name: String,
    pub param: String,
    pub min_val: f32,
    pub max_val: f32,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct RuleSet {
    rules: Vec<ProportionRule>,
}

#[allow(dead_code)]
pub fn new_rule_set() -> RuleSet {
    RuleSet { rules: Vec::new() }
}

#[allow(dead_code)]
pub fn add_rule(rs: &mut RuleSet, name: &str, param: &str, min_val: f32, max_val: f32) {
    rs.rules.push(ProportionRule {
        name: name.to_string(),
        param: param.to_string(),
        min_val,
        max_val,
    });
}

#[allow(dead_code)]
pub fn evaluate_rules(rs: &RuleSet, params: &[(String, f32)]) -> Vec<bool> {
    rs.rules
        .iter()
        .map(|r| {
            params
                .iter()
                .find(|(k, _)| k == &r.param)
                .map(|(_, v)| (r.min_val..=r.max_val).contains(v))
                .unwrap_or(false)
        })
        .collect()
}

#[allow(dead_code)]
pub fn rule_count(rs: &RuleSet) -> usize {
    rs.rules.len()
}

#[allow(dead_code)]
pub fn rule_is_satisfied(rule: &ProportionRule, value: f32) -> bool {
    (rule.min_val..=rule.max_val).contains(&value)
}

#[allow(dead_code)]
pub fn rule_error(rule: &ProportionRule, value: f32) -> f32 {
    if value < rule.min_val {
        rule.min_val - value
    } else if value > rule.max_val {
        value - rule.max_val
    } else {
        0.0
    }
}

#[allow(dead_code)]
pub fn rules_to_json(rs: &RuleSet) -> String {
    let entries: Vec<String> = rs
        .rules
        .iter()
        .map(|r| {
            format!(
                "{{\"name\":\"{}\",\"param\":\"{}\",\"min\":{},\"max\":{}}}",
                r.name, r.param, r.min_val, r.max_val
            )
        })
        .collect();
    format!("[{}]", entries.join(","))
}

#[allow(dead_code)]
pub fn clear_rules(rs: &mut RuleSet) {
    rs.rules.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_rule_set() {
        let rs = new_rule_set();
        assert_eq!(rule_count(&rs), 0);
    }

    #[test]
    fn test_add_rule() {
        let mut rs = new_rule_set();
        add_rule(&mut rs, "height", "h", 0.0, 1.0);
        assert_eq!(rule_count(&rs), 1);
    }

    #[test]
    fn test_evaluate_rules_satisfied() {
        let mut rs = new_rule_set();
        add_rule(&mut rs, "h", "height", 0.0, 1.0);
        let params = vec![("height".to_string(), 0.5)];
        let results = evaluate_rules(&rs, &params);
        assert!(results[0]);
    }

    #[test]
    fn test_evaluate_rules_violated() {
        let mut rs = new_rule_set();
        add_rule(&mut rs, "h", "height", 0.0, 1.0);
        let params = vec![("height".to_string(), 2.0)];
        let results = evaluate_rules(&rs, &params);
        assert!(!results[0]);
    }

    #[test]
    fn test_rule_is_satisfied() {
        let rule = ProportionRule {
            name: "test".into(),
            param: "x".into(),
            min_val: 0.0,
            max_val: 1.0,
        };
        assert!(rule_is_satisfied(&rule, 0.5));
        assert!(!rule_is_satisfied(&rule, 1.5));
    }

    #[test]
    fn test_rule_error() {
        let rule = ProportionRule {
            name: "t".into(),
            param: "x".into(),
            min_val: 0.2,
            max_val: 0.8,
        };
        assert!((rule_error(&rule, 0.5)).abs() < 1e-6);
        assert!((rule_error(&rule, 0.0) - 0.2).abs() < 1e-6);
        assert!((rule_error(&rule, 1.0) - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_rules_to_json() {
        let mut rs = new_rule_set();
        add_rule(&mut rs, "r", "p", 0.0, 1.0);
        let json = rules_to_json(&rs);
        assert!(json.contains("\"name\":\"r\""));
    }

    #[test]
    fn test_clear_rules() {
        let mut rs = new_rule_set();
        add_rule(&mut rs, "r", "p", 0.0, 1.0);
        clear_rules(&mut rs);
        assert_eq!(rule_count(&rs), 0);
    }

    #[test]
    fn test_missing_param() {
        let mut rs = new_rule_set();
        add_rule(&mut rs, "h", "height", 0.0, 1.0);
        let params: Vec<(String, f32)> = vec![];
        let results = evaluate_rules(&rs, &params);
        assert!(!results[0]);
    }

    #[test]
    fn test_boundary_values() {
        let rule = ProportionRule {
            name: "t".into(),
            param: "x".into(),
            min_val: 0.0,
            max_val: 1.0,
        };
        assert!(rule_is_satisfied(&rule, 0.0));
        assert!(rule_is_satisfied(&rule, 1.0));
    }
}
