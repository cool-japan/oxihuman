#![allow(dead_code)]
//! UV channel storage for meshes.

/// A UV channel with per-vertex UV coordinates.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct UvChannel {
    pub name: String,
    pub uvs: Vec<[f32; 2]>,
}

/// Create a new UV channel.
#[allow(dead_code)]
pub fn new_uv_channel(name: &str, count: usize) -> UvChannel {
    UvChannel {
        name: name.to_string(),
        uvs: vec![[0.0; 2]; count],
    }
}

/// Set UV at an index.
#[allow(dead_code)]
pub fn set_uv(channel: &mut UvChannel, index: usize, uv: [f32; 2]) {
    if index < channel.uvs.len() {
        channel.uvs[index] = uv;
    }
}

/// Get UV at an index.
#[allow(dead_code)]
pub fn get_uv(channel: &UvChannel, index: usize) -> [f32; 2] {
    if index < channel.uvs.len() {
        channel.uvs[index]
    } else {
        [0.0; 2]
    }
}

/// Return the number of UV entries.
#[allow(dead_code)]
pub fn uv_channel_count(channel: &UvChannel) -> usize {
    channel.uvs.len()
}

/// Return the channel name.
#[allow(dead_code)]
pub fn uv_channel_name(channel: &UvChannel) -> &str {
    &channel.name
}

/// Serialize UV data to bytes.
#[allow(dead_code)]
pub fn uv_channel_to_bytes(channel: &UvChannel) -> Vec<u8> {
    let mut buf = Vec::new();
    for uv in &channel.uvs {
        for &f in uv {
            buf.extend_from_slice(&f.to_le_bytes());
        }
    }
    buf
}

/// Clear all UVs to zero.
#[allow(dead_code)]
pub fn clear_uv_channel(channel: &mut UvChannel) {
    for uv in &mut channel.uvs {
        *uv = [0.0; 2];
    }
}

/// Compute UV bounds (min, max).
#[allow(dead_code)]
pub fn uv_channel_bounds(channel: &UvChannel) -> ([f32; 2], [f32; 2]) {
    if channel.uvs.is_empty() {
        return ([0.0; 2], [0.0; 2]);
    }
    let mut mn = channel.uvs[0];
    let mut mx = channel.uvs[0];
    for uv in &channel.uvs {
        if uv[0] < mn[0] { mn[0] = uv[0]; }
        if uv[1] < mn[1] { mn[1] = uv[1]; }
        if uv[0] > mx[0] { mx[0] = uv[0]; }
        if uv[1] > mx[1] { mx[1] = uv[1]; }
    }
    (mn, mx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_uv_channel() {
        let c = new_uv_channel("uv0", 4);
        assert_eq!(uv_channel_count(&c), 4);
    }

    #[test]
    fn test_set_get_uv() {
        let mut c = new_uv_channel("uv0", 2);
        set_uv(&mut c, 0, [0.5, 0.5]);
        assert_eq!(get_uv(&c, 0), [0.5, 0.5]);
    }

    #[test]
    fn test_get_uv_oob() {
        let c = new_uv_channel("uv0", 1);
        assert_eq!(get_uv(&c, 10), [0.0; 2]);
    }

    #[test]
    fn test_uv_channel_name() {
        let c = new_uv_channel("lightmap", 1);
        assert_eq!(uv_channel_name(&c), "lightmap");
    }

    #[test]
    fn test_uv_channel_to_bytes() {
        let c = new_uv_channel("uv0", 1);
        assert_eq!(uv_channel_to_bytes(&c).len(), 8);
    }

    #[test]
    fn test_clear_uv_channel() {
        let mut c = new_uv_channel("uv0", 2);
        set_uv(&mut c, 0, [1.0, 1.0]);
        clear_uv_channel(&mut c);
        assert_eq!(get_uv(&c, 0), [0.0; 2]);
    }

    #[test]
    fn test_uv_channel_bounds() {
        let mut c = new_uv_channel("uv0", 3);
        set_uv(&mut c, 0, [0.0, 0.0]);
        set_uv(&mut c, 1, [1.0, 0.5]);
        set_uv(&mut c, 2, [0.5, 1.0]);
        let (mn, mx) = uv_channel_bounds(&c);
        assert_eq!(mn, [0.0, 0.0]);
        assert_eq!(mx, [1.0, 1.0]);
    }

    #[test]
    fn test_uv_channel_bounds_empty() {
        let c = new_uv_channel("uv0", 0);
        let (mn, mx) = uv_channel_bounds(&c);
        assert_eq!(mn, [0.0; 2]);
        assert_eq!(mx, [0.0; 2]);
    }

    #[test]
    fn test_set_uv_oob() {
        let mut c = new_uv_channel("uv0", 1);
        set_uv(&mut c, 100, [1.0, 1.0]); // should not panic
        assert_eq!(uv_channel_count(&c), 1);
    }

    #[test]
    fn test_empty_uv_channel() {
        let c = new_uv_channel("e", 0);
        assert_eq!(uv_channel_count(&c), 0);
    }
}
