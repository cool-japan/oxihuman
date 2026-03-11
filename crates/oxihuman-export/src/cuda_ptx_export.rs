// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! CUDA PTX assembly stub export.

/// PTX ISA version.
pub const PTX_ISA_VERSION: &str = ".version 7.5";
/// PTX target architecture.
pub const PTX_TARGET_SM80: &str = ".target sm_80";

/// A CUDA PTX kernel stub.
pub struct CudaPtxKernel {
    pub name: String,
    pub param_count: u32,
    pub body: String,
}

/// A CUDA PTX export.
pub struct CudaPtxExport {
    pub kernels: Vec<CudaPtxKernel>,
    pub isa_version: String,
    pub target: String,
    pub address_size: u32,
}

/// Create a new CUDA PTX export.
pub fn new_cuda_ptx_export() -> CudaPtxExport {
    CudaPtxExport {
        kernels: Vec::new(),
        isa_version: PTX_ISA_VERSION.to_string(),
        target: PTX_TARGET_SM80.to_string(),
        address_size: 64,
    }
}

/// Add a PTX kernel stub.
pub fn add_cuda_ptx_kernel(exp: &mut CudaPtxExport, name: &str, param_count: u32) {
    exp.kernels.push(CudaPtxKernel {
        name: name.to_string(),
        param_count,
        body: "ret;".to_string(),
    });
}

/// Kernel count.
pub fn cuda_ptx_kernel_count(exp: &CudaPtxExport) -> usize {
    exp.kernels.len()
}

/// Find a kernel by name.
pub fn find_cuda_ptx_kernel<'a>(exp: &'a CudaPtxExport, name: &str) -> Option<&'a CudaPtxKernel> {
    exp.kernels.iter().find(|k| k.name == name)
}

/// Render PTX source stub.
pub fn render_cuda_ptx(exp: &CudaPtxExport) -> String {
    let mut s = format!(
        "{}\n{}\n.address_size {}\n",
        exp.isa_version, exp.target, exp.address_size
    );
    for k in &exp.kernels {
        s.push_str(&format!(
            ".visible .entry {}(.param .b64 param0) {{\n  {}\n}}\n",
            k.name, k.body
        ));
    }
    s
}

/// Validate (at least one kernel, 64-bit address).
pub fn validate_cuda_ptx(exp: &CudaPtxExport) -> bool {
    !exp.kernels.is_empty() && exp.address_size == 64
}

/// Estimated PTX source size in bytes.
pub fn cuda_ptx_size_estimate(exp: &CudaPtxExport) -> usize {
    render_cuda_ptx(exp).len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        let exp = new_cuda_ptx_export();
        assert_eq!(cuda_ptx_kernel_count(&exp), 0 /* empty */);
    }

    #[test]
    fn add_kernel_increments() {
        let mut exp = new_cuda_ptx_export();
        add_cuda_ptx_kernel(&mut exp, "my_kernel", 2);
        assert_eq!(cuda_ptx_kernel_count(&exp), 1 /* one kernel */);
    }

    #[test]
    fn find_kernel_by_name() {
        let mut exp = new_cuda_ptx_export();
        add_cuda_ptx_kernel(&mut exp, "sum_kernel", 3);
        assert!(find_cuda_ptx_kernel(&exp, "sum_kernel").is_some() /* found */);
    }

    #[test]
    fn find_missing_none() {
        let exp = new_cuda_ptx_export();
        assert!(find_cuda_ptx_kernel(&exp, "x").is_none() /* not found */);
    }

    #[test]
    fn render_contains_isa_version() {
        let exp = new_cuda_ptx_export();
        let src = render_cuda_ptx(&exp);
        assert!(src.contains("version") /* isa version */);
    }

    #[test]
    fn render_contains_kernel_name() {
        let mut exp = new_cuda_ptx_export();
        add_cuda_ptx_kernel(&mut exp, "vec_add", 4);
        let src = render_cuda_ptx(&exp);
        assert!(src.contains("vec_add") /* kernel name */);
    }

    #[test]
    fn validate_empty_fails() {
        let exp = new_cuda_ptx_export();
        assert!(!validate_cuda_ptx(&exp) /* no kernels */);
    }

    #[test]
    fn validate_with_kernel_passes() {
        let mut exp = new_cuda_ptx_export();
        add_cuda_ptx_kernel(&mut exp, "k", 1);
        assert!(validate_cuda_ptx(&exp) /* valid */);
    }

    #[test]
    fn address_size_is_64() {
        let exp = new_cuda_ptx_export();
        assert_eq!(exp.address_size, 64 /* 64-bit */);
    }
}
