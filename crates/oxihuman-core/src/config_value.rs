#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigType {
    Int,
    Float,
    Bool,
    Str,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum CfgValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
}

#[allow(dead_code)]
pub fn new_config_int(v: i64) -> CfgValue {
    CfgValue::Int(v)
}

#[allow(dead_code)]
pub fn new_config_float(v: f64) -> CfgValue {
    CfgValue::Float(v)
}

#[allow(dead_code)]
pub fn new_config_bool(v: bool) -> CfgValue {
    CfgValue::Bool(v)
}

#[allow(dead_code)]
pub fn new_config_string(v: &str) -> CfgValue {
    CfgValue::Str(v.to_string())
}

#[allow(dead_code)]
pub fn config_type(v: &CfgValue) -> ConfigType {
    match v {
        CfgValue::Int(_) => ConfigType::Int,
        CfgValue::Float(_) => ConfigType::Float,
        CfgValue::Bool(_) => ConfigType::Bool,
        CfgValue::Str(_) => ConfigType::Str,
    }
}

#[allow(dead_code)]
pub fn config_as_int(v: &CfgValue) -> Option<i64> {
    match v {
        CfgValue::Int(i) => Some(*i),
        _ => None,
    }
}

#[allow(dead_code)]
pub fn config_as_float(v: &CfgValue) -> Option<f64> {
    match v {
        CfgValue::Float(f) => Some(*f),
        _ => None,
    }
}

#[allow(dead_code)]
pub fn config_to_json(v: &CfgValue) -> String {
    match v {
        CfgValue::Int(i) => format!("{{\"type\":\"int\",\"value\":{i}}}"),
        CfgValue::Float(f) => format!("{{\"type\":\"float\",\"value\":{f}}}"),
        CfgValue::Bool(b) => format!("{{\"type\":\"bool\",\"value\":{b}}}"),
        CfgValue::Str(s) => format!("{{\"type\":\"string\",\"value\":\"{s}\"}}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int() {
        let v = new_config_int(42);
        assert_eq!(config_type(&v), ConfigType::Int);
        assert_eq!(config_as_int(&v), Some(42));
    }

    #[test]
    fn test_float() {
        let v = new_config_float(2.75);
        assert_eq!(config_type(&v), ConfigType::Float);
        assert_eq!(config_as_float(&v), Some(2.75));
    }

    #[test]
    fn test_bool() {
        let v = new_config_bool(true);
        assert_eq!(config_type(&v), ConfigType::Bool);
    }

    #[test]
    fn test_string() {
        let v = new_config_string("hello");
        assert_eq!(config_type(&v), ConfigType::Str);
    }

    #[test]
    fn test_int_not_float() {
        let v = new_config_int(1);
        assert_eq!(config_as_float(&v), None);
    }

    #[test]
    fn test_float_not_int() {
        let v = new_config_float(1.0);
        assert_eq!(config_as_int(&v), None);
    }

    #[test]
    fn test_to_json_int() {
        let v = new_config_int(99);
        let j = config_to_json(&v);
        assert!(j.contains("\"type\":\"int\""));
        assert!(j.contains("99"));
    }

    #[test]
    fn test_to_json_float() {
        let v = new_config_float(2.5);
        let j = config_to_json(&v);
        assert!(j.contains("\"type\":\"float\""));
    }

    #[test]
    fn test_to_json_bool() {
        let v = new_config_bool(false);
        let j = config_to_json(&v);
        assert!(j.contains("\"type\":\"bool\""));
        assert!(j.contains("false"));
    }

    #[test]
    fn test_to_json_string() {
        let v = new_config_string("test");
        let j = config_to_json(&v);
        assert!(j.contains("\"type\":\"string\""));
        assert!(j.contains("test"));
    }
}
