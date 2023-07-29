use pyo3::prelude::*;
use std::collections::HashMap;

use crate::levenshtein::{AutomatonState, LevenshteinAutomaton};

struct FindResult<'a>(usize, &'a str);

#[pyclass]
pub struct Trie {
    // Indicates terminal and nice when traversing
    value: Option<String>,
    // Maybe expensive to iterate over O(capacity) rather than O(len)?
    children: HashMap<char, Trie>,
}

#[pymethods]
impl Trie {
    #[new]
    pub fn py_new(items: Option<Vec<String>>) -> Self {
        items.map_or_else(|| Self::new(), |v| Self::from_iter(v))
    }

    #[staticmethod]
    pub fn new() -> Self {
        Self {
            value: None,
            children: HashMap::new(),
        }
    }

    pub fn insert(&mut self, value: String) {
        let mut node = self;
        for c in value.chars() {
            node = node.children.entry(c).or_insert_with(|| Self::new());
        }
        node.value = Some(value);
    }

    pub fn get(&self, value: &str) -> Option<&str> {
        let mut node = self;
        for c in value.chars() {
            node = node.children.get(&c)?;
        }
        node.value.as_deref()
    }

    pub fn contains(&self, value: &str) -> bool {
        self.get(value).is_some()
    }

    pub fn values(&self) -> Vec<&str> {
        self.iter().collect()
    }

    pub fn find_one(&self, query: &str, max_edits: Option<usize>) -> Option<&str> {
        let automaton = LevenshteinAutomaton::new(query);
        Some(
            self.find_automaton(&automaton.start(), max_edits.unwrap_or(usize::MAX))?
                .1,
        )
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
    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a str> + 'a> {
        Box::new(
            self.value
                .iter()
                .map(|v| v.as_str())
                .chain(self.children.values().flat_map(|x| x.iter())),
        )
    }

    fn find_automaton(&self, state: &impl AutomatonState, max_edits: usize) -> Option<FindResult> {
        let mut best = None;
        if !state.can_match(max_edits) {
            return best;
        }
        let distance = state.distance();
        if distance <= max_edits {
            best = self
                .value
                .as_ref()
                .and_then(|k| Some(FindResult(distance, k)));
        }
        for (next, subtrie) in self.children.iter() {
            // Method returns some iff best is none or distance is lower
            if let Some(result) = subtrie.find_automaton(
                &state.step(*next),
                best.as_ref().map_or(max_edits, |x| x.0 - 1),
            ) {
                best = Some(result);
            };
        }
        best
    }
}
