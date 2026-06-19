//! WASM host/guest interop bridge with a real MVP bytecode interpreter.
//!
//! The interpreter is a stack machine that executes WASM MVP bytecode stored in
//! `WasmFunction::bytecode` against the bridge's shared linear memory.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum WasmValueType {
    I32,
    I64,
    F32,
    F64,
    FuncRef,
    ExternRef,
}

/// A typed WASM runtime value.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum WasmValue {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

/// Errors that can occur during WASM execution.
#[derive(Debug, thiserror::Error)]
pub enum WasmError {
    #[error("Function not found: {0}")]
    FunctionNotFound(String),
    #[error("Type mismatch: {0}")]
    TypeMismatch(String),
    #[error("Runtime trap: {0}")]
    Trap(String),
    #[error("Memory out of bounds at offset {0}")]
    MemoryOutOfBounds(usize),
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Stack overflow")]
    StackOverflow,
    #[error("Unimplemented opcode: 0x{0:02X}")]
    UnimplementedOpcode(u8),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WasmBridgeConfig {
    pub memory_pages: u32,
    pub max_functions: usize,
    pub debug_mode: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WasmMemory {
    pub data: Vec<u8>,
    pub size_bytes: usize,
}

/// A WASM function with metadata and executable bytecode.
///
/// `bytecode` = `[local_decl_count : LEB128] (count LEB128 + type byte)* expression_opcodes`
/// An empty `bytecode` is treated as an empty function that returns with no values.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WasmFunction {
    pub name: String,
    pub param_types: Vec<WasmValueType>,
    pub return_types: Vec<WasmValueType>,
    pub bytecode: Vec<u8>,
    pub local_types: Vec<WasmValueType>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WasmBridge {
    pub config: WasmBridgeConfig,
    pub memory: WasmMemory,
    pub functions: Vec<WasmFunction>,
    pub call_count: u64,
}

// ─── Block / control-flow machinery ─────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum BlockKind {
    Block,
    Loop,
    If,
}

#[derive(Debug, Clone)]
struct BlockFrame {
    kind: BlockKind,
    arity: usize,
    height: usize,
    /// Block/If: end PC.  Loop: loop-header PC (for back-edge).
    else_or_end_pc: usize,
}

// ─── LEB128 helpers ──────────────────────────────────────────────────────────

fn read_leb128_u32(code: &[u8], pc: &mut usize) -> u32 {
    let (mut r, mut s) = (0u32, 0u32);
    loop {
        let b = code[*pc];
        *pc += 1;
        r |= ((b & 0x7F) as u32) << s;
        if b & 0x80 == 0 {
            break;
        }
        s += 7;
    }
    r
}

#[allow(dead_code)]
fn read_leb128_u64(code: &[u8], pc: &mut usize) -> u64 {
    let (mut r, mut s) = (0u64, 0u32);
    loop {
        let b = code[*pc];
        *pc += 1;
        r |= ((b & 0x7F) as u64) << s;
        if b & 0x80 == 0 {
            break;
        }
        s += 7;
    }
    r
}

fn read_leb128_i32(code: &[u8], pc: &mut usize) -> i32 {
    let (mut r, mut s) = (0i32, 0u32);
    let last = loop {
        let b = code[*pc];
        *pc += 1;
        r |= ((b & 0x7F) as i32) << s;
        s += 7;
        if b & 0x80 == 0 {
            break b;
        }
    };
    if s < 32 && (last & 0x40) != 0 {
        r |= -1i32 << s;
    }
    r
}

fn read_leb128_i64(code: &[u8], pc: &mut usize) -> i64 {
    let (mut r, mut s) = (0i64, 0u32);
    let last = loop {
        let b = code[*pc];
        *pc += 1;
        r |= ((b & 0x7F) as i64) << s;
        s += 7;
        if b & 0x80 == 0 {
            break b;
        }
    };
    if s < 64 && (last & 0x40) != 0 {
        r |= -1i64 << s;
    }
    r
}

fn skip_leb128(code: &[u8], pc: &mut usize) {
    while *pc < code.len() {
        let b = code[*pc];
        *pc += 1;
        if b & 0x80 == 0 {
            break;
        }
    }
}

// ─── Interpreter ─────────────────────────────────────────────────────────────

const MAX_CALL_DEPTH: usize = 512;
const MAX_STACK_DEPTH: usize = 65536;

struct Interpreter<'a> {
    stack: Vec<WasmValue>,
    locals: Vec<WasmValue>,
    memory: &'a mut WasmMemory,
    code: &'a [u8],
    pc: usize,
    blocks: Vec<BlockFrame>,
    call_depth: usize,
}

impl<'a> Interpreter<'a> {
    fn new(
        memory: &'a mut WasmMemory,
        code: &'a [u8],
        locals: Vec<WasmValue>,
        call_depth: usize,
    ) -> Self {
        Interpreter {
            stack: Vec::new(),
            locals,
            memory,
            code,
            pc: 0,
            blocks: Vec::new(),
            call_depth,
        }
    }

    // ── stack helpers ─────────────────────────────────────────────────────────

    fn push(&mut self, v: WasmValue) -> Result<(), WasmError> {
        if self.stack.len() >= MAX_STACK_DEPTH {
            return Err(WasmError::StackOverflow);
        }
        self.stack.push(v);
        Ok(())
    }
    fn pop(&mut self) -> Result<WasmValue, WasmError> {
        self.stack
            .pop()
            .ok_or_else(|| WasmError::Trap("value stack underflow".into()))
    }
    fn pop_i32(&mut self) -> Result<i32, WasmError> {
        match self.pop()? {
            WasmValue::I32(v) => Ok(v),
            o => Err(WasmError::TypeMismatch(format!(
                "expected i32, got {:?}",
                o
            ))),
        }
    }
    fn pop_i64(&mut self) -> Result<i64, WasmError> {
        match self.pop()? {
            WasmValue::I64(v) => Ok(v),
            o => Err(WasmError::TypeMismatch(format!(
                "expected i64, got {:?}",
                o
            ))),
        }
    }
    fn pop_f32(&mut self) -> Result<f32, WasmError> {
        match self.pop()? {
            WasmValue::F32(v) => Ok(v),
            o => Err(WasmError::TypeMismatch(format!(
                "expected f32, got {:?}",
                o
            ))),
        }
    }
    fn pop_f64(&mut self) -> Result<f64, WasmError> {
        match self.pop()? {
            WasmValue::F64(v) => Ok(v),
            o => Err(WasmError::TypeMismatch(format!(
                "expected f64, got {:?}",
                o
            ))),
        }
    }
    #[allow(dead_code)]
    fn peek_byte(&self) -> Option<u8> {
        self.code.get(self.pc).copied()
    }

    // ── bytecode reading ──────────────────────────────────────────────────────

    fn read_byte(&mut self) -> Result<u8, WasmError> {
        if self.pc >= self.code.len() {
            return Err(WasmError::Trap("unexpected end of bytecode".into()));
        }
        let b = self.code[self.pc];
        self.pc += 1;
        Ok(b)
    }
    fn read_u32_leb(&mut self) -> Result<u32, WasmError> {
        if self.pc >= self.code.len() {
            return Err(WasmError::Trap("unexpected end (LEB128 u32)".into()));
        }
        Ok(read_leb128_u32(self.code, &mut self.pc))
    }
    fn read_i32_leb(&mut self) -> Result<i32, WasmError> {
        if self.pc >= self.code.len() {
            return Err(WasmError::Trap("unexpected end (LEB128 i32)".into()));
        }
        Ok(read_leb128_i32(self.code, &mut self.pc))
    }
    fn read_i64_leb(&mut self) -> Result<i64, WasmError> {
        if self.pc >= self.code.len() {
            return Err(WasmError::Trap("unexpected end (LEB128 i64)".into()));
        }
        Ok(read_leb128_i64(self.code, &mut self.pc))
    }
    fn read_f32(&mut self) -> Result<f32, WasmError> {
        if self.pc + 4 > self.code.len() {
            return Err(WasmError::Trap("unexpected end (f32)".into()));
        }
        let b = [
            self.code[self.pc],
            self.code[self.pc + 1],
            self.code[self.pc + 2],
            self.code[self.pc + 3],
        ];
        self.pc += 4;
        Ok(f32::from_le_bytes(b))
    }
    fn read_f64(&mut self) -> Result<f64, WasmError> {
        if self.pc + 8 > self.code.len() {
            return Err(WasmError::Trap("unexpected end (f64)".into()));
        }
        let b: [u8; 8] = self.code[self.pc..self.pc + 8].try_into().unwrap_or([0; 8]);
        self.pc += 8;
        Ok(f64::from_le_bytes(b))
    }

