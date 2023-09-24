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
import posixpath
import re
import textwrap
from typing import Optional, cast

from . import (
    Match,
    MultiLineMatch,
    Problem,
    SingleLineMatch,
    version_string,
)
from . import _buildlog_consultant_rs  # type: ignore

logger = logging.getLogger(__name__)


class MissingPythonModule(Problem, kind="missing-python-module"):

    module: str
    python_version: Optional[str] = None
    minimum_version: Optional[str] = None

    def __str__(self) -> str:
        if self.python_version:
            ret = "Missing python %s module: " % self.python_version
        else:
            ret = "Missing python module: "
        ret += self.module
        if self.minimum_version:
            return ret + " (>= %s)" % self.minimum_version
        else:
            return ret

    def __repr__(self) -> str:
        return "{}({!r}, python_version={!r}, minimum_version={!r})".format(
            type(self).__name__,
            self.module,
            self.python_version,
            self.minimum_version,
        )


class SetuptoolScmVersionIssue(Problem, kind="setuptools-scm-version-issue"):

    def __str__(self) -> str:
        return "setuptools-scm was unable to find version"


class MissingOCamlPackage(Problem, kind='missing-ocaml-package'):

    package: str

    def __str__(self) -> str:
        return "Missing OCaml package: %s" % self.package


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
            return ret + " (>= %s)" % self.minimum_version
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
        return "{}({!r}, python_version={!r}, minimum_version={!r})".format(
            type(self).__name__,
            self.distribution,
            self.python_version,
            self.minimum_version,
        )


class VcsControlDirectoryNeeded(Problem, kind='vcs-control-directory-needed'):

    vcs: list[str]

    def __str__(self) -> str:
        return "Version control directory needed"


class PatchApplicationFailed(Problem, kind="patch-application-failed"):

    patchname: str

    def __str__(self) -> str:
        return "Patch application failed: %s" % self.patchname


class MissingVagueDependency(Problem, kind="missing-vague-dependency"):

    name: str
    url: Optional[str] = None
    minimum_version: Optional[str] = None
    current_version: Optional[str] = None

    def __repr__(self) -> str:
        return "{}({!r}, url={!r}, minimum_version={!r}, current_version={!r})".format(
            type(self).__name__, self.name,
            self.url, self.minimum_version, self.current_version)

    def __str__(self) -> str:
        return "Missing dependency: %s" % self.name


class MissingQt(Problem, kind="missing-qt"):

    minimum_version: Optional[str] = None

    def __str__(self) -> str:
        if self.minimum_version:
            return "Missing QT installation (at least %s)" % (
                self.minimum_version)
        return "Missing QT installation"


class MissingQtModules(Problem, kind="missing-qt-modules"):

    modules: list[str]

    def __str__(self) -> str:
        return "Missing QT modules: %r" % self.modules


class MissingX11(Problem, kind="missing-x11"):
    def __str__(self) -> str:
        return "Missing X11 headers"


class MissingGitIdentity(Problem, kind="missing-git-identity"):
    def __str__(self) -> str:
        return "Missing Git Identity"


class MissingFile(Problem, kind="missing-file"):

    path: str

    def __str__(self) -> str:
        return "Missing file: %s" % self.path


class MissingCommandOrBuildFile(Problem, kind="missing-command-or-build-file"):

    filename: str

    @property
    def command(self):
        return self.filename

    def __str__(self) -> str:
        return "Missing command or build file: %s" % self.filename


class MissingBuildFile(Problem, kind="missing-build-file"):

    filename: str

    def __str__(self) -> str:
        return "Missing build file: %s" % self.filename


def file_not_found(m):
    if m.group(1).startswith("/") and not m.group(1).startswith("/<<PKGBUILDDIR>>"):
        return MissingFile(m.group(1))
    elif m.group(1).startswith("/<<PKGBUILDDIR>>/"):
        return MissingBuildFile(m.group(1)[len("/<<PKGBUILDDIR>>/"):])
    if m.group(1) == '.git/HEAD':
        return VcsControlDirectoryNeeded(['git'])
    if m.group(1) == 'CVS/Root':
        return VcsControlDirectoryNeeded(['cvs'])
    if '/' not in m.group(1):
        # Maybe a missing command?
        return MissingBuildFile(m.group(1))
    return None


def file_not_found_maybe_executable(m):
    if m.group(1).startswith("/") and not m.group(1).startswith("/<<PKGBUILDDIR>>"):
        return MissingFile(m.group(1))
    if '/' not in m.group(1):
        # Maybe a missing command?
        return MissingCommandOrBuildFile(m.group(1))
    return None


def webpack_file_missing(m):
    path = posixpath.join(m.group(2), m.group(1))
    if path.startswith("/") and not path.startswith("/<<PKGBUILDDIR>>"):
        return MissingFile(path)
    return None


class MissingJDKFile(Problem, kind="missing-jdk-file"):

    jdk_path: str
    filename: str

    def __str__(self) -> str:
        return f"Missing JDK file {self.filename} (JDK Path: {self.jdk_path})"


class MissingJDK(Problem, kind="missing-jdk"):

    jdk_path: str

    def __str__(self) -> str:
        return "Missing JDK (JDK Path: %s)" % (self.jdk_path)


class MissingJRE(Problem, kind="missing-jre"):
    def __str__(self) -> str:
        return "Missing JRE"


def interpreter_missing(m):
    if m.group(1).startswith("/"):
        if m.group(1).startswith("/<<PKGBUILDDIR>>"):
            return None
        return MissingFile(m.group(1))
    if "/" in m.group(1):
        return None
    return MissingCommand(m.group(1))


class ChrootNotFound(Problem, kind="chroot-not-found"):

    chroot: str

    def __str__(self) -> str:
        return "Chroot not found: %s" % self.chroot


class MissingSprocketsFile(Problem, kind="missing-sprockets-file"):

    name: str
    content_type: str

    def __str__(self) -> str:
        return f"Missing sprockets file: {self.name} (type: {self.content_type})"


class MissingGoPackage(Problem, kind="missing-go-package"):

    package: str

    def __str__(self) -> str:
        return "Missing Go package: %s" % self.package


class MissingCHeader(Problem, kind="missing-c-header"):

    header: str

    def __str__(self) -> str:
        return "Missing C Header: %s" % self.header


class MissingNodeModule(Problem, kind="missing-node-module"):

    module: str

    def __str__(self) -> str:
        return "Missing Node Module: %s" % self.module


class MissingNodePackage(Problem, kind="missing-node-package"):

    package: str

    def __str__(self) -> str:
        return "Missing Node Package: %s" % self.package


def node_module_missing(m):
    if m.group(1).startswith("/<<PKGBUILDDIR>>/"):
        return None
    if m.group(1).startswith("./"):
        return None
    return MissingNodeModule(m.group(1))


class MissingCommand(Problem, kind="command-missing"):

    command: str

    def __str__(self) -> str:
        return "Missing command: %s" % self.command

    def __repr__(self) -> str:
        return f"{type(self).__name__}({self.command!r})"


class NotExecutableFile(Problem, kind="command-not-executable"):

    path: str

    def __str__(self) -> str:
        return "Command not executable: %s" % self.path


class MissingSecretGpgKey(Problem, kind="no-secret-gpg-key"):
    def __str__(self) -> str:
        return "No secret GPG key is present"


class MissingVcVersionerVersion(Problem, kind="no-vcversioner-version"):
    def __str__(self) -> str:
        return "vcversion could not find a git directory or version.txt file"


class MissingConfigure(Problem, kind="missing-configure"):
    def __str__(self) -> str:
        return "Missing configure script"


def command_missing(m):
    command = m.group(1)
    if "PKGBUILDDIR" in command:
        return None
    if command == "./configure":
        return MissingConfigure()
    if command.startswith("./") or command.startswith("../"):
        return None
    if command == "debian/rules":
        return None
    return MissingCommand(command)


class MissingJavaScriptRuntime(Problem, kind="javascript-runtime-missing"):
    def __str__(self) -> str:
        return "Missing JavaScript Runtime"


class MissingPHPExtension(Problem, kind="missing-php-extension"):

    extension: str

    def __str__(self) -> str:
        return "Missing PHP Extension: %s" % self.extension


class MinimumAutoconfTooOld(Problem, kind="minimum-autoconf-too-old"):

    minimum_version: str

    def __str__(self) -> str:
        return "configure.{ac,in} should require newer autoconf %s" % self.minimum_version


class MissingPkgConfig(Problem, kind="missing-pkg-config-package"):

    module: str
    minimum_version: Optional[str] = None

    def __str__(self) -> str:
        if self.minimum_version:
            return "Missing pkg-config file: {} (>= {})".format(
                self.module,
                self.minimum_version,
            )
        else:
            return "Missing pkg-config file: %s" % self.module

    def __repr__(self) -> str:
        return "{}({!r}, minimum_version={!r})".format(
            type(self).__name__,
            self.module,
            self.minimum_version,
        )


