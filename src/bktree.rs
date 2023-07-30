use pyo3::prelude::*;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{HashMap, VecDeque};
use std::iter::once;

use crate::levenshtein;

struct Tree {
    value: String,
    children: HashMap<usize, Tree>,
}

impl Tree {
    fn new(value: String) -> Self {
        Self {
            value,
            children: HashMap::new(),
        }
    }

    fn insert(&mut self, value: String) {
        let distance = levenshtein::levenshtein(&value, &self.value);
        if distance == 0 {
            return;
        }
        match self.children.entry(distance) {
            Occupied(mut entry) => entry.get_mut().insert(value),
            Vacant(entry) => {
                entry.insert(Self::new(value));
            }
        };
    }
}

/// BK-tree storing the strings to search against
#[pyclass]
pub struct BKTree {
    tree: Option<Tree>,
}

#[pymethods]
impl BKTree {
    #[new]
    pub fn py_new(items: Option<Vec<String>>) -> Self {
        items.map_or_else(Self::new, Self::from_iter)
    }

    #[staticmethod]
    pub fn new() -> Self {
        Self { tree: None }
    }

    pub fn insert(&mut self, value: String) {
        match self.tree.as_mut() {
            Some(t) => t.insert(value),
            None => {
                self.tree = Some(Tree::new(value));
            }
        }
    }

    pub fn get(&self, value: &str) -> Option<&str> {
        let mut node = self.tree.as_ref()?;
        loop {
            let distance = levenshtein::levenshtein(value, &node.value);
            if distance == 0 {
                break;
            }
            node = node.children.get(&distance)?;
        }
        Some(&node.value)
    }

    pub fn contains(&self, value: &str) -> bool {
        self.get(value).is_some()
    }

    pub fn values(&self) -> Vec<&str> {
        self.iter().collect()
    }

    /// Find best match in BK-tree for query
    pub fn find_one(&self, query: &str, max_edits: Option<usize>) -> Option<(&str, usize)> {
        let tree = self.tree.as_ref()?;
        let mut candidates = VecDeque::new();
        candidates.push_back(tree);

        let mut best = None;
        let mut max_edits = max_edits.unwrap_or(usize::MAX);

        while let Some(node) = candidates.pop_front() {
            let distance = levenshtein::levenshtein(query, &node.value);
            if distance <= max_edits {
                max_edits = distance;
                best = Some((node.value.as_str(), distance));
            }
            if !node.children.is_empty() {
                let lower = distance - max_edits;
                let upper = distance + max_edits;
                candidates.extend(node.children.iter().filter_map(|(d, c)| {
                    if lower < *d && *d < upper {
                        Some(c)
                    } else {
                        None
                    }
                }));
            }
        }
        best
    }
}

impl Default for BKTree {
    fn default() -> Self {
        Self::new()
    }
}

impl Extend<String> for BKTree {
    fn extend<I: IntoIterator<Item = String>>(&mut self, iter: I) {
        for item in iter {
            self.insert(item);
        }
    }
}

impl FromIterator<String> for BKTree {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        let mut tree = Self::new();
        tree.extend(iter);
        tree
    }
}

impl<'a> IntoIterator for &'a Tree {
    type Item = &'a str;
    type IntoIter = Box<dyn Iterator<Item = &'a str> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Tree {
    pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a str> + 'a> {
        Box::new(once(self.value.as_str()).chain(self.children.values().flat_map(|x| x.iter())))
    }
}

impl<'a> IntoIterator for &'a BKTree {
    type Item = &'a str;
    type IntoIter = Box<dyn Iterator<Item = &'a str> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl BKTree {
    pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a str> + 'a> {
        Box::new(self.tree.iter().flatten())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn empty_str() {
        let mut tree = BKTree::new();
        assert!(!tree.contains(""));
        assert_eq!(tree.iter().count(), 0);
        tree.insert("".to_string());
        assert!(tree.contains(""));
        assert_eq!(tree.iter().collect::<Vec<_>>(), vec![""]);
    }

    #[test]
    fn values() {
        let mut tree = BKTree::new();
        assert!(!tree.contains(""));

        tree.insert("foo".to_string());
        tree.insert("bar".to_string());
        tree.insert("baz".to_string());
        assert!(!tree.contains(""));
        assert!(tree.contains("foo"));
        assert_eq!(
            tree.iter().collect::<HashSet<_>>(),
            HashSet::from(["foo", "bar", "baz"])
        );

        tree.insert("".to_string());
        assert!(tree.contains(""));
        assert_eq!(
            tree.iter().collect::<HashSet<_>>(),
            HashSet::from(["foo", "bar", "baz", ""])
        );
    }

    #[test]
    fn find() {
        let tree = BKTree::from_iter(vec!["foo".to_string(), "bar".to_string()]);
        assert_eq!(tree.find_one("", Some(2)), None);
        assert_eq!(tree.find_one("baz", Some(2)), Some(("bar", 1)));
        assert_eq!(tree.find_one("baz", None), Some(("bar", 1)));
        assert_eq!(tree.find_one("baz", Some(0)), None);
    }
}
