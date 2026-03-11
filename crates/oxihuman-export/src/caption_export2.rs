//! Caption/subtitle track export.
#![allow(dead_code)]

/// A single caption entry.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct CaptionEntry2 {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

/// A caption track.
#[allow(dead_code)]
pub struct CaptionTrack2 {
    pub entries: Vec<CaptionEntry2>,
    pub language: String,
}

/// Create a new caption track.
#[allow(dead_code)]
pub fn new_caption_track2(language: &str) -> CaptionTrack2 {
    CaptionTrack2 { entries: Vec::new(), language: language.to_string() }
}

/// Add a caption entry.
#[allow(dead_code)]
pub fn add_caption2(track: &mut CaptionTrack2, start_ms: u64, end_ms: u64, text: &str) {
    track.entries.push(CaptionEntry2 { start_ms, end_ms, text: text.to_string() });
}

fn ms_to_srt(ms: u64) -> String {
    let h = ms / 3_600_000; let m = (ms % 3_600_000) / 60_000;
    let s = (ms % 60_000) / 1000; let ms_rem = ms % 1000;
    format!("{:02}:{:02}:{:02},{:03}", h, m, s, ms_rem)
}

/// Export captions in SRT format.
#[allow(dead_code)]
pub fn export_captions2_srt(track: &CaptionTrack2) -> String {
    track.entries.iter().enumerate().map(|(i, e)| {
        format!("{}\n{} --> {}\n{}\n\n", i+1, ms_to_srt(e.start_ms), ms_to_srt(e.end_ms), e.text)
    }).collect()
}

/// Export captions in WebVTT format.
#[allow(dead_code)]
pub fn export_captions2_vtt(track: &CaptionTrack2) -> String {
    let mut s = "WEBVTT\n\n".to_string();
    for e in &track.entries {
        let start = ms_to_srt(e.start_ms).replace(',', ".");
        let end = ms_to_srt(e.end_ms).replace(',', ".");
        s.push_str(&format!("{} --> {}\n{}\n\n", start, end, e.text));
    }
    s
}

/// Get caption count.
#[allow(dead_code)]
pub fn caption2_count(track: &CaptionTrack2) -> usize { track.entries.len() }

/// Get caption entry at index.
#[allow(dead_code)]
pub fn caption2_at(track: &CaptionTrack2, i: usize) -> Option<&CaptionEntry2> { track.entries.get(i) }

/// Get duration of a caption (end - start in ms).
#[allow(dead_code)]
pub fn caption2_duration(entry: &CaptionEntry2) -> u64 { entry.end_ms.saturating_sub(entry.start_ms) }

/// Get text of a caption.
#[allow(dead_code)]
pub fn caption2_text(entry: &CaptionEntry2) -> &str { &entry.text }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_caption_track() {
        let t = new_caption_track2("en");
        assert_eq!(caption2_count(&t), 0);
    }

    #[test]
    fn test_add_caption() {
        let mut t = new_caption_track2("en");
        add_caption2(&mut t, 0, 2000, "Hello");
        assert_eq!(caption2_count(&t), 1);
    }

    #[test]
    fn test_export_srt() {
        let mut t = new_caption_track2("en");
        add_caption2(&mut t, 0, 1500, "Hi");
        let s = export_captions2_srt(&t);
        assert!(s.contains("Hi"));
    }

    #[test]
    fn test_export_vtt() {
        let mut t = new_caption_track2("en");
        add_caption2(&mut t, 1000, 3000, "World");
        let s = export_captions2_vtt(&t);
        assert!(s.contains("WEBVTT"));
    }

    #[test]
    fn test_caption_at() {
        let mut t = new_caption_track2("en");
        add_caption2(&mut t, 0, 1000, "Test");
        let e = caption2_at(&t, 0).unwrap();
        assert_eq!(caption2_text(e), "Test");
    }

    #[test]
    fn test_caption_at_oob() {
        let t = new_caption_track2("en");
        assert!(caption2_at(&t, 0).is_none());
    }

    #[test]
    fn test_caption_duration() {
        let e = CaptionEntry2 { start_ms: 1000, end_ms: 3500, text:"x".to_string() };
        assert_eq!(caption2_duration(&e), 2500);
    }

    #[test]
    fn test_caption_text() {
        let e = CaptionEntry2 { start_ms: 0, end_ms: 1000, text: "Hello World".to_string() };
        assert_eq!(caption2_text(&e), "Hello World");
    }

    #[test]
    fn test_vtt_contains_timestamps() {
        let mut t = new_caption_track2("ja");
        add_caption2(&mut t, 0, 2000, "こんにちは");
        let s = export_captions2_vtt(&t);
        assert!(s.contains("-->"));
    }
}
