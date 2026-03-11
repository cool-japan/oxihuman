//! Hierarchical state machine for animation and behavior control.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SmState {
    pub id: u32,
    pub name: String,
    pub parent: Option<u32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SmTransition {
    pub from: u32,
    pub to: u32,
    pub condition: String,
    pub priority: i32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StateMachine {
    pub states: Vec<SmState>,
    pub transitions: Vec<SmTransition>,
    pub current: u32,
    pub history: Vec<u32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SmConfig {
    pub max_history: usize,
    pub allow_self_transitions: bool,
}

#[allow(dead_code)]
pub fn default_sm_config() -> SmConfig {
    SmConfig {
        max_history: 64,
        allow_self_transitions: false,
    }
}

#[allow(dead_code)]
pub fn new_state_machine(initial: u32, cfg: SmConfig) -> StateMachine {
    let _ = cfg;
    StateMachine {
        states: Vec::new(),
        transitions: Vec::new(),
        current: initial,
        history: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn sm_add_state(sm: &mut StateMachine, state: SmState) {
    sm.states.push(state);
}

#[allow(dead_code)]
pub fn sm_add_transition(sm: &mut StateMachine, t: SmTransition) {
    sm.transitions.push(t);
}

#[allow(dead_code)]
pub fn sm_transition_to(sm: &mut StateMachine, target: u32) -> bool {
    if sm.current == target {
        return false;
    }
    let valid = sm
        .transitions
        .iter()
        .any(|t| t.from == sm.current && t.to == target);
    if valid {
        sm.history.push(sm.current);
        sm.current = target;
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn sm_current_state(sm: &StateMachine) -> u32 {
    sm.current
}

#[allow(dead_code)]
pub fn sm_state_name(sm: &StateMachine, id: u32) -> &str {
    for s in &sm.states {
        if s.id == id {
            return &s.name;
        }
    }
    ""
}

#[allow(dead_code)]
pub fn sm_can_transition(sm: &StateMachine, target: u32) -> bool {
    sm.transitions
        .iter()
        .any(|t| t.from == sm.current && t.to == target)
}

#[allow(dead_code)]
pub fn sm_state_count(sm: &StateMachine) -> usize {
    sm.states.len()
}

#[allow(dead_code)]
pub fn sm_history_len(sm: &StateMachine) -> usize {
    sm.history.len()
}

#[allow(dead_code)]
pub fn state_machine_to_json(sm: &StateMachine) -> String {
    let states_json: Vec<String> = sm
        .states
        .iter()
        .map(|s| {
            let parent_str = match s.parent {
                Some(p) => format!("{}", p),
                None => "null".to_string(),
            };
            format!(r#"{{"id":{},"name":"{}","parent":{}}}"#, s.id, s.name, parent_str)
        })
        .collect();
    format!(
        r#"{{"current":{},"state_count":{},"history_len":{},"states":[{}]}}"#,
        sm.current,
        sm.states.len(),
        sm.history.len(),
        states_json.join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sm() -> StateMachine {
        let cfg = default_sm_config();
        let mut sm = new_state_machine(0, cfg);
        sm_add_state(&mut sm, SmState { id: 0, name: "idle".to_string(), parent: None });
        sm_add_state(&mut sm, SmState { id: 1, name: "walk".to_string(), parent: None });
        sm_add_state(&mut sm, SmState { id: 2, name: "run".to_string(), parent: None });
        sm_add_transition(&mut sm, SmTransition { from: 0, to: 1, condition: "start".to_string(), priority: 0 });
        sm_add_transition(&mut sm, SmTransition { from: 1, to: 2, condition: "speed_up".to_string(), priority: 0 });
        sm_add_transition(&mut sm, SmTransition { from: 2, to: 0, condition: "stop".to_string(), priority: 0 });
        sm
    }

    #[test]
    fn test_initial_state() {
        let sm = make_sm();
        assert_eq!(sm_current_state(&sm), 0);
    }

    #[test]
    fn test_valid_transition() {
        let mut sm = make_sm();
        let ok = sm_transition_to(&mut sm, 1);
        assert!(ok);
        assert_eq!(sm_current_state(&sm), 1);
        assert_eq!(sm_history_len(&sm), 1);
    }

    #[test]
    fn test_invalid_transition() {
        let mut sm = make_sm();
        let ok = sm_transition_to(&mut sm, 2);
        assert!(!ok);
        assert_eq!(sm_current_state(&sm), 0);
    }

    #[test]
    fn test_can_transition() {
        let sm = make_sm();
        assert!(sm_can_transition(&sm, 1));
        assert!(!sm_can_transition(&sm, 2));
    }

    #[test]
    fn test_state_name() {
        let sm = make_sm();
        assert_eq!(sm_state_name(&sm, 0), "idle");
        assert_eq!(sm_state_name(&sm, 1), "walk");
        assert_eq!(sm_state_name(&sm, 99), "");
    }

    #[test]
    fn test_state_count() {
        let sm = make_sm();
        assert_eq!(sm_state_count(&sm), 3);
    }

    #[test]
    fn test_to_json() {
        let sm = make_sm();
        let j = state_machine_to_json(&sm);
        assert!(j.contains("\"current\":0"));
        assert!(j.contains("\"state_count\":3"));
    }
}
