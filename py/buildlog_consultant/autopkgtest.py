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

import logging

from . import Problem, _buildlog_consultant_rs

logger = logging.getLogger(__name__)


class AutopkgtestDepsUnsatisfiable(Problem, kind="badpkg"):
    args: list[str]

    @classmethod
    def from_blame_line(cls, line):
        args = []
        entries = line[len("blame: ") :].rstrip("\n").split(" ")
        for entry in entries:
            try:
                (kind, arg) = entry.split(":", 1)
            except ValueError:
                kind = None
                arg = entry
            args.append((kind, arg))
            if kind not in ("deb", "arg", "dsc", None):
                logger.warn("unknown entry %s on badpkg line", entry)
        return cls(args)


class AutopkgtestTimedOut(Problem, kind="timed-out"):
    def __str__(self) -> str:
        return "Timed out"


class XDGRunTimeNotSet(Problem, kind="xdg-runtime-dir-not-set"):
    def __str__(self) -> str:
        return "XDG_RUNTIME_DIR is not set"


class AutopkgtestTestbedFailure(Problem, kind="testbed-failure"):
    reason: str

    def __eq__(self, other):
        return type(self) is type(other) and self.reason == other.reason

    def __repr__(self) -> str:
        return f"{type(self).__name__}({self.reason!r})"

    def __str__(self) -> str:
        return self.reason


class AutopkgtestDepChrootDisappeared(Problem, kind="testbed-chroot-disappeared"):
    def __str__(self) -> str:
        return "chroot disappeared"

    def __repr__(self) -> str:
        return f"{type(self).__name__}()"

    def __eq__(self, other):
        return isinstance(self, type(other))


class AutopkgtestErroneousPackage(Problem, kind="erroneous-package"):
    reason: str

    def __eq__(self, other):
        return type(self) is type(other) and self.reason == other.reason

    def __repr__(self) -> str:
        return f"{type(self).__name__}({self.reason!r})"

    def __str__(self) -> str:
        return self.reason


class AutopkgtestStderrFailure(Problem, kind="stderr-output"):
    stderr_line: str

    def __eq__(self, other):
        return isinstance(self, type(other)) and self.stderr_line == other.stderr_line

    def __repr__(self) -> str:
        return f"{type(self).__name__}({self.stderr_line!r})"

    def __str__(self) -> str:
        return f"output on stderr: {self.stderr_line}"


class AutopkgtestTestbedSetupFailure(Problem, kind="testbed-setup-failure"):
    command: str
    exit_status: int
    error: str

    def __str__(self) -> str:
        return "Error setting up testbed %r failed (%d): %s" % (
            self.command,
            self.exit_status,
            self.error,
        )


find_autopkgtest_failure_description = _buildlog_consultant_rs.find_autopkgtest_failure_description
