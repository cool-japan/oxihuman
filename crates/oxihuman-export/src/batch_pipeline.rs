// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Parallel batch character export — generate and export multiple character
//! variants from a parameter grid.

#![allow(dead_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ── Public types ──────────────────────────────────────────────────────────────

/// Specification for a single character export job.
pub struct BatchCharacterSpec {
    pub id: String,
    pub params: HashMap<String, f32>,
    pub output_format: BatchOutputFormat,
    pub output_path: PathBuf,
}

/// Supported output formats for batch export.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatchOutputFormat {
    Glb,
    Obj,
    Stl,
    Json,
    Csv,
}

impl BatchOutputFormat {
    fn extension(self) -> &'static str {
        match self {
            BatchOutputFormat::Glb => "glb",
            BatchOutputFormat::Obj => "obj",
            BatchOutputFormat::Stl => "stl",
            BatchOutputFormat::Json => "json",
            BatchOutputFormat::Csv => "csv",
        }
    }

    fn name(self) -> &'static str {
        match self {
            BatchOutputFormat::Glb => "glb",
            BatchOutputFormat::Obj => "obj",
            BatchOutputFormat::Stl => "stl",
            BatchOutputFormat::Json => "json",
            BatchOutputFormat::Csv => "csv",
        }
    }
}

/// Configuration for a batch run.
pub struct BatchConfig {
    /// Optional path to a base .obj mesh file. If `None`, a stub tetrahedron
    /// is generated in-memory.
    pub base_obj_path: Option<PathBuf>,
    /// Max parallel jobs (config only; sequential execution for now).
    pub max_parallel: usize,
    /// Skip export if the output file already exists.
    pub skip_existing: bool,
    /// Print per-job messages to stdout.
    pub verbose: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        BatchConfig {
            base_obj_path: None,
            max_parallel: 4,
            skip_existing: false,
            verbose: false,
        }
    }
}

/// Aggregated results for a batch run.
pub struct BatchResult {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub skipped: usize,
    /// `(id, error_message)` for each failed job.
    pub errors: Vec<(String, String)>,
}

// ── Tetrahedron stub ──────────────────────────────────────────────────────────

/// Generate a minimal 4-vertex tetrahedron in OBJ text format.
fn stub_tetrahedron_obj() -> String {
    concat!(
        "# OxiHuman stub tetrahedron\n",
        "v  1.0  1.0  1.0\n",
        "v -1.0 -1.0  1.0\n",
        "v -1.0  1.0 -1.0\n",
        "v  1.0 -1.0 -1.0\n",
        "f 1 2 3\n",
        "f 1 2 4\n",
        "f 1 3 4\n",
        "f 2 3 4\n",
    )
    .to_string()
}

// ── Core batch runner ─────────────────────────────────────────────────────────

/// Run a batch export for each spec in `specs`.
pub fn run_batch(specs: &[BatchCharacterSpec], cfg: &BatchConfig) -> BatchResult {
    let total = specs.len();
    let mut succeeded = 0usize;
    let mut failed = 0usize;
    let mut skipped = 0usize;
    let mut errors: Vec<(String, String)> = Vec::new();

    for spec in specs {
        // Skip if output already exists and skip_existing is set.
        if cfg.skip_existing && spec.output_path.exists() {
            if cfg.verbose {
                println!("[batch] skip (exists): {}", spec.output_path.display());
            }
            skipped += 1;
            continue;
        }

        if cfg.verbose {
            println!(
                "[batch] exporting: {} → {}",
                spec.id,
                spec.output_path.display()
            );
        }

        match export_one(spec, cfg) {
            Ok(()) => {
                succeeded += 1;
            }
            Err(e) => {
                if cfg.verbose {
                    eprintln!("[batch] FAILED {}: {}", spec.id, e);
                }
                errors.push((spec.id.clone(), e));
                failed += 1;
            }
        }
    }

    BatchResult {
        total,
        succeeded,
        failed,
        skipped,
        errors,
    }
}

/// Export a single character spec.
fn export_one(spec: &BatchCharacterSpec, cfg: &BatchConfig) -> Result<(), String> {
    // Ensure the parent directory exists.
    if let Some(parent) = spec.output_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create_dir_all: {}", e))?;
    }

    // Load or generate mesh content.
    let mesh_content = if let Some(ref base_path) = cfg.base_obj_path {
        std::fs::read_to_string(base_path).map_err(|e| format!("reading base OBJ: {}", e))?
    } else {
        stub_tetrahedron_obj()
    };

    // Produce output based on format.
    let content = match spec.output_format {
        BatchOutputFormat::Obj => mesh_content,
        BatchOutputFormat::Stl => obj_to_stl_stub(&mesh_content, &spec.id),
        BatchOutputFormat::Glb => obj_to_glb_stub(&mesh_content),
        BatchOutputFormat::Json => params_to_json(&spec.params),
        BatchOutputFormat::Csv => params_to_csv(&spec.params),
    };

    std::fs::write(&spec.output_path, content).map_err(|e| format!("writing output: {}", e))?;

    Ok(())
}

