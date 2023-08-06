use pyo3::prelude::*;
use std::collections::HashMap;

use crate::levenshtein::{AutomatonState, LevenshteinAutomaton};

/// Trie storing the strings to search against
#[pyclass]
#[derive(Debug, Default, Clone)]
pub struct Trie {
    // Indicates terminal and nice when traversing
    value: Option<String>,
    // Expensive to iterate over HashMap as O(capacity) rather than O(len)
    children_index: HashMap<char, usize>,
    children: Vec<(char, Trie)>,
}

#[pymethods]
impl Trie {
    #[new]
    pub fn py_new(items: Option<Vec<String>>) -> Self {
        items.map_or_else(Self::new, Self::from_iter)
    }

    #[staticmethod]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, value: String) {
        let mut node = self;
        for c in value.chars() {
            let idx = node.children_index.entry(c).or_insert_with(|| {
                node.children.push((c, Self::new()));
                node.children.len() - 1
            });
            node = &mut node.children[*idx].1;
        }
        node.value = Some(value);
    }

    pub fn get(&self, value: &str) -> Option<&str> {
        let mut node = self;
        for c in value.chars() {
            let idx = node.children_index.get(&c)?;
            node = &node.children[*idx].1;
        }
        node.value.as_deref()
    }

    pub fn contains(&self, value: &str) -> bool {
        self.get(value).is_some()
    }

    pub fn values(&self) -> Vec<&str> {
        self.iter().collect()
    }

    /// Find best match in trie for query
    pub fn find_one(&self, query: &str, max_edits: Option<u32>) -> Option<(&str, u32)> {
        let automaton = LevenshteinAutomaton::new(query);
        self.find_automaton(&automaton.start(), max_edits.unwrap_or(u32::MAX))
    }
}

impl Extend<String> for Trie {
    fn extend<I: IntoIterator<Item = String>>(&mut self, iter: I) {
        for item in iter {
            self.insert(item);
        }
    }
}

impl FromIterator<String> for Trie {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        let mut trie = Self::new();
        trie.extend(iter);
        trie
    }
}

impl<'a> IntoIterator for &'a Trie {
    type Item = &'a str;
    type IntoIter = Box<dyn Iterator<Item = &'a str> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Trie {
    pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a str> + 'a> {
        Box::new(
            self.value
                .iter()
                .map(|v| v.as_str())
                .chain(self.children.iter().flat_map(|x| x.1.iter())),
        )
    }

    fn find_automaton(&self, state: &impl AutomatonState, max_edits: u32) -> Option<(&str, u32)> {
        if !state.can_match(max_edits) {
            return None;
        }
        let this = self
            .value
            .as_ref()
            .map(|v| (v.as_str(), state.distance()))
            .filter(|x| x.1 <= max_edits);
        self.children.iter().fold(this, |best, (next, subtrie)| {
            // Method returns some iff best is none or distance is lower
            best.map_or(Some(max_edits), |x| x.1.checked_sub(1))
                .and_then(|max_edits| subtrie.find_automaton(&state.step(*next), max_edits))
                .or(best)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn empty_str() {
        let mut trie = Trie::new();
        assert!(!trie.contains(""));
        assert_eq!(trie.iter().count(), 0);
        trie.insert("".to_string());
        assert!(trie.contains(""));
        assert_eq!(trie.iter().collect::<Vec<_>>(), vec![""]);
    }

    #[test]
    fn values() {
        let mut trie = Trie::new();
        assert!(!trie.contains(""));

        trie.insert("foo".to_string());
        trie.insert("bar".to_string());
        trie.insert("baz".to_string());
        assert!(!trie.contains(""));
        assert!(trie.contains("foo"));
        assert_eq!(
            trie.iter().collect::<HashSet<_>>(),
            HashSet::from(["foo", "bar", "baz"])
        );

        trie.insert("".to_string());
        assert!(trie.contains(""));
        assert_eq!(
            trie.iter().collect::<HashSet<_>>(),
            HashSet::from(["foo", "bar", "baz", ""])
        );
    }

    #[test]
    fn find() {
        let trie = Trie::from_iter(vec!["foo".to_string(), "bar".to_string()]);
        assert_eq!(trie.find_one("", Some(2)), None);
        assert_eq!(trie.find_one("baz", Some(2)), Some(("bar", 1)));
        assert_eq!(trie.find_one("baz", None), Some(("bar", 1)));
        assert_eq!(trie.find_one("baz", Some(0)), None);
    }
}
