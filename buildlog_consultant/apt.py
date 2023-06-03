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
from typing import Optional, TypedDict

import yaml
from debian.changelog import Version
from debian.deb822 import PkgRelation

from . import Match, MultiLineMatch, Problem, SingleLineMatch
from .common import NoSpaceOnDevice


class DpkgError(Problem, kind="dpkg-error"):

    error: str

    def __eq__(self, other):
        return isinstance(other, type(self)) and self.error == other.error

    def __str__(self) -> str:
        return "Dpkg Error: %s" % self.error

    def __repr__(self) -> str:
        return f"{type(self).__name__}({self.error!r})"


class AptUpdateError(Problem, kind="apt-update-error"):
    """Apt update error."""


class AptFetchFailure(AptUpdateError, kind="apt-file-fetch-failure"):
    """Apt file fetch failed."""

    url: str
    error: str

    def __eq__(self, other):
        if not isinstance(other, type(self)):
            return False
        if self.url != other.url:
            return False
        if self.error != other.error:
            return False
        return True

    def __str__(self) -> str:
        return "Apt file fetch error: %s" % self.error


class AptMissingReleaseFile(AptUpdateError, kind="missing-release-file"):

    url: str

    def __eq__(self, other):
        if not isinstance(other, type(self)):
            return False
        if self.url != self.url:
            return False
        return True

    def __str__(self) -> str:
        return "Missing release file: %s" % self.url


class AptPackageUnknown(Problem, kind="apt-package-unknown"):

    package: str

    def __eq__(self, other):
        return isinstance(other, type(self)) and self.package == other.package

    def __str__(self) -> str:
        return "Unknown package: %s" % self.package

    def __repr__(self) -> str:
        return f"{type(self).__name__}({self.package!r})"


class AptBrokenPackages(Problem, kind="apt-broken-packages"):

    description: str
    broken: Optional[str] = None

    def __str__(self) -> str:
        if self.broken:
            return "Broken apt packages: %r" % self.broken
        return f"Broken apt packages: {self.description}"

    def __repr__(self) -> str:
        return f"{type(self).__name__}({self.description!r}, {self.broken!r})"

    def __eq__(self, other):
        return (isinstance(other, type(self))
                and self.description == other.description
                and self.broken == other.broken)


