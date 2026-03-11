#![allow(dead_code)]
//! Expression triggers that fire based on parameter conditions.

/// Condition under which a trigger fires.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TriggerCondition {
    /// Parameter name to watch.
    pub param: String,
    /// Threshold value.
    pub threshold: f32,
    /// If true, fires when param >= threshold; otherwise when param < threshold.
    pub above: bool,
}

/// An expression trigger that activates morphs based on conditions.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionTrigger {
    /// Trigger name.
    pub name: String,
    /// Conditions to evaluate.
    pub conditions: Vec<TriggerCondition>,
    /// Whether the trigger has been fired.
    pub fired: bool,
    /// Whether the trigger is enabled.
    pub active: bool,
}

/// Create a new [`ExpressionTrigger`].
#[allow(dead_code)]
pub fn new_expression_trigger(name: &str) -> ExpressionTrigger {
    ExpressionTrigger {
        name: name.to_string(),
        conditions: Vec::new(),
        fired: false,
        active: true,
    }
}

/// Add a trigger condition.
#[allow(dead_code)]
pub fn add_trigger(trigger: &mut ExpressionTrigger, param: &str, threshold: f32, above: bool) {
    trigger.conditions.push(TriggerCondition {
        param: param.to_string(),
        threshold,
        above,
    });
}

/// Check all conditions against a set of named parameter values.
/// Returns true if all conditions are satisfied.
#[allow(dead_code)]
pub fn check_triggers(trigger: &ExpressionTrigger, params: &[(&str, f32)]) -> bool {
    if !trigger.active {
        return false;
    }
    trigger.conditions.iter().all(|c| {
        let value = params
            .iter()
            .find(|(name, _)| *name == c.param)
            .map(|(_, v)| *v)
            .unwrap_or(0.0);
        if c.above {
            value >= c.threshold
        } else {
            value < c.threshold
        }
    })
}

/// Return the number of conditions.
#[allow(dead_code)]
pub fn trigger_count(trigger: &ExpressionTrigger) -> usize {
    trigger.conditions.len()
}

/// Mark the trigger as fired.
#[allow(dead_code)]
pub fn trigger_fire(trigger: &mut ExpressionTrigger) {
    trigger.fired = true;
}

/// Check if the trigger is currently active.
#[allow(dead_code)]
pub fn trigger_is_active(trigger: &ExpressionTrigger) -> bool {
    trigger.active
}

/// Reset the trigger (unfired, re-enable).
#[allow(dead_code)]
pub fn trigger_reset(trigger: &mut ExpressionTrigger) {
    trigger.fired = false;
    trigger.active = true;
}

/// Serialize triggers to a JSON-like string.
#[allow(dead_code)]
pub fn triggers_to_json(trigger: &ExpressionTrigger) -> String {
    let mut s = format!(
        "{{\"name\":\"{}\",\"fired\":{},\"active\":{},\"conditions\":[",
        trigger.name, trigger.fired, trigger.active
    );
    for (i, c) in trigger.conditions.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"param\":\"{}\",\"threshold\":{:.4},\"above\":{}}}",
            c.param, c.threshold, c.above
        ));
    }
    s.push_str("]}");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_expression_trigger() {
        let t = new_expression_trigger("blink");
        assert_eq!(t.name, "blink");
        assert!(t.active);
        assert!(!t.fired);
    }

    #[test]
    fn test_add_trigger() {
        let mut t = new_expression_trigger("blink");
        add_trigger(&mut t, "eye_close", 0.8, true);
        assert_eq!(trigger_count(&t), 1);
    }

    #[test]
    fn test_check_triggers_satisfied() {
        let mut t = new_expression_trigger("blink");
        add_trigger(&mut t, "eye_close", 0.5, true);
        let params = [("eye_close", 0.8_f32)];
        assert!(check_triggers(&t, &params));
    }

    #[test]
    fn test_check_triggers_not_satisfied() {
        let mut t = new_expression_trigger("blink");
        add_trigger(&mut t, "eye_close", 0.5, true);
        let params = [("eye_close", 0.2_f32)];
        assert!(!check_triggers(&t, &params));
    }

    #[test]
    fn test_check_triggers_below() {
        let mut t = new_expression_trigger("relax");
        add_trigger(&mut t, "tension", 0.3, false);
        let params = [("tension", 0.1_f32)];
        assert!(check_triggers(&t, &params));
    }

    #[test]
    fn test_trigger_fire_and_reset() {
        let mut t = new_expression_trigger("blink");
        trigger_fire(&mut t);
        assert!(t.fired);
        trigger_reset(&mut t);
        assert!(!t.fired);
        assert!(t.active);
    }

    #[test]
    fn test_trigger_is_active() {
        let mut t = new_expression_trigger("blink");
        assert!(trigger_is_active(&t));
        t.active = false;
        assert!(!trigger_is_active(&t));
    }

    #[test]
    fn test_check_triggers_inactive() {
        let mut t = new_expression_trigger("blink");
        t.active = false;
        add_trigger(&mut t, "eye_close", 0.0, true);
        let params = [("eye_close", 1.0_f32)];
        assert!(!check_triggers(&t, &params));
    }

    #[test]
    fn test_triggers_to_json() {
        let mut t = new_expression_trigger("blink");
        add_trigger(&mut t, "eye_close", 0.5, true);
        let json = triggers_to_json(&t);
        assert!(json.contains("blink"));
        assert!(json.contains("eye_close"));
    }

    #[test]
    fn test_empty_conditions_always_true() {
        let t = new_expression_trigger("empty");
        let params: [(&str, f32); 0] = [];
        assert!(check_triggers(&t, &params));
    }
}
