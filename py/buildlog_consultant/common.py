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
from typing import Optional

from . import (
    Problem,
    _buildlog_consultant_rs,  # type: ignore
    )

logger = logging.getLogger(__name__)


class MissingPythonModule(Problem, kind="missing-python-module"):
    module: str
    python_version: Optional[str] = None
    minimum_version: Optional[str] = None

    def __str__(self) -> str:
        if self.python_version:
            ret = f"Missing python {self.python_version} module: "
        else:
            ret = "Missing python module: "
        ret += self.module
        if self.minimum_version:
            return ret + f" (>= {self.minimum_version})"
        else:
            return ret

    def __repr__(self) -> str:
        return f"{type(self).__name__}({self.module!r}, python_version={self.python_version!r}, minimum_version={self.minimum_version!r})"


class SetuptoolScmVersionIssue(Problem, kind="setuptools-scm-version-issue"):
    def __str__(self) -> str:
        return "setuptools-scm was unable to find version"


class MissingOCamlPackage(Problem, kind="missing-ocaml-package"):
    package: str

    def __str__(self) -> str:
        return f"Missing OCaml package: {self.package}"


class MissingPythonDistribution(Problem, kind="missing-python-distribution"):
    distribution: str
    python_version: Optional[int] = None
    minimum_version: Optional[str] = None

    def __str__(self) -> str:
        if self.python_version:
            ret = "Missing python %d distribution: " % self.python_version
        else:
            ret = "Missing python distribution: "
        ret += self.distribution
        if self.minimum_version:
            return ret + f" (>= {self.minimum_version})"
        else:
            return ret

    @classmethod
    def from_requirement_str(cls, text, python_version=None):
        from requirements.requirement import Requirement

        req = Requirement.parse(text)
        if len(req.specs) == 1 and req.specs[0][0] == ">=":
            return cls(req.name, python_version, req.specs[0][1])
        return cls(req.name, python_version)

    def __repr__(self) -> str:
        return f"{type(self).__name__}({self.distribution!r}, python_version={self.python_version!r}, minimum_version={self.minimum_version!r})"


class VcsControlDirectoryNeeded(Problem, kind="vcs-control-directory-needed"):
    vcs: list[str]

    def __str__(self) -> str:
        return "Version control directory needed"


class PatchApplicationFailed(Problem, kind="patch-application-failed"):
    patchname: str

    def __str__(self) -> str:
        return f"Patch application failed: {self.patchname}"


class MissingVagueDependency(Problem, kind="missing-vague-dependency"):
    name: str
    url: Optional[str] = None
    minimum_version: Optional[str] = None
    current_version: Optional[str] = None

    def __repr__(self) -> str:
        return f"{type(self).__name__}({self.name!r}, url={self.url!r}, minimum_version={self.minimum_version!r}, current_version={self.current_version!r})"

    def __str__(self) -> str:
        return f"Missing dependency: {self.name}"


class MissingQt(Problem, kind="missing-qt"):
    minimum_version: Optional[str] = None

    def __str__(self) -> str:
        if self.minimum_version:
            return f"Missing QT installation (at least {self.minimum_version})"
        return "Missing QT installation"


class MissingQtModules(Problem, kind="missing-qt-modules"):
    modules: list[str]

    def __str__(self) -> str:
        return f"Missing QT modules: {self.modules!r}"


class MissingX11(Problem, kind="missing-x11"):
    def __str__(self) -> str:
        return "Missing X11 headers"


class MissingGitIdentity(Problem, kind="missing-git-identity"):
    def __str__(self) -> str:
        return "Missing Git Identity"


class MissingFile(Problem, kind="missing-file"):
    path: str

    def __str__(self) -> str:
        return f"Missing file: {self.path}"


class MissingCommandOrBuildFile(Problem, kind="missing-command-or-build-file"):
    filename: str

    @property
    def command(self):
        return self.filename

    def __str__(self) -> str:
        return f"Missing command or build file: {self.filename}"


