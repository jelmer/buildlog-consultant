#!/usr/bin/python
# Copyright (C) 2019-2021 Jelmer Vernooij <jelmer@jelmer.uk>
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

from ..common import (
    CMakeFilesMissing,
    CMakeNeedExactVersion,
    find_build_failure_description,
    CcacheError,
    DebhelperPatternNotFound,
    DisappearedSymbols,
    DuplicateDHCompatLevel,
    DhLinkDestinationIsDirectory,
    MismatchGettextVersions,
    MissingBuildFile,
    MissingConfigure,
    MissingJavaScriptRuntime,
    MissingJVM,
    MissingConfigStatusInput,
    MissingCHeader,
    MissingDHCompatLevel,
    UnsupportedDebhelperCompatLevel,
    MissingJDKFile,
    MissingJDK,
    MissingJRE,
    MissingIntrospectionTypelib,
    MissingPythonModule,
    MissingPythonDistribution,
    MissingGoPackage,
    MissingFile,
    MissingMavenArtifacts,
    MissingNodeModule,
    MissingCommand,
    MissingPkgConfig,
    MissingCMakeComponents,
    MissingVcVersionerVersion,
    MissingPerlFile,
    MissingPerlModule,
    MissingPerlPredeclared,
    MissingLatexFile,
    MissingPhpClass,
    MissingRubyGem,
    MissingSetupPyCommand,
    MissingValaPackage,
    MissingXmlEntity,
    MissingVagueDependency,
    MissingLibrary,
    MissingJavaClass,
    MissingRPackage,
    MissingAutoconfMacro,
    MissingSprocketsFile,
    MissingAutomakeInput,
    MissingGoModFile,
    NeedPgBuildExtUpdateControl,
    DhMissingUninstalled,
    DhUntilUnsupported,
    DhAddonLoadFailure,
    NoSpaceOnDevice,
    DhWithOrderIncorrect,
    UpstartFilePresent,
    DirectoryNonExistant,
    UnknownCertificateAuthority,
    MissingGitIdentity,
    VcsControlDirectoryNeeded,
)
import unittest


