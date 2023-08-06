use pyo3::prelude::*;

pub trait AutomatonState {
    fn step(&self, value: char) -> Self;
    fn distance(&self) -> u32;
    fn can_match(&self, max_edits: u32) -> bool;
}

pub struct LevenshteinAutomaton<'a> {
    string: &'a str,
    len: usize,
    mask: u64,
    chars: [char; 64],
}

#[derive(Clone)]
enum LevenshteinState {
    General(Vec<u32>),
    Bitvector { vp: u64, vn: u64, offset: u32 },
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
        Self::new_assume_len(string, len)
    }

    fn new_assume_len(string: &'a str, len: usize) -> Self {
        let mut chars = ['\0'; 64];
        for (i, c) in string.chars().take(64).enumerate() {
            chars[i] = c;
        }
        Self {
            string,
            len,
            mask: 1u64.checked_shl(len as u32).unwrap_or(0).wrapping_sub(1),
            chars,
        }
    }

    pub fn start(&self) -> LevenshteinAutomatonState {
        LevenshteinAutomatonState {
            m: self,
            state: if self.len <= 64 {
                LevenshteinState::Bitvector {
                    vp: self.mask,
                    vn: 0,
                    offset: 0,
                }
            } else {
                LevenshteinState::General((0..).take(self.len + 1).collect())
            },
        }
    }
}

impl LevenshteinAutomatonState<'_> {
    fn step_mut(&mut self, value: char) {
        match self.state {
            General(ref mut v) => {
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
                for c in &self.m.chars[..self.m.len] {
                    if c == &value {
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
                hn <<= 1;

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

    fn distance(&self) -> u32 {
        match &self.state {
            General(v) => *v.last().unwrap(),
            Bitvector { vp, vn, offset } => {
                offset + (vp & self.m.mask).count_ones() - (vn & self.m.mask).count_ones()
            }
        }
    }

    fn can_match(&self, max_edits: u32) -> bool {
        match &self.state {
            General(v) => v.iter().min().unwrap() <= &max_edits,
            Bitvector { vp, vn, offset } => {
                offset <= &max_edits || {
                    let mut vpi = vp & self.m.mask;
                    let mut nvni = !(vn & self.m.mask);
                    while vpi != 0 && !nvni != 0 {
                        // The minimum is preserved in this operation
                        // Earlier positive steps cancel out later negative ones
                        let x = nvni.wrapping_add(vpi);
                        vpi &= x;
                        nvni |= x;
                    }
                    offset - nvni.count_zeros()
                } <= max_edits
            }
        }
    }
}

/// Find the Levenshtein distance between two strings
#[pyfunction]
pub fn levenshtein(a: &str, b: &str) -> u32 {
    if a == b {
        return 0;
    }
    let len_a = a.chars().count();
    let len_b = b.chars().count();

    let (a, len_a, b) = if (len_a < len_b || len_a > 64) && len_b <= 64 {
        (b, len_b, a)
    } else {
        (a, len_a, b)
    };
    let automaton = LevenshteinAutomaton::new_assume_len(a, len_a);
    let mut state = automaton.start();
    for value in b.chars() {
        state.step_mut(value);
    }
    state.distance()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distances() {
        assert_eq!(levenshtein("foo", "bar"), 3);
        assert_eq!(levenshtein("foo", ""), 3);
        assert_eq!(levenshtein("", "bar"), 3);
        assert_eq!(levenshtein("bar", "baz"), 1);
        assert_eq!(levenshtein("foo", "foo"), 0);
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("ab", "aacbb"), 3);

        assert_eq!(levenshtein(&"abcd".repeat(16), &"abcd".repeat(16)), 0);
        assert_eq!(levenshtein(&"abcde".repeat(13), &""), 65);
        assert_eq!(levenshtein(&"abcde".repeat(13), &"a".repeat(65)), 52);
        assert_eq!(levenshtein(&"abcd".repeat(64), &"abcd".repeat(16)), 192);
        assert_eq!(levenshtein(&"abcd".repeat(64), &"abcd".repeat(128)), 256);
    }

    #[test]
    fn automaton() {
        let automaton = LevenshteinAutomaton::new("kitten");
        let mut state = automaton.start();
        assert_eq!(state.distance(), 6);
        assert!(state.can_match(0));
        assert!(state.can_match(u32::MAX));

        state = state.step('s');
        assert_eq!(state.distance(), 6);
        assert!(!state.can_match(0));
        assert!(state.can_match(1));

        state = state.step('i');
        assert_eq!(state.distance(), 5);
        assert!(!state.can_match(0));
        assert!(state.can_match(1));

        state = state.step('t');
        assert_eq!(state.distance(), 4);
        assert!(!state.can_match(0));
        assert!(state.can_match(1));

        state = state.step('t');
        assert_eq!(state.distance(), 3);
        assert!(!state.can_match(0));
        assert!(state.can_match(1));

        state = state.step('i');
        assert_eq!(state.distance(), 3);
        assert!(!state.can_match(1));
        assert!(state.can_match(2));

        state = state.step('n');
        assert_eq!(state.distance(), 2);
        assert!(!state.can_match(1));
        assert!(state.can_match(2));

        state = state.step('g');
        assert_eq!(state.distance(), 3);
        assert!(!state.can_match(2));
        assert!(state.can_match(3));
    }

    #[test]
    fn long_automaton() {
        let string = "abcd".repeat(64);
        let automaton = LevenshteinAutomaton::new(&string);
        let mut state = automaton.start();
        for _i in 0..128 {
            state = state.step('a');
        }
        assert_eq!(state.distance(), 192);
        assert!(!state.can_match(0));
        assert!(!state.can_match(95));
        assert!(state.can_match(96));
        assert!(state.can_match(u32::MAX));
    }
}
