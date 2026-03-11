// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct Delta {
    pub vid: u32,
    pub dx: f32,
    pub dy: f32,
    pub dz: f32,
}

#[derive(Debug, Clone)]
pub struct TargetFile {
    pub name: String,
    pub deltas: Vec<Delta>,
}

pub fn parse_target(name: &str, src: &str) -> Result<TargetFile> {
    let mut deltas = Vec::new();
    for (lineno, line) in src.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 4 {
            continue; // skip malformed lines
        }
        let vid: u32 = parts[0]
            .parse()
            .with_context(|| format!("line {}: bad vid", lineno + 1))?;
        let dx: f32 = parts[1]
            .parse()
            .with_context(|| format!("line {}: bad dx", lineno + 1))?;
        let dy: f32 = parts[2]
            .parse()
            .with_context(|| format!("line {}: bad dy", lineno + 1))?;
        let dz: f32 = parts[3]
            .parse()
            .with_context(|| format!("line {}: bad dz", lineno + 1))?;
        deltas.push(Delta { vid, dx, dy, dz });
    }
    deltas.sort_unstable_by_key(|d| d.vid);
    Ok(TargetFile {
        name: name.to_string(),
        deltas,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_target() {
        let src = "# comment\n1358 0 -.006 -.006\n1359 .001 -.004 -.005\n";
        let t = parse_target("test", src).unwrap();
        assert_eq!(t.name, "test");
        assert_eq!(t.deltas.len(), 2);
        assert_eq!(t.deltas[0].vid, 1358);
        assert!((t.deltas[0].dz + 0.006).abs() < 1e-6);
        assert_eq!(t.deltas[1].vid, 1359);
    }

    #[test]
    fn skip_malformed_lines() {
        let src = "# header\n1 2 3\n100 0.1 0.2 0.3\n";
        let t = parse_target("t", src).unwrap();
        assert_eq!(t.deltas.len(), 1);
        assert_eq!(t.deltas[0].vid, 100);
    }

    #[test]
    fn deltas_are_sorted_by_vid() {
        let src = "# comment\n500 0.1 0.2 0.3\n100 0.4 0.5 0.6\n300 0.7 0.8 0.9\n";
        let t = parse_target("sorted", src).unwrap();
        assert_eq!(t.deltas[0].vid, 100);
        assert_eq!(t.deltas[1].vid, 300);
        assert_eq!(t.deltas[2].vid, 500);
    }

    #[test]
    fn parse_real_target_file() {
        // Load a real .target file from the MakeHuman resource directory
        let path = "/media/kitasan/Backup/resource/makehuman/makehuman/data/targets/armslegs";
        let read_dir = match std::fs::read_dir(path) {
            Ok(d) => d,
            Err(_) => return, // skip if resource path not available on this machine
        };
        let entries: Vec<_> = read_dir
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|x| x == "target").unwrap_or(false))
            .take(1)
            .collect();
        if let Some(entry) = entries.first() {
            let src = std::fs::read_to_string(entry.path()).unwrap();
            let t = parse_target("real", &src).unwrap();
            assert!(!t.deltas.is_empty(), "real target should have deltas");
        }
    }
}
