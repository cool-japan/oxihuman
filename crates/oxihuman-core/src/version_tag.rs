// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Semantic version tag.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VersionTag {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum VersionCompare {
    Equal,
    Less,
    Greater,
}

#[allow(dead_code)]
pub fn new_version_tag(major: u32, minor: u32, patch: u32) -> VersionTag {
    VersionTag { major, minor, patch }
}

#[allow(dead_code)]
pub fn version_to_string(v: &VersionTag) -> String {
    format!("{}.{}.{}", v.major, v.minor, v.patch)
}

#[allow(dead_code)]
pub fn version_compare(a: &VersionTag, b: &VersionTag) -> VersionCompare {
    if a.major != b.major {
        if a.major < b.major { VersionCompare::Less } else { VersionCompare::Greater }
    } else if a.minor != b.minor {
        if a.minor < b.minor { VersionCompare::Less } else { VersionCompare::Greater }
    } else if a.patch != b.patch {
        if a.patch < b.patch { VersionCompare::Less } else { VersionCompare::Greater }
    } else {
        VersionCompare::Equal
    }
}

#[allow(dead_code)]
pub fn version_is_compatible(a: &VersionTag, b: &VersionTag) -> bool {
    a.major == b.major
}

#[allow(dead_code)]
pub fn version_bump_major(v: &VersionTag) -> VersionTag {
    VersionTag { major: v.major + 1, minor: 0, patch: 0 }
}

#[allow(dead_code)]
pub fn version_bump_minor(v: &VersionTag) -> VersionTag {
    VersionTag { major: v.major, minor: v.minor + 1, patch: 0 }
}

#[allow(dead_code)]
pub fn version_bump_patch(v: &VersionTag) -> VersionTag {
    VersionTag { major: v.major, minor: v.minor, patch: v.patch + 1 }
}

#[allow(dead_code)]
pub fn version_from_str(s: &str) -> Option<VersionTag> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let major = parts[0].parse::<u32>().ok()?;
    let minor = parts[1].parse::<u32>().ok()?;
    let patch = parts[2].parse::<u32>().ok()?;
    Some(VersionTag { major, minor, patch })
}

#[allow(dead_code)]
pub fn version_is_prerelease(v: &VersionTag) -> bool {
    v.major == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_version() {
        let v = new_version_tag(1, 2, 3);
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_to_string() {
        let v = new_version_tag(1, 2, 3);
        assert_eq!(version_to_string(&v), "1.2.3");
    }

    #[test]
    fn test_compare_equal() {
        let a = new_version_tag(1, 0, 0);
        let b = new_version_tag(1, 0, 0);
        assert_eq!(version_compare(&a, &b), VersionCompare::Equal);
    }

    #[test]
    fn test_compare_less() {
        let a = new_version_tag(1, 0, 0);
        let b = new_version_tag(2, 0, 0);
        assert_eq!(version_compare(&a, &b), VersionCompare::Less);
    }

    #[test]
    fn test_compare_greater() {
        let a = new_version_tag(2, 0, 0);
        let b = new_version_tag(1, 0, 0);
        assert_eq!(version_compare(&a, &b), VersionCompare::Greater);
    }

    #[test]
    fn test_is_compatible() {
        let a = new_version_tag(2, 1, 0);
        let b = new_version_tag(2, 5, 3);
        assert!(version_is_compatible(&a, &b));
        let c = new_version_tag(3, 0, 0);
        assert!(!version_is_compatible(&a, &c));
    }

    #[test]
    fn test_bump_major() {
        let v = new_version_tag(1, 5, 3);
        let bumped = version_bump_major(&v);
        assert_eq!(bumped.major, 2);
        assert_eq!(bumped.minor, 0);
        assert_eq!(bumped.patch, 0);
    }

    #[test]
    fn test_from_str() {
        let v = version_from_str("3.1.4").expect("should succeed");
        assert_eq!(v.major, 3);
        assert_eq!(v.minor, 1);
        assert_eq!(v.patch, 4);
        assert!(version_from_str("bad").is_none());
    }

    #[test]
    fn test_is_prerelease() {
        let v = new_version_tag(0, 9, 0);
        assert!(version_is_prerelease(&v));
        let v2 = new_version_tag(1, 0, 0);
        assert!(!version_is_prerelease(&v2));
    }
}
