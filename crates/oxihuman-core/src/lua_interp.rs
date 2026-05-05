// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
//! Lua evaluator: environment, interpreter, helpers, and global env builder.
//! Included via #[path] from lua.rs.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// All types (LuaValue, Env, Interpreter, etc.) live in the parent lua.rs module.
// Since this file is included via `#[path]`, `super` refers to lua.rs's parent.
// Use the types directly — they're in scope from the enclosing module.
#[allow(unused_imports)]
use super::*;

// ===== Env implementation =====

impl Env {
    pub(super) fn new_child(parent: Rc<RefCell<Env>>) -> Rc<RefCell<Env>> {
        Rc::new(RefCell::new(Env {
            vars: HashMap::new(),
            parent: Some(parent),
        }))
    }

    pub(super) fn get(&self, name: &str) -> Option<LuaValue> {
        if let Some(v) = self.vars.get(name) {
            return Some(v.clone());
        }
        self.parent.as_ref().and_then(|p| p.borrow().get(name))
    }

    pub(super) fn set(&mut self, name: &str, val: LuaValue) {
        if self.vars.contains_key(name) {
            self.vars.insert(name.to_string(), val);
            return;
        }
        if let Some(parent) = &self.parent {
            let has = parent.borrow().vars.contains_key(name);
            if has || parent.borrow().has_in_chain(name) {
                parent.borrow_mut().set(name, val);
                return;
            }
        }
        self.vars.insert(name.to_string(), val);
    }

    pub(super) fn has_in_chain(&self, name: &str) -> bool {
        if self.vars.contains_key(name) {
            return true;
        }
        self.parent
            .as_ref()
            .is_some_and(|p| p.borrow().has_in_chain(name))
    }

    pub(super) fn define_local(&mut self, name: &str, val: LuaValue) {
        self.vars.insert(name.to_string(), val);
    }
}

// ===== Interpreter implementation =====

impl Interpreter {
    pub(super) fn check_timeout(&self) -> Result<(), String> {
        if self.timeout_ms > 0 && self.stmt_count.is_multiple_of(1000) {
            let elapsed = self.start_time.elapsed().as_millis() as u64;
            if elapsed > self.timeout_ms {
                return Err(format!("script timeout ({}ms)", self.timeout_ms));
            }
        }
        Ok(())
    }

    pub(super) fn exec_block(
        &mut self,
        stmts: &[Stmt],
        env: &Rc<RefCell<Env>>,
    ) -> Result<Signal, String> {
        for stmt in stmts {
            self.stmt_count += 1;
            if self.stmt_count.is_multiple_of(1000) {
                self.check_timeout()?;
            }
            let sig = self.exec_stmt(stmt, env)?;
            match sig {
                Signal::None => {}
                other => return Ok(other),
            }
        }
        Ok(Signal::None)
    }

