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
import re
from typing import Tuple, Optional, List, Dict, Union

from . import Problem
from .apt import find_apt_get_failure, AptFetchFailure
from .common import find_build_failure_description, ChrootNotFound


logger = logging.getLogger(__name__)


class AutopkgtestDepsUnsatisfiable(Problem):

    kind = "badpkg"

    def __init__(self, args):
        self.args = args

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

    def __eq__(self, other):
        return type(self) == type(other) and self.args == other.args

    def __repr__(self):
        return "%s(args=%r)" % (type(self).__name__, self.args)


class AutopkgtestTimedOut(Problem):

    kind = "timed-out"

    def __init__(self):
        pass

    def __str__(self):
        return "Timed out"

    def __repr__(self):
        return "%s()" % (type(self).__name__)

    def __eq__(self, other):
        return isinstance(self, type(other))


class AutopkgtestTestbedFailure(Problem):

    kind = "testbed-failure"

    def __init__(self, reason):
        self.reason = reason

    def __eq__(self, other):
        return type(self) == type(other) and self.reason == other.reason

    def __repr__(self):
        return "%s(%r)" % (type(self).__name__, self.reason)

    def __str__(self):
        return self.reason


class AutopkgtestDepChrootDisappeared(Problem):

    kind = "testbed-chroot-disappeared"

    def __init__(self):
        pass

    def __str__(self):
        return "chroot disappeared"

    def __repr__(self):
        return "%s()" % (type(self).__name__)

    def __eq__(self, other):
        return isinstance(self, type(other))


class AutopkgtestErroneousPackage(Problem):

    kind = "erroneous-package"

    def __init__(self, reason):
        self.reason = reason

    def __eq__(self, other):
        return type(self) == type(other) and self.reason == other.reason

    def __repr__(self):
        return "%s(%r)" % (type(self).__name__, self.reason)

    def __str__(self):
        return self.reason


class AutopkgtestStderrFailure(Problem):

    kind = "stderr-output"

    def __init__(self, stderr_line):
        self.stderr_line = stderr_line

    def __eq__(self, other):
        return isinstance(self, type(other)) and self.stderr_line == other.stderr_line

    def __repr__(self):
        return "%s(%r)" % (type(self).__name__, self.stderr_line)

    def __str__(self):
        return "output on stderr: %s" % self.stderr_line


def parse_autopgktest_line(line: str) -> Union[str, Tuple[str, Union[Tuple[str, ...]]]]:
    m = re.match(r"autopkgtest \[([0-9:]+)\]: (.*)", line)
    if not m:
        return line
    timestamp = m.group(1)
    message = m.group(2)
    if message.startswith("@@@@@@@@@@@@@@@@@@@@ source "):
        return (timestamp, ("source",))
    elif message.startswith("@@@@@@@@@@@@@@@@@@@@ summary"):
        return (timestamp, ("summary",))
    elif message.startswith("test "):
        (testname, test_status) = message[len("test ") :].rstrip("\n").split(": ", 1)
        if test_status == "[-----------------------":
            return (
                timestamp,
                (
                    "test",
                    testname,
                    "begin output",
                ),
            )
        elif test_status == "-----------------------]":
            return (
                timestamp,
                (
                    "test",
                    testname,
                    "end output",
                ),
            )
        elif test_status == (" - - - - - - - - - - results - - - - - - - - - -"):
            return (
                timestamp,
                (
                    "test",
                    testname,
                    "results",
                ),
            )
        elif test_status == (" - - - - - - - - - - stderr - - - - - - - - - -"):
            return (
                timestamp,
                (
                    "test",
                    testname,
                    "stderr",
                ),
            )
        elif test_status == "preparing testbed":
            return (timestamp, ("test", testname, "prepare testbed"))
        else:
            return (timestamp, ("test", testname, test_status))
    elif message.startswith("ERROR:"):
        return (timestamp, ("error", message[len("ERROR: ") :]))
    else:
        return (timestamp, (message,))


def parse_autopkgtest_summary(lines):
    i = 0
    while i < len(lines):
        line = lines[i]
        m = re.match("([^ ]+)(?:[ ]+)PASS", line)
        if m:
            yield i, m.group(1), "PASS", None, []
            i += 1
            continue
        m = re.match("([^ ]+)(?:[ ]+)(FAIL|PASS|SKIP) (.+)", line)
        if not m:
            i += 1
            continue
        testname = m.group(1)
        result = m.group(2)
        reason = m.group(3)
        offset = i
        extra = []
        if reason == "badpkg":
            while i + 1 < len(lines) and (
                lines[i + 1].startswith("badpkg:") or lines[i + 1].startswith("blame:")
            ):
                extra.append(lines[i + 1])
                i += 1
        yield offset, testname, result, reason, extra
        i += 1