    // ── memory load/store ─────────────────────────────────────────────────────

    fn effective_addr(&self, base: u32, offset: u32) -> usize {
        (base as usize).wrapping_add(offset as usize)
    }

    // Loads — returns error if out of bounds.
    fn mload_i32(&self, a: usize) -> Result<i32, WasmError> {
        if a + 4 > self.memory.data.len() {
            return Err(WasmError::MemoryOutOfBounds(a));
        }
        Ok(i32::from_le_bytes(
            self.memory.data[a..a + 4].try_into().unwrap_or([0; 4]),
        ))
    }
    fn mload_i64(&self, a: usize) -> Result<i64, WasmError> {
        if a + 8 > self.memory.data.len() {
            return Err(WasmError::MemoryOutOfBounds(a));
        }
        Ok(i64::from_le_bytes(
            self.memory.data[a..a + 8].try_into().unwrap_or([0; 8]),
        ))
    }
    fn mload_f32(&self, a: usize) -> Result<f32, WasmError> {
        if a + 4 > self.memory.data.len() {
            return Err(WasmError::MemoryOutOfBounds(a));
        }
        Ok(f32::from_le_bytes(
            self.memory.data[a..a + 4].try_into().unwrap_or([0; 4]),
        ))
    }
    fn mload_f64(&self, a: usize) -> Result<f64, WasmError> {
        if a + 8 > self.memory.data.len() {
            return Err(WasmError::MemoryOutOfBounds(a));
        }
        Ok(f64::from_le_bytes(
            self.memory.data[a..a + 8].try_into().unwrap_or([0; 8]),
        ))
    }
    fn mload_i32_ext(&self, a: usize, bytes: usize, signed: bool) -> Result<i32, WasmError> {
        if a + bytes > self.memory.data.len() {
            return Err(WasmError::MemoryOutOfBounds(a));
        }
        let mut buf = [0u8; 4];
        buf[..bytes].copy_from_slice(&self.memory.data[a..a + bytes]);
        let raw = u32::from_le_bytes(buf);
        Ok(if signed {
            sign_extend_u32(raw, bytes * 8)
        } else {
            raw as i32
        })
    }
    fn mload_i64_ext(&self, a: usize, bytes: usize, signed: bool) -> Result<i64, WasmError> {
        if a + bytes > self.memory.data.len() {
            return Err(WasmError::MemoryOutOfBounds(a));
        }
        let mut buf = [0u8; 8];
        buf[..bytes].copy_from_slice(&self.memory.data[a..a + bytes]);
        let raw = u64::from_le_bytes(buf);
        Ok(if signed {
            sign_extend_u64(raw, bytes * 8)
        } else {
            raw as i64
        })
    }

    // Stores
    fn mstore_i32(&mut self, a: usize, v: i32) -> Result<(), WasmError> {
        if a + 4 > self.memory.data.len() {
            return Err(WasmError::MemoryOutOfBounds(a));
        }
        self.memory.data[a..a + 4].copy_from_slice(&v.to_le_bytes());
        Ok(())
    }
    fn mstore_i64(&mut self, a: usize, v: i64) -> Result<(), WasmError> {
        if a + 8 > self.memory.data.len() {
            return Err(WasmError::MemoryOutOfBounds(a));
        }
        self.memory.data[a..a + 8].copy_from_slice(&v.to_le_bytes());
        Ok(())
    }
    fn mstore_f32(&mut self, a: usize, v: f32) -> Result<(), WasmError> {
        if a + 4 > self.memory.data.len() {
            return Err(WasmError::MemoryOutOfBounds(a));
        }
        self.memory.data[a..a + 4].copy_from_slice(&v.to_le_bytes());
        Ok(())
    }
    fn mstore_f64(&mut self, a: usize, v: f64) -> Result<(), WasmError> {
        if a + 8 > self.memory.data.len() {
            return Err(WasmError::MemoryOutOfBounds(a));
        }
        self.memory.data[a..a + 8].copy_from_slice(&v.to_le_bytes());
        Ok(())
    }
    fn mstore_trunc(&mut self, a: usize, bytes: usize, v: u64) -> Result<(), WasmError> {
        if a + bytes > self.memory.data.len() {
            return Err(WasmError::MemoryOutOfBounds(a));
        }
        let src = v.to_le_bytes();
        self.memory.data[a..a + bytes].copy_from_slice(&src[..bytes]);
        Ok(())
    }

    // ── mem_arg helper (reads align+offset) ──────────────────────────────────

    fn read_mem_arg(&mut self) -> Result<u32, WasmError> {
        let _align = self.read_u32_leb()?;
        self.read_u32_leb()
    }

    fn pop_addr_offset(&mut self, offset: u32) -> Result<usize, WasmError> {
        let base = self.pop_i32()? as u32;
        Ok(self.effective_addr(base, offset))
    }

    // ── scan helper (for block entry) ─────────────────────────────────────────

    /// Scan forward from `start_pc` to find the matching `else` and `end` offsets.
    /// Returns `(else_pc, end_pc)`; if no else, `else_pc == end_pc`.
    fn scan_to_else_or_end(&self, start_pc: usize) -> Result<(usize, usize), WasmError> {
        let (mut depth, mut pc) = (1usize, start_pc);
        let mut first_else: Option<usize> = None;
        while pc < self.code.len() {
            let op = self.code[pc];
            pc += 1;
            match op {
                0x02..=0x04 => {
                    depth += 1;
                    pc += 1;
                } // nested block/loop/if + block_type
                0x05 if depth == 1 && first_else.is_none() => {
                    first_else = Some(pc - 1);
                }
                0x0B => {
                    depth -= 1;
                    if depth == 0 {
                        let end_pc = pc - 1;
                        return Ok((first_else.unwrap_or(end_pc), end_pc));
                    }
                }
                0x20..=0x24 => {
                    skip_leb128(self.code, &mut pc);
                }
                0x28..=0x3E => {
                    skip_leb128(self.code, &mut pc);
                    skip_leb128(self.code, &mut pc);
                }
                0x3F | 0x40 => {
                    pc += 1;
                }
                0x41 => {
                    skip_leb128(self.code, &mut pc);
                }
                0x42 => {
                    skip_leb128(self.code, &mut pc);
                }
                0x43 => {
                    pc += 4;
                }
                0x44 => {
                    pc += 8;
                }
                0x0C | 0x0D => {
                    skip_leb128(self.code, &mut pc);
                }
                0x0E => {
                    let n = read_leb128_u32(self.code, &mut pc) as usize;
                    for _ in 0..=n {
                        skip_leb128(self.code, &mut pc);
                    }
                }
                0x10 | 0x11 => {
                    skip_leb128(self.code, &mut pc);
                    if op == 0x11 {
                        skip_leb128(self.code, &mut pc);
                    }
                }
                _ => {}
            }
        }
        Err(WasmError::Trap("unmatched block (no end found)".into()))
    }

    // ── branch helper ─────────────────────────────────────────────────────────

    fn do_branch(&mut self, depth: u32) -> Result<Option<usize>, WasmError> {
        let depth = depth as usize;
        if depth >= self.blocks.len() {
            return Ok(None);
        }
        let idx = self.blocks.len() - 1 - depth;
        let frame = self.blocks[idx].clone();
        let slen = self.stack.len();
        if slen < frame.arity {
            return Err(WasmError::Trap("branch: stack underflow".into()));
        }
        let results: Vec<WasmValue> = self.stack[slen - frame.arity..].to_vec();
        self.stack.truncate(frame.height);
        for v in results {
            self.push(v)?;
        }
        self.blocks.truncate(idx);
        let new_pc = match frame.kind {
            BlockKind::Loop => frame.else_or_end_pc,
            _ => {
                let end = frame.else_or_end_pc;
                if end < self.code.len() && self.code[end] == 0x0B {
                    end + 1
                } else {
                    end
                }
            }
        };
        Ok(Some(new_pc))
    }

    // ── main dispatch loop ────────────────────────────────────────────────────

