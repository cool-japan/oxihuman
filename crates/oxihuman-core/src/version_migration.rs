//! Version migration utilities for assets and schemas.

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct MigrationStep {
    pub from_version: SemVer,
    pub to_version: SemVer,
    pub description: String,
    pub breaking: bool,
}

#[allow(dead_code)]
pub struct MigrationPlan {
    pub steps: Vec<MigrationStep>,
    pub source: SemVer,
    pub target: SemVer,
}

#[allow(dead_code)]
pub struct MigrationRegistry {
    pub steps: Vec<MigrationStep>,
}

#[allow(dead_code)]
pub fn new_semver(major: u32, minor: u32, patch: u32) -> SemVer {
    SemVer {
        major,
        minor,
        patch,
    }
}

#[allow(dead_code)]
pub fn semver_parse(s: &str) -> Option<SemVer> {
    let parts: Vec<&str> = s.splitn(3, '.').collect();
    if parts.len() != 3 {
        return None;
    }
    let major = parts[0].parse::<u32>().ok()?;
    let minor = parts[1].parse::<u32>().ok()?;
    let patch = parts[2].parse::<u32>().ok()?;
    Some(SemVer {
        major,
        minor,
        patch,
    })
}

#[allow(dead_code)]
pub fn semver_compare(a: &SemVer, b: &SemVer) -> std::cmp::Ordering {
    a.major
        .cmp(&b.major)
        .then(a.minor.cmp(&b.minor))
        .then(a.patch.cmp(&b.patch))
}

#[allow(dead_code)]
pub fn semver_to_string(v: &SemVer) -> String {
    format!("{}.{}.{}", v.major, v.minor, v.patch)
}

#[allow(dead_code)]
pub fn is_breaking_change(from: &SemVer, to: &SemVer) -> bool {
    to.major > from.major
}

#[allow(dead_code)]
pub fn new_migration_registry() -> MigrationRegistry {
    MigrationRegistry { steps: Vec::new() }
}

#[allow(dead_code)]
pub fn register_migration(registry: &mut MigrationRegistry, step: MigrationStep) {
    registry.steps.push(step);
}

/// Find a chain of steps from `from` to `to` using BFS.
#[allow(dead_code)]
pub fn plan_migration(
    registry: &MigrationRegistry,
    from: &SemVer,
    to: &SemVer,
) -> Option<MigrationPlan> {
    if from == to {
        return Some(MigrationPlan {
            steps: Vec::new(),
            source: from.clone(),
            target: to.clone(),
        });
    }

    // BFS over version graph
    use std::collections::{HashMap, VecDeque};

    let mut queue: VecDeque<SemVer> = VecDeque::new();
    // predecessor: current -> (predecessor, step_index)
    let mut prev: HashMap<String, (SemVer, usize)> = HashMap::new();

    queue.push_back(from.clone());

    while let Some(current) = queue.pop_front() {
        for (idx, step) in registry.steps.iter().enumerate() {
            if step.from_version == current {
                let next = step.to_version.clone();
                let next_key = semver_to_string(&next);
                if !prev.contains_key(&next_key)
                    && semver_to_string(&next) != semver_to_string(from)
                {
                    prev.insert(next_key.clone(), (current.clone(), idx));
                    if &next == to {
                        // Reconstruct path
                        let mut path_steps: Vec<MigrationStep> = Vec::new();
                        let mut cur = next;
                        loop {
                            let cur_key = semver_to_string(&cur);
                            if let Some((pred, sidx)) = prev.get(&cur_key) {
                                path_steps.push(registry.steps[*sidx].clone());
                                if pred == from {
                                    break;
                                }
                                cur = pred.clone();
                            } else {
                                break;
                            }
                        }
                        path_steps.reverse();
                        return Some(MigrationPlan {
                            steps: path_steps,
                            source: from.clone(),
                            target: to.clone(),
                        });
                    }
                    queue.push_back(next);
                }
            }
        }
    }
    None
}