class MissingGoRuntime(Problem, kind="missing-go-runtime"):
    def __str__(self) -> str:
        return "go runtime is missing"


def pkg_config_missing(m):
    expr = m.group(1).strip().split("\t")[0]
    if ">=" in expr:
        pkg, minimum = expr.split(">=", 1)
        return MissingPkgConfig(pkg.strip(), minimum.strip())
    if " " not in expr:
        return MissingPkgConfig(expr)
    # Hmm
    return None


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


class UnsupportedDebhelperCompatLevel(Problem, kind="unsupported-debhelper-compat-level"):

    oldest_supported: int
    requested: int

    def __str__(self) -> str:
        return "Request debhelper compat level %d lower than supported %d" % (
            self.requested, self.oldest_supported)


class NoSpaceOnDevice(Problem, kind="no-space-on-device", is_global=True):
    def __str__(self) -> str:
        return "No space on device"


class MissingPerlPredeclared(Problem, kind="missing-perl-predeclared"):

    name: str

    def __str__(self) -> str:
        return "missing predeclared function: %s" % self.name


class MissingPerlDistributionFile(Problem, kind="missing-perl-distribution-file"):

    filename: str

    def __str__(self) -> str:
        return "Missing perl distribution file: %s" % self.filename


class InvalidCurrentUser(Problem, kind="invalid-current-user"):

    user: str

    def __str__(self) -> str:
        return "Can not run as %s" % self.user


class MissingPerlModule(Problem, kind="missing-perl-module"):

    filename: Optional[str]
    module: str
    inc: Optional[list[str]] = None
    minimum_version: Optional[str] = None

    def __str__(self) -> str:
        if self.filename:
            return "Missing Perl module: {} (filename: {!r})".format(
                self.module,
                self.filename,
            )
        else:
            return "Missing Perl Module: %s" % self.module


class MissingPerlFile(Problem, kind="missing-perl-file"):

    filename: str
    inc: Optional[list[str]] = None

    def __str__(self) -> str:
        return f"Missing Perl file: {self.filename} (inc: {self.inc!r})"


class MissingMavenArtifacts(Problem, kind="missing-maven-artifacts"):

    artifacts: list[tuple[str, str, str, str]]

    def __str__(self) -> str:
        return "Missing maven artifacts: %r" % self.artifacts

    def __repr__(self) -> str:
        return f"{type(self).__name__}({self.artifacts!r})"


class DhUntilUnsupported(Problem, kind="dh-until-unsupported"):
    def __str__(self) -> str:
        return "dh --until is no longer supported"


class DhAddonLoadFailure(Problem, kind="dh-addon-load-failure"):

    name: str
    path: str

    def __str__(self) -> str:
        return "dh addon loading failed: %s" % self.name


class DhMissingUninstalled(Problem, kind="dh-missing-uninstalled"):

    missing_file: str

    def __str__(self) -> str:
        return "File built by Debian not installed: %r" % self.missing_file


class DhLinkDestinationIsDirectory(Problem, kind="dh-link-destination-is-directory"):

    path: str

    def __str__(self) -> str:
        return "Link destination %s is directory" % self.path


class MissingXmlEntity(Problem, kind="missing-xml-entity"):

    url: str

    def __str__(self) -> str:
        return "Missing XML entity: %s" % self.url


class CcacheError(Problem, kind="ccache-error"):

    error: str

    def __str__(self) -> str:
        return "ccache error: %s" % self.error


class MissingDebianBuildDep(Problem, kind='missing-debian-build-dep'):

    dep: str

    def __str__(self) -> str:
        return f"Missing Debian Build-Depends: {self.dep}"


class MissingGoSumEntry(Problem, kind="missing-go.sum-entry"):

    package: str
    version: str

    def __str__(self) -> str:
        return "Missing go.sum entry: {}@{}".format(
            self.package, self.version)


class MissingLibrary(Problem, kind="missing-library"):

    library: str

    def __str__(self) -> str:
        return "missing library: %s" % self.library


class MissingStaticLibrary(Problem, kind="missing-static-library"):

    library: str
    filename: str

    def __str__(self) -> str:
        return "missing static library: %s" % self.library


class MissingRubyGem(Problem, kind="missing-ruby-gem"):

    gem: str
    version: Optional[str] = None

    def __str__(self) -> str:
        if self.version:
            return f"missing ruby gem: {self.gem} (>= {self.version})"
        else:
            return "missing ruby gem: %s" % self.gem


class MissingRubyFile(Problem, kind="missing-ruby-file"):

    filename: str

    def __str__(self) -> str:
        return f"Missing ruby file: {self.filename}"


class MissingPhpClass(Problem, kind="missing-php-class"):

    php_class: str

    def __str__(self) -> str:
        return "missing PHP class: %s" % self.php_class


class MissingJavaClass(Problem, kind="missing-java-class"):

    classname: str

    def __str__(self) -> str:
        return "missing java class: %s" % self.classname


class MissingRPackage(Problem, kind="missing-r-package"):

    package: str
    minimum_version: Optional[str] = None

    def __str__(self) -> str:
        if self.minimum_version:
            return "missing R package: {} (>= {})".format(
                self.package,
                self.minimum_version,
            )
        else:
            return "missing R package: %s" % self.package


def r_missing_package(m):
    fragment = m.group(1)
    deps = [dep.strip("‘’' ") for dep in fragment.split(",")]
    return MissingRPackage(deps[0])


class DebhelperPatternNotFound(Problem, kind="debhelper-pattern-not-found"):

    pattern: str
    tool: str
    directories: list[str]

    def __str__(self) -> str:
        return "debhelper ({}) expansion failed for {!r} (directories: {!r})".format(
            self.tool,
            self.pattern,
            self.directories,
        )


class GnomeCommonMissing(Problem, kind="missing-gnome-common"):
    def __str__(self) -> str:
        return "gnome-common is not installed"


class MissingXfceDependency(Problem, kind="missing-xfce-dependency"):

    package: str

    def __str__(self) -> str:
        return "Missing XFCE build dependency: %s" % (self.package)


class MissingAutomakeInput(Problem, kind="missing-automake-input"):

    path: str

    def __str__(self) -> str:
        return "automake input file %s missing" % self.path


class MissingAutoconfMacro(Problem, kind="missing-autoconf-macro"):

    macro: str
    need_rebuild: bool = False

    def __str__(self) -> str:
        return "autoconf macro %s missing" % self.macro


class MissingGnomeCommonDependency(Problem, kind="missing-gnome-common-dependency"):

    package: str
    minimum_version: Optional[str] = None

    def __str__(self) -> str:
        return "Missing gnome-common dependency: {}: (>= {})".format(
            self.package,
            self.minimum_version,
        )


class MissingConfigStatusInput(Problem, kind="missing-config.status-input"):

    path: str

    def __str__(self) -> str:
        return "missing config.status input %s" % self.path


class MissingJVM(Problem, kind="missing-jvm"):
    def __str__(self) -> str:
        return "Missing JVM"


class MissingPerlManifest(Problem, kind="missing-perl-manifest"):

    def __str__(self) -> str:
        return "missing Perl MANIFEST"


class UpstartFilePresent(Problem, kind="upstart-file-present"):

    filename: str

    def __str__(self) -> str:
        return "Upstart file present: %s" % self.filename


class NeedPgBuildExtUpdateControl(Problem, kind="need-pg-buildext-updatecontrol"):

    generated_path: str
    template_path: str

    def __str__(self) -> str:
        return "Need to run 'pg_buildext updatecontrol' to update %s" % (
            self.generated_path
        )


class MissingValaPackage(Problem, kind="missing-vala-package"):

    package: str

    def __str__(self) -> str:
        return "Missing Vala package: %s" % self.package


class DirectoryNonExistant(Problem, kind="local-directory-not-existing"):

    path: str

    def __str__(self) -> str:
        return "Directory does not exist: %s" % self.path


class ImageMagickDelegateMissing(Problem, kind="imagemagick-delegate-missing"):

    delegate: str

    def __str__(self) -> str:
        return "Imagemagick missing delegate: %s" % self.delegate


class DebianVersionRejected(Problem, kind="debian-version-rejected"):

    version: str

    def __str__(self) -> str:
        return "Debian Version Rejected; %s" % self.version


class ValaCompilerCannotCompile(Problem, kind="valac-cannot-compile"):

    def __str__(self) -> str:
        return "valac can not compile"


class MissingHaskellDependencies(Problem, kind="missing-haskell-dependencies"):

    deps: list[str]

    def __repr__(self) -> str:
        return f"{type(self).__name__}({self.deps!r})"

    def __str__(self) -> str:
        return "Missing Haskell dependencies: %r" % self.deps


