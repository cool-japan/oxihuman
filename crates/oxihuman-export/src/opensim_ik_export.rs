// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! OpenSim inverse kinematics task export.

#[derive(Debug, Clone)]
pub struct IkMarker {
    pub name: String,
    pub weight: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone)]
pub struct IkCoordinateTask {
    pub name: String,
    pub weight: f64,
    pub value: f64,
}

#[derive(Debug, Clone)]
pub struct OpenSimIkSetup {
    pub model_file: String,
    pub marker_file: String,
    pub output_motion_file: String,
    pub start_time: f64,
    pub end_time: f64,
    pub report_errors: bool,
    pub marker_tasks: Vec<IkMarker>,
    pub coordinate_tasks: Vec<IkCoordinateTask>,
}

pub fn new_opensim_ik_setup(model_file: &str, marker_file: &str) -> OpenSimIkSetup {
    OpenSimIkSetup {
        model_file: model_file.to_string(),
        marker_file: marker_file.to_string(),
        output_motion_file: "ik_output.mot".to_string(),
        start_time: 0.0,
        end_time: 1.0,
        report_errors: true,
        marker_tasks: Vec::new(),
        coordinate_tasks: Vec::new(),
    }
}

pub fn add_ik_marker_task(setup: &mut OpenSimIkSetup, name: &str, weight: f64) {
    setup.marker_tasks.push(IkMarker {
        name: name.to_string(),
        weight,
        x: 0.0,
        y: 0.0,
        z: 0.0,
    });
}

pub fn add_ik_coordinate_task(setup: &mut OpenSimIkSetup, name: &str, weight: f64, value: f64) {
    setup.coordinate_tasks.push(IkCoordinateTask {
        name: name.to_string(),
        weight,
        value,
    });
}

pub fn render_opensim_ik_xml(setup: &OpenSimIkSetup) -> String {
    let mut s = "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n<OpenSimDocument Version=\"40000\">\n<InverseKinematicsTool>\n".to_string();
    s.push_str(&format!(
        "  <model_file>{}</model_file>\n",
        setup.model_file
    ));
    s.push_str(&format!(
        "  <marker_file>{}</marker_file>\n",
        setup.marker_file
    ));
    s.push_str(&format!(
        "  <output_motion_file>{}</output_motion_file>\n",
        setup.output_motion_file
    ));
    s.push_str(&format!(
        "  <time_range>{} {}</time_range>\n",
        setup.start_time, setup.end_time
    ));
    s.push_str("  <IKTaskSet>\n  <objects>\n");
    for mk in &setup.marker_tasks {
        s.push_str(&format!(
            "    <IKMarkerTask name=\"{}\">\n      <weight>{}</weight>\n    </IKMarkerTask>\n",
            mk.name, mk.weight
        ));
    }
    for ct in &setup.coordinate_tasks {
        s.push_str(&format!("    <IKCoordinateTask name=\"{}\">\n      <weight>{}</weight>\n      <value>{}</value>\n    </IKCoordinateTask>\n", ct.name, ct.weight, ct.value));
    }
    s.push_str("  </objects>\n  </IKTaskSet>\n</InverseKinematicsTool>\n</OpenSimDocument>\n");
    s
}

pub fn export_opensim_ik(setup: &OpenSimIkSetup) -> Vec<u8> {
    render_opensim_ik_xml(setup).into_bytes()
}
pub fn ik_marker_task_count(setup: &OpenSimIkSetup) -> usize {
    setup.marker_tasks.len()
}
pub fn ik_coordinate_task_count(setup: &OpenSimIkSetup) -> usize {
    setup.coordinate_tasks.len()
}
pub fn validate_opensim_ik(setup: &OpenSimIkSetup) -> bool {
    !setup.model_file.is_empty() && setup.end_time > setup.start_time
}
pub fn opensim_ik_size_bytes(setup: &OpenSimIkSetup) -> usize {
    render_opensim_ik_xml(setup).len()
}

pub fn default_ik_setup() -> OpenSimIkSetup {
    let mut s = new_opensim_ik_setup("model.osim", "markers.trc");
    add_ik_marker_task(&mut s, "RASIS", 1.0);
    add_ik_marker_task(&mut s, "LASIS", 1.0);
    add_ik_coordinate_task(&mut s, "hip_flexion_r", 1.0, 0.0);
    s
}

pub fn ik_set_time_range(setup: &mut OpenSimIkSetup, start: f64, end: f64) {
    setup.start_time = start;
    setup.end_time = end;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_opensim_ik_setup() {
        let s = new_opensim_ik_setup("m.osim", "mk.trc");
        assert_eq!(s.model_file, "m.osim");
    }

    #[test]
    fn test_add_ik_marker_task() {
        let mut s = new_opensim_ik_setup("m", "mk");
        add_ik_marker_task(&mut s, "M1", 1.0);
        assert_eq!(ik_marker_task_count(&s), 1);
    }

    #[test]
    fn test_render_contains_ik_tool() {
        let s = new_opensim_ik_setup("m", "mk");
        let xml = render_opensim_ik_xml(&s);
        assert!(xml.contains("InverseKinematicsTool"));
    }

    #[test]
    fn test_export_ik_nonempty() {
        let s = new_opensim_ik_setup("m", "mk");
        assert!(!export_opensim_ik(&s).is_empty());
    }

    #[test]
    fn test_validate_opensim_ik() {
        let s = new_opensim_ik_setup("m", "mk");
        assert!(validate_opensim_ik(&s));
    }

    #[test]
    fn test_default_ik_setup() {
        let s = default_ik_setup();
        assert!(ik_marker_task_count(&s) >= 2);
        assert!(ik_coordinate_task_count(&s) >= 1);
    }

    #[test]
    fn test_ik_set_time_range() {
        let mut s = new_opensim_ik_setup("m", "mk");
        ik_set_time_range(&mut s, 0.5, 2.5);
        assert!((s.end_time - 2.5).abs() < 1e-9);
    }

    #[test]
    fn test_opensim_ik_size_bytes() {
        let s = new_opensim_ik_setup("m", "mk");
        assert!(opensim_ik_size_bytes(&s) > 0);
    }
}
