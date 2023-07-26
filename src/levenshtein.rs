use pyo3::prelude::*;

/// Find the Levenshtein distance between two strings
#[pyfunction]
pub fn levenshtein(a: &str, b: &str) -> usize {
    if a == b {
        return 0;
    }

    let len_a = a.chars().count();
    let len_b = b.chars().count();
    if len_a == 0 {
        return len_b;
    }
    if len_b == 0 {
        return len_a;
    }

    // Row of the matrix of Levenshtein distances
    let mut row: Vec<usize> = (1..).take(len_b).collect();
    let mut add = 0;
    let mut sub;
    let mut del;

    for (i, char_a) in a.chars().enumerate() {
        add = i;
        sub = i;
        for (j, char_b) in b.chars().enumerate() {
            del = row[j];
            sub = if char_a == char_b { sub } else { sub + 1 };
            add = sub.min(add + 1).min(del + 1);
            sub = del;
            row[j] = add;
        }
    }
    add
}
