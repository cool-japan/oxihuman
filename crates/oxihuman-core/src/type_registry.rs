//! Dynamic type registration with metadata.

#[allow(dead_code)]
#[derive(Clone)]
pub struct TypeMetadata {
    pub type_name: String,
    pub display_name: String,
    pub category: String,
    pub version: u32,
    pub serializable: bool,
    pub properties: Vec<(String, String)>,
}

#[allow(dead_code)]
pub struct TypeRegistry {
    pub entries: Vec<TypeMetadata>,
}

#[allow(dead_code)]
pub fn new_type_registry() -> TypeRegistry {
    TypeRegistry {
        entries: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn register_type(registry: &mut TypeRegistry, meta: TypeMetadata) {
    // Replace if already registered with the same name
    for entry in &mut registry.entries {
        if entry.type_name == meta.type_name {
            *entry = meta;
            return;
        }
    }
    registry.entries.push(meta);
}

#[allow(dead_code)]
pub fn get_type<'a>(registry: &'a TypeRegistry, name: &str) -> Option<&'a TypeMetadata> {
    registry.entries.iter().find(|e| e.type_name == name)
}

#[allow(dead_code)]
pub fn types_in_category<'a>(registry: &'a TypeRegistry, category: &str) -> Vec<&'a TypeMetadata> {
    registry
        .entries
        .iter()
        .filter(|e| e.category == category)
        .collect()
}

#[allow(dead_code)]
pub fn type_count(registry: &TypeRegistry) -> usize {
    registry.entries.len()
}

#[allow(dead_code)]
pub fn unregister_type(registry: &mut TypeRegistry, name: &str) -> bool {
    let before = registry.entries.len();
    registry.entries.retain(|e| e.type_name != name);
    registry.entries.len() < before
}

#[allow(dead_code)]
pub fn has_type(registry: &TypeRegistry, name: &str) -> bool {
    registry.entries.iter().any(|e| e.type_name == name)
}

#[allow(dead_code)]
pub fn all_categories(registry: &TypeRegistry) -> Vec<&str> {
    let mut cats: Vec<&str> = Vec::new();
    for entry in &registry.entries {
        let cat = entry.category.as_str();
        if !cats.contains(&cat) {
            cats.push(cat);
        }
    }
    cats
}

#[allow(dead_code)]
pub fn serializable_types(registry: &TypeRegistry) -> Vec<&TypeMetadata> {
    registry.entries.iter().filter(|e| e.serializable).collect()
}

#[allow(dead_code)]
pub fn add_property(meta: &mut TypeMetadata, prop_name: &str, type_str: &str) {
    meta.properties
        .push((prop_name.to_string(), type_str.to_string()));
}

#[allow(dead_code)]
pub fn property_count(meta: &TypeMetadata) -> usize {
    meta.properties.len()
}

#[allow(dead_code)]
pub fn type_registry_to_json(registry: &TypeRegistry) -> String {
    let mut s = String::new();
    s.push_str("{\"types\":[");
    for (i, entry) in registry.entries.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str("{\"type_name\":\"");
        s.push_str(&entry.type_name);
        s.push_str("\",\"display_name\":\"");
        s.push_str(&entry.display_name);
        s.push_str("\",\"category\":\"");
        s.push_str(&entry.category);
        s.push_str("\",\"version\":");
        s.push_str(&entry.version.to_string());
        s.push_str(",\"serializable\":");
        s.push_str(if entry.serializable { "true" } else { "false" });
        s.push_str(",\"properties\":[");
        for (j, (pname, ptype)) in entry.properties.iter().enumerate() {
            if j > 0 {
                s.push(',');
            }
            s.push_str("{\"name\":\"");
            s.push_str(pname);
            s.push_str("\",\"type\":\"");
            s.push_str(ptype);
            s.push_str("\"}");
        }
        s.push_str("]}");
    }
    s.push_str("]}");
    s
}