class AutopkgtestTestbedSetupFailure(Problem):

    kind = "testbed-setup-failure"

    def __init__(self, command, exit_status, error):
        self.command = command
        self.exit_status = exit_status
        self.error = error

    def __str__(self):
        return "Error setting up testbed %r failed (%d): %s" % (
            self.command,
            self.exit_status,
            self.error,
        )

    def __repr__(self):
        return "%s(%r, %r, %r)" % (
            type(self).__name__,
            self.command,
            self.exit_status,
            self.error,
        )

    def __eq__(self, other):
        return (
            isinstance(other, type(self))
            and self.command == other.command
            and self.exit_status == other.exit_status
            and self.error == other.error
        )


def find_autopkgtest_failure_description(  # noqa: C901
    lines: List[str],
) -> Tuple[Optional[int], Optional[str], Optional["Problem"], Optional[str]]:
    """Find the autopkgtest failure in output.

    Returns:
      tuple with (line offset, testname, error, description)
    """
    error: Optional["Problem"]
    test_output: Dict[Tuple[str, ...], List[str]] = {}
    test_output_offset: Dict[Tuple[str, ...], int] = {}
    current_field: Optional[Tuple[str, ...]] = None
    i = -1
    while i < len(lines) - 1:
        i += 1
        line = lines[i]
        parsed = parse_autopgktest_line(line)
        if isinstance(parsed, tuple):
            (timestamp, content) = parsed
            if content[0] == "test":
                if content[2] == "end output":
                    current_field = None
                    continue
                elif content[2] == "begin output":
                    current_field = (content[1], "output")
                else:
                    current_field = (content[1], content[2])
                if current_field in test_output:
                    logger.warn("duplicate output fields for %r", current_field)
                test_output[current_field] = []
                test_output_offset[current_field] = i + 1
            elif content == ("summary",):
                current_field = ("summary",)
                test_output[current_field] = []
                test_output_offset[current_field] = i + 1
            elif content[0] == "error":
                if content[1].startswith('"') and content[1].count('"') == 1:
                    sublines = [content[1]]
                    while i < len(lines):
                        i += 1
                        sublines += lines[i]
                        if lines[i].count('"') == 1:
                            break
                    content = (content[0], "".join(sublines))
                last_test: Optional[str]
                if current_field is not None:
                    last_test = current_field[0]
                else:
                    last_test = None
                msg = content[1]
                m = re.fullmatch('"(.*)" failed with stderr "(.*)("?)', msg)
                if m:
                    stderr = m.group(2)
                    m = re.fullmatch(
                        "W: (.*): " "Failed to stat file: No such file or directory",
                        stderr,
                    )
                    if m:
                        error = AutopkgtestDepChrootDisappeared()
                        return (i + 1, last_test, error, stderr)
                m = re.fullmatch(r"testbed failure: (.*)", msg)
                if m:
                    testbed_failure_reason = m.group(1)
                    if (
                        current_field is not None
                        and testbed_failure_reason
                        == "testbed auxverb failed with exit code 255"
                    ):
                        field = (current_field[0], "output")
                        (match, error) = find_build_failure_description(
                            test_output[field]
                        )
                        if error is not None:
                            assert match is not None
                            return (
                                test_output_offset[field] + match.lineno,
                                last_test,
                                error,
                                match.line,
                            )

                    if (
                        testbed_failure_reason
                        == "sent `auxverb_debug_fail', got `copy-failed', "
                        "expected `ok...'"
                    ):
                        (match, error) = find_build_failure_description(
                            lines
                        )
                        if error is not None:
                            assert match is not None
                            return (match.lineno, last_test, error, match.line)

                    if (
                        testbed_failure_reason
                        == "cannot send to testbed: [Errno 32] Broken pipe"
                    ):
                        offset, line, error = find_testbed_setup_failure(lines)
                        if error and offset:
                            return (offset, last_test, error, line)
                    if (
                        testbed_failure_reason
                        == "apt repeatedly failed to download packages"
                    ):
                        match, error = find_apt_get_failure(lines)
                        if error and match:
                            return (match.lineno, last_test, error, match.line)
                        return (
                            i + 1,
                            last_test,
                            AptFetchFailure(None, testbed_failure_reason),
                            None,
                        )
                    return (
                        i + 1,
                        last_test,
                        AutopkgtestTestbedFailure(testbed_failure_reason),
                        None,
                    )
                m = re.fullmatch(r"erroneous package: (.*)", msg)
                if m:
                    (match, error) = find_build_failure_description(
                        lines[:i]
                    )
                    if error and match:
                        return (match.lineno, last_test, error, match.line)
                    return (
                        i + 1,
                        last_test,
                        AutopkgtestErroneousPackage(m.group(1)),
                        None,
                    )
                if current_field is not None:
                    match, error = find_apt_get_failure(
                        test_output[current_field]
                    )
                    if (
                        error is not None
                        and match is not None
                        and current_field in test_output_offset
                    ):
                        return (
                            test_output_offset[current_field] + match.lineno,
                            last_test,
                            error,
                            match.line,
                        )
                if msg == 'autopkgtest':
                    if lines[i+1].rstrip() == ': error cleaning up:':
                        return (
                            test_output_offset[current_field] + 1,
                            last_test,
                            AutopkgtestTimedOut(),
                            lines[i-1].rstrip())
                return (i + 1, last_test, None, msg)
        else:
            if current_field:
                test_output[current_field].append(line)

    try:
        summary_lines = test_output[("summary",)]
        summary_offset = test_output_offset[("summary",)]
    except KeyError:
        while lines and not lines[-1].strip():
            lines.pop(-1)
        if not lines:
            return (None, None, None, None)
        else:
            return (len(lines), lines[-1], None, None)
    else:
        for (lineno, testname, result, reason, extra) in parse_autopkgtest_summary(
            summary_lines
        ):
            if result in ("PASS", "SKIP"):
                continue
            assert result == "FAIL"
            if reason == "timed out":
                error = AutopkgtestTimedOut()
                return (summary_offset + lineno + 1, testname, error, reason)
            elif reason.startswith("stderr: "):
                output = reason[len("stderr: ") :]
                stderr_lines = test_output.get((testname, "stderr"), [])
                stderr_offset = test_output_offset.get((testname, "stderr"))
                description: Optional[str]
                if stderr_lines:
                    (match, error) = find_build_failure_description(
                        stderr_lines
                    )
                    if match is not None and stderr_offset is not None:
                        offset = match.lineno + stderr_offset - 1
                        description = match.line
                    else:
                        offset = None
                        description = None
                else:
                    (match, error) = find_build_failure_description([output])
                    offset = None
                    if match is not None:
                        description = match.line
                    else:
                        description = None
                if offset is None:
                    offset = summary_offset + lineno
                if error is None:
                    error = AutopkgtestStderrFailure(output)
                    if description is None:
                        description = (
                            "Test %s failed due to "
                            "unauthorized stderr output: %s"
                            % (testname, error.stderr_line)
                        )
                return offset + 1, testname, error, description
            elif reason == "badpkg":
                output_lines = test_output.get((testname, "prepare testbed"), [])
                output_offset = test_output_offset.get((testname, "prepare testbed"))
                if output_lines and output_offset:
                    match, error = find_apt_get_failure(output_lines)
                    if error and match:
                        return (match.offset + output_offset + 1, testname, error, None)
                badpkg = None
                blame = None
                for line in extra:
                    if line.startswith("badpkg: "):
                        badpkg = line[len("badpkg: ") :]
                    if line.startswith("blame: "):
                        blame = line
                if badpkg is not None:
                    description = "Test %s failed: %s" % (testname, badpkg.rstrip("\n"))
                else:
                    description = "Test %s failed" % testname

                error = AutopkgtestDepsUnsatisfiable.from_blame_line(blame)
                return (summary_offset + lineno + 1, testname, error, description)
            else:
                output_lines = test_output.get((testname, "output"), [])
                output_offset = test_output_offset.get((testname, "output"))
                (match, error) = find_build_failure_description(
                    output_lines
                )
                if match is None or output_offset is None:
                    offset = summary_offset + lineno
                else:
                    offset = match.lineno + output_offset
                if match is None:
                    description = "Test %s failed: %s" % (testname, reason)
                else:
                    description = match.line
                return offset + 1, testname, error, description  # type: ignore

    return None, None, None, None


def find_testbed_setup_failure(lines):
    for i in range(len(lines) - 1, 0, -1):
        line = lines[i]
        m = re.fullmatch(
            r"\[(.*)\] failed \(exit status ([0-9]+), stderr \'(.*)\'\)\n", line
        )
        if m:
            command = m.group(1)
            status_code = int(m.group(2))
            stderr = m.group(3)
            m = re.fullmatch(r"E: (.*): Chroot not found\\n", stderr)
            if m:
                return (i + 1, line, ChrootNotFound(m.group(1)))
            return (
                i + 1,
                line,
                AutopkgtestTestbedSetupFailure(command, status_code, stderr),
            )
        m = re.fullmatch(
            r'\<VirtSubproc\>: failure: \[\'(.*)\'\] '
            r'unexpectedly produced stderr output `(.*)', line)
        if m:
            command = m.group(1)
            stderr_output = m.group(2)
            n = re.match(
                r'W: /var/lib/schroot/session/(.*): '
                'Failed to stat file: No such file or directory',
                stderr_output)
            if n:
                return (i + 1, line, AutopkgtestDepChrootDisappeared())
            return (
                i + 1,
                line,
                AutopkgtestTestbedSetupFailure(command, 1, stderr_output))
    return None, None, None
