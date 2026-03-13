// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Documentation generator for morph targets.
//!
//! Produces structured documentation describing available targets,
//! their parameters, ranges, and visual effects in multiple output formats.

use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// Body region enum
// ---------------------------------------------------------------------------

/// Body regions that a morph target can affect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BodyRegion {
    Head,
    Face,
    Eyes,
    Nose,
    Mouth,
    Ears,
    Jaw,
    Forehead,
    Neck,
    Shoulders,
    Chest,
    Abdomen,
    Back,
    Pelvis,
    Hips,
    UpperArm,
    Forearm,
    Hand,
    Fingers,
    Thigh,
    Knee,
    Calf,
    Foot,
    Toes,
    FullBody,
}

impl BodyRegion {
    /// Returns the human-readable label for this region.
    pub fn label(self) -> &'static str {
        match self {
            Self::Head => "Head",
            Self::Face => "Face",
            Self::Eyes => "Eyes",
            Self::Nose => "Nose",
            Self::Mouth => "Mouth",
            Self::Ears => "Ears",
            Self::Jaw => "Jaw",
            Self::Forehead => "Forehead",
            Self::Neck => "Neck",
            Self::Shoulders => "Shoulders",
            Self::Chest => "Chest",
            Self::Abdomen => "Abdomen",
            Self::Back => "Back",
            Self::Pelvis => "Pelvis",
            Self::Hips => "Hips",
            Self::UpperArm => "Upper Arm",
            Self::Forearm => "Forearm",
            Self::Hand => "Hand",
            Self::Fingers => "Fingers",
            Self::Thigh => "Thigh",
            Self::Knee => "Knee",
            Self::Calf => "Calf",
            Self::Foot => "Foot",
            Self::Toes => "Toes",
            Self::FullBody => "Full Body",
        }
    }

    /// All variants in order.
    pub fn all() -> &'static [BodyRegion] {
        &[
            Self::Head,
            Self::Face,
            Self::Eyes,
            Self::Nose,
            Self::Mouth,
            Self::Ears,
            Self::Jaw,
            Self::Forehead,
            Self::Neck,
            Self::Shoulders,
            Self::Chest,
            Self::Abdomen,
            Self::Back,
            Self::Pelvis,
            Self::Hips,
            Self::UpperArm,
            Self::Forearm,
            Self::Hand,
            Self::Fingers,
            Self::Thigh,
            Self::Knee,
            Self::Calf,
            Self::Foot,
            Self::Toes,
            Self::FullBody,
        ]
    }
}

impl fmt::Display for BodyRegion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

// ---------------------------------------------------------------------------
// Documentation entry
// ---------------------------------------------------------------------------

/// A single morph-target documentation entry.
#[derive(Debug, Clone)]
pub struct TargetDocEntry {
    pub name: String,
    pub category: String,
    pub description: String,
    pub min_value: f64,
    pub max_value: f64,
    pub default_value: f64,
    pub affected_region: BodyRegion,
    pub vertex_count_affected: usize,
    pub dependencies: Vec<String>,
    pub conflicts: Vec<String>,
    pub tags: Vec<String>,
}

impl TargetDocEntry {
    /// Convenience builder with required fields; optional vecs default to empty.
    pub fn new(
        name: impl Into<String>,
        category: impl Into<String>,
        description: impl Into<String>,
        region: BodyRegion,
    ) -> Self {
        Self {
            name: name.into(),
            category: category.into(),
            description: description.into(),
            min_value: 0.0,
            max_value: 1.0,
            default_value: 0.0,
            affected_region: region,
            vertex_count_affected: 0,
            dependencies: Vec::new(),
            conflicts: Vec::new(),
            tags: Vec::new(),
        }
    }

    /// Set the value range.
    pub fn with_range(mut self, min: f64, max: f64, default: f64) -> Self {
        self.min_value = min;
        self.max_value = max;
        self.default_value = default;
        self
    }

    /// Set the vertex count.
    pub fn with_vertex_count(mut self, count: usize) -> Self {
        self.vertex_count_affected = count;
        self
    }

    /// Add dependencies.
    pub fn with_dependencies(mut self, deps: Vec<String>) -> Self {
        self.dependencies = deps;
        self
    }

    /// Add conflicts.
    pub fn with_conflicts(mut self, conflicts: Vec<String>) -> Self {
        self.conflicts = conflicts;
        self
    }

    /// Add tags.
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

// ---------------------------------------------------------------------------
// Category tree
// ---------------------------------------------------------------------------

/// A hierarchical category for organising morph targets.
#[derive(Debug, Clone)]
pub struct TargetCategory {
    pub name: String,
    pub description: String,
    pub subcategories: Vec<TargetCategory>,
}

impl TargetCategory {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            subcategories: Vec::new(),
        }
    }

    pub fn with_sub(mut self, sub: TargetCategory) -> Self {
        self.subcategories.push(sub);
        self
    }
}

