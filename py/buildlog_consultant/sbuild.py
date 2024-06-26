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
from typing import BinaryIO, Optional, Union

from . import (
    Match,
    Problem,
    SingleLineMatch,
    version_string,
)
from ._buildlog_consultant_rs import (
    SbuildLog,
    SbuildLogSection,
    parse_sbuild_log,
)
from .apt import (
    find_apt_get_failure,
    find_apt_get_update_failure,
    find_install_deps_failure_description,
)
from .autopkgtest import find_autopkgtest_failure_description
from .common import (
    ChrootNotFound,
    NoSpaceOnDevice,
    PatchApplicationFailed,
    find_build_failure_description,
)

__all__ = [
    "SbuildFailure",
    "parse_sbuild_log",
    "SbuildLog",
    "SbuildLogSection",
]

logger = logging.getLogger(__name__)


class SbuildFailure(Exception):
    """Sbuild failed to run."""

    def __init__(
        self,
        stage: Optional[str],
        description: Optional[str],
        error: Optional["Problem"] = None,
        phase: Optional[Union[tuple[str], tuple[str, Optional[str]]]] = None,
        section: Optional["SbuildLogSection"] = None,
        match: Optional[Match] = None,
    ) -> None:
        self.stage = stage
        self.description = description
        self.error = error
        self.phase = phase
        self.section = section
        self.match = match

    def __repr__(self) -> str:
        return f"{type(self).__name__}({self.stage!r}, {self.description!r}, error={self.error!r}, phase={self.phase!r})"

    def json(self):
        ret = {
            "stage": self.stage,
            "phase": self.phase,
            "section": self.section.title if self.section else None,
            "origin": self.match.origin if self.match else None,
            "lineno": (
                (self.section.offsets[0] if self.section else 0) + self.match.lineno
            )
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


class DpkgSourceLocalChanges(Problem, kind="unexpected-local-upstream-changes"):
    diff_file: Optional[str] = None
    files: Optional[list[str]] = None

    def __repr__(self) -> str:
        if self.files is None:
            return f"<{type(self).__name__}()>"
        if len(self.files) < 5:
            return f"{type(self).__name__}({self.files!r})"
        return "<%s(%d files)>" % (type(self).__name__, len(self.files))

    def __str__(self) -> str:
        if self.files and len(self.files) < 5:
            return f"Tree has local changes: {self.files!r}"
        elif self.files:
            return "Tree has local changes: %d files" % len(self.files)
        else:
            return "Tree has local changes"


class DpkgSourceUnrepresentableChanges(Problem, kind="unrepresentable-local-changes"):
    def __str__(self) -> str:
        return "Tree has unrepresentable local changes."


class DpkgUnwantedBinaryFiles(Problem, kind="unwanted-binary-files"):
    def __str__(self) -> str:
        return "Tree has unwanted binary files."


class DpkgBinaryFileChanged(Problem, kind="changed-binary-files"):
    paths: list[str]

    def __str__(self) -> str:
        return f"Tree has binary files with changes: {self.paths!r}"


class MissingControlFile(Problem, kind="missing-control-file"):
    path: str

    def __str__(self) -> str:
        return f"Tree is missing control file {self.path}"


class UnableToFindUpstreamTarball(Problem, kind="unable-to-find-upstream-tarball"):
    package: str
    version: str

    def __str__(self) -> str:
        return (
            "Unable to find the needed upstream tarball for "
            f"{self.package}, version {self.version}."
        )


class SourceFormatUnbuildable(Problem, kind="source-format-unbuildable"):
    source_format: str
    reason: str

    def __str__(self) -> str:
        return f"Source format {self.source_format} unusable: {self.reason}"


class SourceFormatUnsupported(Problem, kind="unsupported-source-format"):
    source_format: str

    def __str__(self) -> str:
        return f"Source format {self.source_format!r} unsupported"


class PatchFileMissing(Problem, kind="patch-file-missing"):
    path: str

    def __str__(self) -> str:
        return f"Patch file {self.path} missing"


class UnknownMercurialExtraFields(Problem, kind="unknown-mercurial-extra-fields"):
    field: str

    def __str__(self) -> str:
        return f"Unknown Mercurial extra fields: {self.field}"


class UpstreamPGPSignatureVerificationFailed(
    Problem, kind="upstream-pgp-signature-verification-failed"
):
    def __str__(self) -> str:
        return "Unable to verify the PGP signature on the upstream source"


class UScanRequestVersionMissing(Problem, kind="uscan-requested-version-missing"):
    version: str

    def __str__(self) -> str:
        return f"UScan can not find requested version {self.version}."


class DebcargoFailure(Problem, kind="debcargo-failed"):
    reason: str

    def __str__(self) -> str:
        if self.reason:
            return f"Debcargo failed: {self.reason}"
        else:
            return "Debcargo failed"


class ChangelogParseError(Problem, kind="changelog-parse-failed"):
    reason: str

    def __str__(self) -> str:
        return f"Changelog failed to parse: {self.reason}"


class UScanFailed(Problem, kind="uscan-failed"):
    url: str
    reason: str

    def __str__(self) -> str:
        return f"UScan failed to download {self.url}: {self.reason}."


class InconsistentSourceFormat(Problem, kind="inconsistent-source-format"):
    version: Optional[str] = None
    source_format: Optional[str] = None

    def __str__(self) -> str:
        return "Inconsistent source format between version and source format"


class UpstreamMetadataFileParseError(Problem, kind="debian-upstream-metadata-invalid"):
    path: str
    reason: str

    def __str__(self) -> str:
        return f"{self.path} is invalid"


class DpkgSourcePackFailed(Problem, kind="dpkg-source-pack-failed"):
    reason: Optional[str] = None

    def __str__(self) -> str:
        if self.reason:
            return f"Packing source directory failed: {self.reason}"
        else:
            return "Packing source directory failed."


class DpkgBadVersion(Problem, kind="dpkg-bad-version"):
    version: str
    reason: Optional[str] = None

    def __str__(self) -> str:
        if self.reason:
            return f"Version ({self.version}) is invalid: {self.reason}"
        else:
            return f"Version ({self.version}) is invalid"


class MissingDebcargoCrate(Problem, kind="debcargo-missing-crate"):
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

    def __str__(self) -> str:
        ret = f"debcargo can't find crate {self.crate}"
        if self.version:
            ret += f" (version: {self.version})"
        return ret


def find_preamble_failure_description(  # noqa: C901
    lines: list[str],
) -> tuple[Optional[SingleLineMatch], Optional[Problem]]:
    ret: tuple[Optional[SingleLineMatch], Optional[Problem]] = (None, None)
    OFFSET = 100
    err: Problem
    for i in range(1, OFFSET):
        lineno = len(lines) - i
        if lineno < 0:
            break
        line = lines[lineno].strip("\n")
        m = re.fullmatch(
            "dpkg-source: error: aborting due to unexpected upstream "
            "changes, see (.*)",
            line,
        )
        if m:
            diff_file = m.group(1)
            j = lineno - 1
            files: list[str] = []
            while j > 0:
                if lines[j] == (
                    "dpkg-source: info: local changes detected, "
                    "the modified files are:\n"
                ):
                    err = DpkgSourceLocalChanges(diff_file, files)
                    return SingleLineMatch.from_lines(
                        lines, lineno, origin="direct regex"
                    ), err
                files.append(lines[j].strip())
                j -= 1
            err = DpkgSourceLocalChanges(diff_file)
            return SingleLineMatch.from_lines(lines, lineno, origin="direct regex"), err
        if line == "dpkg-source: error: unrepresentable changes to source":
            err = DpkgSourceUnrepresentableChanges()
            return SingleLineMatch.from_lines(lines, lineno, origin="direct match"), err
        if re.match(
            "dpkg-source: error: detected ([0-9]+) unwanted binary " "file.*", line
        ):
            err = DpkgUnwantedBinaryFiles()
            return SingleLineMatch.from_lines(lines, lineno, origin="direct regex"), err
        m = re.match(
            "dpkg-source: error: cannot read (.*/debian/control): "
            "No such file or directory",
            line,
        )
        if m:
            err = MissingControlFile(m.group(1))
            return SingleLineMatch.from_lines(lines, lineno, origin="direct regex"), err
        m = re.match("dpkg-source: error: .*: No space left on device", line)
        if m:
            err = NoSpaceOnDevice()
            return SingleLineMatch.from_lines(lines, lineno, origin="direct regex"), err
        m = re.match("tar: .*: Cannot write: No space left on device", line)
        if m:
            err = NoSpaceOnDevice()
            return SingleLineMatch.from_lines(lines, lineno, origin="direct regex"), err
        m = re.match(
            "dpkg-source: error: cannot represent change to (.*): "
            "binary file contents changed",
            line,
        )
        if m:
            err = DpkgBinaryFileChanged([m.group(1)])
            return SingleLineMatch.from_lines(lines, lineno, origin="direct regex"), err

        m = re.match(
            r"dpkg-source: error: source package format \'(.*)\' is not "
            r"supported: Can\'t locate (.*) in \@INC "
            r"\(you may need to install the (.*) module\) "
            r"\(\@INC contains: (.*)\) at \(eval [0-9]+\) line [0-9]+\.",
            line,
        )
        if m:
            err = SourceFormatUnsupported(m.group(1))
            return SingleLineMatch.from_lines(lines, lineno, origin="direct regex"), err

        m = re.match("E: Failed to package source directory (.*)", line)
        if m:
            err = DpkgSourcePackFailed()
            ret = SingleLineMatch.from_lines(lines, lineno, origin="direct regex"), err

        m = re.match("E: Bad version unknown in (.*)", line)
        if m and lines[lineno - 1].startswith("LINE: "):
            m = re.match(
                r"dpkg-parsechangelog: warning: .*\(l[0-9]+\): "
                r"version \'(.*)\' is invalid: (.*)",
                lines[lineno - 2],
            )
            if m:
                err = DpkgBadVersion(m.group(1), m.group(2))
                return SingleLineMatch.from_lines(
                    lines, lineno, origin="direct regex"
                ), err

        m = re.match("Patch (.*) does not apply \\(enforce with -f\\)\n", line)
        if m:
            patchname = m.group(1).split("/")[-1]
            err = PatchApplicationFailed(patchname)
            return SingleLineMatch.from_lines(lines, lineno, origin="direct regex"), err
        m = re.match(
            r"dpkg-source: error: LC_ALL=C patch .* "
            r"--reject-file=- < .*\/debian\/patches\/([^ ]+) "
            r"subprocess returned exit status 1",
            line,
        )
        if m:
            patchname = m.group(1)
            err = PatchApplicationFailed(patchname)
            return SingleLineMatch.from_lines(lines, lineno, origin="direct regex"), err
        m = re.match(
            "dpkg-source: error: " "can't build with source format '(.*)': " "(.*)",
            line,
        )
        if m:
            err = SourceFormatUnbuildable(m.group(1), m.group(2))
            return SingleLineMatch.from_lines(lines, lineno, origin="direct regex"), err
        m = re.match(
            "dpkg-source: error: cannot read (.*): " "No such file or directory",
            line,
        )
        if m:
            err = PatchFileMissing(m.group(1).split("/", 1)[1])
            return SingleLineMatch.from_lines(lines, lineno, origin="direct regex"), err
        m = re.match(
            "dpkg-source: error: "
            "source package format '(.*)' is not supported: "
            "(.*)",
            line,
        )
        if m:
            (unused_match, p) = find_build_failure_description([m.group(2)])
            if p is None:
                p = SourceFormatUnsupported(m.group(1))
            return SingleLineMatch.from_lines(lines, lineno, origin="direct regex"), p
        m = re.match(
            "breezy.errors.NoSuchRevision: " "(.*) has no revision b'(.*)'",
            line,
        )
        if m:
            err = MissingRevision(m.group(2).encode())
            return SingleLineMatch.from_lines(lines, lineno, origin="direct regex"), err

        m = re.match(
            r"fatal: ambiguous argument \'(.*)\': "
            r"unknown revision or path not in the working tree.",
            line,
        )
        if m:
            err = PristineTarTreeMissing(m.group(1))
            return SingleLineMatch.from_lines(lines, lineno, origin="direct regex"), err

        m = re.match("dpkg-source: error: (.*)", line)
        if m:
            err = DpkgSourcePackFailed(m.group(1))
            ret = SingleLineMatch.from_lines(lines, lineno, origin="direct regex"), err

    return ret


class DebcargoUnacceptablePredicate(Problem, kind="debcargo-unacceptable-predicate"):
    crate: str
    predicate: str

    def __str__(self) -> str:
        return f"Cannot represent prerelease part of dependency: {self.predicate}"


class DebcargoUnacceptableComparator(Problem, kind="debcargo-unacceptable-comparator"):
    crate: str
    comparator: str

    def __str__(self) -> str:
        return f"Cannot represent prerelease part of dependency: {self.comparator}"


def _parse_debcargo_failure(m, pl):
    MORE_TAIL = "\x1b[0m\n"
    MORE_HEAD1 = "\x1b[1;31mSomething failed: "
    MORE_HEAD2 = "\x1b[1;31mdebcargo failed: "
    if pl[-1].endswith(MORE_TAIL):
        extra = [pl[-1][: -len(MORE_TAIL)]]
        for line in reversed(pl[:-1]):
            if extra[0].startswith(MORE_HEAD1):
                extra[0] = extra[0][len(MORE_HEAD1) :]
                break
            if extra[0].startswith(MORE_HEAD2):
                extra[0] = extra[0][len(MORE_HEAD2) :]
                break
            extra.insert(0, line)
        else:
            extra = []
        if extra and extra[-1].strip() == (
            "Try `debcargo update` to update the crates.io index."
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
                return DebcargoUnacceptablePredicate(m.group(1), m.group(2))
            m = re.match(
                r"Cannot represent prerelease part of dependency: (.*) Comparator \{ (.*) \}",
                extra[0],
            )
            if m:
                return DebcargoUnacceptableComparator(m.group(1), m.group(2))
        else:
            return DebcargoFailure("".join(extra))

    return DebcargoFailure("Debcargo failed to run")


class UScanTooManyRequests(Problem, kind="uscan-too-many-requests"):
    url: str

    def __str__(self) -> str:
        return f"UScan: {self.url}: too many requests"


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
        r"UScan failed to run: In watchfile (.*), reading webpage "
        r"(.*) failed: 429 too many requests\.",
        lambda m, pl: UScanTooManyRequests(m.group(2)),
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
        r"Inconsistency between source format and version: "
        r"version (.*) is( not)? native, format '(.*)' is( not)? native\.",
        lambda m, pl: InconsistentSourceFormat(m.group(1), m.group(2)),
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


def parse_brz_error(line: str, prior_lines: list[str]) -> tuple[Optional[Problem], str]:
    error: Problem
    line = line.strip()
    for search_re, fn in _BRZ_ERRORS:
        m = search_re.match(line)
        if m:
            error = fn(m, prior_lines)
            return (error, str(error))
    if line.startswith("UScan failed to run"):
        return (UScanFailed(None, line[len("UScan failed to run: ") :]), line)
    if line.startswith("Unable to parse changelog: "):
        return (ChangelogParseError(line[len("Unable to parse changelog: ") :]), line)
    return (None, line.split("\n")[0])


class MissingRevision(Problem, kind="missing-revision"):
    revision: bytes

    def json(self):
        return {"revision": self.revision.decode("utf-8")}

    @classmethod
    def from_json(cls, json):
        return cls(revision=json["revision"].encode("utf-8"))

    def __str__(self) -> str:
        return f"Missing revision: {self.revision!r}"


class PristineTarTreeMissing(Problem, kind="pristine-tar-missing-tree"):
    treeish: str

    def __str__(self) -> str:
        return f"pristine-tar can not find tree {self.treeish!r}"


def find_creation_session_error(lines):
    ret = None, None
    for i in range(len(lines) - 1, 0, -1):
        line = lines[i]
        if line.startswith("E: "):
            ret = SingleLineMatch.from_lines(lines, i, origin="direct regex"), None
        m = re.fullmatch(
            "E: Chroot for distribution (.*), architecture (.*) not found\n", line
        )
        if m:
            return SingleLineMatch.from_lines(
                lines, i, origin="direct regex"
            ), ChrootNotFound(f"{m.group(1)}-{m.group(2)}-sbuild")
        if line.endswith(": No space left on device\n"):
            return SingleLineMatch.from_lines(
                lines, i, origin="direct regex"
            ), NoSpaceOnDevice()

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


def find_failure_fetch_src(sbuildlog, failed_stage):
    section = sbuildlog.get_section("fetch source files")
    if not section:
        logging.warning("expected section: fetch source files")
        return None
    section_lines = section.lines
    if not section_lines[0].strip():
        section_lines = section_lines[1:]
    match: Optional[Match]
    if len(section_lines) == 1 and section_lines[0].startswith("E: Could not find "):
        match, error = find_preamble_failure_description(
            sbuildlog.get_section_lines(None)
        )
        return SbuildFailure("unpack", str(error), error, section=section, match=match)
    (match, error) = find_apt_get_failure(section.lines)
    description = f"build failed stage {failed_stage}"
    return SbuildFailure(
        failed_stage, description, error=error, phase=None, section=section, match=match
    )


def find_failure_create_session(sbuildlog, failed_stage):
    section = sbuildlog.get_section(None)
    match, error = find_creation_session_error(section.lines)
    phase = ("create-session",)
    description = f"build failed stage {failed_stage}"
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
    description = f"build failed stage {failed_stage}"
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
        description = f"build failed stage {failed_stage}"
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
        description = f"build failed stage {failed_stage}"
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
        description = f"build failed stage {failed_stage}"
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
        description = f"build failed stage {failed_stage}"
    return SbuildFailure(
        failed_stage, description, error=error, phase=None, section=section, match=match
    )


def find_failure_check_space(sbuildlog, failed_stage):
    section = sbuildlog.get_section("cleanup")
    (match, error) = find_check_space_failure_description(section.lines)
    if error:
        description = str(error)
    else:
        description = f"build failed stage {failed_stage}"
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
        description = f"build failed stage {failed_stage}"
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


def worker_failure_from_sbuild_log(f: Union[SbuildLog, BinaryIO]) -> SbuildFailure:
    match: Optional[Match]

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
        if failed_stage is None:
            raise KeyError
        overall_failure = FAILED_STAGE_FAIL_FINDERS[failed_stage](
            sbuildlog, failed_stage
        )
    except KeyError:
        if failed_stage is not None:
            logging.warning("unknown failed stage: %s", failed_stage)
            description = f"build failed stage {failed_stage}"
            return SbuildFailure(
                failed_stage,
                description,
                error=None,
                phase=None,
                section=None,
                match=None,
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
    current_contents: list[str] = []

    header_re = re.compile(r"==\> (.*) \<==\n")
    for i in range(len(lines) - 1, -1, -1):
        m = header_re.match(lines[i])
        if m:
            files[m.group(1)] = current_contents
            current_contents = []
            lines = lines[:i]
            continue

    return lines, files


class ArchitectureNotInList(Problem, kind="arch-not-in-list"):
    arch: str
    arch_list: list[str]

    def __str__(self) -> str:
        return f"Architecture {self.arch} not a build arch"


def find_arch_check_failure_description(
    lines: list[str],
) -> tuple[SingleLineMatch, Optional[Problem]]:
    for offset, line in enumerate(lines):
        m = re.match(
            r"E: dsc: (.*) not in arch list or does not match any arch "
            r"wildcards: (.*) -- skipping",
            line,
        )
        if m:
            error = ArchitectureNotInList(m.group(1), m.group(2))
            return SingleLineMatch.from_lines(
                lines, offset, origin="direct regex"
            ), error
    return SingleLineMatch.from_lines(
        lines, len(lines) - 1, origin="direct regex"
    ), None


class InsufficientDiskSpace(Problem, kind="insufficient-disk-space"):
    needed: int
    free: int

    def __str__(self) -> str:
        return "Insufficient disk space for build. " "Need: %d KiB, free: %s KiB" % (
            self.needed,
            self.free,
        )


def find_check_space_failure_description(
    lines,
) -> tuple[Optional[SingleLineMatch], Optional[Problem]]:
    for offset, line in enumerate(lines):
        if line == "E: Disk space is probably not sufficient for building.\n":
            m = re.fullmatch(
                r"I: Source needs ([0-9]+) KiB, " r"while ([0-9]+) KiB is free.\)\n",
                lines[offset + 1],
            )
            if m:
                return (
                    SingleLineMatch.from_lines(lines, offset, origin="direct regex"),
                    InsufficientDiskSpace(int(m.group(1)), int(m.group(2))),
                )
            return SingleLineMatch.from_lines(lines, offset, origin="direct"), None
    return None, None


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
            logging.info("Failed stage: %s", failed_stage)
        failure = worker_failure_from_sbuild_log(sbuildlog)

        if args.json:
            json.dump(failure.json(), sys.stdout, indent=4)

    if failure.error:
        logging.info("Error: %s", failure.error)
    if failure.match and failure.section:
        logging.info(
            "Failed line: %d:", (failure.section.offsets[0] + failure.match.lineno)
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
