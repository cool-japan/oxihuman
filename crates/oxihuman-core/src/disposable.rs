#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Disposable {
    disposed: bool,
    callbacks: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DisposeGuard {
    active: bool,
    id: u32,
}

static mut GUARD_COUNTER: u32 = 0;
static mut DISPOSED_COUNTER: u32 = 0;

#[allow(dead_code)]
pub fn new_dispose_guard() -> DisposeGuard {
    let id = unsafe {
        GUARD_COUNTER += 1;
        GUARD_COUNTER
    };
    DisposeGuard { active: true, id }
}

#[allow(dead_code)]
pub fn dispose(d: &mut Disposable) {
    if !d.disposed {
        d.disposed = true;
        unsafe {
            DISPOSED_COUNTER += 1;
        }
    }
}

#[allow(dead_code)]
pub fn is_disposed(d: &Disposable) -> bool {
    d.disposed
}

#[allow(dead_code)]
pub fn on_dispose(d: &mut Disposable, callback_name: &str) {
    d.callbacks.push(callback_name.to_string());
}

#[allow(dead_code)]
pub fn guard_count() -> u32 {
    unsafe { GUARD_COUNTER }
}

#[allow(dead_code)]
pub fn disposed_count() -> u32 {
    unsafe { DISPOSED_COUNTER }
}

#[allow(dead_code)]
pub fn clear_disposables(d: &mut Disposable) {
    d.callbacks.clear();
    d.disposed = false;
}

#[allow(dead_code)]
pub fn guard_is_active(g: &DisposeGuard) -> bool {
    g.active
}

#[allow(dead_code)]
fn new_disposable() -> Disposable {
    Disposable {
        disposed: false,
        callbacks: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_disposable() {
        let d = new_disposable();
        assert!(!is_disposed(&d));
    }

    #[test]
    fn test_dispose() {
        let mut d = new_disposable();
        dispose(&mut d);
        assert!(is_disposed(&d));
    }

    #[test]
    fn test_double_dispose() {
        let mut d = new_disposable();
        dispose(&mut d);
        dispose(&mut d);
        assert!(is_disposed(&d));
    }

    #[test]
    fn test_on_dispose() {
        let mut d = new_disposable();
        on_dispose(&mut d, "cleanup");
        assert_eq!(d.callbacks.len(), 1);
    }

    #[test]
    fn test_clear_disposables() {
        let mut d = new_disposable();
        on_dispose(&mut d, "cb1");
        dispose(&mut d);
        clear_disposables(&mut d);
        assert!(!is_disposed(&d));
        assert!(d.callbacks.is_empty());
    }

    #[test]
    fn test_new_guard() {
        let g = new_dispose_guard();
        assert!(guard_is_active(&g));
    }

    #[test]
    fn test_guard_id() {
        let g = new_dispose_guard();
        assert!(g.id > 0);
    }

    #[test]
    fn test_guard_active() {
        let mut g = new_dispose_guard();
        g.active = false;
        assert!(!guard_is_active(&g));
    }

    #[test]
    fn test_multiple_callbacks() {
        let mut d = new_disposable();
        on_dispose(&mut d, "a");
        on_dispose(&mut d, "b");
        on_dispose(&mut d, "c");
        assert_eq!(d.callbacks.len(), 3);
    }

    #[test]
    fn test_guard_count_increments() {
        let before = guard_count();
        let _g = new_dispose_guard();
        assert!(guard_count() > before);
    }
}