def find_apt_get_failure(lines: list[str]) -> tuple[Optional[Match], Optional[Problem]]:  # noqa: C901
    """Find the key failure line in apt-get-output.

    Returns:
      tuple with (match, error object)
    """
    problem: Problem
    ret = (None, None)
    OFFSET = 50
    for i in range(1, OFFSET):
        lineno = len(lines) - i
        if lineno < 0:
            break
        line = lines[lineno].strip("\n")
        if line.startswith("E: Failed to fetch "):
            m = re.match("^E: Failed to fetch ([^ ]+)  (.*)", line)
            if m:
                if "No space left on device" in m.group(2):
                    problem = NoSpaceOnDevice()
                else:
                    problem = AptFetchFailure(m.group(1), m.group(2))
                return SingleLineMatch.from_lines(lines, lineno, origin="direct regex"), problem
            return SingleLineMatch.from_lines(lines, lineno, origin="direct regex"), None
        if line == "E: Broken packages":
            error = AptBrokenPackages(lines[lineno - 1].strip())
            return SingleLineMatch.from_lines(lines, lineno - 1, origin="direct match"), error
        if line == "E: Unable to correct problems, you have held broken packages.":
            offsets = []
            broken = []
            for j in range(lineno - 1, 0, -1):
                m = re.match(r'\s*Depends: (.*) but it is not (going to be installed|installable)', lines[j])
                if m:
                    offsets.append(j)
                    broken.append(m.group(1))
                    continue
                m = re.match(r'\s*(.*) : Depends: (.*) but it is not (going to be installed|installable)', lines[j])
                if m:
                    offsets.append(j)
                    broken.append(m.group(2))
                    continue
                break
            error = AptBrokenPackages(lines[lineno].strip(), broken)
            return MultiLineMatch.from_lines(lines, offsets + [lineno], origin="direct match"), error
        m = re.match("E: The repository '([^']+)' does not have a Release file.", line)
        if m:
            return SingleLineMatch.from_lines(lines, lineno, origin="direct regex"), AptMissingReleaseFile(
                m.group(1)
            )
        m = re.match(
            "dpkg-deb: error: unable to write file '(.*)': " "No space left on device",
            line,
        )
        if m:
            return SingleLineMatch.from_lines(lines, lineno, origin="direct regex"), NoSpaceOnDevice()
        m = re.match(r"E: You don't have enough free space in (.*)\.", line)
        if m:
            return SingleLineMatch.from_lines(lines, lineno, origin="direct regex"), NoSpaceOnDevice()
        if line.startswith("E: ") and ret[0] is None:
            ret = SingleLineMatch.from_lines(lines, lineno, origin="direct regex"), None
        m = re.match(r"E: Unable to locate package (.*)", line)
        if m:
            return SingleLineMatch.from_lines(lines, lineno, origin="direct regex"), AptPackageUnknown(
                m.group(1)
            )
        if line == "E: Write error - write (28: No space left on device)":
            return SingleLineMatch.from_lines(lines, lineno, origin="direct regex"), NoSpaceOnDevice()
        m = re.match(r"dpkg: error: (.*)", line)
        if m:
            if m.group(1).endswith(": No space left on device"):
                return SingleLineMatch.from_lines(lines, lineno, origin="direct regex"), NoSpaceOnDevice()
            return SingleLineMatch.from_lines(lines, lineno, origin="direct regex"), DpkgError(m.group(1))
        m = re.match(r"dpkg: error processing package (.*) \((.*)\):", line)
        if m:
            return (
                SingleLineMatch.from_lines(lines, lineno + 1, origin="direct regex"),
                DpkgError(f"processing package {m.group(1)} ({m.group(2)})"),
            )

    for i, line in enumerate(lines):
        m = re.match(
            r" cannot copy extracted data for '(.*)' to "
            r"'(.*)': failed to write \(No space left on device\)",
            line,
        )
        if m:
            return SingleLineMatch.from_lines(lines, lineno, origin="direct regex"), NoSpaceOnDevice()
        m = re.match(r" .*: No space left on device", line)
        if m:
            return SingleLineMatch.from_lines(lines, i, origin="direct regex"), NoSpaceOnDevice()

    return ret


def find_apt_get_update_failure(sbuildlog):
    focus_section = "update chroot"
    lines = sbuildlog.get_section_lines(focus_section)
    match, error = find_apt_get_failure(lines)
    return focus_section, match, error


def find_cudf_output(lines):
    for i in range(len(lines) - 1, 0, -1):
        if lines[i].startswith("output-version: "):
            break
    else:
        return None
    output = []
    while lines[i].strip():
        output.append(lines[i])
        i += 1

    return yaml.safe_load("\n".join(output))


class ParsedRelation(TypedDict):
    name: str
    archqual: Optional[str]
    version: Optional[tuple[str, str]]
    arch: Optional[list['PkgRelation.ArchRestriction']]
    restrictions: Optional[list[list['PkgRelation.BuildRestriction']]]


class UnsatisfiedAptDependencies(Problem, kind="unsatisfied-apt-dependencies"):

    relations: list[list[list[ParsedRelation]]]

    def __str__(self) -> str:
        return "Unsatisfied APT dependencies: %s" % (
            PkgRelation.str(self.relations))  # type: ignore

    @classmethod
    def from_str(cls, text):
        return cls(PkgRelation.parse_relations(text))

    def __eq__(self, other):
        return isinstance(other, type(self)) and other.relations == self.relations

    def json(self):
        return PkgRelation.str(self.relations)  # type: ignore

    @classmethod
    def from_json(cls, data):
        if isinstance(data, str):
            return cls.from_str(data)
        relations = []
        for relation in data['relations']:
            sub = []
            for entry in relation:
                pkg = {
                    'name': entry['name'],
                    'archqual': entry.get('archqual'),
                    'arch': entry.get('arch'),
                    'restrictions': entry.get('restrictions'),
                    'version':
                        (entry['version'][0], Version(entry['version'][1]))
                        if entry['version'] else None,
                }
                sub.append(pkg)
            relations.append(sub)
        return cls(relations=relations)

    def __repr__(self) -> str:
        return "{}.from_str({!r})".format(
            type(self).__name__,
            PkgRelation.str(self.relations),  # type: ignore
        )


