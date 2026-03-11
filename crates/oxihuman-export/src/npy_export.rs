//! NumPy `.npy` format export stub — serialises `f32` arrays in the npy binary
//! format using a hand-crafted header, without any external dependencies.
//!
//! Only the `float32` (`<f4`) dtype is supported; shape and data are stored
//! according to the NumPy format v1.0 specification.

// ──────────────────────────────────────────────────────────────────────────────
// Constants
// ──────────────────────────────────────────────────────────────────────────────

/// Magic bytes that begin every `.npy` file.
const NPY_MAGIC: &[u8] = b"\x93NUMPY";
/// Format major version.
const NPY_MAJOR: u8 = 1;
/// Format minor version.
const NPY_MINOR: u8 = 0;

// ──────────────────────────────────────────────────────────────────────────────
// Structs
// ──────────────────────────────────────────────────────────────────────────────

/// Configuration for npy export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NpyExportConfig {
    /// Whether to write data in C-order (row-major). Always `true` for this stub.
    pub c_order: bool,
    /// Byte-order character: `'<'` for little-endian.
    pub byte_order: char,
}

/// An in-memory NumPy array of `f32` values.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NpyArray {
    /// Shape of the array (e.g. `[n]`, `[rows, cols]`).
    pub shape: Vec<usize>,
    /// Flat data buffer in C-order.
    pub data: Vec<f32>,
}

// ──────────────────────────────────────────────────────────────────────────────
// Functions
// ──────────────────────────────────────────────────────────────────────────────

/// Return a default [`NpyExportConfig`].
#[allow(dead_code)]
pub fn default_npy_config() -> NpyExportConfig {
    NpyExportConfig {
        c_order: true,
        byte_order: '<',
    }
}

/// Create a new [`NpyArray`] with the given shape and zeroed data.
#[allow(dead_code)]
pub fn new_npy_array(shape: Vec<usize>) -> NpyArray {
    let count: usize = shape.iter().product();
    NpyArray {
        shape,
        data: vec![0.0_f32; count],
    }
}

/// Replace the data buffer of an [`NpyArray`].
#[allow(dead_code)]
pub fn npy_set_data(arr: &mut NpyArray, data: Vec<f32>) {
    arr.data = data;
}

/// Return a reference to the shape slice.
#[allow(dead_code)]
pub fn npy_shape(arr: &NpyArray) -> &[usize] {
    &arr.shape
}

/// Return the total number of elements.
#[allow(dead_code)]
pub fn npy_element_count(arr: &NpyArray) -> usize {
    arr.data.len()
}

/// Return the dtype string as it appears in the npy header (e.g. `"<f4"`).
#[allow(dead_code)]
pub fn npy_dtype_string(_arr: &NpyArray) -> String {
    "<f4".to_string()
}

/// Build the npy header bytes (magic + version + header dict, padded to 64-byte
/// boundary).
#[allow(dead_code)]
pub fn npy_header_bytes(arr: &NpyArray) -> Vec<u8> {
    // Build the Python-style dict string that forms the npy header.
    let shape_str = if arr.shape.len() == 1 {
        format!("({},)", arr.shape[0])
    } else {
        let parts: Vec<String> = arr.shape.iter().map(|d| d.to_string()).collect();
        format!("({})", parts.join(", "))
    };
    let order_char = if arr.shape.len() <= 1 { 'F' } else { 'C' };
    let dict = format!(
        "{{'descr': '<f4', 'fortran_order': {}, 'shape': {}, }}",
        if order_char == 'F' { "True" } else { "False" },
        shape_str
    );

    // Header length must be a multiple of 64 (v1.0 uses 16-byte alignment, but
    // NumPy rounds the whole file header up to a multiple of 64 for mmap compat).
    // Total prefix = 6 (magic) + 1 (major) + 1 (minor) + 2 (HEADER_LEN u16) = 10 bytes.
    let prefix_len = 10_usize;
    let raw_header = format!("{}\n", dict);
    let needed = prefix_len + raw_header.len();
    let padded_len = (needed + 63) & !63;
    let pad = padded_len - needed;
    let padded_header = format!("{}{}", raw_header, " ".repeat(pad));

    let mut out = Vec::with_capacity(padded_len);
    out.extend_from_slice(NPY_MAGIC);
    out.push(NPY_MAJOR);
    out.push(NPY_MINOR);
    // HEADER_LEN as little-endian u16
    let hlen = padded_header.len() as u16;
    out.push((hlen & 0xff) as u8);
    out.push((hlen >> 8) as u8);
    out.extend_from_slice(padded_header.as_bytes());
    out
}

