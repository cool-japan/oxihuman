// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
//! Lua 5.4-compatible tree-walk interpreter for plugin and automation support.
//! Implements lexer, recursive-descent parser, and evaluator with built-ins.
//! The evaluator is split into lua_interp.rs included below.

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::time::Instant;

// ===== Public Types =====

/// A Lua value, including tables and functions.
#[derive(Debug, Clone)]
pub enum LuaValue {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Table(Rc<RefCell<HashMap<TableKey, LuaValue>>>),
    Function(Rc<LuaFunction>),
}

/// Hash-safe table key (Nil cannot be a key in Lua).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TableKey {
    Bool(bool),
    Int(i64),
    /// Float stored as `to_bits()` for consistent Hash/Eq — NaN == NaN.
    Float(u64),
    Str(String),
    /// Table identity by pointer.
    TablePtr(usize),
    /// Function identity by pointer.
    FuncPtr(usize),
}

impl TableKey {
    pub fn from_value(v: &LuaValue) -> Option<TableKey> {
        match v {
            LuaValue::Nil => None,
            LuaValue::Bool(b) => Some(TableKey::Bool(*b)),
            LuaValue::Int(i) => Some(TableKey::Int(*i)),
            LuaValue::Float(f) => Some(TableKey::Float(f.to_bits())),
            LuaValue::Str(s) => Some(TableKey::Str(s.clone())),
            LuaValue::Table(rc) => Some(TableKey::TablePtr(Rc::as_ptr(rc) as usize)),
            LuaValue::Function(rc) => Some(TableKey::FuncPtr(Rc::as_ptr(rc) as usize)),
        }
    }
}

impl PartialEq for LuaValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LuaValue::Nil, LuaValue::Nil) => true,
            (LuaValue::Bool(a), LuaValue::Bool(b)) => a == b,
            (LuaValue::Int(a), LuaValue::Int(b)) => a == b,
            (LuaValue::Float(a), LuaValue::Float(b)) => a.to_bits() == b.to_bits(),
            (LuaValue::Str(a), LuaValue::Str(b)) => a == b,
            (LuaValue::Table(a), LuaValue::Table(b)) => Rc::ptr_eq(a, b),
            (LuaValue::Function(a), LuaValue::Function(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl Eq for LuaValue {}

impl Hash for LuaValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            LuaValue::Nil => 0u8.hash(state),
            LuaValue::Bool(b) => {
                1u8.hash(state);
                b.hash(state);
            }
            LuaValue::Int(i) => {
                2u8.hash(state);
                i.hash(state);
            }
            LuaValue::Float(f) => {
                3u8.hash(state);
                f.to_bits().hash(state);
            }
            LuaValue::Str(s) => {
                4u8.hash(state);
                s.hash(state);
            }
            LuaValue::Table(rc) => {
                5u8.hash(state);
                (Rc::as_ptr(rc) as usize).hash(state);
            }
            LuaValue::Function(rc) => {
                6u8.hash(state);
                (Rc::as_ptr(rc) as usize).hash(state);
            }
        }
    }
}

/// Lua function representation (opaque to external users).
#[derive(Debug)]
#[non_exhaustive]
pub enum LuaFunction {
    #[doc(hidden)]
    Closure(LuaClosure),
    #[doc(hidden)]
    Builtin(String),
}

/// Closure data — internal; fields are crate-accessible only.
#[derive(Debug)]
pub struct LuaClosure {
    pub(crate) params: Vec<String>,
    pub(crate) body: Vec<Stmt>,
    pub(crate) env: Rc<RefCell<Env>>,
}

/// Interpreter configuration.
#[derive(Debug, Clone)]
pub struct LuaConfig {
    pub max_stack_depth: usize,
    pub timeout_ms: u64,
    pub sandbox: bool,
}

/// A Lua script with source text and a name for error messages.
#[derive(Debug, Clone)]
pub struct LuaScript {
    pub source: String,
    pub name: String,
}

/// Result of executing a Lua script.
#[derive(Debug, Clone)]
pub struct LuaResult {
    pub return_values: Vec<LuaValue>,
    pub error: Option<String>,
    pub executed: bool,
    /// Captured output from `print(...)` calls.
    pub print_output: Vec<String>,
}

/// Main stub holding configuration and interpreter state.
pub struct LuaStub {
    pub config: LuaConfig,
    pub globals: HashMap<String, LuaValue>,
    pub call_count: u64,
    /// Output collected from print() calls during last execution.
    pub last_print_output: Vec<String>,
}

// ===== Internal types used by evaluator =====

/// Execution signal — break out of a block with a reason.
#[derive(Debug)]
pub(crate) enum Signal {
    None,
    Return(Vec<LuaValue>),
    Break,
}

/// Lexical environment.
#[derive(Debug)]
pub(crate) struct Env {
    pub(crate) vars: HashMap<String, LuaValue>,
    pub(crate) parent: Option<Rc<RefCell<Env>>>,
}

/// The tree-walk interpreter state.
pub(crate) struct Interpreter {
    max_stack_depth: usize,
    timeout_ms: u64,
    start_time: Instant,
    call_depth: usize,
    stmt_count: u64,
    print_buf: Rc<RefCell<Vec<String>>>,
}

