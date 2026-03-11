// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Scene graph with transform hierarchy, mesh instances, and light nodes.

// ── Transform ─────────────────────────────────────────────────────────────────

/// TRS (translate / rotate / scale) transform.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct Transform {
    pub translation: [f32; 3],
    /// Rotation quaternion stored as [x, y, z, w].
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl Transform {
    /// The identity transform: no translation, no rotation, unit scale.
    #[allow(dead_code)]
    pub fn identity() -> Self {
        Transform {
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        }
    }

    /// Identity transform with a custom translation.
    #[allow(dead_code)]
    pub fn with_translation(t: [f32; 3]) -> Self {
        Transform {
            translation: t,
            ..Transform::identity()
        }
    }

    /// Uniform scale transform (no translation, no rotation).
    #[allow(dead_code)]
    pub fn with_scale(s: f32) -> Self {
        Transform {
            scale: [s, s, s],
            ..Transform::identity()
        }
    }

    /// Compute the column-major 4×4 TRS matrix.
    ///
    /// The matrix is `T * R * S` where R comes from the quaternion.
    #[allow(dead_code)]
    pub fn matrix(&self) -> [[f32; 4]; 4] {
        let [qx, qy, qz, qw] = self.rotation;
        let [sx, sy, sz] = self.scale;
        let [tx, ty, tz] = self.translation;

        // Rotation matrix from quaternion (column-major).
        let x2 = qx + qx;
        let y2 = qy + qy;
        let z2 = qz + qz;
        let xx = qx * x2;
        let xy = qx * y2;
        let xz = qx * z2;
        let yy = qy * y2;
        let yz = qy * z2;
        let zz = qz * z2;
        let wx = qw * x2;
        let wy = qw * y2;
        let wz = qw * z2;

        // Rotation + scale columns.
        let r00 = (1.0 - (yy + zz)) * sx;
        let r10 = (xy + wz) * sx;
        let r20 = (xz - wy) * sx;

        let r01 = (xy - wz) * sy;
        let r11 = (1.0 - (xx + zz)) * sy;
        let r21 = (yz + wx) * sy;

        let r02 = (xz + wy) * sz;
        let r12 = (yz - wx) * sz;
        let r22 = (1.0 - (xx + yy)) * sz;

        // Column-major: m[col][row]
        [
            [r00, r10, r20, 0.0],
            [r01, r11, r21, 0.0],
            [r02, r12, r22, 0.0],
            [tx, ty, tz, 1.0],
        ]
    }
}

// ── Light ─────────────────────────────────────────────────────────────────────

/// Type of light source.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LightKind {
    Directional,
    Point,
    Spot,
}

/// A light source attached to a scene node.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Light {
    pub kind: LightKind,
    /// RGB colour (linear, typically 0..1).
    pub color: [f32; 3],
    /// Luminous intensity (lux for directional, candela for point/spot).
    pub intensity: f32,
    /// Maximum effective range (meters).  `None` for directional lights.
    pub range: Option<f32>,
}

// ── NodeContent ───────────────────────────────────────────────────────────────

/// Payload stored in a [`SceneNode`].
#[allow(dead_code)]
pub enum NodeContent {
    Empty,
    Mesh {
        mesh_idx: usize,
        material_idx: usize,
    },
    Light(Light),
    Camera {
        fov_y_deg: f32,
        near: f32,
        far: f32,
    },
}

// ── SceneNode / Scene ─────────────────────────────────────────────────────────

/// A single node in the scene graph.
#[allow(dead_code)]
pub struct SceneNode {
    pub name: String,
    pub transform: Transform,
    pub content: NodeContent,
    /// Indices into [`Scene::nodes`] that are direct children of this node.
    pub children: Vec<usize>,
}

/// The scene graph root container.
#[allow(dead_code)]
pub struct Scene {
    pub nodes: Vec<SceneNode>,
    /// Top-level node indices (nodes without a parent).
    pub root_nodes: Vec<usize>,
}

