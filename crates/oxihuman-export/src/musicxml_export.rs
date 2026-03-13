// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! MusicXML stub export.

/// A MusicXML note.
#[derive(Debug, Clone)]
pub struct MxmlNote {
    pub step: char,
    pub octave: i32,
    pub duration: u32,
    pub note_type: String,
    pub alter: i32,
    pub is_rest: bool,
}

impl MxmlNote {
    pub fn new(step: char, octave: i32, duration: u32, note_type: impl Into<String>) -> Self {
        Self {
            step,
            octave,
            duration,
            note_type: note_type.into(),
            alter: 0,
            is_rest: false,
        }
    }

    pub fn rest(duration: u32, note_type: impl Into<String>) -> Self {
        Self {
            step: 'R',
            octave: 4,
            duration,
            note_type: note_type.into(),
            alter: 0,
            is_rest: true,
        }
    }

    pub fn to_xml(&self) -> String {
        if self.is_rest {
            format!(
                "      <note><rest/><duration>{}</duration><type>{}</type></note>",
                self.duration, self.note_type
            )
        } else {
            let alter_str = if self.alter != 0 {
                format!("<alter>{}</alter>", self.alter)
            } else {
                String::new()
            };
            format!(
                "      <note><pitch><step>{}</step>{}<octave>{}</octave></pitch><duration>{}</duration><type>{}</type></note>",
                self.step, alter_str, self.octave, self.duration, self.note_type
            )
        }
    }
}

/// A MusicXML measure.
#[derive(Debug, Clone, Default)]
pub struct MxmlMeasure {
    pub number: u32,
    pub notes: Vec<MxmlNote>,
}

impl MxmlMeasure {
    pub fn new(number: u32) -> Self {
        Self {
            number,
            notes: Vec::new(),
        }
    }

    pub fn add_note(&mut self, note: MxmlNote) {
        self.notes.push(note);
    }

    pub fn to_xml(&self) -> String {
        let mut xml = format!("    <measure number=\"{}\">\n", self.number);
        for note in &self.notes {
            xml.push_str(&note.to_xml());
            xml.push('\n');
        }
        xml.push_str("    </measure>\n");
        xml
    }
}

/// A MusicXML part (instrument part).
#[derive(Debug, Clone, Default)]
pub struct MxmlPart {
    pub id: String,
    pub name: String,
    pub measures: Vec<MxmlMeasure>,
}

impl MxmlPart {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            measures: Vec::new(),
        }
    }

    pub fn add_measure(&mut self, measure: MxmlMeasure) {
        self.measures.push(measure);
    }
}

/// Generate a full MusicXML document string.
pub fn generate_musicxml(parts: &[MxmlPart], title: &str) -> String {
    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str(
        "<!DOCTYPE score-partwise PUBLIC \"-//Recordare//DTD MusicXML 4.0 Partwise//EN\"\n",
    );
    xml.push_str("  \"http://www.musicxml.org/dtds/partwise.dtd\">\n");
    xml.push_str("<score-partwise version=\"4.0\">\n");
    xml.push_str("  <work><work-title>");
    xml.push_str(title);
    xml.push_str("</work-title></work>\n");
    xml.push_str("  <part-list>\n");
    for part in parts {
        xml.push_str(&format!("    <score-part id=\"{}\">\n", part.id));
        xml.push_str(&format!(
            "      <part-name>{}</part-name>\n    </score-part>\n",
            part.name
        ));
    }
    xml.push_str("  </part-list>\n");
    for part in parts {
        xml.push_str(&format!("  <part id=\"{}\">\n", part.id));
        for measure in &part.measures {
            xml.push_str(&measure.to_xml());
        }
        xml.push_str("  </part>\n");
    }
    xml.push_str("</score-partwise>\n");
    xml
}

/// Validate that a string is a MusicXML document.
pub fn is_valid_musicxml(src: &str) -> bool {
    src.contains("<score-partwise") && src.contains("</score-partwise>")
}

/// Count total notes across all parts and measures.
pub fn count_total_notes(parts: &[MxmlPart]) -> usize {
    parts
        .iter()
        .flat_map(|p| p.measures.iter())
        .flat_map(|m| m.notes.iter())
        .count()
}

/// Build a simple one-bar piano part with a whole note C.
pub fn single_note_part(note_step: char, octave: i32) -> MxmlPart {
    let mut part = MxmlPart::new("P1", "Piano");
    let mut measure = MxmlMeasure::new(1);
    measure.add_note(MxmlNote::new(note_step, octave, 4, "whole"));
    part.add_measure(measure);
    part
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_to_xml_contains_pitch() {
        let note = MxmlNote::new('C', 4, 4, "whole");
        let xml = note.to_xml();
        assert!(xml.contains("<step>C</step>") /* pitch step */);
        assert!(xml.contains("<octave>4</octave>") /* octave */);
    }

    #[test]
    fn test_rest_to_xml() {
        let rest = MxmlNote::rest(4, "whole");
        let xml = rest.to_xml();
        assert!(xml.contains("<rest/>") /* rest element */);
    }

    #[test]
    fn test_measure_to_xml_number() {
        let measure = MxmlMeasure::new(1);
        let xml = measure.to_xml();
        assert!(xml.contains("number=\"1\"") /* measure number */);
    }

    #[test]
    fn test_generate_musicxml_valid() {
        let part = single_note_part('C', 4);
        let xml = generate_musicxml(&[part], "Test");
        assert!(is_valid_musicxml(&xml) /* valid MusicXML */);
    }

    #[test]
    fn test_generate_musicxml_title() {
        let xml = generate_musicxml(&[], "My Piece");
        assert!(xml.contains("My Piece") /* title in XML */);
    }

    #[test]
    fn test_count_total_notes() {
        let part = single_note_part('G', 4);
        assert_eq!(count_total_notes(&[part]), 1 /* one note */);
    }

    #[test]
    fn test_count_notes_empty() {
        assert_eq!(count_total_notes(&[]), 0 /* empty parts */);
    }

    #[test]
    fn test_is_valid_musicxml_false() {
        assert!(!is_valid_musicxml("<xml>random</xml>") /* not MusicXML */);
    }

    #[test]
    fn test_note_with_alter() {
        let mut note = MxmlNote::new('F', 4, 1, "quarter");
        note.alter = 1; /* F# */
        let xml = note.to_xml();
        assert!(xml.contains("<alter>1</alter>") /* sharp */);
    }
}
