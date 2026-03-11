// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct MerkleTree {
    pub hashes: Vec<u64>,
    pub leaf_count: usize,
}

pub fn merkle_combine_hashes(a: u64, b: u64) -> u64 {
    a ^ b.rotate_left(17)
}

pub fn new_merkle_tree(leaf_hashes: &[u64]) -> MerkleTree {
    if leaf_hashes.is_empty() {
        return MerkleTree {
            hashes: vec![0],
            leaf_count: 0,
        };
    }
    let n = leaf_hashes.len();
    /* build a complete binary tree; pad with last leaf if odd */
    let mut level: Vec<u64> = leaf_hashes.to_vec();
    let mut all: Vec<u64> = level.clone();
    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        let mut i = 0;
        while i < level.len() {
            let a = level[i];
            let b = if i + 1 < level.len() {
                level[i + 1]
            } else {
                level[i]
            };
            next.push(merkle_combine_hashes(a, b));
            i += 2;
        }
        all.extend_from_slice(&next);
        level = next;
    }
    MerkleTree {
        hashes: all,
        leaf_count: n,
    }
}

pub fn merkle_root(t: &MerkleTree) -> u64 {
    *t.hashes.last().unwrap_or(&0)
}

pub fn merkle_leaf_count(t: &MerkleTree) -> usize {
    t.leaf_count
}

pub fn merkle_verify_leaf(t: &MerkleTree, index: usize, leaf_hash: u64) -> bool {
    if index >= t.leaf_count {
        return false;
    }
    t.hashes.get(index).copied() == Some(leaf_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_tree() {
        /* empty input gives root 0 */
        let t = new_merkle_tree(&[]);
        assert_eq!(merkle_root(&t), 0);
        assert_eq!(merkle_leaf_count(&t), 0);
    }

    #[test]
    fn single_leaf() {
        /* single leaf: root equals that leaf */
        let t = new_merkle_tree(&[42]);
        assert_eq!(t.hashes[0], 42);
        assert_eq!(merkle_leaf_count(&t), 1);
    }

    #[test]
    fn two_leaves() {
        /* two leaves combine into one root */
        let t = new_merkle_tree(&[1, 2]);
        assert_eq!(merkle_leaf_count(&t), 2);
        let expected = merkle_combine_hashes(1, 2);
        assert_eq!(merkle_root(&t), expected);
    }

    #[test]
    fn verify_leaf_valid() {
        /* valid index returns true when hash matches */
        let t = new_merkle_tree(&[10, 20, 30]);
        assert!(merkle_verify_leaf(&t, 1, 20));
    }

    #[test]
    fn verify_leaf_invalid_hash() {
        /* wrong hash returns false */
        let t = new_merkle_tree(&[10, 20, 30]);
        assert!(!merkle_verify_leaf(&t, 0, 99));
    }

    #[test]
    fn verify_leaf_out_of_bounds() {
        /* out-of-bounds index returns false */
        let t = new_merkle_tree(&[1, 2]);
        assert!(!merkle_verify_leaf(&t, 99, 0));
    }

    #[test]
    fn combine_hashes_symmetric_xor_variant() {
        /* combining then re-combining differs from identity */
        let h = merkle_combine_hashes(0xABCD, 0x1234);
        assert_ne!(h, 0xABCD);
    }
}
