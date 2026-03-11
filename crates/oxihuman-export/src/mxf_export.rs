// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! MXF (Media Exchange Format) stub export.

/// MXF operational pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MxfOpPattern {
    Op1a,
    Op1b,
    OpAtom,
}

impl MxfOpPattern {
    pub fn name(&self) -> &'static str {
        match self {
            MxfOpPattern::Op1a => "OP1a",
            MxfOpPattern::Op1b => "OP1b",
            MxfOpPattern::OpAtom => "OPAtom",
        }
    }
}

/// MXF essence track.
#[derive(Debug, Clone)]
pub struct MxfTrack {
    pub track_id: u32,
    pub essence_type: String,
    pub duration_frames: u32,
    pub sample_rate: (u32, u32),
}

/// MXF stub export.
#[derive(Debug, Clone)]
pub struct MxfExport {
    pub op_pattern: MxfOpPattern,
    pub tracks: Vec<MxfTrack>,
    pub package_name: String,
}

/// Create a new MXF export.
pub fn new_mxf_export(package_name: &str, op: MxfOpPattern) -> MxfExport {
    MxfExport {
        op_pattern: op,
        tracks: Vec::new(),
        package_name: package_name.to_string(),
    }
}

/// Add a track to the MXF.
pub fn mxf_add_track(
    export: &mut MxfExport,
    track_id: u32,
    essence_type: &str,
    duration: u32,
    rate_num: u32,
    rate_den: u32,
) {
    export.tracks.push(MxfTrack {
        track_id,
        essence_type: essence_type.to_string(),
        duration_frames: duration,
        sample_rate: (rate_num, rate_den),
    });
}

/// Track count.
pub fn mxf_track_count(export: &MxfExport) -> usize {
    export.tracks.len()
}

/// Total duration (max across tracks).
pub fn mxf_duration_frames(export: &MxfExport) -> u32 {
    export
        .tracks
        .iter()
        .map(|t| t.duration_frames)
        .max()
        .unwrap_or(0)
}

/// Validate the MXF.
pub fn validate_mxf(export: &MxfExport) -> bool {
    !export.package_name.is_empty() && export.tracks.iter().all(|t| t.sample_rate.1 > 0)
}

/// Generate a stub MXF header as bytes.
pub fn mxf_header_bytes(export: &MxfExport) -> Vec<u8> {
    let mut out = Vec::new();
    /* MXF key: 060e2b34... (stub) */
    out.extend_from_slice(&[0x06, 0x0e, 0x2b, 0x34, 0x02, 0x05, 0x01, 0x01]);
    out.extend_from_slice(&(export.tracks.len() as u32).to_le_bytes());
    out
}

/// Estimate MXF file size.
pub fn mxf_size_estimate(export: &MxfExport) -> usize {
    /* Header (512) + 1MB per video frame stub */
    512 + export
        .tracks
        .iter()
        .map(|t| t.duration_frames as usize * 64)
        .sum::<usize>()
}

/// Find a track by essence type.
pub fn mxf_find_track_by_type<'a>(export: &'a MxfExport, etype: &str) -> Option<&'a MxfTrack> {
    export.tracks.iter().find(|t| t.essence_type == etype)
}

/// OP pattern name.
pub fn mxf_op_name(export: &MxfExport) -> &'static str {
    export.op_pattern.name()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> MxfExport {
        let mut exp = new_mxf_export("MyPackage", MxfOpPattern::Op1a);
        mxf_add_track(&mut exp, 1, "JPEG2000", 100, 24, 1);
        mxf_add_track(&mut exp, 2, "PCM", 2400, 48000, 1);
        exp
    }

    #[test]
    fn test_track_count() {
        assert_eq!(mxf_track_count(&sample()), 2);
    }

    #[test]
    fn test_duration() {
        assert_eq!(mxf_duration_frames(&sample()), 2400);
    }

    #[test]
    fn test_validate() {
        assert!(validate_mxf(&sample()));
    }

    #[test]
    fn test_header_bytes() {
        let exp = sample();
        let hdr = mxf_header_bytes(&exp);
        assert_eq!(&hdr[0..4], &[0x06, 0x0e, 0x2b, 0x34]);
    }

    #[test]
    fn test_size_estimate() {
        assert!(mxf_size_estimate(&sample()) > 512);
    }

    #[test]
    fn test_find_by_type() {
        let exp = sample();
        assert!(mxf_find_track_by_type(&exp, "JPEG2000").is_some());
        assert!(mxf_find_track_by_type(&exp, "H264").is_none());
    }

    #[test]
    fn test_op_name() {
        let exp = sample();
        assert_eq!(mxf_op_name(&exp), "OP1a");
    }

    #[test]
    fn test_op_pattern_names() {
        assert_eq!(MxfOpPattern::Op1b.name(), "OP1b");
        assert_eq!(MxfOpPattern::OpAtom.name(), "OPAtom");
    }

    #[test]
    fn test_empty_duration() {
        let exp = new_mxf_export("empty", MxfOpPattern::OpAtom);
        assert_eq!(mxf_duration_frames(&exp), 0);
    }
}
