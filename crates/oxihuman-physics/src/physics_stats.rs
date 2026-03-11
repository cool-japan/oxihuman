#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PhysicsStats {
    step_times: Vec<f32>,
    body_count: u32,
    constraint_count: u32,
}

#[allow(dead_code)]
pub fn new_physics_stats() -> PhysicsStats {
    PhysicsStats {
        step_times: Vec::new(),
        body_count: 0,
        constraint_count: 0,
    }
}

#[allow(dead_code)]
pub fn record_step_time(stats: &mut PhysicsStats, time_ms: f32) {
    stats.step_times.push(time_ms);
}

#[allow(dead_code)]
pub fn average_step_time(stats: &PhysicsStats) -> f32 {
    if stats.step_times.is_empty() {
        return 0.0;
    }
    let sum: f32 = stats.step_times.iter().sum();
    sum / stats.step_times.len() as f32
}

#[allow(dead_code)]
pub fn max_step_time(stats: &PhysicsStats) -> f32 {
    stats.step_times.iter().copied().fold(0.0f32, f32::max)
}

#[allow(dead_code)]
pub fn total_steps(stats: &PhysicsStats) -> usize {
    stats.step_times.len()
}

#[allow(dead_code)]
pub fn body_count_stats(stats: &PhysicsStats) -> u32 {
    stats.body_count
}

#[allow(dead_code)]
pub fn constraint_count_stats(stats: &PhysicsStats) -> u32 {
    stats.constraint_count
}

#[allow(dead_code)]
pub fn stats_to_json(stats: &PhysicsStats) -> String {
    format!(
        "{{\"avg_step\":{},\"max_step\":{},\"total_steps\":{},\"bodies\":{},\"constraints\":{}}}",
        average_step_time(stats),
        max_step_time(stats),
        total_steps(stats),
        stats.body_count,
        stats.constraint_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let s = new_physics_stats();
        assert_eq!(total_steps(&s), 0);
    }

    #[test]
    fn test_record() {
        let mut s = new_physics_stats();
        record_step_time(&mut s, 1.0);
        assert_eq!(total_steps(&s), 1);
    }

    #[test]
    fn test_average() {
        let mut s = new_physics_stats();
        record_step_time(&mut s, 2.0);
        record_step_time(&mut s, 4.0);
        assert!((average_step_time(&s) - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_average_empty() {
        let s = new_physics_stats();
        assert_eq!(average_step_time(&s), 0.0);
    }

    #[test]
    fn test_max() {
        let mut s = new_physics_stats();
        record_step_time(&mut s, 1.0);
        record_step_time(&mut s, 5.0);
        record_step_time(&mut s, 3.0);
        assert!((max_step_time(&s) - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_body_count() {
        let mut s = new_physics_stats();
        s.body_count = 10;
        assert_eq!(body_count_stats(&s), 10);
    }

    #[test]
    fn test_constraint_count() {
        let mut s = new_physics_stats();
        s.constraint_count = 5;
        assert_eq!(constraint_count_stats(&s), 5);
    }

    #[test]
    fn test_to_json() {
        let s = new_physics_stats();
        let j = stats_to_json(&s);
        assert!(j.contains("\"avg_step\":"));
        assert!(j.contains("\"total_steps\":"));
    }

    #[test]
    fn test_max_empty() {
        let s = new_physics_stats();
        assert_eq!(max_step_time(&s), 0.0);
    }

    #[test]
    fn test_multiple_records() {
        let mut s = new_physics_stats();
        for i in 0..10 {
            record_step_time(&mut s, i as f32);
        }
        assert_eq!(total_steps(&s), 10);
    }
}
