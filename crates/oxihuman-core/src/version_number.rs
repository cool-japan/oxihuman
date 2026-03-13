// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Semantic version with optional pre-release label.
#[allow(dead_code)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre: String,
}

/// Parse a semantic version string like "1.2.3" or "1.2.3-alpha".
#[allow(dead_code)]
pub fn parse_version(s: &str) -> Option<Version> {
    let (main, pre) = if let Some(idx) = s.find('-') {
        (&s[..idx], s[idx + 1..].to_string())
    } else {
        (s, String::new())
    };
    let parts: Vec<&str> = main.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let major = parts[0].parse::<u32>().ok()?;
    let minor = parts[1].parse::<u32>().ok()?;
    let patch = parts[2].parse::<u32>().ok()?;
    Some(Version { major, minor, patch, pre })
}

/// Convert a `Version` back to its string representation.
#[allow(dead_code)]
pub fn version_to_string(v: &Version) -> String {
    if v.pre.is_empty() {
        format!("{}.{}.{}", v.major, v.minor, v.patch)
    } else {
        format!("{}.{}.{}-{}", v.major, v.minor, v.patch, v.pre)
    }
}

/// Compare two versions (major, then minor, then patch).
#[allow(dead_code)]
pub fn version_cmp(a: &Version, b: &Version) -> std::cmp::Ordering {
    a.major
        .cmp(&b.major)
        .then(a.minor.cmp(&b.minor))
        .then(a.patch.cmp(&b.patch))
}

/// Returns true if `base` satisfies `req`: same major, req.minor <= base.minor.
#[allow(dead_code)]
pub fn is_compatible(base: &Version, req: &Version) -> bool {
    base.major == req.major && req.minor <= base.minor
}

/// Return the zero version "0.0.0".
#[allow(dead_code)]
pub fn version_zero() -> Version {
    Version { major: 0, minor: 0, patch: 0, pre: String::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn parse_simple() {
        let v = parse_version("1.2.3").expect("should succeed");
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert!(v.pre.is_empty());
    }

    #[test]
    fn parse_with_pre() {
        let v = parse_version("2.0.0-beta").expect("should succeed");
        assert_eq!(v.major, 2);
        assert_eq!(v.pre, "beta");
    }

    #[test]
    fn parse_invalid_returns_none() {
        assert!(parse_version("1.2").is_none());
        assert!(parse_version("abc").is_none());
    }

    #[test]
    fn version_to_string_simple() {
        let v = Version { major: 3, minor: 1, patch: 4, pre: String::new() };
        assert_eq!(version_to_string(&v), "3.1.4");
    }

    #[test]
    fn version_to_string_with_pre() {
        let v = Version { major: 1, minor: 0, patch: 0, pre: "rc1".to_string() };
        assert_eq!(version_to_string(&v), "1.0.0-rc1");
    }

    #[test]
    fn version_cmp_major() {
        let a = parse_version("2.0.0").expect("should succeed");
        let b = parse_version("1.0.0").expect("should succeed");
        assert_eq!(version_cmp(&a, &b), Ordering::Greater);
    }

    #[test]
    fn version_cmp_equal() {
        let a = parse_version("1.2.3").expect("should succeed");
        let b = parse_version("1.2.3").expect("should succeed");
        assert_eq!(version_cmp(&a, &b), Ordering::Equal);
    }

    #[test]
    fn is_compatible_same_major() {
        let base = parse_version("1.5.0").expect("should succeed");
        let req = parse_version("1.3.0").expect("should succeed");
        assert!(is_compatible(&base, &req));
    }

    #[test]
    fn is_compatible_different_major() {
        let base = parse_version("2.0.0").expect("should succeed");
        let req = parse_version("1.0.0").expect("should succeed");
        assert!(!is_compatible(&base, &req));
    }

    #[test]
    fn version_zero_is_all_zeros() {
        let v = version_zero();
        assert_eq!(v.major, 0);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
    }
}
