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
from typing import List, Tuple, Iterator, BinaryIO, Optional, Union

from dataclasses import dataclass
import logging

from . import Problem, problem, SingleLineMatch, version_string
from .apt import (
    find_apt_get_failure,
    find_apt_get_update_failure,
    find_install_deps_failure_description,
)
from .autopkgtest import find_autopkgtest_failure_description
from .common import find_build_failure_description, NoSpaceOnDevice, ChrootNotFound, PatchApplicationFailed

__all__ = [
    "SbuildFailure",
    "parse_sbuild_log",
    "SbuildLog",
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
        section: Optional["SbuildLogSection"] = None,
        match: Optional[SingleLineMatch] = None,
    ):
        self.stage = stage
        self.description = description
        self.error = error
        self.phase = phase
        self.section = section
        self.match = match

    def __repr__(self):
        return "%s(%r, %r, error=%r, phase=%r)" % (
            type(self).__name__,
            self.stage,
            self.description,
            self.error,
            self.phase,
        )

    def json(self):
        ret = {
            "stage": self.stage,
            "phase": self.phase,
            "section": self.section.title if self.section else None,
            "lineno": (self.section.offsets[0] + self.match.lineno)
            if self.match
            else None,
        }
        if self.error:
            ret["kind"] = self.error.kind
            try:
                ret["details"] = self.error.json()
            except NotImplementedError:
                ret["details"] = None
        return ret


@problem("unexpected-local-upstream-changes")
class DpkgSourceLocalChanges:

    files: Optional[List[str]] = None

    def __repr__(self):
        if len(self.files) < 5:
            return "%s(%r)" % (type(self).__name__, self.files)
        else:
            return "<%s(%d files)>" % (type(self).__name__, len(self.files))

    def __str__(self):
        if self.files and len(self.files) < 5:
            return "Tree has local changes: %r" % self.files
        elif self.files:
            return "Tree has local changes: %d files" % len(self.files)
        else:
            return "Tree has local changes"


@problem("unrepresentable-local-changes")
class DpkgSourceUnrepresentableChanges:
    def __str__(self):
        return "Tree has unrepresentable local changes."


@problem("unwanted-binary-files")
class DpkgUnwantedBinaryFiles:
    def __str__(self):
        return "Tree has unwanted binary files."


@problem("changed-binary-files")
class DpkgBinaryFileChanged:

    paths: List[str]

    def __str__(self):
        return "Tree has binary files with changes: %r" % self.paths


@problem("missing-control-file")
class MissingControlFile:

    path: str

    def __str__(self):
        return "Tree is missing control file %s" % self.path


@problem("unable-to-find-upstream-tarball")
class UnableToFindUpstreamTarball:

    package: str
    version: str

    def __str__(self):
        return "Unable to find the needed upstream tarball for " "%s, version %s." % (
            self.package,
            self.version,
        )


@problem("source-format-unbuildable")
class SourceFormatUnbuildable:

    source_format: str
    reason: str

    def __str__(self):
        return "Source format %s unusable: %s" % (
            self.source_format, self.reason)


@problem("unsupported-source-format")
class SourceFormatUnsupported:

    source_format: str

    def __str__(self):
        return "Source format %r unsupported" % self.source_format


@problem("patch-file-missing")
class PatchFileMissing:

    path: str

    def __str__(self):
        return "Patch file %s missing" % self.path


@problem("unknown-mercurial-extra-fields")
class UnknownMercurialExtraFields:

    field: str

    def __str__(self):
        return "Unknown Mercurial extra fields: %s" % self.field


@problem("upstream-pgp-signature-verification-failed")
class UpstreamPGPSignatureVerificationFailed:
    def __str__(self):
        return "Unable to verify the PGP signature on the upstream source"


@problem("uscan-requested-version-missing")
class UScanRequestVersionMissing:

    version: str

    def __str__(self):
        return "UScan can not find requested version %s." % self.version