// ---------------------------------------------------------------------------
// Output format
// ---------------------------------------------------------------------------

/// Supported documentation output formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocFormat {
    PlainText,
    Html,
    Json,
    Csv,
}

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------

/// Summary statistics for the target collection.
#[derive(Debug, Clone)]
pub struct DocSummary {
    pub total_targets: usize,
    pub categories: Vec<(String, usize)>,
    pub regions: Vec<(BodyRegion, usize)>,
    pub avg_vertices_affected: f64,
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Documentation generator for morph targets.
///
/// Produces structured documentation describing available targets,
/// their parameters, ranges, and visual effects.
pub struct TargetDocGenerator {
    targets: Vec<TargetDocEntry>,
    categories: Vec<TargetCategory>,
}

impl Default for TargetDocGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl TargetDocGenerator {
    /// Create an empty generator.
    pub fn new() -> Self {
        Self {
            targets: Vec::new(),
            categories: Vec::new(),
        }
    }

    /// Add a target entry.
    pub fn add_target(&mut self, entry: TargetDocEntry) {
        self.targets.push(entry);
    }

    /// Add a top-level category.
    pub fn add_category(&mut self, category: TargetCategory) {
        self.categories.push(category);
    }

    /// Return the number of registered targets.
    pub fn target_count(&self) -> usize {
        self.targets.len()
    }

    // -----------------------------------------------------------------------
    // Auto-categorise
    // -----------------------------------------------------------------------

    /// Auto-categorise targets whose `category` field is empty by inspecting
    /// name patterns (case-insensitive prefix / substring matching).
    pub fn auto_categorize(&mut self) {
        let rules: &[(&[&str], &str)] = &[
            (&["eye", "brow", "pupil", "iris", "lid"], "Eyes"),
            (&["nose", "nostril", "nasal", "bridge"], "Nose"),
            (&["mouth", "lip", "teeth", "tongue", "smile", "frown"], "Mouth"),
            (&["ear", "lobe"], "Ears"),
            (&["jaw", "chin"], "Jaw"),
            (&["forehead", "temple"], "Forehead"),
            (&["face", "cheek"], "Face"),
            (&["head", "skull", "cranium"], "Head"),
            (&["neck", "throat", "adam"], "Neck"),
            (&["shoulder", "clavicle", "scapula"], "Shoulders"),
            (&["chest", "pectoral", "breast", "sternum"], "Chest"),
            (&["abdomen", "belly", "stomach", "navel", "waist"], "Abdomen"),
            (&["back", "spine", "lumbar", "thorac"], "Back"),
            (&["pelvis", "sacrum", "coccyx"], "Pelvis"),
            (&["hip", "gluteal", "buttock"], "Hips"),
            (&["upper_arm", "bicep", "tricep", "deltoid"], "Upper Arm"),
            (&["forearm", "wrist", "ulna", "radius"], "Forearm"),
            (&["hand", "palm", "knuckle"], "Hand"),
            (&["finger", "thumb", "index", "pinky", "ring_finger"], "Fingers"),
            (&["thigh", "quad", "hamstring"], "Thigh"),
            (&["knee", "patella", "kneecap"], "Knee"),
            (&["calf", "shin", "tibia", "fibula"], "Calf"),
            (&["foot", "heel", "arch", "sole", "ankle"], "Foot"),
            (&["toe", "big_toe", "little_toe"], "Toes"),
        ];

        for target in &mut self.targets {
            if !target.category.is_empty() {
                continue;
            }
            let lower = target.name.to_ascii_lowercase();
            let mut matched = false;
            for (keywords, cat) in rules {
                for kw in *keywords {
                    if lower.contains(kw) {
                        target.category = (*cat).to_string();
                        matched = true;
                        break;
                    }
                }
                if matched {
                    break;
                }
            }
            if !matched {
                target.category = "Uncategorised".to_string();
            }
        }

        // Ensure discovered categories are in the category list.
        let existing: std::collections::HashSet<String> =
            self.categories.iter().map(|c| c.name.clone()).collect();
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        for t in &self.targets {
            if !existing.contains(&t.category) && seen.insert(t.category.clone()) {
                self.categories.push(TargetCategory::new(
                    t.category.clone(),
                    format!("Auto-generated category for {}", t.category),
                ));
            }
        }
    }

    // -----------------------------------------------------------------------
    // Query helpers
    // -----------------------------------------------------------------------

    /// Search targets by name, description, or tags (case-insensitive).
    pub fn search(&self, query: &str) -> Vec<&TargetDocEntry> {
        let q = query.to_ascii_lowercase();
        self.targets
            .iter()
            .filter(|t| {
                t.name.to_ascii_lowercase().contains(&q)
                    || t.description.to_ascii_lowercase().contains(&q)
                    || t.tags
                        .iter()
                        .any(|tag| tag.to_ascii_lowercase().contains(&q))
            })
            .collect()
    }