class UnsatisfiedAptConflicts(Problem, kind="unsatisfied-apt-conflicts"):

    relations: list[list[list[ParsedRelation]]]

    def __str__(self) -> str:
        return "Unsatisfied APT conflicts: %s" % PkgRelation.str(
            self.relations)  # type: ignore


def error_from_dose3_report(report):
    def fixup_relation(rel):
        for o in rel:
            for d in o:
                if d['version']:
                    try:
                        newoperator = {'<': '<<', '>': '>>'}[d['version'][0]]
                    except KeyError:
                        pass
                    else:
                        d['version'] = (newoperator, d['version'][1])
    packages = [entry["package"] for entry in report]
    assert packages == ["sbuild-build-depends-main-dummy"]
    if report[0]["status"] != "broken":
        return None
    missing = []
    conflict = []
    for reason in report[0]["reasons"]:
        if "missing" in reason:
            relation = PkgRelation.parse_relations(
                reason["missing"]["pkg"]["unsat-dependency"]
            )
            fixup_relation(relation)
            missing.extend(relation)
        if "conflict" in reason:
            relation = PkgRelation.parse_relations(
                reason["conflict"]["pkg1"]["unsat-conflict"]
            )
            fixup_relation(relation)
            conflict.extend(relation)
    if missing:
        return UnsatisfiedAptDependencies(missing)
    if conflict:
        return UnsatisfiedAptConflicts(conflict)


def find_install_deps_failure_description(sbuildlog) -> tuple[Optional[str], Optional[Match], Optional[Problem]]:
    error = None

    DOSE3_SECTION = "install dose3 build dependencies (aspcud-based resolver)"
    dose3_lines = sbuildlog.get_section_lines(DOSE3_SECTION)
    if dose3_lines:
        dose3_output = find_cudf_output(dose3_lines)
        if dose3_output:
            error = error_from_dose3_report(dose3_output["report"])
        return DOSE3_SECTION, None, error

    SECTION = "install package build dependencies"
    build_dependencies_lines = sbuildlog.get_section_lines(SECTION)
    if build_dependencies_lines:
        dose3_output = find_cudf_output(build_dependencies_lines)
        if dose3_output:
            error = error_from_dose3_report(dose3_output["report"])
            return SECTION, None, error
        match, error = find_apt_get_failure(build_dependencies_lines)
        return SECTION, match, error

    for section in sbuildlog.sections:
        if section.title is None:
            continue
        if re.match("install (.*) build dependencies.*", section.title.lower()):
            match, error = find_apt_get_failure(section.lines)
            if match is not None:
                return section.title, match, error

    return section.title, None, error


if __name__ == '__main__':
    import argparse
    import logging
    parser = argparse.ArgumentParser()
    parser.add_argument("--debug", action="store_true", help="Display debug output.")
    parser.add_argument(
        "--context", "-c", type=int, default=5, help="Number of context lines to print."
    )

    parser.add_argument("path", type=str)
    args = parser.parse_args()
    if args.debug:
        loglevel = logging.DEBUG
    else:
        loglevel = logging.INFO

    logging.basicConfig(level=loglevel, format="%(message)s")

    with open(args.path) as f:
        lines = list(f.readlines())
    match, error = find_apt_get_failure(lines)

    if error:
        logging.info("Error: %s", error)
    if match:
        logging.info("Failed line: %d:", match.lineno)
        for i in range(
            max(0, match.offset - args.context),
            min(len(lines), match.offset + args.context + 1),
        ):
            logging.info(
                " %s  %s",
                ">" if match.offset == i else " ",
                lines[i].rstrip("\n"),
            )