// ── Format converters (stubs that produce valid minimal output) ───────────────

fn obj_to_stl_stub(obj: &str, name: &str) -> String {
    let mut stl = format!("solid {}\n", name);
    // Parse triangles from OBJ and emit dummy normals
    let verts: Vec<[f32; 3]> = obj
        .lines()
        .filter(|l| l.starts_with("v "))
        .filter_map(|l| {
            let mut it = l[2..].split_whitespace();
            let x: f32 = it.next()?.parse().ok()?;
            let y: f32 = it.next()?.parse().ok()?;
            let z: f32 = it.next()?.parse().ok()?;
            Some([x, y, z])
        })
        .collect();
    for line in obj.lines().filter(|l| l.starts_with("f ")) {
        let indices: Vec<usize> = line[2..]
            .split_whitespace()
            .filter_map(|t| t.split('/').next()?.parse::<usize>().ok())
            .collect();
        if indices.len() >= 3 {
            let (a, b, c) = (indices[0] - 1, indices[1] - 1, indices[2] - 1);
            if a < verts.len() && b < verts.len() && c < verts.len() {
                stl.push_str("  facet normal 0 0 0\n    outer loop\n");
                for &idx in &[a, b, c] {
                    let v = verts[idx];
                    stl.push_str(&format!("      vertex {} {} {}\n", v[0], v[1], v[2]));
                }
                stl.push_str("    endloop\n  endfacet\n");
            }
        }
    }
    stl.push_str(&format!("endsolid {}\n", name));
    stl
}

fn obj_to_glb_stub(obj: &str) -> String {
    // Return a minimal JSON representation (true GLB would be binary).
    // For batch testing purposes this is a text placeholder.
    format!(
        "{{\"type\":\"glb-stub\",\"source_lines\":{}}}",
        obj.lines().count()
    )
}

fn params_to_json(params: &HashMap<String, f32>) -> String {
    let mut pairs: Vec<String> = params
        .iter()
        .map(|(k, v)| format!("  \"{}\": {:.6}", k, v))
        .collect();
    pairs.sort();
    format!("{{\n{}\n}}", pairs.join(",\n"))
}

fn params_to_csv(params: &HashMap<String, f32>) -> String {
    let mut keys: Vec<&String> = params.keys().collect();
    keys.sort();
    let header = keys
        .iter()
        .map(|k| k.as_str())
        .collect::<Vec<_>>()
        .join(",");
    let values = keys
        .iter()
        .map(|k| format!("{:.6}", params[*k]))
        .collect::<Vec<_>>()
        .join(",");
    format!("{}\n{}\n", header, values)
}

// ── Parameter grid ────────────────────────────────────────────────────────────

/// Generate a Cartesian product of parameter ranges.
///
/// `ranges`: `name → (min, max, steps)`.
/// Returns a vector of parameter maps — one per grid point.
pub fn generate_param_grid(
    ranges: &HashMap<String, (f32, f32, usize)>,
) -> Vec<HashMap<String, f32>> {
    // Sort keys for deterministic ordering.
    let mut keys: Vec<String> = ranges.keys().cloned().collect();
    keys.sort();

    // Compute values per key.
    let values_per_key: Vec<(String, Vec<f32>)> = keys
        .iter()
        .map(|k| {
            let (lo, hi, steps) = ranges[k];
            let vals = if steps <= 1 {
                vec![lo]
            } else {
                (0..steps)
                    .map(|i| lo + (hi - lo) * (i as f32) / ((steps - 1) as f32))
                    .collect()
            };
            (k.clone(), vals)
        })
        .collect();

    // Cartesian product via iterative expansion.
    let mut result: Vec<HashMap<String, f32>> = vec![HashMap::new()];

    for (key, vals) in &values_per_key {
        let mut next = Vec::with_capacity(result.len() * vals.len());
        for existing in &result {
            for &v in vals {
                let mut m = existing.clone();
                m.insert(key.clone(), v);
                next.push(m);
            }
        }
        result = next;
    }

    result
}

// ── Spec builders ─────────────────────────────────────────────────────────────

