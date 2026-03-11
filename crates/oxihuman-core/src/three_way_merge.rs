// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Three-way text merge algorithm stub.
//!
//! Given a common ancestor (`base`) and two diverged versions (`ours` / `theirs`),
//! produces a merged result or reports conflicts where both sides changed the same region.

/// Outcome of merging a single region.
#[derive(Debug, Clone, PartialEq)]
pub enum MergeRegion {
    /// Both sides agree — use the common text.
    Common(Vec<String>),
    /// Only one side changed.
    Ours(Vec<String>),
    /// Only the other side changed.
    Theirs(Vec<String>),
    /// Both sides changed differently — a conflict.
    Conflict {
        ours: Vec<String>,
        theirs: Vec<String>,
    },
}

/// The result of a three-way merge.
#[derive(Debug, Clone)]
pub struct MergeResult {
    pub regions: Vec<MergeRegion>,
}

impl MergeResult {
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
        }
    }

    pub fn has_conflicts(&self) -> bool {
        self.regions
            .iter()
            .any(|r| matches!(r, MergeRegion::Conflict { .. }))
    }

    pub fn conflict_count(&self) -> usize {
        self.regions
            .iter()
            .filter(|r| matches!(r, MergeRegion::Conflict { .. }))
            .count()
    }

    pub fn region_count(&self) -> usize {
        self.regions.len()
    }

    /// Collect all merged lines (conflict regions represented by both sides).
    pub fn collect_lines(&self, prefer_ours: bool) -> Vec<String> {
        let mut out = Vec::new();
        for r in &self.regions {
            match r {
                MergeRegion::Common(ls) => out.extend(ls.iter().cloned()),
                MergeRegion::Ours(ls) => out.extend(ls.iter().cloned()),
                MergeRegion::Theirs(ls) => out.extend(ls.iter().cloned()),
                MergeRegion::Conflict { ours, theirs } => {
                    if prefer_ours {
                        out.extend(ours.iter().cloned());
                    } else {
                        out.extend(theirs.iter().cloned());
                    }
                }
            }
        }
        out
    }
}

impl Default for MergeResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Perform a three-way merge on line slices.
pub fn three_way_merge(base: &[&str], ours: &[&str], theirs: &[&str]) -> MergeResult {
    let mut result = MergeResult::new();
    let n = base.len().max(ours.len()).max(theirs.len());

    let mut bi = 0usize;
    let mut oi = 0usize;
    let mut ti = 0usize;

    while bi < base.len() || oi < ours.len() || ti < theirs.len() {
        let b = base.get(bi).copied();
        let o = ours.get(oi).copied();
        let t = theirs.get(ti).copied();

        match (b, o, t) {
            (Some(bv), Some(ov), Some(tv)) if bv == ov && bv == tv => {
                /* Unchanged on both sides */
                result
                    .regions
                    .push(MergeRegion::Common(vec![bv.to_string()]));
                bi += 1;
                oi += 1;
                ti += 1;
            }
            (Some(bv), Some(ov), Some(tv)) if bv != ov && bv == tv => {
                /* Only ours changed */
                result.regions.push(MergeRegion::Ours(vec![ov.to_string()]));
                bi += 1;
                oi += 1;
                ti += 1;
            }
            (Some(bv), Some(ov), Some(tv)) if bv == ov && bv != tv => {
                /* Only theirs changed */
                result
                    .regions
                    .push(MergeRegion::Theirs(vec![tv.to_string()]));
                bi += 1;
                oi += 1;
                ti += 1;
            }
            (_, Some(ov), Some(tv)) => {
                /* Conflict or both changed */
                result.regions.push(MergeRegion::Conflict {
                    ours: vec![ov.to_string()],
                    theirs: vec![tv.to_string()],
                });
                if bi < base.len() {
                    bi += 1;
                }
                oi += 1;
                ti += 1;
            }
            (_, Some(ov), None) => {
                result.regions.push(MergeRegion::Ours(vec![ov.to_string()]));
                if bi < base.len() {
                    bi += 1;
                }
                oi += 1;
            }
            (_, None, Some(tv)) => {
                result
                    .regions
                    .push(MergeRegion::Theirs(vec![tv.to_string()]));
                if bi < base.len() {
                    bi += 1;
                }
                ti += 1;
            }
            _ => break,
        }
    }
    let _ = n;
    result
}

/// Check whether all regions in a merge result are conflict-free.
pub fn is_clean_merge(result: &MergeResult) -> bool {
    !result.has_conflicts()
}

/// Count non-conflicting merged lines.
pub fn clean_line_count(result: &MergeResult) -> usize {
    result
        .regions
        .iter()
        .map(|r| match r {
            MergeRegion::Common(ls) | MergeRegion::Ours(ls) | MergeRegion::Theirs(ls) => ls.len(),
            MergeRegion::Conflict { ours, theirs } => ours.len() + theirs.len(),
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_change_is_clean() {
        let lines = ["a", "b", "c"];
        let r = three_way_merge(&lines, &lines, &lines);
        assert!(r.is_clean_merge_check());
    }

    /* Helper extension for tests */
    impl MergeResult {
        fn is_clean_merge_check(&self) -> bool {
            is_clean_merge(self)
        }
    }

    #[test]
    fn test_ours_change() {
        let base = ["a"];
        let ours = ["b"];
        let theirs = ["a"];
        let r = three_way_merge(&base, &ours, &theirs);
        assert!(!r.has_conflicts());
        assert!(r.regions.iter().any(|r| matches!(r, MergeRegion::Ours(_))));
    }

    #[test]
    fn test_theirs_change() {
        let base = ["a"];
        let ours = ["a"];
        let theirs = ["c"];
        let r = three_way_merge(&base, &ours, &theirs);
        assert!(!r.has_conflicts());
    }

    #[test]
    fn test_conflict_detected() {
        let base = ["a"];
        let ours = ["b"];
        let theirs = ["c"];
        let r = three_way_merge(&base, &ours, &theirs);
        assert!(r.has_conflicts());
        assert_eq!(r.conflict_count(), 1);
    }

    #[test]
    fn test_collect_lines_prefer_ours() {
        let base = ["a"];
        let ours = ["b"];
        let theirs = ["c"];
        let r = three_way_merge(&base, &ours, &theirs);
        let lines = r.collect_lines(true);
        assert_eq!(lines, vec!["b"]);
    }

    #[test]
    fn test_collect_lines_prefer_theirs() {
        let base = ["a"];
        let ours = ["b"];
        let theirs = ["c"];
        let r = three_way_merge(&base, &ours, &theirs);
        let lines = r.collect_lines(false);
        assert_eq!(lines, vec!["c"]);
    }

    #[test]
    fn test_region_count() {
        let base = ["a", "b"];
        let ours = ["a", "x"];
        let theirs = ["a", "y"];
        let r = three_way_merge(&base, &ours, &theirs);
        assert!(r.region_count() > 0);
    }

    #[test]
    fn test_clean_line_count_nonzero() {
        let base = ["a"];
        let r = three_way_merge(&base, &base, &base);
        assert!(clean_line_count(&r) > 0);
    }

    #[test]
    fn test_default() {
        let r = MergeResult::default();
        assert_eq!(r.region_count(), 0);
    }
}
