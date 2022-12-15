#!/usr/bin/python
# Copyright (C) 2019-2021 Jelmer Vernooij <jelmer@jelmer.uk>
# encoding: utf-8
#
# This program is free software; you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation; either version 2 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program; if not, write to the Free Software
# Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA

from typing import List, Dict, Type

__version__ = (0, 0, 31)
version_string = '.'.join(map(str, __version__))


problem_clses: Dict[str, Type["Problem"]] = {}


class Problem:

    kind: str
    is_global: bool = False

    def __init_subclass__(cls, kind: str, is_global: bool = False, **kwargs):
        super().__init_subclass__(**kwargs)
        cls.kind = kind
        cls.is_global = is_global
        if kind in problem_clses:
            raise AssertionError('class %r already registered for kind %s (not %r)' % (
                problem_clses[kind], kind, cls))
        problem_clses[kind] = cls

    def __init__(self, *args, **kwargs):
        for name, arg in list(zip(
                list(type(self).__annotations__.keys()),
                list(args))) + list(kwargs.items()):
            setattr(self, name, arg)

    def json(self):
        ret = {}
        for key in type(self).__annotations__.keys():
            ret[key] = getattr(self, key)
        return ret

    @classmethod
    def from_json(cls, data):
        return cls(**data)

    def __eq__(self, other):
        if not isinstance(self, type(other)):
            return False
        if self.kind != other.kind:
            return False
        for name in type(self).__annotations__.keys():
            if getattr(self, name) != getattr(other, name):
                return False
        return True


class Match:

    line: str
    lines: List[str]
    lineno: int
    linenos: List[int]
    offset: int
    offsets: List[int]


class SingleLineMatch(Match):

    offset: int
    line: str

    def __init__(self, offset: int, line: str):
        self.offset = offset
        self.line = line

    def __repr__(self):
        return "%s(%r, %r)" % (type(self).__name__, self.offset, self.line)

    def __eq__(self, other):
        return (
            isinstance(self, type(other))
            and self.offset == other.offset
            and self.line == other.line
        )

    @property
    def lines(self) -> List[str]:  # type: ignore
        return [self.line]

    @property
    def linenos(self) -> List[int]:  # type: ignore
        return [self.lineno]

    @property
    def offsets(self) -> List[int]:  # type: ignore
        return [self.offset]

    @property
    def lineno(self) -> int:  # type: ignore
        return self.offset + 1

    @classmethod
    def from_lines(cls, lines, offset):
        return cls(offset, lines[offset])


class MultiLineMatch(Match):

    offsets: List[int]
    lines: List[str]

    def __init__(self, offsets: List[int], lines: List[str]):
        self.offsets = offsets
        self.lines = lines

    def __repr__(self):
        return "%s(%r, %r)" % (type(self).__name__, self.offsets, self.lines)

    def __eq__(self, other):
        return (
            isinstance(self, type(other))
            and self.offsets == other.offsets
            and self.lines == other.lines
        )

    @property
    def line(self):
        return self.lines[-1]

    @property
    def offset(self):
        return self.offsets[-1]

    @property
    def lineno(self):
        return self.offset + 1

    @property
    def linenos(self):
        return [o + 1 for o in self.offsets]

    @classmethod
    def from_lines(cls, lines, offsets):
        return cls(offsets, [lines[o] for o in offsets])
