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

import re
from typing import List, Tuple, Iterator, BinaryIO, Optional, Union, Dict

import logging

from . import Problem
from .apt import (
    find_apt_get_update_failure,
    find_install_deps_failure_description,
)
from .autopkgtest import find_autopkgtest_failure_description
from .common import find_build_failure_description, NoSpaceOnDevice

__all__ = [
    "SbuildFailure",
    "parse_sbuild_log",
]

logger = logging.getLogger(__name__)


class SbuildFailure(Exception):
    """Sbuild failed to run."""

    def __init__(
        self,
        stage: Optional[str],
        description: Optional[str],
        error: Optional["Problem"] = None,
        phase: Optional[Union[Tuple[str], Tuple[str, Optional[str]]]] = None,
    ):
        self.stage = stage
        self.description = description
        self.error = error
        self.phase = phase

    def __repr__(self):
        return "%s(%r, %r, error=%r, phase=%r)" % (
            type(self).__name__,
            self.stage,
            self.description,
            self.error,
            self.phase,
        )


SBUILD_FOCUS_SECTION: Dict[Optional[str], str] = {
    "build": "build",
    "run-post-build-commands": "post build commands",
    "post-build": "post build",
    "install-deps": "install package build dependencies",
    "explain-bd-uninstallable": "install package build dependencies",
    "apt-get-update": "update chroot",
    "arch-check": "check architectures",
    "check-space": "cleanup",
}


class DpkgSourceLocalChanges(Problem):

    kind = "unexpected-local-upstream-changes"

    def __init__(self, files=None):
        self.files = files

    def __eq__(self, other):
        return isinstance(other, type(self)) and self.files == other.files

    def __repr__(self):
        return "%s(%r)" % (type(self).__name__, self.files)

    def __str__(self):
        if self.files:
            return "Tree has local changes: %r" % self.files
        else:
            return "Tree has local changes"


class DpkgSourceUnrepresentableChanges(Problem):

    kind = "unrepresentable-local-changes"

    def __str__(self):
        return "Tree has unrepresentable local changes."


class DpkgUnwantedBinaryFiles(Problem):

    kind = "unwanted-binary-files"

    def __str__(self):
        return "Tree has unwanted binary files."


class DpkgBinaryFileChanged(Problem):

    kind = "changed-binary-files"

    def __init__(self, paths):
        self.paths = paths

    def __repr__(self):
        return "%s(%r)" % (type(self).__name__, self.paths)

    def __eq__(self, other):
        return isinstance(other, type(self)) and self.paths == other.paths

    def __str__(self):
        return "Tree has binary files with changes: %r" % self.paths


class MissingControlFile(Problem):

    kind = "missing-control-file"

    def __init__(self, path):
        self.path = path

    def __eq__(self, other):
        return isinstance(self, type(other)) and self.path == other.path

    def __repr__(self):
        return "%s(%r)" % (type(self).__name__, self.path)

    def __str__(self):
        return "Tree is missing control file %s" % self.path


class UnableToFindUpstreamTarball(Problem):

    kind = "unable-to-find-upstream-tarball"

    def __init__(self, package, version):
        self.package = package
        self.version = version

    def __str__(self):
        return "Unable to find the needed upstream tarball for " "%s, version %s." % (
            self.package,
            self.version,
        )


class PatchApplicationFailed(Problem):

    kind = "patch-application-failed"

    def __init__(self, patchname):
        self.patchname = patchname

    def __str__(self):
        return "Patch application failed: %s" % self.patchname


class SourceFormatUnbuildable(Problem):

    kind = "source-format-unbuildable"

    def __init__(self, source_format):
        self.source_format = source_format

    def __str__(self):
        return "Source format %s unbuildable" % self.source_format


class SourceFormatUnsupported(Problem):

    kind = "unsupported-source-format"

    def __init__(self, source_format):
        self.source_format = source_format

    def __str__(self):
        return "Source format %r unsupported" % self.source_format


class PatchFileMissing(Problem):

    kind = "patch-file-missing"

    def __init__(self, path):
        self.path = path

    def __str__(self):
        return "Patch file %s missing" % self.path


class UnknownMercurialExtraFields(Problem):

    kind = "unknown-mercurial-extra-fields"

    def __init__(self, field):
        self.field = field

    def __str__(self):
        return "Unknown Mercurial extra fields: %s" % self.field


