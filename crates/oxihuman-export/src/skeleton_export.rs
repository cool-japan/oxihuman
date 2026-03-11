//! Skeleton/rig export (JSON and BVH stub).

#[allow(dead_code)]
pub struct ExportBone {
    pub id: u32,
    pub name: String,
    pub parent_id: Option<u32>,
    pub head: [f32; 3],
    pub tail: [f32; 3],
    pub rotation: [f32; 4],
    pub length: f32,
}

#[allow(dead_code)]
pub struct SkeletonExport {
    pub name: String,
    pub bones: Vec<ExportBone>,
    pub frame_rate: f32,
    pub frames: Vec<Vec<([f32; 3], [f32; 4])>>,
}

#[allow(dead_code)]
pub fn new_skeleton_export(name: &str) -> SkeletonExport {
    SkeletonExport {
        name: name.to_string(),
        bones: Vec::new(),
        frame_rate: 30.0,
        frames: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_export_bone(skel: &mut SkeletonExport, bone: ExportBone) {
    skel.bones.push(bone);
}

#[allow(dead_code)]
pub fn add_skeleton_frame(skel: &mut SkeletonExport, poses: Vec<([f32; 3], [f32; 4])>) {
    skel.frames.push(poses);
}

#[allow(dead_code)]
pub fn skeleton_to_json(skel: &SkeletonExport) -> String {
    let bone_strs: Vec<String> = skel
        .bones
        .iter()
        .map(|b| {
            let parent = match b.parent_id {
                Some(p) => format!("{p}"),
                None => "null".to_string(),
            };
            format!(
                r#"{{"id":{},"name":"{}","parent_id":{},"head":[{},{},{}],"tail":[{},{},{}],"rotation":[{},{},{},{}],"length":{}}}"#,
                b.id,
                b.name,
                parent,
                b.head[0], b.head[1], b.head[2],
                b.tail[0], b.tail[1], b.tail[2],
                b.rotation[0], b.rotation[1], b.rotation[2], b.rotation[3],
                b.length
            )
        })
        .collect();

    format!(
        r#"{{"name":"{}","frame_rate":{},"bones":[{}]}}"#,
        skel.name,
        skel.frame_rate,
        bone_strs.join(",")
    )
}

#[allow(dead_code)]
pub fn skeleton_to_bvh_stub(skel: &SkeletonExport) -> String {
    let mut out = String::new();
    out.push_str("HIERARCHY\n");
    out.push_str("ROOT\n");
    out.push_str("{\n");
    for bone in &skel.bones {
        out.push_str(&format!(
            "  JOINT {}\n  {{\n    OFFSET {} {} {}\n  }}\n",
            bone.name, bone.head[0], bone.head[1], bone.head[2]
        ));
    }
    out.push_str("}\n");
    out.push_str("MOTION\n");
    out.push_str(&format!("Frames: {}\n", skel.frames.len()));
    out.push_str(&format!("Frame Time: {}\n", 1.0 / skel.frame_rate.max(1.0)));
    for frame in &skel.frames {
        let values: Vec<String> = frame
            .iter()
            .flat_map(|(pos, rot)| {
                vec![
                    format!("{}", pos[0]),
                    format!("{}", pos[1]),
                    format!("{}", pos[2]),
                    format!("{}", rot[0]),
                    format!("{}", rot[1]),
                    format!("{}", rot[2]),
                    format!("{}", rot[3]),
                ]
            })
            .collect();
        out.push_str(&values.join(" "));
        out.push('\n');
    }
    out
}

#[allow(dead_code)]
pub fn bone_count_export(skel: &SkeletonExport) -> usize {
    skel.bones.len()
}

#[allow(dead_code)]
pub fn frame_count(skel: &SkeletonExport) -> usize {
    skel.frames.len()
}

#[allow(dead_code)]
#[allow(clippy::needless_lifetimes)]
pub fn get_export_bone<'a>(skel: &'a SkeletonExport, name: &str) -> Option<&'a ExportBone> {
    skel.bones.iter().find(|b| b.name == name)
}

#[allow(dead_code)]
pub fn root_bones(skel: &SkeletonExport) -> Vec<&ExportBone> {
    skel.bones
        .iter()
        .filter(|b| b.parent_id.is_none())
        .collect()
}

#[allow(dead_code)]
pub fn child_bones(skel: &SkeletonExport, parent_id: u32) -> Vec<&ExportBone> {
    skel.bones
        .iter()
        .filter(|b| b.parent_id == Some(parent_id))
        .collect()
}

#[allow(dead_code)]
pub fn bone_world_matrix(bone: &ExportBone) -> [[f32; 4]; 4] {
    let [qx, qy, qz, qw] = bone.rotation;
    // Build rotation matrix from quaternion
    let r00 = 1.0 - 2.0 * (qy * qy + qz * qz);
    let r01 = 2.0 * (qx * qy - qz * qw);
    let r02 = 2.0 * (qx * qz + qy * qw);
    let r10 = 2.0 * (qx * qy + qz * qw);
    let r11 = 1.0 - 2.0 * (qx * qx + qz * qz);
    let r12 = 2.0 * (qy * qz - qx * qw);
    let r20 = 2.0 * (qx * qz - qy * qw);
    let r21 = 2.0 * (qy * qz + qx * qw);
    let r22 = 1.0 - 2.0 * (qx * qx + qy * qy);
    let [tx, ty, tz] = bone.head;
    [
        [r00, r01, r02, 0.0],
        [r10, r11, r12, 0.0],
        [r20, r21, r22, 0.0],
        [tx, ty, tz, 1.0],
    ]
}