    /// Return targets belonging to a specific category (exact, case-insensitive).
    pub fn by_category(&self, category: &str) -> Vec<&TargetDocEntry> {
        let cat = category.to_ascii_lowercase();
        self.targets
            .iter()
            .filter(|t| t.category.to_ascii_lowercase() == cat)
            .collect()
    }

    /// Return targets affecting a specific body region.
    pub fn by_region(&self, region: BodyRegion) -> Vec<&TargetDocEntry> {
        self.targets
            .iter()
            .filter(|t| t.affected_region == region)
            .collect()
    }

    /// Compute summary statistics.
    pub fn summary(&self) -> DocSummary {
        let total = self.targets.len();

        // Categories
        let mut cat_counts: HashMap<String, usize> = HashMap::new();
        for t in &self.targets {
            *cat_counts.entry(t.category.clone()).or_insert(0) += 1;
        }
        let mut categories: Vec<(String, usize)> = cat_counts.into_iter().collect();
        categories.sort_by(|a, b| b.1.cmp(&a.1));

        // Regions
        let mut reg_counts: HashMap<BodyRegion, usize> = HashMap::new();
        for t in &self.targets {
            *reg_counts.entry(t.affected_region).or_insert(0) += 1;
        }
        let mut regions: Vec<(BodyRegion, usize)> = reg_counts.into_iter().collect();
        regions.sort_by(|a, b| b.1.cmp(&a.1));

        let avg = if total == 0 {
            0.0
        } else {
            self.targets
                .iter()
                .map(|t| t.vertex_count_affected as f64)
                .sum::<f64>()
                / total as f64
        };

        DocSummary {
            total_targets: total,
            categories,
            regions,
            avg_vertices_affected: avg,
        }
    }

    // -----------------------------------------------------------------------
    // Generation dispatch
    // -----------------------------------------------------------------------

    /// Generate documentation in the requested format.
    pub fn generate(&self, format: DocFormat) -> anyhow::Result<String> {
        match format {
            DocFormat::PlainText => self.generate_text(),
            DocFormat::Html => self.generate_html(),
            DocFormat::Json => self.generate_json(),
            DocFormat::Csv => self.generate_csv(),
        }
    }

    // -----------------------------------------------------------------------
    // Plain text
    // -----------------------------------------------------------------------

    fn generate_text(&self) -> anyhow::Result<String> {
        let mut out = String::with_capacity(4096);
        let summary = self.summary();

        out.push_str("==========================================================\n");
        out.push_str("  Morph Target Documentation\n");
        out.push_str("==========================================================\n\n");

        // Summary section
        out.push_str(&format!("Total targets: {}\n", summary.total_targets));
        out.push_str(&format!(
            "Average vertices affected: {:.1}\n\n",
            summary.avg_vertices_affected
        ));

        // Categories overview
        out.push_str("Categories:\n");
        for (cat, count) in &summary.categories {
            out.push_str(&format!("  - {} ({})\n", cat, count));
        }
        out.push('\n');

        // Targets grouped by category
        let grouped = self.group_by_category();
        for (cat, targets) in &grouped {
            out.push_str("----------------------------------------------------------\n");
            out.push_str(&format!("  Category: {}\n", cat));
            out.push_str("----------------------------------------------------------\n\n");

            for t in targets {
                out.push_str(&format!("  Name: {}\n", t.name));
                out.push_str(&format!("  Description: {}\n", t.description));
                out.push_str(&format!(
                    "  Range: [{:.2}, {:.2}]  Default: {:.2}\n",
                    t.min_value, t.max_value, t.default_value
                ));
                out.push_str(&format!("  Region: {}\n", t.affected_region));
                out.push_str(&format!("  Vertices: {}\n", t.vertex_count_affected));
                if !t.dependencies.is_empty() {
                    out.push_str(&format!("  Dependencies: {}\n", t.dependencies.join(", ")));
                }
                if !t.conflicts.is_empty() {
                    out.push_str(&format!("  Conflicts: {}\n", t.conflicts.join(", ")));
                }
                if !t.tags.is_empty() {
                    out.push_str(&format!("  Tags: {}\n", t.tags.join(", ")));
                }
                out.push('\n');
            }
        }

        Ok(out)
    }

    // -----------------------------------------------------------------------
    // HTML
    // -----------------------------------------------------------------------

