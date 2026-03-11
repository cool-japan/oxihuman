//! Minimal Lua scripting interface stub for plugin and automation support.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum LuaValue {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LuaConfig {
    pub max_stack_depth: usize,
    pub timeout_ms: u64,
    pub sandbox: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LuaScript {
    pub source: String,
    pub name: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LuaResult {
    pub return_values: Vec<LuaValue>,
    pub error: Option<String>,
    pub executed: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LuaStub {
    pub config: LuaConfig,
    pub globals: Vec<(String, LuaValue)>,
    pub call_count: u64,
}

#[allow(dead_code)]
pub fn default_lua_config() -> LuaConfig {
    LuaConfig {
        max_stack_depth: 64,
        timeout_ms: 5000,
        sandbox: true,
    }
}

#[allow(dead_code)]
pub fn new_lua_stub(cfg: LuaConfig) -> LuaStub {
    LuaStub {
        config: cfg,
        globals: Vec::new(),
        call_count: 0,
    }
}

#[allow(dead_code)]
pub fn lua_set_global(stub: &mut LuaStub, name: &str, val: LuaValue) {
    if let Some(entry) = stub.globals.iter_mut().find(|(k, _)| k == name) {
        entry.1 = val;
    } else {
        stub.globals.push((name.to_string(), val));
    }
}

#[allow(dead_code)]
pub fn lua_get_global<'a>(stub: &'a LuaStub, name: &str) -> Option<&'a LuaValue> {
    stub.globals.iter().find(|(k, _)| k == name).map(|(_, v)| v)
}

#[allow(dead_code)]
pub fn lua_execute(stub: &mut LuaStub, _script: &LuaScript) -> LuaResult {
    stub.call_count += 1;
    LuaResult {
        return_values: vec![LuaValue::Nil],
        error: None,
        executed: true,
    }
}

#[allow(dead_code)]
pub fn lua_value_type_name(v: &LuaValue) -> &'static str {
    match v {
        LuaValue::Nil => "nil",
        LuaValue::Bool(_) => "boolean",
        LuaValue::Int(_) => "integer",
        LuaValue::Float(_) => "float",
        LuaValue::Str(_) => "string",
    }
}

#[allow(dead_code)]
pub fn lua_value_to_json(v: &LuaValue) -> String {
    match v {
        LuaValue::Nil => "null".to_string(),
        LuaValue::Bool(b) => format!("{}", b),
        LuaValue::Int(i) => format!("{}", i),
        LuaValue::Float(f) => format!("{}", f),
        LuaValue::Str(s) => format!("\"{}\"", s),
    }
}

#[allow(dead_code)]
pub fn lua_result_to_json(r: &LuaResult) -> String {
    let vals: Vec<String> = r.return_values.iter().map(lua_value_to_json).collect();
    let vals_str = vals.join(",");
    let err_str = match &r.error {
        Some(e) => format!("\"{}\"", e),
        None => "null".to_string(),
    };
    format!(
        "{{\"return_values\":[{}],\"error\":{},\"executed\":{}}}",
        vals_str, err_str, r.executed
    )
}

#[allow(dead_code)]
pub fn lua_stub_to_json(stub: &LuaStub) -> String {
    let globals_count = stub.globals.len();
    format!(
        "{{\"call_count\":{},\"globals_count\":{},\"sandbox\":{}}}",
        stub.call_count, globals_count, stub.config.sandbox
    )
}

#[allow(dead_code)]
pub fn new_lua_script(name: &str, source: &str) -> LuaScript {
    LuaScript {
        source: source.to_string(),
        name: name.to_string(),
    }
}

#[allow(dead_code)]
pub fn lua_global_count(stub: &LuaStub) -> usize {
    stub.globals.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_lua_config();
        assert_eq!(cfg.max_stack_depth, 64);
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
        assert_eq!(
            lua_value_type_name(&LuaValue::Str("".to_string())),
            "string"
        );
    }

    #[test]
    fn test_global_count_and_update() {
        let cfg = default_lua_config();
        let mut stub = new_lua_stub(cfg);
        lua_set_global(&mut stub, "a", LuaValue::Bool(false));
        lua_set_global(&mut stub, "b", LuaValue::Float(std::f64::consts::PI));
        assert_eq!(lua_global_count(&stub), 2);
        // update existing key
        lua_set_global(&mut stub, "a", LuaValue::Bool(true));
        assert_eq!(lua_global_count(&stub), 2);
    }

    #[test]
    fn test_result_to_json() {
        let r = LuaResult {
            return_values: vec![LuaValue::Nil],
            error: None,
            executed: true,
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
}
