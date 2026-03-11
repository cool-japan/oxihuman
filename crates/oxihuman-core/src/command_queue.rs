//! Command queue for deferred execution with priority levels.

/// Priority levels for queued commands.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CommandPriority {
    /// Must execute immediately / first.
    Critical,
    /// High importance.
    High,
    /// Default importance.
    Normal,
    /// Background / best-effort.
    Low,
}

/// A single command waiting in the queue.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct QueuedCommand {
    /// Unique id assigned at enqueue time.
    pub id: u64,
    /// Human-readable label.
    pub label: String,
    /// Priority tier.
    pub priority: CommandPriority,
    /// Monotonic sequence number for FIFO within same priority.
    pub sequence: u64,
}

/// A priority-aware command queue.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct CommandQueue {
    /// Queued commands, kept sorted by (priority, sequence).
    commands: Vec<QueuedCommand>,
    /// Running counter for unique ids.
    next_id: u64,
    /// Running counter for insertion order.
    next_seq: u64,
    /// Total number of commands ever enqueued.
    total_enqueued: u64,
    /// Maximum number of commands the queue has ever held.
    max_depth: usize,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Map a `CommandPriority` to a sort key (lower = higher priority).
#[allow(dead_code)]
fn priority_rank(p: &CommandPriority) -> u8 {
    match p {
        CommandPriority::Critical => 0,
        CommandPriority::High => 1,
        CommandPriority::Normal => 2,
        CommandPriority::Low => 3,
    }
}

/// Sort the internal list by (priority_rank, sequence).
#[allow(dead_code)]
fn sort_commands(cmds: &mut [QueuedCommand]) {
    cmds.sort_by(|a, b| {
        let pa = priority_rank(&a.priority);
        let pb = priority_rank(&b.priority);
        pa.cmp(&pb).then(a.sequence.cmp(&b.sequence))
    });
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

/// Create a new, empty `CommandQueue`.
#[allow(dead_code)]
pub fn new_command_queue() -> CommandQueue {
    CommandQueue {
        commands: Vec::new(),
        next_id: 1,
        next_seq: 0,
        total_enqueued: 0,
        max_depth: 0,
    }
}

// ---------------------------------------------------------------------------
// Enqueue / Dequeue
// ---------------------------------------------------------------------------

/// Enqueue a single command with the given label and priority.
/// Returns the assigned command id.
#[allow(dead_code)]
pub fn enqueue(queue: &mut CommandQueue, label: &str, priority: CommandPriority) -> u64 {
    let id = queue.next_id;
    queue.next_id += 1;
    let seq = queue.next_seq;
    queue.next_seq += 1;
    queue.total_enqueued += 1;
    queue.commands.push(QueuedCommand {
        id,
        label: label.to_string(),
        priority,
        sequence: seq,
    });
    sort_commands(&mut queue.commands);
    if queue.commands.len() > queue.max_depth {
        queue.max_depth = queue.commands.len();
    }
    id
}

/// Remove and return the highest-priority (lowest rank) command.
/// Returns `None` if the queue is empty.
#[allow(dead_code)]
pub fn dequeue(queue: &mut CommandQueue) -> Option<QueuedCommand> {
    if queue.commands.is_empty() {
        None
    } else {
        Some(queue.commands.remove(0))
    }
}

/// Peek at the next command without removing it.
#[allow(dead_code)]
pub fn peek_next(queue: &CommandQueue) -> Option<&QueuedCommand> {
    queue.commands.first()
}

// ---------------------------------------------------------------------------
// Queries
// ---------------------------------------------------------------------------

/// Return the number of commands currently in the queue.
#[allow(dead_code)]
pub fn command_count(queue: &CommandQueue) -> usize {
    queue.commands.len()
}

/// Return `true` if the queue has no commands.
#[allow(dead_code)]
pub fn is_queue_empty(queue: &CommandQueue) -> bool {
    queue.commands.is_empty()
}

/// Return `true` if there is at least one command of the given priority.
#[allow(dead_code)]
pub fn has_priority(queue: &CommandQueue, priority: &CommandPriority) -> bool {
    queue.commands.iter().any(|c| &c.priority == priority)
}

/// Return the total number of commands ever enqueued.
#[allow(dead_code)]
pub fn total_enqueued(queue: &CommandQueue) -> u64 {
    queue.total_enqueued
}

/// Return the historical maximum queue depth.
#[allow(dead_code)]
pub fn max_queue_depth(queue: &CommandQueue) -> usize {
    queue.max_depth
}

// ---------------------------------------------------------------------------
// Batch / bulk operations
// ---------------------------------------------------------------------------

/// Remove all commands from the queue.
#[allow(dead_code)]
pub fn clear_queue(queue: &mut CommandQueue) {
    queue.commands.clear();
}

/// Remove and return all commands, ordered by priority then sequence.
#[allow(dead_code)]
pub fn drain_all(queue: &mut CommandQueue) -> Vec<QueuedCommand> {
    let mut out = std::mem::take(&mut queue.commands);
    sort_commands(&mut out);
    out
}

/// Enqueue a batch of `(label, priority)` pairs. Returns the assigned ids.
#[allow(dead_code)]
pub fn enqueue_batch(queue: &mut CommandQueue, items: &[(&str, CommandPriority)]) -> Vec<u64> {
    let mut ids = Vec::with_capacity(items.len());
    for (label, pri) in items {
        ids.push(enqueue(queue, label, pri.clone()));
    }
    ids
}

/// Collect references to all commands of a given priority.
#[allow(dead_code)]
pub fn commands_by_priority<'a>(
    queue: &'a CommandQueue,
    priority: &CommandPriority,
) -> Vec<&'a QueuedCommand> {
    queue
        .commands
        .iter()
        .filter(|c| &c.priority == priority)
        .collect()
}

