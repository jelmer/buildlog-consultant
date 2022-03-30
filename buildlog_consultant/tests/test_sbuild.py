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

from ..sbuild import (
    find_brz_build_error,
    parse_brz_error,
    InconsistentSourceFormat,
    MissingDebcargoCrate,
)


class ParseBrzErrorTests(unittest.TestCase):
    def test_inconsistent_source_format(self):
        self.assertEqual(
            (
                InconsistentSourceFormat(),
                "Inconsistent source format between version and source " "format",
            ),
            parse_brz_error(
                "Inconsistency between source format and version: version "
                "is not native, format is native.",
                [],
            ),
        )


class FindBrzBuildErrorTests(unittest.TestCase):
    def test_missing_debcargo_crate(self):
        lines = [
            "Using crate name: version-check, version 0.9.2",
            "   Updating crates.io index\n",
            "\x1b[1;31mSomething failed: Couldn't find any crate "
            "matching version-check = 0.9.2\n",
            " Try `debcargo update` to update the crates.io index.\x1b[0m\n",
            "brz: ERROR: Debcargo failed to run.\n",
        ]
        err, line = find_brz_build_error(lines)
        self.assertEqual(
            line, "debcargo can't find crate version-check (version: 0.9.2)"
        )
        self.assertEqual(err, MissingDebcargoCrate("version-check", "0.9.2"))

    def test_missing_debcargo_crate2(self):
        lines = """\
Running 'sbuild -A -s -v'
Building using working tree
Building package in merge mode
Using crate name: utf8parse, version 0.10.1+git20220116.1.dfac57e
    Updating crates.io index
    Updating crates.io index
\x1b[1;31mdebcargo failed: Couldn't find any crate matching utf8parse =0.10.1
Try `debcargo update` to update the crates.io index.\x1b[0m
brz: ERROR: Debcargo failed to run.
""".splitlines(True)
        err, line = find_brz_build_error(lines)
        self.assertEqual(
            line, "debcargo can't find crate utf8parse (version: 0.10.1)"
        )
        self.assertEqual(err, MissingDebcargoCrate("utf8parse", "0.10.1"))
