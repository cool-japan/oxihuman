// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Export morph offsets to CSV format: "name,idx,x,y,z".
#[allow(dead_code)]
pub fn morph_to_csv(name: &str, offsets: &[[f32; 3]]) -> String {
    let mut out = morph_header();
    out.push('\n');
    for (i, o) in offsets.iter().enumerate() {
        out.push_str(&offset_to_csv_row(i, *o));
        out.push('\n');
        // Prepend name for each row (name,idx,x,y,z)
    }
    // Rebuild properly
    let mut result = morph_header();
    result.push('\n');
    for (i, o) in offsets.iter().enumerate() {
        result.push_str(name);
        result.push(',');
        result.push_str(&offset_to_csv_row(i, *o));
        result.push('\n');
    }
    result
}

/// Parse CSV rows ("name,idx,x,y,z") back into a Vec of offset triples.
#[allow(dead_code)]
pub fn parse_morph_offsets_csv(csv: &str) -> Vec<[f32; 3]> {
    let mut offsets = Vec::new();
    for line in csv.lines() {
        // Skip header line
        if line.starts_with("name,") {
            continue;
        }
        let parts: Vec<&str> = line.splitn(5, ',').collect();
        if parts.len() < 5 {
            continue;
        }
        let x = parts[2].trim().parse::<f32>().unwrap_or(0.0);
        let y = parts[3].trim().parse::<f32>().unwrap_or(0.0);
        let z = parts[4].trim().parse::<f32>().unwrap_or(0.0);
        offsets.push([x, y, z]);
    }
    offsets
}

/// Return the CSV header line.
#[allow(dead_code)]
pub fn morph_header() -> String {
    "name,idx,x,y,z".to_string()
}

/// Encode a single offset as "idx,x,y,z".
#[allow(dead_code)]
pub fn offset_to_csv_row(idx: usize, offset: [f32; 3]) -> String {
    format!("{},{},{},{}", idx, offset[0], offset[1], offset[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn morph_header_content() {
        assert_eq!(morph_header(), "name,idx,x,y,z");
    }

    #[test]
    fn offset_to_csv_row_format() {
        let row = offset_to_csv_row(0, [1.0, 2.0, 3.0]);
        assert_eq!(row, "0,1,2,3");
    }

    #[test]
    fn morph_to_csv_contains_header() {
        let csv = morph_to_csv("smile", &[[0.1, 0.2, 0.3]]);
        assert!(csv.contains("name,idx,x,y,z"));
    }

    #[test]
    fn morph_to_csv_contains_name() {
        let csv = morph_to_csv("wink", &[[0.0, 0.0, 0.0]]);
        assert!(csv.contains("wink"));
    }

    #[test]
    fn morph_to_csv_empty_offsets() {
        let csv = morph_to_csv("none", &[]);
        assert!(csv.contains("name,idx,x,y,z"));
    }

    #[test]
    fn parse_morph_offsets_csv_roundtrip() {
        let offsets = vec![[0.1f32, 0.2, 0.3], [0.4, 0.5, 0.6]];
        let csv = morph_to_csv("m", &offsets);
        let parsed = parse_morph_offsets_csv(&csv);
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn parse_morph_offsets_csv_skips_header() {
        let csv = "name,idx,x,y,z\nsmile,0,0.1,0.2,0.3";
        let v = parse_morph_offsets_csv(csv);
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn parse_morph_offsets_csv_values_correct() {
        let csv = "name,idx,x,y,z\nm,0,0.5,0.6,0.7";
        let v = parse_morph_offsets_csv(csv);
        assert!((v[0][0] - 0.5).abs() < 1e-5);
        assert!((v[0][1] - 0.6).abs() < 1e-5);
        assert!((v[0][2] - 0.7).abs() < 1e-5);
    }

    #[test]
    fn parse_morph_offsets_csv_empty() {
        let v = parse_morph_offsets_csv("");
        assert!(v.is_empty());
    }

    #[test]
    fn morph_to_csv_row_count() {
        let offsets = vec![[0.0; 3]; 4];
        let csv = morph_to_csv("t", &offsets);
        // header line + 4 data lines (in the second half of morph_to_csv)
        let data_lines = csv.lines().filter(|l| !l.starts_with("name,") && !l.is_empty()).count();
        assert_eq!(data_lines, 4);
    }
}
