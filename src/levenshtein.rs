use pyo3::prelude::*;
use std::collections::HashMap;

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

pub struct LevenshteinAutomaton64 {
    block: HashMap<char, u64>,
    len: u32,
}

pub struct LevenshteinAutomaton64State<'a> {
    m: &'a LevenshteinAutomaton64,
    vp: u64,
    vn: u64,
    offset: usize,
}

impl LevenshteinAutomaton64 {
    pub fn new(string: &str) -> Self {
        let mut block = HashMap::new();
        let mut x = 1;
        let mut len = 0;
        for c in string.chars() {
            block.entry(c).and_modify(|e| *e |= x).or_insert(x);
            x <<= 1;
            len += 1;
        }
        Self { block, len }
    }

    pub fn start(&self) -> LevenshteinAutomaton64State {
        LevenshteinAutomaton64State {
            m: &self,
            vp: 1u64.checked_shl(self.len).unwrap_or(0).wrapping_sub(1),
            vn: 0,
            offset: 0,
        }
    }
}

impl LevenshteinAutomaton64State<'_> {
    pub fn step(&self, value: char) -> Self {
        // Myers as described by Hyyro
        // Step 1: D0
        let pm = *self.m.block.get(&value).unwrap_or(&0);
        let d0 = (((pm & self.vp).wrapping_add(self.vp)) ^ self.vp) | pm | self.vn;
        // Step 2-3: HP and HN
        let mut hp = self.vn | !(d0 | self.vp);
        let mut hn = d0 & self.vp;
        // Step 4-5: D[m,j]
        // currDist += (hp & mask) != 0
        // currDist -= (hn & mask) != 0
        // Step 6-7: VP and VN
        hp = (hp << 1) | 1;
        hn = hn << 1;
        Self {
            m: self.m,
            vp: hn | !(d0 | hp),
            vn: hp & d0,
            offset: self.offset + 1,
        }
    }

    pub fn distance(&self) -> usize {
        let mask = 1u64.checked_shl(self.m.len).unwrap_or(0).wrapping_sub(1);
        self.offset + (self.vp & mask).count_ones() as usize
            - (self.vn & mask).count_ones() as usize
    }

    pub fn can_match(&self, max_edits: usize) -> bool {
        let mut current = self.offset;
        (0..)
            .take(self.m.len as usize)
            .map(|i| {
                let mask = 1 << i;
                if self.vp & mask != 0 {
                    current += 1;
                }
                if self.vn & mask != 0 {
                    current -= 1;
                }
                current
            })
            .min()
            .unwrap_or(self.offset)
            <= max_edits
    }
}

/// Find the Levenshtein distance between two strings
#[pyfunction]
pub fn levenshtein(a: &str, b: &str) -> usize {
    if a == b {
        return 0;
    }
    if a.chars().count() <= 64 {
        let automaton = LevenshteinAutomaton64::new(a);
        let mut state = automaton.start();
        for value in b.chars() {
            state = state.step(value);
        }
        return state.distance();
    }
    let automaton = LevenshteinAutomaton::new(a);
    let mut state = automaton.start();
    for value in b.chars() {
        state.step_mut(value);
    }
    state.distance()
}
