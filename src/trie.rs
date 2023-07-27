use pyo3::prelude::*;
use std::borrow::Cow;
use std::collections::HashMap;

use crate::automaton::{LevenshteinAutomaton, LevenshteinAutomatonState};

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
    ) -> (usize, Cow<'_, str>) {
        let mut best = usize::MAX;
        let mut value = "".into();
        if !state.can_match(max_edits) {
            return (best, value);
        }
        if self.is_terminal {
            best = best.min(state.distance());
        }
        for (next, subtrie) in self.children.iter() {
            let (distance, tail) =
                subtrie.find_automaton(&state.step(*next), max_edits.min(best - 1));
            if distance < best {
                best = distance;
                value = (next.to_string() + &tail).into();
            }
        }
        (best, value)
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
        let (distance, value) =
            self.find_automaton(&automaton.start(), max_edits.unwrap_or(usize::MAX));
        if distance <= max_edits.unwrap_or(usize::MAX) {
            return Some(value.to_string());
        }
        None
    }
}
