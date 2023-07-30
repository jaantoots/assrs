use pyo3::prelude::*;

use crate::bktree::BKTree;
use crate::trie::Trie;

mod bktree;
mod levenshtein;
mod trie;

/// Find the best match in a list of choices
///
/// Returns (choice, distance, index) or None (for empty choices)
#[pyfunction]
fn levenshtein_extract(query: &str, choices: Vec<&str>) -> Option<(String, usize, usize)> {
    choices
        .iter()
        .map(|x| (x, levenshtein::levenshtein(query, x)))
        .enumerate()
        .min_by_key(|(_i, (_x, d))| *d)
        .map(|(i, (x, d))| (x.to_string(), d, i))
}

/// Approximate string searching
#[pymodule]
fn assrs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(levenshtein::levenshtein, m)?)?;
    m.add_function(wrap_pyfunction!(levenshtein_extract, m)?)?;
    m.add_class::<BKTree>()?;
    m.add_class::<Trie>()?;
    Ok(())
}