class UpstreamPGPSignatureVerificationFailed(Problem):

    kind = "upstream-pgp-signature-verification-failed"

    def __init__(self):
        pass

    def __str__(self):
        return "Unable to verify the PGP signature on the upstream source"


class UScanRequestVersionMissing(Problem):

    kind = "uscan-requested-version-missing"

    def __init__(self, version):
        self.version = version

    def __str__(self):
        return "UScan can not find requested version %s." % self.version

    def __repr__(self):
        return "%s(%r)" % (type(self).__name__, self.version)

    def __eq__(self, other):
        return isinstance(self, type(other)) and self.version == other.version


class DebcargoFailure(Problem):

    kind = "debcargo-failed"

    def __init__(self):
        pass

    def __str__(self):
        return "Debcargo failed"

    def __repr__(self):
        return "%s()" % type(self).__name__

    def __eq__(self, other):
        return isinstance(other, type(self))


class UScanFailed(Problem):

    kind = "uscan-failed"

    def __init__(self, url, reason):
        self.url = url
        self.reason = reason

    def __str__(self):
        return "UScan failed to download %s: %s." % (self.url, self.reason)

    def __repr__(self):
        return "%s(%r, %r)" % (type(self).__name__, self.url, self.reason)

    def __eq__(self, other):
        return (
            isinstance(self, type(other))
            and self.url == other.url
            and self.reason == other.reason
        )


class InconsistentSourceFormat(Problem):

    kind = "inconsistent-source-format"

    def __init__(self):
        pass

    def __eq__(self, other):
        return isinstance(other, type(self))

    def __str__(self):
        return "Inconsistent source format between version and source format"


class UpstreamMetadataFileParseError(Problem):

    kind = "debian-upstream-metadata-invalid"

    def __init__(self, path, reason):
        self.path = path
        self.reason = reason

    def __eq__(self, other):
        return (
            isinstance(other, type(self))
            and self.path == other.path
            and self.reason == other.reason
        )

    def __repr__(self):
        return "%s(%r, %r)" % (type(self).__name__, self.path, self.reason)

    def __str__(self):
        return "%s is invalid" % self.path


class DpkgSourcePackFailed(Problem):

    kind = "dpkg-source-pack-failed"

    def __init__(self, reason=None):
        self.reason = reason

    def __eq__(self, other):
        return isinstance(other, type(self)) and other.reason == self.reason

    def __repr__(self):
        return "%s(%r)" % (type(self).__name__, self.reason)

    def __str__(self):
        if self.reason:
            return "Packing source directory failed: %s" % self.reason
        else:
            return "Packing source directory failed."


def find_preamble_failure_description(
    lines: List[str],
) -> Tuple[Optional[int], Optional[str], Optional[Problem]]:
    ret: Tuple[Optional[int], Optional[str], Optional[Problem]] = (None, None, None)
    OFFSET = 20
    err: Problem
    for i in range(1, OFFSET):
        lineno = len(lines) - i
        if lineno < 0:
            break
        line = lines[lineno].strip("\n")
        if line.startswith(
            "dpkg-source: error: aborting due to unexpected upstream " "changes, see "
        ):
            j = lineno - 1
            files: List[str] = []
            while j > 0:
                if lines[j] == (
                    "dpkg-source: info: local changes detected, "
                    "the modified files are:\n"
                ):
                    error = DpkgSourceLocalChanges(files)
                    return lineno + 1, str(error), error
                files.append(lines[j].strip())
                j -= 1
            err = DpkgSourceLocalChanges()
            return lineno + 1, str(error), err
        if line == "dpkg-source: error: unrepresentable changes to source":
            err = DpkgSourceUnrepresentableChanges()
            return lineno + 1, line, err
        if re.match(
            "dpkg-source: error: detected ([0-9]+) unwanted binary " "file.*", line
        ):
            err = DpkgUnwantedBinaryFiles()
            return lineno + 1, line, err
        m = re.match(
            "dpkg-source: error: cannot read (.*/debian/control): "
            "No such file or directory",
            line,
        )
        if m:
            err = MissingControlFile(m.group(1))
            return lineno + 1, line, err
        m = re.match("dpkg-source: error: .*: No space left on device", line)
        if m:
            err = NoSpaceOnDevice()
            return lineno + 1, line, err
        m = re.match("tar: .*: Cannot write: No space left on device", line)
        if m:
            err = NoSpaceOnDevice()
            return lineno + 1, line, err
        m = re.match(
            "dpkg-source: error: cannot represent change to (.*): "
            "binary file contents changed",
            line,
        )
        if m:
            err = DpkgBinaryFileChanged([m.group(1)])
            return lineno + 1, line, err

        m = re.match(
            r"dpkg-source: error: source package format \'(.*)\' is not "
            r"supported: Can\'t locate (.*) in \@INC "
            r"\(you may need to install the (.*) module\) "
            r"\(\@INC contains: (.*)\) at \(eval [0-9]+\) line [0-9]+\.",
            line,
        )
        if m:
            err = SourceFormatUnsupported(m.group(1))
            return lineno + 1, line, err

        m = re.match("dpkg-source: error: (.*)", line)
        if m:
            err = DpkgSourcePackFailed(m.group(1))
            ret = lineno + 1, line, err

        m = re.match("E: Failed to package source directory (.*)", line)
        if m:
            err = DpkgSourcePackFailed()
            ret = lineno + 1, line, err

    return ret


