//! Dynamic byte buffer with cursor-based read/write operations.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ByteBuffer {
    pub data: Vec<u8>,
    pub read_pos: usize,
    pub write_pos: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ByteBufferError {
    pub message: String,
}

#[allow(dead_code)]
pub fn new_byte_buffer() -> ByteBuffer {
    ByteBuffer {
        data: Vec::new(),
        read_pos: 0,
        write_pos: 0,
    }
}

#[allow(dead_code)]
pub fn byte_buffer_with_capacity(cap: usize) -> ByteBuffer {
    ByteBuffer {
        data: Vec::with_capacity(cap),
        read_pos: 0,
        write_pos: 0,
    }
}

#[allow(dead_code)]
pub fn bb_write_u8(buf: &mut ByteBuffer, v: u8) {
    buf.data.push(v);
    buf.write_pos += 1;
}

#[allow(dead_code)]
pub fn bb_write_u16(buf: &mut ByteBuffer, v: u16) {
    let bytes = v.to_le_bytes();
    buf.data.extend_from_slice(&bytes);
    buf.write_pos += 2;
}

#[allow(dead_code)]
pub fn bb_write_u32(buf: &mut ByteBuffer, v: u32) {
    let bytes = v.to_le_bytes();
    buf.data.extend_from_slice(&bytes);
    buf.write_pos += 4;
}

#[allow(dead_code)]
pub fn bb_write_f32(buf: &mut ByteBuffer, v: f32) {
    let bytes = v.to_le_bytes();
    buf.data.extend_from_slice(&bytes);
    buf.write_pos += 4;
}

#[allow(dead_code)]
pub fn bb_write_bytes(buf: &mut ByteBuffer, data: &[u8]) {
    let n = data.len();
    buf.data.extend_from_slice(data);
    buf.write_pos += n;
}

#[allow(dead_code)]
pub fn bb_read_u8(buf: &mut ByteBuffer) -> Result<u8, ByteBufferError> {
    if buf.read_pos + 1 > buf.data.len() {
        return Err(ByteBufferError {
            message: String::from("Not enough bytes to read u8"),
        });
    }
    let v = buf.data[buf.read_pos];
    buf.read_pos += 1;
    Ok(v)
}

#[allow(dead_code)]
pub fn bb_read_u16(buf: &mut ByteBuffer) -> Result<u16, ByteBufferError> {
    if buf.read_pos + 2 > buf.data.len() {
        return Err(ByteBufferError {
            message: String::from("Not enough bytes to read u16"),
        });
    }
    let bytes: [u8; 2] = [buf.data[buf.read_pos], buf.data[buf.read_pos + 1]];
    buf.read_pos += 2;
    Ok(u16::from_le_bytes(bytes))
}

#[allow(dead_code)]
pub fn bb_read_u32(buf: &mut ByteBuffer) -> Result<u32, ByteBufferError> {
    if buf.read_pos + 4 > buf.data.len() {
        return Err(ByteBufferError {
            message: String::from("Not enough bytes to read u32"),
        });
    }
    let bytes: [u8; 4] = [
        buf.data[buf.read_pos],
        buf.data[buf.read_pos + 1],
        buf.data[buf.read_pos + 2],
        buf.data[buf.read_pos + 3],
    ];
    buf.read_pos += 4;
    Ok(u32::from_le_bytes(bytes))
}

#[allow(dead_code)]
pub fn bb_read_f32(buf: &mut ByteBuffer) -> Result<f32, ByteBufferError> {
    if buf.read_pos + 4 > buf.data.len() {
        return Err(ByteBufferError {
            message: String::from("Not enough bytes to read f32"),
        });
    }
    let bytes: [u8; 4] = [
        buf.data[buf.read_pos],
        buf.data[buf.read_pos + 1],
        buf.data[buf.read_pos + 2],
        buf.data[buf.read_pos + 3],
    ];
    buf.read_pos += 4;
    Ok(f32::from_le_bytes(bytes))
}

#[allow(dead_code)]
pub fn bb_read_bytes(buf: &mut ByteBuffer, n: usize) -> Result<Vec<u8>, ByteBufferError> {
    if buf.read_pos + n > buf.data.len() {
        return Err(ByteBufferError {
            message: format!(
                "Not enough bytes: need {}, have {}",
                n,
                buf.data.len() - buf.read_pos
            ),
        });
    }
    let v = buf.data[buf.read_pos..buf.read_pos + n].to_vec();
    buf.read_pos += n;
    Ok(v)
}

