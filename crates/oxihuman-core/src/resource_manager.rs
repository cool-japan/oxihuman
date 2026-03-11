//! Resource lifecycle management (load/unload/reference counting).

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum ResourceState {
    Unloaded,
    Loading,
    Loaded,
    Failed(String),
}

#[allow(dead_code)]
pub struct Resource {
    pub id: u32,
    pub key: String,
    pub resource_type: String,
    pub state: ResourceState,
    pub data: Option<Vec<u8>>,
    pub ref_count: u32,
    pub size_bytes: usize,
}

#[allow(dead_code)]
pub struct ResourceManager {
    pub resources: Vec<Resource>,
    pub next_id: u32,
    pub total_loaded_bytes: usize,
}

#[allow(dead_code)]
pub fn new_resource_manager() -> ResourceManager {
    ResourceManager {
        resources: Vec::new(),
        next_id: 1,
        total_loaded_bytes: 0,
    }
}

#[allow(dead_code)]
pub fn register_resource(mgr: &mut ResourceManager, key: &str, rtype: &str) -> u32 {
    let id = mgr.next_id;
    mgr.next_id += 1;
    mgr.resources.push(Resource {
        id,
        key: key.to_string(),
        resource_type: rtype.to_string(),
        state: ResourceState::Unloaded,
        data: None,
        ref_count: 0,
        size_bytes: 0,
    });
    id
}

#[allow(dead_code)]
pub fn load_resource(mgr: &mut ResourceManager, id: u32, data: Vec<u8>) {
    if let Some(r) = mgr.resources.iter_mut().find(|r| r.id == id) {
        let size = data.len();
        // subtract old size if already loaded
        if r.state == ResourceState::Loaded {
            mgr.total_loaded_bytes = mgr.total_loaded_bytes.saturating_sub(r.size_bytes);
        }
        r.size_bytes = size;
        r.data = Some(data);
        r.state = ResourceState::Loaded;
        mgr.total_loaded_bytes += size;
    }
}

#[allow(dead_code)]
pub fn fail_resource(mgr: &mut ResourceManager, id: u32, reason: &str) {
    if let Some(r) = mgr.resources.iter_mut().find(|r| r.id == id) {
        if r.state == ResourceState::Loaded {
            mgr.total_loaded_bytes = mgr.total_loaded_bytes.saturating_sub(r.size_bytes);
        }
        r.state = ResourceState::Failed(reason.to_string());
        r.data = None;
        r.size_bytes = 0;
    }
}

#[allow(dead_code)]
pub fn unload_resource(mgr: &mut ResourceManager, id: u32) {
    if let Some(r) = mgr.resources.iter_mut().find(|r| r.id == id) {
        if r.state == ResourceState::Loaded {
            mgr.total_loaded_bytes = mgr.total_loaded_bytes.saturating_sub(r.size_bytes);
        }
        r.data = None;
        r.size_bytes = 0;
        r.state = ResourceState::Unloaded;
    }
}

#[allow(dead_code)]
pub fn get_resource(mgr: &ResourceManager, id: u32) -> Option<&Resource> {
    mgr.resources.iter().find(|r| r.id == id)
}

#[allow(dead_code)]
pub fn get_by_key<'a>(mgr: &'a ResourceManager, key: &str) -> Option<&'a Resource> {
    mgr.resources.iter().find(|r| r.key == key)
}

#[allow(dead_code)]
pub fn retain_resource(mgr: &mut ResourceManager, id: u32) {
    if let Some(r) = mgr.resources.iter_mut().find(|r| r.id == id) {
        r.ref_count += 1;
    }
}

#[allow(dead_code)]
pub fn release_resource(mgr: &mut ResourceManager, id: u32) {
    let should_unload = if let Some(r) = mgr.resources.iter_mut().find(|r| r.id == id) {
        if r.ref_count > 0 {
            r.ref_count -= 1;
        }
        r.ref_count == 0 && r.state == ResourceState::Loaded
    } else {
        false
    };
    if should_unload {
        unload_resource(mgr, id);
    }
}

#[allow(dead_code)]
pub fn loaded_count(mgr: &ResourceManager) -> usize {
    mgr.resources
        .iter()
        .filter(|r| r.state == ResourceState::Loaded)
        .count()
}

#[allow(dead_code)]
pub fn failed_count(mgr: &ResourceManager) -> usize {
    mgr.resources
        .iter()
        .filter(|r| matches!(r.state, ResourceState::Failed(_)))
        .count()
}

#[allow(dead_code)]
pub fn total_memory(mgr: &ResourceManager) -> usize {
    mgr.total_loaded_bytes
}