BRZ_ERRORS = [
    (
        "Unable to find the needed upstream tarball for "
        "package (.*), version (.*)\\.",
        lambda m: UnableToFindUpstreamTarball(m.group(1), m.group(2)),
    ),
    (
        "Unknown mercurial extra fields in (.*): b'(.*)'.",
        lambda m: UnknownMercurialExtraFields(m.group(2)),
    ),
    (
        "UScan failed to run: OpenPGP signature did not verify..",
        lambda m: UpstreamPGPSignatureVerificationFailed(),
    ),
    (
        r"Inconsistency between source format and version: "
        r"version is( not)? native, format is( not)? native\.",
        lambda m: InconsistentSourceFormat(),
    ),
    (
        r"UScan failed to run: In (.*) no matching hrefs "
        "for version (.*) in watch line",
        lambda m: UScanRequestVersionMissing(m.group(2)),
    ),
    (
        r"UScan failed to run: In directory ., downloading \s+" r"(.*) failed: (.*)",
        lambda m: UScanFailed(m.group(1), m.group(2)),
    ),
    (
        r"UScan failed to run: In watchfile debian/watch, "
        r"reading webpage\n  (.*) failed: (.*)",
        lambda m: UScanFailed(m.group(1), m.group(2)),
    ),
    (
        r"Unable to parse upstream metadata file (.*): (.*)",
        lambda m: UpstreamMetadataFileParseError(m.group(1), m.group(2)),
    ),
    (r"Debcargo failed to run\.", lambda m: DebcargoFailure()),
]


_BRZ_ERRORS = [(re.compile(r), fn) for (r, fn) in BRZ_ERRORS]


def parse_brz_error(line: str) -> Tuple[Optional[Problem], str]:
    error: Problem
    line = line.strip()
    for search_re, fn in _BRZ_ERRORS:
        m = search_re.match(line)
        if m:
            error = fn(m)
            return (error, str(error))
    if line.startswith("UScan failed to run"):
        return (None, line)
    return (None, line.split("\n")[0])


class MissingRevision(Problem):

    kind = "missing-revision"

    def __init__(self, revision):
        self.revision = revision

    def __str__(self):
        return "Missing revision: %r" % self.revision


def find_creation_session_error(lines):
    ret = None, None, None
    for i in range(len(lines) - 1, 0, -1):
        line = lines[i]
        if line.startswith("E: "):
            ret = i + 1, line, None
        if line.endswith(": No space left on device\n"):
            return i + 1, line, NoSpaceOnDevice()

    return ret


def find_brz_build_error(lines):
    for i in range(len(lines) - 1, 0, -1):
        line = lines[i]
        if line.startswith("brz: ERROR: "):
            rest = [line[len("brz: ERROR: ") :]]
            for n in lines[i + 1 :]:
                if n.startswith(" "):
                    rest.append(n)
            return parse_brz_error("".join(rest))
    return (None, None)


