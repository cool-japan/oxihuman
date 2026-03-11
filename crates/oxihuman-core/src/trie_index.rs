//! Prefix-trie index for fast string prefix lookups.
//!
//! Each character in a key is stored as an ASCII byte in a flat node list.
//! Supports insertion, exact-match lookup, prefix search, and deletion.
//! Useful for morph-target name auto-completion and hierarchical asset search.

/// Configuration for the trie index.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TrieConfig {
    /// Maximum number of trie nodes (each character uses one node).
    pub max_nodes: usize,
}

#[allow(dead_code)]
impl TrieConfig {
    fn new() -> Self {
        Self { max_nodes: 4096 }
    }
}

/// Returns the default trie configuration.
#[allow(dead_code)]
pub fn default_trie_config() -> TrieConfig {
    TrieConfig::new()
}

/// A single node in the trie.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TrieNode {
    /// Child indices into the node array, keyed by byte value.
    /// Value of 0 means "no child" (root is always index 0, never a valid child).
    pub children: [u32; 128],
    /// Whether this node marks the end of a complete word.
    pub is_end: bool,
}

impl TrieNode {
    fn new() -> Self {
        Self {
            children: [0u32; 128],
            is_end: false,
        }
    }
}

/// Prefix-trie index.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TrieIndex {
    config: TrieConfig,
    nodes: Vec<TrieNode>,
    word_count: usize,
}

/// Creates a new `TrieIndex` with the given configuration.
/// The root node is always at index 0.
#[allow(dead_code)]
pub fn new_trie_index(config: TrieConfig) -> TrieIndex {
    TrieIndex {
        config,
        nodes: vec![TrieNode::new()], // root
        word_count: 0,
    }
}

/// Inserts `word` into the trie. Returns `false` if the node limit is reached.
#[allow(dead_code)]
pub fn trie_insert(trie: &mut TrieIndex, word: &str) -> bool {
    let mut cur: usize = 0;
    for byte in word.bytes() {
        let idx = byte as usize;
        if idx >= 128 {
            continue; // ignore non-ASCII
        }
        if trie.nodes[cur].children[idx] == 0 {
            if trie.nodes.len() >= trie.config.max_nodes {
                return false;
            }
            let new_idx = trie.nodes.len() as u32;
            trie.nodes.push(TrieNode::new());
            trie.nodes[cur].children[idx] = new_idx;
        }
        cur = trie.nodes[cur].children[idx] as usize;
    }
    if !trie.nodes[cur].is_end {
        trie.nodes[cur].is_end = true;
        trie.word_count += 1;
    }
    true
}

/// Returns `true` if `word` is an exact match in the trie.
#[allow(dead_code)]
pub fn trie_contains(trie: &TrieIndex, word: &str) -> bool {
    let mut cur: usize = 0;
    for byte in word.bytes() {
        let idx = byte as usize;
        if idx >= 128 {
            return false;
        }
        let next = trie.nodes[cur].children[idx] as usize;
        if next == 0 {
            return false;
        }
        cur = next;
    }
    trie.nodes[cur].is_end
}

/// Collects all words in the subtree rooted at `node_idx`, prepending `prefix`.
fn collect_words(trie: &TrieIndex, node_idx: usize, prefix: &str, results: &mut Vec<String>) {
    if trie.nodes[node_idx].is_end {
        results.push(prefix.to_string());
    }
    for (byte_val, &child_idx) in trie.nodes[node_idx].children.iter().enumerate() {
        if child_idx != 0 {
            let ch = byte_val as u8 as char;
            let mut new_prefix = prefix.to_string();
            new_prefix.push(ch);
            collect_words(trie, child_idx as usize, &new_prefix, results);
        }
    }
}

/// Returns all words that start with `prefix`.
#[allow(dead_code)]
pub fn trie_prefix_search(trie: &TrieIndex, prefix: &str) -> Vec<String> {
    let mut cur: usize = 0;
    for byte in prefix.bytes() {
        let idx = byte as usize;
        if idx >= 128 {
            return Vec::new();
        }
        let next = trie.nodes[cur].children[idx] as usize;
        if next == 0 {
            return Vec::new();
        }
        cur = next;
    }
    let mut results = Vec::new();
    collect_words(trie, cur, prefix, &mut results);
    results
}

