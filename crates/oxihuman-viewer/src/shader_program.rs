#![allow(dead_code)]
//! Shader program: represents a GPU shader program with stages.

/// A stage in a shader program.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ProgramStage {
    Vertex,
    Fragment,
    Geometry,
    Compute,
}

/// A shader program.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShaderProgram {
    name: String,
    stages: Vec<ProgramStage>,
    valid: bool,
}

/// Create a new shader program.
#[allow(dead_code)]
pub fn new_shader_program(name: &str) -> ShaderProgram {
    ShaderProgram {
        name: name.to_string(),
        stages: Vec::new(),
        valid: false,
    }
}

/// Add a stage to the program.
#[allow(dead_code)]
pub fn add_stage(program: &mut ShaderProgram, stage: ProgramStage) {
    program.stages.push(stage);
}

/// Return the number of stages.
#[allow(dead_code)]
pub fn stage_count_sp(program: &ShaderProgram) -> usize {
    program.stages.len()
}

/// Return the program name.
#[allow(dead_code)]
pub fn program_name(program: &ShaderProgram) -> &str {
    &program.name
}

/// Stub compile: marks the program as valid if it has at least vertex and fragment stages.
#[allow(dead_code)]
pub fn compile_program_stub(program: &mut ShaderProgram) -> bool {
    let has_vertex = program.stages.contains(&ProgramStage::Vertex);
    let has_fragment = program.stages.contains(&ProgramStage::Fragment);
    program.valid = has_vertex && has_fragment;
    program.valid
}

/// Serialize to JSON-like string.
#[allow(dead_code)]
pub fn program_to_json(program: &ShaderProgram) -> String {
    let stages_str: Vec<&str> = program
        .stages
        .iter()
        .map(|s| match s {
            ProgramStage::Vertex => "vertex",
            ProgramStage::Fragment => "fragment",
            ProgramStage::Geometry => "geometry",
            ProgramStage::Compute => "compute",
        })
        .collect();
    format!(
        "{{\"name\":\"{}\",\"stages\":[{}],\"valid\":{}}}",
        program.name,
        stages_str.iter().map(|s| format!("\"{s}\"")).collect::<Vec<_>>().join(","),
        program.valid
    )
}

/// Check if the program is valid.
#[allow(dead_code)]
pub fn program_is_valid(program: &ShaderProgram) -> bool {
    program.valid
}

/// Clear all stages and mark as invalid.
#[allow(dead_code)]
pub fn program_clear(program: &mut ShaderProgram) {
    program.stages.clear();
    program.valid = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_program() {
        let p = new_shader_program("test");
        assert_eq!(program_name(&p), "test");
        assert!(!program_is_valid(&p));
    }

    #[test]
    fn test_add_stage() {
        let mut p = new_shader_program("s");
        add_stage(&mut p, ProgramStage::Vertex);
        assert_eq!(stage_count_sp(&p), 1);
    }

    #[test]
    fn test_compile_valid() {
        let mut p = new_shader_program("s");
        add_stage(&mut p, ProgramStage::Vertex);
        add_stage(&mut p, ProgramStage::Fragment);
        assert!(compile_program_stub(&mut p));
    }

    #[test]
    fn test_compile_invalid() {
        let mut p = new_shader_program("s");
        add_stage(&mut p, ProgramStage::Vertex);
        assert!(!compile_program_stub(&mut p));
    }

    #[test]
    fn test_to_json() {
        let p = new_shader_program("test");
        let json = program_to_json(&p);
        assert!(json.contains("\"name\":\"test\""));
    }

    #[test]
    fn test_clear() {
        let mut p = new_shader_program("s");
        add_stage(&mut p, ProgramStage::Vertex);
        program_clear(&mut p);
        assert_eq!(stage_count_sp(&p), 0);
        assert!(!program_is_valid(&p));
    }

    #[test]
    fn test_stage_count() {
        let mut p = new_shader_program("s");
        add_stage(&mut p, ProgramStage::Vertex);
        add_stage(&mut p, ProgramStage::Fragment);
        add_stage(&mut p, ProgramStage::Geometry);
        assert_eq!(stage_count_sp(&p), 3);
    }

    #[test]
    fn test_compute_stage() {
        let mut p = new_shader_program("compute");
        add_stage(&mut p, ProgramStage::Compute);
        assert_eq!(stage_count_sp(&p), 1);
        assert!(!compile_program_stub(&mut p));
    }

    #[test]
    fn test_program_name() {
        let p = new_shader_program("my_shader");
        assert_eq!(program_name(&p), "my_shader");
    }

    #[test]
    fn test_valid_after_compile() {
        let mut p = new_shader_program("s");
        add_stage(&mut p, ProgramStage::Vertex);
        add_stage(&mut p, ProgramStage::Fragment);
        compile_program_stub(&mut p);
        assert!(program_is_valid(&p));
    }
}