    pub(super) fn exec_stmt(
        &mut self,
        stmt: &Stmt,
        env: &Rc<RefCell<Env>>,
    ) -> Result<Signal, String> {
        match stmt {
            Stmt::Local { names, values } => {
                let vals = self.eval_expr_list(values, env, names.len())?;
                for (i, name) in names.iter().enumerate() {
                    let v = vals.get(i).cloned().unwrap_or(LuaValue::Nil);
                    env.borrow_mut().define_local(name, v);
                }
                Ok(Signal::None)
            }

            Stmt::Assign { targets, values } => {
                let vals = self.eval_expr_list(values, env, targets.len())?;
                for (i, target) in targets.iter().enumerate() {
                    let val = vals.get(i).cloned().unwrap_or(LuaValue::Nil);
                    self.assign_target(target, val, env)?;
                }
                Ok(Signal::None)
            }

            Stmt::Do(block) => {
                let child = Env::new_child(Rc::clone(env));
                self.exec_block(block, &child)
            }

            Stmt::While { cond, body } => {
                loop {
                    let v = self.eval_expr(cond, env)?;
                    if !lua_truthy(&v) {
                        break;
                    }
                    let child = Env::new_child(Rc::clone(env));
                    match self.exec_block(body, &child)? {
                        Signal::Break => break,
                        Signal::Return(vals) => return Ok(Signal::Return(vals)),
                        Signal::None => {}
                    }
                }
                Ok(Signal::None)
            }

            Stmt::Repeat { body, cond } => {
                loop {
                    let child = Env::new_child(Rc::clone(env));
                    match self.exec_block(body, &child)? {
                        Signal::Break => break,
                        Signal::Return(vals) => return Ok(Signal::Return(vals)),
                        Signal::None => {}
                    }
                    let v = self.eval_expr(cond, &child)?;
                    if lua_truthy(&v) {
                        break;
                    }
                }
                Ok(Signal::None)
            }

            Stmt::If {
                cond,
                then_block,
                elseif_blocks,
                else_block,
            } => {
                let v = self.eval_expr(cond, env)?;
                if lua_truthy(&v) {
                    let child = Env::new_child(Rc::clone(env));
                    return self.exec_block(then_block, &child);
                }
                for (ec, eb) in elseif_blocks {
                    let v = self.eval_expr(ec, env)?;
                    if lua_truthy(&v) {
                        let child = Env::new_child(Rc::clone(env));
                        return self.exec_block(eb, &child);
                    }
                }
                if let Some(eb) = else_block {
                    let child = Env::new_child(Rc::clone(env));
                    return self.exec_block(eb, &child);
                }
                Ok(Signal::None)
            }

            Stmt::NumFor {
                var,
                start,
                limit,
                step,
                body,
            } => {
                let start_v = self.eval_expr(start, env)?;
                let limit_v = self.eval_expr(limit, env)?;
                let step_v = match step {
                    Some(e) => self.eval_expr(e, env)?,
                    None => LuaValue::Int(1),
                };
                match (&start_v, &limit_v, &step_v) {
                    (LuaValue::Int(s), LuaValue::Int(l), LuaValue::Int(st)) => {
                        let (mut i, lim, step) = (*s, *l, *st);
                        if step == 0 {
                            return Err("'for' step is zero".to_string());
                        }
                        loop {
                            if step > 0 && i > lim {
                                break;
                            }
                            if step < 0 && i < lim {
                                break;
                            }
                            let child = Env::new_child(Rc::clone(env));
                            child.borrow_mut().define_local(var, LuaValue::Int(i));
                            match self.exec_block(body, &child)? {
                                Signal::Break => break,
                                Signal::Return(vals) => return Ok(Signal::Return(vals)),
                                Signal::None => {}
                            }
                            i = i.wrapping_add(step);
                        }
                    }
                    _ => {
                        let mut i = to_float(&start_v).ok_or("'for' start not a number")?;
                        let lim = to_float(&limit_v).ok_or("'for' limit not a number")?;
                        let step = to_float(&step_v).ok_or("'for' step not a number")?;
                        if step == 0.0 {
                            return Err("'for' step is zero".to_string());
                        }
                        loop {
                            if step > 0.0 && i > lim {
                                break;
                            }
                            if step < 0.0 && i < lim {
                                break;
                            }
                            let child = Env::new_child(Rc::clone(env));
                            child.borrow_mut().define_local(var, LuaValue::Float(i));
                            match self.exec_block(body, &child)? {
                                Signal::Break => break,
                                Signal::Return(vals) => return Ok(Signal::Return(vals)),
                                Signal::None => {}
                            }
                            i += step;
                        }
                    }
                }
                Ok(Signal::None)
            }

            Stmt::GenFor {
                vars,
                iterators,
                body,
            } => {
                let iter_vals = self.eval_expr_list(iterators, env, 3)?;
                let iter_func = iter_vals.first().cloned().unwrap_or(LuaValue::Nil);
                let state = iter_vals.get(1).cloned().unwrap_or(LuaValue::Nil);
                let mut control = iter_vals.get(2).cloned().unwrap_or(LuaValue::Nil);
                loop {
                    let results =
                        self.call_function(&iter_func, vec![state.clone(), control.clone()])?;
                    if results.is_empty() || matches!(results[0], LuaValue::Nil) {
                        break;
                    }
                    control = results[0].clone();
                    let child = Env::new_child(Rc::clone(env));
                    for (i, var_name) in vars.iter().enumerate() {
                        let v = results.get(i).cloned().unwrap_or(LuaValue::Nil);
                        child.borrow_mut().define_local(var_name, v);
                    }
                    match self.exec_block(body, &child)? {
                        Signal::Break => break,
                        Signal::Return(vals) => return Ok(Signal::Return(vals)),
                        Signal::None => {}
                    }
                }
                Ok(Signal::None)
            }

            Stmt::FuncDef {
                name,
                method,
                params,
                body,
            } => {
                let func_val =
                    LuaValue::Function(Rc::new(LuaFunction::Closure(super::LuaClosure {
                        params: params.clone(),
                        body: body.clone(),
                        env: Rc::clone(env),
                    })));
                if name.len() == 1 && method.is_none() {
                    env.borrow_mut().set(&name[0], func_val);
                } else if name.len() == 1 {
                    let tbl_val = env.borrow().get(&name[0]);
                    if let Some(LuaValue::Table(rc)) = tbl_val {
                        let key = TableKey::Str(method.as_deref().unwrap_or("").to_string());
                        rc.borrow_mut().insert(key, func_val);
                    } else {
                        return Err(format!("cannot assign method to non-table '{}'", name[0]));
                    }
                } else {
                    let mut tbl_val = env
                        .borrow()
                        .get(&name[0])
                        .ok_or_else(|| format!("undefined variable '{}'", name[0]))?;
                    for part in &name[1..name.len() - 1] {
                        tbl_val = match tbl_val {
                            LuaValue::Table(rc) => {
                                let key = TableKey::Str(part.clone());
                                rc.borrow().get(&key).cloned().unwrap_or(LuaValue::Nil)
                            }
                            _ => return Err(format!("cannot index non-table for '{}'", part)),
                        };
                    }
                    let last_key = if let Some(m) = method {
                        m.clone()
                    } else {
                        name.last().cloned().unwrap_or_default()
                    };
                    if let LuaValue::Table(rc) = tbl_val {
                        rc.borrow_mut().insert(TableKey::Str(last_key), func_val);
                    } else {
                        return Err("cannot assign function to non-table field".to_string());
                    }
                }
                Ok(Signal::None)
            }

            Stmt::LocalFunc { name, params, body } => {
                env.borrow_mut().define_local(name, LuaValue::Nil);
                let func_val =
                    LuaValue::Function(Rc::new(LuaFunction::Closure(super::LuaClosure {
                        params: params.clone(),
                        body: body.clone(),
                        env: Rc::clone(env),
                    })));
                env.borrow_mut().define_local(name, func_val);
                Ok(Signal::None)
            }

            Stmt::Return(exprs) => {
                let vals = self.eval_expr_list(exprs, env, 0)?;
                Ok(Signal::Return(vals))
            }

            Stmt::Break => Ok(Signal::Break),

            Stmt::Expr(expr) => {
                self.eval_expr(expr, env)?;
                Ok(Signal::None)
            }
        }
    }