#[allow(dead_code)]
pub fn skeleton_duration(skel: &SkeletonExport) -> f32 {
    frame_count(skel) as f32 / skel.frame_rate.max(f32::EPSILON)
}

#[allow(dead_code)]
pub fn bind_pose_snapshot(skel: &SkeletonExport) -> Vec<([f32; 3], [f32; 4])> {
    skel.bones.iter().map(|b| (b.head, b.rotation)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bone(id: u32, name: &str, parent: Option<u32>) -> ExportBone {
        ExportBone {
            id,
            name: name.to_string(),
            parent_id: parent,
            head: [0.0, 0.0, 0.0],
            tail: [0.0, 1.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            length: 1.0,
        }
    }

    #[test]
    fn test_new_skeleton_export() {
        let skel = new_skeleton_export("human");
        assert_eq!(skel.name, "human");
        assert!(skel.bones.is_empty());
        assert!(skel.frames.is_empty());
    }

    #[test]
    fn test_add_export_bone() {
        let mut skel = new_skeleton_export("s");
        add_export_bone(&mut skel, make_bone(0, "root", None));
        assert_eq!(bone_count_export(&skel), 1);
    }

    #[test]
    fn test_add_skeleton_frame() {
        let mut skel = new_skeleton_export("s");
        add_export_bone(&mut skel, make_bone(0, "root", None));
        let poses = vec![([0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 1.0])];
        add_skeleton_frame(&mut skel, poses);
        assert_eq!(frame_count(&skel), 1);
    }

    #[test]
    fn test_bone_count_export() {
        let mut skel = new_skeleton_export("s");
        assert_eq!(bone_count_export(&skel), 0);
        add_export_bone(&mut skel, make_bone(0, "root", None));
        add_export_bone(&mut skel, make_bone(1, "spine", Some(0)));
        assert_eq!(bone_count_export(&skel), 2);
    }

    #[test]
    fn test_frame_count() {
        let mut skel = new_skeleton_export("s");
        assert_eq!(frame_count(&skel), 0);
        add_skeleton_frame(&mut skel, vec![]);
        add_skeleton_frame(&mut skel, vec![]);
        assert_eq!(frame_count(&skel), 2);
    }

    #[test]
    fn test_get_export_bone_by_name() {
        let mut skel = new_skeleton_export("s");
        add_export_bone(&mut skel, make_bone(0, "hips", None));
        add_export_bone(&mut skel, make_bone(1, "spine", Some(0)));
        let bone = get_export_bone(&skel, "spine");
        assert!(bone.is_some());
        assert_eq!(bone.unwrap().id, 1);
    }

    #[test]
    fn test_get_export_bone_not_found() {
        let skel = new_skeleton_export("s");
        assert!(get_export_bone(&skel, "missing").is_none());
    }

    #[test]
    fn test_root_bones() {
        let mut skel = new_skeleton_export("s");
        add_export_bone(&mut skel, make_bone(0, "root", None));
        add_export_bone(&mut skel, make_bone(1, "child", Some(0)));
        add_export_bone(&mut skel, make_bone(2, "root2", None));
        let roots = root_bones(&skel);
        assert_eq!(roots.len(), 2);
    }

    #[test]
    fn test_child_bones() {
        let mut skel = new_skeleton_export("s");
        add_export_bone(&mut skel, make_bone(0, "root", None));
        add_export_bone(&mut skel, make_bone(1, "child1", Some(0)));
        add_export_bone(&mut skel, make_bone(2, "child2", Some(0)));
        add_export_bone(&mut skel, make_bone(3, "grandchild", Some(1)));
        let children = child_bones(&skel, 0);
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn test_skeleton_to_json_nonempty() {
        let mut skel = new_skeleton_export("test_rig");
        add_export_bone(&mut skel, make_bone(0, "root", None));
        let json = skeleton_to_json(&skel);
        assert!(!json.is_empty());
        assert!(json.contains("test_rig"));
        assert!(json.contains("root"));
    }

    #[test]
    fn test_bvh_stub_contains_hierarchy() {
        let skel = new_skeleton_export("s");
        let bvh = skeleton_to_bvh_stub(&skel);
        assert!(bvh.contains("HIERARCHY"));
        assert!(bvh.contains("MOTION"));
    }

    #[test]
    fn test_skeleton_duration() {
        let mut skel = new_skeleton_export("s");
        skel.frame_rate = 24.0;
        add_skeleton_frame(&mut skel, vec![]);
        add_skeleton_frame(&mut skel, vec![]);
        add_skeleton_frame(&mut skel, vec![]);
        let dur = skeleton_duration(&skel);
        assert!((dur - 3.0 / 24.0).abs() < 1e-5);
    }

    #[test]
    fn test_bind_pose_snapshot() {
        let mut skel = new_skeleton_export("s");
        add_export_bone(&mut skel, make_bone(0, "root", None));
        add_export_bone(&mut skel, make_bone(1, "spine", Some(0)));
        let snapshot = bind_pose_snapshot(&skel);
        assert_eq!(snapshot.len(), 2);
    }

    #[test]
    fn test_bone_world_matrix_identity() {
        let bone = ExportBone {
            id: 0,
            name: "b".to_string(),
            parent_id: None,
            head: [0.0, 0.0, 0.0],
            tail: [0.0, 1.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            length: 1.0,
        };
        let m = bone_world_matrix(&bone);
        assert!((m[0][0] - 1.0).abs() < 1e-5);
        assert!((m[1][1] - 1.0).abs() < 1e-5);
        assert!((m[2][2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_skeleton_duration_zero_frames() {
        let skel = new_skeleton_export("s");
        let dur = skeleton_duration(&skel);
        assert!((dur - 0.0).abs() < 1e-5);
    }
}