    fn generate_html(&self) -> anyhow::Result<String> {
        let summary = self.summary();
        let grouped = self.group_by_category();

        let mut html = String::with_capacity(8192);

        // Doctype + head
        html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        html.push_str("<meta charset=\"utf-8\">\n");
        html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
        html.push_str("<title>Morph Target Documentation</title>\n");

        // Inline CSS
        html.push_str("<style>\n");
        html.push_str(INLINE_CSS);
        html.push_str("</style>\n");

        html.push_str("</head>\n<body>\n");

        // Header
        html.push_str("<header><h1>Morph Target Documentation</h1></header>\n");

        // Search box
        html.push_str("<div class=\"search-box\">\n");
        html.push_str(
            "<input type=\"text\" id=\"searchInput\" \
             placeholder=\"Search targets...\" onkeyup=\"filterTargets()\">\n",
        );
        html.push_str("</div>\n");

        // Summary
        html.push_str("<section class=\"summary\">\n");
        html.push_str("<h2>Summary</h2>\n");
        html.push_str(&format!(
            "<p>Total targets: <strong>{}</strong></p>\n",
            summary.total_targets
        ));
        html.push_str(&format!(
            "<p>Average vertices affected: <strong>{:.1}</strong></p>\n",
            summary.avg_vertices_affected
        ));
        html.push_str("</section>\n");

        // Table of contents
        html.push_str("<nav class=\"toc\">\n");
        html.push_str("<h2>Table of Contents</h2>\n<ul>\n");
        for (cat, targets) in &grouped {
            let anchor = slug(cat);
            html.push_str(&format!(
                "<li><a href=\"#{}\">{}</a> ({})</li>\n",
                html_escape(&anchor),
                html_escape(cat),
                targets.len()
            ));
        }
        html.push_str("</ul>\n</nav>\n");

        // Targets by category
        html.push_str("<main>\n");
        for (cat, targets) in &grouped {
            let anchor = slug(cat);
            html.push_str(&format!(
                "<section class=\"category\" id=\"{}\">\n",
                html_escape(&anchor)
            ));
            html.push_str(&format!("<h2>{}</h2>\n", html_escape(cat)));

            html.push_str("<table>\n<thead><tr>\
                <th>Name</th><th>Description</th><th>Range</th>\
                <th>Default</th><th>Region</th><th>Vertices</th>\
                <th>Tags</th>\
                </tr></thead>\n<tbody>\n");

            for t in targets {
                html.push_str("<tr class=\"target-row\">\n");
                html.push_str(&format!("<td class=\"target-name\">{}</td>\n", html_escape(&t.name)));
                html.push_str(&format!("<td>{}</td>\n", html_escape(&t.description)));
                html.push_str(&format!(
                    "<td>[{:.2}, {:.2}]</td>\n",
                    t.min_value, t.max_value
                ));
                html.push_str(&format!("<td>{:.2}</td>\n", t.default_value));
                html.push_str(&format!("<td>{}</td>\n", html_escape(t.affected_region.label())));
                html.push_str(&format!("<td>{}</td>\n", t.vertex_count_affected));
                html.push_str(&format!(
                    "<td>{}</td>\n",
                    html_escape(&t.tags.join(", "))
                ));
                html.push_str("</tr>\n");

                // Detail row for dependencies / conflicts
                if !t.dependencies.is_empty() || !t.conflicts.is_empty() {
                    html.push_str("<tr class=\"detail-row\">\n<td colspan=\"7\">\n");
                    if !t.dependencies.is_empty() {
                        html.push_str(&format!(
                            "<em>Dependencies:</em> {}<br>\n",
                            html_escape(&t.dependencies.join(", "))
                        ));
                    }
                    if !t.conflicts.is_empty() {
                        html.push_str(&format!(
                            "<em>Conflicts:</em> {}\n",
                            html_escape(&t.conflicts.join(", "))
                        ));
                    }
                    html.push_str("</td>\n</tr>\n");
                }
            }

            html.push_str("</tbody>\n</table>\n</section>\n");
        }
        html.push_str("</main>\n");

        // Inline JS for search
        html.push_str("<script>\n");
        html.push_str(INLINE_JS);
        html.push_str("</script>\n");

        html.push_str("</body>\n</html>\n");

        Ok(html)
    }

    // -----------------------------------------------------------------------
    // JSON
    // -----------------------------------------------------------------------