    fn assign_target(
        &mut self,
        target: &Expr,
        val: LuaValue,
        env: &Rc<RefCell<Env>>,
    ) -> Result<(), String> {
        match target {
            Expr::Var(name) => {
                env.borrow_mut().set(name, val);
                Ok(())
            }
            Expr::FieldAccess { table, field } => {
                let tbl = self.eval_expr(table, env)?;
                match tbl {
                    LuaValue::Table(rc) => {
                        rc.borrow_mut().insert(TableKey::Str(field.clone()), val);
                        Ok(())
                    }
                    _ => Err(format!("attempt to index non-table for field '{}'", field)),
                }
            }
            Expr::IndexAccess { table, key } => {
                let tbl = self.eval_expr(table, env)?;
                let k = self.eval_expr(key, env)?;
                match tbl {
                    LuaValue::Table(rc) => {
                        let tk = TableKey::from_value(&k).ok_or("table key cannot be nil")?;
                        rc.borrow_mut().insert(tk, val);
                        Ok(())
                    }
                    _ => Err("attempt to index non-table".to_string()),
                }
            }
            _ => Err(format!("invalid assignment target: {:?}", target)),
        }
    }

    /// Evaluate a list of expressions, expanding multi-returns from the last call.
    pub(super) fn eval_expr_list(
        &mut self,
        exprs: &[Expr],
        env: &Rc<RefCell<Env>>,
        _min: usize,
    ) -> Result<Vec<LuaValue>, String> {
        if exprs.is_empty() {
            return Ok(Vec::new());
        }
        let mut result = Vec::new();
        // Evaluate all but last normally (single value)
        for expr in &exprs[..exprs.len() - 1] {
            result.push(self.eval_expr(expr, env)?);
        }
        // Expand multi-return from last expression (function call)
        let last = &exprs[exprs.len() - 1];
        let extra = self.eval_expr_multi(last, env)?;
        result.extend(extra);
        Ok(result)
    }

    /// Evaluate an expression, returning all values for function calls.
    fn eval_expr_multi(
        &mut self,
        expr: &Expr,
        env: &Rc<RefCell<Env>>,
    ) -> Result<Vec<LuaValue>, String> {
        match expr {
            Expr::FuncCall { func, args } => {
                let func_val = self.eval_expr(func, env)?;
                let arg_vals: Vec<LuaValue> = args
                    .iter()
                    .map(|a| self.eval_expr(a, env))
                    .collect::<Result<Vec<_>, _>>()?;
                self.call_function(&func_val, arg_vals)
            }
            Expr::MethodCall { obj, method, args } => {
                let obj_val = self.eval_expr(obj, env)?;
                let func_val = match &obj_val {
                    LuaValue::Table(rc) => rc
                        .borrow()
                        .get(&TableKey::Str(method.clone()))
                        .cloned()
                        .unwrap_or(LuaValue::Nil),
                    _ => {
                        return Err(format!(
                            "attempt to index a {} for method '{}'",
                            lua_value_type_name(&obj_val),
                            method
                        ))
                    }
                };
                let mut arg_vals = vec![obj_val];
                for a in args {
                    arg_vals.push(self.eval_expr(a, env)?);
                }
                self.call_function(&func_val, arg_vals)
            }
            _ => Ok(vec![self.eval_expr(expr, env)?]),
        }
    }

