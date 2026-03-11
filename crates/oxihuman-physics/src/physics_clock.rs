#![allow(dead_code)]

/// A physics simulation clock.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PhysicsClock {
    dt: f32,
    total_time: f64,
    frame_count: u64,
    scale: f32,
    paused: bool,
}

/// Creates a new physics clock with the given fixed timestep.
#[allow(dead_code)]
pub fn new_physics_clock(dt: f32) -> PhysicsClock {
    PhysicsClock {
        dt,
        total_time: 0.0,
        frame_count: 0,
        scale: 1.0,
        paused: false,
    }
}

/// Advances the clock by one tick.
#[allow(dead_code)]
pub fn clock_tick(clock: &mut PhysicsClock) {
    if !clock.paused {
        clock.total_time += (clock.dt * clock.scale) as f64;
        clock.frame_count += 1;
    }
}

/// Returns the fixed timestep.
#[allow(dead_code)]
pub fn clock_dt(clock: &PhysicsClock) -> f32 {
    clock.dt * clock.scale
}

/// Returns total accumulated time.
#[allow(dead_code)]
pub fn clock_total_time(clock: &PhysicsClock) -> f64 {
    clock.total_time
}

/// Returns the frame count.
#[allow(dead_code)]
pub fn clock_frame_count(clock: &PhysicsClock) -> u64 {
    clock.frame_count
}

/// Resets the clock.
#[allow(dead_code)]
pub fn clock_reset(clock: &mut PhysicsClock) {
    clock.total_time = 0.0;
    clock.frame_count = 0;
}

/// Sets the time scale.
#[allow(dead_code)]
pub fn clock_scale(clock: &mut PhysicsClock, scale: f32) {
    clock.scale = scale;
}

/// Returns whether the clock is paused.
#[allow(dead_code)]
pub fn clock_is_paused(clock: &PhysicsClock) -> bool {
    clock.paused
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_clock() {
        let c = new_physics_clock(0.01);
        assert!((clock_dt(&c) - 0.01).abs() < f32::EPSILON);
    }

    #[test]
    fn test_tick() {
        let mut c = new_physics_clock(0.01);
        clock_tick(&mut c);
        assert!(clock_total_time(&c) > 0.0);
        assert_eq!(clock_frame_count(&c), 1);
    }

    #[test]
    fn test_multiple_ticks() {
        let mut c = new_physics_clock(0.01);
        for _ in 0..100 {
            clock_tick(&mut c);
        }
        assert_eq!(clock_frame_count(&c), 100);
    }

    #[test]
    fn test_reset() {
        let mut c = new_physics_clock(0.01);
        clock_tick(&mut c);
        clock_reset(&mut c);
        assert!(clock_total_time(&c).abs() < f64::EPSILON);
        assert_eq!(clock_frame_count(&c), 0);
    }

    #[test]
    fn test_scale() {
        let mut c = new_physics_clock(0.01);
        clock_scale(&mut c, 2.0);
        assert!((clock_dt(&c) - 0.02).abs() < 1e-5);
    }

    #[test]
    fn test_paused() {
        let mut c = new_physics_clock(0.01);
        c.paused = true;
        clock_tick(&mut c);
        assert_eq!(clock_frame_count(&c), 0);
    }

    #[test]
    fn test_is_paused() {
        let c = new_physics_clock(0.01);
        assert!(!clock_is_paused(&c));
    }

    #[test]
    fn test_total_time_accuracy() {
        let mut c = new_physics_clock(0.1);
        for _ in 0..10 {
            clock_tick(&mut c);
        }
        assert!((clock_total_time(&c) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_scale_zero() {
        let mut c = new_physics_clock(0.01);
        clock_scale(&mut c, 0.0);
        clock_tick(&mut c);
        assert!(clock_total_time(&c).abs() < f64::EPSILON);
    }

    #[test]
    fn test_frame_count_after_reset() {
        let mut c = new_physics_clock(0.01);
        clock_tick(&mut c);
        clock_tick(&mut c);
        clock_reset(&mut c);
        clock_tick(&mut c);
        assert_eq!(clock_frame_count(&c), 1);
    }
}
