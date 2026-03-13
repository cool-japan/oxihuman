// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result};
use std::path::Path;

use crate::category::TargetCategory;
use crate::parser::target::parse_target;

/// A single entry in the target search index.
pub struct TargetEntry {
    pub name: String,
    pub category: TargetCategory,
    /// Optional filesystem path to the .target file.
    pub path: Option<String>,
    pub delta_count: usize,
    pub tags: Vec<String>,
}

/// Searchable in-memory index of morph targets.
#[derive(Default)]
pub struct TargetIndex {
    entries: Vec<TargetEntry>,
}

impl TargetIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, entry: TargetEntry) {
        self.entries.push(entry);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Return all entries that belong to the given category.
    pub fn by_category(&self, cat: &TargetCategory) -> Vec<&TargetEntry> {
        self.entries.iter().filter(|e| &e.category == cat).collect()
    }

    /// Case-insensitive substring search on name or tags.
    pub fn search(&self, query: &str) -> Vec<&TargetEntry> {
        let q = query.to_lowercase();
        self.entries
            .iter()
            .filter(|e| {
                e.name.to_lowercase().contains(&q)
                    || e.tags.iter().any(|t| t.to_lowercase().contains(&q))
            })
            .collect()
    }

    /// Look up an entry by exact name (case-sensitive).
    pub fn by_name(&self, name: &str) -> Option<&TargetEntry> {
        self.entries.iter().find(|e| e.name == name)
    }

    pub fn all(&self) -> &[TargetEntry] {
        &self.entries
    }

    /// Walk `dir`, add one `TargetEntry` per `.target` file found.
    ///
    /// The category is parsed from the name of the directory that directly
    /// contains the file.  Returns the number of entries added.
    pub fn scan_dir(&mut self, dir: &Path) -> Result<usize> {
        if !dir.exists() {
            anyhow::bail!("directory does not exist: {}", dir.display());
        }

        let mut added = 0usize;
        for entry in walkdir(dir)? {
            let path = entry?;
            if path.extension().and_then(|e| e.to_str()) != Some("target") {
                continue;
            }

            // Category comes from the parent directory name.
            let cat_name = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("other");
            let category = TargetCategory::from_str(cat_name);

            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            let src = std::fs::read_to_string(&path)
                .with_context(|| format!("reading {}", path.display()))?;
            let tf =
                parse_target(&stem, &src).with_context(|| format!("parsing {}", path.display()))?;

            self.entries.push(TargetEntry {
                name: stem,
                category,
                path: Some(path.to_string_lossy().into_owned()),
                delta_count: tf.deltas.len(),
                tags: Vec::new(),
            });
            added += 1;
        }
        Ok(added)
    }

    /// Build a `TargetIndex` by scanning a directory with a [`TargetScanner`].
    pub fn from_dir(dir: &Path) -> Result<Self> {
        let scanner = TargetScanner::new(dir)?;
        Ok(scanner.collect_all())
    }

    /// Return just the entry names (for use in `AssetManifest::allowed_targets`).
    pub fn to_manifest_targets(&self) -> Vec<String> {
        self.entries.iter().map(|e| e.name.clone()).collect()
    }
}

// ---------------------------------------------------------------------------
// TargetScanner — streaming directory walker
// ---------------------------------------------------------------------------

/// Streaming target scanner — walks a directory incrementally and yields entries.
///
/// Call [`TargetScanner::new`] to initialise the scanner (performs a shallow
/// directory enumeration to build the pending list), then drive it with
/// [`next_entry`][TargetScanner::next_entry] or drain everything at once with
/// [`collect_all`][TargetScanner::collect_all].
pub struct TargetScanner {
    /// Files yet to be parsed (`.target` extension only).
    pending: Vec<std::path::PathBuf>,
    /// Count of files processed so far.
    done: usize,
    /// Best-guess total derived from the initial directory walk.
    total_estimate: usize,
}