    pub(super) fn eval_expr(
        &mut self,
        expr: &Expr,
        env: &Rc<RefCell<Env>>,
    ) -> Result<LuaValue, String> {
        match expr {
            Expr::Nil => Ok(LuaValue::Nil),
            Expr::True => Ok(LuaValue::Bool(true)),
            Expr::False => Ok(LuaValue::Bool(false)),
            Expr::Number(n) => {
                if n.fract() == 0.0 && n.abs() < 9.007_199_254_740_992e15 {
                    Ok(LuaValue::Int(*n as i64))
                } else {
                    Ok(LuaValue::Float(*n))
                }
            }
            Expr::Str(s) => Ok(LuaValue::Str(s.clone())),
            Expr::Var(name) => Ok(env.borrow().get(name).unwrap_or(LuaValue::Nil)),
            Expr::FuncDef { params, body } => Ok(LuaValue::Function(Rc::new(
                LuaFunction::Closure(super::LuaClosure {
                    params: params.clone(),
                    body: body.clone(),
                    env: Rc::clone(env),
                }),
            ))),
            Expr::TableCtor(fields) => {
                let map: HashMap<TableKey, LuaValue> = HashMap::new();
                let tbl = Rc::new(RefCell::new(map));
                let mut seq_idx: i64 = 1;
                for field in fields {
                    match field {
                        TableField::Indexed { key, val } => {
                            let k = self.eval_expr(key, env)?;
                            let v = self.eval_expr(val, env)?;
                            let tk = TableKey::from_value(&k).ok_or("table key cannot be nil")?;
                            tbl.borrow_mut().insert(tk, v);
                        }
                        TableField::Named { name, val } => {
                            let v = self.eval_expr(val, env)?;
                            tbl.borrow_mut().insert(TableKey::Str(name.clone()), v);
                        }
                        TableField::Positional(val) => {
                            let v = self.eval_expr(val, env)?;
                            tbl.borrow_mut().insert(TableKey::Int(seq_idx), v);
                            seq_idx += 1;
                        }
                    }
                }
                Ok(LuaValue::Table(tbl))
            }
            Expr::FieldAccess { table, field } => {
                let tbl = self.eval_expr(table, env)?;
                match &tbl {
                    LuaValue::Table(rc) => Ok(rc
                        .borrow()
                        .get(&TableKey::Str(field.clone()))
                        .cloned()
                        .unwrap_or(LuaValue::Nil)),
                    _ => Err(format!(
                        "attempt to index '{}' (a {})",
                        field,
                        lua_value_type_name(&tbl)
                    )),
                }
            }
            Expr::IndexAccess { table, key } => {
                let tbl = self.eval_expr(table, env)?;
                let k = self.eval_expr(key, env)?;
                match &tbl {
                    LuaValue::Table(rc) => {
                        let tk = TableKey::from_value(&k).ok_or("nil index")?;
                        Ok(rc.borrow().get(&tk).cloned().unwrap_or(LuaValue::Nil))
                    }
                    _ => Err(format!("attempt to index a {}", lua_value_type_name(&tbl))),
                }
            }
            Expr::FuncCall { func, args } => {
                let func_val = self.eval_expr(func, env)?;
                let arg_vals: Vec<LuaValue> = args
                    .iter()
                    .map(|a| self.eval_expr(a, env))
                    .collect::<Result<Vec<_>, _>>()?;
                let results = self.call_function(&func_val, arg_vals)?;
                Ok(results.into_iter().next().unwrap_or(LuaValue::Nil))
            }
            Expr::MethodCall { obj, method, args } => {
                let obj_val = self.eval_expr(obj, env)?;
                let func_val = match &obj_val {
                    LuaValue::Table(rc) => rc
                        .borrow()
                        .get(&TableKey::Str(method.clone()))
                        .cloned()
                        .unwrap_or(LuaValue::Nil),
                    _ => {
                        return Err(format!(
                            "attempt to index a {} for method '{}'",
                            lua_value_type_name(&obj_val),
                            method
                        ))
                    }
                };
                let mut arg_vals = vec![obj_val];
                for a in args {
                    arg_vals.push(self.eval_expr(a, env)?);
                }
                let results = self.call_function(&func_val, arg_vals)?;
                Ok(results.into_iter().next().unwrap_or(LuaValue::Nil))
            }
            Expr::UnOp { op, operand } => {
                let v = self.eval_expr(operand, env)?;
                match op {
                    UnOp::Neg => match v {
                        LuaValue::Int(i) => Ok(LuaValue::Int(-i)),
                        LuaValue::Float(f) => Ok(LuaValue::Float(-f)),
                        _ => Err(format!("attempt to negate a {}", lua_value_type_name(&v))),
                    },
                    UnOp::Not => Ok(LuaValue::Bool(!lua_truthy(&v))),
                    UnOp::Len => match v {
                        LuaValue::Str(s) => Ok(LuaValue::Int(s.len() as i64)),
                        LuaValue::Table(rc) => {
                            let map = rc.borrow();
                            let mut len: i64 = 0;
                            while map.contains_key(&TableKey::Int(len + 1)) {
                                len += 1;
                            }
                            Ok(LuaValue::Int(len))
                        }
                        _ => Err(format!(
                            "attempt to get length of a {}",
                            lua_value_type_name(&v)
                        )),
                    },
                }
            }
            Expr::BinOp { op, left, right } => {
                match op {
                    BinOp::And => {
                        let l = self.eval_expr(left, env)?;
                        if !lua_truthy(&l) {
                            return Ok(l);
                        }
                        return self.eval_expr(right, env);
                    }
                    BinOp::Or => {
                        let l = self.eval_expr(left, env)?;
                        if lua_truthy(&l) {
                            return Ok(l);
                        }
                        return self.eval_expr(right, env);
                    }
                    _ => {}
                }
                let l = self.eval_expr(left, env)?;
                let r = self.eval_expr(right, env)?;
                eval_binop(*op, l, r)
            }
        }
    }

    pub(super) fn call_function(
        &mut self,
        func: &LuaValue,
        args: Vec<LuaValue>,
    ) -> Result<Vec<LuaValue>, String> {
        self.call_depth += 1;
        if self.call_depth > self.max_stack_depth {
            self.call_depth -= 1;
            return Err(format!(
                "stack overflow (max depth {})",
                self.max_stack_depth
            ));
        }
        let result = self.call_function_inner(func, args);
        self.call_depth -= 1;
        result
    }

    fn call_function_inner(
        &mut self,
        func: &LuaValue,
        args: Vec<LuaValue>,
    ) -> Result<Vec<LuaValue>, String> {
        match func {
            LuaValue::Function(rc) => match rc.as_ref() {
                LuaFunction::Closure(closure) => {
                    let call_env = Env::new_child(Rc::clone(&closure.env));
                    for (i, param) in closure.params.iter().enumerate() {
                        let v = args.get(i).cloned().unwrap_or(LuaValue::Nil);
                        call_env.borrow_mut().define_local(param, v);
                    }
                    match self.exec_block(&closure.body, &call_env)? {
                        Signal::Return(vals) => Ok(vals),
                        _ => Ok(Vec::new()),
                    }
                }
                LuaFunction::Builtin(name) => self.call_builtin(name, args),
            },
            _ => Err(format!(
                "attempt to call a {} value",
                lua_value_type_name(func)
            )),
        }
    }

