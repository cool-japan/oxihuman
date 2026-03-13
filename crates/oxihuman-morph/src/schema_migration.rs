// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Schema version management and migration for morph parameters.
//!
//! Supports evolving parameter schemas over time with automatic migration
//! path finding (BFS) and transformation of parameter data between versions.

use std::collections::{HashMap, VecDeque};
use std::fmt;

/// Schema version for morph parameters.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SchemaVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SchemaVersion {
    /// Create a new schema version.
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Parse a version string like "1.2.3".
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        let parts: Vec<&str> = s.trim().split('.').collect();
        if parts.len() != 3 {
            anyhow::bail!("invalid schema version '{}': expected MAJOR.MINOR.PATCH", s);
        }
        let major = parts[0]
            .parse::<u32>()
            .map_err(|e| anyhow::anyhow!("invalid major version '{}': {}", parts[0], e))?;
        let minor = parts[1]
            .parse::<u32>()
            .map_err(|e| anyhow::anyhow!("invalid minor version '{}': {}", parts[1], e))?;
        let patch = parts[2]
            .parse::<u32>()
            .map_err(|e| anyhow::anyhow!("invalid patch version '{}': {}", parts[2], e))?;
        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Migration operations that transform parameter data.
#[derive(Debug, Clone)]
pub enum MigrationOp {
    /// Add a new parameter with default value.
    AddParameter {
        name: String,
        default: f64,
        min: f64,
        max: f64,
    },
    /// Remove a parameter.
    RemoveParameter { name: String },
    /// Rename a parameter.
    RenameParameter { old_name: String, new_name: String },
    /// Split one parameter into multiple (name, weight).
    SplitParameter {
        source: String,
        targets: Vec<(String, f64)>,
    },
    /// Merge multiple parameters into one (name, weight).
    MergeParameters {
        sources: Vec<(String, f64)>,
        target: String,
    },
    /// Rescale parameter range.
    RescaleParameter {
        name: String,
        old_range: (f64, f64),
        new_range: (f64, f64),
    },
    /// Apply a linear transform to parameter value.
    TransformParameter {
        name: String,
        scale: f64,
        offset: f64,
    },
    /// Add dependency between parameters.
    AddDependency {
        param: String,
        depends_on: String,
        factor: f64,
    },
}

/// A single migration step between schema versions.
pub struct Migration {
    pub from: SchemaVersion,
    pub to: SchemaVersion,
    pub description: String,
    pub operations: Vec<MigrationOp>,
}

impl Migration {
    /// Create a new migration.
    pub fn new(from: SchemaVersion, to: SchemaVersion, description: &str) -> Self {
        Self {
            from,
            to,
            description: description.to_string(),
            operations: Vec::new(),
        }
    }

    /// Add an operation to this migration.
    pub fn add_op(&mut self, op: MigrationOp) {
        self.operations.push(op);
    }

    /// Apply this migration to a parameter set, producing a new parameter set.
    pub fn apply(&self, params: &[(String, f64)]) -> anyhow::Result<Vec<(String, f64)>> {
        let mut result: Vec<(String, f64)> = params.to_vec();

        for op in &self.operations {
            result = apply_op(&result, op)?;
        }

        Ok(result)
    }
}

