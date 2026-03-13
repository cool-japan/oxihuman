// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Csound score and orchestra stub export.

/// A Csound instrument definition.
#[derive(Debug, Clone)]
pub struct CsoundInstrument {
    pub number: u32,
    pub name: String,
    pub body: String,
}

impl CsoundInstrument {
    pub fn new(number: u32, name: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            number,
            name: name.into(),
            body: body.into(),
        }
    }

    pub fn to_orc_section(&self) -> String {
        format!(
            "instr {}\n    /* {} */\n{}\nendin\n",
            self.number, self.name, self.body
        )
    }
}

/// A Csound score note event (i-statement).
#[derive(Debug, Clone)]
pub struct CsoundNote {
    pub instrument: u32,
    pub start_time: f64,
    pub duration: f64,
    pub params: Vec<f64>,
}

impl CsoundNote {
    pub fn new(instrument: u32, start_time: f64, duration: f64, params: Vec<f64>) -> Self {
        Self {
            instrument,
            start_time,
            duration,
            params,
        }
    }

    pub fn to_score_line(&self) -> String {
        let mut line = format!("i{} {} {}", self.instrument, self.start_time, self.duration);
        for p in &self.params {
            line.push_str(&format!(" {}", p));
        }
        line
    }
}

/// A Csound program: orchestra + score.
#[derive(Debug, Clone, Default)]
pub struct CsoundProgram {
    pub orchestra_header: String,
    pub instruments: Vec<CsoundInstrument>,
    pub score_notes: Vec<CsoundNote>,
    pub sample_rate: u32,
    pub krate: u32,
}

impl CsoundProgram {
    pub fn new(sample_rate: u32, krate: u32) -> Self {
        Self {
            orchestra_header: String::new(),
            instruments: Vec::new(),
            score_notes: Vec::new(),
            sample_rate,
            krate,
        }
    }

    pub fn add_instrument(&mut self, instr: CsoundInstrument) {
        self.instruments.push(instr);
    }

    pub fn add_note(&mut self, note: CsoundNote) {
        self.score_notes.push(note);
    }
}

/// Generate a full Csound .csd file (orchestra + score) string.
pub fn generate_csd(prog: &CsoundProgram) -> String {
    let mut csd = String::new();
    csd.push_str("<CsoundSynthesizer>\n<CsOptions>\n-d -o dac\n</CsOptions>\n");
    csd.push_str("<CsInstruments>\n");
    csd.push_str(&format!("sr = {}\n", prog.sample_rate));
    csd.push_str(&format!("kr = {}\n", prog.krate));
    csd.push_str("nchnls = 2\n");
    csd.push_str("0dbfs = 1\n");
    if !prog.orchestra_header.is_empty() {
        csd.push_str(&prog.orchestra_header);
        csd.push('\n');
    }
    for instr in &prog.instruments {
        csd.push_str(&instr.to_orc_section());
    }
    csd.push_str("</CsInstruments>\n");
    csd.push_str("<CsScore>\n");
    for note in &prog.score_notes {
        csd.push_str(&note.to_score_line());
        csd.push('\n');
    }
    csd.push_str("e\n</CsScore>\n</CsoundSynthesizer>\n");
    csd
}

/// Validate that a string is a Csound CSD file.
pub fn is_valid_csd(src: &str) -> bool {
    src.contains("<CsoundSynthesizer>") && src.contains("<CsInstruments>")
}

/// Count instruments in a CsoundProgram.
pub fn count_csound_instruments(prog: &CsoundProgram) -> usize {
    prog.instruments.len()
}

/// Build a minimal sine wave Csound program.
pub fn sine_wave_csound_program(freq: f64, duration: f64) -> CsoundProgram {
    let body = format!("    aout oscil 0.5, {}, 1\n    outs aout, aout\n", freq);
    let mut prog = CsoundProgram::new(44100, 4410);
    prog.add_instrument(CsoundInstrument::new(1, "Sine", body));
    prog.add_note(CsoundNote::new(1, 0.0, duration, vec![freq]));
    prog
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instrument_to_orc() {
        let instr = CsoundInstrument::new(1, "test", "    out 0");
        let orc = instr.to_orc_section();
        assert!(orc.contains("instr 1") /* instrument header */);
        assert!(orc.contains("endin") /* end marker */);
    }

    #[test]
    fn test_note_to_score_line() {
        let note = CsoundNote::new(1, 0.0, 2.5, vec![440.0]);
        let line = note.to_score_line();
        assert!(line.starts_with("i1") /* instrument number */);
        assert!(line.contains("2.5") /* duration */);
    }

    #[test]
    fn test_generate_csd_valid() {
        let prog = CsoundProgram::new(44100, 4410);
        let csd = generate_csd(&prog);
        assert!(is_valid_csd(&csd) /* valid CSD */);
    }

    #[test]
    fn test_generate_csd_has_sr() {
        let prog = CsoundProgram::new(48000, 4800);
        let csd = generate_csd(&prog);
        assert!(csd.contains("sr = 48000") /* correct sample rate */);
    }

    #[test]
    fn test_count_instruments_empty() {
        let prog = CsoundProgram::new(44100, 4410);
        assert_eq!(count_csound_instruments(&prog), 0 /* empty program */);
    }

    #[test]
    fn test_sine_wave_program() {
        let prog = sine_wave_csound_program(440.0, 1.0);
        assert_eq!(count_csound_instruments(&prog), 1 /* one instrument */);
    }

    #[test]
    fn test_sine_wave_csd_valid() {
        let prog = sine_wave_csound_program(220.0, 2.0);
        let csd = generate_csd(&prog);
        assert!(is_valid_csd(&csd) /* sine wave CSD is valid */);
    }

    #[test]
    fn test_is_valid_csd_false() {
        assert!(!is_valid_csd("not a csound file") /* invalid */);
    }

    #[test]
    fn test_note_params_in_line() {
        let note = CsoundNote::new(2, 1.0, 3.0, vec![880.0, 0.5]);
        let line = note.to_score_line();
        assert!(line.contains("880") /* frequency in line */);
    }
}
