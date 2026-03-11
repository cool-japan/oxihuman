//! Streaming data buffer — circular FIFO buffer for byte streams with read/write cursors.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StreamBufferConfig {
    pub capacity: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StreamBuffer {
    pub config: StreamBufferConfig,
    data: Vec<u8>,
    read_pos: usize,
    write_pos: usize,
    len: usize,
}

#[allow(dead_code)]
pub fn default_stream_buffer_config() -> StreamBufferConfig {
    StreamBufferConfig { capacity: 4096 }
}

#[allow(dead_code)]
pub fn new_stream_buffer(cfg: &StreamBufferConfig) -> StreamBuffer {
    StreamBuffer {
        data: vec![0u8; cfg.capacity],
        config: cfg.clone(),
        read_pos: 0,
        write_pos: 0,
        len: 0,
    }
}

/// Write bytes into the circular buffer. Returns the number of bytes actually written.
#[allow(dead_code)]
pub fn stream_write(buf: &mut StreamBuffer, data: &[u8]) -> usize {
    let cap = buf.config.capacity;
    let space = cap - buf.len;
    let n = data.len().min(space);
    for (i, &byte) in data.iter().enumerate().take(n) {
        let idx = (buf.write_pos + i) % cap;
        buf.data[idx] = byte;
    }
    buf.write_pos = (buf.write_pos + n) % cap;
    buf.len += n;
    n
}

/// Read bytes from the circular buffer into `out`. Returns the number of bytes read.
#[allow(dead_code)]
pub fn stream_read(buf: &mut StreamBuffer, out: &mut [u8]) -> usize {
    let n = out.len().min(buf.len);
    let cap = buf.config.capacity;
    for (i, slot) in out.iter_mut().enumerate().take(n) {
        *slot = buf.data[(buf.read_pos + i) % cap];
    }
    buf.read_pos = (buf.read_pos + n) % cap;
    buf.len -= n;
    n
}

/// Peek bytes without advancing the read cursor.
#[allow(dead_code)]
pub fn stream_peek(buf: &StreamBuffer, out: &mut [u8]) -> usize {
    let n = out.len().min(buf.len);
    let cap = buf.config.capacity;
    for (i, slot) in out.iter_mut().enumerate().take(n) {
        *slot = buf.data[(buf.read_pos + i) % cap];
    }
    n
}

/// Number of bytes currently available to read.
#[allow(dead_code)]
pub fn stream_available(buf: &StreamBuffer) -> usize {
    buf.len
}

/// Total capacity of the buffer.
#[allow(dead_code)]
pub fn stream_capacity(buf: &StreamBuffer) -> usize {
    buf.config.capacity
}

#[allow(dead_code)]
pub fn stream_is_empty(buf: &StreamBuffer) -> bool {
    buf.len == 0
}

#[allow(dead_code)]
pub fn stream_is_full(buf: &StreamBuffer) -> bool {
    buf.len == buf.config.capacity
}

#[allow(dead_code)]
pub fn stream_clear(buf: &mut StreamBuffer) {
    buf.read_pos = 0;
    buf.write_pos = 0;
    buf.len = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_stream_buffer_config();
        assert_eq!(cfg.capacity, 4096);
    }

    #[test]
    fn test_new_buffer_empty() {
        let cfg = default_stream_buffer_config();
        let buf = new_stream_buffer(&cfg);
        assert!(stream_is_empty(&buf));
        assert_eq!(stream_available(&buf), 0);
    }

    #[test]
    fn test_write_and_read() {
        let cfg = StreamBufferConfig { capacity: 16 };
        let mut buf = new_stream_buffer(&cfg);
        let written = stream_write(&mut buf, &[1, 2, 3, 4]);
        assert_eq!(written, 4);
        assert_eq!(stream_available(&buf), 4);

        let mut out = [0u8; 4];
        let read = stream_read(&mut buf, &mut out);
        assert_eq!(read, 4);
        assert_eq!(out, [1, 2, 3, 4]);
        assert!(stream_is_empty(&buf));
    }

    #[test]
    fn test_write_overflow_returns_partial() {
        let cfg = StreamBufferConfig { capacity: 4 };
        let mut buf = new_stream_buffer(&cfg);
        let n = stream_write(&mut buf, &[1, 2, 3, 4, 5, 6]);
        assert_eq!(n, 4);
        assert!(stream_is_full(&buf));
    }

    #[test]
    fn test_peek_does_not_consume() {
        let cfg = StreamBufferConfig { capacity: 8 };
        let mut buf = new_stream_buffer(&cfg);
        stream_write(&mut buf, &[10, 20, 30]);
        let mut peek = [0u8; 2];
        let peeked = stream_peek(&buf, &mut peek);
        assert_eq!(peeked, 2);
        assert_eq!(peek, [10, 20]);
        assert_eq!(stream_available(&buf), 3);
    }

    #[test]
    fn test_wrap_around() {
        let cfg = StreamBufferConfig { capacity: 4 };
        let mut buf = new_stream_buffer(&cfg);
        stream_write(&mut buf, &[1, 2, 3]);
        let mut tmp = [0u8; 2];
        stream_read(&mut buf, &mut tmp);
        // now read_pos=2, write_pos=3, len=1
        stream_write(&mut buf, &[4, 5]);
        // wraps around
        let mut out = [0u8; 3];
        let n = stream_read(&mut buf, &mut out);
        assert_eq!(n, 3);
        assert_eq!(&out[..3], &[3, 4, 5]);
    }

    #[test]
    fn test_clear() {
        let cfg = StreamBufferConfig { capacity: 8 };
        let mut buf = new_stream_buffer(&cfg);
        stream_write(&mut buf, &[1, 2, 3]);
        stream_clear(&mut buf);
        assert!(stream_is_empty(&buf));
        assert_eq!(stream_available(&buf), 0);
    }

    #[test]
    fn test_capacity() {
        let cfg = StreamBufferConfig { capacity: 32 };
        let buf = new_stream_buffer(&cfg);
        assert_eq!(stream_capacity(&buf), 32);
    }
}