class MissingBuildFile(Problem, kind="missing-build-file"):
    filename: str

    def __str__(self) -> str:
        return f"Missing build file: {self.filename}"

class MissingJDKFile(Problem, kind="missing-jdk-file"):
    jdk_path: str
    filename: str

    def __str__(self) -> str:
        return f"Missing JDK file {self.filename} (JDK Path: {self.jdk_path})"


class MissingJDK(Problem, kind="missing-jdk"):
    jdk_path: str

    def __str__(self) -> str:
        return f"Missing JDK (JDK Path: {self.jdk_path})"


class MissingJRE(Problem, kind="missing-jre"):
    def __str__(self) -> str:
        return "Missing JRE"


class ChrootNotFound(Problem, kind="chroot-not-found"):
    chroot: str

    def __str__(self) -> str:
        return f"Chroot not found: {self.chroot}"


class MissingSprocketsFile(Problem, kind="missing-sprockets-file"):
    name: str
    content_type: str

    def __str__(self) -> str:
        return f"Missing sprockets file: {self.name} (type: {self.content_type})"


class MissingGoPackage(Problem, kind="missing-go-package"):
    package: str

    def __str__(self) -> str:
        return f"Missing Go package: {self.package}"


class MissingCHeader(Problem, kind="missing-c-header"):
    header: str

    def __str__(self) -> str:
        return f"Missing C Header: {self.header}"


class MissingNodeModule(Problem, kind="missing-node-module"):
    module: str

    def __str__(self) -> str:
        return f"Missing Node Module: {self.module}"


class MissingNodePackage(Problem, kind="missing-node-package"):
    package: str

    def __str__(self) -> str:
        return f"Missing Node Package: {self.package}"


class MissingCommand(Problem, kind="command-missing"):
    command: str

    def __str__(self) -> str:
        return f"Missing command: {self.command}"

    def __repr__(self) -> str:
        return f"{type(self).__name__}({self.command!r})"


class NotExecutableFile(Problem, kind="command-not-executable"):
    path: str

    def __str__(self) -> str:
        return f"Command not executable: {self.path}"


class MissingSecretGpgKey(Problem, kind="no-secret-gpg-key"):
    def __str__(self) -> str:
        return "No secret GPG key is present"


class MissingVcVersionerVersion(Problem, kind="no-vcversioner-version"):
    def __str__(self) -> str:
        return "vcversion could not find a git directory or version.txt file"


class MissingConfigure(Problem, kind="missing-configure"):
    def __str__(self) -> str:
        return "Missing configure script"


class MissingJavaScriptRuntime(Problem, kind="javascript-runtime-missing"):
    def __str__(self) -> str:
        return "Missing JavaScript Runtime"


class MissingPHPExtension(Problem, kind="missing-php-extension"):
    extension: str

    def __str__(self) -> str:
        return f"Missing PHP Extension: {self.extension}"


class MinimumAutoconfTooOld(Problem, kind="minimum-autoconf-too-old"):
    minimum_version: str

    def __str__(self) -> str:
        return (
            f"configure.{{ac,in}} should require newer autoconf {self.minimum_version}"
        )


class MissingPkgConfig(Problem, kind="missing-pkg-config-package"):
    module: str
    minimum_version: Optional[str] = None

    def __str__(self) -> str:
        if self.minimum_version:
            return f"Missing pkg-config file: {self.module} (>= {self.minimum_version})"
        else:
            return f"Missing pkg-config file: {self.module}"

    def __repr__(self) -> str:
        return f"{type(self).__name__}({self.module!r}, minimum_version={self.minimum_version!r})"


class MissingGoRuntime(Problem, kind="missing-go-runtime"):
    def __str__(self) -> str:
        return "go runtime is missing"


class MissingCMakeComponents(Problem, kind="missing-cmake-components"):
    name: str
    components: list[str]

    def __str__(self) -> str:
        return f"Missing {self.name} components: {self.components!r}"


