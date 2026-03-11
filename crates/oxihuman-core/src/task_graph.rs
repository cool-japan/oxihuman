// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Async-style task dependency graph scheduler (synchronous execution, DAG-based ordering).

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    Ready,
    Running,
    Completed,
    Failed(String),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Task {
    pub id: usize,
    pub name: String,
    pub dependencies: Vec<usize>,
    pub status: TaskStatus,
    pub result: Option<String>,
    pub priority: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TaskGraph {
    pub tasks: Vec<Task>,
    pub next_id: usize,
}

/// Create an empty task graph.
#[allow(dead_code)]
pub fn new_task_graph() -> TaskGraph {
    TaskGraph {
        tasks: Vec::new(),
        next_id: 0,
    }
}

/// Add a task to the graph and return its assigned ID.
#[allow(dead_code)]
pub fn add_task(graph: &mut TaskGraph, name: &str, deps: Vec<usize>, priority: u32) -> usize {
    let id = graph.next_id;
    graph.next_id += 1;
    graph.tasks.push(Task {
        id,
        name: name.to_string(),
        dependencies: deps,
        status: TaskStatus::Pending,
        result: None,
        priority,
    });
    id
}

/// Return IDs of tasks whose dependencies are all Completed and whose own status is Pending.
#[allow(dead_code)]
pub fn get_ready_tasks(graph: &TaskGraph) -> Vec<usize> {
    let completed_ids: std::collections::HashSet<usize> = graph
        .tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Completed)
        .map(|t| t.id)
        .collect();

    graph
        .tasks
        .iter()
        .filter(|t| {
            t.status == TaskStatus::Pending
                && t.dependencies.iter().all(|dep| completed_ids.contains(dep))
        })
        .map(|t| t.id)
        .collect()
}

/// Mark a task as Completed with an optional result value.
#[allow(dead_code)]
pub fn mark_complete(graph: &mut TaskGraph, id: usize, result: Option<String>) {
    if let Some(task) = graph.tasks.iter_mut().find(|t| t.id == id) {
        task.status = TaskStatus::Completed;
        task.result = result;
    }
}

/// Mark a task as Failed with an error message.
#[allow(dead_code)]
pub fn mark_failed(graph: &mut TaskGraph, id: usize, error: String) {
    if let Some(task) = graph.tasks.iter_mut().find(|t| t.id == id) {
        task.status = TaskStatus::Failed(error);
    }
}

/// Compute a topological order using Kahn's algorithm. Returns Err if a cycle is detected.
#[allow(dead_code)]
pub fn topological_order(graph: &TaskGraph) -> Result<Vec<usize>, String> {
    let n = graph.tasks.len();
    if n == 0 {
        return Ok(vec![]);
    }

    // Build id → index map
    let id_to_idx: std::collections::HashMap<usize, usize> = graph
        .tasks
        .iter()
        .enumerate()
        .map(|(i, t)| (t.id, i))
        .collect();

    let mut in_degree = vec![0usize; n];
    let mut adj: Vec<Vec<usize>> = vec![vec![]; n]; // adj[i] = list of indices that depend on i

    for (idx, task) in graph.tasks.iter().enumerate() {
        for &dep_id in &task.dependencies {
            if let Some(&dep_idx) = id_to_idx.get(&dep_id) {
                adj[dep_idx].push(idx);
                in_degree[idx] += 1;
            }
        }
    }

    let mut queue: std::collections::VecDeque<usize> = in_degree
        .iter()
        .enumerate()
        .filter(|(_, &d)| d == 0)
        .map(|(i, _)| i)
        .collect();

    let mut order = Vec::with_capacity(n);

    while let Some(idx) = queue.pop_front() {
        order.push(graph.tasks[idx].id);
        for &neighbor in &adj[idx] {
            in_degree[neighbor] -= 1;
            if in_degree[neighbor] == 0 {
                queue.push_back(neighbor);
            }
        }
    }

    if order.len() != n {
        Err("Cycle detected in task graph".to_string())
    } else {
        Ok(order)
    }
}