/// Apply a single migration operation to a parameter list.
fn apply_op(params: &[(String, f64)], op: &MigrationOp) -> anyhow::Result<Vec<(String, f64)>> {
    match op {
        MigrationOp::AddParameter {
            name,
            default,
            min: _,
            max: _,
        } => {
            let mut out = params.to_vec();
            // Only add if not already present
            if !out.iter().any(|(n, _)| n == name) {
                out.push((name.clone(), *default));
            }
            Ok(out)
        }

        MigrationOp::RemoveParameter { name } => {
            let out: Vec<(String, f64)> =
                params.iter().filter(|(n, _)| n != name).cloned().collect();
            Ok(out)
        }

        MigrationOp::RenameParameter { old_name, new_name } => {
            let out: Vec<(String, f64)> = params
                .iter()
                .map(|(n, v)| {
                    if n == old_name {
                        (new_name.clone(), *v)
                    } else {
                        (n.clone(), *v)
                    }
                })
                .collect();
            Ok(out)
        }

        MigrationOp::SplitParameter { source, targets } => {
            let source_val = params.iter().find(|(n, _)| n == source).map(|(_, v)| *v);
            let val = source_val.unwrap_or(0.0);

            let mut out: Vec<(String, f64)> = params
                .iter()
                .filter(|(n, _)| n != source)
                .cloned()
                .collect();

            for (target_name, weight) in targets {
                out.push((target_name.clone(), val * weight));
            }
            Ok(out)
        }

        MigrationOp::MergeParameters { sources, target } => {
            let mut merged_val = 0.0;
            let source_names: Vec<&str> = sources.iter().map(|(n, _)| n.as_str()).collect();

            for (src_name, weight) in sources {
                if let Some((_, v)) = params.iter().find(|(n, _)| n == src_name) {
                    merged_val += v * weight;
                }
            }

            let mut out: Vec<(String, f64)> = params
                .iter()
                .filter(|(n, _)| !source_names.contains(&n.as_str()))
                .cloned()
                .collect();
            out.push((target.clone(), merged_val));
            Ok(out)
        }

        MigrationOp::RescaleParameter {
            name,
            old_range,
            new_range,
        } => {
            let old_span = old_range.1 - old_range.0;
            let new_span = new_range.1 - new_range.0;

            let out: Vec<(String, f64)> = params
                .iter()
                .map(|(n, v)| {
                    if n == name {
                        if old_span.abs() < f64::EPSILON {
                            // Degenerate old range: map to midpoint of new range
                            (n.clone(), new_range.0 + new_span * 0.5)
                        } else {
                            let normalized = (v - old_range.0) / old_span;
                            let new_val = new_range.0 + normalized * new_span;
                            (n.clone(), new_val)
                        }
                    } else {
                        (n.clone(), *v)
                    }
                })
                .collect();
            Ok(out)
        }

        MigrationOp::TransformParameter {
            name,
            scale,
            offset,
        } => {
            let out: Vec<(String, f64)> = params
                .iter()
                .map(|(n, v)| {
                    if n == name {
                        (n.clone(), v * scale + offset)
                    } else {
                        (n.clone(), *v)
                    }
                })
                .collect();
            Ok(out)
        }

        MigrationOp::AddDependency {
            param,
            depends_on,
            factor,
        } => {
            let dep_val = params
                .iter()
                .find(|(n, _)| n == depends_on)
                .map(|(_, v)| *v)
                .unwrap_or(0.0);

            let out: Vec<(String, f64)> = params
                .iter()
                .map(|(n, v)| {
                    if n == param {
                        (n.clone(), v + dep_val * factor)
                    } else {
                        (n.clone(), *v)
                    }
                })
                .collect();
            Ok(out)
        }
    }
}

/// Parameter set with schema version.
#[derive(Debug, Clone)]
pub struct VersionedParams {
    pub version: SchemaVersion,
    pub parameters: Vec<(String, f64)>,
}

impl VersionedParams {
    /// Create a new versioned parameter set.
    pub fn new(version: SchemaVersion, parameters: Vec<(String, f64)>) -> Self {
        Self {
            version,
            parameters,
        }
    }

    /// Look up a parameter value by name.
    pub fn get(&self, name: &str) -> Option<f64> {
        self.parameters
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| *v)
    }

    /// Number of parameters.
    pub fn len(&self) -> usize {
        self.parameters.len()
    }

    /// Whether the parameter set is empty.
    pub fn is_empty(&self) -> bool {
        self.parameters.is_empty()
    }
}

/// Schema migration registry.
///
/// Stores migration steps and finds shortest migration paths via BFS.
pub struct MigrationRegistry {
    migrations: Vec<Migration>,
}