@problem("debcargo-failed")
class DebcargoFailure:

    reason: str

    def __str__(self):
        if self.reason:
            return "Debcargo failed: %s" % self.reason
        else:
            return "Debcargo failed"


@problem("uscan-failed")
class UScanFailed:

    url: str
    reason: str

    def __str__(self):
        return "UScan failed to download %s: %s." % (self.url, self.reason)


@problem("inconsistent-source-format")
class InconsistentSourceFormat:
    def __str__(self):
        return "Inconsistent source format between version and source format"


@problem("debian-upstream-metadata-invalid")
class UpstreamMetadataFileParseError:

    path: str
    reason: str

    def __str__(self):
        return "%s is invalid" % self.path


@problem("dpkg-source-pack-failed")
class DpkgSourcePackFailed:

    reason: Optional[str] = None

    def __str__(self):
        if self.reason:
            return "Packing source directory failed: %s" % self.reason
        else:
            return "Packing source directory failed."


@problem("dpkg-bad-version")
class DpkgBadVersion:

    version: str
    reason: Optional[str] = None

    def __str__(self):
        if self.reason:
            return "Version (%s) is invalid: %s" % (self.version, self.reason)
        else:
            return "Version (%s) is invalid" % self.version


@problem("debcargo-missing-crate")
class MissingDebcargoCrate:

    crate: str
    version: Optional[str] = None

    @classmethod
    def from_string(cls, text):
        text = text.strip()
        if "=" in text:
            (crate, version) = text.split("=")
            return cls(crate.strip(), version.strip())
        else:
            return cls(text)

    def __str__(self):
        ret = "debcargo can't find crate %s" % self.crate
        if self.version:
            ret += " (version: %s)" % self.version
        return ret


