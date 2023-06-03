#!/usr/bin/python
# Copyright (C) 2022 Jelmer Vernooij <jelmer@jelmer.uk>
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

import sys

print("""\
The buildlog_consultant module itself is not executable.
However, depending on the type of file you are trying to analyse,
you may want to execute one of:

  * buildlog_consultant.autopkgtest (for Debian autopkgtest logs)
  * buildlog_consultant.common (for regular build logs)
  * buildlog_consultant.sbuild (for sbuild logs)
""")
sys.exit(1)
