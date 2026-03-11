#![allow(dead_code)]
//! Face extrusion operations.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceExtrude {
    pub positions: Vec<[f32;3]>,
    pub indices: Vec<u32>,
    pub amount: f32,
    pub direction: [f32;3],
}

fn cross3(a:[f32;3],b:[f32;3])->[f32;3]{[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]}
fn sub3(a:[f32;3],b:[f32;3])->[f32;3]{[a[0]-b[0],a[1]-b[1],a[2]-b[2]]}
fn norm3(a:[f32;3])->f32{(a[0]*a[0]+a[1]*a[1]+a[2]*a[2]).sqrt()}
fn normalize3(a:[f32;3])->[f32;3]{let l=norm3(a);if l<1e-10{[0.0,1.0,0.0]}else{[a[0]/l,a[1]/l,a[2]/l]}}

#[allow(dead_code)]
pub fn extrude_face(positions: &[[f32;3]], indices: &[u32], face_idx: usize, amount: f32) -> FaceExtrude {
    extrude_faces(positions, indices, &[face_idx], amount)
}

#[allow(dead_code)]
pub fn extrude_faces(positions: &[[f32;3]], indices: &[u32], face_indices: &[usize], amount: f32) -> FaceExtrude {
    let mut new_pos = positions.to_vec();
    let mut new_idx = indices.to_vec();
    let dir = if !face_indices.is_empty() {
        let fi = face_indices[0];
        if fi * 3 + 2 < indices.len() {
            let (a,b,c) = (indices[fi*3] as usize, indices[fi*3+1] as usize, indices[fi*3+2] as usize);
            normalize3(cross3(sub3(positions[b],positions[a]), sub3(positions[c],positions[a])))
        } else { [0.0, 1.0, 0.0] }
    } else { [0.0, 1.0, 0.0] };
    for &fi in face_indices {
        if fi * 3 + 2 >= indices.len() { continue; }
        let base = new_pos.len() as u32;
        for k in 0..3 {
            let vi = indices[fi*3+k] as usize;
            let p = positions[vi];
            new_pos.push([p[0]+dir[0]*amount, p[1]+dir[1]*amount, p[2]+dir[2]*amount]);
        }
        new_idx.push(base); new_idx.push(base+1); new_idx.push(base+2);
    }
    FaceExtrude { positions: new_pos, indices: new_idx, amount, direction: dir }
}

#[allow(dead_code)]
pub fn extrude_amount(fe: &FaceExtrude) -> f32 { fe.amount }
#[allow(dead_code)]
pub fn extrude_direction(fe: &FaceExtrude) -> [f32;3] { fe.direction }
#[allow(dead_code)]
pub fn extrude_vertex_count(fe: &FaceExtrude) -> usize { fe.positions.len() }
#[allow(dead_code)]
pub fn extrude_face_count(fe: &FaceExtrude) -> usize { fe.indices.len() / 3 }
#[allow(dead_code)]
pub fn extrude_to_json(fe: &FaceExtrude) -> String {
    format!("{{\"vertices\":{},\"faces\":{},\"amount\":{:.6}}}", fe.positions.len(), fe.indices.len()/3, fe.amount)
}
#[allow(dead_code)]
pub fn extrude_validate(fe: &FaceExtrude) -> bool { fe.indices.len().is_multiple_of(3) && fe.indices.iter().all(|&i| (i as usize) < fe.positions.len()) }

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> (Vec<[f32;3]>, Vec<u32>) { (vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0]], vec![0,1,2]) }
    #[test] fn test_extrude_face() { let (p,i) = data(); let fe = extrude_face(&p,&i, 0, 1.0); assert_eq!(extrude_vertex_count(&fe), 6); }
    #[test] fn test_extrude_faces() { let (p,i) = data(); let fe = extrude_faces(&p,&i, &[0], 1.0); assert_eq!(extrude_face_count(&fe), 2); }
    #[test] fn test_amount() { let (p,i) = data(); let fe = extrude_face(&p,&i, 0, 2.5); assert!((extrude_amount(&fe) - 2.5).abs() < 1e-6); }
    #[test] fn test_direction() { let (p,i) = data(); let fe = extrude_face(&p,&i, 0, 1.0); let d = extrude_direction(&fe); assert!(norm3(d) > 0.9); }
    #[test] fn test_vertex_count() { let (p,i) = data(); let fe = extrude_face(&p,&i, 0, 1.0); assert_eq!(extrude_vertex_count(&fe), 6); }
    #[test] fn test_face_count() { let (p,i) = data(); let fe = extrude_face(&p,&i, 0, 1.0); assert_eq!(extrude_face_count(&fe), 2); }
    #[test] fn test_json() { let (p,i) = data(); let fe = extrude_face(&p,&i, 0, 1.0); assert!(extrude_to_json(&fe).contains("vertices")); }
    #[test] fn test_validate() { let (p,i) = data(); let fe = extrude_face(&p,&i, 0, 1.0); assert!(extrude_validate(&fe)); }
    #[test] fn test_empty() { let fe = extrude_faces(&[],&[], &[], 1.0); assert_eq!(extrude_vertex_count(&fe), 0); }
    #[test] fn test_zero_amount() { let (p,i) = data(); let fe = extrude_face(&p,&i, 0, 0.0); assert_eq!(extrude_vertex_count(&fe), 6); }
}
