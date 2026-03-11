//! Internationalization string tables.

#[allow(dead_code)]
pub struct LocaleString {
    pub key: String,
    pub value: String,
    pub context: Option<String>,
}

#[allow(dead_code)]
pub struct LocaleTable {
    pub locale_id: String,
    pub name: String,
    pub entries: Vec<LocaleString>,
}

#[allow(dead_code)]
pub struct LocalizationSystem {
    pub tables: Vec<LocaleTable>,
    pub active_locale: String,
    pub fallback_locale: String,
}

#[allow(dead_code)]
pub fn new_localization(fallback: &str) -> LocalizationSystem {
    LocalizationSystem {
        tables: Vec::new(),
        active_locale: fallback.to_string(),
        fallback_locale: fallback.to_string(),
    }
}

#[allow(dead_code)]
pub fn add_locale_table(sys: &mut LocalizationSystem, table: LocaleTable) {
    sys.tables.push(table);
}

#[allow(dead_code)]
pub fn set_active_locale(sys: &mut LocalizationSystem, locale_id: &str) {
    sys.active_locale = locale_id.to_string();
}

/// Look up a key in a given locale table, returning the value if found.
fn find_in_table<'a>(tables: &'a [LocaleTable], locale_id: &str, key: &str) -> Option<&'a str> {
    tables
        .iter()
        .find(|t| t.locale_id == locale_id)
        .and_then(|t| t.entries.iter().find(|e| e.key == key))
        .map(|e| e.value.as_str())
}

#[allow(dead_code)]
pub fn translate<'a>(sys: &'a LocalizationSystem, key: &'a str) -> &'a str {
    if let Some(v) = find_in_table(&sys.tables, &sys.active_locale, key) {
        return v;
    }
    if let Some(v) = find_in_table(&sys.tables, &sys.fallback_locale, key) {
        return v;
    }
    key
}

#[allow(dead_code)]
pub fn translate_with_context<'a>(
    sys: &'a LocalizationSystem,
    key: &'a str,
    context: &str,
) -> &'a str {
    let find_with_ctx = |tables: &'a [LocaleTable], locale: &str| -> Option<&'a str> {
        tables
            .iter()
            .find(|t| t.locale_id == locale)
            .and_then(|t| {
                t.entries
                    .iter()
                    .find(|e| {
                        e.key == key && e.context.as_deref().map(|c| c == context).unwrap_or(false)
                    })
                    .or_else(|| t.entries.iter().find(|e| e.key == key))
            })
            .map(|e| e.value.as_str())
    };

    if let Some(v) = find_with_ctx(&sys.tables, &sys.active_locale) {
        return v;
    }
    if let Some(v) = find_with_ctx(&sys.tables, &sys.fallback_locale) {
        return v;
    }
    key
}

#[allow(dead_code)]
pub fn has_key(sys: &LocalizationSystem, locale_id: &str, key: &str) -> bool {
    sys.tables
        .iter()
        .find(|t| t.locale_id == locale_id)
        .map(|t| t.entries.iter().any(|e| e.key == key))
        .unwrap_or(false)
}

#[allow(dead_code)]
pub fn locale_count(sys: &LocalizationSystem) -> usize {
    sys.tables.len()
}

#[allow(dead_code)]
pub fn key_count(table: &LocaleTable) -> usize {
    table.entries.len()
}