    fn run(&mut self, n_results: usize) -> Result<Vec<WasmValue>, WasmError> {
        if self.call_depth > MAX_CALL_DEPTH {
            return Err(WasmError::StackOverflow);
        }

        while self.pc < self.code.len() {
            let op = self.read_byte()?;
            match op {
                // ── control ───────────────────────────────────────────────────
                0x00 => return Err(WasmError::Trap("unreachable".into())),
                0x01 => { /* nop */ }
                0x02 => {
                    // block
                    let _ = self.read_byte()?;
                    let scan = self.pc;
                    let (_ep, end) = self.scan_to_else_or_end(scan)?;
                    self.blocks.push(BlockFrame {
                        kind: BlockKind::Block,
                        arity: 0,
                        height: self.stack.len(),
                        else_or_end_pc: end,
                    });
                }
                0x03 => {
                    // loop
                    let _ = self.read_byte()?;
                    let hdr = self.pc;
                    let scan = self.pc;
                    let _ = self.scan_to_else_or_end(scan)?;
                    self.blocks.push(BlockFrame {
                        kind: BlockKind::Loop,
                        arity: 0,
                        height: self.stack.len(),
                        else_or_end_pc: hdr,
                    });
                }
                0x04 => {
                    // if
                    let _ = self.read_byte()?;
                    let cond = self.pop_i32()?;
                    let scan = self.pc;
                    let (ep, end) = self.scan_to_else_or_end(scan)?;
                    self.blocks.push(BlockFrame {
                        kind: BlockKind::If,
                        arity: 0,
                        height: self.stack.len(),
                        else_or_end_pc: end,
                    });
                    if cond == 0 {
                        if ep < end {
                            self.pc = ep + 1;
                        } else {
                            self.pc = end + 1;
                            self.blocks.pop();
                        }
                    }
                }
                0x05 => {
                    // else (reached in then-branch)
                    let end = self
                        .blocks
                        .last()
                        .map(|f| f.else_or_end_pc)
                        .ok_or_else(|| WasmError::Trap("else without if".into()))?;
                    self.pc = end + 1;
                    self.blocks.pop();
                }
                0x0B => {
                    // end
                    if self.blocks.is_empty() {
                        break;
                    }
                    let frame = self.blocks.pop().expect("block frame");
                    let slen = self.stack.len();
                    if slen < frame.arity {
                        return Err(WasmError::Trap("end: stack underflow".into()));
                    }
                    let res: Vec<WasmValue> = self.stack[slen - frame.arity..].to_vec();
                    self.stack.truncate(frame.height);
                    for v in res {
                        self.push(v)?;
                    }
                }
                0x0C => {
                    let d = self.read_u32_leb()?;
                    match self.do_branch(d)? {
                        Some(p) => self.pc = p,
                        None => break,
                    }
                }
                0x0D => {
                    let d = self.read_u32_leb()?;
                    let c = self.pop_i32()?;
                    if c != 0 {
                        match self.do_branch(d)? {
                            Some(p) => self.pc = p,
                            None => break,
                        }
                    }
                }
                0x0E => {
                    // br_table
                    let n = self.read_u32_leb()? as usize;
                    let mut lbs = Vec::with_capacity(n + 1);
                    for _ in 0..=n {
                        lbs.push(self.read_u32_leb()?);
                    }
                    let idx = self.pop_i32()? as usize;
                    let d = if idx < n { lbs[idx] } else { lbs[n] };
                    match self.do_branch(d)? {
                        Some(p) => self.pc = p,
                        None => break,
                    }
                }
                0x0F => break, // return
                0x10 => {
                    let _ = self.read_u32_leb()?;
                    return Err(WasmError::Trap("call not supported".into()));
                }
                0x11 => {
                    let _ = self.read_u32_leb()?;
                    let _ = self.read_u32_leb()?;
                    return Err(WasmError::Trap("call_indirect not supported".into()));
                }

                // ── parametric ────────────────────────────────────────────────
                0x1A => {
                    self.pop()?;
                }
                0x1B => {
                    let c = self.pop_i32()?;
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(if c != 0 { a } else { b })?;
                }

                // ── locals ────────────────────────────────────────────────────
                0x20 => {
                    let i = self.read_u32_leb()? as usize;
                    let v = self
                        .locals
                        .get(i)
                        .cloned()
                        .ok_or_else(|| WasmError::Trap(format!("local.get: {} oob", i)))?;
                    self.push(v)?;
                }
                0x21 => {
                    let i = self.read_u32_leb()? as usize;
                    let v = self.pop()?;
                    *self
                        .locals
                        .get_mut(i)
                        .ok_or_else(|| WasmError::Trap(format!("local.set: {} oob", i)))? = v;
                }
                0x22 => {
                    let i = self.read_u32_leb()? as usize;
                    let v = self
                        .stack
                        .last()
                        .cloned()
                        .ok_or_else(|| WasmError::Trap("local.tee: underflow".into()))?;
                    *self
                        .locals
                        .get_mut(i)
                        .ok_or_else(|| WasmError::Trap(format!("local.tee: {} oob", i)))? = v;
                }
                0x23 => {
                    let _ = self.read_u32_leb()?;
                    return Err(WasmError::Trap("global.get not supported".into()));
                }
                0x24 => {
                    let _ = self.read_u32_leb()?;
                    return Err(WasmError::Trap("global.set not supported".into()));
                }

                // ── memory loads (0x28..=0x35) ────────────────────────────────
                0x28 => {
                    let off = self.read_mem_arg()?;
                    let a = self.pop_addr_offset(off)?;
                    let v = self.mload_i32(a)?;
                    self.push(WasmValue::I32(v))?;
                }
                0x29 => {
                    let off = self.read_mem_arg()?;
                    let a = self.pop_addr_offset(off)?;
                    let v = self.mload_i64(a)?;
                    self.push(WasmValue::I64(v))?;
                }
                0x2A => {
                    let off = self.read_mem_arg()?;
                    let a = self.pop_addr_offset(off)?;
                    let v = self.mload_f32(a)?;
                    self.push(WasmValue::F32(v))?;
                }
                0x2B => {
                    let off = self.read_mem_arg()?;
                    let a = self.pop_addr_offset(off)?;
                    let v = self.mload_f64(a)?;
                    self.push(WasmValue::F64(v))?;
                }
                0x2C => {
                    let off = self.read_mem_arg()?;
                    let a = self.pop_addr_offset(off)?;
                    let v = self.mload_i32_ext(a, 1, true)?;
                    self.push(WasmValue::I32(v))?;
                }
                0x2D => {
                    let off = self.read_mem_arg()?;
                    let a = self.pop_addr_offset(off)?;
                    let v = self.mload_i32_ext(a, 1, false)?;
                    self.push(WasmValue::I32(v))?;
                }
                0x2E => {
                    let off = self.read_mem_arg()?;
                    let a = self.pop_addr_offset(off)?;
                    let v = self.mload_i32_ext(a, 2, true)?;
                    self.push(WasmValue::I32(v))?;
                }
                0x2F => {
                    let off = self.read_mem_arg()?;
                    let a = self.pop_addr_offset(off)?;
                    let v = self.mload_i32_ext(a, 2, false)?;
                    self.push(WasmValue::I32(v))?;
                }
                0x30 => {
                    let off = self.read_mem_arg()?;
                    let a = self.pop_addr_offset(off)?;
                    let v = self.mload_i64_ext(a, 1, true)?;
                    self.push(WasmValue::I64(v))?;
                }
                0x31 => {
                    let off = self.read_mem_arg()?;
                    let a = self.pop_addr_offset(off)?;
                    let v = self.mload_i64_ext(a, 1, false)?;
                    self.push(WasmValue::I64(v))?;
                }
                0x32 => {
                    let off = self.read_mem_arg()?;
                    let a = self.pop_addr_offset(off)?;
                    let v = self.mload_i64_ext(a, 2, true)?;
                    self.push(WasmValue::I64(v))?;
                }
                0x33 => {
                    let off = self.read_mem_arg()?;
                    let a = self.pop_addr_offset(off)?;
                    let v = self.mload_i64_ext(a, 2, false)?;
                    self.push(WasmValue::I64(v))?;
                }
                0x34 => {
                    let off = self.read_mem_arg()?;
                    let a = self.pop_addr_offset(off)?;
                    let v = self.mload_i64_ext(a, 4, true)?;
                    self.push(WasmValue::I64(v))?;
                }
                0x35 => {
                    let off = self.read_mem_arg()?;
                    let a = self.pop_addr_offset(off)?;
                    let v = self.mload_i64_ext(a, 4, false)?;
                    self.push(WasmValue::I64(v))?;
                }

                // ── memory stores (0x36..=0x3E) ───────────────────────────────
                0x36 => {
                    let off = self.read_mem_arg()?;
                    let v = self.pop_i32()?;
                    let a = self.pop_addr_offset(off)?;
                    self.mstore_i32(a, v)?;
                }
                0x37 => {
                    let off = self.read_mem_arg()?;
                    let v = self.pop_i64()?;
                    let a = self.pop_addr_offset(off)?;
                    self.mstore_i64(a, v)?;
                }
                0x38 => {
                    let off = self.read_mem_arg()?;
                    let v = self.pop_f32()?;
                    let a = self.pop_addr_offset(off)?;
                    self.mstore_f32(a, v)?;
                }
                0x39 => {
                    let off = self.read_mem_arg()?;
                    let v = self.pop_f64()?;
                    let a = self.pop_addr_offset(off)?;
                    self.mstore_f64(a, v)?;
                }
                0x3A => {
                    let off = self.read_mem_arg()?;
                    let v = self.pop_i32()? as u64;
                    let a = self.pop_addr_offset(off)?;
                    self.mstore_trunc(a, 1, v)?;
                }
                0x3B => {
                    let off = self.read_mem_arg()?;
                    let v = self.pop_i32()? as u64;
                    let a = self.pop_addr_offset(off)?;
                    self.mstore_trunc(a, 2, v)?;
                }
                0x3C => {
                    let off = self.read_mem_arg()?;
                    let v = self.pop_i64()? as u64;
                    let a = self.pop_addr_offset(off)?;
                    self.mstore_trunc(a, 1, v)?;
                }
                0x3D => {
                    let off = self.read_mem_arg()?;
                    let v = self.pop_i64()? as u64;
                    let a = self.pop_addr_offset(off)?;
                    self.mstore_trunc(a, 2, v)?;
                }
                0x3E => {
                    let off = self.read_mem_arg()?;
                    let v = self.pop_i64()? as u64;
                    let a = self.pop_addr_offset(off)?;
                    self.mstore_trunc(a, 4, v)?;
                }

                // ── memory.size / memory.grow ─────────────────────────────────
                0x3F => {
                    let _ = self.read_byte()?;
                    self.push(WasmValue::I32((self.memory.data.len() / 65536) as i32))?;
                }
                0x40 => {
                    let _ = self.read_byte()?;
                    let n = self.pop_i32()?;
                    let cur = (self.memory.data.len() / 65536) as i32;
                    if n >= 0 {
                        let nb = self.memory.data.len() + (n as usize * 65536);
                        self.memory.data.resize(nb, 0);
                        self.memory.size_bytes = nb;
                        self.push(WasmValue::I32(cur))?;
                    } else {
                        self.push(WasmValue::I32(-1))?;
                    }
                }

                // ── constants ─────────────────────────────────────────────────
                0x41 => {
                    let v = self.read_i32_leb()?;
                    self.push(WasmValue::I32(v))?;
                }
                0x42 => {
                    let v = self.read_i64_leb()?;
                    self.push(WasmValue::I64(v))?;
                }
                0x43 => {
                    let v = self.read_f32()?;
                    self.push(WasmValue::F32(v))?;
                }
                0x44 => {
                    let v = self.read_f64()?;
                    self.push(WasmValue::F64(v))?;
                }

                // ── i32 comparisons (0x45..=0x4F) ────────────────────────────
                0x45 => {
                    let a = self.pop_i32()?;
                    self.push(WasmValue::I32(bool_to_i32(a == 0)))?;
                }
                0x46 => {
                    let (b, a) = (self.pop_i32()?, self.pop_i32()?);
                    self.push(WasmValue::I32(bool_to_i32(a == b)))?;
                }
                0x47 => {
                    let (b, a) = (self.pop_i32()?, self.pop_i32()?);
                    self.push(WasmValue::I32(bool_to_i32(a != b)))?;
                }
                0x48 => {
                    let (b, a) = (self.pop_i32()?, self.pop_i32()?);
                    self.push(WasmValue::I32(bool_to_i32(a < b)))?;
                }
                0x49 => {
                    let (b, a) = (self.pop_i32()? as u32, self.pop_i32()? as u32);
                    self.push(WasmValue::I32(bool_to_i32(a < b)))?;
                }
                0x4A => {
                    let (b, a) = (self.pop_i32()?, self.pop_i32()?);
                    self.push(WasmValue::I32(bool_to_i32(a > b)))?;
                }
                0x4B => {
                    let (b, a) = (self.pop_i32()? as u32, self.pop_i32()? as u32);
                    self.push(WasmValue::I32(bool_to_i32(a > b)))?;
                }
                0x4C => {
                    let (b, a) = (self.pop_i32()?, self.pop_i32()?);
                    self.push(WasmValue::I32(bool_to_i32(a <= b)))?;
                }
                0x4D => {
                    let (b, a) = (self.pop_i32()? as u32, self.pop_i32()? as u32);
                    self.push(WasmValue::I32(bool_to_i32(a <= b)))?;
                }
                0x4E => {
                    let (b, a) = (self.pop_i32()?, self.pop_i32()?);
                    self.push(WasmValue::I32(bool_to_i32(a >= b)))?;
                }
                0x4F => {
                    let (b, a) = (self.pop_i32()? as u32, self.pop_i32()? as u32);
                    self.push(WasmValue::I32(bool_to_i32(a >= b)))?;
                }

                // ── i64 comparisons (0x50..=0x5A) ────────────────────────────
                0x50 => {
                    let a = self.pop_i64()?;
                    self.push(WasmValue::I32(bool_to_i32(a == 0)))?;
                }
                0x51 => {
                    let (b, a) = (self.pop_i64()?, self.pop_i64()?);
                    self.push(WasmValue::I32(bool_to_i32(a == b)))?;
                }
                0x52 => {
                    let (b, a) = (self.pop_i64()?, self.pop_i64()?);
                    self.push(WasmValue::I32(bool_to_i32(a != b)))?;
                }
                0x53 => {
                    let (b, a) = (self.pop_i64()?, self.pop_i64()?);
                    self.push(WasmValue::I32(bool_to_i32(a < b)))?;
                }
                0x54 => {
                    let (b, a) = (self.pop_i64()? as u64, self.pop_i64()? as u64);
                    self.push(WasmValue::I32(bool_to_i32(a < b)))?;
                }
                0x55 => {
                    let (b, a) = (self.pop_i64()?, self.pop_i64()?);
                    self.push(WasmValue::I32(bool_to_i32(a > b)))?;
                }
                0x56 => {
                    let (b, a) = (self.pop_i64()? as u64, self.pop_i64()? as u64);
                    self.push(WasmValue::I32(bool_to_i32(a > b)))?;
                }
                0x57 => {
                    let (b, a) = (self.pop_i64()?, self.pop_i64()?);
                    self.push(WasmValue::I32(bool_to_i32(a <= b)))?;
                }
                0x58 => {
                    let (b, a) = (self.pop_i64()? as u64, self.pop_i64()? as u64);
                    self.push(WasmValue::I32(bool_to_i32(a <= b)))?;
                }
                0x59 => {
                    let (b, a) = (self.pop_i64()?, self.pop_i64()?);
                    self.push(WasmValue::I32(bool_to_i32(a >= b)))?;
                }
                0x5A => {
                    let (b, a) = (self.pop_i64()? as u64, self.pop_i64()? as u64);
                    self.push(WasmValue::I32(bool_to_i32(a >= b)))?;
                }

                // ── f32 comparisons (0x5B..=0x60) ────────────────────────────
                0x5B => {
                    let (b, a) = (self.pop_f32()?, self.pop_f32()?);
                    self.push(WasmValue::I32(bool_to_i32(a == b)))?;
                }
                0x5C => {
                    let (b, a) = (self.pop_f32()?, self.pop_f32()?);
                    self.push(WasmValue::I32(bool_to_i32(a != b)))?;
                }
                0x5D => {
                    let (b, a) = (self.pop_f32()?, self.pop_f32()?);
                    self.push(WasmValue::I32(bool_to_i32(a < b)))?;
                }
                0x5E => {
                    let (b, a) = (self.pop_f32()?, self.pop_f32()?);
                    self.push(WasmValue::I32(bool_to_i32(a > b)))?;
                }
                0x5F => {
                    let (b, a) = (self.pop_f32()?, self.pop_f32()?);
                    self.push(WasmValue::I32(bool_to_i32(a <= b)))?;
                }
                0x60 => {
                    let (b, a) = (self.pop_f32()?, self.pop_f32()?);
                    self.push(WasmValue::I32(bool_to_i32(a >= b)))?;
                }

                // ── f64 comparisons (0x61..=0x66) ────────────────────────────
                0x61 => {
                    let (b, a) = (self.pop_f64()?, self.pop_f64()?);
                    self.push(WasmValue::I32(bool_to_i32(a == b)))?;
                }
                0x62 => {
                    let (b, a) = (self.pop_f64()?, self.pop_f64()?);
                    self.push(WasmValue::I32(bool_to_i32(a != b)))?;
                }
                0x63 => {
                    let (b, a) = (self.pop_f64()?, self.pop_f64()?);
                    self.push(WasmValue::I32(bool_to_i32(a < b)))?;
                }
                0x64 => {
                    let (b, a) = (self.pop_f64()?, self.pop_f64()?);
                    self.push(WasmValue::I32(bool_to_i32(a > b)))?;
                }
                0x65 => {
                    let (b, a) = (self.pop_f64()?, self.pop_f64()?);
                    self.push(WasmValue::I32(bool_to_i32(a <= b)))?;
                }
                0x66 => {
                    let (b, a) = (self.pop_f64()?, self.pop_f64()?);
                    self.push(WasmValue::I32(bool_to_i32(a >= b)))?;
                }

                // ── i32 unary (0x67..=0x69) ───────────────────────────────────
                0x67 => {
                    let a = self.pop_i32()? as u32;
                    self.push(WasmValue::I32(a.leading_zeros() as i32))?;
                }
                0x68 => {
                    let a = self.pop_i32()? as u32;
                    self.push(WasmValue::I32(a.trailing_zeros() as i32))?;
                }
                0x69 => {
                    let a = self.pop_i32()? as u32;
                    self.push(WasmValue::I32(a.count_ones() as i32))?;
                }

                // ── i32 binary arithmetic (0x6A..=0x78) ──────────────────────
                0x6A => {
                    let (b, a) = (self.pop_i32()?, self.pop_i32()?);
                    self.push(WasmValue::I32(a.wrapping_add(b)))?;
                }
                0x6B => {
                    let (b, a) = (self.pop_i32()?, self.pop_i32()?);
                    self.push(WasmValue::I32(a.wrapping_sub(b)))?;
                }
                0x6C => {
                    let (b, a) = (self.pop_i32()?, self.pop_i32()?);
                    self.push(WasmValue::I32(a.wrapping_mul(b)))?;
                }
                0x6D => {
                    // i32.div_s
                    let (b, a) = (self.pop_i32()?, self.pop_i32()?);
                    if b == 0 {
                        return Err(WasmError::DivisionByZero);
                    }
                    if a == i32::MIN && b == -1 {
                        return Err(WasmError::Trap("i32 div overflow".into()));
                    }
                    self.push(WasmValue::I32(a / b))?;
                }
                0x6E => {
                    // i32.div_u
                    let (b, a) = (self.pop_i32()? as u32, self.pop_i32()? as u32);
                    if b == 0 {
                        return Err(WasmError::DivisionByZero);
                    }
                    self.push(WasmValue::I32((a / b) as i32))?;
                }
                0x6F => {
                    // i32.rem_s
                    let (b, a) = (self.pop_i32()?, self.pop_i32()?);
                    if b == 0 {
                        return Err(WasmError::DivisionByZero);
                    }
                    self.push(WasmValue::I32(a.wrapping_rem(b)))?;
                }
                0x70 => {
                    // i32.rem_u
                    let (b, a) = (self.pop_i32()? as u32, self.pop_i32()? as u32);
                    if b == 0 {
                        return Err(WasmError::DivisionByZero);
                    }
                    self.push(WasmValue::I32((a % b) as i32))?;
                }
                0x71 => {
                    let (b, a) = (self.pop_i32()?, self.pop_i32()?);
                    self.push(WasmValue::I32(a & b))?;
                }
                0x72 => {
                    let (b, a) = (self.pop_i32()?, self.pop_i32()?);
                    self.push(WasmValue::I32(a | b))?;
                }
                0x73 => {
                    let (b, a) = (self.pop_i32()?, self.pop_i32()?);
                    self.push(WasmValue::I32(a ^ b))?;
                }
                0x74 => {
                    let (b, a) = (self.pop_i32()? as u32 & 31, self.pop_i32()?);
                    self.push(WasmValue::I32(a.wrapping_shl(b)))?;
                }
                0x75 => {
                    let (b, a) = (self.pop_i32()? as u32 & 31, self.pop_i32()?);
                    self.push(WasmValue::I32(a.wrapping_shr(b)))?;
                }
                0x76 => {
                    let (b, a) = (self.pop_i32()? as u32 & 31, self.pop_i32()? as u32);
                    self.push(WasmValue::I32(a.wrapping_shr(b) as i32))?;
                }
                0x77 => {
                    let (b, a) = (self.pop_i32()? as u32 & 31, self.pop_i32()? as u32);
                    self.push(WasmValue::I32(a.rotate_left(b) as i32))?;
                }
                0x78 => {
                    let (b, a) = (self.pop_i32()? as u32 & 31, self.pop_i32()? as u32);
                    self.push(WasmValue::I32(a.rotate_right(b) as i32))?;
                }

                // ── i64 unary (0x79..=0x7B) ───────────────────────────────────
                0x79 => {
                    let a = self.pop_i64()? as u64;
                    self.push(WasmValue::I64(a.leading_zeros() as i64))?;
                }
                0x7A => {
                    let a = self.pop_i64()? as u64;
                    self.push(WasmValue::I64(a.trailing_zeros() as i64))?;
                }
                0x7B => {
                    let a = self.pop_i64()? as u64;
                    self.push(WasmValue::I64(a.count_ones() as i64))?;
                }

                // ── i64 binary arithmetic (0x7C..=0x8A) ──────────────────────
                0x7C => {
                    let (b, a) = (self.pop_i64()?, self.pop_i64()?);
                    self.push(WasmValue::I64(a.wrapping_add(b)))?;
                }
                0x7D => {
                    let (b, a) = (self.pop_i64()?, self.pop_i64()?);
                    self.push(WasmValue::I64(a.wrapping_sub(b)))?;
                }
                0x7E => {
                    let (b, a) = (self.pop_i64()?, self.pop_i64()?);
                    self.push(WasmValue::I64(a.wrapping_mul(b)))?;
                }
                0x7F => {
                    // i64.div_s
                    let (b, a) = (self.pop_i64()?, self.pop_i64()?);
                    if b == 0 {
                        return Err(WasmError::DivisionByZero);
                    }
                    if a == i64::MIN && b == -1 {
                        return Err(WasmError::Trap("i64 div overflow".into()));
                    }
                    self.push(WasmValue::I64(a / b))?;
                }
                0x80 => {
                    let (b, a) = (self.pop_i64()? as u64, self.pop_i64()? as u64);
                    if b == 0 {
                        return Err(WasmError::DivisionByZero);
                    }
                    self.push(WasmValue::I64((a / b) as i64))?;
                }
                0x81 => {
                    let (b, a) = (self.pop_i64()?, self.pop_i64()?);
                    if b == 0 {
                        return Err(WasmError::DivisionByZero);
                    }
                    self.push(WasmValue::I64(a.wrapping_rem(b)))?;
                }
                0x82 => {
                    let (b, a) = (self.pop_i64()? as u64, self.pop_i64()? as u64);
                    if b == 0 {
                        return Err(WasmError::DivisionByZero);
                    }
                    self.push(WasmValue::I64((a % b) as i64))?;
                }
                0x83 => {
                    let (b, a) = (self.pop_i64()?, self.pop_i64()?);
                    self.push(WasmValue::I64(a & b))?;
                }
                0x84 => {
                    let (b, a) = (self.pop_i64()?, self.pop_i64()?);
                    self.push(WasmValue::I64(a | b))?;
                }
                0x85 => {
                    let (b, a) = (self.pop_i64()?, self.pop_i64()?);
                    self.push(WasmValue::I64(a ^ b))?;
                }
                0x86 => {
                    let (b, a) = (self.pop_i64()? as u32 & 63, self.pop_i64()?);
                    self.push(WasmValue::I64(a.wrapping_shl(b)))?;
                }
                0x87 => {
                    let (b, a) = (self.pop_i64()? as u32 & 63, self.pop_i64()?);
                    self.push(WasmValue::I64(a.wrapping_shr(b)))?;
                }
                0x88 => {
                    let (b, a) = (self.pop_i64()? as u32 & 63, self.pop_i64()? as u64);
                    self.push(WasmValue::I64(a.wrapping_shr(b) as i64))?;
                }
                0x89 => {
                    let (b, a) = (self.pop_i64()? as u32 & 63, self.pop_i64()? as u64);
                    self.push(WasmValue::I64(a.rotate_left(b) as i64))?;
                }
                0x8A => {
                    let (b, a) = (self.pop_i64()? as u32 & 63, self.pop_i64()? as u64);
                    self.push(WasmValue::I64(a.rotate_right(b) as i64))?;
                }

                // ── f32 arithmetic (0x8B..=0x98) ──────────────────────────────
                0x8B => {
                    let a = self.pop_f32()?;
                    self.push(WasmValue::F32(a.abs()))?;
                }
                0x8C => {
                    let a = self.pop_f32()?;
                    self.push(WasmValue::F32(-a))?;
                }
                0x8D => {
                    let a = self.pop_f32()?;
                    self.push(WasmValue::F32(a.ceil()))?;
                }
                0x8E => {
                    let a = self.pop_f32()?;
                    self.push(WasmValue::F32(a.floor()))?;
                }
                0x8F => {
                    let a = self.pop_f32()?;
                    self.push(WasmValue::F32(a.trunc()))?;
                }
                0x90 => {
                    let a = self.pop_f32()?;
                    self.push(WasmValue::F32(a.round()))?;
                }
                0x91 => {
                    let a = self.pop_f32()?;
                    self.push(WasmValue::F32(a.sqrt()))?;
                }
                0x92 => {
                    let (b, a) = (self.pop_f32()?, self.pop_f32()?);
                    self.push(WasmValue::F32(a + b))?;
                }
                0x93 => {
                    let (b, a) = (self.pop_f32()?, self.pop_f32()?);
                    self.push(WasmValue::F32(a - b))?;
                }
                0x94 => {
                    let (b, a) = (self.pop_f32()?, self.pop_f32()?);
                    self.push(WasmValue::F32(a * b))?;
                }
                0x95 => {
                    let (b, a) = (self.pop_f32()?, self.pop_f32()?);
                    self.push(WasmValue::F32(a / b))?;
                }
                0x96 => {
                    let (b, a) = (self.pop_f32()?, self.pop_f32()?);
                    self.push(WasmValue::F32(a.min(b)))?;
                }
                0x97 => {
                    let (b, a) = (self.pop_f32()?, self.pop_f32()?);
                    self.push(WasmValue::F32(a.max(b)))?;
                }
                0x98 => {
                    let (b, a) = (self.pop_f32()?, self.pop_f32()?);
                    self.push(WasmValue::F32(a.copysign(b)))?;
                }

                // ── f64 arithmetic (0x99..=0xA6) ──────────────────────────────
                0x99 => {
                    let a = self.pop_f64()?;
                    self.push(WasmValue::F64(a.abs()))?;
                }
                0x9A => {
                    let a = self.pop_f64()?;
                    self.push(WasmValue::F64(-a))?;
                }
                0x9B => {
                    let a = self.pop_f64()?;
                    self.push(WasmValue::F64(a.ceil()))?;
                }
                0x9C => {
                    let a = self.pop_f64()?;
                    self.push(WasmValue::F64(a.floor()))?;
                }
                0x9D => {
                    let a = self.pop_f64()?;
                    self.push(WasmValue::F64(a.trunc()))?;
                }
                0x9E => {
                    let a = self.pop_f64()?;
                    self.push(WasmValue::F64(a.round()))?;
                }
                0x9F => {
                    let a = self.pop_f64()?;
                    self.push(WasmValue::F64(a.sqrt()))?;
                }
                0xA0 => {
                    let (b, a) = (self.pop_f64()?, self.pop_f64()?);
                    self.push(WasmValue::F64(a + b))?;
                }
                0xA1 => {
                    let (b, a) = (self.pop_f64()?, self.pop_f64()?);
                    self.push(WasmValue::F64(a - b))?;
                }
                0xA2 => {
                    let (b, a) = (self.pop_f64()?, self.pop_f64()?);
                    self.push(WasmValue::F64(a * b))?;
                }
                0xA3 => {
                    let (b, a) = (self.pop_f64()?, self.pop_f64()?);
                    self.push(WasmValue::F64(a / b))?;
                }
                0xA4 => {
                    let (b, a) = (self.pop_f64()?, self.pop_f64()?);
                    self.push(WasmValue::F64(a.min(b)))?;
                }
                0xA5 => {
                    let (b, a) = (self.pop_f64()?, self.pop_f64()?);
                    self.push(WasmValue::F64(a.max(b)))?;
                }
                0xA6 => {
                    let (b, a) = (self.pop_f64()?, self.pop_f64()?);
                    self.push(WasmValue::F64(a.copysign(b)))?;
                }

                // ── conversions (0xA7..=0xBF) ─────────────────────────────────
                0xA7 => {
                    let a = self.pop_i64()?;
                    self.push(WasmValue::I32(a as i32))?;
                }
                0xA8 => {
                    let a = self.pop_f32()?;
                    check_float_f32(a)?;
                    self.push(WasmValue::I32(a as i32))?;
                }
                0xA9 => {
                    let a = self.pop_f32()?;
                    check_float_f32(a)?;
                    self.push(WasmValue::I32(a as u32 as i32))?;
                }
                0xAA => {
                    let a = self.pop_f64()?;
                    check_float_f64(a)?;
                    self.push(WasmValue::I32(a as i32))?;
                }
                0xAB => {
                    let a = self.pop_f64()?;
                    check_float_f64(a)?;
                    self.push(WasmValue::I32(a as u32 as i32))?;
                }
                0xAC => {
                    let a = self.pop_i32()?;
                    self.push(WasmValue::I64(a as i64))?;
                }
                0xAD => {
                    let a = self.pop_i32()? as u32;
                    self.push(WasmValue::I64(a as i64))?;
                }
                0xAE => {
                    let a = self.pop_f32()?;
                    check_float_f32(a)?;
                    self.push(WasmValue::I64(a as i64))?;
                }
                0xAF => {
                    let a = self.pop_f32()?;
                    check_float_f32(a)?;
                    self.push(WasmValue::I64(a as u64 as i64))?;
                }
                0xB0 => {
                    let a = self.pop_f64()?;
                    check_float_f64(a)?;
                    self.push(WasmValue::I64(a as i64))?;
                }
                0xB1 => {
                    let a = self.pop_f64()?;
                    check_float_f64(a)?;
                    self.push(WasmValue::I64(a as u64 as i64))?;
                }
                0xB2 => {
                    let a = self.pop_i32()?;
                    self.push(WasmValue::F32(a as f32))?;
                }
                0xB3 => {
                    let a = self.pop_i32()? as u32;
                    self.push(WasmValue::F32(a as f32))?;
                }
                0xB4 => {
                    let a = self.pop_i64()?;
                    self.push(WasmValue::F32(a as f32))?;
                }
                0xB5 => {
                    let a = self.pop_i64()? as u64;
                    self.push(WasmValue::F32(a as f32))?;
                }
                0xB6 => {
                    let a = self.pop_f64()?;
                    self.push(WasmValue::F32(a as f32))?;
                }
                0xB7 => {
                    let a = self.pop_i32()?;
                    self.push(WasmValue::F64(a as f64))?;
                }
                0xB8 => {
                    let a = self.pop_i32()? as u32;
                    self.push(WasmValue::F64(a as f64))?;
                }
                0xB9 => {
                    let a = self.pop_i64()?;
                    self.push(WasmValue::F64(a as f64))?;
                }
                0xBA => {
                    let a = self.pop_i64()? as u64;
                    self.push(WasmValue::F64(a as f64))?;
                }
                0xBB => {
                    let a = self.pop_f32()?;
                    self.push(WasmValue::F64(a as f64))?;
                }
                0xBC => {
                    let a = self.pop_f32()?;
                    self.push(WasmValue::I32(i32::from_le_bytes(a.to_le_bytes())))?;
                }
                0xBD => {
                    let a = self.pop_f64()?;
                    self.push(WasmValue::I64(i64::from_le_bytes(a.to_le_bytes())))?;
                }
                0xBE => {
                    let a = self.pop_i32()?;
                    self.push(WasmValue::F32(f32::from_le_bytes(a.to_le_bytes())))?;
                }
                0xBF => {
                    let a = self.pop_i64()?;
                    self.push(WasmValue::F64(f64::from_le_bytes(a.to_le_bytes())))?;
                }

                other => return Err(WasmError::UnimplementedOpcode(other)),
            }
        }

        // Collect return values from top of stack.
        let slen = self.stack.len();
        Ok(if n_results <= slen {
            self.stack[slen - n_results..].to_vec()
        } else {
            self.stack.clone()
        })
    }
}