def find_preamble_failure_description(  # noqa: C901
    lines: List[str],
) -> Tuple[Optional[SingleLineMatch], Optional[Problem]]:
    ret: Tuple[Optional[int], Optional[str], Optional[Problem]] = (None, None)
    OFFSET = 100
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
                    err = DpkgSourceLocalChanges(files)
                    return SingleLineMatch.from_lines(lines, lineno), err
                files.append(lines[j].strip())
                j -= 1
            err = DpkgSourceLocalChanges()
            return SingleLineMatch.from_lines(lines, lineno), err
        if line == "dpkg-source: error: unrepresentable changes to source":
            err = DpkgSourceUnrepresentableChanges()
            return SingleLineMatch.from_lines(lines, lineno), err
        if re.match(
            "dpkg-source: error: detected ([0-9]+) unwanted binary " "file.*", line
        ):
            err = DpkgUnwantedBinaryFiles()
            return SingleLineMatch.from_lines(lines, lineno), err
        m = re.match(
            "dpkg-source: error: cannot read (.*/debian/control): "
            "No such file or directory",
            line,
        )
        if m:
            err = MissingControlFile(m.group(1))
            return SingleLineMatch.from_lines(lines, lineno), err
        m = re.match("dpkg-source: error: .*: No space left on device", line)
        if m:
            err = NoSpaceOnDevice()
            return SingleLineMatch.from_lines(lines, lineno), err
        m = re.match("tar: .*: Cannot write: No space left on device", line)
        if m:
            err = NoSpaceOnDevice()
            return SingleLineMatch.from_lines(lines, lineno), err
        m = re.match(
            "dpkg-source: error: cannot represent change to (.*): "
            "binary file contents changed",
            line,
        )
        if m:
            err = DpkgBinaryFileChanged([m.group(1)])
            return SingleLineMatch.from_lines(lines, lineno), err

        m = re.match(
            r"dpkg-source: error: source package format \'(.*)\' is not "
            r"supported: Can\'t locate (.*) in \@INC "
            r"\(you may need to install the (.*) module\) "
            r"\(\@INC contains: (.*)\) at \(eval [0-9]+\) line [0-9]+\.",
            line,
        )
        if m:
            err = SourceFormatUnsupported(m.group(1))
            return SingleLineMatch.from_lines(lines, lineno), err

        m = re.match("E: Failed to package source directory (.*)", line)
        if m:
            err = DpkgSourcePackFailed()
            ret = SingleLineMatch.from_lines(lines, lineno), err

        m = re.match("E: Bad version unknown in (.*)", line)
        if m and lines[lineno - 1].startswith("LINE: "):
            m = re.match(
                r"dpkg-parsechangelog: warning: .*\(l[0-9]+\): "
                r"version \'(.*)\' is invalid: (.*)",
                lines[lineno - 2],
            )
            if m:
                err = DpkgBadVersion(m.group(1), m.group(2))
                return SingleLineMatch.from_lines(lines, lineno), err

        m = re.match("Patch (.*) does not apply \\(enforce with -f\\)\n", line)
        if m:
            patchname = m.group(1).split("/")[-1]
            err = PatchApplicationFailed(patchname)
            return SingleLineMatch.from_lines(lines, lineno), err
        m = re.match(
            r"dpkg-source: error: LC_ALL=C patch .* "
            r"--reject-file=- < .*\/debian\/patches\/([^ ]+) "
            r"subprocess returned exit status 1",
            line,
        )
        if m:
            patchname = m.group(1)
            err = PatchApplicationFailed(patchname)
            return SingleLineMatch.from_lines(lines, lineno), err
        m = re.match(
            "dpkg-source: error: " "can't build with source format '(.*)': " "(.*)",
            line,
        )
        if m:
            err = SourceFormatUnbuildable(m.group(1), m.group(2))
            return SingleLineMatch.from_lines(lines, lineno), err
        m = re.match(
            "dpkg-source: error: cannot read (.*): " "No such file or directory",
            line,
        )
        if m:
            err = PatchFileMissing(m.group(1).split("/", 1)[1])
            return SingleLineMatch.from_lines(lines, lineno), err
        m = re.match(
            "dpkg-source: error: "
            "source package format '(.*)' is not supported: "
            "(.*)",
            line,
        )
        if m:
            (match, err) = find_build_failure_description([m.group(2)])
            if err is None:
                err = SourceFormatUnsupported(m.group(1))
            return SingleLineMatch.from_lines(lines, lineno), err
        m = re.match(
            "breezy.errors.NoSuchRevision: " "(.*) has no revision b'(.*)'",
            line,
        )
        if m:
            err = MissingRevision(m.group(2).encode())
            return SingleLineMatch.from_lines(lines, lineno), err

        m = re.match("dpkg-source: error: (.*)", line)
        if m:
            err = DpkgSourcePackFailed(m.group(1))
            ret = SingleLineMatch.from_lines(lines, lineno), err

    return ret


@problem("debcargo-unacceptable-predicate")
class DebcargoUnacceptablePredicate:

    predicate: str

    def __str__(self):
        return "Cannot represent prerelease part of dependency: %s" % (self.predicate)


def _parse_debcargo_failure(m, pl):
    MORE_TAIL = "\x1b[0m\n"
    MORE_HEAD = "\x1b[1;31mSomething failed: "
    if pl[-1].endswith(MORE_TAIL):
        extra = [pl[-1][: -len(MORE_TAIL)]]
        for line in reversed(pl[:-1]):
            if extra[0].startswith(MORE_HEAD):
                extra[0] = extra[0][len(MORE_HEAD) :]
                break
            extra.insert(0, line)
        else:
            extra = []
        if extra and extra[-1] == (
            " Try `debcargo update` to update the crates.io index."
        ):
            n = re.match(r"Couldn\'t find any crate matching (.*)", extra[-2])
            if n:
                return MissingDebcargoCrate.from_string(n.group(1))
            else:
                return DpkgSourcePackFailed(extra[-2])
        elif extra:
            m = re.match(
                r"Cannot represent prerelease part of dependency: (.*) Predicate \{ (.*) \}",
                extra[0],
            )
            if m:
                return DebcargoUnacceptablePredicate(m.group(2))
        else:
            return DebcargoFailure("".join(extra))

    return DebcargoFailure("Debcargo failed to run")


