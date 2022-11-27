#!/usr/bin/python
# Copyright (C) 2022 Jelmer Vernooij <jelmer@jelmer.uk>
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

from buildlog_consultant import Problem, problem_clses


class DummyProblem(Problem, kind='dummy-problem'):

    param: str


class TestJson(unittest.TestCase):

    def test_json(self):
        self.assertEqual(DummyProblem(param="para").json(), {"param": "para"})
        ret = DummyProblem.from_json({"param": 'parameter'})
        self.assertEqual(ret.kind, "dummy-problem")
        self.assertEqual(ret.param, "parameter")

    def test_from_json(self):
        ret = problem_clses['dummy-problem'].from_json({'param': 'para'})
        self.assertIsInstance(ret, DummyProblem)
        self.assertEqual(ret.kind, "dummy-problem")
        self.assertEqual(ret.param, "para")
