# assrs

Approximate String SearcheRS is a Python library written in Rust for querying
an index of strings for the closest match. It implements a Levenshtein Automaton
for quickly searching a trie.

## Usage

### Installation

```
pip install assrs
```

### Quickstart

```python
from assrs import BKTree, Trie, levenshtein

trie = Trie(["foo", "bar"])
trie.find_one("baz")
# ("bar", 1)
trie.find_one("abc", max_edits=1)
# None

tree = BKTree(["foo", "bar"])
tree.find_one("baz")
# ("bar", 1)
tree.find_one("abc", max_edits=1)
# None

levenshtein("kitten", "sitting")
# 3
```

## Discussion

The main problem can be formulated as finding the best match between a query
string and a reasonably large index (e.g. the entire English dictionary, as
below). The similarity between a pair of strings is described by the
[Levenshtein distance](https://en.wikipedia.org/wiki/Levenshtein_distance).

The naive solution is calculating the distance between the query and all the
choices and taking the minimum. This works comparatively well for a small
number of choices and a fast Levenshtein distance implementation like
`rapidfuzz`. However, if the index is large and relatively static (many queries
and few or no changes to the choices) then the necessary computation can be
significantly reduced.

One solution is constructing a [BK-tree](https://en.wikipedia.org/wiki/BK-tree)
for the set of choices. The triangle inequality helps reduce the necessary
distance calculations by limiting the search. In practice, the performance
seems to heavily depend on the insertion order, and is significantly helped by
having a low `max_edits` value.

A better option is using a [trie](https://en.wikipedia.org/wiki/Trie) as an
index; the best match can be found by traversing it with a Levenshtein
Automaton that allows us to exclude subtries which cannot contain a
sufficiently good match. This should allow us to remove unnecessary distance
calculations much more effectively, and can also take advantage of setting
`max_edits`.

The above is combined with a reasonably performant implementation of
Levenshtein distance, using the bitvector algorithm by [Myers][1] for strings
up to 64 characters long.

### Performance

Using [rapidfuzz][4] as a reference.

Taking a dictionary (235,976 words) as index and every 1000th as a query:
- `Trie.find_one`: 2.28ms
- `Trie.find_one(..., max_edits=3)`: 1.31ms
- `BKTree.find_one`: 11.61ms
- `BKTree.find_one(..., max_edits=3)`: 3.70ms
- `levenshtein_extract`: 24.06ms
- `levenshtein` in a Python loop: 36.94ms
- `rapidfuzz.process.extractOne(..., scorer=rapidfuzz.distance.Levenshtein.distance)`: 4.08ms
- `rapidfuzz.distance.Levenshtein.distance` in a Python loop: 43.75ms

However, with 100,000 random strings of length 10 as index and querying random strings:
- `Trie.find_one`: 36.44ms
- `Trie.find_one(..., max_edits=3)`: 15.84ms
- `BKTree.find_one`: 15.36ms
- `BKTree.find_one(..., max_edits=3)`: 13.79ms
- `levenshtein_extract`: 10.89ms
- `levenshtein` in a Python loop: 16.45ms
- `rapidfuzz.process.extractOne(..., scorer=rapidfuzz.distance.Levenshtein.distance)`: 4.29ms
- `rapidfuzz.distance.Levenshtein.distance` in a Python loop: 17.67ms



### Limitations

Currently missing features and known issues:
- poor worst case performance for lookups,
- not using bitvectors for strings longer than 64 characters,
- support for segmenting over grapheme clusters rather than codepoints,
- support for other distance functions,
- standalone Rust crate.

## Resources

- [rapidfuzz][4] as the inspiration and comparison reference
- A [blog post][2] discussing Levenshtein automata and tries
- Myers, G. (1999). A fast bit-vector algorithm for approximate string matching
  based on dynamic programming. Journal of the ACM (JACM), 46(3), 395-415.
  [1][1]
- Hyyrö, H. (2003). A bit-vector algorithm for computing Levenshtein and
  Damerau edit distances. Nord. J. Comput., 10(1), 29-39. [2][2]

[1]: https://dl.acm.org/doi/pdf/10.1145/316542.316550
[2]: https://www.academia.edu/download/39402556/psc02.pdf
[3]: https://julesjacobs.com/2015/06/17/disqus-levenshtein-simple-and-fast.html
[4]: https://github.com/maxbachmann/RapidFuzz
