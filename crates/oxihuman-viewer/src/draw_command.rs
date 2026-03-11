// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Draw command recorder — records and sorts GPU draw calls for submission.

/// Draw primitive type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DrawPrimitive {
    Triangles,
    Lines,
    Points,
    TriangleStrip,
}

/// A single draw command.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct DrawCommand {
    pub vertex_offset: u32,
    pub index_offset: u32,
    pub index_count: u32,
    pub instance_count: u32,
    pub pipeline_id: u32,
    pub sort_key: u64,
    pub primitive: DrawPrimitive,
    pub enabled: bool,
}

impl Default for DrawCommand {
    fn default() -> Self {
        Self {
            vertex_offset: 0,
            index_offset: 0,
            index_count: 0,
            instance_count: 1,
            pipeline_id: 0,
            sort_key: 0,
            primitive: DrawPrimitive::Triangles,
            enabled: true,
        }
    }
}

/// Draw command recorder.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct DrawCommandRecorder {
    pub commands: Vec<DrawCommand>,
    pub max_commands: usize,
}

/// Create new recorder.
#[allow(dead_code)]
pub fn new_draw_recorder(max_commands: usize) -> DrawCommandRecorder {
    DrawCommandRecorder {
        commands: Vec::new(),
        max_commands,
    }
}

/// Record a draw command.
#[allow(dead_code)]
pub fn record_draw(r: &mut DrawCommandRecorder, cmd: DrawCommand) -> bool {
    if r.commands.len() >= r.max_commands {
        return false;
    }
    r.commands.push(cmd);
    true
}

/// Clear all recorded commands.
#[allow(dead_code)]
pub fn clear_commands(r: &mut DrawCommandRecorder) {
    r.commands.clear();
}

/// Count of recorded commands.
#[allow(dead_code)]
pub fn command_count(r: &DrawCommandRecorder) -> usize {
    r.commands.len()
}

/// Total index count across all enabled commands.
#[allow(dead_code)]
pub fn total_index_count(r: &DrawCommandRecorder) -> u32 {
    r.commands
        .iter()
        .filter(|c| c.enabled)
        .map(|c| c.index_count)
        .sum()
}

/// Total instance count across all enabled commands.
#[allow(dead_code)]
pub fn total_instance_count(r: &DrawCommandRecorder) -> u32 {
    r.commands
        .iter()
        .filter(|c| c.enabled)
        .map(|c| c.instance_count)
        .sum()
}

/// Sort commands by sort key (front-to-back or back-to-front).
#[allow(dead_code)]
pub fn sort_commands_ascending(r: &mut DrawCommandRecorder) {
    r.commands.sort_by_key(|c| c.sort_key);
}

/// Sort commands descending (back-to-front for transparency).
#[allow(dead_code)]
pub fn sort_commands_descending(r: &mut DrawCommandRecorder) {
    r.commands.sort_by_key(|c| std::cmp::Reverse(c.sort_key));
}

/// Group commands by pipeline ID.
#[allow(dead_code)]
pub fn group_by_pipeline(r: &mut DrawCommandRecorder) {
    r.commands.sort_by_key(|c| c.pipeline_id);
}

/// Check if recorder is full.
#[allow(dead_code)]
pub fn is_recorder_full(r: &DrawCommandRecorder) -> bool {
    r.commands.len() >= r.max_commands
}

/// Export to JSON-like string.
#[allow(dead_code)]
pub fn draw_command_to_json(r: &DrawCommandRecorder) -> String {
    format!(
        r#"{{"command_count":{},"total_indices":{}}}"#,
        r.commands.len(),
        total_index_count(r)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_recorder_empty() {
        let r = new_draw_recorder(100);
        assert_eq!(command_count(&r), 0);
    }

    #[test]
    fn record_returns_true() {
        let mut r = new_draw_recorder(10);
        assert!(record_draw(&mut r, DrawCommand::default()));
    }

    #[test]
    fn capacity_limit() {
        let mut r = new_draw_recorder(1);
        record_draw(&mut r, DrawCommand::default());
        assert!(!record_draw(&mut r, DrawCommand::default()));
    }

    #[test]
    fn clear_empties() {
        let mut r = new_draw_recorder(10);
        record_draw(&mut r, DrawCommand::default());
        clear_commands(&mut r);
        assert_eq!(command_count(&r), 0);
    }

    #[test]
    fn total_index_count_correct() {
        let mut r = new_draw_recorder(10);
        record_draw(
            &mut r,
            DrawCommand {
                index_count: 300,
                ..Default::default()
            },
        );
        record_draw(
            &mut r,
            DrawCommand {
                index_count: 150,
                ..Default::default()
            },
        );
        assert_eq!(total_index_count(&r), 450);
    }

    #[test]
    fn disabled_excluded_from_total() {
        let mut r = new_draw_recorder(10);
        record_draw(
            &mut r,
            DrawCommand {
                index_count: 100,
                enabled: false,
                ..Default::default()
            },
        );
        assert_eq!(total_index_count(&r), 0);
    }

    #[test]
    fn sort_ascending() {
        let mut r = new_draw_recorder(10);
        record_draw(
            &mut r,
            DrawCommand {
                sort_key: 5,
                ..Default::default()
            },
        );
        record_draw(
            &mut r,
            DrawCommand {
                sort_key: 1,
                ..Default::default()
            },
        );
        sort_commands_ascending(&mut r);
        assert_eq!(r.commands[0].sort_key, 1);
    }

    #[test]
    fn group_by_pipeline_sorts() {
        let mut r = new_draw_recorder(10);
        record_draw(
            &mut r,
            DrawCommand {
                pipeline_id: 3,
                ..Default::default()
            },
        );
        record_draw(
            &mut r,
            DrawCommand {
                pipeline_id: 1,
                ..Default::default()
            },
        );
        group_by_pipeline(&mut r);
        assert_eq!(r.commands[0].pipeline_id, 1);
    }

    #[test]
    fn is_full_check() {
        let mut r = new_draw_recorder(1);
        record_draw(&mut r, DrawCommand::default());
        assert!(is_recorder_full(&r));
    }

    #[test]
    fn json_contains_command_count() {
        let r = new_draw_recorder(10);
        assert!(draw_command_to_json(&r).contains("command_count"));
    }
}
