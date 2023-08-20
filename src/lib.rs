use pyo3::prelude::*;

use crate::bktree::BKTree;
use crate::trie::Trie;

mod bktree;
mod levenshtein;
mod trie;

/// Approximate string searching
#[pymodule]
fn assrs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(levenshtein::levenshtein, m)?)?;
    m.add_function(wrap_pyfunction!(levenshtein::levenshtein_extract, m)?)?;
    m.add_class::<BKTree>()?;
    m.add_class::<Trie>()?;
    Ok(())
}