// ---------------------------------------------------------------------------
// Serialization
// ---------------------------------------------------------------------------

/// Produce a minimal JSON representation of the queue.
#[allow(dead_code)]
pub fn command_queue_to_json(queue: &CommandQueue) -> String {
    let mut s = String::from("{\"commands\":[");
    for (i, c) in queue.commands.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"id\":{},\"label\":\"{}\",\"priority\":\"{:?}\"}}",
            c.id, c.label, c.priority
        ));
    }
    s.push_str(&format!(
        "],\"total_enqueued\":{},\"max_depth\":{}}}",
        queue.total_enqueued, queue.max_depth
    ));
    s
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_command_queue() {
        let q = new_command_queue();
        assert!(is_queue_empty(&q));
        assert_eq!(command_count(&q), 0);
    }

    #[test]
    fn test_enqueue_single() {
        let mut q = new_command_queue();
        let id = enqueue(&mut q, "cmd1", CommandPriority::Normal);
        assert!(id > 0);
        assert_eq!(command_count(&q), 1);
    }

    #[test]
    fn test_dequeue_fifo() {
        let mut q = new_command_queue();
        enqueue(&mut q, "first", CommandPriority::Normal);
        enqueue(&mut q, "second", CommandPriority::Normal);
        let c = dequeue(&mut q).unwrap();
        assert_eq!(c.label, "first");
    }

    #[test]
    fn test_dequeue_empty() {
        let mut q = new_command_queue();
        assert!(dequeue(&mut q).is_none());
    }

    #[test]
    fn test_priority_ordering() {
        let mut q = new_command_queue();
        enqueue(&mut q, "low", CommandPriority::Low);
        enqueue(&mut q, "critical", CommandPriority::Critical);
        enqueue(&mut q, "normal", CommandPriority::Normal);
        let c = dequeue(&mut q).unwrap();
        assert_eq!(c.label, "critical");
    }

    #[test]
    fn test_peek_next() {
        let mut q = new_command_queue();
        assert!(peek_next(&q).is_none());
        enqueue(&mut q, "peek_me", CommandPriority::High);
        let p = peek_next(&q).unwrap();
        assert_eq!(p.label, "peek_me");
        assert_eq!(command_count(&q), 1); // not removed
    }

    #[test]
    fn test_clear_queue() {
        let mut q = new_command_queue();
        enqueue(&mut q, "a", CommandPriority::Normal);
        enqueue(&mut q, "b", CommandPriority::High);
        clear_queue(&mut q);
        assert!(is_queue_empty(&q));
    }

    #[test]
    fn test_drain_all() {
        let mut q = new_command_queue();
        enqueue(&mut q, "a", CommandPriority::Low);
        enqueue(&mut q, "b", CommandPriority::Critical);
        let drained = drain_all(&mut q);
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].label, "b"); // critical first
        assert!(is_queue_empty(&q));
    }

    #[test]
    fn test_enqueue_batch() {
        let mut q = new_command_queue();
        let ids = enqueue_batch(
            &mut q,
            &[
                ("x", CommandPriority::Normal),
                ("y", CommandPriority::High),
                ("z", CommandPriority::Low),
            ],
        );
        assert_eq!(ids.len(), 3);
        assert_eq!(command_count(&q), 3);
    }

    #[test]
    fn test_commands_by_priority() {
        let mut q = new_command_queue();
        enqueue(&mut q, "a", CommandPriority::Normal);
        enqueue(&mut q, "b", CommandPriority::High);
        enqueue(&mut q, "c", CommandPriority::Normal);
        let normals = commands_by_priority(&q, &CommandPriority::Normal);
        assert_eq!(normals.len(), 2);
    }

    #[test]
    fn test_total_enqueued() {
        let mut q = new_command_queue();
        enqueue(&mut q, "a", CommandPriority::Normal);
        enqueue(&mut q, "b", CommandPriority::Normal);
        dequeue(&mut q);
        assert_eq!(total_enqueued(&q), 2);
    }

    #[test]
    fn test_is_queue_empty() {
        let mut q = new_command_queue();
        assert!(is_queue_empty(&q));
        enqueue(&mut q, "x", CommandPriority::Low);
        assert!(!is_queue_empty(&q));
    }

    #[test]
    fn test_has_priority() {
        let mut q = new_command_queue();
        enqueue(&mut q, "a", CommandPriority::High);
        assert!(has_priority(&q, &CommandPriority::High));
        assert!(!has_priority(&q, &CommandPriority::Low));
    }

    #[test]
    fn test_max_queue_depth() {
        let mut q = new_command_queue();
        enqueue(&mut q, "a", CommandPriority::Normal);
        enqueue(&mut q, "b", CommandPriority::Normal);
        enqueue(&mut q, "c", CommandPriority::Normal);
        dequeue(&mut q);
        assert_eq!(max_queue_depth(&q), 3);
    }

    #[test]
    fn test_command_queue_to_json() {
        let mut q = new_command_queue();
        enqueue(&mut q, "test", CommandPriority::Normal);
        let json = command_queue_to_json(&q);
        assert!(json.contains("\"commands\""));
        assert!(json.contains("\"test\""));
        assert!(json.contains("\"total_enqueued\":1"));
    }

    #[test]
    fn test_priority_stable_fifo() {
        let mut q = new_command_queue();
        enqueue(&mut q, "high1", CommandPriority::High);
        enqueue(&mut q, "high2", CommandPriority::High);
        enqueue(&mut q, "high3", CommandPriority::High);
        let c1 = dequeue(&mut q).unwrap();
        let c2 = dequeue(&mut q).unwrap();
        assert_eq!(c1.label, "high1");
        assert_eq!(c2.label, "high2");
    }
}
