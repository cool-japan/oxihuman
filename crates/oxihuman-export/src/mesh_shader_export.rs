// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Mesh shader stub export (DirectX 12 / Vulkan mesh shading pipeline).

/// A mesh shading pipeline stage.
#[derive(Clone, Copy, PartialEq)]
pub enum MeshShaderStage {
    Task,
    Mesh,
    Pixel,
}

impl MeshShaderStage {
    pub fn name(&self) -> &'static str {
        match self {
            MeshShaderStage::Task => "task",
            MeshShaderStage::Mesh => "mesh",
            MeshShaderStage::Pixel => "pixel",
        }
    }
}

/// A mesh shader program.
pub struct MeshShaderProgram {
    pub stage: MeshShaderStage,
    pub entry_point: String,
    pub source: String,
    pub max_vertices: u32,
    pub max_primitives: u32,
}

/// A mesh shader pipeline export.
pub struct MeshShaderExport {
    pub programs: Vec<MeshShaderProgram>,
    pub amplification_factor: u32,
}

/// Create a new mesh shader export.
pub fn new_mesh_shader_export() -> MeshShaderExport {
    MeshShaderExport {
        programs: Vec::new(),
        amplification_factor: 1,
    }
}

/// Add a mesh shader program.
#[allow(clippy::too_many_arguments)]
pub fn add_mesh_shader_program(
    exp: &mut MeshShaderExport,
    stage: MeshShaderStage,
    entry: &str,
    source: &str,
    max_verts: u32,
    max_prims: u32,
) {
    exp.programs.push(MeshShaderProgram {
        stage,
        entry_point: entry.to_string(),
        source: source.to_string(),
        max_vertices: max_verts,
        max_primitives: max_prims,
    });
}

/// Program count.
pub fn mesh_shader_program_count(exp: &MeshShaderExport) -> usize {
    exp.programs.len()
}

/// Find a program by stage.
pub fn find_mesh_shader_program(
    exp: &MeshShaderExport,
    stage: MeshShaderStage,
) -> Option<&MeshShaderProgram> {
    exp.programs.iter().find(|p| p.stage == stage)
}

/// Validate (must have a mesh stage).
pub fn validate_mesh_shader_export(exp: &MeshShaderExport) -> bool {
    find_mesh_shader_program(exp, MeshShaderStage::Mesh).is_some()
}

/// Render a pipeline summary.
pub fn render_mesh_shader_summary(exp: &MeshShaderExport) -> String {
    let stages: Vec<&str> = exp.programs.iter().map(|p| p.stage.name()).collect();
    format!(
        "MeshShader stages:[{}] amplification:{}",
        stages.join(","),
        exp.amplification_factor
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        let exp = new_mesh_shader_export();
        assert_eq!(mesh_shader_program_count(&exp), 0 /* empty */);
    }

    #[test]
    fn add_program_increments() {
        let mut exp = new_mesh_shader_export();
        add_mesh_shader_program(&mut exp, MeshShaderStage::Mesh, "MeshMain", "", 256, 256);
        assert_eq!(mesh_shader_program_count(&exp), 1 /* one program */);
    }

    #[test]
    fn stage_name_correct() {
        assert_eq!(MeshShaderStage::Task.name(), "task" /* task stage */);
    }

    #[test]
    fn find_program_by_stage() {
        let mut exp = new_mesh_shader_export();
        add_mesh_shader_program(&mut exp, MeshShaderStage::Pixel, "PSMain", "", 0, 0);
        assert!(find_mesh_shader_program(&exp, MeshShaderStage::Pixel).is_some() /* found */);
    }

    #[test]
    fn find_missing_none() {
        let exp = new_mesh_shader_export();
        assert!(find_mesh_shader_program(&exp, MeshShaderStage::Task).is_none() /* not found */);
    }

    #[test]
    fn validate_needs_mesh_stage() {
        let mut exp = new_mesh_shader_export();
        assert!(!validate_mesh_shader_export(&exp) /* no mesh stage */);
        add_mesh_shader_program(&mut exp, MeshShaderStage::Mesh, "MeshMain", "", 64, 42);
        assert!(validate_mesh_shader_export(&exp) /* now valid */);
    }

    #[test]
    fn render_summary_contains_stage_names() {
        let mut exp = new_mesh_shader_export();
        add_mesh_shader_program(&mut exp, MeshShaderStage::Mesh, "m", "", 64, 64);
        add_mesh_shader_program(&mut exp, MeshShaderStage::Pixel, "p", "", 0, 0);
        let s = render_mesh_shader_summary(&exp);
        assert!(s.contains("mesh") /* mesh stage */);
        assert!(s.contains("pixel") /* pixel stage */);
    }

    #[test]
    fn max_vertices_stored() {
        let mut exp = new_mesh_shader_export();
        add_mesh_shader_program(&mut exp, MeshShaderStage::Mesh, "m", "", 128, 84);
        let p = find_mesh_shader_program(&exp, MeshShaderStage::Mesh).expect("should succeed");
        assert_eq!(p.max_vertices, 128 /* correct */);
    }

    #[test]
    fn amplification_factor_default_one() {
        let exp = new_mesh_shader_export();
        assert_eq!(exp.amplification_factor, 1 /* default */);
    }
}
