use pyo3::prelude::*;

pub trait AutomatonState {
    fn step(&self, value: char) -> Self;
    fn distance(&self) -> usize;
    fn can_match(&self, max_edits: usize) -> bool;
}

pub struct LevenshteinAutomaton<'a> {
    string: &'a str,
    len: usize,
    mask: u64,
}

#[derive(Clone)]
enum LevenshteinState {
    Generic(Vec<usize>),
    Bitvector { vp: u64, vn: u64, offset: usize },
}

use LevenshteinState::*;

#[derive(Clone)]
pub struct LevenshteinAutomatonState<'a> {
    m: &'a LevenshteinAutomaton<'a>,
    state: LevenshteinState,
}

impl<'a> LevenshteinAutomaton<'a> {
    pub fn new(string: &'a str) -> Self {
        let len = string.chars().count();
        Self {
            string,
            len,
            mask: 1u64.checked_shl(len as u32).unwrap_or(0).wrapping_sub(1),
        }
    }

    pub fn start(&self) -> LevenshteinAutomatonState {
        LevenshteinAutomatonState {
            m: &self,
            state: if self.len <= 64 {
                LevenshteinState::Bitvector {
                    vp: self.mask,
                    vn: 0,
                    offset: 0,
                }
            } else {
                LevenshteinState::Generic((0..).take(self.len + 1).collect())
            },
        }
    }
}

impl LevenshteinAutomatonState<'_> {
    fn step_mut(&mut self, value: char) {
        match self.state {
            Generic(ref mut v) => {
                let mut sub = v[0];
                let mut add = sub + 1;
                let mut del;
                v[0] = add;
                for (i, c) in self.m.string.chars().enumerate() {
                    del = v[i + 1];
                    sub = if c == value { sub } else { sub + 1 };
                    add = sub.min(add + 1).min(del + 1);
                    sub = del;
                    v[i + 1] = add;
                }
            }
            Bitvector {
                ref mut vp,
                ref mut vn,
                ref mut offset,
            } => {
                // Myers as described by Hyyro
                // Step 1: D0
                let mut pm = 0;
                let mut x = 1u64;
                for c in self.m.string.chars() {
                    if c == value {
                        pm |= x;
                    }
                    x <<= 1;
                }
                let d0 = (((pm & *vp).wrapping_add(*vp)) ^ *vp) | pm | *vn;
                // Step 2-3: HP and HN
                let mut hp = *vn | !(d0 | *vp);
                let mut hn = d0 & *vp;
                // Step 4-5: D[m,j]
                // if (hp & mask) != 0 {
                //     score += 1;
                // }
                // if (hn & mask) != 0 {
                //     score -= 1;
                // }
                // Step 6-7: VP and VN
                hp = (hp << 1) | 1;
                hn = hn << 1;

                *vp = hn | !(d0 | hp);
                *vn = hp & d0;
                *offset += 1;
            }
        }
    }
}

impl AutomatonState for LevenshteinAutomatonState<'_> {
    fn step(&self, value: char) -> Self {
        let mut new = self.clone();
        new.step_mut(value);
        new
    }

    fn distance(&self) -> usize {
        match &self.state {
            Generic(v) => *v.last().unwrap(),
            Bitvector { vp, vn, offset } => {
                offset + (vp & self.m.mask).count_ones() as usize
                    - (vn & self.m.mask).count_ones() as usize
            }
        }
    }

    fn can_match(&self, max_edits: usize) -> bool {
        match &self.state {
            Generic(v) => v.iter().min().unwrap() <= &max_edits,
            Bitvector { vp, vn, offset } => {
                (0..)
                    .take(self.m.len)
                    .map(|i| 1 << i)
                    .scan(*offset, |state, mask| {
                        if vp & mask != 0 {
                            *state += 1;
                        }
                        if vn & mask != 0 {
                            *state -= 1;
                        }
                        Some(*state)
                    })
                    .min()
                    .unwrap_or(*offset)
                    <= max_edits
            }
        }
    }
}

/// Find the Levenshtein distance between two strings
#[pyfunction]
pub fn levenshtein(a: &str, b: &str) -> usize {
    if a == b {
        return 0;
    }
    let len_a = a.chars().count();
    let len_b = b.chars().count();

    let (a, b) = if (len_a < len_b || len_a > 64) && len_b <= 64 {
        (b, a)
    } else {
        (a, b)
    };
    let automaton = LevenshteinAutomaton::new(a);
    let mut state = automaton.start();
    for value in b.chars() {
        state.step_mut(value);
    }
    state.distance()
}