class MissingHaskellModule(Problem, kind="missing-haskell-module"):

    module: str

    def __repr__(self) -> str:
        return f"{type(self).__name__}({self.module!r})"

    def __str__(self) -> str:
        return "Missing Haskell module: %r" % self.module


class Matcher:
    def match(self, line: list[str], i: int) -> tuple[list[int], Optional[Problem], str]:
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
                    "Error while matching {!r} against {!r} ({!r}): {!r}".format(
                        self.regexp, lines[i], m, e)) from e
        else:
            err = None
        return [i], err, f"direct regex ({self.regexp.pattern}"


class MissingSetupPyCommand(Problem, kind="missing-setup.py-command"):

    command: str

    def __str__(self) -> str:
        return "missing setup.py subcommand: %s" % self.command


class PythonFileNotFoundErrorMatcher(Matcher):

    final_line_re = re.compile(
        r"^(?:E  +)?FileNotFoundError: \[Errno 2\] "
        r"No such file or directory: \'(.*)\'"
    )

    def match(self, lines, i):
        m = self.final_line_re.fullmatch(lines[i].rstrip("\n"))
        if not m:
            return [], None, None
        if i - 2 >= 0 and "subprocess" in lines[i - 2]:
            return [i], MissingCommand(m.group(1)), f"direct regex ({self.final_line_re.pattern})"
        return [i], file_not_found_maybe_executable(m), None


def cmake_compiler_failure(m):
    compiler_output = textwrap.dedent(m.group(3))
    match, error = find_build_failure_description(compiler_output.splitlines(True))
    return error


def cmake_compiler_missing(m):
    if m.group(1) == "Fortran":
        return MissingFortranCompiler()
    return None


class CMakeNeedExactVersion(Problem, kind="cmake-exact-version-missing"):

    package: str
    version_found: str
    exact_version_needed: str
    path: str

    def __repr__(self) -> str:
        return "{}({!r}, {!r}, {!r}, {!r})".format(
            type(self).__name__,
            self.package,
            self.version_found,
            self.exact_version_needed,
            self.path,
        )

    def __str__(self) -> str:
        return "CMake needs exact package {}, version {}".format(
            self.package,
            self.exact_version_needed,
        )


class CMakeErrorMatcher(Matcher):

    regexp = re.compile(r"CMake (Error|Warning) at (.+):([0-9]+) \((.*)\):")

    cmake_errors = [
        (
            r'Could NOT find (.*) \(missing:\s(.*)\)\s\(found\ssuitable\sversion\s.*',
            lambda m: MissingCMakeComponents(m.group(1), m.group(2).split())),
        (
            r"\s*--\s+Package \'(.*)\', required by \'(.*)\', not found",
            lambda m: MissingPkgConfig(m.group(1)),
        ),
        (
            r'Could not find a package configuration file provided by\s'
            r'"(.*)" \(requested\sversion\s(.*)\)\swith\sany\s+of\s+the\s+following\snames:'
            r'\n\n(  .*\n)+\n.*$',
            lambda m: CMakeFilesMissing(
                [e.strip() for e in m.group(3).splitlines()], m.group(2))
        ),
        (
            r"Could NOT find (.*) \(missing: (.*)\)",
            lambda m: MissingCMakeComponents(m.group(1), m.group(2).split()),
        ),
        (
            r'The (.+) compiler\n\n  "(.*)"\n\nis not able to compile a '
            r"simple test program\.\n\nIt fails with the following output:\n\n"
            r"(.*)\n\n"
            r"CMake will not be able to correctly generate this project.\n$",
            cmake_compiler_failure,
        ),
        (
            r"Could NOT find (.*): Found unsuitable version \"(.*)\",\sbut\s"
            r"required\sis\sexact version \"(.*)\" \(found\s(.*)\)",
            lambda m: CMakeNeedExactVersion(
                m.group(1), m.group(2), m.group(3), m.group(4)
            ),
        ),
        (
            r"(.*) couldn't be found \(missing: .*_LIBRARIES .*_INCLUDE_DIR\)",
            lambda m: MissingVagueDependency(m.group(1)),
        ),
        (
            r"Could NOT find (.*): Found unsuitable version \"(.*)\",\sbut\s"
            r"required\sis\sat\sleast\s\"(.*)\" \(found\s(.*)\)",
            lambda m: MissingPkgConfig(m.group(1), m.group(3)),
        ),
        (
            r'The imported target \"(.*)\" references the file\n\n\s*"(.*)"\n\n'
            r"but this file does not exist\.(.*)",
            lambda m: MissingFile(m.group(2)),
        ),
        (
            r'Could not find a configuration file for package "(.*)"\sthat\sis\s'
            r'compatible\swith\srequested\sversion\s"(.*)"\.',
            lambda m: MissingCMakeConfig(m.group(1), m.group(2)),
        ),
        (
            r'.*Could not find a package configuration file provided by "(.*)"\s+'
            r"with\s+any\s+of\s+the\s+following\s+names:\n\n(  .*\n)+\n.*$",
            lambda m: CMakeFilesMissing([e.strip() for e in m.group(2).splitlines()])
        ),
        (
            r'.*Could not find a package configuration file provided by "(.*)"\s'
            r"\(requested\sversion\s(.+\))\swith\sany\sof\sthe\sfollowing\snames:\n"
            r"\n(  .*\n)+\n.*$",
            lambda m: CMakeFilesMissing([e.strip() for e in m.group(3).splitlines()], m.group(2)),
        ),
        (
            r"No CMAKE_(.*)_COMPILER could be found.\n"
            r"\n"
            r"Tell CMake where to find the compiler by setting either"
            r'\sthe\senvironment\svariable\s"(.*)"\sor\sthe\sCMake\scache'
            r"\sentry\sCMAKE_(.*)_COMPILER\sto\sthe\sfull\spath\sto"
            r"\sthe\scompiler,\sor\sto\sthe\scompiler\sname\sif\sit\sis\sin\s"
            r"the\sPATH.\n",
            lambda m: MissingCommand(m.group(1).lower()),
        ),
        (r'file INSTALL cannot find\s"(.*)".\n', lambda m: MissingFile(m.group(1))),
        (
            r'file INSTALL cannot copy file\n"(.*)"\sto\s"(.*)":\s'
            r"No space left on device.\n",
            lambda m: NoSpaceOnDevice(),
        ),
        (
            r"patch: \*\*\*\* write error : No space left on device",
            lambda m: NoSpaceOnDevice(),
        ),
        (
            r".*\(No space left on device\)",
            lambda m: NoSpaceOnDevice(),
        ),
        (r'file INSTALL cannot copy file\n"(.*)"\nto\n"(.*)"\.\n', None),
        (
            r"Missing (.*)\.  Either your\n"
            r"lib(.*) version is too old, or lib(.*) wasn\'t found in the place you\n"
            r"said.",
            lambda m: MissingLibrary(m.group(1)),
        ),
        (
            r"need (.*) of version (.*)",
            lambda m: MissingVagueDependency(
                m.group(1), minimum_version=m.group(2).strip()
            ),
        ),
        (
            r"\*\*\* (.*) is required to build (.*)\n",
            lambda m: MissingVagueDependency(m.group(1)),
        ),
        (r"\[([^ ]+)\] not found", lambda m: MissingVagueDependency(m.group(1))),
        (r"([^ ]+) not found", lambda m: MissingVagueDependency(m.group(1))),
        (r"error: could not find git .*", lambda m: MissingCommand("git")),
        (r'Could not find \'(.*)\' executable[\!,].*', lambda m: MissingCommand(m.group(1))),
        (r'Could not find (.*)_STATIC_LIBRARIES using the following names: ([a-zA-z0-9_.]+)',
         lambda m: MissingStaticLibrary(m.group(1), m.group(2))),
        ('include could not find (requested|load) file:\n\n  (.*)\n', lambda m: CMakeFilesMissing([m.group(2) + '.cmake' if not m.group(2).endswith('.cmake') else m.group(2)])),
        (r'(.*) and (.*) are required', lambda m: MissingVagueDependency(m.group(1))),
        (r'Please check your (.*) installation', lambda m: MissingVagueDependency(m.group(1))),
        (r'Python module (.*) not found\!', lambda m: MissingPythonModule(m.group(1))),
        (r'\s*could not find ([^\s]+)$', lambda m: MissingVagueDependency(m.group(1))),
        (r'Please install (.*) before installing (.*)\.',
         lambda m: MissingVagueDependency(m.group(1))),
        (r"Please get (.*) from (www\..*)",
         lambda m: MissingVagueDependency(m.group(1), url=m.group(2))),
        (r'Found unsuitable Qt version "" from NOTFOUND, '
         r'this code requires Qt 4.x', lambda m: MissingQt('4')),
        (r'(.*) executable not found\! Please install (.*)\.',
         lambda m: MissingCommand(m.group(2))),
        (r'(.*) tool not found', lambda m: MissingCommand(m.group(1))),
        (r'--   Requested \'(.*) >= (.*)\' but version of (.*) is (.*)',
         lambda m: MissingPkgConfig(m.group(1), m.group(2))),
        (r'--   No package \'(.*)\' found',
         lambda m: MissingPkgConfig(m.group(1))),
        (r'([^ ]+) library not found\.',
         lambda m: MissingLibrary(m.group(1))),
        (r'Please install (.*) so that it is on the PATH and try again\.',
         command_missing),
        (r'-- Unable to find git\.  Setting git revision to \'unknown\'\.',
         lambda m: MissingCommand('git')),
        (r'(.*) must be installed before configuration \& building can '
         r'proceed', lambda m: MissingVagueDependency(m.group(1))),
        (r'(.*) development files not found\.',
         lambda m: MissingVagueDependency(m.group(1))),
        (r'.* but no (.*) dev libraries found',
         lambda m: MissingVagueDependency(m.group(1))),
        (r'Failed to find (.*) \(missing: .*\)',
         lambda m: MissingVagueDependency(m.group(1))),
        (r'Couldn\'t find ([^ ]+) development files\..*',
         lambda m: MissingVagueDependency(m.group(1))),
        (r'Could not find required (.*) package\!',
         lambda m: MissingVagueDependency(m.group(1))),
        (r'Cannot find (.*), giving up\. .*',
         lambda m: MissingVagueDependency(m.group(1))),
        (r'Cannot find (.*)\. (.*) is required for (.*)',
         lambda m: MissingVagueDependency(m.group(1))),
        (r'The development\sfiles\sfor\s(.*)\sare\s'
         r'required\sto\sbuild (.*)\.',
         lambda m: MissingVagueDependency(m.group(1))),
        (r'Required library (.*) not found\.',
         lambda m: MissingVagueDependency(m.group(1))),
        (r'(.*) required to compile (.*)',
         lambda m: MissingVagueDependency(m.group(1))),
        (r'(.*) requires (.*) ([0-9].*) or newer. See (https://.*)\s*',
         lambda m: MissingVagueDependency(m.group(2), minimum_version=m.group(3), url=m.group(4))),
        (r'(.*) requires (.*) ([0-9].*) or newer.\s*',
         lambda m: MissingVagueDependency(m.group(2), minimum_version=m.group(3))),
        (r'(.*) requires (.*) to build',
         lambda m: MissingVagueDependency(m.group(2))),
        (r'(.*) library missing',
         lambda m: MissingVagueDependency(m.group(1))),
        (r'(.*) requires (.*)',
         lambda m: MissingVagueDependency(m.group(2))),
        (r'Could not find ([A-Za-z-]+)',
         lambda m: MissingVagueDependency(m.group(1))),
        (r'(.+) is required for (.*)\.',
         lambda m: MissingVagueDependency(m.group(1))),
        (r'No (.+) version could be found in your system\.',
         lambda m: MissingVagueDependency(m.group(1))),
        (r'([^ ]+) >= (.*) is required',
         lambda m: MissingVagueDependency(m.group(1), minimum_version=m.group(2))),
        (r'\s*([^ ]+) is required',
         lambda m: MissingVagueDependency(m.group(1))),
        (r'([^ ]+) binary not found\!',
         lambda m: MissingCommand(m.group(1))),
        (r'error: could not find git for clone of .*',
         lambda m: MissingCommand('git')),
        (r'Did not find ([^\s]+)',
         lambda m: MissingVagueDependency(m.group(1))),
        (r'Could not find the ([^ ]+) external dependency\.',
         lambda m: MissingVagueDependency(m.group(1))),
        (r'Couldn\'t find (.*)', lambda m: MissingVagueDependency(m.group(1))),
    ]

    @classmethod
    def _extract_error_lines(cls, lines, i):
        linenos = [i]
        error_lines = []
        for j, line in enumerate(lines[i + 1 :]):
            if line.rstrip('\n') and not line.startswith(" "):
                break
            error_lines.append(line.rstrip('\n') + '\n')
            linenos.append(i + 1 + j)
        while error_lines and error_lines[-1].rstrip('\n') == "":
            error_lines.pop(-1)
            linenos.pop(-1)
        return linenos, textwrap.dedent("".join(error_lines)).splitlines(True)

    def match(self, lines, i):
        m = self.regexp.fullmatch(lines[i].rstrip("\n"))
        if not m:
            return [], None, None

        path = m.group(2)  # noqa: F841
        start_lineno = int(m.group(3))  # noqa: F841
        linenos, error_lines = self._extract_error_lines(lines, i)

        for r, fn in self.cmake_errors:
            m = re.match(r, "".join(error_lines), flags=re.DOTALL)
            if m:
                if fn is None:
                    error = None
                else:
                    error = fn(m)
                return linenos, error, f"direct regex ({self.regexp.pattern})"

        return linenos, None, f"direct regex ({self.regexp.pattern})"


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
        return "Unsupported pytest arguments: %r" % self.args

    def __repr__(self) -> str:
        return f"{type(self).__name__}({self.args!r})"


