#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderTimestamp {
    begin: f64,
    end: f64,
    frame: u64,
    valid: bool,
}

#[allow(dead_code)]
pub fn new_render_timestamp(frame: u64) -> RenderTimestamp {
    RenderTimestamp { begin: 0.0, end: 0.0, frame, valid: false }
}

#[allow(dead_code)]
pub fn timestamp_begin(ts: &mut RenderTimestamp, t: f64) { ts.begin = t; ts.valid = false; }

#[allow(dead_code)]
pub fn timestamp_end(ts: &mut RenderTimestamp, t: f64) { ts.end = t; ts.valid = ts.end >= ts.begin; }

#[allow(dead_code)]
pub fn timestamp_elapsed_ms(ts: &RenderTimestamp) -> f64 {
    if ts.valid { (ts.end - ts.begin) * 1000.0 } else { 0.0 }
}

#[allow(dead_code)]
pub fn timestamp_frame(ts: &RenderTimestamp) -> u64 { ts.frame }

#[allow(dead_code)]
pub fn timestamp_to_json(ts: &RenderTimestamp) -> String {
    format!("{{\"frame\":{},\"elapsed_ms\":{:.4}}}", ts.frame, timestamp_elapsed_ms(ts))
}

#[allow(dead_code)]
pub fn timestamp_reset(ts: &mut RenderTimestamp) { ts.begin = 0.0; ts.end = 0.0; ts.valid = false; }

#[allow(dead_code)]
pub fn timestamp_is_valid(ts: &RenderTimestamp) -> bool { ts.valid }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let t = new_render_timestamp(0); assert!(!timestamp_is_valid(&t)); }
    #[test] fn test_begin_end() { let mut t = new_render_timestamp(1); timestamp_begin(&mut t, 0.0); timestamp_end(&mut t, 0.016); assert!(timestamp_is_valid(&t)); }
    #[test] fn test_elapsed() { let mut t = new_render_timestamp(0); timestamp_begin(&mut t, 0.0); timestamp_end(&mut t, 0.016); assert!((timestamp_elapsed_ms(&t) - 16.0).abs() < 0.1); }
    #[test] fn test_frame() { let t = new_render_timestamp(42); assert_eq!(timestamp_frame(&t), 42); }
    #[test] fn test_json() { let t = new_render_timestamp(0); assert!(timestamp_to_json(&t).contains("frame")); }
    #[test] fn test_reset() { let mut t = new_render_timestamp(0); timestamp_begin(&mut t, 0.0); timestamp_end(&mut t, 1.0); timestamp_reset(&mut t); assert!(!timestamp_is_valid(&t)); }
    #[test] fn test_invalid_no_end() { let mut t = new_render_timestamp(0); timestamp_begin(&mut t, 1.0); assert!(!timestamp_is_valid(&t)); }
    #[test] fn test_elapsed_invalid() { let t = new_render_timestamp(0); assert!((timestamp_elapsed_ms(&t)).abs() < 1e-6); }
    #[test] fn test_end_before_begin() { let mut t = new_render_timestamp(0); timestamp_begin(&mut t, 1.0); timestamp_end(&mut t, 0.5); assert!(!timestamp_is_valid(&t)); }
    #[test] fn test_zero_elapsed() { let mut t = new_render_timestamp(0); timestamp_begin(&mut t, 1.0); timestamp_end(&mut t, 1.0); assert!(timestamp_is_valid(&t)); assert!((timestamp_elapsed_ms(&t)).abs() < 1e-6); }
}
