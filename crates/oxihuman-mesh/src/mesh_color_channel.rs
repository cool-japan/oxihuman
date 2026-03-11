#![allow(dead_code)]
//! Per-vertex color channel storage.

/// A color channel with per-vertex RGBA colors.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ColorChannel {
    pub name: String,
    pub colors: Vec<[f32; 4]>,
}

/// Create a new color channel.
#[allow(dead_code)]
pub fn new_color_channel(name: &str, count: usize) -> ColorChannel {
    ColorChannel {
        name: name.to_string(),
        colors: vec![[1.0, 1.0, 1.0, 1.0]; count],
    }
}

/// Set color at an index.
#[allow(dead_code)]
pub fn set_color(channel: &mut ColorChannel, index: usize, color: [f32; 4]) {
    if index < channel.colors.len() {
        channel.colors[index] = color;
    }
}

/// Get color at an index.
#[allow(dead_code)]
pub fn get_color(channel: &ColorChannel, index: usize) -> [f32; 4] {
    if index < channel.colors.len() {
        channel.colors[index]
    } else {
        [0.0; 4]
    }
}

/// Return the number of color entries.
#[allow(dead_code)]
pub fn color_channel_count(channel: &ColorChannel) -> usize {
    channel.colors.len()
}

/// Return the channel name.
#[allow(dead_code)]
pub fn color_channel_name(channel: &ColorChannel) -> &str {
    &channel.name
}

/// Serialize color data to bytes.
#[allow(dead_code)]
pub fn color_to_bytes(channel: &ColorChannel) -> Vec<u8> {
    let mut buf = Vec::new();
    for c in &channel.colors {
        for &f in c {
            buf.extend_from_slice(&f.to_le_bytes());
        }
    }
    buf
}

/// Clear all colors to white.
#[allow(dead_code)]
pub fn clear_color_channel(channel: &mut ColorChannel) {
    for c in &mut channel.colors {
        *c = [1.0, 1.0, 1.0, 1.0];
    }
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn color_channel_to_json(channel: &ColorChannel) -> String {
    format!(
        "{{\"name\":\"{}\",\"count\":{}}}",
        channel.name,
        channel.colors.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_color_channel() {
        let c = new_color_channel("col0", 4);
        assert_eq!(color_channel_count(&c), 4);
    }

    #[test]
    fn test_set_get_color() {
        let mut c = new_color_channel("col0", 2);
        set_color(&mut c, 0, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(get_color(&c, 0), [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_get_color_oob() {
        let c = new_color_channel("col0", 1);
        assert_eq!(get_color(&c, 10), [0.0; 4]);
    }

    #[test]
    fn test_color_channel_name() {
        let c = new_color_channel("vertex_color", 1);
        assert_eq!(color_channel_name(&c), "vertex_color");
    }

    #[test]
    fn test_color_to_bytes() {
        let c = new_color_channel("col0", 1);
        assert_eq!(color_to_bytes(&c).len(), 16); // 4 floats * 4 bytes
    }

    #[test]
    fn test_clear_color_channel() {
        let mut c = new_color_channel("col0", 2);
        set_color(&mut c, 0, [0.0, 0.0, 0.0, 0.0]);
        clear_color_channel(&mut c);
        assert_eq!(get_color(&c, 0), [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_color_channel_to_json() {
        let c = new_color_channel("col0", 3);
        let j = color_channel_to_json(&c);
        assert!(j.contains("\"count\":3"));
    }

    #[test]
    fn test_default_color_white() {
        let c = new_color_channel("col0", 1);
        assert_eq!(get_color(&c, 0), [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_set_color_oob() {
        let mut c = new_color_channel("col0", 1);
        set_color(&mut c, 100, [0.0; 4]); // no panic
        assert_eq!(color_channel_count(&c), 1);
    }

    #[test]
    fn test_empty_color_channel() {
        let c = new_color_channel("e", 0);
        assert_eq!(color_channel_count(&c), 0);
    }
}