#[allow(dead_code)]
pub fn has_migration_path(registry: &MigrationRegistry, from: &SemVer, to: &SemVer) -> bool {
    plan_migration(registry, from, to).is_some()
}

#[allow(dead_code)]
pub fn migration_step_count(plan: &MigrationPlan) -> usize {
    plan.steps.len()
}

#[allow(dead_code)]
pub fn plan_has_breaking(plan: &MigrationPlan) -> bool {
    plan.steps.iter().any(|s| s.breaking)
}

#[allow(dead_code)]
pub fn migration_description(plan: &MigrationPlan) -> Vec<&str> {
    plan.steps.iter().map(|s| s.description.as_str()).collect()
}

#[allow(dead_code)]
pub fn latest_version(registry: &MigrationRegistry) -> Option<SemVer> {
    registry
        .steps
        .iter()
        .map(|s| &s.to_version)
        .max_by(|a, b| semver_compare(a, b))
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(major: u32, minor: u32, patch: u32) -> SemVer {
        new_semver(major, minor, patch)
    }

    fn make_step(from: SemVer, to: SemVer, desc: &str, breaking: bool) -> MigrationStep {
        MigrationStep {
            from_version: from,
            to_version: to,
            description: desc.to_string(),
            breaking,
        }
    }

    #[test]
    fn test_semver_parse_valid() {
        let sv = semver_parse("1.2.3").unwrap();
        assert_eq!(sv.major, 1);
        assert_eq!(sv.minor, 2);
        assert_eq!(sv.patch, 3);
    }

    #[test]
    fn test_semver_parse_invalid() {
        assert!(semver_parse("1.2").is_none());
        assert!(semver_parse("abc").is_none());
        assert!(semver_parse("1.x.3").is_none());
    }

    #[test]
    fn test_semver_compare_less() {
        assert_eq!(
            semver_compare(&v(1, 0, 0), &v(2, 0, 0)),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            semver_compare(&v(1, 2, 3), &v(1, 2, 4)),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            semver_compare(&v(1, 0, 0), &v(1, 1, 0)),
            std::cmp::Ordering::Less
        );
    }

    #[test]
    fn test_semver_compare_equal() {
        assert_eq!(
            semver_compare(&v(1, 2, 3), &v(1, 2, 3)),
            std::cmp::Ordering::Equal
        );
    }

    #[test]
    fn test_semver_compare_greater() {
        assert_eq!(
            semver_compare(&v(2, 0, 0), &v(1, 9, 9)),
            std::cmp::Ordering::Greater
        );
    }

    #[test]
    fn test_semver_to_string() {
        assert_eq!(semver_to_string(&v(3, 14, 0)), "3.14.0");
    }

    #[test]
    fn test_is_breaking_change_true() {
        assert!(is_breaking_change(&v(1, 5, 0), &v(2, 0, 0)));
    }

    #[test]
    fn test_is_breaking_change_false_minor_bump() {
        assert!(!is_breaking_change(&v(1, 0, 0), &v(1, 1, 0)));
    }

    #[test]
    fn test_is_breaking_change_false_patch_bump() {
        assert!(!is_breaking_change(&v(1, 0, 0), &v(1, 0, 1)));
    }

    #[test]
    fn test_register_migration() {
        let mut registry = new_migration_registry();
        register_migration(
            &mut registry,
            make_step(v(1, 0, 0), v(1, 1, 0), "add field", false),
        );
        assert_eq!(registry.steps.len(), 1);
    }

    #[test]
    fn test_plan_migration_single_step() {
        let mut registry = new_migration_registry();
        register_migration(
            &mut registry,
            make_step(v(1, 0, 0), v(1, 1, 0), "bump minor", false),
        );
        let plan = plan_migration(&registry, &v(1, 0, 0), &v(1, 1, 0)).unwrap();
        assert_eq!(migration_step_count(&plan), 1);
    }

    #[test]
    fn test_plan_migration_multi_step() {
        let mut registry = new_migration_registry();
        register_migration(
            &mut registry,
            make_step(v(1, 0, 0), v(1, 1, 0), "step1", false),
        );
        register_migration(
            &mut registry,
            make_step(v(1, 1, 0), v(2, 0, 0), "step2", true),
        );
        let plan = plan_migration(&registry, &v(1, 0, 0), &v(2, 0, 0)).unwrap();
        assert_eq!(migration_step_count(&plan), 2);
    }

    #[test]
    fn test_plan_migration_no_path() {
        let mut registry = new_migration_registry();
        register_migration(
            &mut registry,
            make_step(v(1, 0, 0), v(1, 1, 0), "step1", false),
        );
        let result = plan_migration(&registry, &v(1, 0, 0), &v(3, 0, 0));
        assert!(result.is_none());
    }

    #[test]
    fn test_has_migration_path_true() {
        let mut registry = new_migration_registry();
        register_migration(&mut registry, make_step(v(1, 0, 0), v(1, 1, 0), "s", false));
        assert!(has_migration_path(&registry, &v(1, 0, 0), &v(1, 1, 0)));
    }

    #[test]
    fn test_has_migration_path_false() {
        let registry = new_migration_registry();
        assert!(!has_migration_path(&registry, &v(1, 0, 0), &v(2, 0, 0)));
    }

    #[test]
    fn test_migration_step_count() {
        let mut registry = new_migration_registry();
        register_migration(
            &mut registry,
            make_step(v(1, 0, 0), v(1, 1, 0), "s1", false),
        );
        register_migration(
            &mut registry,
            make_step(v(1, 1, 0), v(1, 2, 0), "s2", false),
        );
        let plan = plan_migration(&registry, &v(1, 0, 0), &v(1, 2, 0)).unwrap();
        assert_eq!(migration_step_count(&plan), 2);
    }

    #[test]
    fn test_plan_has_breaking_true() {
        let mut registry = new_migration_registry();
        register_migration(
            &mut registry,
            make_step(v(1, 0, 0), v(2, 0, 0), "major", true),
        );
        let plan = plan_migration(&registry, &v(1, 0, 0), &v(2, 0, 0)).unwrap();
        assert!(plan_has_breaking(&plan));
    }

    #[test]
    fn test_plan_has_breaking_false() {
        let mut registry = new_migration_registry();
        register_migration(
            &mut registry,
            make_step(v(1, 0, 0), v(1, 1, 0), "minor", false),
        );
        let plan = plan_migration(&registry, &v(1, 0, 0), &v(1, 1, 0)).unwrap();
        assert!(!plan_has_breaking(&plan));
    }

    #[test]
    fn test_latest_version() {
        let mut registry = new_migration_registry();
        register_migration(
            &mut registry,
            make_step(v(1, 0, 0), v(1, 1, 0), "s1", false),
        );
        register_migration(&mut registry, make_step(v(1, 1, 0), v(2, 0, 0), "s2", true));
        let latest = latest_version(&registry).unwrap();
        assert_eq!(latest, v(2, 0, 0));
    }

    #[test]
    fn test_latest_version_none_on_empty() {
        let registry = new_migration_registry();
        assert!(latest_version(&registry).is_none());
    }

    #[test]
    fn test_migration_description() {
        let mut registry = new_migration_registry();
        register_migration(
            &mut registry,
            make_step(v(1, 0, 0), v(1, 1, 0), "add feature", false),
        );
        let plan = plan_migration(&registry, &v(1, 0, 0), &v(1, 1, 0)).unwrap();
        let desc = migration_description(&plan);
        assert_eq!(desc, vec!["add feature"]);
    }

    #[test]
    fn test_new_semver() {
        let sv = new_semver(2, 3, 4);
        assert_eq!(sv.major, 2);
        assert_eq!(sv.minor, 3);
        assert_eq!(sv.patch, 4);
    }
}