class CMakeFilesMissing(Problem, kind="missing-cmake-files"):
    filenames: list[str]
    version: Optional[str] = None

    def __str__(self) -> str:
        if self.version:
            return f"Missing CMake package configuration files (version {self.version}): {self.filenames!r}"
        return f"Missing CMake package configuration files: {self.filenames!r}"


class MissingCMakeConfig(Problem, kind="missing-cmake-config"):
    name: str
    version: str

    def __str__(self) -> str:
        if self.version:
            return f"Missing CMake package configuration for {self.name} (version {self.version})"
        return f"Missing CMake package configuration for {self.name}"


class DhWithOrderIncorrect(Problem, kind="debhelper-argument-order"):
    def __str__(self) -> str:
        return "dh argument order is incorrect"


class UnsupportedDebhelperCompatLevel(
    Problem, kind="unsupported-debhelper-compat-level"
):
    oldest_supported: int
    requested: int

    def __str__(self) -> str:
        return "Request debhelper compat level %d lower than supported %d" % (
            self.requested,
            self.oldest_supported,
        )


class NoSpaceOnDevice(Problem, kind="no-space-on-device", is_global=True):
    def __str__(self) -> str:
        return "No space on device"


class MissingPerlPredeclared(Problem, kind="missing-perl-predeclared"):
    name: str

    def __str__(self) -> str:
        return f"missing predeclared function: {self.name}"


class MissingPerlDistributionFile(Problem, kind="missing-perl-distribution-file"):
    filename: str

    def __str__(self) -> str:
        return f"Missing perl distribution file: {self.filename}"


class InvalidCurrentUser(Problem, kind="invalid-current-user"):
    user: str

    def __str__(self) -> str:
        return f"Can not run as {self.user}"


class MissingPerlModule(Problem, kind="missing-perl-module"):
    filename: Optional[str]
    module: str
    inc: Optional[list[str]] = None
    minimum_version: Optional[str] = None

    def __str__(self) -> str:
        if self.filename:
            return f"Missing Perl module: {self.module} (filename: {self.filename!r})"
        else:
            return f"Missing Perl Module: {self.module}"


class MissingPerlFile(Problem, kind="missing-perl-file"):
    filename: str
    inc: Optional[list[str]] = None

    def __str__(self) -> str:
        return f"Missing Perl file: {self.filename} (inc: {self.inc!r})"


class MissingMavenArtifacts(Problem, kind="missing-maven-artifacts"):
    artifacts: list[tuple[str, str, str, str]]

    def __str__(self) -> str:
        return f"Missing maven artifacts: {self.artifacts!r}"

    def __repr__(self) -> str:
        return f"{type(self).__name__}({self.artifacts!r})"


class DhUntilUnsupported(Problem, kind="dh-until-unsupported"):
    def __str__(self) -> str:
        return "dh --until is no longer supported"


class DhAddonLoadFailure(Problem, kind="dh-addon-load-failure"):
    name: str
    path: str

    def __str__(self) -> str:
        return f"dh addon loading failed: {self.name}"


class DhMissingUninstalled(Problem, kind="dh-missing-uninstalled"):
    missing_file: str

    def __str__(self) -> str:
        return f"File built by Debian not installed: {self.missing_file!r}"


class DhLinkDestinationIsDirectory(Problem, kind="dh-link-destination-is-directory"):
    path: str

    def __str__(self) -> str:
        return f"Link destination {self.path} is directory"


class MissingXmlEntity(Problem, kind="missing-xml-entity"):
    url: str

    def __str__(self) -> str:
        return f"Missing XML entity: {self.url}"


class CcacheError(Problem, kind="ccache-error"):
    error: str

    def __str__(self) -> str:
        return f"ccache error: {self.error}"


class MissingDebianBuildDep(Problem, kind="missing-debian-build-dep"):
    dep: str

    def __str__(self) -> str:
        return f"Missing Debian Build-Depends: {self.dep}"


