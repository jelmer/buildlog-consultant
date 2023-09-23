from typing import Iterator


class SbuildLogSection:
    title: str | None
    offset: tuple[int, int]
    lines: list[str]


def parse_sbuild_log(lines: list[bytes]) -> Iterator[SbuildLogSection]: ...