def worker_failure_from_sbuild_log(f: BinaryIO) -> SbuildFailure:  # noqa: C901
    paragraphs = {}
    for title, offsets, lines in parse_sbuild_log(f):
        if title is not None:
            title = title.lower()
        paragraphs[title] = lines
    if len(paragraphs) == 1:
        offset, description, error = find_preamble_failure_description(paragraphs[None])
        if error:
            return SbuildFailure("unpack", description, error)

    failed_stage = find_failed_stage(paragraphs.get("summary", []))
    focus_section = SBUILD_FOCUS_SECTION.get(failed_stage)
    if failed_stage in ("run-post-build-commands", "post-build"):
        # We used to run autopkgtest as the only post build
        # command.
        failed_stage = "autopkgtest"
    description = None
    phase: Optional[Union[Tuple[str], Tuple[str, Optional[str]]]] = None
    error = None
    section_lines = paragraphs.get(focus_section, [])
    if failed_stage == "create-session":
        offset, description, error = find_creation_session_error(section_lines)
        if error:
            phase = ("create-session",)
    if failed_stage == "build":
        section_lines = strip_useless_build_tail(section_lines)
        match, error = find_build_failure_description(section_lines)
        if error:
            description = str(error)
            phase = ("build",)
        elif match:
            description = match.line.rstrip('\n')
    if failed_stage == "autopkgtest":
        section_lines = strip_useless_build_tail(section_lines)
        (
            apt_offset,
            testname,
            apt_error,
            apt_description,
        ) = find_autopkgtest_failure_description(section_lines)
        if apt_error and not error:
            error = apt_error
            if not apt_description:
                apt_description = str(apt_error)
        if apt_description and not description:
            description = apt_description
        if apt_offset is not None:
            offset = apt_offset
        phase = ("autopkgtest", testname)
    if failed_stage == "apt-get-update":
        focus_section, match, error = find_apt_get_update_failure(
            paragraphs
        )
        if error:
            description = str(error)
        elif match:
            description = match.line.rstrip('\n')
        else:
            description = None
    if failed_stage in ("install-deps", "explain-bd-uninstallable"):
        (focus_section, match, error) = find_install_deps_failure_description(
            paragraphs
        )
        if error:
            description = str(error)
        elif match:
            if match.line.startswith("E: "):
                description = match.line[3:].rstrip('\n')
            else:
                description = match.line.rstrip('\n')
    if failed_stage == "arch-check":
        (offset, line, error) = find_arch_check_failure_description(section_lines)
        if error:
            description = str(error)
    if failed_stage == "check-space":
        (offset, line, error) = find_check_space_failure_description(section_lines)
        if error:
            description = str(error)
    if description is None and failed_stage is not None:
        description = "build failed stage %s" % failed_stage
    if description is None:
        description = "build failed"
        phase = ("buildenv",)
        if list(paragraphs.keys()) == [None]:
            for line in reversed(paragraphs[None]):
                m = re.match("Patch (.*) does not apply \\(enforce with -f\\)\n", line)
                if m:
                    patchname = m.group(1).split("/")[-1]
                    error = PatchApplicationFailed(patchname)
                    description = "Patch %s failed to apply" % patchname
                    break
                m = re.match(
                    r"dpkg-source: error: LC_ALL=C patch .* "
                    r"--reject-file=- < .*\/debian\/patches\/([^ ]+) "
                    r"subprocess returned exit status 1",
                    line,
                )
                if m:
                    patchname = m.group(1)
                    error = PatchApplicationFailed(patchname)
                    description = "Patch %s failed to apply" % patchname
                    break
                m = re.match(
                    "dpkg-source: error: "
                    "can't build with source format '(.*)': "
                    "(.*)",
                    line,
                )
                if m:
                    error = SourceFormatUnbuildable(m.group(1))
                    description = m.group(2)
                    break
                m = re.match(
                    "dpkg-source: error: cannot read (.*): "
                    "No such file or directory",
                    line,
                )
                if m:
                    error = PatchFileMissing(m.group(1).split("/", 1)[1])
                    description = "Patch file %s in series but missing" % (error.path)
                    break
                m = re.match(
                    "dpkg-source: error: "
                    "source package format '(.*)' is not supported: "
                    "(.*)",
                    line,
                )
                if m:
                    (match, error) = find_build_failure_description(
                        [m.group(2)]
                    )
                    if error is None:
                        error = SourceFormatUnsupported(m.group(1))
                    if match is None:
                        description = m.group(2)
                    else:
                        description = match.line.rstrip('\n')
                    break
                m = re.match("dpkg-source: error: (.*)", line)
                if m:
                    error = None
                    description = m.group(1)
                    break
                m = re.match(
                    "breezy.errors.NoSuchRevision: " "(.*) has no revision b'(.*)'",
                    line,
                )
                if m:
                    error = MissingRevision(m.group(2).encode())
                    description = "Revision %r is not present" % (error.revision)
                    break
            else:
                (match, error) = find_build_failure_description(paragraphs[None])
                if match is None:
                    error, description = find_brz_build_error(paragraphs[None])
                else:
                    description = match.line.rstrip('\n')

    return SbuildFailure(failed_stage, description, error=error, phase=phase)


