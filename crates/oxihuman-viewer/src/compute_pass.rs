// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Compute pass descriptor for GPU compute shaders.

/// Workgroup size for a compute dispatch.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorkgroupSize {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

/// Descriptor for a compute pass.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ComputePassDesc {
    pub label: String,
    pub shader_entry: String,
    pub workgroup_size: WorkgroupSize,
    pub dispatch_count: [u32; 3],
}

#[allow(dead_code)]
pub fn default_workgroup_size() -> WorkgroupSize {
    WorkgroupSize { x: 64, y: 1, z: 1 }
}

#[allow(dead_code)]
pub fn new_compute_pass(label: &str, entry: &str) -> ComputePassDesc {
    ComputePassDesc {
        label: label.to_string(),
        shader_entry: entry.to_string(),
        workgroup_size: default_workgroup_size(),
        dispatch_count: [1, 1, 1],
    }
}

#[allow(dead_code)]
pub fn set_dispatch(pass: &mut ComputePassDesc, x: u32, y: u32, z: u32) {
    pass.dispatch_count = [x, y, z];
}

#[allow(dead_code)]
pub fn set_workgroup(pass: &mut ComputePassDesc, x: u32, y: u32, z: u32) {
    pass.workgroup_size = WorkgroupSize { x, y, z };
}

#[allow(dead_code)]
pub fn total_invocations(pass: &ComputePassDesc) -> u64 {
    let wg = &pass.workgroup_size;
    let d = &pass.dispatch_count;
    (wg.x as u64) * (wg.y as u64) * (wg.z as u64) * (d[0] as u64) * (d[1] as u64) * (d[2] as u64)
}

#[allow(dead_code)]
pub fn compute_pass_to_json(pass: &ComputePassDesc) -> String {
    format!(
        r#"{{"label":"{}","entry":"{}","invocations":{}}}"#,
        pass.label,
        pass.shader_entry,
        total_invocations(pass)
    )
}

#[allow(dead_code)]
pub fn dispatch_groups_for_elements(elements: u32, group_size: u32) -> u32 {
    if group_size == 0 {
        return 0;
    }
    elements.div_ceil(group_size)
}

#[allow(dead_code)]
pub fn is_power_of_two(v: u32) -> bool {
    v > 0 && v & (v - 1) == 0
}

#[allow(dead_code)]
pub fn validate_compute_pass(pass: &ComputePassDesc) -> bool {
    let wg = &pass.workgroup_size;
    wg.x > 0
        && wg.y > 0
        && wg.z > 0
        && pass.dispatch_count.iter().all(|&d| d > 0)
        && !pass.label.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_workgroup() {
        let wg = default_workgroup_size();
        assert_eq!(wg.x, 64);
        assert_eq!(wg.y, 1);
    }

    #[test]
    fn test_new_compute_pass() {
        let p = new_compute_pass("test", "main");
        assert_eq!(p.label, "test");
        assert_eq!(p.shader_entry, "main");
    }

    #[test]
    fn test_set_dispatch() {
        let mut p = new_compute_pass("test", "main");
        set_dispatch(&mut p, 4, 4, 1);
        assert_eq!(p.dispatch_count, [4, 4, 1]);
    }

    #[test]
    fn test_total_invocations() {
        let mut p = new_compute_pass("test", "main");
        set_dispatch(&mut p, 2, 1, 1);
        assert_eq!(total_invocations(&p), 128);
    }

    #[test]
    fn test_compute_pass_to_json() {
        let p = new_compute_pass("skinning", "main");
        let j = compute_pass_to_json(&p);
        assert!(j.contains("skinning"));
    }

    #[test]
    fn test_dispatch_groups() {
        assert_eq!(dispatch_groups_for_elements(100, 64), 2);
        assert_eq!(dispatch_groups_for_elements(64, 64), 1);
    }

    #[test]
    fn test_dispatch_groups_zero() {
        assert_eq!(dispatch_groups_for_elements(100, 0), 0);
    }

    #[test]
    fn test_is_power_of_two() {
        assert!(is_power_of_two(64));
        assert!(!is_power_of_two(65));
        assert!(!is_power_of_two(0));
    }

    #[test]
    fn test_validate_pass() {
        let p = new_compute_pass("test", "main");
        assert!(validate_compute_pass(&p));
    }

    #[test]
    fn test_validate_empty_label() {
        let p = new_compute_pass("", "main");
        assert!(!validate_compute_pass(&p));
    }
}
