use pyo3::prelude::*;

#[derive(Debug, Clone)]
pub struct LevenshteinAutomaton<'a> {
    string: &'a str,
    len: usize,
    mask64: u64,
    chars64: [char; 64],
}

impl<'a> LevenshteinAutomaton<'a> {
    pub fn new(string: &'a str) -> Self {
        let len = string.chars().count();
        Self::new_assume_len(string, len)
    }

    fn new_assume_len(string: &'a str, len: usize) -> Self {
        let mut chars64 = ['\0'; 64];
        for (i, c) in string.chars().take(64).enumerate() {
            chars64[i] = c;
        }
        Self {
            string,
            len,
            mask64: 1u64.checked_shl(len as u32).unwrap_or(0).wrapping_sub(1),
            chars64,
        }
    }

    pub fn start(&self) -> LevenshteinState {
        if self.len <= 64 {
            LevenshteinState::Bitvector(LevenshteinBitvector {
                m: self,
                vp: self.mask64,
                vn: 0,
                offset: 0,
            })
        } else {
            LevenshteinState::General(LevenshteinGeneral {
                m: self,
                v: (0..).take(self.len + 1).collect(),
            })
        }
    }

    pub fn distance(&self, other: &str) -> u32 {
        let mut state = self.start();
        for value in other.chars() {
            state.step_mut(value);
        }
        state.distance()
    }
}

#[derive(Debug, Clone)]
pub enum LevenshteinState<'a> {
    General(LevenshteinGeneral<'a>),
    Bitvector(LevenshteinBitvector<'a>),
}

pub trait AutomatonState {
    fn step_mut(&mut self, value: char);
    fn step(&self, value: char) -> Self;
    fn distance(&self) -> u32;
    fn can_match(&self, max_edits: u32) -> bool;
}

impl AutomatonState for LevenshteinState<'_> {
    fn step_mut(&mut self, value: char) {
        match self {
            Self::General(s) => s.step_mut(value),
            Self::Bitvector(s) => s.step_mut(value),
        }
    }

    fn step(&self, value: char) -> Self {
        match self {
            Self::General(s) => Self::General(s.step(value)),
            Self::Bitvector(s) => Self::Bitvector(s.step(value)),
        }
    }

    fn distance(&self) -> u32 {
        match self {
            Self::General(s) => s.distance(),
            Self::Bitvector(s) => s.distance(),
        }
    }

    fn can_match(&self, max_edits: u32) -> bool {
        match self {
            Self::General(s) => s.can_match(max_edits),
            Self::Bitvector(s) => s.can_match(max_edits),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LevenshteinGeneral<'a> {
    m: &'a LevenshteinAutomaton<'a>,
    v: Vec<u32>,
}

impl AutomatonState for LevenshteinGeneral<'_> {
    fn step_mut(&mut self, value: char) {
        let mut sub = self.v[0];
        let mut add = sub + 1;
        let mut del;
        self.v[0] = add;
        for (i, c) in self.m.string.chars().enumerate() {
            del = self.v[i + 1];
            sub = if c == value { sub } else { sub + 1 };
            add = sub.min(add + 1).min(del + 1);
            sub = del;
            self.v[i + 1] = add;
        }
    }

    fn step(&self, value: char) -> Self {
        let mut new = self.clone();
        new.step_mut(value);
        new
    }

    fn distance(&self) -> u32 {
        *self.v.last().unwrap()
    }

    fn can_match(&self, max_edits: u32) -> bool {
        self.v.iter().min().unwrap() <= &max_edits
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LevenshteinBitvector<'a> {
    m: &'a LevenshteinAutomaton<'a>,
    vp: u64,
    vn: u64,
    offset: u32,
}

impl AutomatonState for LevenshteinBitvector<'_> {
    fn step_mut(&mut self, value: char) {
        // Myers as described by Hyyro
        // Step 1: D0
        let mut pm = 0;
        let mut x = 1u64;
        for c in &self.m.chars64[..self.m.len] {
            if c == &value {
                pm |= x;
            }
            x <<= 1;
        }
        let d0 = (((pm & self.vp).wrapping_add(self.vp)) ^ self.vp) | pm | self.vn;
        // Step 2-3: HP and HN
        let mut hp = self.vn | !(d0 | self.vp);
        let mut hn = d0 & self.vp;
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

        self.vp = hn | !(d0 | hp);
        self.vn = hp & d0;
        self.offset += 1;
    }

    fn step(&self, value: char) -> Self {
        let mut new = *self;
        new.step_mut(value);
        new
    }

    fn distance(&self) -> u32 {
        self.offset + (self.vp & self.m.mask64).count_ones()
            - (self.vn & self.m.mask64).count_ones()
    }

    fn can_match(&self, max_edits: u32) -> bool {
        self.offset <= max_edits || {
            let mut vpi = self.vp & self.m.mask64;
            let mut nvni = !(self.vn & self.m.mask64);
            while vpi != 0 && !nvni != 0 {
                // The minimum is preserved in this operation
                // Earlier positive steps cancel out later negative ones
                let x = nvni.wrapping_add(vpi);
                vpi &= x;
                nvni |= x;
            }
            self.offset - nvni.count_zeros()
        } <= max_edits
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
    automaton.distance(b)
}

/// Find the best match in a list of choices
///
/// Returns (choice, distance, index) or None (for empty choices)
#[pyfunction]
pub fn levenshtein_extract(query: &str, choices: Vec<&str>) -> Option<(String, u32, usize)> {
    let mut best = None;
    let automaton = LevenshteinAutomaton::new(query);
    for (i, x) in choices.iter().enumerate() {
        let distance = automaton.distance(x);
        best = Some(best.unwrap_or((distance, i, x)).min((distance, i, x)));
        if distance == 0 {
            break;
        }
    }
    best.map(|x| (x.2.to_string(), x.0, x.1))
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
    fn extract() {
        assert_eq!(levenshtein_extract("foo", vec![]), None);
        assert_eq!(
            levenshtein_extract("bar", vec!["bar"]),
            Some((String::from("bar"), 0, 0))
        );
        assert_eq!(
            levenshtein_extract("baz", vec!["foo", "bar"]),
            Some((String::from("bar"), 1, 1))
        );
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