#[allow(dead_code)]
pub fn bb_remaining(buf: &ByteBuffer) -> usize {
    buf.data.len().saturating_sub(buf.read_pos)
}

#[allow(dead_code)]
pub fn bb_len(buf: &ByteBuffer) -> usize {
    buf.data.len()
}

#[allow(dead_code)]
pub fn bb_reset(buf: &mut ByteBuffer) {
    buf.data.clear();
    buf.read_pos = 0;
    buf.write_pos = 0;
}

#[allow(dead_code)]
pub fn bb_to_vec(buf: &ByteBuffer) -> Vec<u8> {
    buf.data.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_byte_buffer_empty() {
        let buf = new_byte_buffer();
        assert_eq!(bb_len(&buf), 0);
        assert_eq!(bb_remaining(&buf), 0);
    }

    #[test]
    fn test_write_read_u8_roundtrip() {
        let mut buf = new_byte_buffer();
        bb_write_u8(&mut buf, 42);
        assert_eq!(bb_read_u8(&mut buf).expect("should succeed"), 42);
    }

    #[test]
    fn test_write_read_u16_roundtrip() {
        let mut buf = new_byte_buffer();
        bb_write_u16(&mut buf, 1234);
        assert_eq!(bb_read_u16(&mut buf).expect("should succeed"), 1234);
    }

    #[test]
    fn test_write_read_u32_roundtrip() {
        let mut buf = new_byte_buffer();
        bb_write_u32(&mut buf, 0xDEAD_BEEF);
        assert_eq!(bb_read_u32(&mut buf).expect("should succeed"), 0xDEAD_BEEF);
    }

    #[test]
    fn test_write_read_f32_roundtrip() {
        let mut buf = new_byte_buffer();
        bb_write_f32(&mut buf, std::f32::consts::PI);
        let v = bb_read_f32(&mut buf).expect("should succeed");
        assert!((v - std::f32::consts::PI).abs() < 1e-5);
    }

    #[test]
    fn test_write_read_bytes_roundtrip() {
        let mut buf = new_byte_buffer();
        let data = vec![10u8, 20, 30, 40];
        bb_write_bytes(&mut buf, &data);
        let out = bb_read_bytes(&mut buf, 4).expect("should succeed");
        assert_eq!(out, data);
    }

    #[test]
    fn test_bb_remaining_updates() {
        let mut buf = new_byte_buffer();
        bb_write_u8(&mut buf, 1);
        bb_write_u8(&mut buf, 2);
        bb_write_u8(&mut buf, 3);
        assert_eq!(bb_remaining(&buf), 3);
        bb_read_u8(&mut buf).expect("should succeed");
        assert_eq!(bb_remaining(&buf), 2);
    }

    #[test]
    fn test_bb_read_u8_error_when_empty() {
        let mut buf = new_byte_buffer();
        assert!(bb_read_u8(&mut buf).is_err());
    }

    #[test]
    fn test_bb_read_u16_error_on_short() {
        let mut buf = new_byte_buffer();
        bb_write_u8(&mut buf, 0xFF);
        assert!(bb_read_u16(&mut buf).is_err());
    }

    #[test]
    fn test_bb_reset_clears_all() {
        let mut buf = new_byte_buffer();
        bb_write_u32(&mut buf, 999);
        bb_reset(&mut buf);
        assert_eq!(bb_len(&buf), 0);
        assert_eq!(buf.read_pos, 0);
        assert_eq!(buf.write_pos, 0);
    }

    #[test]
    fn test_bb_to_vec() {
        let mut buf = new_byte_buffer();
        bb_write_u8(&mut buf, 7);
        bb_write_u8(&mut buf, 8);
        let v = bb_to_vec(&buf);
        assert_eq!(v, vec![7, 8]);
    }

    #[test]
    fn test_byte_buffer_with_capacity() {
        let buf = byte_buffer_with_capacity(64);
        assert_eq!(bb_len(&buf), 0);
        assert!(buf.data.capacity() >= 64);
    }

    #[test]
    fn test_sequential_writes_and_reads() {
        let mut buf = new_byte_buffer();
        bb_write_u8(&mut buf, 5);
        bb_write_u16(&mut buf, 300);
        bb_write_u32(&mut buf, 70000);
        assert_eq!(bb_read_u8(&mut buf).expect("should succeed"), 5);
        assert_eq!(bb_read_u16(&mut buf).expect("should succeed"), 300);
        assert_eq!(bb_read_u32(&mut buf).expect("should succeed"), 70000);
        assert_eq!(bb_remaining(&buf), 0);
    }
}