/// Serialise the array to a complete `.npy` byte vector (header + data).
#[allow(dead_code)]
pub fn npy_to_bytes(arr: &NpyArray) -> Vec<u8> {
    let mut out = npy_header_bytes(arr);
    for &v in &arr.data {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

/// Write the array to a `.npy` file at `path`.
#[allow(dead_code)]
pub fn npy_write_to_file(arr: &NpyArray, path: &str) -> Result<(), String> {
    let bytes = npy_to_bytes(arr);
    std::fs::write(path, bytes).map_err(|e| e.to_string())
}

/// Build a 1-D [`NpyArray`] from a morph-weight slice.
#[allow(dead_code)]
pub fn npy_from_morph_weights(weights: &[f32]) -> NpyArray {
    NpyArray {
        shape: vec![weights.len()],
        data: weights.to_vec(),
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_npy_config();
        assert!(cfg.c_order);
        assert_eq!(cfg.byte_order, '<');
    }

    #[test]
    fn test_new_array_zeros() {
        let arr = new_npy_array(vec![3, 4]);
        assert_eq!(npy_element_count(&arr), 12);
        assert!(arr.data.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_set_data() {
        let mut arr = new_npy_array(vec![4]);
        npy_set_data(&mut arr, vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(arr.data[2], 3.0);
    }

    #[test]
    fn test_shape_ref() {
        let arr = new_npy_array(vec![5, 6]);
        assert_eq!(npy_shape(&arr), &[5, 6]);
    }

    #[test]
    fn test_dtype_string() {
        let arr = new_npy_array(vec![1]);
        assert_eq!(npy_dtype_string(&arr), "<f4");
    }

    #[test]
    fn test_header_starts_with_magic() {
        let arr = new_npy_array(vec![3]);
        let hdr = npy_header_bytes(&arr);
        assert!(hdr.starts_with(NPY_MAGIC));
    }

    #[test]
    fn test_header_version() {
        let arr = new_npy_array(vec![3]);
        let hdr = npy_header_bytes(&arr);
        assert_eq!(hdr[6], NPY_MAJOR);
        assert_eq!(hdr[7], NPY_MINOR);
    }

    #[test]
    fn test_to_bytes_length() {
        let arr = new_npy_array(vec![10]);
        let bytes = npy_to_bytes(&arr);
        // Header is padded to multiple of 64; data is 10 * 4 = 40 bytes
        // Total must be > 64
        assert!(bytes.len() > 64);
    }

    #[test]
    fn test_write_to_file() {
        let arr = npy_from_morph_weights(&[0.1, 0.5, 0.9]);
        let path = "/tmp/npy_export_test_oxihuman.npy";
        assert!(npy_write_to_file(&arr, path).is_ok());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_from_morph_weights() {
        let weights = [0.0f32, 0.25, 0.5, 0.75, 1.0];
        let arr = npy_from_morph_weights(&weights);
        assert_eq!(arr.shape, vec![5]);
        assert_eq!(arr.data.len(), 5);
        assert!((arr.data[3] - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_header_padded_to_64() {
        let arr = new_npy_array(vec![2, 3]);
        let hdr = npy_header_bytes(&arr);
        assert_eq!(hdr.len() % 64, 0);
    }
}