def parse_sbuild_log(
    f: BinaryIO,
) -> Iterator[Tuple[Optional[str], Tuple[int, int], List[str]]]:
    begin_offset = 1
    lines: List[str] = []
    title = None
    sep = b"+" + (b"-" * 78) + b"+"
    lineno = 0
    line = f.readline()
    lineno += 1
    while line:
        if line.strip() == sep:
            l1 = f.readline()
            l2 = f.readline()
            lineno += 2
            if l1.startswith(b"|") and l1.strip().endswith(b"|") and l2.strip() == sep:
                end_offset = lineno - 3
                # Drop trailing empty lines
                while lines and lines[-1] == "\n":
                    lines.pop(-1)
                    end_offset -= 1
                if lines:
                    yield title, (begin_offset, end_offset), lines
                title = l1.rstrip()[1:-1].strip().decode(errors="replace")
                lines = []
                begin_offset = lineno
            else:
                lines.extend(
                    [
                        line.decode(errors="replace"),
                        l1.decode(errors="replace"),
                        l2.decode(errors="replace"),
                    ]
                )
        else:
            lines.append(line.decode(errors="replace"))
        line = f.readline()
        lineno += 1
    yield title, (begin_offset, lineno), lines


def find_failed_stage(lines: List[str]) -> Optional[str]:
    for line in lines:
        if not line.startswith("Fail-Stage: "):
            continue
        (key, value) = line.split(": ", 1)
        return value.strip()
    return None


DEFAULT_LOOK_BACK = 50


def strip_useless_build_tail(lines, look_back=None):
    if look_back is None:
        look_back = DEFAULT_LOOK_BACK

    # Strip off unuseful tail
    for i, line in enumerate(lines[-look_back:]):
        if line.startswith("Build finished at "):
            lines = lines[: len(lines) - (look_back - i)]
            if lines and lines[-1] == ("-" * 80 + "\n"):
                lines = lines[:-1]
            break
    try:
        end_offset = lines.index("==> config.log <==\n")
    except ValueError:
        pass
    else:
        lines = lines[:end_offset]

    return lines


class ArchitectureNotInList(Problem):

    kind = "arch-not-in-list"

    def __init__(self, arch, arch_list):
        self.arch = arch
        self.arch_list = arch_list

    def __repr__(self):
        return "%s(%r, %r)" % (type(self).__name__, self.arch, self.arch_list)

    def __str__(self):
        return "Architecture %s not a build arch" % (self.arch,)

    def __eq__(self, other):
        return (
            isinstance(other, type(self))
            and self.arch == other.arch
            and self.arch_list == other.arch_list
        )


def find_arch_check_failure_description(lines):
    for offset, line in enumerate(lines):
        m = re.match(
            r"E: dsc: (.*) not in arch list or does not match any arch "
            r"wildcards: (.*) -- skipping",
            line,
        )
        if m:
            error = ArchitectureNotInList(m.group(1), m.group(2))
            return offset, line, error
    return len(lines) - 1, lines[-1], None


class InsufficientDiskSpace(Problem):

    kind = "insufficient-disk-space"

    def __init__(self, needed, free):
        self.needed = needed
        self.free = free

    def __eq__(self, other):
        return (
            isinstance(other, type(self))
            and self.needed == other.needed
            and self.free == other.free
        )

    def __repr__(self):
        return "%s(%r, %r)" % (type(self).__name__, self.needed, self.free)

    def __str__(self):
        return "Insufficient disk space for build. " "Need: %d KiB, free: %s KiB" % (
            self.needed,
            self.free,
        )


def find_check_space_failure_description(lines):
    for offset, line in enumerate(lines):
        if line == "E: Disk space is probably not sufficient for building.\n":
            m = re.fullmatch(
                r"I: Source needs ([0-9]+) KiB, " r"while ([0-9]+) KiB is free.\)\n",
                lines[offset + 1],
            )
            if m:
                return (
                    offset + 1,
                    line,
                    InsufficientDiskSpace(int(m.group(1)), int(m.group(2))),
                )
            return (offset + 1, line, None)
