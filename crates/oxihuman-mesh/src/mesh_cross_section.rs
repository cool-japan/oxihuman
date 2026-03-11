// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cross section: extract a 2D cross-section polygon from a mesh at a plane.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CrossSectionConfig {
    pub axis: usize,
    pub value: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CrossSectionResult {
    pub points: Vec<[f32; 3]>,
    pub segment_count: usize,
}

#[allow(dead_code)]
pub fn default_cross_section_config() -> CrossSectionConfig {
    CrossSectionConfig { axis: 1, value: 0.0 }
}

#[allow(dead_code)]
pub fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [a[0]+(b[0]-a[0])*t, a[1]+(b[1]-a[1])*t, a[2]+(b[2]-a[2])*t]
}

#[allow(dead_code)]
pub fn extract_cross_section(positions: &[[f32; 3]], indices: &[[u32; 3]], config: &CrossSectionConfig) -> CrossSectionResult {
    let ax = config.axis.min(2);
    let val = config.value;
    let mut points = Vec::new();
    for tri in indices {
        let p = [positions[tri[0] as usize], positions[tri[1] as usize], positions[tri[2] as usize]];
        let h = [p[0][ax], p[1][ax], p[2][ax]];
        let mut cross = Vec::new();
        for i in 0..3 {
            let j = (i+1) % 3;
            if (h[i]-val)*(h[j]-val) < 0.0 {
                let t = (val-h[i])/(h[j]-h[i]);
                cross.push(lerp3(p[i], p[j], t));
            }
        }
        if cross.len() >= 2 { points.push(cross[0]); points.push(cross[1]); }
    }
    let seg_count = points.len() / 2;
    CrossSectionResult { points, segment_count: seg_count }
}

#[allow(dead_code)]
pub fn cross_section_point_count(result: &CrossSectionResult) -> usize { result.points.len() }

#[allow(dead_code)]
pub fn cross_section_bounds(result: &CrossSectionResult) -> ([f32; 3], [f32; 3]) {
    if result.points.is_empty() { return ([0.0;3],[0.0;3]); }
    let mut mn = result.points[0]; let mut mx = result.points[0];
    for p in &result.points {
        for i in 0..3 {
            if p[i] < mn[i] {
                mn[i] = p[i];
            } else if p[i] > mx[i] {
                mx[i] = p[i];
            }
        }
    }
    (mn, mx)
}

#[allow(dead_code)]
pub fn cross_section_to_json(result: &CrossSectionResult) -> String {
    format!("{{\"segments\":{},\"points\":{}}}", result.segment_count, result.points.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn slope() -> (Vec<[f32;3]>, Vec<[u32;3]>) {
        (vec![[0.0,-1.0,0.0],[1.0,-1.0,0.0],[0.5,1.0,0.0]], vec![[0,1,2]])
    }
    #[test] fn test_default_config() { let c=default_cross_section_config(); assert_eq!(c.axis,1); }
    #[test] fn test_lerp3() { let p=lerp3([0.0;3],[2.0,2.0,2.0],0.5); assert!((p[0]-1.0).abs()<1e-6); }
    #[test] fn test_extract() { let(p,i)=slope(); let r=extract_cross_section(&p,&i,&default_cross_section_config()); assert!(!r.points.is_empty()); }
    #[test] fn test_no_intersect() { let p=vec![[0.0,5.0,0.0],[1.0,5.0,0.0],[0.5,6.0,0.0]]; let i=vec![[0,1,2]]; let r=extract_cross_section(&p,&i,&default_cross_section_config()); assert!(r.points.is_empty()); }
    #[test] fn test_point_count() { let(p,i)=slope(); let r=extract_cross_section(&p,&i,&default_cross_section_config()); assert!(cross_section_point_count(&r)>0); }
    #[test] fn test_bounds() { let(p,i)=slope(); let r=extract_cross_section(&p,&i,&default_cross_section_config()); let(mn,mx)=cross_section_bounds(&r); assert!(mx[0]>=mn[0]); }
    #[test] fn test_to_json() { let(p,i)=slope(); let r=extract_cross_section(&p,&i,&default_cross_section_config()); assert!(cross_section_to_json(&r).contains("segments")); }
    #[test] fn test_empty() { let r=extract_cross_section(&[],&[],&default_cross_section_config()); assert_eq!(r.segment_count,0); }
    #[test] fn test_x_axis() { let(p,i)=slope(); let c=CrossSectionConfig{axis:0,value:0.5}; let r=extract_cross_section(&p,&i,&c); assert!(r.segment_count<=2); }
    #[test] fn test_segment_count() { let(p,i)=slope(); let r=extract_cross_section(&p,&i,&default_cross_section_config()); assert_eq!(r.segment_count, r.points.len()/2); }
}
