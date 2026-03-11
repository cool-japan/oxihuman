// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

//! A real TOML parser supporting sections, inline tables, arrays of tables,
//! multiline strings, literal strings, various numeric formats, booleans,
//! date/time values, and comments.

/// A TOML value.
#[derive(Debug, Clone, PartialEq)]
pub enum TomlValue {
    /// A string value.
    Str(String),
    /// A 64-bit integer value.
    Int(i64),
    /// A 64-bit floating-point value.
    Float(f64),
    /// A boolean value.
    Bool(bool),
    /// A date/time value stored as its original string representation.
    DateTime(String),
    /// An array of TOML values.
    Array(Vec<TomlValue>),
    /// A table (ordered map of key-value pairs).
    Table(TomlTable),
}

/// An ordered map of key-value pairs representing a TOML table.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TomlTable {
    pub entries: Vec<(String, TomlValue)>,
}

impl TomlTable {
    /// Create an empty table.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a key-value pair (overwrites if key exists).
    pub fn insert(&mut self, key: impl Into<String>, value: TomlValue) {
        let key = key.into();
        for entry in &mut self.entries {
            if entry.0 == key {
                entry.1 = value;
                return;
            }
        }
        self.entries.push((key, value));
    }

    /// Look up a value by simple (non-dotted) key.
    pub fn get(&self, key: &str) -> Option<&TomlValue> {
        self.entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Look up a value by simple key (mutable).
    pub fn get_mut(&mut self, key: &str) -> Option<&mut TomlValue> {
        self.entries
            .iter_mut()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    /// Look up a value by dotted key path (e.g. "section.key").
    pub fn get_path(&self, path: &str) -> Option<&TomlValue> {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return None;
        }
        if parts.len() == 1 {
            return self.get(parts[0]);
        }
        let mut current = self;
        for &part in &parts[..parts.len() - 1] {
            match current.get(part) {
                Some(TomlValue::Table(t)) => current = t,
                _ => return None,
            }
        }
        current.get(parts[parts.len() - 1])
    }

    /// Return the number of top-level entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// A parsed TOML document backed by a flat entries list (backward compatible).
#[derive(Debug, Clone, Default)]
pub struct TomlDoc {
    pub entries: Vec<(String, TomlValue)>,
}

impl TomlDoc {
    /// Look up a value by dotted key path (e.g. "section.key").
    pub fn get(&self, path: &str) -> Option<&TomlValue> {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return None;
        }
        self.get_path_inner(&parts)
    }

    fn get_path_inner(&self, parts: &[&str]) -> Option<&TomlValue> {
        if parts.is_empty() {
            return None;
        }
        if parts.len() == 1 {
            return self
                .entries
                .iter()
                .find(|(k, _)| k == parts[0])
                .map(|(_, v)| v);
        }
        let first = parts[0];
        let val = self
            .entries
            .iter()
            .find(|(k, _)| k == first)
            .map(|(_, v)| v)?;
        match val {
            TomlValue::Table(t) => t.get_path(&parts[1..].join(".")),
            _ => None,
        }
    }
}

/// TOML parse error with line number.
#[derive(Debug, Clone, PartialEq)]
pub struct TomlParseError {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

impl std::fmt::Display for TomlParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TOML parse error at line {}:{}: {}",
            self.line, self.col, self.message
        )
    }
}

impl std::error::Error for TomlParseError {}

fn mk_err(line: usize, col: usize, msg: impl Into<String>) -> TomlParseError {
    TomlParseError {
        line,
        col,
        message: msg.into(),
    }
}

// ---------------------------------------------------------------------------
// Character-level parser state
// ---------------------------------------------------------------------------