/// Removes `word` from the trie. Returns `true` if the word was present.
/// Note: does not reclaim nodes (they remain in the pool for future use).
#[allow(dead_code)]
pub fn trie_remove(trie: &mut TrieIndex, word: &str) -> bool {
    let mut cur: usize = 0;
    for byte in word.bytes() {
        let idx = byte as usize;
        if idx >= 128 {
            return false;
        }
        let next = trie.nodes[cur].children[idx] as usize;
        if next == 0 {
            return false;
        }
        cur = next;
    }
    if trie.nodes[cur].is_end {
        trie.nodes[cur].is_end = false;
        trie.word_count -= 1;
        true
    } else {
        false
    }
}

/// Returns the total number of nodes allocated (including root).
#[allow(dead_code)]
pub fn trie_node_count(trie: &TrieIndex) -> usize {
    trie.nodes.len()
}

/// Returns the number of complete words stored in the trie.
#[allow(dead_code)]
pub fn trie_word_count(trie: &TrieIndex) -> usize {
    trie.word_count
}

/// Serialises the trie metadata to a simple JSON string.
#[allow(dead_code)]
pub fn trie_to_json(trie: &TrieIndex) -> String {
    format!(
        "{{\"node_count\":{},\"word_count\":{}}}",
        trie.nodes.len(),
        trie.word_count
    )
}

/// Clears all words from the trie, resetting to a single root node.
#[allow(dead_code)]
pub fn trie_clear(trie: &mut TrieIndex) {
    trie.nodes.clear();
    trie.nodes.push(TrieNode::new());
    trie.word_count = 0;
}

// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make_trie() -> TrieIndex {
        let mut t = new_trie_index(default_trie_config());
        trie_insert(&mut t, "hello");
        trie_insert(&mut t, "help");
        trie_insert(&mut t, "world");
        trie_insert(&mut t, "word");
        t
    }

    #[test]
    fn test_contains_inserted_word() {
        let t = make_trie();
        assert!(trie_contains(&t, "hello"));
        assert!(trie_contains(&t, "world"));
    }

    #[test]
    fn test_not_contains_absent_word() {
        let t = make_trie();
        assert!(!trie_contains(&t, "hell")); // prefix, not full word
        assert!(!trie_contains(&t, "xyz"));
    }

    #[test]
    fn test_word_count() {
        let t = make_trie();
        assert_eq!(trie_word_count(&t), 4);
    }

    #[test]
    fn test_prefix_search_hel() {
        let t = make_trie();
        let mut results = trie_prefix_search(&t, "hel");
        results.sort();
        assert_eq!(results, vec!["hello", "help"]);
    }

    #[test]
    fn test_prefix_search_wor() {
        let t = make_trie();
        let mut results = trie_prefix_search(&t, "wor");
        results.sort();
        assert_eq!(results, vec!["word", "world"]);
    }

    #[test]
    fn test_prefix_search_no_match() {
        let t = make_trie();
        let results = trie_prefix_search(&t, "xyz");
        assert!(results.is_empty());
    }

    #[test]
    fn test_remove_word() {
        let mut t = make_trie();
        assert!(trie_remove(&mut t, "hello"));
        assert!(!trie_contains(&t, "hello"));
        assert_eq!(trie_word_count(&t), 3);
    }

    #[test]
    fn test_remove_absent_word() {
        let mut t = make_trie();
        assert!(!trie_remove(&mut t, "nope"));
    }

    #[test]
    fn test_clear() {
        let mut t = make_trie();
        trie_clear(&mut t);
        assert_eq!(trie_word_count(&t), 0);
        assert_eq!(trie_node_count(&t), 1); // root only
    }

    #[test]
    fn test_to_json() {
        let t = make_trie();
        let json = trie_to_json(&t);
        assert!(json.contains("word_count"));
        assert!(json.contains("node_count"));
    }

    #[test]
    fn test_duplicate_insert_no_double_count() {
        let mut t = make_trie();
        trie_insert(&mut t, "hello");
        assert_eq!(trie_word_count(&t), 4);
    }
}