// ─── Free helpers ─────────────────────────────────────────────────────────────

#[inline]
fn bool_to_i32(b: bool) -> i32 {
    if b {
        1
    } else {
        0
    }
}

fn check_float_f32(v: f32) -> Result<(), WasmError> {
    if v.is_nan() || v.is_infinite() {
        Err(WasmError::Trap("float conversion: invalid value".into()))
    } else {
        Ok(())
    }
}
fn check_float_f64(v: f64) -> Result<(), WasmError> {
    if v.is_nan() || v.is_infinite() {
        Err(WasmError::Trap("float conversion: invalid value".into()))
    } else {
        Ok(())
    }
}

fn sign_extend_u32(raw: u32, bits: usize) -> i32 {
    let shift = 32 - bits;
    ((raw << shift) as i32) >> shift
}
fn sign_extend_u64(raw: u64, bits: usize) -> i64 {
    let shift = 64 - bits;
    ((raw << shift) as i64) >> shift
}

// ─── Public API ──────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn default_wasm_bridge_config() -> WasmBridgeConfig {
    WasmBridgeConfig {
        memory_pages: 1,
        max_functions: 256,
        debug_mode: false,
    }
}

#[allow(dead_code)]
pub fn new_wasm_bridge(cfg: WasmBridgeConfig) -> WasmBridge {
    let size_bytes = cfg.memory_pages as usize * 65536;
    WasmBridge {
        config: cfg,
        memory: WasmMemory {
            data: vec![0u8; size_bytes],
            size_bytes,
        },
        functions: Vec::new(),
        call_count: 0,
    }
}