/// Execute tasks sequentially in topological order, calling `executor` for each.
#[allow(dead_code)]
pub fn execute_sequential(
    graph: &mut TaskGraph,
    executor: &mut dyn FnMut(usize, &str) -> Result<String, String>,
) {
    let order = match topological_order(graph) {
        Ok(o) => o,
        Err(e) => {
            // Mark all as failed
            for task in &mut graph.tasks {
                task.status = TaskStatus::Failed(e.clone());
            }
            return;
        }
    };

    for id in order {
        let (name, deps_ok) = {
            let task = match graph.tasks.iter().find(|t| t.id == id) {
                Some(t) => t,
                None => continue,
            };
            let deps_ok = task.dependencies.iter().all(|dep_id| {
                graph
                    .tasks
                    .iter()
                    .find(|t| t.id == *dep_id)
                    .map(|t| t.status == TaskStatus::Completed)
                    .unwrap_or(false)
            });
            (task.name.clone(), deps_ok)
        };

        if !deps_ok {
            mark_failed(graph, id, "dependency failed".to_string());
            continue;
        }

        if let Some(task) = graph.tasks.iter_mut().find(|t| t.id == id) {
            task.status = TaskStatus::Running;
        }

        match executor(id, &name) {
            Ok(result) => mark_complete(graph, id, Some(result)),
            Err(err) => mark_failed(graph, id, err),
        }
    }
}

/// Total number of tasks in the graph.
#[allow(dead_code)]
pub fn task_count(graph: &TaskGraph) -> usize {
    graph.tasks.len()
}

/// Number of completed tasks.
#[allow(dead_code)]
pub fn completed_count(graph: &TaskGraph) -> usize {
    graph
        .tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Completed)
        .count()
}

/// Number of failed tasks.
#[allow(dead_code)]
pub fn failed_count(graph: &TaskGraph) -> usize {
    graph
        .tasks
        .iter()
        .filter(|t| matches!(t.status, TaskStatus::Failed(_)))
        .count()
}

/// Number of pending tasks.
#[allow(dead_code)]
pub fn pending_count(graph: &TaskGraph) -> usize {
    graph
        .tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Pending)
        .count()
}

/// Reset all tasks to Pending status.
#[allow(dead_code)]
pub fn reset_graph(graph: &mut TaskGraph) {
    for task in &mut graph.tasks {
        task.status = TaskStatus::Pending;
        task.result = None;
    }
}

/// Serialize the graph to a JSON string.
#[allow(dead_code)]
pub fn graph_to_json(graph: &TaskGraph) -> String {
    let tasks_json: Vec<String> = graph
        .tasks
        .iter()
        .map(|t| {
            let status = match &t.status {
                TaskStatus::Pending => "\"Pending\"".to_string(),
                TaskStatus::Ready => "\"Ready\"".to_string(),
                TaskStatus::Running => "\"Running\"".to_string(),
                TaskStatus::Completed => "\"Completed\"".to_string(),
                TaskStatus::Failed(e) => format!("{{\"Failed\":{e:?}}}"),
            };
            let deps: Vec<String> = t.dependencies.iter().map(|d| d.to_string()).collect();
            let result = match &t.result {
                Some(r) => format!("{r:?}"),
                None => "null".to_string(),
            };
            format!(
                r#"{{"id":{},"name":{:?},"dependencies":[{}],"status":{},"result":{},"priority":{}}}"#,
                t.id,
                t.name,
                deps.join(","),
                status,
                result,
                t.priority,
            )
        })
        .collect();

    format!(
        r#"{{"tasks":[{}],"next_id":{}}}"#,
        tasks_json.join(","),
        graph.next_id
    )
}

