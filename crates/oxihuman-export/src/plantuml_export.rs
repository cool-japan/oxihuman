// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export state machine as a PlantUML diagram.

#![allow(dead_code)]

/// A PlantUML state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PlantState {
    pub name: String,
    pub label: String,
}

/// A PlantUML transition.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PlantTransition {
    pub from: String,
    pub to: String,
    pub guard: String,
}

/// A PlantUML state machine export.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct PlantUmlExport {
    pub title: String,
    pub states: Vec<PlantState>,
    pub transitions: Vec<PlantTransition>,
    pub initial_state: String,
    pub final_states: Vec<String>,
}

/// Create a new PlantUML export.
#[allow(dead_code)]
pub fn new_plantuml_export(title: &str) -> PlantUmlExport {
    PlantUmlExport {
        title: title.to_string(),
        states: Vec::new(),
        transitions: Vec::new(),
        initial_state: String::new(),
        final_states: Vec::new(),
    }
}

/// Add a state.
#[allow(dead_code)]
pub fn add_plant_state(doc: &mut PlantUmlExport, name: &str, label: &str) {
    doc.states.push(PlantState {
        name: name.to_string(),
        label: label.to_string(),
    });
}

/// Add a transition.
#[allow(dead_code)]
pub fn add_plant_transition(doc: &mut PlantUmlExport, from: &str, to: &str, guard: &str) {
    doc.transitions.push(PlantTransition {
        from: from.to_string(),
        to: to.to_string(),
        guard: guard.to_string(),
    });
}

/// Set the initial state name.
#[allow(dead_code)]
pub fn set_initial_state(doc: &mut PlantUmlExport, name: &str) {
    doc.initial_state = name.to_string();
}

/// Add a final state.
#[allow(dead_code)]
pub fn add_final_state(doc: &mut PlantUmlExport, name: &str) {
    doc.final_states.push(name.to_string());
}

/// Return state count.
#[allow(dead_code)]
pub fn plant_state_count(doc: &PlantUmlExport) -> usize {
    doc.states.len()
}

/// Return transition count.
#[allow(dead_code)]
pub fn plant_transition_count(doc: &PlantUmlExport) -> usize {
    doc.transitions.len()
}

/// Serialise as PlantUML text.
#[allow(dead_code)]
pub fn to_plantuml_string(doc: &PlantUmlExport) -> String {
    let mut out = String::from("@startuml\n");
    if !doc.title.is_empty() {
        out.push_str(&format!("title {}\n", doc.title));
    }
    if !doc.initial_state.is_empty() {
        out.push_str(&format!("[*] --> {}\n", doc.initial_state));
    }
    for state in &doc.states {
        if state.label.is_empty() || state.label == state.name {
            out.push_str(&format!("state {}\n", state.name));
        } else {
            out.push_str(&format!("state \"{}\" as {}\n", state.label, state.name));
        }
    }
    for tr in &doc.transitions {
        if tr.guard.is_empty() {
            out.push_str(&format!("{} --> {}\n", tr.from, tr.to));
        } else {
            out.push_str(&format!("{} --> {} : {}\n", tr.from, tr.to, tr.guard));
        }
    }
    for fs in &doc.final_states {
        out.push_str(&format!("{} --> [*]\n", fs));
    }
    out.push_str("@enduml");
    out
}

/// Find a state by name.
#[allow(dead_code)]
pub fn find_plant_state<'a>(doc: &'a PlantUmlExport, name: &str) -> Option<&'a PlantState> {
    doc.states.iter().find(|s| s.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_plantuml_empty() {
        let doc = new_plantuml_export("FSM");
        assert_eq!(plant_state_count(&doc), 0);
        assert_eq!(plant_transition_count(&doc), 0);
    }

    #[test]
    fn test_add_state() {
        let mut doc = new_plantuml_export("T");
        add_plant_state(&mut doc, "idle", "Idle");
        assert_eq!(plant_state_count(&doc), 1);
    }

    #[test]
    fn test_add_transition() {
        let mut doc = new_plantuml_export("T");
        add_plant_transition(&mut doc, "idle", "walk", "move");
        assert_eq!(plant_transition_count(&doc), 1);
    }

    #[test]
    fn test_to_plantuml_contains_startuml() {
        let doc = new_plantuml_export("T");
        let s = to_plantuml_string(&doc);
        assert!(s.contains("@startuml"));
    }

    #[test]
    fn test_to_plantuml_contains_title() {
        let doc = new_plantuml_export("MyFSM");
        let s = to_plantuml_string(&doc);
        assert!(s.contains("MyFSM"));
    }

    #[test]
    fn test_to_plantuml_contains_state() {
        let mut doc = new_plantuml_export("T");
        add_plant_state(&mut doc, "idle", "");
        let s = to_plantuml_string(&doc);
        assert!(s.contains("idle"));
    }

    #[test]
    fn test_to_plantuml_contains_transition() {
        let mut doc = new_plantuml_export("T");
        add_plant_transition(&mut doc, "a", "b", "go");
        let s = to_plantuml_string(&doc);
        assert!(s.contains("-->"));
    }

    #[test]
    fn test_initial_state() {
        let mut doc = new_plantuml_export("T");
        set_initial_state(&mut doc, "idle");
        let s = to_plantuml_string(&doc);
        assert!(s.contains("[*] --> idle"));
    }

    #[test]
    fn test_final_state() {
        let mut doc = new_plantuml_export("T");
        add_final_state(&mut doc, "done");
        let s = to_plantuml_string(&doc);
        assert!(s.contains("done --> [*]"));
    }

    #[test]
    fn test_find_plant_state() {
        let mut doc = new_plantuml_export("T");
        add_plant_state(&mut doc, "walk", "Walking");
        assert!(find_plant_state(&doc, "walk").is_some());
        assert!(find_plant_state(&doc, "ghost").is_none());
    }
}