BRZ_ERRORS = [
    (
        "Unable to find the needed upstream tarball for "
        "package (.*), version (.*)\\.",
        lambda m, pl: UnableToFindUpstreamTarball(m.group(1), m.group(2)),
    ),
    (
        "Unknown mercurial extra fields in (.*): b'(.*)'.",
        lambda m, pl: UnknownMercurialExtraFields(m.group(2)),
    ),
    (
        "UScan failed to run: OpenPGP signature did not verify..",
        lambda m, pl: UpstreamPGPSignatureVerificationFailed(),
    ),
    (
        r"Inconsistency between source format and version: "
        r"version is( not)? native, format is( not)? native\.",
        lambda m, pl: InconsistentSourceFormat(),
    ),
    (
        r"UScan failed to run: In (.*) no matching hrefs "
        "for version (.*) in watch line",
        lambda m, pl: UScanRequestVersionMissing(m.group(2)),
    ),
    (
        r"UScan failed to run: In directory ., downloading \s+" r"(.*) failed: (.*)",
        lambda m, pl: UScanFailed(m.group(1), m.group(2)),
    ),
    (
        r"UScan failed to run: In watchfile debian/watch, "
        r"reading webpage\n  (.*) failed: (.*)",
        lambda m, pl: UScanFailed(m.group(1), m.group(2)),
    ),
    (
        r"Unable to parse upstream metadata file (.*): (.*)",
        lambda m, pl: UpstreamMetadataFileParseError(m.group(1), m.group(2)),
    ),
    (r"Debcargo failed to run\.", _parse_debcargo_failure),
    (
        r"\[Errno 28\] No space left on device",
        lambda m, pl: NoSpaceOnDevice(),
    ),
]


_BRZ_ERRORS = [(re.compile(r), fn) for (r, fn) in BRZ_ERRORS]


def parse_brz_error(line: str, prior_lines: List[str]) -> Tuple[Optional[Problem], str]:
    error: Problem
    line = line.strip()
    for search_re, fn in _BRZ_ERRORS:
        m = search_re.match(line)
        if m:
            error = fn(m, prior_lines)
            return (error, str(error))
    if line.startswith("UScan failed to run"):
        return (None, line)
    return (None, line.split("\n")[0])


@problem("missing-revision")
class MissingRevision:

    revision: bytes

    def __str__(self):
        return "Missing revision: %r" % self.revision


def find_creation_session_error(lines):
    ret = None, None
    for i in range(len(lines) - 1, 0, -1):
        line = lines[i]
        if line.startswith("E: "):
            ret = SingleLineMatch.from_lines(lines, i), None
        m = re.fullmatch(
            "E: Chroot for distribution (.*), architecture (.*) not found\n", line
        )
        if m:
            return SingleLineMatch.from_lines(lines, i), ChrootNotFound(
                "%s-%s-sbuild" % (m.group(1), m.group(2))
            )
        if line.endswith(": No space left on device\n"):
            return SingleLineMatch.from_lines(lines, i), NoSpaceOnDevice()

    return ret


def find_brz_build_error(lines):
    for i in range(len(lines) - 1, 0, -1):
        line = lines[i]
        if line.startswith("brz: ERROR: "):
            rest = [line[len("brz: ERROR: ") :]]
            for n in lines[i + 1 :]:
                if n.startswith(" "):
                    rest.append(n)
            return parse_brz_error("".join(rest), lines[:i])
    return (None, None)


@dataclass
class SbuildLogSection:

    title: Optional[str]
    offsets: Tuple[int, int]
    lines: List[str]


@dataclass
class SbuildLog(object):

    sections: List[SbuildLogSection]

    def get_section(self, title):
        for section in self.sections:
            if section.title is None and title is None:
                return section
            if section.title and title and section.title.lower() == title.lower():
                return section

    def get_section_lines(self, title):
        section = self.get_section(title)
        if section:
            return section.lines
        return []

    def section_titles(self):
        return [section.title for section in self.sections]

    @classmethod
    def parse(cls, f: BinaryIO):
        sections = []
        for section in parse_sbuild_log(f):
            logging.debug(
                "Section %s (lines %d-%d)"
                % (section.title, section.offsets[0], section.offsets[1])
            )
            sections.append(section)
        return cls(sections)

    def get_failed_stage(self) -> Optional[str]:
        return find_failed_stage(self.get_section_lines("summary"))


