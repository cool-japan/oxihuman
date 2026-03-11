// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::{bail, Context, Result};

#[derive(Debug, Clone)]
pub struct VertexBinding {
    pub base_verts: [u32; 3],
    pub weights: [f32; 3],
    pub offset: [f32; 3],
}

#[derive(Debug, Clone, Default)]
pub struct ClothingBinding {
    pub uuid: String,
    pub basemesh: String,
    pub name: String,
    pub obj_file: String,
    pub vertex_map: Vec<VertexBinding>,
}

pub fn parse_mhclo(src: &str) -> Result<ClothingBinding> {
    let mut binding = ClothingBinding::default();
    let mut in_verts = false;
    let mut expected_count: Option<usize> = None;

    for (lineno, line) in src.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if in_verts {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 9 {
                let bv0: u32 = parts[0]
                    .parse()
                    .with_context(|| format!("line {}: bv0", lineno))?;
                let bv1: u32 = parts[1]
                    .parse()
                    .with_context(|| format!("line {}: bv1", lineno))?;
                let bv2: u32 = parts[2]
                    .parse()
                    .with_context(|| format!("line {}: bv2", lineno))?;
                let w0: f32 = parts[3]
                    .parse()
                    .with_context(|| format!("line {}: w0", lineno))?;
                let w1: f32 = parts[4]
                    .parse()
                    .with_context(|| format!("line {}: w1", lineno))?;
                let w2: f32 = parts[5]
                    .parse()
                    .with_context(|| format!("line {}: w2", lineno))?;
                let ox: f32 = parts[6]
                    .parse()
                    .with_context(|| format!("line {}: ox", lineno))?;
                let oy: f32 = parts[7]
                    .parse()
                    .with_context(|| format!("line {}: oy", lineno))?;
                let oz: f32 = parts[8]
                    .parse()
                    .with_context(|| format!("line {}: oz", lineno))?;
                binding.vertex_map.push(VertexBinding {
                    base_verts: [bv0, bv1, bv2],
                    weights: [w0, w1, w2],
                    offset: [ox, oy, oz],
                });
            }
            continue;
        }

        // key-value lines
        if let Some((key, val)) = line.split_once(char::is_whitespace) {
            match key.trim() {
                "uuid" => binding.uuid = val.trim().to_string(),
                "basemesh" => binding.basemesh = val.trim().to_string(),
                "name" => binding.name = val.trim().to_string(),
                "obj_file" => binding.obj_file = val.trim().to_string(),
                "verts" => {
                    expected_count = Some(
                        val.trim()
                            .parse()
                            .with_context(|| format!("line {}: verts count", lineno))?,
                    );
                    in_verts = true;
                }
                _ => {}
            }
        }
    }

    if let Some(expected) = expected_count {
        if binding.vertex_map.len() != expected {
            bail!(
                "expected {} vertex bindings, got {}",
                expected,
                binding.vertex_map.len()
            );
        }
    }

    Ok(binding)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
uuid 9776b2e4-test
basemesh hm08
name base
obj_file base.obj
verts 2
 6736  6735  7315 0.00000 0.79324 0.20676 0.00000 0.00356 0.02785
 6737  6736  7316 0.10000 0.60000 0.30000 0.00100 0.00200 0.00300
"#;

    #[test]
    fn parse_basic_mhclo() {
        let b = parse_mhclo(SAMPLE).unwrap();
        assert_eq!(b.uuid, "9776b2e4-test");
        assert_eq!(b.basemesh, "hm08");
        assert_eq!(b.vertex_map.len(), 2);
        assert_eq!(b.vertex_map[0].base_verts, [6736, 6735, 7315]);
        assert!((b.vertex_map[0].weights[1] - 0.79324).abs() < 1e-5);
    }
}