#[allow(dead_code)]
pub fn new_locale_table(locale_id: &str, name: &str) -> LocaleTable {
    LocaleTable {
        locale_id: locale_id.to_string(),
        name: name.to_string(),
        entries: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_locale_string(table: &mut LocaleTable, key: &str, value: &str) {
    table.entries.push(LocaleString {
        key: key.to_string(),
        value: value.to_string(),
        context: None,
    });
}

#[allow(dead_code)]
pub fn export_locale_json(table: &LocaleTable) -> String {
    let mut json = String::from("{");
    for (i, entry) in table.entries.iter().enumerate() {
        if i > 0 {
            json.push(',');
        }
        json.push('"');
        json.push_str(&entry.key);
        json.push_str("\":\"");
        json.push_str(&entry.value);
        json.push('"');
    }
    json.push('}');
    json
}

#[allow(dead_code)]
pub fn import_locale_strings(table: &mut LocaleTable, data: &[(String, String)]) {
    for (key, value) in data {
        table.entries.push(LocaleString {
            key: key.clone(),
            value: value.clone(),
            context: None,
        });
    }
}

#[allow(dead_code)]
pub fn missing_keys(
    sys: &LocalizationSystem,
    reference_locale: &str,
    target_locale: &str,
) -> Vec<String> {
    let ref_keys: Vec<&str> = sys
        .tables
        .iter()
        .find(|t| t.locale_id == reference_locale)
        .map(|t| t.entries.iter().map(|e| e.key.as_str()).collect())
        .unwrap_or_default();

    ref_keys
        .into_iter()
        .filter(|k| !has_key(sys, target_locale, k))
        .map(|k| k.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_en_table() -> LocaleTable {
        let mut t = new_locale_table("en-US", "English");
        add_locale_string(&mut t, "greeting", "Hello");
        add_locale_string(&mut t, "farewell", "Goodbye");
        t
    }

    fn make_ja_table() -> LocaleTable {
        let mut t = new_locale_table("ja-JP", "Japanese");
        add_locale_string(&mut t, "greeting", "こんにちは");
        t
    }

    #[test]
    fn test_new_localization() {
        let sys = new_localization("en-US");
        assert_eq!(sys.fallback_locale, "en-US");
        assert_eq!(sys.active_locale, "en-US");
        assert!(sys.tables.is_empty());
    }

    #[test]
    fn test_add_locale_table() {
        let mut sys = new_localization("en-US");
        add_locale_table(&mut sys, make_en_table());
        assert_eq!(locale_count(&sys), 1);
    }

    #[test]
    fn test_locale_count() {
        let mut sys = new_localization("en-US");
        add_locale_table(&mut sys, make_en_table());
        add_locale_table(&mut sys, make_ja_table());
        assert_eq!(locale_count(&sys), 2);
    }

    #[test]
    fn test_key_count() {
        let table = make_en_table();
        assert_eq!(key_count(&table), 2);
    }

    #[test]
    fn test_translate_known_key() {
        let mut sys = new_localization("en-US");
        add_locale_table(&mut sys, make_en_table());
        assert_eq!(translate(&sys, "greeting"), "Hello");
    }

    #[test]
    fn test_translate_unknown_falls_back_to_key() {
        let mut sys = new_localization("en-US");
        add_locale_table(&mut sys, make_en_table());
        assert_eq!(translate(&sys, "unknown_key"), "unknown_key");
    }

    #[test]
    fn test_translate_active_locale_priority() {
        let mut sys = new_localization("en-US");
        add_locale_table(&mut sys, make_en_table());
        add_locale_table(&mut sys, make_ja_table());
        set_active_locale(&mut sys, "ja-JP");
        assert_eq!(translate(&sys, "greeting"), "こんにちは");
    }

    #[test]
    fn test_translate_fallback_when_key_missing_in_active() {
        let mut sys = new_localization("en-US");
        add_locale_table(&mut sys, make_en_table());
        add_locale_table(&mut sys, make_ja_table());
        set_active_locale(&mut sys, "ja-JP");
        // "farewell" is not in ja-JP, should fall back to en-US
        assert_eq!(translate(&sys, "farewell"), "Goodbye");
    }

    #[test]
    fn test_has_key_true() {
        let mut sys = new_localization("en-US");
        add_locale_table(&mut sys, make_en_table());
        assert!(has_key(&sys, "en-US", "greeting"));
    }

    #[test]
    fn test_has_key_false() {
        let mut sys = new_localization("en-US");
        add_locale_table(&mut sys, make_en_table());
        assert!(!has_key(&sys, "en-US", "nonexistent"));
    }

    #[test]
    fn test_export_locale_json_non_empty() {
        let table = make_en_table();
        let json = export_locale_json(&table);
        assert!(!json.is_empty());
        assert!(json.contains("greeting"));
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_import_locale_strings() {
        let mut table = new_locale_table("en-US", "English");
        let data = vec![
            ("key1".to_string(), "val1".to_string()),
            ("key2".to_string(), "val2".to_string()),
        ];
        import_locale_strings(&mut table, &data);
        assert_eq!(key_count(&table), 2);
    }

    #[test]
    fn test_missing_keys_detects_gap() {
        let mut sys = new_localization("en-US");
        add_locale_table(&mut sys, make_en_table());
        add_locale_table(&mut sys, make_ja_table());
        let missing = missing_keys(&sys, "en-US", "ja-JP");
        assert!(missing.contains(&"farewell".to_string()));
        assert!(!missing.contains(&"greeting".to_string()));
    }

    #[test]
    fn test_missing_keys_empty_when_complete() {
        let mut sys = new_localization("en-US");
        add_locale_table(&mut sys, make_en_table());
        let mut full = make_en_table();
        full.locale_id = "fr-FR".to_string();
        add_locale_table(&mut sys, full);
        let missing = missing_keys(&sys, "en-US", "fr-FR");
        assert!(missing.is_empty());
    }

    #[test]
    fn test_translate_with_context() {
        let mut sys = new_localization("en-US");
        let mut table = new_locale_table("en-US", "English");
        table.entries.push(LocaleString {
            key: "action".to_string(),
            value: "File (verb)".to_string(),
            context: Some("verb".to_string()),
        });
        table.entries.push(LocaleString {
            key: "action".to_string(),
            value: "File (noun)".to_string(),
            context: Some("noun".to_string()),
        });
        add_locale_table(&mut sys, table);
        let v = translate_with_context(&sys, "action", "verb");
        assert_eq!(v, "File (verb)");
    }

    #[test]
    fn test_new_locale_table() {
        let table = new_locale_table("de-DE", "German");
        assert_eq!(table.locale_id, "de-DE");
        assert_eq!(table.name, "German");
        assert!(table.entries.is_empty());
    }
}