impl TargetScanner {
    /// Create a scanner from a directory path.
    ///
    /// Immediately performs a recursive walk of `dir` to collect all `.target`
    /// file paths into the pending list — no file parsing happens here.
    /// Returns an error if `dir` does not exist.
    pub fn new(dir: &Path) -> Result<Self> {
        if !dir.exists() {
            anyhow::bail!("directory does not exist: {}", dir.display());
        }

        // Collect all .target files via the internal recursive walker.
        let pending: Vec<std::path::PathBuf> = walkdir(dir)?
            .filter_map(|r| r.ok())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("target"))
            .collect();

        let total_estimate = pending.len();
        Ok(Self {
            pending,
            done: 0,
            total_estimate,
        })
    }

    /// Number of files processed so far.
    pub fn done(&self) -> usize {
        self.done
    }

    /// Estimated total number of `.target` files found during initialisation.
    pub fn total(&self) -> usize {
        self.total_estimate
    }

    /// Progress in `[0.0, 1.0]`.  Returns `1.0` when there are no files or
    /// when all files have been processed.
    pub fn progress(&self) -> f32 {
        if self.total_estimate == 0 {
            return 1.0;
        }
        self.done as f32 / self.total_estimate as f32
    }

    /// Returns `true` when all files have been processed.
    pub fn is_done(&self) -> bool {
        self.pending.is_empty()
    }

    /// Process the next pending file and return a [`TargetEntry`].
    ///
    /// Returns `None` when all files have been processed.  Files that fail
    /// to parse are silently skipped (the counter is still incremented).
    pub fn next_entry(&mut self) -> Option<TargetEntry> {
        loop {
            let path = self.pending.pop()?;
            self.done += 1;

            let cat_name = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("other");
            let category = TargetCategory::from_str(cat_name);

            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            let src = match std::fs::read_to_string(&path) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let tf = match parse_target(&stem, &src) {
                Ok(t) => t,
                Err(_) => continue,
            };

            return Some(TargetEntry {
                name: stem,
                category,
                path: Some(path.to_string_lossy().into_owned()),
                delta_count: tf.deltas.len(),
                tags: Vec::new(),
            });
        }
    }

    /// Drain all remaining entries into a [`TargetIndex`].
    pub fn collect_all(mut self) -> TargetIndex {
        let mut idx = TargetIndex::new();
        while let Some(entry) = self.next_entry() {
            idx.add(entry);
        }
        idx
    }
}

// ---------------------------------------------------------------------------
// Internal helper: a minimal recursive directory walker so we avoid pulling in
// the `walkdir` crate as a new dependency.
// ---------------------------------------------------------------------------

