// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! HDF5 stub export — describe HDF5 dataset/group hierarchy for mesh data.

/// An HDF5 attribute (key-value pair).
#[derive(Debug, Clone)]
pub struct Hdf5Attr {
    pub key: String,
    pub value: String,
}

/// An HDF5 dataset descriptor.
#[derive(Debug, Clone)]
pub struct Hdf5Dataset {
    pub name: String,
    pub dtype: String,
    pub shape: Vec<usize>,
    pub attrs: Vec<Hdf5Attr>,
}

impl Hdf5Dataset {
    pub fn new(name: &str, dtype: &str, shape: Vec<usize>) -> Self {
        Self { name: name.into(), dtype: dtype.into(), shape, attrs: vec![] }
    }
    pub fn add_attr(&mut self, key: &str, val: &str) {
        self.attrs.push(Hdf5Attr { key: key.into(), value: val.into() });
    }
}

/// An HDF5 group (can contain datasets and sub-groups).
#[derive(Debug, Clone, Default)]
pub struct Hdf5Group {
    pub name: String,
    pub datasets: Vec<Hdf5Dataset>,
    pub subgroups: Vec<Hdf5Group>,
    pub attrs: Vec<Hdf5Attr>,
}

impl Hdf5Group {
    pub fn new(name: &str) -> Self {
        Self { name: name.into(), ..Default::default() }
    }
}

/// An HDF5 file stub.
#[derive(Debug, Clone, Default)]
pub struct Hdf5FileDef {
    pub filename: String,
    pub root: Hdf5Group,
}

/// Create a new HDF5 file definition.
pub fn new_hdf5_file(filename: &str) -> Hdf5FileDef {
    Hdf5FileDef { filename: filename.into(), root: Hdf5Group::new("/") }
}

/// Add a dataset to the root group.
pub fn add_dataset_to_root(file: &mut Hdf5FileDef, ds: Hdf5Dataset) {
    file.root.datasets.push(ds);
}

/// Add a subgroup to root.
pub fn add_group_to_root(file: &mut Hdf5FileDef, group: Hdf5Group) {
    file.root.subgroups.push(group);
}

/// Export mesh positions and indices as an HDF5 stub.
pub fn export_mesh_hdf5(positions: &[[f32; 3]], indices: &[u32]) -> Hdf5FileDef {
    let mut file = new_hdf5_file("mesh.h5");
    let mut pos_ds = Hdf5Dataset::new("positions", "float32", vec![positions.len(), 3]);
    pos_ds.add_attr("units", "meters");
    let idx_ds = Hdf5Dataset::new("indices", "uint32", vec![indices.len()]);
    add_dataset_to_root(&mut file, pos_ds);
    add_dataset_to_root(&mut file, idx_ds);
    file
}

/// Generate a schema summary string.
pub fn hdf5_schema_to_string(file: &Hdf5FileDef) -> String {
    let mut s = format!("HDF5 file: {}\n", file.filename);
    s.push_str(&group_to_string(&file.root, 0));
    s
}

fn group_to_string(g: &Hdf5Group, indent: usize) -> String {
    let pad = " ".repeat(indent);
    let mut s = format!("{}GROUP {}\n", pad, g.name);
    for ds in &g.datasets {
        let shape: Vec<String> = ds.shape.iter().map(|n| n.to_string()).collect();
        s.push_str(&format!("{}  DATASET {} [{}] dtype={}\n", pad, ds.name, shape.join("x"), ds.dtype));
    }
    for sg in &g.subgroups {
        s.push_str(&group_to_string(sg, indent + 2));
    }
    s
}

/// Count datasets in the root group.
pub fn dataset_count(file: &Hdf5FileDef) -> usize {
    file.root.datasets.len()
}

/// Total element count across all datasets.
pub fn total_element_count(file: &Hdf5FileDef) -> usize {
    file.root.datasets.iter().map(|ds| ds.shape.iter().product::<usize>()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_hdf5_file() {
        /* new file has empty root */
        let f = new_hdf5_file("test.h5");
        assert_eq!(f.filename, "test.h5");
        assert_eq!(dataset_count(&f), 0);
    }

    #[test]
    fn test_add_dataset_increases_count() {
        /* adding dataset increases count */
        let mut f = new_hdf5_file("a.h5");
        add_dataset_to_root(&mut f, Hdf5Dataset::new("d", "float32", vec![10, 3]));
        assert_eq!(dataset_count(&f), 1);
    }

    #[test]
    fn test_export_mesh_hdf5_two_datasets() {
        /* mesh export adds positions + indices datasets */
        let p = vec![[0.0f32,0.0,0.0]; 5];
        let i: Vec<u32> = vec![0,1,2];
        let f = export_mesh_hdf5(&p, &i);
        assert_eq!(dataset_count(&f), 2);
    }

    #[test]
    fn test_total_element_count() {
        /* total elements = product of shapes */
        let mut f = new_hdf5_file("x.h5");
        add_dataset_to_root(&mut f, Hdf5Dataset::new("d", "float32", vec![4, 3]));
        assert_eq!(total_element_count(&f), 12);
    }

    #[test]
    fn test_hdf5_schema_to_string_contains_filename() {
        /* schema string contains filename */
        let f = new_hdf5_file("scene.h5");
        let s = hdf5_schema_to_string(&f);
        assert!(s.contains("scene.h5"));
    }

    #[test]
    fn test_dataset_attr() {
        /* attrs are stored correctly */
        let mut ds = Hdf5Dataset::new("pos", "float32", vec![10, 3]);
        ds.add_attr("units", "mm");
        assert_eq!(ds.attrs[0].key, "units");
        assert_eq!(ds.attrs[0].value, "mm");
    }

    #[test]
    fn test_add_group_to_root() {
        /* adding subgroup works */
        let mut f = new_hdf5_file("f.h5");
        add_group_to_root(&mut f, Hdf5Group::new("geometry"));
        assert_eq!(f.root.subgroups.len(), 1);
    }

    #[test]
    fn test_schema_string_contains_dataset_name() {
        /* schema string mentions dataset names */
        let mut f = new_hdf5_file("x.h5");
        add_dataset_to_root(&mut f, Hdf5Dataset::new("mydata", "float64", vec![5]));
        let s = hdf5_schema_to_string(&f);
        assert!(s.contains("mydata"));
    }

    #[test]
    fn test_group_name_stored() {
        /* group name is stored */
        let g = Hdf5Group::new("measurements");
        assert_eq!(g.name, "measurements");
    }

    #[test]
    fn test_dataset_shape_stored() {
        /* shape is stored correctly */
        let ds = Hdf5Dataset::new("d", "int32", vec![100, 4]);
        assert_eq!(ds.shape, vec![100, 4]);
    }
}
