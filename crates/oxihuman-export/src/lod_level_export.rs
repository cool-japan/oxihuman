#![allow(dead_code)]
//! Export LOD (Level of Detail) data.

/// LOD level export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct LodLevelExport {
    pub levels: Vec<LodLevel>,
}

/// A single LOD level.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct LodLevel {
    pub vertex_count: usize,
    pub face_count: usize,
    pub screen_size: f32,
}

/// Export LOD levels.
#[allow(dead_code)]
pub fn export_lod_levels(levels: &[LodLevel]) -> LodLevelExport {
    LodLevelExport { levels: levels.to_vec() }
}

/// Get the number of LOD levels.
#[allow(dead_code)]
pub fn lod_count_export(export: &LodLevelExport) -> usize {
    export.levels.len()
}

/// Get vertex count at a given LOD level.
#[allow(dead_code)]
pub fn lod_vertex_count_at(export: &LodLevelExport, index: usize) -> Option<usize> {
    export.levels.get(index).map(|l| l.vertex_count)
}

/// Get face count at a given LOD level.
#[allow(dead_code)]
pub fn lod_face_count_at(export: &LodLevelExport, index: usize) -> Option<usize> {
    export.levels.get(index).map(|l| l.face_count)
}

/// Get screen size threshold at a given LOD level.
#[allow(dead_code)]
pub fn lod_screen_size(export: &LodLevelExport, index: usize) -> Option<f32> {
    export.levels.get(index).map(|l| l.screen_size)
}

/// Convert LOD levels to JSON.
#[allow(dead_code)]
pub fn lod_to_json(export: &LodLevelExport) -> String {
    let levels_str: Vec<String> = export.levels.iter().map(|l| {
        format!(
            "{{\"vertex_count\":{},\"face_count\":{},\"screen_size\":{:.4}}}",
            l.vertex_count, l.face_count, l.screen_size
        )
    }).collect();
    format!("{{\"lod_count\":{},\"levels\":[{}]}}", export.levels.len(), levels_str.join(","))
}

/// Get total size across all LOD levels (sum of vertices).
#[allow(dead_code)]
pub fn lod_total_size(export: &LodLevelExport) -> usize {
    export.levels.iter().map(|l| l.vertex_count).sum()
}

/// Validate LOD levels (screen sizes should be decreasing, counts should be decreasing).
#[allow(dead_code)]
pub fn validate_lod_levels(export: &LodLevelExport) -> bool {
    if export.levels.len() < 2 {
        return true;
    }
    for w in export.levels.windows(2) {
        if w[1].screen_size > w[0].screen_size {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> LodLevelExport {
        export_lod_levels(&[
            LodLevel { vertex_count: 1000, face_count: 500, screen_size: 1.0 },
            LodLevel { vertex_count: 500, face_count: 250, screen_size: 0.5 },
            LodLevel { vertex_count: 100, face_count: 50, screen_size: 0.1 },
        ])
    }

    #[test]
    fn test_export_lod_levels() {
        let lod = sample();
        assert_eq!(lod_count_export(&lod), 3);
    }

    #[test]
    fn test_lod_vertex_count_at() {
        let lod = sample();
        assert_eq!(lod_vertex_count_at(&lod, 0), Some(1000));
    }

    #[test]
    fn test_lod_vertex_count_oob() {
        let lod = sample();
        assert_eq!(lod_vertex_count_at(&lod, 10), None);
    }

    #[test]
    fn test_lod_face_count_at() {
        let lod = sample();
        assert_eq!(lod_face_count_at(&lod, 1), Some(250));
    }

    #[test]
    fn test_lod_screen_size() {
        let lod = sample();
        assert!((lod_screen_size(&lod, 2).unwrap() - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_lod_to_json() {
        let lod = sample();
        let j = lod_to_json(&lod);
        assert!(j.contains("lod_count"));
    }

    #[test]
    fn test_lod_total_size() {
        let lod = sample();
        assert_eq!(lod_total_size(&lod), 1600);
    }

    #[test]
    fn test_validate_lod_levels() {
        let lod = sample();
        assert!(validate_lod_levels(&lod));
    }

    #[test]
    fn test_validate_lod_levels_bad() {
        let lod = export_lod_levels(&[
            LodLevel { vertex_count: 100, face_count: 50, screen_size: 0.1 },
            LodLevel { vertex_count: 1000, face_count: 500, screen_size: 1.0 },
        ]);
        assert!(!validate_lod_levels(&lod));
    }

    #[test]
    fn test_validate_lod_levels_single() {
        let lod = export_lod_levels(&[
            LodLevel { vertex_count: 100, face_count: 50, screen_size: 1.0 },
        ]);
        assert!(validate_lod_levels(&lod));
    }
}
