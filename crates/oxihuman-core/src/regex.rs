// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Regex engine with NFA-based matching (Thompson's construction).
//!
//! Provides two APIs:
//! - **Glob matching**: `regex_match`, `regex_match_all`, etc. (original API)
//! - **Full regex**: `Regex::new(pattern)` with `.is_match()`, `.find()`, `.find_all()`
//!
//! Supported syntax: `.` `*` `+` `?` `{n,m}` `[abc]` `[^abc]` `[a-z]`
//! `\d` `\w` `\s` `\D` `\W` `\S` `^` `$` `|` `()` grouping

use std::fmt;

// ---------------------------------------------------------------------------
// Glob pattern matching (original public API, unchanged)
// ---------------------------------------------------------------------------

pub fn regex_match(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    wildcard_match(&p, &t)
}

fn wildcard_match(p: &[char], t: &[char]) -> bool {
    let m = p.len();
    let n = t.len();
    let mut dp = vec![vec![false; n + 1]; m + 1];
    dp[0][0] = true;
    for i in 1..=m {
        if p[i - 1] == '*' {
            dp[i][0] = dp[i - 1][0];
        }
    }
    for i in 1..=m {
        for j in 1..=n {
            if p[i - 1] == '*' {
                dp[i][j] = dp[i - 1][j] || dp[i][j - 1];
            } else if p[i - 1] == '?' || p[i - 1] == t[j - 1] {
                dp[i][j] = dp[i - 1][j - 1];
            }
        }
    }
    dp[m][n]
}

pub fn regex_match_all(pattern: &str, texts: &[&str]) -> Vec<usize> {
    texts
        .iter()
        .enumerate()
        .filter(|(_, &t)| regex_match(pattern, t))
        .map(|(i, _)| i)
        .collect()
}

pub fn regex_first_match<'a>(pattern: &str, texts: &[&'a str]) -> Option<&'a str> {
    texts.iter().find(|&&t| regex_match(pattern, t)).copied()
}

pub fn regex_count_matches(pattern: &str, texts: &[&str]) -> usize {
    texts.iter().filter(|&&t| regex_match(pattern, t)).count()
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Error returned when a regex pattern fails to parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegexError {
    pub message: String,
    pub position: usize,
}

impl fmt::Display for RegexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "regex error at position {}: {}",
            self.position, self.message
        )
    }
}

impl std::error::Error for RegexError {}

// ---------------------------------------------------------------------------
// AST
// ---------------------------------------------------------------------------

/// A single element or range inside a character class `[...]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CharRange {
    Single(char),
    Range(char, char),
}

impl CharRange {
    fn matches(&self, c: char) -> bool {
        match self {
            CharRange::Single(x) => c == *x,
            CharRange::Range(lo, hi) => c >= *lo && c <= *hi,
        }
    }
}

/// Quantifier kind for `Repeat` nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepeatKind {
    ZeroOrMore,
    OneOrMore,
    Optional,
    Range(usize, Option<usize>),
}

/// Abstract syntax tree for a parsed regex pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegexAst {
    Literal(char),
    Dot,
    Anchor(AnchorKind),
    CharClass {
        ranges: Vec<CharRange>,
        negated: bool,
    },
    Concat(Vec<RegexAst>),
    Alt(Box<RegexAst>, Box<RegexAst>),
    Repeat(Box<RegexAst>, RepeatKind),
    Group(Box<RegexAst>),
}