fn walkdir(dir: &Path) -> Result<impl Iterator<Item = Result<std::path::PathBuf>>> {
    let mut stack: Vec<std::path::PathBuf> = vec![dir.to_path_buf()];
    let mut files: Vec<std::path::PathBuf> = Vec::new();

    while let Some(current) = stack.pop() {
        for entry in std::fs::read_dir(&current)
            .with_context(|| format!("reading dir {}", current.display()))?
        {
            let entry = entry.with_context(|| format!("dir entry in {}", current.display()))?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                files.push(path);
            }
        }
    }

    Ok(files.into_iter().map(Ok))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    fn make_entry(name: &str, cat: TargetCategory, tags: Vec<&str>) -> TargetEntry {
        TargetEntry {
            name: name.to_string(),
            category: cat,
            path: None,
            delta_count: 0,
            tags: tags.into_iter().map(|s| s.to_string()).collect(),
        }
    }

    // ------------------------------------------------------------------
    // add / len / is_empty
    // ------------------------------------------------------------------

    #[test]
    fn new_index_is_empty() {
        let idx = TargetIndex::new();
        assert!(idx.is_empty());
        assert_eq!(idx.len(), 0);
    }

    #[test]
    fn add_increases_len() {
        let mut idx = TargetIndex::new();
        assert!(idx.is_empty());
        idx.add(make_entry("foo", TargetCategory::Height, vec![]));
        assert!(!idx.is_empty());
        assert_eq!(idx.len(), 1);
        idx.add(make_entry("bar", TargetCategory::Weight, vec![]));
        assert_eq!(idx.len(), 2);
    }

    // ------------------------------------------------------------------
    // by_category
    // ------------------------------------------------------------------

    #[test]
    fn by_category_returns_correct_subset() {
        let mut idx = TargetIndex::new();
        idx.add(make_entry("h1", TargetCategory::Height, vec![]));
        idx.add(make_entry("h2", TargetCategory::Height, vec![]));
        idx.add(make_entry("w1", TargetCategory::Weight, vec![]));

        let heights = idx.by_category(&TargetCategory::Height);
        assert_eq!(heights.len(), 2);
        assert!(heights.iter().all(|e| e.category == TargetCategory::Height));

        let weights = idx.by_category(&TargetCategory::Weight);
        assert_eq!(weights.len(), 1);
    }

    #[test]
    fn by_category_no_match_returns_empty() {
        let mut idx = TargetIndex::new();
        idx.add(make_entry("h1", TargetCategory::Height, vec![]));
        let muscles = idx.by_category(&TargetCategory::Muscle);
        assert!(muscles.is_empty());
    }

    // ------------------------------------------------------------------
    // search — case-insensitive, name & tags
    // ------------------------------------------------------------------

    #[test]
    fn search_is_case_insensitive() {
        let mut idx = TargetIndex::new();
        idx.add(make_entry("FaceSmile", TargetCategory::Expression, vec![]));

        assert_eq!(idx.search("facesmile").len(), 1);
        assert_eq!(idx.search("FACESMILE").len(), 1);
        assert_eq!(idx.search("FaceSmile").len(), 1);
    }

    #[test]
    fn search_matches_name_prefix() {
        let mut idx = TargetIndex::new();
        idx.add(make_entry("height-up", TargetCategory::Height, vec![]));
        idx.add(make_entry("height-down", TargetCategory::Height, vec![]));
        idx.add(make_entry("weight-high", TargetCategory::Weight, vec![]));

        let res = idx.search("height");
        assert_eq!(res.len(), 2);
    }

    #[test]
    fn search_matches_name_substring() {
        let mut idx = TargetIndex::new();
        idx.add(make_entry(
            "brow-inner-up",
            TargetCategory::Eyebrows,
            vec![],
        ));
        idx.add(make_entry(
            "brow-outer-up",
            TargetCategory::Eyebrows,
            vec![],
        ));
        idx.add(make_entry("chin-round", TargetCategory::Chin, vec![]));

        let res = idx.search("inner");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].name, "brow-inner-up");
    }

    #[test]
    fn search_matches_tags() {
        let mut idx = TargetIndex::new();
        idx.add(make_entry(
            "arm-long",
            TargetCategory::ArmsLegs,
            vec!["elongation", "limb"],
        ));
        idx.add(make_entry(
            "leg-long",
            TargetCategory::ArmsLegs,
            vec!["elongation", "limb"],
        ));
        idx.add(make_entry("chin-sharp", TargetCategory::Chin, vec!["face"]));

        let res = idx.search("elongation");
        assert_eq!(res.len(), 2);

        let res2 = idx.search("face");
        assert_eq!(res2.len(), 1);
    }

    #[test]
    fn search_empty_index_returns_empty() {
        let idx = TargetIndex::new();
        assert!(idx.search("anything").is_empty());
    }

    #[test]
    fn search_no_match_returns_empty() {
        let mut idx = TargetIndex::new();
        idx.add(make_entry("height-up", TargetCategory::Height, vec![]));
        assert!(idx.search("zzznomatch").is_empty());
    }

    // ------------------------------------------------------------------
    // by_name
    // ------------------------------------------------------------------

    #[test]
    fn by_name_found() {
        let mut idx = TargetIndex::new();
        idx.add(make_entry("chin-round", TargetCategory::Chin, vec![]));
        let e = idx.by_name("chin-round");
        assert!(e.is_some());
        assert_eq!(e.expect("should succeed").name, "chin-round");
    }

    #[test]
    fn by_name_not_found() {
        let idx = TargetIndex::new();
        assert!(idx.by_name("nonexistent").is_none());
    }

    // ------------------------------------------------------------------
    // to_manifest_targets
    // ------------------------------------------------------------------

    #[test]
    fn to_manifest_targets_returns_all_names() {
        let mut idx = TargetIndex::new();
        idx.add(make_entry("a", TargetCategory::Height, vec![]));
        idx.add(make_entry("b", TargetCategory::Weight, vec![]));
        idx.add(make_entry("c", TargetCategory::Muscle, vec![]));

        let names = idx.to_manifest_targets();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"a".to_string()));
        assert!(names.contains(&"b".to_string()));
        assert!(names.contains(&"c".to_string()));
    }

    // ------------------------------------------------------------------
    // scan_dir
    // ------------------------------------------------------------------

    /// Create a minimal .target file with `n` delta lines.
    fn write_target_file(path: &std::path::Path, n: usize) {
        let mut out = String::new();
        for i in 0..n {
            out.push_str(&format!("{} 0.1 0.2 0.3\n", i));
        }
        let mut f = fs::File::create(path).expect("failed to create target file");
        f.write_all(out.as_bytes())
            .expect("failed to write target file");
    }

    #[test]
    fn scan_dir_finds_three_target_files() {
        let tmp = tempdir();
        // Create sub-dirs that name the categories.
        let height_dir = tmp.join("height");
        let weight_dir = tmp.join("weight");
        fs::create_dir_all(&height_dir).expect("should succeed");
        fs::create_dir_all(&weight_dir).expect("should succeed");

        write_target_file(&height_dir.join("height-up.target"), 5);
        write_target_file(&height_dir.join("height-down.target"), 3);
        write_target_file(&weight_dir.join("weight-high.target"), 7);

        let mut idx = TargetIndex::new();
        let added = idx.scan_dir(&tmp).expect("should succeed");
        assert_eq!(added, 3);
        assert_eq!(idx.len(), 3);
    }

    #[test]
    fn scan_dir_parses_category_from_dir_name() {
        let tmp = tempdir();
        let age_dir = tmp.join("age");
        fs::create_dir_all(&age_dir).expect("should succeed");
        write_target_file(&age_dir.join("young.target"), 2);

        let mut idx = TargetIndex::new();
        idx.scan_dir(&tmp).expect("should succeed");

        let entry = idx.by_name("young").expect("should succeed");
        assert_eq!(entry.category, TargetCategory::Age);
    }

    #[test]
    fn scan_dir_counts_deltas_correctly() {
        let tmp = tempdir();
        let dir = tmp.join("height");
        fs::create_dir_all(&dir).expect("should succeed");
        write_target_file(&dir.join("test.target"), 8);

        let mut idx = TargetIndex::new();
        idx.scan_dir(&tmp).expect("should succeed");

        let entry = idx.by_name("test").expect("should succeed");
        assert_eq!(entry.delta_count, 8);
    }

    #[test]
    fn scan_dir_nonexistent_returns_error() {
        let mut idx = TargetIndex::new();
        let result = idx.scan_dir(std::path::Path::new("/tmp/this_does_not_exist_oxihuman"));
        assert!(result.is_err());
    }

    // ------------------------------------------------------------------
    // TargetScanner tests (8 new tests)
    // ------------------------------------------------------------------

    /// Set up a temp dir with 3 .target files in subdirectories.
    fn setup_scanner_dir() -> std::path::PathBuf {
        let tmp = tempdir();
        let height_dir = tmp.join("height");
        let weight_dir = tmp.join("weight");
        fs::create_dir_all(&height_dir).expect("failed to create height dir");
        fs::create_dir_all(&weight_dir).expect("failed to create weight dir");
        write_target_file(&height_dir.join("height-up.target"), 4);
        write_target_file(&height_dir.join("height-down.target"), 2);
        write_target_file(&weight_dir.join("weight-high.target"), 6);
        tmp
    }

    #[test]
    fn scanner_new_on_valid_dir_succeeds() {
        let tmp = setup_scanner_dir();
        let scanner = TargetScanner::new(&tmp);
        assert!(
            scanner.is_ok(),
            "TargetScanner::new should succeed on valid dir"
        );
    }

    #[test]
    fn scanner_total_returns_three() {
        let tmp = setup_scanner_dir();
        let scanner = TargetScanner::new(&tmp).expect("should succeed");
        assert_eq!(scanner.total(), 3, "total() should report 3 .target files");
    }

    #[test]
    fn scanner_progress_is_zero_initially() {
        let tmp = setup_scanner_dir();
        let scanner = TargetScanner::new(&tmp).expect("should succeed");
        assert!(
            (scanner.progress() - 0.0).abs() < f32::EPSILON,
            "progress() should be 0.0 before any processing"
        );
    }

    #[test]
    fn scanner_progress_is_one_after_collect_all() {
        let tmp = setup_scanner_dir();
        let scanner = TargetScanner::new(&tmp).expect("should succeed");
        let idx = scanner.collect_all();
        // The scanner is consumed; verify the index has 3 entries as a proxy.
        assert_eq!(idx.len(), 3);
    }

    #[test]
    fn scanner_next_entry_yields_some_then_none() {
        let tmp = setup_scanner_dir();
        let mut scanner = TargetScanner::new(&tmp).expect("should succeed");
        let mut count = 0usize;
        while scanner.next_entry().is_some() {
            count += 1;
        }
        assert_eq!(count, 3, "next_entry() should yield exactly 3 entries");
        assert!(
            scanner.next_entry().is_none(),
            "next_entry() should return None after all files processed"
        );
    }

    #[test]
    fn scanner_collect_all_returns_index_with_three_entries() {
        let tmp = setup_scanner_dir();
        let scanner = TargetScanner::new(&tmp).expect("should succeed");
        let idx = scanner.collect_all();
        assert_eq!(
            idx.len(),
            3,
            "collect_all() should produce index with 3 entries"
        );
    }

    #[test]
    fn scanner_is_done_after_collect_all_via_next_entry() {
        let tmp = setup_scanner_dir();
        let mut scanner = TargetScanner::new(&tmp).expect("should succeed");
        while scanner.next_entry().is_some() {}
        assert!(
            scanner.is_done(),
            "is_done() should be true after all entries consumed"
        );
    }

    #[test]
    fn target_index_from_dir_matches_scan_dir() {
        let tmp = setup_scanner_dir();
        // from_dir
        let idx_from_dir = TargetIndex::from_dir(&tmp).expect("should succeed");
        // scan_dir
        let mut idx_scan = TargetIndex::new();
        idx_scan.scan_dir(&tmp).expect("should succeed");

        assert_eq!(
            idx_from_dir.len(),
            idx_scan.len(),
            "from_dir and scan_dir should find the same number of entries"
        );
        // Verify every name found by scan_dir also appears in from_dir.
        for entry in idx_scan.all() {
            assert!(
                idx_from_dir.by_name(&entry.name).is_some(),
                "from_dir should contain entry '{}'",
                entry.name
            );
        }
    }

    #[test]
    fn scanner_new_on_nonexistent_dir_returns_err() {
        let result = TargetScanner::new(std::path::Path::new("/tmp/no_such_dir_oxihuman_scanner"));
        assert!(
            result.is_err(),
            "TargetScanner::new should error on missing dir"
        );
    }

    // ------------------------------------------------------------------
    // Helper: create a unique temp directory (no extra crate needed).
    // ------------------------------------------------------------------
    fn tempdir() -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::time::{SystemTime, UNIX_EPOCH};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let pid = std::process::id();
        let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::path::PathBuf::from(format!(
            "/tmp/oxihuman_target_index_test_{}_{}_{}",
            nanos, pid, seq
        ));
        fs::create_dir_all(&path).expect("failed to create temp dir");
        path
    }
}