class UnsupportedPytestConfigOption(Problem, kind="unsupported-pytest-config-option"):

    name: str

    def __str__(self) -> str:
        return f"Unsupported pytest configuration option: {self.name}"


class MissingPytestFixture(Problem, kind="missing-pytest-fixture"):

    fixture: str

    def __str__(self) -> str:
        return "Missing pytest fixture: %s" % self.fixture

    def __repr__(self) -> str:
        return f"{type(self).__name__}({self.fixture!r})"


class MissingCargoCrate(Problem, kind="missing-cargo-crate"):

    crate: str
    requirement: Optional[str] = None

    def __str__(self) -> str:
        if self.requirement:
            return f"Missing crate: {self.crate} ({self.requirement})"
        else:
            return "Missing crate: %s" % self.crate


def cargo_missing_requirement(m):
    try:
        crate, requirement = m.group(1).split(" ", 1)
    except ValueError:
        crate = m.group(1)
        requirement = None
    return MissingCargoCrate(crate, requirement)


class MissingLatexFile(Problem, kind="missing-latex-file"):

    filename: str

    def __str__(self) -> str:
        return "Missing LaTeX file: %s" % self.filename


class MissingFontspec(Problem, kind="missing-fontspec"):

    fontspec: str

    def __str__(self) -> str:
        return "Missing font spec: %s" % self.fontspec


class MissingDHCompatLevel(Problem, kind="missing-dh-compat-level"):

    command: str

    def __str__(self) -> str:
        return "Missing DH Compat Level (command: %s)" % self.command


class DuplicateDHCompatLevel(Problem, kind="duplicate-dh-compat-level"):

    command: str

    def __str__(self) -> str:
        return "DH Compat Level specified twice (command: %s)" % self.command


class MissingIntrospectionTypelib(Problem, kind="missing-introspection-typelib"):

    library: str

    def __str__(self) -> str:
        return "Missing introspection typelib: %s" % self.library


class UnknownCertificateAuthority(Problem, kind="unknown-certificate-authority"):

    url: str

    def __str__(self) -> str:
        return "Unknown Certificate Authority for %s" % self.url


class MissingXDisplay(Problem, kind="missing-x-display"):
    def __str__(self) -> str:
        return "No X Display"


class MissingPostgresExtension(Problem, kind="missing-postgresql-extension"):

    extension: str

    def __str__(self) -> str:
        return "Missing postgres extension: %s" % self.extension


class MissingLuaModule(Problem, kind="missing-lua-module"):

    module: str

    def __str__(self) -> str:
        return "Missing Lua Module: %s" % self.module


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
        return "Mismatch versions ({}, {})".format(
            self.makefile_version, self.autoconf_version)


class DisappearedSymbols(Problem, kind="disappeared-symbols"):

    def __str__(self) -> str:
        return "Disappeared symbols"


class MissingGnulibDirectory(Problem, kind="missing-gnulib-directory"):

    directory: str

    def __str__(self) -> str:
        return "Missing gnulib directory %s" % self.directory


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
        return "ESM-only module %s must use import()" % self.path


