pub struct LevenshteinAutomaton {
    chars: Vec<char>,
}

pub struct LevenshteinAutomatonState<'a> {
    m: &'a LevenshteinAutomaton,
    row: Vec<usize>,
}

impl LevenshteinAutomaton {
    pub fn new(string: &str) -> Self {
        Self {
            chars: string.chars().collect(),
        }
    }

    pub fn start(&self) -> LevenshteinAutomatonState {
        LevenshteinAutomatonState {
            m: &self,
            row: (0..).take(self.chars.len() + 1).collect(),
        }
    }
}

impl<'a> LevenshteinAutomatonState<'a> {
    pub fn step(&self, value: char) -> Self {
        let mut add = *self.row.first().unwrap();
        let mut sub = add;
        Self {
            m: &self.m,
            row: self
                .row
                .iter()
                .enumerate()
                .map(|(i, x)| {
                    sub = if i != 0 && self.m.chars[i - 1] == value {
                        sub
                    } else {
                        sub + 1
                    };
                    add = sub.min(add + 1).min(x + 1);
                    sub = *x;
                    add
                })
                .collect(),
        }
    }

    pub fn distance(&self) -> usize {
        *self.row.last().unwrap()
    }

    pub fn can_match(&self, max_edits: usize) -> bool {
        self.row.iter().min().unwrap() <= &max_edits
    }
}