    fn call_builtin(&mut self, name: &str, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, String> {
        match name {
            "print" => {
                let parts: Vec<String> = args.iter().map(lua_tostring).collect();
                let line = parts.join("\t") + "\n";
                self.print_buf.borrow_mut().push(line);
                Ok(Vec::new())
            }
            "tostring" => {
                let v = args.into_iter().next().unwrap_or(LuaValue::Nil);
                Ok(vec![LuaValue::Str(lua_tostring(&v))])
            }
            "tonumber" => {
                let v = args.into_iter().next().unwrap_or(LuaValue::Nil);
                match v {
                    LuaValue::Int(i) => Ok(vec![LuaValue::Int(i)]),
                    LuaValue::Float(f) => Ok(vec![LuaValue::Float(f)]),
                    LuaValue::Str(s) => {
                        if let Ok(i) = s.trim().parse::<i64>() {
                            Ok(vec![LuaValue::Int(i)])
                        } else if let Ok(f) = s.trim().parse::<f64>() {
                            Ok(vec![LuaValue::Float(f)])
                        } else {
                            Ok(vec![LuaValue::Nil])
                        }
                    }
                    _ => Ok(vec![LuaValue::Nil]),
                }
            }
            "type" => {
                let v = args.into_iter().next().unwrap_or(LuaValue::Nil);
                Ok(vec![LuaValue::Str(lua_value_type_name(&v).to_string())])
            }
            "error" => {
                let msg = args.into_iter().next().unwrap_or(LuaValue::Nil);
                Err(lua_tostring(&msg))
            }
            "assert" => {
                let v = args.first().cloned().unwrap_or(LuaValue::Nil);
                if lua_truthy(&v) {
                    Ok(args)
                } else {
                    let msg = args
                        .get(1)
                        .map(lua_tostring)
                        .unwrap_or_else(|| "assertion failed!".to_string());
                    Err(msg)
                }
            }
            "ipairs" => {
                let tbl = args.into_iter().next().unwrap_or(LuaValue::Nil);
                match tbl {
                    LuaValue::Table(rc) => {
                        let iter_func = LuaValue::Function(Rc::new(LuaFunction::Builtin(
                            "ipairs_next".to_string(),
                        )));
                        Ok(vec![iter_func, LuaValue::Table(rc), LuaValue::Int(0)])
                    }
                    _ => Err("ipairs expects table".to_string()),
                }
            }
            "ipairs_next" => {
                let tbl = args.first().cloned().unwrap_or(LuaValue::Nil);
                let idx = match args.get(1) {
                    Some(LuaValue::Int(i)) => *i,
                    _ => return Ok(vec![LuaValue::Nil]),
                };
                let next_idx = idx + 1;
                match tbl {
                    LuaValue::Table(rc) => {
                        let v = rc.borrow().get(&TableKey::Int(next_idx)).cloned();
                        match v {
                            Some(val) if !matches!(val, LuaValue::Nil) => {
                                Ok(vec![LuaValue::Int(next_idx), val])
                            }
                            _ => Ok(vec![LuaValue::Nil]),
                        }
                    }
                    _ => Ok(vec![LuaValue::Nil]),
                }
            }
            "pairs" => {
                let tbl = args.into_iter().next().unwrap_or(LuaValue::Nil);
                match tbl {
                    LuaValue::Table(rc) => {
                        let next_func =
                            LuaValue::Function(Rc::new(LuaFunction::Builtin("next".to_string())));
                        Ok(vec![next_func, LuaValue::Table(rc), LuaValue::Nil])
                    }
                    _ => Err("pairs expects table".to_string()),
                }
            }
            "next" => {
                let tbl = args.first().cloned().unwrap_or(LuaValue::Nil);
                let key = args.get(1).cloned().unwrap_or(LuaValue::Nil);
                match tbl {
                    LuaValue::Table(rc) => {
                        let map = rc.borrow();
                        let keys: Vec<&TableKey> = map.keys().collect();
                        if matches!(key, LuaValue::Nil) {
                            if let Some(k) = keys.first() {
                                let v = map.get(k).cloned().unwrap_or(LuaValue::Nil);
                                let kv = tablekey_to_value(k);
                                return Ok(vec![kv, v]);
                            }
                            return Ok(vec![LuaValue::Nil]);
                        }
                        let tk = match TableKey::from_value(&key) {
                            Some(k) => k,
                            None => return Ok(vec![LuaValue::Nil]),
                        };
                        let mut found = false;
                        for k in &keys {
                            if found {
                                let v = map.get(k).cloned().unwrap_or(LuaValue::Nil);
                                return Ok(vec![tablekey_to_value(k), v]);
                            }
                            if **k == tk {
                                found = true;
                            }
                        }
                        Ok(vec![LuaValue::Nil])
                    }
                    _ => Ok(vec![LuaValue::Nil]),
                }
            }
            "math.floor" => {
                let v = args.into_iter().next().unwrap_or(LuaValue::Nil);
                match v {
                    LuaValue::Int(i) => Ok(vec![LuaValue::Int(i)]),
                    LuaValue::Float(f) => Ok(vec![LuaValue::Int(f.floor() as i64)]),
                    _ => Err("math.floor expects number".to_string()),
                }
            }
            "math.ceil" => {
                let v = args.into_iter().next().unwrap_or(LuaValue::Nil);
                match v {
                    LuaValue::Int(i) => Ok(vec![LuaValue::Int(i)]),
                    LuaValue::Float(f) => Ok(vec![LuaValue::Int(f.ceil() as i64)]),
                    _ => Err("math.ceil expects number".to_string()),
                }
            }
            "math.sqrt" => {
                let v = args.into_iter().next().unwrap_or(LuaValue::Nil);
                let f = to_float(&v).ok_or("math.sqrt expects number")?;
                Ok(vec![LuaValue::Float(f.sqrt())])
            }
            "math.abs" => {
                let v = args.into_iter().next().unwrap_or(LuaValue::Nil);
                match v {
                    LuaValue::Int(i) => Ok(vec![LuaValue::Int(i.abs())]),
                    LuaValue::Float(f) => Ok(vec![LuaValue::Float(f.abs())]),
                    _ => Err("math.abs expects number".to_string()),
                }
            }
            "math.max" => {
                if args.is_empty() {
                    return Err("math.max requires arguments".to_string());
                }
                let mut result = args[0].clone();
                for v in &args[1..] {
                    let a = to_float(&result).ok_or("math.max expects numbers")?;
                    let b = to_float(v).ok_or("math.max expects numbers")?;
                    if b > a {
                        result = v.clone();
                    }
                }
                Ok(vec![result])
            }
            "math.min" => {
                if args.is_empty() {
                    return Err("math.min requires arguments".to_string());
                }
                let mut result = args[0].clone();
                for v in &args[1..] {
                    let a = to_float(&result).ok_or("math.min expects numbers")?;
                    let b = to_float(v).ok_or("math.min expects numbers")?;
                    if b < a {
                        result = v.clone();
                    }
                }
                Ok(vec![result])
            }
            "string.len" => {
                let v = args.into_iter().next().unwrap_or(LuaValue::Nil);
                match v {
                    LuaValue::Str(s) => Ok(vec![LuaValue::Int(s.len() as i64)]),
                    _ => Err("string.len expects string".to_string()),
                }
            }
            "string.sub" => {
                let s = match args.first() {
                    Some(LuaValue::Str(s)) => s.clone(),
                    _ => return Err("string.sub expects string".to_string()),
                };
                let len = s.len() as i64;
                let i = match args.get(1) {
                    Some(LuaValue::Int(i)) => *i,
                    Some(LuaValue::Float(f)) => *f as i64,
                    _ => return Err("string.sub expects integer start".to_string()),
                };
                let j = match args.get(2) {
                    Some(LuaValue::Int(j)) => *j,
                    Some(LuaValue::Float(f)) => *f as i64,
                    None => -1,
                    _ => return Err("string.sub expects integer end".to_string()),
                };
                let start = lua_str_index(i, len);
                let end = lua_str_index(j, len) + 1;
                if start >= end || start >= s.len() {
                    return Ok(vec![LuaValue::Str(String::new())]);
                }
                let end = end.min(s.len());
                Ok(vec![LuaValue::Str(s[start..end].to_string())])
            }
            "string.find" => {
                let s = match args.first() {
                    Some(LuaValue::Str(s)) => s.clone(),
                    _ => return Err("string.find expects string".to_string()),
                };
                let pattern = match args.get(1) {
                    Some(LuaValue::Str(p)) => p.clone(),
                    _ => return Err("string.find expects string pattern".to_string()),
                };
                if let Some(pos) = s.find(pattern.as_str()) {
                    let start = pos as i64 + 1;
                    let end = (pos + pattern.len()) as i64;
                    Ok(vec![LuaValue::Int(start), LuaValue::Int(end)])
                } else {
                    Ok(vec![LuaValue::Nil])
                }
            }
            "string.format" => {
                let fmt = match args.first() {
                    Some(LuaValue::Str(s)) => s.clone(),
                    _ => return Err("string.format expects format string".to_string()),
                };
                let result = string_format(&fmt, &args[1..])?;
                Ok(vec![LuaValue::Str(result)])
            }
            "table.insert" => {
                let tbl = args.first().cloned().unwrap_or(LuaValue::Nil);
                match tbl {
                    LuaValue::Table(rc) => {
                        let val = args.get(1).cloned().unwrap_or(LuaValue::Nil);
                        let mut map = rc.borrow_mut();
                        let mut len: i64 = 0;
                        while map.contains_key(&TableKey::Int(len + 1)) {
                            len += 1;
                        }
                        map.insert(TableKey::Int(len + 1), val);
                        Ok(Vec::new())
                    }
                    _ => Err("table.insert expects table".to_string()),
                }
            }
            "table.concat" => {
                let tbl = match args.first() {
                    Some(LuaValue::Table(rc)) => rc.clone(),
                    _ => return Err("table.concat expects table".to_string()),
                };
                let sep = match args.get(1) {
                    Some(LuaValue::Str(s)) => s.clone(),
                    None => String::new(),
                    _ => return Err("table.concat separator must be string".to_string()),
                };
                let map = tbl.borrow();
                let mut parts = Vec::new();
                let mut idx: i64 = 1;
                while let Some(v) = map.get(&TableKey::Int(idx)) {
                    parts.push(lua_tostring(v));
                    idx += 1;
                }
                Ok(vec![LuaValue::Str(parts.join(&sep))])
            }
            other => Err(format!("call to unknown built-in '{}'", other)),
        }
    }
}

// ===== Helper functions =====

pub(super) fn lua_truthy(v: &LuaValue) -> bool {
    !matches!(v, LuaValue::Nil | LuaValue::Bool(false))
}

pub(super) fn to_float(v: &LuaValue) -> Option<f64> {
    match v {
        LuaValue::Int(i) => Some(*i as f64),
        LuaValue::Float(f) => Some(*f),
        LuaValue::Str(s) => s.trim().parse().ok(),
        _ => None,
    }
}

fn to_int_or_float(v: &LuaValue) -> Option<LuaValue> {
    match v {
        LuaValue::Int(_) | LuaValue::Float(_) => Some(v.clone()),
        LuaValue::Str(s) => {
            if let Ok(i) = s.trim().parse::<i64>() {
                Some(LuaValue::Int(i))
            } else if let Ok(f) = s.trim().parse::<f64>() {
                Some(LuaValue::Float(f))
            } else {
                None
            }
        }
        _ => None,
    }
}

pub(super) fn eval_binop(op: BinOp, l: LuaValue, r: LuaValue) -> Result<LuaValue, String> {
    match op {
        BinOp::Concat => {
            let ls = lua_tostring(&l);
            let rs = lua_tostring(&r);
            return Ok(LuaValue::Str(ls + &rs));
        }
        BinOp::Eq => return Ok(LuaValue::Bool(l == r)),
        BinOp::Ne => return Ok(LuaValue::Bool(l != r)),
        _ => {}
    }

    match op {
        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
            return eval_compare(op, l, r);
        }
        _ => {}
    }

