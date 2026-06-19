// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! HDF5 v0 binary encoder for neural-network weights.
//!
//! Produces real HDF5 superblock v0 binary files with:
//! - correct 8-byte file signature
//! - superblock v0 at offset 8 with `size_of_offsets=8` / `size_of_lengths=8`
//! - dataset object headers v1, dataspace, datatype, and data-layout messages
//! - raw payload bytes embedded in the file
//! - a local heap, symbol-table node, B-tree node and root-group object header

use oxihuman_core::CursorWriter;

// ─────────────────────────────────────────────────────────────────────────────
// Public types
// ─────────────────────────────────────────────────────────────────────────────

/// HDF5 dataset data type.
#[derive(Debug, Clone, PartialEq)]
pub enum Hdf5Dtype {
    Float32,
    Float64,
    Int32,
    Int64,
    Uint8,
}

impl Hdf5Dtype {
    /// Returns itemsize in bytes.
    pub fn itemsize(&self) -> usize {
        match self {
            Hdf5Dtype::Float32 | Hdf5Dtype::Int32 => 4,
            Hdf5Dtype::Float64 | Hdf5Dtype::Int64 => 8,
            Hdf5Dtype::Uint8 => 1,
        }
    }
}

/// Concrete payload stored in an HDF5 dataset.
#[derive(Debug, Clone)]
pub enum Hdf5Payload {
    Float32(Vec<f32>),
    Float64(Vec<f64>),
    Int32(Vec<i32>),
    Int64(Vec<i64>),
    Uint8(Vec<u8>),
    Empty,
}

/// HDF5 dataset stub (a named tensor stored in a group).
#[derive(Debug, Clone)]
pub struct Hdf5Dataset {
    pub name: String,
    pub dtype: Hdf5Dtype,
    pub shape: Vec<usize>,
    /// Raw data payload.  Defaults to `Hdf5Payload::Empty`.
    pub payload: Hdf5Payload,
}

impl Hdf5Dataset {
    /// Returns total element count.
    pub fn numel(&self) -> usize {
        self.shape.iter().product::<usize>().max(1)
    }

    /// Returns total byte size.
    pub fn byte_size(&self) -> usize {
        self.numel() * self.dtype.itemsize()
    }
}

/// HDF5 group stub (mirrors Keras / h5py group layout).
#[derive(Debug, Clone)]
pub struct Hdf5Group {
    pub name: String,
    pub datasets: Vec<Hdf5Dataset>,
    pub subgroups: Vec<Hdf5Group>,
    pub attributes: Vec<(String, String)>,
}

impl Hdf5Group {
    /// Creates a new empty group.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            datasets: vec![],
            subgroups: vec![],
            attributes: vec![],
        }
    }

    /// Adds a dataset to the group.
    pub fn add_dataset(&mut self, ds: Hdf5Dataset) {
        self.datasets.push(ds);
    }

    /// Adds a subgroup.
    pub fn add_subgroup(&mut self, sg: Hdf5Group) {
        self.subgroups.push(sg);
    }

    /// Adds an attribute.
    pub fn add_attribute(&mut self, key: &str, val: &str) {
        self.attributes.push((key.to_string(), val.to_string()));
    }

    /// Recursive total byte size.
    pub fn total_byte_size(&self) -> usize {
        let ds_bytes: usize = self.datasets.iter().map(|d| d.byte_size()).sum();
        let sub_bytes: usize = self.subgroups.iter().map(|g| g.total_byte_size()).sum();
        ds_bytes + sub_bytes
    }
}

/// HDF5 weights file stub.
#[derive(Debug, Clone)]
pub struct Hdf5WeightsExport {
    pub filename: String,
    pub root: Hdf5Group,
    pub hdf5_version: String,
}

