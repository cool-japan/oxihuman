// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Face split operations: split faces at midpoints, centers, or edges.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceSplitOpsResult { pub positions: Vec<[f32;3]>, pub indices: Vec<[u32;3]>, pub new_vertices: usize, pub new_faces: usize }

#[allow(dead_code)]
pub fn face_center(a:[f32;3],b:[f32;3],c:[f32;3]) -> [f32;3] {
    [(a[0]+b[0]+c[0])/3.0,(a[1]+b[1]+c[1])/3.0,(a[2]+b[2]+c[2])/3.0]
}

#[allow(dead_code)]
pub fn split_face_at_center(positions:&[[f32;3]], tri:[u32;3]) -> ([f32;3], [[u32;3];3]) {
    let c = face_center(positions[tri[0] as usize],positions[tri[1] as usize],positions[tri[2] as usize]);
    let ci = positions.len() as u32;
    (c, [[tri[0],tri[1],ci],[tri[1],tri[2],ci],[tri[2],tri[0],ci]])
}

#[allow(dead_code)]
pub fn split_all_faces(positions:&[[f32;3]], indices:&[[u32;3]]) -> FaceSplitOpsResult {
    let mut new_pos = positions.to_vec(); let mut new_idx = Vec::new();
    for tri in indices {
        let (c, _tris) = split_face_at_center(&new_pos, *tri);
        let ci = new_pos.len() as u32; new_pos.push(c);
        new_idx.push([tri[0],tri[1],ci]); new_idx.push([tri[1],tri[2],ci]); new_idx.push([tri[2],tri[0],ci]);
    }
    let _ = new_idx;
    let new_v = new_pos.len()-positions.len();
    let new_f = indices.len()*3;
    let mut result_idx = Vec::new();
    let mut pos2 = positions.to_vec();
    for tri in indices {
        let c = face_center(positions[tri[0] as usize],positions[tri[1] as usize],positions[tri[2] as usize]);
        let ci = pos2.len() as u32; pos2.push(c);
        result_idx.push([tri[0],tri[1],ci]); result_idx.push([tri[1],tri[2],ci]); result_idx.push([tri[2],tri[0],ci]);
    }
    FaceSplitOpsResult { positions: pos2, indices: result_idx, new_vertices: new_v, new_faces: new_f }
}

#[allow(dead_code)]
pub fn split_selected_faces(positions:&[[f32;3]], indices:&[[u32;3]], face_indices:&[usize]) -> FaceSplitOpsResult {
    let selected: std::collections::HashSet<usize> = face_indices.iter().copied().collect();
    let mut new_pos = positions.to_vec(); let mut new_idx = Vec::new(); let mut new_v=0; let mut new_f=0;
    for (fi, tri) in indices.iter().enumerate() {
        if selected.contains(&fi) {
            let c = face_center(positions[tri[0] as usize],positions[tri[1] as usize],positions[tri[2] as usize]);
            let ci = new_pos.len() as u32; new_pos.push(c);
            new_idx.push([tri[0],tri[1],ci]); new_idx.push([tri[1],tri[2],ci]); new_idx.push([tri[2],tri[0],ci]);
            new_v+=1; new_f+=3;
        } else { new_idx.push(*tri); }
    }
    FaceSplitOpsResult { positions: new_pos, indices: new_idx, new_vertices: new_v, new_faces: new_f }
}

#[allow(dead_code)]
pub fn face_split_ops_to_json(r:&FaceSplitOpsResult) -> String {
    format!("{{\"vertices\":{},\"faces\":{},\"new_verts\":{},\"new_faces\":{}}}", r.positions.len(), r.indices.len(), r.new_vertices, r.new_faces)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tri() -> (Vec<[f32;3]>,Vec<[u32;3]>) { (vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0]], vec![[0,1,2]]) }
    #[test] fn test_face_center() { let c=face_center([0.0,0.0,0.0],[3.0,0.0,0.0],[0.0,3.0,0.0]); assert!((c[0]-1.0).abs()<1e-6); }
    #[test] fn test_split_face_at_center() { let p=vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0]]; let(_,tris)=split_face_at_center(&p,[0,1,2]); assert_eq!(tris.len(),3); }
    #[test] fn test_split_all() { let(p,i)=tri(); let r=split_all_faces(&p,&i); assert_eq!(r.indices.len(),3); }
    #[test] fn test_new_vertices() { let(p,i)=tri(); let r=split_all_faces(&p,&i); assert_eq!(r.positions.len(), p.len()+i.len()); }
    #[test] fn test_selected() { let(p,i)=tri(); let r=split_selected_faces(&p,&i,&[0]); assert_eq!(r.indices.len(),3); }
    #[test] fn test_none_selected() { let(p,i)=tri(); let r=split_selected_faces(&p,&i,&[]); assert_eq!(r.indices.len(),1); }
    #[test] fn test_to_json() { let(p,i)=tri(); let r=split_all_faces(&p,&i); assert!(face_split_ops_to_json(&r).contains("new_verts")); }
    #[test] fn test_empty() { let r=split_all_faces(&[],&[]); assert!(r.positions.is_empty()); }
    #[test] fn test_two_faces() {
        let p=vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0],[1.5,1.0,0.0]];
        let i=vec![[0,1,2],[1,3,2]]; let r=split_all_faces(&p,&i); assert_eq!(r.indices.len(),6);
    }
    #[test] fn test_new_face_count() { let(p,i)=tri(); let r=split_all_faces(&p,&i); assert_eq!(r.new_faces,3); }
}
