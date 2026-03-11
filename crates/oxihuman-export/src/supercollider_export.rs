// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! SuperCollider SynthDef stub export.

/// A SuperCollider UGen (unit generator) stub.
#[derive(Debug, Clone)]
pub struct ScUgen {
    pub name: String,
    pub inputs: Vec<String>,
    pub rate: ScRate,
}

impl ScUgen {
    pub fn new(name: impl Into<String>, inputs: Vec<String>, rate: ScRate) -> Self {
        Self {
            name: name.into(),
            inputs,
            rate,
        }
    }
}

/// SuperCollider computation rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScRate {
    Audio,
    Control,
    Scalar,
}

impl ScRate {
    pub fn suffix(&self) -> &'static str {
        match self {
            ScRate::Audio => ".ar",
            ScRate::Control => ".kr",
            ScRate::Scalar => ".ir",
        }
    }
}

/// A SuperCollider SynthDef stub.
#[derive(Debug, Clone, Default)]
pub struct ScSynthDef {
    pub name: String,
    pub ugens: Vec<ScUgen>,
    pub args: Vec<(String, f64)>,
}

impl ScSynthDef {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ugens: Vec::new(),
            args: Vec::new(),
        }
    }

    pub fn add_arg(&mut self, name: impl Into<String>, default: f64) {
        self.args.push((name.into(), default));
    }

    pub fn add_ugen(&mut self, ugen: ScUgen) {
        self.ugens.push(ugen);
    }
}

/// SuperCollider export configuration.
#[derive(Debug, Clone, Default)]
pub struct ScExportConfig {
    pub server_address: String,
    pub server_port: u16,
}

impl ScExportConfig {
    pub fn new(address: impl Into<String>, port: u16) -> Self {
        Self {
            server_address: address.into(),
            server_port: port,
        }
    }
}

/// Generate SuperCollider sclang source code for a SynthDef.
pub fn generate_sc_synthdef(synth: &ScSynthDef, _cfg: &ScExportConfig) -> String {
    let mut src = String::new();
    src.push_str(&format!("/* Auto-generated SynthDef: {} */\n", synth.name));
    src.push_str(&format!("SynthDef('{}', {{\n", synth.name));
    if !synth.args.is_empty() {
        let args_str: String = synth
            .args
            .iter()
            .map(|(n, v)| format!("    |{} = {}|", n, v))
            .collect::<Vec<_>>()
            .join(", ");
        src.push_str(&format!("{};\n", args_str));
    }
    for ugen in &synth.ugens {
        let inputs = synth
            .ugens
            .iter()
            .flat_map(|u| u.inputs.iter().cloned())
            .collect::<Vec<_>>()
            .join(", ");
        src.push_str(&format!(
            "    var sig = {}{}({});\n",
            ugen.name,
            ugen.rate.suffix(),
            inputs
        ));
    }
    src.push_str("    Out.ar(0, sig);\n");
    src.push_str("}).add;\n");
    src
}

/// Count UGens in a SynthDef.
pub fn count_ugens(synth: &ScSynthDef) -> usize {
    synth.ugens.len()
}

/// Build a minimal sine tone SynthDef.
pub fn sine_tone_synthdef(name: impl Into<String>, freq: f64, amp: f64) -> ScSynthDef {
    let mut synth = ScSynthDef::new(name);
    synth.add_arg("freq", freq);
    synth.add_arg("amp", amp);
    synth.add_ugen(ScUgen::new("SinOsc", vec!["freq".into()], ScRate::Audio));
    synth
}

/// Check if source contains a SynthDef definition.
pub fn has_synthdef_definition(src: &str) -> bool {
    src.contains("SynthDef(")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc_rate_suffix() {
        assert_eq!(ScRate::Audio.suffix(), ".ar" /* audio rate */);
        assert_eq!(ScRate::Control.suffix(), ".kr" /* control rate */);
        assert_eq!(ScRate::Scalar.suffix(), ".ir" /* scalar rate */);
    }

    #[test]
    fn test_synthdef_new_empty() {
        let s = ScSynthDef::new("test");
        assert_eq!(s.name, "test" /* correct name */);
        assert!(s.ugens.is_empty() /* no UGens initially */);
    }

    #[test]
    fn test_add_arg() {
        let mut s = ScSynthDef::new("x");
        s.add_arg("freq", 440.0);
        assert_eq!(s.args.len(), 1 /* one argument */);
    }

    #[test]
    fn test_count_ugens() {
        let synth = sine_tone_synthdef("tone", 440.0, 0.5);
        assert_eq!(count_ugens(&synth), 1 /* one SinOsc UGen */);
    }

    #[test]
    fn test_generate_source_has_synthdef() {
        let synth = sine_tone_synthdef("tone", 440.0, 0.5);
        let cfg = ScExportConfig::default();
        let src = generate_sc_synthdef(&synth, &cfg);
        assert!(has_synthdef_definition(&src) /* SynthDef present */);
    }

    #[test]
    fn test_generate_source_contains_name() {
        let synth = sine_tone_synthdef("mySynth", 440.0, 0.5);
        let cfg = ScExportConfig::default();
        let src = generate_sc_synthdef(&synth, &cfg);
        assert!(src.contains("mySynth") /* name in source */);
    }

    #[test]
    fn test_generate_source_contains_sinosc() {
        let synth = sine_tone_synthdef("osc", 220.0, 1.0);
        let cfg = ScExportConfig::default();
        let src = generate_sc_synthdef(&synth, &cfg);
        assert!(src.contains("SinOsc") /* SinOsc UGen in source */);
    }

    #[test]
    fn test_sc_export_config_new() {
        let cfg = ScExportConfig::new("127.0.0.1", 57110);
        assert_eq!(cfg.server_port, 57110 /* default SC port */);
    }

    #[test]
    fn test_has_synthdef_definition_false() {
        assert!(!has_synthdef_definition("// empty sclang file") /* no SynthDef */);
    }
}
