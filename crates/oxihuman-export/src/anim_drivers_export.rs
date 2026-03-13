#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export animation drivers (expression-driven values).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DriverVar {
    pub name: String,
    pub target_id: String,
    pub data_path: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnimDriverExport {
    pub data_path: String,
    pub expression: String,
    pub vars: Vec<DriverVar>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct AnimDriversExport {
    pub drivers: Vec<AnimDriverExport>,
}

#[allow(dead_code)]
pub fn new_anim_drivers_export() -> AnimDriversExport {
    AnimDriversExport { drivers: Vec::new() }
}

#[allow(dead_code)]
pub fn add_anim_driver(exp: &mut AnimDriversExport, path: &str, expr: &str) {
    exp.drivers.push(AnimDriverExport {
        data_path: path.to_string(),
        expression: expr.to_string(),
        vars: Vec::new(),
    });
}

#[allow(dead_code)]
pub fn add_driver_var(driver: &mut AnimDriverExport, name: &str, target: &str, dpath: &str) {
    driver.vars.push(DriverVar {
        name: name.to_string(),
        target_id: target.to_string(),
        data_path: dpath.to_string(),
    });
}

#[allow(dead_code)]
pub fn export_anim_drivers_to_json(exp: &AnimDriversExport) -> String {
    let mut drivers_json = String::new();
    for (i, d) in exp.drivers.iter().enumerate() {
        if i > 0 {
            drivers_json.push(',');
        }
        let vars: Vec<String> = d
            .vars
            .iter()
            .map(|v| {
                format!(
                    r#"{{"name":"{}","target":"{}","path":"{}"}}"#,
                    v.name, v.target_id, v.data_path
                )
            })
            .collect();
        drivers_json.push_str(&format!(
            r#"{{"path":"{}","expr":"{}","vars":[{}]}}"#,
            d.data_path,
            d.expression,
            vars.join(",")
        ));
    }
    format!(r#"{{"drivers":[{}]}}"#, drivers_json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        let e = new_anim_drivers_export();
        assert!(e.drivers.is_empty());
    }

    #[test]
    fn add_driver_increases_count() {
        let mut e = new_anim_drivers_export();
        add_anim_driver(&mut e, "pose.bones[\"arm\"].location[0]", "x * 2");
        assert_eq!(e.drivers.len(), 1);
    }

    #[test]
    fn driver_expression_stored() {
        let mut e = new_anim_drivers_export();
        add_anim_driver(&mut e, "path", "expr_abc");
        assert_eq!(e.drivers[0].expression, "expr_abc");
    }

    #[test]
    fn add_var_to_driver() {
        let mut e = new_anim_drivers_export();
        add_anim_driver(&mut e, "p", "x");
        add_driver_var(&mut e.drivers[0], "v1", "obj", "location[0]");
        assert_eq!(e.drivers[0].vars.len(), 1);
    }

    #[test]
    fn var_fields_stored() {
        let mut e = new_anim_drivers_export();
        add_anim_driver(&mut e, "p", "x");
        add_driver_var(&mut e.drivers[0], "myvar", "myobj", "loc[0]");
        assert_eq!(e.drivers[0].vars[0].name, "myvar");
    }

    #[test]
    fn export_json_has_drivers() {
        let e = new_anim_drivers_export();
        let j = export_anim_drivers_to_json(&e);
        assert!(j.contains("drivers"));
    }

    #[test]
    fn export_json_has_path() {
        let mut e = new_anim_drivers_export();
        add_anim_driver(&mut e, "some.path", "1+1");
        let j = export_anim_drivers_to_json(&e);
        assert!(j.contains("some.path"));
    }

    #[test]
    fn export_json_has_vars_when_present() {
        let mut e = new_anim_drivers_export();
        add_anim_driver(&mut e, "p", "x");
        add_driver_var(&mut e.drivers[0], "v", "t", "d");
        let j = export_anim_drivers_to_json(&e);
        assert!(j.contains("vars"));
    }

    #[test]
    fn multiple_drivers() {
        let mut e = new_anim_drivers_export();
        add_anim_driver(&mut e, "p1", "e1");
        add_anim_driver(&mut e, "p2", "e2");
        assert_eq!(e.drivers.len(), 2);
    }
}