class MissingGoSumEntry(Problem, kind="missing-go.sum-entry"):
    package: str
    version: str

    def __str__(self) -> str:
        return f"Missing go.sum entry: {self.package}@{self.version}"


class MissingLibrary(Problem, kind="missing-library"):
    library: str

    def __str__(self) -> str:
        return f"missing library: {self.library}"


class MissingStaticLibrary(Problem, kind="missing-static-library"):
    library: str
    filename: str

    def __str__(self) -> str:
        return f"missing static library: {self.library}"


class MissingRubyGem(Problem, kind="missing-ruby-gem"):
    gem: str
    version: Optional[str] = None

    def __str__(self) -> str:
        if self.version:
            return f"missing ruby gem: {self.gem} (>= {self.version})"
        else:
            return f"missing ruby gem: {self.gem}"


class MissingRubyFile(Problem, kind="missing-ruby-file"):
    filename: str

    def __str__(self) -> str:
        return f"Missing ruby file: {self.filename}"


class MissingPhpClass(Problem, kind="missing-php-class"):
    php_class: str

    def __str__(self) -> str:
        return f"missing PHP class: {self.php_class}"


class MissingJavaClass(Problem, kind="missing-java-class"):
    classname: str

    def __str__(self) -> str:
        return f"missing java class: {self.classname}"


class MissingRPackage(Problem, kind="missing-r-package"):
    package: str
    minimum_version: Optional[str] = None

    def __str__(self) -> str:
        if self.minimum_version:
            return f"missing R package: {self.package} (>= {self.minimum_version})"
        else:
            return f"missing R package: {self.package}"


class DebhelperPatternNotFound(Problem, kind="debhelper-pattern-not-found"):
    pattern: str
    tool: str
    directories: list[str]

    def __str__(self) -> str:
        return f"debhelper ({self.tool}) expansion failed for {self.pattern!r} (directories: {self.directories!r})"


class GnomeCommonMissing(Problem, kind="missing-gnome-common"):
    def __str__(self) -> str:
        return "gnome-common is not installed"


class MissingXfceDependency(Problem, kind="missing-xfce-dependency"):
    package: str

    def __str__(self) -> str:
        return f"Missing XFCE build dependency: {self.package}"


class MissingAutomakeInput(Problem, kind="missing-automake-input"):
    path: str

    def __str__(self) -> str:
        return f"automake input file {self.path} missing"


class MissingAutoconfMacro(Problem, kind="missing-autoconf-macro"):
    macro: str
    need_rebuild: bool = False

    def __str__(self) -> str:
        return f"autoconf macro {self.macro} missing"


class MissingGnomeCommonDependency(Problem, kind="missing-gnome-common-dependency"):
    package: str
    minimum_version: Optional[str] = None

    def __str__(self) -> str:
        return f"Missing gnome-common dependency: {self.package}: (>= {self.minimum_version})"


class MissingConfigStatusInput(Problem, kind="missing-config.status-input"):
    path: str

    def __str__(self) -> str:
        return f"missing config.status input {self.path}"


class MissingJVM(Problem, kind="missing-jvm"):
    def __str__(self) -> str:
        return "Missing JVM"


class MissingPerlManifest(Problem, kind="missing-perl-manifest"):
    def __str__(self) -> str:
        return "missing Perl MANIFEST"


class UpstartFilePresent(Problem, kind="upstart-file-present"):
    filename: str

    def __str__(self) -> str:
        return f"Upstart file present: {self.filename}"


class NeedPgBuildExtUpdateControl(Problem, kind="need-pg-buildext-updatecontrol"):
    generated_path: str
    template_path: str

    def __str__(self) -> str:
        return f"Need to run 'pg_buildext updatecontrol' to update {self.generated_path}"


class MissingValaPackage(Problem, kind="missing-vala-package"):
    package: str

    def __str__(self) -> str:
        return f"Missing Vala package: {self.package}"


class DirectoryNonExistant(Problem, kind="local-directory-not-existing"):
    path: str

    def __str__(self) -> str:
        return f"Directory does not exist: {self.path}"


