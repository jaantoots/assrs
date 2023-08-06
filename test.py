# pylint: disable=invalid-name,missing-docstring

import random
import string
import time
import timeit

import rapidfuzz

import assrs

NUMBER = 1


def timer(func, samples, *args, number=NUMBER, repeat=3, name=None, **kwargs):
    if name is None:
        name = repr(func)

    def wrapped():
        for x in samples:
            func(x, *args, **kwargs)

    times = timeit.repeat(wrapped, number=number, repeat=repeat)
    print(f"{name:8s} {min(times)/len(samples)*1000:.2f}ms")


def run(choices, samples):
    print(len(choices), len(samples))

    tr = assrs.Trie(choices)
    timer(tr.find_one, samples)
    timer(tr.find_one, samples, 3)

    t = assrs.BKTree(choices)
    timer(t.find_one, samples)
    timer(t.find_one, samples, 3)

    timer(assrs.levenshtein_extract, samples, choices, number=1)
    timer(
        lambda x: min(choices, key=lambda s: assrs.levenshtein(x, s)),
        samples,
        number=1,
        name="assrs.levenshtein",
    )
    timer(
        rapidfuzz.process.extractOne,
        samples,
        choices,
        scorer=rapidfuzz.distance.Levenshtein.distance,
    )
    timer(
        lambda x: min(
            choices, key=lambda s: rapidfuzz.distance.Levenshtein.distance(x, s)
        ),
        samples,
        number=1,
        name="rapidfuzz.distance.Levenshtein",
    )

    print()


def random_words(n, length=10):
    return [
        "".join(
            random.choice(string.ascii_letters + string.digits) for _ in range(length)
        )
        for _ in range(n)
    ]


def main():
    with open("/usr/share/dict/words", encoding="utf-8") as f:
        words = [l.strip() for l in f]

    time.sleep(1)
    run(random_words(100_000), random_words(100))
    run(words, words[::1000])


if __name__ == "__main__":
    main()
