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

from debian.changelog import Version
from debian.deb822 import PkgRelation
from typing import List, Optional, Tuple
import yaml

from . import Problem, SingleLineMatch, problem
from .common import NoSpaceOnDevice


class DpkgError(Problem):

    kind = "dpkg-error"

    def __init__(self, error):
        self.error = error

    def __eq__(self, other):
        return isinstance(other, type(self)) and self.error == other.error

    def __str__(self):
        return "Dpkg Error: %s" % self.error

    def __repr__(self):
        return "%s(%r)" % (type(self).__name__, self.error)


class AptUpdateError(Problem):
    """Apt update error."""

    kind = "apt-update-error"


class AptFetchFailure(AptUpdateError):
    """Apt file fetch failed."""

    kind = "apt-file-fetch-failure"

    def __init__(self, url, error):
        self.url = url
        self.error = error

    def __eq__(self, other):
        if not isinstance(other, type(self)):
            return False
        if self.url != other.url:
            return False
        if self.error != other.error:
            return False
        return True

    def __str__(self):
        return "Apt file fetch error: %s" % self.error


class AptMissingReleaseFile(AptUpdateError):

    kind = "missing-release-file"

    def __init__(self, url):
        self.url = url

    def __eq__(self, other):
        if not isinstance(other, type(self)):
            return False
        if self.url != self.url:
            return False
        return True

    def __str__(self):
        return "Missing release file: %s" % self.url


class AptPackageUnknown(Problem):

    kind = "apt-package-unknown"

    def __init__(self, package):
        self.package = package

    def __eq__(self, other):
        return isinstance(other, type(self)) and self.package == other.package

    def __str__(self):
        return "Unknown package: %s" % self.package

    def __repr__(self):
        return "%s(%r)" % (type(self).__name__, self.package)


class AptBrokenPackages(Problem):

    kind = "apt-broken-packages"

    def __init__(self, description):
        self.description = description

    def __str__(self):
        return "Broken apt packages: %s" % (self.description,)

    def __repr__(self):
        return "%s(%r)" % (type(self).__name__, self.description)

    def __eq__(self, other):
        return isinstance(other, type(self)) and self.description == other.description


def find_apt_get_failure(lines):  # noqa: C901
    """Find the key failure line in apt-get-output.

    Returns:
      tuple with (match, error object)
    """
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
                return SingleLineMatch.from_lines(lines, lineno), problem
            return SingleLineMatch.from_lines(lines, lineno), None
        if line in (
            "E: Broken packages",
            "E: Unable to correct problems, you have held broken " "packages.",
        ):
            error = AptBrokenPackages(lines[lineno - 1].strip())
            return SingleLineMatch.from_lines(lines, lineno - 1), error
        m = re.match("E: The repository '([^']+)' does not have a Release file.", line)
        if m:
            return SingleLineMatch.from_lines(lines, lineno), AptMissingReleaseFile(
                m.group(1)
            )
        m = re.match(
            "dpkg-deb: error: unable to write file '(.*)': " "No space left on device",
            line,
        )
        if m:
            return SingleLineMatch.from_lines(lines, lineno), NoSpaceOnDevice()
        m = re.match(r"E: You don't have enough free space in (.*)\.", line)
        if m:
            return SingleLineMatch.from_lines(lines, lineno), NoSpaceOnDevice()
        if line.startswith("E: ") and ret[0] is None:
            ret = SingleLineMatch.from_lines(lines, lineno), None
        m = re.match(r"E: Unable to locate package (.*)", line)
        if m:
            return SingleLineMatch.from_lines(lines, lineno), AptPackageUnknown(
                m.group(1)
            )
        if line == "E: Write error - write (28: No space left on device)":
            return SingleLineMatch.from_lines(lines, lineno), NoSpaceOnDevice()
        m = re.match(r"dpkg: error: (.*)", line)
        if m:
            if m.group(1).endswith(": No space left on device"):
                return SingleLineMatch.from_lines(lines, lineno), NoSpaceOnDevice()
            return SingleLineMatch.from_lines(lines, lineno), DpkgError(m.group(1))
        m = re.match(r"dpkg: error processing package (.*) \((.*)\):", line)
        if m:
            return (
                SingleLineMatch.from_lines(lines, lineno + 1),
                DpkgError("processing package %s (%s)" % (m.group(1), m.group(2))),
            )

    for i, line in enumerate(lines):
        m = re.match(
            r" cannot copy extracted data for '(.*)' to "
            r"'(.*)': failed to write \(No space left on device\)",
            line,
        )
        if m:
            return SingleLineMatch.from_lines(lines, lineno), NoSpaceOnDevice()
        m = re.match(r" .*: No space left on device", line)
        if m:
            return SingleLineMatch.from_lines(lines, i), NoSpaceOnDevice()

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


try:
    from typing import TypedDict
except ImportError:  # python < 3.9
    from typing import Dict, Any

    ParsedRelation = Dict[str, Dict[str, Any]]
else:
    ParsedRelation = TypedDict(
        "ParsedRelation",
        {
            "name": str,
            "archqual": Optional[str],
            "version": Optional[Tuple[str, str]],
            "arch": Optional[List["PkgRelation.ArchRestriction"]],
            "restrictions": Optional[List[List["PkgRelation.BuildRestriction"]]],
        },
    )


@problem("unsatisfied-apt-dependencies")
class UnsatisfiedAptDependencies:

    relations: List[List[List[ParsedRelation]]]

    def __str__(self):
        return "Unsatisfied APT dependencies: %s" % PkgRelation.str(self.relations)

    @classmethod
    def from_str(cls, text):
        return cls(PkgRelation.parse_relations(text))

    @classmethod
    def from_json(cls, data):
        relations = []
        for relation in data['relations']:
            sub = []
            for entry in relation:
                pkg = {
                    'name': entry['name'],
                    'archqual': entry.get('archqual'),
                    'arch': entry.get('arch'),
                    'restrictions': entry.get('restrictions'),
                    'version': (entry['version'][0], Version(entry['version'][1])) if entry['version'] else None,
                    }
                sub.append(pkg)
            relations.append(sub)
        return cls(relations=relations)

    def __repr__(self):
        return "%s.from_str(%r)" % (
            type(self).__name__,
            PkgRelation.str(self.relations),
        )


@problem("unsatisfied-apt-conflicts")
class UnsatisfiedAptConflicts:

    relations: List[List[List[ParsedRelation]]]

    def __str__(self):
        return "Unsatisfied APT conflicts: %s" % PkgRelation.str(self.relations)


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


def find_install_deps_failure_description(sbuildlog) -> Tuple[Optional[str], Optional[SingleLineMatch], Optional[Problem]]:
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