/// Build `BatchCharacterSpec` list from a param grid.
pub fn specs_from_param_grid(
    grid: &[HashMap<String, f32>],
    format: BatchOutputFormat,
    out_dir: &Path,
) -> Vec<BatchCharacterSpec> {
    grid.iter()
        .enumerate()
        .map(|(i, params)| {
            let id = format!("char_{:04}", i);
            let output_path = out_dir.join(format!("{}.{}", id, format.extension()));
            BatchCharacterSpec {
                id,
                params: params.clone(),
                output_format: format,
                output_path,
            }
        })
        .collect()
}

// ── Summaries ─────────────────────────────────────────────────────────────────

/// Return a human-readable summary line for a `BatchResult`.
pub fn batch_result_summary(result: &BatchResult) -> String {
    format!(
        "Batch: total={} succeeded={} failed={} skipped={}",
        result.total, result.succeeded, result.failed, result.skipped,
    )
}

/// Return a human-readable count/format breakdown for a slice of specs.
pub fn estimate_batch_size(specs: &[BatchCharacterSpec]) -> String {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for spec in specs {
        *counts.entry(spec.output_format.name()).or_insert(0) += 1;
    }
    let mut parts: Vec<String> = counts
        .iter()
        .map(|(fmt, n)| format!("{}×{}", n, fmt))
        .collect();
    parts.sort();
    format!("{} specs ({})", specs.len(), parts.join(", "))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn tmpdir(suffix: &str) -> PathBuf {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        let p = PathBuf::from(format!("/tmp/oxihuman_batch_{}_{}", suffix, nanos));
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    // 1. Two params × two steps → 4 items
    #[test]
    fn param_grid_two_params_two_steps_is_four() {
        let mut ranges = HashMap::new();
        ranges.insert("height".to_string(), (0.0f32, 1.0, 2));
        ranges.insert("weight".to_string(), (0.0f32, 1.0, 2));
        let grid = generate_param_grid(&ranges);
        assert_eq!(grid.len(), 4);
    }

    // 2. One param × three steps → 3 items
    #[test]
    fn param_grid_one_param_three_steps_is_three() {
        let mut ranges = HashMap::new();
        ranges.insert("age".to_string(), (20.0f32, 60.0, 3));
        let grid = generate_param_grid(&ranges);
        assert_eq!(grid.len(), 3);
    }

    // 3. Empty ranges → 1 item (the empty map)
    #[test]
    fn param_grid_empty_ranges_gives_one() {
        let ranges: HashMap<String, (f32, f32, usize)> = HashMap::new();
        let grid = generate_param_grid(&ranges);
        assert_eq!(grid.len(), 1);
        assert!(grid[0].is_empty());
    }

    // 4. specs_from_param_grid count matches
    #[test]
    fn specs_from_param_grid_count_matches() {
        let mut ranges = HashMap::new();
        ranges.insert("height".to_string(), (0.0f32, 1.0, 3));
        ranges.insert("weight".to_string(), (0.0f32, 1.0, 2));
        let grid = generate_param_grid(&ranges);
        let out_dir = Path::new("/tmp");
        let specs = specs_from_param_grid(&grid, BatchOutputFormat::Json, out_dir);
        assert_eq!(specs.len(), grid.len()); // 3 × 2 = 6
    }

    // 5. specs_from_param_grid sets correct extension
    #[test]
    fn specs_from_param_grid_correct_extension() {
        let mut ranges = HashMap::new();
        ranges.insert("x".to_string(), (0.0f32, 1.0, 2));
        let grid = generate_param_grid(&ranges);
        let out_dir = Path::new("/tmp");
        let specs = specs_from_param_grid(&grid, BatchOutputFormat::Stl, out_dir);
        for spec in &specs {
            assert!(
                spec.output_path
                    .extension()
                    .map(|e| e == "stl")
                    .unwrap_or(false),
                "expected .stl extension"
            );
        }
    }

    // 6. batch_result_summary contains the numbers
    #[test]
    fn batch_result_summary_contains_numbers() {
        let result = BatchResult {
            total: 10,
            succeeded: 7,
            failed: 2,
            skipped: 1,
            errors: vec![("id1".into(), "err".into()), ("id2".into(), "err2".into())],
        };
        let s = batch_result_summary(&result);
        assert!(s.contains("10"), "should contain total");
        assert!(s.contains('7'), "should contain succeeded");
        assert!(s.contains('2'), "should contain failed");
        assert!(s.contains('1'), "should contain skipped");
    }

    // 7. run_batch with 3 JSON specs succeeds
    #[test]
    fn run_batch_three_json_specs_succeed() {
        let out_dir = tmpdir("batch_json");
        let mut ranges = HashMap::new();
        ranges.insert("height".to_string(), (0.5f32, 1.0, 3));
        let grid = generate_param_grid(&ranges);
        let specs = specs_from_param_grid(&grid, BatchOutputFormat::Json, &out_dir);
        let cfg = BatchConfig::default();
        let result = run_batch(&specs, &cfg);
        assert_eq!(result.total, 3);
        assert_eq!(result.succeeded, 3);
        assert_eq!(result.failed, 0);
        assert_eq!(result.skipped, 0);
    }

    // 8. run_batch with OBJ format creates files
    #[test]
    fn run_batch_obj_creates_files() {
        let out_dir = tmpdir("batch_obj");
        let specs = vec![BatchCharacterSpec {
            id: "test_char".to_string(),
            params: HashMap::new(),
            output_format: BatchOutputFormat::Obj,
            output_path: out_dir.join("test_char.obj"),
        }];
        let cfg = BatchConfig::default();
        let result = run_batch(&specs, &cfg);
        assert_eq!(result.succeeded, 1);
        assert!(out_dir.join("test_char.obj").exists());
    }

    // 9. skip_existing logic skips existing files
    #[test]
    fn run_batch_skip_existing_skips() {
        let out_dir = tmpdir("batch_skip");
        let path = out_dir.join("char_0000.json");
        // Pre-create the file
        std::fs::write(&path, "{}").unwrap();

        let specs = vec![BatchCharacterSpec {
            id: "char_0000".to_string(),
            params: HashMap::new(),
            output_format: BatchOutputFormat::Json,
            output_path: path,
        }];
        let cfg = BatchConfig {
            skip_existing: true,
            ..Default::default()
        };
        let result = run_batch(&specs, &cfg);
        assert_eq!(result.skipped, 1);
        assert_eq!(result.succeeded, 0);
    }

    // 10. failed spec captured in errors
    #[test]
    fn run_batch_failed_spec_captured() {
        // Output path in a non-existent dir that cannot be created — use a
        // file path where the "parent dir" is an existing regular file.
        let out_dir = tmpdir("batch_fail");
        let blocker = out_dir.join("blocker");
        std::fs::write(&blocker, b"I am a file, not a dir").unwrap();
        // Try to write into blocker/char.json — parent is a file, not a dir
        let bad_path = blocker.join("char.json");
        let specs = vec![BatchCharacterSpec {
            id: "bad-char".to_string(),
            params: HashMap::new(),
            output_format: BatchOutputFormat::Json,
            output_path: bad_path,
        }];
        let cfg = BatchConfig::default();
        let result = run_batch(&specs, &cfg);
        assert_eq!(result.failed, 1);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].0, "bad-char");
    }

    // 11. estimate_batch_size output contains count and format
    #[test]
    fn estimate_batch_size_output_contains_info() {
        let mut ranges = HashMap::new();
        ranges.insert("h".to_string(), (0.0f32, 1.0, 2));
        let grid = generate_param_grid(&ranges);
        let specs = specs_from_param_grid(&grid, BatchOutputFormat::Csv, Path::new("/tmp"));
        let s = estimate_batch_size(&specs);
        assert!(s.contains('2'), "should mention count 2");
        assert!(s.contains("csv"), "should mention format csv");
    }

    // 12. run_batch CSV format produces comma-separated content
    #[test]
    fn run_batch_csv_produces_valid_csv() {
        let out_dir = tmpdir("batch_csv");
        let mut params = HashMap::new();
        params.insert("height".to_string(), 0.75f32);
        params.insert("weight".to_string(), 0.5f32);
        let specs = vec![BatchCharacterSpec {
            id: "csv_char".to_string(),
            params,
            output_format: BatchOutputFormat::Csv,
            output_path: out_dir.join("csv_char.csv"),
        }];
        let cfg = BatchConfig::default();
        let result = run_batch(&specs, &cfg);
        assert_eq!(result.succeeded, 1);
        let content = std::fs::read_to_string(out_dir.join("csv_char.csv")).unwrap();
        assert!(content.contains(','), "CSV should contain commas");
    }

    // 13. Param grid with 1 step returns boundary values
    #[test]
    fn param_grid_one_step_returns_min() {
        let mut ranges = HashMap::new();
        ranges.insert("muscle".to_string(), (0.3f32, 0.9, 1));
        let grid = generate_param_grid(&ranges);
        assert_eq!(grid.len(), 1);
        let val = grid[0]["muscle"];
        assert!((val - 0.3).abs() < 1e-5);
    }
}