// ===== Public API =====

/// Create default configuration.
pub fn default_lua_config() -> LuaConfig {
    LuaConfig {
        max_stack_depth: 200,
        timeout_ms: 5000,
        sandbox: true,
    }
}

/// Create a new stub with given configuration.
pub fn new_lua_stub(cfg: LuaConfig) -> LuaStub {
    LuaStub {
        config: cfg,
        globals: HashMap::new(),
        call_count: 0,
        last_print_output: Vec::new(),
    }
}

/// Set a global variable.
pub fn lua_set_global(stub: &mut LuaStub, name: &str, val: LuaValue) {
    stub.globals.insert(name.to_string(), val);
}

/// Get a global variable.
pub fn lua_get_global<'a>(stub: &'a LuaStub, name: &str) -> Option<&'a LuaValue> {
    stub.globals.get(name)
}

/// Count globals.
pub fn lua_global_count(stub: &LuaStub) -> usize {
    stub.globals.len()
}

/// Execute a Lua script.
pub fn lua_execute(stub: &mut LuaStub, script: &LuaScript) -> LuaResult {
    stub.call_count += 1;

    let tokens = match lex(&script.source) {
        Ok(t) => t,
        Err(e) => {
            return LuaResult {
                return_values: Vec::new(),
                error: Some(format!("[{}] lexer: {}", script.name, e)),
                executed: false,
                print_output: Vec::new(),
            };
        }
    };

    let stmts = match parse(&tokens) {
        Ok(s) => s,
        Err(e) => {
            return LuaResult {
                return_values: Vec::new(),
                error: Some(format!("[{}] parse: {}", script.name, e)),
                executed: false,
                print_output: Vec::new(),
            };
        }
    };

    let print_buf: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let env = lua_interp::build_global_env(&stub.globals, Rc::clone(&print_buf));
    let mut interp = Interpreter {
        max_stack_depth: stub.config.max_stack_depth,
        timeout_ms: stub.config.timeout_ms,
        start_time: Instant::now(),
        call_depth: 0,
        stmt_count: 0,
        print_buf: Rc::clone(&print_buf),
    };

    let result = interp.exec_block(&stmts, &env);
    let captured = print_buf.borrow().clone();
    stub.last_print_output = captured.clone();

    match result {
        Ok(sig) => {
            let return_values = match sig {
                Signal::Return(vals) => vals,
                _ => Vec::new(),
            };
            LuaResult {
                return_values,
                error: None,
                executed: true,
                print_output: captured,
            }
        }
        Err(e) => LuaResult {
            return_values: Vec::new(),
            error: Some(format!("[{}] runtime: {}", script.name, e)),
            executed: true,
            print_output: captured,
        },
    }
}

/// Create a new script.
pub fn new_lua_script(name: &str, source: &str) -> LuaScript {
    LuaScript {
        source: source.to_string(),
        name: name.to_string(),
    }
}

/// Return the Lua type name for a value.
pub fn lua_value_type_name(v: &LuaValue) -> &'static str {
    match v {
        LuaValue::Nil => "nil",
        LuaValue::Bool(_) => "boolean",
        LuaValue::Int(_) => "integer",
        LuaValue::Float(_) => "float",
        LuaValue::Str(_) => "string",
        LuaValue::Table(_) => "table",
        LuaValue::Function(_) => "function",
    }
}

/// Serialize a value to JSON.
pub fn lua_value_to_json(v: &LuaValue) -> String {
    lua_value_to_json_depth(v, 0)
}

fn lua_value_to_json_depth(v: &LuaValue, depth: usize) -> String {
    if depth > 16 {
        return "\"[max depth]\"".to_string();
    }
    match v {
        LuaValue::Nil => "null".to_string(),
        LuaValue::Bool(b) => format!("{}", b),
        LuaValue::Int(i) => format!("{}", i),
        LuaValue::Float(f) => format!("{}", f),
        LuaValue::Str(s) => {
            format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
        }
        LuaValue::Table(rc) => {
            let map = rc.borrow();
            let entries: Vec<String> = map
                .iter()
                .map(|(k, val)| {
                    let key_str = match k {
                        TableKey::Int(i) => format!("\"{}\"", i),
                        TableKey::Str(s) => format!("\"{}\"", s),
                        TableKey::Bool(b) => format!("\"{}\"", b),
                        TableKey::Float(bits) => {
                            format!("\"{}\"", f64::from_bits(*bits))
                        }
                        TableKey::TablePtr(p) => format!("\"table@{}\"", p),
                        TableKey::FuncPtr(p) => format!("\"func@{}\"", p),
                    };
                    format!("{}:{}", key_str, lua_value_to_json_depth(val, depth + 1))
                })
                .collect();
            format!("{{{}}}", entries.join(","))
        }
        LuaValue::Function(_) => "\"[function]\"".to_string(),
    }
}

/// Serialize a result to JSON.
pub fn lua_result_to_json(r: &LuaResult) -> String {
    let vals: Vec<String> = r.return_values.iter().map(lua_value_to_json).collect();
    let vals_str = vals.join(",");
    let err_str = match &r.error {
        Some(e) => format!("\"{}\"", e.replace('"', "\\\"")),
        None => "null".to_string(),
    };
    format!(
        "{{\"return_values\":[{}],\"error\":{},\"executed\":{}}}",
        vals_str, err_str, r.executed
    )
}

