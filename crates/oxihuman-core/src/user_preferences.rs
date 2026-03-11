//! User preferences storage with typed values.

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum PrefValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Color([f32; 4]),
    Vec2([f32; 2]),
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Preference {
    pub key: String,
    pub value: PrefValue,
    pub category: String,
    pub description: String,
}

#[allow(dead_code)]
pub struct UserPreferences {
    pub prefs: Vec<Preference>,
    pub dirty: bool,
}

#[allow(dead_code)]
pub fn new_user_preferences() -> UserPreferences {
    UserPreferences {
        prefs: Vec::new(),
        dirty: false,
    }
}

#[allow(dead_code)]
pub fn set_pref(prefs: &mut UserPreferences, key: &str, value: PrefValue, category: &str) {
    if let Some(existing) = prefs.prefs.iter_mut().find(|p| p.key == key) {
        existing.value = value;
        existing.category = category.to_string();
    } else {
        prefs.prefs.push(Preference {
            key: key.to_string(),
            value,
            category: category.to_string(),
            description: String::new(),
        });
    }
    prefs.dirty = true;
}

#[allow(dead_code)]
pub fn get_pref<'a>(prefs: &'a UserPreferences, key: &str) -> Option<&'a Preference> {
    prefs.prefs.iter().find(|p| p.key == key)
}

#[allow(dead_code)]
pub fn get_bool(prefs: &UserPreferences, key: &str, default: bool) -> bool {
    match get_pref(prefs, key) {
        Some(p) => match &p.value {
            PrefValue::Bool(v) => *v,
            _ => default,
        },
        None => default,
    }
}

#[allow(dead_code)]
pub fn get_int(prefs: &UserPreferences, key: &str, default: i64) -> i64 {
    match get_pref(prefs, key) {
        Some(p) => match &p.value {
            PrefValue::Int(v) => *v,
            _ => default,
        },
        None => default,
    }
}

#[allow(dead_code)]
pub fn get_float(prefs: &UserPreferences, key: &str, default: f64) -> f64 {
    match get_pref(prefs, key) {
        Some(p) => match &p.value {
            PrefValue::Float(v) => *v,
            _ => default,
        },
        None => default,
    }
}

#[allow(dead_code)]
pub fn get_string(prefs: &UserPreferences, key: &str, default: &str) -> String {
    match get_pref(prefs, key) {
        Some(p) => match &p.value {
            PrefValue::Str(v) => v.clone(),
            _ => default.to_string(),
        },
        None => default.to_string(),
    }
}

#[allow(dead_code)]
pub fn remove_pref(prefs: &mut UserPreferences, key: &str) -> bool {
    let before = prefs.prefs.len();
    prefs.prefs.retain(|p| p.key != key);
    let removed = prefs.prefs.len() < before;
    if removed {
        prefs.dirty = true;
    }
    removed
}

#[allow(dead_code)]
pub fn pref_count(prefs: &UserPreferences) -> usize {
    prefs.prefs.len()
}

#[allow(dead_code)]
pub fn prefs_in_category<'a>(prefs: &'a UserPreferences, cat: &str) -> Vec<&'a Preference> {
    prefs.prefs.iter().filter(|p| p.category == cat).collect()
}

#[allow(dead_code)]
pub fn mark_clean(prefs: &mut UserPreferences) {
    prefs.dirty = false;
}

#[allow(dead_code)]
pub fn reset_to_defaults(prefs: &mut UserPreferences) {
    prefs.prefs.clear();
    prefs.dirty = false;
}

#[allow(dead_code)]
pub fn preferences_to_json(prefs: &UserPreferences) -> String {
    let mut out = String::from("{\"prefs\":[");
    for (i, p) in prefs.prefs.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let val_str = match &p.value {
            PrefValue::Bool(v) => format!("{v}"),
            PrefValue::Int(v) => format!("{v}"),
            PrefValue::Float(v) => format!("{v}"),
            PrefValue::Str(v) => format!("\"{}\"", v.replace('"', "\\\"")),
            PrefValue::Color(c) => format!("[{},{},{},{}]", c[0], c[1], c[2], c[3]),
            PrefValue::Vec2(v) => format!("[{},{}]", v[0], v[1]),
        };
        out.push_str(&format!(
            "{{\"key\":\"{}\",\"value\":{},\"category\":\"{}\"}}",
            p.key.replace('"', "\\\""),
            val_str,
            p.category.replace('"', "\\\"")
        ));
    }
    out.push_str("]}");
    out
}

