// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Huffman encoding/decoding (frequency-based).

#![allow(dead_code)]

use std::collections::{BinaryHeap, HashMap};
use std::cmp::Reverse;

/// A Huffman tree node.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum HuffNode {
    Leaf { symbol: u8, freq: u64 },
    Internal { freq: u64, left: Box<HuffNode>, right: Box<HuffNode> },
}

impl HuffNode {
    #[allow(dead_code)]
    pub fn freq(&self) -> u64 {
        match self {
            HuffNode::Leaf { freq, .. } => *freq,
            HuffNode::Internal { freq, .. } => *freq,
        }
    }
}

impl PartialEq for HuffNode {
    fn eq(&self, other: &Self) -> bool { self.freq() == other.freq() }
}
impl Eq for HuffNode {}
impl PartialOrd for HuffNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(other)) }
}
impl Ord for HuffNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering { self.freq().cmp(&other.freq()) }
}

/// Build frequency table from data.
#[allow(dead_code)]
pub fn build_freq_table(data: &[u8]) -> HashMap<u8, u64> {
    let mut freq = HashMap::new();
    for &b in data {
        *freq.entry(b).or_insert(0u64) += 1;
    }
    freq
}

/// Build Huffman tree from frequency table.
#[allow(dead_code)]
pub fn build_tree(freq: &HashMap<u8, u64>) -> Option<HuffNode> {
    if freq.is_empty() {
        return None;
    }
    let mut heap: BinaryHeap<Reverse<HuffNode>> = freq
        .iter()
        .map(|(&sym, &f)| Reverse(HuffNode::Leaf { symbol: sym, freq: f }))
        .collect();

    while heap.len() > 1 {
        let Some(Reverse(left)) = heap.pop() else { break };
        let Some(Reverse(right)) = heap.pop() else { break };
        let combined_freq = left.freq() + right.freq();
        heap.push(Reverse(HuffNode::Internal {
            freq: combined_freq,
            left: Box::new(left),
            right: Box::new(right),
        }));
    }
    heap.pop().map(|Reverse(node)| node)
}

/// Generate code table: symbol -> bit string (as Vec<bool>).
#[allow(dead_code)]
pub fn build_code_table(tree: &HuffNode) -> HashMap<u8, Vec<bool>> {
    let mut table = HashMap::new();
    build_codes(tree, &mut Vec::new(), &mut table);
    table
}

fn build_codes(node: &HuffNode, prefix: &mut Vec<bool>, table: &mut HashMap<u8, Vec<bool>>) {
    match node {
        HuffNode::Leaf { symbol, .. } => {
            let code = if prefix.is_empty() { vec![false] } else { prefix.clone() };
            table.insert(*symbol, code);
        }
        HuffNode::Internal { left, right, .. } => {
            prefix.push(false);
            build_codes(left, prefix, table);
            prefix.pop();
            prefix.push(true);
            build_codes(right, prefix, table);
            prefix.pop();
        }
    }
}

/// Encode data using code table, return bit vector.
#[allow(dead_code)]
pub fn encode(data: &[u8], codes: &HashMap<u8, Vec<bool>>) -> Vec<bool> {
    let mut bits = Vec::new();
    for &b in data {
        if let Some(code) = codes.get(&b) {
            bits.extend_from_slice(code);
        }
    }
    bits
}

/// Decode bit vector using Huffman tree.
#[allow(dead_code)]
pub fn decode(bits: &[bool], tree: &HuffNode) -> Vec<u8> {
    if bits.is_empty() {
        return Vec::new();
    }
    let mut output = Vec::new();
    let mut node = tree;
    for &bit in bits {
        match node {
            HuffNode::Leaf { symbol, .. } => {
                output.push(*symbol);
                node = tree;
                node = if bit {
                    if let HuffNode::Internal { right, .. } = node { right } else { break }
                } else if let HuffNode::Internal { left, .. } = node {
                    left
                } else {
                    break
                };
            }
            HuffNode::Internal { left, right, .. } => {
                node = if bit { right } else { left };
            }
        }
    }
    if let HuffNode::Leaf { symbol, .. } = node {
        output.push(*symbol);
    }
    output
}

/// Average code length (bits per symbol).
#[allow(dead_code)]
pub fn avg_code_length(freq: &HashMap<u8, u64>, codes: &HashMap<u8, Vec<bool>>) -> f64 {
    let total: u64 = freq.values().sum();
    if total == 0 {
        return 0.0;
    }
    let weighted: f64 = freq
        .iter()
        .map(|(sym, &f)| f as f64 * codes.get(sym).map_or(0, |c| c.len()) as f64)
        .sum();
    weighted / total as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let data = b"aabbbcccc";
        let freq = build_freq_table(data);
        let tree = build_tree(&freq).expect("should succeed");
        let codes = build_code_table(&tree);
        let bits = encode(data, &codes);
        let decoded = decode(&bits, &tree);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_freq_table() {
        let freq = build_freq_table(b"aabb");
        assert_eq!(freq[&b'a'], 2);
        assert_eq!(freq[&b'b'], 2);
    }

    #[test]
    fn test_single_symbol() {
        let data = b"aaaa";
        let freq = build_freq_table(data);
        let tree = build_tree(&freq).expect("should succeed");
        let codes = build_code_table(&tree);
        let bits = encode(data, &codes);
        let decoded = decode(&bits, &tree);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_empty_data() {
        let freq: HashMap<u8, u64> = HashMap::new();
        assert!(build_tree(&freq).is_none());
    }

    #[test]
    fn test_code_length_shorter_for_frequent() {
        let freq = build_freq_table(b"aaaabbbc");
        let tree = build_tree(&freq).expect("should succeed");
        let codes = build_code_table(&tree);
        let len_a = codes[&b'a'].len();
        let len_c = codes[&b'c'].len();
        assert!(len_a <= len_c, "a={len_a} c={len_c}");
    }

    #[test]
    fn test_avg_code_length() {
        let data = b"aabbbb";
        let freq = build_freq_table(data);
        let tree = build_tree(&freq).expect("should succeed");
        let codes = build_code_table(&tree);
        let avg = avg_code_length(&freq, &codes);
        assert!(avg > 0.0 && avg <= 8.0);
    }

    #[test]
    fn test_all_unique_symbols() {
        let data = b"abcdef";
        let freq = build_freq_table(data);
        let tree = build_tree(&freq).expect("should succeed");
        let codes = build_code_table(&tree);
        assert_eq!(codes.len(), 6);
    }

    #[test]
    fn test_tree_freq_root() {
        let freq = build_freq_table(b"aabbcc");
        let tree = build_tree(&freq).expect("should succeed");
        assert_eq!(tree.freq(), 6);
    }

    #[test]
    fn test_encode_produces_bits() {
        let data = b"abc";
        let freq = build_freq_table(data);
        let tree = build_tree(&freq).expect("should succeed");
        let codes = build_code_table(&tree);
        let bits = encode(data, &codes);
        assert!(!bits.is_empty());
    }

    #[test]
    fn test_decode_empty_bits() {
        let freq = build_freq_table(b"a");
        let tree = build_tree(&freq).expect("should succeed");
        let decoded = decode(&[], &tree);
        assert!(decoded.is_empty());
    }
}
