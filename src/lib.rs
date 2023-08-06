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
fn levenshtein_extract(query: &str, choices: Vec<&str>) -> Option<(String, u32, usize)> {
    let mut best = None;
    for (i, x) in choices.iter().enumerate() {
        let distance = levenshtein::levenshtein(query, x);
        best = Some(best.unwrap_or((distance, i, x)).min((distance, i, x)));
        if distance == 0 {
            break;
        }
    }
    best.map(|x| (x.2.to_string(), x.0, x.1))
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
