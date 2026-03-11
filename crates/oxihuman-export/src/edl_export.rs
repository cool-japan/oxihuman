// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! CMX 3600 EDL (Edit Decision List) export.

/// An EDL event entry.
#[derive(Debug, Clone)]
pub struct EdlEvent {
    pub event_num: u32,
    pub reel: String,
    pub channels: String,
    pub transition: String,
    pub src_in: String,
    pub src_out: String,
    pub rec_in: String,
    pub rec_out: String,
}

/// A CMX 3600 EDL export.
#[derive(Debug, Clone)]
pub struct EdlExport {
    pub title: String,
    pub fcm: String, /* Frame Count Mode: "NON-DROP FRAME" or "DROP FRAME" */
    pub events: Vec<EdlEvent>,
}

/// SMPTE timecode from frame count at 24fps.
pub fn frames_to_timecode(frames: u32, fps: u32) -> String {
    let fps = fps.max(1);
    let f = frames % fps;
    let s = (frames / fps) % 60;
    let m = (frames / fps / 60) % 60;
    let h = frames / fps / 3600;
    format!("{:02}:{:02}:{:02}:{:02}", h, m, s, f)
}

/// Create a new EDL export.
pub fn new_edl_export(title: &str, drop_frame: bool) -> EdlExport {
    EdlExport {
        title: title.to_string(),
        fcm: if drop_frame {
            "DROP FRAME".to_string()
        } else {
            "NON-DROP FRAME".to_string()
        },
        events: Vec::new(),
    }
}

/// Add an event to the EDL.
#[allow(clippy::too_many_arguments)]
pub fn edl_add_event(
    export: &mut EdlExport,
    reel: &str,
    channels: &str,
    transition: &str,
    src_in: &str,
    src_out: &str,
    rec_in: &str,
    rec_out: &str,
) {
    let event_num = export.events.len() as u32 + 1;
    export.events.push(EdlEvent {
        event_num,
        reel: reel.to_string(),
        channels: channels.to_string(),
        transition: transition.to_string(),
        src_in: src_in.to_string(),
        src_out: src_out.to_string(),
        rec_in: rec_in.to_string(),
        rec_out: rec_out.to_string(),
    });
}

/// Return the event count.
pub fn edl_event_count(export: &EdlExport) -> usize {
    export.events.len()
}

/// Serialize the EDL to a string in CMX 3600 format.
pub fn edl_to_string(export: &EdlExport) -> String {
    let mut out = format!("TITLE: {}\nFCM: {}\n\n", export.title, export.fcm);
    for ev in &export.events {
        out.push_str(&format!(
            "{:03}  {}  {}  {}  {}  {}  {}  {}\n",
            ev.event_num,
            ev.reel,
            ev.channels,
            ev.transition,
            ev.src_in,
            ev.src_out,
            ev.rec_in,
            ev.rec_out,
        ));
    }
    out
}

/// Estimate the EDL file size.
pub fn edl_size_bytes(export: &EdlExport) -> usize {
    edl_to_string(export).len()
}

/// Find events for a specific reel.
pub fn events_for_reel<'a>(export: &'a EdlExport, reel: &str) -> Vec<&'a EdlEvent> {
    export.events.iter().filter(|e| e.reel == reel).collect()
}

/// Validate the EDL.
pub fn validate_edl(export: &EdlExport) -> bool {
    !export.title.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_edl() -> EdlExport {
        let mut exp = new_edl_export("MY_EDIT", false);
        edl_add_event(
            &mut exp,
            "A001",
            "V",
            "C",
            "01:00:00:00",
            "01:00:04:00",
            "01:00:00:00",
            "01:00:04:00",
        );
        edl_add_event(
            &mut exp,
            "B002",
            "V",
            "C",
            "01:00:04:00",
            "01:00:08:00",
            "01:00:04:00",
            "01:00:08:00",
        );
        exp
    }

    #[test]
    fn test_event_count() {
        assert_eq!(edl_event_count(&sample_edl()), 2);
    }

    #[test]
    fn test_to_string_title() {
        assert!(edl_to_string(&sample_edl()).contains("MY_EDIT"));
    }

    #[test]
    fn test_to_string_event() {
        assert!(edl_to_string(&sample_edl()).contains("A001"));
    }

    #[test]
    fn test_validate() {
        assert!(validate_edl(&sample_edl()));
    }

    #[test]
    fn test_events_for_reel() {
        let exp = sample_edl();
        assert_eq!(events_for_reel(&exp, "A001").len(), 1);
        assert_eq!(events_for_reel(&exp, "B002").len(), 1);
        assert_eq!(events_for_reel(&exp, "C003").len(), 0);
    }

    #[test]
    fn test_frames_to_timecode() {
        let tc = frames_to_timecode(0, 24);
        assert_eq!(tc, "00:00:00:00");
    }

    #[test]
    fn test_frames_to_timecode_24fps() {
        /* 24 frames = 1 second at 24fps */
        let tc = frames_to_timecode(24, 24);
        assert_eq!(tc, "00:00:01:00");
    }

    #[test]
    fn test_edl_size_positive() {
        assert!(edl_size_bytes(&sample_edl()) > 0);
    }

    #[test]
    fn test_fcm_drop_frame() {
        let exp = new_edl_export("X", true);
        assert!(exp.fcm.contains("DROP FRAME"));
    }
}
