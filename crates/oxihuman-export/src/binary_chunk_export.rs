#![allow(dead_code)]
//! Export binary chunk data.

/// A binary chunk header.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkHeader {
    pub chunk_type: u32,
    pub chunk_size: u32,
}

/// A binary chunk.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryChunk {
    pub header: ChunkHeader,
    pub data: Vec<u8>,
}

/// Create a new binary chunk.
#[allow(dead_code)]
pub fn new_binary_chunk(chunk_type: u32, data: Vec<u8>) -> BinaryChunk {
    let size = data.len() as u32;
    BinaryChunk {
        header: ChunkHeader { chunk_type, chunk_size: size },
        data,
    }
}

/// Write data to a chunk (append).
#[allow(dead_code)]
pub fn chunk_write(chunk: &mut BinaryChunk, data: &[u8]) {
    chunk.data.extend_from_slice(data);
    chunk.header.chunk_size = chunk.data.len() as u32;
}

/// Get the data size of a chunk.
#[allow(dead_code)]
pub fn chunk_size(chunk: &BinaryChunk) -> u32 {
    chunk.header.chunk_size
}

/// Get the chunk type.
#[allow(dead_code)]
pub fn chunk_type(chunk: &BinaryChunk) -> u32 {
    chunk.header.chunk_type
}

/// Convert chunk to bytes (header + data).
#[allow(dead_code)]
pub fn chunk_to_bytes(chunk: &BinaryChunk) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(8 + chunk.data.len());
    bytes.extend_from_slice(&chunk.header.chunk_type.to_le_bytes());
    bytes.extend_from_slice(&chunk.header.chunk_size.to_le_bytes());
    bytes.extend_from_slice(&chunk.data);
    bytes
}

/// Parse a chunk from bytes.
#[allow(dead_code)]
pub fn chunk_from_bytes(bytes: &[u8]) -> Option<BinaryChunk> {
    if bytes.len() < 8 {
        return None;
    }
    let ct = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let cs = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    let end = 8 + cs as usize;
    if bytes.len() < end {
        return None;
    }
    Some(BinaryChunk {
        header: ChunkHeader { chunk_type: ct, chunk_size: cs },
        data: bytes[8..end].to_vec(),
    })
}

/// Get the header size in bytes.
#[allow(dead_code)]
pub fn chunk_header_size() -> usize {
    8
}

/// Validate a chunk (size must match data length).
#[allow(dead_code)]
pub fn validate_chunk(chunk: &BinaryChunk) -> bool {
    chunk.header.chunk_size == chunk.data.len() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_binary_chunk() {
        let c = new_binary_chunk(1, vec![0xAA, 0xBB]);
        assert_eq!(chunk_type(&c), 1);
        assert_eq!(chunk_size(&c), 2);
    }

    #[test]
    fn test_chunk_write() {
        let mut c = new_binary_chunk(1, vec![]);
        chunk_write(&mut c, &[1, 2, 3]);
        assert_eq!(chunk_size(&c), 3);
    }

    #[test]
    fn test_chunk_to_bytes() {
        let c = new_binary_chunk(1, vec![0xFF]);
        let bytes = chunk_to_bytes(&c);
        assert_eq!(bytes.len(), 9);
    }

    #[test]
    fn test_chunk_from_bytes() {
        let c = new_binary_chunk(42, vec![1, 2, 3, 4]);
        let bytes = chunk_to_bytes(&c);
        let parsed = chunk_from_bytes(&bytes);
        assert!(parsed.is_some());
        let parsed = parsed.expect("should succeed");
        assert_eq!(chunk_type(&parsed), 42);
        assert_eq!(parsed.data, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_chunk_from_bytes_too_short() {
        assert!(chunk_from_bytes(&[0, 1, 2]).is_none());
    }

    #[test]
    fn test_chunk_header_size() {
        assert_eq!(chunk_header_size(), 8);
    }

    #[test]
    fn test_validate_chunk() {
        let c = new_binary_chunk(1, vec![1, 2]);
        assert!(validate_chunk(&c));
    }

    #[test]
    fn test_validate_chunk_bad() {
        let c = BinaryChunk {
            header: ChunkHeader { chunk_type: 1, chunk_size: 99 },
            data: vec![1],
        };
        assert!(!validate_chunk(&c));
    }

    #[test]
    fn test_chunk_roundtrip() {
        let original = new_binary_chunk(100, vec![10, 20, 30]);
        let bytes = chunk_to_bytes(&original);
        let restored = chunk_from_bytes(&bytes).expect("should succeed");
        assert_eq!(original, restored);
    }

    #[test]
    fn test_empty_chunk() {
        let c = new_binary_chunk(0, vec![]);
        assert_eq!(chunk_size(&c), 0);
        assert!(validate_chunk(&c));
    }
}
