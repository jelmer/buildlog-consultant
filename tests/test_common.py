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

import unittest

from buildlog_consultant.common import (
    CMakeFilesMissing,
    CMakeNeedExactVersion,
    MissingCHeader,
    MissingCMakeComponents,
    MissingFile,
    MissingGoPackage,
    MissingIntrospectionTypelib,
    MissingJDK,
    MissingJDKFile,
    MissingJRE,
    MissingLatexFile,
    MissingNodeModule,
    MissingPythonDistribution,
    MissingPythonModule,
    MissingSetupPyCommand,
    MissingVagueDependency,
    MissingValaPackage,
    UnsupportedPytestArguments,
    UnsupportedPytestConfigOption,
    find_build_failure_description,
    find_secondary_build_failure,
)


class FindBuildFailureDescriptionTests(unittest.TestCase):
    def run_test(self, lines, lineno, err=None):
        self.maxDiff = None
        (match, actual_err) = find_build_failure_description(lines)
        if match is not None:
            self.assertEqual(match.line, lines[lineno - 1])
            self.assertEqual(lineno, match.lineno)
        else:
            self.assertIsNone(match)
        if err:
            assert match, f"err ({err}) provided but match missing"
            self.assertEqual(actual_err, err, f"{actual_err} != {err}; origin: {match.origin}")
        else:
            self.assertIs(None, actual_err)

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
""".splitlines(True),
            16,
            MissingFile("/usr/lib/x86_64-linux-gnu/libEGL.so"),
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
""".splitlines(True),
            8,
            CMakeFilesMissing(["KDEGitCommitHooks.cmake"]),
        )

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
""".splitlines(True),
            1,
            CMakeFilesMissing(["sensor_msgsConfig.cmake", "sensor_msgs-config.cmake"]),
        )
        self.run_test(
            """\
CMake Error at /usr/share/cmake-3.22/Modules/FindPackageHandleStandardArgs.cmake:230 (message):
  Could NOT find KF5 (missing: Plasma PlasmaQuick Wayland ModemManagerQt
  NetworkManagerQt) (found suitable version "5.92.0", minimum required is
  "5.86")
""".splitlines(True),
            4,
            MissingCMakeComponents(
                "KF5",
                [
                    "Plasma",
                    "PlasmaQuick",
                    "Wayland",
                    "ModemManagerQt",
                    "NetworkManagerQt",
                ],
            ),
        )

    def test_cmake_missing_exact_version(self):
        self.run_test(
            """\
CMake Error at /usr/share/cmake-3.18/Modules/FindPackageHandleStandardArgs.cmake:165 (message):
  Could NOT find SignalProtocol: Found unsuitable version "2.3.3", but
  required is exact version "2.3.2" (found
  /usr/lib/x86_64-linux-gnu/libsignal-protocol-c.so)
""".splitlines(True),
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
            ["CMake Error at CMakeLists.txt:213 (message):", "  could not find zlib"],
            2,
            MissingVagueDependency("zlib"),
        )
        self.run_test(
            """\
-- Found LibSolv_ext: /usr/lib/x86_64-linux-gnu/libsolvext.so  
-- Found LibSolv: /usr/include /usr/lib/x86_64-linux-gnu/libsolv.so;/usr/lib/x86_64-linux-gnu/libsolvext.so
-- No usable gpgme flavours found.
CMake Error at cmake/modules/FindGpgme.cmake:398 (message):
  Did not find GPGME
Call Stack (most recent call first):
  CMakeLists.txt:223 (FIND_PACKAGE)
  """.splitlines(True),
            5,
            MissingVagueDependency("GPGME"),
        )

    def test_pytest_args(self):
        self.run_test(
            [
                "pytest: error: unrecognized arguments: --cov=janitor "
                "--cov-report=html --cov-report=term-missing:skip-covered"
            ],
            1,
            UnsupportedPytestArguments(
                [
                    "--cov=janitor",
                    "--cov-report=html",
                    "--cov-report=term-missing:skip-covered",
                ]
            ),
        )

    def test_pytest_config(self):
        self.run_test(
            [
                "INTERNALERROR> pytest.PytestConfigWarning: "
                "Unknown config option: asyncio_mode"
            ],
            1,
            UnsupportedPytestConfigOption("asyncio_mode"),
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
        self.maxDiff = None
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
            [
                "\x1b[1m\x1b[31m[!] \x1b[1mError: Cannot find module '@rollup/plugin-buble'"
            ],
            1,
            MissingNodeModule("@rollup/plugin-buble"),
        )

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
""".splitlines(True),
            12,
            MissingSetupPyCommand("test"),
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
            MissingIntrospectionTypelib("GnomeDesktop"),
        )

    def test_missing_boost_components(self):
        self.run_test(
            """\
CMake Error at /usr/share/cmake-3.18/Modules/FindPackageHandleStandardArgs.cmake:165 (message):
  Could NOT find Boost (missing: program_options filesystem system graph
  serialization iostreams) (found suitable version "1.74.0", minimum required
  is "1.55.0")
Call Stack (most recent call first):
  /usr/share/cmake-3.18/Modules/FindPackageHandleStandardArgs.cmake:458 (_FPHSA_FAILURE_MESSAGE)
  /usr/share/cmake-3.18/Modules/FindBoost.cmake:2177 (find_package_handle_standard_args)
  src/CMakeLists.txt:4 (find_package)
""".splitlines(True),
            4,
            MissingCMakeComponents(
                "Boost",
                [
                    "program_options",
                    "filesystem",
                    "system",
                    "graph",
                    "serialization",
                    "iostreams",
                ],
            ),
        )

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
                "***",
            ],
            4,
            MissingVagueDependency("pkg-config", minimum_version="0.9.0"),
        )

class SecondaryErrorFinder(unittest.TestCase):
    def assertMatches(self, line):
        m = find_secondary_build_failure([line], 100)
        self.assertIsNotNone(m)

    def assertNotMatches(self, line):
        m = find_secondary_build_failure([line], 100)
        self.assertIsNone(m)

    def test_unknown_option(self):
        self.assertMatches("Unknown option --foo")
        self.assertNotMatches("Unknown option --foo, ignoring.")
