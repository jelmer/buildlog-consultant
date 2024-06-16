from collections.abc import Iterator
from typing import BinaryIO

class SbuildLog:
    def get_section_lines(self, section: str | None) -> list[str] | None: ...
    def get_section(self, section: str | None) -> SbuildLogSection | None: ...
    def section_titles(self) -> list[str]: ...
    @staticmethod
    def parse(f: BinaryIO) -> SbuildLog: ...
    def get_failed_stage(self) -> str | None: ...

    sections: list[SbuildLogSection]

class SbuildLogSection:
    title: str | None
    offsets: tuple[int, int]
    lines: list[str]

def parse_sbuild_log(lines: list[bytes]) -> Iterator[SbuildLogSection]: ...

class Match:
    line: str

    offset: int

    origin: str

    lineno: int

class Problem:
    kind: str

    def json(self): ...

def match_lines(
    lines: list[str], lineno: int
) -> tuple[Match | None, Problem | None]: ...
def find_secondary_build_failure(lines: list[str], lineno: int) -> Match | None: ...
