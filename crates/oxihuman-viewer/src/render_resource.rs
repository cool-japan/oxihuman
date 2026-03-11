#![allow(dead_code)]
//! GPU render resource abstraction.

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum ResourceType {
    Texture,
    Buffer,
    Shader,
    Framebuffer,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct RenderResource {
    name: String,
    rtype: ResourceType,
    size_bytes: u64,
    loaded: bool,
    hash: u64,
    age_ms: u64,
}

#[allow(dead_code)]
pub fn new_render_resource(name: &str, rtype: ResourceType, size_bytes: u64) -> RenderResource {
    RenderResource {
        name: name.to_string(),
        rtype,
        size_bytes,
        loaded: true,
        hash: {
            let mut h = 5381_u64;
            for b in name.bytes() {
                h = h.wrapping_mul(33).wrapping_add(b as u64);
            }
            h
        },
        age_ms: 0,
    }
}

#[allow(dead_code)]
pub fn resource_name(r: &RenderResource) -> &str {
    &r.name
}

#[allow(dead_code)]
pub fn resource_type(r: &RenderResource) -> &ResourceType {
    &r.rtype
}

#[allow(dead_code)]
pub fn resource_size_bytes(r: &RenderResource) -> u64 {
    r.size_bytes
}

#[allow(dead_code)]
pub fn resource_is_loaded(r: &RenderResource) -> bool {
    r.loaded
}

#[allow(dead_code)]
pub fn resource_to_json(r: &RenderResource) -> String {
    let type_name = match &r.rtype {
        ResourceType::Texture => "texture",
        ResourceType::Buffer => "buffer",
        ResourceType::Shader => "shader",
        ResourceType::Framebuffer => "framebuffer",
    };
    format!(
        "{{\"name\":\"{}\",\"type\":\"{}\",\"size\":{}}}",
        r.name, type_name, r.size_bytes
    )
}

#[allow(dead_code)]
pub fn resource_hash(r: &RenderResource) -> u64 {
    r.hash
}

#[allow(dead_code)]
pub fn resource_age_ms(r: &RenderResource) -> u64 {
    r.age_ms
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_render_resource() {
        let r = new_render_resource("tex0", ResourceType::Texture, 4096);
        assert_eq!(resource_name(&r), "tex0");
    }

    #[test]
    fn test_resource_type() {
        let r = new_render_resource("buf", ResourceType::Buffer, 1024);
        assert_eq!(*resource_type(&r), ResourceType::Buffer);
    }

    #[test]
    fn test_resource_size_bytes() {
        let r = new_render_resource("s", ResourceType::Shader, 512);
        assert_eq!(resource_size_bytes(&r), 512);
    }

    #[test]
    fn test_resource_is_loaded() {
        let r = new_render_resource("f", ResourceType::Framebuffer, 0);
        assert!(resource_is_loaded(&r));
    }

    #[test]
    fn test_resource_to_json() {
        let r = new_render_resource("t", ResourceType::Texture, 100);
        let json = resource_to_json(&r);
        assert!(json.contains("\"type\":\"texture\""));
    }

    #[test]
    fn test_resource_hash() {
        let r = new_render_resource("abc", ResourceType::Buffer, 0);
        assert!(resource_hash(&r) > 0);
    }

    #[test]
    fn test_resource_age_ms() {
        let r = new_render_resource("x", ResourceType::Shader, 0);
        assert_eq!(resource_age_ms(&r), 0);
    }

    #[test]
    fn test_different_hashes() {
        let r1 = new_render_resource("a", ResourceType::Texture, 0);
        let r2 = new_render_resource("b", ResourceType::Texture, 0);
        assert_ne!(resource_hash(&r1), resource_hash(&r2));
    }

    #[test]
    fn test_resource_name() {
        let r = new_render_resource("my_resource", ResourceType::Buffer, 2048);
        assert_eq!(resource_name(&r), "my_resource");
    }

    #[test]
    fn test_framebuffer_type() {
        let r = new_render_resource("fb", ResourceType::Framebuffer, 8192);
        assert_eq!(*resource_type(&r), ResourceType::Framebuffer);
    }
}
