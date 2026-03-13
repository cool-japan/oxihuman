// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A buffered command that stores an opcode and payload.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct BufferedCommand {
    pub opcode: u32,
    pub payload: Vec<u8>,
}

#[allow(dead_code)]
impl BufferedCommand {
    pub fn new(opcode: u32, payload: Vec<u8>) -> Self {
        Self { opcode, payload }
    }

    pub fn empty(opcode: u32) -> Self {
        Self {
            opcode,
            payload: Vec::new(),
        }
    }

    pub fn payload_size(&self) -> usize {
        self.payload.len()
    }
}

/// A double-buffered command queue: write to one side, flush from the other.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct CommandBuffer {
    write_buf: Vec<BufferedCommand>,
    read_buf: Vec<BufferedCommand>,
    total_flushed: usize,
}

#[allow(dead_code)]
impl CommandBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn submit(&mut self, cmd: BufferedCommand) {
        self.write_buf.push(cmd);
    }

    pub fn submit_opcode(&mut self, opcode: u32) {
        self.write_buf.push(BufferedCommand::empty(opcode));
    }

    pub fn swap(&mut self) {
        std::mem::swap(&mut self.write_buf, &mut self.read_buf);
        self.write_buf.clear();
    }

    pub fn drain_read(&mut self) -> Vec<BufferedCommand> {
        self.total_flushed += self.read_buf.len();
        std::mem::take(&mut self.read_buf)
    }

    pub fn pending_count(&self) -> usize {
        self.write_buf.len()
    }

    pub fn readable_count(&self) -> usize {
        self.read_buf.len()
    }

    pub fn total_flushed(&self) -> usize {
        self.total_flushed
    }

    pub fn is_empty(&self) -> bool {
        self.write_buf.is_empty() && self.read_buf.is_empty()
    }

    pub fn clear_all(&mut self) {
        self.write_buf.clear();
        self.read_buf.clear();
    }

    pub fn pending_payload_bytes(&self) -> usize {
        self.write_buf.iter().map(|c| c.payload_size()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let buf = CommandBuffer::new();
        assert!(buf.is_empty());
    }

    #[test]
    fn test_submit_and_pending() {
        let mut buf = CommandBuffer::new();
        buf.submit_opcode(1);
        buf.submit_opcode(2);
        assert_eq!(buf.pending_count(), 2);
    }

    #[test]
    fn test_swap() {
        let mut buf = CommandBuffer::new();
        buf.submit_opcode(1);
        buf.swap();
        assert_eq!(buf.pending_count(), 0);
        assert_eq!(buf.readable_count(), 1);
    }

    #[test]
    fn test_drain_read() {
        let mut buf = CommandBuffer::new();
        buf.submit_opcode(10);
        buf.swap();
        let cmds = buf.drain_read();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].opcode, 10);
    }

    #[test]
    fn test_total_flushed() {
        let mut buf = CommandBuffer::new();
        buf.submit_opcode(1);
        buf.swap();
        buf.drain_read();
        assert_eq!(buf.total_flushed(), 1);
    }

    #[test]
    fn test_clear_all() {
        let mut buf = CommandBuffer::new();
        buf.submit_opcode(1);
        buf.swap();
        buf.submit_opcode(2);
        buf.clear_all();
        assert!(buf.is_empty());
    }

    #[test]
    fn test_buffered_command_payload() {
        let cmd = BufferedCommand::new(5, vec![1, 2, 3]);
        assert_eq!(cmd.payload_size(), 3);
    }

    #[test]
    fn test_pending_payload_bytes() {
        let mut buf = CommandBuffer::new();
        buf.submit(BufferedCommand::new(1, vec![0; 10]));
        buf.submit(BufferedCommand::new(2, vec![0; 5]));
        assert_eq!(buf.pending_payload_bytes(), 15);
    }

    #[test]
    fn test_swap_clears_write() {
        let mut buf = CommandBuffer::new();
        buf.submit_opcode(1);
        buf.submit_opcode(2);
        buf.swap();
        assert_eq!(buf.pending_count(), 0);
        assert_eq!(buf.readable_count(), 2);
    }

    #[test]
    fn test_double_swap() {
        let mut buf = CommandBuffer::new();
        buf.submit_opcode(1);
        buf.swap();
        buf.submit_opcode(2);
        buf.swap();
        let cmds = buf.drain_read();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].opcode, 2);
    }
}