#[allow(dead_code)]
pub fn garbage_collect(mgr: &mut ResourceManager) {
    let ids_to_unload: Vec<u32> = mgr
        .resources
        .iter()
        .filter(|r| r.ref_count == 0 && r.state == ResourceState::Loaded)
        .map(|r| r.id)
        .collect();
    for id in ids_to_unload {
        unload_resource(mgr, id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_resource_manager() {
        let mgr = new_resource_manager();
        assert!(mgr.resources.is_empty());
        assert_eq!(mgr.total_loaded_bytes, 0);
    }

    #[test]
    fn test_register_resource() {
        let mut mgr = new_resource_manager();
        let id = register_resource(&mut mgr, "texture/grass", "texture");
        assert_eq!(id, 1);
        let r = get_resource(&mgr, id).unwrap();
        assert_eq!(r.state, ResourceState::Unloaded);
        assert_eq!(r.key, "texture/grass");
    }

    #[test]
    fn test_load_resource() {
        let mut mgr = new_resource_manager();
        let id = register_resource(&mut mgr, "mesh/head", "mesh");
        load_resource(&mut mgr, id, vec![1u8; 1024]);
        let r = get_resource(&mgr, id).unwrap();
        assert_eq!(r.state, ResourceState::Loaded);
        assert_eq!(r.size_bytes, 1024);
        assert_eq!(total_memory(&mgr), 1024);
    }

    #[test]
    fn test_unload_resource() {
        let mut mgr = new_resource_manager();
        let id = register_resource(&mut mgr, "a", "t");
        load_resource(&mut mgr, id, vec![0u8; 512]);
        unload_resource(&mut mgr, id);
        let r = get_resource(&mgr, id).unwrap();
        assert_eq!(r.state, ResourceState::Unloaded);
        assert_eq!(total_memory(&mgr), 0);
    }

    #[test]
    fn test_fail_resource() {
        let mut mgr = new_resource_manager();
        let id = register_resource(&mut mgr, "b", "t");
        fail_resource(&mut mgr, id, "file not found");
        let r = get_resource(&mgr, id).unwrap();
        assert!(matches!(r.state, ResourceState::Failed(_)));
    }

    #[test]
    fn test_retain_release_auto_unload() {
        let mut mgr = new_resource_manager();
        let id = register_resource(&mut mgr, "c", "t");
        load_resource(&mut mgr, id, vec![1u8; 256]);
        retain_resource(&mut mgr, id);
        assert_eq!(get_resource(&mgr, id).unwrap().ref_count, 1);
        release_resource(&mut mgr, id);
        let r = get_resource(&mgr, id).unwrap();
        assert_eq!(r.state, ResourceState::Unloaded);
        assert_eq!(total_memory(&mgr), 0);
    }

    #[test]
    fn test_total_memory_multiple() {
        let mut mgr = new_resource_manager();
        let id1 = register_resource(&mut mgr, "r1", "t");
        let id2 = register_resource(&mut mgr, "r2", "t");
        load_resource(&mut mgr, id1, vec![0u8; 100]);
        load_resource(&mut mgr, id2, vec![0u8; 200]);
        assert_eq!(total_memory(&mgr), 300);
    }

    #[test]
    fn test_garbage_collect() {
        let mut mgr = new_resource_manager();
        let id1 = register_resource(&mut mgr, "g1", "t");
        let id2 = register_resource(&mut mgr, "g2", "t");
        load_resource(&mut mgr, id1, vec![0u8; 50]);
        load_resource(&mut mgr, id2, vec![0u8; 50]);
        retain_resource(&mut mgr, id1);
        // id2 has ref_count=0, should be GC'd
        garbage_collect(&mut mgr);
        assert_eq!(
            get_resource(&mgr, id2).unwrap().state,
            ResourceState::Unloaded
        );
        assert_eq!(
            get_resource(&mgr, id1).unwrap().state,
            ResourceState::Loaded
        );
    }

    #[test]
    fn test_get_by_key() {
        let mut mgr = new_resource_manager();
        let id = register_resource(&mut mgr, "unique/key", "mesh");
        load_resource(&mut mgr, id, vec![1u8; 32]);
        let r = get_by_key(&mgr, "unique/key").unwrap();
        assert_eq!(r.id, id);
    }

    #[test]
    fn test_loaded_count() {
        let mut mgr = new_resource_manager();
        let id1 = register_resource(&mut mgr, "k1", "t");
        let id2 = register_resource(&mut mgr, "k2", "t");
        register_resource(&mut mgr, "k3", "t");
        load_resource(&mut mgr, id1, vec![0u8; 10]);
        load_resource(&mut mgr, id2, vec![0u8; 10]);
        assert_eq!(loaded_count(&mgr), 2);
    }

    #[test]
    fn test_failed_count() {
        let mut mgr = new_resource_manager();
        let id1 = register_resource(&mut mgr, "f1", "t");
        let id2 = register_resource(&mut mgr, "f2", "t");
        register_resource(&mut mgr, "f3", "t");
        fail_resource(&mut mgr, id1, "err");
        fail_resource(&mut mgr, id2, "err2");
        assert_eq!(failed_count(&mgr), 2);
    }

    #[test]
    fn test_ids_increment() {
        let mut mgr = new_resource_manager();
        let id1 = register_resource(&mut mgr, "a", "t");
        let id2 = register_resource(&mut mgr, "b", "t");
        assert!(id2 > id1);
    }

    #[test]
    fn test_retain_multiple_refs() {
        let mut mgr = new_resource_manager();
        let id = register_resource(&mut mgr, "multi", "t");
        load_resource(&mut mgr, id, vec![0u8; 100]);
        retain_resource(&mut mgr, id);
        retain_resource(&mut mgr, id);
        release_resource(&mut mgr, id);
        // still loaded (ref_count = 1)
        assert_eq!(get_resource(&mgr, id).unwrap().state, ResourceState::Loaded);
        release_resource(&mut mgr, id);
        // now unloaded
        assert_eq!(
            get_resource(&mgr, id).unwrap().state,
            ResourceState::Unloaded
        );
    }
}
