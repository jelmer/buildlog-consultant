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

from ..autopkgtest import (
    AutopkgtestTestbedFailure,
    AutopkgtestDepsUnsatisfiable,
    AutopkgtestDepChrootDisappeared,
    AutopkgtestTimedOut,
    AutopkgtestStderrFailure,
    find_autopkgtest_failure_description,
)
from ..common import MissingCommand, MissingFile


class FindAutopkgtestFailureDescriptionTests(unittest.TestCase):
    def test_empty(self):
        self.assertEqual(
            (None, None, None, None), find_autopkgtest_failure_description([])
        )

    def test_no_match(self):
        self.assertEqual(
            (1, "blalblala\n", None, None),
            find_autopkgtest_failure_description(["blalblala\n"]),
        )

    def test_unknown_error(self):
        self.assertEqual(
            (2, "python-bcolz", None, "Test python-bcolz failed: some error"),
            find_autopkgtest_failure_description(
                [
                    "autopkgtest [07:58:03]: @@@@@@@@@@@@@@@@@@@@ summary\n",
                    "python-bcolz         FAIL some error\n",
                ]
            ),
        )

    def test_timed_out(self):
        error = AutopkgtestTimedOut()
        self.assertEqual(
            (2, "unit-tests", error, "timed out"),
            find_autopkgtest_failure_description(
                [
                    "autopkgtest [07:58:03]: @@@@@@@@@@@@@@@@@@@@ summary\n",
                    "unit-tests           FAIL timed out",
                ]
            ),
        )

    def test_deps(self):
        error = AutopkgtestDepsUnsatisfiable(
            [
                (
                    "arg",
                    "/home/janitor/tmp/tmppvupofwl/build-area/"
                    "bcolz-doc_1.2.1+ds2-4~jan+lint1_all.deb",
                ),
                ("deb", "bcolz-doc"),
                (
                    "arg",
                    "/home/janitor/tmp/tmppvupofwl/build-area/python-"
                    "bcolz-dbgsym_1.2.1+ds2-4~jan+lint1_amd64.deb",
                ),
                ("deb", "python-bcolz-dbgsym"),
                (
                    "arg",
                    "/home/janitor/tmp/"
                    "tmppvupofwl/build-area/python-bcolz_1.2.1+ds2-4~jan"
                    "+lint1_amd64.deb",
                ),
                ("deb", "python-bcolz"),
                (
                    "arg",
                    "/home/janitor/tmp/tmppvupofwl/build-area/"
                    "python3-bcolz-dbgsym_1.2.1+ds2-4~jan+lint1_amd64.deb",
                ),
                ("deb", "python3-bcolz-dbgsym"),
                (
                    "arg",
                    "/home/janitor/tmp/tmppvupofwl/build-area/python3-"
                    "bcolz_1.2.1+ds2-4~jan+lint1_amd64.deb",
                ),
                ("deb", "python3-bcolz"),
                (
                    None,
                    "/home/janitor/tmp/tmppvupofwl/build-area/"
                    "bcolz_1.2.1+ds2-4~jan+lint1.dsc",
                ),
            ]
        )

        self.assertEqual(
            (
                2,
                "python-bcolz",
                error,
                "Test python-bcolz failed: Test dependencies are unsatisfiable. "
                "A common reason is that your testbed is out of date "
                "with respect to the archive, and you need to use a "
                "current testbed or run apt-get update or use -U.",
            ),
            find_autopkgtest_failure_description(
                [
                    "autopkgtest [07:58:03]: @@@@@@@@@@@@@@@@@@@@ summary\n",
                    "python-bcolz         FAIL badpkg\n",
                    "blame: arg:/home/janitor/tmp/tmppvupofwl/build-area/"
                    "bcolz-doc_1.2.1+ds2-4~jan+lint1_all.deb deb:bcolz-doc "
                    "arg:/home/janitor/tmp/tmppvupofwl/build-area/python-"
                    "bcolz-dbgsym_1.2.1+ds2-4~jan+lint1_amd64.deb "
                    "deb:python-bcolz-dbgsym arg:/home/janitor/tmp/"
                    "tmppvupofwl/build-area/python-bcolz_1.2.1+ds2-4~jan"
                    "+lint1_amd64.deb deb:python-bcolz arg:/home/janitor/"
                    "tmp/tmppvupofwl/build-area/python3-bcolz-dbgsym_1.2.1"
                    "+ds2-4~jan+lint1_amd64.deb deb:python3-bcolz-dbgsym "
                    "arg:/home/janitor/tmp/tmppvupofwl/build-area/python3-"
                    "bcolz_1.2.1+ds2-4~jan+lint1_amd64.deb deb:python3-"
                    "bcolz /home/janitor/tmp/tmppvupofwl/build-area/"
                    "bcolz_1.2.1+ds2-4~jan+lint1.dsc\n",
                    "badpkg: Test dependencies are unsatisfiable. "
                    "A common reason is that your testbed is out of date "
                    "with respect to the archive, and you need to use a "
                    "current testbed or run apt-get update or use -U.\n",
                ]
            ),
        )
        error = AutopkgtestDepsUnsatisfiable(
            [
                (
                    "arg",
                    "/home/janitor/tmp/tmpgbn5jhou/build-area/cmake"
                    "-extras_1.3+17.04.20170310-6~jan+unchanged1_all.deb",
                ),
                ("deb", "cmake-extras"),
                (
                    None,
                    "/home/janitor/tmp/tmpgbn5jhou/"
                    "build-area/cmake-extras_1.3+17.04.20170310-6~jan.dsc",
                ),
            ]
        )
        self.assertEqual(
            (
                2,
                "intltool",
                error,
                "Test intltool failed: Test dependencies are unsatisfiable. "
                "A common reason is that your testbed is out of date with "
                "respect to the archive, and you need to use a current testbed "
                "or run apt-get update or use -U.",
            ),
            find_autopkgtest_failure_description(
                [
                    "autopkgtest [07:58:03]: @@@@@@@@@@@@@@@@@@@@ summary\n",
                    "intltool             FAIL badpkg",
                    "blame: arg:/home/janitor/tmp/tmpgbn5jhou/build-area/cmake"
                    "-extras_1.3+17.04.20170310-6~jan+unchanged1_all.deb "
                    "deb:cmake-extras /home/janitor/tmp/tmpgbn5jhou/"
                    "build-area/cmake-extras_1.3+17.04.20170310-6~jan.dsc",
                    "badpkg: Test dependencies are unsatisfiable. A common "
                    "reason is that your testbed is out of date with respect "
                    "to the archive, and you need to use a current testbed or "
                    "run apt-get update or use -U.",
                ]
            ),
        )

    def test_session_disappeared(self):
        error = AutopkgtestDepChrootDisappeared()
        self.assertEqual(
            (4, None, error, "<VirtSubproc>: failure: ['chmod', '1777', '/tmp/autopkgtest.JLqPpH'] unexpectedly produced stderr output `W: /var/lib/schroot/session/unstable-amd64-sbuild-dbcdb3f2-53ed-4f84-8f0d-2c53ebe71010: Failed to stat file: No such file or directory"),
            find_autopkgtest_failure_description("""\
autopkgtest [22:52:18]: starting date: 2021-04-01
autopkgtest [22:52:18]: version 5.16
autopkgtest [22:52:18]: host osuosl167-amd64; command line: /usr/bin/autopkgtest '/tmp/tmpb0o8ai2j/build-area/liquid-dsp_1.2.0+git20210131.9ae84d8-1~jan+deb1_amd64.changes' --no-auto-control -- schroot unstable-amd64-sbuild
<VirtSubproc>: failure: ['chmod', '1777', '/tmp/autopkgtest.JLqPpH'] unexpectedly produced stderr output `W: /var/lib/schroot/session/unstable-amd64-sbuild-dbcdb3f2-53ed-4f84-8f0d-2c53ebe71010: Failed to stat file: No such file or directory
'
autopkgtest [22:52:19]: ERROR: testbed failure: cannot send to testbed: [Errno 32] Broken pipe
""".splitlines(False)))

    def test_stderr(self):
        error = AutopkgtestStderrFailure("some output")
        self.assertEqual(
            (
                6,
                "intltool",
                error,
                "Test intltool failed due to unauthorized stderr output: "
                "some output",
            ),
            find_autopkgtest_failure_description(
                [
                    "intltool            FAIL stderr: some output",
                    "autopkgtest [20:49:00]: test intltool:"
                    "  - - - - - - - - - - stderr - - - - - - - - - -",
                    "some output",
                    "some more output",
                    "autopkgtest [20:49:00]: @@@@@@@@@@@@@@@@@@@@ summary",
                    "intltool            FAIL stderr: some output",
                ]
            ),
        )
        self.assertEqual(
            (2, "intltool", MissingCommand("ss"), "/tmp/bla: 12: ss: not found"),
            find_autopkgtest_failure_description(
                [
                    "autopkgtest [20:49:00]: test intltool:"
                    "  - - - - - - - - - - stderr - - - - - - - - - -",
                    "/tmp/bla: 12: ss: not found",
                    "some more output",
                    "autopkgtest [20:49:00]: @@@@@@@@@@@@@@@@@@@@ summary",
                    "intltool            FAIL stderr: /tmp/bla: 12: ss: not found",
                ]
            ),
        )
        self.assertEqual(
            (
                2,
                "command10",
                MissingCommand("uptime"),
                'Can\'t exec "uptime": No such file or directory at '
                "/usr/lib/nagios/plugins/check_uptime line 529.",
            ),
            find_autopkgtest_failure_description(
                [
                    "autopkgtest [07:58:03]: @@@@@@@@@@@@@@@@@@@@ summary\n",
                    'command10            FAIL stderr: Can\'t exec "uptime": '
                    "No such file or directory at "
                    "/usr/lib/nagios/plugins/check_uptime line 529.",
                ]
            ),
        )

    def test_testbed_failure(self):
        error = AutopkgtestTestbedFailure(
            "sent `copyup /tmp/autopkgtest.9IStGJ/build.0Pm/src/ "
            "/tmp/autopkgtest.output.icg0g8e6/tests-tree/', got "
            "`timeout', expected `ok...'"
        )
        self.assertEqual(
            (1, None, error, None),
            find_autopkgtest_failure_description(
                [
                    "autopkgtest [12:46:18]: ERROR: testbed failure: sent "
                    "`copyup /tmp/autopkgtest.9IStGJ/build.0Pm/src/ "
                    "/tmp/autopkgtest.output.icg0g8e6/tests-tree/', got "
                    "`timeout', expected `ok...'\n"
                ]
            ),
        )

    def test_testbed_failure_with_test(self):
        error = AutopkgtestTestbedFailure("testbed auxverb failed with exit code 255")
        self.assertEqual(
            (4, "phpunit", error, None),
            find_autopkgtest_failure_description(
                """\
Removing autopkgtest-satdep (0) ...
autopkgtest [06:59:00]: test phpunit: [-----------------------
PHP Fatal error:  Declaration of Wicked_TestCase::setUp() must \
be compatible with PHPUnit\\Framework\\TestCase::setUp(): void in \
/tmp/autopkgtest.5ShOBp/build.ViG/src/wicked-2.0.8/test/Wicked/\
TestCase.php on line 31
autopkgtest [06:59:01]: ERROR: testbed failure: testbed auxverb \
failed with exit code 255
Exiting with 16
""".splitlines(
                    True
                )
            ),
        )

    def test_test_command_failure(self):
        self.assertEqual(
            (
                7,
                "command2",
                MissingFile("/usr/share/php/Pimple/autoload.php"),
                'Cannot open file "/usr/share/php/Pimple/autoload.php".\n',
            ),
            find_autopkgtest_failure_description(
                """\
Removing autopkgtest-satdep (0) ...
autopkgtest [01:30:11]: test command2: phpunit --bootstrap /usr/autoload.php
autopkgtest [01:30:11]: test command2: [-----------------------
PHPUnit 8.5.2 by Sebastian Bergmann and contributors.

Cannot open file "/usr/share/php/Pimple/autoload.php".

autopkgtest [01:30:12]: test command2: -----------------------]
autopkgtest [01:30:12]: test command2:  \
- - - - - - - - - - results - - - - - - - - - -
command2             FAIL non-zero exit status 1
autopkgtest [01:30:12]: @@@@@@@@@@@@@@@@@@@@ summary
command1             PASS
command2             FAIL non-zero exit status 1
Exiting with 4
""".splitlines(
                    True
                )
            ),
        )

    def test_dpkg_failure(self):
        self.assertEqual(
            (
                8,
                "runtestsuite",
                AutopkgtestDepChrootDisappeared(),
                """\
W: /var/lib/schroot/session/unstable-amd64-\
sbuild-7fb1b836-14f9-4709-8584-cbbae284db97: \
Failed to stat file: No such file or directory""",
            ),
            find_autopkgtest_failure_description(
                """\
autopkgtest [19:19:19]: test require: [-----------------------
autopkgtest [19:19:20]: test require: -----------------------]
autopkgtest [19:19:20]: test require:  \
- - - - - - - - - - results - - - - - - - - - -
require              PASS
autopkgtest [19:19:20]: test runtestsuite: preparing testbed
Get:1 file:/tmp/autopkgtest.hdIETy/binaries  InRelease
Ign:1 file:/tmp/autopkgtest.hdIETy/binaries  InRelease
autopkgtest [19:19:23]: ERROR: "dpkg --unpack \
/tmp/autopkgtest.hdIETy/4-autopkgtest-satdep.deb" failed with \
stderr "W: /var/lib/schroot/session/unstable-amd64-sbuild-\
7fb1b836-14f9-4709-8584-cbbae284db97: Failed to stat file: \
No such file or directory
""".splitlines(
                    True
                )
            ),
        )

    def test_last_stderr_line(self):
        self.assertEqual(
            (11, "unmunge", None, "Test unmunge failed: non-zero exit status 2"),
            find_autopkgtest_failure_description(
                """\
autopkgtest [17:38:49]: test unmunge: [-----------------------
munge: Error: Failed to access "/run/munge/munge.socket.2": \
No such file or directory
unmunge: Error: No credential specified
autopkgtest [17:38:50]: test unmunge: -----------------------]
autopkgtest [17:38:50]: test unmunge: \
 - - - - - - - - - - results - - - - - - - - - -
unmunge              FAIL non-zero exit status 2
autopkgtest [17:38:50]: test unmunge: \
 - - - - - - - - - - stderr - - - - - - - - - -
munge: Error: Failed to access "/run/munge/munge.socket.2": \
No such file or directory
unmunge: Error: No credential specified
autopkgtest [17:38:50]: @@@@@@@@@@@@@@@@@@@@ summary
unmunge              FAIL non-zero exit status 2
Exiting with 4
""".splitlines(
                    True
                )
            ),
        )

    def test_python_error_in_output(self):
        self.assertEqual(
            (
                7,
                "unit-tests-3",
                None,
                "builtins.OverflowError: mktime argument out of range\n",
            ),
            find_autopkgtest_failure_description(
                """\
autopkgtest [14:55:35]: test unit-tests-3: [-----------------------
  File "twisted/test/test_log.py", line 511, in test_getTimezoneOffsetWithout
    self._getTimezoneOffsetTest("Africa/Johannesburg", -7200, -7200)
  File "twisted/test/test_log.py", line 460, in _getTimezoneOffsetTest
    daylight = time.mktime(localDaylightTuple)
builtins.OverflowError: mktime argument out of range
-------------------------------------------------------------------------------
Ran 12377 tests in 143.490s

143.4904797077179 12377 12377 1 0 2352
autopkgtest [14:58:01]: test unit-tests-3: -----------------------]
autopkgtest [14:58:01]: test unit-tests-3: \
 - - - - - - - - - - results - - - - - - - - - -
unit-tests-3         FAIL non-zero exit status 1
autopkgtest [14:58:01]: @@@@@@@@@@@@@@@@@@@@ summary
unit-tests-3         FAIL non-zero exit status 1
Exiting with 4
""".splitlines(
                    True
                )
            ),
        )
