// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! FACS Action Unit coactivation rules.
//!
//! When multiple Action Units activate simultaneously, their combined effect
//! departs from simple linear superposition.  This module encodes three classes
//! of scientifically-grounded interaction drawn from Ekman & Friesen (1978) and
//! subsequent FACS literature:
//!
//! * **Amplify** — both AUs reinforce each other (e.g. AU6 + AU12 Duchenne smile).
//! * **Suppress** — one AU partially cancels the other (e.g. AU12 vs AU15).
//! * **Gate** — one AU is a prerequisite for the other to express fully
//!   (e.g. AU25 gates AU26).
//!
//! The primary entry-points are [`default_coactivation_rules`] and
//! [`apply_coactivation_rules`].

#![allow(dead_code)]

use std::collections::HashMap;

use crate::facs::ActionUnit;

// ---------------------------------------------------------------------------
// Re-export the canonical FacsState alias so callers can use it from here.
// ---------------------------------------------------------------------------

/// FACS activation state: a mapping from Action Unit to intensity in [0, 1].
pub type FacsState = HashMap<ActionUnit, f32>;

// ---------------------------------------------------------------------------
// FacsCoactivationKind
// ---------------------------------------------------------------------------

/// The nature of the interaction between a pair of Action Units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FacsCoactivationKind {
    /// **Amplify**: when both AUs are active, each is boosted by
    /// `factor × sqrt(au_a × au_b)`.  Models muscle synergies that produce a
    /// supra-additive result (e.g. Duchenne smile).
    Amplify,
    /// **Suppress**: each AU reduces the other by `factor × partner_intensity`.
    /// Models antagonistic muscles whose simultaneous activation partially
    /// cancels the visible deformation (e.g. smile vs lip-corner depression).
    Suppress,
    /// **Gate**: `au_a` is a prerequisite for `au_b`.  The effective value of
    /// `au_b` is scaled by `sqrt(au_a)`.  When `au_a` is absent the visible
    /// effect of `au_b` is attenuated proportionally.
    Gate,
}

// ---------------------------------------------------------------------------
// FacsCoactivationRule
// ---------------------------------------------------------------------------

/// A single coactivation rule relating two Action Units.
#[derive(Debug, Clone)]
pub struct FacsCoactivationRule {
    /// First Action Unit in the pair.
    pub au_a: ActionUnit,
    /// Second Action Unit in the pair.
    pub au_b: ActionUnit,
    /// Interaction kind (Amplify / Suppress / Gate).
    pub kind: FacsCoactivationKind,
    /// Interaction strength in (0, 1].
    pub factor: f32,
}

// ---------------------------------------------------------------------------
// FacsCoactivationRules
// ---------------------------------------------------------------------------

/// Collection of coactivation rules applied as a post-process over a
/// [`FacsState`].
#[derive(Debug, Clone, Default)]
pub struct FacsCoactivationRules {
    /// Ordered list of rules to apply.  Each rule reads from the *original*
    /// input state so that rule order does not create cascading distortions
    /// within a single call to [`apply_coactivation_rules`].
    pub rules: Vec<FacsCoactivationRule>,
}

// ---------------------------------------------------------------------------
// default_coactivation_rules
// ---------------------------------------------------------------------------

