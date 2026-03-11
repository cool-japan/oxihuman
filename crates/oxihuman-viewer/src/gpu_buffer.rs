// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]
//! GPU buffer abstraction for vertex/index/uniform data.

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum BufferUsage {
    Vertex,
    Index,
    Uniform,
    Storage,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct GpuBuffer {
    data: Vec<u8>,
    usage: BufferUsage,
    mapped: bool,
}

#[allow(dead_code)]
pub fn new_gpu_buffer(usage: BufferUsage, capacity: usize) -> GpuBuffer {
    GpuBuffer {
        data: vec![0_u8; capacity],
        usage,
        mapped: false,
    }
}

#[allow(dead_code)]
pub fn buffer_write(buf: &mut GpuBuffer, offset: usize, data: &[u8]) -> bool {
    if offset + data.len() > buf.data.len() {
        return false;
    }
    buf.data[offset..offset + data.len()].copy_from_slice(data);
    true
}

#[allow(dead_code)]
pub fn buffer_read(buf: &GpuBuffer, offset: usize, len: usize) -> Option<&[u8]> {
    if offset + len > buf.data.len() {
        return None;
    }
    Some(&buf.data[offset..offset + len])
}

#[allow(dead_code)]
pub fn buffer_size_gpu(buf: &GpuBuffer) -> usize {
    buf.data.len()
}

#[allow(dead_code)]
pub fn buffer_usage(buf: &GpuBuffer) -> &BufferUsage {
    &buf.usage
}

#[allow(dead_code)]
pub fn buffer_is_mapped(buf: &GpuBuffer) -> bool {
    buf.mapped
}

#[allow(dead_code)]
pub fn buffer_clear(buf: &mut GpuBuffer) {
    for b in &mut buf.data {
        *b = 0;
    }
}

#[allow(dead_code)]
pub fn buffer_to_json(buf: &GpuBuffer) -> String {
    let usage = match &buf.usage {
        BufferUsage::Vertex => "vertex",
        BufferUsage::Index => "index",
        BufferUsage::Uniform => "uniform",
        BufferUsage::Storage => "storage",
    };
    format!(
        "{{\"usage\":\"{}\",\"size\":{},\"mapped\":{}}}",
        usage,
        buf.data.len(),
        buf.mapped
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_gpu_buffer() {
        let b = new_gpu_buffer(BufferUsage::Vertex, 1024);
        assert_eq!(buffer_size_gpu(&b), 1024);
    }

    #[test]
    fn test_buffer_write() {
        let mut b = new_gpu_buffer(BufferUsage::Vertex, 16);
        assert!(buffer_write(&mut b, 0, &[1, 2, 3, 4]));
    }

    #[test]
    fn test_buffer_write_overflow() {
        let mut b = new_gpu_buffer(BufferUsage::Vertex, 4);
        assert!(!buffer_write(&mut b, 0, &[0; 8]));
    }

    #[test]
    fn test_buffer_read() {
        let mut b = new_gpu_buffer(BufferUsage::Index, 16);
        buffer_write(&mut b, 0, &[10, 20, 30]);
        let data = buffer_read(&b, 0, 3).unwrap();
        assert_eq!(data, &[10, 20, 30]);
    }

    #[test]
    fn test_buffer_read_overflow() {
        let b = new_gpu_buffer(BufferUsage::Vertex, 4);
        assert!(buffer_read(&b, 0, 8).is_none());
    }

    #[test]
    fn test_buffer_usage() {
        let b = new_gpu_buffer(BufferUsage::Uniform, 64);
        assert_eq!(*buffer_usage(&b), BufferUsage::Uniform);
    }

    #[test]
    fn test_buffer_is_mapped() {
        let b = new_gpu_buffer(BufferUsage::Storage, 128);
        assert!(!buffer_is_mapped(&b));
    }

    #[test]
    fn test_buffer_clear() {
        let mut b = new_gpu_buffer(BufferUsage::Vertex, 8);
        buffer_write(&mut b, 0, &[1, 2, 3, 4, 5, 6, 7, 8]);
        buffer_clear(&mut b);
        let data = buffer_read(&b, 0, 4).unwrap();
        assert_eq!(data, &[0, 0, 0, 0]);
    }

    #[test]
    fn test_buffer_to_json() {
        let b = new_gpu_buffer(BufferUsage::Vertex, 256);
        let json = buffer_to_json(&b);
        assert!(json.contains("\"usage\":\"vertex\""));
    }

    #[test]
    fn test_storage_buffer() {
        let b = new_gpu_buffer(BufferUsage::Storage, 512);
        assert_eq!(*buffer_usage(&b), BufferUsage::Storage);
        assert_eq!(buffer_size_gpu(&b), 512);
    }
}