    match (&l, &r) {
        (LuaValue::Int(a), LuaValue::Int(b)) => match op {
            BinOp::Add => Ok(LuaValue::Int(a.wrapping_add(*b))),
            BinOp::Sub => Ok(LuaValue::Int(a.wrapping_sub(*b))),
            BinOp::Mul => Ok(LuaValue::Int(a.wrapping_mul(*b))),
            BinOp::Div => {
                if *b == 0 {
                    Ok(LuaValue::Float(if *a >= 0 {
                        f64::INFINITY
                    } else {
                        f64::NEG_INFINITY
                    }))
                } else {
                    Ok(LuaValue::Float(*a as f64 / *b as f64))
                }
            }
            BinOp::Mod => {
                if *b == 0 {
                    return Err("attempt to perform 'n%0'".to_string());
                }
                let result = a.wrapping_rem(*b);
                if (result < 0) != (*b < 0) && result != 0 {
                    Ok(LuaValue::Int(result + b))
                } else {
                    Ok(LuaValue::Int(result))
                }
            }
            BinOp::Pow => Ok(LuaValue::Float((*a as f64).powf(*b as f64))),
            _ => unreachable!(),
        },
        _ => {
            let a = to_int_or_float(&l).ok_or_else(|| {
                format!(
                    "attempt to perform arithmetic on a {} value",
                    lua_value_type_name(&l)
                )
            })?;
            let b = to_int_or_float(&r).ok_or_else(|| {
                format!(
                    "attempt to perform arithmetic on a {} value",
                    lua_value_type_name(&r)
                )
            })?;
            let af = to_float(&a).unwrap_or(0.0);
            let bf = to_float(&b).unwrap_or(0.0);
            let result = match op {
                BinOp::Add => af + bf,
                BinOp::Sub => af - bf,
                BinOp::Mul => af * bf,
                BinOp::Div => af / bf,
                BinOp::Mod => af - (af / bf).floor() * bf,
                BinOp::Pow => af.powf(bf),
                _ => unreachable!(),
            };
            if result.fract() == 0.0 && result.abs() < 9.007_199_254_740_992e15 {
                Ok(LuaValue::Int(result as i64))
            } else {
                Ok(LuaValue::Float(result))
            }
        }
    }
}

