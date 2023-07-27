use pyo3::prelude::*;
use std::collections::HashMap;

use crate::automaton::{LevenshteinAutomaton, LevenshteinAutomatonState};

struct FindResult(usize, Vec<char>);

#[pyclass]
pub struct Trie {
    is_terminal: bool,
    // Maybe expensive to iterate over O(capacity) rather than O(len)?
    children: HashMap<char, Trie>,
}

impl Trie {
    pub fn new() -> Self {
        Self {
            is_terminal: false,
            children: HashMap::new(),
        }
    }

    fn find_automaton(
        &self,
        state: &LevenshteinAutomatonState,
        max_edits: usize,
    ) -> Option<FindResult> {
        let mut best = None;
        if !state.can_match(max_edits) {
            return best;
        }
        let distance = state.distance();
        if self.is_terminal && distance <= max_edits {
            best = Some(FindResult(distance, Vec::new()));
        }
        for (next, subtrie) in self.children.iter() {
            // Method returns some iff best is none or distance is lower
            if let Some(mut result) = subtrie.find_automaton(
                &state.step(*next),
                best.as_ref().map_or(max_edits, |x| x.0 - 1),
            ) {
                result.1.push(*next);
                best = Some(result);
            };
        }
        best
    }
}

#[pymethods]
impl Trie {
    #[new]
    pub fn py_new(items: Option<Vec<&str>>) -> Self {
        let mut trie = Self::new();
        if let Some(items) = items {
            for item in items {
                trie.add(item);
            }
        }
        trie
    }

    pub fn add(&mut self, item: &str) {
        let mut node = self;
        for value in item.chars() {
            node = node.children.entry(value).or_insert_with(|| Self::new());
        }
        node.is_terminal = true;
    }

    fn find_one(&self, string: &str, max_edits: Option<usize>) -> Option<String> {
        let automaton = LevenshteinAutomaton::new(string);
        Some(
            self.find_automaton(&automaton.start(), max_edits.unwrap_or(usize::MAX))?
                .1
                .iter()
                .rev()
                .collect(),
        )
    }
}
