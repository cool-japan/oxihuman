// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Step pattern generator stub.

/// A single step in a pattern.
#[derive(Debug, Clone, PartialEq)]
pub struct StepEvent {
    pub time: f32,
    pub foot_id: usize,
    pub x: f32,
    pub y: f32,
    pub duration: f32,
}

/// Step pattern for a biped.
#[derive(Debug, Clone, Default)]
pub struct StepPattern {
    pub events: Vec<StepEvent>,
    pub cycle_duration: f32,
}

impl StepPattern {
    pub fn new(cycle_duration: f32) -> Self {
        Self {
            events: Vec::new(),
            cycle_duration,
        }
    }

    pub fn add_event(&mut self, evt: StepEvent) {
        self.events.push(evt);
        self.events.sort_by(|a, b| {
            a.time
                .partial_cmp(&b.time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

/// Generate a symmetric biped walk step pattern for N cycles.
pub fn generate_walk_pattern(
    n_cycles: usize,
    step_length: f32,
    step_width: f32,
    cycle_duration: f32,
) -> StepPattern {
    let mut pat = StepPattern::new(cycle_duration);
    let half = cycle_duration * 0.5;
    for i in 0..n_cycles {
        let t_base = i as f32 * cycle_duration;
        /* left foot (foot_id = 0) */
        pat.add_event(StepEvent {
            time: t_base,
            foot_id: 0,
            x: (i as f32 + 1.0) * step_length,
            y: step_width * 0.5,
            duration: half,
        });
        /* right foot (foot_id = 1) */
        pat.add_event(StepEvent {
            time: t_base + half,
            foot_id: 1,
            x: (i as f32 + 1.0) * step_length,
            y: -step_width * 0.5,
            duration: half,
        });
    }
    pat
}

/// Return whether the pattern has alternating left/right steps.
pub fn pattern_alternates(pat: &StepPattern) -> bool {
    pat.events.windows(2).all(|w| w[0].foot_id != w[1].foot_id)
}

/// Return the total duration of the pattern.
pub fn pattern_total_duration(pat: &StepPattern) -> f32 {
    pat.events
        .iter()
        .map(|e| e.time + e.duration)
        .fold(0.0f32, f32::max)
}

/// Find the active step at time `t` for a given foot.
pub fn active_step(pat: &StepPattern, foot_id: usize, t: f32) -> Option<&StepEvent> {
    pat.events
        .iter()
        .find(|e| e.foot_id == foot_id && (e.time..=(e.time + e.duration)).contains(&t))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_walk_pattern_event_count() {
        /* 2 events per cycle */
        let pat = generate_walk_pattern(3, 0.3, 0.1, 1.0);
        assert_eq!(pat.event_count(), 6);
    }

    #[test]
    fn test_walk_pattern_alternates() {
        /* events alternate feet */
        let pat = generate_walk_pattern(4, 0.3, 0.1, 1.0);
        assert!(pattern_alternates(&pat));
    }

    #[test]
    fn test_total_duration_positive() {
        /* total duration is positive */
        let pat = generate_walk_pattern(2, 0.3, 0.1, 1.0);
        assert!(pattern_total_duration(&pat) > 0.0);
    }

    #[test]
    fn test_active_step_found() {
        /* active step found at correct time */
        let pat = generate_walk_pattern(2, 0.3, 0.1, 1.0);
        let s = active_step(&pat, 0, 0.1);
        assert!(s.is_some());
    }

    #[test]
    fn test_active_step_not_found() {
        /* no step found for wrong foot at given time */
        let pat = generate_walk_pattern(2, 0.3, 0.1, 1.0);
        /* foot 1 starts at t=0.5, so at t=0.1 it should not be active */
        let s = active_step(&pat, 1, 0.1);
        assert!(s.is_none());
    }

    #[test]
    fn test_add_event_sorted() {
        /* events are sorted by time */
        let mut pat = StepPattern::new(1.0);
        pat.add_event(StepEvent {
            time: 0.8,
            foot_id: 1,
            x: 0.0,
            y: 0.0,
            duration: 0.2,
        });
        pat.add_event(StepEvent {
            time: 0.2,
            foot_id: 0,
            x: 0.0,
            y: 0.0,
            duration: 0.2,
        });
        assert!(pat.events[0].time <= pat.events[1].time);
    }

    #[test]
    fn test_empty_pattern() {
        /* empty pattern has zero duration */
        let pat = StepPattern::new(1.0);
        assert_eq!(pattern_total_duration(&pat), 0.0);
    }

    #[test]
    fn test_empty_no_active() {
        /* no active step in empty pattern */
        let pat = StepPattern::new(1.0);
        assert!(active_step(&pat, 0, 0.0).is_none());
    }

    #[test]
    fn test_zero_cycles() {
        /* zero cycles generates no events */
        let pat = generate_walk_pattern(0, 0.3, 0.1, 1.0);
        assert_eq!(pat.event_count(), 0);
    }
}