fn eval_compare(op: BinOp, l: LuaValue, r: LuaValue) -> Result<LuaValue, String> {
    match (&l, &r) {
        (LuaValue::Int(a), LuaValue::Int(b)) => {
            let result = match op {
                BinOp::Lt => a < b,
                BinOp::Le => a <= b,
                BinOp::Gt => a > b,
                BinOp::Ge => a >= b,
                _ => unreachable!(),
            };
            Ok(LuaValue::Bool(result))
        }
        (LuaValue::Float(a), LuaValue::Float(b)) => {
            let result = match op {
                BinOp::Lt => a < b,
                BinOp::Le => a <= b,
                BinOp::Gt => a > b,
                BinOp::Ge => a >= b,
                _ => unreachable!(),
            };
            Ok(LuaValue::Bool(result))
        }
        (LuaValue::Int(a), LuaValue::Float(b)) => {
            let a = *a as f64;
            let result = match op {
                BinOp::Lt => a < *b,
                BinOp::Le => a <= *b,
                BinOp::Gt => a > *b,
                BinOp::Ge => a >= *b,
                _ => unreachable!(),
            };
            Ok(LuaValue::Bool(result))
        }
        (LuaValue::Float(a), LuaValue::Int(b)) => {
            let b = *b as f64;
            let result = match op {
                BinOp::Lt => *a < b,
                BinOp::Le => *a <= b,
                BinOp::Gt => *a > b,
                BinOp::Ge => *a >= b,
                _ => unreachable!(),
            };
            Ok(LuaValue::Bool(result))
        }
        (LuaValue::Str(a), LuaValue::Str(b)) => {
            let result = match op {
                BinOp::Lt => a < b,
                BinOp::Le => a <= b,
                BinOp::Gt => a > b,
                BinOp::Ge => a >= b,
                _ => unreachable!(),
            };
            Ok(LuaValue::Bool(result))
        }
        _ => Err(format!(
            "attempt to compare {} with {}",
            lua_value_type_name(&l),
            lua_value_type_name(&r)
        )),
    }
}

pub(super) fn lua_tostring(v: &LuaValue) -> String {
    match v {
        LuaValue::Nil => "nil".to_string(),
        LuaValue::Bool(b) => format!("{}", b),
        LuaValue::Int(i) => format!("{}", i),
        LuaValue::Float(f) => {
            if f.fract() == 0.0 && f.abs() < 1e15 {
                format!("{:.1}", f)
            } else {
                format!("{}", f)
            }
        }
        LuaValue::Str(s) => s.clone(),
        LuaValue::Table(rc) => format!("table: {:p}", Rc::as_ptr(rc)),
        LuaValue::Function(rc) => format!("function: {:p}", Rc::as_ptr(rc)),
    }
}

pub(super) fn lua_str_index(i: i64, len: i64) -> usize {
    if i >= 0 {
        ((i - 1).max(0)) as usize
    } else {
        let pos = len + i;
        pos.max(0) as usize
    }
}

pub(super) fn tablekey_to_value(k: &TableKey) -> LuaValue {
    match k {
        TableKey::Bool(b) => LuaValue::Bool(*b),
        TableKey::Int(i) => LuaValue::Int(*i),
        TableKey::Float(bits) => LuaValue::Float(f64::from_bits(*bits)),
        TableKey::Str(s) => LuaValue::Str(s.clone()),
        TableKey::TablePtr(_) | TableKey::FuncPtr(_) => LuaValue::Nil,
    }
}