build_failure_regexps = [
    CMakeErrorMatcher(),
    (
        r"error: failed to select a version for the requirement `(.*)`",
        cargo_missing_requirement,
    ),
    (r"^Environment variable \$SOURCE_DATE_EPOCH: No digits were found: $", None),
    (
        r"\[ERROR\] LazyFont - Failed to read font file (.*) "
        r"\<java.io.FileNotFoundException: (.*) \(No such file or directory\)\>"
        r"java.io.FileNotFoundException: (.*) \(No such file or directory\)",
        lambda m: MissingFile(m.group(1)),
    ),
    (r"qt.qpa.xcb: could not connect to display", lambda m: MissingXDisplay()),
    (r'\(.*:[0-9]+\): Gtk-WARNING \*\*: [0-9]{2}:[0-9]{2}:[0-9]{2}\.[0-9]{3}: cannot open display: ', lambda m: MissingXDisplay()),
    (
        r"\s*Package (.*) was not found in the pkg-config search path.",
        lambda m: MissingPkgConfig(m.group(1)),
    ),
    (
        r"Can't open display",
        lambda m: MissingXDisplay(),
    ),
    (
        r"Can't open (.+): No such file or directory.*",
        file_not_found,
    ),
    (
        r'pkg-config does not know (.*) at .*\.',
        lambda m: MissingPkgConfig(m.group(1)),
    ),
    (
        r'\*\*\* Please install (.*) \(atleast version (.*)\) or adjust',
        lambda m: MissingPkgConfig(m.group(1), m.group(2))
    ),
    (
        r"go runtime is required: https://golang.org/doc/install",
        lambda m: MissingGoRuntime(),
    ),
    (
        r"\%Error: '(.*)' must be installed to build",
        lambda m: MissingCommand(m.group(1)),
    ),
    (
        r'configure: error: "Could not find (.*) in PATH"',
        lambda m: MissingCommand(m.group(1)),
    ),
    (
        r'Could not find executable (.*)',
        lambda m: MissingCommand(m.group(1))
    ),
    (
        r"go: .*: Get \"(.*)\": x509: certificate signed by unknown authority",
        lambda m: UnknownCertificateAuthority(m.group(1)),
    ),
    (
        r".*.go:[0-9]+:[0-9]+: .*: Get \"(.*)\": x509: certificate signed by unknown authority",
        lambda m: UnknownCertificateAuthority(m.group(1)),
    ),
    (
        r"fatal: unable to access '(.*)': server certificate verification failed. CAfile: none CRLfile: none",
        lambda m: UnknownCertificateAuthority(m.group(1)),
    ),
    (
        r'curl: \(77\) error setting certificate verify locations:  CAfile: (.*) CApath: (.*)',
        lambda m: MissingFile(m.group(1))
    ),
    (
        r"\t\(Do you need to predeclare (.*)\?\)",
        lambda m: MissingPerlPredeclared(m.group(1)),
    ),
    (
        r"Bareword \"(.*)\" not allowed while \"strict subs\" in use at "
        r"Makefile.PL line ([0-9]+).",
        lambda m: MissingPerlPredeclared(m.group(1)),
    ),
    (
        r'String found where operator expected at Makefile.PL line ([0-9]+), '
        'near "([a-z0-9_]+).*"',
        lambda m: MissingPerlPredeclared(m.group(2)),
    ),
    (r"  vignette builder 'knitr' not found", lambda m: MissingRPackage("knitr")),
    (
        r"fatal: unable to auto-detect email address \(got \'.*\'\)",
        lambda m: MissingGitIdentity(),
    ),
    (
        r"E       fatal: unable to auto-detect email address \(got \'.*\'\)",
        lambda m: MissingGitIdentity(),
    ),
    (r"gpg: no default secret key: No secret key", lambda m: MissingSecretGpgKey()),
    (
        r"ERROR: FAILED--Further testing stopped: "
        r"Test requires module \'(.*)\' but it\'s not found",
        lambda m: MissingPerlModule(None, m.group(1)),
    ),
    (
        r"(subprocess.CalledProcessError|error): "
        r"Command \'\[\'/usr/bin/python([0-9.]*)\', \'-m\', \'pip\', "
        r"\'--disable-pip-version-check\', \'wheel\', \'--no-deps\', \'-w\', "
        r".*, \'([^-][^\']+)\'\]\' "
        r"returned non-zero exit status 1.",
        lambda m: MissingPythonDistribution.from_requirement_str(
            m.group(3), python_version=(int(m.group(2)[0]) if m.group(2) else None)
        ),
    ),
    (
        r"vcversioner: \[\'git\', .*, \'describe\', \'--tags\', \'--long\'\] "
        r"failed and \'(.*)/version.txt\' isn\'t present\.",
        lambda m: MissingVcVersionerVersion(),
    ),
    (
        r"vcversioner: no VCS could be detected in '(.*)' and "
        r"'(.*)/version.txt' isn't present\.",
        lambda m: MissingVcVersionerVersion(),
    ),

    (
        r"You don't have a working TeX binary \(tex\) installed anywhere in",
        lambda m: MissingCommand("tex"),
    ),
    (
        r"# Module \'(.*)\' is not installed",
        lambda m: MissingPerlModule(None, m.group(1)),
    ),
    (
        r'Base class package "(.*)" is empty.',
        lambda m: MissingPerlModule(None, m.group(1)),
    ),
    (
        r"    \!  (.*::.*) is not installed",
        lambda m: MissingPerlModule(None, m.group(1)),
    ),
    (
        r'Cannot find (.*) in @INC at (.*) line ([0-9]+)\.',
        lambda m: MissingPerlModule(None, m.group(1)),
    ),
    (
        r'(.*::.*) (.*) is required to configure our .* dependency, '
        r'please install it manually or upgrade your CPAN/CPANPLUS',
        lambda m: MissingPerlModule(None, m.group(1), minimum_version=m.group(2))
    ),
    (
        r"configure: error: Missing lib(.*)\.",
        lambda m: MissingLibrary(m.group(1)),
    ),
    (
        r"OSError: (.*): cannot open shared object file: No such file or directory",
        lambda m: MissingFile(m.group(1)),
    ),
    (
        r'The "(.*)" executable has not been found\.',
        lambda m: MissingCommand(m.group(1)),
    ),
    (
        r"  '\! LaTeX Error: File `(.*)' not found.'",
        lambda m: MissingLatexFile(m.group(1)),
    ),

    (
        r"\! LaTeX Error: File `(.*)\' not found\.",
        lambda m: MissingLatexFile(m.group(1)),
    ),
    (
        r"(\!|.*:[0-9]+:) Package fontspec Error: The font \"(.*)\" cannot be found\.",
        lambda m: MissingFontspec(m.group(2)),
    ),
    (r"  vignette builder \'(.*)\' not found", lambda m: MissingRPackage(m.group(1))),
    (
        r"Error: package [‘'](.*)[’'] (.*) was found, but >= (.*) is required by [‘'](.*)[’']",
        lambda m: MissingRPackage(m.group(1), m.group(3)),
    ),
    (
        r'there is no package called \'(.*)\'',
        lambda m: MissingRPackage(m.group(1))
    ),
    (r"Error in .*: there is no package called ‘(.*)’", lambda m: MissingRPackage(m.group(1))),
    (r"  there is no package called \'(.*)\'", lambda m: MissingRPackage(m.group(1))),
    (
        r"Exception: cannot execute command due to missing interpreter: (.*)",
        command_missing,
    ),
    (
        r'E: Build killed with signal TERM after ([0-9]+) minutes of inactivity',
        lambda m: InactiveKilled(int(m.group(1)))
    ),

    (r'\[.*Authority\] PAUSE credentials not found in "config.ini" or "dist.ini" or "~/.pause"\! '
     r'Please set it or specify an authority for this plugin. at inline delegation in '
     r'Dist::Zilla::Plugin::Authority for logger->log_fatal \(attribute declared in '
     r'/usr/share/perl5/Dist/Zilla/Role/Plugin.pm at line [0-9]+\) line [0-9]+\.',
     lambda m: MissingPauseCredentials()),

    (
        r'npm ERR\! ERROR: \[Errno 2\] No such file or directory: \'(.*)\'',
        file_not_found
    ),
    (
        r'\*\*\* error: gettext infrastructure mismatch: using a Makefile\.in\.in '
        r'from gettext version ([0-9.]+) but the autoconf macros are from gettext '
        r'version ([0-9.]+)',
        lambda m: MismatchGettextVersions(m.group(1), m.group(2))),

    (
        r'You need to install the (.*) package to use this program\.',
        lambda m: MissingVagueDependency(m.group(1))
    ),
    (
        r'You need to install (.*)',
        lambda m: MissingVagueDependency(m.group(1))),

    (
        r"configure: error: You don't seem to have the (.*) library installed\..*",
        lambda m: MissingVagueDependency(m.group(1))),

    (
        r'configure: error: You need (.*) installed',
        lambda m: MissingVagueDependency(m.group(1))
    ),

    (
        r'open3: exec of cme (.*) failed: No such file or directory '
        r'at .*/Dist/Zilla/Plugin/Run/Role/Runner.pm line [0-9]+\.',
        lambda m: MissingPerlModule(None, 'App::Cme::Command::' + m.group(1))
    ),

    (
        r'pg_ctl: cannot be run as (.*)',
        lambda m: InvalidCurrentUser(m.group(1)),
    ),

    (
        r'([^ ]+) \(for section ([^ ]+)\) does not appear to be installed',
        lambda m: MissingPerlModule(None, m.group(1))),

    (
        r'(.*) version (.*) required--this is only version (.*) '
        r'at .*\.pm line [0-9]+\.',
        lambda m: MissingPerlModule(None, m.group(1), minimum_version=m.group(2)),
    ),

    (
        r'Bailout called\.  Further testing stopped:  '
        r'YOU ARE MISSING REQUIRED MODULES: \[ ([^,]+)(.*) \]:',
        lambda m: MissingPerlModule(None, m.group(1))
    ),

    (
        r'CMake Error: CMake was unable to find a build program corresponding'
        r' to "(.*)".  CMAKE_MAKE_PROGRAM is not set\.  You probably need to '
        r'select a different build tool\.',
        lambda m: MissingVagueDependency(m.group(1))
    ),

    (
        r"Dist currently only works with Git or Mercurial repos",
        lambda m: VcsControlDirectoryNeeded(['git', 'hg']),
    ),

    (
        r'GitHubMeta: need a .git\/config file, and you don\'t have one',
        lambda m: VcsControlDirectoryNeeded(['git'])
    ),

    (
        r"Exception: Versioning for this project requires either an sdist "
        r"tarball, or access to an upstream git repository\. It's also "
        r"possible that there is a mismatch between the package name "
        r"in setup.cfg and the argument given to pbr\.version\.VersionInfo\. "
        r"Project name .* was given, but was not able to be found\.",
        lambda m: VcsControlDirectoryNeeded(["git"])
    ),

    (r'configure: error: no suitable Python interpreter found',
     lambda m: MissingCommand('python')),

    (r'Could not find external command "(.*)"',
     lambda m: MissingCommand(m.group(1))),

    (r'  Failed to find (.*) development headers\.',
     lambda m: MissingVagueDependency(m.group(1))),

    (r'\*\*\* \Subdirectory \'(.*)\' does not yet exist. '
     r'Use \'./gitsub.sh pull\' to create it, or set the '
     r'environment variable GNULIB_SRCDIR\.',
     lambda m: MissingGnulibDirectory(m.group(1))
     ),

    (r'configure: error: Cap\'n Proto compiler \(capnp\) not found.',
     lambda m: MissingCommand('capnp')),

    (r'lua: (.*):(\d+): module \'(.*)\' not found:',
     lambda m: MissingLuaModule(m.group(3))),

    (r'Unknown key\(s\) in sphinx_gallery_conf:', None),

    (r'(.+\.gir):In (.*): error: (.*)', None),
    (r'(.+\.gir):[0-9]+\.[0-9]+-[0-9]+\.[0-9]+: error: (.*)', None),

    (r'psql:.*\.sql:[0-9]+: ERROR:  (.*)', None),

    (r'intltoolize: \'(.*)\' is out of date: use \'--force\' to overwrite',
     None),

    (r"E: pybuild pybuild:[0-9]+: cannot detect build system, please "
     r"use --system option or set PYBUILD_SYSTEM env\. variable", None),

    (r'--   Requested \'(.*) >= (.*)\' but version of (.*) is (.*)',
     lambda m: MissingPkgConfig(m.group(1), minimum_version=m.group(2))),

    (r'.*Could not find (.*) lib/headers, please set '
     r'.* or ensure (.*).pc is in PKG_CONFIG_PATH\.',
     lambda m: MissingPkgConfig(m.group(2))),

    (r'go: go.mod file not found in current directory or any parent directory; '
     r'see \'go help modules\'', lambda m: MissingGoModFile()),

    (r'go: cannot find main module, but found Gopkg.lock in (.*)',
     lambda m: MissingGoModFile()),

    (r'go: updates to go.mod needed; to update it:',
     lambda m: OutdatedGoModFile()),

    (r'(c\+\+|collect2|cc1|g\+\+): fatal error: .*', None),

    (r'fatal: making (.*): failed to create tests\/decode.trs', None),

    # ocaml
    (r'Please specify at most one of .*', None),

    # Python lint
    (r'.*\.py:[0-9]+:[0-9]+: [A-Z][0-9][0-9][0-9] .*', None),

    (r'PHPUnit requires the \"(.*)\" extension\.',
     lambda m: MissingPHPExtension(m.group(1))),

    (r'     \[exec\] PHPUnit requires the "(.*)" extension\.',
     lambda m: MissingPHPExtension(m.group(1))),

    (r".*/gnulib-tool: \*\*\* minimum supported autoconf version is (.*)\. ",
     lambda m: MinimumAutoconfTooOld(m.group(1))),

    (r"configure.(ac|in):[0-9]+: error: Autoconf version (.*) or higher is required",
     lambda m: MissingVagueDependency("autoconf", minimum_version=m.group(2))),

    (r'# Error: The file "(MANIFEST|META.yml)" is missing from '
     'this distribution\\. .*',
     lambda m: MissingPerlDistributionFile(m.group(1)),
     ),

    (r"^  ([^ ]+) does not exist$", file_not_found),

    (r'\s*> Cannot find \'\.git\' directory',
     lambda m: VcsControlDirectoryNeeded(['git'])),
    (r'Unable to find the \'(.*)\' executable\. .*',
     lambda m: MissingCommand(m.group(1))),
    (r'\[@RSRCHBOY\/CopyrightYearFromGit\]  -  '
     r'412 No \.git subdirectory found',
     lambda m: VcsControlDirectoryNeeded(['git'])),
    (r'Couldn\'t find version control data \(git/hg/bzr/svn supported\)',
     lambda m: VcsControlDirectoryNeeded(['git', 'hg', 'bzr', 'svn'])),
    (r'RuntimeError: Unable to determine package version. '
     r'No local Git clone detected, and no version file found at .*',
     lambda m: VcsControlDirectoryNeeded(['git'])),
    (r'"(.*)" failed to start: "No such file or directory" '
     r'at .*.pm line [0-9]+\.',
     lambda m: MissingCommand(m.group(1))),
    (r'Can\'t find ([^ ]+)\.', lambda m: MissingCommand(m.group(1))),
    (r'Error: spawn (.*) ENOENT',
     lambda m: MissingCommand(m.group(1))),

    (r'E ImportError: Failed to initialize: Bad (.*) executable\.',
     lambda m: MissingCommand(m.group(1))),

    (r'ESLint couldn\'t find the config "(.*)" to extend from\. '
     r'Please check that the name of the config is correct\.',
     None),
    (
        r'E OSError: no library called "cairo-2" was found',
        lambda m: MissingLibrary(m.group(1))
    ),
    (
        r"ERROR: \[Errno 2\] No such file or directory: '(.*)'",
        file_not_found_maybe_executable,
    ),
    (
        r"error: \[Errno 2\] No such file or directory: '(.*)'",
        file_not_found_maybe_executable,
    ),
    (
        r'We need the Python library (.+) to be installed\. .*',
        lambda m: MissingPythonDistribution(m.group(1))
    ),

    # Waf
    (
        r'Checking for header (.+\.h|.+\.hpp)\s+: not found ',
        lambda m: MissingCHeader(m.group(1))
    ),

    (
        r'000: File does not exist (.*)',
        file_not_found,
    ),

    (
        r'ERROR: Coverage for lines \(([0-9.]+)%\) does not meet '
        r'global threshold \(([0-9]+)%\)',
        lambda m: CodeCoverageTooLow(float(m.group(1)), float(m.group(2)))
    ),

    (
        r'Error \[ERR_REQUIRE_ESM\]: '
        r'Must use import to load ES Module: (.*)',
        lambda m: ESModuleMustUseImport(m.group(1)),
    ),

    (r".* (/<<BUILDDIR>>/.*): No such file or directory",
     file_not_found),

    (r"Cannot open file `(.*)' in mode `(.*)' \(No such file or directory\)",
     file_not_found),

    (r"[^:]+: cannot stat \'(.*)\': No such file or directory", file_not_found),
    (r"cat: (.*): No such file or directory", file_not_found),

    (r"ls: cannot access \'(.*)\': No such file or directory", file_not_found),
    (r"Problem opening (.*): No such file or directory at (.*) line ([0-9]+)\.",
     file_not_found),

    (r"/bin/bash: (.*): No such file or directory", file_not_found),
    (r'\(The package \"(.*)\" was not found when loaded as a Node module '
     r'from the directory \".*\"\.\)', lambda m: MissingNodePackage(m.group(1))),
    (r'\+\-\- UNMET DEPENDENCY (.*)', lambda m: MissingNodePackage(m.group(1))),

    (r'Project ERROR: Unknown module\(s\) in QT: (.*)',
     lambda m: MissingQtModules(m.group(1).split())),

    (r'(.*):(\d+):(\d+): '
     r'ERROR: Vala compiler \'.*\' can not compile programs',
     lambda m: ValaCompilerCannotCompile()),

    (r'(.*):(\d+):(\d+): ERROR: Problem encountered: '
     r'Cannot load ([^ ]+) library\. (.*)',
     lambda m: MissingLibrary(m.group(4))),

    (r"go: (.*)@(.*): missing go.sum entry; to add it:",
     lambda m: MissingGoSumEntry(m.group(1), m.group(2))),

    (r'E: pybuild pybuild:(.*): configure: plugin (.*) failed with: '
     r'PEP517 plugin dependencies are not available\. '
     r'Please Build-Depend on (.*)\.',
     lambda m: MissingDebianBuildDep(m.group(1))),

    # ADD NEW REGEXES ABOVE THIS LINE

    (r'configure: error: Can not find "(.*)" .* in your PATH',
     lambda m: MissingCommand(m.group(1))),

    # Intentionally at the bottom of the list.
    (r'([^ ]+) package not found\. Please install from (https://[^ ]+)',
     lambda m: MissingVagueDependency(m.group(1), url=m.group(2))),
    (r'([^ ]+) package not found\. Please use \'pip install .*\' first',
     lambda m: MissingPythonDistribution(m.group(1))),

    (r".*: No space left on device", lambda m: NoSpaceOnDevice()),
    (r".*(No space left on device).*", lambda m: NoSpaceOnDevice()),

    (r'ocamlfind: Package `(.*)\' not found',
     lambda m: MissingOCamlPackage(m.group(1))),
    # Not a very unique ocaml-specific pattern :(
    (r'Error: Library "(.*)" not found.',
     lambda m: MissingOCamlPackage(m.group(1))),

    # ADD NEW REGEXES ABOVE THIS LINE

    # Intentionally at the bottom of the list, since they're quite broad.
    (r'configure: error: ([^ ]+) development files not found',
     lambda m: MissingVagueDependency(m.group(1))),
    (r'Exception: ([^ ]+) development files not found\..*',
     lambda m: MissingVagueDependency(m.group(1))),
    (r'Exception: Couldn\'t find (.*) source libs\!',
     lambda m: MissingVagueDependency(m.group(1))),
    ('configure: error: \'(.*)\' command was not found',
     lambda m: MissingCommand(m.group(1))),
    (
        r"configure: error: (.*) not present.*",
        lambda m: MissingVagueDependency(m.group(1))
    ),
    (
        r"configure: error: (.*) >= (.*) not found",
        lambda m: MissingVagueDependency(m.group(1), minimum_version=m.group(2))
    ),
    (
        r"configure: error: (.*) headers (could )?not (be )?found",
        lambda m: MissingVagueDependency(m.group(1)),
    ),
    (
        r"configure: error: (.*) ([0-9].*) (could )?not (be )?found",
        lambda m: MissingVagueDependency(m.group(1), minimum_version=m.group(2)),
    ),
    (
        r"configure: error: (.*) (could )?not (be )?found",
        lambda m: MissingVagueDependency(m.group(1)),
    ),
    (
        r"configure: error: (.*) ([0-9.]+) is required to build.*",
        lambda m: MissingVagueDependency(m.group(1), minimum_version=m.group(2)),
    ),
    (
        ".*meson.build:([0-9]+):([0-9]+): ERROR: Problem encountered: (.*) (.*) or later required",
        lambda m: MissingVagueDependency(m.group(3), minimum_version=m.group(4)),
    ),

    (
        r"configure: error: Please install (.*) from (http:\/\/[^ ]+)",
        lambda m: MissingVagueDependency(m.group(1), url=m.group(2)),
    ),
    (
        r"configure: error: Required package (.*) (is ?)not available\.",
        lambda m: MissingVagueDependency(m.group(1)),
    ),
    (
        r"Error\! You need to have (.*) \((.*)\) around.",
        lambda m: MissingVagueDependency(m.group(1), url=m.group(2)),
    ),
    (
        r"configure: error: You don\'t have (.*) installed",
        lambda m: MissingVagueDependency(m.group(1)),
    ),
    (
        r"configure: error: Could not find a recent version of (.*)",
        lambda m: MissingVagueDependency(m.group(1)),
    ),
    (
        r"configure: error: Unable to locate (.*)",
        lambda m: MissingVagueDependency(m.group(1)),
    ),
    (
        r"configure: error: Missing the (.* library)",
        lambda m: MissingVagueDependency(m.group(1)),
    ),
    (
        r"configure: error: (.*) requires (.* libraries), .*",
        lambda m: MissingVagueDependency(m.group(2)),
    ),
    (
        r"configure: error: (.*) requires ([^ ]+)\.",
        lambda m: MissingVagueDependency(m.group(2))
    ),
    (
        r"(.*) cannot be discovered in ([^ ]+)",
        lambda m: MissingVagueDependency(m.group(1))
    ),
    (
        r"configure: error: Missing required program '(.*)'.*",
        lambda m: MissingVagueDependency(m.group(1)),
    ),
    (
        r"configure: error: Missing (.*)\.",
        lambda m: MissingVagueDependency(m.group(1)),
    ),
    (
        r"configure: error: Unable to find (.*), please install (.*)",
        lambda m: MissingVagueDependency(m.group(2)),
    ),
    (r"configure: error: (.*) Not found", lambda m: MissingVagueDependency(m.group(1))),
    (
        r"configure: error: You need to install (.*)",
        lambda m: MissingVagueDependency(m.group(1)),
    ),
    (
        r'configure: error: (.*) \((.*)\) not found\.',
        lambda m: MissingVagueDependency(m.group(2))
    ),
    (
        r'configure: error: (.*) libraries are required for compilation',
        lambda m: MissingVagueDependency(m.group(1))
    ),
    (
        r'configure: error: .*Make sure you have (.*) installed\.',
        lambda m: MissingVagueDependency(m.group(1))
    ),
    (
        r'error: Cannot find (.*) in the usual places. .*',
        lambda m: MissingVagueDependency(m.group(1))),
    (
        r'Makefile:[0-9]+: \*\*\* "(.*) was not found"\.  Stop\.',
        lambda m: MissingVagueDependency(m.group(1))
    ),
    (
        r'Makefile:[0-9]+: \*\*\* '
        r'\"At least (.*) version (.*) is needed to build (.*)\.".  Stop\.',
        lambda m: MissingVagueDependency(m.group(1), minimum_version=m.group(2))
    ),
    (r"([a-z0-9A-Z]+) not found", lambda m: MissingVagueDependency(m.group(1))),
    (r'ERROR:  Unable to locate (.*)\.', lambda m: MissingVagueDependency(m.group(1))),
    ('\x1b\\[1;31merror: (.*) not found\x1b\\[0;32m', lambda m: MissingVagueDependency(m.group(1))),
    (r'You do not have (.*) correctly installed\. .*',
     lambda m: MissingVagueDependency(m.group(1))),
    (r'Error: (.*) is not available on your system',
     lambda m: MissingVagueDependency(m.group(1)),
     ),
    (r'ERROR: (.*) (.*) or later is required',
     lambda m: MissingVagueDependency(m.group(1), minimum_version=m.group(2))),
    (r'configure: error: .*Please install the \'(.*)\' package\.',
     lambda m: MissingVagueDependency(m.group(1))),
    (r'Error: Please install ([^ ]+) package',
     lambda m: MissingVagueDependency(m.group(1))),
    (r'configure: error: <(.*\.h)> is required',
     lambda m: MissingCHeader(m.group(1))),
    (r'configure: error: ([^ ]+) is required',
     lambda m: MissingVagueDependency(m.group(1))),
    (r'configure: error: you should install ([^ ]+) first',
     lambda m: MissingVagueDependency(m.group(1))),
    (r'configure: error: .*You need (.*) installed.',
     lambda m: MissingVagueDependency(m.group(1))),
    (r'To build (.*) you need (.*)',
     lambda m: MissingVagueDependency(m.group(1))),
    (r'.*Can\'t ([^\. ]+)\. (.*)',
     lambda m: MissingVagueDependency(m.group(1))),
    (r'([^ ]+) >= (.*) is required',
     lambda m: MissingVagueDependency(m.group(1), m.group(2))),
    (r'.*: ERROR: (.*) needs to be installed to run these tests',
     lambda m: MissingVagueDependency(m.group(1))),
    (r'ERROR: Unable to locate (.*)\.',
     lambda m: MissingVagueDependency(m.group(1))),
    (r'ERROR: Cannot find command \'(.*)\' - do you '
     r'have \'(.*)\' installed and in your PATH\?',
     lambda m: MissingCommand(m.group(1))),
    (r'ValueError: no ([^ ]+) installed, .*',
     lambda m: MissingVagueDependency(m.group(1))),
    (r'This project needs (.*) in order to build\. .*',
     lambda m: MissingVagueDependency(m.group(1))),
    (r'ValueError: Unable to find (.+)',
     lambda m: MissingVagueDependency(m.group(1))),
    (r'([^ ]+) executable not found\. .*',
     lambda m: MissingCommand(m.group(1))),
    (r'ERROR: InvocationError for command could not find executable (.*)',
     lambda m: MissingCommand(m.group(1))),
    (r'E ImportError: Unable to find ([^ ]+) shared library',
     lambda m: MissingLibrary(m.group(1))),
    (r'\s*([^ ]+) library not found on the system',
     lambda m: MissingLibrary(m.group(1))),
    (r'\s*([^ ]+) library not found(\.?)',
     lambda m: MissingLibrary(m.group(1))),
    (r'.*Please install ([^ ]+) libraries\.',
     lambda m: MissingVagueDependency(m.group(1))),
    (r'Error: Please install (.*) package',
     lambda m: MissingVagueDependency(m.group(1))),
    (r'Please get ([^ ]+) from (www\..*)\.',
     lambda m: MissingVagueDependency(m.group(1), url=m.group(2))),
    (r'Please install ([^ ]+) so that it is on the PATH and try again\.',
     lambda m: MissingCommand(m.group(1))),
    (r'configure: error: No (.*) binary found in (.*)',
     lambda m: MissingCommand(m.group(1))),
    (r'Could not find ([A-Za-z-]+)',
     lambda m: MissingVagueDependency(m.group(1))),
    (r'No ([^ ]+) includes and libraries found',
     lambda m: MissingVagueDependency(m.group(1))),
    (r'Required library (.*) not found\.',
     lambda m: MissingVagueDependency(m.group(1))),
    (r'Missing ([^ ]+) boost library, .*',
     lambda m: MissingLibrary(m.group(1))),
    (r'configure: error: ([^ ]+) needed\!',
     lambda m: MissingVagueDependency(m.group(1))),
    (r'\*\*\* (.*) not found, please install it \*\*\*',
     lambda m: MissingVagueDependency(m.group(1))),
    (
        r"configure: error: could not find ([^ ]+)",
        lambda m: MissingVagueDependency(m.group(1)),
    ),
    (r'([^ ]+) is required for ([^ ]+)\.',
     lambda m: MissingVagueDependency(m.group(1))),

    (r'configure: error: \*\*\* No ([^.])\! '
     r'Install (.*) development headers/libraries! \*\*\*',
     lambda m: MissingVagueDependency(m.group(1))),

    (r'configure: error: \'(.*)\' cannot be found',
     lambda m: MissingVagueDependency(m.group(1))),

    (r'No (.*) includes and libraries found',
     lambda m: MissingVagueDependency(m.group(1))),

    (r'\s*No (.*) version could be found in your system\.',
     lambda m: MissingVagueDependency(m.group(1))),

    (r'You need (.+)', lambda m: MissingVagueDependency(m.group(1))),

    (r'configure: error: ([^ ]+) is needed',
     lambda m: MissingVagueDependency(m.group(1))),

    (r'configure: error: Cannot find ([^ ]+)\.',
     lambda m: MissingVagueDependency(m.group(1))),

    (r'configure: error: ([^ ]+) requested but not installed\.',
     lambda m: MissingVagueDependency(m.group(1))),

    (r'We need the Python library (.+) to be installed\..*',
     lambda m: MissingPythonDistribution(m.group(1))),

    (r'(.*) uses (.*) \(.*\) for installation but (.*) was not found',
     lambda m: MissingVagueDependency(m.group(1))),

    (r'ERROR: could not locate the \'([^ ]+)\' utility',
     lambda m: MissingCommand(m.group(1))),

    (r'Can\'t find (.*) libs. Exiting',
     lambda m: MissingLibrary(m.group(1))),
]


