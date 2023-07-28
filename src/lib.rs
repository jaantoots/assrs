use pyo3::prelude::*;

use crate::bktree::BKTreeLevenshtein;
use crate::trie::Trie;

mod bktree;
mod levenshtein;
mod trie;

#[pyfunction]
fn levenshtein_extract(a: &str, b: Vec<&str>) -> Option<(usize, usize)> {
    b.iter()
        .map(|x| levenshtein::levenshtein(a, x))
        .enumerate()
        .min_by_key(|(_i, x)| *x)
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
