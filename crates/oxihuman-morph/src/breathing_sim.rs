//! Breathing simulation for chest/belly deformation.

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BreathPhase {
    Inhale,
    Exhale,
    Hold,
}

#[allow(dead_code)]
pub struct BreathCycle {
    pub phase: BreathPhase,
    pub time: f32,
    pub inhale_duration: f32,
    pub exhale_duration: f32,
    pub hold_duration: f32,
    pub amplitude: f32,
}

#[allow(dead_code)]
pub struct BreathRegion {
    pub name: String,
    pub vertex_indices: Vec<usize>,
    pub direction: [f32; 3],
    pub contribution: f32,
}

#[allow(dead_code)]
pub struct BreathingState {
    pub cycle: BreathCycle,
    pub regions: Vec<BreathRegion>,
    pub breath_value: f32,
}

#[allow(dead_code)]
pub fn default_breath_cycle() -> BreathCycle {
    BreathCycle {
        phase: BreathPhase::Inhale,
        time: 0.0,
        inhale_duration: 2.0,
        exhale_duration: 2.0,
        hold_duration: 0.0,
        amplitude: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_breathing_state() -> BreathingState {
    BreathingState {
        cycle: default_breath_cycle(),
        regions: Vec::new(),
        breath_value: 0.0,
    }
}

#[allow(dead_code)]
pub fn breath_value_at(cycle: &BreathCycle) -> f32 {
    match cycle.phase {
        BreathPhase::Inhale => {
            if cycle.inhale_duration > 0.0 {
                (cycle.time / cycle.inhale_duration).clamp(0.0, 1.0)
            } else {
                1.0
            }
        }
        BreathPhase::Exhale => {
            if cycle.exhale_duration > 0.0 {
                1.0 - (cycle.time / cycle.exhale_duration).clamp(0.0, 1.0)
            } else {
                0.0
            }
        }
        BreathPhase::Hold => 1.0,
    }
}

#[allow(dead_code)]
pub fn advance_breath(state: &mut BreathingState, dt: f32) {
    state.cycle.time += dt;
    loop {
        match state.cycle.phase {
            BreathPhase::Inhale => {
                if state.cycle.time >= state.cycle.inhale_duration {
                    state.cycle.time -= state.cycle.inhale_duration;
                    if state.cycle.hold_duration > 0.0 {
                        state.cycle.phase = BreathPhase::Hold;
                    } else {
                        state.cycle.phase = BreathPhase::Exhale;
                    }
                } else {
                    break;
                }
            }
            BreathPhase::Hold => {
                if state.cycle.time >= state.cycle.hold_duration {
                    state.cycle.time -= state.cycle.hold_duration;
                    state.cycle.phase = BreathPhase::Exhale;
                } else {
                    break;
                }
            }
            BreathPhase::Exhale => {
                if state.cycle.time >= state.cycle.exhale_duration {
                    state.cycle.time -= state.cycle.exhale_duration;
                    state.cycle.phase = BreathPhase::Inhale;
                } else {
                    break;
                }
            }
        }
    }
    state.breath_value = breath_value_at(&state.cycle);
}

#[allow(dead_code)]
pub fn apply_breathing(positions: &mut [[f32; 3]], state: &BreathingState) {
    let bv = state.breath_value;
    let amp = state.cycle.amplitude;
    for region in &state.regions {
        let disp = [
            region.direction[0] * bv * amp * region.contribution,
            region.direction[1] * bv * amp * region.contribution,
            region.direction[2] * bv * amp * region.contribution,
        ];
        for &vi in &region.vertex_indices {
            if vi < positions.len() {
                positions[vi][0] += disp[0];
                positions[vi][1] += disp[1];
                positions[vi][2] += disp[2];
            }
        }
    }
}

#[allow(dead_code)]
pub fn add_breath_region(state: &mut BreathingState, region: BreathRegion) {
    state.regions.push(region);
}

#[allow(dead_code)]
pub fn set_breath_rate(state: &mut BreathingState, breaths_per_minute: f32) {
    let cycle_time = 60.0 / breaths_per_minute.max(0.001);
    let half = cycle_time / 2.0;
    state.cycle.inhale_duration = half;
    state.cycle.exhale_duration = half;
}

#[allow(dead_code)]
pub fn set_breath_amplitude(state: &mut BreathingState, amplitude: f32) {
    state.cycle.amplitude = amplitude;
}

#[allow(dead_code)]
pub fn inhale_value(state: &BreathingState) -> f32 {
    state.breath_value
}

#[allow(dead_code)]
pub fn exhale_value(state: &BreathingState) -> f32 {
    1.0 - state.breath_value
}

#[allow(dead_code)]
pub fn current_phase(state: &BreathingState) -> BreathPhase {
    state.cycle.phase
}

#[allow(dead_code)]
pub fn breath_region_count(state: &BreathingState) -> usize {
    state.regions.len()
}

#[allow(dead_code)]
pub fn blend_breath_states(a: &BreathingState, b: &BreathingState, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    a.breath_value * (1.0 - t) + b.breath_value * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_breath_cycle() {
        let cycle = default_breath_cycle();
        assert_eq!(cycle.phase, BreathPhase::Inhale);
        assert_eq!(cycle.time, 0.0);
        assert!(cycle.inhale_duration > 0.0);
        assert!(cycle.exhale_duration > 0.0);
        assert!(cycle.amplitude > 0.0);
    }

    #[test]
    fn test_new_breathing_state() {
        let state = new_breathing_state();
        assert_eq!(state.breath_value, 0.0);
        assert!(state.regions.is_empty());
    }

    #[test]
    fn test_advance_breath_increases_time() {
        let mut state = new_breathing_state();
        advance_breath(&mut state, 0.5);
        // time should have advanced (may wrap to next phase), breath_value updated
        let bv = state.breath_value;
        assert!((0.0..=1.0).contains(&bv));
    }

    #[test]
    fn test_advance_breath_changes_phase() {
        let mut state = new_breathing_state();
        // skip through the full inhale duration
        advance_breath(&mut state, 2.5);
        // should have moved to Exhale
        assert_eq!(current_phase(&state), BreathPhase::Exhale);
    }

    #[test]
    fn test_breath_value_at_inhale_start() {
        let cycle = default_breath_cycle();
        // At time=0, inhale phase: value = 0/2 = 0
        assert_eq!(breath_value_at(&cycle), 0.0);
    }

    #[test]
    fn test_breath_value_at_inhale_midpoint() {
        let mut cycle = default_breath_cycle();
        cycle.time = 1.0;
        // At time=1, inhale phase: value = 1/2 = 0.5
        assert!((breath_value_at(&cycle) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_breath_value_at_exhale() {
        let mut cycle = default_breath_cycle();
        cycle.phase = BreathPhase::Exhale;
        cycle.time = 1.0;
        // At time=1, exhale phase: value = 1 - 1/2 = 0.5
        assert!((breath_value_at(&cycle) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_set_breath_rate() {
        let mut state = new_breathing_state();
        set_breath_rate(&mut state, 15.0);
        // 15 bpm => cycle = 4s, half = 2s each
        assert!((state.cycle.inhale_duration - 2.0).abs() < 1e-4);
        assert!((state.cycle.exhale_duration - 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_set_breath_amplitude() {
        let mut state = new_breathing_state();
        set_breath_amplitude(&mut state, 2.5);
        assert_eq!(state.cycle.amplitude, 2.5);
    }

    #[test]
    fn test_add_breath_region() {
        let mut state = new_breathing_state();
        let region = BreathRegion {
            name: "chest".to_string(),
            vertex_indices: vec![0, 1, 2],
            direction: [0.0, 1.0, 0.0],
            contribution: 0.8,
        };
        add_breath_region(&mut state, region);
        assert_eq!(breath_region_count(&state), 1);
    }

    #[test]
    fn test_breath_region_count() {
        let mut state = new_breathing_state();
        add_breath_region(
            &mut state,
            BreathRegion {
                name: "r1".to_string(),
                vertex_indices: vec![],
                direction: [1.0, 0.0, 0.0],
                contribution: 0.5,
            },
        );
        add_breath_region(
            &mut state,
            BreathRegion {
                name: "r2".to_string(),
                vertex_indices: vec![],
                direction: [0.0, 1.0, 0.0],
                contribution: 0.5,
            },
        );
        assert_eq!(breath_region_count(&state), 2);
    }

    #[test]
    fn test_inhale_exhale_values_sum_to_one() {
        let mut state = new_breathing_state();
        state.breath_value = 0.7;
        let iv = inhale_value(&state);
        let ev = exhale_value(&state);
        assert!((iv + ev - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_blend_breath_states() {
        let mut a = new_breathing_state();
        a.breath_value = 0.0;
        let mut b = new_breathing_state();
        b.breath_value = 1.0;
        let blended = blend_breath_states(&a, &b, 0.5);
        assert!((blended - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_blend_breath_states_extremes() {
        let mut a = new_breathing_state();
        a.breath_value = 0.3;
        let mut b = new_breathing_state();
        b.breath_value = 0.9;
        assert!((blend_breath_states(&a, &b, 0.0) - 0.3).abs() < 1e-5);
        assert!((blend_breath_states(&a, &b, 1.0) - 0.9).abs() < 1e-5);
    }

    #[test]
    fn test_apply_breathing_displaces_positions() {
        let mut state = new_breathing_state();
        state.breath_value = 1.0;
        state.cycle.amplitude = 1.0;
        let region = BreathRegion {
            name: "chest".to_string(),
            vertex_indices: vec![0],
            direction: [0.0, 1.0, 0.0],
            contribution: 1.0,
        };
        add_breath_region(&mut state, region);
        let mut positions = [[0.0_f32; 3]; 2];
        apply_breathing(&mut positions, &state);
        assert!((positions[0][1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_current_phase_initial() {
        let state = new_breathing_state();
        assert_eq!(current_phase(&state), BreathPhase::Inhale);
    }
}