#[allow(dead_code)]
pub fn preferences_from_pairs(pairs: &[(String, String, String)]) -> UserPreferences {
    let mut prefs = new_user_preferences();
    for (key, value_str, category) in pairs {
        set_pref(&mut prefs, key, PrefValue::Str(value_str.clone()), category);
    }
    prefs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_user_preferences() {
        let p = new_user_preferences();
        assert_eq!(pref_count(&p), 0);
        assert!(!p.dirty);
    }

    #[test]
    fn test_set_and_get_pref() {
        let mut p = new_user_preferences();
        set_pref(&mut p, "key1", PrefValue::Bool(true), "general");
        let pref = get_pref(&p, "key1");
        assert!(pref.is_some());
        assert_eq!(pref.unwrap().value, PrefValue::Bool(true));
    }

    #[test]
    fn test_set_pref_updates_existing() {
        let mut p = new_user_preferences();
        set_pref(&mut p, "key1", PrefValue::Int(10), "general");
        set_pref(&mut p, "key1", PrefValue::Int(20), "general");
        assert_eq!(pref_count(&p), 1);
        assert_eq!(get_int(&p, "key1", 0), 20);
    }

    #[test]
    fn test_get_bool_with_default() {
        let p = new_user_preferences();
        assert!(get_bool(&p, "missing", true));
        assert!(!get_bool(&p, "missing", false));
    }

    #[test]
    fn test_get_int_with_default() {
        let p = new_user_preferences();
        assert_eq!(get_int(&p, "missing", 42), 42);
    }

    #[test]
    fn test_get_float_with_default() {
        let p = new_user_preferences();
        assert!((get_float(&p, "missing", 2.78) - 2.78).abs() < 1e-10);
    }

    #[test]
    fn test_get_string_with_default() {
        let p = new_user_preferences();
        assert_eq!(get_string(&p, "missing", "default"), "default");
    }

    #[test]
    fn test_get_bool_value() {
        let mut p = new_user_preferences();
        set_pref(&mut p, "flag", PrefValue::Bool(false), "ui");
        assert!(!get_bool(&p, "flag", true));
    }

    #[test]
    fn test_get_string_value() {
        let mut p = new_user_preferences();
        set_pref(&mut p, "theme", PrefValue::Str("dark".to_string()), "ui");
        assert_eq!(get_string(&p, "theme", "light"), "dark");
    }

    #[test]
    fn test_remove_pref() {
        let mut p = new_user_preferences();
        set_pref(&mut p, "key1", PrefValue::Int(1), "general");
        assert!(remove_pref(&mut p, "key1"));
        assert_eq!(pref_count(&p), 0);
        assert!(!remove_pref(&mut p, "key1"));
    }

    #[test]
    fn test_pref_count() {
        let mut p = new_user_preferences();
        assert_eq!(pref_count(&p), 0);
        set_pref(&mut p, "a", PrefValue::Int(1), "x");
        set_pref(&mut p, "b", PrefValue::Int(2), "x");
        assert_eq!(pref_count(&p), 2);
    }

    #[test]
    fn test_prefs_in_category() {
        let mut p = new_user_preferences();
        set_pref(&mut p, "a", PrefValue::Int(1), "ui");
        set_pref(&mut p, "b", PrefValue::Int(2), "graphics");
        set_pref(&mut p, "c", PrefValue::Int(3), "ui");
        let ui_prefs = prefs_in_category(&p, "ui");
        assert_eq!(ui_prefs.len(), 2);
    }

    #[test]
    fn test_mark_clean() {
        let mut p = new_user_preferences();
        set_pref(&mut p, "a", PrefValue::Int(1), "x");
        assert!(p.dirty);
        mark_clean(&mut p);
        assert!(!p.dirty);
    }

    #[test]
    fn test_reset_to_defaults() {
        let mut p = new_user_preferences();
        set_pref(&mut p, "a", PrefValue::Int(1), "x");
        reset_to_defaults(&mut p);
        assert_eq!(pref_count(&p), 0);
        assert!(!p.dirty);
    }

    #[test]
    fn test_preferences_to_json() {
        let mut p = new_user_preferences();
        set_pref(&mut p, "theme", PrefValue::Str("dark".to_string()), "ui");
        let json = preferences_to_json(&p);
        assert!(json.contains("\"theme\""));
        assert!(json.contains("\"dark\""));
    }

    #[test]
    fn test_preferences_from_pairs() {
        let pairs = vec![
            ("key1".to_string(), "value1".to_string(), "cat1".to_string()),
            ("key2".to_string(), "value2".to_string(), "cat2".to_string()),
        ];
        let p = preferences_from_pairs(&pairs);
        assert_eq!(pref_count(&p), 2);
        assert_eq!(get_string(&p, "key1", ""), "value1");
    }
}