#[allow(dead_code)]
pub fn wasm_memory_size(bridge: &WasmBridge) -> usize {
    bridge.memory.size_bytes
}

#[allow(dead_code)]
pub fn wasm_read_u32(bridge: &WasmBridge, offset: usize) -> Option<u32> {
    if offset + 4 > bridge.memory.data.len() {
        return None;
    }
    Some(u32::from_le_bytes([
        bridge.memory.data[offset],
        bridge.memory.data[offset + 1],
        bridge.memory.data[offset + 2],
        bridge.memory.data[offset + 3],
    ]))
}

#[allow(dead_code)]
pub fn wasm_write_u32(bridge: &mut WasmBridge, offset: usize, val: u32) -> bool {
    if offset + 4 > bridge.memory.data.len() {
        return false;
    }
    let b = val.to_le_bytes();
    bridge.memory.data[offset..offset + 4].copy_from_slice(&b);
    true
}

#[allow(dead_code)]
pub fn register_wasm_function(bridge: &mut WasmBridge, func: WasmFunction) {
    if bridge.functions.len() < bridge.config.max_functions {
        bridge.functions.push(func);
    }
}

#[allow(dead_code)]
pub fn wasm_function_count(bridge: &WasmBridge) -> usize {
    bridge.functions.len()
}

