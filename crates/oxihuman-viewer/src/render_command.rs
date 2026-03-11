// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Render command recording.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum RenderCommandKind {
    DrawMesh(u32),
    SetMaterial(u32),
    SetViewport(u32, u32, u32, u32),
    Clear([f32; 4]),
    EndPass,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderCommandBuffer {
    pub commands: Vec<RenderCommandKind>,
}

#[allow(dead_code)]
pub fn new_render_command_buffer() -> RenderCommandBuffer {
    RenderCommandBuffer { commands: Vec::new() }
}

#[allow(dead_code)]
pub fn rcb_push(buf: &mut RenderCommandBuffer, cmd: RenderCommandKind) {
    buf.commands.push(cmd);
}

#[allow(dead_code)]
pub fn rcb_command_count(buf: &RenderCommandBuffer) -> usize {
    buf.commands.len()
}

#[allow(dead_code)]
pub fn rcb_clear(buf: &mut RenderCommandBuffer) {
    buf.commands.clear();
}

#[allow(dead_code)]
pub fn rcb_draw_mesh_count(buf: &RenderCommandBuffer) -> usize {
    buf.commands.iter().filter(|c| matches!(c, RenderCommandKind::DrawMesh(_))).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_count() {
        let mut b = new_render_command_buffer();
        rcb_push(&mut b, RenderCommandKind::DrawMesh(1));
        assert_eq!(rcb_command_count(&b), 1);
    }

    #[test]
    fn test_clear() {
        let mut b = new_render_command_buffer();
        rcb_push(&mut b, RenderCommandKind::DrawMesh(1));
        rcb_clear(&mut b);
        assert_eq!(rcb_command_count(&b), 0);
    }

    #[test]
    fn test_draw_mesh_count() {
        let mut b = new_render_command_buffer();
        rcb_push(&mut b, RenderCommandKind::DrawMesh(1));
        rcb_push(&mut b, RenderCommandKind::DrawMesh(2));
        rcb_push(&mut b, RenderCommandKind::SetMaterial(3));
        assert_eq!(rcb_draw_mesh_count(&b), 2);
    }

    #[test]
    fn test_mixed_commands() {
        let mut b = new_render_command_buffer();
        rcb_push(&mut b, RenderCommandKind::Clear([0.0; 4]));
        rcb_push(&mut b, RenderCommandKind::SetViewport(0, 0, 1920, 1080));
        rcb_push(&mut b, RenderCommandKind::DrawMesh(5));
        rcb_push(&mut b, RenderCommandKind::EndPass);
        assert_eq!(rcb_command_count(&b), 4);
        assert_eq!(rcb_draw_mesh_count(&b), 1);
    }

    #[test]
    fn test_empty_draw_mesh_count() {
        let b = new_render_command_buffer();
        assert_eq!(rcb_draw_mesh_count(&b), 0);
    }

    #[test]
    fn test_end_pass_not_counted_as_draw() {
        let mut b = new_render_command_buffer();
        rcb_push(&mut b, RenderCommandKind::EndPass);
        assert_eq!(rcb_draw_mesh_count(&b), 0);
    }

    #[test]
    fn test_set_material_not_counted_as_draw() {
        let mut b = new_render_command_buffer();
        rcb_push(&mut b, RenderCommandKind::SetMaterial(99));
        assert_eq!(rcb_draw_mesh_count(&b), 0);
    }

    #[test]
    fn test_initial_count_zero() {
        let b = new_render_command_buffer();
        assert_eq!(rcb_command_count(&b), 0);
    }
}
