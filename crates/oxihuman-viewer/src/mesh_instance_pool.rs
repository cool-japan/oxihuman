#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct MeshInstance {
    mesh_id: u32,
    transform: [f32; 16],
    visible: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshInstancePool {
    instances: Vec<MeshInstance>,
}

#[allow(dead_code)]
pub fn new_mesh_instance_pool() -> MeshInstancePool {
    MeshInstancePool { instances: Vec::new() }
}

#[allow(dead_code)]
pub fn add_mesh_instance(pool: &mut MeshInstancePool, mesh_id: u32, transform: [f32; 16]) -> usize {
    let idx = pool.instances.len();
    pool.instances.push(MeshInstance { mesh_id, transform, visible: true });
    idx
}

#[allow(dead_code)]
pub fn instance_count_mip(pool: &MeshInstancePool) -> usize { pool.instances.len() }

#[allow(dead_code)]
pub fn instance_at_mip(pool: &MeshInstancePool, idx: usize) -> u32 {
    if idx < pool.instances.len() { pool.instances[idx].mesh_id } else { 0 }
}

#[allow(dead_code)]
pub fn instance_transform_mip(pool: &MeshInstancePool, idx: usize) -> [f32; 16] {
    if idx < pool.instances.len() { pool.instances[idx].transform } else { [0.0; 16] }
}

#[allow(dead_code)]
pub fn instance_pool_to_json(pool: &MeshInstancePool) -> String {
    format!("{{\"count\":{}}}", pool.instances.len())
}

#[allow(dead_code)]
pub fn clear_instance_pool(pool: &mut MeshInstancePool) { pool.instances.clear(); }

#[allow(dead_code)]
pub fn instance_is_visible(pool: &MeshInstancePool, idx: usize) -> bool {
    if idx < pool.instances.len() { pool.instances[idx].visible } else { false }
}

fn identity_matrix() -> [f32; 16] {
    let mut m = [0.0_f32; 16];
    m[0] = 1.0; m[5] = 1.0; m[10] = 1.0; m[15] = 1.0;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let p = new_mesh_instance_pool(); assert_eq!(instance_count_mip(&p), 0); }
    #[test] fn test_add() { let mut p = new_mesh_instance_pool(); add_mesh_instance(&mut p, 1, identity_matrix()); assert_eq!(instance_count_mip(&p), 1); }
    #[test] fn test_at() { let mut p = new_mesh_instance_pool(); add_mesh_instance(&mut p, 42, identity_matrix()); assert_eq!(instance_at_mip(&p, 0), 42); }
    #[test] fn test_at_oob() { let p = new_mesh_instance_pool(); assert_eq!(instance_at_mip(&p, 0), 0); }
    #[test] fn test_transform() { let mut p = new_mesh_instance_pool(); let m = identity_matrix(); add_mesh_instance(&mut p, 1, m); assert!((instance_transform_mip(&p, 0)[0] - 1.0).abs() < 1e-6); }
    #[test] fn test_json() { let p = new_mesh_instance_pool(); assert!(instance_pool_to_json(&p).contains("count")); }
    #[test] fn test_clear() { let mut p = new_mesh_instance_pool(); add_mesh_instance(&mut p, 1, identity_matrix()); clear_instance_pool(&mut p); assert_eq!(instance_count_mip(&p), 0); }
    #[test] fn test_visible() { let mut p = new_mesh_instance_pool(); add_mesh_instance(&mut p, 1, identity_matrix()); assert!(instance_is_visible(&p, 0)); }
    #[test] fn test_visible_oob() { let p = new_mesh_instance_pool(); assert!(!instance_is_visible(&p, 0)); }
    #[test] fn test_multiple() { let mut p = new_mesh_instance_pool(); for i in 0..5 { add_mesh_instance(&mut p, i, identity_matrix()); } assert_eq!(instance_count_mip(&p), 5); }
}
