//! Hair/fur groom data export for DCC tool integration.

// ── Types ─────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum GroomFormat {
    Json,
    Abc,
    Usd,
    Binary,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GroomExportConfig {
    pub format: GroomFormat,
    pub strand_count_limit: usize,
    pub segments_per_strand: usize,
    pub include_clumping: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GroomStrand {
    pub root: [f32; 3],
    pub segments: Vec<[f32; 3]>,
    pub width: f32,
    pub clump_id: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GroomData {
    pub strands: Vec<GroomStrand>,
    pub root_count: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GroomExportResult {
    pub data_string: String,
    pub strand_count: usize,
    pub total_segments: usize,
}

// ── Functions ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn default_groom_export_config() -> GroomExportConfig {
    GroomExportConfig {
        format: GroomFormat::Json,
        strand_count_limit: 10_000,
        segments_per_strand: 8,
        include_clumping: true,
    }
}

#[allow(dead_code)]
pub fn new_groom_data() -> GroomData {
    GroomData {
        strands: Vec::new(),
        root_count: 0,
    }
}

#[allow(dead_code)]
pub fn add_groom_strand(data: &mut GroomData, strand: GroomStrand) {
    data.root_count += 1;
    data.strands.push(strand);
}

#[allow(dead_code)]
pub fn new_groom_strand(root: [f32; 3], width: f32) -> GroomStrand {
    GroomStrand {
        root,
        segments: Vec::new(),
        width,
        clump_id: 0,
    }
}

#[allow(dead_code)]
pub fn add_segment(strand: &mut GroomStrand, pos: [f32; 3]) {
    strand.segments.push(pos);
}

#[allow(dead_code)]
pub fn export_groom(data: &GroomData, cfg: &GroomExportConfig) -> GroomExportResult {
    let limit = cfg.strand_count_limit;
    let exported: Vec<&GroomStrand> = data.strands.iter().take(limit).collect();
    let strand_count = exported.len();
    let total_segments: usize = exported.iter().map(|s| s.segments.len()).sum();

    let fmt_name = groom_format_name(cfg);
    let data_string = format!(
        "{{\"format\":\"{}\",\"strands\":{},\"segments\":{}}}",
        fmt_name, strand_count, total_segments
    );

    GroomExportResult {
        data_string,
        strand_count,
        total_segments,
    }
}

#[allow(dead_code)]
pub fn groom_strand_length(strand: &GroomStrand) -> f32 {
    if strand.segments.is_empty() {
        return 0.0;
    }
    let mut len = 0.0f32;
    let mut prev = strand.root;
    for &seg in &strand.segments {
        let dx = seg[0] - prev[0];
        let dy = seg[1] - prev[1];
        let dz = seg[2] - prev[2];
        len += (dx * dx + dy * dy + dz * dz).sqrt();
        prev = seg;
    }
    len
}

#[allow(dead_code)]
pub fn groom_total_segments(data: &GroomData) -> usize {
    data.strands.iter().map(|s| s.segments.len()).sum()
}

#[allow(dead_code)]
pub fn groom_format_name(cfg: &GroomExportConfig) -> &'static str {
    match cfg.format {
        GroomFormat::Json => "json",
        GroomFormat::Abc => "abc",
        GroomFormat::Usd => "usd",
        GroomFormat::Binary => "binary",
    }
}

#[allow(dead_code)]
pub fn groom_export_result_to_json(r: &GroomExportResult) -> String {
    format!(
        "{{\"strand_count\":{},\"total_segments\":{},\"data\":\"{}\"}}",
        r.strand_count, r.total_segments, r.data_string
    )
}

#[allow(dead_code)]
pub fn validate_groom(data: &GroomData) -> bool {
    if data.strands.is_empty() {
        return false;
    }
    data.strands.iter().all(|s| s.width > 0.0)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_json_format() {
        let cfg = default_groom_export_config();
        assert_eq!(cfg.format, GroomFormat::Json);
        assert_eq!(cfg.strand_count_limit, 10_000);
    }

    #[test]
    fn add_strand_increments_root_count() {
        let mut data = new_groom_data();
        let strand = new_groom_strand([0.0, 0.0, 0.0], 0.01);
        add_groom_strand(&mut data, strand);
        assert_eq!(data.root_count, 1);
        assert_eq!(data.strands.len(), 1);
    }

    #[test]
    fn add_segment_grows_strand() {
        let mut strand = new_groom_strand([0.0, 0.0, 0.0], 0.01);
        add_segment(&mut strand, [0.0, 1.0, 0.0]);
        add_segment(&mut strand, [0.0, 2.0, 0.0]);
        assert_eq!(strand.segments.len(), 2);
    }

    #[test]
    fn groom_strand_length_single_segment() {
        let mut strand = new_groom_strand([0.0, 0.0, 0.0], 0.01);
        add_segment(&mut strand, [0.0, 3.0, 0.0]);
        let len = groom_strand_length(&strand);
        assert!((len - 3.0).abs() < 1e-5);
    }

    #[test]
    fn groom_total_segments_sums_all() {
        let mut data = new_groom_data();
        let mut s1 = new_groom_strand([0.0, 0.0, 0.0], 0.01);
        add_segment(&mut s1, [0.0, 1.0, 0.0]);
        let mut s2 = new_groom_strand([1.0, 0.0, 0.0], 0.01);
        add_segment(&mut s2, [1.0, 1.0, 0.0]);
        add_segment(&mut s2, [1.0, 2.0, 0.0]);
        add_groom_strand(&mut data, s1);
        add_groom_strand(&mut data, s2);
        assert_eq!(groom_total_segments(&data), 3);
    }

    #[test]
    fn export_groom_respects_limit() {
        let mut data = new_groom_data();
        for i in 0..5 {
            let mut s = new_groom_strand([i as f32, 0.0, 0.0], 0.01);
            add_segment(&mut s, [i as f32, 1.0, 0.0]);
            add_groom_strand(&mut data, s);
        }
        let mut cfg = default_groom_export_config();
        cfg.strand_count_limit = 3;
        let result = export_groom(&data, &cfg);
        assert_eq!(result.strand_count, 3);
    }

    #[test]
    fn validate_groom_fails_empty() {
        let data = new_groom_data();
        assert!(!validate_groom(&data));
    }

    #[test]
    fn validate_groom_fails_zero_width() {
        let mut data = new_groom_data();
        let strand = new_groom_strand([0.0, 0.0, 0.0], 0.0);
        add_groom_strand(&mut data, strand);
        assert!(!validate_groom(&data));
    }

    #[test]
    fn format_names_correct() {
        let mut cfg = default_groom_export_config();
        cfg.format = GroomFormat::Abc;
        assert_eq!(groom_format_name(&cfg), "abc");
        cfg.format = GroomFormat::Usd;
        assert_eq!(groom_format_name(&cfg), "usd");
        cfg.format = GroomFormat::Binary;
        assert_eq!(groom_format_name(&cfg), "binary");
    }

    #[test]
    fn result_to_json_contains_strand_count() {
        let r = GroomExportResult {
            data_string: "{}".to_string(),
            strand_count: 42,
            total_segments: 84,
        };
        let json = groom_export_result_to_json(&r);
        assert!(json.contains("42"));
        assert!(json.contains("84"));
    }
}
