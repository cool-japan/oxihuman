// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Faust DSP language stub export.

/// A Faust DSP export configuration.
#[derive(Debug, Clone)]
pub struct FaustExportConfig {
    pub process_name: String,
    pub sample_rate: u32,
    pub num_inputs: u32,
    pub num_outputs: u32,
}

impl Default for FaustExportConfig {
    fn default() -> Self {
        Self {
            process_name: "process".to_string(),
            sample_rate: 44100,
            num_inputs: 0,
            num_outputs: 1,
        }
    }
}

/// A Faust DSP parameter.
#[derive(Debug, Clone)]
pub struct FaustParam {
    pub name: String,
    pub default_value: f64,
    pub min_value: f64,
    pub max_value: f64,
}

impl FaustParam {
    pub fn new(
        name: impl Into<String>,
        default_value: f64,
        min_value: f64,
        max_value: f64,
    ) -> Self {
        Self {
            name: name.into(),
            default_value,
            min_value,
            max_value,
        }
    }
}

/// A Faust DSP program stub.
#[derive(Debug, Clone, Default)]
pub struct FaustProgram {
    pub imports: Vec<String>,
    pub params: Vec<FaustParam>,
    pub process_expr: String,
}

impl FaustProgram {
    pub fn new() -> Self {
        Self {
            imports: vec!["stdfaust.lib".to_string()],
            params: Vec::new(),
            process_expr: "0".to_string(),
        }
    }

    pub fn add_import(&mut self, lib: impl Into<String>) {
        self.imports.push(lib.into());
    }

    pub fn add_param(&mut self, param: FaustParam) {
        self.params.push(param);
    }

    pub fn set_process(&mut self, expr: impl Into<String>) {
        self.process_expr = expr.into();
    }
}

/// Generate Faust DSP source code from a program stub.
pub fn generate_faust_source(prog: &FaustProgram, cfg: &FaustExportConfig) -> String {
    let mut src = String::new();
    src.push_str(&format!(
        "/* Auto-generated Faust DSP — {} */\n",
        cfg.process_name
    ));
    for import in &prog.imports {
        src.push_str(&format!("import(\"{}\");\n", import));
    }
    src.push('\n');
    for p in &prog.params {
        src.push_str(&format!(
            "{} = hslider(\"{}\", {}, {}, {}, 0.01);\n",
            p.name.to_lowercase().replace(' ', "_"),
            p.name,
            p.default_value,
            p.min_value,
            p.max_value,
        ));
    }
    src.push('\n');
    src.push_str(&format!("process = {};\n", prog.process_expr));
    src
}

/// Count the number of lines in generated Faust source.
pub fn count_faust_lines(src: &str) -> usize {
    src.lines().count()
}

/// Check if the source contains a valid `process` definition.
pub fn has_process_definition(src: &str) -> bool {
    src.lines().any(|l| l.trim_start().starts_with("process"))
}

/// Build a minimal sine oscillator Faust program.
pub fn sine_osc_program(freq_hz: f64) -> FaustProgram {
    let mut prog = FaustProgram::new();
    prog.add_param(FaustParam::new("Frequency", freq_hz, 20.0, 20000.0));
    prog.set_process("os.osc(freq)");
    prog
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = FaustExportConfig::default();
        assert_eq!(cfg.sample_rate, 44100 /* standard sample rate */);
    }

    #[test]
    fn test_new_program_has_import() {
        let prog = FaustProgram::new();
        assert!(!prog.imports.is_empty() /* default has at least one import */);
    }

    #[test]
    fn test_generate_source_has_process() {
        let prog = FaustProgram::new();
        let cfg = FaustExportConfig::default();
        let src = generate_faust_source(&prog, &cfg);
        assert!(has_process_definition(&src) /* source has process def */);
    }

    #[test]
    fn test_generate_source_has_import() {
        let prog = FaustProgram::new();
        let cfg = FaustExportConfig::default();
        let src = generate_faust_source(&prog, &cfg);
        assert!(src.contains("import(") /* source has import statement */);
    }

    #[test]
    fn test_count_faust_lines_positive() {
        let src = "import(\"stdfaust.lib\");\nprocess = 0;\n";
        assert!(count_faust_lines(src) >= 2 /* at least 2 lines */);
    }

    #[test]
    fn test_has_process_false() {
        assert!(!has_process_definition("import(\"x\");\n") /* no process def */);
    }

    #[test]
    fn test_add_param_appears_in_source() {
        let mut prog = FaustProgram::new();
        prog.add_param(FaustParam::new("Gain", 0.5, 0.0, 1.0));
        let cfg = FaustExportConfig::default();
        let src = generate_faust_source(&prog, &cfg);
        assert!(src.contains("hslider") /* slider generated for param */);
    }

    #[test]
    fn test_sine_osc_program() {
        let prog = sine_osc_program(440.0);
        let cfg = FaustExportConfig::default();
        let src = generate_faust_source(&prog, &cfg);
        assert!(src.contains("os.osc") /* oscillator process */);
    }

    #[test]
    fn test_set_process() {
        let mut prog = FaustProgram::new();
        prog.set_process("no.noise");
        assert_eq!(prog.process_expr, "no.noise" /* custom process */);
    }
}
