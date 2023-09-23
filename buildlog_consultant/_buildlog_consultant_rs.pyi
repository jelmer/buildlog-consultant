from typing import Iterator, BinaryIO

class  SbuildLog:

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
