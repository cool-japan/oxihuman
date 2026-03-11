// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Fur card generation from surface normals.

/// A single fur card: a quad anchored at a surface point.
#[derive(Debug, Clone)]
pub struct FurCard {
    pub root: [f32; 3],
    pub tip: [f32; 3],
    pub width: f32,
    pub uvs: [[f32; 2]; 4],
}

impl FurCard {
    /// Create a fur card from root position, direction, length and width.
    pub fn new(root: [f32; 3], direction: [f32; 3], length: f32, width: f32) -> Self {
        let tip = [
            root[0] + direction[0] * length,
            root[1] + direction[1] * length,
            root[2] + direction[2] * length,
        ];
        let uvs = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        Self {
            root,
            tip,
            width,
            uvs,
        }
    }

    /// Return card length.
    pub fn length(&self) -> f32 {
        let dx = self.tip[0] - self.root[0];
        let dy = self.tip[1] - self.root[1];
        let dz = self.tip[2] - self.root[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// Fur card collection.
#[derive(Debug, Clone)]
pub struct FurCardMesh {
    pub cards: Vec<FurCard>,
}

impl FurCardMesh {
    /// Create empty fur card mesh.
    pub fn new() -> Self {
        Self { cards: Vec::new() }
    }

    /// Add a fur card.
    pub fn add_card(&mut self, card: FurCard) {
        self.cards.push(card);
    }

    /// Return card count.
    pub fn card_count(&self) -> usize {
        self.cards.len()
    }

    /// Total vertex count (4 per card).
    pub fn vertex_count(&self) -> usize {
        self.cards.len() * 4
    }

    /// Total index count (6 per card for two triangles).
    pub fn index_count(&self) -> usize {
        self.cards.len() * 6
    }
}

impl Default for FurCardMesh {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate fur cards from surface positions and normals.
pub fn generate_fur_cards(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    length: f32,
    width: f32,
) -> FurCardMesh {
    let mut mesh = FurCardMesh::new();
    let count = positions.len().min(normals.len());
    for i in 0..count {
        let card = FurCard::new(positions[i], normals[i], length, width);
        mesh.add_card(card);
    }
    mesh
}

/// Compute average card length.
pub fn average_card_length(mesh: &FurCardMesh) -> f32 {
    if mesh.cards.is_empty() {
        return 0.0;
    }
    let sum: f32 = mesh.cards.iter().map(|c| c.length()).sum();
    sum / mesh.cards.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card() -> FurCard {
        FurCard::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 5.0, 0.1)
    }

    #[test]
    fn test_card_length() {
        /* card length matches requested length */
        let c = card();
        assert!((c.length() - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_add_card() {
        /* card count increments correctly */
        let mut mesh = FurCardMesh::new();
        mesh.add_card(card());
        assert_eq!(mesh.card_count(), 1);
    }

    #[test]
    fn test_vertex_count() {
        /* vertex count is 4 per card */
        let mut mesh = FurCardMesh::new();
        mesh.add_card(card());
        mesh.add_card(card());
        assert_eq!(mesh.vertex_count(), 8);
    }

    #[test]
    fn test_index_count() {
        /* index count is 6 per card */
        let mut mesh = FurCardMesh::new();
        mesh.add_card(card());
        assert_eq!(mesh.index_count(), 6);
    }

    #[test]
    fn test_generate_fur_cards() {
        /* generate creates one card per position/normal pair */
        let positions = vec![[0.0_f32; 3]; 5];
        let normals = vec![[0.0_f32, 1.0, 0.0]; 5];
        let mesh = generate_fur_cards(&positions, &normals, 3.0, 0.1);
        assert_eq!(mesh.card_count(), 5);
    }

    #[test]
    fn test_average_card_length_empty() {
        /* average length on empty mesh is 0 */
        let mesh = FurCardMesh::new();
        assert_eq!(average_card_length(&mesh), 0.0);
    }

    #[test]
    fn test_average_card_length() {
        /* average length is correct */
        let mut mesh = FurCardMesh::new();
        mesh.add_card(FurCard::new([0.0; 3], [1.0, 0.0, 0.0], 4.0, 0.1));
        mesh.add_card(FurCard::new([0.0; 3], [1.0, 0.0, 0.0], 6.0, 0.1));
        assert!((average_card_length(&mesh) - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_generate_mismatched_lengths() {
        /* generate uses min of positions/normals length */
        let positions = vec![[0.0_f32; 3]; 3];
        let normals = vec![[0.0_f32, 1.0, 0.0]; 5];
        let mesh = generate_fur_cards(&positions, &normals, 2.0, 0.05);
        assert_eq!(mesh.card_count(), 3);
    }
}