/// Anchor position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchorKind {
    Start,
    End,
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn new(pattern: &str) -> Self {
        Self {
            chars: pattern.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn expect(&mut self, expected: char) -> Result<(), RegexError> {
        match self.advance() {
            Some(c) if c == expected => Ok(()),
            _ => Err(RegexError {
                message: format!("expected '{expected}'"),
                position: self.pos,
            }),
        }
    }

    /// Parse a full regex (handles alternation at the top level).
    fn parse(&mut self) -> Result<RegexAst, RegexError> {
        let node = self.parse_alternation()?;
        if self.pos < self.chars.len() {
            return Err(RegexError {
                message: "unexpected character".into(),
                position: self.pos,
            });
        }
        Ok(node)
    }

    fn parse_alternation(&mut self) -> Result<RegexAst, RegexError> {
        let mut left = self.parse_concat()?;
        while self.peek() == Some('|') {
            self.advance();
            let right = self.parse_concat()?;
            left = RegexAst::Alt(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_concat(&mut self) -> Result<RegexAst, RegexError> {
        let mut items = Vec::new();
        while let Some(c) = self.peek() {
            if c == '|' || c == ')' {
                break;
            }
            items.push(self.parse_repeat()?);
        }
        match items.len() {
            0 => Ok(RegexAst::Concat(Vec::new())),
            1 => Ok(items.remove(0)),
            _ => Ok(RegexAst::Concat(items)),
        }
    }

    fn parse_repeat(&mut self) -> Result<RegexAst, RegexError> {
        let mut node = self.parse_atom()?;
        loop {
            match self.peek() {
                Some('*') => {
                    self.advance();
                    node = RegexAst::Repeat(Box::new(node), RepeatKind::ZeroOrMore);
                }
                Some('+') => {
                    self.advance();
                    node = RegexAst::Repeat(Box::new(node), RepeatKind::OneOrMore);
                }
                Some('?') => {
                    self.advance();
                    node = RegexAst::Repeat(Box::new(node), RepeatKind::Optional);
                }
                Some('{') => {
                    let kind = self.parse_range_quantifier()?;
                    node = RegexAst::Repeat(Box::new(node), kind);
                }
                _ => break,
            }
        }
        Ok(node)
    }

    fn parse_range_quantifier(&mut self) -> Result<RepeatKind, RegexError> {
        let start_pos = self.pos;
        self.advance(); // consume '{'
        let min = self.parse_number().ok_or_else(|| RegexError {
            message: "expected number in range quantifier".into(),
            position: self.pos,
        })?;
        match self.peek() {
            Some('}') => {
                self.advance();
                Ok(RepeatKind::Range(min, Some(min)))
            }
            Some(',') => {
                self.advance();
                if self.peek() == Some('}') {
                    self.advance();
                    Ok(RepeatKind::Range(min, None))
                } else {
                    let max = self.parse_number().ok_or_else(|| RegexError {
                        message: "expected number after comma in range quantifier".into(),
                        position: self.pos,
                    })?;
                    self.expect('}')?;
                    if max < min {
                        return Err(RegexError {
                            message: "max less than min in range quantifier".into(),
                            position: start_pos,
                        });
                    }
                    Ok(RepeatKind::Range(min, Some(max)))
                }
            }
            _ => Err(RegexError {
                message: "invalid range quantifier".into(),
                position: self.pos,
            }),
        }
    }

    fn parse_number(&mut self) -> Option<usize> {
        let mut digits = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                digits.push(c);
                self.advance();
            } else {
                break;
            }
        }
        if digits.is_empty() {
            None
        } else {
            digits.parse().ok()
        }
    }

    fn parse_atom(&mut self) -> Result<RegexAst, RegexError> {
        match self.peek() {
            None => Err(RegexError {
                message: "unexpected end of pattern".into(),
                position: self.pos,
            }),
            Some('^') => {
                self.advance();
                Ok(RegexAst::Anchor(AnchorKind::Start))
            }
            Some('$') => {
                self.advance();
                Ok(RegexAst::Anchor(AnchorKind::End))
            }
            Some('.') => {
                self.advance();
                Ok(RegexAst::Dot)
            }
            Some('(') => {
                self.advance();
                let inner = self.parse_alternation()?;
                self.expect(')')?;
                Ok(RegexAst::Group(Box::new(inner)))
            }
            Some('[') => self.parse_char_class(),
            Some('\\') => self.parse_escape(),
            Some(c) if is_meta(c) => Err(RegexError {
                message: format!("unexpected metacharacter '{c}'"),
                position: self.pos,
            }),
            Some(c) => {
                self.advance();
                Ok(RegexAst::Literal(c))
            }
        }
    }

    fn parse_escape(&mut self) -> Result<RegexAst, RegexError> {
        self.advance(); // consume '\\'
        match self.advance() {
            None => Err(RegexError {
                message: "trailing backslash".into(),
                position: self.pos,
            }),
            Some('d') => Ok(RegexAst::CharClass {
                ranges: vec![CharRange::Range('0', '9')],
                negated: false,
            }),
            Some('D') => Ok(RegexAst::CharClass {
                ranges: vec![CharRange::Range('0', '9')],
                negated: true,
            }),
            Some('w') => Ok(RegexAst::CharClass {
                ranges: vec![
                    CharRange::Range('a', 'z'),
                    CharRange::Range('A', 'Z'),
                    CharRange::Range('0', '9'),
                    CharRange::Single('_'),
                ],
                negated: false,
            }),
            Some('W') => Ok(RegexAst::CharClass {
                ranges: vec![
                    CharRange::Range('a', 'z'),
                    CharRange::Range('A', 'Z'),
                    CharRange::Range('0', '9'),
                    CharRange::Single('_'),
                ],
                negated: true,
            }),
            Some('s') => Ok(RegexAst::CharClass {
                ranges: vec![
                    CharRange::Single(' '),
                    CharRange::Single('\t'),
                    CharRange::Single('\n'),
                    CharRange::Single('\r'),
                    CharRange::Single('\x0C'),
                ],
                negated: false,
            }),
            Some('S') => Ok(RegexAst::CharClass {
                ranges: vec![
                    CharRange::Single(' '),
                    CharRange::Single('\t'),
                    CharRange::Single('\n'),
                    CharRange::Single('\r'),
                    CharRange::Single('\x0C'),
                ],
                negated: true,
            }),
            Some(c) if is_escapable(c) => Ok(RegexAst::Literal(c)),
            Some(c) => Err(RegexError {
                message: format!("invalid escape '\\{c}'"),
                position: self.pos - 1,
            }),
        }
    }

    fn parse_char_class(&mut self) -> Result<RegexAst, RegexError> {
        self.advance(); // consume '['
        let negated = self.peek() == Some('^');
        if negated {
            self.advance();
        }
        let mut ranges = Vec::new();
        // handle ']' as first char in class (literal)
        if self.peek() == Some(']') {
            ranges.push(CharRange::Single(']'));
            self.advance();
        }
        while self.peek() != Some(']') {
            let c = self.advance().ok_or_else(|| RegexError {
                message: "unterminated character class".into(),
                position: self.pos,
            })?;
            let c = if c == '\\' {
                self.parse_class_escape()?
            } else {
                c
            };
            if self.peek() == Some('-') {
                let dash_pos = self.pos;
                self.advance(); // consume '-'
                if self.peek() == Some(']') {
                    // 'c' and '-' are both literals
                    ranges.push(CharRange::Single(c));
                    ranges.push(CharRange::Single('-'));
                } else {
                    let end = self.advance().ok_or_else(|| RegexError {
                        message: "unterminated character class".into(),
                        position: self.pos,
                    })?;
                    let end = if end == '\\' {
                        self.parse_class_escape()?
                    } else {
                        end
                    };
                    if end < c {
                        return Err(RegexError {
                            message: "invalid range in character class".into(),
                            position: dash_pos,
                        });
                    }
                    ranges.push(CharRange::Range(c, end));
                }
            } else {
                ranges.push(CharRange::Single(c));
            }
        }
        self.expect(']')?;
        Ok(RegexAst::CharClass { ranges, negated })
    }

    fn parse_class_escape(&mut self) -> Result<char, RegexError> {
        match self.advance() {
            None => Err(RegexError {
                message: "trailing backslash in character class".into(),
                position: self.pos,
            }),
            Some('n') => Ok('\n'),
            Some('t') => Ok('\t'),
            Some('r') => Ok('\r'),
            Some(c) => Ok(c),
        }
    }
}

fn is_meta(c: char) -> bool {
    matches!(c, '*' | '+' | '?' | '{' | '}' | ')' | '|')
}

fn is_escapable(c: char) -> bool {
    matches!(
        c,
        '.' | '*'
            | '+'
            | '?'
            | '{'
            | '}'
            | '('
            | ')'
            | '['
            | ']'
            | '|'
            | '^'
            | '$'
            | '\\'
            | 'n'
            | 't'
            | 'r'
    )
}

// ---------------------------------------------------------------------------
// NFA types and transitions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum Transition {
    Char(char),
    Dot,
    CharClass {
        ranges: Vec<CharRange>,
        negated: bool,
    },
    AnchorStart,
    AnchorEnd,
}

impl Transition {
    fn matches(&self, c: char, at_start: bool, at_end: bool) -> bool {
        match self {
            Transition::Char(expected) => c == *expected,
            Transition::Dot => true,
            Transition::CharClass { ranges, negated } => {
                let found = ranges.iter().any(|r| r.matches(c));
                if *negated {
                    !found
                } else {
                    found
                }
            }
            Transition::AnchorStart => at_start,
            Transition::AnchorEnd => at_end,
        }
    }

    fn is_anchor(&self) -> bool {
        matches!(self, Transition::AnchorStart | Transition::AnchorEnd)
    }
}

/// A single state in the NFA.
#[derive(Debug, Clone)]
pub struct NfaState {
    transitions: Vec<(Transition, usize)>,
    epsilon: Vec<usize>,
    accepting: bool,
}

impl NfaState {
    fn new() -> Self {
        Self {
            transitions: Vec::new(),
            epsilon: Vec::new(),
            accepting: false,
        }
    }
}

/// A compiled NFA fragment: start state and accept state indices.
struct Fragment {
    start: usize,
    accept: usize,
}

struct NfaBuilder {
    states: Vec<NfaState>,
}

impl NfaBuilder {
    fn new() -> Self {
        Self { states: Vec::new() }
    }

    fn add_state(&mut self) -> usize {
        let id = self.states.len();
        self.states.push(NfaState::new());
        id
    }

    fn compile(&mut self, ast: &RegexAst) -> Fragment {
        match ast {
            RegexAst::Literal(c) => {
                let start = self.add_state();
                let accept = self.add_state();
                self.states[start]
                    .transitions
                    .push((Transition::Char(*c), accept));
                Fragment { start, accept }
            }
            RegexAst::Dot => {
                let start = self.add_state();
                let accept = self.add_state();
                self.states[start]
                    .transitions
                    .push((Transition::Dot, accept));
                Fragment { start, accept }
            }
            RegexAst::Anchor(kind) => {
                let start = self.add_state();
                let accept = self.add_state();
                let t = match kind {
                    AnchorKind::Start => Transition::AnchorStart,
                    AnchorKind::End => Transition::AnchorEnd,
                };
                self.states[start].transitions.push((t, accept));
                Fragment { start, accept }
            }
            RegexAst::CharClass { ranges, negated } => {
                let start = self.add_state();
                let accept = self.add_state();
                self.states[start].transitions.push((
                    Transition::CharClass {
                        ranges: ranges.clone(),
                        negated: *negated,
                    },
                    accept,
                ));
                Fragment { start, accept }
            }
            RegexAst::Concat(items) => {
                if items.is_empty() {
                    let start = self.add_state();
                    let accept = self.add_state();
                    self.states[start].epsilon.push(accept);
                    return Fragment { start, accept };
                }
                let first = self.compile(&items[0]);
                let mut current_accept = first.accept;
                for item in &items[1..] {
                    let frag = self.compile(item);
                    self.states[current_accept].epsilon.push(frag.start);
                    current_accept = frag.accept;
                }
                Fragment {
                    start: first.start,
                    accept: current_accept,
                }
            }
            RegexAst::Alt(left, right) => {
                let start = self.add_state();
                let accept = self.add_state();
                let l = self.compile(left);
                let r = self.compile(right);
                self.states[start].epsilon.push(l.start);
                self.states[start].epsilon.push(r.start);
                self.states[l.accept].epsilon.push(accept);
                self.states[r.accept].epsilon.push(accept);
                Fragment { start, accept }
            }
            RegexAst::Repeat(inner, kind) => self.compile_repeat(inner, kind),
            RegexAst::Group(inner) => self.compile(inner),
        }
    }

    fn compile_repeat(&mut self, inner: &RegexAst, kind: &RepeatKind) -> Fragment {
        match kind {
            RepeatKind::ZeroOrMore => {
                let start = self.add_state();
                let accept = self.add_state();
                let frag = self.compile(inner);
                self.states[start].epsilon.push(frag.start);
                self.states[start].epsilon.push(accept);
                self.states[frag.accept].epsilon.push(frag.start);
                self.states[frag.accept].epsilon.push(accept);
                Fragment { start, accept }
            }
            RepeatKind::OneOrMore => {
                let start = self.add_state();
                let accept = self.add_state();
                let frag = self.compile(inner);
                self.states[start].epsilon.push(frag.start);
                self.states[frag.accept].epsilon.push(frag.start);
                self.states[frag.accept].epsilon.push(accept);
                Fragment { start, accept }
            }
            RepeatKind::Optional => {
                let start = self.add_state();
                let accept = self.add_state();
                let frag = self.compile(inner);
                self.states[start].epsilon.push(frag.start);
                self.states[start].epsilon.push(accept);
                self.states[frag.accept].epsilon.push(accept);
                Fragment { start, accept }
            }
            RepeatKind::Range(min, max) => {
                let start = self.add_state();
                let mut current = start;

                // Mandatory copies
                for _ in 0..*min {
                    let frag = self.compile(inner);
                    self.states[current].epsilon.push(frag.start);
                    current = frag.accept;
                }

                match max {
                    Some(max_val) => {
                        let optional_count = max_val.saturating_sub(*min);
                        let accept = self.add_state();
                        self.states[current].epsilon.push(accept);
                        for _ in 0..optional_count {
                            let frag = self.compile(inner);
                            self.states[current].epsilon.push(frag.start);
                            self.states[frag.accept].epsilon.push(accept);
                            current = frag.accept;
                        }
                        Fragment { start, accept }
                    }
                    None => {
                        // {min,} = min copies then *
                        let accept = self.add_state();
                        let frag = self.compile(inner);
                        self.states[current].epsilon.push(frag.start);
                        self.states[current].epsilon.push(accept);
                        self.states[frag.accept].epsilon.push(frag.start);
                        self.states[frag.accept].epsilon.push(accept);
                        Fragment { start, accept }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// NFA simulation
// ---------------------------------------------------------------------------

/// Compute epsilon closure of a set of states.
fn epsilon_closure(nfa: &[NfaState], states: &[usize]) -> Vec<usize> {
    let mut visited = vec![false; nfa.len()];
    let mut stack: Vec<usize> = states.to_vec();
    let mut result = Vec::new();
    for &s in states {
        if s < visited.len() {
            visited[s] = true;
        }
    }
    while let Some(s) = stack.pop() {
        result.push(s);
        for &eps in &nfa[s].epsilon {
            if eps < visited.len() && !visited[eps] {
                visited[eps] = true;
                stack.push(eps);
            }
        }
    }
    result
}

/// Advance states through anchor transitions that match the current position.
/// Anchors don't consume a character, so they act like conditional epsilons.
fn advance_anchors(nfa: &[NfaState], states: &[usize], at_start: bool, at_end: bool) -> Vec<usize> {
    let mut result: Vec<usize> = states.to_vec();
    let mut changed = true;
    let mut visited = vec![false; nfa.len()];
    for &s in states {
        if s < visited.len() {
            visited[s] = true;
        }
    }
    while changed {
        changed = false;
        let current = result.clone();
        for &s in &current {
            for (trans, target) in &nfa[s].transitions {
                if trans.is_anchor()
                    && trans.matches('\0', at_start, at_end)
                    && *target < visited.len()
                    && !visited[*target]
                {
                    visited[*target] = true;
                    result.push(*target);
                    let eps = epsilon_closure(nfa, &[*target]);
                    for e in eps {
                        if e < visited.len() && !visited[e] {
                            visited[e] = true;
                            result.push(e);
                        }
                    }
                    changed = true;
                }
            }
        }
    }
    result
}

/// Step NFA: given current states and an input char, return next states.
fn step_nfa(nfa: &[NfaState], states: &[usize], c: char) -> Vec<usize> {
    let mut next = Vec::new();
    for &s in states {
        for (trans, target) in &nfa[s].transitions {
            if !trans.is_anchor() && trans.matches(c, false, false) {
                next.push(*target);
            }
        }
    }
    epsilon_closure(nfa, &next)
}

fn any_accepting(nfa: &[NfaState], states: &[usize]) -> bool {
    states.iter().any(|&s| nfa[s].accepting)
}

// ---------------------------------------------------------------------------
// Regex public API
// ---------------------------------------------------------------------------

/// A compiled regular expression backed by a Thompson NFA.
#[derive(Debug, Clone)]
pub struct Regex {
    nfa: Vec<NfaState>,
    start: usize,
    has_start_anchor: bool,
}

/// A match result from `find` or `find_all`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    pub start: usize,
    pub end: usize,
}

impl Match {
    /// Extract the matched substring from the original text.
    pub fn as_str<'a>(&self, text: &'a str) -> &'a str {
        &text[self.start..self.end]
    }

    /// Length of the match in bytes.
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Whether this is a zero-width match.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

impl Regex {
    /// Compile a regex pattern into an NFA.
    pub fn new(pattern: &str) -> Result<Self, RegexError> {
        let mut parser = Parser::new(pattern);
        let ast = parser.parse()?;
        let has_start_anchor = ast_has_start_anchor(&ast);
        let mut builder = NfaBuilder::new();
        let frag = builder.compile(&ast);
        builder.states[frag.accept].accepting = true;
        Ok(Self {
            nfa: builder.states,
            start: frag.start,
            has_start_anchor,
        })
    }

    /// Check if the entire text matches the pattern.
    pub fn is_match(&self, text: &str) -> bool {
        let chars: Vec<char> = text.chars().collect();
        let n = chars.len();
        let initial = epsilon_closure(&self.nfa, &[self.start]);
        let states = advance_anchors(&self.nfa, &initial, true, n == 0);
        if n == 0 {
            return any_accepting(&self.nfa, &states);
        }
        let mut current = states;
        for (i, &c) in chars.iter().enumerate() {
            current = step_nfa(&self.nfa, &current, c);
            let at_end = i + 1 == n;
            current = advance_anchors(&self.nfa, &current, false, at_end);
            if current.is_empty() {
                return false;
            }
        }
        any_accepting(&self.nfa, &current)
    }

    /// Find the first (leftmost) match of the pattern in the text.
    pub fn find(&self, text: &str) -> Option<Match> {
        let chars: Vec<char> = text.chars().collect();
        let n = chars.len();
        let byte_offsets = compute_byte_offsets(&chars);
        let total_bytes = text.len();

        let start_limit = if self.has_start_anchor { 1 } else { n + 1 };

        for start_idx in 0..start_limit {
            let initial = epsilon_closure(&self.nfa, &[self.start]);
            let at_start = start_idx == 0;
            let at_end = start_idx == n;
            let states = advance_anchors(&self.nfa, &initial, at_start, at_end);

            if any_accepting(&self.nfa, &states) {
                let byte_start = byte_offset_at(&byte_offsets, start_idx, total_bytes);
                return Some(Match {
                    start: byte_start,
                    end: byte_start,
                });
            }

            let mut current = states;
            let mut last_match: Option<usize> = None;

            for (i, &ch) in chars[start_idx..n].iter().enumerate() {
                let i = start_idx + i;
                current = step_nfa(&self.nfa, &current, ch);
                let char_end = i + 1;
                let at_end_now = char_end == n;
                current = advance_anchors(&self.nfa, &current, false, at_end_now);

                if any_accepting(&self.nfa, &current) {
                    let byte_end = byte_offset_at(&byte_offsets, char_end, total_bytes);
                    last_match = Some(byte_end);
                }
                if current.is_empty() {
                    break;
                }
            }

            if let Some(byte_end) = last_match {
                let byte_start = byte_offset_at(&byte_offsets, start_idx, total_bytes);
                return Some(Match {
                    start: byte_start,
                    end: byte_end,
                });
            }
        }
        None
    }

    /// Find all non-overlapping matches in the text.
    pub fn find_all(&self, text: &str) -> Vec<Match> {
        let mut matches = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let n = chars.len();
        let byte_offsets = compute_byte_offsets(&chars);
        let total_bytes = text.len();

        let mut start_idx = 0;
        while start_idx <= n {
            if self.has_start_anchor && start_idx > 0 {
                break;
            }

            let initial = epsilon_closure(&self.nfa, &[self.start]);
            let at_start = start_idx == 0;
            let at_end = start_idx == n;
            let states = advance_anchors(&self.nfa, &initial, at_start, at_end);

            let mut current = states.clone();
            let mut last_match: Option<usize> = None;

            if any_accepting(&self.nfa, &current) {
                last_match = Some(start_idx);
            }

            for (i, &ch) in chars[start_idx..n].iter().enumerate() {
                let i = start_idx + i;
                current = step_nfa(&self.nfa, &current, ch);
                let char_end = i + 1;
                let at_end_now = char_end == n;
                current = advance_anchors(&self.nfa, &current, false, at_end_now);

                if any_accepting(&self.nfa, &current) {
                    last_match = Some(char_end);
                }
                if current.is_empty() {
                    break;
                }
            }

            if let Some(end_char) = last_match {
                let byte_start = byte_offset_at(&byte_offsets, start_idx, total_bytes);
                let byte_end = byte_offset_at(&byte_offsets, end_char, total_bytes);
                matches.push(Match {
                    start: byte_start,
                    end: byte_end,
                });
                if end_char == start_idx {
                    start_idx += 1;
                } else {
                    start_idx = end_char;
                }
            } else {
                start_idx += 1;
            }
        }
        matches
    }

    /// Check if the pattern matches anywhere in the text.
    pub fn is_match_anywhere(&self, text: &str) -> bool {
        self.find(text).is_some()
    }
}

/// Compute byte offsets for each char index.
fn compute_byte_offsets(chars: &[char]) -> Vec<usize> {
    chars
        .iter()
        .scan(0usize, |acc, c| {
            let off = *acc;
            *acc += c.len_utf8();
            Some(off)
        })
        .collect()
}

fn byte_offset_at(offsets: &[usize], char_idx: usize, total_bytes: usize) -> usize {
    if char_idx < offsets.len() {
        offsets[char_idx]
    } else {
        total_bytes
    }
}

/// Check if the AST begins with a start anchor (for optimization).
fn ast_has_start_anchor(ast: &RegexAst) -> bool {
    match ast {
        RegexAst::Anchor(AnchorKind::Start) => true,
        RegexAst::Concat(items) => items.first().is_some_and(ast_has_start_anchor),
        RegexAst::Group(inner) => ast_has_start_anchor(inner),
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Original glob tests (unchanged) ----

    #[test]
    fn test_exact_match() {
        assert!(regex_match("hello", "hello"));
        assert!(!regex_match("hello", "world"));
    }

    #[test]
    fn test_star_wildcard() {
        assert!(regex_match("h*o", "hello"));
        assert!(regex_match("*", "anything"));
        assert!(regex_match("*.rs", "main.rs"));
    }

    #[test]
    fn test_question_wildcard() {
        assert!(regex_match("h?llo", "hello"));
        assert!(!regex_match("h?llo", "hllo"));
    }

    #[test]
    fn test_match_all() {
        let texts = &["foo", "bar", "foobar", "baz"];
        let indices = regex_match_all("foo*", texts);
        assert!(indices.contains(&0));
        assert!(indices.contains(&2));
        assert!(!indices.contains(&1));
    }

    #[test]
    fn test_first_match() {
        let texts = &["abc", "def", "abx"];
        let m = regex_first_match("ab*", texts);
        assert_eq!(m, Some("abc"));
    }

    #[test]
    fn test_count_matches() {
        let texts = &["cat", "car", "bar", "cap"];
        let n = regex_count_matches("ca*", texts);
        assert_eq!(n, 3);
    }

    #[test]
    fn test_empty_pattern() {
        assert!(regex_match("", ""));
        assert!(!regex_match("", "a"));
    }

    // ---- NFA regex tests ----

    #[test]
    fn test_regex_literal() {
        let re = Regex::new("abc").expect("should succeed");
        assert!(re.is_match("abc"));
        assert!(!re.is_match("ab"));
        assert!(!re.is_match("abcd"));
    }

    #[test]
    fn test_regex_dot() {
        let re = Regex::new("a.c").expect("should succeed");
        assert!(re.is_match("abc"));
        assert!(re.is_match("axc"));
        assert!(!re.is_match("ac"));
    }

    #[test]
    fn test_regex_star() {
        let re = Regex::new("ab*c").expect("should succeed");
        assert!(re.is_match("ac"));
        assert!(re.is_match("abc"));
        assert!(re.is_match("abbbc"));
        assert!(!re.is_match("abbc_extra"));
    }

    #[test]
    fn test_regex_plus() {
        let re = Regex::new("ab+c").expect("should succeed");
        assert!(!re.is_match("ac"));
        assert!(re.is_match("abc"));
        assert!(re.is_match("abbbc"));
    }

    #[test]
    fn test_regex_optional() {
        let re = Regex::new("ab?c").expect("should succeed");
        assert!(re.is_match("ac"));
        assert!(re.is_match("abc"));
        assert!(!re.is_match("abbc"));
    }

    #[test]
    fn test_regex_alternation() {
        let re = Regex::new("cat|dog").expect("should succeed");
        assert!(re.is_match("cat"));
        assert!(re.is_match("dog"));
        assert!(!re.is_match("bird"));
    }

    #[test]
    fn test_regex_grouping() {
        let re = Regex::new("(ab)+").expect("should succeed");
        assert!(re.is_match("ab"));
        assert!(re.is_match("abab"));
        assert!(!re.is_match(""));
    }

    #[test]
    fn test_regex_char_class() {
        let re = Regex::new("[abc]").expect("should succeed");
        assert!(re.is_match("a"));
        assert!(re.is_match("b"));
        assert!(!re.is_match("d"));
    }

    #[test]
    fn test_regex_char_class_range() {
        let re = Regex::new("[a-z]+").expect("should succeed");
        assert!(re.is_match("hello"));
        assert!(!re.is_match("HELLO"));
        assert!(!re.is_match(""));
    }

    #[test]
    fn test_regex_negated_class() {
        let re = Regex::new("[^0-9]+").expect("should succeed");
        assert!(re.is_match("abc"));
        assert!(!re.is_match("123"));
    }

    #[test]
    fn test_regex_digit_shorthand() {
        let re = Regex::new("\\d+").expect("should succeed");
        assert!(re.is_match("42"));
        assert!(!re.is_match("abc"));
    }

    #[test]
    fn test_regex_word_shorthand() {
        let re = Regex::new("\\w+").expect("should succeed");
        assert!(re.is_match("hello_123"));
        assert!(!re.is_match("!!!"));
    }

    #[test]
    fn test_regex_space_shorthand() {
        let re = Regex::new("a\\sb").expect("should succeed");
        assert!(re.is_match("a b"));
        assert!(re.is_match("a\tb"));
        assert!(!re.is_match("ab"));
    }

    #[test]
    fn test_regex_negated_shorthands() {
        let re = Regex::new("\\D+").expect("should succeed");
        assert!(re.is_match("abc"));
        assert!(!re.is_match("42"));

        let re2 = Regex::new("\\W").expect("should succeed");
        assert!(re2.is_match("!"));
        assert!(!re2.is_match("a"));

        let re3 = Regex::new("\\S+").expect("should succeed");
        assert!(re3.is_match("abc"));
        assert!(!re3.is_match(" "));
    }

    #[test]
    fn test_regex_anchors() {
        let re = Regex::new("^abc$").expect("should succeed");
        assert!(re.is_match("abc"));
        assert!(!re.is_match("xabc"));
        assert!(!re.is_match("abcx"));
    }

    #[test]
    fn test_regex_start_anchor_only() {
        let re = Regex::new("^abc").expect("should succeed");
        assert!(re.is_match("abc"));
        assert!(!re.is_match("abcdef"));
        let m = re.find("abcdef");
        assert!(m.is_some());
        let m = m.expect("should succeed");
        assert_eq!(m.start, 0);
        assert_eq!(m.end, 3);
    }

    #[test]
    fn test_regex_range_quantifier() {
        let re = Regex::new("a{2,4}").expect("should succeed");
        assert!(!re.is_match("a"));
        assert!(re.is_match("aa"));
        assert!(re.is_match("aaa"));
        assert!(re.is_match("aaaa"));
        assert!(!re.is_match("aaaaa"));
    }

    #[test]
    fn test_regex_exact_quantifier() {
        let re = Regex::new("a{3}").expect("should succeed");
        assert!(!re.is_match("aa"));
        assert!(re.is_match("aaa"));
        assert!(!re.is_match("aaaa"));
    }

    #[test]
    fn test_regex_min_quantifier() {
        let re = Regex::new("a{2,}").expect("should succeed");
        assert!(!re.is_match("a"));
        assert!(re.is_match("aa"));
        assert!(re.is_match("aaaaaaa"));
    }

    #[test]
    fn test_regex_find() {
        let re = Regex::new("\\d+").expect("should succeed");
        let m = re.find("abc 123 def");
        assert!(m.is_some());
        let m = m.expect("should succeed");
        assert_eq!(m.as_str("abc 123 def"), "123");
    }

    #[test]
    fn test_regex_find_all() {
        let re = Regex::new("\\d+").expect("should succeed");
        let text = "12 ab 34 cd 56";
        let matches = re.find_all(text);
        assert_eq!(matches.len(), 3);
        assert_eq!(matches[0].as_str(text), "12");
        assert_eq!(matches[1].as_str(text), "34");
        assert_eq!(matches[2].as_str(text), "56");
    }

    #[test]
    fn test_regex_find_no_match() {
        let re = Regex::new("xyz").expect("should succeed");
        assert!(re.find("abc def").is_none());
    }

    #[test]
    fn test_regex_is_match_anywhere() {
        let re = Regex::new("world").expect("should succeed");
        assert!(re.is_match_anywhere("hello world"));
        assert!(!re.is_match_anywhere("hello earth"));
    }

    #[test]
    fn test_regex_escaped_metachar() {
        let re = Regex::new("a\\.b").expect("should succeed");
        assert!(re.is_match("a.b"));
        assert!(!re.is_match("axb"));
    }

    #[test]
    fn test_regex_empty_pattern() {
        let re = Regex::new("").expect("should succeed");
        assert!(re.is_match(""));
    }

    #[test]
    fn test_regex_complex_pattern() {
        let re = Regex::new("(a|b)*c").expect("should succeed");
        assert!(re.is_match("c"));
        assert!(re.is_match("ac"));
        assert!(re.is_match("bc"));
        assert!(re.is_match("ababc"));
        assert!(!re.is_match("abc_trailing"));
    }

    #[test]
    fn test_regex_email_like() {
        let re = Regex::new("\\w+@\\w+\\.\\w+").expect("should succeed");
        assert!(re.is_match("user@host.com"));
        assert!(!re.is_match("user@"));
    }

    #[test]
    fn test_regex_parse_error() {
        assert!(Regex::new("[abc").is_err());
        assert!(Regex::new("(abc").is_err());
        assert!(Regex::new("*").is_err());
        assert!(Regex::new("+").is_err());
    }

    #[test]
    fn test_regex_find_all_words() {
        let re = Regex::new("[a-zA-Z]+").expect("should succeed");
        let text = "hello, world! foo";
        let matches = re.find_all(text);
        assert_eq!(matches.len(), 3);
        assert_eq!(matches[0].as_str(text), "hello");
        assert_eq!(matches[1].as_str(text), "world");
        assert_eq!(matches[2].as_str(text), "foo");
    }

    #[test]
    fn test_regex_nested_groups() {
        let re = Regex::new("((ab)+c)+").expect("should succeed");
        assert!(re.is_match("abc"));
        assert!(re.is_match("ababc"));
        assert!(re.is_match("abcababc"));
        assert!(!re.is_match("ac"));
    }

    #[test]
    fn test_regex_end_anchor_find() {
        let re = Regex::new("\\d+$").expect("should succeed");
        let m = re.find("abc123");
        assert!(m.is_some());
        assert_eq!(m.expect("should succeed").as_str("abc123"), "123");
    }

    #[test]
    fn test_regex_unicode() {
        let re = Regex::new("..").expect("should succeed");
        assert!(re.is_match("\u{00e9}\u{00e9}"));
        let m = re.find("a\u{00e9}b");
        assert!(m.is_some());
        let m = m.expect("should succeed");
        assert_eq!(m.as_str("a\u{00e9}b"), "a\u{00e9}");
    }

    #[test]
    fn test_regex_match_len() {
        let re = Regex::new("[a-z]+").expect("should succeed");
        let m = re.find("123abc456");
        assert!(m.is_some());
        let m = m.expect("should succeed");
        assert_eq!(m.len(), 3);
        assert!(!m.is_empty());
    }
}
