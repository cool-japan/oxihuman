#![allow(dead_code)]

//! Descriptor set with binding slots and resource handles.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceHandle {
    Buffer(u32),
    Texture(u32),
    Sampler(u32),
    None,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DescriptorBinding {
    pub slot: u32,
    pub array_index: u32,
    pub resource: ResourceHandle,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DescriptorSet {
    pub set_index: u32,
    pub bindings: HashMap<u32, DescriptorBinding>,
    pub dirty: bool,
    pub generation: u32,
}

#[allow(dead_code)]
pub fn new_descriptor_set(set_index: u32) -> DescriptorSet {
    DescriptorSet {
        set_index,
        bindings: HashMap::new(),
        dirty: false,
        generation: 0,
    }
}

#[allow(dead_code)]
pub fn ds_bind(set: &mut DescriptorSet, slot: u32, resource: ResourceHandle) {
    set.bindings.insert(
        slot,
        DescriptorBinding {
            slot,
            array_index: 0,
            resource,
        },
    );
    set.dirty = true;
}

#[allow(dead_code)]
pub fn ds_bind_array(set: &mut DescriptorSet, slot: u32, array_index: u32, resource: ResourceHandle) {
    let key = slot * 1000 + array_index;
    set.bindings.insert(
        key,
        DescriptorBinding {
            slot,
            array_index,
            resource,
        },
    );
    set.dirty = true;
}

#[allow(dead_code)]
pub fn ds_get(set: &DescriptorSet, slot: u32) -> Option<ResourceHandle> {
    set.bindings.get(&slot).map(|b| b.resource)
}

#[allow(dead_code)]
pub fn ds_unbind(set: &mut DescriptorSet, slot: u32) {
    set.bindings.remove(&slot);
    set.dirty = true;
}

#[allow(dead_code)]
pub fn ds_binding_count(set: &DescriptorSet) -> usize {
    set.bindings.len()
}

#[allow(dead_code)]
pub fn ds_flush(set: &mut DescriptorSet) {
    if set.dirty {
        set.dirty = false;
        set.generation += 1;
    }
}

#[allow(dead_code)]
pub fn ds_clear(set: &mut DescriptorSet) {
    set.bindings.clear();
    set.dirty = true;
}

#[allow(dead_code)]
pub fn ds_is_dirty(set: &DescriptorSet) -> bool {
    set.dirty
}

#[allow(dead_code)]
pub fn ds_to_json(set: &DescriptorSet) -> String {
    format!(
        "{{\"set_index\":{},\"binding_count\":{},\"dirty\":{},\"generation\":{}}}",
        set.set_index,
        set.bindings.len(),
        set.dirty,
        set.generation
    )
}

#[allow(dead_code)]
pub fn ds_has_slot(set: &DescriptorSet, slot: u32) -> bool {
    set.bindings.contains_key(&slot)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_descriptor_set() {
        let s = new_descriptor_set(0);
        assert_eq!(ds_binding_count(&s), 0);
        assert!(!ds_is_dirty(&s));
    }

    #[test]
    fn test_bind() {
        let mut s = new_descriptor_set(0);
        ds_bind(&mut s, 0, ResourceHandle::Buffer(1));
        assert_eq!(ds_binding_count(&s), 1);
        assert!(ds_is_dirty(&s));
    }

    #[test]
    fn test_get() {
        let mut s = new_descriptor_set(0);
        ds_bind(&mut s, 0, ResourceHandle::Texture(5));
        assert_eq!(ds_get(&s, 0), Some(ResourceHandle::Texture(5)));
    }

    #[test]
    fn test_unbind() {
        let mut s = new_descriptor_set(0);
        ds_bind(&mut s, 0, ResourceHandle::Buffer(1));
        ds_unbind(&mut s, 0);
        assert!(!ds_has_slot(&s, 0));
    }

    #[test]
    fn test_flush_clears_dirty() {
        let mut s = new_descriptor_set(0);
        ds_bind(&mut s, 0, ResourceHandle::Sampler(1));
        ds_flush(&mut s);
        assert!(!ds_is_dirty(&s));
    }

    #[test]
    fn test_flush_increments_generation() {
        let mut s = new_descriptor_set(0);
        ds_bind(&mut s, 0, ResourceHandle::Buffer(1));
        let gen0 = s.generation;
        ds_flush(&mut s);
        assert!(s.generation > gen0);
    }

    #[test]
    fn test_clear() {
        let mut s = new_descriptor_set(0);
        ds_bind(&mut s, 0, ResourceHandle::Buffer(1));
        ds_clear(&mut s);
        assert_eq!(ds_binding_count(&s), 0);
    }

    #[test]
    fn test_bind_array() {
        let mut s = new_descriptor_set(0);
        ds_bind_array(&mut s, 5, 0, ResourceHandle::Texture(10));
        ds_bind_array(&mut s, 5, 1, ResourceHandle::Texture(11));
        assert_eq!(ds_binding_count(&s), 2);
    }

    #[test]
    fn test_get_nonexistent() {
        let s = new_descriptor_set(0);
        assert!(ds_get(&s, 99).is_none());
    }

    #[test]
    fn test_to_json() {
        let s = new_descriptor_set(2);
        let json = ds_to_json(&s);
        assert!(json.contains("\"set_index\":2"));
    }
}
