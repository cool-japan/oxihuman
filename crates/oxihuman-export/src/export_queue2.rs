//! Export task queue management.
#![allow(dead_code)]

/// A single export task.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ExportTask2 {
    pub format: String,
    pub path: String,
    pub priority: u32,
}

/// A queue of export tasks.
#[allow(dead_code)]
pub struct ExportQueue2 {
    pub tasks: std::collections::VecDeque<ExportTask2>,
}

/// Create a new, empty export queue.
#[allow(dead_code)]
pub fn new_export_queue2() -> ExportQueue2 {
    ExportQueue2 { tasks: std::collections::VecDeque::new() }
}

/// Enqueue a task.
#[allow(dead_code)]
pub fn enqueue_task2(queue: &mut ExportQueue2, task: ExportTask2) {
    queue.tasks.push_back(task);
}

/// Dequeue the next task.
#[allow(dead_code)]
pub fn dequeue_task2(queue: &mut ExportQueue2) -> Option<ExportTask2> {
    queue.tasks.pop_front()
}

/// Get queue length.
#[allow(dead_code)]
pub fn queue2_len(queue: &ExportQueue2) -> usize {
    queue.tasks.len()
}

/// Check if queue is empty.
#[allow(dead_code)]
pub fn queue2_is_empty(queue: &ExportQueue2) -> bool {
    queue.tasks.is_empty()
}

/// Clear all tasks.
#[allow(dead_code)]
pub fn clear_queue2(queue: &mut ExportQueue2) {
    queue.tasks.clear();
}

/// Get the format of a task.
#[allow(dead_code)]
pub fn task2_format(task: &ExportTask2) -> &str {
    &task.format
}

/// Get the path of a task.
#[allow(dead_code)]
pub fn task2_path(task: &ExportTask2) -> &str {
    &task.path
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_task(fmt: &str, path: &str) -> ExportTask2 {
        ExportTask2 { format: fmt.to_string(), path: path.to_string(), priority: 0 }
    }

    #[test]
    fn test_new_queue_empty() {
        let q = new_export_queue2();
        assert!(queue2_is_empty(&q));
    }

    #[test]
    fn test_enqueue_task() {
        let mut q = new_export_queue2();
        enqueue_task2(&mut q, make_task("glb", "/out/a.glb"));
        assert_eq!(queue2_len(&q), 1);
    }

    #[test]
    fn test_dequeue_task() {
        let mut q = new_export_queue2();
        enqueue_task2(&mut q, make_task("obj", "/out/b.obj"));
        let t = dequeue_task2(&mut q).unwrap();
        assert_eq!(task2_format(&t), "obj");
    }

    #[test]
    fn test_dequeue_empty() {
        let mut q = new_export_queue2();
        assert!(dequeue_task2(&mut q).is_none());
    }

    #[test]
    fn test_queue_len() {
        let mut q = new_export_queue2();
        enqueue_task2(&mut q, make_task("stl", "/a.stl"));
        enqueue_task2(&mut q, make_task("ply", "/b.ply"));
        assert_eq!(queue2_len(&q), 2);
    }

    #[test]
    fn test_clear_queue() {
        let mut q = new_export_queue2();
        enqueue_task2(&mut q, make_task("glb", "/c.glb"));
        clear_queue2(&mut q);
        assert!(queue2_is_empty(&q));
    }

    #[test]
    fn test_task_path() {
        let t = make_task("fbx", "/out/mesh.fbx");
        assert_eq!(task2_path(&t), "/out/mesh.fbx");
    }

    #[test]
    fn test_fifo_order() {
        let mut q = new_export_queue2();
        enqueue_task2(&mut q, make_task("a", "/a"));
        enqueue_task2(&mut q, make_task("b", "/b"));
        let first = dequeue_task2(&mut q).unwrap();
        assert_eq!(first.format, "a");
    }

    #[test]
    fn test_task_priority() {
        let t = ExportTask2 { format: "glb".to_string(), path: "/x.glb".to_string(), priority: 5 };
        assert_eq!(t.priority, 5);
    }
}