impl Default for Hdf5WeightsExport {
    fn default() -> Self {
        Self {
            filename: String::new(),
            root: Hdf5Group::new("/"),
            hdf5_version: "1.12.0".to_string(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Existing public functions (unchanged API)
// ─────────────────────────────────────────────────────────────────────────────

/// Creates a new HDF5 weights export stub.
pub fn new_hdf5_weights_export(filename: &str) -> Hdf5WeightsExport {
    Hdf5WeightsExport {
        filename: filename.to_string(),
        ..Default::default()
    }
}

/// Returns the total number of top-level groups.
pub fn hdf5_group_count(export: &Hdf5WeightsExport) -> usize {
    export.root.subgroups.len()
}

/// Returns the total estimated data size in bytes.
pub fn hdf5_data_size(export: &Hdf5WeightsExport) -> usize {
    export.root.total_byte_size() + 2048 /* HDF5 superblock + B-tree overhead */
}

/// Validates the export (non-empty filename, at least one root subgroup).
pub fn validate_hdf5_weights(export: &Hdf5WeightsExport) -> bool {
    !export.filename.is_empty() && !export.root.subgroups.is_empty()
}

/// Returns a JSON-like summary.
pub fn hdf5_summary_json(export: &Hdf5WeightsExport) -> String {
    format!(
        "{{\"file\":\"{}\",\"hdf5_version\":\"{}\",\"root_groups\":{},\"data_bytes\":{}}}",
        export.filename,
        export.hdf5_version,
        hdf5_group_count(export),
        hdf5_data_size(export)
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// New public binary-encoding API
// ─────────────────────────────────────────────────────────────────────────────

/// Serialise a complete `Hdf5WeightsExport` to a real HDF5 v0 binary file.
///
/// The resulting `Vec<u8>` starts with the canonical HDF5 8-byte signature,
/// has a superblock v0 at offset 8, and embeds each dataset's payload as raw
/// little-endian bytes inside proper object-header structures.
pub fn to_hdf5_bytes(export: &Hdf5WeightsExport) -> Vec<u8> {
    // Collect all root-level datasets.
    let datasets: &[Hdf5Dataset] = &export.root.datasets;

    // ── First pass: compute raw payload bytes ─────────────────────────────
    let raw_payloads: Vec<Vec<u8>> = datasets
        .iter()
        .map(|d| payload_to_bytes(&d.payload))
        .collect();

    // ── We need a two-pass write because the dataset object headers contain
    //    the absolute file address of the dataset raw data, which is not known
    //    until we have accounted for the header's own size.
    //
    // Layout (each size computed in the pass below):
    //   0 ..  8   : HDF5 signature
    //   8 .. 56   : superblock v0 (48 bytes)
    //  56 ..      : for each dataset:
    //                 object header v1 (fixed known size per dataset)
    //                 raw data bytes
    //              : local heap (HEAP + names)
    //              : symbol-table node (SNOD)
    //              : B-tree node (TREE)
    //              : root-group object header
    // ─────────────────────────────────────────────────────────────────────

    const SIG_SIZE: usize = 8;
    // Superblock v0 actual byte count:
    //  5  (version bytes) + 1 (sz_offsets) + 1 (sz_lengths) + 1 (reserved)
    //  + 2 (leaf_k) + 2 (internal_k) + 4 (flags)
    //  + 8 (base) + 8 (free_space) + 8 (eof) + 8 (driver_info)
    //  + 8 (sym_entry.link_name_off) + 8 (sym_entry.ohdr_addr)
    //  + 4 (cache_type) + 4 (reserved) + 8 (btree_addr) + 8 (lheap_addr)
    //  = 88 bytes
    const SUPERBLOCK_SIZE: usize = 88;
    const HEADER_BASE: usize = SIG_SIZE + SUPERBLOCK_SIZE;

    // Compute the size of each dataset's object header up-front.
    // We need to know where raw data lives before writing the layout message.
    let ohdr_sizes: Vec<usize> = datasets
        .iter()
        .map(|d| {
            let shape_u64: Vec<u64> = d.shape.iter().map(|&x| x as u64).collect();
            compute_dataset_ohdr_size(&shape_u64)
        })
        .collect();

    // Compute absolute start offsets for each (ohdr, data) pair.
    let mut dataset_ohdr_addrs: Vec<u64> = Vec::with_capacity(datasets.len());
    let mut dataset_data_addrs: Vec<u64> = Vec::with_capacity(datasets.len());
    let mut cursor: usize = HEADER_BASE;
    for (i, _) in datasets.iter().enumerate() {
        dataset_ohdr_addrs.push(cursor as u64);
        cursor += ohdr_sizes[i];
        dataset_data_addrs.push(cursor as u64);
        cursor += raw_payloads[i].len();
    }

    // Addresses for group structures.
    let lheap_addr = cursor as u64;
    let lheap_data: Vec<u8> = build_local_heap_data(datasets.iter().map(|d| d.name.as_str()));
    let lheap_size = 32 + round_up8(lheap_data.len()); // header (32) + data segment
    cursor += lheap_size;

    let snod_addr = cursor as u64;
    let snod_size = compute_snod_size(datasets.len());
    cursor += snod_size;

    let btree_addr = cursor as u64;
    // B-tree node: "TREE"(4) + type(1) + level(1) + entries_used(2)
    //              + left_sib(8) + right_sib(8) + key[0](8) + child[0](8) + key[1](8) = 48
    let btree_size = 48; // fixed-size TREE node for one SNOD child
    cursor += btree_size;

    let root_ohdr_addr = cursor as u64;
    let root_ohdr_size = compute_group_ohdr_size();
    cursor += root_ohdr_size;

    let eof_addr = cursor as u64;

    // ── Second pass: write everything ────────────────────────────────────
    let mut w = CursorWriter::with_capacity(cursor);

    // Signature
    w.write_bytes(b"\x89HDF\r\n\x1a\n");

    // Superblock v0
    write_superblock_v0(
        &mut w,
        eof_addr,
        root_ohdr_addr,
        btree_addr,
        lheap_addr,
    );

    // Dataset object headers + raw data
    for (i, ds) in datasets.iter().enumerate() {
        let shape_u64: Vec<u64> = ds.shape.iter().map(|&x| x as u64).collect();
        let data_addr = dataset_data_addrs[i];
        let data_size = raw_payloads[i].len() as u64;
        write_dataset_object_header_v1(&mut w, &ds.dtype, &shape_u64, data_addr, data_size);
        w.write_bytes(&raw_payloads[i]);
    }

    // Local heap
    write_local_heap(&mut w, &lheap_data, lheap_size);

    // Build symbol-table entries: (name_heap_offset, ohdr_addr)
    let name_offsets = compute_name_heap_offsets(datasets.iter().map(|d| d.name.as_str()));
    let snod_entries: Vec<(u64, u64)> = name_offsets
        .iter()
        .zip(dataset_ohdr_addrs.iter())
        .map(|(&off, &addr)| (off, addr))
        .collect();
    write_symbol_table_node(&mut w, &snod_entries);

    // B-tree node
    write_btree_node(&mut w, snod_addr, snod_entries.len() as u32);

    // Root group object header
    write_group_object_header(&mut w, btree_addr, lheap_addr);

    // Patch superblock EOF address and all pointer fields — the superblock was
    // written with the correct values already because we computed addresses
    // in the first pass, so no back-patching is needed.

    debug_assert_eq!(w.len(), cursor, "size mismatch between passes");

    w.into_bytes()
}

/// Convenience wrapper: build a minimal single-dataset HDF5 file from a mesh
/// float array, given a dataset name and shape.
pub fn export_mesh_hdf5_bytes(name: &str, shape: &[u64], data: &[f32]) -> Vec<u8> {
    let shape_usize: Vec<usize> = shape.iter().map(|&x| x as usize).collect();
    let mut export = new_hdf5_weights_export("mesh.h5");
    export.root.add_dataset(Hdf5Dataset {
        name: name.to_string(),
        dtype: Hdf5Dtype::Float32,
        shape: shape_usize,
        payload: Hdf5Payload::Float32(data.to_vec()),
    });
    to_hdf5_bytes(&export)
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal helpers — payload serialisation
// ─────────────────────────────────────────────────────────────────────────────

fn payload_to_bytes(payload: &Hdf5Payload) -> Vec<u8> {
    match payload {
        Hdf5Payload::Float32(v) => v.iter().flat_map(|x| x.to_le_bytes()).collect(),
        Hdf5Payload::Float64(v) => v.iter().flat_map(|x| x.to_le_bytes()).collect(),
        Hdf5Payload::Int32(v) => v.iter().flat_map(|x| x.to_le_bytes()).collect(),
        Hdf5Payload::Int64(v) => v.iter().flat_map(|x| x.to_le_bytes()).collect(),
        Hdf5Payload::Uint8(v) => v.clone(),
        Hdf5Payload::Empty => Vec::new(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal helpers — structure writers
// ─────────────────────────────────────────────────────────────────────────────

const UNDEF_ADDR: u64 = 0xFFFF_FFFF_FFFF_FFFF;

/// Write HDF5 superblock v0 (88 bytes following the 8-byte signature).
///
/// ```text
/// u8  version_superblock    = 0
/// u8  version_free_space    = 0
/// u8  version_root_group    = 0
/// u8  reserved              = 0
/// u8  version_shared_hdr    = 0
/// u8  size_of_offsets       = 8
/// u8  size_of_lengths       = 8
/// u8  reserved              = 0
/// u16 leaf_k                = 4
/// u16 internal_k            = 16
/// u32 file_consistency_flags= 0
/// u64 base_address          = 0
/// u64 free_space_address    = UNDEF
/// u64 end_of_file_address
/// u64 driver_info_address   = UNDEF
/// --- root group symbol-table entry (40 bytes) ---
/// u64 link_name_offset      = 0
/// u64 object_header_address
/// u32 cache_type            = 1
/// u32 reserved              = 0
/// u64 b_tree_address
/// u64 local_heap_address
/// ```
fn write_superblock_v0(
    w: &mut CursorWriter,
    eof_addr: u64,
    root_ohdr: u64,
    btree: u64,
    lheap: u64,
) {
    // Version bytes + reserved
    w.write_bytes(&[0u8, 0, 0, 0, 0]); // sb_ver, free_sp_ver, rg_ver, reserved, shared_hdr_ver
    w.write_u8(8); // size_of_offsets
    w.write_u8(8); // size_of_lengths
    w.write_u8(0); // reserved

    w.write_u16_le(4); // leaf_k
    w.write_u16_le(16); // internal_k
    w.write_u32_le(0); // file_consistency_flags

    w.write_u64_le(0); // base_address
    w.write_u64_le(UNDEF_ADDR); // free_space_address
    w.write_u64_le(eof_addr); // end_of_file_address
    w.write_u64_le(UNDEF_ADDR); // driver_info_address

    // Root group symbol-table entry
    w.write_u64_le(0); // link_name_offset (points to "" in local heap)
    w.write_u64_le(root_ohdr); // object_header_address
    w.write_u32_le(1); // cache_type = 1 (group)
    w.write_u32_le(0); // reserved
    w.write_u64_le(btree); // b_tree_address
    w.write_u64_le(lheap); // local_heap_address
}

// ── Datatype message bytes ────────────────────────────────────────────────────

/// Build the fixed-size datatype message body for the given dtype.
///
/// Layout follows HDF5 spec §4.5.2 (version 1 datatype header).
fn datatype_bytes(dtype: &Hdf5Dtype) -> Vec<u8> {
    match dtype {
        // Float32 — class 1 (floating-point), 4 bytes, IEEE 754 little-endian
        // version_class = 0x11  (version=1 in high nibble, class=1 in low nibble)
        // bit_field_0 = 0x20    (byte order LE, see HDF5 spec table 4)
        // size = 4
        // Remaining bytes encode mantissa/exponent positions per spec §4.5.2.1
        Hdf5Dtype::Float32 => vec![
            0x11, 0x20, 0x00, 0x00, // version+class, bit_fields[0..2]
            0x04, 0x00, 0x00, 0x00, // element_size = 4
            0x00, 0x00,             // bit_offset_vtype = 0
            0x17,                   // bit_precision_mantissa = 23
            0x08,                   // exponent_location = 23 wait — stored separately
            0x00, 0x17,             // mantissa_location=0, mantissa_size=23
            0x7f, 0x00, 0x00, 0x00, // exponent_bias = 127
        ],
        // Float64 — class 1, 8 bytes, IEEE 754 double LE
        Hdf5Dtype::Float64 => vec![
            0x11, 0x20, 0x00, 0x00,
            0x08, 0x00, 0x00, 0x00,
            0x00, 0x00,
            0x34,       // mantissa bits = 52
            0x0b,       // exp location = 52, size = 11
            0x00, 0x34,
            0xff, 0x03, 0x00, 0x00, // bias = 1023
        ],
        // Int32 — class 0 (fixed-point), 4 bytes, signed LE
        // version_class = 0x10 (version=1, class=0)
        // bit_field_0 = 0x08   (little-endian, signed, see HDF5 spec table 2)
        Hdf5Dtype::Int32 => vec![
            0x10, 0x08, 0x00, 0x00,
            0x04, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x20, 0x00, 0x00, 0x00, // bit_offset=0, bit_precision=32
        ],
        // Int64 — class 0, 8 bytes, signed LE
        Hdf5Dtype::Int64 => vec![
            0x10, 0x08, 0x00, 0x00,
            0x08, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x40, 0x00, 0x00, 0x00, // precision=64
        ],
        // Uint8 — class 0, 1 byte, unsigned LE
        // bit_field_0 = 0x00 (unsigned)
        Hdf5Dtype::Uint8 => vec![
            0x10, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x08, 0x00, 0x00, 0x00, // precision=8
        ],
    }
}

// ── Dataspace message ─────────────────────────────────────────────────────────

/// Build the dataspace message body.
///
/// ```text
/// u8  version        = 1
/// u8  dimensionality = len(shape)
/// u8  flags          = 0
/// u8  reserved       = 0
/// u32 reserved       = 0
/// [u64 dim_size; ndims]
/// ```
fn write_dataspace_msg(shape: &[u64]) -> Vec<u8> {
    let ndims = shape.len();
    let mut v = Vec::with_capacity(8 + 8 * ndims);
    v.push(1u8); // version
    v.push(ndims as u8); // dimensionality
    v.push(0u8); // flags
    v.push(0u8); // reserved
    v.extend_from_slice(&0u32.to_le_bytes()); // reserved
    for &d in shape {
        v.extend_from_slice(&d.to_le_bytes());
    }
    v
}

// ── Data Layout message (v1, contiguous) ──────────────────────────────────────

/// Build the data layout message body for a contiguous dataset (layout class 1).
///
/// ```text
/// u8  version        = 1
/// u8  ndims          = n_shape_dims + 1  (extra dim = element size)
/// u8  layout_class   = 1
/// u8  reserved       = 0
/// u32 reserved       = 0
/// [u32 dim_size; ndims]  — shape dims followed by element_size
/// u64 data_address
/// u32 data_size
/// ```
fn write_data_layout_msg(
    dtype: &Hdf5Dtype,
    shape: &[u64],
    data_addr: u64,
    data_size: u64,
) -> Vec<u8> {
    let ndims = shape.len() + 1; // +1 for element-size pseudo-dim
    let mut v = Vec::with_capacity(8 + 4 * ndims + 12);
    v.push(1u8); // version
    v.push(ndims as u8); // dimensionality (ndims + 1)
    v.push(1u8); // layout_class = 1 (contiguous)
    v.push(0u8); // reserved
    v.extend_from_slice(&0u32.to_le_bytes()); // reserved

    // Dimension sizes (u32 each)
    for &d in shape {
        v.extend_from_slice(&(d as u32).to_le_bytes());
    }
    // Last pseudo-dimension = element size
    v.extend_from_slice(&(dtype.itemsize() as u32).to_le_bytes());

    // Data address (u64) and size (u32)
    v.extend_from_slice(&data_addr.to_le_bytes());
    v.extend_from_slice(&(data_size as u32).to_le_bytes());
    v
}

// ── Object header v1 helpers ──────────────────────────────────────────────────

/// Round `n` up to the next multiple of 8.
#[inline]
fn round_up8(n: usize) -> usize {
    (n + 7) & !7
}

/// Write a single HDF5 header message (v1 format).
///
/// ```text
/// u16 message_type
/// u16 message_data_size
/// u8  flags = 0
/// u8  reserved[3]
/// [message_data — padded to 8-byte boundary]
/// ```
fn write_header_message(w: &mut CursorWriter, msg_type: u16, data: &[u8]) {
    let padded = round_up8(data.len());
    w.write_u16_le(msg_type);
    w.write_u16_le(padded as u16);
    w.write_u8(0); // flags
    w.write_bytes(&[0u8; 3]); // reserved
    w.write_bytes(data);
    // zero-pad to 8-byte boundary
    let pad = padded - data.len();
    for _ in 0..pad {
        w.write_u8(0);
    }
}

/// Compute total size (bytes) of a single header message including its header
/// and padding.
#[inline]
fn header_message_size(data_len: usize) -> usize {
    8 + round_up8(data_len) // 8-byte msg header + padded data
}

/// Compute the byte size of a dataset object header v1 (including the
/// outer fixed header but excluding padding at the end of the message area).
fn compute_dataset_ohdr_size(shape: &[u64]) -> usize {
    // We use Float32 as a placeholder for sizing; the datatype bytes vary by
    // dtype, so we use the maximum across all types (Int64/Float64 = 14 bytes).
    // However since we are called with a specific shape we use the max size (18).
    // The actual datatype bytes are in 12..18-byte range — we always pad to 8 so
    // every dtype message pads to 16.  Using 18 as the max is safe.
    let ds_size = write_dataspace_msg(shape).len();
    let dt_size = 18; // max datatype message size (Float64 / Int64)
    let dl_size = write_data_layout_msg(&Hdf5Dtype::Float32, shape, 0, 0).len();

    // NIL terminator: one header message of 0 data bytes
    let nil_size = header_message_size(0);

    let msg_area = header_message_size(ds_size)
        + header_message_size(dt_size)
        + header_message_size(dl_size)
        + nil_size;

    // Object header v1: 16-byte fixed header + message area
    16 + msg_area
}

/// Write a dataset object header v1.
fn write_dataset_object_header_v1(
    w: &mut CursorWriter,
    dtype: &Hdf5Dtype,
    shape: &[u64],
    data_addr: u64,
    data_size: u64,
) {
    let ds_data = write_dataspace_msg(shape);
    let dt_data = datatype_bytes(dtype);
    let dl_data = write_data_layout_msg(dtype, shape, data_addr, data_size);

    // We need to know the exact message-area size used so we can write the
    // correct value in the object header preamble.  We use the padded sizes.
    // For the "compute_dataset_ohdr_size" we sized using dt_size=18; the actual
    // dt_data might be shorter, so we pad to the same value so addresses match.
    const MAX_DT_SIZE: usize = 18;
    let dt_padded_extra = MAX_DT_SIZE - dt_data.len(); // extra zeros after real dt_data

    let msg_area_size = header_message_size(ds_data.len())
        + 8
        + round_up8(MAX_DT_SIZE) // datatype message (padded to MAX_DT_SIZE then to 8)
        + header_message_size(dl_data.len())
        + header_message_size(0); // NIL

    // Object header v1 fixed header (16 bytes):
    //   u8  version = 1
    //   u8  reserved = 0
    //   u16 total_number_of_header_messages
    //   u32 object_reference_count = 1
    //   u32 object_header_size   (size of message area)
    //   u32 reserved = 0
    w.write_u8(1); // version
    w.write_u8(0); // reserved
    w.write_u16_le(3); // 3 data messages (dataspace, datatype, layout) + NIL counted separately
    w.write_u32_le(1); // reference count
    w.write_u32_le(msg_area_size as u32);
    w.write_u32_le(0); // reserved

    // Message 1: Dataspace (type 0x0001)
    write_header_message(w, 0x0001, &ds_data);

    // Message 2: Datatype (type 0x0003) — pad to MAX_DT_SIZE
    let mut dt_padded = dt_data.clone();
    dt_padded.resize(MAX_DT_SIZE, 0);
    write_header_message(w, 0x0003, &dt_padded);
    let _ = dt_padded_extra; // accounted for in resize above

    // Message 3: Data Layout (type 0x0008)
    write_header_message(w, 0x0008, &dl_data);

    // NIL message (type 0x0000)
    write_header_message(w, 0x0000, &[]);
}

// ── Local heap ────────────────────────────────────────────────────────────────

/// Build the raw data segment of the local heap: a sequence of null-terminated
/// UTF-8 strings, with a leading empty string at offset 0 (for the root name).
fn build_local_heap_data<'a>(names: impl Iterator<Item = &'a str>) -> Vec<u8> {
    let mut v: Vec<u8> = Vec::new();
    // Offset 0 = empty string (root group name "")
    v.push(0u8);
    for n in names {
        v.extend_from_slice(n.as_bytes());
        v.push(0u8);
    }
    v
}

/// Compute the heap offset of each name inside the data segment.
fn compute_name_heap_offsets<'a>(names: impl Iterator<Item = &'a str>) -> Vec<u64> {
    let mut offsets = Vec::new();
    let mut pos: usize = 1; // skip leading '\0' at offset 0
    for n in names {
        offsets.push(pos as u64);
        pos += n.len() + 1; // name bytes + null terminator
    }
    offsets
}

/// Write a local heap structure.
///
/// ```text
/// "HEAP"                  (4 bytes)
/// u8  version = 0
/// u8  reserved[3]
/// u64 data_segment_size   (aligned to 8)
/// u64 offset_free_list    = UNDEF
/// u64 data_segment_address (immediately after this 32-byte header)
/// [data segment, zero-padded to 8-byte boundary]
/// ```
fn write_local_heap(w: &mut CursorWriter, data: &[u8], total_lheap_size: usize) {
    let ds_size_padded = round_up8(data.len());
    let header_size = 32usize;
    let data_segment_addr = (w.len() + header_size) as u64;

    w.write_bytes(b"HEAP");
    w.write_u8(0); // version
    w.write_bytes(&[0u8; 3]); // reserved
    w.write_u64_le(ds_size_padded as u64);
    w.write_u64_le(UNDEF_ADDR); // free list = UNDEF (no free space)
    w.write_u64_le(data_segment_addr);

    // Data segment
    w.write_bytes(data);
    // Pad data to 8-byte boundary
    let pad = ds_size_padded - data.len();
    for _ in 0..pad {
        w.write_u8(0);
    }

    // Sanity: total_lheap_size should equal header_size + ds_size_padded
    debug_assert_eq!(total_lheap_size, header_size + ds_size_padded);
}

// ── Symbol-table node (SNOD) ──────────────────────────────────────────────────

/// Compute the byte size of a symbol-table node for `n` entries.
///
/// Each SNOD entry is 40 bytes; the node header is 8 bytes.
fn compute_snod_size(n: usize) -> usize {
    8 + 40 * n.max(1) // always at least 1 slot (spec requires non-zero)
}

/// Write a symbol-table node (SNOD).
///
/// ```text
/// "SNOD"       (4 bytes)
/// u8  version = 1
/// u8  reserved = 0
/// u16 num_symbols
/// [Symbol entry; num_symbols]:
///   u64 link_name_offset
///   u64 object_header_address
///   u32 cache_type       = 0 (dataset)
///   u32 reserved         = 0
///   u64 scratch[2]       = 0 (16 bytes of scratch space)
/// ```
fn write_symbol_table_node(w: &mut CursorWriter, entries: &[(u64, u64)]) {
    w.write_bytes(b"SNOD");
    w.write_u8(1); // version
    w.write_u8(0); // reserved
    let n_sym = entries.len().max(1) as u16;
    w.write_u16_le(n_sym);

    for &(name_off, ohdr_addr) in entries {
        w.write_u64_le(name_off); // link_name_offset
        w.write_u64_le(ohdr_addr); // object_header_address
        w.write_u32_le(0); // cache_type = 0 (dataset, no cache)
        w.write_u32_le(0); // reserved
        w.write_u64_le(0); // scratch pad 0
        w.write_u64_le(0); // scratch pad 1
    }

    // If no entries, write one empty slot to satisfy HDF5 minimum node size.
    if entries.is_empty() {
        w.write_u64_le(0);
        w.write_u64_le(UNDEF_ADDR);
        w.write_u32_le(0);
        w.write_u32_le(0);
        w.write_u64_le(0);
        w.write_u64_le(0);
    }
}

// ── B-tree node (TREE) ────────────────────────────────────────────────────────

/// Write a minimal level-0 B-tree node pointing to one symbol-table node.
///
/// ```text
/// "TREE"               (4 bytes)
/// u8  node_type  = 0   (group B-tree)
/// u8  node_level = 0   (leaf)
/// u16 entries_used
/// u64 left_sibling  = UNDEF
/// u64 right_sibling = UNDEF
/// u64 key[0]        = 0
/// u64 child[0]      = snod_addr
/// u64 key[1]        = n_entries (upper bound key)
/// ```
fn write_btree_node(w: &mut CursorWriter, snod_addr: u64, n_entries: u32) {
    w.write_bytes(b"TREE");
    w.write_u8(0); // node_type = 0 (group)
    w.write_u8(0); // node_level = 0 (leaf)
    w.write_u16_le(1); // entries_used = 1 child
    w.write_u64_le(UNDEF_ADDR); // left sibling = undefined
    w.write_u64_le(UNDEF_ADDR); // right sibling = undefined
    // Key-child pairs for 1 child (leaf node with entries_used=1 has 2 keys):
    w.write_u64_le(0); // key[0]
    w.write_u64_le(snod_addr); // child[0]
    w.write_u64_le(n_entries as u64); // key[1]
}

// ── Root group object header ──────────────────────────────────────────────────

/// Compute the size of the root group object header.
fn compute_group_ohdr_size() -> usize {
    // Symbol Table Message (type 0x0011) body: 16 bytes (two u64s: btree + lheap)
    // NIL terminator: 8 header bytes + 0 data
    let group_msg_size = header_message_size(16);
    let nil_size = header_message_size(0);
    16 + group_msg_size + nil_size
}

/// Write the root group object header v1.
///
/// Contains a single "Symbol Table Message" (type 0x0011) that records
/// the B-tree and local heap addresses, followed by a NIL terminator.
fn write_group_object_header(w: &mut CursorWriter, btree_addr: u64, lheap_addr: u64) {
    // Symbol Table Message body:
    //   u64 b_tree_address
    //   u64 local_heap_address
    let mut sym_body = Vec::with_capacity(16);
    sym_body.extend_from_slice(&btree_addr.to_le_bytes());
    sym_body.extend_from_slice(&lheap_addr.to_le_bytes());

    let msg_area_size = header_message_size(sym_body.len()) + header_message_size(0);

    // Object header v1 fixed header (16 bytes)
    w.write_u8(1); // version
    w.write_u8(0); // reserved
    w.write_u16_le(1); // number of header messages
    w.write_u32_le(1); // reference count
    w.write_u32_le(msg_area_size as u32);
    w.write_u32_le(0); // reserved

    // Message: Symbol Table (type 0x0011)
    write_header_message(w, 0x0011, &sym_body);

    // NIL terminator
    write_header_message(w, 0x0000, &[]);
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use std::io::Write;

    fn sample_export() -> Hdf5WeightsExport {
        let mut e = new_hdf5_weights_export("model.h5");
        let mut layer_group = Hdf5Group::new("dense_1");
        layer_group.add_dataset(Hdf5Dataset {
            name: "kernel".into(),
            dtype: Hdf5Dtype::Float32,
            shape: vec![512, 256],
            payload: Hdf5Payload::Empty,
        });
        layer_group.add_dataset(Hdf5Dataset {
            name: "bias".into(),
            dtype: Hdf5Dtype::Float32,
            shape: vec![256],
            payload: Hdf5Payload::Empty,
        });
        layer_group.add_attribute("keras_version", "2.13.0");
        e.root.add_subgroup(layer_group);
        e
    }

    #[test]
    fn filename_stored() {
        let e = new_hdf5_weights_export("weights.h5");
        assert_eq!(e.filename, "weights.h5");
    }

    #[test]
    fn group_count() {
        let e = sample_export();
        assert_eq!(hdf5_group_count(&e), 1);
    }

    #[test]
    fn validate_complete() {
        let e = sample_export();
        assert!(validate_hdf5_weights(&e));
    }

    #[test]
    fn validate_empty_false() {
        let e = new_hdf5_weights_export("model.h5");
        assert!(!validate_hdf5_weights(&e));
    }

    #[test]
    fn data_size_positive() {
        let e = sample_export();
        assert!(hdf5_data_size(&e) > 0);
    }

    #[test]
    fn dtype_itemsize_f64() {
        assert_eq!(Hdf5Dtype::Float64.itemsize(), 8);
    }

    #[test]
    fn dataset_byte_size() {
        let ds = Hdf5Dataset {
            name: "w".into(),
            dtype: Hdf5Dtype::Float32,
            shape: vec![4, 4],
            payload: Hdf5Payload::Empty,
        };
        assert_eq!(ds.byte_size(), 64);
    }

    #[test]
    fn summary_json_has_file() {
        let e = sample_export();
        let json = hdf5_summary_json(&e);
        assert!(json.contains("model.h5"));
    }

    #[test]
    fn attribute_stored() {
        let e = sample_export();
        let g = &e.root.subgroups[0];
        assert_eq!(g.attributes[0].0, "keras_version");
    }

    // ── New binary-encoding tests ────────────────────────────────────────────

    /// Helper: build an export with one Float32 dataset and concrete payload.
    fn single_float32_export() -> Hdf5WeightsExport {
        let mut e = new_hdf5_weights_export("single.h5");
        e.root.add_dataset(Hdf5Dataset {
            name: "weights".into(),
            dtype: Hdf5Dtype::Float32,
            shape: vec![3],
            payload: Hdf5Payload::Float32(vec![1.0f32, 2.0, 3.0]),
        });
        e
    }

    #[test]
    fn hdf5_signature_correct() {
        let e = single_float32_export();
        let bytes = to_hdf5_bytes(&e);
        assert_eq!(&bytes[0..8], b"\x89HDF\r\n\x1a\n",
            "file must start with canonical HDF5 signature");
    }

    #[test]
    fn superblock_version_zero() {
        let e = single_float32_export();
        let bytes = to_hdf5_bytes(&e);
        assert_eq!(bytes[8], 0, "superblock version must be 0");
    }

    #[test]
    fn superblock_size_of_offsets_eight() {
        let e = single_float32_export();
        let bytes = to_hdf5_bytes(&e);
        // offset 8 = superblock start, +5 = size_of_offsets field
        assert_eq!(bytes[8 + 5], 8, "size_of_offsets must be 8");
    }

    #[test]
    fn superblock_size_of_lengths_eight() {
        let e = single_float32_export();
        let bytes = to_hdf5_bytes(&e);
        assert_eq!(bytes[8 + 6], 8, "size_of_lengths must be 8");
    }

    #[test]
    fn float32_payload_present_in_file() {
        let payload_data = [1.0f32, 2.0, 3.0];
        let e = single_float32_export();
        let bytes = to_hdf5_bytes(&e);

        // Build expected raw bytes for [1.0, 2.0, 3.0] in LE
        let expected: Vec<u8> = payload_data.iter()
            .flat_map(|x| x.to_le_bytes())
            .collect();

        let found = bytes
            .windows(expected.len())
            .any(|w| w == expected.as_slice());
        assert!(found, "Float32 payload bytes must be present in the HDF5 file");
    }

    #[test]
    fn empty_export_produces_valid_header() {
        let e = new_hdf5_weights_export("empty.h5");
        let bytes = to_hdf5_bytes(&e);
        // Must have signature + superblock at minimum
        assert!(bytes.len() >= 56, "file must be at least 56 bytes");
        assert_eq!(&bytes[0..8], b"\x89HDF\r\n\x1a\n");
        assert_eq!(bytes[8], 0); // superblock version
    }

    #[test]
    fn multiple_datasets_all_payloads_present() {
        let mut e = new_hdf5_weights_export("multi.h5");
        e.root.add_dataset(Hdf5Dataset {
            name: "a".into(),
            dtype: Hdf5Dtype::Float32,
            shape: vec![2],
            payload: Hdf5Payload::Float32(vec![10.0f32, 20.0]),
        });
        e.root.add_dataset(Hdf5Dataset {
            name: "b".into(),
            dtype: Hdf5Dtype::Int32,
            shape: vec![2],
            payload: Hdf5Payload::Int32(vec![7i32, -3]),
        });

        let bytes = to_hdf5_bytes(&e);

        let exp_a: Vec<u8> = [10.0f32, 20.0].iter().flat_map(|x| x.to_le_bytes()).collect();
        let exp_b: Vec<u8> = [7i32, -3].iter().flat_map(|x| x.to_le_bytes()).collect();

        assert!(bytes.windows(exp_a.len()).any(|w| w == exp_a.as_slice()),
            "dataset 'a' payload not found");
        assert!(bytes.windows(exp_b.len()).any(|w| w == exp_b.as_slice()),
            "dataset 'b' payload not found");
    }

    #[test]
    fn export_mesh_hdf5_bytes_signature() {
        let data = vec![0.0f32; 9]; // 3 vertices * 3 components
        let bytes = export_mesh_hdf5_bytes("positions", &[3, 3], &data);
        assert_eq!(&bytes[0..8], b"\x89HDF\r\n\x1a\n");
    }

    #[test]
    fn export_mesh_hdf5_bytes_writes_temp_file() {
        let data: Vec<f32> = (0..12).map(|i| i as f32 * 0.1).collect();
        let bytes = export_mesh_hdf5_bytes("verts", &[4, 3], &data);

        let mut path = temp_dir();
        path.push("oxihuman_test_mesh.h5");

        let mut f = std::fs::File::create(&path).expect("create temp file");
        f.write_all(&bytes).expect("write bytes");

        let file_meta = std::fs::metadata(&path).expect("metadata");
        assert_eq!(file_meta.len(), bytes.len() as u64);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn payload_to_bytes_float32_roundtrip() {
        let v = vec![1.5f32, -0.25, std::f32::consts::PI];
        let b = payload_to_bytes(&Hdf5Payload::Float32(v.clone()));
        assert_eq!(b.len(), 12);
        for (i, &expected) in v.iter().enumerate() {
            let got = f32::from_le_bytes([b[4*i], b[4*i+1], b[4*i+2], b[4*i+3]]);
            assert!((got - expected).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn payload_to_bytes_int64_roundtrip() {
        let v = vec![i64::MAX, i64::MIN, 0i64];
        let b = payload_to_bytes(&Hdf5Payload::Int64(v.clone()));
        assert_eq!(b.len(), 24);
        for (i, &expected) in v.iter().enumerate() {
            let got = i64::from_le_bytes(b[8*i..8*i+8].try_into().unwrap());
            assert_eq!(got, expected);
        }
    }

    #[test]
    fn payload_empty_is_zero_bytes() {
        let b = payload_to_bytes(&Hdf5Payload::Empty);
        assert!(b.is_empty());
    }

    #[test]
    fn eof_address_matches_file_length() {
        let e = single_float32_export();
        let bytes = to_hdf5_bytes(&e);

        // EOF address is at superblock offset 8+16 = 24 (after base, free_space)
        // superblock starts at offset 8
        // field layout: 5 version bytes + 3 (offsets/lengths/reserved) + 4 (leaf/internal k)
        // + 4 (flags) + 8 (base) + 8 (free_space) = 32 bytes into superblock
        // eof_addr is at superblock_start(8) + 32 = offset 40
        let eof_offset = 8 + 5 + 3 + 4 + 4 + 8 + 8;
        let eof_raw = &bytes[eof_offset..eof_offset + 8];
        let eof = u64::from_le_bytes(eof_raw.try_into().unwrap());
        assert_eq!(eof as usize, bytes.len(),
            "superblock eof_address must equal file length");
    }
}
