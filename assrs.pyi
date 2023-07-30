from typing import List, Optional, Tuple, final

@final
class Trie:
    def __init__(self, items: Optional[List[str]] = ...) -> None: ...
    @staticmethod
    def new() -> "Trie": ...
    def insert(self, value: str) -> None: ...
    def get(self, value: str) -> Optional[str]: ...
    def contains(self, value: str) -> bool: ...
    def values(self) -> List[str]: ...
    def find_one(
        self, query: str, max_edits: Optional[int] = ...
    ) -> Optional[Tuple[str, int]]: ...

@final
class BKTree:
    def __init__(self, items: Optional[List[str]] = ...) -> None: ...
    @staticmethod
    def new() -> "BKTree": ...
    def insert(self, value: str) -> None: ...
    def get(self, value: str) -> Optional[str]: ...
    def contains(self, value: str) -> bool: ...
    def values(self) -> List[str]: ...
    def find_one(
        self, query: str, max_edits: Optional[int] = ...
    ) -> Optional[Tuple[str, int]]: ...

def levenshtein(a: str, b: str) -> int: ...
def levenshtein_extract(
    query: str, choices: List[str]
) -> Optional[Tuple[str, int, int]]: ...
