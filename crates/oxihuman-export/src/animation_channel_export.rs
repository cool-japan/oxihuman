#![allow(dead_code)]

//! Animation channel export.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnimChannelExport {
    pub target: String,
    pub path: String,
    pub sampler_index: usize,
    pub node: String,
}

#[allow(dead_code)]
pub fn export_anim_channel(target: &str, path: &str, sampler_index: usize, node: &str) -> AnimChannelExport {
    AnimChannelExport { target: target.into(), path: path.into(), sampler_index, node: node.into() }
}

#[allow(dead_code)]
pub fn channel_target(ch: &AnimChannelExport) -> &str { &ch.target }

#[allow(dead_code)]
pub fn channel_sampler_index(ch: &AnimChannelExport) -> usize { ch.sampler_index }

#[allow(dead_code)]
pub fn channel_path(ch: &AnimChannelExport) -> &str { &ch.path }

#[allow(dead_code)]
pub fn channel_to_json(ch: &AnimChannelExport) -> String {
    format!("{{\"target\":\"{}\",\"path\":\"{}\",\"sampler\":{},\"node\":\"{}\"}}", ch.target, ch.path, ch.sampler_index, ch.node)
}

#[allow(dead_code)]
pub fn channel_node(ch: &AnimChannelExport) -> &str { &ch.node }

#[allow(dead_code)]
pub fn channel_export_size(ch: &AnimChannelExport) -> usize {
    ch.target.len() + ch.path.len() + ch.node.len() + 4
}

#[allow(dead_code)]
pub fn validate_channel(ch: &AnimChannelExport) -> bool {
    !ch.target.is_empty() && !ch.path.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export() { let c = export_anim_channel("translation", "node0", 0, "bone1"); assert_eq!(channel_target(&c), "translation"); }
    #[test]
    fn test_path() { let c = export_anim_channel("t", "rotation", 0, "n"); assert_eq!(channel_path(&c), "rotation"); }
    #[test]
    fn test_sampler() { let c = export_anim_channel("t", "p", 5, "n"); assert_eq!(channel_sampler_index(&c), 5); }
    #[test]
    fn test_node() { let c = export_anim_channel("t", "p", 0, "hip"); assert_eq!(channel_node(&c), "hip"); }
    #[test]
    fn test_to_json() { let c = export_anim_channel("t", "p", 0, "n"); assert!(channel_to_json(&c).contains("\"target\":\"t\"")); }
    #[test]
    fn test_export_size() { let c = export_anim_channel("t", "p", 0, "n"); assert!(channel_export_size(&c) > 0); }
    #[test]
    fn test_validate() { assert!(validate_channel(&export_anim_channel("t", "p", 0, "n"))); }
    #[test]
    fn test_validate_empty_target() { assert!(!validate_channel(&export_anim_channel("", "p", 0, "n"))); }
    #[test]
    fn test_validate_empty_path() { assert!(!validate_channel(&export_anim_channel("t", "", 0, "n"))); }
    #[test]
    fn test_channel_target_str() { let c = export_anim_channel("scale", "p", 0, "n"); assert_eq!(channel_target(&c), "scale"); }
}
