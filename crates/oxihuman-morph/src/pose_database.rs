//! Searchable pose database with metadata and similarity search.

#[allow(dead_code)]
pub struct PoseEntry {
    pub id: u64,
    pub name: String,
    pub tags: Vec<String>,
    pub joints: Vec<[f32; 4]>,
    pub metadata: Vec<(String, String)>,
    pub thumbnail_hash: u64,
}

#[allow(dead_code)]
pub struct PoseDatabase {
    pub entries: Vec<PoseEntry>,
    pub next_id: u64,
}

#[allow(dead_code)]
pub fn new_pose_database() -> PoseDatabase {
    PoseDatabase {
        entries: Vec::new(),
        next_id: 1,
    }
}

#[allow(dead_code)]
pub fn add_pose_entry(
    db: &mut PoseDatabase,
    name: &str,
    joints: Vec<[f32; 4]>,
    tags: Vec<String>,
) -> u64 {
    let id = db.next_id;
    db.next_id += 1;
    db.entries.push(PoseEntry {
        id,
        name: name.to_string(),
        tags,
        joints,
        metadata: Vec::new(),
        thumbnail_hash: 0,
    });
    id
}

#[allow(dead_code)]
pub fn get_pose(db: &PoseDatabase, id: u64) -> Option<&PoseEntry> {
    db.entries.iter().find(|e| e.id == id)
}

#[allow(dead_code)]
pub fn search_by_name<'a>(db: &'a PoseDatabase, query: &str) -> Vec<&'a PoseEntry> {
    let query_lower = query.to_lowercase();
    db.entries
        .iter()
        .filter(|e| e.name.to_lowercase().contains(&query_lower))
        .collect()
}

#[allow(dead_code)]
pub fn search_by_tag<'a>(db: &'a PoseDatabase, tag: &str) -> Vec<&'a PoseEntry> {
    let tag_lower = tag.to_lowercase();
    db.entries
        .iter()
        .filter(|e| e.tags.iter().any(|t| t.to_lowercase() == tag_lower))
        .collect()
}

#[allow(dead_code)]
pub fn pose_similarity(a: &PoseEntry, b: &PoseEntry) -> f32 {
    let len = a.joints.len().min(b.joints.len());
    if len == 0 {
        return 0.0;
    }
    let mut dot = 0.0_f32;
    let mut norm_a = 0.0_f32;
    let mut norm_b = 0.0_f32;
    for i in 0..len {
        for k in 0..4 {
            dot += a.joints[i][k] * b.joints[i][k];
            norm_a += a.joints[i][k] * a.joints[i][k];
            norm_b += b.joints[i][k] * b.joints[i][k];
        }
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom < 1e-9 {
        0.0
    } else {
        (dot / denom).clamp(-1.0, 1.0)
    }
}

#[allow(dead_code)]
pub fn nearest_pose<'a>(db: &'a PoseDatabase, query_joints: &[[f32; 4]]) -> Option<&'a PoseEntry> {
    if db.entries.is_empty() {
        return None;
    }
    let query_entry = PoseEntry {
        id: 0,
        name: String::new(),
        tags: Vec::new(),
        joints: query_joints.to_vec(),
        metadata: Vec::new(),
        thumbnail_hash: 0,
    };
    db.entries.iter().max_by(|a, b| {
        let sa = pose_similarity(a, &query_entry);
        let sb = pose_similarity(b, &query_entry);
        sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
    })
}

#[allow(dead_code)]
pub fn remove_pose(db: &mut PoseDatabase, id: u64) -> bool {
    let before = db.entries.len();
    db.entries.retain(|e| e.id != id);
    db.entries.len() < before
}

#[allow(dead_code)]
pub fn pose_count(db: &PoseDatabase) -> usize {
    db.entries.len()
}

#[allow(dead_code)]
pub fn all_tags(db: &PoseDatabase) -> Vec<&str> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for entry in &db.entries {
        for tag in &entry.tags {
            if seen.insert(tag.as_str()) {
                result.push(tag.as_str());
            }
        }
    }
    result
}

