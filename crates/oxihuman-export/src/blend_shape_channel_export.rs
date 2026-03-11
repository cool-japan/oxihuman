#![allow(dead_code)]
//! Blend shape channel export.

/// Blend shape channel export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct BlendShapeChannelExport {
    pub channels: Vec<BlendChannel>,
}

/// A single blend channel.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct BlendChannel {
    pub name: String,
    pub weight: f32,
}

/// Export blend channels.
#[allow(dead_code)]
pub fn export_blend_channels(names: &[&str], weights: &[f32]) -> BlendShapeChannelExport {
    let len = names.len().min(weights.len());
    let channels = (0..len)
        .map(|i| BlendChannel {
            name: names[i].to_string(),
            weight: weights[i],
        })
        .collect();
    BlendShapeChannelExport { channels }
}

/// Get channel count.
#[allow(dead_code)]
pub fn channel_count_bsce(e: &BlendShapeChannelExport) -> usize {
    e.channels.len()
}

/// Get channel name.
#[allow(dead_code)]
pub fn channel_name_bsce(e: &BlendShapeChannelExport, index: usize) -> &str {
    if index < e.channels.len() {
        &e.channels[index].name
    } else {
        ""
    }
}

/// Get channel weight.
#[allow(dead_code)]
pub fn channel_weight(e: &BlendShapeChannelExport, index: usize) -> f32 {
    if index < e.channels.len() {
        e.channels[index].weight
    } else {
        0.0
    }
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn channel_to_json(e: &BlendShapeChannelExport) -> String {
    format!("{{\"channel_count\":{}}}", e.channels.len())
}

/// Serialize to bytes.
#[allow(dead_code)]
pub fn channel_to_bytes(e: &BlendShapeChannelExport) -> Vec<u8> {
    let mut bytes = Vec::new();
    for ch in &e.channels {
        bytes.extend_from_slice(&(ch.weight).to_le_bytes());
    }
    bytes
}

/// Get export size in bytes.
#[allow(dead_code)]
pub fn channel_export_size(e: &BlendShapeChannelExport) -> usize {
    e.channels.len() * 4
}

/// Validate channels.
#[allow(dead_code)]
pub fn validate_channels(e: &BlendShapeChannelExport) -> bool {
    e.channels
        .iter()
        .all(|c| !c.name.is_empty() && (0.0..=1.0).contains(&c.weight))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_blend_channels() {
        let e = export_blend_channels(&["smile", "blink"], &[0.5, 1.0]);
        assert_eq!(e.channels.len(), 2);
    }

    #[test]
    fn test_channel_count() {
        let e = export_blend_channels(&["a"], &[0.0]);
        assert_eq!(channel_count_bsce(&e), 1);
    }

    #[test]
    fn test_channel_name() {
        let e = export_blend_channels(&["smile"], &[0.5]);
        assert_eq!(channel_name_bsce(&e, 0), "smile");
        assert_eq!(channel_name_bsce(&e, 5), "");
    }

    #[test]
    fn test_channel_weight() {
        let e = export_blend_channels(&["x"], &[0.75]);
        assert!((channel_weight(&e, 0) - 0.75).abs() < 1e-6);
        assert!((channel_weight(&e, 5)).abs() < 1e-6);
    }

    #[test]
    fn test_channel_to_json() {
        let e = export_blend_channels(&["a"], &[0.0]);
        let j = channel_to_json(&e);
        assert!(j.contains("channel_count"));
    }

    #[test]
    fn test_channel_to_bytes() {
        let e = export_blend_channels(&["a"], &[1.0]);
        let b = channel_to_bytes(&e);
        assert_eq!(b.len(), 4);
    }

    #[test]
    fn test_channel_export_size() {
        let e = export_blend_channels(&["a", "b"], &[0.0, 0.0]);
        assert_eq!(channel_export_size(&e), 8);
    }

    #[test]
    fn test_validate_channels() {
        let e = export_blend_channels(&["a"], &[0.5]);
        assert!(validate_channels(&e));
    }

    #[test]
    fn test_validate_channels_invalid() {
        let e = export_blend_channels(&["a"], &[1.5]);
        assert!(!validate_channels(&e));
    }

    #[test]
    fn test_export_empty() {
        let e = export_blend_channels(&[], &[]);
        assert_eq!(channel_count_bsce(&e), 0);
    }
}