/// Return the length of the critical (longest dependency chain) path.
#[allow(dead_code)]
pub fn critical_path_length(graph: &TaskGraph) -> usize {
    let id_to_idx: std::collections::HashMap<usize, usize> = graph
        .tasks
        .iter()
        .enumerate()
        .map(|(i, t)| (t.id, i))
        .collect();

    let order = match topological_order(graph) {
        Ok(o) => o,
        Err(_) => return 0,
    };

    let mut depth = vec![0usize; graph.tasks.len()];

    for id in &order {
        if let Some(&idx) = id_to_idx.get(id) {
            let task = &graph.tasks[idx];
            let max_dep_depth = task
                .dependencies
                .iter()
                .filter_map(|dep_id| id_to_idx.get(dep_id))
                .map(|&di| depth[di])
                .max()
                .unwrap_or(0);
            depth[idx] = max_dep_depth + 1;
        }
    }

    depth.into_iter().max().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_task_graph_empty() {
        let g = new_task_graph();
        assert_eq!(task_count(&g), 0);
    }

    #[test]
    fn test_add_task_returns_incremental_ids() {
        let mut g = new_task_graph();
        let a = add_task(&mut g, "A", vec![], 0);
        let b = add_task(&mut g, "B", vec![], 0);
        assert_eq!(a, 0);
        assert_eq!(b, 1);
        assert_eq!(task_count(&g), 2);
    }

    #[test]
    fn test_get_ready_tasks_no_deps() {
        let mut g = new_task_graph();
        add_task(&mut g, "A", vec![], 0);
        add_task(&mut g, "B", vec![], 0);
        let ready = get_ready_tasks(&g);
        assert_eq!(ready.len(), 2);
    }

    #[test]
    fn test_get_ready_tasks_with_deps() {
        let mut g = new_task_graph();
        let a = add_task(&mut g, "A", vec![], 0);
        add_task(&mut g, "B", vec![a], 0);
        let ready = get_ready_tasks(&g);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0], a);
    }

    #[test]
    fn test_mark_complete() {
        let mut g = new_task_graph();
        let id = add_task(&mut g, "A", vec![], 0);
        mark_complete(&mut g, id, Some("done".to_string()));
        assert_eq!(completed_count(&g), 1);
    }

    #[test]
    fn test_mark_failed() {
        let mut g = new_task_graph();
        let id = add_task(&mut g, "A", vec![], 0);
        mark_failed(&mut g, id, "oops".to_string());
        assert_eq!(failed_count(&g), 1);
    }

    #[test]
    fn test_topological_order_linear() {
        let mut g = new_task_graph();
        let a = add_task(&mut g, "A", vec![], 0);
        let b = add_task(&mut g, "B", vec![a], 0);
        let c = add_task(&mut g, "C", vec![b], 0);
        let order = topological_order(&g).unwrap();
        assert_eq!(order, vec![a, b, c]);
    }

    #[test]
    fn test_topological_order_cycle_detected() {
        let mut g = new_task_graph();
        let a = add_task(&mut g, "A", vec![], 0);
        let b = add_task(&mut g, "B", vec![a], 0);
        // Manually create a cycle: A depends on B
        g.tasks[0].dependencies.push(b);
        let result = topological_order(&g);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_sequential_all_complete() {
        let mut g = new_task_graph();
        let a = add_task(&mut g, "A", vec![], 0);
        let _b = add_task(&mut g, "B", vec![a], 0);
        execute_sequential(&mut g, &mut |_, name| Ok(format!("done:{name}")));
        assert_eq!(completed_count(&g), 2);
    }

    #[test]
    fn test_execute_sequential_failure_propagates() {
        let mut g = new_task_graph();
        let a = add_task(&mut g, "A", vec![], 0);
        add_task(&mut g, "B", vec![a], 0);
        execute_sequential(&mut g, &mut |id, _| {
            if id == 0 {
                Err("fail".to_string())
            } else {
                Ok("ok".to_string())
            }
        });
        assert_eq!(failed_count(&g), 2); // B fails due to dep
    }

    #[test]
    fn test_pending_count() {
        let mut g = new_task_graph();
        add_task(&mut g, "A", vec![], 0);
        add_task(&mut g, "B", vec![], 0);
        assert_eq!(pending_count(&g), 2);
    }

    #[test]
    fn test_reset_graph() {
        let mut g = new_task_graph();
        let id = add_task(&mut g, "A", vec![], 0);
        mark_complete(&mut g, id, Some("result".to_string()));
        reset_graph(&mut g);
        assert_eq!(pending_count(&g), 1);
        assert_eq!(completed_count(&g), 0);
    }

    #[test]
    fn test_graph_to_json_contains_task_name() {
        let mut g = new_task_graph();
        add_task(&mut g, "MyTask", vec![], 5);
        let json = graph_to_json(&g);
        assert!(json.contains("MyTask"));
        assert!(json.contains("priority"));
    }

    #[test]
    fn test_critical_path_length_linear() {
        let mut g = new_task_graph();
        let a = add_task(&mut g, "A", vec![], 0);
        let b = add_task(&mut g, "B", vec![a], 0);
        add_task(&mut g, "C", vec![b], 0);
        assert_eq!(critical_path_length(&g), 3);
    }

    #[test]
    fn test_critical_path_length_empty() {
        let g = new_task_graph();
        assert_eq!(critical_path_length(&g), 0);
    }

    #[test]
    fn test_critical_path_branching() {
        let mut g = new_task_graph();
        let a = add_task(&mut g, "A", vec![], 0);
        let b = add_task(&mut g, "B", vec![a], 0);
        add_task(&mut g, "C", vec![a], 0); // shorter branch
        add_task(&mut g, "D", vec![b], 0); // longer branch
                                           // Longest: A -> B -> D = 3
        assert_eq!(critical_path_length(&g), 3);
    }
}