/// Serialize a stub to JSON.
pub fn lua_stub_to_json(stub: &LuaStub) -> String {
    let globals_count = stub.globals.len();
    format!(
        "{{\"call_count\":{},\"globals_count\":{},\"sandbox\":{}}}",
        stub.call_count, globals_count, stub.config.sandbox
    )
}

// ===== Lexer =====

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    LuaStr(String),
    True,
    False,
    Nil,
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    EqEq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
    Assign,
    And,
    Or,
    Not,
    Hash,
    DotDot,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Dot,
    Comma,
    Semi,
    Colon,
    If,
    Then,
    Else,
    Elseif,
    End,
    While,
    Do,
    For,
    In,
    Return,
    Local,
    Function,
    Repeat,
    Until,
    Break,
    Eof,
}

fn lex(src: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = src.chars().collect();
    let mut pos = 0;
    let mut tokens = Vec::new();

    while pos < chars.len() {
        if chars[pos].is_whitespace() {
            pos += 1;
            continue;
        }

        // Line / long comments
        if pos + 1 < chars.len() && chars[pos] == '-' && chars[pos + 1] == '-' {
            pos += 2;
            if pos + 1 < chars.len() && chars[pos] == '[' && chars[pos + 1] == '[' {
                pos += 2;
                loop {
                    if pos + 1 >= chars.len() {
                        return Err("unterminated long comment".to_string());
                    }
                    if chars[pos] == ']' && chars[pos + 1] == ']' {
                        pos += 2;
                        break;
                    }
                    pos += 1;
                }
            } else {
                while pos < chars.len() && chars[pos] != '\n' {
                    pos += 1;
                }
            }
            continue;
        }

        // Decimal numbers (including leading-dot like .5)
        if chars[pos].is_ascii_digit()
            || (chars[pos] == '.' && pos + 1 < chars.len() && chars[pos + 1].is_ascii_digit())
        {
            let start = pos;
            while pos < chars.len() && (chars[pos].is_ascii_digit() || chars[pos] == '.') {
                pos += 1;
            }
            if pos < chars.len() && (chars[pos] == 'e' || chars[pos] == 'E') {
                pos += 1;
                if pos < chars.len() && (chars[pos] == '+' || chars[pos] == '-') {
                    pos += 1;
                }
                while pos < chars.len() && chars[pos].is_ascii_digit() {
                    pos += 1;
                }
            }
            let s: String = chars[start..pos].iter().collect();
            let n: f64 = s.parse().map_err(|_| format!("invalid number: {}", s))?;
            tokens.push(Token::Number(n));
            continue;
        }

        // Hex numbers
        if chars[pos] == '0'
            && pos + 1 < chars.len()
            && (chars[pos + 1] == 'x' || chars[pos + 1] == 'X')
        {
            pos += 2;
            let start = pos;
            while pos < chars.len() && chars[pos].is_ascii_hexdigit() {
                pos += 1;
            }
            let s: String = chars[start..pos].iter().collect();
            let n = i64::from_str_radix(&s, 16).map_err(|_| format!("invalid hex: {}", s))?;
            tokens.push(Token::Number(n as f64));
            continue;
        }

        // Quoted strings
        if chars[pos] == '"' || chars[pos] == '\'' {
            let quote = chars[pos];
            pos += 1;
            let mut s = String::new();
            while pos < chars.len() && chars[pos] != quote {
                if chars[pos] == '\\' {
                    pos += 1;
                    if pos >= chars.len() {
                        return Err("unterminated string escape".to_string());
                    }
                    match chars[pos] {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        'r' => s.push('\r'),
                        '\\' => s.push('\\'),
                        '"' => s.push('"'),
                        '\'' => s.push('\''),
                        '0' => s.push('\0'),
                        c => {
                            s.push('\\');
                            s.push(c);
                        }
                    }
                } else {
                    s.push(chars[pos]);
                }
                pos += 1;
            }
            if pos >= chars.len() {
                return Err("unterminated string literal".to_string());
            }
            pos += 1;
            tokens.push(Token::LuaStr(s));
            continue;
        }

        // Long strings [[...]]
        if chars[pos] == '[' && pos + 1 < chars.len() && chars[pos + 1] == '[' {
            pos += 2;
            let mut s = String::new();
            loop {
                if pos + 1 >= chars.len() {
                    return Err("unterminated long string".to_string());
                }
                if chars[pos] == ']' && chars[pos + 1] == ']' {
                    pos += 2;
                    break;
                }
                s.push(chars[pos]);
                pos += 1;
            }
            tokens.push(Token::LuaStr(s));
            continue;
        }

        // Identifiers and keywords
        if chars[pos].is_alphabetic() || chars[pos] == '_' {
            let start = pos;
            while pos < chars.len() && (chars[pos].is_alphanumeric() || chars[pos] == '_') {
                pos += 1;
            }
            let word: String = chars[start..pos].iter().collect();
            let tok = match word.as_str() {
                "true" => Token::True,
                "false" => Token::False,
                "nil" => Token::Nil,
                "and" => Token::And,
                "or" => Token::Or,
                "not" => Token::Not,
                "if" => Token::If,
                "then" => Token::Then,
                "else" => Token::Else,
                "elseif" => Token::Elseif,
                "end" => Token::End,
                "while" => Token::While,
                "do" => Token::Do,
                "for" => Token::For,
                "in" => Token::In,
                "return" => Token::Return,
                "local" => Token::Local,
                "function" => Token::Function,
                "repeat" => Token::Repeat,
                "until" => Token::Until,
                "break" => Token::Break,
                _ => Token::Ident(word),
            };
            tokens.push(tok);
            continue;
        }

        // Two-character operators
        if pos + 1 < chars.len() {
            let two: String = chars[pos..pos + 2].iter().collect();
            let tok = match two.as_str() {
                "==" => Some(Token::EqEq),
                "~=" => Some(Token::NotEq),
                "<=" => Some(Token::Le),
                ">=" => Some(Token::Ge),
                ".." => Some(Token::DotDot),
                _ => None,
            };
            if let Some(t) = tok {
                tokens.push(t);
                pos += 2;
                continue;
            }
        }

        // Single-character operators
        let tok = match chars[pos] {
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '/' => Token::Slash,
            '%' => Token::Percent,
            '^' => Token::Caret,
            '<' => Token::Lt,
            '>' => Token::Gt,
            '=' => Token::Assign,
            '#' => Token::Hash,
            '(' => Token::LParen,
            ')' => Token::RParen,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            '.' => Token::Dot,
            ',' => Token::Comma,
            ';' => Token::Semi,
            ':' => Token::Colon,
            c => return Err(format!("unexpected character: {:?}", c)),
        };
        tokens.push(tok);
        pos += 1;
    }

    tokens.push(Token::Eof);
    Ok(tokens)
}

