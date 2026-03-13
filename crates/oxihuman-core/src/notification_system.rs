//! In-app notification system with severity and dismissal.

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum NotificationSeverity {
    Info,
    Warning,
    Error,
    Success,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Notification {
    pub id: u64,
    pub title: String,
    pub message: String,
    pub severity: NotificationSeverity,
    pub timestamp: f64,
    pub dismissed: bool,
    pub persistent: bool,
    pub duration_secs: f32,
}

#[allow(dead_code)]
pub struct NotificationSystem {
    pub notifications: Vec<Notification>,
    pub next_id: u64,
    pub current_time: f64,
    pub max_notifications: usize,
}

#[allow(dead_code)]
pub fn new_notification_system(max: usize) -> NotificationSystem {
    NotificationSystem {
        notifications: Vec::new(),
        next_id: 1,
        current_time: 0.0,
        max_notifications: max,
    }
}

#[allow(dead_code)]
pub fn push_notification(
    sys: &mut NotificationSystem,
    title: &str,
    msg: &str,
    severity: NotificationSeverity,
    duration: f32,
) -> u64 {
    let id = sys.next_id;
    sys.next_id += 1;
    let persistent = duration <= 0.0;
    sys.notifications.push(Notification {
        id,
        title: title.to_string(),
        message: msg.to_string(),
        severity,
        timestamp: sys.current_time,
        dismissed: false,
        persistent,
        duration_secs: duration,
    });
    // Trim to max if needed — remove oldest dismissed first, then oldest overall
    if sys.notifications.len() > sys.max_notifications {
        let excess = sys.notifications.len() - sys.max_notifications;
        let mut removed = 0;
        sys.notifications.retain(|n| {
            if removed < excess && n.dismissed {
                removed += 1;
                false
            } else {
                true
            }
        });
        if sys.notifications.len() > sys.max_notifications {
            let still_excess = sys.notifications.len() - sys.max_notifications;
            sys.notifications.drain(0..still_excess);
        }
    }
    id
}

#[allow(dead_code)]
pub fn dismiss_notification(sys: &mut NotificationSystem, id: u64) -> bool {
    if let Some(n) = sys.notifications.iter_mut().find(|n| n.id == id) {
        if !n.dismissed {
            n.dismissed = true;
            return true;
        }
    }
    false
}

#[allow(dead_code)]
pub fn advance_notifications(sys: &mut NotificationSystem, dt: f64) {
    sys.current_time += dt;
    for n in &mut sys.notifications {
        if !n.dismissed && !n.persistent {
            let elapsed = sys.current_time - n.timestamp;
            if elapsed >= n.duration_secs as f64 {
                n.dismissed = true;
            }
        }
    }
    // Remove old dismissed notifications if over max
    while sys.notifications.len() > sys.max_notifications {
        let pos = sys.notifications.iter().position(|n| n.dismissed);
        match pos {
            Some(i) => {
                sys.notifications.remove(i);
            }
            None => break,
        }
    }
}

#[allow(dead_code)]
pub fn active_notifications(sys: &NotificationSystem) -> Vec<&Notification> {
    sys.notifications.iter().filter(|n| !n.dismissed).collect()
}

#[allow(dead_code)]
pub fn notification_by_id(sys: &NotificationSystem, id: u64) -> Option<&Notification> {
    sys.notifications.iter().find(|n| n.id == id)
}

#[allow(dead_code)]
pub fn notification_count(sys: &NotificationSystem) -> usize {
    sys.notifications.len()
}

#[allow(dead_code)]
pub fn active_count(sys: &NotificationSystem) -> usize {
    sys.notifications.iter().filter(|n| !n.dismissed).count()
}

#[allow(dead_code)]
pub fn notifications_by_severity(
    sys: &NotificationSystem,
    sev: NotificationSeverity,
) -> Vec<&Notification> {
    sys.notifications
        .iter()
        .filter(|n| n.severity == sev)
        .collect()
}

#[allow(dead_code)]
pub fn clear_all_notifications(sys: &mut NotificationSystem) {
    sys.notifications.clear();
}

#[allow(dead_code)]
pub fn has_errors(sys: &NotificationSystem) -> bool {
    sys.notifications
        .iter()
        .any(|n| !n.dismissed && n.severity == NotificationSeverity::Error)
}

#[allow(dead_code)]
pub fn push_info(sys: &mut NotificationSystem, title: &str, msg: &str) -> u64 {
    push_notification(sys, title, msg, NotificationSeverity::Info, 5.0)
}

#[allow(dead_code)]
pub fn push_error(sys: &mut NotificationSystem, title: &str, msg: &str) -> u64 {
    push_notification(sys, title, msg, NotificationSeverity::Error, 10.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_notification_system() {
        let sys = new_notification_system(10);
        assert_eq!(notification_count(&sys), 0);
        assert_eq!(sys.max_notifications, 10);
        assert_eq!(sys.next_id, 1);
    }

    #[test]
    fn test_push_notification() {
        let mut sys = new_notification_system(10);
        let id = push_notification(&mut sys, "Test", "Message", NotificationSeverity::Info, 5.0);
        assert_eq!(id, 1);
        assert_eq!(notification_count(&sys), 1);
    }

    #[test]
    fn test_dismiss_notification() {
        let mut sys = new_notification_system(10);
        let id = push_notification(&mut sys, "Test", "Msg", NotificationSeverity::Info, 5.0);
        assert!(dismiss_notification(&mut sys, id));
        assert!(!dismiss_notification(&mut sys, id));
        assert_eq!(active_count(&sys), 0);
    }

    #[test]
    fn test_dismiss_nonexistent() {
        let mut sys = new_notification_system(10);
        assert!(!dismiss_notification(&mut sys, 999));
    }

    #[test]
    fn test_advance_auto_dismisses() {
        let mut sys = new_notification_system(10);
        push_notification(&mut sys, "T", "M", NotificationSeverity::Info, 3.0);
        assert_eq!(active_count(&sys), 1);
        advance_notifications(&mut sys, 4.0);
        assert_eq!(active_count(&sys), 0);
    }

    #[test]
    fn test_advance_persistent_not_dismissed() {
        let mut sys = new_notification_system(10);
        push_notification(&mut sys, "T", "M", NotificationSeverity::Warning, 0.0);
        advance_notifications(&mut sys, 100.0);
        assert_eq!(active_count(&sys), 1);
    }

    #[test]
    fn test_active_notifications() {
        let mut sys = new_notification_system(10);
        let id1 = push_notification(&mut sys, "A", "M1", NotificationSeverity::Info, 5.0);
        push_notification(&mut sys, "B", "M2", NotificationSeverity::Warning, 5.0);
        dismiss_notification(&mut sys, id1);
        let active = active_notifications(&sys);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].title, "B");
    }

    #[test]
    fn test_notification_by_id() {
        let mut sys = new_notification_system(10);
        let id = push_notification(&mut sys, "Title", "Msg", NotificationSeverity::Info, 5.0);
        let n = notification_by_id(&sys, id);
        assert!(n.is_some());
        assert_eq!(n.expect("should succeed").title, "Title");
        assert!(notification_by_id(&sys, 999).is_none());
    }

    #[test]
    fn test_notifications_by_severity() {
        let mut sys = new_notification_system(10);
        push_notification(&mut sys, "E1", "M", NotificationSeverity::Error, 5.0);
        push_notification(&mut sys, "E2", "M", NotificationSeverity::Error, 5.0);
        push_notification(&mut sys, "I1", "M", NotificationSeverity::Info, 5.0);
        let errors = notifications_by_severity(&sys, NotificationSeverity::Error);
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_has_errors_true() {
        let mut sys = new_notification_system(10);
        push_error(&mut sys, "Error", "Something broke");
        assert!(has_errors(&sys));
    }

    #[test]
    fn test_has_errors_false() {
        let mut sys = new_notification_system(10);
        push_info(&mut sys, "Info", "All good");
        assert!(!has_errors(&sys));
    }

    #[test]
    fn test_has_errors_dismissed() {
        let mut sys = new_notification_system(10);
        let id = push_error(&mut sys, "Error", "Bad");
        dismiss_notification(&mut sys, id);
        assert!(!has_errors(&sys));
    }

    #[test]
    fn test_clear_all_notifications() {
        let mut sys = new_notification_system(10);
        push_info(&mut sys, "A", "M1");
        push_info(&mut sys, "B", "M2");
        clear_all_notifications(&mut sys);
        assert_eq!(notification_count(&sys), 0);
    }

    #[test]
    fn test_push_info() {
        let mut sys = new_notification_system(10);
        let id = push_info(&mut sys, "Hello", "World");
        let n = notification_by_id(&sys, id).expect("should succeed");
        assert_eq!(n.severity, NotificationSeverity::Info);
        assert!((n.duration_secs - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_push_error() {
        let mut sys = new_notification_system(10);
        let id = push_error(&mut sys, "Oops", "Failed");
        let n = notification_by_id(&sys, id).expect("should succeed");
        assert_eq!(n.severity, NotificationSeverity::Error);
        assert!((n.duration_secs - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_notification_count() {
        let mut sys = new_notification_system(10);
        assert_eq!(notification_count(&sys), 0);
        push_info(&mut sys, "A", "M");
        push_info(&mut sys, "B", "M");
        assert_eq!(notification_count(&sys), 2);
    }

    #[test]
    fn test_active_count() {
        let mut sys = new_notification_system(10);
        let id = push_info(&mut sys, "A", "M");
        push_info(&mut sys, "B", "M");
        assert_eq!(active_count(&sys), 2);
        dismiss_notification(&mut sys, id);
        assert_eq!(active_count(&sys), 1);
    }
}