/// Build a [`FacsCoactivationRules`] set containing all scientifically-grounded
/// rules described in the FACS literature.
///
/// # Rule catalogue
///
/// | # | AUs | Kind | Factor | Rationale |
/// |---|-----|------|--------|-----------|
/// | 1 | AU1 + AU2 | Amplify | 0.20 | Surprise/fear: inner + outer brow raise |
/// | 2 | AU6 + AU12 | Amplify | 0.15 | Duchenne smile: cheek raise + lip corner pull |
/// | 3 | AU9 + AU17 | Amplify | 0.10 | Disgust intensifier: nose wrinkle + chin raise |
/// | 4 | AU23 + AU24 | Amplify | 0.20 | Anger marker: lip tightener + lip pressor |
/// | 5 | AU4 + AU15 | Amplify | 0.15 | Sadness: brow lower + lip corner depress |
/// | 6 | AU5 + AU20 | Amplify | 0.15 | Fear: upper lid raise + lip stretch |
/// | 7 | AU1 + AU4 | Suppress | 0.20 | Brow raise vs brow lower partial cancellation |
/// | 8 | AU12 + AU15 | Suppress | 0.30 | Smile cancels lip corner depression |
/// | 9 | AU6 + AU7 | Suppress | 0.10 | Cheek raise vs lid tightener |
/// | 10 | AU25 + AU28 | Suppress | 0.50 | Open lips cancels lip suck |
/// | 11 | AU25 → AU26 | Gate | 0.50 | Lips-part prerequisite for jaw drop |
/// | 12 | AU5 → AU7 | Gate | 0.30 | Upper lid raise enhances lid tighten visibility |
#[allow(dead_code)]
pub fn default_coactivation_rules() -> FacsCoactivationRules {
    let rules = vec![
        // --- Synergistic (Amplify) pairs ---
        // 1. AU1 + AU2: surprise/fear component (inner + outer brow raise)
        FacsCoactivationRule {
            au_a: ActionUnit::AU1,
            au_b: ActionUnit::AU2,
            kind: FacsCoactivationKind::Amplify,
            factor: 0.20,
        },
        // 2. AU6 + AU12: genuine Duchenne smile (cheek raise + lip corner pull)
        FacsCoactivationRule {
            au_a: ActionUnit::AU6,
            au_b: ActionUnit::AU12,
            kind: FacsCoactivationKind::Amplify,
            factor: 0.15,
        },
        // 3. AU9 + AU17: disgust intensifier (nose wrinkle + chin raise)
        FacsCoactivationRule {
            au_a: ActionUnit::AU9,
            au_b: ActionUnit::AU17,
            kind: FacsCoactivationKind::Amplify,
            factor: 0.10,
        },
        // 4. AU23 + AU24: anger marker (lip tightener + lip pressor)
        FacsCoactivationRule {
            au_a: ActionUnit::AU23,
            au_b: ActionUnit::AU24,
            kind: FacsCoactivationKind::Amplify,
            factor: 0.20,
        },
        // 5. AU4 + AU15: sadness (brow lower + lip corner depress)
        FacsCoactivationRule {
            au_a: ActionUnit::AU4,
            au_b: ActionUnit::AU15,
            kind: FacsCoactivationKind::Amplify,
            factor: 0.15,
        },
        // 6. AU5 + AU20: fear (upper lid raise + lip stretch)
        FacsCoactivationRule {
            au_a: ActionUnit::AU5,
            au_b: ActionUnit::AU20,
            kind: FacsCoactivationKind::Amplify,
            factor: 0.15,
        },
        // --- Antagonistic (Suppress) pairs ---
        // 7. AU1 + AU4: inner brow raise partially cancels brow lower
        FacsCoactivationRule {
            au_a: ActionUnit::AU1,
            au_b: ActionUnit::AU4,
            kind: FacsCoactivationKind::Suppress,
            factor: 0.20,
        },
        // 8. AU12 + AU15: smile cancels lip corner depression
        FacsCoactivationRule {
            au_a: ActionUnit::AU12,
            au_b: ActionUnit::AU15,
            kind: FacsCoactivationKind::Suppress,
            factor: 0.30,
        },
        // 9. AU6 + AU7: cheek raise vs upper lid tighten (slight suppression)
        FacsCoactivationRule {
            au_a: ActionUnit::AU6,
            au_b: ActionUnit::AU7,
            kind: FacsCoactivationKind::Suppress,
            factor: 0.10,
        },
        // 10. AU25 + AU28: open lips cancels lip suck
        FacsCoactivationRule {
            au_a: ActionUnit::AU25,
            au_b: ActionUnit::AU28,
            kind: FacsCoactivationKind::Suppress,
            factor: 0.50,
        },
        // --- Prerequisite (Gate) rules ---
        // 11. AU25 → AU26: lips-part required for full jaw-drop expression
        FacsCoactivationRule {
            au_a: ActionUnit::AU25,
            au_b: ActionUnit::AU26,
            kind: FacsCoactivationKind::Gate,
            factor: 0.50,
        },
        // 12. AU5 → AU7: upper lid raise enhances lid-tighten visibility
        FacsCoactivationRule {
            au_a: ActionUnit::AU5,
            au_b: ActionUnit::AU7,
            kind: FacsCoactivationKind::Gate,
            factor: 0.30,
        },
    ];

    FacsCoactivationRules { rules }
}

