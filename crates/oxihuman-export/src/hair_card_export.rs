#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export hair cards (flat quads for hair rendering).

#[allow(dead_code)]
pub struct HairCard {
    pub root: [f32; 3],
    pub tip: [f32; 3],
    pub width: f32,
    pub uv_row: f32,
}

#[allow(dead_code)]
pub struct HairCardExport {
    pub name: String,
    pub cards: Vec<HairCard>,
}

#[allow(dead_code)]
pub fn new_hair_card_export(name: &str) -> HairCardExport {
    HairCardExport {
        name: name.to_string(),
        cards: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_card(
    exp: &mut HairCardExport,
    root: [f32; 3],
    tip: [f32; 3],
    width: f32,
    uv_row: f32,
) {
    exp.cards.push(HairCard { root, tip, width, uv_row });
}

#[allow(dead_code)]
pub fn export_hair_cards_to_json(exp: &HairCardExport) -> String {
    let mut s = format!("{{\"name\":\"{}\",\"cards\":[", exp.name);
    for (i, c) in exp.cards.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"root\":[{},{},{}],\"tip\":[{},{},{}],\"width\":{},\"uv_row\":{}}}",
            c.root[0], c.root[1], c.root[2],
            c.tip[0], c.tip[1], c.tip[2],
            c.width, c.uv_row
        ));
    }
    s.push_str("]}");
    s
}

#[allow(dead_code)]
pub fn card_count(exp: &HairCardExport) -> usize {
    exp.cards.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_exp() -> HairCardExport {
        let mut exp = new_hair_card_export("hair");
        add_card(&mut exp, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.1, 0.0);
        add_card(&mut exp, [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], 0.1, 0.5);
        exp
    }

    #[test]
    fn new_export_is_empty() {
        let exp = new_hair_card_export("test");
        assert_eq!(card_count(&exp), 0);
    }

    #[test]
    fn add_card_increases_count() {
        let mut exp = new_hair_card_export("test");
        add_card(&mut exp, [0.0; 3], [0.0, 1.0, 0.0], 0.1, 0.0);
        assert_eq!(card_count(&exp), 1);
    }

    #[test]
    fn name_preserved() {
        let exp = new_hair_card_export("my_hair");
        assert_eq!(exp.name, "my_hair");
    }

    #[test]
    fn card_count_matches() {
        let exp = make_exp();
        assert_eq!(card_count(&exp), 2);
    }

    #[test]
    fn json_contains_name() {
        let exp = make_exp();
        let json = export_hair_cards_to_json(&exp);
        assert!(json.contains("hair"));
    }

    #[test]
    fn json_contains_root() {
        let exp = make_exp();
        let json = export_hair_cards_to_json(&exp);
        assert!(json.contains("root"));
    }

    #[test]
    fn json_contains_tip() {
        let exp = make_exp();
        let json = export_hair_cards_to_json(&exp);
        assert!(json.contains("tip"));
    }

    #[test]
    fn card_width_stored() {
        let exp = make_exp();
        assert!((exp.cards[0].width - 0.1).abs() < 1e-6);
    }

    #[test]
    fn card_uv_row_stored() {
        let exp = make_exp();
        assert!((exp.cards[1].uv_row - 0.5).abs() < 1e-6);
    }

    #[test]
    fn json_brackets_valid() {
        let exp = make_exp();
        let json = export_hair_cards_to_json(&exp);
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
    }
}