#[allow(dead_code)]
pub fn validate_type_meta(meta: &TypeMetadata) -> Vec<String> {
    let mut errors = Vec::new();
    if meta.type_name.is_empty() {
        errors.push("type_name must not be empty".to_string());
    }
    if meta.display_name.is_empty() {
        errors.push("display_name must not be empty".to_string());
    }
    if meta.category.is_empty() {
        errors.push("category must not be empty".to_string());
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_meta(name: &str, cat: &str, serializable: bool) -> TypeMetadata {
        TypeMetadata {
            type_name: name.to_string(),
            display_name: name.to_string(),
            category: cat.to_string(),
            version: 1,
            serializable,
            properties: Vec::new(),
        }
    }

    #[test]
    fn test_new_type_registry() {
        let r = new_type_registry();
        assert_eq!(type_count(&r), 0);
    }

    #[test]
    fn test_register_type() {
        let mut r = new_type_registry();
        register_type(&mut r, make_meta("Foo", "shapes", true));
        assert_eq!(type_count(&r), 1);
    }

    #[test]
    fn test_register_type_overwrite() {
        let mut r = new_type_registry();
        register_type(&mut r, make_meta("Foo", "shapes", true));
        register_type(&mut r, make_meta("Foo", "other", false));
        assert_eq!(type_count(&r), 1);
        assert_eq!(r.entries[0].category, "other");
    }

    #[test]
    fn test_get_type() {
        let mut r = new_type_registry();
        register_type(&mut r, make_meta("Bar", "mesh", false));
        let found = get_type(&r, "Bar");
        assert!(found.is_some());
        assert_eq!(found.expect("should succeed").category, "mesh");
    }

    #[test]
    fn test_get_type_missing() {
        let r = new_type_registry();
        assert!(get_type(&r, "Missing").is_none());
    }

    #[test]
    fn test_types_in_category() {
        let mut r = new_type_registry();
        register_type(&mut r, make_meta("A", "cat1", true));
        register_type(&mut r, make_meta("B", "cat2", true));
        register_type(&mut r, make_meta("C", "cat1", false));
        let cat1 = types_in_category(&r, "cat1");
        assert_eq!(cat1.len(), 2);
    }

    #[test]
    fn test_unregister_type() {
        let mut r = new_type_registry();
        register_type(&mut r, make_meta("X", "misc", false));
        assert!(has_type(&r, "X"));
        let removed = unregister_type(&mut r, "X");
        assert!(removed);
        assert!(!has_type(&r, "X"));
    }

    #[test]
    fn test_unregister_missing() {
        let mut r = new_type_registry();
        assert!(!unregister_type(&mut r, "nope"));
    }

    #[test]
    fn test_has_type() {
        let mut r = new_type_registry();
        register_type(&mut r, make_meta("Z", "core", true));
        assert!(has_type(&r, "Z"));
        assert!(!has_type(&r, "NotZ"));
    }

    #[test]
    fn test_all_categories() {
        let mut r = new_type_registry();
        register_type(&mut r, make_meta("A", "cat1", true));
        register_type(&mut r, make_meta("B", "cat2", true));
        register_type(&mut r, make_meta("C", "cat1", false));
        let cats = all_categories(&r);
        assert_eq!(cats.len(), 2);
        assert!(cats.contains(&"cat1"));
        assert!(cats.contains(&"cat2"));
    }

    #[test]
    fn test_serializable_types() {
        let mut r = new_type_registry();
        register_type(&mut r, make_meta("S1", "a", true));
        register_type(&mut r, make_meta("S2", "b", false));
        register_type(&mut r, make_meta("S3", "a", true));
        let ser = serializable_types(&r);
        assert_eq!(ser.len(), 2);
    }

    #[test]
    fn test_add_property() {
        let mut meta = make_meta("T", "c", false);
        add_property(&mut meta, "width", "f32");
        add_property(&mut meta, "height", "f32");
        assert_eq!(property_count(&meta), 2);
    }

    #[test]
    fn test_type_registry_to_json_non_empty() {
        let mut r = new_type_registry();
        register_type(&mut r, make_meta("MyType", "core", true));
        let json = type_registry_to_json(&r);
        assert!(!json.is_empty());
        assert!(json.contains("MyType"));
        assert!(json.contains("core"));
    }

    #[test]
    fn test_validate_type_meta_valid() {
        let meta = make_meta("Valid", "cat", true);
        let errors = validate_type_meta(&meta);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_type_meta_invalid() {
        let meta = TypeMetadata {
            type_name: "".to_string(),
            display_name: "".to_string(),
            category: "".to_string(),
            version: 1,
            serializable: false,
            properties: Vec::new(),
        };
        let errors = validate_type_meta(&meta);
        assert_eq!(errors.len(), 3);
    }
}