// ===== AST =====

#[derive(Debug, Clone)]
pub(crate) enum Expr {
    Nil,
    True,
    False,
    Number(f64),
    Str(String),
    Var(String),
    BinOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnOp {
        op: UnOp,
        operand: Box<Expr>,
    },
    TableCtor(Vec<TableField>),
    FieldAccess {
        table: Box<Expr>,
        field: String,
    },
    IndexAccess {
        table: Box<Expr>,
        key: Box<Expr>,
    },
    FuncCall {
        func: Box<Expr>,
        args: Vec<Expr>,
    },
    MethodCall {
        obj: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    FuncDef {
        params: Vec<String>,
        body: Vec<Stmt>,
    },
}

#[derive(Debug, Clone)]
pub(crate) enum TableField {
    Indexed { key: Expr, val: Expr },
    Named { name: String, val: Expr },
    Positional(Expr),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Concat,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum UnOp {
    Neg,
    Not,
    Len,
}

#[derive(Debug, Clone)]
pub(crate) enum Stmt {
    Assign {
        targets: Vec<Expr>,
        values: Vec<Expr>,
    },
    Local {
        names: Vec<String>,
        values: Vec<Expr>,
    },
    Do(Vec<Stmt>),
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    Repeat {
        body: Vec<Stmt>,
        cond: Expr,
    },
    If {
        cond: Expr,
        then_block: Vec<Stmt>,
        elseif_blocks: Vec<(Expr, Vec<Stmt>)>,
        else_block: Option<Vec<Stmt>>,
    },
    NumFor {
        var: String,
        start: Expr,
        limit: Expr,
        step: Option<Expr>,
        body: Vec<Stmt>,
    },
    GenFor {
        vars: Vec<String>,
        iterators: Vec<Expr>,
        body: Vec<Stmt>,
    },
    FuncDef {
        name: Vec<String>,
        method: Option<String>,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    LocalFunc {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Return(Vec<Expr>),
    Break,
    Expr(Expr),
}

// ===== Parser =====

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn peek_at(&self, offset: usize) -> &Token {
        let idx = self.pos + offset;
        if idx < self.tokens.len() {
            &self.tokens[idx]
        } else {
            &Token::Eof
        }
    }

    fn advance(&mut self) -> &Token {
        let t = &self.tokens[self.pos];
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, expected: &Token) -> Result<(), String> {
        let t = self.advance();
        if t == expected {
            Ok(())
        } else {
            Err(format!("expected {:?}, got {:?}", expected, t))
        }
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        match self.advance().clone() {
            Token::Ident(name) => Ok(name),
            t => Err(format!("expected identifier, got {:?}", t)),
        }
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        loop {
            while self.peek() == &Token::Semi {
                self.advance();
            }
            match self.peek() {
                Token::Eof | Token::End | Token::Else | Token::Elseif | Token::Until => break,
                Token::Return => {
                    self.advance();
                    let vals = self.parse_expr_list()?;
                    if self.peek() == &Token::Semi {
                        self.advance();
                    }
                    stmts.push(Stmt::Return(vals));
                    break;
                }
                _ => {
                    let stmt = self.parse_stmt()?;
                    stmts.push(stmt);
                }
            }
        }
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match self.peek().clone() {
            Token::Local => {
                self.advance();
                if self.peek() == &Token::Function {
                    self.advance();
                    let name = self.expect_ident()?;
                    let (params, body) = self.parse_func_body()?;
                    Ok(Stmt::LocalFunc { name, params, body })
                } else {
                    let mut names = vec![self.expect_ident()?];
                    while self.peek() == &Token::Comma {
                        self.advance();
                        names.push(self.expect_ident()?);
                    }
                    let values = if self.peek() == &Token::Assign {
                        self.advance();
                        self.parse_expr_list()?
                    } else {
                        Vec::new()
                    };
                    Ok(Stmt::Local { names, values })
                }
            }
            Token::Function => {
                self.advance();
                let mut name_parts = vec![self.expect_ident()?];
                let mut method = None;
                loop {
                    match self.peek() {
                        Token::Dot => {
                            self.advance();
                            name_parts.push(self.expect_ident()?);
                        }
                        Token::Colon => {
                            self.advance();
                            method = Some(self.expect_ident()?);
                            break;
                        }
                        _ => break,
                    }
                }
                let (mut params, body) = self.parse_func_body()?;
                if method.is_some() {
                    params.insert(0, "self".to_string());
                }
                Ok(Stmt::FuncDef {
                    name: name_parts,
                    method,
                    params,
                    body,
                })
            }
            Token::If => self.parse_if(),
            Token::While => {
                self.advance();
                let cond = self.parse_expr()?;
                self.expect(&Token::Do)?;
                let body = self.parse_block()?;
                self.expect(&Token::End)?;
                Ok(Stmt::While { cond, body })
            }
            Token::Repeat => {
                self.advance();
                let body = self.parse_block()?;
                self.expect(&Token::Until)?;
                let cond = self.parse_expr()?;
                Ok(Stmt::Repeat { body, cond })
            }
            Token::For => {
                self.advance();
                let first_var = self.expect_ident()?;
                if self.peek() == &Token::Assign {
                    self.advance();
                    let start = self.parse_expr()?;
                    self.expect(&Token::Comma)?;
                    let limit = self.parse_expr()?;
                    let step = if self.peek() == &Token::Comma {
                        self.advance();
                        Some(self.parse_expr()?)
                    } else {
                        None
                    };
                    self.expect(&Token::Do)?;
                    let body = self.parse_block()?;
                    self.expect(&Token::End)?;
                    Ok(Stmt::NumFor {
                        var: first_var,
                        start,
                        limit,
                        step,
                        body,
                    })
                } else {
                    let mut vars = vec![first_var];
                    while self.peek() == &Token::Comma {
                        self.advance();
                        vars.push(self.expect_ident()?);
                    }
                    self.expect(&Token::In)?;
                    let iterators = self.parse_expr_list()?;
                    self.expect(&Token::Do)?;
                    let body = self.parse_block()?;
                    self.expect(&Token::End)?;
                    Ok(Stmt::GenFor {
                        vars,
                        iterators,
                        body,
                    })
                }
            }
            Token::Do => {
                self.advance();
                let block = self.parse_block()?;
                self.expect(&Token::End)?;
                Ok(Stmt::Do(block))
            }
            Token::Break => {
                self.advance();
                Ok(Stmt::Break)
            }
            _ => {
                let expr = self.parse_suffix_expr()?;
                let mut targets = vec![expr];
                while self.peek() == &Token::Comma {
                    self.advance();
                    targets.push(self.parse_suffix_expr()?);
                }
                if self.peek() == &Token::Assign {
                    self.advance();
                    let values = self.parse_expr_list()?;
                    Ok(Stmt::Assign { targets, values })
                } else if targets.len() == 1 {
                    match &targets[0] {
                        Expr::FuncCall { .. } | Expr::MethodCall { .. } => {
                            Ok(Stmt::Expr(targets.remove(0)))
                        }
                        _ => Err(format!(
                            "unexpected expression as statement: {:?}",
                            targets[0]
                        )),
                    }
                } else {
                    Err("multi-target without assignment".to_string())
                }
            }
        }
    }

    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.expect(&Token::If)?;
        let cond = self.parse_expr()?;
        self.expect(&Token::Then)?;
        let then_block = self.parse_block()?;
        let mut elseif_blocks = Vec::new();
        let mut else_block = None;
        loop {
            match self.peek() {
                Token::Elseif => {
                    self.advance();
                    let ec = self.parse_expr()?;
                    self.expect(&Token::Then)?;
                    let eb = self.parse_block()?;
                    elseif_blocks.push((ec, eb));
                }
                Token::Else => {
                    self.advance();
                    else_block = Some(self.parse_block()?);
                    break;
                }
                _ => break,
            }
        }
        self.expect(&Token::End)?;
        Ok(Stmt::If {
            cond,
            then_block,
            elseif_blocks,
            else_block,
        })
    }