/// Execute a named WASM function with the given arguments.
///
/// The function's `bytecode` starts with a LEB128 local-variable declaration
/// block, then the expression opcodes.  Empty `bytecode` returns `Ok([])`.
#[allow(dead_code)]
pub fn wasm_call(
    bridge: &mut WasmBridge,
    name: &str,
    args: &[WasmValue],
) -> Result<Vec<WasmValue>, WasmError> {
    let func = bridge
        .functions
        .iter()
        .find(|f| f.name == name)
        .cloned()
        .ok_or_else(|| WasmError::FunctionNotFound(name.to_string()))?;

    bridge.call_count += 1;
    if func.bytecode.is_empty() {
        return Ok(Vec::new());
    }

    // Decode local-variable declaration header.
    let mut pc = 0usize;
    let n_decls = read_leb128_u32(&func.bytecode, &mut pc) as usize;
    let mut extra_locals: Vec<WasmValue> = Vec::new();
    for _ in 0..n_decls {
        let count = read_leb128_u32(&func.bytecode, &mut pc) as usize;
        let type_byte = func.bytecode[pc];
        pc += 1;
        let default = match type_byte {
            0x7F => WasmValue::I32(0),
            0x7E => WasmValue::I64(0),
            0x7D => WasmValue::F32(0.0),
            0x7C => WasmValue::F64(0.0),
            _ => WasmValue::I32(0),
        };
        for _ in 0..count {
            extra_locals.push(default.clone());
        }
    }

    let mut locals: Vec<WasmValue> = args.to_vec();
    locals.extend(extra_locals);

    let expr_code = &func.bytecode[pc..];
    let n_results = func.return_types.len();
    let mut interp = Interpreter::new(&mut bridge.memory, expr_code, locals, 0);
    interp.run(n_results)
}

