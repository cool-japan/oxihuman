#![allow(dead_code)]
//! Face tagging utilities.

/// A face tag.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct FaceTag {
    pub face_index: usize,
    pub tag: String,
}

/// Face tag set.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct FaceTagSet {
    pub tags: Vec<FaceTag>,
}

/// Create a new face tag set.
#[allow(dead_code)]
pub fn new_face_tag_set() -> FaceTagSet {
    FaceTagSet { tags: Vec::new() }
}

/// Tag a face.
#[allow(dead_code)]
pub fn tag_face(set: &mut FaceTagSet, face_index: usize, tag: &str) {
    // Remove existing tag for this face
    set.tags.retain(|t| t.face_index != face_index);
    set.tags.push(FaceTag {
        face_index,
        tag: tag.to_string(),
    });
}

/// Untag a face.
#[allow(dead_code)]
pub fn untag_face(set: &mut FaceTagSet, face_index: usize) {
    set.tags.retain(|t| t.face_index != face_index);
}

/// Check if a face is tagged.
#[allow(dead_code)]
pub fn is_face_tagged(set: &FaceTagSet, face_index: usize) -> bool {
    set.tags.iter().any(|t| t.face_index == face_index)
}

/// Count tagged faces.
#[allow(dead_code)]
pub fn tagged_count(set: &FaceTagSet) -> usize {
    set.tags.len()
}

/// Get tag name for a face.
#[allow(dead_code)]
pub fn tag_name_at(set: &FaceTagSet, face_index: usize) -> Option<&str> {
    set.tags
        .iter()
        .find(|t| t.face_index == face_index)
        .map(|t| t.tag.as_str())
}

/// Clear all tags.
#[allow(dead_code)]
pub fn clear_tags(set: &mut FaceTagSet) {
    set.tags.clear();
}

/// Serialize tags to JSON.
#[allow(dead_code)]
pub fn tags_to_json(set: &FaceTagSet) -> String {
    format!("{{\"tag_count\":{}}}", set.tags.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_face_tag_set() {
        let s = new_face_tag_set();
        assert!(s.tags.is_empty());
    }

    #[test]
    fn test_tag_face() {
        let mut s = new_face_tag_set();
        tag_face(&mut s, 0, "head");
        assert_eq!(s.tags.len(), 1);
    }

    #[test]
    fn test_tag_face_replace() {
        let mut s = new_face_tag_set();
        tag_face(&mut s, 0, "head");
        tag_face(&mut s, 0, "body");
        assert_eq!(s.tags.len(), 1);
        assert_eq!(s.tags[0].tag, "body");
    }

    #[test]
    fn test_untag_face() {
        let mut s = new_face_tag_set();
        tag_face(&mut s, 0, "head");
        untag_face(&mut s, 0);
        assert!(s.tags.is_empty());
    }

    #[test]
    fn test_is_face_tagged() {
        let mut s = new_face_tag_set();
        tag_face(&mut s, 0, "head");
        assert!(is_face_tagged(&s, 0));
        assert!(!is_face_tagged(&s, 1));
    }

    #[test]
    fn test_tagged_count() {
        let mut s = new_face_tag_set();
        tag_face(&mut s, 0, "a");
        tag_face(&mut s, 1, "b");
        assert_eq!(tagged_count(&s), 2);
    }

    #[test]
    fn test_tag_name_at() {
        let mut s = new_face_tag_set();
        tag_face(&mut s, 0, "head");
        assert_eq!(tag_name_at(&s, 0), Some("head"));
        assert_eq!(tag_name_at(&s, 5), None);
    }

    #[test]
    fn test_clear_tags() {
        let mut s = new_face_tag_set();
        tag_face(&mut s, 0, "a");
        clear_tags(&mut s);
        assert!(s.tags.is_empty());
    }

    #[test]
    fn test_tags_to_json() {
        let s = new_face_tag_set();
        let j = tags_to_json(&s);
        assert!(j.contains("tag_count"));
    }

    #[test]
    fn test_untag_nonexistent() {
        let mut s = new_face_tag_set();
        untag_face(&mut s, 5);
        assert!(s.tags.is_empty());
    }
}
