// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! ICC color profile stub export — generates basic ICC profile byte stubs.

/// ICC profile class identifiers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IccProfileClass {
    Display,
    Output,
    Input,
    ColorSpace,
}

/// ICC color space identifier.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IccColorSpace {
    Rgb,
    Cmyk,
    Lab,
    Xyz,
}

/// A stub ICC profile descriptor.
#[derive(Debug, Clone)]
pub struct IccProfile {
    pub description: String,
    pub class: IccProfileClass,
    pub color_space: IccColorSpace,
    pub version: (u8, u8, u8),
    pub white_point: [f32; 3],
}

impl Default for IccProfile {
    fn default() -> Self {
        Self {
            description: "sRGB IEC61966-2.1".to_string(),
            class: IccProfileClass::Display,
            color_space: IccColorSpace::Rgb,
            version: (4, 3, 0),
            white_point: [0.9504559, 1.0, 1.0890578],
        }
    }
}

/// Exports an ICC profile as a byte stub (header only).
pub fn export_icc_profile(profile: &IccProfile) -> Vec<u8> {
    let mut blob = Vec::new();
    blob.extend_from_slice(b"ICCP");
    blob.push(profile.version.0);
    blob.push(profile.version.1);
    blob.push(profile.version.2);
    blob.extend_from_slice(profile.description.as_bytes());
    blob
}

/// Checks if an ICC profile describes an RGB color space.
pub fn is_rgb_profile(profile: &IccProfile) -> bool {
    profile.color_space == IccColorSpace::Rgb
}

/// Returns the D50-adapted XYZ white point for a profile.
pub fn profile_white_point(profile: &IccProfile) -> [f32; 3] {
    profile.white_point
}

/// Validates basic ICC profile fields.
pub fn validate_icc_profile(profile: &IccProfile) -> bool {
    !profile.description.is_empty() && profile.white_point[1] > 0.0
}

/// Returns the ICC class name as a 4-byte tag string.
pub fn class_tag(class: IccProfileClass) -> &'static str {
    match class {
        IccProfileClass::Display => "mntr",
        IccProfileClass::Output => "prtr",
        IccProfileClass::Input => "scnr",
        IccProfileClass::ColorSpace => "spac",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_profile_is_rgb() {
        /* Default profile should be RGB */
        assert!(is_rgb_profile(&IccProfile::default()));
    }

    #[test]
    fn test_export_starts_with_magic() {
        /* Exported blob should start with ICCP magic bytes */
        let blob = export_icc_profile(&IccProfile::default());
        assert_eq!(&blob[..4], b"ICCP");
    }

    #[test]
    fn test_export_nonempty() {
        /* Export should produce non-empty blob */
        assert!(!export_icc_profile(&IccProfile::default()).is_empty());
    }

    #[test]
    fn test_validate_default_profile() {
        /* Default profile should validate */
        assert!(validate_icc_profile(&IccProfile::default()));
    }

    #[test]
    fn test_validate_empty_description_fails() {
        /* Empty description should fail validation */
        let p = IccProfile {
            description: String::new(),
            ..Default::default()
        };
        assert!(!validate_icc_profile(&p));
    }

    #[test]
    fn test_white_point_y_is_one() {
        /* D50 white point Y should be 1.0 for default profile */
        let wp = profile_white_point(&IccProfile::default());
        assert!((wp[1] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_class_tag_display() {
        /* Display class should return mntr tag */
        assert_eq!(class_tag(IccProfileClass::Display), "mntr");
    }

    #[test]
    fn test_class_tag_output() {
        /* Output class should return prtr tag */
        assert_eq!(class_tag(IccProfileClass::Output), "prtr");
    }

    #[test]
    fn test_version_in_export() {
        /* Version bytes should appear in exported blob */
        let p = IccProfile {
            version: (4, 3, 0),
            ..Default::default()
        };
        let blob = export_icc_profile(&p);
        assert!(blob.contains(&4u8));
    }

    #[test]
    fn test_is_rgb_cmyk_false() {
        /* CMYK profile should not be reported as RGB */
        let p = IccProfile {
            color_space: IccColorSpace::Cmyk,
            ..Default::default()
        };
        assert!(!is_rgb_profile(&p));
    }
}
