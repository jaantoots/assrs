use pyo3::prelude::*;

use crate::bktree::BKTreeLevenshtein;
use crate::trie::Trie;

mod bktree;
mod levenshtein;
mod trie;

#[pyfunction]
fn levenshtein_extract(a: &str, b: Vec<&str>) -> (usize, usize) {
    let mut best = usize::MAX;
    let mut idx = 0;
    let mut current;
    for (i, choice) in b.iter().enumerate() {
        current = levenshtein::levenshtein(a, choice);
        if current < best {
            best = current;
            idx = i;
        }
    }
    (best, idx)
}

/// A Python module implemented in Rust.
#[pymodule]
fn assrs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(levenshtein::levenshtein, m)?)?;
    m.add_function(wrap_pyfunction!(levenshtein_extract, m)?)?;
    m.add_class::<BKTreeLevenshtein>()?;
    m.add_class::<Trie>()?;
    Ok(())
}