def find_failure_fetch_src(sbuildlog, failed_stage):
    section = sbuildlog.get_section("fetch source files")
    if not section:
        logging.warning("expected section: fetch source files")
        return None
    section_lines = section.lines
    if not section_lines[0].strip():
        section_lines = section_lines[1:]
    if len(section_lines) == 1 and section_lines[0].startswith("E: Could not find "):
        match, error = find_preamble_failure_description(
            sbuildlog.get_section_lines(None)
        )
        return SbuildFailure("unpack", str(error), error, section=section, match=match)
    (match, error) = find_apt_get_failure(section.lines)
    description = "build failed stage %s" % failed_stage
    return SbuildFailure(
        failed_stage, description, error=error, phase=None, section=section, match=match
    )


def find_failure_create_session(sbuildlog, failed_stage):
    section = sbuildlog.get_section(None)
    match, error = find_creation_session_error(section.lines)
    phase = ("create-session",)
    description = "build failed stage %s" % failed_stage
    return SbuildFailure(
        failed_stage,
        description,
        error=error,
        phase=phase,
        section=section,
        match=match,
    )


def find_failure_unpack(sbuildlog, failed_stage):
    section = sbuildlog.get_section("build")
    match, error = find_preamble_failure_description(section.lines)
    if error:
        return SbuildFailure(
            failed_stage, str(error), error, section=section, match=match
        )
    description = "build failed stage %s" % failed_stage
    return SbuildFailure(
        failed_stage, description, error=error, phase=None, section=section, match=match
    )


def find_failure_build(sbuildlog, failed_stage):
    section = sbuildlog.get_section("build")
    phase = ("build",)
    section_lines, files = strip_build_tail(section.lines)
    match, error = find_build_failure_description(section_lines)
    if error:
        description = str(error)
    elif match:
        description = match.line.rstrip("\n")
    else:
        description = "build failed stage %s" % failed_stage
    return SbuildFailure(
        failed_stage,
        description,
        error=error,
        phase=phase,
        section=section,
        match=match,
    )


def find_failure_autopkgtest(sbuildlog, failed_stage):
    focus_section = {
        "run-post-build-commands": "post build commands",
        "post-build": "post build",
        "autopkgtest": "autopkgtest",
    }[failed_stage]
    section = sbuildlog.get_section(focus_section)
    if section is not None:
        (
            match,
            testname,
            error,
            description,
        ) = find_autopkgtest_failure_description(section.lines)
        if not description:
            description = str(error)
        phase = ("autopkgtest", testname)
    else:
        description = None
        error = None
        match = None
        phase = None
    if not description:
        description = "build failed stage %s" % failed_stage
    return SbuildFailure(
        failed_stage,
        description,
        error=error,
        phase=phase,
        section=section,
        match=match,
    )


def find_failure_apt_get_update(sbuildlog, failed_stage):
    focus_section, match, error = find_apt_get_update_failure(sbuildlog)
    if error:
        description = str(error)
    elif match:
        description = match.line.rstrip("\n")
    else:
        description = "build failed stage %s" % failed_stage
    return SbuildFailure(
        failed_stage,
        description,
        error=error,
        phase=None,
        section=sbuildlog.get_section(focus_section),
        match=match,
    )


def find_failure_arch_check(sbuildlog, failed_stage):
    section = sbuildlog.get_section(
        "check architectures",
    )
    (match, error) = find_arch_check_failure_description(section.lines)
    if error:
        description = str(error)
    else:
        description = "build failed stage %s" % failed_stage
    return SbuildFailure(
        failed_stage, description, error=error, phase=None, section=section, match=match
    )


