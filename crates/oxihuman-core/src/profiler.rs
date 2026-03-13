// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Simple performance profiler with named sections.

#[allow(dead_code)]
pub struct ProfileSpan {
    pub name: String,
    pub start_ns: u64,
    pub end_ns: u64,
    pub depth: u32,
}

#[allow(dead_code)]
pub struct ProfileFrame {
    pub spans: Vec<ProfileSpan>,
    pub frame_number: u64,
    pub total_ns: u64,
}

#[allow(dead_code)]
pub struct Profiler {
    pub frames: Vec<ProfileFrame>,
    pub current_frame: ProfileFrame,
    pub stack: Vec<String>,
    pub max_frames: usize,
    pub enabled: bool,
    pub frame_counter: u64,
    pub simulated_ns: u64,
}

#[allow(dead_code)]
pub fn new_profiler(max_frames: usize) -> Profiler {
    Profiler {
        frames: Vec::new(),
        current_frame: ProfileFrame {
            spans: Vec::new(),
            frame_number: 0,
            total_ns: 0,
        },
        stack: Vec::new(),
        max_frames,
        enabled: true,
        frame_counter: 0,
        simulated_ns: 0,
    }
}

#[allow(dead_code)]
pub fn begin_span(profiler: &mut Profiler, name: &str) {
    if !profiler.enabled {
        return;
    }
    profiler.stack.push(name.to_string());
    // We store the start time via simulated_ns for testing; increment it slightly
    profiler.simulated_ns += 1;
    // We will record start_ns when we actually create the span in end_span.
    // For tracking start, we push a marker into current_frame with the start time.
    profiler.current_frame.spans.push(ProfileSpan {
        name: name.to_string(),
        start_ns: profiler.simulated_ns,
        end_ns: 0,
        depth: (profiler.stack.len() as u32).saturating_sub(1),
    });
}

#[allow(dead_code)]
pub fn end_span(profiler: &mut Profiler) {
    if !profiler.enabled {
        return;
    }
    if profiler.stack.is_empty() {
        return;
    }
    let Some(name) = profiler.stack.pop() else {
        return;
    };
    profiler.simulated_ns += 1;
    let end_ns = profiler.simulated_ns;
    // Find last unclosed span with this name
    if let Some(span) = profiler
        .current_frame
        .spans
        .iter_mut()
        .rev()
        .find(|s| s.name == name && s.end_ns == 0)
    {
        span.end_ns = end_ns;
    }
}

#[allow(dead_code)]
pub fn end_frame(profiler: &mut Profiler) {
    if !profiler.enabled {
        return;
    }
    let frame_number = profiler.frame_counter;
    let total_ns = profiler
        .current_frame
        .spans
        .iter()
        .filter(|s| s.depth == 0)
        .map(span_duration_ns)
        .sum();
    let frame = ProfileFrame {
        spans: std::mem::take(&mut profiler.current_frame.spans),
        frame_number,
        total_ns,
    };
    profiler.frames.push(frame);
    if profiler.frames.len() > profiler.max_frames {
        profiler.frames.remove(0);
    }
    profiler.frame_counter += 1;
    profiler.current_frame = ProfileFrame {
        spans: Vec::new(),
        frame_number: profiler.frame_counter,
        total_ns: 0,
    };
    profiler.stack.clear();
}

#[allow(dead_code)]
pub fn frame_count_profiler(profiler: &Profiler) -> usize {
    profiler.frames.len()
}

#[allow(dead_code)]
pub fn last_frame(profiler: &Profiler) -> Option<&ProfileFrame> {
    profiler.frames.last()
}

#[allow(dead_code)]
pub fn span_by_name<'a>(frame: &'a ProfileFrame, name: &str) -> Option<&'a ProfileSpan> {
    frame.spans.iter().find(|s| s.name == name)
}

#[allow(dead_code)]
pub fn total_frame_ns(frame: &ProfileFrame) -> u64 {
    frame.total_ns
}

#[allow(dead_code)]
pub fn average_frame_ns(profiler: &Profiler) -> u64 {
    if profiler.frames.is_empty() {
        return 0;
    }
    let total: u64 = profiler.frames.iter().map(|f| f.total_ns).sum();
    total / profiler.frames.len() as u64
}

#[allow(dead_code)]
pub fn span_duration_ns(span: &ProfileSpan) -> u64 {
    span.end_ns.saturating_sub(span.start_ns)
}

#[allow(dead_code)]
pub fn hottest_span(frame: &ProfileFrame) -> Option<&ProfileSpan> {
    frame.spans.iter().max_by_key(|s| span_duration_ns(s))
}

#[allow(dead_code)]
pub fn profiler_to_json(profiler: &Profiler) -> String {
    let mut out = String::from("{\"frames\":[");
    for (fi, frame) in profiler.frames.iter().enumerate() {
        if fi > 0 {
            out.push(',');
        }
        out.push_str(&format!(
            "{{\"frame_number\":{},\"total_ns\":{},\"spans\":[",
            frame.frame_number, frame.total_ns
        ));
        for (si, span) in frame.spans.iter().enumerate() {
            if si > 0 {
                out.push(',');
            }
            out.push_str(&format!(
                "{{\"name\":\"{}\",\"start_ns\":{},\"end_ns\":{},\"depth\":{}}}",
                span.name, span.start_ns, span.end_ns, span.depth
            ));
        }
        out.push_str("]}");
    }
    out.push_str("]}");
    out
}

#[allow(dead_code)]
pub fn enable_profiler(profiler: &mut Profiler) {
    profiler.enabled = true;
}

#[allow(dead_code)]
pub fn disable_profiler(profiler: &mut Profiler) {
    profiler.enabled = false;
}