// ---------------------------------------------------------------------------
// apply_coactivation_rules
// ---------------------------------------------------------------------------

/// Apply coactivation rules to a [`FacsState`] and return the adjusted state.
///
/// All rules read intensities from the *original* `state` snapshot so that no
/// rule contaminates the inputs of a later rule within the same call.
///
/// # Algorithm
///
/// For each rule:
/// * Fetch `au_a_val` and `au_b_val` from the *original* state (default 0.0).
/// * If both values are below the near-zero threshold (< 0.01), skip.
/// * **Amplify**: `boost = factor × sqrt(au_a_val × au_b_val)`.
///   Each non-zero participant is incremented by `boost` and clamped to [0, 1].
/// * **Suppress**: `reduction_a = factor × au_b_val`, `reduction_b = factor × au_a_val`.
///   Each participant is decremented by its reduction and floored at 0.
/// * **Gate**: `new_b = au_b_val × sqrt(au_a_val)` clamped to [0, 1].
///   Only `au_b` is modified.
#[allow(dead_code)]
pub fn apply_coactivation_rules(state: &FacsState, rules: &FacsCoactivationRules) -> FacsState {
    // Start from a clone of the input; accumulate adjustments into `result`
    // while reading all intensities from the original `state`.
    let mut result: FacsState = state.clone();

    for rule in &rules.rules {
        let au_a_val = *state.get(&rule.au_a).unwrap_or(&0.0_f32);
        let au_b_val = *state.get(&rule.au_b).unwrap_or(&0.0_f32);

        // Skip rules where both participants are essentially inactive.
        if au_a_val < 0.01 && au_b_val < 0.01 {
            continue;
        }

        match rule.kind {
            FacsCoactivationKind::Amplify => {
                // Geometric mean ensures boost is zero when either AU is zero.
                let boost = rule.factor * (au_a_val * au_b_val).sqrt();

                // Only update an AU in the result if it was already non-zero
                // in the original state (avoid injecting phantom activations).
                if au_a_val > 0.0 {
                    let entry = result.entry(rule.au_a.clone()).or_insert(0.0);
                    *entry = (*entry + boost).clamp(0.0, 1.0);
                }
                if au_b_val > 0.0 {
                    let entry = result.entry(rule.au_b.clone()).or_insert(0.0);
                    *entry = (*entry + boost).clamp(0.0, 1.0);
                }
            }

            FacsCoactivationKind::Suppress => {
                let reduction_a = rule.factor * au_b_val;
                let reduction_b = rule.factor * au_a_val;

                {
                    let entry = result.entry(rule.au_a.clone()).or_insert(0.0);
                    *entry = (*entry - reduction_a).max(0.0);
                }
                {
                    let entry = result.entry(rule.au_b.clone()).or_insert(0.0);
                    *entry = (*entry - reduction_b).max(0.0);
                }
            }

            FacsCoactivationKind::Gate => {
                // sqrt provides a smooth, concave response: at au_a = 0.25 the
                // gating is 50 %; at au_a = 1.0 it is 100 %.
                let gate_scale = au_a_val.sqrt();
                let new_b = (au_b_val * gate_scale).clamp(0.0, 1.0);
                // Only write back if au_b was already present in the original state.
                if state.contains_key(&rule.au_b) {
                    result.insert(rule.au_b.clone(), new_b);
                }
            }
        }
    }

    result
}

// ---------------------------------------------------------------------------
// coactivation_rule_count
// ---------------------------------------------------------------------------

/// Return the number of rules in a [`FacsCoactivationRules`] set.
#[allow(dead_code)]
#[inline]
pub fn coactivation_rule_count(rules: &FacsCoactivationRules) -> usize {
    rules.rules.len()
}

// ---------------------------------------------------------------------------
// add_coactivation_rule
// ---------------------------------------------------------------------------