compiled_build_failure_regexps = []
for entry in build_failure_regexps:
    try:
        matcher: Matcher
        if isinstance(entry, tuple):
            (regexp, cb) = entry
            matcher = SingleLineMatcher(regexp, cb)
        else:
            matcher = entry  # type: ignore
        compiled_build_failure_regexps.append(matcher)
    except re.error as e:
        raise Exception(f"Error in {regexp}: {e}") from e


find_secondary_build_failure = _buildlog_consultant_rs.find_secondary_build_failure


def find_build_failure_description(  # noqa: C901
    lines: list[str],
) -> tuple[Optional[Match], Optional[Problem]]:
    """Find the key failure line in build output.

    Returns:
      tuple with (match object, error object)
    """
    OFFSET = 250
    # Is this cmake-specific, or rather just kf5 / qmake ?
    cmake = False
    # We search backwards for clear errors.
    for i in range(1, OFFSET):
        lineno = len(lines) - i
        if lineno < 0:
            break
        if "cmake" in lines[lineno]:
            cmake = True
        mm, merr = _buildlog_consultant_rs.match_lines(lines, lineno)
        if mm:
            return cast(Match, mm), cast(Problem, merr)
        for matcher in compiled_build_failure_regexps:
            linenos, err, origin = matcher.match(lines, lineno)
            if linenos:
                logger.debug('Found match against %r on %r (lines %r): %r',
                             matcher, [lines[n] for n in linenos], linenos,
                             err)
                return MultiLineMatch.from_lines(lines, linenos, origin=origin), err

    # TODO(jelmer): Remove this in favour of CMakeErrorMatcher above.
    if cmake:
        missing_file_pat = re.compile(
            r"\s*The imported target \"(.*)\" references the file"
        )
        binary_pat = re.compile(r"  Could NOT find (.*) \(missing: .*\)")
        cmake_files_pat = re.compile(
            "^  Could not find a package configuration file provided "
            'by "(.*)" with any of the following names:'
        )
        # Urgh, multi-line regexes---
        for lineno in range(len(lines)):
            line = lines[lineno].rstrip("\n")
            rm = re.fullmatch(binary_pat, line)
            if rm:
                return (
                    SingleLineMatch.from_lines(
                        lines, lineno, origin=f"direct regex ({binary_pat}"),
                    MissingCommand(rm.group(1).lower()),
                )
            rm = re.fullmatch(missing_file_pat, line)
            if rm:
                lineno += 1
                while lineno < len(lines) and not line:
                    lineno += 1
                if lines[lineno + 2].startswith("  but this file does not exist."):
                    rm = re.fullmatch(r'\s*"(.*)"', line)
                    if rm:
                        filename = rm.group(1)
                    else:
                        filename = line
                    return (
                        SingleLineMatch.from_lines(
                            lines, lineno, origin=f"direct regex {missing_file_pat}"),
                        MissingFile(filename),
                    )
                continue
            if lineno + 1 < len(lines):
                rm = re.fullmatch(
                    cmake_files_pat,
                    line + " " + lines[lineno + 1].lstrip(" ").strip("\n"),
                )
                if rm and lines[lineno + 2] == "\n":
                    i = 3
                    filenames = []
                    while lines[lineno + i].strip():
                        filenames.append(lines[lineno + i].strip())
                        i += 1
                    return (
                        SingleLineMatch.from_lines(
                            lines, lineno, origin="direct regex (cmake)"),
                        CMakeFilesMissing(filenames),
                    )

    # And forwards for vague ("secondary") errors.
    match = find_secondary_build_failure(lines, OFFSET)
    if match:
        return cast(Match, match), None

    return None, None


