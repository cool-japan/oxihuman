#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ProgressTracker {
    current: f32,
    total: f32,
}

#[allow(dead_code)]
pub fn new_progress_tracker(total: f32) -> ProgressTracker {
    ProgressTracker {
        current: 0.0,
        total: if total <= 0.0 { 1.0 } else { total },
    }
}

#[allow(dead_code)]
pub fn set_progress(pt: &mut ProgressTracker, value: f32) {
    pt.current = value.clamp(0.0, pt.total);
}

#[allow(dead_code)]
pub fn get_progress(pt: &ProgressTracker) -> f32 {
    pt.current
}

#[allow(dead_code)]
pub fn progress_percent(pt: &ProgressTracker) -> f32 {
    (pt.current / pt.total) * 100.0
}

#[allow(dead_code)]
pub fn is_complete(pt: &ProgressTracker) -> bool {
    (pt.current - pt.total).abs() < f32::EPSILON
}

#[allow(dead_code)]
pub fn progress_reset(pt: &mut ProgressTracker) {
    pt.current = 0.0;
}

#[allow(dead_code)]
pub fn progress_increment(pt: &mut ProgressTracker, amount: f32) {
    pt.current = (pt.current + amount).min(pt.total);
}

#[allow(dead_code)]
pub fn progress_to_json(pt: &ProgressTracker) -> String {
    format!(
        "{{\"current\":{},\"total\":{},\"percent\":{}}}",
        pt.current,
        pt.total,
        progress_percent(pt)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let pt = new_progress_tracker(100.0);
        assert_eq!(get_progress(&pt), 0.0);
    }

    #[test]
    fn test_set() {
        let mut pt = new_progress_tracker(100.0);
        set_progress(&mut pt, 50.0);
        assert_eq!(get_progress(&pt), 50.0);
    }

    #[test]
    fn test_percent() {
        let mut pt = new_progress_tracker(200.0);
        set_progress(&mut pt, 100.0);
        assert!((progress_percent(&pt) - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_complete() {
        let mut pt = new_progress_tracker(10.0);
        set_progress(&mut pt, 10.0);
        assert!(is_complete(&pt));
    }

    #[test]
    fn test_not_complete() {
        let pt = new_progress_tracker(10.0);
        assert!(!is_complete(&pt));
    }

    #[test]
    fn test_reset() {
        let mut pt = new_progress_tracker(50.0);
        set_progress(&mut pt, 25.0);
        progress_reset(&mut pt);
        assert_eq!(get_progress(&pt), 0.0);
    }

    #[test]
    fn test_increment() {
        let mut pt = new_progress_tracker(100.0);
        progress_increment(&mut pt, 30.0);
        progress_increment(&mut pt, 20.0);
        assert_eq!(get_progress(&pt), 50.0);
    }

    #[test]
    fn test_increment_clamp() {
        let mut pt = new_progress_tracker(10.0);
        progress_increment(&mut pt, 15.0);
        assert_eq!(get_progress(&pt), 10.0);
    }

    #[test]
    fn test_to_json() {
        let pt = new_progress_tracker(100.0);
        let j = progress_to_json(&pt);
        assert!(j.contains("\"current\":"));
        assert!(j.contains("\"total\":"));
    }

    #[test]
    fn test_negative_total() {
        let pt = new_progress_tracker(-5.0);
        assert_eq!(pt.total, 1.0);
    }
}