#[allow(dead_code)]
pub fn pose_database_to_json(db: &PoseDatabase) -> String {
    let mut out = String::from("{\"entries\":[");
    for (i, entry) in db.entries.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!(
            "{{\"id\":{},\"name\":\"{}\",\"tags\":[{}]}}",
            entry.id,
            entry.name.replace('"', "\\\""),
            entry
                .tags
                .iter()
                .map(|t| format!("\"{}\"", t.replace('"', "\\\"")))
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    out.push_str("]}");
    out
}

#[allow(dead_code)]
pub fn import_poses(db: &mut PoseDatabase, entries: Vec<PoseEntry>) {
    for mut entry in entries {
        entry.id = db.next_id;
        db.next_id += 1;
        db.entries.push(entry);
    }
}

#[allow(dead_code)]
pub fn sort_by_name(db: &mut PoseDatabase) {
    db.entries.sort_by(|a, b| a.name.cmp(&b.name));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_joints(v: f32) -> Vec<[f32; 4]> {
        vec![[v, 0.0, 0.0, 0.0], [0.0, v, 0.0, 0.0]]
    }

    #[test]
    fn test_new_pose_database() {
        let db = new_pose_database();
        assert_eq!(pose_count(&db), 0);
        assert_eq!(db.next_id, 1);
    }

    #[test]
    fn test_add_pose_entry() {
        let mut db = new_pose_database();
        let id = add_pose_entry(
            &mut db,
            "T-Pose",
            make_joints(1.0),
            vec!["idle".to_string()],
        );
        assert_eq!(id, 1);
        assert_eq!(pose_count(&db), 1);
    }

    #[test]
    fn test_get_pose() {
        let mut db = new_pose_database();
        let id = add_pose_entry(&mut db, "Walk", make_joints(1.0), vec![]);
        let entry = get_pose(&db, id);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().name, "Walk");
    }

    #[test]
    fn test_get_pose_not_found() {
        let db = new_pose_database();
        assert!(get_pose(&db, 99).is_none());
    }

    #[test]
    fn test_search_by_name() {
        let mut db = new_pose_database();
        add_pose_entry(&mut db, "T-Pose", make_joints(1.0), vec![]);
        add_pose_entry(&mut db, "Walk Cycle", make_joints(0.5), vec![]);
        let results = search_by_name(&db, "walk");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Walk Cycle");
    }

    #[test]
    fn test_search_by_name_case_insensitive() {
        let mut db = new_pose_database();
        add_pose_entry(&mut db, "Running Pose", make_joints(1.0), vec![]);
        let results = search_by_name(&db, "RUNNING");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_by_tag() {
        let mut db = new_pose_database();
        add_pose_entry(
            &mut db,
            "T-Pose",
            make_joints(1.0),
            vec!["idle".to_string()],
        );
        add_pose_entry(&mut db, "Run", make_joints(0.5), vec!["motion".to_string()]);
        let results = search_by_tag(&db, "idle");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "T-Pose");
    }

    #[test]
    fn test_pose_similarity_identical() {
        let joints = make_joints(1.0);
        let entry_a = PoseEntry {
            id: 1,
            name: "A".to_string(),
            tags: vec![],
            joints: joints.clone(),
            metadata: vec![],
            thumbnail_hash: 0,
        };
        let entry_b = PoseEntry {
            id: 2,
            name: "B".to_string(),
            tags: vec![],
            joints,
            metadata: vec![],
            thumbnail_hash: 0,
        };
        let sim = pose_similarity(&entry_a, &entry_b);
        assert!((sim - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_pose_similarity_different() {
        let entry_a = PoseEntry {
            id: 1,
            name: "A".to_string(),
            tags: vec![],
            joints: vec![[1.0, 0.0, 0.0, 0.0]],
            metadata: vec![],
            thumbnail_hash: 0,
        };
        let entry_b = PoseEntry {
            id: 2,
            name: "B".to_string(),
            tags: vec![],
            joints: vec![[-1.0, 0.0, 0.0, 0.0]],
            metadata: vec![],
            thumbnail_hash: 0,
        };
        let sim = pose_similarity(&entry_a, &entry_b);
        assert!(sim < 0.0);
    }

    #[test]
    fn test_nearest_pose() {
        let mut db = new_pose_database();
        add_pose_entry(&mut db, "A", vec![[1.0, 0.0, 0.0, 0.0]], vec![]);
        add_pose_entry(&mut db, "B", vec![[0.0, 1.0, 0.0, 0.0]], vec![]);
        let query = vec![[1.0, 0.0, 0.0, 0.0]];
        let nearest = nearest_pose(&db, &query);
        assert!(nearest.is_some());
        assert_eq!(nearest.unwrap().name, "A");
    }

    #[test]
    fn test_nearest_pose_empty() {
        let db = new_pose_database();
        assert!(nearest_pose(&db, &[[1.0, 0.0, 0.0, 0.0]]).is_none());
    }

    #[test]
    fn test_remove_pose() {
        let mut db = new_pose_database();
        let id = add_pose_entry(&mut db, "Test", make_joints(1.0), vec![]);
        assert!(remove_pose(&mut db, id));
        assert_eq!(pose_count(&db), 0);
        assert!(!remove_pose(&mut db, id));
    }

    #[test]
    fn test_pose_count() {
        let mut db = new_pose_database();
        assert_eq!(pose_count(&db), 0);
        add_pose_entry(&mut db, "A", make_joints(1.0), vec![]);
        add_pose_entry(&mut db, "B", make_joints(0.5), vec![]);
        assert_eq!(pose_count(&db), 2);
    }

    #[test]
    fn test_all_tags() {
        let mut db = new_pose_database();
        add_pose_entry(
            &mut db,
            "A",
            make_joints(1.0),
            vec!["idle".to_string(), "standing".to_string()],
        );
        add_pose_entry(
            &mut db,
            "B",
            make_joints(0.5),
            vec!["idle".to_string(), "motion".to_string()],
        );
        let tags = all_tags(&db);
        assert_eq!(tags.len(), 3);
        assert!(tags.contains(&"idle"));
        assert!(tags.contains(&"standing"));
        assert!(tags.contains(&"motion"));
    }

    #[test]
    fn test_sort_by_name() {
        let mut db = new_pose_database();
        add_pose_entry(&mut db, "Zebra", make_joints(1.0), vec![]);
        add_pose_entry(&mut db, "Alpha", make_joints(0.5), vec![]);
        sort_by_name(&mut db);
        assert_eq!(db.entries[0].name, "Alpha");
        assert_eq!(db.entries[1].name, "Zebra");
    }

    #[test]
    fn test_pose_database_to_json() {
        let mut db = new_pose_database();
        add_pose_entry(&mut db, "Test", make_joints(1.0), vec!["tag1".to_string()]);
        let json = pose_database_to_json(&db);
        assert!(json.contains("\"name\":\"Test\""));
        assert!(json.contains("\"tag1\""));
    }

    #[test]
    fn test_import_poses() {
        let mut db = new_pose_database();
        let entry = PoseEntry {
            id: 999,
            name: "Imported".to_string(),
            tags: vec![],
            joints: make_joints(1.0),
            metadata: vec![],
            thumbnail_hash: 0,
        };
        import_poses(&mut db, vec![entry]);
        assert_eq!(pose_count(&db), 1);
        assert_eq!(db.entries[0].id, 1);
    }
}