impl Scene {
    /// Create an empty scene.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Scene {
            nodes: Vec::new(),
            root_nodes: Vec::new(),
        }
    }

    /// Append a node and return its index.
    ///
    /// The node is also added to `root_nodes`; call [`Scene::add_child`] to
    /// re-parent it afterwards if needed.
    #[allow(dead_code)]
    pub fn add_node(&mut self, node: SceneNode) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(node);
        self.root_nodes.push(idx);
        idx
    }

    /// Record `child` as a child of `parent` and remove it from `root_nodes`.
    #[allow(dead_code)]
    pub fn add_child(&mut self, parent: usize, child: usize) {
        if let Some(p) = self.nodes.get_mut(parent) {
            if !p.children.contains(&child) {
                p.children.push(child);
            }
        }
        self.root_nodes.retain(|&r| r != child);
    }

    /// Compute the world-space transform matrix for the given node by
    /// accumulating transforms from a root ancestor down to the node.
    #[allow(dead_code)]
    pub fn world_transform(&self, node_idx: usize) -> [[f32; 4]; 4] {
        // Build the ancestor chain using a linear search (scene is small).
        let chain = self.ancestor_chain(node_idx);
        let mut mat = mat4_identity();
        for &idx in chain.iter().rev() {
            let local = self.nodes[idx].transform.matrix();
            mat = mat4_multiply(&mat, &local);
        }
        mat
    }

    /// Returns a vec of node indices on the path from a root down to `node_idx`
    /// (inclusive of `node_idx`, ordered root-first).
    fn ancestor_chain(&self, node_idx: usize) -> Vec<usize> {
        // Find the parent of each node via a reverse scan.
        let parent_of = |target: usize| -> Option<usize> {
            self.nodes
                .iter()
                .enumerate()
                .find(|(_, n)| n.children.contains(&target))
                .map(|(i, _)| i)
        };

        let mut chain = vec![node_idx];
        let mut current = node_idx;
        while let Some(p) = parent_of(current) {
            chain.push(p);
            current = p;
        }
        chain
    }

    /// Indices of all nodes that carry [`NodeContent::Mesh`] content.
    #[allow(dead_code)]
    pub fn nodes_with_mesh(&self) -> Vec<usize> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(i, n)| {
                if matches!(n.content, NodeContent::Mesh { .. }) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Collect all light nodes as `(node_index, &Light)` pairs.
    #[allow(dead_code)]
    pub fn lights(&self) -> Vec<(usize, &Light)> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(i, n)| {
                if let NodeContent::Light(ref l) = n.content {
                    Some((i, l))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Find the first node whose name equals `name`.
    #[allow(dead_code)]
    pub fn find_node(&self, name: &str) -> Option<usize> {
        self.nodes
            .iter()
            .enumerate()
            .find(|(_, n)| n.name == name)
            .map(|(i, _)| i)
    }
}

impl Default for Scene {
    fn default() -> Self {
        Scene::new()
    }
}

// ── Matrix helpers ────────────────────────────────────────────────────────────

/// 4×4 identity matrix in column-major order.
#[allow(dead_code)]
pub fn mat4_identity() -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// Multiply two column-major 4×4 matrices: returns `a * b`.
#[allow(dead_code)]
pub fn mat4_multiply(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut out = [[0.0f32; 4]; 4];
    // out[col][row] = sum_k a[k][row] * b[col][k]
    for col in 0..4 {
        for row in 0..4 {
            let mut s = 0.0f32;
            for k in 0..4 {
                s += a[k][row] * b[col][k];
            }
            out[col][row] = s;
        }
    }
    out
}

// ── Default scene ─────────────────────────────────────────────────────────────

/// Build a minimal default scene with one body mesh, one directional light,
/// and one camera.
#[allow(dead_code)]
pub fn default_scene() -> Scene {
    let mut scene = Scene::new();

    scene.add_node(SceneNode {
        name: "body".to_string(),
        transform: Transform::identity(),
        content: NodeContent::Mesh {
            mesh_idx: 0,
            material_idx: 0,
        },
        children: Vec::new(),
    });

    scene.add_node(SceneNode {
        name: "sun".to_string(),
        transform: Transform::identity(),
        content: NodeContent::Light(Light {
            kind: LightKind::Directional,
            color: [1.0, 0.98, 0.95],
            intensity: 1.0,
            range: None,
        }),
        children: Vec::new(),
    });

    scene.add_node(SceneNode {
        name: "camera".to_string(),
        transform: Transform::with_translation([0.0, 1.0, -3.0]),
        content: NodeContent::Camera {
            fov_y_deg: 60.0,
            near: 0.01,
            far: 1000.0,
        },
        children: Vec::new(),
    });

    scene
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq_mat(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> bool {
        for col in 0..4 {
            for row in 0..4 {
                if (a[col][row] - b[col][row]).abs() > 1e-5 {
                    return false;
                }
            }
        }
        true
    }

    #[test]
    fn transform_identity_matrix_is_identity() {
        let t = Transform::identity();
        let m = t.matrix();
        let expected = mat4_identity();
        assert!(
            approx_eq_mat(&m, &expected),
            "identity transform must yield identity matrix"
        );
    }

    #[test]
    fn transform_with_translation() {
        let t = Transform::with_translation([1.0, 2.0, 3.0]);
        let m = t.matrix();
        // Translation lives in column 3 (rows 0..2).
        assert!((m[3][0] - 1.0).abs() < 1e-5);
        assert!((m[3][1] - 2.0).abs() < 1e-5);
        assert!((m[3][2] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn transform_with_scale() {
        let t = Transform::with_scale(2.0);
        let m = t.matrix();
        // Diagonal of the rotation-scale block should be 2.
        assert!((m[0][0] - 2.0).abs() < 1e-5);
        assert!((m[1][1] - 2.0).abs() < 1e-5);
        assert!((m[2][2] - 2.0).abs() < 1e-5);
        // No translation.
        assert!((m[3][0]).abs() < 1e-5);
        assert!((m[3][1]).abs() < 1e-5);
        assert!((m[3][2]).abs() < 1e-5);
    }

    #[test]
    fn mat4_identity_is_identity() {
        let id = mat4_identity();
        for (col, col_data) in id.iter().enumerate() {
            for (row, &val) in col_data.iter().enumerate() {
                let expected = if col == row { 1.0 } else { 0.0 };
                assert!((val - expected).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn mat4_multiply_identity_identity() {
        let id = mat4_identity();
        let result = mat4_multiply(&id, &id);
        assert!(approx_eq_mat(&result, &id));
    }

    #[test]
    fn mat4_multiply_translation_accumulates() {
        let t1 = Transform::with_translation([1.0, 0.0, 0.0]).matrix();
        let t2 = Transform::with_translation([0.0, 2.0, 0.0]).matrix();
        let combined = mat4_multiply(&t1, &t2);
        // Translation column.
        assert!((combined[3][0] - 1.0).abs() < 1e-5);
        assert!((combined[3][1] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn scene_new_is_empty() {
        let s = Scene::new();
        assert!(s.nodes.is_empty());
        assert!(s.root_nodes.is_empty());
    }

    #[test]
    fn scene_add_node_returns_index() {
        let mut s = Scene::new();
        let idx = s.add_node(SceneNode {
            name: "root".to_string(),
            transform: Transform::identity(),
            content: NodeContent::Empty,
            children: Vec::new(),
        });
        assert_eq!(idx, 0);
        assert_eq!(s.nodes.len(), 1);
    }

    #[test]
    fn scene_add_child_links_nodes() {
        let mut s = Scene::new();
        let parent = s.add_node(SceneNode {
            name: "parent".to_string(),
            transform: Transform::identity(),
            content: NodeContent::Empty,
            children: Vec::new(),
        });
        let child = s.add_node(SceneNode {
            name: "child".to_string(),
            transform: Transform::identity(),
            content: NodeContent::Empty,
            children: Vec::new(),
        });
        s.add_child(parent, child);
        assert!(s.nodes[parent].children.contains(&child));
        // child should no longer be a root.
        assert!(!s.root_nodes.contains(&child));
    }

    #[test]
    fn scene_find_node_found() {
        let mut s = Scene::new();
        s.add_node(SceneNode {
            name: "body".to_string(),
            transform: Transform::identity(),
            content: NodeContent::Empty,
            children: Vec::new(),
        });
        assert_eq!(s.find_node("body"), Some(0));
    }

    #[test]
    fn scene_find_node_not_found() {
        let s = Scene::new();
        assert_eq!(s.find_node("missing"), None);
    }

    #[test]
    fn scene_nodes_with_mesh() {
        let mut s = Scene::new();
        s.add_node(SceneNode {
            name: "mesh_node".to_string(),
            transform: Transform::identity(),
            content: NodeContent::Mesh {
                mesh_idx: 0,
                material_idx: 0,
            },
            children: Vec::new(),
        });
        s.add_node(SceneNode {
            name: "empty_node".to_string(),
            transform: Transform::identity(),
            content: NodeContent::Empty,
            children: Vec::new(),
        });
        let mesh_nodes = s.nodes_with_mesh();
        assert_eq!(mesh_nodes.len(), 1);
        assert_eq!(mesh_nodes[0], 0);
    }

    #[test]
    fn scene_lights_returns_correct_count() {
        let s = default_scene();
        let lights = s.lights();
        assert_eq!(lights.len(), 1);
        assert!(matches!(lights[0].1.kind, LightKind::Directional));
    }

    #[test]
    fn world_transform_root_equals_local() {
        let mut s = Scene::new();
        let t = Transform::with_translation([5.0, 0.0, 0.0]);
        let expected = t.matrix();
        let idx = s.add_node(SceneNode {
            name: "n".to_string(),
            transform: t,
            content: NodeContent::Empty,
            children: Vec::new(),
        });
        let world = s.world_transform(idx);
        assert!(approx_eq_mat(&world, &expected));
    }

    #[test]
    fn world_transform_child_accumulates_parent() {
        let mut s = Scene::new();
        let parent = s.add_node(SceneNode {
            name: "parent".to_string(),
            transform: Transform::with_translation([1.0, 0.0, 0.0]),
            content: NodeContent::Empty,
            children: Vec::new(),
        });
        let child = s.add_node(SceneNode {
            name: "child".to_string(),
            transform: Transform::with_translation([0.0, 2.0, 0.0]),
            content: NodeContent::Empty,
            children: Vec::new(),
        });
        s.add_child(parent, child);
        let world = s.world_transform(child);
        // World translation should be parent + child.
        assert!((world[3][0] - 1.0).abs() < 1e-5, "x should be 1");
        assert!((world[3][1] - 2.0).abs() < 1e-5, "y should be 2");
    }

    #[test]
    fn default_scene_has_three_nodes() {
        let s = default_scene();
        assert_eq!(s.nodes.len(), 3);
    }

    #[test]
    fn default_scene_contains_mesh_light_camera() {
        let s = default_scene();
        assert_eq!(s.nodes_with_mesh().len(), 1);
        assert_eq!(s.lights().len(), 1);
        assert!(s.find_node("camera").is_some());
    }
}