def as_json(m, problem):
    ret = {}
    if m:
        ret["lineno"] = m.lineno
        ret["line"] = m.line
        ret["origin"] = m.origin
    if problem:
        ret["problem"] = problem.kind
        try:
            ret["details"] = problem.json()
        except NotImplementedError:
            ret["details"] = None
    return ret


def main(argv=None):
    import argparse
    import json

    parser = argparse.ArgumentParser("analyse-build-log")
    parser.add_argument("path", type=str, default="-", nargs="?")
    parser.add_argument("--context", "-c", type=int, default=5)
    parser.add_argument("--json", action="store_true", help="Output JSON.")
    parser.add_argument("--debug", action="store_true")
    parser.add_argument(
        "--version", action="version", version="%(prog)s " + version_string
    )
    args = parser.parse_args(argv)

    if args.debug:
        loglevel = logging.DEBUG
    else:
        loglevel = logging.INFO

    logging.basicConfig(level=loglevel, format="%(message)s")

    if args.path == '-':
        args.path = '/dev/stdin'

    with open(args.path) as f:
        lines = list(f.readlines())

    m, problem = find_build_failure_description(lines)

    if args.json:
        ret = as_json(m, problem)
        json.dump(ret, sys.stdout, indent=4)
    else:
        if not m:
            logging.info("No issues found")
        else:
            if len(m.linenos) == 1:
                logging.info("Issue found at line %d:", m.lineno)
            else:
                logging.info(
                    "Issue found at lines %d-%d:", m.linenos[0], m.linenos[-1])
            for i in range(
                max(0, m.offsets[0] - args.context),
                min(len(lines), m.offsets[-1] + args.context + 1),
            ):
                logging.info(
                    " %s  %s", ">" if i in m.offsets else " ",
                    lines[i].rstrip("\n")
                )

        if problem:
            logging.info("Identified issue: %s: %s", problem.kind, problem)


if __name__ == "__main__":
    import sys

    sys.exit(main(sys.argv[1:]))
