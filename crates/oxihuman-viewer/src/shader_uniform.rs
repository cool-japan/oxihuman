// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! ShaderUniform — shader uniform block management.

#![allow(dead_code)]

/// A single shader uniform value.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ShaderUniform {
    Float(f32),
    Vec3([f32; 3]),
    Mat4([[f32; 4]; 4]),
}

/// Named entry in a uniform block.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct UniformEntry {
    name: String,
    value: ShaderUniform,
}

/// A block of named uniforms.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct UniformBlock {
    entries: Vec<UniformEntry>,
}

/// Create a new empty uniform block.
#[allow(dead_code)]
pub fn new_uniform_block() -> UniformBlock {
    UniformBlock::default()
}

/// Set a float uniform.
#[allow(dead_code)]
pub fn set_uniform_f32(block: &mut UniformBlock, name: &str, value: f32) {
    if let Some(e) = block.entries.iter_mut().find(|e| e.name == name) {
        e.value = ShaderUniform::Float(value);
    } else {
        block.entries.push(UniformEntry {
            name: name.to_owned(),
            value: ShaderUniform::Float(value),
        });
    }
}

/// Set a vec3 uniform.
#[allow(dead_code)]
pub fn set_uniform_vec3(block: &mut UniformBlock, name: &str, value: [f32; 3]) {
    if let Some(e) = block.entries.iter_mut().find(|e| e.name == name) {
        e.value = ShaderUniform::Vec3(value);
    } else {
        block.entries.push(UniformEntry {
            name: name.to_owned(),
            value: ShaderUniform::Vec3(value),
        });
    }
}

/// Set a mat4 uniform.
#[allow(dead_code)]
pub fn set_uniform_mat4(block: &mut UniformBlock, name: &str, value: [[f32; 4]; 4]) {
    if let Some(e) = block.entries.iter_mut().find(|e| e.name == name) {
        e.value = ShaderUniform::Mat4(value);
    } else {
        block.entries.push(UniformEntry {
            name: name.to_owned(),
            value: ShaderUniform::Mat4(value),
        });
    }
}

/// Number of uniforms in the block.
#[allow(dead_code)]
pub fn uniform_count(block: &UniformBlock) -> usize {
    block.entries.len()
}

/// Byte size of the uniform block (stub: f32=4, vec3=12, mat4=64).
#[allow(dead_code)]
pub fn uniform_block_size(block: &UniformBlock) -> usize {
    block
        .entries
        .iter()
        .map(|e| match &e.value {
            ShaderUniform::Float(_) => 4,
            ShaderUniform::Vec3(_) => 12,
            ShaderUniform::Mat4(_) => 64,
        })
        .sum()
}

/// Return the name of the uniform at `index`.
#[allow(dead_code)]
pub fn uniform_name_at(block: &UniformBlock, index: usize) -> Option<&str> {
    block.entries.get(index).map(|e| e.name.as_str())
}

/// Serialize the uniform block to raw bytes (little-endian).
#[allow(dead_code)]
pub fn uniform_to_bytes(block: &UniformBlock) -> Vec<u8> {
    let mut bytes = Vec::new();
    for entry in &block.entries {
        match &entry.value {
            ShaderUniform::Float(v) => bytes.extend_from_slice(&v.to_le_bytes()),
            ShaderUniform::Vec3(v) => {
                for c in v {
                    bytes.extend_from_slice(&c.to_le_bytes());
                }
            }
            ShaderUniform::Mat4(m) => {
                for col in m {
                    for c in col {
                        bytes.extend_from_slice(&c.to_le_bytes());
                    }
                }
            }
        }
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_uniform_block() {
        let b = new_uniform_block();
        assert_eq!(uniform_count(&b), 0);
    }

    #[test]
    fn test_set_uniform_f32() {
        let mut b = new_uniform_block();
        set_uniform_f32(&mut b, "alpha", 0.5);
        assert_eq!(uniform_count(&b), 1);
    }

    #[test]
    fn test_set_uniform_vec3() {
        let mut b = new_uniform_block();
        set_uniform_vec3(&mut b, "color", [1.0, 0.0, 0.0]);
        assert_eq!(uniform_count(&b), 1);
    }

    #[test]
    fn test_set_uniform_mat4() {
        let mut b = new_uniform_block();
        let identity = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
        set_uniform_mat4(&mut b, "mvp", identity);
        assert_eq!(uniform_count(&b), 1);
    }

    #[test]
    fn test_uniform_block_size() {
        let mut b = new_uniform_block();
        set_uniform_f32(&mut b, "a", 1.0);
        set_uniform_vec3(&mut b, "b", [0.0; 3]);
        assert_eq!(uniform_block_size(&b), 16); // 4 + 12
    }

    #[test]
    fn test_uniform_name_at() {
        let mut b = new_uniform_block();
        set_uniform_f32(&mut b, "alpha", 0.5);
        assert_eq!(uniform_name_at(&b, 0), Some("alpha"));
        assert_eq!(uniform_name_at(&b, 1), None);
    }

    #[test]
    fn test_uniform_to_bytes_float() {
        let mut b = new_uniform_block();
        set_uniform_f32(&mut b, "x", 1.0);
        let bytes = uniform_to_bytes(&b);
        assert_eq!(bytes.len(), 4);
    }

    #[test]
    fn test_overwrite_uniform() {
        let mut b = new_uniform_block();
        set_uniform_f32(&mut b, "x", 1.0);
        set_uniform_f32(&mut b, "x", 2.0);
        assert_eq!(uniform_count(&b), 1);
    }

    #[test]
    fn test_uniform_to_bytes_vec3() {
        let mut b = new_uniform_block();
        set_uniform_vec3(&mut b, "c", [1.0, 2.0, 3.0]);
        let bytes = uniform_to_bytes(&b);
        assert_eq!(bytes.len(), 12);
    }

    #[test]
    fn test_uniform_to_bytes_mat4() {
        let mut b = new_uniform_block();
        set_uniform_mat4(&mut b, "m", [[0.0; 4]; 4]);
        let bytes = uniform_to_bytes(&b);
        assert_eq!(bytes.len(), 64);
    }
}
