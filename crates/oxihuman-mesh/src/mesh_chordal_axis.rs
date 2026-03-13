// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Chordal axis transform for triangle meshes (skeleton from Delaunay triangulation).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChordalSegment {
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub face_index: usize,
}

#[allow(dead_code)]
pub fn edge_midpoint(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [(a[0]+b[0])*0.5, (a[1]+b[1])*0.5, (a[2]+b[2])*0.5]
}

#[allow(dead_code)]
pub fn triangle_centroid(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    [(a[0]+b[0]+c[0])/3.0, (a[1]+b[1]+c[1])/3.0, (a[2]+b[2]+c[2])/3.0]
}

#[allow(dead_code)]
pub fn chordal_axis_segments(positions: &[[f32; 3]], faces: &[[u32; 3]]) -> Vec<ChordalSegment> {
    let mut segments = Vec::new();
    for (fi, f) in faces.iter().enumerate() {
        let (a, b, c) = (positions[f[0] as usize], positions[f[1] as usize], positions[f[2] as usize]);
        let cent = triangle_centroid(a, b, c);
        for k in 0..3 {
            let p0 = positions[f[k] as usize];
            let p1 = positions[f[(k + 1) % 3] as usize];
            let mid = edge_midpoint(p0, p1);
            segments.push(ChordalSegment { start: cent, end: mid, face_index: fi });
        }
    }
    segments
}

#[allow(dead_code)]
pub fn chordal_segment_length(seg: &ChordalSegment) -> f32 {
    let d = [seg.end[0]-seg.start[0], seg.end[1]-seg.start[1], seg.end[2]-seg.start[2]];
    (d[0]*d[0]+d[1]*d[1]+d[2]*d[2]).sqrt()
}

#[allow(dead_code)]
pub fn chordal_total_length(segs: &[ChordalSegment]) -> f32 {
    segs.iter().map(chordal_segment_length).sum()
}

#[allow(dead_code)]
pub fn chordal_segment_count(segs: &[ChordalSegment]) -> usize { segs.len() }

#[allow(dead_code)]
pub fn chordal_to_json(segs: &[ChordalSegment]) -> String {
    format!("{{\"segments\":{},\"total_length\":{:.4}}}", segs.len(), chordal_total_length(segs))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tri() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        (vec![[0.0,0.0,0.0],[3.0,0.0,0.0],[0.0,3.0,0.0]], vec![[0,1,2]])
    }

    #[test] fn test_edge_mid() { let m = edge_midpoint([0.0,0.0,0.0],[2.0,0.0,0.0]); assert!((m[0]-1.0).abs()<1e-5); }
    #[test] fn test_centroid() { let c = triangle_centroid([0.0,0.0,0.0],[3.0,0.0,0.0],[0.0,3.0,0.0]); assert!((c[0]-1.0).abs()<1e-5); }
    #[test] fn test_segments_count() { let(p,f)=tri(); let s=chordal_axis_segments(&p,&f); assert_eq!(s.len(),3); }
    #[test] fn test_segment_length() { let(p,f)=tri(); let s=chordal_axis_segments(&p,&f); assert!(chordal_segment_length(&s[0])>0.0); }
    #[test] fn test_total_length() { let(p,f)=tri(); let s=chordal_axis_segments(&p,&f); assert!(chordal_total_length(&s)>0.0); }
    #[test] fn test_count_fn() { let(p,f)=tri(); let s=chordal_axis_segments(&p,&f); assert_eq!(chordal_segment_count(&s),3); }
    #[test] fn test_to_json() { let(p,f)=tri(); let s=chordal_axis_segments(&p,&f); assert!(chordal_to_json(&s).contains("total_length")); }
    #[test] fn test_empty() { let s=chordal_axis_segments(&[],&[]); assert!(s.is_empty()); }
    #[test] fn test_face_index() { let(p,f)=tri(); let s=chordal_axis_segments(&p,&f); assert_eq!(s[0].face_index, 0); }
    #[test] fn test_two_tris() {
        let p = vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0],[1.0,1.0,0.0]];
        let f = vec![[0,1,2],[1,3,2]];
        let s = chordal_axis_segments(&p,&f);
        assert_eq!(s.len(), 6);
    }
}
