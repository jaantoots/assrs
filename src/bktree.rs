use pyo3::prelude::*;
use std::collections::{HashMap, VecDeque};

use crate::levenshtein;

struct StrTree(String, HashMap<usize, StrTree>);

#[pyclass]
pub struct BKTreeLevenshtein {
    tree: Option<StrTree>,
}

#[pymethods]
impl BKTreeLevenshtein {
    #[new]
    pub fn new(items: Option<Vec<String>>) -> Self {
        let mut bktree = BKTreeLevenshtein { tree: None };
        if let Some(items) = items {
            for item in items {
                bktree.add(&item);
            }
        }
        bktree
    }

    pub fn add(&mut self, item: &str) {
        let mut node = self
            .tree
            .get_or_insert_with(|| StrTree(item.to_string(), HashMap::new()));
        loop {
            let parent = &node.0;
            let children = &mut node.1;
            let distance = levenshtein::levenshtein(&item, &parent);
            if distance == 0 {
                break;
            }
            node = children
                .entry(distance)
                .or_insert_with(|| StrTree(item.to_string(), HashMap::new()));
        }
    }

    pub fn find_one(&self, item: &str) -> Option<String> {
        let tree = self.tree.as_ref()?;
        let mut candidates = VecDeque::new();
        candidates.push_back(tree);

        let mut best: Option<&String> = None;
        let mut best_distance = usize::MAX;

        while let Some(StrTree(candidate, children)) = candidates.pop_front() {
            let distance = levenshtein::levenshtein(&candidate, &item);
            if distance <= best_distance {
                best_distance = distance;
                best = Some(candidate);
            }
            if !children.is_empty() {
                let lower = distance - best_distance;
                let upper = distance + best_distance;
                candidates.extend(children.iter().filter_map(|(d, c)| {
                    if lower < *d && *d < upper {
                        Some(c)
                    } else {
                        None
                    }
                }));
            }
        }

        best.cloned()
    }
}