impl Default for MigrationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MigrationRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            migrations: Vec::new(),
        }
    }

    /// Register a migration step.
    pub fn register(&mut self, migration: Migration) {
        self.migrations.push(migration);
    }

    /// Find migration path from source to target version using BFS.
    ///
    /// Returns the ordered list of migrations to apply.
    pub fn find_path(
        &self,
        from: &SchemaVersion,
        to: &SchemaVersion,
    ) -> anyhow::Result<Vec<&Migration>> {
        if from == to {
            return Ok(Vec::new());
        }

        // Build adjacency: version -> list of migration indices
        let mut adj: HashMap<&SchemaVersion, Vec<usize>> = HashMap::new();
        for (i, m) in self.migrations.iter().enumerate() {
            adj.entry(&m.from).or_default().push(i);
        }

        // BFS to find shortest path
        let mut visited: HashMap<&SchemaVersion, Option<usize>> = HashMap::new();
        visited.insert(from, None);

        let mut queue: VecDeque<&SchemaVersion> = VecDeque::new();
        queue.push_back(from);

        let mut found = false;

        while let Some(current) = queue.pop_front() {
            if current == to {
                found = true;
                break;
            }

            if let Some(edges) = adj.get(current) {
                for &idx in edges {
                    let next = &self.migrations[idx].to;
                    if !visited.contains_key(next) {
                        visited.insert(next, Some(idx));
                        queue.push_back(next);
                    }
                }
            }
        }

        if !found {
            anyhow::bail!("no migration path found from version {} to {}", from, to);
        }

        // Reconstruct path by walking backwards from `to`
        let mut path_indices = Vec::new();
        let mut current = to;
        while current != from {
            let idx = visited.get(current).and_then(|opt| *opt).ok_or_else(|| {
                anyhow::anyhow!("internal error: broken BFS parent chain at {}", current)
            })?;
            path_indices.push(idx);
            current = &self.migrations[idx].from;
        }

        path_indices.reverse();
        let path: Vec<&Migration> = path_indices.iter().map(|&i| &self.migrations[i]).collect();
        Ok(path)
    }

    /// Migrate parameters from one version to another.
    pub fn migrate(
        &self,
        params: &VersionedParams,
        target: &SchemaVersion,
    ) -> anyhow::Result<VersionedParams> {
        let path = self.find_path(&params.version, target)?;
        let mut current_params = params.parameters.clone();

        for migration in &path {
            current_params = migration.apply(&current_params)?;
        }

        Ok(VersionedParams {
            version: target.clone(),
            parameters: current_params,
        })
    }

    /// Get current (latest) schema version.
    ///
    /// Returns the maximum `to` version across all registered migrations,
    /// or `None` if no migrations are registered.
    pub fn current_version(&self) -> Option<&SchemaVersion> {
        self.migrations.iter().map(|m| &m.to).max()
    }

    /// Validate that the migration chain is complete (no gaps).
    ///
    /// Checks that every version that appears as a `to` target (except the latest)
    /// also appears as a `from` source of at least one other migration, forming
    /// a connected chain from the earliest to the latest version.
    pub fn validate_chain(&self) -> anyhow::Result<()> {
        if self.migrations.is_empty() {
            return Ok(());
        }

        // Collect all source and target versions
        let mut from_set: std::collections::HashSet<&SchemaVersion> =
            std::collections::HashSet::new();
        let mut to_set: std::collections::HashSet<&SchemaVersion> =
            std::collections::HashSet::new();

        for m in &self.migrations {
            from_set.insert(&m.from);
            to_set.insert(&m.to);
        }

        // Find the earliest version (a `from` that is never a `to`)
        let roots: Vec<&&SchemaVersion> =
            from_set.iter().filter(|v| !to_set.contains(**v)).collect();

        if roots.is_empty() {
            anyhow::bail!("migration chain has no root version (cycle detected)");
        }

        // Find the latest version (a `to` that is never a `from`)
        let leaves: Vec<&&SchemaVersion> =
            to_set.iter().filter(|v| !from_set.contains(**v)).collect();

        if leaves.is_empty() {
            anyhow::bail!("migration chain has no leaf version (cycle detected)");
        }

        // Verify that every root can reach every leaf via BFS
        for root in &roots {
            for leaf in &leaves {
                if self.find_path(root, leaf).is_err() {
                    anyhow::bail!("migration chain gap: no path from {} to {}", root, leaf);
                }
            }
        }

        Ok(())
    }

    /// Create default registry with built-in migrations for OxiHuman.
    pub fn with_defaults() -> Self {
        let mut reg = Self::new();

        // v0.1.0 -> v0.2.0: Split "body_weight" into "body_fat" + "muscle_mass"
        let mut m1 = Migration::new(
            SchemaVersion::new(0, 1, 0),
            SchemaVersion::new(0, 2, 0),
            "Split body_weight into body_fat and muscle_mass",
        );
        m1.add_op(MigrationOp::SplitParameter {
            source: "body_weight".to_string(),
            targets: vec![
                ("body_fat".to_string(), 0.4),
                ("muscle_mass".to_string(), 0.6),
            ],
        });
        reg.register(m1);

        // v0.2.0 -> v0.3.0: Rename "breast_size" to "chest_volume", add "chest_shape"
        let mut m2 = Migration::new(
            SchemaVersion::new(0, 2, 0),
            SchemaVersion::new(0, 3, 0),
            "Rename breast_size to chest_volume, add chest_shape",
        );
        m2.add_op(MigrationOp::RenameParameter {
            old_name: "breast_size".to_string(),
            new_name: "chest_volume".to_string(),
        });
        m2.add_op(MigrationOp::AddParameter {
            name: "chest_shape".to_string(),
            default: 0.5,
            min: 0.0,
            max: 1.0,
        });
        reg.register(m2);

        // v0.3.0 -> v1.0.0: Rescale all parameters from [0,1] to [-1,1], add 10 facial params
        let mut m3 = Migration::new(
            SchemaVersion::new(0, 3, 0),
            SchemaVersion::new(1, 0, 0),
            "Rescale parameters from [0,1] to [-1,1], add facial parameters",
        );

        // Rescale existing params that are carried forward.
        // We rescale the known params from prior migrations:
        for param_name in &["body_fat", "muscle_mass", "chest_volume", "chest_shape"] {
            m3.add_op(MigrationOp::RescaleParameter {
                name: param_name.to_string(),
                old_range: (0.0, 1.0),
                new_range: (-1.0, 1.0),
            });
        }

        // Add 10 new facial parameters
        let facial_params = [
            "face_width",
            "face_length",
            "jaw_width",
            "jaw_angle",
            "cheekbone_height",
            "brow_ridge",
            "nose_bridge_width",
            "nose_length",
            "lip_fullness",
            "chin_projection",
        ];
        for name in &facial_params {
            m3.add_op(MigrationOp::AddParameter {
                name: name.to_string(),
                default: 0.0,
                min: -1.0,
                max: 1.0,
            });
        }
        reg.register(m3);

        reg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_version_parse() {
        let v = SchemaVersion::parse("1.2.3").expect("should parse");
        assert_eq!(v, SchemaVersion::new(1, 2, 3));
    }

    #[test]
    fn test_schema_version_parse_invalid() {
        assert!(SchemaVersion::parse("1.2").is_err());
        assert!(SchemaVersion::parse("abc.0.0").is_err());
        assert!(SchemaVersion::parse("").is_err());
    }

    #[test]
    fn test_schema_version_display() {
        let v = SchemaVersion::new(0, 1, 0);
        assert_eq!(v.to_string(), "0.1.0");
    }

    #[test]
    fn test_schema_version_ordering() {
        let v010 = SchemaVersion::new(0, 1, 0);
        let v020 = SchemaVersion::new(0, 2, 0);
        let v100 = SchemaVersion::new(1, 0, 0);
        assert!(v010 < v020);
        assert!(v020 < v100);
    }

    #[test]
    fn test_empty_migration_path() {
        let reg = MigrationRegistry::new();
        let v = SchemaVersion::new(0, 1, 0);
        let path = reg.find_path(&v, &v).expect("same version");
        assert!(path.is_empty());
    }

    #[test]
    fn test_no_path_found() {
        let reg = MigrationRegistry::new();
        let v1 = SchemaVersion::new(0, 1, 0);
        let v2 = SchemaVersion::new(0, 2, 0);
        assert!(reg.find_path(&v1, &v2).is_err());
    }

    #[test]
    fn test_single_step_migration_path() {
        let mut reg = MigrationRegistry::new();
        reg.register(Migration::new(
            SchemaVersion::new(0, 1, 0),
            SchemaVersion::new(0, 2, 0),
            "test",
        ));
        let path = reg
            .find_path(&SchemaVersion::new(0, 1, 0), &SchemaVersion::new(0, 2, 0))
            .expect("should find");
        assert_eq!(path.len(), 1);
        assert_eq!(path[0].from, SchemaVersion::new(0, 1, 0));
        assert_eq!(path[0].to, SchemaVersion::new(0, 2, 0));
    }

    #[test]
    fn test_multi_step_migration_path() {
        let reg = MigrationRegistry::with_defaults();
        let path = reg
            .find_path(&SchemaVersion::new(0, 1, 0), &SchemaVersion::new(1, 0, 0))
            .expect("should find full path");
        assert_eq!(path.len(), 3);
        assert_eq!(path[0].from, SchemaVersion::new(0, 1, 0));
        assert_eq!(path[0].to, SchemaVersion::new(0, 2, 0));
        assert_eq!(path[1].from, SchemaVersion::new(0, 2, 0));
        assert_eq!(path[1].to, SchemaVersion::new(0, 3, 0));
        assert_eq!(path[2].from, SchemaVersion::new(0, 3, 0));
        assert_eq!(path[2].to, SchemaVersion::new(1, 0, 0));
    }

    #[test]
    fn test_bfs_finds_shortest_path() {
        let mut reg = MigrationRegistry::new();
        // Direct path: A -> C
        reg.register(Migration::new(
            SchemaVersion::new(0, 1, 0),
            SchemaVersion::new(0, 3, 0),
            "direct",
        ));
        // Longer path: A -> B -> C
        reg.register(Migration::new(
            SchemaVersion::new(0, 1, 0),
            SchemaVersion::new(0, 2, 0),
            "step1",
        ));
        reg.register(Migration::new(
            SchemaVersion::new(0, 2, 0),
            SchemaVersion::new(0, 3, 0),
            "step2",
        ));
        let path = reg
            .find_path(&SchemaVersion::new(0, 1, 0), &SchemaVersion::new(0, 3, 0))
            .expect("should find");
        // BFS should find the direct single-step path
        assert_eq!(path.len(), 1);
        assert_eq!(path[0].description, "direct");
    }

    #[test]
    fn test_add_parameter_op() {
        let params: Vec<(String, f64)> = vec![("height".to_string(), 0.5)];
        let mut m = Migration::new(
            SchemaVersion::new(0, 1, 0),
            SchemaVersion::new(0, 2, 0),
            "add weight",
        );
        m.add_op(MigrationOp::AddParameter {
            name: "weight".to_string(),
            default: 0.7,
            min: 0.0,
            max: 1.0,
        });
        let result = m.apply(&params).expect("should apply");
        assert_eq!(result.len(), 2);
        assert_eq!(result[1], ("weight".to_string(), 0.7));
    }

    #[test]
    fn test_remove_parameter_op() {
        let params = vec![("height".to_string(), 0.5), ("weight".to_string(), 0.7)];
        let mut m = Migration::new(
            SchemaVersion::new(0, 1, 0),
            SchemaVersion::new(0, 2, 0),
            "remove weight",
        );
        m.add_op(MigrationOp::RemoveParameter {
            name: "weight".to_string(),
        });
        let result = m.apply(&params).expect("should apply");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "height");
    }

    #[test]
    fn test_rename_parameter_op() {
        let params = vec![("old_name".to_string(), 0.3)];
        let mut m = Migration::new(
            SchemaVersion::new(0, 1, 0),
            SchemaVersion::new(0, 2, 0),
            "rename",
        );
        m.add_op(MigrationOp::RenameParameter {
            old_name: "old_name".to_string(),
            new_name: "new_name".to_string(),
        });
        let result = m.apply(&params).expect("should apply");
        assert_eq!(result[0].0, "new_name");
        assert!((result[0].1 - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn test_split_parameter_op() {
        let params = vec![("body_weight".to_string(), 1.0)];
        let mut m = Migration::new(
            SchemaVersion::new(0, 1, 0),
            SchemaVersion::new(0, 2, 0),
            "split",
        );
        m.add_op(MigrationOp::SplitParameter {
            source: "body_weight".to_string(),
            targets: vec![
                ("body_fat".to_string(), 0.4),
                ("muscle_mass".to_string(), 0.6),
            ],
        });
        let result = m.apply(&params).expect("should apply");
        assert_eq!(result.len(), 2);
        assert!((result[0].1 - 0.4).abs() < f64::EPSILON);
        assert!((result[1].1 - 0.6).abs() < f64::EPSILON);
    }

    #[test]
    fn test_merge_parameters_op() {
        let params = vec![
            ("body_fat".to_string(), 0.4),
            ("muscle_mass".to_string(), 0.6),
        ];
        let mut m = Migration::new(
            SchemaVersion::new(0, 2, 0),
            SchemaVersion::new(0, 3, 0),
            "merge",
        );
        m.add_op(MigrationOp::MergeParameters {
            sources: vec![
                ("body_fat".to_string(), 0.5),
                ("muscle_mass".to_string(), 0.5),
            ],
            target: "body_weight".to_string(),
        });
        let result = m.apply(&params).expect("should apply");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "body_weight");
        // 0.4*0.5 + 0.6*0.5 = 0.5
        assert!((result[0].1 - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rescale_parameter_op() {
        let params = vec![("x".to_string(), 0.5)];
        let mut m = Migration::new(
            SchemaVersion::new(0, 1, 0),
            SchemaVersion::new(0, 2, 0),
            "rescale",
        );
        m.add_op(MigrationOp::RescaleParameter {
            name: "x".to_string(),
            old_range: (0.0, 1.0),
            new_range: (-1.0, 1.0),
        });
        let result = m.apply(&params).expect("should apply");
        // 0.5 in [0,1] -> 0.0 in [-1,1]
        assert!((result[0].1 - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_transform_parameter_op() {
        let params = vec![("x".to_string(), 2.0)];
        let mut m = Migration::new(
            SchemaVersion::new(0, 1, 0),
            SchemaVersion::new(0, 2, 0),
            "transform",
        );
        m.add_op(MigrationOp::TransformParameter {
            name: "x".to_string(),
            scale: 3.0,
            offset: -1.0,
        });
        let result = m.apply(&params).expect("should apply");
        // 2.0 * 3.0 + (-1.0) = 5.0
        assert!((result[0].1 - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_add_dependency_op() {
        let params = vec![("a".to_string(), 1.0), ("b".to_string(), 2.0)];
        let mut m = Migration::new(
            SchemaVersion::new(0, 1, 0),
            SchemaVersion::new(0, 2, 0),
            "dep",
        );
        m.add_op(MigrationOp::AddDependency {
            param: "a".to_string(),
            depends_on: "b".to_string(),
            factor: 0.5,
        });
        let result = m.apply(&params).expect("should apply");
        // a = 1.0 + 2.0*0.5 = 2.0
        assert!((result[0].1 - 2.0).abs() < f64::EPSILON);
        assert!((result[1].1 - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_full_migration_v010_to_v100() {
        let reg = MigrationRegistry::with_defaults();
        let params = VersionedParams::new(
            SchemaVersion::new(0, 1, 0),
            vec![
                ("body_weight".to_string(), 0.8),
                ("breast_size".to_string(), 0.6),
                ("height".to_string(), 0.5),
            ],
        );

        let result = reg
            .migrate(&params, &SchemaVersion::new(1, 0, 0))
            .expect("should migrate");

        assert_eq!(result.version, SchemaVersion::new(1, 0, 0));

        // body_weight split -> body_fat=0.32, muscle_mass=0.48
        // then rescaled from [0,1] to [-1,1]:
        //   body_fat: 0.32 -> 0.32*2 - 1 = -0.36
        //   muscle_mass: 0.48 -> 0.48*2 - 1 = -0.04
        let body_fat = result.get("body_fat").expect("body_fat should exist");
        assert!((body_fat - (-0.36)).abs() < 1e-10);

        let muscle_mass = result.get("muscle_mass").expect("muscle_mass should exist");
        assert!((muscle_mass - (-0.04)).abs() < 1e-10);

        // breast_size renamed to chest_volume=0.6, then rescaled:
        //   0.6 -> 0.6*2 - 1 = 0.2
        let chest_vol = result
            .get("chest_volume")
            .expect("chest_volume should exist");
        assert!((chest_vol - 0.2).abs() < 1e-10);

        // chest_shape added with default 0.5, then rescaled:
        //   0.5 -> 0.5*2 - 1 = 0.0
        let chest_shape = result.get("chest_shape").expect("chest_shape should exist");
        assert!((chest_shape - 0.0).abs() < 1e-10);

        // height is not rescaled by v0.3.0->v1.0.0 (only known params are rescaled)
        let height = result.get("height").expect("height should exist");
        assert!((height - 0.5).abs() < f64::EPSILON);

        // 10 new facial params should exist with default 0.0
        assert!(result.get("face_width").is_some());
        assert!(result.get("chin_projection").is_some());
    }

    #[test]
    fn test_validate_chain_defaults() {
        let reg = MigrationRegistry::with_defaults();
        reg.validate_chain().expect("default chain should be valid");
    }

    #[test]
    fn test_validate_chain_gap() {
        let mut reg = MigrationRegistry::new();
        reg.register(Migration::new(
            SchemaVersion::new(0, 1, 0),
            SchemaVersion::new(0, 2, 0),
            "step1",
        ));
        // Gap: missing 0.2.0 -> 0.3.0
        reg.register(Migration::new(
            SchemaVersion::new(0, 3, 0),
            SchemaVersion::new(1, 0, 0),
            "step3",
        ));
        // Validate should still pass because it only checks root->leaf reachability
        // and these are independent chains (two roots, two leaves)
        // But from 0.1.0 we can't reach 1.0.0
        assert!(reg.validate_chain().is_err());
    }

    #[test]
    fn test_current_version() {
        let reg = MigrationRegistry::with_defaults();
        let v = reg.current_version().expect("should have a version");
        assert_eq!(*v, SchemaVersion::new(1, 0, 0));
    }

    #[test]
    fn test_current_version_empty() {
        let reg = MigrationRegistry::new();
        assert!(reg.current_version().is_none());
    }

    #[test]
    fn test_versioned_params_helpers() {
        let vp = VersionedParams::new(
            SchemaVersion::new(0, 1, 0),
            vec![("a".to_string(), 1.0), ("b".to_string(), 2.0)],
        );
        assert_eq!(vp.len(), 2);
        assert!(!vp.is_empty());
        assert!((vp.get("a").expect("a exists") - 1.0).abs() < f64::EPSILON);
        assert!(vp.get("c").is_none());
    }

    #[test]
    fn test_add_parameter_idempotent() {
        // Adding a parameter that already exists should not duplicate it
        let params = vec![("x".to_string(), 0.5)];
        let mut m = Migration::new(
            SchemaVersion::new(0, 1, 0),
            SchemaVersion::new(0, 2, 0),
            "add existing",
        );
        m.add_op(MigrationOp::AddParameter {
            name: "x".to_string(),
            default: 0.0,
            min: 0.0,
            max: 1.0,
        });
        let result = m.apply(&params).expect("should apply");
        assert_eq!(result.len(), 1);
        // Original value preserved
        assert!((result[0].1 - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rescale_degenerate_range() {
        let params = vec![("x".to_string(), 5.0)];
        let mut m = Migration::new(
            SchemaVersion::new(0, 1, 0),
            SchemaVersion::new(0, 2, 0),
            "degenerate",
        );
        m.add_op(MigrationOp::RescaleParameter {
            name: "x".to_string(),
            old_range: (3.0, 3.0), // zero-width old range
            new_range: (0.0, 10.0),
        });
        let result = m.apply(&params).expect("should apply");
        // Degenerate maps to midpoint of new range
        assert!((result[0].1 - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_split_missing_source() {
        // Splitting a parameter that doesn't exist treats source value as 0.0
        let params: Vec<(String, f64)> = vec![];
        let mut m = Migration::new(
            SchemaVersion::new(0, 1, 0),
            SchemaVersion::new(0, 2, 0),
            "split missing",
        );
        m.add_op(MigrationOp::SplitParameter {
            source: "nonexistent".to_string(),
            targets: vec![("a".to_string(), 0.5), ("b".to_string(), 0.5)],
        });
        let result = m.apply(&params).expect("should apply");
        assert_eq!(result.len(), 2);
        assert!((result[0].1).abs() < f64::EPSILON);
        assert!((result[1].1).abs() < f64::EPSILON);
    }
}
