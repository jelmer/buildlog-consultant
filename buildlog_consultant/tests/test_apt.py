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

from ..apt import (
    AptFetchFailure,
    AptMissingReleaseFile,
    find_apt_get_failure,
)


class FindAptGetFailureDescriptionTests(unittest.TestCase):
    def run_test(self, lines, lineno, err=None):
        (match, actual_err) = find_apt_get_failure(lines)
        if lineno is not None:
            self.assertEqual(match.line, lines[lineno - 1])
            self.assertEqual(match.lineno, lineno)
        else:
            self.assertIsNone(match)
        if err:
            self.assertEqual(actual_err, err)
        else:
            self.assertIs(None, actual_err)

    def test_make_missing_rule(self):
        self.run_test(
            [
                """\
E: Failed to fetch http://janitor.debian.net/blah/Packages.xz  \
File has unexpected size (3385796 != 3385720). Mirror sync in progress? [IP]\
"""
            ],
            1,
            AptFetchFailure(
                "http://janitor.debian.net/blah/Packages.xz",
                "File has unexpected size (3385796 != 3385720). "
                "Mirror sync in progress? [IP]",
            ),
        )

    def test_missing_release_file(self):
        self.run_test(
            [
                """\
E: The repository 'https://janitor.debian.net blah/ Release' \
does not have a Release file.\
"""
            ],
            1,
            AptMissingReleaseFile("http://janitor.debian.net/ blah/ Release"),
        )

    def test_vague(self):
        self.run_test(["E: Stuff is broken"], 1, None)
