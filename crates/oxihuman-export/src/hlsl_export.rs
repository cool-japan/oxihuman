// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! HLSL shader source export stub.

/// HLSL shader target profile.
#[derive(Clone, PartialEq)]
pub enum HlslProfile {
    Vs50,
    Ps50,
    Cs50,
    Gs50,
    Hs50,
    Ds50,
}

impl HlslProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            HlslProfile::Vs50 => "vs_5_0",
            HlslProfile::Ps50 => "ps_5_0",
            HlslProfile::Cs50 => "cs_5_0",
            HlslProfile::Gs50 => "gs_5_0",
            HlslProfile::Hs50 => "hs_5_0",
            HlslProfile::Ds50 => "ds_5_0",
        }
    }
}

/// A single HLSL shader.
pub struct HlslShader {
    pub profile: HlslProfile,
    pub entry_point: String,
    pub source: String,
    pub defines: Vec<(String, String)>,
}

/// An HLSL export containing multiple shaders.
pub struct HlslExport {
    pub shaders: Vec<HlslShader>,
    pub namespace: String,
}

/// Create a new HLSL export.
pub fn new_hlsl_export(namespace: &str) -> HlslExport {
    HlslExport {
        shaders: Vec::new(),
        namespace: namespace.to_string(),
    }
}

/// Add an HLSL shader.
pub fn add_hlsl_shader(exp: &mut HlslExport, profile: HlslProfile, entry: &str, source: &str) {
    exp.shaders.push(HlslShader {
        profile,
        entry_point: entry.to_string(),
        source: source.to_string(),
        defines: Vec::new(),
    });
}

/// Add a preprocessor define to the last shader.
pub fn add_hlsl_define(exp: &mut HlslExport, key: &str, value: &str) -> bool {
    if let Some(s) = exp.shaders.last_mut() {
        s.defines.push((key.to_string(), value.to_string()));
        true
    } else {
        false
    }
}

/// Shader count.
pub fn hlsl_shader_count(exp: &HlslExport) -> usize {
    exp.shaders.len()
}

/// Find a shader by profile.
pub fn find_hlsl_shader<'a>(exp: &'a HlslExport, profile: &HlslProfile) -> Option<&'a HlslShader> {
    exp.shaders.iter().find(|s| &s.profile == profile)
}

/// Render a shader source with defines prepended.
pub fn render_hlsl_shader(shader: &HlslShader) -> String {
    let mut s = String::new();
    for (k, v) in &shader.defines {
        s.push_str(&format!("#define {k} {v}\n"));
    }
    s.push_str(&shader.source);
    s
}

/// Validate (at least one shader).
pub fn validate_hlsl_export(exp: &HlslExport) -> bool {
    !exp.shaders.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        let exp = new_hlsl_export("Game");
        assert_eq!(hlsl_shader_count(&exp), 0 /* empty */);
    }

    #[test]
    fn add_shader_increments() {
        let mut exp = new_hlsl_export("G");
        add_hlsl_shader(
            &mut exp,
            HlslProfile::Vs50,
            "VSMain",
            "float4 VSMain():SV_Position{return 0;}",
        );
        assert_eq!(hlsl_shader_count(&exp), 1 /* one */);
    }

    #[test]
    fn profile_as_str_correct() {
        assert_eq!(HlslProfile::Ps50.as_str(), "ps_5_0" /* pixel shader */);
    }

    #[test]
    fn find_shader_by_profile() {
        let mut exp = new_hlsl_export("N");
        add_hlsl_shader(&mut exp, HlslProfile::Cs50, "CSMain", "void CSMain(){}");
        assert!(find_hlsl_shader(&exp, &HlslProfile::Cs50).is_some() /* found */);
    }

    #[test]
    fn find_missing_none() {
        let exp = new_hlsl_export("N");
        assert!(find_hlsl_shader(&exp, &HlslProfile::Gs50).is_none() /* not found */);
    }

    #[test]
    fn add_define_works() {
        let mut exp = new_hlsl_export("N");
        add_hlsl_shader(&mut exp, HlslProfile::Ps50, "PSMain", "");
        assert!(add_hlsl_define(&mut exp, "MAX_LIGHTS", "8") /* added */);
    }

    #[test]
    fn render_contains_define() {
        let mut exp = new_hlsl_export("N");
        add_hlsl_shader(&mut exp, HlslProfile::Ps50, "PSMain", "void PSMain(){}");
        add_hlsl_define(&mut exp, "USE_HDR", "1");
        let s = find_hlsl_shader(&exp, &HlslProfile::Ps50).expect("should succeed");
        let src = render_hlsl_shader(s);
        assert!(src.contains("USE_HDR") /* define in output */);
    }

    #[test]
    fn validate_non_empty() {
        let mut exp = new_hlsl_export("N");
        assert!(!validate_hlsl_export(&exp) /* empty */);
        add_hlsl_shader(&mut exp, HlslProfile::Vs50, "VS", "");
        assert!(validate_hlsl_export(&exp) /* valid */);
    }

    #[test]
    fn add_define_no_shader_false() {
        let mut exp = new_hlsl_export("N");
        assert!(!add_hlsl_define(&mut exp, "X", "1") /* no shader */);
    }
}
