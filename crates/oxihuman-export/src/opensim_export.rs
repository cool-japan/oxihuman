// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! OpenSim musculoskeletal model export.

#[derive(Debug, Clone)]
pub struct OpenSimBody {
    pub name: String,
    pub mass: f64,
    pub inertia: [f64; 6],
    pub com: [f64; 3],
}

#[derive(Debug, Clone)]
pub struct OpenSimJoint {
    pub name: String,
    pub parent_body: String,
    pub child_body: String,
    pub location_in_parent: [f64; 3],
    pub location_in_child: [f64; 3],
}

#[derive(Debug, Clone)]
pub struct OpenSimMuscle {
    pub name: String,
    pub origin_body: String,
    pub insertion_body: String,
    pub optimal_fiber_length: f64,
    pub tendon_slack_length: f64,
    pub max_isometric_force: f64,
}

#[derive(Debug, Clone)]
pub struct OpenSimModel {
    pub name: String,
    pub bodies: Vec<OpenSimBody>,
    pub joints: Vec<OpenSimJoint>,
    pub muscles: Vec<OpenSimMuscle>,
}

pub fn new_opensim_model(name: &str) -> OpenSimModel {
    OpenSimModel {
        name: name.to_string(),
        bodies: Vec::new(),
        joints: Vec::new(),
        muscles: Vec::new(),
    }
}

pub fn add_opensim_body(model: &mut OpenSimModel, name: &str, mass: f64) {
    model.bodies.push(OpenSimBody {
        name: name.to_string(),
        mass,
        inertia: [0.0; 6],
        com: [0.0; 3],
    });
}

pub fn add_opensim_joint(model: &mut OpenSimModel, name: &str, parent: &str, child: &str) {
    model.joints.push(OpenSimJoint {
        name: name.to_string(),
        parent_body: parent.to_string(),
        child_body: child.to_string(),
        location_in_parent: [0.0; 3],
        location_in_child: [0.0; 3],
    });
}

pub fn add_opensim_muscle(
    model: &mut OpenSimModel,
    name: &str,
    origin: &str,
    insertion: &str,
    opt_fiber_len: f64,
    tendon_slack: f64,
    max_force: f64,
) {
    model.muscles.push(OpenSimMuscle {
        name: name.to_string(),
        origin_body: origin.to_string(),
        insertion_body: insertion.to_string(),
        optimal_fiber_length: opt_fiber_len,
        tendon_slack_length: tendon_slack,
        max_isometric_force: max_force,
    });
}

pub fn render_opensim_xml(model: &OpenSimModel) -> String {
    let mut s = format!("<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n<OpenSimDocument Version=\"40000\">\n<Model name=\"{}\">\n", model.name);
    s.push_str("<BodySet>\n<objects>\n");
    for b in &model.bodies {
        s.push_str(&format!(
            "  <Body name=\"{}\">\n    <mass>{}</mass>\n  </Body>\n",
            b.name, b.mass
        ));
    }
    s.push_str("</objects></BodySet>\n");
    s.push_str("<JointSet>\n<objects>\n");
    for j in &model.joints {
        s.push_str(&format!("  <PinJoint name=\"{}\">\n    <parent_frame>{}</parent_frame>\n    <child_frame>{}</child_frame>\n  </PinJoint>\n",
            j.name, j.parent_body, j.child_body));
    }
    s.push_str("</objects></JointSet>\n");
    s.push_str("<ForceSet>\n<objects>\n");
    for m in &model.muscles {
        s.push_str(&format!("  <Millard2012EquilibriumMuscle name=\"{}\">\n    <optimal_fiber_length>{}</optimal_fiber_length>\n    <max_isometric_force>{}</max_isometric_force>\n  </Millard2012EquilibriumMuscle>\n",
            m.name, m.optimal_fiber_length, m.max_isometric_force));
    }
    s.push_str("</objects></ForceSet>\n</Model>\n</OpenSimDocument>\n");
    s
}

pub fn export_opensim(model: &OpenSimModel) -> Vec<u8> {
    render_opensim_xml(model).into_bytes()
}
pub fn opensim_body_count(model: &OpenSimModel) -> usize {
    model.bodies.len()
}
pub fn opensim_muscle_count(model: &OpenSimModel) -> usize {
    model.muscles.len()
}
pub fn validate_opensim_model(model: &OpenSimModel) -> bool {
    !model.name.is_empty() && !model.bodies.is_empty()
}
pub fn opensim_size_bytes(model: &OpenSimModel) -> usize {
    render_opensim_xml(model).len()
}

pub fn default_biped_opensim_model() -> OpenSimModel {
    let mut m = new_opensim_model("BipedModel");
    add_opensim_body(&mut m, "pelvis", 10.0);
    add_opensim_body(&mut m, "femur_r", 8.0);
    add_opensim_joint(&mut m, "hip_r", "pelvis", "femur_r");
    add_opensim_muscle(&mut m, "rect_fem_r", "pelvis", "femur_r", 0.08, 0.35, 800.0);
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_opensim_model() {
        let m = new_opensim_model("Test");
        assert_eq!(m.name, "Test");
    }

    #[test]
    fn test_add_body() {
        let mut m = new_opensim_model("T");
        add_opensim_body(&mut m, "pelvis", 10.0);
        assert_eq!(opensim_body_count(&m), 1);
    }

    #[test]
    fn test_render_contains_opensim_tag() {
        let m = new_opensim_model("T");
        let s = render_opensim_xml(&m);
        assert!(s.contains("OpenSimDocument"));
    }

    #[test]
    fn test_export_opensim_nonempty() {
        let m = new_opensim_model("T");
        assert!(!export_opensim(&m).is_empty());
    }

    #[test]
    fn test_validate_opensim_model() {
        let mut m = new_opensim_model("T");
        add_opensim_body(&mut m, "b", 1.0);
        assert!(validate_opensim_model(&m));
    }

    #[test]
    fn test_default_biped() {
        let m = default_biped_opensim_model();
        assert!(opensim_body_count(&m) >= 2);
        assert!(opensim_muscle_count(&m) >= 1);
    }

    #[test]
    fn test_opensim_size_bytes() {
        let m = new_opensim_model("T");
        assert!(opensim_size_bytes(&m) > 0);
    }

    #[test]
    fn test_muscle_in_render() {
        let mut m = new_opensim_model("T");
        add_opensim_body(&mut m, "b1", 1.0);
        add_opensim_body(&mut m, "b2", 1.0);
        add_opensim_muscle(&mut m, "muscle1", "b1", "b2", 0.1, 0.2, 500.0);
        let s = render_opensim_xml(&m);
        assert!(s.contains("muscle1"));
    }
}