class FindBuildFailureDescriptionTests(unittest.TestCase):
    def run_test(self, lines, lineno, err=None):
        (match, actual_err) = find_build_failure_description(lines)
        if match is not None:
            self.assertEqual(match.line, lines[lineno - 1])
            self.assertEqual(lineno, match.lineno)
        else:
            self.assertIsNone(match)
        if err:
            self.assertEqual(actual_err, err)
        else:
            self.assertIs(None, actual_err)

    def test_make_missing_rule(self):
        self.run_test(
            [
                "make[1]: *** No rule to make target 'nno.autopgen.bin', "
                "needed by 'dan-nno.autopgen.bin'.  Stop."
            ],
            1,
            MissingBuildFile('nno.autopgen.bin'),
        )
        self.run_test(
            [
                "make[1]: *** No rule to make target '/usr/share/blah/blah', "
                "needed by 'dan-nno.autopgen.bin'.  Stop."
            ],
            1,
            MissingFile("/usr/share/blah/blah"),
        )
        self.run_test(
            [
                "debian/rules:4: /usr/share/openstack-pkg-tools/pkgos.make: "
                "No such file or directory"
            ],
            1,
            MissingFile("/usr/share/openstack-pkg-tools/pkgos.make"),
        )

    def test_git_identity(self):
        self.run_test(
            [
                "fatal: unable to auto-detect email address "
                "(got 'jenkins@osuosl167-amd64.(none)')"
            ],
            1,
            MissingGitIdentity(),
        )

    def test_ioerror(self):
        self.run_test(
            [
                "E   IOError: [Errno 2] No such file or directory: "
                "'/usr/lib/python2.7/poly1305/rfc7539.txt'"
            ],
            1,
            MissingFile("/usr/lib/python2.7/poly1305/rfc7539.txt"),
        )

    def test_upstart_file_present(self):
        self.run_test(
            [
                "dh_installinit: upstart jobs are no longer supported!  "
                "Please remove debian/sddm.upstart and check if you "
                "need to add a conffile removal"
            ],
            1,
            UpstartFilePresent("debian/sddm.upstart"),
        )

    def test_missing_go_mod_file(self):
        self.run_test(
            [
                "go: go.mod file not found in current directory or any "
                "parent directory; see 'go help modules'"
            ], 1, MissingGoModFile())

    def test_missing_javascript_runtime(self):
        self.run_test(
            [
                "ExecJS::RuntimeUnavailable: "
                "Could not find a JavaScript runtime. "
                "See https://github.com/rails/execjs for a list "
                "of available runtimes."
            ],
            1,
            MissingJavaScriptRuntime(),
        )

    def test_directory_missing(self):
        self.run_test(
            [
                "debian/components/build: 19: cd: can't cd to rollup-plugin",
            ],
            1,
            DirectoryNonExistant("rollup-plugin"),
        )

    def test_vcs_control_directory(self):
        self.run_test(
            ["   > Cannot find '.git' directory"],
            1,
            VcsControlDirectoryNeeded(['git']))

    def test_missing_sprockets_file(self):
        self.run_test(
            [
                "Sprockets::FileNotFound: couldn't find file "
                "'activestorage' with type 'application/javascript'"
            ],
            1,
            MissingSprocketsFile("activestorage", "application/javascript"),
        )

    def test_gxx_missing_file(self):
        self.run_test(
            [
                "g++: error: /usr/lib/x86_64-linux-gnu/libGL.so: "
                "No such file or directory"
            ],
            1,
            MissingFile("/usr/lib/x86_64-linux-gnu/libGL.so"),
        )

    def test_build_xml_missing_file(self):
        self.run_test(
            ["/<<PKGBUILDDIR>>/build.xml:59: " "/<<PKGBUILDDIR>>/lib does not exist."],
            1,
            MissingBuildFile('lib')
        )

    def test_vignette_builder(self):
        self.run_test(
            ["  vignette builder 'R.rsp' not found"], 1, MissingRPackage("R.rsp")
        )

    def test_dh_missing_addon(self):
        self.run_test(
            [
                "   dh_auto_clean -O--buildsystem=pybuild",
                "E: Please add appropriate interpreter package to Build-Depends, "
                "see pybuild(1) for details.this: $VAR1 = bless( {",
                "     'py3vers' => '3.8',",
                "     'py3def' => '3.8',",
                "     'pyvers' => '',",
                "     'parallel' => '2',",
                "     'cwd' => '/<<PKGBUILDDIR>>',",
                "     'sourcedir' => '.',",
                "     'builddir' => undef,",
                "     'pypydef' => '',",
                "     'pydef' => ''",
                "   }, 'Debian::Debhelper::Buildsystem::pybuild' );",
                "deps: $VAR1 = [];",
            ],
            2,
            DhAddonLoadFailure("pybuild", "Debian/Debhelper/Buildsystem/pybuild.pm"),
        )

    def test_libtoolize_missing_file(self):
        self.run_test(
            ["libtoolize:   error: '/usr/share/aclocal/ltdl.m4' " "does not exist."],
            1,
            MissingFile("/usr/share/aclocal/ltdl.m4"),
        )

    def test_ruby_missing_file(self):
        self.run_test(
            [
                "Error: Error: ENOENT: no such file or directory, "
                "open '/usr/lib/nodejs/requirejs/text.js'"
            ],
            1,
            MissingFile("/usr/lib/nodejs/requirejs/text.js"),
        )

    def test_vcversioner(self):
        self.run_test(
            [
                "vcversioner: ['git', '--git-dir', '/build/tmp0tlam4pe/pyee/.git', "
                "'describe', '--tags', '--long'] failed and "
                "'/build/tmp0tlam4pe/pyee/version.txt' isn't present."
            ],
            1,
            MissingVcVersionerVersion(),
        )

    def test_python_missing_file(self):
        self.run_test(
            [
                "python3.7: can't open file '/usr/bin/blah.py': "
                "[Errno 2] No such file or directory"
            ],
            1,
            MissingFile("/usr/bin/blah.py"),
        )
        self.run_test(
            [
                "python3.7: can't open file 'setup.py': "
                "[Errno 2] No such file or directory"
            ],
            1,
            MissingBuildFile('setup.py')
        )
        self.run_test(
            [
                "E           FileNotFoundError: [Errno 2] "
                "No such file or directory: "
                "'/usr/share/firmware-microbit-micropython/firmware.hex'"
            ],
            1,
            MissingFile("/usr/share/firmware-microbit-micropython/firmware.hex"),
        )

    def test_vague(self):
        self.run_test(
            [
                "configure: error: Please install gnu flex from http://www.gnu.org/software/flex/"
            ],
            1,
            MissingVagueDependency("gnu flex", "http://www.gnu.org/software/flex/"),
        )
        self.run_test(
            ["RuntimeError: cython is missing"], 1,
            MissingVagueDependency("cython"))
        self.run_test(
            [
                "configure: error:",
                "",
                "        Unable to find the Multi Emulator Super System (MESS).",
            ],
            3,
            MissingVagueDependency("the Multi Emulator Super System (MESS)"),
        )
        self.run_test(
            ["configure: error: libwandio 4.0.0 or better is required to compile "
             "this version of libtrace. If you have installed libwandio in a "
             "non-standard location please use LDFLAGS to specify the location of "
             "the library. WANDIO can be obtained from "
             "http://research.wand.net.nz/software/libwandio.php"],
            1, MissingVagueDependency("libwandio", minimum_version="4.0.0"))
        self.run_test(
            ["configure: error: libpcap0.8 or greater is required to compile "
             "libtrace. If you have installed it in a non-standard location please "
             "use LDFLAGS to specify the location of the library"],
            1, MissingVagueDependency("libpcap0.8"))

    def test_gettext_mismatch(self):
        self.run_test(
            ["*** error: gettext infrastructure mismatch: using a "
             "Makefile.in.in from gettext version 0.19 but the autoconf "
             "macros are from gettext version 0.20"],
            1, MismatchGettextVersions('0.19', '0.20'))

    def test_multi_line_configure_error(self):
        self.run_test(["configure: error:", "", "        Some other error."], 3, None)
        self.run_test([
            "configure: error:",
            "",
            "   Unable to find the Multi Emulator Super System (MESS).",
            "",
            "   Please install MESS, or specify the MESS command with",
            "   a MESS environment variable.",
            "",
            "e.g. MESS=/path/to/program/mess ./configure"
            ], 3, MissingVagueDependency("the Multi Emulator Super System (MESS)"))

    def test_interpreter_missing(self):
        self.run_test(
            [
                "/bin/bash: /usr/bin/rst2man: /usr/bin/python: "
                "bad interpreter: No such file or directory"
            ],
            1,
            MissingFile("/usr/bin/python"),
        )
        self.run_test(
            ["env: ‘/<<PKGBUILDDIR>>/socket-activate’: " "No such file or directory"],
            1,
            None,
        )

    def test_webpack_missing(self):
        self.run_test(
            [
                "ERROR in Entry module not found: "
                "Error: Can't resolve 'index.js' in '/<<PKGBUILDDIR>>'"
            ],
            1,
            None,
        )

    def test_installdocs_missing(self):
        self.run_test(
            [
                'dh_installdocs: Cannot find (any matches for) "README.txt" '
                "(tried in ., debian/tmp)"
            ],
            1,
            DebhelperPatternNotFound("README.txt", "installdocs", [".", "debian/tmp"]),
        )

    def test_cmake_missing_file(self):
        self.run_test(
            """\
CMake Error at /usr/lib/x86_64-/cmake/Qt5Gui/Qt5GuiConfig.cmake:27 (message):
  The imported target "Qt5::Gui" references the file

     "/usr/lib/x86_64-linux-gnu/libEGL.so"

  but this file does not exist.  Possible reasons include:

  * The file was deleted, renamed, or moved to another location.

  * An install or uninstall procedure did not complete successfully.

  * The installation package was faulty and contained

     "/usr/lib/x86_64-linux-gnu/cmake/Qt5Gui/Qt5GuiConfigExtras.cmake"

  but not all the files it references.

Call Stack (most recent call first):
  /usr/lib/x86_64-linux-gnu/QtGui/Qt5Gui.cmake:63 (_qt5_Gui_check_file_exists)
  /usr/lib/x86_64-linux-gnu/QtGui/Qt5Gui.cmake:85 (_qt5gui_find_extra_libs)
  /usr/lib/x86_64-linux-gnu/QtGui/Qt5Gui.cmake:186 (include)
  /usr/lib/x86_64-linux-gnu/QtWidgets/Qt5Widgets.cmake:101 (find_package)
  /usr/lib/x86_64-linux-gnu/Qt/Qt5Config.cmake:28 (find_package)
  CMakeLists.txt:34 (find_package)
dh_auto_configure: cd obj-x86_64-linux-gnu && cmake with args
""".splitlines(
                True
            ),
            16,
            MissingFile("/usr/lib/x86_64-linux-gnu/libEGL.so"),
        )

    def test_meson_missing_git(self):
        self.run_test(
            ["meson.build:13:0: ERROR: Git program not found."],
            1,
            MissingCommand("git"),
        )

    def test_meson_missing_lib(self):
        self.run_test(
            ["meson.build:85:0: ERROR: C++ shared or static library 'vulkan-1' not found"],
            1, MissingLibrary("vulkan-1"))

    def test_meson_version(self):
        self.run_test(
            ["meson.build:1:0: ERROR: Meson version is 0.49.2 but "
             "project requires >=0.50."], 1,
            MissingVagueDependency("meson", minimum_version="0.50"))

    def test_need_pgbuildext(self):
        self.run_test(
            [
                "Error: debian/control needs updating from debian/control.in. "
                "Run 'pg_buildext updatecontrol'."
            ],
            1,
            NeedPgBuildExtUpdateControl("debian/control", "debian/control.in"),
        )

    def test_cmake_missing_command(self):
        self.run_test(
            [
                "  Could NOT find Git (missing: GIT_EXECUTABLE)",
                "dh_auto_configure: cd obj-x86_64-linux-gnu && cmake with args",
            ],
            1,
            MissingCommand("git"),
        )

    def test_cmake_missing_include(self):
        self.run_test(
            """\
-- Performing Test _OFFT_IS_64BIT
-- Performing Test _OFFT_IS_64BIT - Success
-- Performing Test HAVE_DATE_TIME
-- Performing Test HAVE_DATE_TIME - Success
CMake Error at CMakeLists.txt:43 (include):
  include could not find load file:

    KDEGitCommitHooks


-- Found KF5Activities: /usr/lib/x86_64-linux-gnu/cmake/KF5Activities/KF5ActivitiesConfig.cmake (found version "5.78.0") 
-- Found KF5Config: /usr/lib/x86_64-linux-gnu/cmake/KF5Config/KF5ConfigConfig.cmake (found version "5.78.0") 
""".splitlines(True), 8, CMakeFilesMissing(['KDEGitCommitHooks.cmake']))

    def test_cmake_missing_cmake_files(self):
        self.run_test(
            """\
  Could not find a package configuration file provided by "sensor_msgs" with
  any of the following names:

    sensor_msgsConfig.cmake
    sensor_msgs-config.cmake

  Add the installation prefix of "sensor_msgs" to CMAKE_PREFIX_PATH or set
  "sensor_msgs_DIR" to a directory containing one of the above files.  If
  "sensor_msgs" provides a separate development package or SDK, be sure it
  has been installed.
dh_auto_configure: cd obj-x86_64-linux-gnu && cmake with args
""".splitlines(
                True
            ),
            1,
            CMakeFilesMissing(["sensor_msgsConfig.cmake", "sensor_msgs-config.cmake"]),
        )
        self.run_test(
            """\
CMake Error at /usr/share/cmake-3.22/Modules/FindPackageHandleStandardArgs.cmake:230 (message):
  Could NOT find KF5 (missing: Plasma PlasmaQuick Wayland ModemManagerQt
  NetworkManagerQt) (found suitable version "5.92.0", minimum required is
  "5.86")
""".splitlines(True), 4, MissingCMakeComponents("KF5", ["Plasma", "PlasmaQuick", "Wayland", "ModemManagerQt", "NetworkManagerQt"]))

    def test_cmake_missing_exact_version(self):
        self.run_test(
            """\
CMake Error at /usr/share/cmake-3.18/Modules/FindPackageHandleStandardArgs.cmake:165 (message):
  Could NOT find SignalProtocol: Found unsuitable version "2.3.3", but
  required is exact version "2.3.2" (found
  /usr/lib/x86_64-linux-gnu/libsignal-protocol-c.so)
""".splitlines(
                True
            ),
            4,
            CMakeNeedExactVersion(
                "SignalProtocol",
                "2.3.3",
                "2.3.2",
                "/usr/lib/x86_64-linux-gnu/libsignal-protocol-c.so",
            ),
        )

    def test_cmake_missing_vague(self):
        self.run_test(
            ["CMake Error at CMakeLists.txt:84 (MESSAGE):", "  alut not found"],
            2,
            MissingVagueDependency("alut"),
        )
        self.run_test(
            ["CMake Error at CMakeLists.txt:213 (message):",
             "  could not find zlib"], 2, MissingVagueDependency("zlib"))

    def test_dh_compat_dupe(self):
        self.run_test(
            [
                "dh_autoreconf: debhelper compat level specified both in "
                "debian/compat and via build-dependency on debhelper-compat"
            ],
            1,
            DuplicateDHCompatLevel("dh_autoreconf"),
        )

    def test_dh_compat_missing(self):
        self.run_test(
            ["dh_clean: Please specify the compatibility level in " "debian/compat"],
            1,
            MissingDHCompatLevel("dh_clean"),
        )

    def test_dh_compat_too_old(self):
        self.run_test([
            "dh_clean: error: Compatibility levels before 7 are no longer "
            "supported (level 5 requested)"], 1,
            UnsupportedDebhelperCompatLevel(7, 5))

    def test_dh_udeb_shared_library(self):
        self.run_test(
            [
                "dh_makeshlibs: The udeb libepoxy0-udeb (>= 1.3) does not contain"
                " any shared libraries but --add-udeb=libepoxy0-udeb (>= 1.3) "
                "was passed!?"
            ],
            1,
        )

    def test_dh_systemd(self):
        self.run_test(
            [
                "dh: unable to load addon systemd: dh: The systemd-sequence is "
                "no longer provided in compat >= 11, please rely on "
                "dh_installsystemd instead"
            ],
            1,
        )

    def test_dh_before(self):
        self.run_test(
            [
                "dh: The --before option is not supported any longer (#932537). "
                "Use override targets instead."
            ],
            1,
        )

    def test_distutils_missing(self):
        self.run_test(
            [
                "distutils.errors.DistutilsError: Could not find suitable "
                "distribution for Requirement.parse('pytest-runner')"
            ],
            1,
            MissingPythonDistribution("pytest-runner", None),
        )
        self.run_test(
            [
                "distutils.errors.DistutilsError: Could not find suitable "
                "distribution for Requirement.parse('certifi>=2019.3.9')"
            ],
            1,
            MissingPythonDistribution("certifi", None, "2019.3.9"),
        )
        self.run_test(
            [
                "distutils.errors.DistutilsError: Could not find suitable "
                "distribution for Requirement.parse('cffi; "
                'platform_python_implementation == "CPython"\')'
            ],
            1,
            MissingPythonDistribution("cffi", None),
        )
        self.run_test(
            [
                "error: Could not find suitable distribution for "
                "Requirement.parse('gitlab')"
            ],
            1,
            MissingPythonDistribution("gitlab", None),
        )
        self.run_test(
            [
                "pkg_resources.DistributionNotFound: The 'configparser>=3.5' "
                "distribution was not found and is required by importlib-metadata"
            ],
            1,
            MissingPythonDistribution("configparser", None, "3.5"),
        )
        self.run_test(
            [
                "error: Command '['/usr/bin/python3.9', '-m', 'pip', "
                "'--disable-pip-version-check', 'wheel', '--no-deps', '-w', "
                "'/tmp/tmp973_8lhm', '--quiet', 'asynctest']' "
                "returned non-zero exit status 1."
            ],
            1,
            MissingPythonDistribution("asynctest", python_version=3),
        )
        self.run_test(
            [
                "subprocess.CalledProcessError: Command "
                "'['/usr/bin/python', '-m', 'pip', "
                "'--disable-pip-version-check', 'wheel', '--no-deps', "
                "'-w', '/tmp/tmpm2l3kcgv', '--quiet', 'setuptools_scm']' "
                "returned non-zero exit status 1."
            ],
            1,
            MissingPythonDistribution("setuptools_scm"),
        )

    def test_lazy_font(self):
        self.run_test(
            [
                "[ERROR] LazyFont - Failed to read font file "
                "/usr/share/texlive/texmf-dist/fonts/opentype/public/"
                "stix2-otf/STIX2Math.otf "
                "<java.io.FileNotFoundException: /usr/share/texlive/texmf-dist/"
                "fonts/opentype/public/stix2-otf/STIX2Math.otf "
                "(No such file or directory)>java.io.FileNotFoundException: "
                "/usr/share/texlive/texmf-dist/fonts/opentype/public/stix2-otf"
                "/STIX2Math.otf (No such file or directory)"
            ],
            1,
            MissingFile(
                "/usr/share/texlive/texmf-dist/fonts/opentype/"
                "public/stix2-otf/STIX2Math.otf"
            ),
        )

    def test_missing_latex_files(self):
        self.run_test(
            ["! LaTeX Error: File `fancyvrb.sty' not found."],
            1,
            MissingLatexFile("fancyvrb.sty"),
        )

    def test_pytest_import(self):
        self.run_test(
            ["E   ImportError: cannot import name cmod"], 1, MissingPythonModule("cmod")
        )
        self.run_test(
            ["E   ImportError: No module named mock"], 1, MissingPythonModule("mock")
        )
        self.run_test(
            [
                "pluggy.manager.PluginValidationError: "
                "Plugin 'xdist.looponfail' could not be loaded: "
                "(pytest 3.10.1 (/usr/lib/python2.7/dist-packages), "
                "Requirement.parse('pytest>=4.4.0'))!"
            ],
            1,
            MissingPythonModule("pytest", 2, "4.4.0"),
        )
        self.run_test(
            [
                "ImportError: Error importing plugin "
                '"tests.plugins.mock_libudev": No module named mock'
            ],
            1,
            MissingPythonModule("mock"),
        )

    def test_sed(self):
        self.run_test(
            ["sed: can't read /etc/locale.gen: No such file or directory"],
            1,
            MissingFile("/etc/locale.gen"),
        )

    def test_python2_import(self):
        self.run_test(
            ["ImportError: No module named pytz"], 1, MissingPythonModule("pytz")
        )
        self.run_test(["ImportError: cannot import name SubfieldBase"], 1, None)

    def test_python3_import(self):
        self.run_test(
            ["ModuleNotFoundError: No module named 'django_crispy_forms'"],
            1,
            MissingPythonModule("django_crispy_forms", 3),
        )
        self.run_test(
            [" ModuleNotFoundError: No module named 'Cython'"],
            1,
            MissingPythonModule("Cython", 3),
        )
        self.run_test(
            ["ModuleNotFoundError: No module named 'distro'"],
            1,
            MissingPythonModule("distro", 3),
        )
        self.run_test(
            ["E   ModuleNotFoundError: No module named 'twisted'"],
            1,
            MissingPythonModule("twisted", 3),
        )
        self.run_test(
            [
                "E   ImportError: cannot import name 'async_poller' "
                "from 'msrest.polling' "
                "(/usr/lib/python3/dist-packages/msrest/polling/__init__.py)"
            ],
            1,
            MissingPythonModule("msrest.polling.async_poller"),
        )
        self.run_test(
            ["/usr/bin/python3: No module named sphinx"],
            1,
            MissingPythonModule("sphinx", 3),
        )
        self.run_test(
            [
                "Could not import extension sphinx.ext.pngmath (exception: "
                "No module named pngmath)"
            ],
            1,
            MissingPythonModule("pngmath"),
        )
        self.run_test(
            [
                "/usr/bin/python3: Error while finding module specification "
                "for 'pep517.build' "
                "(ModuleNotFoundError: No module named 'pep517')"
            ],
            1,
            MissingPythonModule("pep517", python_version=3),
        )

    def test_sphinx(self):
        self.run_test(
            [
                "There is a syntax error in your configuration file: "
                "Unknown syntax: Constant"
            ],
            1,
            None,
        )

    def test_go_missing(self):
        self.run_test(
            [
                "src/github.com/vuls/config/config.go:30:2: cannot find package "
                '"golang.org/x/xerrors" in any of:'
            ],
            1,
            MissingGoPackage("golang.org/x/xerrors"),
        )

    def test_c_header_missing(self):
        self.run_test(
            ["cdhit-common.h:39:9: fatal error: zlib.h: No such file " "or directory"],
            1,
            MissingCHeader("zlib.h"),
        )
        self.run_test(
            [
                "/<<PKGBUILDDIR>>/Kernel/Operation_Vector.cpp:15:10: "
                "fatal error: petscvec.h: No such file or directory"
            ],
            1,
            MissingCHeader("petscvec.h"),
        )
        self.run_test(
            [
                "src/bubble.h:27:10: fatal error: DBlurEffectWidget: "
                "No such file or directory"
            ],
            1,
            MissingCHeader("DBlurEffectWidget"),
        )

    def test_missing_jdk_file(self):
        self.run_test(
            [
                "> Could not find tools.jar. Please check that "
                "/usr/lib/jvm/java-8-openjdk-amd64 contains a "
                "valid JDK installation.",
            ],
            1,
            MissingJDKFile("/usr/lib/jvm/java-8-openjdk-amd64", "tools.jar"),
        )

    def test_missing_jdk(self):
        self.run_test(
            [
                "> Kotlin could not find the required JDK tools in "
                "the Java installation "
                "'/usr/lib/jvm/java-8-openjdk-amd64/jre' used by Gradle. "
                "Make sure Gradle is running on a JDK, not JRE.",
            ],
            1,
            MissingJDK("/usr/lib/jvm/java-8-openjdk-amd64/jre"),
        )

    def test_missing_jre(self):
        self.run_test(
            [
                "ERROR: JAVA_HOME is not set and no 'java' command "
                "could be found in your PATH."
            ],
            1,
            MissingJRE(),
        )

    def test_node_module_missing(self):
        self.run_test(
            ["Error: Cannot find module 'tape'"], 1, MissingNodeModule("tape")
        )
        self.run_test(
            [
                "✖ [31mERROR:[39m Cannot find module '/<<PKGBUILDDIR>>/test'",
            ],
            1,
            None,
        )
        self.run_test(
            ["npm ERR! [!] Error: Cannot find module '@rollup/plugin-buble'"],
            1,
            MissingNodeModule("@rollup/plugin-buble"),
        )
        self.run_test(
            ["npm ERR! Error: Cannot find module 'fs-extra'"],
            1,
            MissingNodeModule("fs-extra"),
        )
        self.run_test(
            ["\x1b[1m\x1b[31m[!] \x1b[1mError: Cannot find module '@rollup/plugin-buble'"],
            1, MissingNodeModule('@rollup/plugin-buble'))

    def test_setup_py_command(self):
        self.run_test(
            """\
/usr/lib/python3.9/distutils/dist.py:274: UserWarning: Unknown distribution option: 'long_description_content_type'
  warnings.warn(msg)
/usr/lib/python3.9/distutils/dist.py:274: UserWarning: Unknown distribution option: 'test_suite'
  warnings.warn(msg)
/usr/lib/python3.9/distutils/dist.py:274: UserWarning: Unknown distribution option: 'python_requires'
  warnings.warn(msg)
usage: setup.py [global_opts] cmd1 [cmd1_opts] [cmd2 [cmd2_opts] ...]
   or: setup.py --help [cmd1 cmd2 ...]
   or: setup.py --help-commands
   or: setup.py cmd --help

error: invalid command 'test'
""".splitlines(
                True
            ),
            12,
            MissingSetupPyCommand("test"),
        )

    def test_command_missing(self):
        self.run_test(
            ["./ylwrap: line 176: yacc: command not found"], 1, MissingCommand("yacc")
        )
        self.run_test(["/bin/sh: 1: cmake: not found"], 1, MissingCommand("cmake"))
        self.run_test(["sh: 1: git: not found"], 1, MissingCommand("git"))
        self.run_test(
            ["/usr/bin/env: ‘python3’: No such file or directory"],
            1,
            MissingCommand("python3"),
        )
        self.run_test(
            ["%Error: 'flex' must be installed to build"], 1, MissingCommand("flex")
        )
        self.run_test(
            ['pkg-config: exec: "pkg-config": executable file not found in $PATH'],
            1,
            MissingCommand("pkg-config"),
        )
        self.run_test(
            ['Can\'t exec "git": No such file or directory at ' "Makefile.PL line 25."],
            1,
            MissingCommand("git"),
        )
        self.run_test(
            [
                "vcver.scm.git.GitCommandError: 'git describe --tags --match 'v*'"
                " --abbrev=0' returned an error code 127"
            ],
            1,
            MissingCommand("git"),
        )
        self.run_test(
            ["make[1]: docker: Command not found"], 1, MissingCommand("docker")
        )
        self.run_test(["make[1]: git: Command not found"], 1, MissingCommand("git"))
        self.run_test(["make[1]: ./docker: Command not found"], 1, None)
        self.run_test(
            ["make: dh_elpa: Command not found"], 1, MissingCommand("dh_elpa")
        )
        self.run_test(
            ["/bin/bash: valac: command not found"], 1, MissingCommand("valac")
        )
        self.run_test(
            ["E: Failed to execute “python3”: No such file or directory"],
            1,
            MissingCommand("python3"),
        )
        self.run_test(
            [
                'Can\'t exec "cmake": No such file or directory at '
                "/usr/share/perl5/Debian/Debhelper/Dh_Lib.pm line 484."
            ],
            1,
            MissingCommand("cmake"),
        )
        self.run_test(
            [
                "Invalid gemspec in [unicorn.gemspec]: "
                "No such file or directory - git"
            ],
            1,
            MissingCommand("git"),
        )
        self.run_test(
            [
                "dbus-run-session: failed to exec 'xvfb-run': "
                "No such file or directory"
            ],
            1,
            MissingCommand("xvfb-run"),
        )
        self.run_test(["/bin/sh: 1: ./configure: not found"], 1, MissingConfigure())
        self.run_test(
            ["xvfb-run: error: xauth command not found"], 1, MissingCommand("xauth")
        )
        self.run_test(
            [
                "meson.build:39:2: ERROR: Program(s) ['wrc'] "
                "not found or not executable"
            ],
            1,
            MissingCommand("wrc"),
        )
        self.run_test(
            [
                "/tmp/autopkgtest.FnbV06/build.18W/src/debian/tests/"
                "blas-testsuite: 7: dpkg-architecture: not found"
            ],
            1,
            MissingCommand("dpkg-architecture"),
        )
        self.run_test(
            [
                "Traceback (most recent call last):",
                '  File "/usr/lib/python3/dist-packages/mesonbuild/mesonmain.py", line 140, in run',
                "    return options.run_func(options)",
                '  File "/usr/lib/python3/dist-packages/mesonbuild/mdist.py", line 267, in run',
                "    names = create_dist_git(dist_name, archives, src_root, bld_root, dist_sub, b.dist_scripts, subprojects)",
                '  File "/usr/lib/python3/dist-packages/mesonbuild/mdist.py", line 119, in create_dist_git',
                "    git_clone(src_root, distdir)",
                '  File "/usr/lib/python3/dist-packages/mesonbuild/mdist.py", line 108, in git_clone',
                "    if git_have_dirty_index(src_root):",
                '  File "/usr/lib/python3/dist-packages/mesonbuild/mdist.py", line 104, in git_have_dirty_index',
                "    ret = subprocess.call(['git', '-C', src_root, 'diff-index', '--quiet', 'HEAD'])",
                '  File "/usr/lib/python3.9/subprocess.py", line 349, in call',
                "    with Popen(*popenargs, **kwargs) as p:",
                '  File "/usr/lib/python3.9/subprocess.py", line 951, in __init__',
                "    self._execute_child(args, executable, preexec_fn, close_fds,",
                '  File "/usr/lib/python3.9/subprocess.py", line 1823, in _execute_child',
                "    raise child_exception_type(errno_num, err_msg, err_filename)",
                "FileNotFoundError: [Errno 2] No such file or directory: 'git'",
            ],
            18,
            MissingCommand("git"),
        )
        self.run_test(
            ['> Cannot run program "git": error=2, No such file or directory'],
            1,
            MissingCommand("git"),
        )
        self.run_test(
            ["E ImportError: Bad git executable"], 1,
            MissingCommand("git"))
        self.run_test(
            ["E ImportError: Bad git executable."], 1,
            MissingCommand("git"))

    def test_ts_error(self):
        self.run_test(
            [
                "blah/tokenizer.ts(175,21): error TS2532: "
                "Object is possibly 'undefined'."
            ],
            1,
            None,
        )

    def test_nim_error(self):
        self.run_test(
            [
                "/<<PKGBUILDDIR>>/msgpack4nim.nim(470, 6) "
                "Error: usage of 'isNil' is a user-defined error"
            ],
            1,
            None,
        )

    def test_scala_error(self):
        self.run_test(
            [
                "core/src/main/scala/org/json4s/JsonFormat.scala:131: "
                "error: No JSON deserializer found for type List[T]. "
                "Try to implement an implicit Reader or JsonFormat for this type."
            ],
            1,
            None,
        )

    def test_vala_error(self):
        self.run_test(
            [
                "../src/Backend/FeedServer.vala:60.98-60.148: error: "
                "The name `COLLECTION_CREATE_NONE' does not exist in "
                "the context of `Secret.CollectionCreateFlags'"
            ],
            1,
            None,
        )
        self.run_test(
            [
                "error: Package `glib-2.0' not found in specified Vala "
                "API directories or GObject-Introspection GIR directories"
            ],
            1,
            MissingValaPackage("glib-2.0"),
        )

    def test_gir(self):
        self.run_test(
            ["ValueError: Namespace GnomeDesktop not available"],
            1,
            MissingIntrospectionTypelib("GnomeDesktop"))

    def test_missing_boost_components(self):
        self.run_test("""\
CMake Error at /usr/share/cmake-3.18/Modules/FindPackageHandleStandardArgs.cmake:165 (message):
  Could NOT find Boost (missing: program_options filesystem system graph
  serialization iostreams) (found suitable version "1.74.0", minimum required
  is "1.55.0")
Call Stack (most recent call first):
  /usr/share/cmake-3.18/Modules/FindPackageHandleStandardArgs.cmake:458 (_FPHSA_FAILURE_MESSAGE)
  /usr/share/cmake-3.18/Modules/FindBoost.cmake:2177 (find_package_handle_standard_args)
  src/CMakeLists.txt:4 (find_package)
""".splitlines(True), 4, MissingCMakeComponents("Boost", [
            'program_options', 'filesystem', 'system', 'graph', 'serialization', 'iostreams']))

    def test_pkg_config_too_old(self):
        self.run_test(
            [
                "checking for pkg-config... no",
                "",
                "*** Your version of pkg-config is too old. You need atleast",
                "*** pkg-config 0.9.0 or newer. You can download pkg-config",
                "*** from the freedesktop.org software repository at",
                "***",
                "***    https://www.freedesktop.org/wiki/Software/pkg-config/",
                "***"
            ], 4, MissingVagueDependency("pkg-config", minimum_version="0.9.0"))

    def test_pkg_config_missing(self):
        self.run_test(
            [
                "configure: error: Package requirements "
                "(apertium-3.2 >= 3.2.0) were not met:"
            ],
            1,
            MissingPkgConfig("apertium-3.2", "3.2.0"),
        )
        self.run_test(
            [
                'meson.build:10:0: ERROR: Dependency "gssdp-1.2" not '
                "found, tried pkgconfig"
            ],
            1,
            MissingPkgConfig("gssdp-1.2"),
        )
        self.run_test(
            [
                "src/plugins/sysprof/meson.build:3:0: "
                'ERROR: Dependency "sysprof-3" not found, tried pkgconfig'
            ],
            1,
            MissingPkgConfig("sysprof-3"),
        )
        self.run_test(
            [
                "meson.build:84:0: ERROR: Invalid version of dependency, "
                "need 'libpeas-1.0' ['>= 1.24.0'] found '1.22.0'."
            ],
            1,
            MissingPkgConfig("libpeas-1.0", "1.24.0"),
        )
        self.run_test(
            [
                "meson.build:233:0: ERROR: Invalid version of dependency, need 'vte-2.91' ['>=0.63.0'] found '0.62.3'."
            ],
            1,
            MissingPkgConfig("vte-2.91", "0.63.0"),
        )

        self.run_test(["No package 'tepl-3' found"], 1, MissingPkgConfig("tepl-3"))
        self.run_test(
            ["Requested 'vte-2.91 >= 0.59.0' but version of vte is 0.58.2"],
            1,
            MissingPkgConfig("vte-2.91", "0.59.0"),
        )
        self.run_test(
            ["configure: error: x86_64-linux-gnu-pkg-config sdl2 couldn't " "be found"],
            1,
            MissingPkgConfig("sdl2"),
        )
        self.run_test(
            ["configure: error: No package 'libcrypto' found"],
            1,
            MissingPkgConfig("libcrypto"),
        )
        self.run_test(
            [
                "-- Checking for module 'gtk+-3.0'",
                "--   Package 'gtk+-3.0', required by 'virtual:world', not found",
            ],
            2,
            MissingPkgConfig("gtk+-3.0"),
        )
        self.run_test(
            [
                "configure: error: libfilezilla not found: Package dependency "
                "requirement 'libfilezilla >= 0.17.1' could not be satisfied."
            ],
            1,
            MissingPkgConfig("libfilezilla", "0.17.1"),
        )

    def test_pkgconf(self):
        self.run_test(
            [
                "checking for LAPACK... "
                'configure: error: "Cannot check for existence of module lapack without pkgconf"'
            ],
            1,
            MissingCommand("pkgconf"),
        )

    def test_dh_with_order(self):
        self.run_test(
            [
                "dh: Unknown sequence --with "
                "(options should not come before the sequence)"
            ],
            1,
            DhWithOrderIncorrect(),
        )

    def test_no_disk_space(self):
        self.run_test(
            [
                "/usr/bin/install: error writing '"
                "/<<PKGBUILDDIR>>/debian/tmp/usr/lib/gcc/"
                "x86_64-linux-gnu/8/cc1objplus': No space left on device"
            ],
            1,
            NoSpaceOnDevice(),
        )

        self.run_test(
            [
                "OSError: [Errno 28] No space left on device",
            ],
            1,
            NoSpaceOnDevice(),
        )

    def test_segmentation_fault(self):
        self.run_test(
            [
                "/bin/bash: line 3:  7392 Segmentation fault      "
                'itstool -m "${mo}" ${d}/C/index.docbook ${d}/C/legal.xml'
            ],
            1,
        )

    def test_missing_perl_plugin(self):
        self.run_test(
            [
                "Required plugin bundle Dist::Zilla::PluginBundle::Git isn't "
                "installed."
            ],
            1,
            MissingPerlModule(None, "Dist::Zilla::PluginBundle::Git", None),
        )
        self.run_test(
            ["Required plugin Dist::Zilla::Plugin::PPPort isn't installed."],
            1,
            MissingPerlModule(filename=None, module="Dist::Zilla::Plugin::PPPort"),
        )

    def test_perl_expand(self):
        self.run_test(
            [">(error): Could not expand [ 'Dist::Inkt::Profile::TOBYINK'"],
            1,
            MissingPerlModule(None, module="Dist::Inkt::Profile::TOBYINK"),
        )

    def test_perl_missing_predeclared(self):
        self.run_test(
            [
                "String found where operator expected at Makefile.PL line 13, near \"author_tests 'xt'\"",
                "\t(Do you need to predeclare author_tests?)",
                "syntax error at Makefile.PL line 13, near \"author_tests 'xt'\"",
                '"strict subs" in use at Makefile.PL line 13.',
            ],
            2,
            MissingPerlPredeclared("author_tests"),
        )
        self.run_test(
            ["String found where operator expected at Makefile.PL line 8, near \"readme_from    'lib/URL/Encode.pod'\""],
            1, MissingPerlPredeclared("readme_from"))

        self.run_test(
            [
                'Bareword "use_test_base" not allowed while "strict subs" in use at Makefile.PL line 12.'
            ],
            1,
            MissingPerlPredeclared("use_test_base"),
        )

    def test_unknown_cert_authority(self):
        self.run_test(
            [
                "go: github.com/golangci/golangci-lint@v1.24.0: Get "
                '"https://proxy.golang.org/github.com/golangci/'
                'golangci-lint/@v/v1.24.0.mod": x509: '
                "certificate signed by unknown authority"
            ],
            1,
            UnknownCertificateAuthority(
                "https://proxy.golang.org/github.com/golangci/"
                "golangci-lint/@v/v1.24.0.mod"
            ),
        )

    def test_missing_perl_module(self):
        self.run_test(
            [
                "Converting tags.ledger... Can't locate String/Interpolate.pm in "
                "@INC (you may need to install the String::Interpolate module) "
                "(@INC contains: /etc/perl /usr/local/lib/x86_64-linux-gnu/perl/"
                "5.28.1 /usr/local/share/perl/5.28.1 /usr/lib/x86_64-linux-gnu/"
                "perl5/5.28 /usr/share/perl5 /usr/lib/x86_64-linux-gnu/perl/5.28 "
                "/usr/share/perl/5.28 /usr/local/lib/site_perl "
                "/usr/lib/x86_64-linux-gnu/perl-base) at "
                "../bin/ledger2beancount line 23."
            ],
            1,
            MissingPerlModule(
                "String/Interpolate.pm",
                "String::Interpolate",
                [
                    "/etc/perl",
                    "/usr/local/lib/x86_64-linux-gnu/perl/5.28.1",
                    "/usr/local/share/perl/5.28.1",
                    "/usr/lib/x86_64-linux-gnu/perl5/5.28",
                    "/usr/share/perl5",
                    "/usr/lib/x86_64-linux-gnu/perl/5.28",
                    "/usr/share/perl/5.28",
                    "/usr/local/lib/site_perl",
                    "/usr/lib/x86_64-linux-gnu/perl-base",
                ],
            ),
        )
        self.run_test(
            [
                "Can't locate Test/Needs.pm in @INC "
                "(you may need to install the Test::Needs module) "
                "(@INC contains: t/lib /<<PKGBUILDDIR>>/blib/lib "
                "/<<PKGBUILDDIR>>/blib/arch /etc/perl "
                "/usr/local/lib/x86_64-linux-gnu/perl/5.30.0 "
                "/usr/local/share/perl/5.30.0 /usr/lib/x86_64-linux-gnu/perl5/5.30"
                " /usr/share/perl5 /usr/lib/x86_64-linux-gnu/perl/5.30 "
                "/usr/share/perl/5.30 /usr/local/lib/site_perl "
                "/usr/lib/x86_64-linux-gnu/perl-base .) at "
                "t/anon-basic.t line 7."
            ],
            1,
            MissingPerlModule(
                "Test/Needs.pm",
                "Test::Needs",
                [
                    "t/lib",
                    "/<<PKGBUILDDIR>>/blib/lib",
                    "/<<PKGBUILDDIR>>/blib/arch",
                    "/etc/perl",
                    "/usr/local/lib/x86_64-linux-gnu/perl/5.30.0",
                    "/usr/local/share/perl/5.30.0",
                    "/usr/lib/x86_64-linux-gnu/perl5/5.30",
                    "/usr/share/perl5",
                    "/usr/lib/x86_64-linux-gnu/perl/5.30",
                    "/usr/share/perl/5.30",
                    "/usr/local/lib/site_perl",
                    "/usr/lib/x86_64-linux-gnu/perl-base",
                    ".",
                ],
            ),
        )
        self.run_test(
            ["- ExtUtils::Depends         ...missing. (would need 0.302)"], 1,
            MissingPerlModule(None, "ExtUtils::Depends", None, "0.302"))
        self.run_test(
            ['Can\'t locate object method "new" via package "Dist::Inkt::Profile::TOBYINK" '
             '(perhaps you forgot to load "Dist::Inkt::Profile::TOBYINK"?) at '
             '/usr/share/perl5/Dist/Inkt.pm line 208.'], 1,
            MissingPerlModule(None, "Dist::Inkt::Profile::TOBYINK", None))
        self.run_test(
            ["Can't locate ExtUtils/Depends.pm in @INC (you may need to "
             "install the ExtUtils::Depends module) (@INC contains: "
             "/etc/perl /usr/local/lib/x86_64-linux-gnu/perl/5.32.1 "
             "/usr/local/share/perl/5.32.1 /usr/lib/x86_64-linux-gnu/perl5/5.32 "
             "/usr/share/perl5 /usr/lib/x86_64-linux-gnu/perl-base "
             "/usr/lib/x86_64-linux-gnu/perl/5.32 "
             "/usr/share/perl/5.32 /usr/local/lib/site_perl) at "
             "(eval 11) line 1."], 1, MissingPerlModule(
                 "ExtUtils/Depends.pm", "ExtUtils::Depends", [
                     "/etc/perl",
                     "/usr/local/lib/x86_64-linux-gnu/perl/5.32.1",
                     "/usr/local/share/perl/5.32.1",
                     "/usr/lib/x86_64-linux-gnu/perl5/5.32",
                     "/usr/share/perl5", "/usr/lib/x86_64-linux-gnu/perl-base",
                     "/usr/lib/x86_64-linux-gnu/perl/5.32",
                     "/usr/share/perl/5.32", "/usr/local/lib/site_perl"]))
        self.run_test(
            ["Pod::Weaver::Plugin::WikiDoc (for section -WikiDoc) "
             "does not appear to be installed"], 1,
            MissingPerlModule(None, "Pod::Weaver::Plugin::WikiDoc"))
        self.run_test(
            ["List::Util version 1.56 required--this is only version 1.55 "
             "at /build/tmpttq5hhpt/package/blib/lib/List/AllUtils.pm line 8."],
            1, MissingPerlModule(None, "List::Util", minimum_version="1.56"))

    def test_missing_perl_file(self):
        self.run_test(
            [
                "Can't locate debian/perldl.conf in @INC (@INC contains: "
                "/<<PKGBUILDDIR>>/inc /etc/perl /usr/local/lib/x86_64-linux-gnu"
                "/perl/5.28.1 /usr/local/share/perl/5.28.1 /usr/lib/"
                "x86_64-linux-gnu/perl5/5.28 /usr/share/perl5 "
                "/usr/lib/x86_64-linux-gnu/perl/5.28 /usr/share/perl/5.28 "
                "/usr/local/lib/site_perl /usr/lib/x86_64-linux-gnu/perl-base) "
                "at Makefile.PL line 131."
            ],
            1,
            MissingPerlFile(
                "debian/perldl.conf",
                [
                    "/<<PKGBUILDDIR>>/inc",
                    "/etc/perl",
                    "/usr/local/lib/x86_64-linux-gnu/perl/5.28.1",
                    "/usr/local/share/perl/5.28.1",
                    "/usr/lib/x86_64-linux-gnu/perl5/5.28",
                    "/usr/share/perl5",
                    "/usr/lib/x86_64-linux-gnu/perl/5.28",
                    "/usr/share/perl/5.28",
                    "/usr/local/lib/site_perl",
                    "/usr/lib/x86_64-linux-gnu/perl-base",
                ],
            ),
        )
        self.run_test(
            ['Can\'t open perl script "Makefile.PL": No such file or directory'],
            1,
            MissingPerlFile("Makefile.PL"),
        )

    def test_missing_maven_artifacts(self):
        self.run_test(
            [
                "[ERROR] Failed to execute goal on project byteman-bmunit5: Could "
                "not resolve dependencies for project "
                "org.jboss.byteman:byteman-bmunit5:jar:4.0.7: The following "
                "artifacts could not be resolved: "
                "org.junit.jupiter:junit-jupiter-api:jar:5.4.0, "
                "org.junit.jupiter:junit-jupiter-params:jar:5.4.0, "
                "org.junit.jupiter:junit-jupiter-engine:jar:5.4.0: "
                "Cannot access central (https://repo.maven.apache.org/maven2) "
                "in offline mode and the artifact "
                "org.junit.jupiter:junit-jupiter-api:jar:5.4.0 has not been "
                "downloaded from it before. -> [Help 1]"
            ],
            1,
            MissingMavenArtifacts(
                [
                    "org.junit.jupiter:junit-jupiter-api:jar:5.4.0",
                    "org.junit.jupiter:junit-jupiter-params:jar:5.4.0",
                    "org.junit.jupiter:junit-jupiter-engine:jar:5.4.0",
                ]
            ),
        )
        self.run_test(
            [
                "[ERROR] Failed to execute goal on project opennlp-uima: Could "
                "not resolve dependencies for project "
                "org.apache.opennlp:opennlp-uima:jar:1.9.2-SNAPSHOT: Cannot "
                "access ApacheIncubatorRepository "
                "(http://people.apache.org/repo/m2-incubating-repository/) in "
                "offline mode and the artifact "
                "org.apache.opennlp:opennlp-tools:jar:debian has not been "
                "downloaded from it before. -> [Help 1]"
            ],
            1,
            MissingMavenArtifacts(["org.apache.opennlp:opennlp-tools:jar:debian"]),
        )
        self.run_test(
            [
                "[ERROR] Failed to execute goal on project bookkeeper-server: "
                "Could not resolve dependencies for project "
                "org.apache.bookkeeper:bookkeeper-server:jar:4.4.0: Cannot "
                "access central (https://repo.maven.apache.org/maven2) in "
                "offline mode and the artifact io.netty:netty:jar:debian "
                "has not been downloaded from it before. -> [Help 1]"
            ],
            1,
            MissingMavenArtifacts(["io.netty:netty:jar:debian"]),
        )
        self.run_test(
            [
                "[ERROR] Unresolveable build extension: Plugin "
                "org.apache.felix:maven-bundle-plugin:2.3.7 or one of its "
                "dependencies could not be resolved: Cannot access central "
                "(https://repo.maven.apache.org/maven2) in offline mode and "
                "the artifact org.apache.felix:maven-bundle-plugin:jar:2.3.7 "
                "has not been downloaded from it before. @"
            ],
            1,
            MissingMavenArtifacts(["org.apache.felix:maven-bundle-plugin:2.3.7"]),
        )
        self.run_test(
            [
                "[ERROR] Plugin org.apache.maven.plugins:maven-jar-plugin:2.6 "
                "or one of its dependencies could not be resolved: Cannot access "
                "central (https://repo.maven.apache.org/maven2) in offline mode "
                "and the artifact "
                "org.apache.maven.plugins:maven-jar-plugin:jar:2.6 has not been "
                "downloaded from it before. -> [Help 1]"
            ],
            1,
            MissingMavenArtifacts(["org.apache.maven.plugins:maven-jar-plugin:2.6"]),
        )

        self.run_test(
            [
                "[FATAL] Non-resolvable parent POM for "
                "org.joda:joda-convert:2.2.1: Cannot access central "
                "(https://repo.maven.apache.org/maven2) in offline mode "
                "and the artifact org.joda:joda-parent:pom:1.4.0 has not "
                "been downloaded from it before. and 'parent.relativePath' "
                "points at wrong local POM @ line 8, column 10"
            ],
            1,
            MissingMavenArtifacts(["org.joda:joda-parent:pom:1.4.0"]),
        )

        self.run_test(
            [
                "[ivy:retrieve] \t\t:: "
                "com.carrotsearch.randomizedtesting#junit4-ant;"
                "${/com.carrotsearch.randomizedtesting/junit4-ant}: not found"
            ],
            1,
            MissingMavenArtifacts(
                ["com.carrotsearch.randomizedtesting:junit4-ant:jar:debian"]
            ),
        )

    def test_maven_errors(self):
        self.run_test(
            [
                "[ERROR] Failed to execute goal "
                "org.apache.maven.plugins:maven-jar-plugin:3.1.2:jar "
                "(default-jar) on project xslthl: Execution default-jar of goal "
                "org.apache.maven.plugins:maven-jar-plugin:3.1.2:jar failed: "
                "An API incompatibility was encountered while executing "
                "org.apache.maven.plugins:maven-jar-plugin:3.1.2:jar: "
                "java.lang.NoSuchMethodError: "
                "'void org.codehaus.plexus.util.DirectoryScanner."
                "setFilenameComparator(java.util.Comparator)'"
            ],
            1,
            None,
        )

    def test_dh_missing_uninstalled(self):
        self.run_test(
            [
                "dh_missing --fail-missing",
                "dh_missing: usr/share/man/man1/florence_applet.1 exists in "
                "debian/tmp but is not installed to anywhere",
                "dh_missing: usr/lib/x86_64-linux-gnu/libflorence-1.0.la exists "
                "in debian/tmp but is not installed to anywhere",
                "dh_missing: missing files, aborting",
            ],
            3,
            DhMissingUninstalled("usr/lib/x86_64-linux-gnu/libflorence-1.0.la"),
        )

    def test_dh_until_unsupported(self):
        self.run_test(
            [
                "dh: The --until option is not supported any longer (#932537). "
                "Use override targets instead."
            ],
            1,
            DhUntilUnsupported(),
        )

    def test_missing_xml_entity(self):
        self.run_test(
            [
                "I/O error : Attempt to load network entity "
                "http://www.oasis-open.org/docbook/xml/4.5/docbookx.dtd"
            ],
            1,
            MissingXmlEntity("http://www.oasis-open.org/docbook/xml/4.5/docbookx.dtd"),
        )

    def test_ccache_error(self):
        self.run_test(
            [
                "ccache: error: Failed to create directory "
                "/sbuild-nonexistent/.ccache/tmp: Permission denied"
            ],
            1,
            CcacheError(
                "Failed to create directory "
                "/sbuild-nonexistent/.ccache/tmp: Permission denied"
            ),
        )

    def test_dh_addon_load_failure(self):
        self.run_test(
            [
                "dh: unable to load addon nodejs: "
                "Debian/Debhelper/Sequence/nodejs.pm did not return a true "
                "value at (eval 11) line 1."
            ],
            1,
            DhAddonLoadFailure("nodejs", "Debian/Debhelper/Sequence/nodejs.pm"),
        )

    def test_missing_library(self):
        self.run_test(
            ["/usr/bin/ld: cannot find -lpthreads"], 1, MissingLibrary("pthreads")
        )
        self.run_test(
            [
                "./testFortranCompiler.f:4: undefined reference to `sgemm_'",
            ],
            1,
        )
        self.run_test(
            [
                "writer.d:59: error: undefined reference to 'sam_hdr_parse_'",
            ],
            1,
        )

    def test_fpic(self):
        self.run_test(
            [
                "/usr/bin/ld: pcap-linux.o: relocation R_X86_64_PC32 against "
                "symbol `stderr@@GLIBC_2.2.5' can not be used when making a "
                "shared object; recompile with -fPIC"
            ],
            1,
            None,
        )

    def test_rspec(self):
        self.run_test(
            [
                "rspec ./spec/acceptance/cookbook_resource_spec.rb:20 "
                "# Client API operations downloading a cookbook when the "
                "cookbook of the name/version is found downloads the "
                "cookbook to the destination"
            ],
            1,
            None,
        )

    def test_multiple_definition(self):
        self.run_test(
            [
                "./dconf-paths.c:249: multiple definition of "
                "`dconf_is_rel_dir'; client/libdconf-client.a(dconf-paths.c.o):"
                "./obj-x86_64-linux-gnu/../common/dconf-paths.c:249: "
                "first defined here"
            ],
            1,
        )
        self.run_test(
            [
                "/usr/bin/ld: ../lib/libaxe.a(stream.c.o):(.bss+0x10): "
                "multiple definition of `gsl_message_mask'; "
                "../lib/libaxe.a(error.c.o):(.bss+0x8): first defined here"
            ],
            1,
        )

    def test_missing_ruby_gem(self):
        self.run_test(
            [
                "Could not find gem 'childprocess (~> 0.5)', which is "
                "required by gem 'selenium-webdriver', in any of the sources."
            ],
            1,
            MissingRubyGem("childprocess", "0.5"),
        )
        self.run_test(
            [
                "Could not find gem 'rexml', which is required by gem "
                "'rubocop', in any of the sources."
            ],
            1,
            MissingRubyGem("rexml"),
        )
        self.run_test(
            [
                "/usr/lib/ruby/2.5.0/rubygems/dependency.rb:310:in `to_specs': "
                "Could not find 'http-parser' (~> 1.2.0) among 59 total gem(s) "
                "(Gem::MissingSpecError)"
            ],
            1,
            MissingRubyGem("http-parser", "1.2.0"),
        )
        self.run_test(
            [
                "/usr/lib/ruby/2.5.0/rubygems/dependency.rb:312:in `to_specs': "
                "Could not find 'celluloid' (~> 0.17.3) - did find: "
                "[celluloid-0.16.0] (Gem::MissingSpecVersionError)"
            ],
            1,
            MissingRubyGem("celluloid", "0.17.3"),
        )
        self.run_test(
            [
                "/usr/lib/ruby/2.5.0/rubygems/dependency.rb:312:in `to_specs': "
                "Could not find 'i18n' (~> 0.7) - did find: [i18n-1.5.3] "
                "(Gem::MissingSpecVersionError)"
            ],
            1,
            MissingRubyGem("i18n", "0.7"),
        )
        self.run_test(
            [
                "/usr/lib/ruby/2.5.0/rubygems/dependency.rb:310:in `to_specs': "
                "Could not find 'sassc' (>= 2.0.0) among 34 total gem(s) "
                "(Gem::MissingSpecError)"
            ],
            1,
            MissingRubyGem("sassc", "2.0.0"),
        )
        self.run_test(
            [
                "/usr/lib/ruby/2.7.0/bundler/resolver.rb:290:in "
                "`block in verify_gemfile_dependencies_are_found!': "
                "Could not find gem 'rake-compiler' in any of the gem sources "
                "listed in your Gemfile. (Bundler::GemNotFound)"
            ],
            1,
            MissingRubyGem("rake-compiler"),
        )
        self.run_test(
            [
                "/usr/lib/ruby/2.7.0/rubygems.rb:275:in `find_spec_for_exe': "
                "can't find gem rdoc (>= 0.a) with executable rdoc "
                "(Gem::GemNotFoundException)"
            ],
            1,
            MissingRubyGem("rdoc", "0.a"),
        )

    def test_missing_php_class(self):
        self.run_test(
            [
                "PHP Fatal error:  Uncaught Error: Class "
                "'PHPUnit_Framework_TestCase' not found in "
                "/tmp/autopkgtest.gO7h1t/build.b1p/src/Horde_Text_Diff-"
                "2.2.0/test/Horde/Text/Diff/EngineTest.php:9"
            ],
            1,
            MissingPhpClass("PHPUnit_Framework_TestCase"),
        )

    def test_missing_java_class(self):
        self.run_test(
            """\
Caused by: java.lang.ClassNotFoundException: org.codehaus.Xpp3r$Builder
\tat org.codehaus.strategy.SelfFirstStrategy.loadClass(lfFirstStrategy.java:50)
\tat org.codehaus.realm.ClassRealm.unsynchronizedLoadClass(ClassRealm.java:271)
\tat org.codehaus.realm.ClassRealm.loadClass(ClassRealm.java:247)
\tat org.codehaus.realm.ClassRealm.loadClass(ClassRealm.java:239)
\t... 46 more
""".splitlines(),
            1,
            MissingJavaClass("org.codehaus.Xpp3r$Builder"),
        )

    def test_install_docs_link(self):
        self.run_test(
            """\
dh_installdocs: --link-doc not allowed between sympow and sympow-data (one is \
arch:all and the other not)""".splitlines(),
            1,
        )

    def test_r_missing(self):
        self.run_test(
            [
                "ERROR: dependencies ‘ellipsis’, ‘pkgload’ are not available "
                "for package ‘testthat’"
            ],
            1,
            MissingRPackage("ellipsis"),
        )
        self.run_test(
            [
                "  namespace ‘DBI’ 1.0.0 is being loaded, "
                "but >= 1.0.0.9003 is required"
            ],
            1,
            MissingRPackage("DBI", "1.0.0.9003"),
        )
        self.run_test(
            [
                "  namespace ‘spatstat.utils’ 1.13-0 is already loaded, "
                "but >= 1.15.0 is required"
            ],
            1,
            MissingRPackage("spatstat.utils", "1.15.0"),
        )
        self.run_test(
            [
                "Error in library(zeligverse) : there is no package called "
                "'zeligverse'"
            ],
            1,
            MissingRPackage("zeligverse"),
        )
        self.run_test(
            ["there is no package called 'mockr'"], 1, MissingRPackage("mockr")
        )
        self.run_test(
            [
                "ERROR: dependencies 'igraph', 'matlab', 'expm', 'RcppParallel' are not available for package 'markovchain'"
            ],
            1,
            MissingRPackage("igraph"),
        )
        self.run_test(
            [
                "Error: package 'BH' 1.66.0-1 was found, but >= 1.75.0.0 is required by 'RSQLite'"
            ],
            1,
            MissingRPackage("BH", "1.75.0.0"),
        )
        self.run_test(
            [
                "Error: package ‘AnnotationDbi’ 1.52.0 was found, but >= 1.53.1 is required by ‘GO.db’"
            ], 1,
            MissingRPackage("AnnotationDbi", "1.53.1"))
        self.run_test(
            ["  namespace 'alakazam' 1.1.0 is being loaded, but >= 1.1.0.999 is required"],
            1,
            MissingRPackage('alakazam', '1.1.0.999'))

    def test_mv_stat(self):
        self.run_test(
            ["mv: cannot stat '/usr/res/boss.png': No such file or directory"],
            1,
            MissingFile("/usr/res/boss.png"),
        )
        self.run_test(["mv: cannot stat 'res/boss.png': No such file or directory"], 1)

    def test_dh_link_error(self):
        self.run_test(
            [
                "dh_link: link destination debian/r-cran-crosstalk/usr/lib/R/"
                "site-library/crosstalk/lib/ionrangeslider is a directory"
            ],
            1,
            DhLinkDestinationIsDirectory(
                "debian/r-cran-crosstalk/usr/lib/R/site-library/crosstalk/"
                "lib/ionrangeslider"
            ),
        )

    def test_go_test(self):
        self.run_test(
            ["FAIL\tgithub.com/edsrzf/mmap-go\t0.083s"],
            1,
            None,
        )

    def test_debhelper_pattern(self):
        self.run_test(
            [
                "dh_install: Cannot find (any matches for) "
                '"server/etc/gnumed/gnumed-restore.conf" '
                "(tried in ., debian/tmp)"
            ],
            1,
            DebhelperPatternNotFound(
                "server/etc/gnumed/gnumed-restore.conf", "install", [".", "debian/tmp"]
            ),
        )

    def test_symbols(self):
        self.run_test(
            [
                "dpkg-gensymbols: error: some symbols or patterns disappeared in "
                "the symbols file: see diff output below"
            ],
            1,
            DisappearedSymbols(),
        )

    def test_autoconf_macro(self):
        self.run_test(
            ["configure.in:1802: error: possibly undefined macro: " "AC_CHECK_CCA"],
            1,
            MissingAutoconfMacro("AC_CHECK_CCA"),
        )
        self.run_test(
            ["./configure: line 12569: PKG_PROG_PKG_CONFIG: command not found"],
            1,
            MissingAutoconfMacro("PKG_PROG_PKG_CONFIG"),
        )
        self.run_test(
            [
                "checking for gawk... (cached) mawk",
                "./configure: line 2368: syntax error near unexpected token `APERTIUM,'",
                "./configure: line 2368: `PKG_CHECK_MODULES(APERTIUM, apertium >= 3.7.1)'",
            ],
            3,
            MissingAutoconfMacro("PKG_CHECK_MODULES", need_rebuild=True),
        )
        self.run_test(
            [
                "checking for libexif to use... ./configure: line 15968: syntax error near unexpected token `LIBEXIF,libexif'",
                "./configure: line 15968: `\t\t\t\t\t\tPKG_CHECK_MODULES(LIBEXIF,libexif >= 0.6.18,have_LIBEXIF=yes,:)'",
            ],
            2,
            MissingAutoconfMacro("PKG_CHECK_MODULES", need_rebuild=True))

    def test_autoconf_version(self):
        self.run_test(
            ["configure.ac:13: error: Autoconf version 2.71 or higher is required"], 1,
            MissingVagueDependency("autoconf", minimum_version="2.71"))

    def test_claws_version(self):
        self.run_test(
            ["configure: error: libetpan 0.57 not found"], 1,
            MissingVagueDependency(
                'libetpan', minimum_version='0.57'))

    def test_config_status_input(self):
        self.run_test(
            ["config.status: error: cannot find input file: " "`po/Makefile.in.in'"],
            1,
            MissingConfigStatusInput("po/Makefile.in.in"),
        )

    def test_jvm(self):
        self.run_test(
            [
                "ERROR: JAVA_HOME is set to an invalid "
                "directory: /usr/lib/jvm/default-java/"
            ],
            1,
            MissingJVM(),
        )

    def test_cp(self):
        self.run_test(
            [
                "cp: cannot stat "
                "'/<<PKGBUILDDIR>>/debian/patches/lshw-gtk.desktop': "
                "No such file or directory"
            ],
            1,
            MissingBuildFile("debian/patches/lshw-gtk.desktop"),
        )

    def test_bash_redir_missing(self):
        self.run_test(
            ["/bin/bash: idna-tables-properties.csv: "
             "No such file or directory"],
            1, MissingBuildFile("idna-tables-properties.csv"))

    def test_automake_input(self):
        self.run_test(
            [
                "automake: error: cannot open < gtk-doc.make: "
                "No such file or directory"
            ],
            1,
            MissingAutomakeInput("gtk-doc.make"),
        )

    def test_shellcheck(self):
        self.run_test(
            [
                " " * 40 + "^----^ SC2086: "
                "Double quote to prevent globbing and word splitting."
            ],
            1,
            None,
        )
