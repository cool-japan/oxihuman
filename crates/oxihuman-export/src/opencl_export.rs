// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! OpenCL kernel stub export.

/// An OpenCL kernel argument.
pub struct ClKernelArg {
    pub type_name: String,
    pub name: String,
    pub is_global: bool,
}

/// An OpenCL kernel.
pub struct ClKernel {
    pub name: String,
    pub args: Vec<ClKernelArg>,
    pub body: String,
}

/// An OpenCL program export.
pub struct OpenClExport {
    pub kernels: Vec<ClKernel>,
    pub pragmas: Vec<String>,
}

/// Create a new OpenCL export.
pub fn new_opencl_export() -> OpenClExport {
    OpenClExport {
        kernels: Vec::new(),
        pragmas: vec!["#pragma OPENCL EXTENSION cl_khr_fp64 : enable".to_string()],
    }
}

/// Add a kernel stub.
pub fn add_cl_kernel(exp: &mut OpenClExport, name: &str, body: &str) {
    exp.kernels.push(ClKernel {
        name: name.to_string(),
        args: Vec::new(),
        body: body.to_string(),
    });
}

/// Add an argument to the last kernel.
pub fn add_cl_kernel_arg(
    exp: &mut OpenClExport,
    type_name: &str,
    arg_name: &str,
    is_global: bool,
) -> bool {
    if let Some(k) = exp.kernels.last_mut() {
        k.args.push(ClKernelArg {
            type_name: type_name.to_string(),
            name: arg_name.to_string(),
            is_global,
        });
        true
    } else {
        false
    }
}

/// Kernel count.
pub fn cl_kernel_count(exp: &OpenClExport) -> usize {
    exp.kernels.len()
}

/// Find a kernel by name.
pub fn find_cl_kernel<'a>(exp: &'a OpenClExport, name: &str) -> Option<&'a ClKernel> {
    exp.kernels.iter().find(|k| k.name == name)
}

/// Render OpenCL source.
pub fn render_opencl_source(exp: &OpenClExport) -> String {
    let mut s = String::new();
    for p in &exp.pragmas {
        s.push_str(p);
        s.push('\n');
    }
    for k in &exp.kernels {
        let args: Vec<String> = k
            .args
            .iter()
            .map(|a| {
                if a.is_global {
                    format!("__global {} {}", a.type_name, a.name)
                } else {
                    format!("{} {}", a.type_name, a.name)
                }
            })
            .collect();
        s.push_str(&format!(
            "__kernel void {}({}) {{\n  {}\n}}\n",
            k.name,
            args.join(", "),
            k.body
        ));
    }
    s
}

/// Validate (at least one kernel).
pub fn validate_opencl_export(exp: &OpenClExport) -> bool {
    !exp.kernels.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        let exp = new_opencl_export();
        assert_eq!(cl_kernel_count(&exp), 0 /* empty */);
    }

    #[test]
    fn add_kernel_increments() {
        let mut exp = new_opencl_export();
        add_cl_kernel(&mut exp, "my_kernel", "int id = get_global_id(0);");
        assert_eq!(cl_kernel_count(&exp), 1 /* one kernel */);
    }

    #[test]
    fn add_arg_to_kernel() {
        let mut exp = new_opencl_export();
        add_cl_kernel(&mut exp, "k", "");
        let ok = add_cl_kernel_arg(&mut exp, "float*", "data", true);
        assert!(ok /* added */);
        assert_eq!(exp.kernels[0].args.len(), 1 /* one arg */);
    }

    #[test]
    fn add_arg_no_kernel_fails() {
        let mut exp = new_opencl_export();
        assert!(!add_cl_kernel_arg(&mut exp, "int", "x", false) /* no kernel */);
    }

    #[test]
    fn find_kernel_by_name() {
        let mut exp = new_opencl_export();
        add_cl_kernel(&mut exp, "sum", "");
        assert!(find_cl_kernel(&exp, "sum").is_some() /* found */);
    }

    #[test]
    fn find_missing_none() {
        let exp = new_opencl_export();
        assert!(find_cl_kernel(&exp, "x").is_none() /* not found */);
    }

    #[test]
    fn render_contains_kernel_keyword() {
        let mut exp = new_opencl_export();
        add_cl_kernel(&mut exp, "k", "");
        let src = render_opencl_source(&exp);
        assert!(src.contains("__kernel") /* keyword */);
    }

    #[test]
    fn render_contains_global_arg() {
        let mut exp = new_opencl_export();
        add_cl_kernel(&mut exp, "k", "");
        add_cl_kernel_arg(&mut exp, "float*", "buf", true);
        let src = render_opencl_source(&exp);
        assert!(src.contains("__global") /* global qualifier */);
    }

    #[test]
    fn validate_empty_fails() {
        let exp = new_opencl_export();
        assert!(!validate_opencl_export(&exp) /* empty */);
    }
}
