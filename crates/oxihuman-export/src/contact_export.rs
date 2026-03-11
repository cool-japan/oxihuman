// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export contact point data (physics simulation results).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ContactPoint {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub depth: f32,
    pub body_a: u32,
    pub body_b: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ContactExportData {
    pub frame: u32,
    pub contacts: Vec<ContactPoint>,
}

#[allow(dead_code)]
pub fn new_contact_export(frame: u32) -> ContactExportData {
    ContactExportData { frame, contacts: Vec::new() }
}

#[allow(dead_code)]
pub fn ce_add(ce: &mut ContactExportData, pos: [f32; 3], normal: [f32; 3], depth: f32, a: u32, b: u32) {
    ce.contacts.push(ContactPoint { position: pos, normal, depth, body_a: a, body_b: b });
}

#[allow(dead_code)]
pub fn ce_count(ce: &ContactExportData) -> usize { ce.contacts.len() }

#[allow(dead_code)]
pub fn ce_max_depth(ce: &ContactExportData) -> f32 {
    ce.contacts.iter().map(|c| c.depth).fold(0.0f32, f32::max)
}

#[allow(dead_code)]
pub fn ce_avg_depth(ce: &ContactExportData) -> f32 {
    if ce.contacts.is_empty() { return 0.0; }
    ce.contacts.iter().map(|c| c.depth).sum::<f32>() / ce.contacts.len() as f32
}

#[allow(dead_code)]
pub fn ce_bodies_involved(ce: &ContactExportData) -> Vec<u32> {
    let mut bodies: Vec<u32> = ce.contacts.iter().flat_map(|c| [c.body_a, c.body_b]).collect();
    bodies.sort_unstable();
    bodies.dedup();
    bodies
}

#[allow(dead_code)]
pub fn ce_validate(ce: &ContactExportData) -> bool {
    ce.contacts.iter().all(|c| c.depth >= 0.0)
}

#[allow(dead_code)]
pub fn ce_to_json(ce: &ContactExportData) -> String {
    format!("{{\"frame\":{},\"contacts\":{},\"max_depth\":{:.4}}}", ce.frame, ce.contacts.len(), ce_max_depth(ce))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> ContactExportData {
        let mut c = new_contact_export(10);
        ce_add(&mut c, [1.0,0.0,0.0], [0.0,1.0,0.0], 0.05, 0, 1);
        ce_add(&mut c, [2.0,0.0,0.0], [0.0,1.0,0.0], 0.1, 0, 2);
        c
    }

    #[test] fn test_new() { let c = new_contact_export(0); assert_eq!(ce_count(&c), 0); }
    #[test] fn test_add() { assert_eq!(ce_count(&sample()), 2); }
    #[test] fn test_max_depth() { assert!((ce_max_depth(&sample()) - 0.1).abs() < 1e-5); }
    #[test] fn test_avg_depth() { assert!((ce_avg_depth(&sample()) - 0.075).abs() < 1e-5); }
    #[test] fn test_bodies() { let b = ce_bodies_involved(&sample()); assert_eq!(b.len(), 3); }
    #[test] fn test_validate() { assert!(ce_validate(&sample())); }
    #[test] fn test_to_json() { assert!(ce_to_json(&sample()).contains("frame")); }
    #[test] fn test_frame() { assert_eq!(sample().frame, 10); }
    #[test] fn test_empty_avg() { let c = new_contact_export(0); assert!((ce_avg_depth(&c)).abs() < 1e-6); }
    #[test] fn test_position() { let s = sample(); assert!((s.contacts[0].position[0] - 1.0).abs() < 1e-6); }
}
