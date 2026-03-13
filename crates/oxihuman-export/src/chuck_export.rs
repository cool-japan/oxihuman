// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! ChucK music programming language stub export.

/// A ChucK UGen type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChuckUgen {
    SinOsc,
    SawOsc,
    SqrOsc,
    TriOsc,
    Noise,
    Gain,
    Dac,
    Adc,
    Other(String),
}

impl ChuckUgen {
    pub fn class_name(&self) -> String {
        match self {
            ChuckUgen::SinOsc => "SinOsc".to_string(),
            ChuckUgen::SawOsc => "SawOsc".to_string(),
            ChuckUgen::SqrOsc => "SqrOsc".to_string(),
            ChuckUgen::TriOsc => "TriOsc".to_string(),
            ChuckUgen::Noise => "Noise".to_string(),
            ChuckUgen::Gain => "Gain".to_string(),
            ChuckUgen::Dac => "dac".to_string(),
            ChuckUgen::Adc => "adc".to_string(),
            ChuckUgen::Other(s) => s.clone(),
        }
    }
}

/// A ChucK variable declaration.
#[derive(Debug, Clone)]
pub struct ChuckVar {
    pub var_type: String,
    pub name: String,
    pub ugen: Option<ChuckUgen>,
}

impl ChuckVar {
    pub fn new_ugen(name: impl Into<String>, ugen: ChuckUgen) -> Self {
        let var_type = ugen.class_name();
        Self {
            var_type,
            name: name.into(),
            ugen: Some(ugen),
        }
    }

    pub fn to_declaration(&self) -> String {
        format!("{} {} => dac;", self.var_type, self.name)
    }
}

/// A ChucK program sequence.
#[derive(Debug, Clone, Default)]
pub struct ChuckProgram {
    pub statements: Vec<String>,
}

impl ChuckProgram {
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
        }
    }

    pub fn add_statement(&mut self, stmt: impl Into<String>) {
        self.statements.push(stmt.into());
    }

    pub fn add_ugen_chain(&mut self, src: &str, dst: &str) {
        /* Chuck => connection */
        self.statements.push(format!("{} => {};", src, dst));
    }

    pub fn add_freq_set(&mut self, ugen_name: &str, freq: f64) {
        self.statements.push(format!("{} => {}:", freq, ugen_name));
        self.statements
            .push(format!("    {}.freq({});", ugen_name, freq));
    }

    pub fn add_advance_time(&mut self, duration_ms: f64) {
        self.statements.push(format!("{}::ms => now;", duration_ms));
    }
}

/// Generate ChucK source code from a program.
pub fn generate_chuck_source(prog: &ChuckProgram) -> String {
    let mut src = String::new();
    src.push_str("/* Auto-generated ChucK program */\n");
    for stmt in &prog.statements {
        src.push_str(stmt);
        src.push('\n');
    }
    src
}

/// Build a minimal ChucK sine tone program.
pub fn sine_tone_chuck_program(freq: f64, duration_ms: f64, gain: f64) -> ChuckProgram {
    let mut prog = ChuckProgram::new();
    prog.add_statement("SinOsc s => dac;");
    prog.add_statement(format!("{} => s.freq;", freq));
    prog.add_statement(format!("{} => s.gain;", gain));
    prog.add_advance_time(duration_ms);
    prog
}

/// Count statements in a ChucK program.
pub fn count_chuck_statements(prog: &ChuckProgram) -> usize {
    prog.statements.len()
}

/// Check if a ChucK source string has a time advance statement.
pub fn has_time_advance(src: &str) -> bool {
    src.contains("=> now")
}

/// Count how many UGen connections exist in the source.
pub fn count_connections(src: &str) -> usize {
    src.lines().filter(|l| l.contains("=>")).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chuck_ugen_class_name() {
        assert_eq!(
            ChuckUgen::SinOsc.class_name(),
            "SinOsc" /* SinOsc name */
        );
        assert_eq!(ChuckUgen::Dac.class_name(), "dac" /* dac lowercase */);
    }

    #[test]
    fn test_chuck_var_declaration() {
        let v = ChuckVar::new_ugen("s", ChuckUgen::SinOsc);
        let decl = v.to_declaration();
        assert!(decl.contains("SinOsc") /* class name */);
        assert!(decl.contains("=> dac") /* connected to dac */);
    }

    #[test]
    fn test_add_statement_count() {
        let mut prog = ChuckProgram::new();
        prog.add_statement("SinOsc s => dac;");
        assert_eq!(count_chuck_statements(&prog), 1 /* one statement */);
    }

    #[test]
    fn test_generate_source_non_empty() {
        let prog = sine_tone_chuck_program(440.0, 500.0, 0.5);
        let src = generate_chuck_source(&prog);
        assert!(!src.is_empty() /* non-empty source */);
    }

    #[test]
    fn test_sine_tone_has_time_advance() {
        let prog = sine_tone_chuck_program(440.0, 1000.0, 0.5);
        let src = generate_chuck_source(&prog);
        assert!(has_time_advance(&src) /* has time advance */);
    }

    #[test]
    fn test_count_connections_positive() {
        let prog = sine_tone_chuck_program(440.0, 500.0, 0.5);
        let src = generate_chuck_source(&prog);
        assert!(count_connections(&src) > 0 /* at least one connection */);
    }

    #[test]
    fn test_advance_time_format() {
        let mut prog = ChuckProgram::new();
        prog.add_advance_time(250.0);
        assert!(prog.statements[0].contains("250") /* duration */);
        assert!(prog.statements[0].contains("=> now") /* advance syntax */);
    }

    #[test]
    fn test_has_time_advance_false() {
        assert!(!has_time_advance("SinOsc s => dac;\n") /* no time advance */);
    }

    #[test]
    fn test_other_ugen() {
        let u = ChuckUgen::Other("ADSR".to_string());
        assert_eq!(u.class_name(), "ADSR" /* custom UGen */);
    }
}