/// Backward-compatible stub: calls `wasm_call` with no arguments.
/// Returns `true` if the call succeeds (function found and executed without trap).
#[allow(dead_code)]
pub fn wasm_call_stub(bridge: &mut WasmBridge, name: &str) -> bool {
    wasm_call(bridge, name, &[]).is_ok()
}

#[allow(dead_code)]
pub fn value_type_name(t: &WasmValueType) -> &'static str {
    match t {
        WasmValueType::I32 => "i32",
        WasmValueType::I64 => "i64",
        WasmValueType::F32 => "f32",
        WasmValueType::F64 => "f64",
        WasmValueType::FuncRef => "funcref",
        WasmValueType::ExternRef => "externref",
    }
}

#[allow(dead_code)]
pub fn wasm_bridge_to_json(bridge: &WasmBridge) -> String {
    format!(
        "{{\"memory_pages\":{},\"function_count\":{},\"call_count\":{},\"debug_mode\":{}}}",
        bridge.config.memory_pages,
        bridge.functions.len(),
        bridge.call_count,
        bridge.config.debug_mode
    )
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── backward-compatible tests ─────────────────────────────────────────────

    #[test]
    fn test_default_config() {
        let cfg = default_wasm_bridge_config();
        assert_eq!(cfg.memory_pages, 1);
        assert_eq!(cfg.max_functions, 256);
        assert!(!cfg.debug_mode);
    }

    #[test]
    fn test_new_bridge_memory_size() {
        let bridge = new_wasm_bridge(default_wasm_bridge_config());
        assert_eq!(wasm_memory_size(&bridge), 65536);
    }

    #[test]
    fn test_write_read_u32() {
        let mut bridge = new_wasm_bridge(default_wasm_bridge_config());
        assert!(wasm_write_u32(&mut bridge, 0, 0xDEAD_BEEF));
        assert_eq!(wasm_read_u32(&bridge, 0), Some(0xDEAD_BEEF));
    }

    #[test]
    fn test_out_of_bounds_returns_none() {
        let bridge = new_wasm_bridge(default_wasm_bridge_config());
        assert!(wasm_read_u32(&bridge, 65534).is_none());
    }

    #[test]
    fn test_register_and_call_function() {
        let mut bridge = new_wasm_bridge(default_wasm_bridge_config());
        let func = WasmFunction {
            name: "add".to_string(),
            param_types: vec![WasmValueType::I32, WasmValueType::I32],
            return_types: vec![WasmValueType::I32],
            bytecode: Vec::new(),
            local_types: Vec::new(),
        };
        register_wasm_function(&mut bridge, func);
        assert_eq!(wasm_function_count(&bridge), 1);
        assert!(wasm_call_stub(&mut bridge, "add"));
        assert_eq!(bridge.call_count, 1);
    }

    #[test]
    fn test_call_unknown_function() {
        let mut bridge = new_wasm_bridge(default_wasm_bridge_config());
        assert!(!wasm_call_stub(&mut bridge, "nonexistent"));
        assert_eq!(bridge.call_count, 0);
    }

    #[test]
    fn test_value_type_names() {
        assert_eq!(value_type_name(&WasmValueType::I32), "i32");
        assert_eq!(value_type_name(&WasmValueType::I64), "i64");
        assert_eq!(value_type_name(&WasmValueType::F32), "f32");
        assert_eq!(value_type_name(&WasmValueType::F64), "f64");
        assert_eq!(value_type_name(&WasmValueType::FuncRef), "funcref");
        assert_eq!(value_type_name(&WasmValueType::ExternRef), "externref");
    }

    #[test]
    fn test_bridge_to_json() {
        let bridge = new_wasm_bridge(default_wasm_bridge_config());
        let json = wasm_bridge_to_json(&bridge);
        assert!(json.contains("memory_pages"));
        assert!(json.contains("call_count"));
    }

    // ── interpreter tests ─────────────────────────────────────────────────────

    fn make_func(
        name: &str,
        params: Vec<WasmValueType>,
        rets: Vec<WasmValueType>,
        bytecode: Vec<u8>,
    ) -> WasmFunction {
        WasmFunction {
            name: name.to_string(),
            param_types: params,
            return_types: rets,
            bytecode,
            local_types: Vec::new(),
        }
    }

    /// (param i32 i32) (result i32): local.get 0, local.get 1, i32.add, end
    /// bytecode = [0 local decls] + [0x20,0x00, 0x20,0x01, 0x6A, 0x0B]
    #[test]
    fn test_wasm_call_add_function() {
        let mut bridge = new_wasm_bridge(default_wasm_bridge_config());
        let f = make_func(
            "add",
            vec![WasmValueType::I32, WasmValueType::I32],
            vec![WasmValueType::I32],
            vec![0x00, 0x20, 0x00, 0x20, 0x01, 0x6A, 0x0B],
        );
        register_wasm_function(&mut bridge, f);
        let r = wasm_call(&mut bridge, "add", &[WasmValue::I32(3), WasmValue::I32(4)]).unwrap();
        assert_eq!(r, vec![WasmValue::I32(7)]);
    }

    /// i32.const 42 (two-byte: 0x41 0x2A), end
    #[test]
    fn test_wasm_call_constant() {
        let mut bridge = new_wasm_bridge(default_wasm_bridge_config());
        let f = make_func(
            "c42",
            vec![],
            vec![WasmValueType::I32],
            vec![0x00, 0x41, 0x2A, 0x0B],
        );
        register_wasm_function(&mut bridge, f);
        assert_eq!(
            wasm_call(&mut bridge, "c42", &[]).unwrap(),
            vec![WasmValue::I32(42)]
        );
    }

    /// Write 99 to memory[0] and read it back.
    /// bytecode: [0 local decls][i32.const 0][i32.const 99 (two-byte LEB)][i32.store align=2 offset=0][end]
    /// i32.const 99: 0x41, 0xE3, 0x00  (99 = 0x63; as two-byte signed LEB: 0xE3 0x00)
    #[test]
    fn test_wasm_call_memory_write_read() {
        let mut bridge = new_wasm_bridge(default_wasm_bridge_config());
        let bytecode = vec![
            0x00, // 0 local decls
            0x41, 0x00, // i32.const 0 (address)
            0x41, 0xE3, 0x00, // i32.const 99 (proper two-byte signed LEB128)
            0x36, 0x02, 0x00, // i32.store align=2 offset=0
            0x0B, // end
        ];
        let f = make_func("write_99", vec![], vec![], bytecode);
        register_wasm_function(&mut bridge, f);
        let result = wasm_call(&mut bridge, "write_99", &[]);
        assert!(result.is_ok(), "Expected Ok but got: {:?}", result);
        assert_eq!(wasm_read_u32(&bridge, 0), Some(99));
    }

    #[test]
    fn test_wasm_call_unknown_function_error() {
        let mut bridge = new_wasm_bridge(default_wasm_bridge_config());
        let r = wasm_call(&mut bridge, "no_such_function", &[]);
        assert!(matches!(r, Err(WasmError::FunctionNotFound(_))));
    }

    /// Empty bytecode → Ok([])
    #[test]
    fn test_wasm_call_no_bytecode_succeeds() {
        let mut bridge = new_wasm_bridge(default_wasm_bridge_config());
        let f = make_func("empty", vec![], vec![], vec![0x00, 0x0B]);
        register_wasm_function(&mut bridge, f);
        assert_eq!(wasm_call(&mut bridge, "empty", &[]).unwrap(), vec![]);
    }

    // ── additional correctness ────────────────────────────────────────────────

    #[test]
    fn test_wasm_call_i32_sub() {
        let mut bridge = new_wasm_bridge(default_wasm_bridge_config());
        let f = make_func(
            "sub",
            vec![WasmValueType::I32, WasmValueType::I32],
            vec![WasmValueType::I32],
            vec![0x00, 0x20, 0x00, 0x20, 0x01, 0x6B, 0x0B],
        );
        register_wasm_function(&mut bridge, f);
        assert_eq!(
            wasm_call(&mut bridge, "sub", &[WasmValue::I32(10), WasmValue::I32(3)]).unwrap(),
            vec![WasmValue::I32(7)]
        );
    }

    #[test]
    fn test_wasm_call_i32_mul() {
        let mut bridge = new_wasm_bridge(default_wasm_bridge_config());
        let f = make_func(
            "mul",
            vec![WasmValueType::I32, WasmValueType::I32],
            vec![WasmValueType::I32],
            vec![0x00, 0x20, 0x00, 0x20, 0x01, 0x6C, 0x0B],
        );
        register_wasm_function(&mut bridge, f);
        assert_eq!(
            wasm_call(&mut bridge, "mul", &[WasmValue::I32(6), WasmValue::I32(7)]).unwrap(),
            vec![WasmValue::I32(42)]
        );
    }

    #[test]
    fn test_wasm_call_div_by_zero_error() {
        let mut bridge = new_wasm_bridge(default_wasm_bridge_config());
        let f = make_func(
            "div",
            vec![WasmValueType::I32, WasmValueType::I32],
            vec![WasmValueType::I32],
            vec![0x00, 0x20, 0x00, 0x20, 0x01, 0x6D, 0x0B],
        );
        register_wasm_function(&mut bridge, f);
        let r = wasm_call(&mut bridge, "div", &[WasmValue::I32(10), WasmValue::I32(0)]);
        assert!(matches!(r, Err(WasmError::DivisionByZero)));
    }

    #[test]
    fn test_wasm_call_i32_eqz() {
        let mut bridge = new_wasm_bridge(default_wasm_bridge_config());
        let f = make_func(
            "eqz",
            vec![WasmValueType::I32],
            vec![WasmValueType::I32],
            vec![0x00, 0x20, 0x00, 0x45, 0x0B],
        );
        register_wasm_function(&mut bridge, f);
        assert_eq!(
            wasm_call(&mut bridge, "eqz", &[WasmValue::I32(0)]).unwrap(),
            vec![WasmValue::I32(1)]
        );
        assert_eq!(
            wasm_call(&mut bridge, "eqz", &[WasmValue::I32(5)]).unwrap(),
            vec![WasmValue::I32(0)]
        );
    }

    #[test]
    fn test_wasm_call_local_tee() {
        let mut bridge = new_wasm_bridge(default_wasm_bridge_config());
        // local.get 0, local.tee 0, drop, local.get 0, end
        let f = make_func(
            "tee",
            vec![WasmValueType::I32],
            vec![WasmValueType::I32],
            vec![0x00, 0x20, 0x00, 0x22, 0x00, 0x1A, 0x20, 0x00, 0x0B],
        );
        register_wasm_function(&mut bridge, f);
        assert_eq!(
            wasm_call(&mut bridge, "tee", &[WasmValue::I32(77)]).unwrap(),
            vec![WasmValue::I32(77)]
        );
    }

    #[test]
    fn test_wasm_call_select() {
        let mut bridge = new_wasm_bridge(default_wasm_bridge_config());
        // i32.const 10, i32.const 20, i32.const 1, select → 10
        let f = make_func(
            "sel",
            vec![],
            vec![WasmValueType::I32],
            vec![0x00, 0x41, 0x0A, 0x41, 0x14, 0x41, 0x01, 0x1B, 0x0B],
        );
        register_wasm_function(&mut bridge, f);
        assert_eq!(
            wasm_call(&mut bridge, "sel", &[]).unwrap(),
            vec![WasmValue::I32(10)]
        );
    }

    #[test]
    fn test_wasm_call_f32_add() {
        let mut bridge = new_wasm_bridge(default_wasm_bridge_config());
        let mut bc = vec![0x00u8]; // 0 locals
        bc.push(0x43);
        bc.extend_from_slice(&1.5f32.to_le_bytes());
        bc.push(0x43);
        bc.extend_from_slice(&2.5f32.to_le_bytes());
        bc.push(0x92);
        bc.push(0x0B); // f32.add, end
        let f = make_func("f32a", vec![], vec![WasmValueType::F32], bc);
        register_wasm_function(&mut bridge, f);
        assert_eq!(
            wasm_call(&mut bridge, "f32a", &[]).unwrap(),
            vec![WasmValue::F32(4.0)]
        );
    }

    #[test]
    fn test_wasm_call_i32_conversion() {
        let mut bridge = new_wasm_bridge(default_wasm_bridge_config());
        // i32.const 300 (LEB: 0xAC 0x02), i64.extend_i32_s, i32.wrap_i64, end
        let f = make_func(
            "rt",
            vec![],
            vec![WasmValueType::I32],
            vec![0x00, 0x41, 0xAC, 0x02, 0xAC, 0xA7, 0x0B],
        );
        register_wasm_function(&mut bridge, f);
        assert_eq!(
            wasm_call(&mut bridge, "rt", &[]).unwrap(),
            vec![WasmValue::I32(300)]
        );
    }
}