#[allow(dead_code)]
pub fn clear_profiler(profiler: &mut Profiler) {
    profiler.frames.clear();
    profiler.current_frame.spans.clear();
    profiler.stack.clear();
    profiler.frame_counter = 0;
    profiler.simulated_ns = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_frame_with_span(name: &str, start: u64, end: u64) -> ProfileFrame {
        ProfileFrame {
            spans: vec![ProfileSpan {
                name: name.to_string(),
                start_ns: start,
                end_ns: end,
                depth: 0,
            }],
            frame_number: 0,
            total_ns: end - start,
        }
    }

    #[test]
    fn test_new_profiler() {
        let p = new_profiler(10);
        assert_eq!(p.max_frames, 10);
        assert!(p.enabled);
        assert!(p.frames.is_empty());
        assert_eq!(p.frame_counter, 0);
    }

    #[test]
    fn test_begin_end_span() {
        let mut p = new_profiler(10);
        begin_span(&mut p, "render");
        end_span(&mut p);
        assert!(!p.current_frame.spans.is_empty());
        let span = &p.current_frame.spans[0];
        assert_eq!(span.name, "render");
        assert!(span.end_ns > span.start_ns);
    }

    #[test]
    fn test_end_frame() {
        let mut p = new_profiler(10);
        begin_span(&mut p, "update");
        end_span(&mut p);
        end_frame(&mut p);
        assert_eq!(frame_count_profiler(&p), 1);
        assert_eq!(p.frame_counter, 1);
    }

    #[test]
    fn test_frame_count() {
        let mut p = new_profiler(10);
        for _ in 0..3 {
            begin_span(&mut p, "x");
            end_span(&mut p);
            end_frame(&mut p);
        }
        assert_eq!(frame_count_profiler(&p), 3);
    }

    #[test]
    fn test_max_frames_limit() {
        let mut p = new_profiler(2);
        for _ in 0..5 {
            begin_span(&mut p, "x");
            end_span(&mut p);
            end_frame(&mut p);
        }
        assert!(frame_count_profiler(&p) <= 2);
    }

    #[test]
    fn test_last_frame() {
        let mut p = new_profiler(10);
        assert!(last_frame(&p).is_none());
        begin_span(&mut p, "a");
        end_span(&mut p);
        end_frame(&mut p);
        assert!(last_frame(&p).is_some());
    }

    #[test]
    fn test_span_by_name() {
        let frame = make_frame_with_span("physics", 100, 200);
        let span = span_by_name(&frame, "physics");
        assert!(span.is_some());
        assert_eq!(span.expect("should succeed").name, "physics");
        assert!(span_by_name(&frame, "missing").is_none());
    }

    #[test]
    fn test_span_duration_ns() {
        let span = ProfileSpan {
            name: "x".to_string(),
            start_ns: 100,
            end_ns: 250,
            depth: 0,
        };
        assert_eq!(span_duration_ns(&span), 150);
    }

    #[test]
    fn test_hottest_span() {
        let frame = ProfileFrame {
            spans: vec![
                ProfileSpan {
                    name: "a".to_string(),
                    start_ns: 0,
                    end_ns: 50,
                    depth: 0,
                },
                ProfileSpan {
                    name: "b".to_string(),
                    start_ns: 50,
                    end_ns: 200,
                    depth: 0,
                },
                ProfileSpan {
                    name: "c".to_string(),
                    start_ns: 200,
                    end_ns: 210,
                    depth: 0,
                },
            ],
            frame_number: 0,
            total_ns: 210,
        };
        let hot = hottest_span(&frame).expect("should succeed");
        assert_eq!(hot.name, "b");
    }

    #[test]
    fn test_average_frame_ns() {
        let mut p = new_profiler(10);
        assert_eq!(average_frame_ns(&p), 0);
        // Manually push frames for deterministic testing
        p.frames.push(ProfileFrame {
            spans: vec![],
            frame_number: 0,
            total_ns: 100,
        });
        p.frames.push(ProfileFrame {
            spans: vec![],
            frame_number: 1,
            total_ns: 200,
        });
        assert_eq!(average_frame_ns(&p), 150);
    }

    #[test]
    fn test_enable_disable() {
        let mut p = new_profiler(10);
        disable_profiler(&mut p);
        assert!(!p.enabled);
        begin_span(&mut p, "x");
        end_span(&mut p);
        end_frame(&mut p);
        assert_eq!(frame_count_profiler(&p), 0);
        enable_profiler(&mut p);
        assert!(p.enabled);
    }

    #[test]
    fn test_clear_profiler() {
        let mut p = new_profiler(10);
        begin_span(&mut p, "a");
        end_span(&mut p);
        end_frame(&mut p);
        clear_profiler(&mut p);
        assert_eq!(frame_count_profiler(&p), 0);
        assert_eq!(p.frame_counter, 0);
    }

    #[test]
    fn test_profiler_to_json() {
        let mut p = new_profiler(10);
        begin_span(&mut p, "render");
        end_span(&mut p);
        end_frame(&mut p);
        let json = profiler_to_json(&p);
        assert!(json.contains("render"));
        assert!(json.contains("frames"));
    }

    #[test]
    fn test_end_span_without_begin() {
        let mut p = new_profiler(10);
        // Should not panic
        end_span(&mut p);
        assert!(p.current_frame.spans.is_empty());
    }

    #[test]
    fn test_nested_spans() {
        let mut p = new_profiler(10);
        begin_span(&mut p, "outer");
        begin_span(&mut p, "inner");
        end_span(&mut p);
        end_span(&mut p);
        end_frame(&mut p);
        let frame = last_frame(&p).expect("should succeed");
        let inner = span_by_name(frame, "inner").expect("should succeed");
        assert_eq!(inner.depth, 1);
    }
}