def find_failure_check_space(sbuildlog, failed_stage):
    section = sbuildlog.get_section("cleanup")
    (match, error) = find_check_space_failure_description(section.lines)
    if error:
        description = str(error)
    else:
        description = "build failed stage %s" % failed_stage
    return SbuildFailure(
        failed_stage, description, error=error, phase=None, section=section, match=match
    )


def find_failure_install_deps(sbuildlog, failed_stage):
    (focus_section, match, error) = find_install_deps_failure_description(sbuildlog)
    if error:
        description = str(error)
    elif match:
        if match.line.startswith("E: "):
            description = match.line[3:].rstrip("\n")
        else:
            description = match.line.rstrip("\n")
    else:
        description = "build failed stage %s" % failed_stage
    phase = ("build",)
    return SbuildFailure(
        failed_stage,
        description,
        error=error,
        phase=phase,
        section=sbuildlog.get_section(focus_section),
        match=match,
    )


FAILED_STAGE_FAIL_FINDERS = {
    "fetch-src": find_failure_fetch_src,
    "create-session": find_failure_create_session,
    "unpack": find_failure_unpack,
    "build": find_failure_build,
    "apt-get-update": find_failure_apt_get_update,
    "arch-check": find_failure_arch_check,
    "check-space": find_failure_check_space,
    "install-deps": find_failure_install_deps,
    "explain-bd-uninstallable": find_failure_install_deps,
    "autopkgtest": find_failure_autopkgtest,
    # We run autopkgtest as only post-build step at the moment.
    "run-post-build-commands": find_failure_autopkgtest,
    "post-build": find_failure_autopkgtest,
}


def worker_failure_from_sbuild_log(f: BinaryIO) -> SbuildFailure:  # noqa: C901
    if isinstance(f, SbuildLog):
        sbuildlog = f
    else:
        sbuildlog = SbuildLog.parse(f)

    # TODO(jelmer): Doesn't this do the same thing as the tail?
    if len(sbuildlog.sections) == 1:
        match, error = find_preamble_failure_description(sbuildlog.sections[0].lines)
        if error:
            return SbuildFailure(
                "unpack", str(error), error, section=sbuildlog.sections[0], match=match
            )

    failed_stage = sbuildlog.get_failed_stage()
    try:
        overall_failure = FAILED_STAGE_FAIL_FINDERS[failed_stage](
            sbuildlog, failed_stage
        )
    except KeyError:
        if failed_stage is not None:
            logging.warning("unknown failed stage: %s", failed_stage)
            description = "build failed stage %s" % failed_stage
            return SbuildFailure(
                failed_stage, description, error=None, phase=None, section=None, match=None
            )
    else:
        if overall_failure is not None:
            return overall_failure

    description = "build failed"
    phase = ("buildenv",)
    if sbuildlog.section_titles() == [None]:
        section = sbuildlog.sections[0]
        match, error = find_preamble_failure_description(section.lines)
        if error is not None:
            description = str(error)
        else:
            (match, error) = find_build_failure_description(section.lines)
            if match is None:
                error, description = find_brz_build_error(section.lines)
            else:
                description = match.line.rstrip("\n")

        return SbuildFailure(
            failed_stage,
            description,
            error=error,
            phase=phase,
            section=section,
            match=match,
        )
    return SbuildFailure(
        failed_stage,
        description,
        error=None,
        phase=phase,
        section=None,
        match=None,
    )


def parse_sbuild_log(f: BinaryIO) -> Iterator[SbuildLogSection]:
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
                    yield SbuildLogSection(title, (begin_offset, end_offset), lines)
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
    yield SbuildLogSection(title, (begin_offset, lineno), lines)


def find_failed_stage(lines: List[str]) -> Optional[str]:
    for line in lines:
        if not line.startswith("Fail-Stage: "):
            continue
        (key, value) = line.split(": ", 1)
        return value.strip()
    return None


DEFAULT_LOOK_BACK = 50