    fn parse_func_body(&mut self) -> Result<(Vec<String>, Vec<Stmt>), String> {
        self.expect(&Token::LParen)?;
        let mut params = Vec::new();
        if self.peek() != &Token::RParen {
            params.push(self.expect_ident()?);
            while self.peek() == &Token::Comma {
                self.advance();
                if let Token::DotDot = self.peek() {
                    self.advance();
                    if self.peek() == &Token::Dot {
                        self.advance();
                    }
                    break;
                }
                params.push(self.expect_ident()?);
            }
        }
        self.expect(&Token::RParen)?;
        let body = self.parse_block()?;
        self.expect(&Token::End)?;
        Ok((params, body))
    }

    fn parse_expr_list(&mut self) -> Result<Vec<Expr>, String> {
        let first = self.parse_expr()?;
        let mut exprs = vec![first];
        while self.peek() == &Token::Comma {
            self.advance();
            exprs.push(self.parse_expr()?);
        }
        Ok(exprs)
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_or_expr()
    }

    fn parse_or_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and_expr()?;
        while self.peek() == &Token::Or {
            self.advance();
            let right = self.parse_and_expr()?;
            left = Expr::BinOp {
                op: BinOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_compare_expr()?;
        while self.peek() == &Token::And {
            self.advance();
            let right = self.parse_compare_expr()?;
            left = Expr::BinOp {
                op: BinOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_compare_expr(&mut self) -> Result<Expr, String> {
        let left = self.parse_concat_expr()?;
        let op = match self.peek() {
            Token::EqEq => BinOp::Eq,
            Token::NotEq => BinOp::Ne,
            Token::Lt => BinOp::Lt,
            Token::Le => BinOp::Le,
            Token::Gt => BinOp::Gt,
            Token::Ge => BinOp::Ge,
            _ => return Ok(left),
        };
        self.advance();
        let right = self.parse_concat_expr()?;
        Ok(Expr::BinOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn parse_concat_expr(&mut self) -> Result<Expr, String> {
        let left = self.parse_add_expr()?;
        if self.peek() == &Token::DotDot {
            self.advance();
            let right = self.parse_concat_expr()?;
            return Ok(Expr::BinOp {
                op: BinOp::Concat,
                left: Box::new(left),
                right: Box::new(right),
            });
        }
        Ok(left)
    }

    fn parse_add_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_mul_expr()?;
        loop {
            let op = match self.peek() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_mul_expr()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_mul_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary_expr()?;
        loop {
            let op = match self.peek() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary_expr()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary_expr(&mut self) -> Result<Expr, String> {
        match self.peek().clone() {
            Token::Minus => {
                self.advance();
                let e = self.parse_unary_expr()?;
                Ok(Expr::UnOp {
                    op: UnOp::Neg,
                    operand: Box::new(e),
                })
            }
            Token::Not => {
                self.advance();
                let e = self.parse_unary_expr()?;
                Ok(Expr::UnOp {
                    op: UnOp::Not,
                    operand: Box::new(e),
                })
            }
            Token::Hash => {
                self.advance();
                let e = self.parse_unary_expr()?;
                Ok(Expr::UnOp {
                    op: UnOp::Len,
                    operand: Box::new(e),
                })
            }
            _ => self.parse_power_expr(),
        }
    }

    fn parse_power_expr(&mut self) -> Result<Expr, String> {
        let base = self.parse_suffix_expr()?;
        if self.peek() == &Token::Caret {
            self.advance();
            let exp = self.parse_unary_expr()?;
            return Ok(Expr::BinOp {
                op: BinOp::Pow,
                left: Box::new(base),
                right: Box::new(exp),
            });
        }
        Ok(base)
    }

    fn parse_suffix_expr(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary_expr()?;
        loop {
            match self.peek().clone() {
                Token::Dot => {
                    self.advance();
                    let field = self.expect_ident()?;
                    expr = Expr::FieldAccess {
                        table: Box::new(expr),
                        field,
                    };
                }
                Token::LBracket => {
                    self.advance();
                    let key = self.parse_expr()?;
                    self.expect(&Token::RBracket)?;
                    expr = Expr::IndexAccess {
                        table: Box::new(expr),
                        key: Box::new(key),
                    };
                }
                Token::Colon => {
                    self.advance();
                    let method = self.expect_ident()?;
                    let args = self.parse_call_args()?;
                    expr = Expr::MethodCall {
                        obj: Box::new(expr),
                        method,
                        args,
                    };
                }
                Token::LParen | Token::LBrace | Token::LuaStr(_) => {
                    let args = self.parse_call_args()?;
                    expr = Expr::FuncCall {
                        func: Box::new(expr),
                        args,
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_call_args(&mut self) -> Result<Vec<Expr>, String> {
        match self.peek().clone() {
            Token::LParen => {
                self.advance();
                if self.peek() == &Token::RParen {
                    self.advance();
                    return Ok(Vec::new());
                }
                let args = self.parse_expr_list()?;
                self.expect(&Token::RParen)?;
                Ok(args)
            }
            Token::LBrace => {
                let tbl = self.parse_table_ctor()?;
                Ok(vec![tbl])
            }
            Token::LuaStr(s) => {
                let s = s.clone();
                self.advance();
                Ok(vec![Expr::Str(s)])
            }
            t => Err(format!("expected function call args, got {:?}", t)),
        }
    }

    fn parse_primary_expr(&mut self) -> Result<Expr, String> {
        match self.peek().clone() {
            Token::Nil => {
                self.advance();
                Ok(Expr::Nil)
            }
            Token::True => {
                self.advance();
                Ok(Expr::True)
            }
            Token::False => {
                self.advance();
                Ok(Expr::False)
            }
            Token::Number(n) => {
                let num = n;
                self.advance();
                Ok(Expr::Number(num))
            }
            Token::LuaStr(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::Str(s))
            }
            Token::Ident(name) => {
                let name = name.clone();
                self.advance();
                Ok(Expr::Var(name))
            }
            Token::LParen => {
                self.advance();
                let e = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(e)
            }
            Token::LBrace => self.parse_table_ctor(),
            Token::Function => {
                self.advance();
                let (params, body) = self.parse_func_body()?;
                Ok(Expr::FuncDef { params, body })
            }
            t => Err(format!("unexpected token in expression: {:?}", t)),
        }
    }

    fn parse_table_ctor(&mut self) -> Result<Expr, String> {
        self.expect(&Token::LBrace)?;
        let mut fields = Vec::new();
        while self.peek() != &Token::RBrace {
            let field = match self.peek().clone() {
                Token::LBracket => {
                    self.advance();
                    let key = self.parse_expr()?;
                    self.expect(&Token::RBracket)?;
                    self.expect(&Token::Assign)?;
                    let val = self.parse_expr()?;
                    TableField::Indexed { key, val }
                }
                Token::Ident(_) if self.peek_at(1) == &Token::Assign => {
                    if let Token::Ident(name) = self.advance().clone() {
                        self.advance(); // '='
                        let val = self.parse_expr()?;
                        TableField::Named { name, val }
                    } else {
                        unreachable!()
                    }
                }
                _ => {
                    let val = self.parse_expr()?;
                    TableField::Positional(val)
                }
            };
            fields.push(field);
            match self.peek() {
                Token::Comma | Token::Semi => {
                    self.advance();
                }
                _ => break,
            }
        }
        self.expect(&Token::RBrace)?;
        Ok(Expr::TableCtor(fields))
    }
}

fn parse(tokens: &[Token]) -> Result<Vec<Stmt>, String> {
    let mut p = Parser::new(tokens);
    let block = p.parse_block()?;
    if p.peek() != &Token::Eof {
        return Err(format!("unexpected token at end: {:?}", p.peek()));
    }
    Ok(block)
}

// ===== Evaluator (included from lua_interp.rs) =====

#[path = "lua_interp.rs"]
mod lua_interp;

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;

    fn run(src: &str) -> LuaResult {
        let cfg = default_lua_config();
        let mut stub = new_lua_stub(cfg);
        let script = new_lua_script("test", src);
        lua_execute(&mut stub, &script)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_lua_config();
        assert_eq!(cfg.max_stack_depth, 200);
        assert_eq!(cfg.timeout_ms, 5000);
        assert!(cfg.sandbox);
    }

    #[test]
    fn test_new_stub() {
        let cfg = default_lua_config();
        let stub = new_lua_stub(cfg);
        assert_eq!(stub.call_count, 0);
        assert_eq!(lua_global_count(&stub), 0);
    }

    #[test]
    fn test_set_get_global() {
        let cfg = default_lua_config();
        let mut stub = new_lua_stub(cfg);
        lua_set_global(&mut stub, "x", LuaValue::Int(42));
        let v = lua_get_global(&stub, "x");
        assert!(v.is_some());
        if let Some(LuaValue::Int(i)) = v {
            assert_eq!(*i, 42);
        } else {
            panic!("expected Int");
        }
    }

    #[test]
    fn test_execute_increments_count() {
        let cfg = default_lua_config();
        let mut stub = new_lua_stub(cfg);
        let script = new_lua_script("test", "return 1");
        let result = lua_execute(&mut stub, &script);
        assert!(result.executed);
        assert!(result.error.is_none());
        assert_eq!(stub.call_count, 1);
    }

    #[test]
    fn test_value_type_names() {
        assert_eq!(lua_value_type_name(&LuaValue::Nil), "nil");
        assert_eq!(lua_value_type_name(&LuaValue::Bool(true)), "boolean");
        assert_eq!(lua_value_type_name(&LuaValue::Int(0)), "integer");
        assert_eq!(lua_value_type_name(&LuaValue::Float(0.0)), "float");
        assert_eq!(lua_value_type_name(&LuaValue::Str(String::new())), "string");
    }

    #[test]
    fn test_global_count_and_update() {
        let cfg = default_lua_config();
        let mut stub = new_lua_stub(cfg);
        lua_set_global(&mut stub, "a", LuaValue::Bool(false));
        lua_set_global(&mut stub, "b", LuaValue::Float(std::f64::consts::PI));
        assert_eq!(lua_global_count(&stub), 2);
        lua_set_global(&mut stub, "a", LuaValue::Bool(true));
        assert_eq!(lua_global_count(&stub), 2);
    }

    #[test]
    fn test_result_to_json() {
        let r = LuaResult {
            return_values: vec![LuaValue::Nil],
            error: None,
            executed: true,
            print_output: Vec::new(),
        };
        let json = lua_result_to_json(&r);
        assert!(json.contains("return_values"));
        assert!(json.contains("null"));
    }

    #[test]
    fn test_stub_to_json() {
        let cfg = default_lua_config();
        let stub = new_lua_stub(cfg);
        let json = lua_stub_to_json(&stub);
        assert!(json.contains("call_count"));
        assert!(json.contains("sandbox"));
    }

    #[test]
    fn test_execute_empty_script() {
        let r = run("");
        assert!(r.error.is_none());
        assert!(r.executed);
    }

    #[test]
    fn test_execute_literal_return() {
        let r = run("return 42");
        assert!(r.error.is_none());
        assert_eq!(r.return_values.len(), 1);
        assert!(matches!(r.return_values[0], LuaValue::Int(42)));
    }

    #[test]
    fn test_execute_arithmetic() {
        let r = run("return 2 + 3 * 4");
        assert!(r.error.is_none());
        assert_eq!(r.return_values.len(), 1);
        assert!(matches!(r.return_values[0], LuaValue::Int(14)));
    }

    #[test]
    fn test_execute_string_concat() {
        let r = run(r#"return "hello" .. " " .. "world""#);
        assert!(r.error.is_none());
        if let LuaValue::Str(s) = &r.return_values[0] {
            assert_eq!(s, "hello world");
        } else {
            panic!("expected string");
        }
    }

    #[test]
    fn test_execute_if_then_else() {
        let r = run("if 1 > 0 then return 1 else return 0 end");
        assert!(r.error.is_none());
        assert!(matches!(r.return_values[0], LuaValue::Int(1)));
    }

    #[test]
    fn test_execute_while_loop() {
        let r = run("local s = 0\nfor i = 1, 10 do s = s + i end\nreturn s");
        assert!(r.error.is_none(), "error: {:?}", r.error);
        assert!(matches!(r.return_values[0], LuaValue::Int(55)));
    }

    #[test]
    fn test_global_set_get() {
        let cfg = default_lua_config();
        let mut stub = new_lua_stub(cfg);
        lua_set_global(&mut stub, "myval", LuaValue::Int(99));
        let script = new_lua_script("test", "return myval");
        let r = lua_execute(&mut stub, &script);
        assert!(r.error.is_none());
        assert!(matches!(r.return_values[0], LuaValue::Int(99)));
    }

    #[test]
    fn test_execute_function_def_call() {
        let r = run("function add(a, b) return a + b end\nreturn add(3, 4)");
        assert!(r.error.is_none(), "error: {:?}", r.error);
        assert!(matches!(r.return_values[0], LuaValue::Int(7)));
    }

    #[test]
    fn test_error_on_syntax_error() {
        let r = run("return )(garbage");
        assert!(r.error.is_some());
        assert!(!r.executed);
    }

    #[test]
    fn test_max_stack_depth() {
        // Use a very low stack depth to test graceful overflow detection
        // without causing Rust's own stack overflow.
        let cfg = LuaConfig {
            max_stack_depth: 20,
            timeout_ms: 5000,
            sandbox: true,
        };
        let mut stub = new_lua_stub(cfg);
        let script = new_lua_script("test", "function inf() return inf() end\ninf()");
        let r = lua_execute(&mut stub, &script);
        assert!(r.error.is_some());
        let err = r.error.as_deref().unwrap_or("");
        assert!(
            err.contains("stack overflow") || err.contains("stack"),
            "error was: {}",
            err
        );
    }

    #[test]
    fn test_print_output_captured() {
        let r = run(r#"print("hello")"#);
        assert!(r.error.is_none(), "error: {:?}", r.error);
        assert_eq!(r.print_output.len(), 1);
        assert!(r.print_output[0].contains("hello"));
    }

    #[test]
    fn test_table_operations() {
        let r = run("local t = {10, 20, 30}\nreturn t[1] + t[2] + t[3]");
        assert!(r.error.is_none(), "error: {:?}", r.error);
        assert!(matches!(r.return_values[0], LuaValue::Int(60)));
    }

    #[test]
    fn test_elseif_chain() {
        let r = run(
            "local x = 5\nif x == 1 then return 1\nelseif x == 5 then return 5\nelse return 0 end",
        );
        assert!(r.error.is_none());
        assert!(matches!(r.return_values[0], LuaValue::Int(5)));
    }

    #[test]
    fn test_string_concat_and_tostring() {
        let r = run(r#"return tostring(42) .. " bottles""#);
        assert!(r.error.is_none(), "error: {:?}", r.error);
        if let LuaValue::Str(s) = &r.return_values[0] {
            assert_eq!(s, "42 bottles");
        } else {
            panic!("expected string");
        }
    }

    #[test]
    fn test_math_floor_ceil() {
        let r = run("return math.floor(3.7)");
        assert!(r.error.is_none(), "error: {:?}", r.error);
        assert!(matches!(r.return_values[0], LuaValue::Int(3)));
    }

    #[test]
    fn test_ipairs_loop() {
        let r = run(
            "local s = 0\nlocal t = {1,2,3,4,5}\nfor i,v in ipairs(t) do s = s + v end\nreturn s",
        );
        assert!(r.error.is_none(), "error: {:?}", r.error);
        assert!(matches!(r.return_values[0], LuaValue::Int(15)));
    }
}
