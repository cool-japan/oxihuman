// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Changelog entry serialization (Keep-a-Changelog format).

/// A changelog entry in Keep-a-Changelog style.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChangelogEntry {
    pub version: String,
    pub date: String,
    pub added: Vec<String>,
    pub changed: Vec<String>,
    pub deprecated: Vec<String>,
    pub removed: Vec<String>,
    pub fixed: Vec<String>,
    pub security: Vec<String>,
}

impl ChangelogEntry {
    #[allow(dead_code)]
    pub fn new(version: &str, date: &str) -> Self {
        Self {
            version: version.to_string(),
            date: date.to_string(),
            added: Vec::new(),
            changed: Vec::new(),
            deprecated: Vec::new(),
            removed: Vec::new(),
            fixed: Vec::new(),
            security: Vec::new(),
        }
    }
}

/// A full changelog.
#[allow(dead_code)]
pub struct Changelog {
    pub title: String,
    pub entries: Vec<ChangelogEntry>,
}

impl Changelog {
    #[allow(dead_code)]
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            entries: Vec::new(),
        }
    }
}

/// Add an entry.
#[allow(dead_code)]
pub fn add_changelog_entry(log: &mut Changelog, entry: ChangelogEntry) {
    log.entries.push(entry);
}

/// Serialize to Keep-a-Changelog Markdown format.
#[allow(dead_code)]
pub fn export_changelog_md(log: &Changelog) -> String {
    let mut out = format!("# {}\n\n", log.title);
    out.push_str("All notable changes to this project will be documented in this file.\n\n");
    out.push_str(
        "The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).\n\n",
    );
    for entry in &log.entries {
        out.push_str(&format!("## [{}] - {}\n\n", entry.version, entry.date));
        fn section(out: &mut String, title: &str, items: &[String]) {
            if !items.is_empty() {
                out.push_str(&format!("### {title}\n"));
                for item in items {
                    out.push_str(&format!("- {item}\n"));
                }
                out.push('\n');
            }
        }
        section(&mut out, "Added", &entry.added);
        section(&mut out, "Changed", &entry.changed);
        section(&mut out, "Deprecated", &entry.deprecated);
        section(&mut out, "Removed", &entry.removed);
        section(&mut out, "Fixed", &entry.fixed);
        section(&mut out, "Security", &entry.security);
    }
    out
}

/// Number of entries.
#[allow(dead_code)]
pub fn entry_count(log: &Changelog) -> usize {
    log.entries.len()
}

/// Total changes across all entries.
#[allow(dead_code)]
pub fn total_changes(log: &Changelog) -> usize {
    log.entries
        .iter()
        .map(|e| {
            e.added.len()
                + e.changed.len()
                + e.deprecated.len()
                + e.removed.len()
                + e.fixed.len()
                + e.security.len()
        })
        .sum()
}

/// Find entry by version.
#[allow(dead_code)]
pub fn find_entry_by_version<'a>(log: &'a Changelog, version: &str) -> Option<&'a ChangelogEntry> {
    log.entries.iter().find(|e| e.version == version)
}

/// Latest version string.
#[allow(dead_code)]
pub fn latest_version(log: &Changelog) -> Option<&str> {
    log.entries.first().map(|e| e.version.as_str())
}

/// Add a fix note to an entry.
#[allow(dead_code)]
pub fn add_fix(entry: &mut ChangelogEntry, fix: &str) {
    entry.fixed.push(fix.to_string());
}

/// Add an addition to an entry.
#[allow(dead_code)]
pub fn add_addition(entry: &mut ChangelogEntry, item: &str) {
    entry.added.push(item.to_string());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_log() -> Changelog {
        let mut log = Changelog::new("My Project Changelog");
        let mut e1 = ChangelogEntry::new("1.1.0", "2026-01-15");
        add_addition(&mut e1, "New mesh export format");
        add_fix(&mut e1, "Fixed UV seam artifact");
        add_changelog_entry(&mut log, e1);
        let e2 = ChangelogEntry::new("1.0.0", "2026-01-01");
        add_changelog_entry(&mut log, e2);
        log
    }

    #[test]
    fn entry_count_two() {
        let log = sample_log();
        assert_eq!(entry_count(&log), 2);
    }

    #[test]
    fn total_changes_correct() {
        let log = sample_log();
        assert_eq!(total_changes(&log), 2);
    }

    #[test]
    fn md_contains_version() {
        let log = sample_log();
        let md = export_changelog_md(&log);
        assert!(md.contains("1.1.0"));
    }

    #[test]
    fn md_contains_added_section() {
        let log = sample_log();
        let md = export_changelog_md(&log);
        assert!(md.contains("### Added"));
    }

    #[test]
    fn md_contains_fixed_section() {
        let log = sample_log();
        let md = export_changelog_md(&log);
        assert!(md.contains("### Fixed"));
    }

    #[test]
    fn find_entry_by_version_some() {
        let log = sample_log();
        let e = find_entry_by_version(&log, "1.0.0");
        assert!(e.is_some());
    }

    #[test]
    fn find_entry_by_version_none() {
        let log = sample_log();
        assert!(find_entry_by_version(&log, "0.0.0").is_none());
    }

    #[test]
    fn latest_version_is_first() {
        let log = sample_log();
        assert_eq!(latest_version(&log), Some("1.1.0"));
    }

    #[test]
    fn md_starts_with_title() {
        let log = sample_log();
        let md = export_changelog_md(&log);
        assert!(md.starts_with("# My Project Changelog"));
    }

    #[test]
    fn empty_log_no_entries() {
        let log = Changelog::new("Empty");
        assert_eq!(entry_count(&log), 0);
        assert!(latest_version(&log).is_none());
    }
}
