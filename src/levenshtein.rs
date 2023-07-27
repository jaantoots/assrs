use pyo3::prelude::*;

pub struct LevenshteinAutomaton<'a> {
    string: &'a str,
}

#[derive(Clone)]
pub struct LevenshteinAutomatonState<'a> {
    m: &'a LevenshteinAutomaton<'a>,
    row: Vec<usize>,
}

impl<'a> LevenshteinAutomaton<'a> {
    pub fn new(string: &'a str) -> Self {
        Self { string }
    }

    pub fn start(&self) -> LevenshteinAutomatonState {
        LevenshteinAutomatonState {
            m: &self,
            row: (0..).take(self.string.chars().count() + 1).collect(),
        }
    }
}

impl<'a> LevenshteinAutomatonState<'a> {
    pub fn step_mut(&mut self, value: char) {
        let mut sub = self.row[0];
        let mut add = sub + 1;
        let mut del;
        self.row[0] = add;
        for (i, c) in self.m.string.chars().enumerate() {
            del = self.row[i + 1];
            sub = if c == value { sub } else { sub + 1 };
            add = sub.min(add + 1).min(del + 1);
            sub = del;
            self.row[i + 1] = add;
        }
    }

    pub fn step(&self, value: char) -> Self {
        let mut new = self.clone();
        new.step_mut(value);
        new
    }

    pub fn distance(&self) -> usize {
        *self.row.last().unwrap()
    }

    pub fn can_match(&self, max_edits: usize) -> bool {
        self.row.iter().min().unwrap() <= &max_edits
    }
}

/// Find the Levenshtein distance between two strings
#[pyfunction]
pub fn levenshtein(a: &str, b: &str) -> usize {
    if a == b {
        return 0;
    }
    let automaton = LevenshteinAutomaton::new(a);
    let mut state = automaton.start();
    for value in b.chars() {
        state.step_mut(value);
    }
    state.distance()
}