def strip_build_tail(lines, look_back=None):
    if look_back is None:
        look_back = DEFAULT_LOOK_BACK

    # Strip off unuseful tail
    for i, line in enumerate(lines[-look_back:]):
        if line.startswith("Build finished at "):
            lines = lines[: len(lines) - (look_back - i)]
            if lines and lines[-1] == ("-" * 80 + "\n"):
                lines = lines[:-1]
            break

    files = {}
    current_contents = []

    header_re = re.compile(r"==\> (.*) \<==\n")
    for i in range(len(lines) - 1, -1, -1):
        m = header_re.match(lines[i])
        if m:
            files[m.group(1)] = current_contents
            current_contents = []
            lines = lines[:i]
            continue

    return lines, files


@problem("arch-not-in-list")
class ArchitectureNotInList:

    arch: str
    arch_list: List[str]

    def __str__(self):
        return "Architecture %s not a build arch" % (self.arch,)


def find_arch_check_failure_description(
    lines: List[str],
) -> Tuple[SingleLineMatch, Optional[Problem]]:
    for offset, line in enumerate(lines):
        m = re.match(
            r"E: dsc: (.*) not in arch list or does not match any arch "
            r"wildcards: (.*) -- skipping",
            line,
        )
        if m:
            error = ArchitectureNotInList(m.group(1), m.group(2))
            return SingleLineMatch.from_lines(lines, offset), error
    return SingleLineMatch.from_lines(lines, len(lines) - 1), None


@problem("insufficient-disk-space")
class InsufficientDiskSpace:

    needed: int
    free: int

    def __str__(self):
        return "Insufficient disk space for build. " "Need: %d KiB, free: %s KiB" % (
            self.needed,
            self.free,
        )


def find_check_space_failure_description(
    lines,
) -> Tuple[SingleLineMatch, Optional[Problem]]:
    for offset, line in enumerate(lines):
        if line == "E: Disk space is probably not sufficient for building.\n":
            m = re.fullmatch(
                r"I: Source needs ([0-9]+) KiB, " r"while ([0-9]+) KiB is free.\)\n",
                lines[offset + 1],
            )
            if m:
                return (
                    SingleLineMatch.from_lines(lines, offset),
                    InsufficientDiskSpace(int(m.group(1)), int(m.group(2))),
                )
            return SingleLineMatch.from_lines(lines, offset), None


def main(argv=None):
    import argparse
    import json

    parser = argparse.ArgumentParser("analyse-sbuild-log")
    parser.add_argument("--debug", action="store_true", help="Display debug output.")
    parser.add_argument("--json", action="store_true", help="Output JSON.")
    parser.add_argument(
        "--context", "-c", type=int, default=5, help="Number of context lines to print."
    )
    parser.add_argument(
        "--version", action="version", version="%(prog)s " + version_string
    )
    parser.add_argument("path", type=str)
    args = parser.parse_args()

    if args.debug:
        loglevel = logging.DEBUG
    elif args.json:
        loglevel = logging.WARNING
    else:
        loglevel = logging.INFO

    logging.basicConfig(level=loglevel, format="%(message)s")

    with open(args.path, "rb") as f:
        sbuildlog = SbuildLog.parse(f)

        failed_stage = sbuildlog.get_failed_stage()
        if failed_stage:
            logging.info("Failed stage: %s" % failed_stage)
        failure = worker_failure_from_sbuild_log(sbuildlog)

        if args.json:
            json.dump(failure.json(), sys.stdout, indent=4)

    if failure.error:
        logging.info("Error: %s" % failure.error)
    if failure.match:
        logging.info(
            "Failed line: %d:" % (failure.section.offsets[0] + failure.match.lineno)
        )
        for i in range(
            max(0, failure.match.offset - args.context),
            min(len(failure.section.lines), failure.match.offset + args.context + 1),
        ):
            logging.info(
                " %s  %s",
                ">" if failure.match.offset == i else " ",
                failure.section.lines[i].rstrip("\n"),
            )


if __name__ == "__main__":
    import sys

    sys.exit(main(sys.argv))
