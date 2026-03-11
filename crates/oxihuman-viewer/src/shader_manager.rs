// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]
//! Shader program manager.

#[allow(dead_code)]
pub struct ShaderProgram {
    pub id: u32,
    pub name: String,
    pub vert_src_len: usize,
    pub frag_src_len: usize,
}

#[allow(dead_code)]
pub struct ShaderManager {
    pub programs: Vec<ShaderProgram>,
    pub next_id: u32,
}

#[allow(dead_code)]
pub fn new_shader_manager() -> ShaderManager {
    ShaderManager { programs: Vec::new(), next_id: 1 }
}

#[allow(dead_code)]
pub fn sm_create(mgr: &mut ShaderManager, name: &str, vert_src: &str, frag_src: &str) -> u32 {
    let id = mgr.next_id;
    mgr.next_id += 1;
    mgr.programs.push(ShaderProgram {
        id,
        name: name.to_string(),
        vert_src_len: vert_src.len(),
        frag_src_len: frag_src.len(),
    });
    id
}

#[allow(dead_code)]
pub fn sm_find(mgr: &ShaderManager, name: &str) -> Option<u32> {
    mgr.programs.iter().find(|p| p.name == name).map(|p| p.id)
}

#[allow(dead_code)]
pub fn sm_count(mgr: &ShaderManager) -> usize {
    mgr.programs.len()
}

#[allow(dead_code)]
pub fn sm_destroy(mgr: &mut ShaderManager, id: u32) -> bool {
    if let Some(idx) = mgr.programs.iter().position(|p| p.id == id) {
        mgr.programs.remove(idx);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create() {
        let mut mgr = new_shader_manager();
        let id = sm_create(&mut mgr, "pbr", "void main(){}", "void main(){}");
        assert!(id > 0);
    }

    #[test]
    fn test_find() {
        let mut mgr = new_shader_manager();
        let id = sm_create(&mut mgr, "pbr", "void main(){}", "void main(){}");
        assert_eq!(sm_find(&mgr, "pbr"), Some(id));
    }

    #[test]
    fn test_find_missing_returns_none() {
        let mgr = new_shader_manager();
        assert_eq!(sm_find(&mgr, "none"), None);
    }

    #[test]
    fn test_count() {
        let mut mgr = new_shader_manager();
        sm_create(&mut mgr, "a", "", "");
        sm_create(&mut mgr, "b", "", "");
        assert_eq!(sm_count(&mgr), 2);
    }

    #[test]
    fn test_destroy() {
        let mut mgr = new_shader_manager();
        let id = sm_create(&mut mgr, "x", "", "");
        assert!(sm_destroy(&mut mgr, id));
        assert_eq!(sm_count(&mgr), 0);
    }

    #[test]
    fn test_destroy_missing() {
        let mut mgr = new_shader_manager();
        assert!(!sm_destroy(&mut mgr, 999));
    }

    #[test]
    fn test_unique_ids() {
        let mut mgr = new_shader_manager();
        let id1 = sm_create(&mut mgr, "a", "", "");
        let id2 = sm_create(&mut mgr, "b", "", "");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_vert_src_len_stored() {
        let mut mgr = new_shader_manager();
        sm_create(&mut mgr, "s", "hello", "");
        assert_eq!(mgr.programs[0].vert_src_len, 5);
    }
}