struct Parser<'a> {
    input: &'a str,
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.get(self.pos).copied()?;
        self.pos += 1;
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    fn at_end(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn skip_ws(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_ws_and_nl(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        if self.peek() == Some('#') {
            while let Some(ch) = self.peek() {
                if ch == '\n' {
                    break;
                }
                self.advance();
            }
        }
    }

    fn skip_insignificant(&mut self) {
        loop {
            self.skip_ws_and_nl();
            if self.peek() == Some('#') {
                self.skip_comment();
            } else {
                break;
            }
        }
    }

    fn skip_line_tail(&mut self) {
        self.skip_ws();
        if self.peek() == Some('#') {
            self.skip_comment();
        }
    }

    fn expect(&mut self, expected: char) -> Result<(), TomlParseError> {
        match self.advance() {
            Some(ch) if ch == expected => Ok(()),
            Some(ch) => Err(mk_err(
                self.line,
                self.col,
                format!("expected '{}', found '{}'", expected, ch),
            )),
            None => Err(mk_err(
                self.line,
                self.col,
                format!("expected '{}', found end of input", expected),
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// Key parsing
// ---------------------------------------------------------------------------

fn parse_bare_key(p: &mut Parser) -> Result<String, TomlParseError> {
    let mut key = String::new();
    while let Some(ch) = p.peek() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            key.push(ch);
            p.advance();
        } else {
            break;
        }
    }
    if key.is_empty() {
        return Err(mk_err(p.line, p.col, "expected a key"));
    }
    Ok(key)
}

fn parse_key(p: &mut Parser) -> Result<String, TomlParseError> {
    match p.peek() {
        Some('"') => parse_basic_string(p),
        Some('\'') => parse_literal_string(p),
        _ => parse_bare_key(p),
    }
}

fn parse_dotted_key(p: &mut Parser) -> Result<Vec<String>, TomlParseError> {
    let mut parts = vec![parse_key(p)?];
    loop {
        p.skip_ws();
        if p.peek() == Some('.') {
            p.advance();
            p.skip_ws();
            parts.push(parse_key(p)?);
        } else {
            break;
        }
    }
    Ok(parts)
}

// ---------------------------------------------------------------------------
// String parsing
// ---------------------------------------------------------------------------

fn parse_basic_string(p: &mut Parser) -> Result<String, TomlParseError> {
    let line = p.line;
    let col = p.col;
    p.expect('"')?;

    if p.peek() == Some('"') && p.peek_at(1) == Some('"') {
        p.advance();
        p.advance();
        return parse_ml_basic_string(p, line, col);
    }

    let mut s = String::new();
    loop {
        match p.advance() {
            Some('"') => return Ok(s),
            Some('\\') => {
                let esc = parse_escape(p)?;
                s.push(esc);
            }
            Some('\n') | None => {
                return Err(mk_err(line, col, "unterminated basic string"));
            }
            Some(ch) => s.push(ch),
        }
    }
}

fn parse_ml_basic_string(
    p: &mut Parser,
    sl: usize,
    sc: usize,
) -> Result<String, TomlParseError> {
    // Skip first newline after opening """
    if p.peek() == Some('\n') {
        p.advance();
    } else if p.peek() == Some('\r') && p.peek_at(1) == Some('\n') {
        p.advance();
        p.advance();
    }

    let mut s = String::new();
    loop {
        match p.peek() {
            Some('"') if p.peek_at(1) == Some('"') && p.peek_at(2) == Some('"') => {
                p.advance();
                p.advance();
                p.advance();
                return Ok(s);
            }
            Some('\\') => {
                p.advance();
                if matches!(p.peek(), Some('\n') | Some('\r')) {
                    while let Some(ch) = p.peek() {
                        if ch == '\n' || ch == '\r' || ch == ' ' || ch == '\t' {
                            p.advance();
                        } else {
                            break;
                        }
                    }
                } else {
                    let esc = parse_escape(p)?;
                    s.push(esc);
                }
            }
            Some(_) => {
                if let Some(ch) = p.advance() {
                    s.push(ch);
                }
            }
            None => {
                return Err(mk_err(sl, sc, "unterminated multiline basic string"));
            }
        }
    }
}

fn parse_escape(p: &mut Parser) -> Result<char, TomlParseError> {
    let line = p.line;
    let col = p.col;
    match p.advance() {
        Some('b') => Ok('\u{0008}'),
        Some('t') => Ok('\t'),
        Some('n') => Ok('\n'),
        Some('f') => Ok('\u{000C}'),
        Some('r') => Ok('\r'),
        Some('"') => Ok('"'),
        Some('\\') => Ok('\\'),
        Some('u') => parse_unicode_esc(p, 4),
        Some('U') => parse_unicode_esc(p, 8),
        Some(ch) => Err(mk_err(line, col, format!("invalid escape: \\{}", ch))),
        None => Err(mk_err(line, col, "unexpected end of input in escape")),
    }
}

fn parse_unicode_esc(p: &mut Parser, digits: usize) -> Result<char, TomlParseError> {
    let line = p.line;
    let col = p.col;
    let mut hex = String::with_capacity(digits);
    for _ in 0..digits {
        match p.advance() {
            Some(ch) if ch.is_ascii_hexdigit() => hex.push(ch),
            _ => return Err(mk_err(line, col, "invalid unicode escape")),
        }
    }
    let code = u32::from_str_radix(&hex, 16)
        .map_err(|_| mk_err(line, col, "invalid unicode escape value"))?;
    char::from_u32(code).ok_or_else(|| mk_err(line, col, "invalid unicode code point"))
}

fn parse_literal_string(p: &mut Parser) -> Result<String, TomlParseError> {
    let line = p.line;
    let col = p.col;
    p.expect('\'')?;

    if p.peek() == Some('\'') && p.peek_at(1) == Some('\'') {
        p.advance();
        p.advance();
        return parse_ml_literal_string(p, line, col);
    }

    let mut s = String::new();
    loop {
        match p.advance() {
            Some('\'') => return Ok(s),
            Some('\n') | None => {
                return Err(mk_err(line, col, "unterminated literal string"));
            }
            Some(ch) => s.push(ch),
        }
    }
}

fn parse_ml_literal_string(
    p: &mut Parser,
    sl: usize,
    sc: usize,
) -> Result<String, TomlParseError> {
    if p.peek() == Some('\n') {
        p.advance();
    } else if p.peek() == Some('\r') && p.peek_at(1) == Some('\n') {
        p.advance();
        p.advance();
    }

    let mut s = String::new();
    loop {
        match p.peek() {
            Some('\'') if p.peek_at(1) == Some('\'') && p.peek_at(2) == Some('\'') => {
                p.advance();
                p.advance();
                p.advance();
                return Ok(s);
            }
            Some(_) => {
                if let Some(ch) = p.advance() {
                    s.push(ch);
                }
            }
            None => {
                return Err(mk_err(sl, sc, "unterminated multiline literal string"));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Value parsing
// ---------------------------------------------------------------------------

fn parse_value_full(p: &mut Parser) -> Result<TomlValue, TomlParseError> {
    p.skip_ws();
    let line = p.line;
    let col = p.col;

    match p.peek() {
        Some('"') => parse_basic_string(p).map(TomlValue::Str),
        Some('\'') => parse_literal_string(p).map(TomlValue::Str),
        Some('[') => parse_array_val(p),
        Some('{') => parse_inline_table(p),
        Some('t') => parse_bool_true(p),
        Some('f') => parse_bool_false(p),
        Some('i') => parse_kw_inf(p, 1.0),
        Some('n') => parse_kw_nan(p, 1.0),
        Some('+') | Some('-') => parse_signed(p),
        Some(ch) if ch.is_ascii_digit() => parse_num_or_dt(p),
        _ => Err(mk_err(line, col, "unexpected value")),
    }
}

fn parse_bool_true(p: &mut Parser) -> Result<TomlValue, TomlParseError> {
    let line = p.line;
    let col = p.col;
    for expected in ['t', 'r', 'u', 'e'] {
        match p.advance() {
            Some(ch) if ch == expected => {}
            _ => return Err(mk_err(line, col, "invalid boolean")),
        }
    }
    Ok(TomlValue::Bool(true))
}

fn parse_bool_false(p: &mut Parser) -> Result<TomlValue, TomlParseError> {
    let line = p.line;
    let col = p.col;
    for expected in ['f', 'a', 'l', 's', 'e'] {
        match p.advance() {
            Some(ch) if ch == expected => {}
            _ => return Err(mk_err(line, col, "invalid boolean")),
        }
    }
    Ok(TomlValue::Bool(false))
}

fn parse_kw_inf(p: &mut Parser, sign: f64) -> Result<TomlValue, TomlParseError> {
    let line = p.line;
    let col = p.col;
    for expected in ['i', 'n', 'f'] {
        match p.advance() {
            Some(ch) if ch == expected => {}
            _ => return Err(mk_err(line, col, "invalid value")),
        }
    }
    Ok(TomlValue::Float(sign * f64::INFINITY))
}

fn parse_kw_nan(p: &mut Parser, _sign: f64) -> Result<TomlValue, TomlParseError> {
    let line = p.line;
    let col = p.col;
    for expected in ['n', 'a', 'n'] {
        match p.advance() {
            Some(ch) if ch == expected => {}
            _ => return Err(mk_err(line, col, "invalid value")),
        }
    }
    Ok(TomlValue::Float(f64::NAN))
}

fn parse_signed(p: &mut Parser) -> Result<TomlValue, TomlParseError> {
    let sign_ch = p.advance().ok_or_else(|| mk_err(p.line, p.col, "unexpected eof"))?;
    let sign: f64 = if sign_ch == '-' { -1.0 } else { 1.0 };

    if p.peek() == Some('i') {
        return parse_kw_inf(p, sign);
    }
    if p.peek() == Some('n') {
        return parse_kw_nan(p, sign);
    }

    let num_str = collect_num_chars(p);
    let full = format!("{}{}", sign_ch, num_str);
    parse_num_string(&full, p.line, p.col)
}

fn collect_num_chars(p: &mut Parser) -> String {
    let mut s = String::new();
    while let Some(ch) = p.peek() {
        if ch.is_ascii_alphanumeric()
            || ch == '.'
            || ch == '_'
            || ch == '-'
            || ch == '+'
            || ch == ':'
            || ch == 'e'
            || ch == 'E'
            || ch == 'T'
            || ch == 'Z'
        {
            s.push(ch);
            p.advance();
        } else {
            break;
        }
    }
    s
}

fn parse_num_or_dt(p: &mut Parser) -> Result<TomlValue, TomlParseError> {
    let line = p.line;
    let col = p.col;
    let s = collect_num_chars(p);
    if looks_like_datetime(&s) {
        return Ok(TomlValue::DateTime(s));
    }
    parse_num_string(&s, line, col)
}

fn looks_like_datetime(s: &str) -> bool {
    let clean = s.replace('_', "");
    if clean.len() >= 10 {
        let bytes = clean.as_bytes();
        if bytes.len() >= 10
            && bytes[4] == b'-'
            && bytes[7] == b'-'
            && bytes[0].is_ascii_digit()
            && bytes[5].is_ascii_digit()
            && bytes[8].is_ascii_digit()
        {
            return true;
        }
    }
    if clean.len() >= 8 && clean.contains(':') {
        let parts: Vec<&str> = clean.split(':').collect();
        if parts.len() >= 2 && parts[0].len() == 2 && parts[0].chars().all(|c| c.is_ascii_digit())
        {
            return true;
        }
    }
    false
}

fn parse_num_string(s: &str, line: usize, col: usize) -> Result<TomlValue, TomlParseError> {
    let clean = s.replace('_', "");

    if clean.is_empty() {
        return Err(mk_err(line, col, "empty number"));
    }

    // Hex
    if clean.starts_with("0x") || clean.starts_with("0X") {
        let val = i64::from_str_radix(&clean[2..], 16)
            .map_err(|e| mk_err(line, col, format!("invalid hex integer: {}", e)))?;
        return Ok(TomlValue::Int(val));
    }
    // Octal
    if clean.starts_with("0o") || clean.starts_with("0O") {
        let val = i64::from_str_radix(&clean[2..], 8)
            .map_err(|e| mk_err(line, col, format!("invalid octal integer: {}", e)))?;
        return Ok(TomlValue::Int(val));
    }
    // Binary
    if clean.starts_with("0b") || clean.starts_with("0B") {
        let val = i64::from_str_radix(&clean[2..], 2)
            .map_err(|e| mk_err(line, col, format!("invalid binary integer: {}", e)))?;
        return Ok(TomlValue::Int(val));
    }

    // Float (contains '.', 'e', or 'E')
    if clean.contains('.') || clean.contains('e') || clean.contains('E') {
        let val: f64 = clean
            .parse()
            .map_err(|e| mk_err(line, col, format!("invalid float: {}", e)))?;
        return Ok(TomlValue::Float(val));
    }

    // Integer
    let val: i64 = clean
        .parse()
        .map_err(|e| mk_err(line, col, format!("invalid integer: {}", e)))?;
    Ok(TomlValue::Int(val))
}

fn parse_array_val(p: &mut Parser) -> Result<TomlValue, TomlParseError> {
    p.expect('[')?;
    let mut items = Vec::new();
    loop {
        p.skip_insignificant();
        if p.peek() == Some(']') {
            p.advance();
            return Ok(TomlValue::Array(items));
        }
        let val = parse_value_full(p)?;
        items.push(val);
        p.skip_insignificant();
        match p.peek() {
            Some(',') => {
                p.advance();
            }
            Some(']') => {}
            _ => {
                return Err(mk_err(p.line, p.col, "expected ',' or ']' in array"));
            }
        }
    }
}

fn parse_inline_table(p: &mut Parser) -> Result<TomlValue, TomlParseError> {
    p.expect('{')?;
    let mut table = TomlTable::new();
    p.skip_ws();
    if p.peek() == Some('}') {
        p.advance();
        return Ok(TomlValue::Table(table));
    }
    loop {
        p.skip_ws();
        let key_parts = parse_dotted_key(p)?;
        p.skip_ws();
        p.expect('=')?;
        p.skip_ws();
        let val = parse_value_full(p)?;
        insert_dotted(&mut table, &key_parts, val, p.line)?;
        p.skip_ws();
        match p.peek() {
            Some(',') => {
                p.advance();
            }
            Some('}') => {
                p.advance();
                return Ok(TomlValue::Table(table));
            }
            _ => {
                return Err(mk_err(p.line, p.col, "expected ',' or '}' in inline table"));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Table insertion helpers
// ---------------------------------------------------------------------------

fn insert_dotted(
    table: &mut TomlTable,
    parts: &[String],
    value: TomlValue,
    line: usize,
) -> Result<(), TomlParseError> {
    if parts.is_empty() {
        return Err(mk_err(line, 0, "empty key"));
    }
    if parts.len() == 1 {
        table.insert(parts[0].clone(), value);
        return Ok(());
    }
    let first = &parts[0];
    let existing = table.get_mut(first);
    match existing {
        Some(TomlValue::Table(sub)) => {
            insert_dotted(sub, &parts[1..], value, line)?;
        }
        None => {
            let mut sub = TomlTable::new();
            insert_dotted(&mut sub, &parts[1..], value, line)?;
            table.insert(first.clone(), TomlValue::Table(sub));
        }
        _ => {
            return Err(mk_err(line, 0, format!("key '{}' is not a table", first)));
        }
    }
    Ok(())
}

fn ensure_table_path<'a>(
    root: &'a mut TomlTable,
    parts: &[String],
    line: usize,
) -> Result<&'a mut TomlTable, TomlParseError> {
    let mut current = root;
    for part in parts {
        let exists = current.get(part).is_some();
        if !exists {
            current.insert(part.clone(), TomlValue::Table(TomlTable::new()));
        }
        let entry = current.get_mut(part);
        match entry {
            Some(TomlValue::Table(t)) => current = t,
            _ => {
                return Err(mk_err(
                    line,
                    0,
                    format!("key '{}' conflicts with existing non-table value", part),
                ));
            }
        }
    }
    Ok(current)
}

fn ensure_aot_path<'a>(
    root: &'a mut TomlTable,
    parts: &[String],
    line: usize,
) -> Result<&'a mut TomlTable, TomlParseError> {
    if parts.is_empty() {
        return Err(mk_err(line, 0, "empty array-of-tables header"));
    }
    let (prefix, last) = parts.split_at(parts.len() - 1);
    let last_key = &last[0];
    let parent = ensure_table_path(root, prefix, line)?;
    let exists = parent.get(last_key).is_some();
    if !exists {
        parent.insert(last_key.clone(), TomlValue::Array(Vec::new()));
    }
    match parent.get_mut(last_key) {
        Some(TomlValue::Array(arr)) => {
            arr.push(TomlValue::Table(TomlTable::new()));
            let idx = arr.len() - 1;
            match &mut arr[idx] {
                TomlValue::Table(t) => Ok(t),
                _ => Err(mk_err(line, 0, "internal error")),
            }
        }
        _ => Err(mk_err(
            line,
            0,
            format!("key '{}' is not an array of tables", last_key),
        )),
    }
}

fn get_last_aot<'a>(
    root: &'a mut TomlTable,
    parts: &[String],
    line: usize,
) -> Result<&'a mut TomlTable, TomlParseError> {
    if parts.is_empty() {
        return Err(mk_err(line, 0, "empty path"));
    }
    let (prefix, last) = parts.split_at(parts.len() - 1);
    let last_key = &last[0];
    let parent = ensure_table_path(root, prefix, line)?;
    match parent.get_mut(last_key) {
        Some(TomlValue::Array(arr)) => {
            let len = arr.len();
            if len == 0 {
                return Err(mk_err(line, 0, "array of tables is empty"));
            }
            match &mut arr[len - 1] {
                TomlValue::Table(t) => Ok(t),
                _ => Err(mk_err(line, 0, "last element is not a table")),
            }
        }
        _ => Err(mk_err(
            line,
            0,
            format!("key '{}' is not an array of tables", last_key),
        )),
    }
}

// ---------------------------------------------------------------------------
// Section header parsing
// ---------------------------------------------------------------------------

enum SectionKind {
    Table(Vec<String>),
    ArrayOfTables(Vec<String>),
}

fn parse_section_header(p: &mut Parser) -> Result<SectionKind, TomlParseError> {
    p.expect('[')?;
    let is_aot = p.peek() == Some('[');
    if is_aot {
        p.advance();
    }
    p.skip_ws();
    let parts = parse_dotted_key(p)?;
    p.skip_ws();
    if is_aot {
        p.expect(']')?;
    }
    p.expect(']')?;
    if is_aot {
        Ok(SectionKind::ArrayOfTables(parts))
    } else {
        Ok(SectionKind::Table(parts))
    }
}

// ---------------------------------------------------------------------------
// Top-level document parser
// ---------------------------------------------------------------------------

/// Parse a TOML string into a `TomlDoc` (full parser).
///
/// Supports sections, inline tables, arrays of tables, multiline strings,
/// literal strings, various numeric formats, booleans, date/time values,
/// comments, and dotted keys.
pub fn parse_toml_full(input: &str) -> Result<TomlDoc, TomlParseError> {
    let mut p = Parser::new(input);
    let mut root = TomlTable::new();
    let mut current_path: Vec<String> = Vec::new();
    let mut is_aot = false;

    loop {
        p.skip_insignificant();
        if p.at_end() {
            break;
        }

        match p.peek() {
            Some('[') => {
                let header = parse_section_header(&mut p)?;
                p.skip_line_tail();
                if p.peek() == Some('\n') {
                    p.advance();
                } else if p.peek() == Some('\r') {
                    p.advance();
                    if p.peek() == Some('\n') {
                        p.advance();
                    }
                }
                match header {
                    SectionKind::Table(parts) => {
                        current_path = parts.clone();
                        is_aot = false;
                        ensure_table_path(&mut root, &parts, p.line)?;
                    }
                    SectionKind::ArrayOfTables(parts) => {
                        current_path = parts.clone();
                        is_aot = true;
                        ensure_aot_path(&mut root, &parts, p.line)?;
                    }
                }
            }
            Some('#') => {
                p.skip_comment();
            }
            Some(_) => {
                let key_parts = parse_dotted_key(&mut p)?;
                p.skip_ws();
                p.expect('=')?;
                p.skip_ws();
                let val = parse_value_full(&mut p)?;
                p.skip_line_tail();

                if current_path.is_empty() {
                    insert_dotted(&mut root, &key_parts, val, p.line)?;
                } else if is_aot {
                    let target = get_last_aot(&mut root, &current_path, p.line)?;
                    insert_dotted(target, &key_parts, val, p.line)?;
                } else {
                    let target = ensure_table_path(&mut root, &current_path, p.line)?;
                    insert_dotted(target, &key_parts, val, p.line)?;
                }
            }
            None => break,
        }
    }

    Ok(TomlDoc {
        entries: root.entries,
    })
}

// ---------------------------------------------------------------------------
// Backward-compatible simple parser
// ---------------------------------------------------------------------------

/// Parse a simple `key = value` text format into a `TomlDoc`.
///
/// Delegates to the full parser; falls back to line-by-line on error.
pub fn parse_toml_simple(text: &str) -> TomlDoc {
    match parse_toml_full(text) {
        Ok(doc) => doc,
        Err(_) => {
            let mut doc = TomlDoc::default();
            for line in text.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some(eq_pos) = line.find('=') {
                    let key = line[..eq_pos].trim().to_string();
                    let val_str = line[eq_pos + 1..].trim();
                    let value = parse_value_simple_fallback(val_str);
                    doc.entries.push((key, value));
                }
            }
            doc
        }
    }
}

fn parse_value_simple_fallback(s: &str) -> TomlValue {
    if s == "true" {
        return TomlValue::Bool(true);
    }
    if s == "false" {
        return TomlValue::Bool(false);
    }
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        return TomlValue::Str(s[1..s.len() - 1].to_string());
    }
    if s.starts_with('[') && s.ends_with(']') {
        let inner = &s[1..s.len() - 1];
        let items: Vec<TomlValue> = inner
            .split(',')
            .map(|item| parse_value_simple_fallback(item.trim()))
            .collect();
        return TomlValue::Array(items);
    }
    if s.contains('.') {
        if let Ok(f) = s.parse::<f64>() {
            return TomlValue::Float(f);
        }
    }
    if let Ok(i) = s.parse::<i64>() {
        return TomlValue::Int(i);
    }
    TomlValue::Str(s.to_string())
}

// ---------------------------------------------------------------------------
// Public accessor helpers (backward compatible)
// ---------------------------------------------------------------------------

/// Get an integer value by key.
pub fn toml_get_int(doc: &TomlDoc, key: &str) -> Option<i64> {
    doc.entries.iter().find(|(k, _)| k == key).and_then(|(_, v)| {
        if let TomlValue::Int(i) = v {
            Some(*i)
        } else {
            None
        }
    })
}

/// Get a float value by key.
pub fn toml_get_float(doc: &TomlDoc, key: &str) -> Option<f64> {
    doc.entries.iter().find(|(k, _)| k == key).and_then(|(_, v)| {
        if let TomlValue::Float(f) = v {
            Some(*f)
        } else {
            None
        }
    })
}

/// Get a string value by key.
pub fn toml_get_str<'a>(doc: &'a TomlDoc, key: &str) -> Option<&'a str> {
    doc.entries.iter().find(|(k, _)| k == key).and_then(|(_, v)| {
        if let TomlValue::Str(s) = v {
            Some(s.as_str())
        } else {
            None
        }
    })
}

/// Get a boolean value by key.
pub fn toml_get_bool(doc: &TomlDoc, key: &str) -> Option<bool> {
    doc.entries.iter().find(|(k, _)| k == key).and_then(|(_, v)| {
        if let TomlValue::Bool(b) = v {
            Some(*b)
        } else {
            None
        }
    })
}

/// Get the number of entries in the document.
pub fn entry_count(doc: &TomlDoc) -> usize {
    doc.entries.len()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Original backward-compatibility tests ---

    #[test]
    fn parse_int() {
        let doc = parse_toml_simple("age = 42");
        assert_eq!(toml_get_int(&doc, "age"), Some(42));
    }

    #[test]
    fn parse_float() {
        let doc = parse_toml_simple("ratio = 1.5");
        assert!((toml_get_float(&doc, "ratio").unwrap() - 1.5).abs() < 1e-9);
    }

    #[test]
    fn parse_bool_true() {
        let doc = parse_toml_simple("enabled = true");
        assert_eq!(toml_get_bool(&doc, "enabled"), Some(true));
    }

    #[test]
    fn parse_bool_false() {
        let doc = parse_toml_simple("debug = false");
        assert_eq!(toml_get_bool(&doc, "debug"), Some(false));
    }

    #[test]
    fn parse_string() {
        let doc = parse_toml_simple(r#"name = "Alice""#);
        assert_eq!(toml_get_str(&doc, "name"), Some("Alice"));
    }

    #[test]
    fn missing_key_returns_none() {
        let doc = parse_toml_simple("x = 1");
        assert_eq!(toml_get_int(&doc, "y"), None);
    }

    #[test]
    fn entry_count_correct() {
        let doc = parse_toml_simple("a = 1\nb = 2\nc = 3");
        assert_eq!(entry_count(&doc), 3);
    }

    #[test]
    fn comment_ignored() {
        let doc = parse_toml_simple("# comment\nval = 7");
        assert_eq!(toml_get_int(&doc, "val"), Some(7));
        assert_eq!(entry_count(&doc), 1);
    }

    #[test]
    fn empty_input() {
        let doc = parse_toml_simple("");
        assert_eq!(entry_count(&doc), 0);
    }

    #[test]
    fn parse_array() {
        let doc = parse_toml_simple("nums = [1, 2, 3]");
        let entry = doc.entries.iter().find(|(k, _)| k == "nums");
        assert!(entry.is_some());
        if let Some((_, TomlValue::Array(items))) = entry {
            assert_eq!(items.len(), 3);
        }
    }

    // --- Full parser tests ---

    #[test]
    fn test_section_header() {
        let input = "[server]\nport = 8080\nhost = \"localhost\"\n";
        let doc = parse_toml_full(input).unwrap();
        let server = doc.get("server");
        assert!(server.is_some());
        if let Some(TomlValue::Table(t)) = server {
            assert_eq!(t.get("port"), Some(&TomlValue::Int(8080)));
            assert_eq!(
                t.get("host"),
                Some(&TomlValue::Str("localhost".to_string()))
            );
        } else {
            panic!("expected table");
        }
    }

    #[test]
    fn test_nested_section() {
        let input = "[database.connection]\ntimeout = 30\n";
        let doc = parse_toml_full(input).unwrap();
        let val = doc.get("database.connection.timeout");
        assert_eq!(val, Some(&TomlValue::Int(30)));
    }

    #[test]
    fn test_inline_table() {
        let input = "point = { x = 1, y = 2 }\n";
        let doc = parse_toml_full(input).unwrap();
        if let Some(TomlValue::Table(t)) = doc.get("point") {
            assert_eq!(t.get("x"), Some(&TomlValue::Int(1)));
            assert_eq!(t.get("y"), Some(&TomlValue::Int(2)));
        } else {
            panic!("expected inline table");
        }
    }

    #[test]
    fn test_array_of_tables() {
        let input = "[[products]]\nname = \"Hammer\"\n\n[[products]]\nname = \"Nail\"\n";
        let doc = parse_toml_full(input).unwrap();
        if let Some(TomlValue::Array(arr)) = doc.get("products") {
            assert_eq!(arr.len(), 2);
            if let TomlValue::Table(t) = &arr[0] {
                assert_eq!(t.get("name"), Some(&TomlValue::Str("Hammer".to_string())));
            }
            if let TomlValue::Table(t) = &arr[1] {
                assert_eq!(t.get("name"), Some(&TomlValue::Str("Nail".to_string())));
            }
        } else {
            panic!("expected array of tables");
        }
    }

    #[test]
    fn test_multiline_basic_string() {
        let input = "bio = \"\"\"\nRoses are red\nViolets are blue\"\"\"\n";
        let doc = parse_toml_full(input).unwrap();
        assert_eq!(
            doc.get("bio"),
            Some(&TomlValue::Str("Roses are red\nViolets are blue".to_string()))
        );
    }

    #[test]
    fn test_multiline_literal_string() {
        let input = "regex = '''\n\\d{2,3}\n'''\n";
        let doc = parse_toml_full(input).unwrap();
        assert_eq!(
            doc.get("regex"),
            Some(&TomlValue::Str("\\d{2,3}\n".to_string()))
        );
    }

    #[test]
    fn test_literal_string() {
        let input = "path = 'C:\\Users\\admin'\n";
        let doc = parse_toml_full(input).unwrap();
        assert_eq!(
            doc.get("path"),
            Some(&TomlValue::Str("C:\\Users\\admin".to_string()))
        );
    }

    #[test]
    fn test_hex_integer() {
        let input = "color = 0xFF00FF\n";
        let doc = parse_toml_full(input).unwrap();
        assert_eq!(doc.get("color"), Some(&TomlValue::Int(0xFF00FF)));
    }

    #[test]
    fn test_octal_integer() {
        let input = "perm = 0o755\n";
        let doc = parse_toml_full(input).unwrap();
        assert_eq!(doc.get("perm"), Some(&TomlValue::Int(0o755)));
    }

    #[test]
    fn test_binary_integer() {
        let input = "mask = 0b11010110\n";
        let doc = parse_toml_full(input).unwrap();
        assert_eq!(doc.get("mask"), Some(&TomlValue::Int(0b11010110)));
    }

    #[test]
    fn test_underscore_integer() {
        let input = "big = 1_000_000\n";
        let doc = parse_toml_full(input).unwrap();
        assert_eq!(doc.get("big"), Some(&TomlValue::Int(1_000_000)));
    }

    #[test]
    fn test_float_exponent() {
        let input = "sci = 5e+22\n";
        let doc = parse_toml_full(input).unwrap();
        if let Some(TomlValue::Float(f)) = doc.get("sci") {
            assert!((f - 5e22).abs() < 1e16);
        } else {
            panic!("expected float");
        }
    }

    #[test]
    fn test_inf_nan() {
        let input = "a = inf\nb = -inf\nc = nan\n";
        let doc = parse_toml_full(input).unwrap();
        if let Some(TomlValue::Float(f)) = doc.get("a") {
            assert!(f.is_infinite() && f.is_sign_positive());
        } else {
            panic!("expected +inf");
        }
        if let Some(TomlValue::Float(f)) = doc.get("b") {
            assert!(f.is_infinite() && f.is_sign_negative());
        } else {
            panic!("expected -inf");
        }
        if let Some(TomlValue::Float(f)) = doc.get("c") {
            assert!(f.is_nan());
        } else {
            panic!("expected nan");
        }
    }

    #[test]
    fn test_datetime() {
        let input = "created = 2024-01-15T10:30:00Z\n";
        let doc = parse_toml_full(input).unwrap();
        assert_eq!(
            doc.get("created"),
            Some(&TomlValue::DateTime("2024-01-15T10:30:00Z".to_string()))
        );
    }

    #[test]
    fn test_dotted_key() {
        let input = "fruit.apple.color = \"red\"\n";
        let doc = parse_toml_full(input).unwrap();
        let val = doc.get("fruit.apple.color");
        assert_eq!(val, Some(&TomlValue::Str("red".to_string())));
    }

    #[test]
    fn test_nested_array() {
        let input = "data = [[1, 2], [3, 4]]\n";
        let doc = parse_toml_full(input).unwrap();
        if let Some(TomlValue::Array(outer)) = doc.get("data") {
            assert_eq!(outer.len(), 2);
            if let TomlValue::Array(inner) = &outer[0] {
                assert_eq!(inner.len(), 2);
            }
        } else {
            panic!("expected nested array");
        }
    }

    #[test]
    fn test_error_line_number() {
        let input = "a = 1\nb = 2\nc = @invalid\n";
        let result = parse_toml_full(input);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.line, 3);
        }
    }

    #[test]
    fn test_escape_sequences() {
        let input = r#"msg = "hello\nworld\t!""#;
        let doc = parse_toml_full(input).unwrap();
        assert_eq!(
            doc.get("msg"),
            Some(&TomlValue::Str("hello\nworld\t!".to_string()))
        );
    }

    #[test]
    fn test_empty_inline_table() {
        let input = "empty = {}\n";
        let doc = parse_toml_full(input).unwrap();
        if let Some(TomlValue::Table(t)) = doc.get("empty") {
            assert!(t.is_empty());
        } else {
            panic!("expected empty table");
        }
    }

    #[test]
    fn test_mixed_doc() {
        let input = r#"
title = "Test"

[owner]
name = "Tom"
age = 30

[database]
ports = [8001, 8001, 8002]
enabled = true
"#;
        let doc = parse_toml_full(input).unwrap();
        assert_eq!(doc.get("title"), Some(&TomlValue::Str("Test".to_string())));
        assert_eq!(
            doc.get("owner.name"),
            Some(&TomlValue::Str("Tom".to_string()))
        );
        assert_eq!(doc.get("owner.age"), Some(&TomlValue::Int(30)));
        assert_eq!(doc.get("database.enabled"), Some(&TomlValue::Bool(true)));
        if let Some(TomlValue::Array(ports)) = doc.get("database.ports") {
            assert_eq!(ports.len(), 3);
        } else {
            panic!("expected ports array");
        }
    }

    #[test]
    fn test_toml_table_get_path() {
        let mut root = TomlTable::new();
        let mut sub = TomlTable::new();
        sub.insert("key", TomlValue::Int(42));
        root.insert("section", TomlValue::Table(sub));
        assert_eq!(root.get_path("section.key"), Some(&TomlValue::Int(42)));
    }

    #[test]
    fn test_negative_integer() {
        let input = "temp = -40\n";
        let doc = parse_toml_full(input).unwrap();
        assert_eq!(doc.get("temp"), Some(&TomlValue::Int(-40)));
    }

    #[test]
    fn test_positive_float() {
        let input = "val = +3.14\n";
        let doc = parse_toml_full(input).unwrap();
        if let Some(TomlValue::Float(f)) = doc.get("val") {
            assert!((f - 3.14).abs() < 1e-9);
        } else {
            panic!("expected float");
        }
    }

    #[test]
    fn test_date_only() {
        let input = "day = 2024-06-15\n";
        let doc = parse_toml_full(input).unwrap();
        assert_eq!(
            doc.get("day"),
            Some(&TomlValue::DateTime("2024-06-15".to_string()))
        );
    }

    #[test]
    fn test_quoted_key() {
        let input = r#""weird key" = 1"#;
        let doc = parse_toml_full(input).unwrap();
        assert_eq!(doc.get("weird key"), Some(&TomlValue::Int(1)));
    }

    #[test]
    fn test_inline_comment_after_value() {
        let input = "x = 42 # this is a comment\n";
        let doc = parse_toml_full(input).unwrap();
        assert_eq!(doc.get("x"), Some(&TomlValue::Int(42)));
    }
}
