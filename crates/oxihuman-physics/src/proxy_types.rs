// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Proxy primitive types and JSON serialization / deserialization.

/// A capsule collision primitive (line segment + radius).
#[derive(Debug, Clone, PartialEq)]
pub struct CapsuleProxy {
    /// Bottom center of the capsule.
    pub center_a: [f32; 3],
    /// Top center of the capsule.
    pub center_b: [f32; 3],
    /// Radius of the capsule.
    pub radius: f32,
    /// Label (e.g. "torso", "head", "arm_l").
    pub label: String,
}

impl CapsuleProxy {
    pub fn new(center_a: [f32; 3], center_b: [f32; 3], radius: f32, label: &str) -> Self {
        CapsuleProxy {
            center_a,
            center_b,
            radius,
            label: label.to_string(),
        }
    }
}

/// A sphere collision primitive.
#[derive(Debug, Clone, PartialEq)]
pub struct SphereProxy {
    pub center: [f32; 3],
    pub radius: f32,
    pub label: String,
}

impl SphereProxy {
    pub fn new(center: [f32; 3], radius: f32, label: &str) -> Self {
        SphereProxy {
            center,
            radius,
            label: label.to_string(),
        }
    }
}

/// A box (AABB) collision primitive.
#[derive(Debug, Clone, PartialEq)]
pub struct BoxProxy {
    pub center: [f32; 3],
    pub half_extents: [f32; 3],
    pub label: String,
}

/// Complete set of collision proxies for a humanoid body.
#[derive(Debug, Default, Clone)]
pub struct BodyProxies {
    pub capsules: Vec<CapsuleProxy>,
    pub spheres: Vec<SphereProxy>,
    pub boxes: Vec<BoxProxy>,
}

impl BodyProxies {
    pub fn new() -> Self {
        BodyProxies::default()
    }

    pub fn total_count(&self) -> usize {
        self.capsules.len() + self.spheres.len() + self.boxes.len()
    }
}

// ── JSON serialization ────────────────────────────────────────────────────────

/// Serialize a `[f32; 3]` to a compact JSON array string.
fn fmt_vec3(v: [f32; 3]) -> String {
    format!("[{},{},{}]", v[0], v[1], v[2])
}

/// Escape a string for safe embedding in JSON (handles `"` and `\`).
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

/// Serialize [`BodyProxies`] to a JSON string.
///
/// Output format:
/// ```json
/// {
///   "capsules": [...],
///   "spheres": [...],
///   "boxes": [...]
/// }
/// ```
pub fn proxies_to_json(proxies: &BodyProxies) -> String {
    let mut out = String::with_capacity(512);
    out.push_str("{\n  \"capsules\": [\n");

    for (i, c) in proxies.capsules.iter().enumerate() {
        let comma = if i + 1 < proxies.capsules.len() {
            ","
        } else {
            ""
        };
        out.push_str(&format!(
            "    {{\"label\":\"{}\",\"center_a\":{},\"center_b\":{},\"radius\":{}}}{}\n",
            json_escape(&c.label),
            fmt_vec3(c.center_a),
            fmt_vec3(c.center_b),
            c.radius,
            comma
        ));
    }

    out.push_str("  ],\n  \"spheres\": [\n");

    for (i, s) in proxies.spheres.iter().enumerate() {
        let comma = if i + 1 < proxies.spheres.len() {
            ","
        } else {
            ""
        };
        out.push_str(&format!(
            "    {{\"label\":\"{}\",\"center\":{},\"radius\":{}}}{}\n",
            json_escape(&s.label),
            fmt_vec3(s.center),
            s.radius,
            comma
        ));
    }

    out.push_str("  ],\n  \"boxes\": [\n");

    for (i, b) in proxies.boxes.iter().enumerate() {
        let comma = if i + 1 < proxies.boxes.len() { "," } else { "" };
        out.push_str(&format!(
            "    {{\"label\":\"{}\",\"center\":{},\"half_extents\":{}}}{}\n",
            json_escape(&b.label),
            fmt_vec3(b.center),
            fmt_vec3(b.half_extents),
            comma
        ));
    }

    out.push_str("  ]\n}");
    out
}

/// Deserialize [`BodyProxies`] from a JSON string produced by [`proxies_to_json`].
///
/// Uses `serde_json` for reliable parsing.
pub fn proxies_from_json(s: &str) -> anyhow::Result<BodyProxies> {
    let v: serde_json::Value = serde_json::from_str(s)?;

    let parse_vec3 = |arr: &serde_json::Value| -> anyhow::Result<[f32; 3]> {
        let a = arr
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("expected array for vec3"))?;
        if a.len() != 3 {
            anyhow::bail!("vec3 must have 3 elements, got {}", a.len());
        }
        Ok([
            a[0].as_f64()
                .ok_or_else(|| anyhow::anyhow!("expected float"))? as f32,
            a[1].as_f64()
                .ok_or_else(|| anyhow::anyhow!("expected float"))? as f32,
            a[2].as_f64()
                .ok_or_else(|| anyhow::anyhow!("expected float"))? as f32,
        ])
    };

    let get_str = |obj: &serde_json::Value, key: &str| -> anyhow::Result<String> {
        obj[key]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("missing string field '{key}'"))
    };

    let get_f32 = |obj: &serde_json::Value, key: &str| -> anyhow::Result<f32> {
        obj[key]
            .as_f64()
            .map(|f| f as f32)
            .ok_or_else(|| anyhow::anyhow!("missing float field '{key}'"))
    };

    let mut proxies = BodyProxies::new();

    if let Some(caps) = v["capsules"].as_array() {
        for c in caps {
            proxies.capsules.push(CapsuleProxy {
                label: get_str(c, "label")?,
                center_a: parse_vec3(&c["center_a"])?,
                center_b: parse_vec3(&c["center_b"])?,
                radius: get_f32(c, "radius")?,
            });
        }
    }

    if let Some(spheres) = v["spheres"].as_array() {
        for s in spheres {
            proxies.spheres.push(SphereProxy {
                label: get_str(s, "label")?,
                center: parse_vec3(&s["center"])?,
                radius: get_f32(s, "radius")?,
            });
        }
    }

    if let Some(boxes) = v["boxes"].as_array() {
        for b in boxes {
            proxies.boxes.push(BoxProxy {
                label: get_str(b, "label")?,
                center: parse_vec3(&b["center"])?,
                half_extents: parse_vec3(&b["half_extents"])?,
            });
        }
    }

    Ok(proxies)
}