    fn generate_json(&self) -> anyhow::Result<String> {
        // Build manually to avoid pulling in serde Serialize on every type.
        let mut out = String::with_capacity(4096);
        out.push_str("{\n");

        // Summary
        let summary = self.summary();
        out.push_str(&format!(
            "  \"total_targets\": {},\n",
            summary.total_targets
        ));
        out.push_str(&format!(
            "  \"avg_vertices_affected\": {:.1},\n",
            summary.avg_vertices_affected
        ));

        // Categories summary
        out.push_str("  \"category_counts\": {\n");
        for (i, (cat, count)) in summary.categories.iter().enumerate() {
            let comma = if i + 1 < summary.categories.len() {
                ","
            } else {
                ""
            };
            out.push_str(&format!(
                "    {}: {}{}",
                json_string(cat),
                count,
                comma
            ));
            out.push('\n');
        }
        out.push_str("  },\n");

        // Targets array
        out.push_str("  \"targets\": [\n");
        for (i, t) in self.targets.iter().enumerate() {
            out.push_str("    {\n");
            out.push_str(&format!("      \"name\": {},\n", json_string(&t.name)));
            out.push_str(&format!(
                "      \"category\": {},\n",
                json_string(&t.category)
            ));
            out.push_str(&format!(
                "      \"description\": {},\n",
                json_string(&t.description)
            ));
            out.push_str(&format!("      \"min_value\": {},\n", format_f64(t.min_value)));
            out.push_str(&format!("      \"max_value\": {},\n", format_f64(t.max_value)));
            out.push_str(&format!(
                "      \"default_value\": {},\n",
                format_f64(t.default_value)
            ));
            out.push_str(&format!(
                "      \"affected_region\": {},\n",
                json_string(t.affected_region.label())
            ));
            out.push_str(&format!(
                "      \"vertex_count_affected\": {},\n",
                t.vertex_count_affected
            ));
            out.push_str(&format!(
                "      \"dependencies\": [{}],\n",
                t.dependencies
                    .iter()
                    .map(|d| json_string(d))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            out.push_str(&format!(
                "      \"conflicts\": [{}],\n",
                t.conflicts
                    .iter()
                    .map(|c| json_string(c))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            out.push_str(&format!(
                "      \"tags\": [{}]\n",
                t.tags
                    .iter()
                    .map(|tg| json_string(tg))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));

            let comma = if i + 1 < self.targets.len() {
                ","
            } else {
                ""
            };
            out.push_str(&format!("    }}{}\n", comma));
        }
        out.push_str("  ]\n");

        out.push_str("}\n");
        Ok(out)
    }

    // -----------------------------------------------------------------------
    // CSV
    // -----------------------------------------------------------------------

    fn generate_csv(&self) -> anyhow::Result<String> {
        let mut out = String::with_capacity(4096);
        out.push_str(
            "name,category,description,min_value,max_value,default_value,\
             affected_region,vertex_count_affected,dependencies,conflicts,tags\n",
        );

        for t in &self.targets {
            out.push_str(&csv_field(&t.name));
            out.push(',');
            out.push_str(&csv_field(&t.category));
            out.push(',');
            out.push_str(&csv_field(&t.description));
            out.push(',');
            out.push_str(&format_f64(t.min_value));
            out.push(',');
            out.push_str(&format_f64(t.max_value));
            out.push(',');
            out.push_str(&format_f64(t.default_value));
            out.push(',');
            out.push_str(&csv_field(t.affected_region.label()));
            out.push(',');
            out.push_str(&t.vertex_count_affected.to_string());
            out.push(',');
            out.push_str(&csv_field(&t.dependencies.join("; ")));
            out.push(',');
            out.push_str(&csv_field(&t.conflicts.join("; ")));
            out.push(',');
            out.push_str(&csv_field(&t.tags.join("; ")));
            out.push('\n');
        }

        Ok(out)
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Group targets by category, preserving insertion order within each group.
    fn group_by_category(&self) -> Vec<(String, Vec<&TargetDocEntry>)> {
        let mut map: HashMap<String, Vec<&TargetDocEntry>> = HashMap::new();
        let mut order: Vec<String> = Vec::new();

        for t in &self.targets {
            if !map.contains_key(&t.category) {
                order.push(t.category.clone());
            }
            map.entry(t.category.clone()).or_default().push(t);
        }

        order
            .into_iter()
            .filter_map(|cat| {
                let entries = map.remove(&cat)?;
                Some((cat, entries))
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Static CSS / JS
// ---------------------------------------------------------------------------

const INLINE_CSS: &str = r#"
* { box-sizing: border-box; margin: 0; padding: 0; }
body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    line-height: 1.6; color: #333; max-width: 1100px; margin: 0 auto; padding: 1rem;
    background: #fafafa;
}
header { text-align: center; padding: 1.5rem 0; border-bottom: 2px solid #4a90d9; margin-bottom: 1rem; }
header h1 { color: #2c3e50; font-size: 1.8rem; }
h2 { color: #34495e; margin: 1rem 0 0.5rem; font-size: 1.3rem; }
.search-box { margin: 1rem 0; text-align: center; }
.search-box input {
    width: 60%; padding: 0.5rem 1rem; font-size: 1rem;
    border: 1px solid #ccc; border-radius: 4px;
}
.search-box input:focus { outline: none; border-color: #4a90d9; box-shadow: 0 0 4px rgba(74,144,217,0.3); }
.summary { background: #fff; padding: 1rem; border-radius: 6px; margin-bottom: 1rem; border: 1px solid #e0e0e0; }
.toc { background: #fff; padding: 1rem; border-radius: 6px; margin-bottom: 1rem; border: 1px solid #e0e0e0; }
.toc ul { list-style: none; padding-left: 1rem; }
.toc li { margin: 0.25rem 0; }
.toc a { color: #4a90d9; text-decoration: none; }
.toc a:hover { text-decoration: underline; }
table { width: 100%; border-collapse: collapse; margin-bottom: 1rem; background: #fff; border: 1px solid #ddd; }
th, td { padding: 0.5rem 0.75rem; text-align: left; border-bottom: 1px solid #eee; font-size: 0.9rem; }
th { background: #4a90d9; color: #fff; font-weight: 600; }
tr:hover { background: #f5f8fd; }
.detail-row td { background: #f9f9f9; font-size: 0.85rem; color: #555; }
.category { margin-bottom: 2rem; }
"#;

const INLINE_JS: &str = r#"
function filterTargets() {
    var input = document.getElementById('searchInput');
    var filter = input.value.toLowerCase();
    var rows = document.querySelectorAll('.target-row');
    for (var i = 0; i < rows.length; i++) {
        var nameCell = rows[i].querySelector('.target-name');
        var text = rows[i].textContent.toLowerCase();
        if (text.indexOf(filter) > -1) {
            rows[i].style.display = '';
            var next = rows[i].nextElementSibling;
            if (next && next.classList.contains('detail-row')) {
                next.style.display = '';
            }
        } else {
            rows[i].style.display = 'none';
            var next = rows[i].nextElementSibling;
            if (next && next.classList.contains('detail-row')) {
                next.style.display = 'none';
            }
        }
    }
}
"#;

// ---------------------------------------------------------------------------
// Free-standing helpers
// ---------------------------------------------------------------------------

/// Minimal HTML entity escaping.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Create a URL-safe slug from a string.
fn slug(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

/// Wrap a value in JSON double-quotes, escaping as needed.
fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Format an f64 with no trailing zeros (but at least one decimal).
fn format_f64(v: f64) -> String {
    if v == v.floor() {
        format!("{:.1}", v)
    } else {
        format!("{}", v)
    }
}

/// Quote a CSV field if it contains commas, quotes, or newlines.
fn csv_field(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        let escaped = s.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        s.to_string()
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_generator() -> TargetDocGenerator {
        let mut gen = TargetDocGenerator::new();
        gen.add_target(
            TargetDocEntry::new("eye_open_left", "Eyes", "Opens the left eye", BodyRegion::Eyes)
                .with_range(0.0, 1.0, 0.0)
                .with_vertex_count(120)
                .with_tags(vec!["facial".into(), "expression".into()]),
        );
        gen.add_target(
            TargetDocEntry::new("eye_open_right", "Eyes", "Opens the right eye", BodyRegion::Eyes)
                .with_range(0.0, 1.0, 0.0)
                .with_vertex_count(118)
                .with_dependencies(vec!["eye_open_left".into()])
                .with_tags(vec!["facial".into(), "expression".into()]),
        );
        gen.add_target(
            TargetDocEntry::new(
                "nose_width",
                "Nose",
                "Adjusts nose width",
                BodyRegion::Nose,
            )
            .with_range(-1.0, 1.0, 0.0)
            .with_vertex_count(85)
            .with_conflicts(vec!["nose_pinch".into()]),
        );
        gen.add_target(
            TargetDocEntry::new(
                "chest_size",
                "Body",
                "Adjusts overall chest size",
                BodyRegion::Chest,
            )
            .with_range(0.0, 2.0, 1.0)
            .with_vertex_count(450),
        );
        gen.add_category(TargetCategory::new("Eyes", "Eye-related morph targets"));
        gen.add_category(TargetCategory::new("Nose", "Nose-related morph targets"));
        gen.add_category(TargetCategory::new("Body", "Body morph targets"));
        gen
    }

    #[test]
    fn test_basic_creation() {
        let gen = TargetDocGenerator::new();
        assert_eq!(gen.target_count(), 0);
    }

    #[test]
    fn test_add_target_and_count() {
        let gen = sample_generator();
        assert_eq!(gen.target_count(), 4);
    }

    #[test]
    fn test_summary() {
        let gen = sample_generator();
        let summary = gen.summary();
        assert_eq!(summary.total_targets, 4);
        assert!(!summary.categories.is_empty());
        assert!(!summary.regions.is_empty());
        // avg = (120+118+85+450)/4 = 193.25
        assert!((summary.avg_vertices_affected - 193.25).abs() < 0.01);
    }

    #[test]
    fn test_search_by_name() {
        let gen = sample_generator();
        let results = gen.search("eye");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_by_tag() {
        let gen = sample_generator();
        let results = gen.search("expression");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_by_description() {
        let gen = sample_generator();
        let results = gen.search("width");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "nose_width");
    }

    #[test]
    fn test_search_no_match() {
        let gen = sample_generator();
        let results = gen.search("zzz_nonexistent_zzz");
        assert!(results.is_empty());
    }

    #[test]
    fn test_by_category() {
        let gen = sample_generator();
        let eyes = gen.by_category("Eyes");
        assert_eq!(eyes.len(), 2);
        let body = gen.by_category("Body");
        assert_eq!(body.len(), 1);
    }

    #[test]
    fn test_by_category_case_insensitive() {
        let gen = sample_generator();
        let eyes = gen.by_category("eyes");
        assert_eq!(eyes.len(), 2);
    }

    #[test]
    fn test_by_region() {
        let gen = sample_generator();
        let nose = gen.by_region(BodyRegion::Nose);
        assert_eq!(nose.len(), 1);
        let eyes = gen.by_region(BodyRegion::Eyes);
        assert_eq!(eyes.len(), 2);
    }

    #[test]
    fn test_auto_categorize() {
        let mut gen = TargetDocGenerator::new();
        gen.add_target(TargetDocEntry::new(
            "eye_squint",
            "",
            "Squints the eyes",
            BodyRegion::Eyes,
        ));
        gen.add_target(TargetDocEntry::new(
            "lip_curl",
            "",
            "Curls the lip",
            BodyRegion::Mouth,
        ));
        gen.add_target(TargetDocEntry::new(
            "some_random_target",
            "",
            "A mystery target",
            BodyRegion::FullBody,
        ));

        gen.auto_categorize();

        assert_eq!(gen.targets[0].category, "Eyes");
        assert_eq!(gen.targets[1].category, "Mouth");
        assert_eq!(gen.targets[2].category, "Uncategorised");
        // Auto categories should be added
        assert!(gen.categories.iter().any(|c| c.name == "Eyes"));
        assert!(gen.categories.iter().any(|c| c.name == "Mouth"));
        assert!(gen.categories.iter().any(|c| c.name == "Uncategorised"));
    }

    #[test]
    fn test_auto_categorize_preserves_existing() {
        let mut gen = TargetDocGenerator::new();
        gen.add_target(TargetDocEntry::new(
            "eye_widen",
            "Custom Category",
            "Widens eyes",
            BodyRegion::Eyes,
        ));
        gen.auto_categorize();
        // Should not overwrite the already-set category
        assert_eq!(gen.targets[0].category, "Custom Category");
    }

    // -----------------------------------------------------------------------
    // Format tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_generate_plain_text() {
        let gen = sample_generator();
        let text = gen.generate(DocFormat::PlainText).expect("plain text generation failed");
        assert!(text.contains("Morph Target Documentation"));
        assert!(text.contains("eye_open_left"));
        assert!(text.contains("nose_width"));
        assert!(text.contains("chest_size"));
        assert!(text.contains("Total targets: 4"));
        assert!(text.contains("Dependencies: eye_open_left"));
        assert!(text.contains("Conflicts: nose_pinch"));
    }

    #[test]
    fn test_generate_html() {
        let gen = sample_generator();
        let html = gen.generate(DocFormat::Html).expect("html generation failed");
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>Morph Target Documentation</title>"));
        assert!(html.contains("eye_open_left"));
        assert!(html.contains("filterTargets"));
        // Check inline CSS is present
        assert!(html.contains("font-family"));
        // Table of contents links
        assert!(html.contains("<nav class=\"toc\">"));
        assert!(html.contains("</html>"));
    }

    #[test]
    fn test_generate_html_escapes_special_chars() {
        let mut gen = TargetDocGenerator::new();
        gen.add_target(TargetDocEntry::new(
            "test<script>",
            "Cat&egory",
            "Desc with \"quotes\"",
            BodyRegion::Face,
        ));
        let html = gen.generate(DocFormat::Html).expect("html generation failed");
        assert!(html.contains("test&lt;script&gt;"));
        assert!(html.contains("Cat&amp;egory"));
        assert!(html.contains("Desc with &quot;quotes&quot;"));
    }

    #[test]
    fn test_generate_json() {
        let gen = sample_generator();
        let json_str = gen.generate(DocFormat::Json).expect("json generation failed");
        assert!(json_str.contains("\"total_targets\": 4"));
        assert!(json_str.contains("\"eye_open_left\""));
        assert!(json_str.contains("\"nose_width\""));
        assert!(json_str.contains("\"avg_vertices_affected\""));

        // Validate it is parseable JSON
        let parsed: serde_json::Value =
            serde_json::from_str(&json_str).expect("generated JSON is not valid");
        let targets = parsed["targets"].as_array().expect("targets should be an array");
        assert_eq!(targets.len(), 4);
    }

    #[test]
    fn test_generate_json_escapes() {
        let mut gen = TargetDocGenerator::new();
        gen.add_target(TargetDocEntry::new(
            "test\"name",
            "cat\\slash",
            "line\nnewline",
            BodyRegion::Head,
        ));
        let json_str = gen.generate(DocFormat::Json).expect("json generation failed");
        let parsed: serde_json::Value =
            serde_json::from_str(&json_str).expect("escaped JSON must be valid");
        let name = parsed["targets"][0]["name"].as_str().expect("name must be string");
        assert_eq!(name, "test\"name");
    }

    #[test]
    fn test_generate_csv() {
        let gen = sample_generator();
        let csv_str = gen.generate(DocFormat::Csv).expect("csv generation failed");
        let lines: Vec<&str> = csv_str.lines().collect();
        // header + 4 data rows
        assert_eq!(lines.len(), 5);
        assert!(lines[0].starts_with("name,category,"));
        assert!(csv_str.contains("eye_open_left"));
        assert!(csv_str.contains("nose_width"));
    }

    #[test]
    fn test_csv_quoting() {
        let mut gen = TargetDocGenerator::new();
        gen.add_target(TargetDocEntry::new(
            "field,with,commas",
            "cat",
            "desc with \"quotes\"",
            BodyRegion::Face,
        ));
        let csv_str = gen.generate(DocFormat::Csv).expect("csv generation failed");
        // Fields with commas or quotes must be quoted
        assert!(csv_str.contains("\"field,with,commas\""));
        assert!(csv_str.contains("\"desc with \"\"quotes\"\"\""));
    }

    #[test]
    fn test_empty_generator_all_formats() {
        let gen = TargetDocGenerator::new();
        for fmt in &[DocFormat::PlainText, DocFormat::Html, DocFormat::Json, DocFormat::Csv] {
            let result = gen.generate(*fmt);
            assert!(result.is_ok(), "format {:?} should succeed on empty generator", fmt);
        }
    }

    #[test]
    fn test_body_region_display() {
        assert_eq!(BodyRegion::UpperArm.to_string(), "Upper Arm");
        assert_eq!(BodyRegion::FullBody.to_string(), "Full Body");
        assert_eq!(BodyRegion::Eyes.to_string(), "Eyes");
    }

    #[test]
    fn test_body_region_all_variants() {
        let all = BodyRegion::all();
        assert_eq!(all.len(), 25);
    }

    #[test]
    fn test_target_doc_entry_builder() {
        let entry = TargetDocEntry::new("test", "cat", "desc", BodyRegion::Head)
            .with_range(-1.0, 1.0, 0.5)
            .with_vertex_count(200)
            .with_dependencies(vec!["dep_a".into()])
            .with_conflicts(vec!["conf_b".into()])
            .with_tags(vec!["tag1".into()]);
        assert_eq!(entry.min_value, -1.0);
        assert_eq!(entry.max_value, 1.0);
        assert_eq!(entry.default_value, 0.5);
        assert_eq!(entry.vertex_count_affected, 200);
        assert_eq!(entry.dependencies, vec!["dep_a"]);
        assert_eq!(entry.conflicts, vec!["conf_b"]);
        assert_eq!(entry.tags, vec!["tag1"]);
    }

    #[test]
    fn test_target_category_with_sub() {
        let cat = TargetCategory::new("Face", "Facial targets")
            .with_sub(TargetCategory::new("Eyes", "Eye targets"))
            .with_sub(TargetCategory::new("Mouth", "Mouth targets"));
        assert_eq!(cat.subcategories.len(), 2);
        assert_eq!(cat.subcategories[0].name, "Eyes");
    }

    #[test]
    fn test_group_by_category_order() {
        let gen = sample_generator();
        let grouped = gen.group_by_category();
        // First target has category "Eyes", second group "Nose", third "Body"
        assert_eq!(grouped[0].0, "Eyes");
        assert_eq!(grouped[1].0, "Nose");
        assert_eq!(grouped[2].0, "Body");
    }

    #[test]
    fn test_summary_empty() {
        let gen = TargetDocGenerator::new();
        let summary = gen.summary();
        assert_eq!(summary.total_targets, 0);
        assert!(summary.categories.is_empty());
        assert!(summary.regions.is_empty());
        assert_eq!(summary.avg_vertices_affected, 0.0);
    }

    // -----------------------------------------------------------------------
    // Helper function tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_html_escape_fn() {
        assert_eq!(html_escape("<b>hi</b>"), "&lt;b&gt;hi&lt;/b&gt;");
        assert_eq!(html_escape("a&b"), "a&amp;b");
        assert_eq!(html_escape("\"quote\""), "&quot;quote&quot;");
    }

    #[test]
    fn test_slug() {
        assert_eq!(slug("Upper Arm"), "upper-arm");
        assert_eq!(slug("Eyes"), "eyes");
        assert_eq!(slug("Full Body"), "full-body");
    }

    #[test]
    fn test_json_string_fn() {
        assert_eq!(json_string("hello"), "\"hello\"");
        assert_eq!(json_string("say \"hi\""), "\"say \\\"hi\\\"\"");
        assert_eq!(json_string("back\\slash"), "\"back\\\\slash\"");
        assert_eq!(json_string("line\nbreak"), "\"line\\nbreak\"");
    }

    #[test]
    fn test_csv_field_fn() {
        assert_eq!(csv_field("simple"), "simple");
        assert_eq!(csv_field("has,comma"), "\"has,comma\"");
        assert_eq!(csv_field("has\"quote"), "\"has\"\"quote\"");
    }

    #[test]
    fn test_format_f64_fn() {
        assert_eq!(format_f64(1.0), "1.0");
        assert_eq!(format_f64(0.0), "0.0");
        assert_eq!(format_f64(0.5), "0.5");
        assert_eq!(format_f64(-1.0), "-1.0");
    }
}