class ImageMagickDelegateMissing(Problem, kind="imagemagick-delegate-missing"):
    delegate: str

    def __str__(self) -> str:
        return f"Imagemagick missing delegate: {self.delegate}"


class DebianVersionRejected(Problem, kind="debian-version-rejected"):
    version: str

    def __str__(self) -> str:
        return f"Debian Version Rejected; {self.version}"


class ValaCompilerCannotCompile(Problem, kind="valac-cannot-compile"):
    def __str__(self) -> str:
        return "valac can not compile"


class MissingHaskellDependencies(Problem, kind="missing-haskell-dependencies"):
    deps: list[str]

    def __repr__(self) -> str:
        return f"{type(self).__name__}({self.deps!r})"

    def __str__(self) -> str:
        return f"Missing Haskell dependencies: {self.deps!r}"


class MissingHaskellModule(Problem, kind="missing-haskell-module"):
    module: str

    def __repr__(self) -> str:
        return f"{type(self).__name__}({self.module!r})"

    def __str__(self) -> str:
        return f"Missing Haskell module: {self.module!r}"


class Matcher:
    def match(
        self, line: list[str], i: int
    ) -> tuple[list[int], Optional[Problem], str]:
        raise NotImplementedError(self.match)


class MatcherError(Exception):
    """Error during matching."""


class SingleLineMatcher(Matcher):
    def __init__(self, regexp, cb=None) -> None:
        self.regexp = re.compile(regexp)
        self.cb = cb

    def __repr__(self) -> str:
        return f"<{type(self).__name__}({self.regexp.pattern!r})>"

    def match(self, lines, i):
        m = self.regexp.match(lines[i].rstrip("\n"))
        if not m:
            return [], None, None
        if self.cb:
            try:
                err = self.cb(m)
            except (ValueError, IndexError) as e:
                raise MatcherError(
                    f"Error while matching {self.regexp!r} against {lines[i]!r} ({m!r}): {e!r}"
                ) from e
        else:
            err = None
        return [i], err, f"direct regex ({self.regexp.pattern}"


class MissingSetupPyCommand(Problem, kind="missing-setup.py-command"):
    command: str

    def __str__(self) -> str:
        return f"missing setup.py subcommand: {self.command}"


class CMakeNeedExactVersion(Problem, kind="cmake-exact-version-missing"):
    package: str
    version_found: str
    exact_version_needed: str
    path: str

    def __repr__(self) -> str:
        return f"{type(self).__name__}({self.package!r}, {self.version_found!r}, {self.exact_version_needed!r}, {self.path!r})"

    def __str__(self) -> str:
        return f"CMake needs exact package {self.package}, version {self.exact_version_needed}"


class MissingFortranCompiler(Problem, kind="missing-fortran-compiler"):
    def __str__(self) -> str:
        return "No Fortran compiler found"


class MissingCSharpCompiler(Problem, kind="missing-c#-compiler"):
    def __str__(self) -> str:
        return "No C# compiler found"


class MissingRustCompiler(Problem, kind="missing-rust-compiler"):
    def __str__(self) -> str:
        return "No Rust compiler found"


class MissingAssembler(Problem, kind="missing-assembler"):
    def __str__(self) -> str:
        return "No assembler found"


class MissingLibtool(Problem, kind="missing-libtool"):
    def __str__(self) -> str:
        return "Libtool is missing"


class UnsupportedPytestArguments(Problem, kind="unsupported-pytest-arguments"):
    args: list[str]

    def __str__(self) -> str:
        return f"Unsupported pytest arguments: {self.args!r}"

    def __repr__(self) -> str:
        return f"{type(self).__name__}({self.args!r})"


class UnsupportedPytestConfigOption(Problem, kind="unsupported-pytest-config-option"):
    name: str

    def __str__(self) -> str:
        return f"Unsupported pytest configuration option: {self.name}"