/// Append a new rule to a [`FacsCoactivationRules`] set.
#[allow(dead_code)]
#[inline]
pub fn add_coactivation_rule(rules: &mut FacsCoactivationRules, rule: FacsCoactivationRule) {
    rules.rules.push(rule);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // ------------------------------------------------------------------
    // 1. default_coactivation_rules returns at least 8 rules
    // ------------------------------------------------------------------
    #[test]
    fn test_default_rules_non_empty() {
        let rules = default_coactivation_rules();
        assert!(
            rules.rules.len() >= 8,
            "expected at least 8 default rules, got {}",
            rules.rules.len()
        );
    }

    // ------------------------------------------------------------------
    // 2. Amplify: AU6 + AU12 → both values increase
    // ------------------------------------------------------------------
    #[test]
    fn test_amplify_both_active() {
        let mut state: FacsState = HashMap::new();
        state.insert(ActionUnit::AU6, 0.8_f32);
        state.insert(ActionUnit::AU12, 0.7_f32);

        let rules = FacsCoactivationRules {
            rules: vec![FacsCoactivationRule {
                au_a: ActionUnit::AU6,
                au_b: ActionUnit::AU12,
                kind: FacsCoactivationKind::Amplify,
                factor: 0.15,
            }],
        };

        let result = apply_coactivation_rules(&state, &rules);

        let au6_out = *result.get(&ActionUnit::AU6).expect("AU6 missing");
        let au12_out = *result.get(&ActionUnit::AU12).expect("AU12 missing");

        assert!(au6_out > 0.8, "AU6 should have increased: {au6_out}");
        assert!(au12_out > 0.7, "AU12 should have increased: {au12_out}");
        assert!(au6_out <= 1.0, "AU6 clamped above 1.0: {au6_out}");
        assert!(au12_out <= 1.0, "AU12 clamped above 1.0: {au12_out}");
    }

    // ------------------------------------------------------------------
    // 3. Suppress: AU1 + AU4 → both values decrease
    // ------------------------------------------------------------------
    #[test]
    fn test_suppress_both_active() {
        let mut state: FacsState = HashMap::new();
        state.insert(ActionUnit::AU1, 0.5_f32);
        state.insert(ActionUnit::AU4, 0.8_f32);

        let rules = FacsCoactivationRules {
            rules: vec![FacsCoactivationRule {
                au_a: ActionUnit::AU1,
                au_b: ActionUnit::AU4,
                kind: FacsCoactivationKind::Suppress,
                factor: 0.20,
            }],
        };

        let result = apply_coactivation_rules(&state, &rules);

        let au1_out = *result.get(&ActionUnit::AU1).expect("AU1 missing");
        let au4_out = *result.get(&ActionUnit::AU4).expect("AU4 missing");

        // AU1_out = 0.5 - 0.20 * 0.8 = 0.34
        assert!(
            au1_out < 0.5,
            "AU1 should have decreased from 0.5, got {au1_out}"
        );
        // AU4_out = 0.8 - 0.20 * 0.5 = 0.70
        assert!(
            au4_out < 0.8,
            "AU4 should have decreased from 0.8, got {au4_out}"
        );
        assert!(au1_out >= 0.0, "AU1 must not go negative: {au1_out}");
        assert!(au4_out >= 0.0, "AU4 must not go negative: {au4_out}");
    }

    // ------------------------------------------------------------------
    // 4. Gate: AU26 reduced without AU25; preserved with full AU25
    // ------------------------------------------------------------------
    #[test]
    fn test_gate_au25_au26() {
        // Without AU25: AU26 should collapse to ~0.
        let mut state_no_gate: FacsState = HashMap::new();
        state_no_gate.insert(ActionUnit::AU25, 0.0_f32);
        state_no_gate.insert(ActionUnit::AU26, 0.8_f32);

        // With AU25 fully active: AU26 should equal its original value.
        let mut state_gated: FacsState = HashMap::new();
        state_gated.insert(ActionUnit::AU25, 1.0_f32);
        state_gated.insert(ActionUnit::AU26, 0.8_f32);

        let rules = FacsCoactivationRules {
            rules: vec![FacsCoactivationRule {
                au_a: ActionUnit::AU25,
                au_b: ActionUnit::AU26,
                kind: FacsCoactivationKind::Gate,
                factor: 0.50,
            }],
        };

        let result_no_gate = apply_coactivation_rules(&state_no_gate, &rules);
        let result_gated = apply_coactivation_rules(&state_gated, &rules);

        let au26_no_gate = *result_no_gate.get(&ActionUnit::AU26).expect("AU26 missing");
        let au26_gated = *result_gated.get(&ActionUnit::AU26).expect("AU26 missing");

        // sqrt(0.0) = 0.0 → au26 = 0.8 * 0.0 = 0.0
        assert!(
            au26_no_gate < 0.01,
            "AU26 should be near 0 without AU25, got {au26_no_gate}"
        );
        // sqrt(1.0) = 1.0 → au26 = 0.8 * 1.0 = 0.8
        assert!(
            (au26_gated - 0.8).abs() < 1e-5,
            "AU26 should be ~0.8 with full AU25, got {au26_gated}"
        );
    }

    // ------------------------------------------------------------------
    // 5. Amplify rule with au_a = 0 has no effect on au_b
    // ------------------------------------------------------------------
    #[test]
    fn test_no_effect_when_one_zero() {
        let mut state: FacsState = HashMap::new();
        // AU6 absent (zero)
        state.insert(ActionUnit::AU12, 0.7_f32);

        let rules = FacsCoactivationRules {
            rules: vec![FacsCoactivationRule {
                au_a: ActionUnit::AU6,
                au_b: ActionUnit::AU12,
                kind: FacsCoactivationKind::Amplify,
                factor: 0.15,
            }],
        };

        let result = apply_coactivation_rules(&state, &rules);

        // boost = 0.15 * sqrt(0.0 * 0.7) = 0.0 → AU12 unchanged
        let au12_out = *result.get(&ActionUnit::AU12).expect("AU12 missing");
        assert!(
            (au12_out - 0.7).abs() < 1e-6,
            "AU12 should be unchanged when partner is absent, got {au12_out}"
        );
        // AU6 must not be injected as a phantom activation.
        let au6_phantom = result.get(&ActionUnit::AU6).copied().unwrap_or(0.0);
        assert!(
            au6_phantom < 1e-6,
            "AU6 should not gain phantom activation, got {au6_phantom}"
        );
    }

    // ------------------------------------------------------------------
    // 6. Result AU values stay in [0, 1]
    // ------------------------------------------------------------------
    #[test]
    fn test_result_clamped_to_unit() {
        let mut state: FacsState = HashMap::new();
        state.insert(ActionUnit::AU6, 0.99_f32);
        state.insert(ActionUnit::AU12, 0.99_f32);

        let rules = FacsCoactivationRules {
            rules: vec![FacsCoactivationRule {
                au_a: ActionUnit::AU6,
                au_b: ActionUnit::AU12,
                kind: FacsCoactivationKind::Amplify,
                factor: 1.0, // extreme factor; would exceed 1.0 without clamping
            }],
        };

        let result = apply_coactivation_rules(&state, &rules);

        for (au, &val) in &result {
            assert!(
                (0.0..=1.0).contains(&val),
                "{au:?} value {val} is outside [0, 1]"
            );
        }
    }

    // ------------------------------------------------------------------
    // 7. Empty rules returns a state equal to the input
    // ------------------------------------------------------------------
    #[test]
    fn test_empty_rules_returns_identical() {
        let mut state: FacsState = HashMap::new();
        state.insert(ActionUnit::AU1, 0.4_f32);
        state.insert(ActionUnit::AU12, 0.6_f32);
        state.insert(ActionUnit::AU26, 0.3_f32);

        let empty_rules = FacsCoactivationRules::default();
        let result = apply_coactivation_rules(&state, &empty_rules);

        assert_eq!(result.len(), state.len());
        for (au, &expected) in &state {
            let actual = *result.get(au).expect("AU missing in empty-rule result");
            assert!(
                (actual - expected).abs() < 1e-6,
                "{au:?}: expected {expected}, got {actual}"
            );
        }
    }

    // ------------------------------------------------------------------
    // 8. coactivation_rule_count and add_coactivation_rule
    // ------------------------------------------------------------------
    #[test]
    fn test_rule_count() {
        let rules = default_coactivation_rules();
        let initial_count = rules.rules.len();
        assert_eq!(
            coactivation_rule_count(&rules),
            initial_count,
            "coactivation_rule_count mismatch"
        );

        // Confirm add_coactivation_rule increments the count by exactly 1.
        let mut rules_mut = rules.clone();
        let extra = FacsCoactivationRule {
            au_a: ActionUnit::AU9,
            au_b: ActionUnit::AU15,
            kind: FacsCoactivationKind::Suppress,
            factor: 0.10,
        };
        add_coactivation_rule(&mut rules_mut, extra);
        assert_eq!(
            coactivation_rule_count(&rules_mut),
            initial_count + 1,
            "count should increment after add_coactivation_rule"
        );
    }
}
