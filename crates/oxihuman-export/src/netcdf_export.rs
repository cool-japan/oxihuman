// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! NetCDF stub export — describe NetCDF-4 variable/dimension layout for mesh data.

/// NetCDF data type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NcType {
    Byte,
    Short,
    Int,
    Float,
    Double,
    Char,
}

impl NcType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NcType::Byte => "byte",
            NcType::Short => "short",
            NcType::Int => "int",
            NcType::Float => "float",
            NcType::Double => "double",
            NcType::Char => "char",
        }
    }
}

/// A NetCDF dimension.
#[derive(Debug, Clone)]
pub struct NcDimension {
    pub name: String,
    pub size: usize,
    pub unlimited: bool,
}

/// A NetCDF attribute (global or variable-level).
#[derive(Debug, Clone)]
pub struct NcAttribute {
    pub name: String,
    pub value: String,
}

/// A NetCDF variable.
#[derive(Debug, Clone)]
pub struct NcVariable {
    pub name: String,
    pub nc_type: NcType,
    pub dims: Vec<String>,
    pub attrs: Vec<NcAttribute>,
}

impl NcVariable {
    pub fn new(name: &str, nc_type: NcType, dims: Vec<&str>) -> Self {
        Self { name: name.into(), nc_type, dims: dims.into_iter().map(|s| s.into()).collect(), attrs: vec![] }
    }
    pub fn add_attr(&mut self, name: &str, value: &str) {
        self.attrs.push(NcAttribute { name: name.into(), value: value.into() });
    }
}

/// A NetCDF file stub.
#[derive(Debug, Clone, Default)]
pub struct NetCdfFileDef {
    pub filename: String,
    pub dimensions: Vec<NcDimension>,
    pub global_attrs: Vec<NcAttribute>,
    pub variables: Vec<NcVariable>,
}

/// Create a new NetCDF file definition.
pub fn new_netcdf_file(filename: &str) -> NetCdfFileDef {
    NetCdfFileDef {
        filename: filename.into(),
        global_attrs: vec![NcAttribute { name: "Conventions".into(), value: "CF-1.8".into() }],
        ..Default::default()
    }
}

/// Add a dimension.
pub fn add_dimension(file: &mut NetCdfFileDef, name: &str, size: usize, unlimited: bool) {
    file.dimensions.push(NcDimension { name: name.into(), size, unlimited });
}

/// Add a variable.
pub fn add_nc_variable(file: &mut NetCdfFileDef, var: NcVariable) {
    file.variables.push(var);
}

/// Export mesh positions as NetCDF stub.
pub fn export_mesh_netcdf(positions: &[[f32; 3]]) -> NetCdfFileDef {
    let mut file = new_netcdf_file("mesh.nc");
    add_dimension(&mut file, "vertex", positions.len(), false);
    add_dimension(&mut file, "ndim", 3, false);
    let mut pos_var = NcVariable::new("positions", NcType::Float, vec!["vertex", "ndim"]);
    pos_var.add_attr("units", "meter");
    pos_var.add_attr("long_name", "vertex positions");
    add_nc_variable(&mut file, pos_var);
    file
}

/// Serialize NetCDF CDL (ASCII header).
pub fn to_cdl_string(file: &NetCdfFileDef) -> String {
    let mut s = format!("netcdf {} {{\n", file.filename.trim_end_matches(".nc"));
    s.push_str("dimensions:\n");
    for d in &file.dimensions {
        let sz = if d.unlimited { "UNLIMITED".into() } else { d.size.to_string() };
        s.push_str(&format!("\t{} = {} ;\n", d.name, sz));
    }
    s.push_str("variables:\n");
    for v in &file.variables {
        let dims = v.dims.join(", ");
        s.push_str(&format!("\t{} {}({}) ;\n", v.nc_type.as_str(), v.name, dims));
        for a in &v.attrs {
            s.push_str(&format!("\t\t{}:{} = \"{}\" ;\n", v.name, a.name, a.value));
        }
    }
    s.push_str("// global attributes:\n");
    for ga in &file.global_attrs {
        s.push_str(&format!("\t\t:{} = \"{}\" ;\n", ga.name, ga.value));
    }
    s.push_str("}\n");
    s
}

/// Number of variables defined.
pub fn variable_count(file: &NetCdfFileDef) -> usize {
    file.variables.len()
}

/// Number of dimensions defined.
pub fn dimension_count(file: &NetCdfFileDef) -> usize {
    file.dimensions.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_netcdf_file_has_conventions() {
        /* global attribute Conventions is set */
        let f = new_netcdf_file("x.nc");
        assert!(f.global_attrs.iter().any(|a| a.name == "Conventions"));
    }

    #[test]
    fn test_add_dimension_increases_count() {
        /* adding dimension increases count */
        let mut f = new_netcdf_file("x.nc");
        add_dimension(&mut f, "time", 100, true);
        assert_eq!(dimension_count(&f), 1);
    }

    #[test]
    fn test_add_variable_increases_count() {
        /* adding variable increases count */
        let mut f = new_netcdf_file("x.nc");
        add_nc_variable(&mut f, NcVariable::new("temp", NcType::Float, vec!["time"]));
        assert_eq!(variable_count(&f), 1);
    }

    #[test]
    fn test_export_mesh_netcdf_dims() {
        /* mesh export has vertex and ndim dimensions */
        let p = vec![[0.0f32;3]; 10];
        let f = export_mesh_netcdf(&p);
        assert_eq!(dimension_count(&f), 2);
    }

    #[test]
    fn test_to_cdl_contains_filename() {
        /* CDL output contains filename */
        let f = new_netcdf_file("data.nc");
        assert!(to_cdl_string(&f).contains("data"));
    }

    #[test]
    fn test_to_cdl_contains_variable() {
        /* CDL output contains variable names */
        let p = vec![[1.0f32,2.0,3.0]];
        let f = export_mesh_netcdf(&p);
        assert!(to_cdl_string(&f).contains("positions"));
    }

    #[test]
    fn test_nc_type_as_str() {
        /* NcType::Float maps to "float" */
        assert_eq!(NcType::Float.as_str(), "float");
    }

    #[test]
    fn test_variable_attr_stored() {
        /* variable attributes are stored */
        let mut v = NcVariable::new("x", NcType::Double, vec!["n"]);
        v.add_attr("units", "km");
        assert_eq!(v.attrs[0].name, "units");
    }

    #[test]
    fn test_unlimited_dimension_in_cdl() {
        /* unlimited dimension shows UNLIMITED in CDL */
        let mut f = new_netcdf_file("x.nc");
        add_dimension(&mut f, "time", 0, true);
        assert!(to_cdl_string(&f).contains("UNLIMITED"));
    }

    #[test]
    fn test_no_variables_zero_count() {
        /* no variables → zero count */
        let f = new_netcdf_file("x.nc");
        assert_eq!(variable_count(&f), 0);
    }
}