/// Basic `string.format` with %d, %s, %f, %i, %g support.
pub(super) fn string_format(fmt: &str, args: &[LuaValue]) -> Result<String, String> {
    let mut result = String::new();
    let chars: Vec<char> = fmt.chars().collect();
    let mut pos = 0;
    let mut arg_idx = 0;

    while pos < chars.len() {
        if chars[pos] != '%' {
            result.push(chars[pos]);
            pos += 1;
            continue;
        }
        pos += 1;
        if pos >= chars.len() {
            return Err("format string ends with '%'".to_string());
        }
        if chars[pos] == '%' {
            result.push('%');
            pos += 1;
            continue;
        }
        while pos < chars.len() && "0-+ #".contains(chars[pos]) {
            pos += 1;
        }
        while pos < chars.len() && chars[pos].is_ascii_digit() {
            pos += 1;
        }
        if pos < chars.len() && chars[pos] == '.' {
            pos += 1;
            while pos < chars.len() && chars[pos].is_ascii_digit() {
                pos += 1;
            }
        }
        if pos >= chars.len() {
            return Err("truncated format specifier".to_string());
        }
        let spec = chars[pos];
        pos += 1;
        let arg = args.get(arg_idx).cloned().unwrap_or(LuaValue::Nil);
        arg_idx += 1;
        match spec {
            'd' | 'i' => {
                let i = match arg {
                    LuaValue::Int(i) => i,
                    LuaValue::Float(f) => f as i64,
                    _ => {
                        return Err(format!(
                            "bad argument #{} to format (number expected)",
                            arg_idx
                        ))
                    }
                };
                result.push_str(&format!("{}", i));
            }
            's' => {
                result.push_str(&lua_tostring(&arg));
            }
            'f' => {
                let f = to_float(&arg).ok_or_else(|| {
                    format!("bad argument #{} to format (number expected)", arg_idx)
                })?;
                result.push_str(&format!("{:.6}", f));
            }
            'g' => {
                let f = to_float(&arg).ok_or_else(|| {
                    format!("bad argument #{} to format (number expected)", arg_idx)
                })?;
                result.push_str(&format!("{}", f));
            }
            'q' => {
                let s = lua_tostring(&arg);
                result.push('"');
                for c in s.chars() {
                    match c {
                        '"' => result.push_str("\\\""),
                        '\\' => result.push_str("\\\\"),
                        '\n' => result.push_str("\\n"),
                        c => result.push(c),
                    }
                }
                result.push('"');
            }
            other => return Err(format!("unsupported format specifier '%{}'", other)),
        }
    }
    Ok(result)
}

// ===== Global Environment Builder =====

pub(super) fn build_global_env(
    user_globals: &HashMap<String, LuaValue>,
    _print_buf: Rc<RefCell<Vec<String>>>,
) -> Rc<RefCell<Env>> {
    let env = Rc::new(RefCell::new(Env {
        vars: HashMap::new(),
        parent: None,
    }));

    let builtins = [
        "print",
        "tostring",
        "tonumber",
        "type",
        "error",
        "assert",
        "ipairs",
        "ipairs_next",
        "pairs",
        "next",
    ];
    for name in &builtins {
        let func = LuaValue::Function(Rc::new(LuaFunction::Builtin(name.to_string())));
        env.borrow_mut().vars.insert(name.to_string(), func);
    }

    // math table
    let math_tbl: HashMap<TableKey, LuaValue> = [
        ("floor", "math.floor"),
        ("ceil", "math.ceil"),
        ("sqrt", "math.sqrt"),
        ("abs", "math.abs"),
        ("max", "math.max"),
        ("min", "math.min"),
    ]
    .iter()
    .map(|(k, v)| {
        let func = LuaValue::Function(Rc::new(LuaFunction::Builtin(v.to_string())));
        (TableKey::Str(k.to_string()), func)
    })
    .chain([
        (
            TableKey::Str("pi".to_string()),
            LuaValue::Float(std::f64::consts::PI),
        ),
        (
            TableKey::Str("huge".to_string()),
            LuaValue::Float(f64::INFINITY),
        ),
    ])
    .collect();
    env.borrow_mut().vars.insert(
        "math".to_string(),
        LuaValue::Table(Rc::new(RefCell::new(math_tbl))),
    );

    // string table
    let string_tbl: HashMap<TableKey, LuaValue> = [
        ("len", "string.len"),
        ("sub", "string.sub"),
        ("find", "string.find"),
        ("format", "string.format"),
    ]
    .iter()
    .map(|(k, v)| {
        let func = LuaValue::Function(Rc::new(LuaFunction::Builtin(v.to_string())));
        (TableKey::Str(k.to_string()), func)
    })
    .collect();
    env.borrow_mut().vars.insert(
        "string".to_string(),
        LuaValue::Table(Rc::new(RefCell::new(string_tbl))),
    );

    // table module
    let table_tbl: HashMap<TableKey, LuaValue> =
        [("insert", "table.insert"), ("concat", "table.concat")]
            .iter()
            .map(|(k, v)| {
                let func = LuaValue::Function(Rc::new(LuaFunction::Builtin(v.to_string())));
                (TableKey::Str(k.to_string()), func)
            })
            .collect();
    env.borrow_mut().vars.insert(
        "table".to_string(),
        LuaValue::Table(Rc::new(RefCell::new(table_tbl))),
    );

    // Inject user globals
    for (name, val) in user_globals {
        env.borrow_mut().vars.insert(name.clone(), val.clone());
    }

    env
}
