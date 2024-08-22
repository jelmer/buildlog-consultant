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

import re  # noqa: I001
from typing import Optional, TypedDict

import yaml

from debian.changelog import Version
from debian.deb822 import PkgRelation

from . import Match, Problem, _buildlog_consultant_rs


class DpkgError(Problem, kind="dpkg-error"):
    error: str

    def __eq__(self, other):
        return isinstance(other, type(self)) and self.error == other.error

    def __str__(self) -> str:
        return f"Dpkg Error: {self.error}"

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
        return f"Apt file fetch error: {self.error}"


class AptMissingReleaseFile(AptUpdateError, kind="missing-release-file"):
    url: str

    def __eq__(self, other):
        if not isinstance(other, type(self)):
            return False
        if self.url != self.url:
            return False
        return True

    def __str__(self) -> str:
        return f"Missing release file: {self.url}"


class AptPackageUnknown(Problem, kind="apt-package-unknown"):
    package: str

    def __eq__(self, other):
        return isinstance(other, type(self)) and self.package == other.package

    def __str__(self) -> str:
        return f"Unknown package: {self.package}"

    def __repr__(self) -> str:
        return f"{type(self).__name__}({self.package!r})"


class AptBrokenPackages(Problem, kind="apt-broken-packages"):
    description: str
    broken: Optional[str] = None

    def __str__(self) -> str:
        if self.broken:
            return f"Broken apt packages: {self.broken!r}"
        return f"Broken apt packages: {self.description}"

    def __repr__(self) -> str:
        return f"{type(self).__name__}({self.description!r}, {self.broken!r})"

    def __eq__(self, other):
        return (
            isinstance(other, type(self))
            and self.description == other.description
            and self.broken == other.broken
        )


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
    arch: Optional[list["PkgRelation.ArchRestriction"]]
    restrictions: Optional[list[list["PkgRelation.BuildRestriction"]]]


class UnsatisfiedAptDependencies(Problem, kind="unsatisfied-apt-dependencies"):
    relations: list[list[list[ParsedRelation]]]

    def __str__(self) -> str:
        return f"Unsatisfied APT dependencies: {PkgRelation.str(self.relations)}"  # type: ignore

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
        for relation in data["relations"]:
            sub = []
            for entry in relation:
                pkg = {
                    "name": entry["name"],
                    "archqual": entry.get("archqual"),
                    "arch": entry.get("arch"),
                    "restrictions": entry.get("restrictions"),
                    "version": (entry["version"][0], Version(entry["version"][1]))
                    if entry["version"]
                    else None,
                }
                sub.append(pkg)
            relations.append(sub)
        return cls(relations=relations)

    def __repr__(self) -> str:
        return "{type(self).__name__}.from_str({PkgRelation.str(self.relations)!r})"  # type: ignore


class UnsatisfiedAptConflicts(Problem, kind="unsatisfied-apt-conflicts"):
    relations: list[list[list[ParsedRelation]]]

    def __str__(self) -> str:
        return f"Unsatisfied APT conflicts: {PkgRelation.str(self.relations)}"  # type: ignore


def error_from_dose3_report(report):
    def fixup_relation(rel):
        for o in rel:
            for d in o:
                if d["version"]:
                    try:
                        newoperator = {"<": "<<", ">": ">>"}[d["version"][0]]
                    except KeyError:
                        pass
                    else:
                        d["version"] = (newoperator, d["version"][1])

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


def find_install_deps_failure_description(
    sbuildlog
) -> tuple[Optional[str], Optional[Match], Optional[Problem]]:
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


find_apt_get_failure = _buildlog_consultant_rs.find_apt_get_failure
