// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Fixed-window time series ring buffer.

#[derive(Debug, Clone)]
pub struct TimeSeriesSample {
    pub timestamp: f64,
    pub value: f64,
}

#[derive(Debug, Clone)]
pub struct TimeSeriesBuffer {
    buf: Vec<TimeSeriesSample>,
    capacity: usize,
    head: usize,
    len: usize,
}

impl TimeSeriesBuffer {
    pub fn new(capacity: usize) -> Self {
        TimeSeriesBuffer {
            buf: Vec::with_capacity(capacity),
            capacity,
            head: 0,
            len: 0,
        }
    }

    pub fn push(&mut self, timestamp: f64, value: f64) {
        let sample = TimeSeriesSample { timestamp, value };
        if self.len < self.capacity {
            self.buf.push(sample);
            self.len += 1;
        } else {
            self.buf[self.head] = sample;
            self.head = (self.head + 1) % self.capacity;
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = &TimeSeriesSample> {
        let (a, b) = if self.len < self.capacity {
            (&self.buf[..self.len], &self.buf[0..0])
        } else {
            let (lo, hi) = self.buf.split_at(self.head);
            (hi, lo)
        };
        b.iter().chain(a.iter())
    }

    pub fn mean(&self) -> Option<f64> {
        if self.is_empty() {
            return None;
        }
        let sum: f64 = self.iter().map(|s| s.value).sum();
        Some(sum / self.len as f64)
    }

    pub fn latest(&self) -> Option<&TimeSeriesSample> {
        if self.is_empty() {
            return None;
        }
        if self.len < self.capacity {
            self.buf.last()
        } else {
            let prev = self.head.saturating_sub(1);
            let idx = if self.head == 0 {
                self.capacity - 1
            } else {
                prev
            };
            self.buf.get(idx)
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

pub fn buffer_variance(buf: &TimeSeriesBuffer) -> Option<f64> {
    let mean = buf.mean()?;
    let var = buf.iter().map(|s| (s.value - mean).powi(2)).sum::<f64>() / buf.len() as f64;
    Some(var)
}

pub fn buffer_min_max(buf: &TimeSeriesBuffer) -> Option<(f64, f64)> {
    let mut it = buf.iter();
    let first = it.next()?;
    let (min, max) = it.fold((first.value, first.value), |(mn, mx), s| {
        (mn.min(s.value), mx.max(s.value))
    });
    Some((min, max))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_len() {
        let mut buf = TimeSeriesBuffer::new(5);
        buf.push(1.0, 10.0);
        buf.push(2.0, 20.0);
        assert_eq!(buf.len(), 2);
    }

    #[test]
    fn test_ring_overwrite() {
        let mut buf = TimeSeriesBuffer::new(3);
        for i in 0..5 {
            buf.push(i as f64, i as f64 * 10.0);
        }
        assert_eq!(buf.len(), 3 /* capacity capped at 3 */,);
    }

    #[test]
    fn test_mean_simple() {
        let mut buf = TimeSeriesBuffer::new(4);
        buf.push(1.0, 2.0);
        buf.push(2.0, 4.0);
        buf.push(3.0, 6.0);
        let m = buf.mean().expect("should succeed");
        assert!((m - 4.0).abs() < 1e-10 /* mean of 2,4,6 = 4 */,);
    }

    #[test]
    fn test_empty_mean_none() {
        let buf = TimeSeriesBuffer::new(5);
        assert!(buf.mean().is_none() /* empty buffer has no mean */,);
    }

    #[test]
    fn test_variance() {
        let mut buf = TimeSeriesBuffer::new(4);
        buf.push(0.0, 2.0);
        buf.push(1.0, 4.0);
        buf.push(2.0, 6.0);
        let var = buffer_variance(&buf).expect("should succeed");
        assert!(var > 0.0 /* non-zero variance */,);
    }

    #[test]
    fn test_min_max() {
        let mut buf = TimeSeriesBuffer::new(10);
        buf.push(0.0, 5.0);
        buf.push(1.0, 1.0);
        buf.push(2.0, 9.0);
        let (mn, mx) = buffer_min_max(&buf).expect("should succeed");
        assert_eq!(mn, 1.0);
        assert_eq!(mx, 9.0);
    }

    #[test]
    fn test_is_empty() {
        let buf = TimeSeriesBuffer::new(5);
        assert!(buf.is_empty() /* newly created buffer is empty */,);
    }

    #[test]
    fn test_capacity() {
        let buf = TimeSeriesBuffer::new(8);
        assert_eq!(buf.capacity(), 8);
    }

    #[test]
    fn test_latest_after_overwrite() {
        let mut buf = TimeSeriesBuffer::new(3);
        buf.push(1.0, 10.0);
        buf.push(2.0, 20.0);
        buf.push(3.0, 30.0);
        buf.push(4.0, 40.0);
        let latest = buf.latest().expect("should succeed");
        assert!((latest.value - 40.0).abs() < 1e-10, /* latest should be 40 */);
    }
}
