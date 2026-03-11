//! WASM host/guest interop bridge stub for cross-boundary data passing.

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

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WasmFunction {
    pub name: String,
    pub param_types: Vec<WasmValueType>,
    pub return_types: Vec<WasmValueType>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WasmBridge {
    pub config: WasmBridgeConfig,
    pub memory: WasmMemory,
    pub functions: Vec<WasmFunction>,
    pub call_count: u64,
}

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
    let page_size = 65536usize;
    let size_bytes = cfg.memory_pages as usize * page_size;
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
    let bytes = [
        bridge.memory.data[offset],
        bridge.memory.data[offset + 1],
        bridge.memory.data[offset + 2],
        bridge.memory.data[offset + 3],
    ];
    Some(u32::from_le_bytes(bytes))
}

#[allow(dead_code)]
pub fn wasm_write_u32(bridge: &mut WasmBridge, offset: usize, val: u32) -> bool {
    if offset + 4 > bridge.memory.data.len() {
        return false;
    }
    let bytes = val.to_le_bytes();
    bridge.memory.data[offset] = bytes[0];
    bridge.memory.data[offset + 1] = bytes[1];
    bridge.memory.data[offset + 2] = bytes[2];
    bridge.memory.data[offset + 3] = bytes[3];
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

#[allow(dead_code)]
pub fn wasm_call_stub(bridge: &mut WasmBridge, name: &str) -> bool {
    let found = bridge.functions.iter().any(|f| f.name == name);
    if found {
        bridge.call_count += 1;
    }
    found
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_wasm_bridge_config();
        assert_eq!(cfg.memory_pages, 1);
        assert_eq!(cfg.max_functions, 256);
        assert!(!cfg.debug_mode);
    }

    #[test]
    fn test_new_bridge_memory_size() {
        let cfg = default_wasm_bridge_config();
        let bridge = new_wasm_bridge(cfg);
        assert_eq!(wasm_memory_size(&bridge), 65536);
    }

    #[test]
    fn test_write_read_u32() {
        let cfg = default_wasm_bridge_config();
        let mut bridge = new_wasm_bridge(cfg);
        assert!(wasm_write_u32(&mut bridge, 0, 0xDEAD_BEEF));
        let v = wasm_read_u32(&bridge, 0);
        assert_eq!(v, Some(0xDEAD_BEEF));
    }

    #[test]
    fn test_out_of_bounds_returns_none() {
        let cfg = default_wasm_bridge_config();
        let bridge = new_wasm_bridge(cfg);
        let v = wasm_read_u32(&bridge, 65534);
        assert!(v.is_none());
    }

    #[test]
    fn test_register_and_call_function() {
        let cfg = default_wasm_bridge_config();
        let mut bridge = new_wasm_bridge(cfg);
        let func = WasmFunction {
            name: "add".to_string(),
            param_types: vec![WasmValueType::I32, WasmValueType::I32],
            return_types: vec![WasmValueType::I32],
        };
        register_wasm_function(&mut bridge, func);
        assert_eq!(wasm_function_count(&bridge), 1);
        assert!(wasm_call_stub(&mut bridge, "add"));
        assert_eq!(bridge.call_count, 1);
    }

    #[test]
    fn test_call_unknown_function() {
        let cfg = default_wasm_bridge_config();
        let mut bridge = new_wasm_bridge(cfg);
        let result = wasm_call_stub(&mut bridge, "nonexistent");
        assert!(!result);
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
        let cfg = default_wasm_bridge_config();
        let bridge = new_wasm_bridge(cfg);
        let json = wasm_bridge_to_json(&bridge);
        assert!(json.contains("memory_pages"));
        assert!(json.contains("call_count"));
    }
}
