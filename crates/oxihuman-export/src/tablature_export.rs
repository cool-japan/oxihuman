// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Generic ASCII tablature stub export.

/// A single fret position on a string.
#[derive(Debug, Clone, Copy)]
pub struct TabNote {
    pub string_idx: usize,
    pub fret: u8,
}

impl TabNote {
    pub fn new(string_idx: usize, fret: u8) -> Self {
        Self { string_idx, fret }
    }
}

/// A column in an ASCII tab (one beat / time slice).
#[derive(Debug, Clone, Default)]
pub struct TabColumn {
    pub frets: Vec<Option<u8>>,
    pub is_bar_line: bool,
}

impl TabColumn {
    pub fn new_beat(num_strings: usize) -> Self {
        Self {
            frets: vec![None; num_strings],
            is_bar_line: false,
        }
    }

    pub fn bar_line(num_strings: usize) -> Self {
        Self {
            frets: vec![None; num_strings],
            is_bar_line: true,
        }
    }

    pub fn set_fret(&mut self, string_idx: usize, fret: u8) {
        if string_idx < self.frets.len() {
            self.frets[string_idx] = Some(fret);
        }
    }
}

/// An ASCII tablature staff.
#[derive(Debug, Clone, Default)]
pub struct TabStaff {
    pub string_names: Vec<String>,
    pub columns: Vec<TabColumn>,
}

impl TabStaff {
    pub fn new(string_names: Vec<String>) -> Self {
        Self {
            string_names,
            columns: Vec::new(),
        }
    }

    pub fn guitar_standard() -> Self {
        let names = vec!["e", "B", "G", "D", "A", "E"]
            .into_iter()
            .map(str::to_string)
            .collect();
        Self::new(names)
    }

    pub fn add_column(&mut self, col: TabColumn) {
        self.columns.push(col);
    }

    pub fn add_bar_line(&mut self) {
        let n = self.string_names.len();
        self.columns.push(TabColumn::bar_line(n));
    }

    pub fn num_strings(&self) -> usize {
        self.string_names.len()
    }
}

/// Render an ASCII tablature staff to a string.
pub fn render_tab_ascii(staff: &TabStaff) -> String {
    let mut rows: Vec<String> = staff
        .string_names
        .iter()
        .map(|s| format!("{}-", s))
        .collect();
    for col in &staff.columns {
        if col.is_bar_line {
            for row in &mut rows {
                row.push('|');
            }
        } else {
            for (i, row) in rows.iter_mut().enumerate() {
                match col.frets.get(i).copied().flatten() {
                    Some(f) => {
                        if f >= 10 {
                            row.push_str(&format!("{}", f));
                        } else {
                            row.push_str(&format!("{}-", f));
                        }
                    }
                    None => row.push_str("--"),
                }
            }
        }
    }
    /* Add trailing bar line */
    for row in &mut rows {
        row.push('|');
    }
    rows.join("\n") + "\n"
}

/// Count note columns (non-bar-line) in the staff.
pub fn count_tab_beats(staff: &TabStaff) -> usize {
    staff.columns.iter().filter(|c| !c.is_bar_line).count()
}

/// Count total notes placed in the staff.
pub fn count_tab_notes(staff: &TabStaff) -> usize {
    staff
        .columns
        .iter()
        .flat_map(|c| c.frets.iter())
        .filter(|f| f.is_some())
        .count()
}

/// Build a simple pentatonic riff tab for guitar.
pub fn pentatonic_riff_tab() -> TabStaff {
    let mut staff = TabStaff::guitar_standard();
    let n = staff.num_strings();
    /* Low E string (index 5): frets 0, 3, 5 */
    for &fret in &[0u8, 3, 5] {
        let mut col = TabColumn::new_beat(n);
        col.set_fret(5, fret);
        staff.add_column(col);
    }
    staff.add_bar_line();
    /* A string (index 4): frets 2, 5 */
    for &fret in &[2u8, 5] {
        let mut col = TabColumn::new_beat(n);
        col.set_fret(4, fret);
        staff.add_column(col);
    }
    staff
}

/// Validate that a rendered tab has the expected number of string lines.
pub fn tab_has_correct_string_count(src: &str, expected: usize) -> bool {
    src.lines().filter(|l| !l.is_empty()).count() == expected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guitar_standard_strings() {
        let staff = TabStaff::guitar_standard();
        assert_eq!(staff.num_strings(), 6 /* standard 6-string guitar */);
    }

    #[test]
    fn test_tab_column_set_fret() {
        let mut col = TabColumn::new_beat(6);
        col.set_fret(0, 5);
        assert_eq!(col.frets[0], Some(5) /* fret set */);
    }

    #[test]
    fn test_tab_column_out_of_bounds() {
        let mut col = TabColumn::new_beat(2);
        col.set_fret(10, 3); /* out of bounds: no panic */
        assert_eq!(col.frets.len(), 2 /* unchanged length */);
    }

    #[test]
    fn test_render_tab_ascii_lines() {
        let staff = TabStaff::guitar_standard();
        let rendered = render_tab_ascii(&staff);
        assert!(tab_has_correct_string_count(&rendered, 6) /* 6 lines */);
    }

    #[test]
    fn test_count_tab_beats() {
        let staff = pentatonic_riff_tab();
        assert_eq!(count_tab_beats(&staff), 5 /* 3 + 2 beats */);
    }

    #[test]
    fn test_count_tab_notes() {
        let staff = pentatonic_riff_tab();
        assert_eq!(count_tab_notes(&staff), 5 /* 5 placed frets */);
    }

    #[test]
    fn test_render_contains_string_name() {
        let staff = TabStaff::guitar_standard();
        let rendered = render_tab_ascii(&staff);
        assert!(rendered.contains('e') /* high-e string name */);
    }

    #[test]
    fn test_bar_line_renders() {
        let mut staff = TabStaff::guitar_standard();
        staff.add_bar_line();
        let rendered = render_tab_ascii(&staff);
        /* Bar lines appear as | characters */
        assert!(rendered.contains('|') /* bar line character */);
    }

    #[test]
    fn test_pentatonic_riff_has_bar_line() {
        let staff = pentatonic_riff_tab();
        let has_bar = staff.columns.iter().any(|c| c.is_bar_line);
        assert!(has_bar /* riff has at least one bar line */);
    }
}