class MissingPytestFixture(Problem, kind="missing-pytest-fixture"):
    fixture: str

    def __str__(self) -> str:
        return f"Missing pytest fixture: {self.fixture}"

    def __repr__(self) -> str:
        return f"{type(self).__name__}({self.fixture!r})"


class MissingCargoCrate(Problem, kind="missing-cargo-crate"):
    crate: str
    requirement: Optional[str] = None

    def __str__(self) -> str:
        if self.requirement:
            return f"Missing crate: {self.crate} ({self.requirement})"
        else:
            return f"Missing crate: {self.crate}"


class MissingLatexFile(Problem, kind="missing-latex-file"):
    filename: str

    def __str__(self) -> str:
        return f"Missing LaTeX file: {self.filename}"


class MissingFontspec(Problem, kind="missing-fontspec"):
    fontspec: str

    def __str__(self) -> str:
        return f"Missing font spec: {self.fontspec}"


class MissingDHCompatLevel(Problem, kind="missing-dh-compat-level"):
    command: str

    def __str__(self) -> str:
        return f"Missing DH Compat Level (command: {self.command})"


class DuplicateDHCompatLevel(Problem, kind="duplicate-dh-compat-level"):
    command: str

    def __str__(self) -> str:
        return f"DH Compat Level specified twice (command: {self.command})"


class MissingIntrospectionTypelib(Problem, kind="missing-introspection-typelib"):
    library: str

    def __str__(self) -> str:
        return f"Missing introspection typelib: {self.library}"


class UnknownCertificateAuthority(Problem, kind="unknown-certificate-authority"):
    url: str

    def __str__(self) -> str:
        return f"Unknown Certificate Authority for {self.url}"


class MissingXDisplay(Problem, kind="missing-x-display"):
    def __str__(self) -> str:
        return "No X Display"


class MissingPostgresExtension(Problem, kind="missing-postgresql-extension"):
    extension: str

    def __str__(self) -> str:
        return f"Missing postgres extension: {self.extension}"


class MissingLuaModule(Problem, kind="missing-lua-module"):
    module: str

    def __str__(self) -> str:
        return f"Missing Lua Module: {self.module}"


class Cancelled(Problem, kind="cancelled"):
    def __str__(self) -> str:
        return "Cancelled by runner or job manager"


class InactiveKilled(Problem, kind="inactive-killed"):
    minutes: int

    def __str__(self) -> str:
        return "Killed due to inactivity"


class MissingPauseCredentials(Problem, kind="missing-pause-credentials"):
    def __str__(self) -> str:
        return "Missing credentials for PAUSE"


class MismatchGettextVersions(Problem, kind="mismatch-gettext-versions"):
    makefile_version: str
    autoconf_version: str

    def __str__(self) -> str:
        return f"Mismatch versions ({self.makefile_version}, {self.autoconf_version})"


class DisappearedSymbols(Problem, kind="disappeared-symbols"):
    def __str__(self) -> str:
        return "Disappeared symbols"


class MissingGnulibDirectory(Problem, kind="missing-gnulib-directory"):
    directory: str

    def __str__(self) -> str:
        return f"Missing gnulib directory {self.directory}"


class MissingGoModFile(Problem, kind="missing-go.mod-file"):
    def __str__(self) -> str:
        return "go.mod file is missing"


class OutdatedGoModFile(Problem, kind="outdated-go.mod-file"):
    def __str__(self) -> str:
        return "go.mod file is outdated"


class CodeCoverageTooLow(Problem, kind="code-coverage-too-low"):
    actual: float
    required: float

    def __str__(self) -> str:
        return f"Code coverage too low: {self.actual:f} < {self.required:f}"


class ESModuleMustUseImport(Problem, kind="esmodule-must-use-import"):
    path: str

    def __str__(self) -> str:
        return f"ESM-only module {self.path} must use import()"


find_secondary_build_failure = _buildlog_consultant_rs.find_secondary_build_failure
find_build_failure_description = _buildlog_consultant_rs.find_build_failure_description
